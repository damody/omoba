use omoba_client_runtime::{
    config::ClientRuntimeConfig,
    evidence::EvidenceRecorder,
    input_bridge::{InputBridge, InputDecision},
    presentation_bridge::{
        ready_envelope, snapshot_envelope, PresentationHub, PRESENTATION_MAGIC,
        PRESENTATION_PROTOCOL_VERSION,
    },
    replica_host::ReplicaHost,
    session::SelectiveSession,
    shutdown::{ShutdownReason, ShutdownToken},
};
use omoba_core::{
    game_proto::{renderer_ipc_envelope, CriticalInputResult, RendererIpcEnvelope},
    kcp::client::LockstepInbound,
};
use prost::Message;
use std::{collections::BTreeMap, sync::Arc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
    let evidence = EvidenceRecorder::create(&config, replica.global_seed())?;
    let mut presentation = PresentationHub::bind(&config).await?;
    let mut input_bridge = InputBridge::default();
    let mut presentation_sequence = 1_u64;
    let mut scripted_move_sent = false;
    let mut scripted_move_origin = None;
    let mut scripted_move_applied = false;
    let mut scripted_hidden_target_sent = false;
    let mut screenshot_marked = false;
    let mut fault_injected = false;
    let mut pending_frames = BTreeMap::new();
    let mut replay_requested_from = None;
    let mut replay_request_id = 1_u64;
    presentation.publish_latest(ready_envelope(
        presentation_sequence,
        &config,
        session.start.server_tick,
        session.start.replica_start_tick,
    ));
    let (shutdown, mut shutdown_rx) = ShutdownToken::new();
    let ctrl_c = shutdown.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            ctrl_c.cancel(ShutdownReason::Requested);
        }
    });
    log::info!(
        "client-runtime ready player_id={} team_id={} replica_tick={} presentation={}",
        config.player_id,
        replica.team_id(),
        session.start.replica_start_tick,
        config.presentation_bind
    );
    loop {
        tokio::select! {
            changed = shutdown_rx.changed() => {
                if changed.is_err() || shutdown_rx.borrow().is_some() {
                    if let Some(evidence) = &evidence { evidence.record_network_event("session-stopped", replica.expected_team_sequence(), replica.next_replica_tick(), "SHUTDOWN")?; }
                    log::error!("client-runtime stopping: {:?}", shutdown_rx.borrow().clone());
                    break;
                }
            },
            inbound = session.inbound.recv() => match inbound {
                Some(LockstepInbound::TeamTickFrame { msg, encoded, .. }) => {
                    if let Some(evidence) = &evidence { evidence.record_network_event("frame-received", msg.team_sequence, msg.replica_tick, "OK")?; }
                    if msg.team_id != config.team_id {
                        if let Some(evidence) = &evidence { evidence.record_network_event("wrong-team-rejected", msg.team_sequence, msg.replica_tick, "WRONG_TEAM")?; }
                        shutdown.cancel(ShutdownReason::UnsafeSession("wrong team frame".into()));
                        continue;
                    }
                    if let Some(evidence) = &evidence { evidence.record_wire_frame(&encoded)?; }
                    if msg.team_sequence >= replica.expected_team_sequence() {
                        pending_frames.entry(msg.team_sequence).or_insert((msg, encoded));
                    } else if let Some(evidence) = &evidence {
                        evidence.record_network_event("duplicate-rejected", msg.team_sequence, msg.replica_tick, "DUPLICATE")?;
                    }
                    while let Some((ready_msg, ready_encoded)) = pending_frames.remove(&replica.expected_team_sequence()) {
                        replay_requested_from = None;
                        let ready_sequence = ready_msg.team_sequence;
                        if let Err(error) = apply_ready_frame(
                            &config, &mut session, &mut replica, &mut presentation, &evidence,
                            &shutdown, &mut presentation_sequence, &mut scripted_move_sent,
                            &mut scripted_move_origin, &mut scripted_move_applied,
                            &mut scripted_hidden_target_sent,
                            &mut screenshot_marked, &mut fault_injected, ready_msg, ready_encoded,
                        ).await {
                            if let Some(evidence) = &evidence { evidence.record_network_event("frame-rejected", ready_sequence, replica.next_replica_tick(), "UNSAFE_FRAME")?; }
                            log::error!("ordered team frame rejected at sequence {ready_sequence}: {error}");
                            shutdown.cancel(ShutdownReason::UnsafeSession(error.to_string()));
                            break;
                        }
                    }
                    let expected = replica.expected_team_sequence();
                    if pending_frames.first_key_value().is_some_and(|(&sequence, _)| sequence > expected)
                        && replay_requested_from != Some(expected)
                    {
                        log::warn!("team frame gap; requesting replay from sequence {expected}");
                        if let Some(evidence) = &evidence { evidence.record_network_event("replay-requested", expected, replica.next_replica_tick(), "SEQUENCE_GAP")?; }
                        replay_request_id = replay_request_id.saturating_add(1);
                        if let Err(error) = session.client.request_team_replay(replay_request_id, expected, replica.view_epoch()).await {
                            shutdown.cancel(ShutdownReason::UnsafeSession(error.to_string()));
                        } else {
                            replay_requested_from = Some(expected);
                        }
                    }
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
                            replay_requested_from = None;
                            let view_epoch = msg.view_epoch.as_ref().map_or(0, |value| value.value);
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
                Some(LockstepInbound::SecureTargetInputResult { msg, .. }) => {
                    if let Some(evidence) = &evidence { evidence.record_network_event("secure-input-result", 0, replica.next_replica_tick(), if msg.accepted { "SERVER_ACCEPTED" } else { "SERVER_INVALID_TARGET" })?; }
                    presentation_sequence = presentation_sequence.saturating_add(1);
                    presentation.publish_critical(critical_result(
                        presentation_sequence,
                        msg.request_id,
                        u32::try_from(msg.request_id).unwrap_or(0),
                        msg.accepted,
                        if msg.accepted { "SERVER_ACCEPTED" } else { "SERVER_INVALID_TARGET" },
                        replica.next_replica_tick(),
                    )).await?;
                }
                Some(_) => {}
                None => shutdown.cancel(ShutdownReason::ServerDisconnected),
            },
            renderer_input = presentation.recv_input() => {
                let Some(renderer_input) = renderer_input else { continue; };
                let request_id = renderer_input.request_id;
                match input_bridge.validate(renderer_input, config.player_id, &replica) {
                    InputDecision::Accepted { input_id, input, secure_target } => {
                        let network_lead_ticks = u64::from(session.start.tick_rate_hz.max(1));
                        let target_tick = u32::try_from(replica.next_replica_tick().saturating_add(network_lead_ticks)).unwrap_or(u32::MAX);
                        let result = if let Some(target) = secure_target {
                            let Some(actor) = replica.owned_hero_reference(config.player_id) else {
                                presentation_sequence = presentation_sequence.saturating_add(1);
                                presentation.publish_critical(critical_result(presentation_sequence, request_id, input_id, false, "OWN_HERO_NOT_DISCLOSED", u64::from(target_tick))).await?;
                                continue;
                            };
                            session.client.submit_secure_target(&omoba_core::game_proto::SecureTargetInput {
                                request_id: u64::from(input_id), player_id: config.player_id,
                                input_tick: u64::from(target_tick), actor: Some(actor), target: Some(target),
                                action_kind: 0, sanitized_payload: input.encode_to_vec(),
                            }).await.map(|_| (0, 0))
                        } else {
                            session.client.submit_input(target_tick, input, input_id).await
                        };
                        if let Some(evidence) = &evidence { evidence.record_network_event("input-forwarded", 0, u64::from(target_tick), if result.is_ok() { "FORWARDED" } else { "SERVER_TRANSPORT_ERROR" })?; }
                        presentation_sequence = presentation_sequence.saturating_add(1);
                        presentation.publish_critical(critical_result(
                            presentation_sequence, request_id, input_id, result.is_ok(),
                            if result.is_ok() { "FORWARDED" } else { "SERVER_TRANSPORT_ERROR" },
                            u64::from(target_tick),
                        )).await?;
                    }
                    InputDecision::Rejected { request_id, code } => {
                        if let Some(evidence) = &evidence { evidence.record_network_event("input-local-rejection", 0, replica.next_replica_tick(), code)?; }
                        presentation_sequence = presentation_sequence.saturating_add(1);
                        presentation.publish_critical(critical_result(
                            presentation_sequence, request_id, 0, false, code, replica.next_replica_tick(),
                        )).await?;
                    }
                }
            },
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
    shutdown: &ShutdownToken,
    presentation_sequence: &mut u64,
    scripted_move_sent: &mut bool,
    scripted_move_origin: &mut Option<(i64, i64)>,
    scripted_move_applied: &mut bool,
    scripted_hidden_target_sent: &mut bool,
    screenshot_marked: &mut bool,
    fault_injected: &mut bool,
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
    let step_started = std::time::Instant::now();
    let Some(report) = replica.apply_encoded_frame(&encoded, revision)? else {
        return Ok(());
    };
    if let Some(evidence) = evidence {
        evidence.record_checkpoint(&report)?;
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
    if !*scripted_move_sent
        && config
            .scripted_move_tick
            .is_some_and(|tick| report.replica_tick >= tick)
    {
        *scripted_move_sent = true;
        *scripted_move_origin = replica.owned_hero_position(config.player_id);
        let destination = if config.team_id == 1 {
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
                },
            )),
        })?;
        if let Some(evidence) = evidence {
            evidence.record_marker("scripted-move", report.replica_tick)?;
        }
    }
    if *scripted_move_sent && !*scripted_move_applied {
        if let (Some(origin), Some(current)) = (
            *scripted_move_origin,
            replica.owned_hero_position(config.player_id),
        ) {
            if current != origin {
                *scripted_move_applied = true;
                if let Some(evidence) = evidence {
                    evidence.record_marker("scripted-move-applied", report.replica_tick)?;
                }
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
                    input_tick: replica.next_replica_tick().saturating_add(3),
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
    };
    if let Err(error) = session.client.report_replica_checkpoint(&checkpoint).await {
        shutdown.cancel(ShutdownReason::UnsafeSession(error.to_string()));
        return Ok(());
    }
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
    if report.team_sequence % u64::from(divisor) == 0 && presentation.presentation_enabled() {
        *presentation_sequence = presentation_sequence.saturating_add(1);
        let snapshot = replica.extract_presentation_source();
        if let Some(evidence) = evidence {
            evidence.record_filtered_world(&snapshot)?;
        }
        let envelope = snapshot_envelope(
            *presentation_sequence,
            msg.server_tick,
            replica.view_epoch(),
            snapshot,
        );
        if let Some(evidence) = evidence {
            evidence.record_presentation(&envelope)?;
        }
        presentation.publish_latest(envelope);
    }
    Ok(())
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
