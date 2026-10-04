use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Instant,
};

use prost::Message;
use serde::Serialize;
use sha2::{Digest, Sha256};

use omoba_core::{game_proto::RendererIpcEnvelope, runtime::FilteredRenderSnapshot};

use crate::{config::ClientRuntimeConfig, replica_host::ReplicaApplyReport, ClientRuntimeError};

#[derive(Debug)]
pub struct EvidenceRecorder {
    root: PathBuf,
    team_id: u32,
    started: Instant,
    frame_capture: Mutex<std::fs::File>,
    checkpoint_timeline: Mutex<std::fs::File>,
    presentation_capture: Mutex<std::fs::File>,
    network_events: Mutex<std::fs::File>,
}

#[derive(Serialize)]
struct RuntimeManifest<'a> {
    schema_version: u32,
    process_role: &'static str,
    pid: u32,
    player_id: u32,
    team_id: u32,
    server_addr: String,
    presentation_addr: String,
    content_hash: &'a str,
    global_seed_sha256: String,
    executable_path: String,
    executable_sha256: String,
}

#[derive(Serialize)]
struct CheckpointLine {
    team_id: u32,
    replica_tick: u64,
    team_sequence: u64,
    authority_revision: u64,
    pre_repair_hash: String,
    post_repair_hash: String,
}

impl EvidenceRecorder {
    pub fn create(
        config: &ClientRuntimeConfig,
        global_seed: u64,
    ) -> Result<Option<Self>, ClientRuntimeError> {
        if !config.test_mode {
            return Ok(None);
        }
        let Some(base) = config.evidence_dir.as_ref() else {
            return Err(ClientRuntimeError::Config(
                "test mode requires evidence directory".into(),
            ));
        };
        let root = base.join(format!("team-{}-runtime", config.team_id));
        fs::create_dir_all(&root).map_err(io_error)?;
        let executable = std::env::current_exe().map_err(io_error)?;
        let executable_bytes = fs::read(&executable).map_err(io_error)?;
        let manifest = RuntimeManifest {
            schema_version: 1,
            process_role: "external-team-replica-runtime",
            pid: std::process::id(),
            player_id: config.player_id,
            team_id: config.team_id,
            server_addr: config.server_addr.to_string(),
            presentation_addr: config.presentation_bind.to_string(),
            content_hash: &config.content_hash,
            global_seed_sha256: hex_hash(&global_seed.to_be_bytes()),
            executable_path: executable.display().to_string(),
            executable_sha256: hex_hash(&executable_bytes),
        };
        write_json(root.join("manifest.json"), &manifest)?;
        let frame_capture = Mutex::new(append_file(&root.join("team-frame.capture"))?);
        let checkpoint_timeline = Mutex::new(append_file(&root.join("filtered-timeline.jsonl"))?);
        let presentation_capture = Mutex::new(append_file(&root.join("presentation.capture"))?);
        let network_events = Mutex::new(append_file(&root.join("network-events.jsonl"))?);
        Ok(Some(Self {
            root,
            team_id: config.team_id,
            started: Instant::now(),
            frame_capture,
            checkpoint_timeline,
            presentation_capture,
            network_events,
        }))
    }

    pub fn record_wire_frame(&self, bytes: &[u8]) -> Result<(), ClientRuntimeError> {
        let mut file = lock_file(&self.frame_capture)?;
        write_framed(&mut file, bytes)
    }

    pub fn record_checkpoint(&self, report: &ReplicaApplyReport) -> Result<(), ClientRuntimeError> {
        let line = CheckpointLine {
            team_id: self.team_id,
            replica_tick: report.replica_tick,
            team_sequence: report.team_sequence,
            authority_revision: report.authority_revision,
            pre_repair_hash: hex::encode(report.pre_repair_hash),
            post_repair_hash: hex::encode(report.post_repair_hash),
        };
        let mut file = lock_file(&self.checkpoint_timeline)?;
        write_json_line(&mut file, &line)
    }

    pub fn record_component_digests(
        &self,
        replica_tick: u64,
        rows: &[omoba_core::runtime::DisclosedComponentDigest],
    ) -> Result<(), ClientRuntimeError> {
        write_json(
            self.root.join(format!("components-{replica_tick}.json")),
            rows,
        )
    }

    pub fn record_filtered_world(
        &self,
        snapshot: &FilteredRenderSnapshot,
    ) -> Result<(), ClientRuntimeError> {
        #[derive(Serialize)]
        struct SafeWorld<'a> {
            team_id: u32,
            replica_tick: u64,
            render_ids: Vec<u64>,
            component_schema_ids: Vec<Vec<u32>>,
            #[serde(skip)]
            _source: &'a FilteredRenderSnapshot,
        }
        let value = SafeWorld {
            team_id: snapshot.team_id,
            replica_tick: snapshot.replica_tick,
            render_ids: snapshot.entities.iter().map(|e| e.replica_id).collect(),
            component_schema_ids: snapshot
                .entities
                .iter()
                .map(|e| e.components.keys().copied().collect())
                .collect(),
            _source: snapshot,
        };
        write_json(self.root.join("filtered-world.latest.json"), &value)
    }

    pub fn record_presentation(
        &self,
        envelope: &RendererIpcEnvelope,
    ) -> Result<(), ClientRuntimeError> {
        let mut file = lock_file(&self.presentation_capture)?;
        write_framed(&mut file, &envelope.encode_to_vec())
    }
    pub fn record_marker(&self, name: &str, tick: u64) -> Result<(), ClientRuntimeError> {
        fs::write(self.root.join(format!("{name}.tick")), tick.to_string()).map_err(io_error)
    }
    pub fn record_move_evidence(
        &self,
        replica_tick: u64,
        origin: (i64, i64),
        current: (i64, i64),
    ) -> Result<(), ClientRuntimeError> {
        #[derive(Serialize)]
        struct MoveEvidence {
            replica_tick: u64,
            origin_x_raw: i64,
            origin_y_raw: i64,
            current_x_raw: i64,
            current_y_raw: i64,
        }
        write_json(
            self.root.join("scripted-move-evidence.json"),
            &MoveEvidence {
                replica_tick,
                origin_x_raw: origin.0,
                origin_y_raw: origin.1,
                current_x_raw: current.0,
                current_y_raw: current.1,
            },
        )
    }
    pub fn record_network_event(
        &self,
        kind: &str,
        team_sequence: u64,
        replica_tick: u64,
        code: &str,
    ) -> Result<(), ClientRuntimeError> {
        #[derive(Serialize)]
        struct Event<'a> {
            kind: &'a str,
            team_sequence: u64,
            replica_tick: u64,
            code: &'a str,
            monotonic_us: u64,
        }
        let mut file = lock_file(&self.network_events)?;
        write_json_line(
            &mut file,
            &Event {
                kind,
                team_sequence,
                replica_tick,
                code,
                monotonic_us: self.started.elapsed().as_micros() as u64,
            },
        )
    }
}

fn write_json(path: PathBuf, value: &(impl Serialize + ?Sized)) -> Result<(), ClientRuntimeError> {
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|e| ClientRuntimeError::Ipc(e.to_string()))?;
    fs::write(path, bytes).map_err(io_error)
}

fn write_json_line(
    file: &mut std::fs::File,
    value: &impl Serialize,
) -> Result<(), ClientRuntimeError> {
    serde_json::to_writer(&mut *file, value).map_err(|e| ClientRuntimeError::Ipc(e.to_string()))?;
    file.write_all(b"\n").map_err(io_error)
}

fn write_framed(file: &mut std::fs::File, bytes: &[u8]) -> Result<(), ClientRuntimeError> {
    file.write_all(&(bytes.len() as u32).to_be_bytes())
        .map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)
}

fn lock_file(
    file: &Mutex<std::fs::File>,
) -> Result<std::sync::MutexGuard<'_, std::fs::File>, ClientRuntimeError> {
    file.lock()
        .map_err(|_| ClientRuntimeError::Ipc("evidence file lock poisoned".into()))
}

fn append_file(path: &Path) -> Result<std::fs::File, ClientRuntimeError> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(io_error)
}

fn hex_hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn io_error(error: std::io::Error) -> ClientRuntimeError {
    ClientRuntimeError::Ipc(error.to_string())
}

#[cfg(test)]
mod income_capture_tests {
    use super::*;

    #[test]
    #[ignore = "requires OMOBA_UE_SCOREBOARD_CAPTURE_ROOT from a completed dual Unreal scoreboard smoke"]
    fn real_unreal_scoreboard_capture() {
        use omoba_core::{game_proto::{renderer_ipc_envelope::Payload, TeamTickFrame}, runtime::FilteredRenderSnapshot};
        let root = PathBuf::from(std::env::var("OMOBA_UE_SCOREBOARD_CAPTURE_ROOT").unwrap());
        let report: serde_json::Value = serde_json::from_slice(&fs::read(root.join("unreal-ipc-smoke-report.json")).unwrap()).unwrap();
        assert_eq!(report["success"], true);
        assert_eq!(report["cleanup_verified"], true);
        assert_eq!(report["scoreboard_smoke"], true);
        assert_eq!(report["tick_rate_hz"], 60);
        fn frames<T: prost::Message + Default>(path: PathBuf) -> Vec<T> {
            let bytes = fs::read(path).unwrap();
            let mut cursor = 0;
            let mut result = Vec::new();
            while cursor < bytes.len() {
                assert!(cursor + 4 <= bytes.len());
                let size = u32::from_be_bytes(bytes[cursor..cursor+4].try_into().unwrap()) as usize;
                cursor += 4;
                assert!(size > 0 && cursor + size <= bytes.len());
                result.push(T::decode(&bytes[cursor..cursor+size]).unwrap());
                cursor += size;
            }
            result
        }
        let mut shared = std::collections::BTreeMap::new();
        for team in 1..=2u32 {
            let folder = root.join(format!("team-{team}-runtime"));
            let mut authority = std::collections::BTreeMap::new();
            for frame in frames::<TeamTickFrame>(folder.join("team-frame.capture")) {
                assert_eq!(frame.team_id, team);
                let board = crate::scoreboard::project(&FilteredRenderSnapshot {
                    team_id: team, replica_tick: frame.replica_tick, entities: vec![],
                    public_events: frame.step.unwrap().public_events, external_effects: vec![],
                    memory_directives: vec![], remembered_presentations: Default::default(),
                }).expect("complete authority board");
                assert_eq!(board.rows.len(), 2);
                if let Some(other) = shared.insert(frame.replica_tick, board.clone()) { assert_eq!(other, board); }
                authority.insert(frame.replica_tick, board);
            }
            let ui = &report["teams"][team as usize - 1]["scoreboard_ui"];
            assert_eq!(ui["complete"], true);
            let ui_tick = ui["tick"].as_u64().unwrap();
            let dead_ui = (report["scoreboard_death_smoke"] == true)
                .then(|| &report["teams"][team as usize - 1]["scoreboard_dead_ui"]);
            let dead_tick = dead_ui.map(|ui| {
                assert_eq!(ui["complete"], true);
                let tick = ui["tick"].as_u64().unwrap();
                assert!(tick > ui_tick);
                tick
            });
            let mut count = 0;
            let mut matched_ui = false;
            let mut matched_dead = false;
            let mut finished_with_score = false;
            for envelope in frames::<RendererIpcEnvelope>(folder.join("presentation.capture")) {
                let Some(Payload::Snapshot(snapshot)) = envelope.payload else { continue };
                assert_eq!(snapshot.team_id, team);
                let hud = snapshot.moba_hud.expect("MOBA HUD");
                assert_eq!(hud.player_id, team);
                let board = hud.scoreboard.expect("IPC scoreboard");
                assert_eq!(&board, authority.get(&snapshot.replica_tick.checked_sub(1).unwrap()).expect("exact raw tick"));
                let shown_ui = if snapshot.replica_tick == ui_tick { Some(ui) }
                    else if dead_tick == Some(snapshot.replica_tick) { dead_ui } else { None };
                if let Some(shown_ui) = shown_ui {
                    let rows = shown_ui["rows"].as_array().unwrap();
                    assert_eq!(rows.len(), board.rows.len());
                    for (raw, shown) in board.rows.iter().zip(rows) {
                        assert_eq!(shown["player_id"], raw.player_id);
                        assert_eq!(shown["team_id"], raw.team_id);
                        assert_eq!(shown["kills"], raw.kills);
                        assert_eq!(shown["deaths"], raw.deaths);
                        assert_eq!(shown["assists"], raw.assists);
                    }
                    if snapshot.replica_tick == ui_tick { matched_ui = true; }
                    else {
                        assert_eq!(hud.phase, 1);
                        assert!(hud.hero.is_none(), "dead UI must have no live hero");
                        assert!(board.rows.iter().any(|r| r.player_id == team && r.deaths > 0));
                        matched_dead = true;
                    }
                }
                if hud.phase == 2 && dead_ui.is_some() {
                    assert!(board.rows.iter().any(|r| r.player_id == team && r.deaths > 0));
                    finished_with_score = true;
                }
                count += 1;
            }
            assert!(count > 10 && matched_ui);
            if dead_ui.is_some() { assert!(matched_dead && finished_with_score); }
            println!("real Unreal scoreboard team={team} snapshots={count} exact_ui_tick={ui_tick}");
        }
    }

    #[test]
    #[ignore = "requires OMOBA_FIRST_LEARN_CAPTURE_ROOT from a completed real two-runtime first-learning smoke"]
    fn real_first_learning_capture_matches_authority_and_private_input_contract() {
        use omoba_core::game_proto::{renderer_ipc_envelope::Payload,TeamTickFrame,player_input::Action,PlayerInput};
        fn frames<T:prost::Message+Default>(path:PathBuf)->Vec<T> {
            let bytes=fs::read(path).unwrap(); let mut cursor=0; let mut result=Vec::new();
            while cursor<bytes.len() {
                assert!(cursor+4<=bytes.len(),"truncated prefix");
                let len=u32::from_be_bytes(bytes[cursor..cursor+4].try_into().unwrap()) as usize; cursor+=4;
                assert!(len>0 && len<=8*1024*1024 && cursor+len<=bytes.len(),"invalid capture body");
                result.push(T::decode(&bytes[cursor..cursor+len]).unwrap()); cursor+=len;
            }
            result
        }
        let root=PathBuf::from(std::env::var("OMOBA_FIRST_LEARN_CAPTURE_ROOT").unwrap());
        let report:serde_json::Value=serde_json::from_slice(&fs::read(root.join("moba-runtime-smoke-report.json")).unwrap()).unwrap();
        assert_eq!(report["success"],true); assert_eq!(report["cleanup_verified"],true);
        assert_eq!(report["tick_rate_hz"],60); assert_eq!(report["first_learning"]["contract"],1);
        assert_eq!(report["first_learning"]["injected_gameplay_state"],false);
        let server=fs::read_to_string(root.join("logs/server.stderr.log")).unwrap()
            + &fs::read_to_string(root.join("logs/server.stdout.log")).unwrap();
        for player in 1..=2u32 {
            assert!(server.contains(&format!("not learned (pid={player})")));
            assert!(server.contains(&format!("AbilityUpgrade failed pid={player} ability_index=2: AbilityUpgrade: requires hero level 6")));
            let folder=root.join(format!("team-{player}-runtime"));
            let mut ranks=std::collections::BTreeMap::new(); let mut progression=std::collections::BTreeMap::new();
            let mut upgrades=Vec::new(); let mut casts=Vec::new();
            for frame in frames::<TeamTickFrame>(folder.join("team-frame.capture")) {
                assert_eq!(frame.team_id,player);
                let step=frame.step.unwrap();
                for input in step.accepted_inputs {
                    assert_eq!(input.player_id,player,"opponent private input leaked");
                    match PlayerInput::decode(input.sanitized_payload.as_slice()).unwrap().action {
                        Some(Action::UpgradeAbility(value))=>upgrades.push((value.ability_index,input.input_id)),
                        Some(Action::CastAbility(value))=>casts.push((value.ability_index,input.input_id)),
                        _=>{}
                    }
                }
                for event in step.public_events {
                    let map=if event.event_kind==omoba_core::runtime::FactKind::CommittedAbilityRanks as u32 {&mut ranks}
                        else if event.event_kind==omoba_core::runtime::FactKind::CommittedProgression as u32 {&mut progression}
                        else {continue};
                    assert_eq!(event.sanitized_payload.len(),16);
                    let values:Vec<_>=event.sanitized_payload.chunks_exact(4).map(|b|i32::from_le_bytes(b.try_into().unwrap())).collect();
                    map.insert((frame.replica_tick,event.subject.unwrap().value),values);
                }
            }
            assert_eq!(upgrades,vec![(2,2),(3,3)],"duplicate renderer request must not allocate another input");
            assert_eq!(casts,vec![(3,1),(3,4)]);
            let mut count=0; let mut unlearned=false; let mut learned=false; let mut cast=false;
            for envelope in frames::<RendererIpcEnvelope>(folder.join("presentation.capture")) {
                let Some(Payload::Snapshot(snapshot))=envelope.payload else {continue};
                assert_eq!(snapshot.team_id,player); let hud=snapshot.moba_hud.unwrap(); assert_eq!(hud.player_id,player);
                let Some(hero)=hud.hero else {continue}; if hud.phase!=1 {continue;}
                let key=(snapshot.replica_tick.checked_sub(1).unwrap(),hero.render_id);
                let raw=ranks.get(&key).expect("exact raw rank tick"); let progress=progression.get(&key).expect("exact raw progression tick");
                assert_eq!((hero.level,hero.skill_points),(progress[0] as u32,progress[3] as u32));
                assert_eq!(hero.abilities.len(),4);
                for (slot,ability) in hero.abilities.iter().enumerate() {
                    assert_eq!(ability.level,raw[slot] as u32);
                    assert!(if slot==3 {ability.level<=1} else {ability.level==0});
                }
                let rank=hero.abilities[3].level;
                assert_eq!(hero.skill_points+rank,hero.level,"one birth point plus earned level points");
                if rank==0 {assert_eq!(hero.abilities[3].cooldown_raw,0);unlearned=true;}
                if rank==1 {learned=true;if hero.abilities[3].cooldown_raw>0 {cast=true;}}
                count+=1;
            }
            assert!(count>100 && unlearned && learned && cast);
            assert!(report["teams"][player as usize-1]["post_learning_checkpoints"].as_u64().unwrap()>=2);
            println!("real first learning player={player} snapshots={count} upgrade_input=3 exact_rank_sp=true");
        }
    }

    #[test]
    #[ignore = "requires OMOBA_SCORE_CAPTURE_ROOT from a completed real three-runtime roster smoke"]
    fn real_three_player_owner_score_capture() {
        use omoba_core::game_proto::renderer_ipc_envelope::Payload;
        let root = PathBuf::from(std::env::var("OMOBA_SCORE_CAPTURE_ROOT").unwrap());
        let report: serde_json::Value = serde_json::from_slice(&fs::read(root.join("moba-runtime-smoke-report.json")).unwrap()).unwrap();
        assert_eq!(report["success"],true);
        assert_eq!(report["cleanup_verified"],true);
        let combat = report.get("combat");
        let require_board = combat.is_some_and(|c| c["scoreboard_contract"]==1);
        let require_xp = combat.is_some_and(|c| c["xp_contract"]==1 || c["xp_contract"]==2);
        let lane_xp = combat.is_some_and(|c| c["xp_contract"]==2);
        let upgrades = combat.is_some_and(|c| c["upgrade_contract"]==1);
        let mut observed_lane_xp = false;
        let mut shared_boards = std::collections::BTreeMap::new();
        fn decode_capture<T: prost::Message + Default>(path: PathBuf) -> Vec<T> {
            let bytes = fs::read(path).unwrap();
            let mut cursor = 0;
            let mut messages = Vec::new();
            while cursor < bytes.len() {
                assert!(cursor+4<=bytes.len(),"truncated wire prefix");
                let len = u32::from_be_bytes(bytes[cursor..cursor+4].try_into().unwrap()) as usize;
                cursor += 4;
                assert!(len>0 && cursor+len<=bytes.len(),"truncated wire body");
                messages.push(T::decode(&bytes[cursor..cursor+len]).unwrap());
                cursor += len;
            }
            messages
        }
        for (player,team,folder) in [(1,1,"team-1-runtime"),(2,2,"team-2-runtime"),(3,1,"player-3/team-1-runtime")] {
            let manifest:serde_json::Value = serde_json::from_slice(&fs::read(root.join(folder).join("manifest.json")).unwrap()).unwrap();
            assert_eq!(manifest["player_id"],player);
            assert_eq!(manifest["team_id"],team);
            let namespaces = [omoba_core::runtime::SINGLE_LANE_KILLS_METRIC_ID,
                omoba_core::runtime::SINGLE_LANE_DEATHS_METRIC_ID,omoba_core::runtime::SINGLE_LANE_ASSISTS_METRIC_ID];
            let mut authority_scores = std::collections::BTreeMap::new();
            let mut attack_acceptances = Vec::new();
            let mut authority_boards = std::collections::BTreeMap::new();
            let mut authority_progression = std::collections::BTreeMap::new();
            let mut authority_ranks = std::collections::BTreeMap::new();
            let mut upgrade_acceptances = Vec::new();
            for frame in decode_capture::<omoba_core::game_proto::TeamTickFrame>(root.join(folder).join("team-frame.capture")) {
                assert_eq!(frame.team_id,team);
                let step = frame.step.unwrap();
                for input in &step.accepted_inputs {
                    if upgrades && input.action_kind == 10 {
                        assert!(input.actor.is_some() && input.target.is_none());
                        let decoded = omoba_core::game_proto::PlayerInput::decode(input.sanitized_payload.as_slice()).unwrap();
                        assert!(matches!(decoded.action, Some(omoba_core::game_proto::player_input::Action::UpgradeAbility(value)) if value.ability_index == 0));
                        assert!(if team == 1 { [1,3].contains(&input.player_id) } else { input.player_id == 2 }, "enemy upgrade input leaked");
                        if input.player_id == player { upgrade_acceptances.push(input.input_id); }
                    }
                    if input.player_id == player && input.action_kind == 3 {
                        assert!(input.actor.is_some() && input.target.is_some());
                        attack_acceptances.push((input.input_id,frame.replica_tick));
                    }
                }
                let events = step.public_events;
                if upgrades {
                    for event in &events {
                        if event.event_kind == omoba_core::runtime::FactKind::CommittedAbilityRanks as u32 {
                            assert_eq!(event.sanitized_payload.len(),16);
                            let values: Vec<_> = event.sanitized_payload.chunks_exact(4)
                                .map(|v| i32::from_le_bytes(v.try_into().unwrap())).collect();
                            authority_ranks.insert((frame.replica_tick,event.subject.as_ref().unwrap().value), values);
                        }
                    }
                }
                if require_xp {
                    for event in &events {
                        if event.event_kind == omoba_core::runtime::FactKind::CommittedProgression as u32 {
                            assert_eq!(event.sanitized_payload.len(),16);
                            let values: Vec<_> = event.sanitized_payload.chunks_exact(4)
                                .map(|v| i32::from_le_bytes(v.try_into().unwrap())).collect();
                            authority_progression.insert((frame.replica_tick,event.subject.as_ref().unwrap().value), values);
                        }
                    }
                }
                if require_board {
                    let board = crate::scoreboard::project(&omoba_core::runtime::FilteredRenderSnapshot {
                        team_id:team,replica_tick:frame.replica_tick,entities:vec![],public_events:events.clone(),
                        external_effects:vec![],memory_directives:vec![],remembered_presentations:Default::default(),
                    }).expect("complete public board in wire");
                    assert_eq!(board.rows.len(),3);
                    if let Some(other) = shared_boards.insert(frame.replica_tick,board.clone()) {
                        assert_eq!(board,other,"cross-team public scoreboard differs");
                    }
                    authority_boards.insert(frame.replica_tick,board);
                }
                for event in &events {
                    let p = &event.sanitized_payload;
                    if event.event_kind != omoba_core::runtime::FactKind::Hud as u32 || p.len()!=20 {continue}
                    let id = u64::from_le_bytes(p[4..12].try_into().unwrap());
                    for ns in namespaces {
                        if let Some(owner) = omoba_core::runtime::single_lane_metric_player(id,ns) {
                            assert_eq!(p[0..4],team.to_le_bytes(),"enemy score metric leaked");
                            assert!(if team==1 {[1,3].contains(&owner)} else {owner==2});
                        }
                    }
                }
                let values:Vec<_> = namespaces.into_iter().map(|ns| {
                    let id = omoba_core::runtime::single_lane_player_metric(ns,player);
                    let event = events.iter().find(|e| e.event_kind==omoba_core::runtime::FactKind::Hud as u32
                        && e.sanitized_payload.len()==20 && e.sanitized_payload[4..12]==id.to_le_bytes()).expect("wire score metric");
                    u32::try_from(i64::from_le_bytes(event.sanitized_payload[12..20].try_into().unwrap())).unwrap()
                }).collect();
                authority_scores.insert(frame.replica_tick,(values[0],values[1],values[2]));
            }
            let data = fs::read(root.join(folder).join("presentation.capture")).unwrap();
            let mut offset = 0;
            let mut observed = 0;
            let mut last_tick = 0;
            let mut last_score = (0,0,0);
            let mut observed_reward_xp = false;
            let mut observed_upgrade = false;
            while offset < data.len() {
                assert!(offset+4<=data.len(),"truncated capture prefix");
                let len = u32::from_be_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
                offset += 4;
                assert!(len>0 && offset+len<=data.len(),"truncated capture body");
                let envelope = RendererIpcEnvelope::decode(&data[offset..offset+len]).unwrap();
                offset += len;
                let Some(Payload::Snapshot(snapshot)) = envelope.payload else {continue};
                let hud = snapshot.moba_hud.expect("MOBA HUD");
                let score = hud.score.expect("authority score supported");
                assert_eq!(snapshot.team_id,team);
                assert_eq!(hud.player_id,player);
                assert_eq!(score.player_id,player,"teammate HUD substituted");
                // Even without attack input, NPCs can kill heroes. Validate
                // SelectiveReplica advances world.tick after applying the frame;
                // the render snapshot identifies that post-step world state.
                let frame_tick = snapshot.replica_tick.checked_sub(1).expect("post-step snapshot tick");
                let expected = authority_scores.get(&frame_tick).unwrap_or_else(|| panic!(
                    "player {player} snapshot tick {} missing in wire range {:?}..{:?}",snapshot.replica_tick,
                    authority_scores.keys().next(),authority_scores.keys().next_back()));
                if require_board {
                    assert_eq!(hud.scoreboard.as_ref().expect("public IPC scoreboard"),authority_boards.get(&frame_tick).unwrap());
                }
                assert_eq!((score.kills,score.deaths,score.assists),*expected);
                last_score = *expected;
                if require_xp {
                    if let Some(hero) = &hud.hero {
                        if hud.phase == 1 {
                            let raw = authority_progression.get(&(frame_tick,hero.render_id)).unwrap_or_else(|| panic!("missing active owner XP player={player} tick={frame_tick} render={}",hero.render_id));
                            assert_eq!((hero.level,hero.experience,hero.skill_points),(raw[0] as u32,raw[1] as u32,raw[3] as u32));
                            if upgrades {
                                let ranks = authority_ranks.get(&(frame_tick,hero.render_id)).expect("raw rank fact");
                                assert_eq!(hero.abilities.len(),4);
                                for (slot, ability) in hero.abilities.iter().enumerate() {
                                    assert_eq!(ability.level, ranks[slot] as u32, "IPC rank differs from authority");
                                    assert_eq!(ability.level, if slot == 0 && player != 2 && ability.level == 2 { 2 } else { 1 });
                                }
                                observed_upgrade |= hero.abilities[0].level == 2;
                            }
                        }
                        let mut expected = omoba_core::runtime::Hero::default();
                        expected.add_moba_experience(score.kills * omoba_template_ids::MOBA_HERO_KILL_XP
                            + score.assists * omoba_template_ids::MOBA_HERO_ASSIST_XP);
                        if lane_xp {
                            // Lane deaths need no player kill credit. Exact IPC
                            // values were checked against the raw authority fact
                            // above; KDA alone is only a minimum XP lower bound.
                            let actual=(hero.level,hero.experience);
                            let minimum=(expected.level as u32,expected.experience as u32);
                            assert!(actual>=minimum,"missing combat XP player {player}");
                            let spent = if upgrades { hero.abilities.iter().map(|a| a.level.saturating_sub(1)).sum::<u32>() } else { 0 };
                            assert_eq!(hero.skill_points,(hero.level-1).checked_sub(spent).expect("more points spent than earned"),"point conservation");
                            observed_lane_xp |= actual>minimum;
                        } else {
                            assert_eq!((hero.level,hero.experience,hero.skill_points),
                                (expected.level as u32,expected.experience as u32,expected.skill_points as u32),"XP paid more than once / wrong owner player {player}");
                        }
                        observed_reward_xp |= score.kills > 0 || score.assists > 0;
                    }
                }
                if combat.is_some() {
                    let economy = snapshot.owner_economy.expect("persistent combat economy");
                    assert_eq!(economy.player_id,player);
                    let passive = ((hud.elapsed_raw - 2*1024).max(0)/1024) as u32
                        * omoba_template_ids::MOBA_PASSIVE_GOLD_PER_SECOND;
                    let earned = score.kills * omoba_template_ids::MOBA_HERO_KILL_GOLD
                        + score.assists * omoba_template_ids::MOBA_HERO_ASSIST_GOLD;
                    assert_eq!(economy.gold,i32::try_from(passive+earned).unwrap(),
                        "player {player} tick {} reward must be exactly once",snapshot.replica_tick);
                }
                assert!(snapshot.replica_tick>=last_tick);
                last_tick = snapshot.replica_tick;
                observed += 1;
            }
            assert!(observed>=100 && last_tick>=1080,"player {player} insufficient live IPC coverage");
            if let Some(combat) = combat {
                if upgrades {
                    if player != 2 { assert!(observed_upgrade); assert_eq!(upgrade_acceptances,vec![3]); }
                    else { assert!(!observed_upgrade); assert!(upgrade_acceptances.is_empty()); }
                }
                if require_xp && player != 2 { assert!(observed_reward_xp, "missing live rewarded XP player {player}"); }
                let score = &combat["scores"][(player-1) as usize];
                assert_eq!(last_score,(score["kills"].as_u64().unwrap() as u32,
                    score["deaths"].as_u64().unwrap() as u32,score["assists"].as_u64().unwrap() as u32));
                assert!(last_tick>=combat["settlement_tick"].as_u64().unwrap()+240);
                if player!=2 { assert_eq!(attack_acceptances.len(),1); assert_eq!(attack_acceptances[0].0,2); }
                else { assert!(attack_acceptances.is_empty()); }
            }
            eprintln!("real owner score capture player={player} team={team} snapshots={observed} last_tick={last_tick}");
        }
        if lane_xp {assert!(observed_lane_xp,"no live lane XP observed above kill/assist rewards");}
    }

    #[test]
    #[ignore = "requires OMOBA_UE_UPGRADE_CAPTURE_ROOT from a completed dual Unreal upgrade run"]
    fn real_unreal_ability_upgrade_capture() {
        use omoba_core::game_proto::{renderer_ipc_envelope::Payload, TeamTickFrame};
        let root = PathBuf::from(std::env::var("OMOBA_UE_UPGRADE_CAPTURE_ROOT").unwrap());
        let report: serde_json::Value = serde_json::from_slice(&fs::read(root.join("unreal-ipc-smoke-report.json")).unwrap()).unwrap();
        assert_eq!(report["success"], true);
        assert_eq!(report["cleanup_verified"], true);
        assert_eq!(report["upgrade_smoke"], true);
        assert_eq!(report["tick_rate_hz"], 60);
        let cast_mode = !report["first_learning_cast"].is_null();
        let learn_slot = if cast_mode { 3usize } else { 0usize };
        let first_learning = !report["first_learning"].is_null();
        let initial_rank = if first_learning { 0u32 } else { 1u32 };
        if first_learning {
            assert_eq!(report["first_learning"]["input_route"], if cast_mode {"unreal-bound-CtrlR-delegate"} else {"unreal-bound-CtrlQ-delegate"});
            assert_eq!(report["first_learning"]["injected_gameplay_state"], false);
        }
        fn frames<T: prost::Message + Default>(path: PathBuf) -> Vec<T> {
            let bytes = fs::read(path).unwrap(); let mut cursor=0; let mut result=Vec::new();
            while cursor<bytes.len() {
                assert!(cursor+4<=bytes.len());
                let size=u32::from_be_bytes(bytes[cursor..cursor+4].try_into().unwrap()) as usize; cursor+=4;
                assert!(size>0 && size<=8*1024*1024 && cursor+size<=bytes.len());
                result.push(T::decode(&bytes[cursor..cursor+size]).unwrap()); cursor+=size;
            }
            result
        }
        for team in 1..=2u32 {
            let folder=root.join(format!("team-{team}-runtime"));
            let shown=&report["teams"][team as usize-1]["upgrade"];
            assert_eq!(shown["complete"],true);
            if first_learning {
                assert_eq!(shown["before"]["rank"],0);
                assert_eq!(shown["after"]["rank"],1);
            }
            let id=shown["before"]["input_id"].as_u64().unwrap();
            let cast=&report["teams"][team as usize-1]["first_learning_cast"];
            let cast_id=if cast_mode {assert_eq!(cast["complete"],true);cast["before"]["input_id"].as_u64().unwrap()} else {0};
            let mut ranks=std::collections::BTreeMap::new();
            let mut progression=std::collections::BTreeMap::new();
            let mut vitals=std::collections::BTreeMap::new();
            let mut cast_acceptances=Vec::new();
            let mut cast_tick=None;
            let mut accepted=Vec::new();
            for frame in frames::<TeamTickFrame>(folder.join("team-frame.capture")) {
                assert_eq!(frame.team_id,team);
                let step=frame.step.unwrap();
                for input in step.accepted_inputs {
                    if first_learning {assert_eq!(input.player_id,team,"opponent private input leaked");}
                    if input.action_kind==10 {
                        assert_eq!(input.player_id,team,"enemy upgrade input leaked");
                        assert!(input.actor.is_some() && input.target.is_none());
                        let decoded=omoba_core::game_proto::PlayerInput::decode(input.sanitized_payload.as_slice()).unwrap();
                        assert!(matches!(decoded.action,Some(omoba_core::game_proto::player_input::Action::UpgradeAbility(v)) if v.ability_index==learn_slot as u32));
                        accepted.push(input.input_id);
                    }
                    if cast_mode {
                        let decoded=omoba_core::game_proto::PlayerInput::decode(input.sanitized_payload.as_slice()).unwrap();
                        if let Some(omoba_core::game_proto::player_input::Action::CastAbility(v))=decoded.action {
                            assert_eq!(v.ability_index,3);
                            cast_acceptances.push(input.input_id);
                            assert!(cast_tick.replace(frame.replica_tick).is_none());
                        }
                    }
                }
                for event in step.public_events {
                    // EquipmentStats is the final per-tick hero settlement;
                    // Vitals is only emitted for targets touched by outcomes.
                    if cast_mode && event.event_kind==omoba_core::runtime::FactKind::CommittedEquipmentStats as u32 {
                        assert_eq!(event.sanitized_payload.len(),40);
                        let values:Vec<_>=event.sanitized_payload[..16].chunks_exact(8).map(|v|i64::from_le_bytes(v.try_into().unwrap())).collect();
                        vitals.insert((frame.replica_tick,event.subject.unwrap().value),values);
                        continue;
                    }
                    let map=if event.event_kind==omoba_core::runtime::FactKind::CommittedAbilityRanks as u32 { &mut ranks }
                        else if event.event_kind==omoba_core::runtime::FactKind::CommittedProgression as u32 { &mut progression }
                        else {continue};
                    assert_eq!(event.sanitized_payload.len(),16);
                    let values:Vec<_>=event.sanitized_payload.chunks_exact(4).map(|v|i32::from_le_bytes(v.try_into().unwrap())).collect();
                    map.insert((frame.replica_tick,event.subject.unwrap().value),values);
                }
            }
            assert_eq!(accepted,vec![id],"original upgrade accepted exactly once");
            if cast_mode {assert_eq!(cast_acceptances,vec![cast_id],"original R cast accepted exactly once");}
            let mut count=0; let mut matched=[false;2];
            let mut cast_matched=[false;2];
            let mut cast_hero=None;
            for envelope in frames::<RendererIpcEnvelope>(folder.join("presentation.capture")) {
                let Some(Payload::Snapshot(snapshot))=envelope.payload else {continue};
                assert_eq!(snapshot.team_id,team);
                let hud=snapshot.moba_hud.expect("MOBA HUD"); assert_eq!(hud.player_id,team);
                let Some(hero)=hud.hero else {continue};
                if hud.phase!=1 {continue;}
                let key=(snapshot.replica_tick.checked_sub(1).unwrap(),hero.render_id);
                let raw=progression.get(&key).expect("exact raw progression");
                assert_eq!((hero.level,hero.experience,hero.skill_points),(raw[0] as u32,raw[1] as u32,raw[3] as u32));
                let raw=ranks.get(&key).expect("exact raw ranks"); assert_eq!(hero.abilities.len(),4);
                for (slot,ability) in hero.abilities.iter().enumerate() {
                    assert_eq!(ability.level,raw[slot] as u32);
                    assert!(if slot==learn_slot {[initial_rank,initial_rank+1].contains(&ability.level)} else {ability.level==initial_rank});
                }
                let spent=hero.abilities.iter().map(|a|a.level-initial_rank).sum::<u32>();
                assert_eq!(hero.skill_points,(hero.level-1+u32::from(first_learning)).checked_sub(spent).unwrap());
                if first_learning {assert!(hero.abilities.iter().enumerate().all(|(slot,a)| (cast_mode && slot==learn_slot && a.level==1) || a.cooldown_raw==0));}
                if cast_mode {
                    let raw=vitals.get(&key).expect("exact raw committed vitals");
                    assert_eq!((hero.hp_raw,hero.max_hp_raw),(raw[0],raw[1]));
                    for (index,stage) in ["before","after"].iter().enumerate() {
                        let ui=&cast[stage];
                        if snapshot.replica_tick==ui["tick"].as_u64().unwrap() {
                            for (field,value) in [("hp",hero.hp_raw),("max_hp",hero.max_hp_raw),("cooldown",hero.abilities[3].cooldown_raw)] {
                                assert!((ui[field].as_f64().unwrap()-value as f64/1024.0).abs()<0.002,"native {field} differs from raw at exact tick");
                            }
                            assert_eq!(hero.abilities[3].level,1);
                            assert!(if index==0 {hero.hp_raw<hero.max_hp_raw && hero.abilities[3].cooldown_raw==0} else {hero.abilities[3].cooldown_raw>0});
                            cast_matched[index]=true;
                            if index==0 {cast_hero=Some(hero.render_id);}
                        }
                    }
                }
                for (index,stage) in ["before","after"].iter().enumerate() {
                    let ui=&shown[stage];
                    if snapshot.replica_tick==ui["tick"].as_u64().unwrap() {
                        assert_eq!(ui["rank"],hero.abilities[learn_slot].level);
                        assert_eq!(ui["sp"],hero.skill_points);
                        assert_eq!(ui["level"],hero.level);
                        assert_eq!(ui["xp"],hero.experience);
                        matched[index]=true;
                    }
                }
                count+=1;
            }
            assert!(count>100 && matched==[true,true]);
            if cast_mode {
                assert_eq!(cast_matched,[true,true]);
                let tick=cast_tick.unwrap();let render_id=cast_hero.unwrap();
                let before=vitals.get(&(tick-1,render_id)).expect("raw pre-cast HP");
                let after=vitals.get(&(tick,render_id)).expect("raw cast-step HP");
                let definition=omoba_template_ids::active_ability_const(omoba_template_ids::ability_by_name("apprentice_mend").unwrap()).unwrap();
                let heal=definition.extras.iter().find(|(key,_)|*key=="heal").unwrap().1[0].raw();
                assert!(heal>0);
                assert_eq!(after[0],(before[0]+heal).min(after[1]),"authority cast-step must commit Lua rank-one heal (this fixture requires no concurrent damage)");
                assert!(after[0]>before[0]);
                println!("real Unreal learned R team={team} cast_input={cast_id} cast_tick={tick} hp_before_raw={} hp_after_raw={} lua_heal_raw={heal}",before[0],after[0]);
            }
            println!("real Unreal upgrade team={team} snapshots={count} input={id}");
        }
    }

    #[test]
    #[ignore = "requires OMOBA_UE_RECALL_CAPTURE_ROOT from a completed real Unreal recall run"]
    fn real_unreal_single_lane_recall_capture() {
        use omoba_core::game_proto::renderer_ipc_envelope::Payload;
        use std::io::Read;
        let root = PathBuf::from(std::env::var("OMOBA_UE_RECALL_CAPTURE_ROOT").unwrap());
        for team in [1, 2] {
            let log = fs::read_to_string(root.join(format!("logs/runtime-p{team}.stderr.log"))).unwrap();
            assert!(!log.contains("recall smoke"), "internal injection is not Unreal input");
            let mut file = std::io::BufReader::new(fs::File::open(root.join(format!("team-{team}-runtime/presentation.capture"))).unwrap());
            let mut channels = Vec::new();
            let mut channel: Option<(u64, i64, i64)> = None;
            let mut last_tick = 0;
            let mut snapshots = 0;
            loop {
                let mut prefix = [0; 4];
                if file.read(&mut prefix[..1]).unwrap() == 0 { break; }
                file.read_exact(&mut prefix[1..]).unwrap();
                let len = u32::from_be_bytes(prefix) as usize;
                assert!(len > 0 && len <= 8 * 1024 * 1024);
                let mut bytes = vec![0; len]; file.read_exact(&mut bytes).unwrap();
                let envelope = RendererIpcEnvelope::decode(bytes.as_slice()).unwrap();
                let Some(Payload::Snapshot(snapshot)) = envelope.payload else { continue; };
                assert_eq!(snapshot.team_id, team);
                let Some(hud) = snapshot.moba_hud else { continue; };
                assert_eq!(hud.player_id, team);
                assert!(hud.recall_protocol_enabled);
                assert!(snapshot.replica_tick >= last_tick);
                if snapshot.replica_tick == last_tick { continue; }
                last_tick = snapshot.replica_tick;
                snapshots += 1;
                assert!((0..=8 * 1024).contains(&hud.recall_remaining_raw));
                if hud.recall_remaining_raw > 0 {
                    assert_eq!(hud.phase, 1);
                    assert!(hud.hero.as_ref().is_some_and(|hero| hero.hp_raw > 0));
                    if let Some((_, _, previous)) = &mut channel {
                        assert!(hud.recall_remaining_raw < *previous, "countdown must decrease without reset");
                        *previous = hud.recall_remaining_raw;
                    } else {
                        channel = Some((snapshot.replica_tick, hud.recall_remaining_raw, hud.recall_remaining_raw));
                    }
                } else if let Some((start, first, previous)) = channel.take() {
                    channels.push((start, snapshot.replica_tick, first, previous));
                }
            }
            assert!(channel.is_none(), "capture ended during recall");
            assert_eq!(channels.len(), 2, "expected canceled and completed channels");
            let canceled = channels[0].1 - channels[0].0;
            assert!((120..=130).contains(&canceled), "first channel must be movement-canceled after two seconds");
            let complete = channels[1].1 - channels[1].0;
            let expected = (channels[1].2 as u64 * 60) / 1024;
            assert!(complete.abs_diff(expected) <= 2 && complete >= 470);
            assert!(channels[1].3 <= 35, "last completed sample must be near zero");
            assert!(snapshots > 1000);
            println!("real recall team={team} snapshots={snapshots} canceled_ticks={canceled} completed_ticks={complete}");
        }
    }

    #[test]
    #[ignore = "requires OMOBA_SHOP_CAPTURE_ROOT from a completed real transaction run"]
    fn real_single_lane_shop_transaction_capture() {
        let root = PathBuf::from(std::env::var("OMOBA_SHOP_CAPTURE_ROOT").unwrap());
        validate_shop_capture(&root, false);
    }

    #[test]
    #[ignore = "requires OMOBA_UE_SHOP_CAPTURE_ROOT from a completed real Unreal transaction run"]
    fn real_unreal_single_lane_shop_transaction_capture() {
        let root = PathBuf::from(std::env::var("OMOBA_UE_SHOP_CAPTURE_ROOT").unwrap());
        validate_shop_capture(&root, true);
    }

    fn validate_shop_capture(root: &std::path::Path, unreal: bool) {
        use omoba_core::game_proto::renderer_ipc_envelope::Payload;
        use std::io::Read;
        for team in [1, 2] {
            let log = fs::read_to_string(root.join(format!("logs/runtime-p{team}.stderr.log"))).unwrap();
            assert!(!log.contains("conflicting shop receipt"));
            if unreal {
                let ue = fs::read_to_string(root.join(format!("logs/ue-p{team}.stdout.log"))).unwrap();
                for (stage, request) in [(0,1), (2,2), (4,3)] {
                    assert!(ue.contains(&format!("OM_SHOP_SMOKE queued player={team} stage={stage} accepted=1 request={request}")));
                }
                assert!(ue.contains(&format!("OM_SHOP_SMOKE rejected player={team} input=1 code=6")));
                assert!(ue.contains(&format!("OM_SHOP_SMOKE bought player={team} input=2")));
                assert!(ue.contains(&format!("OM_SHOP_SMOKE complete player={team} input=3")));
                assert!(ue.lines().any(|line| line.contains(&format!("OM_SHOP_SMOKE complete player={team} input=3")) && line.contains("pending=0")));
                assert!(!log.contains("shop smoke exact retries"), "Unreal test must not inject runtime gameplay intents");
            } else {
                assert!(log.contains("shop receipt recovery input_id=2 status=2 terminal=true"));
                for id in 1..=3 { assert!(log.contains(&format!("shop smoke exact retries player={team} input_id={id} count=5"))); }
            }
            let mut file = std::io::BufReader::new(fs::File::open(root.join(format!("team-{team}-runtime/presentation.capture"))).unwrap());
            let mut receipts = std::collections::BTreeMap::new();
            let mut snapshots = 0;
            let mut equipped = false;
            let mut last_gold = 0;
            loop {
                let mut prefix = [0;4];
                if file.read(&mut prefix[..1]).unwrap() == 0 { break; }
                file.read_exact(&mut prefix[1..]).unwrap();
                let len = u32::from_be_bytes(prefix) as usize;
                assert!(len > 0 && len <= 8 * 1024 * 1024);
                let mut data = vec![0;len]; file.read_exact(&mut data).unwrap();
                let envelope = RendererIpcEnvelope::decode(data.as_slice()).unwrap();
                let Some(Payload::Snapshot(snapshot)) = envelope.payload else { continue };
                let (Some(hud), Some(economy)) = (snapshot.moba_hud, snapshot.owner_economy) else { continue };
                assert_eq!(snapshot.team_id, team); assert_eq!(economy.player_id, team); assert_eq!(hud.player_id, team);
                if unreal { assert!(economy.shop_protocol_enabled); }
                let mut net_spent = 0;
                for receipt in &snapshot.shop_receipts {
                    assert_eq!(receipt.player_id, team);
                    if let Some(original) = receipts.insert(receipt.input_id, receipt.clone()) { assert_eq!(original, *receipt, "original receipt changed"); }
                    if receipt.result_code == 0 { net_spent += if receipt.action_kind == 17 { 350 } else { -175 }; }
                }
                let income = (hud.elapsed_raw - 2048).max(0) / 1024 * 2;
                assert_eq!(economy.gold, income as i32 - net_spent, "duplicate execution or wrong settlement at team {team} tick {}", snapshot.replica_tick);
                assert_eq!(economy.slots.len(), 6);
                let occupied: Vec<_> = economy.slots.iter().filter(|s| s.catalog_id != 0).collect();
                assert!(occupied.len() <= 1);
                if !occupied.is_empty() { assert_eq!(occupied[0].catalog_id, 1); equipped = true; }
                snapshots += 1; last_gold = economy.gold;
            }
            assert_eq!(receipts.len(), 3);
            assert_eq!((receipts[&1].action_kind, receipts[&1].result_code), (17,6));
            assert_eq!((receipts[&2].action_kind, receipts[&2].result_code), (17,0));
            assert_eq!((receipts[&3].action_kind, receipts[&3].result_code), (18,0));
            assert!(equipped && snapshots > 1000 && last_gold >= 175);
            println!("real shop team={team} snapshots={snapshots} immutable_receipts=3 gold={last_gold}");
        }
    }

    /// Opt-in validation of the real two-runtime smoke capture, not a fixture.
    #[test]
    #[ignore = "requires OMOBA_INCOME_CAPTURE_ROOT from a completed real smoke run"]
    fn real_single_lane_owner_income_capture() {
        use omoba_core::game_proto::renderer_ipc_envelope::Payload;
        let root = PathBuf::from(std::env::var("OMOBA_INCOME_CAPTURE_ROOT").unwrap());
        for team in [1, 2] {
            let data =
                fs::read(root.join(format!("team-{team}-runtime/presentation.capture"))).unwrap();
            let mut offset = 0;
            let mut observed = 0;
            let mut last_gold = 0;
            let mut last_tick = 0;
            while offset < data.len() {
                assert!(offset + 4 <= data.len(), "truncated capture prefix");
                let len = u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                assert!(
                    len > 0 && offset + len <= data.len(),
                    "truncated capture body"
                );
                let envelope = RendererIpcEnvelope::decode(&data[offset..offset + len]).unwrap();
                offset += len;
                let Some(Payload::Snapshot(snapshot)) = envelope.payload else {
                    continue;
                };
                let (Some(hud), Some(economy)) = (snapshot.moba_hud, snapshot.owner_economy) else {
                    continue;
                };
                assert_eq!(snapshot.team_id, team);
                assert_eq!(hud.player_id, team);
                assert_eq!(economy.player_id, team);
                // This smoke uses the shipped 2-second warmup / 2 gold per
                // active second, no pause and no gameplay shop submissions.
                assert_ne!(hud.phase, 2, "smoke must finish before match termination");
                let expected = ((hud.elapsed_raw - 2 * 1024).max(0) / 1024 * 2) as i32;
                assert_eq!(
                    economy.gold, expected,
                    "team {team} tick {}",
                    snapshot.replica_tick
                );
                assert_eq!(economy.slots.len(), 6);
                assert!(economy.slots.iter().all(|slot| slot.catalog_id == 0));
                assert!(snapshot.shop_receipts.is_empty());
                observed += 1;
                last_gold = economy.gold;
                last_tick = snapshot.replica_tick;
            }
            assert!(
                observed >= 100 && last_tick >= 1080 && last_gold >= 12,
                "insufficient real income coverage"
            );
            println!("real owner income team={team} snapshots={observed} tick={last_tick} gold={last_gold}");
        }
    }
}
