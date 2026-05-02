//! Build-time generated template ids.
//!
//! Source of truth: `omb/Story/templates.json`.
//! Design: `docs/plans/2026-04-25-template-id-codegen-design.md`.
//!
//! Each category (tower, hero, ability, buff, summon, creep, projectile_kind)
//! gets its own `#[repr(transparent)]` newtype wrapping `u16`. Id 0 is reserved
//! as UNSPECIFIED. Forward lookup (`*_by_name`) + reverse lookup (`*_id_str`,
//! `*_display`) are generated as match statements.

#![allow(clippy::too_many_lines)]

pub use omoba_sim::Fixed32;

/// Tower numerical stats, single source of truth — `omb/Story/templates.json` 的
/// `towers[].{atk, asd_interval, ...}` 透過 build.rs 編譯期生成 `TOWER_*_STATS`
/// const + `tower_stats(id)` lookup。base_content 的 tower scripts 直接 import
/// 對應 const，避免每個塔 script 各自在 .rs 裡寫一份 hardcode 數值。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerStats {
    pub atk: Fixed32,
    pub asd_interval: Fixed32,
    pub range: Fixed32,
    pub bullet_speed: Fixed32,
    pub splash_radius: Fixed32,
    pub hit_radius: Fixed32,
    pub slow_factor: Fixed32,
    pub slow_duration: Fixed32,
    pub cost: i32,
    pub footprint: Fixed32,
    pub hp: Fixed32,
    pub turn_speed_deg: Fixed32,
}

/// Hero level-growth — 對應 templates.json heroes[i].level_growth nested object。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LevelGrowth {
    pub strength_per_level: Fixed32,
    pub agility_per_level: Fixed32,
    pub intelligence_per_level: Fixed32,
    pub damage_per_level: Fixed32,
    pub hp_per_level: Fixed32,
    pub mana_per_level: Fixed32,
}

/// Hero intrinsic stats — 對應 templates.json heroes[i] 全部 stat 欄位。
/// `primary_attribute`：0=strength, 1=agility, 2=intelligence。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct HeroStats {
    pub strength: i32,
    pub agility: i32,
    pub intelligence: i32,
    pub primary_attribute: u8,
    pub attack_range: Fixed32,
    pub base_damage: i32,
    pub base_armor: Fixed32,
    pub base_hp: i32,
    pub base_mana: i32,
    pub move_speed: Fixed32,
    pub turn_speed: Fixed32,
    pub level_growth: LevelGrowth,
}

/// Creep / Enemy intrinsic stats — 對應 templates.json creeps[i]。
/// `enemy_type`：0=caster, 1=melee, 2=ranged, 3=boss
/// `ai_type`：0=defensive, 1=aggressive, 2=patrol, 3=guard, 4=passive, 5=berserker
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CreepStats {
    pub hp: Fixed32,
    pub armor: Fixed32,
    pub magic_resistance: Fixed32,
    pub damage: Fixed32,
    pub attack_range: Fixed32,
    pub move_speed: Fixed32,
    pub enemy_type: u8,
    pub ai_type: u8,
    pub exp_reward: i32,
    pub gold_reward: i32,
}

/// Summon intrinsic stats — 對應 templates.json summons[i]。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SummonStats {
    pub hp: Fixed32,
    pub damage: Fixed32,
    pub duration: Fixed32,
    pub move_speed: Fixed32,
}

/// Ability 類型 — `omoba_core::ability_meta::AbilityType` 的 const-friendly
/// 鏡像（`#[repr(u8)]` enum，可塞進 const struct）。`From<AbilityTypeC>` 在
/// helper crate (base_content) 端做 mapping。
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AbilityTypeC {
    Active = 0,
    Toggle = 1,
    Ultimate = 2,
    Passive = 3,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CastTypeC {
    Instant = 0,
    Channeled = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TargetTypeC {
    None = 0,
    Point = 1,
    Unit = 2,
}

/// 單一等級的核心數值 — const-friendly mirror of `AbilityLevelData`。
/// Per-level extras 走 `AbilityConst.extras` (key, [Fixed32; max_level])。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AbilityLevelDataConst {
    pub cooldown: Fixed32,
    pub mana_cost: Fixed32,
    pub cast_time: Fixed32,
    pub range: Fixed32,
}

/// Ability 數值 const — 對應 templates.json abilities[i]。runtime 由
/// `build_ability_def_from_const(id, &CONST)` 攤成完整 `AbilityDef`（含
/// HashMap 的 levels / extras）。`extras` 是 sorted-by-key 的 `(name, per_level_values)`
/// pairs，per_level_values.len() 必須等於 max_level。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AbilityConst {
    pub ability_type: AbilityTypeC,
    pub cast_type: CastTypeC,
    pub target_type: TargetTypeC,
    pub max_level: u8,
    pub description: &'static str,
    pub levels: &'static [AbilityLevelDataConst],
    pub extras: &'static [(&'static str, &'static [Fixed32])],
}

/// Tower upgrade effect — `omoba_core::tower_meta::StatOp` 的 const-friendly mirror。
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StatOpC {
    Add = 0,
    Mul = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UpgradeEffectKindC {
    StatMod = 0,
    BehaviorFlag = 1,
}

/// Tower upgrade single effect — POD; runtime 端透過 `From` 轉成
/// `omoba_core::tower_meta::UpgradeEffect`。`StatMod` 用 (key, value, op)；
/// `BehaviorFlag` 用 key 當 flag name，value/op 忽略。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UpgradeEffectConst {
    pub kind: UpgradeEffectKindC,
    pub key: &'static str,
    pub value: Fixed32,
    pub op: StatOpC,
}

/// 一個 upgrade 等級（per tower / per path / per level）。
/// `cost` 必須符合 `omoba_core::tower_meta::upgrade_cost(base, level)` 公式；
/// build.rs 編譯期驗證。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UpgradeDefConst {
    pub name: &'static str,
    pub description: &'static str,
    pub cost: i32,
    pub effects: &'static [UpgradeEffectConst],
}

include!(concat!(env!("OUT_DIR"), "/template_ids_gen.rs"));
