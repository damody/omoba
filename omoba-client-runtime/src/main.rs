use omoba_client_runtime::{
    catchup::{ingest_pending_frame, ReplicaLagTracker},
    checkpoint_writer::{spawn_checkpoint_writer, CheckpointQueue},
    config::ClientRuntimeConfig,
    evidence::EvidenceRecorder,
    input_bridge::{InputBridge, InputDecision},
    link_liveness::{InputSettleDeadlines, LinkLiveness, LinkTransition},
    presentation_bridge::{
        assemble_rebased_presentation, communication_envelope, lifecycle_envelope, ready_envelope_for_start, reset_view_envelope,
        PresentationHub, PRESENTATION_MAGIC, PRESENTATION_PROTOCOL_VERSION,
    },
    replica_host::ReplicaHost,
    session::SelectiveSession,
    shutdown::{ShutdownReason, ShutdownToken},
};
use omoba_core::{
    game_proto::{
        renderer_ipc_envelope, ClientTeamHashMismatch, CriticalInputResult, RendererInput,
        RendererIpcEnvelope, ViewEpoch,
    },
    kcp::client::LockstepInbound,
};
use prost::Message;
use std::{collections::BTreeMap, sync::Arc};

fn input_lookahead_ticks() -> u64 {
    u64::from(omoba_core::lockstep_timing::LOCKSTEP_INPUT_LOOKAHEAD_TICKS)
}

#[cfg(target_os = "windows")]
#[link(name = "winmm")]
extern "system" {
    fn timeBeginPeriod(u_period: u32) -> u32;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        timeBeginPeriod(1);
    }
    env_logger::init();
    let config = ClientRuntimeConfig::from_env_args()?;
    log::info!(
        "client-runtime starting player_id={} team_id={} server={} presentation={} protocol={} test_mode={}",
        config.player_id,
        config.team_id,
        config.server_addr,
        config.presentation_bind,
        config.protocol_version,
        config.test_mode
    );
    let (shutdown, shutdown_rx) = ShutdownToken::new();
    let ctrl_c = shutdown.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            ctrl_c.cancel(ShutdownReason::Requested);
        }
    });
    if let Some(shutdown_file) = config.shutdown_file.clone() {
        let file_shutdown = shutdown.clone();
        tokio::spawn(async move {
            loop {
                if shutdown_file.exists() {
                    file_shutdown.cancel(ShutdownReason::Requested);
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        });
    }
    let mut session = SelectiveSession::connect(&config).await?;
    let mut ctx = RuntimeContext {
        config, presentation: None, evidence: None, presentation_sequence: 1,
        resume_token: Vec::new(), shutdown, shutdown_rx, rejoins: 0,
    };
    // One match: a lost link reconnects to the same match (rejoin) instead of exiting.
    loop {
        match run_session(&mut ctx, session).await? {
            SessionEnd::Stopped => break,
            SessionEnd::LinkLost => match reconnect(&mut ctx).await? {
                Some(next) => session = next,
                None => break,
            },
        }
    }
    Ok(())
}

/// State that outlives one server session: the renderer link, evidence, the
/// presentation sequence, the rejoin secret and the process shutdown token.
struct RuntimeContext {
    config: ClientRuntimeConfig,
    presentation: Option<PresentationHub>,
    evidence: Option<EvidenceRecorder>,
    presentation_sequence: u64,
    resume_token: Vec<u8>,
    shutdown: ShutdownToken,
    shutdown_rx: tokio::sync::watch::Receiver<Option<ShutdownReason>>,
    rejoins: u32,
}

/// How one server session ended.
enum SessionEnd {
    /// Shutdown requested, match terminated or unsafe: the process exits.
    Stopped,
    /// The authority link was lost: try to rejoin the same match.
    LinkLost,
}

/// RECONNECTING with backoff until a rejoin succeeds, the player stops the
/// runtime, or the reconnect window closes. `None` means stop.
async fn reconnect(ctx: &mut RuntimeContext) -> anyhow::Result<Option<SelectiveSession>> {
    use omoba_client_runtime::rejoin::{reconnecting_reason, RejoinBackoff, RejoinStep, ATTEMPT_TIMEOUT};
    use omoba_core::game_proto::runtime_session_state::State;
    let stop_requested = |rx: &tokio::sync::watch::Receiver<Option<ShutdownReason>>| rx.borrow().is_some();
    let mut backoff = RejoinBackoff::new(std::time::Instant::now());
    loop {
        let (attempt, after) = match backoff.next(std::time::Instant::now()) {
            RejoinStep::Retry { attempt, after } => (attempt, after),
            RejoinStep::GiveUp { attempts } => {
                log::error!("client-runtime rejoin gave up after {attempts} attempts");
                if let Some(evidence) = &ctx.evidence { evidence.record_network_event("rejoin-gave-up", 0, 0, "SERVER_DISCONNECTED")?; }
                if let Some(presentation) = ctx.presentation.as_mut() {
                    ctx.presentation_sequence = ctx.presentation_sequence.saturating_add(1);
                    presentation.publish_critical(session_state_envelope(ctx.presentation_sequence, State::ServerDisconnected, "rejoin_gave_up")).await?;
                }
                return Ok(None);
            }
        };
        if !after.is_zero() {
            tokio::select! {
                _ = tokio::time::sleep(after) => {}
                _ = ctx.shutdown_rx.changed() => {}
            }
        }
        if stop_requested(&ctx.shutdown_rx) { return Ok(None); }
        // The renderer left (the player pressed 放棄 / quit): give the rejoin up.
        if ctx.presentation.as_ref().is_some_and(|presentation| presentation.renderer_left_for(5_000)) {
            log::warn!("client-runtime rejoin abandoned: the renderer left");
            if let Some(evidence) = &ctx.evidence { evidence.record_network_event("rejoin-abandoned", 0, 0, "SHUTDOWN")?; }
            return Ok(None);
        }
        if let Some(presentation) = ctx.presentation.as_mut() {
            ctx.presentation_sequence = ctx.presentation_sequence.saturating_add(1);
            presentation.publish_critical(session_state_envelope(ctx.presentation_sequence, State::Reconnecting, &reconnecting_reason(attempt))).await?;
        }
        let joined = tokio::select! {
            joined = tokio::time::timeout(ATTEMPT_TIMEOUT, SelectiveSession::connect_resuming(&ctx.config, ctx.resume_token.clone())) => Some(joined),
            _ = ctx.shutdown_rx.changed() => None,
        };
        match joined {
            None => return Ok(None),
            Some(Ok(Ok(session))) => {
                ctx.rejoins = ctx.rejoins.saturating_add(1);
                log::warn!("client-runtime rejoined player_id={} team_id={} attempt={} server_tick={} replica_tick={} rejoins={}",
                    ctx.config.player_id, session.start.team_id, attempt, session.start.server_tick, session.start.replica_start_tick, ctx.rejoins);
                if let Some(evidence) = &ctx.evidence { evidence.record_network_event("rejoined", session.start.next_team_sequence, session.start.replica_start_tick, "READY")?; }
                if let Some(presentation) = ctx.presentation.as_mut() {
                    ctx.presentation_sequence = ctx.presentation_sequence.saturating_add(1);
                    presentation.publish_critical(session_state_envelope(ctx.presentation_sequence, State::Ready, "rejoined")).await?;
                }
                return Ok(Some(session));
            }
            Some(Ok(Err(error))) => log::warn!("client-runtime rejoin attempt {attempt} failed: {error}"),
            Some(Err(_)) => log::warn!("client-runtime rejoin attempt {attempt} timed out after {} ms", ATTEMPT_TIMEOUT.as_millis()),
        }
    }
}

/// One server session: bootstrap the team replica from this TeamGameStart and
/// run until the session stops or its link is lost.
async fn run_session(ctx: &mut RuntimeContext, mut session: SelectiveSession) -> anyhow::Result<SessionEnd> {
    let RuntimeContext { config, presentation: hub, evidence: evidence_slot, presentation_sequence: sequence_slot,
        resume_token, shutdown, shutdown_rx, .. } = ctx;
    let config: &ClientRuntimeConfig = config;
    let shutdown: &ShutdownToken = shutdown;
    // The newest secret replaces the last one; it is never logged.
    *resume_token = session.start.resume_token.clone();
    let mut session_end = SessionEnd::Stopped;
    let mut replica = ReplicaHost::bootstrap(&session.start)?;
    let (checkpoint_queue, checkpoint_writer) =
        spawn_checkpoint_writer(session.client.replica_checkpoint_reporter());
    let mut checkpoint_failure = checkpoint_writer.subscribe_failure();
    let bootstrap_hash = replica.current_team_hash();
    checkpoint_queue
        .enqueue(omoba_core::game_proto::ClientReplicaCheckpointReport {
            team_id: config.team_id,
            frame_sequence: session.start.next_team_sequence.saturating_sub(1),
            replica_tick: session.start.replica_start_tick,
            authority_revision: Some(omoba_core::game_proto::AuthorityRevision { value: 0 }),
            view_epoch: session.start.view_epoch.clone(),
            pre_repair_hash: bootstrap_hash.to_vec(),
            post_repair_hash: bootstrap_hash.to_vec(),
            encoded_frame_hash: <sha2::Sha256 as sha2::Digest>::digest(
                session.start.encode_to_vec(),
            )
            .to_vec(),
        })
        .await?;
    // Evidence and the renderer link belong to the process, not to one session.
    if evidence_slot.is_none() { *evidence_slot = EvidenceRecorder::create(config, replica.global_seed())?; }
    let evidence: &Option<EvidenceRecorder> = evidence_slot;
    if hub.is_none() {
        *hub = Some(PresentationHub::bind(config).await?);
        log::info!(
            "client-runtime presentation listening on {}",
            config.presentation_bind
        );
    }
    let mut presentation: &mut PresentationHub = hub.as_mut().expect("presentation bound");
    let mut input_bridge = InputBridge::resume_after(session.start.last_seen_input_id);
    let shop_version = session.start.shop_protocol_version;
    let shop_rules = session.start.shop_rules_hash.as_str();
    let (transactions, undo) = omoba_core::runtime::shop_transport::shop_capability_from_echo(shop_version, shop_rules)
        .map_err(|reason| anyhow::anyhow!(reason))?;
    input_bridge.set_shop_enabled(transactions);
    input_bridge.set_shop_undo_enabled(undo);
    presentation.set_shop_protocol_enabled(transactions);
    presentation.set_shop_undo_enabled(undo);
    let recall_enabled = session.start.recall_protocol_version == omoba_core::runtime::recall_transport::RECALL_PROTOCOL_VERSION;
    input_bridge.set_recall_enabled(recall_enabled);
    input_bridge.set_commands_enabled(omoba_core::runtime::command_transport::negotiate_command_protocol(
        session.start.command_protocol_version, &session.start.command_rules_hash)
        .map_err(|reason| anyhow::anyhow!("command capability: {reason}"))?);
    let summoner_enabled = omoba_core::runtime::summoner_transport::negotiate_summoner_protocol(
        session.start.summoner_protocol_version, &session.start.summoner_rules_hash)
        .map_err(|reason| anyhow::anyhow!("summoner capability: {reason}"))?;
    input_bridge.set_summoner_enabled(summoner_enabled);
    let ward_enabled = omoba_core::runtime::ward_transport::negotiate_ward_protocol(
        session.start.ward_protocol_version, &session.start.ward_rules_hash)
        .map_err(|reason| anyhow::anyhow!("ward capability: {reason}"))?;
    input_bridge.set_ward_enabled(ward_enabled);
    presentation.set_summoner_protocol_enabled(summoner_enabled);
    presentation.set_recall_protocol_enabled(recall_enabled);
    if config.test_mode && std::env::var("OMOBA_SHOP_QUERY_SMOKE").as_deref() == Ok("1") {
        // No transaction fixture: query a missing shop result only.
        session.client.query_shop_receipt(0x5348_4f50, u32::MAX).await?;
    }
    log::info!("input allocator resumed player={} after_id={}",
        config.player_id, session.start.last_seen_input_id);
    let mut presentation_sequence = *sequence_slot;
    let mut scripted_move_sent = false;
    let mut next_scripted_move_tick = config.scripted_move_tick;
    let mut scripted_move_ordinal = 0_u64;
    let mut scripted_stall_injected = false;
    let mut scripted_move_origin = None;
    let mut scripted_move_applied = false;
    let mut scripted_hidden_target_sent = false;
    let mut screenshot_marked = false;
    let mut fault_injected = false;
    let mut rebase_probe_sent = false;
    let mut pending_frames = BTreeMap::new();
    let mut pending_input_requests = BTreeMap::<u32, u64>::new();
    let mut input_deadlines = InputSettleDeadlines::default();
    let mut replay_requested_from = None;
    let mut replay_request_id = 1_u64;
    let mut awaiting_authoritative_rebase = false;
    let mut latest_server_tick = session.start.server_tick;
    let mut replica_lag = ReplicaLagTracker::new(session.start.replica_start_tick);
    let mut last_presentation_heroes = Vec::<String>::new();
    let mut replica_steps = omoba_core::comp::perf_window::ReplicaStepWindow::new(
        config.player_id,
        config.team_id,
    );
    let mut replica_stages = omoba_core::comp::replica_stage::ReplicaStageWindow::new(
        config.player_id,
        config.team_id,
    );
    log::info!("{}", replica_steps.enable_line());
    maybe_send_communication_hello(&mut presentation, &session.client, config.player_id, config.team_id, replica.view_epoch(), session.start.server_tick).await;
    presentation.publish_latest(ready_envelope_for_start(presentation_sequence,&config,&session.start)?);
    presentation_sequence = presentation_sequence.saturating_add(1);
    presentation
        .publish_critical(reset_view_envelope(
            presentation_sequence,
            config.team_id,
            session.start.server_tick,
            session.start.replica_start_tick,
            replica.view_epoch(),
        ))
        .await?;
    log::info!(
        "client-runtime ready player_id={} team_id={} replica_tick={} presentation={}",
        config.player_id,
        replica.team_id(),
        session.start.replica_start_tick,
        config.presentation_bind
    );
    let mut shop_query_interval = tokio::time::interval(std::time::Duration::from_secs(1));
    let mut liveness_interval = tokio::time::interval(std::time::Duration::from_millis(250));
    let mut link = LinkLiveness::new(std::time::Instant::now(), replica.next_replica_tick());
    loop {
        tokio::select! {
            biased;
            _ = liveness_interval.tick() => {
                use omoba_core::game_proto::runtime_session_state::State;
                for input_id in input_deadlines.expire(latest_server_tick) {
                    let Some(request_id) = pending_input_requests.remove(&input_id) else { continue; };
                    log::warn!("input expired unsettled player={} input_id={} server_tick={}", config.player_id, input_id, latest_server_tick);
                    presentation_sequence = presentation_sequence.saturating_add(1);
                    presentation.publish_critical(critical_result(presentation_sequence, request_id, input_id, false, "INPUT_EXPIRED", latest_server_tick)).await?;
                }
                let progress = replica.next_replica_tick();
                match link.poll(std::time::Instant::now(), progress) {
                    Some(LinkTransition::Stalled { silent_for }) => {
                        log::warn!("authority link stalled: no replica progress for {} ms at replica_tick={}", silent_for.as_millis(), progress);
                        if let Some(evidence) = &evidence { evidence.record_network_event("link-stalled", replica.expected_team_sequence(), progress, "STALLED")?; }
                        presentation_sequence = presentation_sequence.saturating_add(1);
                        presentation.publish_critical(session_state_envelope(presentation_sequence, State::Stalled, "link_stalled")).await?;
                    }
                    Some(LinkTransition::Recovered { stalled_for }) => {
                        log::warn!("authority link recovered after {} ms at replica_tick={}", stalled_for.as_millis(), progress);
                        if let Some(evidence) = &evidence { evidence.record_network_event("link-recovered", replica.expected_team_sequence(), progress, "READY")?; }
                        presentation_sequence = presentation_sequence.saturating_add(1);
                        presentation.publish_critical(session_state_envelope(presentation_sequence, State::Ready, "link_recovered")).await?;
                    }
                    Some(LinkTransition::Lost { silent_for }) => {
                        log::error!("authority link lost: no replica progress for {} ms (tolerance {} ms)", silent_for.as_millis(),
                            omoba_client_runtime::link_liveness::LINK_LOST_AFTER.as_millis());
                        if let Some(evidence) = &evidence { evidence.record_network_event("link-lost", replica.expected_team_sequence(), progress, "RECONNECTING")?; }
                        presentation_sequence = presentation_sequence.saturating_add(1);
                        presentation.publish_critical(session_state_envelope(presentation_sequence, State::Reconnecting, "link_lost")).await?;
                        // Not the end of the match: rejoin it (see `reconnect`).
                        session_end = SessionEnd::LinkLost;
                        break;
                    }
                    None => {}
                }
            }
            _ = shop_query_interval.tick() => {
                for batch in presentation.expire_communication_submissions() {
                    presentation_sequence = presentation_sequence.saturating_add(1);
                    presentation.publish_critical(communication_envelope(presentation_sequence, batch)).await?;
                }
                for id in presentation.due_shop_queries() {
                    session.client.query_shop_receipt(u64::from(id), id).await?;
                }
                for id in presentation.due_undo_queries() {
                    session.client.query_shop_receipt(u64::from(id), id).await?;
                }
            }
            changed = shutdown_rx.changed() => {
                if changed.is_err() || shutdown_rx.borrow().is_some() {
                    if let Some(evidence) = &evidence { evidence.record_network_event("session-stopped", replica.expected_team_sequence(), replica.next_replica_tick(), "SHUTDOWN")?; }
                    log::error!("client-runtime stopping: {:?}", shutdown_rx.borrow().clone());
                    break;
                }
            },
            changed = checkpoint_failure.changed() => {
                if changed.is_err() {
                    shutdown.cancel(ShutdownReason::UnsafeSession("checkpoint writer status closed".into()));
                } else if let Some(error) = checkpoint_failure.borrow().clone() {
                    shutdown.cancel(ShutdownReason::UnsafeSession(format!("checkpoint writer failed: {error}")));
                }
            },
            renderer_input = presentation.recv_input(), if presentation.has_pending_input() => {
                let Some(renderer_input) = renderer_input else { continue; };
                handle_renderer_input(
                    &config,
                    &mut session,
                    &replica,
                    &mut presentation,
                    &evidence,
                    &mut input_bridge,
                    &mut presentation_sequence,
                    &mut scripted_move_sent,
                    &mut scripted_move_origin,
                    &mut pending_input_requests,
                    &mut input_deadlines,
                    &mut latest_server_tick,
                    renderer_input,
                ).await?;
                catch_up_available_frames(
                    &config,
                    &mut session,
                    &mut replica,
                    &mut presentation,
                    &evidence,
                    &mut replica_steps,
                    &mut replica_stages,
                    &checkpoint_queue,
                    &mut pending_frames,
                    &mut replica_lag,
                    &mut awaiting_authoritative_rebase,
                    &mut presentation_sequence,
                    &mut scripted_move_sent,
                    &mut next_scripted_move_tick,
                    &mut scripted_move_ordinal,
                    &mut scripted_stall_injected,
                    &mut scripted_move_origin,
                    &mut scripted_move_applied,
                    &mut scripted_hidden_target_sent,
                    &mut screenshot_marked,
                    &mut fault_injected,
                    &mut pending_input_requests,
                    &mut last_presentation_heroes,
                    &mut latest_server_tick,
                    &mut replay_requested_from,
                    &mut replay_request_id,
                    &mut rebase_probe_sent,
                    &shutdown,
                ).await?;
            },
            inbound = session.inbound.recv() => match inbound {
                Some(LockstepInbound::CommunicationHelloResult { msg }) => {
                    dispatch_communication(&mut presentation, &session.client, &mut presentation_sequence, &LockstepInbound::CommunicationHelloResult { msg }).await?;
                }
                Some(LockstepInbound::CommunicationReceipt { msg }) => {
                    dispatch_communication(&mut presentation, &session.client, &mut presentation_sequence, &LockstepInbound::CommunicationReceipt { msg }).await?;
                }
                Some(LockstepInbound::CommunicationEvent { msg }) => {
                    dispatch_communication(&mut presentation, &session.client, &mut presentation_sequence, &LockstepInbound::CommunicationEvent { msg }).await?;
                }
                Some(LockstepInbound::CommunicationReplay { msg }) => {
                    dispatch_communication(&mut presentation, &session.client, &mut presentation_sequence, &LockstepInbound::CommunicationReplay { msg }).await?;
                }
                Some(LockstepInbound::TeamTickFrame { msg, encoded, .. }) => {
                    maybe_send_communication_hello(&mut presentation, &session.client, config.player_id, config.team_id, replica.view_epoch(), msg.server_tick).await;
                    latest_server_tick = latest_server_tick.max(msg.server_tick);
                    replica_lag.observe_received(msg.replica_tick);
                    if presentation_trace_enabled() {
                        let transitions = frame_transition_labels(&msg);
                        if !transitions.is_empty() {
                            log::warn!(
                                "presentation_trace stage=runtime_frame_receive team={} sequence={} server_tick={} replica_tick={} transitions={:?}",
                                msg.team_id,
                                msg.team_sequence,
                                msg.server_tick,
                                msg.replica_tick,
                                transitions,
                            );
                        }
                    }
                    if let Some(evidence) = &evidence { evidence.record_network_event("frame-received", msg.team_sequence, msg.replica_tick, "OK")?; }
                    if !rebase_probe_sent
                        && config.test_mode
                        && config.team_id == 1
                        && config.rebase_probe_tick.is_some_and(|tick| msg.replica_tick >= tick)
                    {
                        replay_request_id = replay_request_id.saturating_add(1);
                        if let Some(evidence) = &evidence {
                            evidence.record_network_event("rebase-probe-requested", msg.team_sequence, msg.replica_tick, "TEST_HASH_MISMATCH")?;
                        }
                        session.client.report_team_hash_mismatch(&ClientTeamHashMismatch {
                            team_id: config.team_id,
                            frame_sequence: msg.team_sequence,
                            replica_tick: msg.replica_tick,
                            received_hash: vec![0; 32],
                            view_epoch: Some(ViewEpoch { value: replica.view_epoch() }),
                        }).await?;
                        rebase_probe_sent = true;
                    }
                    if msg.team_id != config.team_id {
                        if let Some(evidence) = &evidence { evidence.record_network_event("wrong-team-rejected", msg.team_sequence, msg.replica_tick, "WRONG_TEAM")?; }
                        shutdown.cancel(ShutdownReason::UnsafeSession("wrong team frame".into()));
                        continue;
                    }
                    if let Some(evidence) = &evidence { evidence.record_wire_frame(&encoded)?; }
                    if awaiting_authoritative_rebase {
                        if let Some(evidence) = &evidence {
                            evidence.record_network_event(
                                "frame-deferred-for-rebase",
                                msg.team_sequence,
                                msg.replica_tick,
                                "AWAITING_AUTHORITATIVE_REBASE",
                            )?;
                        }
                    } else if msg.team_sequence >= replica.expected_team_sequence() {
                        ingest_pending_frame(&mut pending_frames, replica.expected_team_sequence(), msg.team_sequence, (msg, encoded));
                    } else if let Some(evidence) = &evidence {
                        evidence.record_network_event("duplicate-rejected", msg.team_sequence, msg.replica_tick, "DUPLICATE")?;
                    }
                    catch_up_available_frames(
                        &config,
                        &mut session,
                        &mut replica,
                        &mut presentation,
                        &evidence,
                        &mut replica_steps,
                        &mut replica_stages,
                        &checkpoint_queue,
                        &mut pending_frames,
                        &mut replica_lag,
                        &mut awaiting_authoritative_rebase,
                        &mut presentation_sequence,
                        &mut scripted_move_sent,
                        &mut next_scripted_move_tick,
                        &mut scripted_move_ordinal,
                        &mut scripted_stall_injected,
                        &mut scripted_move_origin,
                        &mut scripted_move_applied,
                        &mut scripted_hidden_target_sent,
                        &mut screenshot_marked,
                        &mut fault_injected,
                        &mut pending_input_requests,
                        &mut last_presentation_heroes,
                        &mut latest_server_tick,
                        &mut replay_requested_from,
                        &mut replay_request_id,
                        &mut rebase_probe_sent,
                        &shutdown,
                    ).await?;
                }
                Some(LockstepInbound::TeamViewRebaseChunk { msg, .. }) => {
                    if !replica.receive_rebase_chunk(&msg) {
                        log::warn!("rejected unverified rebase chunk");
                    }
                }
                Some(LockstepInbound::TeamViewRebaseManifest { msg, .. }) => {
                    match replica.receive_rebase_manifest(&msg) {
                        Ok(()) => {
                            if let Some(evidence) = &evidence { evidence.record_network_event("rebase-applied", msg.resume_team_sequence, msg.authoritative_tick, "VERIFIED")?; }
                            pending_frames.clear();
                            awaiting_authoritative_rebase = false;
                            replay_requested_from = None;
                            let view_epoch = msg.view_epoch.as_ref().map_or(0, |value| value.value);
                            publish_rebased_presentation(&mut presentation, &mut replica,
                                &mut presentation_sequence, msg.authoritative_tick,
                                session.client.latest_rtt_us().unwrap_or(0)).await?;
                            if let Some((recovery_request, recovery_epoch)) = replica.poll_owner_ward_recovery() {
                                if let Err(error) = session.client.query_owner_ward_recovery(recovery_request, recovery_epoch).await {
                                    log::warn!("owner ward recovery query failed: {error}");
                                }
                            }
                            if let Err(error) = session.client.acknowledge_team_rebase(
                                msg.team_id, msg.resume_team_sequence, view_epoch,
                            ).await {
                                shutdown.cancel(ShutdownReason::UnsafeSession(error.to_string()));
                            } else {
                                replay_request_id = replay_request_id.saturating_add(1);
                                if let Some(evidence) = &evidence { evidence.record_network_event("replay-requested", msg.resume_team_sequence, msg.authoritative_tick, "POST_REBASE")?; }
                                if let Err(error) = session.client.request_team_replay(
                                    replay_request_id, msg.resume_team_sequence, view_epoch,
                                ).await {
                                    shutdown.cancel(ShutdownReason::UnsafeSession(error.to_string()));
                                } else {
                                    replay_requested_from = Some(msg.resume_team_sequence);
                                }
                            }
                        }
                        Err(error) => shutdown.cancel(ShutdownReason::UnsafeSession(error.to_string())),
                    }
                }
                Some(LockstepInbound::OwnerWardEnvelope { msg }) => {
                    if let Err(error) = replica.apply_owner_ward_envelope(&msg) {
                        log::warn!("owner ward envelope rejected: {error}");
                        if let Some((recovery_request, recovery_epoch)) = replica.poll_owner_ward_recovery() {
                            if let Err(error) = session.client.query_owner_ward_recovery(recovery_request, recovery_epoch).await {
                                log::warn!("owner ward recovery query failed: {error}");
                            }
                        }
                    }
                }
                Some(LockstepInbound::ShopReceiptReplay { msg }) => {
                    match presentation.retain_shop_replay(replica.view_epoch(), &msg) {
                        Ok(terminal) => {
                            publish_shop_replay_outcome(&mut presentation, &mut pending_input_requests,
                                &mut presentation_sequence, &msg).await?;
                            log::info!("shop receipt recovery input_id={} status={} terminal={}", msg.input_id, msg.status, terminal);
                        },
                        Err(code) => shutdown.cancel(ShutdownReason::UnsafeSession(code.into())),
                    }
                }
                Some(LockstepInbound::SecureTargetInputResult { msg, .. }) => {
                    if let Some(evidence) = &evidence { evidence.record_network_event("secure-input-result", 0, replica.next_replica_tick(), if msg.accepted { "SERVER_ACCEPTED" } else { "SERVER_INVALID_TARGET" })?; }
                    presentation_sequence = presentation_sequence.saturating_add(1);
                    let renderer_request = secure_result_request_id(&mut pending_input_requests, msg.request_id, msg.accepted);
                    presentation.publish_critical(critical_result(
                        presentation_sequence,
                        renderer_request,
                        u32::try_from(msg.request_id).unwrap_or(0),
                        msg.accepted,
                        if msg.accepted { "SERVER_ACCEPTED" } else { "SERVER_INVALID_TARGET" },
                        replica.next_replica_tick(),
                    )).await?;
                }
                Some(LockstepInbound::MatchTerminated { reason_class, last_safe_sequence }) => {
                    log::error!("match terminated by server: reason={reason_class} last_safe_sequence={last_safe_sequence}");
                    presentation_sequence = presentation_sequence.saturating_add(1);
                    presentation.publish_critical(session_state_envelope(presentation_sequence,
                        omoba_core::game_proto::runtime_session_state::State::Terminated, &format!("match_terminated:{reason_class}"))).await?;
                    shutdown.cancel(ShutdownReason::MatchTerminated(reason_class));
                }
                Some(_) => {}
                // The server closed this session (it saw our link die first).
                // After a stop or a match termination this is just the close.
                None if shutdown_rx.borrow().is_some() => {}
                None => {
                    log::error!("authority session closed by the server; rejoining");
                    if let Some(evidence) = &evidence { evidence.record_network_event("link-lost", replica.expected_team_sequence(), replica.next_replica_tick(), "RECONNECTING")?; }
                    presentation_sequence = presentation_sequence.saturating_add(1);
                    presentation.publish_critical(session_state_envelope(presentation_sequence,
                        omoba_core::game_proto::runtime_session_state::State::Reconnecting, "session_closed")).await?;
                    session_end = SessionEnd::LinkLost;
                    break;
                }
            },
        }
    }
    drop(checkpoint_queue);
    if let Err(error) = checkpoint_writer.finish().await {
        log::warn!("checkpoint writer shutdown failed: {error}");
    }
    if let Err(error) = session.client.shutdown().await {
        log::warn!("client-runtime KCP shutdown failed: {error}");
    }
    *sequence_slot = presentation_sequence;
    Ok(session_end)
}

async fn maybe_send_communication_hello(
    presentation: &mut PresentationHub,
    client: &omoba_core::kcp::client::KcpClient,
    player_id: u32,
    team_id: u32,
    view_epoch: u64,
    server_tick: u64,
) {
    presentation.note_communication_transition(view_epoch, server_tick);
    if !presentation.communication_hello_allowed() {
        return;
    }
    presentation.note_communication_hello_sent();
    if let Err(error) = client.send_communication_hello(player_id, team_id, view_epoch).await {
        log::warn!("communication hello failed: {error}");
    }
}

async fn dispatch_communication(
    presentation: &mut PresentationHub,
    client: &omoba_core::kcp::client::KcpClient,
    sequence: &mut u64,
    inbound: &LockstepInbound,
) -> anyhow::Result<()> {
    let batch = match inbound {
        LockstepInbound::CommunicationHelloResult { msg } => {
            let batch = presentation.apply_communication_hello(msg);
            if let Some(query) = presentation.communication_replay_query() {
                match client.send_communication_replay_query(&query).await {
                    Ok(()) => presentation.note_communication_replay_sent(),
                    Err(error) => log::warn!("communication replay query failed: {error}"),
                }
            }
            batch
        }
        LockstepInbound::CommunicationReceipt { msg } => presentation.apply_communication_receipt(msg),
        LockstepInbound::CommunicationEvent { msg } => presentation.apply_communication_event(msg),
        LockstepInbound::CommunicationReplay { msg } => presentation.apply_communication_replay(msg),
        _ => None,
    };
    if let Some(batch) = batch {
        *sequence = sequence.saturating_add(1);
        presentation.publish_critical(communication_envelope(*sequence, batch)).await?;
    }
    Ok(())
}

async fn publish_rebased_presentation(
    presentation: &mut PresentationHub,
    replica: &mut ReplicaHost,
    sequence: &mut u64,
    authoritative_tick: u64,
    rtt_us: u64,
) -> anyhow::Result<()> {
    let _ = presentation.observe_communication_view(replica.view_epoch());
    *sequence = sequence.saturating_add(1);
    let reset = reset_view_envelope(*sequence, replica.team_id(),
        authoritative_tick, replica.next_replica_tick(), replica.view_epoch());
    let communication = presentation.current_communication_presentation().map(|batch| {
        *sequence = sequence.saturating_add(1);
        communication_envelope(*sequence, batch)
    });
    *sequence = sequence.saturating_add(1);
    let source = replica.extract_presentation_source();
    presentation.set_owner_ward_private(
        replica.view_epoch(),
        replica.owner_ward_protocol(),
        replica.verified_owner_ward_bytes(),
    );
    let snapshot = presentation.snapshot_envelope(*sequence, authoritative_tick,
        replica.view_epoch(), source, rtt_us);
    let ordered = assemble_rebased_presentation(reset, communication, snapshot);
    for envelope in &ordered {
        presentation.publish_critical(envelope.clone()).await?;
    }
    if let Some(snapshot) = ordered.last() {
        presentation.publish_latest(snapshot.clone());
    }
    Ok(())
}

async fn publish_shop_replay_outcome(
    presentation: &mut PresentationHub,
    pending: &mut BTreeMap<u32, u64>,
    sequence: &mut u64,
    reply: &omoba_core::game_proto::ShopReceiptReplay,
) -> anyhow::Result<()> {
    let (accepted, code, tick) = if reply.schema_version == 2 {
        match reply.status {
            2 => {
                let receipt = omoba_core::runtime::shop_undo_wire::ShopUndoReceipt::decode(&reply.receipt)
                    .ok_or_else(|| anyhow::anyhow!("invalid validated shop undo receipt"))?;
                (receipt.result_code == 0, omoba_client_runtime::shop_presentation::undo_result_name(receipt.result_code), receipt.tick)
            }
            3 => (false, "UNDO_RESULT_UNCERTAIN", 0),
            4 => (false, "SHOP_ADMISSION_REJECTED_LATE", 0),
            _ => return Ok(()),
        }
    } else {
        match reply.status {
            2 => {
                let receipt = omoba_core::runtime::shop_receipt::ShopReceipt::decode(&reply.receipt)
                    .ok_or_else(|| anyhow::anyhow!("invalid validated shop receipt"))?;
                (receipt.result_code == 0, omoba_client_runtime::shop_presentation::receipt_result_name(receipt.result_code), receipt.tick)
            }
            3 => (false, "SHOP_RESULT_EXPIRED_UNCERTAIN", 0),
            4 => (false, "SHOP_ADMISSION_REJECTED_LATE", 0),
            _ => return Ok(()),
        }
    };
    if matches!(code, "SHOP_RESULT_EXPIRED_UNCERTAIN" | "SHOP_ADMISSION_REJECTED_LATE") {
        if let Some(request_id) = pending.get(&reply.input_id).copied() {
            presentation.note_shop_uncertain_request(request_id);
        }
    }
    if let Some(request_id) = pending.remove(&reply.input_id) {
        *sequence = sequence.saturating_add(1);
        presentation.publish_critical(critical_result(*sequence, request_id, reply.input_id, accepted, code, tick)).await?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_renderer_input(
    config: &ClientRuntimeConfig,
    session: &mut SelectiveSession,
    replica: &ReplicaHost,
    presentation: &mut PresentationHub,
    evidence: &Option<EvidenceRecorder>,
    input_bridge: &mut InputBridge,
    presentation_sequence: &mut u64,
    scripted_move_sent: &mut bool,
    scripted_move_origin: &mut Option<(i64, i64)>,
    pending_input_requests: &mut BTreeMap<u32, u64>,
    input_deadlines: &mut InputSettleDeadlines,
    latest_server_tick: &mut u64,
    renderer_input: RendererInput,
) -> anyhow::Result<()> {
    let request_id = renderer_input.request_id;
    if matches!(renderer_input.intent, Some(omoba_core::game_proto::renderer_input::Intent::Communication(_))) {
        match presentation.prepare_renderer_communication(&renderer_input, replica) {
            Ok((submit, queued)) => {
                *presentation_sequence = presentation_sequence.saturating_add(1);
                presentation.publish_critical(communication_envelope(*presentation_sequence, queued)).await?;
                if let Err(error) = session.client.send_communication_submit(&submit).await {
                    log::warn!("communication submit transport uncertain: {error}");
                    if let Some(batch) = presentation.fail_communication_submit_locally(submit.request_id) {
                        *presentation_sequence = presentation_sequence.saturating_add(1);
                        presentation.publish_critical(communication_envelope(*presentation_sequence, batch)).await?;
                    }
                }
            }
            Err(refusal) => {
                if let Some(batch) = presentation.refuse_renderer_communication(&renderer_input, refusal) {
                    *presentation_sequence = presentation_sequence.saturating_add(1);
                    presentation.publish_critical(communication_envelope(*presentation_sequence, batch)).await?;
                }
            }
        }
        return Ok(());
    }
    let undo_sequence = match &renderer_input.intent {
        Some(omoba_core::game_proto::renderer_input::Intent::ShopUndo(intent)) => intent.sequence,
        _ => 0,
    };
    let shop_action = match &renderer_input.intent {
        Some(omoba_core::game_proto::renderer_input::Intent::ItemBuy(_)) => 17,
        Some(omoba_core::game_proto::renderer_input::Intent::ItemSell(_)) => 18,
        _ => 0,
    };
    let is_shop = shop_action != 0;
    let is_undo = matches!(renderer_input.intent, Some(omoba_core::game_proto::renderer_input::Intent::ShopUndo(_)));
    let is_ward = matches!(renderer_input.intent, Some(omoba_core::game_proto::renderer_input::Intent::WardPlacement(_)));
    if is_undo && !presentation.can_track_undo() {
        *presentation_sequence = presentation_sequence.saturating_add(1);
        presentation.publish_critical(critical_result(*presentation_sequence, request_id, 0, false, "UNDO_PENDING_CAPACITY", replica.next_replica_tick())).await?;
        return Ok(());
    }
    if is_shop && !presentation.can_track_shop() {
        *presentation_sequence = presentation_sequence.saturating_add(1);
        presentation.publish_critical(critical_result(*presentation_sequence, request_id, 0, false, "SHOP_PENDING_CAPACITY", replica.next_replica_tick())).await?;
        return Ok(());
    }
    input_bridge.set_owner_shop_disclosure(presentation.owner_economy_fact(replica.view_epoch()).cloned().map(|economy| {
        omoba_client_runtime::input_bridge::OwnerShopDisclosure {
            epoch: replica.view_epoch(),
            economy,
            undo: presentation.owner_undo_fact(replica.view_epoch()).cloned(),
        }
    }));
    match input_bridge.validate(renderer_input, config.player_id, replica) {
        InputDecision::RepeatedShop { request_id, input_id } => {
            if is_undo {
                presentation.track_undo(input_id);
                presentation.note_undo_request(replica.view_epoch(), config.player_id, input_id, request_id, undo_sequence);
            } else {
                match presentation.note_shop_request(replica.view_epoch(), config.player_id, input_id, request_id, shop_action) {
                    omoba_client_runtime::shop_presentation::ShopRequestNote::Pending => {
                        presentation.track_shop(input_id);
                        pending_input_requests.insert(input_id, request_id);
                    }
                    omoba_client_runtime::shop_presentation::ShopRequestNote::Completed => {
                        if pending_input_requests.get(&input_id).is_some_and(|existing| *existing != request_id) {
                            *presentation_sequence = presentation_sequence.saturating_add(1);
                            presentation.publish_critical(critical_result(*presentation_sequence, request_id, input_id, false, "SHOP_RESULT_QUERY_ONLY", replica.next_replica_tick())).await?;
                            return Ok(());
                        }
                        pending_input_requests.entry(input_id).or_insert(request_id);
                    }
                    omoba_client_runtime::shop_presentation::ShopRequestNote::Conflict => {
                        *presentation_sequence = presentation_sequence.saturating_add(1);
                        presentation.publish_critical(critical_result(*presentation_sequence, request_id, input_id, false, "SHOP_RESULT_QUERY_ONLY", replica.next_replica_tick())).await?;
                        return Ok(());
                    }
                    omoba_client_runtime::shop_presentation::ShopRequestNote::Capacity => {
                        *presentation_sequence = presentation_sequence.saturating_add(1);
                        presentation.publish_critical(critical_result(*presentation_sequence, request_id, 0, false, "SHOP_PENDING_CAPACITY", replica.next_replica_tick())).await?;
                        return Ok(());
                    }
                }
            }
            *presentation_sequence = presentation_sequence.saturating_add(1);
            presentation.publish_critical(critical_result(*presentation_sequence, request_id, input_id,
                true, if is_undo { "UNDO_RESULT_QUERY_ONLY" } else { "SHOP_RESULT_QUERY_ONLY" }, replica.next_replica_tick())).await?;
        }
        InputDecision::RepeatedWard { request_id, input_id } => {
            presentation.note_ward_request(replica.view_epoch(), config.player_id, 0, input_id, request_id, omoba_core::runtime::ward_transport::WARD_PHASE_FORWARDED);
            pending_input_requests.insert(input_id, request_id);
            *presentation_sequence = presentation_sequence.saturating_add(1);
            presentation.publish_critical(critical_result(*presentation_sequence, request_id, input_id,
                true, "WARD_RESULT_QUERY_ONLY", replica.next_replica_tick())).await?;
        }
        InputDecision::Accepted {
            input_id,
            input,
            secure_target,
        } => {
            if matches!(
                &input.action,
                Some(omoba_core::game_proto::player_input::Action::MoveTo(_))
            ) {
                *scripted_move_sent = true;
                if scripted_move_origin.is_none() {
                    *scripted_move_origin = replica.owned_hero_position(config.player_id);
                }
            }
            let network_lead_ticks = input_lookahead_ticks();
            let freshest_server_tick = (*latest_server_tick).max(session.client.latest_team_server_tick());
            let target_tick =
                u32::try_from(freshest_server_tick.saturating_add(network_lead_ticks))
                    .unwrap_or(u32::MAX);
            // A failed write may have reached the server. Retain the original
            // identity for read-only recovery, never assume no transaction ran.
            if is_shop {
                match presentation.note_shop_request(replica.view_epoch(), config.player_id, input_id, request_id, shop_action) {
                    omoba_client_runtime::shop_presentation::ShopRequestNote::Pending => {
                        presentation.track_shop(input_id);
                        pending_input_requests.insert(input_id, request_id);
                    }
                    omoba_client_runtime::shop_presentation::ShopRequestNote::Completed => {
                        if pending_input_requests.get(&input_id).is_some_and(|existing| *existing != request_id) {
                            *presentation_sequence = presentation_sequence.saturating_add(1);
                            presentation.publish_critical(critical_result(*presentation_sequence, request_id, input_id, false, "SHOP_RESULT_QUERY_ONLY", replica.next_replica_tick())).await?;
                            return Ok(());
                        }
                        pending_input_requests.entry(input_id).or_insert(request_id);
                        *presentation_sequence = presentation_sequence.saturating_add(1);
                        presentation.publish_critical(critical_result(*presentation_sequence, request_id, input_id, true, "SHOP_RESULT_QUERY_ONLY", replica.next_replica_tick())).await?;
                        return Ok(());
                    }
                    omoba_client_runtime::shop_presentation::ShopRequestNote::Conflict => {
                        *presentation_sequence = presentation_sequence.saturating_add(1);
                        presentation.publish_critical(critical_result(*presentation_sequence, request_id, input_id, false, "SHOP_RESULT_QUERY_ONLY", replica.next_replica_tick())).await?;
                        return Ok(());
                    }
                    omoba_client_runtime::shop_presentation::ShopRequestNote::Capacity => {
                        *presentation_sequence = presentation_sequence.saturating_add(1);
                        presentation.publish_critical(critical_result(*presentation_sequence, request_id, 0, false, "SHOP_PENDING_CAPACITY", replica.next_replica_tick())).await?;
                        return Ok(());
                    }
                }
            }
            if is_undo {
                presentation.track_undo(input_id);
                pending_input_requests.insert(input_id, request_id);
                presentation.note_undo_request(replica.view_epoch(), config.player_id, input_id, request_id, undo_sequence);
            }
            if is_ward {
                presentation.note_ward_request(replica.view_epoch(), config.player_id, 0, input_id, request_id, omoba_core::runtime::ward_transport::WARD_PHASE_FORWARDED);
            }
            let result = if let Some(target) = secure_target {
                let Some(actor) = replica.owned_hero_reference(config.player_id) else {
                    *presentation_sequence = presentation_sequence.saturating_add(1);
                    presentation
                        .publish_critical(critical_result(
                            *presentation_sequence,
                            request_id,
                            input_id,
                            false,
                            "OWN_HERO_NOT_DISCLOSED",
                            u64::from(target_tick),
                        ))
                        .await?;
                    return Ok(());
                };
                session
                    .client
                    .submit_secure_target(&omoba_core::game_proto::SecureTargetInput {
                        request_id: u64::from(input_id),
                        player_id: config.player_id,
                        input_tick: u64::from(target_tick),
                        actor: Some(actor),
                        target: Some(target),
                        action_kind: 0,
                        sanitized_payload: input.encode_to_vec(),
                    })
                    .await
                    .map(|_| (0, 0))
            } else {
                let original = input.clone();
                let result = session
                    .client
                    .submit_input(target_tick, input, input_id)
                    .await;
                if is_shop && result.is_ok() && config.test_mode && std::env::var("OMOBA_SHOP_TRANSACTION_SMOKE").as_deref() == Ok("1") {
                    for _ in 0..5 { session.client.submit_input(target_tick, original.clone(), input_id).await?; }
                    log::info!("shop smoke exact retries player={} input_id={} count=5", config.player_id, input_id);
                }
                result
            };
            if result.is_ok() {
                pending_input_requests.insert(input_id, request_id);
                // Shop and undo settle through their own receipt queries.
                if !is_shop && !is_undo { input_deadlines.track(input_id, u64::from(target_tick)); }
                log::info!("input forwarded player={} input_id={} target_tick={}",
                    config.player_id, input_id, target_tick);
            }
            if let Some(evidence) = evidence {
                evidence.record_network_event(
                    "input-forwarded",
                    0,
                    u64::from(target_tick),
                    if result.is_ok() {
                        "FORWARDED"
                    } else if is_undo {
                        "UNDO_RESULT_UNCERTAIN"
                    } else if is_shop {
                        "SHOP_TRANSPORT_UNCERTAIN"
                    } else if is_ward {
                        "WARD_TRANSPORT_UNCERTAIN"
                    } else {
                        "SERVER_TRANSPORT_ERROR"
                    },
                )?;
            }
            if is_ward && result.is_err() {
                presentation.note_ward_request(replica.view_epoch(), config.player_id, 0, input_id, request_id, omoba_core::runtime::ward_transport::WARD_PHASE_UNCERTAIN);
            }
            if is_shop && result.is_err() {
                presentation.note_shop_uncertain(replica.view_epoch(), config.player_id, request_id);
            }
            *presentation_sequence = presentation_sequence.saturating_add(1);
            presentation
                .publish_critical(critical_result(
                    *presentation_sequence,
                    request_id,
                    input_id,
                    result.is_ok(),
                    if result.is_ok() {
                        "FORWARDED"
                    } else if is_undo {
                        "UNDO_RESULT_UNCERTAIN"
                    } else if is_shop {
                        "SHOP_TRANSPORT_UNCERTAIN"
                    } else if is_ward {
                        "WARD_TRANSPORT_UNCERTAIN"
                    } else {
                        "SERVER_TRANSPORT_ERROR"
                    },
                    u64::from(target_tick),
                ))
                .await?;
        }
        InputDecision::Rejected { request_id, code } => {
            if is_ward {
                presentation.note_ward_request(replica.view_epoch(), config.player_id, 0, 0, request_id, omoba_core::runtime::ward_transport::WARD_PHASE_REJECTED);
            }
            if let Some(evidence) = evidence {
                evidence.record_network_event(
                    "input-local-rejection",
                    0,
                    replica.next_replica_tick(),
                    code,
                )?;
            }
            *presentation_sequence = presentation_sequence.saturating_add(1);
            presentation
                .publish_critical(critical_result(
                    *presentation_sequence,
                    request_id,
                    0,
                    false,
                    code,
                    replica.next_replica_tick(),
                ))
                .await?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn catch_up_available_frames(
    config: &ClientRuntimeConfig,
    session: &mut SelectiveSession,
    replica: &mut ReplicaHost,
    presentation: &mut PresentationHub,
    evidence: &Option<EvidenceRecorder>,
    replica_steps: &mut omoba_core::comp::perf_window::ReplicaStepWindow,
    replica_stages: &mut omoba_core::comp::replica_stage::ReplicaStageWindow,
    checkpoint_queue: &CheckpointQueue,
    pending_frames: &mut BTreeMap<u64, (omoba_core::game_proto::TeamTickFrame, Arc<[u8]>)>,
    replica_lag: &mut ReplicaLagTracker,
    awaiting_authoritative_rebase: &mut bool,
    presentation_sequence: &mut u64,
    scripted_move_sent: &mut bool,
    next_scripted_move_tick: &mut Option<u64>,
    scripted_move_ordinal: &mut u64,
    scripted_stall_injected: &mut bool,
    scripted_move_origin: &mut Option<(i64, i64)>,
    scripted_move_applied: &mut bool,
    scripted_hidden_target_sent: &mut bool,
    screenshot_marked: &mut bool,
    fault_injected: &mut bool,
    pending_input_requests: &mut BTreeMap<u32, u64>,
    last_presentation_heroes: &mut Vec<String>,
    latest_server_tick: &mut u64,
    replay_requested_from: &mut Option<u64>,
    replay_request_id: &mut u64,
    rebase_probe_sent: &mut bool,
    shutdown: &ShutdownToken,
) -> anyhow::Result<()> {
    while !*awaiting_authoritative_rebase {
        if pending_frames
            .get(&replica.expected_team_sequence())
            .is_none()
        {
            match session.inbound.try_recv() {
                Ok(LockstepInbound::CommunicationHelloResult { msg }) => {
                    dispatch_communication(presentation, &session.client, presentation_sequence, &LockstepInbound::CommunicationHelloResult { msg }).await?;
                    continue;
                }
                Ok(LockstepInbound::CommunicationReceipt { msg }) => {
                    dispatch_communication(presentation, &session.client, presentation_sequence, &LockstepInbound::CommunicationReceipt { msg }).await?;
                    continue;
                }
                Ok(LockstepInbound::CommunicationEvent { msg }) => {
                    dispatch_communication(presentation, &session.client, presentation_sequence, &LockstepInbound::CommunicationEvent { msg }).await?;
                    continue;
                }
                Ok(LockstepInbound::CommunicationReplay { msg }) => {
                    dispatch_communication(presentation, &session.client, presentation_sequence, &LockstepInbound::CommunicationReplay { msg }).await?;
                    continue;
                }
                Ok(LockstepInbound::TeamTickFrame { msg, encoded, .. }) => {
                    maybe_send_communication_hello(presentation, &session.client, config.player_id, config.team_id, replica.view_epoch(), msg.server_tick).await;
                    *latest_server_tick = (*latest_server_tick).max(msg.server_tick);
                    replica_lag.observe_received(msg.replica_tick);
                    if let Some(evidence) = evidence {
                        evidence.record_network_event(
                            "frame-received",
                            msg.team_sequence,
                            msg.replica_tick,
                            "OK",
                        )?;
                    }
                    if !*rebase_probe_sent
                        && config.test_mode
                        && config.team_id == 1
                        && config
                            .rebase_probe_tick
                            .is_some_and(|tick| msg.replica_tick >= tick)
                    {
                        *replay_request_id = replay_request_id.saturating_add(1);
                        if let Some(evidence) = evidence {
                            evidence.record_network_event(
                                "rebase-probe-requested",
                                msg.team_sequence,
                                msg.replica_tick,
                                "TEST_HASH_MISMATCH",
                            )?;
                        }
                        session
                            .client
                            .report_team_hash_mismatch(&ClientTeamHashMismatch {
                                team_id: config.team_id,
                                frame_sequence: msg.team_sequence,
                                replica_tick: msg.replica_tick,
                                received_hash: vec![0; 32],
                                view_epoch: Some(ViewEpoch {
                                    value: replica.view_epoch(),
                                }),
                            })
                            .await?;
                        *rebase_probe_sent = true;
                    }
                    if msg.team_id != config.team_id {
                        if let Some(evidence) = evidence {
                            evidence.record_network_event(
                                "wrong-team-rejected",
                                msg.team_sequence,
                                msg.replica_tick,
                                "WRONG_TEAM",
                            )?;
                        }
                        shutdown.cancel(ShutdownReason::UnsafeSession("wrong team frame".into()));
                        return Ok(());
                    }
                    if let Some(evidence) = evidence {
                        evidence.record_wire_frame(&encoded)?;
                    }
                    if *awaiting_authoritative_rebase {
                        if let Some(evidence) = evidence {
                            evidence.record_network_event(
                                "frame-deferred-for-rebase",
                                msg.team_sequence,
                                msg.replica_tick,
                                "AWAITING_AUTHORITATIVE_REBASE",
                            )?;
                        }
                    } else {
                        ingest_pending_frame(
                            pending_frames,
                            replica.expected_team_sequence(),
                            msg.team_sequence,
                            (msg, encoded),
                        );
                    }
                    continue;
                }
                Ok(LockstepInbound::TeamViewRebaseChunk { msg, .. }) => {
                    if !replica.receive_rebase_chunk(&msg) {
                        log::warn!("rejected unverified rebase chunk");
                    }
                    return Ok(());
                }
                Ok(LockstepInbound::TeamViewRebaseManifest { msg, .. }) => {
                    match replica.receive_rebase_manifest(&msg) {
                        Ok(()) => {
                            if let Some(evidence) = evidence {
                                evidence.record_network_event(
                                    "rebase-applied",
                                    msg.resume_team_sequence,
                                    msg.authoritative_tick,
                                    "VERIFIED",
                                )?;
                            }
                            pending_frames.clear();
                            *awaiting_authoritative_rebase = false;
                            *replay_requested_from = None;
                            let view_epoch =
                                msg.view_epoch.as_ref().map_or(0, |value| value.value);
                            publish_rebased_presentation(presentation, replica,
                                presentation_sequence, msg.authoritative_tick,
                                session.client.latest_rtt_us().unwrap_or(0)).await?;
                            if let Some((recovery_request, recovery_epoch)) = replica.poll_owner_ward_recovery() {
                                if let Err(error) = session
                                    .client
                                    .query_owner_ward_recovery(recovery_request, recovery_epoch)
                                    .await
                                {
                                    log::warn!("owner ward recovery query failed: {error}");
                                }
                            }
                            if let Err(error) = session
                                .client
                                .acknowledge_team_rebase(
                                    msg.team_id,
                                    msg.resume_team_sequence,
                                    view_epoch,
                                )
                                .await
                            {
                                shutdown.cancel(ShutdownReason::UnsafeSession(error.to_string()));
                            } else {
                                *replay_request_id = replay_request_id.saturating_add(1);
                                if let Some(evidence) = evidence {
                                    evidence.record_network_event(
                                        "replay-requested",
                                        msg.resume_team_sequence,
                                        msg.authoritative_tick,
                                        "POST_REBASE",
                                    )?;
                                }
                                if let Err(error) = session
                                    .client
                                    .request_team_replay(
                                        *replay_request_id,
                                        msg.resume_team_sequence,
                                        view_epoch,
                                    )
                                    .await
                                {
                                    shutdown
                                        .cancel(ShutdownReason::UnsafeSession(error.to_string()));
                                } else {
                                    *replay_requested_from = Some(msg.resume_team_sequence);
                                }
                            }
                        }
                        Err(error) => {
                            shutdown.cancel(ShutdownReason::UnsafeSession(error.to_string()))
                        }
                    }
                    return Ok(());
                }
                Ok(LockstepInbound::OwnerWardEnvelope { msg }) => {
                    if let Err(error) = replica.apply_owner_ward_envelope(&msg) {
                        log::warn!("owner ward envelope rejected: {error}");
                        if let Some((recovery_request, recovery_epoch)) = replica.poll_owner_ward_recovery() {
                            if let Err(error) = session
                                .client
                                .query_owner_ward_recovery(recovery_request, recovery_epoch)
                                .await
                            {
                                log::warn!("owner ward recovery query failed: {error}");
                            }
                        }
                    }
                    continue;
                }
                Ok(LockstepInbound::ShopReceiptReplay { msg }) => {
                    match presentation.retain_shop_replay(replica.view_epoch(), &msg) {
                        Ok(terminal) => {
                            publish_shop_replay_outcome(presentation, pending_input_requests,
                                presentation_sequence, &msg).await?;
                            log::info!("shop receipt recovery input_id={} status={} terminal={}", msg.input_id, msg.status, terminal);
                        },
                        Err(code) => shutdown.cancel(ShutdownReason::UnsafeSession(code.into())),
                    }
                    continue;
                }
                Ok(LockstepInbound::SecureTargetInputResult { msg, .. }) => {
                    if let Some(evidence) = evidence {
                        evidence.record_network_event(
                            "secure-input-result",
                            0,
                            replica.next_replica_tick(),
                            if msg.accepted {
                                "SERVER_ACCEPTED"
                            } else {
                                "SERVER_INVALID_TARGET"
                            },
                        )?;
                    }
                    *presentation_sequence = presentation_sequence.saturating_add(1);
                    let renderer_request = secure_result_request_id(pending_input_requests, msg.request_id, msg.accepted);
                    presentation
                        .publish_critical(critical_result(
                            *presentation_sequence,
                            renderer_request,
                            u32::try_from(msg.request_id).unwrap_or(0),
                            msg.accepted,
                            if msg.accepted {
                                "SERVER_ACCEPTED"
                            } else {
                                "SERVER_INVALID_TARGET"
                            },
                            replica.next_replica_tick(),
                        ))
                        .await?;
                    return Ok(());
                }
                Ok(LockstepInbound::MatchTerminated { reason_class, last_safe_sequence }) => {
                    log::error!("match terminated by server: reason={reason_class} last_safe_sequence={last_safe_sequence}");
                    *presentation_sequence = presentation_sequence.saturating_add(1);
                    presentation.publish_critical(session_state_envelope(*presentation_sequence,
                        omoba_core::game_proto::runtime_session_state::State::Terminated, &format!("match_terminated:{reason_class}"))).await?;
                    shutdown.cancel(ShutdownReason::MatchTerminated(reason_class));
                    return Ok(());
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        let Some((ready_msg, ready_encoded)) =
            pending_frames.remove(&replica.expected_team_sequence())
        else {
            break;
        };
        *replay_requested_from = None;
        let ready_sequence = ready_msg.team_sequence;
        let ready_tick = ready_msg.replica_tick;
        let inbound_depth = session
            .inbound
            .len()
            .saturating_add(pending_frames.len());
        let catch_up_plan = replica_lag.plan_next_frame(inbound_depth);
        let apply_result = apply_ready_frame(
            config,
            session,
            replica,
            presentation,
            evidence,
            replica_steps,
            replica_stages,
            checkpoint_queue,
            catch_up_plan.publish_latest_snapshot,
            presentation_sequence,
            scripted_move_sent,
            next_scripted_move_tick,
            scripted_move_ordinal,
            scripted_stall_injected,
            scripted_move_origin,
            scripted_move_applied,
            scripted_hidden_target_sent,
            screenshot_marked,
            fault_injected,
            pending_input_requests,
            last_presentation_heroes,
            ready_msg,
            ready_encoded,
        )
        .await;
        if let Err(error) = apply_result {
            if let Some(evidence) = evidence {
                evidence.record_network_event(
                    "frame-rejected",
                    ready_sequence,
                    replica.next_replica_tick(),
                    "UNSAFE_FRAME",
                )?;
            }
            log::warn!("ordered team frame rejected at sequence {ready_sequence}; requesting authoritative filtered rebase: {error}");
            if let Some(evidence) = evidence {
                evidence.record_network_event(
                    "repair-requested",
                    ready_sequence,
                    ready_tick,
                    "AUTHORITATIVE_REBASE",
                )?;
            }
            session
                .client
                .report_team_hash_mismatch(&ClientTeamHashMismatch {
                    team_id: config.team_id,
                    frame_sequence: ready_sequence,
                    replica_tick: ready_tick,
                    received_hash: replica.current_team_hash().to_vec(),
                    view_epoch: Some(ViewEpoch {
                        value: replica.view_epoch(),
                    }),
                })
                .await?;
            *awaiting_authoritative_rebase = true;
            break;
        }
        let remaining_depth = session
            .inbound
            .len()
            .saturating_add(pending_frames.len());
        replica_lag.observe_applied(
            ready_tick,
            remaining_depth,
            checkpoint_queue.depth(),
            catch_up_plan.yield_after_frame,
        );
        if catch_up_plan.yield_after_frame {
            tokio::task::yield_now().await;
            if ReplicaLagTracker::should_pause_for_input(
                true,
                presentation.has_pending_input(),
            ) {
                break;
            }
        }
    }
    let expected = replica.expected_team_sequence();
    if !*awaiting_authoritative_rebase
        && pending_frames
            .first_key_value()
            .is_some_and(|(&sequence, _)| sequence > expected)
        && *replay_requested_from != Some(expected)
    {
        log::warn!("team frame gap; requesting replay from sequence {expected}");
        if let Some(evidence) = evidence {
            evidence.record_network_event(
                "replay-requested",
                expected,
                replica.next_replica_tick(),
                "SEQUENCE_GAP",
            )?;
        }
        *replay_request_id = replay_request_id.saturating_add(1);
        if let Err(error) = session
            .client
            .request_team_replay(*replay_request_id, expected, replica.view_epoch())
            .await
        {
            shutdown.cancel(ShutdownReason::UnsafeSession(error.to_string()));
        } else {
            *replay_requested_from = Some(expected);
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn apply_ready_frame(
    config: &ClientRuntimeConfig,
    session: &mut SelectiveSession,
    replica: &mut ReplicaHost,
    presentation: &mut PresentationHub,
    evidence: &Option<EvidenceRecorder>,
    replica_steps: &mut omoba_core::comp::perf_window::ReplicaStepWindow,
    replica_stages: &mut omoba_core::comp::replica_stage::ReplicaStageWindow,
    checkpoint_queue: &CheckpointQueue,
    publish_latest_snapshot: bool,
    presentation_sequence: &mut u64,
    scripted_move_sent: &mut bool,
    next_scripted_move_tick: &mut Option<u64>,
    scripted_move_ordinal: &mut u64,
    scripted_stall_injected: &mut bool,
    scripted_move_origin: &mut Option<(i64, i64)>,
    scripted_move_applied: &mut bool,
    scripted_hidden_target_sent: &mut bool,
    screenshot_marked: &mut bool,
    fault_injected: &mut bool,
    pending_input_requests: &mut BTreeMap<u32, u64>,
    last_presentation_heroes: &mut Vec<String>,
    msg: omoba_core::game_proto::TeamTickFrame,
    encoded: Arc<[u8]>,
) -> anyhow::Result<()> {
    let revision = msg
        .authority_revision
        .as_ref()
        .map_or(0, |value| value.value);
    if !*fault_injected
        && config.team_id == 1
        && config.test_mode
        && config
            .fault_tick
            .is_some_and(|tick| msg.replica_tick >= tick)
    {
        *fault_injected = replica.inject_test_only_fault();
        if *fault_injected {
            if let Some(evidence) = evidence {
                evidence.record_marker("team-1-fault", msg.replica_tick)?;
            }
        }
    }
    if !*scripted_stall_injected
        && config.test_mode
        && config
            .scripted_stall_tick
            .is_some_and(|tick| msg.replica_tick >= tick)
    {
        *scripted_stall_injected = true;
        let stall_ms = config.scripted_stall_ms.unwrap_or_default();
        log::warn!(
            "injecting test-only replica stall replica_tick={} stall_ms={}",
            msg.replica_tick,
            stall_ms,
        );
        tokio::time::sleep(std::time::Duration::from_millis(stall_ms)).await;
        if let Some(evidence) = evidence {
            evidence.record_marker("scripted-stall-complete", msg.replica_tick)?;
        }
    }
    let step_started = std::time::Instant::now();
    let applied = replica.apply_encoded_frame_profiled(&encoded, revision)?;
    let step_ns = step_started.elapsed().as_nanos();
    let Some(report) = applied else {
        let _ = replica.take_stage_profile();
        return Ok(());
    };
    // Stage formatting stays after the outer clock stops, and outside OM_PERF.
    if let Some(profile) = replica.take_stage_profile() {
        if let Ok(sample) = omoba_core::comp::replica_stage::ReplicaStageSample::from_durations(
            report.replica_tick,
            report.team_sequence,
            &profile,
            step_ns,
        ) {
            if let Ok(Some(lines)) = replica_stages.record_bundle(sample) {
                log::info!("{}", lines.stage);
                if let Some(detail) = lines.fixed_step { log::info!("{detail}"); }
                if let Some(dispatcher) = lines.dispatcher { log::info!("{dispatcher}"); }
            }
        }
    }
    // Evidence IO and presentation stay outside this sample. STEP_US_ below is unchanged.
    if let Some(line) = replica_steps.observe_completed(step_ns) {
        log::info!("{line}");
    }
    if let Some(evidence) = evidence {
        evidence.record_checkpoint(&report)?;
        if report.replica_tick % 120 == 0 {
            evidence.record_component_digests(
                report.replica_tick,
                &replica.disclosed_component_digests(),
            )?;
        }
        evidence.record_network_event(
            "frame-applied",
            report.team_sequence,
            report.replica_tick,
            "POST_REPAIR",
        )?;
        let timing = format!("STEP_US_{}", step_started.elapsed().as_micros());
        evidence.record_network_event(
            "replica-step-timing",
            report.team_sequence,
            report.replica_tick,
            &timing,
        )?;
    }
    if next_scripted_move_tick.is_some_and(|tick| report.replica_tick >= tick) {
        let scheduled_tick = next_scripted_move_tick.unwrap_or(report.replica_tick);
        *scripted_move_sent = true;
        if scripted_move_origin.is_none() {
            *scripted_move_origin = replica.owned_hero_position(config.player_id);
        }
        let toward_enemy_side = *scripted_move_ordinal % 2 == 0;
        let destination = if config.test_mode && std::env::var("OMOBA_RECALL_SMOKE").as_deref() == Ok("1") {
            let origin = scripted_move_origin.unwrap_or((0, 0));
            (origin.0 + if config.team_id == 1 { -300 * 1024 } else { 300 * 1024 }, 700 * 1024)
        } else if (config.team_id == 1) == toward_enemy_side {
            (900 * 1024, 700 * 1024)
        } else {
            (-900 * 1024, -700 * 1024)
        };
        presentation.inject_test_input(omoba_core::game_proto::RendererInput {
            request_id: 0xF000_0000 + u64::from(config.team_id),
            player_id: config.player_id,
            disclosure_epoch: replica.view_epoch(),
            intent: Some(omoba_core::game_proto::renderer_input::Intent::MoveTo(
                omoba_core::game_proto::MoveToIntent {
                    x_raw: destination.0,
                    y_raw: destination.1,
                    queued: false,
                },
            )),
        })?;
        if let Some(evidence) = evidence {
            evidence.record_marker("scripted-move", report.replica_tick)?;
        }
        *scripted_move_ordinal = scripted_move_ordinal.saturating_add(1);
        *next_scripted_move_tick = config
            .scripted_move_interval_ticks
            .map(|interval| scheduled_tick.saturating_add(interval));
    }
    if *scripted_move_sent {
        if let (Some(origin), Some(current)) = (
            *scripted_move_origin,
            replica.owned_hero_position(config.player_id),
        ) {
            if current != origin {
                if let Some(evidence) = evidence {
                    evidence.record_move_evidence(report.replica_tick, origin, current)?;
                    if !*scripted_move_applied {
                        evidence.record_marker("scripted-move-applied", report.replica_tick)?;
                    }
                }
                *scripted_move_applied = true;
            }
        }
    }
    if !*scripted_hidden_target_sent
        && config.test_mode
        && config
            .scripted_hidden_target_tick
            .is_some_and(|tick| report.replica_tick >= tick)
    {
        *scripted_hidden_target_sent = true;
        if let Some(actor) = replica.owned_hero_reference(config.player_id) {
            let invalid_target = omoba_core::game_proto::SecureReplicaTarget {
                replica_entity_id: Some(omoba_core::game_proto::ReplicaEntityId {
                    value: u64::MAX - u64::from(config.team_id),
                }),
                view_epoch: Some(omoba_core::game_proto::ViewEpoch {
                    value: replica.view_epoch(),
                }),
                disclosure_epoch: Some(omoba_core::game_proto::DisclosureEpoch { value: 1 }),
            };
            session
                .client
                .submit_secure_target(&omoba_core::game_proto::SecureTargetInput {
                    request_id: 0xE000_0000 + u64::from(config.team_id),
                    player_id: config.player_id,
                    input_tick: msg
                        .server_tick
                        .max(replica.next_replica_tick())
                        .saturating_add(u64::from(session.start.tick_rate_hz.max(1))),
                    actor: Some(actor),
                    target: Some(invalid_target),
                    action_kind: 0,
                    sanitized_payload: Vec::new(),
                })
                .await?;
            if let Some(evidence) = evidence {
                evidence.record_marker("scripted-hidden-target-submitted", report.replica_tick)?;
            }
        }
    }
    if !*screenshot_marked
        && config
            .screenshot_tick
            .is_some_and(|tick| report.replica_tick >= tick)
    {
        *screenshot_marked = true;
        if let Some(evidence) = evidence {
            evidence.record_marker("screenshot", report.replica_tick)?;
        }
    }
    let checkpoint = omoba_core::game_proto::ClientReplicaCheckpointReport {
        team_id: config.team_id,
        frame_sequence: report.team_sequence,
        replica_tick: report.replica_tick,
        authority_revision: Some(omoba_core::game_proto::AuthorityRevision {
            value: report.authority_revision,
        }),
        view_epoch: Some(omoba_core::game_proto::ViewEpoch {
            value: replica.view_epoch(),
        }),
        pre_repair_hash: report.pre_repair_hash.to_vec(),
        post_repair_hash: report.post_repair_hash.to_vec(),
        encoded_frame_hash: report.encoded_frame_hash.to_vec(),
    };
    checkpoint_queue.enqueue(checkpoint).await?;
    if let Some(expected) = msg
        .post_step
        .as_ref()
        .and_then(|post| post.hash_checkpoint.as_ref())
        .and_then(|hash| <[u8; 32]>::try_from(hash.canonical_team_hash.as_slice()).ok())
    {
        if expected != report.post_repair_hash {
            let mismatch = omoba_core::game_proto::ClientTeamHashMismatch {
                team_id: config.team_id,
                frame_sequence: report.team_sequence,
                replica_tick: report.replica_tick,
                received_hash: report.post_repair_hash.to_vec(),
                view_epoch: Some(omoba_core::game_proto::ViewEpoch {
                    value: replica.view_epoch(),
                }),
            };
            let _ = session.client.report_team_hash_mismatch(&mismatch).await;
        }
    }
    let divisor = (session.start.tick_rate_hz.max(1) / config.presentation_hz).max(1);
    if let Some(step) = &msg.step {
        presentation.retain_shop_receipts(replica.view_epoch(), msg.replica_tick, &step.public_events);
    }
    // Capture one-shots every applied step, before the presentation-rate divisor
    // and before catch-up coalesces the latest watch snapshot.
    let mut cues = replica.take_damage_presentation(report.replica_tick);
    cues.extend(replica.ability_presentation(report.replica_tick));
    presentation.retain_effects(replica.view_epoch(), report.replica_tick, cues);
    let mut settled_inputs = omoba_client_runtime::shop_presentation::settled_inputs(&msg, config.player_id);
    let rejections = replica.take_owner_command_rejections(config.player_id);
    omoba_client_runtime::shop_presentation::apply_owner_rejections(&mut settled_inputs, &rejections);
    let applied_local_inputs: Vec<_> = settled_inputs.iter().map(|(id, _, _)| *id).collect();
    if (report.team_sequence % u64::from(divisor) == 0 || !applied_local_inputs.is_empty()) && (presentation.presentation_enabled()
        || config.test_mode && (std::env::var("OMOBA_SHOP_TRANSACTION_SMOKE").as_deref() == Ok("1")
            || std::env::var("OMOBA_RECALL_SMOKE").as_deref() == Ok("1")
            || std::env::var("OMOBA_COMBAT_SMOKE").as_deref() == Ok("1")
            || std::env::var("OMOBA_FIRST_LEARN_SMOKE").as_deref() == Ok("1"))) {
        let mut snapshot = replica.extract_presentation_source();
        if config.test_mode && std::env::var("OMOBA_FIRST_LEARN_SMOKE").as_deref() == Ok("1") {
            presentation.inject_first_learn_smoke(&snapshot,replica.view_epoch())?;
        }
        if config.test_mode && std::env::var("OMOBA_COMBAT_SMOKE").as_deref() == Ok("1") {
            presentation.inject_combat_smoke(&snapshot, replica.view_epoch())?;
            if std::env::var("OMOBA_UPGRADE_SMOKE").as_deref() == Ok("1") {
                presentation.inject_upgrade_smoke(&snapshot, replica.view_epoch())?;
            }
        }
        if config.test_mode && std::env::var("OMOBA_RECALL_SMOKE").as_deref() == Ok("1") {
            presentation.inject_recall_smoke(&snapshot, replica.view_epoch(), config.scripted_move_tick.unwrap_or(360) + 120)?;
        }
        if config.test_mode && std::env::var("OMOBA_SHOP_TRANSACTION_SMOKE").as_deref() == Ok("1") {
            presentation.inject_shop_smoke(&snapshot, replica.view_epoch())?;
            if let Some(id) = presentation.take_shop_smoke_query() {
                session.client.query_shop_receipt(u64::from(id), id).await?;
            }
        }
        if let Some(evidence) = evidence {
            evidence.record_filtered_world(&snapshot)?;
        }
        let directives = std::mem::take(&mut snapshot.memory_directives);
        if presentation_trace_enabled() {
            let hero_identities = snapshot_hero_identity_labels(&snapshot);
            let heroes = snapshot_hero_labels(&snapshot);
            let transitions = directives
                .iter()
                .map(|directive| match directive {
                    omoba_core::runtime::RenderMemoryDirective::Hide {
                        replica_id,
                        disclosure_epoch,
                        ..
                    } => format!("Hide:{replica_id}@{disclosure_epoch}"),
                    omoba_core::runtime::RenderMemoryDirective::Forget {
                        replica_id,
                        disclosure_epoch,
                    } => format!("Forget:{replica_id}@{disclosure_epoch}"),
                })
                .collect::<Vec<_>>();
            if !transitions.is_empty() || hero_identities != *last_presentation_heroes {
                log::warn!(
                    "presentation_trace stage=runtime_emit authoritative_tick={} replica_tick={} snapshot_route={} lifecycle_route={} transitions={:?} heroes={:?}",
                    msg.server_tick,
                    snapshot.replica_tick,
                    if applied_local_inputs.is_empty() { "latest" } else { "critical" },
                    if transitions.is_empty() { "none" } else { "critical" },
                    transitions,
                    heroes,
                );
                *last_presentation_heroes = hero_identities;
            }
        }
        if let Some(envelope) = lifecycle_envelope(
            presentation_sequence.saturating_add(1),
            snapshot.team_id,
            msg.server_tick,
            snapshot.replica_tick,
            replica.view_epoch(),
            directives,
        ) {
            *presentation_sequence = presentation_sequence.saturating_add(1);
            if let Some(evidence) = evidence {
                evidence.record_presentation(&envelope)?;
            }
            presentation.publish_critical(envelope).await?;
        }
        // Catch-up discards intermediate latest state. Do not compute its large
        // fog projection only to throw it away: that work prolongs the backlog.
        // Lifecycle above and input-bearing snapshots remain on the critical path.
        if let Some(route) =
            presentation_snapshot_route(publish_latest_snapshot, &applied_local_inputs)
        {
            *presentation_sequence = presentation_sequence.saturating_add(1);
            for (input_id, accepted, result_code) in &settled_inputs {
                if *accepted && *result_code == "APPLIED_TO_PRESENTATION" {
                    if let Some(request_id) = pending_input_requests.get(input_id) {
                        if presentation.ward_authority_for(replica.view_epoch(), config.player_id, *request_id) == Some(*input_id) {
                            presentation.note_ward_request(
                                replica.view_epoch(),
                                config.player_id,
                                0,
                                *input_id,
                                *request_id,
                                omoba_core::runtime::ward_transport::WARD_PHASE_APPLIED,
                            );
                        }
                    }
                }
            }
            presentation.set_owner_ward_private(
                replica.view_epoch(),
                replica.owner_ward_protocol(),
                replica.verified_owner_ward_bytes(),
            );
            let envelope = presentation.snapshot_envelope(
                *presentation_sequence,
                msg.server_tick,
                replica.view_epoch(),
                snapshot,
                session.client.latest_rtt_us().unwrap_or(0),
            );
            if let Some(evidence) = evidence {
                evidence.record_presentation(&envelope)?;
            }
            match route {
                PresentationSnapshotRoute::Latest => presentation.publish_latest(envelope),
                // State and APPLIED result remain FIFO, not merely an ACK.
                PresentationSnapshotRoute::Critical => {
                    presentation.publish_critical(envelope).await?
                }
            }
        }
    }
    for (input_id, accepted, result_code) in settled_inputs {
        let Some(request_id) = pending_input_requests.remove(&input_id) else {
            continue;
        };
        *presentation_sequence = presentation_sequence.saturating_add(1);
        presentation
            .publish_critical(critical_result(
                *presentation_sequence,
                request_id,
                input_id,
                accepted,
                result_code,
                msg.server_tick,
            ))
            .await?;
    }
    Ok(())
}

fn presentation_trace_enabled() -> bool {
    std::env::var("OMOBA_PRESENTATION_TRACE")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

fn frame_transition_labels(frame: &omoba_core::game_proto::TeamTickFrame) -> Vec<String> {
    use omoba_core::game_proto::transition::Transition;
    frame
        .pre_step
        .as_ref()
        .into_iter()
        .flat_map(|step| &step.transitions)
        .filter_map(|transition| match transition.transition.as_ref()? {
            Transition::Reveal(value) => Some(format!(
                "Reveal:{}@{}",
                value.replica_entity_id.as_ref().map_or(0, |id| id.value),
                value
                    .disclosure_epoch
                    .as_ref()
                    .map_or(0, |epoch| epoch.value)
            )),
            Transition::Hide(value) => Some(format!(
                "Hide:{}@{}",
                value.replica_entity_id.as_ref().map_or(0, |id| id.value),
                value
                    .disclosure_epoch
                    .as_ref()
                    .map_or(0, |epoch| epoch.value)
            )),
            Transition::Forget(value) => Some(format!(
                "Forget:{}@{}",
                value.replica_entity_id.as_ref().map_or(0, |id| id.value),
                value
                    .disclosure_epoch
                    .as_ref()
                    .map_or(0, |epoch| epoch.value)
            )),
            Transition::Replace(value) => Some(format!(
                "Replace:{}@{}",
                value.replica_entity_id.as_ref().map_or(0, |id| id.value),
                value
                    .disclosure_epoch
                    .as_ref()
                    .map_or(0, |epoch| epoch.value)
            )),
        })
        .collect()
}

fn snapshot_hero_labels(snapshot: &omoba_core::runtime::FilteredRenderSnapshot) -> Vec<String> {
    snapshot
        .entities
        .iter()
        .filter_map(|entity| {
            let payload = entity
                .components
                .get(&omoba_core::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)?;
            let state = omoba_core::runtime::decode_demo_render_state(payload)?;
            (state.kind == 1).then(|| {
                format!(
                    "{}@{}:player{}:team{}:pos({}, {})",
                    entity.replica_id,
                    entity.disclosure_epoch,
                    state.owner_player_id,
                    state.team_id,
                    state.x_raw,
                    state.y_raw,
                )
            })
        })
        .collect()
}

fn snapshot_hero_identity_labels(
    snapshot: &omoba_core::runtime::FilteredRenderSnapshot,
) -> Vec<String> {
    snapshot
        .entities
        .iter()
        .filter_map(|entity| {
            let payload = entity
                .components
                .get(&omoba_core::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)?;
            let state = omoba_core::runtime::decode_demo_render_state(payload)?;
            (state.kind == 1).then(|| {
                format!(
                    "{}@{}:player{}:team{}",
                    entity.replica_id,
                    entity.disclosure_epoch,
                    state.owner_player_id,
                    state.team_id,
                )
            })
        })
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
enum PresentationSnapshotRoute {
    Latest,
    Critical,
}

fn presentation_snapshot_route(
    publish_latest: bool,
    applied_local_inputs: &[u32],
) -> Option<PresentationSnapshotRoute> {
    if !applied_local_inputs.is_empty() {
        Some(PresentationSnapshotRoute::Critical)
    } else if publish_latest {
        Some(PresentationSnapshotRoute::Latest)
    } else {
        None
    }
}

/// Terminal runtime state for the renderer: `safe_reason` is a bounded,
/// team-opaque class such as `observer-mismatch-after-rebase`.
fn session_state_envelope(sequence: u64, state: omoba_core::game_proto::runtime_session_state::State, safe_reason: &str) -> RendererIpcEnvelope {
    RendererIpcEnvelope {
        magic: PRESENTATION_MAGIC,
        protocol_version: PRESENTATION_PROTOCOL_VERSION,
        sequence,
        payload: Some(renderer_ipc_envelope::Payload::SessionState(omoba_core::game_proto::RuntimeSessionState {
            state: state as i32,
            safe_reason: safe_reason.chars().filter(char::is_ascii_graphic).take(96).collect(),
        })),
    }
}

fn critical_result(
    sequence: u64,
    request_id: u64,
    input_id: u32,
    accepted: bool,
    result_code: &str,
    authoritative_tick: u64,
) -> RendererIpcEnvelope {
    RendererIpcEnvelope {
        magic: PRESENTATION_MAGIC,
        protocol_version: PRESENTATION_PROTOCOL_VERSION,
        sequence,
        payload: Some(renderer_ipc_envelope::Payload::CriticalInputResult(
            CriticalInputResult {
                request_id,
                input_id,
                accepted,
                result_code: result_code.into(),
                authoritative_tick,
            },
        )),
    }
}

fn secure_result_request_id(
    pending: &mut BTreeMap<u32, u64>,
    server_request_id: u64,
    accepted: bool,
) -> u64 {
    let Ok(input_id) = u32::try_from(server_request_id) else { return server_request_id; };
    if accepted {
        pending.get(&input_id).copied().unwrap_or(server_request_id)
    } else {
        pending.remove(&input_id).unwrap_or(server_request_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catchup_skips_unpublishable_projection_but_never_applied_state() {
        assert_eq!(presentation_snapshot_route(false, &[]), None);
        assert_eq!(presentation_snapshot_route(true, &[]), Some(PresentationSnapshotRoute::Latest));
        assert_eq!(presentation_snapshot_route(false, &[7]), Some(PresentationSnapshotRoute::Critical));
        assert_eq!(presentation_snapshot_route(true, &[7]), Some(PresentationSnapshotRoute::Critical));
    }

    #[test]
    fn external_runtime_uses_low_latency_two_tick_input_lookahead() {
        assert_eq!(input_lookahead_ticks(), 2);
        assert_ne!(input_lookahead_ticks(), 120);
    }

    #[test]
    fn secure_result_retains_renderer_id_until_application_or_rejection() {
        let mut pending = BTreeMap::from([(3, 9001), (4, 9002)]);
        assert_eq!(secure_result_request_id(&mut pending, 3, true), 9001);
        assert_eq!(pending.get(&3), Some(&9001));
        assert_eq!(secure_result_request_id(&mut pending, 4, false), 9002);
        assert!(!pending.contains_key(&4));
        assert_eq!(secure_result_request_id(&mut pending, u64::MAX, false), u64::MAX);
        assert_eq!(pending.len(), 1);
    }
}
