//! Player action simulator

use serde_json;
use rand::{rng, Rng};
use log::{info, debug};
use anyhow::Result;
use vek::Vec2;

use crate::input::commands::*;

/// Player simulator for automated actions
#[derive(Debug, Clone)]
pub struct PlayerSimulator {
    pub player_name: String,
    pub hero_type: String,
    pub current_position: Vec2<f32>,
    pub action_history: Vec<PlayerAction>,
    pub auto_mode_enabled: bool,
}

impl PlayerSimulator {
    /// Create new player simulator
    pub fn new(player_name: String, hero_type: String) -> Self {
        info!("Create player simulator - Player: {}, Hero: {}", player_name, hero_type);

        Self {
            player_name,
            hero_type,
            current_position: Vec2::new(400.0, 300.0),
            action_history: Vec::new(),
            auto_mode_enabled: false,
        }
    }

    /// Perform player action
    pub fn perform_action(&mut self, action: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        debug!("Perform action: {} - Params: {}", action, params);

        let result = match action {
            "move" => self.handle_move(params.clone())?,
            "cast_ability" => self.handle_cast_ability(params.clone())?,
            "attack" => self.handle_attack(params.clone())?,
            _ => {
                return Err(anyhow::anyhow!("Unknown action type: {}", action));
            }
        };

        // Record action
        let action_record = PlayerAction {
            action_type: action.to_string(),
            timestamp: std::time::SystemTime::now(),
            parameters: params,
            result: Some(result.clone()),
        };

        self.action_history.push(action_record);

        if self.action_history.len() > 100 {
            self.action_history.remove(0);
        }

        Ok(result)
    }

    /// Handle move action
    fn handle_move(&mut self, params: serde_json::Value) -> Result<serde_json::Value> {
        let move_params: MoveParams = serde_json::from_value(params)?;

        let target_pos = Vec2::new(move_params.target_x, move_params.target_y);
        let distance = (target_pos - self.current_position).magnitude();

        let max_move_distance = 200.0;
        let actual_target = if distance > max_move_distance {
            let direction = (target_pos - self.current_position).normalized();
            self.current_position + direction * max_move_distance
        } else {
            target_pos
        };

        self.current_position = actual_target;

        Ok(serde_json::json!({
            "x": actual_target.x,
            "y": actual_target.y,
            "distance_moved": distance.min(max_move_distance),
            "success": true
        }))
    }

    /// Handle cast ability action
    fn handle_cast_ability(&mut self, params: serde_json::Value) -> Result<serde_json::Value> {
        let cast_params: CastAbilityParams = serde_json::from_value(params)?;

        if !self.is_ability_valid(&cast_params.ability_id) {
            return Err(anyhow::anyhow!("Ability {} not valid for hero {}", cast_params.ability_id, self.hero_type));
        }

        let cast_position = cast_params.target_position
            .unwrap_or((self.current_position.x, self.current_position.y));

        Ok(serde_json::json!({
            "ability_id": cast_params.ability_id,
            "level": cast_params.level.unwrap_or(1),
            "cast_position": cast_position,
            "target_entity": cast_params.target_entity,
            "success": true
        }))
    }

    /// Handle attack action
    fn handle_attack(&mut self, params: serde_json::Value) -> Result<serde_json::Value> {
        let attack_params: AttackParams = serde_json::from_value(params)?;

        let target_pos = Vec2::new(attack_params.target_position.0, attack_params.target_position.1);
        let target_distance = (target_pos - self.current_position).magnitude();

        let max_attack_range = match attack_params.attack_type.as_str() {
            "basic" => 50.0,
            "ranged" => 150.0,
            "ability" => 200.0,
            _ => 50.0,
        };

        let can_attack = target_distance <= max_attack_range;

        Ok(serde_json::json!({
            "target_position": attack_params.target_position,
            "attack_type": attack_params.attack_type,
            "distance": target_distance,
            "in_range": can_attack,
            "success": can_attack
        }))
    }

    /// Check if ability is valid for this hero
    fn is_ability_valid(&self, ability_id: &str) -> bool {
        self.get_hero_abilities().contains(&ability_id.to_string())
    }

    /// Get hero abilities — 走 omoba_template_ids 生成 lookup（單一來源 templates.json）。
    pub fn get_hero_abilities(&self) -> Vec<String> {
        let id = omoba_template_ids::hero_by_name(&self.hero_type).unwrap_or_default();
        omoba_template_ids::hero_abilities(id)
            .iter()
            .map(|aid| aid.as_str().to_string())
            .collect()
    }

    /// Generate random action for auto mode
    pub fn generate_random_action(&self) -> Option<(String, serde_json::Value)> {
        if !self.auto_mode_enabled {
            return None;
        }

        let mut rng = rng();
        let action_type = rng.random_range(0..4);

        match action_type {
            0 => {
                let target_x = self.current_position.x + rng.random_range(-100.0..100.0);
                let target_y = self.current_position.y + rng.random_range(-100.0..100.0);

                Some(("move".to_string(), serde_json::json!({
                    "target_x": target_x.max(0.0).min(800.0),
                    "target_y": target_y.max(0.0).min(600.0)
                })))
            },
            1 => {
                let abilities = self.get_hero_abilities();
                if !abilities.is_empty() {
                    let ability = &abilities[rng.random_range(0..abilities.len())];

                    Some(("cast_ability".to_string(), serde_json::json!({
                        "ability_id": ability,
                        "target_position": [
                            self.current_position.x + rng.random_range(-50.0..50.0),
                            self.current_position.y + rng.random_range(-50.0..50.0)
                        ],
                        "level": 1
                    })))
                } else {
                    None
                }
            },
            2 => {
                let target_x = self.current_position.x + rng.random_range(-80.0..80.0);
                let target_y = self.current_position.y + rng.random_range(-80.0..80.0);

                Some(("attack".to_string(), serde_json::json!({
                    "target_position": [target_x, target_y],
                    "attack_type": "basic"
                })))
            },
            _ => None
        }
    }

    /// Set auto mode
    pub fn set_auto_mode(&mut self, enabled: bool) {
        self.auto_mode_enabled = enabled;
        info!("Player {} auto mode: {}", self.player_name, if enabled { "enabled" } else { "disabled" });
    }

    /// Update position from game state
    pub fn update_position(&mut self, position: Vec2<f32>) {
        self.current_position = position;
    }
}
