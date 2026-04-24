use anyhow::Result;
use log::*;
use prost::Message;
use serde_json::json;
use tokio::sync::mpsc;
use tokio::io::{ReadHalf, WriteHalf};
use std::sync::Arc;
use tokio::sync::Mutex;

use tokio_kcp::{KcpConfig, KcpStream, KcpNoDelayConfig};

use super::framing::*;
use super::game_proto::*;
use crate::quant::{facing_dequant, fixed_dequant, pos_dequant};

/// KCP client for communicating with the omb game server.
pub struct KcpClient {
    player_name: String,
    writer: Arc<Mutex<WriteHalf<KcpStream>>>,
    event_rx: Option<mpsc::Receiver<GameEventData>>,
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
        Self::spawn_reader(reader, event_tx);

        Ok(Self {
            player_name,
            writer,
            event_rx: Some(event_rx),
        })
    }

    fn spawn_reader(
        mut reader: ReadHalf<KcpStream>,
        event_tx: mpsc::Sender<GameEventData>,
    ) {
        tokio::spawn(async move {
            loop {
                match read_framed(&mut reader).await {
                    Ok(Some((tag, payload))) => {
                        match tag {
                            TAG_GAME_EVENT => {
                                match GameEvent::decode(payload.as_slice()) {
                                    Ok(event) => {
                                        // P2 binary-protocol path: if the server attached a typed
                                        // prost payload, translate it back to the legacy
                                        // {msg_type, action, data(JSON)} shape the frontend
                                        // already consumes. Full wire-byte savings materialize
                                        // because `data_json` is empty on the wire — the
                                        // JSON shim here only reconstructs the in-memory form.
                                        let parsed = if let Some(tp) = event.typed_payload.as_ref() {
                                            let wire_bytes = event.data_json.len();
                                            translate_typed_payload(tp, &event, wire_bytes)
                                        } else {
                                            let payload_bytes = event.data_json.len();
                                            let data = if event.data_json.is_empty() {
                                                serde_json::Value::Null
                                            } else {
                                                serde_json::from_slice(&event.data_json)
                                                    .unwrap_or(serde_json::Value::Null)
                                            };
                                            GameEventData {
                                                topic: event.topic,
                                                msg_type: event.msg_type,
                                                action: event.action,
                                                data,
                                                timestamp_ms: event.timestamp_ms,
                                                payload_bytes,
                                            }
                                        };

                                        if event_tx.send(parsed).await.is_err() {
                                            break;
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
) -> GameEventData {
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

    match tp {
        game_event::TypedPayload::Heartbeat(hb) => {
            let hp_snapshot: Vec<serde_json::Value> = hb.hp_snapshot.iter().map(|e| {
                let hp = e.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
                json!({ "i": e.id as u32, "h": hp })
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
            let d = json!({
                "id": m.id as u32,
                "x": tgt.0,
                "y": tgt.1,
                "facing": facing_dequant(m.facing_q),
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
        game_event::TypedPayload::GameRound(m) => {
            let d = json!({
                "round": m.round,
                "total": m.total,
                "is_running": m.is_running,
            });
            GameEventData { data: d, ..default() }
        }
        game_event::TypedPayload::GameLives(m) => {
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
    }
}
