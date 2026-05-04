use anyhow::Result;
use log::*;
use prost::Message;
use serde_json::json;
use tokio::sync::mpsc;
use tokio::io::{ReadHalf, WriteHalf};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use tokio_kcp::{KcpConfig, KcpStream, KcpNoDelayConfig};

use super::framing::*;
use super::game_proto::*;
use crate::quant::{facing_dequant, fixed_dequant, pos_dequant};
use omoba_template_ids::{creep_display, projectile_id_str, CreepId, ProjectileKindId};

/// P3: client-side cache for hero static metadata.
///
/// The server now emits `HeroStatic` (cold, rare: create/level-up/ability learn)
/// and `HeroHot` (hot, every 0.3s) separately. Legacy omfx expects a single
/// merged `hero.stats` JSON. The shim caches the latest static per entity id,
/// and on each `HeroHot` arrival merges with cache + emits one hero.stats JSON
/// identical to the pre-P3 wire shape so omfx needs zero changes.
///
/// `latest_lives` tracks the most recent `GameLives` event value and is injected
/// into the merged JSON (previously the server stuffed `lives` into hero.stats).
#[derive(Default)]
struct HeroStatsCache {
    statics: HashMap<u64, HeroStatic>,
    latest_lives: Option<i32>,
}

/// KCP client for communicating with the omb game server.
pub struct KcpClient {
    player_name: String,
    writer: Arc<Mutex<WriteHalf<KcpStream>>>,
    event_rx: Option<mpsc::Receiver<GameEventData>>,
    /// Phase 2 lockstep: separate channel fed by the reader task whenever a
    /// 0x11 / 0x12 / 0x14 / 0x16 frame arrives. Taken once via
    /// `subscribe_lockstep`.
    lockstep_rx: Option<mpsc::Receiver<LockstepInbound>>,
    /// Phase 2 lockstep: assigned by GameStart; needed to stamp InputSubmit.
    last_player_id: Option<u32>,
    /// Phase 2 lockstep: master_seed cached for caller.
    last_master_seed: Option<u64>,
}

/// Phase 2 lockstep inbound frames the client surfaces from the kcp reader
/// task. `GameStart` is delivered through this channel as well so a caller
/// awaiting `join_lockstep` can recv from the same stream.
///
/// `wire_bytes` = real UDP cost (post-compression + 5 byte framing).
/// `logical_bytes` = decompressed prost payload length. Lets the consumer
/// HUD show both numbers so the lockstep bandwidth win over the legacy
/// per-event broadcast can be quantified.
#[derive(Debug, Clone)]
pub enum LockstepInbound {
    TickBatch { msg: TickBatch, wire_bytes: usize, logical_bytes: usize },
    StateHash { msg: StateHash, wire_bytes: usize, logical_bytes: usize },
    GameStart { msg: GameStart, wire_bytes: usize, logical_bytes: usize },
    SnapshotResp { msg: SnapshotResp, wire_bytes: usize, logical_bytes: usize },
}

/// P6: result of the per-session sequence gap check. Exposed for tests.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SeqGapResult {
    /// First event ever — just seed `last_seq`, no check.
    InitialSeed,
    /// `event.sequence == last_seq + 1` — no gap.
    Ok,
    /// `event.sequence < last_seq + 1` — duplicate / reorder / wrap? Log only.
    Backwards,
    /// `event.sequence > last_seq + 1` — missed `gap_size` events. Client
    /// should request resync.
    Gap { expected: u64, got: u64, gap_size: u64 },
}

/// Pure function: given the previous last-known sequence and the arriving
/// event's sequence, return a `SeqGapResult`. Split out for unit testing
/// without spinning up a KCP connection.
///
/// `last_seq_opt = None` means no events have been seen yet on this session.
/// `last_seq_opt = Some(n)` means the last successful event had sequence `n`;
/// the NEXT expected value is `n + 1`.
pub fn detect_seq_gap(last_seq_opt: Option<u64>, got: u64) -> SeqGapResult {
    match last_seq_opt {
        None => SeqGapResult::InitialSeed,
        Some(last) => {
            let expected = last.wrapping_add(1);
            if got == expected {
                SeqGapResult::Ok
            } else if got < expected {
                // Likely a reorder (KCP is reliable+ordered so this should be
                // rare) or a duplicate. Log-only; don't resync.
                SeqGapResult::Backwards
            } else {
                SeqGapResult::Gap {
                    expected,
                    got,
                    gap_size: got - expected,
                }
            }
        }
    }
}

/// Parsed game event data for client consumption.
#[derive(Debug, Clone)]
pub struct GameEventData {
    pub topic: String,
    pub msg_type: String,
    pub action: String,
    pub data: serde_json::Value,
    pub timestamp_ms: u64,
    /// 原始 proto data_json bytes 長度；供前端網路吞吐統計用，
    /// 避免在 hot path 做冗餘 serde_json::to_string。
    /// **舊欄位**：=`logical_bytes`（解壓後 prost payload 長度）。
    /// 保留欄位名為了 source-compat。前端要看真實 UDP 成本請用
    /// `wire_bytes` 欄位（`logical_bytes` 約等於 `wire_bytes` × LZ4 解壓比）。
    pub payload_bytes: usize,
    /// **真實 UDP/KCP wire 成本**（LZ4 壓縮後 + framing 1+4 byte header）。
    /// 對應 server 端 `KcpBytesCounter::record(frame.len())` 計的同一個值。
    /// 前端 HUD 顯示真實 bandwidth 應用此欄位，不是 `payload_bytes`。
    pub wire_bytes: usize,
}

impl KcpClient {
    /// Connect to the KCP game server.
    pub async fn connect(addr: &str, player_name: String) -> Result<Self> {
        let mut config = KcpConfig::default();
        config.nodelay = KcpNoDelayConfig::fastest();

        let sock_addr: std::net::SocketAddr = addr.parse()?;
        let stream = KcpStream::connect(&config, sock_addr).await?;
        info!("Connected to KCP server at {}", addr);

        let (reader, writer) = tokio::io::split(stream);
        let writer = Arc::new(Mutex::new(writer));

        // Send subscribe request immediately
        {
            let mut w = writer.lock().await;
            let sub = SubscribeRequest {
                player_name: player_name.clone(),
            };
            write_framed_msg(&mut *w, TAG_SUBSCRIBE_REQUEST, &sub).await?;
        }

        // Spawn background reader task
        let (event_tx, event_rx) = mpsc::channel(10000);
        let (lockstep_tx, lockstep_rx) = mpsc::channel(1024);
        Self::spawn_reader(reader, event_tx, lockstep_tx, writer.clone(), player_name.clone());

        Ok(Self {
            player_name,
            writer,
            event_rx: Some(event_rx),
            lockstep_rx: Some(lockstep_rx),
            last_player_id: None,
            last_master_seed: None,
        })
    }

    fn spawn_reader(
        mut reader: ReadHalf<KcpStream>,
        event_tx: mpsc::Sender<GameEventData>,
        lockstep_tx: mpsc::Sender<LockstepInbound>,
        writer_for_resync: Arc<Mutex<WriteHalf<KcpStream>>>,
        player_name_for_resync: String,
    ) {
        tokio::spawn(async move {
            // P3: local cache lives in the reader task so no locking is needed.
            // HeroStatic arrivals update cache only (no emit); HeroHot arrivals
            // look up the cache + emit a merged legacy hero.stats JSON.
            let mut hero_cache = HeroStatsCache::default();
            // P6: per-session sequence gap detection. None until the first
            // GameEvent arrives; after that, every subsequent event's sequence
            // must equal last + 1.
            let mut last_seq: Option<u64> = None;
            // Throttle resync requests so a pathological flood of gaps
            // doesn't turn into a flood of StateReq traffic. Track the last
            // sequence we requested a resync for; skip if we already asked.
            let mut last_resync_req: Option<u64> = None;
            loop {
                match read_framed(&mut reader).await {
                    Ok(Some((tag, payload, wire_compressed_bytes))) => {
                        match tag {
                            TAG_GAME_EVENT => {
                                match GameEvent::decode(payload.as_slice()) {
                                    Ok(event) => {
                                        // P6: check sequence before decoding payload.
                                        // `sequence` is proto3 uint64 — defaults to 0
                                        // when the server is pre-P6 or hasn't stamped
                                        // (gRPC path), so we only engage gap detection
                                        // once we've seen a non-zero sequence.
                                        let got_seq = event.sequence;
                                        match detect_seq_gap(last_seq, got_seq) {
                                            SeqGapResult::InitialSeed => {
                                                last_seq = Some(got_seq);
                                            }
                                            SeqGapResult::Ok => {
                                                last_seq = Some(got_seq);
                                            }
                                            SeqGapResult::Backwards => {
                                                warn!(
                                                    "⏪ seq backwards (got={}, last_seq={:?}) — keeping state",
                                                    got_seq, last_seq
                                                );
                                                // Do NOT update last_seq — hold the
                                                // highest seen. Duplicate/reorder
                                                // events still get processed.
                                            }
                                            SeqGapResult::Gap { expected, got, gap_size } => {
                                                warn!(
                                                    "⚠️ seq gap: expected={} got={} (missed {} events)",
                                                    expected, got, gap_size
                                                );
                                                // Debounce: don't re-request for the
                                                // same gap the writer already heard
                                                // about.
                                                let should_request = match last_resync_req {
                                                    Some(prev) => prev < expected,
                                                    None => true,
                                                };
                                                if should_request {
                                                    last_resync_req = Some(expected);
                                                    let req = GameStateRequest {
                                                        query_type: "seq-gap".into(),
                                                        // Encode last-known seq as
                                                        // the player_name field —
                                                        // avoids a proto schema bump
                                                        // (see server stub for the
                                                        // matching decode side).
                                                        player_name: expected.saturating_sub(1).to_string(),
                                                    };
                                                    let w = writer_for_resync.clone();
                                                    tokio::spawn(async move {
                                                        let mut w = w.lock().await;
                                                        if let Err(e) = write_framed_msg(
                                                            &mut *w,
                                                            TAG_GAME_STATE_REQUEST,
                                                            &req,
                                                        ).await {
                                                            warn!("Failed to send seq-gap StateReq: {}", e);
                                                        }
                                                    });
                                                }
                                                // Continue processing the out-of-order
                                                // event so we don't double-drop on top
                                                // of the gap (client logic tolerates
                                                // stale HP / position updates).
                                                last_seq = Some(got_seq);
                                            }
                                        }
                                        // Silence the unused-var warning on
                                        // `player_name_for_resync` for builds
                                        // that never hit the Gap branch.
                                        let _ = &player_name_for_resync;
                                        // P9 envelope-strip: every event carries a typed
                                        // `payload` oneof. The shim derives (msg_type, action)
                                        // from the variant and rebuilds JSON for legacy omfx.
                                        // `wire_compressed_bytes` = actual UDP cost (LZ4'd if shrunk + framing);
                                        // `payload.len()` = decompressed prost bytes (logical payload).
                                        let logical_bytes = payload.len();
                                        let parsed_opt: Option<GameEventData> = match event.payload.as_ref() {
                                            Some(p) => translate_typed_payload(p, wire_compressed_bytes, logical_bytes, &mut hero_cache),
                                            None => None,
                                        };

                                        if let Some(parsed) = parsed_opt {
                                            // try_send instead of send().await: when no
                                            // consumer is draining `event_rx` (e.g. Phase 5.1+
                                            // omfx, which only subscribes to the lockstep
                                            // stream), a blocking send fills the 10000-slot
                                            // channel in ~10s of TD_STRESS load and then
                                            // stalls THIS reader task — preventing lockstep
                                            // frames on the same socket from ever being
                                            // delivered. Dropping legacy events on full is
                                            // safe because their consumer is opt-in.
                                            match event_tx.try_send(parsed) {
                                                Ok(()) => {}
                                                Err(mpsc::error::TrySendError::Full(_)) => {
                                                    // No subscriber draining; drop silently.
                                                }
                                                Err(mpsc::error::TrySendError::Closed(_)) => {
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to decode GameEvent: {}", e);
                                    }
                                }
                            }
                            TAG_COMMAND_ACK => {
                                // CommandAck — currently ignored
                            }
                            TAG_GAME_STATE_RESPONSE => {
                                // GameStateResponse — currently not used by client
                            }
                            // ===== Phase 2 Lockstep tags =====
                            TAG_TICK_BATCH => {
                                let logical_bytes = payload.len();
                                match TickBatch::decode(payload.as_slice()) {
                                    Ok(b) => {
                                        if lockstep_tx.send(LockstepInbound::TickBatch {
                                            msg: b,
                                            wire_bytes: wire_compressed_bytes,
                                            logical_bytes,
                                        }).await.is_err() {
                                            break;
                                        }
                                    }
                                    Err(e) => warn!("Failed to decode TickBatch: {}", e),
                                }
                            }
                            TAG_STATE_HASH => {
                                let logical_bytes = payload.len();
                                match StateHash::decode(payload.as_slice()) {
                                    Ok(s) => {
                                        if lockstep_tx.send(LockstepInbound::StateHash {
                                            msg: s,
                                            wire_bytes: wire_compressed_bytes,
                                            logical_bytes,
                                        }).await.is_err() {
                                            break;
                                        }
                                    }
                                    Err(e) => warn!("Failed to decode StateHash: {}", e),
                                }
                            }
                            TAG_GAME_START => {
                                let logical_bytes = payload.len();
                                match GameStart::decode(payload.as_slice()) {
                                    Ok(gs) => {
                                        if lockstep_tx.send(LockstepInbound::GameStart {
                                            msg: gs,
                                            wire_bytes: wire_compressed_bytes,
                                            logical_bytes,
                                        }).await.is_err() {
                                            break;
                                        }
                                    }
                                    Err(e) => warn!("Failed to decode GameStart: {}", e),
                                }
                            }
                            TAG_SNAPSHOT_RESP => {
                                let logical_bytes = payload.len();
                                match SnapshotResp::decode(payload.as_slice()) {
                                    Ok(s) => {
                                        if lockstep_tx.send(LockstepInbound::SnapshotResp {
                                            msg: s,
                                            wire_bytes: wire_compressed_bytes,
                                            logical_bytes,
                                        }).await.is_err() {
                                            break;
                                        }
                                    }
                                    Err(e) => warn!("Failed to decode SnapshotResp: {}", e),
                                }
                            }
                            _ => {
                                warn!("Unknown tag from server: 0x{:02x}", tag);
                            }
                        }
                    }
                    Ok(None) => {
                        info!("KCP connection closed by server");
                        break;
                    }
                    Err(e) => {
                        error!("KCP read error: {}", e);
                        break;
                    }
                }
            }
        });
    }

    /// Send a player command to the server.
    pub async fn send_command(
        &mut self,
        msg_type: &str,
        action: &str,
        data: serde_json::Value,
    ) -> Result<bool> {
        let data_bytes = serde_json::to_vec(&data)?;
        let cmd = PlayerCommand {
            player_name: self.player_name.clone(),
            msg_type: msg_type.to_string(),
            action: action.to_string(),
            data_json: data_bytes,
        };

        let mut w = self.writer.lock().await;
        write_framed_msg(&mut *w, TAG_PLAYER_COMMAND, &cmd).await?;
        Ok(true)
    }

    /// Send a viewport update to the server for spatial filtering.
    pub async fn send_viewport_update(&self, cx: f32, cy: f32, hw: f32, hh: f32) -> Result<()> {
        let vp = ViewportUpdate {
            center_x: cx,
            center_y: cy,
            half_width: hw,
            half_height: hh,
        };
        let mut w = self.writer.lock().await;
        write_framed_msg(&mut *w, TAG_VIEWPORT_UPDATE, &vp).await?;
        Ok(())
    }

    /// Subscribe to game events from the server.
    /// Returns a receiver channel that yields parsed game events.
    pub async fn subscribe_events(
        &mut self,
    ) -> Result<mpsc::Receiver<GameEventData>> {
        self.event_rx
            .take()
            .ok_or_else(|| anyhow::anyhow!("subscribe_events can only be called once"))
    }

    // ===== Phase 2 Lockstep API =====

    /// Take ownership of the lockstep inbound stream. Yields TickBatch /
    /// StateHash / GameStart / SnapshotResp frames as they arrive from the
    /// server. Can only be called once per client.
    pub fn subscribe_lockstep(&mut self) -> Result<mpsc::Receiver<LockstepInbound>> {
        self.lockstep_rx
            .take()
            .ok_or_else(|| anyhow::anyhow!("subscribe_lockstep can only be called once"))
    }

    /// Send a JoinRequest (tag 0x13) and await the server's GameStart reply
    /// (tag 0x14). Returns the assigned `master_seed` from GameStart so the
    /// caller can construct deterministic SimRng streams.
    ///
    /// CAUTION: this method internally drains the lockstep inbound channel
    /// until it sees a GameStart, so do not call this AFTER `subscribe_lockstep`.
    /// The recommended flow is:
    ///   1. `connect`
    ///   2. `join_lockstep` (consumes GameStart from the channel)
    ///   3. `subscribe_lockstep` (now yields only TickBatch/StateHash/SnapshotResp)
    pub async fn join_lockstep(&mut self, player_name: String, observer: bool) -> Result<u64> {
        let role = if observer { JoinRole::RoleObserver } else { JoinRole::RolePlayer };
        let req = JoinRequest {
            player_name: player_name.clone(),
            role: role as i32,
        };
        {
            let mut w = self.writer.lock().await;
            write_framed_msg(&mut *w, TAG_JOIN_REQUEST, &req).await?;
        }
        // Drain the lockstep stream until GameStart arrives.
        let rx = self
            .lockstep_rx
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("lockstep_rx already taken"))?;
        loop {
            match rx.recv().await {
                Some(LockstepInbound::GameStart { msg: gs, .. }) => {
                    self.last_player_id = Some(gs.player_id);
                    self.last_master_seed = Some(gs.master_seed);
                    return Ok(gs.master_seed);
                }
                Some(_) => {
                    // Possible race: a TickBatch may arrive before GameStart
                    // if the server fires it just after `lockstep_joined=true`
                    // is set. We deliberately drop those — Phase 2 callers
                    // should join first, then subscribe.
                    continue;
                }
                None => {
                    anyhow::bail!("lockstep stream closed before GameStart");
                }
            }
        }
    }

    /// Submit a player input targeted at `target_tick`. Caller must have
    /// invoked `join_lockstep` first (otherwise no `player_id` is known).
    ///
    /// Returns `(logical_bytes, wire_bytes)` so the caller can feed network
    /// throughput counters. InputSubmit messages are tiny (well under
    /// `LZ4_THRESHOLD = 128`), so they are never compressed and
    /// `wire = 1 + 4 + logical`.
    pub async fn submit_input(
        &mut self,
        target_tick: u32,
        input: PlayerInput,
    ) -> Result<(usize, usize)> {
        let player_id = self
            .last_player_id
            .ok_or_else(|| anyhow::anyhow!("submit_input before join_lockstep"))?;
        let req = InputSubmit {
            player_id,
            target_tick,
            input: Some(input),
        };
        let logical_bytes = req.encoded_len();
        let mut w = self.writer.lock().await;
        write_framed_msg(&mut *w, TAG_INPUT_SUBMIT, &req).await?;
        let wire_bytes = 1 + 4 + logical_bytes;
        Ok((logical_bytes, wire_bytes))
    }

    /// Request a snapshot from the server. Reply arrives as
    /// `LockstepInbound::SnapshotResp` on the lockstep stream.
    pub async fn request_snapshot(&mut self, from_tick: u32) -> Result<()> {
        let req = SnapshotReq { from_tick };
        let mut w = self.writer.lock().await;
        write_framed_msg(&mut *w, TAG_SNAPSHOT_REQ, &req).await?;
        Ok(())
    }

    /// Most recently observed player_id assigned by GameStart.
    pub fn lockstep_player_id(&self) -> Option<u32> {
        self.last_player_id
    }

    /// Most recently observed master_seed from GameStart.
    pub fn lockstep_master_seed(&self) -> Option<u64> {
        self.last_master_seed
    }
}

/// P9 envelope-strip shim: derive (msg_type, action) from the typed payload
/// variant. Compile-time exhaustive — adding a new proto variant breaks the
/// build here. `LegacyJson` carries its own keys.
pub fn variant_to_legacy_keys(p: &game_event::Payload) -> (String, String) {
    use game_event::Payload::*;
    match p {
        Heartbeat(_)         => ("heartbeat".into(), "tick".into()),
        HeroStatic(_)        => ("hero".into(), "static_internal".into()),
        HeroHot(_)           => ("hero".into(), "stats".into()),
        HeroCreate(_)        => ("hero".into(), "create".into()),
        CreepCreate(_)       => ("creep".into(), "create".into()),
        CreepMove(_)         => ("creep".into(), "M".into()),
        CreepHp(m)           => match super::game_proto::EntityKind::try_from(m.kind).unwrap_or(super::game_proto::EntityKind::Unspecified) {
            super::game_proto::EntityKind::Creep => ("creep".into(), "H".into()),
            super::game_proto::EntityKind::Hero  => ("hero".into(), "H".into()),
            super::game_proto::EntityKind::Unit  => ("unit".into(), "H".into()),
            super::game_proto::EntityKind::Tower => ("tower".into(), "H".into()),
            _                                    => ("entity".into(), "H".into()),
        },
        CreepSlow(_)         => ("creep".into(), "S".into()),
        CreepStall(_)        => ("creep".into(), "stall".into()),
        EntityFacing(_)      => ("entity".into(), "F".into()),
        EntityDeath(m)       => match super::game_proto::EntityKind::try_from(m.kind).unwrap_or(super::game_proto::EntityKind::Unspecified) {
            super::game_proto::EntityKind::Creep      => ("creep".into(), "D".into()),
            super::game_proto::EntityKind::Tower      => ("tower".into(), "D".into()),
            super::game_proto::EntityKind::Hero       => ("hero".into(), "D".into()),
            super::game_proto::EntityKind::Unit       => ("unit".into(), "D".into()),
            super::game_proto::EntityKind::Projectile => ("projectile".into(), "D".into()),
            _                                         => ("entity".into(), "D".into()),
        },
        UnitCreate(_)        => ("unit".into(), "create".into()),
        ProjectileCreate(_)  => ("projectile".into(), "C".into()),
        ProjectileDestroy(_) => ("projectile".into(), "D".into()),
        TowerCreate(_)       => ("tower".into(), "create".into()),
        TowerUpgrade(_)      => ("tower".into(), "upgrade".into()),
        BuffAdd(_)           => ("buff".into(), "buff_add".into()),
        BuffRemove(_)        => ("buff".into(), "buff_remove".into()),
        GameRound(_)         => ("game".into(), "round".into()),
        GameLives(_)         => ("game".into(), "lives".into()),
        GameEnd(_)           => ("game".into(), "end".into()),
        GameExplosion(_)     => ("game".into(), "explosion".into()),
        LegacyJson(m)        => (m.msg_type.clone(), m.action.clone()),
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Translate a prost typed Payload into the legacy JSON-shaped `GameEventData`
/// that omfx already consumes. P9: the wire no longer carries topic / msg_type /
/// action / data_json / timestamp_ms — we derive (msg_type, action) from the
/// variant tag, set topic = "td/all/res" (server already routed), and use the
/// client-local clock for timestamp_ms.
fn translate_typed_payload(
    tp: &game_event::Payload,
    wire_bytes: usize,
    logical_bytes: usize,
    hero_cache: &mut HeroStatsCache,
) -> Option<GameEventData> {
    let (msg_type, action) = variant_to_legacy_keys(tp);
    let topic = "td/all/res".to_string();
    let timestamp_ms = now_ms();
    let default = || GameEventData {
        topic: topic.clone(),
        msg_type: msg_type.clone(),
        action: action.clone(),
        data: serde_json::Value::Null,
        timestamp_ms,
        payload_bytes: logical_bytes,
        wire_bytes,
    };

    let out = match tp {
        game_event::Payload::Heartbeat(hb) => {
            let hp_snapshot: Vec<serde_json::Value> = hb.hp_snapshot.iter().map(|e| {
                let hp = e.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
                json!({ "i": e.id as u32, "h": hp })
            }).collect();
            // P4: creep position sample for client drift correction. Empty
            // Vec (no creeps visible) serialises to an empty array — omfx's
            // snap logic iterates harmlessly.
            let pos_snapshot: Vec<serde_json::Value> = hb.pos_snapshot.iter().map(|e| {
                let (x, y) = e.pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
                json!({ "i": e.id as u32, "x": x, "y": y })
            }).collect();
            // P7 layered: server-side authoritative set of still-alive
            // predeclared-damage projectiles whose target is in this player's
            // viewport. Client uses this to retain its pending_pred_dmg map
            // (entries whose proj_id is NOT in this set have settled).
            let in_flight_projectiles: Vec<serde_json::Value> = hb.in_flight_projectiles
                .iter()
                .map(|&id| serde_json::Value::from(id))
                .collect();
            let d = json!({
                "tick": hb.tick,
                "game_time": hb.game_time,
                "entity_count": hb.entity_count,
                "hero_count": hb.hero_count,
                "unit_count": hb.unit_count,
                "creep_count": hb.creep_count,
                "render_delay_ms": hb.render_delay_ms,
                "hp_snapshot": hp_snapshot,
                "pos_snapshot": pos_snapshot,
                "in_flight_projectiles": in_flight_projectiles,
            });
            GameEventData {
                topic: topic.clone(),
                msg_type: "heartbeat".to_string(),
                action: "tick".to_string(),
                data: d,
                timestamp_ms,
                payload_bytes: logical_bytes,
        wire_bytes,
            }
        }
        game_event::Payload::ProjectileCreate(m) => {
            let start = m.start_pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            let end = m.end_pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            let splash = m.splash_radius.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let hit = m.hit_radius.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            // P7: pre-declared damage for non-AOE projectiles. 0 when
            // splash > 0 or unset. omfx reads this to schedule optimistic HP
            // update at impact tick.
            let damage = m.damage.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            // Reverse-lookup kind_id (now sequential u16 from omoba-template-ids)
            // → original tag string ("tack"/"bomb"/etc.). Unknown ids fall back
            // to "" so omfx's bullet-colour switch defaults gracefully.
            let kind_str = projectile_id_str(ProjectileKindId(m.kind_id as u16));
            let d = json!({
                "id": m.id as u32,
                "target_id": m.target_id as u32,
                "start_pos": { "x": start.0, "y": start.1 },
                "end_pos": { "x": end.0, "y": end.1 },
                "flight_time_ms": m.flight_time_ms,
                "directional": m.directional,
                "splash_radius": splash,
                "hit_radius": hit,
                "kind": kind_str,
                "damage": damage,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::ProjectileDestroy(m) => {
            let d = json!({ "id": m.id as u32 });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::CreepCreate(m) => {
            let pos = m.pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let max_hp = m.max_hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let move_speed = m.move_speed.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            // Reverse-lookup name_id (sequential CreepId u16) → display name.
            // Unknown id → "" (omfx falls back to entity_type string).
            let name_str = creep_display(CreepId(m.name_id as u16));
            let d = json!({
                "id": m.id as u32,
                "entity_id": m.id as u32,
                "name": name_str,
                "position": { "x": pos.0, "y": pos.1 },
                "hp": hp,
                "max_hp": max_hp,
                "move_speed": move_speed,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::CreepMove(m) => {
            let tgt = m.target.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            // P4: reconstruct extrapolation fields. `velocity`/`start_pos`/`start_tick`/`arrival_tick`
            // are zero for legacy emits (handle_creep_stop freeze) — omfx treats that as
            // "lerp only, no extrapolation".
            let velocity = m.velocity.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let start = m.start_pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            let d = json!({
                "id": m.id as u32,
                "x": tgt.0,
                "y": tgt.1,
                "facing": facing_dequant(m.facing_q),
                "velocity": velocity,
                "arrival_tick": m.arrival_tick,
                "start_pos": { "x": start.0, "y": start.1 },
                "start_tick": m.start_tick,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::CreepHp(m) => {
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "hp": hp,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::CreepSlow(m) => {
            let ms = m.move_speed.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "move_speed": ms,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::CreepStall(m) => {
            let pos = m.pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            let d = json!({
                "id": m.id as u32,
                "x": pos.0,
                "y": pos.1,
                "facing": facing_dequant(m.facing_q),
            });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::EntityFacing(m) => {
            let d = json!({
                "id": m.id as u32,
                "facing": facing_dequant(m.facing_q),
            });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::EntityDeath(m) => {
            let d = json!({ "id": m.id as u32 });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::TowerCreate(m) => {
            let pos = m.pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let max_hp = m.max_hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "entity_id": m.id as u32,
                "name": m.name,
                "kind": m.kind,
                "position": { "x": pos.0, "y": pos.1 },
                "hp": hp,
                "max_hp": max_hp,
                "is_base": false,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::TowerUpgrade(m) => {
            let l0 = m.levels.get(0).copied().unwrap_or(0);
            let l1 = m.levels.get(1).copied().unwrap_or(0);
            let l2 = m.levels.get(2).copied().unwrap_or(0);
            let d = json!({
                "tower_id": m.id as u32,
                "levels": [l0, l1, l2],
            });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::BuffAdd(m) => {
            // remaining_ms=0xFFFF sentinel → -1 remaining (infinite/toggle)
            let remaining = if m.remaining_ms == 0xFFFF { -1.0_f32 } else { m.remaining_ms as f32 / 1000.0 };
            let payload: serde_json::Value = serde_json::from_str(&m.payload_json).unwrap_or(serde_json::Value::Null);
            let d = json!({
                "entity_id": m.entity_id as u32,
                "id": m.entity_id as u32,
                "buff_id": m.buff_id,
                "remaining": remaining,
                "payload": payload,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::BuffRemove(m) => {
            let d = json!({
                "entity_id": m.entity_id as u32,
                "id": m.entity_id as u32,
                "buff_id": m.buff_id,
            });
            GameEventData { data: d, ..default() }
        }
        // P3: HeroStatic → update cache only (no emit to omfx).
        // Level-up / ability-learn happens rarely; omfx will see updated
        // name/level/abilities on the next HeroHot merge (≤ 0.3s later).
        game_event::Payload::HeroStatic(m) => {
            hero_cache.statics.insert(m.id, m.clone());
            return None;
        }
        // P3: HeroHot → merge with cached HeroStatic and emit a legacy-shape
        // hero.stats JSON so omfx needs zero changes. First HeroHot before
        // any HeroStatic arrives → empty static fields (omfx uses `if let
        // Some` per field, so it retains last-known values).
        game_event::Payload::HeroHot(m) => {
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let max_hp = m.max_hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let attack_damage = m.attack_damage.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let armor = m.armor.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let magic_resist = m.magic_resist.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let move_speed = m.move_speed.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let attack_range = m.attack_range.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let attack_interval = m.attack_interval.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);

            let buffs: Vec<serde_json::Value> = m.buffs.iter().map(|b| {
                let remaining = if b.remaining_ms == 0xFFFF { -1.0_f32 } else { b.remaining_ms as f32 / 1000.0 };
                let payload: serde_json::Value = serde_json::from_str(&b.payload_json).unwrap_or(serde_json::Value::Null);
                json!({ "id": b.buff_id, "remaining": remaining, "payload": payload })
            }).collect();

            let mut d = json!({
                "id": m.id as u32,
                "gold": m.gold as i32,
                "hp": hp,
                "max_hp": max_hp,
                "attack_damage": attack_damage,
                "armor": armor,
                "magic_resist": magic_resist,
                "move_speed": move_speed,
                "attack_range": attack_range,
                "attack_interval": attack_interval,
                "buffs": buffs,
            });

            // Merge cached HeroStatic fields (name/title/base stats/level/xp/abilities)
            if let Some(st) = hero_cache.statics.get(&m.id) {
                let ability_levels_map: serde_json::Map<String, serde_json::Value> = st.ability_ids.iter().enumerate().map(|(i, id)| {
                    let lvl = st.ability_levels.get(i).map(|p| p.cur as i32).unwrap_or(0);
                    (id.clone(), json!(lvl))
                }).collect();
                // 推斷 primary_attribute：按 HeroStatic 的 base_str/agi/int 取最大；
                // 目前 server Hero.primary_attribute 的判定也是根據角色設計 (strength/agility/
                // intelligence) — 實務上 base 最大的就是 primary。
                let (p_name, p_val) = [
                    ("strength", st.base_str),
                    ("agility", st.base_agi),
                    ("intelligence", st.base_int),
                ].iter().max_by_key(|(_, v)| *v).copied().unwrap_or(("strength", 0));
                let _ = p_val;
                if let Some(obj) = d.as_object_mut() {
                    obj.insert("name".into(), json!(st.name));
                    obj.insert("title".into(), json!(st.title));
                    obj.insert("strength".into(), json!(st.base_str as i32));
                    obj.insert("agility".into(), json!(st.base_agi as i32));
                    obj.insert("intelligence".into(), json!(st.base_int as i32));
                    obj.insert("primary_attribute".into(), json!(p_name));
                    obj.insert("level".into(), json!(st.level as i32));
                    obj.insert("xp".into(), json!(st.xp as i32));
                    obj.insert("xp_next".into(), json!(st.xp_next as i32));
                    obj.insert("skill_points".into(), json!(st.skill_points as i32));
                    obj.insert("abilities".into(), json!(st.ability_ids));
                    obj.insert("ability_levels".into(), serde_json::Value::Object(ability_levels_map));
                }
            }
            // Inject latest lives (tracked from GameLives events) so omfx's
            // TD-mode detection + HUD lives display keep working from hero.stats.
            if let Some(lives) = hero_cache.latest_lives {
                if let Some(obj) = d.as_object_mut() {
                    obj.insert("lives".into(), json!(lives));
                }
            }

            GameEventData {
                topic: topic.clone(),
                msg_type: "hero".to_string(),
                action: "stats".to_string(),
                data: d,
                timestamp_ms,
                payload_bytes: logical_bytes,
        wire_bytes,
            }
        }
        game_event::Payload::GameRound(m) => {
            let d = json!({
                "round": m.round,
                "total": m.total,
                "is_running": m.is_running,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::GameLives(m) => {
            // P3: cache for later injection into merged hero.stats.
            hero_cache.latest_lives = Some(m.lives);
            let d = json!({ "lives": m.lives });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::GameEnd(m) => {
            let d = json!({ "winner": m.winner });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::GameExplosion(m) => {
            let pos = m.pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            let radius = m.radius.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let d = json!({
                "x": pos.0,
                "y": pos.1,
                "radius": radius,
                "duration": m.duration_ms as f32 / 1000.0,
            });
            GameEventData { data: d, ..default() }
        }
        // P9: HeroCreate / UnitCreate visibility-diff (placeholder — omfx
        // currently spawns from creep.create-style payload; these emit a
        // minimal create JSON so the existing dispatch can still see them).
        game_event::Payload::HeroCreate(m) => {
            let pos = m.pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let max_hp = m.max_hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "entity_id": m.id as u32,
                "name": m.name,
                "title": m.title,
                "position": { "x": pos.0, "y": pos.1 },
                "hp": hp,
                "max_hp": max_hp,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::Payload::UnitCreate(m) => {
            let pos = m.pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let max_hp = m.max_hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "entity_id": m.id as u32,
                "name": m.name,
                "position": { "x": pos.0, "y": pos.1 },
                "hp": hp,
                "max_hp": max_hp,
            });
            GameEventData { data: d, ..default() }
        }
        // P9: catch-all for low-frequency irregular events (init / ack /
        // reject / inventory). Decode the carried JSON bytes back into a
        // serde_json::Value so omfx's existing dispatcher sees the original
        // (msg_type, action, data) shape.
        game_event::Payload::LegacyJson(m) => {
            let data = if m.data_json.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::from_slice(&m.data_json).unwrap_or(serde_json::Value::Null)
            };
            GameEventData {
                topic: topic.clone(),
                msg_type: m.msg_type.clone(),
                action: m.action.clone(),
                data,
                timestamp_ms,
                payload_bytes: logical_bytes,
        wire_bytes,
            }
        }
    };
    Some(out)
}

#[cfg(test)]
mod seq_gap_tests {
    use super::{detect_seq_gap, SeqGapResult};

    #[test]
    fn initial_event_seeds() {
        // No prior seq → any incoming sequence just seeds the tracker.
        assert_eq!(detect_seq_gap(None, 0), SeqGapResult::InitialSeed);
        assert_eq!(detect_seq_gap(None, 42), SeqGapResult::InitialSeed);
    }

    #[test]
    fn contiguous_is_ok() {
        assert_eq!(detect_seq_gap(Some(0), 1), SeqGapResult::Ok);
        assert_eq!(detect_seq_gap(Some(99), 100), SeqGapResult::Ok);
    }

    #[test]
    fn gap_of_one_detected() {
        // last=5, got=7 → missed #6.
        match detect_seq_gap(Some(5), 7) {
            SeqGapResult::Gap { expected, got, gap_size } => {
                assert_eq!(expected, 6);
                assert_eq!(got, 7);
                assert_eq!(gap_size, 1);
            }
            other => panic!("expected Gap, got {:?}", other),
        }
    }

    #[test]
    fn large_gap_reports_size() {
        match detect_seq_gap(Some(100), 150) {
            SeqGapResult::Gap { expected, got, gap_size } => {
                assert_eq!(expected, 101);
                assert_eq!(got, 150);
                assert_eq!(gap_size, 49);
            }
            other => panic!("expected Gap, got {:?}", other),
        }
    }

    #[test]
    fn backwards_flagged_not_gap() {
        // Duplicate or out-of-order — do not request resync.
        assert_eq!(detect_seq_gap(Some(10), 10), SeqGapResult::Backwards);
        assert_eq!(detect_seq_gap(Some(10), 5), SeqGapResult::Backwards);
    }

    #[test]
    fn streaming_sequence_smoke() {
        // Simulate a short stream and assert gap is detected at the right spot.
        let incoming = [0u64, 1, 2, 3, 5, 6]; // missed #4
        let mut last: Option<u64> = None;
        let mut gap_seen = false;
        for (i, &s) in incoming.iter().enumerate() {
            match detect_seq_gap(last, s) {
                SeqGapResult::InitialSeed => { last = Some(s); }
                SeqGapResult::Ok => { last = Some(s); }
                SeqGapResult::Backwards => {}
                SeqGapResult::Gap { expected, got, gap_size } => {
                    assert_eq!(i, 4, "gap at index {}", i);
                    assert_eq!(expected, 4);
                    assert_eq!(got, 5);
                    assert_eq!(gap_size, 1);
                    last = Some(s);
                    gap_seen = true;
                }
            }
        }
        assert!(gap_seen, "expected to observe one gap in the stream");
    }
}
