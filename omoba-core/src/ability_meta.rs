//! 技能 metadata schema — client 與 server 共享
//!
//! 純資料定義，不含執行期狀態、不依賴 ECS。client 透過 ListAbilities /
//! GetAbilityDetail 取得 `AbilityDef` 用於 tooltip、技能樹 UI 等用途。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vek::Vec2;

/// 技能類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AbilityType {
    Active,
    Toggle,
    Ultimate,
}

/// 目標類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetType {
    None,
    Point,
    Unit,
}

/// 施法類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CastType {
    Instant,
    Channeled,
}

/// 傷害類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DamageType {
    Physical,
    Magical,
    Pure,
    True,
}

/// 單一等級的數值配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AbilityLevelData {
    pub cooldown: f32,
    pub mana_cost: f32,
    #[serde(default)]
    pub cast_time: f32,
    #[serde(default)]
    pub range: f32,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 目標選擇器（宣告式）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "selector", content = "params")]
pub enum TargetSelector {
    SelfUnit,
    Target,
    AllEnemies,
    AllAllies,
    Nearest { enemy: bool, range: f32 },
    InRadius { center: Vec2<f32>, radius: f32, enemy: bool },
    Custom(String),
}

/// 技能預覽效果（宣告式，供 tooltip 與 client 顯示）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EffectSpec {
    Damage {
        target: TargetSelector,
        amount: f32,
        damage_type: DamageType,
    },
    Heal {
        target: TargetSelector,
        amount: f32,
    },
    Buff {
        target: TargetSelector,
        duration: f32,
        modifiers: HashMap<String, f32>,
    },
    Summon {
        unit_type: String,
        count: u32,
        duration: Option<f32>,
    },
    AreaEffect {
        radius: f32,
        duration: f32,
        tick_interval: f32,
        effects: Vec<EffectSpec>,
    },
    Projectile {
        speed: f32,
        on_hit: Vec<EffectSpec>,
    },
    Transform {
        target: TargetSelector,
        transform_id: String,
        duration: Option<f32>,
    },
    StatusModifier {
        target: TargetSelector,
        modifier_type: String,
        value: f32,
        duration: Option<f32>,
    },
}

/// 條件檢查（施放前置條件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub condition_type: ConditionType,
    #[serde(default)]
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "arg")]
pub enum ConditionType {
    HasBuff(String),
    HealthBelow(f32),
    HealthAbove(f32),
    InRange(f32),
    HasMana(f32),
    Custom(String),
}

/// 技能定義 — 純資料，client 與 server 共享。
///
/// client 透過 ListAbilities 查詢取得，用於 tooltip / 技能樹 UI。
/// server 透過 AbilityRegistry 關聯對應的 script handler。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityDef {
    /// 唯一 ID（例：`B01_sniper_mode`、`tower_dart_attack`）
    pub id: String,
    /// 顯示名稱
    pub name: String,
    /// tooltip 描述
    #[serde(default)]
    pub description: String,
    pub ability_type: AbilityType,
    pub target_type: TargetType,
    #[serde(default = "default_cast_type")]
    pub cast_type: CastType,
    /// 圖示資源 ID（client 顯示用）
    #[serde(default)]
    pub icon: Option<String>,
    /// 最高等級
    #[serde(default = "default_max_level")]
    pub max_level: u8,
    /// 各等級數值；key 為 "1"~"max_level" 字串（與既有 JSON 格式相容）
    #[serde(default)]
    pub levels: HashMap<String, AbilityLevelData>,
    /// 效果宣告（tooltip 預覽用，非執行語義）
    #[serde(default)]
    pub effects_preview: Vec<EffectSpec>,
    /// 施放前置條件
    #[serde(default)]
    pub conditions: Vec<Condition>,
    /// 自由擴充欄位（供特定 handler 讀取）
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

fn default_cast_type() -> CastType {
    CastType::Instant
}

fn default_max_level() -> u8 {
    4
}

impl AbilityDef {
    pub fn get_level_data(&self, level: u8) -> Option<&AbilityLevelData> {
        self.levels.get(&level.to_string())
    }
}

/// 技能請求（server 收到 client PlayerCommand 後建構；
/// handler 在 server 端執行時使用）。
///
/// 這裡用 `u64` 作為 entity 抽象 id（server 端可轉 `specs::Entity`、
/// client 端可轉自己的追蹤 id），避免 ECS 型別外洩到 client。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityRequest {
    pub caster: u64,
    pub ability_id: String,
    pub level: u8,
    #[serde(default)]
    pub target_entity: Option<u64>,
    #[serde(default)]
    pub target_position: Option<Vec2<f32>>,
    #[serde(default)]
    pub additional_params: HashMap<String, serde_json::Value>,
}
