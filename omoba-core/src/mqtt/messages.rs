//! MQTT message format definitions

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// MQTT message format (matches backend MqttMsg)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MqttMessage {
    pub topic: String,
    pub msg: String,
    pub time: SystemTime,
}

/// Player data format (matches backend PlayerData)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerData {
    pub name: String,
    #[serde(rename = "t")]
    pub msg_type: String,
    #[serde(rename = "a")]
    pub action: String,
    #[serde(rename = "d")]
    pub data: serde_json::Value,
}

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

/// Screen response format
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScreenResponse {
    #[serde(rename = "t")]
    pub msg_type: String,
    #[serde(rename = "d")]
    pub data: ScreenData,
}

/// Screen data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScreenData {
    pub area: Option<ScreenArea>,
    pub entities: Option<Vec<NetworkEntity>>,
    pub players: Option<Vec<PlayerState>>,
    pub projectiles: Option<Vec<ProjectileData>>,
    pub terrain: Option<Vec<TerrainData>>,
    #[serde(default)]
    pub timestamp: u64,
}

/// Screen area bounds
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScreenArea {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

/// Network entity data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkEntity {
    pub id: u32,
    pub entity_type: String,
    pub position: (f32, f32),
    pub health: Option<(f32, f32)>,
    #[serde(default)]
    pub state: String,
}

/// Projectile data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectileData {
    pub id: u32,
    pub projectile_type: String,
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub owner: String,
}

/// Terrain data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TerrainData {
    pub position: (f32, f32),
    pub terrain_type: String,
    pub properties: serde_json::Value,
}

/// Test response format
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestResponse {
    pub command: String,
    pub success: bool,
    pub data: serde_json::Value,
    pub timestamp: u64,
    pub execution_time_ms: u64,
}

// ============================================================================
// 後端廣播訊息格式 (Backend Broadcast Message Formats)
// ============================================================================

/// 後端廣播訊息格式 (matches backend MqttMsg format)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BroadcastMessage {
    #[serde(rename = "t")]
    pub msg_type: String,
    #[serde(rename = "a")]
    pub action: String,
    #[serde(rename = "d")]
    pub data: serde_json::Value,
}

/// 位置資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PositionData {
    pub x: f32,
    pub y: f32,
}

/// 英雄創建資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HeroCreateData {
    pub entity_id: u32,
    pub hero_id: String,
    pub name: String,
    pub title: String,
    pub level: u32,
    pub position: PositionData,
    pub hp: f32,
    pub max_hp: f32,
    pub move_speed: f32,
}

/// 單位創建資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnitCreateData {
    pub entity_id: u32,
    pub unit_id: String,
    pub name: String,
    pub unit_type: String,
    pub position: PositionData,
    pub hp: f32,
    pub max_hp: f32,
    pub move_speed: f32,
}

/// 小兵創建資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreepCreateData {
    pub id: u32,
    #[serde(default)]
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub mhp: f32,
}

/// 移動資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MoveData {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

/// 刪除資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeleteData {
    pub id: u32,
}

/// 心跳資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HeartbeatData {
    pub tick: u64,
    pub game_time: f64,
    pub entity_count: u32,
    pub hero_count: u32,
    pub unit_count: u32,
    pub creep_count: u32,
}
