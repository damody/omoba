//! Game state management

use std::collections::HashMap;
use std::time::SystemTime;
use log::{info, warn, debug};
use vek::Vec2;

use crate::state::entities::*;
use crate::state::viewport::Viewport;
use crate::mqtt::messages::*;

/// Game state observer trait for frontends
pub trait GameStateObserver {
    fn on_state_update(&mut self, state: &GameState);
    fn on_entity_added(&mut self, entity: &Entity);
    fn on_entity_removed(&mut self, entity_id: u32);
}

/// Main game state container
#[derive(Debug, Clone)]
pub struct GameState {
    pub local_player: LocalPlayer,
    pub other_players: HashMap<String, PlayerState>,
    pub entities: HashMap<u32, Entity>,
    pub last_update: SystemTime,
    pub sync_errors: u64,
    pub viewport: Viewport,
    pub game_time: f32,
    pub is_paused: bool,
    pub time_scale: f32,
}

impl GameState {
    /// Create new game state
    pub fn new(player_name: String, hero_type: String) -> Self {
        let local_player = LocalPlayer::new(player_name.clone(), hero_type.clone());

        info!("Initialize game state - Player: {}, Hero: {}", player_name, hero_type);

        Self {
            local_player,
            other_players: HashMap::new(),
            entities: HashMap::new(),
            last_update: SystemTime::now(),
            sync_errors: 0,
            viewport: Viewport::default(),
            game_time: 0.0,
            is_paused: false,
            time_scale: 1.0,
        }
    }

    /// Update player position
    pub fn update_player_position(&mut self, player_name: &str, x: f32, y: f32) {
        if player_name == self.local_player.name {
            self.local_player.position = Vec2::new(x, y);
            debug!("Update local player position: ({}, {})", x, y);
        } else {
            if let Some(player) = self.other_players.get_mut(player_name) {
                player.position = (x, y);
            }
            debug!("Update player {} position: ({}, {})", player_name, x, y);
        }

        self.last_update = SystemTime::now();
    }

    /// Update player health
    pub fn update_player_health(&mut self, player_name: &str, current: f32, max: f32) {
        if player_name == self.local_player.name {
            self.local_player.health = (current, max);
            debug!("Update local player health: {}/{}", current, max);
        } else {
            if let Some(player) = self.other_players.get_mut(player_name) {
                player.health = (current, max);
            }
        }

        self.last_update = SystemTime::now();
    }

    /// Sync player state from server
    pub fn sync_player_state(&mut self, player_state: &PlayerState) {
        if player_state.name == self.local_player.name {
            let server_pos = Vec2::new(player_state.position.0, player_state.position.1);
            let pos_diff = (self.local_player.position - server_pos).magnitude();

            if pos_diff > 5.0 {
                warn!("Position sync difference too large: local {:?}, server {:?}",
                      self.local_player.position, server_pos);
                self.sync_errors += 1;
            }

            self.local_player.position = server_pos;
            self.local_player.health = player_state.health;

            debug!("Synced local player state");
        } else {
            self.other_players.insert(player_state.name.clone(), player_state.clone());
            debug!("Updated other player state: {}", player_state.name);
        }

        self.last_update = SystemTime::now();
    }

    /// Add or update entity
    pub fn upsert_entity(&mut self, entity: Entity) {
        self.entities.insert(entity.id, entity);
        self.last_update = SystemTime::now();
    }

    /// Update entity position
    pub fn update_entity_position(&mut self, id: u32, x: f32, y: f32) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.position = Vec2::new(x, y);
            self.last_update = SystemTime::now();
        }
    }

    /// Remove entity
    pub fn remove_entity(&mut self, entity_id: u32) {
        self.entities.remove(&entity_id);
        self.last_update = SystemTime::now();
    }

    /// Update cooldowns
    pub fn update_cooldowns(&mut self, delta_time: f32) {
        if self.is_paused {
            return;
        }

        let dt = delta_time * self.time_scale;
        self.game_time += dt;

        for ability in &mut self.local_player.abilities {
            if ability.cooldown_remaining > 0.0 {
                ability.cooldown_remaining -= dt;
                if ability.cooldown_remaining <= 0.0 {
                    ability.cooldown_remaining = 0.0;
                    ability.is_available = true;
                }
            }
        }

        for item in &mut self.local_player.items {
            if item.cooldown_remaining > 0.0 {
                item.cooldown_remaining -= dt;
                if item.cooldown_remaining <= 0.0 {
                    item.cooldown_remaining = 0.0;
                    item.is_available = true;
                }
            }
        }
    }

    /// Set time scale (for debug)
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.clamp(0.0, 4.0);
        info!("Time scale set to: {}x", self.time_scale);
    }

    /// Toggle pause
    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
        info!("Game {}", if self.is_paused { "paused" } else { "resumed" });
    }

    /// Get status summary
    pub fn get_status_summary(&self) -> String {
        format!(
            "Player: {} ({}) | Pos: ({:.1}, {:.1}) | HP: {:.0}/{:.0} | Time: {:.1}s | Scale: {}x",
            self.local_player.name,
            self.local_player.hero_type,
            self.local_player.position.x,
            self.local_player.position.y,
            self.local_player.health.0,
            self.local_player.health.1,
            self.game_time,
            self.time_scale
        )
    }

    /// Check if state has valid data
    pub fn has_valid_data(&self) -> bool {
        !self.local_player.name.is_empty()
            || !self.other_players.is_empty()
            || !self.entities.is_empty()
    }
}
