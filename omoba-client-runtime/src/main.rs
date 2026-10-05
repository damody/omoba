use omoba_client_runtime::{
    catchup::{ingest_pending_frame, ReplicaLagTracker},
    checkpoint_writer::{spawn_checkpoint_writer, CheckpointQueue},
    config::ClientRuntimeConfig,
    evidence::EvidenceRecorder,
    input_bridge::{InputBridge, InputDecision},
    presentation_bridge::{
        lifecycle_envelope, ready_envelope_for_start, reset_view_envelope,
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
    let mut session = SelectiveSession::connect(&config).await?;
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
    let evidence = EvidenceRecorder::create(&config, replica.global_seed())?;
    let mut presentation = PresentationHub::bind(&config).await?;
    log::info!(
        "client-runtime presentation listening on {}",
        config.presentation_bind
    );
    let mut input_bridge = InputBridge::resume_after(session.start.last_seen_input_id);
    input_bridge.set_shop_enabled(session.start.shop_protocol_version == omoba_core::runtime::shop_transport::SHOP_PROTOCOL_VERSION);
    presentation.set_shop_protocol_enabled(session.start.shop_protocol_version == omoba_core::runtime::shop_transport::SHOP_PROTOCOL_VERSION);
    let recall_enabled = session.start.recall_protocol_version == omoba_core::runtime::recall_transport::RECALL_PROTOCOL_VERSION;
    input_bridge.set_recall_enabled(recall_enabled);
    input_bridge.set_commands_enabled(omoba_core::runtime::command_transport::negotiate_command_protocol(
        session.start.command_protocol_version, &session.start.command_rules_hash)
        .map_err(|reason| anyhow::anyhow!("command capability: {reason}"))?);
    presentation.set_recall_protocol_enabled(recall_enabled);
    if config.test_mode && std::env::var("OMOBA_SHOP_QUERY_SMOKE").as_deref() == Ok("1") {
        // No transaction fixture: query a missing shop result only.
        session.client.query_shop_receipt(0x5348_4f50, u32::MAX).await?;
    }
    log::info!("input allocator resumed player={} after_id={}",
        config.player_id, session.start.last_seen_input_id);
    let mut presentation_sequence = 1_u64;
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
    let mut replay_requested_from = None;
    let mut replay_request_id = 1_u64;
    let mut awaiting_authoritative_rebase = false;
    let mut latest_server_tick = session.start.server_tick;
    let mut replica_lag = ReplicaLagTracker::new(session.start.replica_start_tick);
    let mut last_presentation_heroes = Vec::<String>::new();
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
    let (shutdown, mut shutdown_rx) = ShutdownToken::new();
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
    log::info!(
        "client-runtime ready player_id={} team_id={} replica_tick={} presentation={}",
        config.player_id,
        replica.team_id(),
        session.start.replica_start_tick,
        config.presentation_bind
    );
    let mut shop_query_interval = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        tokio::select! {
            biased;
            _ = shop_query_interval.tick() => {
                for id in presentation.due_shop_queries() {
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
                    &mut latest_server_tick,
                    renderer_input,
                ).await?;
                catch_up_available_frames(
                    &config,
                    &mut session,
                    &mut replica,
                    &mut presentation,
                    &evidence,
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
                Some(LockstepInbound::TeamTickFrame { msg, encoded, .. }) => {
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
                Some(_) => {}
                None => shutdown.cancel(ShutdownReason::ServerDisconnected),
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
    Ok(())
}

async fn publish_rebased_presentation(
    presentation: &mut PresentationHub,
    replica: &mut ReplicaHost,
    sequence: &mut u64,
    authoritative_tick: u64,
    rtt_us: u64,
) -> anyhow::Result<()> {
    *sequence = sequence.saturating_add(1);
    presentation.publish_critical(reset_view_envelope(*sequence, replica.team_id(),
        authoritative_tick, replica.next_replica_tick(), replica.view_epoch())).await?;
    *sequence = sequence.saturating_add(1);
    let source = replica.extract_presentation_source();
    let snapshot = presentation.snapshot_envelope(*sequence, authoritative_tick,
        replica.view_epoch(), source, rtt_us);
    // Ordered live recovery and retained baseline for a renderer reconnect.
    presentation.publish_critical(snapshot.clone()).await?;
    presentation.publish_latest(snapshot);
    Ok(())
}

async fn publish_shop_replay_outcome(
    presentation: &mut PresentationHub,
    pending: &mut BTreeMap<u32, u64>,
    sequence: &mut u64,
    reply: &omoba_core::game_proto::ShopReceiptReplay,
) -> anyhow::Result<()> {
    let (accepted, code, tick) = match reply.status {
        2 => {
            let receipt = omoba_core::runtime::shop_receipt::ShopReceipt::decode(&reply.receipt)
                .ok_or_else(|| anyhow::anyhow!("invalid validated shop receipt"))?;
            (receipt.result_code == 0, omoba_client_runtime::shop_presentation::receipt_result_name(receipt.result_code), receipt.tick)
        }
        3 => (false, "SHOP_RESULT_EXPIRED_UNCERTAIN", 0),
        4 => (false, "SHOP_ADMISSION_REJECTED_LATE", 0),
        _ => return Ok(()),
    };
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
    latest_server_tick: &mut u64,
    renderer_input: RendererInput,
) -> anyhow::Result<()> {
    let request_id = renderer_input.request_id;
    let is_shop = matches!(renderer_input.intent, Some(omoba_core::game_proto::renderer_input::Intent::ItemBuy(_)) | Some(omoba_core::game_proto::renderer_input::Intent::ItemSell(_)));
    if is_shop && !presentation.can_track_shop() {
        *presentation_sequence = presentation_sequence.saturating_add(1);
        presentation.publish_critical(critical_result(*presentation_sequence, request_id, 0, false, "SHOP_PENDING_CAPACITY", replica.next_replica_tick())).await?;
        return Ok(());
    }
    match input_bridge.validate(renderer_input, config.player_id, replica) {
        InputDecision::RepeatedShop { request_id, input_id } => {
            presentation.track_shop(input_id);
            pending_input_requests.insert(input_id, request_id);
            *presentation_sequence = presentation_sequence.saturating_add(1);
            presentation.publish_critical(critical_result(*presentation_sequence, request_id, input_id,
                true, "SHOP_RESULT_QUERY_ONLY", replica.next_replica_tick())).await?;
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
                presentation.track_shop(input_id);
                pending_input_requests.insert(input_id, request_id);
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
                    } else if is_shop {
                        "SHOP_TRANSPORT_UNCERTAIN"
                    } else {
                        "SERVER_TRANSPORT_ERROR"
                    },
                )?;
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
                    } else if is_shop {
                        "SHOP_TRANSPORT_UNCERTAIN"
                    } else {
                        "SERVER_TRANSPORT_ERROR"
                    },
                    u64::from(target_tick),
                ))
                .await?;
        }
        InputDecision::Rejected { request_id, code } => {
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
                Ok(LockstepInbound::TeamTickFrame { msg, encoded, .. }) => {
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
    let Some(report) = replica.apply_encoded_frame(&encoded, revision)? else {
        return Ok(());
    };
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
    let settled_inputs = omoba_client_runtime::shop_presentation::settled_inputs(&msg, config.player_id);
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
