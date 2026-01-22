//! MQTT message handler

use log::{info, warn, debug};
use anyhow::Result;
use vek::Vec2;

use crate::state::{GameState, Entity, EntityType};
use crate::mqtt::messages::*;

/// MQTT message handler
#[derive(Debug, Clone, Default)]
pub struct MqttHandler {
    pub messages_received: u64,
    pub messages_processed: u64,
}

impl MqttHandler {
    /// Create new handler
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle incoming MQTT message
    pub fn handle_message(&mut self, topic: &str, payload: &[u8], game_state: &mut GameState) -> Result<()> {
        self.messages_received += 1;

        let payload_str = String::from_utf8_lossy(payload);
        debug!("Received MQTT message - Topic: {}, Payload: {}", topic, payload_str);

        match self.route_message(topic, &payload_str, game_state) {
            Ok(_) => {
                self.messages_processed += 1;
            },
            Err(e) => {
                warn!("Failed to process message - Topic: {}, Error: {}", topic, e);
            }
        }

        Ok(())
    }

    /// Route message to appropriate handler
    fn route_message(&self, topic: &str, payload: &str, game_state: &mut GameState) -> Result<()> {
        if topic == "td/all/res" {
            self.handle_broadcast_message(payload, game_state)
        } else if topic.starts_with("td/") && topic.ends_with("/send") {
            self.handle_player_state_message(payload, game_state)
        } else if topic.starts_with("td/") && topic.ends_with("/screen_response") {
            self.handle_screen_response(payload, game_state)
        } else if topic == "ability_test/response" {
            self.handle_test_response(payload)
        } else {
            debug!("Unknown topic: {}", topic);
            Ok(())
        }
    }

    /// Handle broadcast message from backend
    fn handle_broadcast_message(&self, payload: &str, game_state: &mut GameState) -> Result<()> {
        let msg: BroadcastMessage = serde_json::from_str(payload)?;

        match (msg.msg_type.as_str(), msg.action.as_str()) {
            // 英雄創建
            ("hero", "create") => {
                if let Ok(data) = serde_json::from_value::<HeroCreateData>(msg.data) {
                    let entity = Entity {
                        id: data.entity_id,
                        entity_type: EntityType::Player(data.name.clone()),
                        position: Vec2::new(data.position.x, data.position.y),
                        health: (data.hp, data.max_hp),
                        owner: None,
                    };
                    game_state.upsert_entity(entity);
                    info!("Created hero: {} at ({}, {})", data.name, data.position.x, data.position.y);
                }
            }

            // 單位創建
            ("unit", "create") | ("unit", "C") => {
                if let Ok(data) = serde_json::from_value::<UnitCreateData>(msg.data) {
                    let entity = Entity {
                        id: data.entity_id,
                        entity_type: EntityType::Summon(data.name.clone()),
                        position: Vec2::new(data.position.x, data.position.y),
                        health: (data.hp, data.max_hp),
                        owner: None,
                    };
                    game_state.upsert_entity(entity);
                    debug!("Created unit: {} at ({}, {})", data.name, data.position.x, data.position.y);
                }
            }

            // 小兵創建
            ("creep", "C") => {
                if let Ok(data) = serde_json::from_value::<CreepCreateData>(msg.data) {
                    let entity = Entity {
                        id: data.id,
                        entity_type: EntityType::Creep(data.name.clone()),
                        position: Vec2::new(data.x, data.y),
                        health: (data.hp, data.mhp),
                        owner: None,
                    };
                    game_state.upsert_entity(entity);
                    debug!("Created creep: id={} at ({}, {})", data.id, data.x, data.y);
                }
            }

            // 移動更新
            ("creep", "M") | ("unit", "M") | ("hero", "M") => {
                if let Ok(data) = serde_json::from_value::<MoveData>(msg.data) {
                    game_state.update_entity_position(data.id, data.x, data.y);
                }
            }

            // 實體刪除/死亡
            (_, "D") => {
                if let Ok(data) = serde_json::from_value::<DeleteData>(msg.data) {
                    game_state.remove_entity(data.id);
                    debug!("Removed entity: id={}", data.id);
                }
            }

            // 心跳
            ("heartbeat", "tick") => {
                if let Ok(data) = serde_json::from_value::<HeartbeatData>(msg.data) {
                    game_state.game_time = data.game_time as f32;
                    debug!("Heartbeat: tick={}, entities={}", data.tick, data.entity_count);
                }
            }

            // 其他未處理的訊息
            _ => {
                debug!("Unhandled broadcast: type={}, action={}", msg.msg_type, msg.action);
            }
        }

        Ok(())
    }

    /// Handle player state message
    fn handle_player_state_message(&self, payload: &str, game_state: &mut GameState) -> Result<()> {
        if let Ok(player_data) = serde_json::from_str::<PlayerData>(payload) {
            match player_data.msg_type.as_str() {
                "position" => {
                    if let (Some(x), Some(y)) = (
                        player_data.data.get("x").and_then(|v| v.as_f64()),
                        player_data.data.get("y").and_then(|v| v.as_f64())
                    ) {
                        game_state.update_player_position(&player_data.name, x as f32, y as f32);
                    }
                },
                "health" => {
                    if let (Some(current), Some(max)) = (
                        player_data.data.get("current").and_then(|v| v.as_f64()),
                        player_data.data.get("max").and_then(|v| v.as_f64())
                    ) {
                        game_state.update_player_health(&player_data.name, current as f32, max as f32);
                    }
                },
                _ => {
                    debug!("Unknown player data type: {}", player_data.msg_type);
                }
            }
        }

        Ok(())
    }

    /// Handle screen response message
    fn handle_screen_response(&self, payload: &str, game_state: &mut GameState) -> Result<()> {
        if let Ok(response) = serde_json::from_str::<ScreenResponse>(payload) {
            // Update viewport
            if let Some(area) = &response.data.area {
                game_state.viewport.center.x = (area.min_x + area.max_x) / 2.0;
                game_state.viewport.center.y = (area.min_y + area.max_y) / 2.0;
                game_state.viewport.width = area.max_x - area.min_x;
                game_state.viewport.height = area.max_y - area.min_y;
            }

            // Update entities
            if let Some(entities) = &response.data.entities {
                for net_entity in entities {
                    let entity = Entity {
                        id: net_entity.id,
                        entity_type: match net_entity.entity_type.as_str() {
                            "player" => EntityType::Player("unknown".to_string()),
                            "summon" => EntityType::Summon(net_entity.entity_type.clone()),
                            "creep" => EntityType::Creep(net_entity.entity_type.clone()),
                            "tower" => EntityType::Tower,
                            "projectile" => EntityType::Projectile,
                            _ => EntityType::Effect,
                        },
                        position: Vec2::new(net_entity.position.0, net_entity.position.1),
                        health: net_entity.health.unwrap_or((100.0, 100.0)),
                        owner: None,
                    };
                    game_state.upsert_entity(entity);
                }
                debug!("Updated {} entities", entities.len());
            }

            // Update players
            if let Some(players) = &response.data.players {
                for player in players {
                    game_state.sync_player_state(player);
                }
                debug!("Updated {} player states", players.len());
            }
        }

        Ok(())
    }

    /// Handle test response
    fn handle_test_response(&self, payload: &str) -> Result<()> {
        if let Ok(response) = serde_json::from_str::<TestResponse>(payload) {
            info!("Test response - Command: {}, Success: {}", response.command, response.success);
        }
        Ok(())
    }

    /// Get statistics
    pub fn get_stats(&self) -> (u64, u64) {
        (self.messages_received, self.messages_processed)
    }
}
