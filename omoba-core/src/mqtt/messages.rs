//! MQTT message format definitions (stub - full implementation in Task 4)

use serde::{Deserialize, Serialize};

/// Ability data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AbilityData {
    pub ability_id: String,
    pub level: u8,
    pub cooldown_remaining: f32,
    pub target_position: Option<(f32, f32)>,
    pub target_entity: Option<u32>,
}

/// Summon data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SummonData {
    pub unit_type: String,
    pub position: (f32, f32),
    pub health: f32,
    pub state: String,
}

/// Player state (full state sync)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerState {
    pub name: String,
    pub hero_type: String,
    pub position: (f32, f32),
    pub health: (f32, f32),
    #[serde(default)]
    pub abilities: Vec<AbilityData>,
    #[serde(default)]
    pub summons: Vec<SummonData>,
}
