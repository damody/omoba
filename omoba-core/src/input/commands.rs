//! 播放器命令定義

use serde::{Deserialize, Serialize};

/// 移動命令參數
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveParams {
    pub target_x: f32,
    pub target_y: f32,
    pub speed: Option<f32>,
}

/// 施放能力指令參數
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastAbilityParams {
    pub ability_id: String,
    pub target_position: Option<(f32, f32)>,
    pub target_entity: Option<u32>,
    pub level: Option<u8>,
}

/// 攻擊命令參數
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackParams {
    pub target_position: (f32, f32),
    pub attack_type: String,
}

/// 玩家動作記錄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAction {
    pub action_type: String,
    pub timestamp: std::time::SystemTime,
    pub parameters: serde_json::Value,
    pub result: Option<serde_json::Value>,
}
