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
    pub payload_bytes: usize,
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
        Self::spawn_reader(reader, event_tx, writer.clone(), player_name.clone());

        Ok(Self {
            player_name,
            writer,
            event_rx: Some(event_rx),
        })
    }

    fn spawn_reader(
        mut reader: ReadHalf<KcpStream>,
        event_tx: mpsc::Sender<GameEventData>,
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
                    Ok(Some((tag, payload))) => {
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
                                        // P2 binary-protocol path: if the server attached a typed
                                        // prost payload, translate it back to the legacy
                                        // {msg_type, action, data(JSON)} shape the frontend
                                        // already consumes. Full wire-byte savings materialize
                                        // because `data_json` is empty on the wire — the
                                        // JSON shim here only reconstructs the in-memory form.
                                        let parsed_opt: Option<GameEventData> = if let Some(tp) = event.typed_payload.as_ref() {
                                            let wire_bytes = event.data_json.len();
                                            translate_typed_payload(tp, &event, wire_bytes, &mut hero_cache)
                                        } else {
                                            let payload_bytes = event.data_json.len();
                                            let data = if event.data_json.is_empty() {
                                                serde_json::Value::Null
                                            } else {
                                                serde_json::from_slice(&event.data_json)
                                                    .unwrap_or(serde_json::Value::Null)
                                            };
                                            Some(GameEventData {
                                                topic: event.topic,
                                                msg_type: event.msg_type,
                                                action: event.action,
                                                data,
                                                timestamp_ms: event.timestamp_ms,
                                                payload_bytes,
                                            })
                                        };

                                        if let Some(parsed) = parsed_opt {
                                            if event_tx.send(parsed).await.is_err() {
                                                break;
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
}

/// Translate a prost typed_payload back into the legacy JSON-shaped
/// `GameEventData` that the existing frontend dispatch expects.
///
/// P2 migration note: this is a temporary JSON shim. The wire-side savings
/// are real (we ship `data_json = []` and only the prost variant); the
/// client-side CPU cost is an extra re-serialization pass. When the frontend
/// learns to consume typed_payload directly, this shim goes away.
fn translate_typed_payload(
    tp: &game_event::TypedPayload,
    event: &GameEvent,
    wire_bytes: usize,
    hero_cache: &mut HeroStatsCache,
) -> Option<GameEventData> {
    // Keep the server-supplied msg_type / action so downstream dispatch
    // (which keys on these strings) routes identically to the JSON path.
    let default = || GameEventData {
        topic: event.topic.clone(),
        msg_type: event.msg_type.clone(),
        action: event.action.clone(),
        data: serde_json::Value::Null,
        timestamp_ms: event.timestamp_ms,
        payload_bytes: wire_bytes,
    };

    let out = match tp {
        game_event::TypedPayload::Heartbeat(hb) => {
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
            });
            GameEventData {
                topic: event.topic.clone(),
                msg_type: "heartbeat".to_string(),
                action: "tick".to_string(),
                data: d,
                timestamp_ms: event.timestamp_ms,
                payload_bytes: wire_bytes,
            }
        }
        game_event::TypedPayload::ProjectileCreate(m) => {
            let start = m.start_pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            let end = m.end_pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            let splash = m.splash_radius.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let hit = m.hit_radius.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            // P7: pre-declared damage for non-AOE projectiles. 0 when
            // splash > 0 or unset. omfx reads this to schedule optimistic HP
            // update at impact tick.
            let damage = m.damage.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "target_id": m.target_id as u32,
                "start_pos": { "x": start.0, "y": start.1 },
                "end_pos": { "x": end.0, "y": end.1 },
                "flight_time_ms": m.flight_time_ms,
                "directional": m.directional,
                "splash_radius": splash,
                "hit_radius": hit,
                "kind": m.kind,
                "damage": damage,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::TypedPayload::ProjectileDestroy(m) => {
            let d = json!({ "id": m.id as u32 });
            GameEventData { data: d, ..default() }
        }
        game_event::TypedPayload::CreepCreate(m) => {
            let pos = m.pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let max_hp = m.max_hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let move_speed = m.move_speed.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "entity_id": m.id as u32,
                "name": m.name,
                "position": { "x": pos.0, "y": pos.1 },
                "hp": hp,
                "max_hp": max_hp,
                "move_speed": move_speed,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::TypedPayload::CreepMove(m) => {
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
        game_event::TypedPayload::CreepHp(m) => {
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "hp": hp,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::TypedPayload::CreepSlow(m) => {
            let ms = m.move_speed.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "move_speed": ms,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::TypedPayload::CreepStall(m) => {
            let pos = m.pos.as_ref().map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q))).unwrap_or((0.0, 0.0));
            let d = json!({
                "id": m.id as u32,
                "x": pos.0,
                "y": pos.1,
                "facing": facing_dequant(m.facing_q),
            });
            GameEventData { data: d, ..default() }
        }
        game_event::TypedPayload::EntityFacing(m) => {
            let d = json!({
                "id": m.id as u32,
                "facing": facing_dequant(m.facing_q),
            });
            GameEventData { data: d, ..default() }
        }
        game_event::TypedPayload::EntityDeath(m) => {
            let d = json!({ "id": m.id as u32 });
            GameEventData { data: d, ..default() }
        }
        game_event::TypedPayload::TowerCreate(m) => {
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
        game_event::TypedPayload::TowerUpgrade(m) => {
            let l0 = m.levels.get(0).copied().unwrap_or(0);
            let l1 = m.levels.get(1).copied().unwrap_or(0);
            let l2 = m.levels.get(2).copied().unwrap_or(0);
            let d = json!({
                "tower_id": m.id as u32,
                "levels": [l0, l1, l2],
            });
            GameEventData { data: d, ..default() }
        }
        game_event::TypedPayload::BuffAdd(m) => {
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
        game_event::TypedPayload::BuffRemove(m) => {
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
        game_event::TypedPayload::HeroStatic(m) => {
            hero_cache.statics.insert(m.id, m.clone());
            return None;
        }
        // P3: HeroHot → merge with cached HeroStatic and emit a legacy-shape
        // hero.stats JSON so omfx needs zero changes. First HeroHot before
        // any HeroStatic arrives → empty static fields (omfx uses `if let
        // Some` per field, so it retains last-known values).
        game_event::TypedPayload::HeroHot(m) => {
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
                topic: event.topic.clone(),
                msg_type: "hero".to_string(),
                action: "stats".to_string(),
                data: d,
                timestamp_ms: event.timestamp_ms,
                payload_bytes: wire_bytes,
            }
        }
        game_event::TypedPayload::GameRound(m) => {
            let d = json!({
                "round": m.round,
                "total": m.total,
                "is_running": m.is_running,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::TypedPayload::GameLives(m) => {
            // P3: cache for later injection into merged hero.stats.
            hero_cache.latest_lives = Some(m.lives);
            let d = json!({ "lives": m.lives });
            GameEventData { data: d, ..default() }
        }
        game_event::TypedPayload::GameEnd(m) => {
            let d = json!({ "winner": m.winner });
            GameEventData { data: d, ..default() }
        }
        game_event::TypedPayload::GameExplosion(m) => {
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
        // Catch-all: preserve legacy JSON form if an unknown variant slips through.
        #[allow(unreachable_patterns)]
        _other => {
            warn!("typed_payload variant not yet handled on client");
            default()
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
