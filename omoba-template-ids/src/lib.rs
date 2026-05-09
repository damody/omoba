//! Build-time generated template ids.
//!
//! Source of truth: Lua builders under `scripts/lua_data`.
//! Design: `docs/plans/2026-04-25-template-id-codegen-design.md`.
//!
//! Each category (tower, hero, ability, buff, summon, creep, projectile_kind)
//! gets its own `#[repr(transparent)]` newtype wrapping `u16`. Id 0 is reserved
//! as UNSPECIFIED. Forward lookup (`*_by_name`) + reverse lookup (`*_id_str`,
//! `*_display`) are generated as match statements.

#![allow(clippy::too_many_lines)]

pub use omoba_sim::Fixed64;

/// Tower numerical stats, single source of truth — `scripts/lua_data/templates.lua` 的
/// `towers[].{atk, asd_interval, ...}` 透過 build.rs 編譯期生成 `TOWER_*_STATS`
/// const + `tower_stats(id)` lookup。base_content 的 tower scripts 直接 import
/// 對應 const，避免每個塔 script 各自在 .rs 裡寫一份 hardcode 數值。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerStats {
    pub atk: Fixed64,
    pub asd_interval: Fixed64,
    pub range: Fixed64,
    pub bullet_speed: Fixed64,
    pub splash_radius: Fixed64,
    pub hit_radius: Fixed64,
    pub slow_factor: Fixed64,
    pub slow_duration: Fixed64,
    pub cost: i32,
    pub footprint: Fixed64,
    pub hp: Fixed64,
    pub turn_speed_deg: Fixed64,
}

/// Hero level-growth — 對應 templates.lua heroes[i].level_growth nested object。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LevelGrowth {
    pub strength_per_level: Fixed64,
    pub agility_per_level: Fixed64,
    pub intelligence_per_level: Fixed64,
    pub damage_per_level: Fixed64,
    pub hp_per_level: Fixed64,
    pub mana_per_level: Fixed64,
}

/// Hero intrinsic stats — 對應 templates.lua heroes[i] 全部 stat 欄位。
/// `primary_attribute`：0=strength, 1=agility, 2=intelligence。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct HeroStats {
    pub strength: i32,
    pub agility: i32,
    pub intelligence: i32,
    pub primary_attribute: u8,
    pub attack_range: Fixed64,
    pub base_damage: i32,
    pub base_armor: Fixed64,
    pub base_hp: i32,
    pub base_mana: i32,
    pub move_speed: Fixed64,
    pub turn_speed: Fixed64,
    pub level_growth: LevelGrowth,
}

/// Creep / Enemy intrinsic stats — 對應 templates.lua creeps[i]。
/// `enemy_type`：0=caster, 1=melee, 2=ranged, 3=boss
/// `ai_type`：0=defensive, 1=aggressive, 2=patrol, 3=guard, 4=passive, 5=berserker
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CreepStats {
    pub hp: Fixed64,
    pub armor: Fixed64,
    pub magic_resistance: Fixed64,
    pub damage: Fixed64,
    pub attack_range: Fixed64,
    pub move_speed: Fixed64,
    pub enemy_type: u8,
    pub ai_type: u8,
    pub exp_reward: i32,
    pub gold_reward: i32,
}

/// Summon intrinsic stats — 對應 templates.lua summons[i]。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SummonStats {
    pub hp: Fixed64,
    pub damage: Fixed64,
    pub duration: Fixed64,
    pub move_speed: Fixed64,
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
/// Per-level extras 走 `AbilityConst.extras` (key, [Fixed64; max_level])。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AbilityLevelDataConst {
    pub cooldown: Fixed64,
    pub mana_cost: Fixed64,
    pub cast_time: Fixed64,
    pub range: Fixed64,
}

/// Ability 數值 const — 對應 templates.lua abilities[i]。runtime 由
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
    pub icon: &'static str,
    pub description: &'static str,
    pub levels: &'static [AbilityLevelDataConst],
    pub extras: &'static [(&'static str, &'static [Fixed64])],
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
    pub value: Fixed64,
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

/// Dependency-light generated data tree for shipped story content.
///
/// Runtime crates can adapt this into their local structs without reading JSON or
/// Lua source files and without making this crate depend on runtime crate types.
#[derive(Copy, Clone, Debug)]
pub enum StoryValue {
    Null,
    Bool(bool),
    Number(f64),
    String(&'static str),
    Array(&'static [StoryValue]),
    Object(&'static [(&'static str, StoryValue)]),
}

#[derive(Copy, Clone, Debug)]
pub struct GeneratedStory {
    pub id: &'static str,
    pub entity: StoryValue,
    pub ability: StoryValue,
    pub mission: StoryValue,
    pub map: StoryValue,
}

include!(concat!(env!("OUT_DIR"), "/template_ids_gen.rs"));
