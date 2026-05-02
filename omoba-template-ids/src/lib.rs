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

/// Tower numerical stats, single source of truth — `omb/Story/templates.json` 的
/// `towers[].{atk, asd_interval, ...}` 透過 build.rs 編譯期生成 `TOWER_*_STATS`
/// const + `tower_stats(id)` lookup。base_content 的 tower scripts 直接 import
/// 對應 const，避免每個塔 script 各自在 .rs 裡寫一份 hardcode 數值。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerStats {
    pub atk: f32,
    pub asd_interval: f32,
    pub range: f32,
    pub bullet_speed: f32,
    pub splash_radius: f32,
    pub hit_radius: f32,
    pub slow_factor: f32,
    pub slow_duration: f32,
    pub cost: i32,
    pub footprint: f32,
    pub hp: f32,
    pub turn_speed_deg: f32,
}

/// Hero level-growth — 對應 templates.json heroes[i].level_growth nested object。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LevelGrowth {
    pub strength_per_level: f32,
    pub agility_per_level: f32,
    pub intelligence_per_level: f32,
    pub damage_per_level: f32,
    pub hp_per_level: f32,
    pub mana_per_level: f32,
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
    pub attack_range: f32,
    pub base_damage: i32,
    pub base_armor: f32,
    pub base_hp: i32,
    pub base_mana: i32,
    pub move_speed: f32,
    pub turn_speed: f32,
    pub level_growth: LevelGrowth,
}

/// Creep / Enemy intrinsic stats — 對應 templates.json creeps[i]。
/// `enemy_type`：0=caster, 1=melee, 2=ranged, 3=boss
/// `ai_type`：0=defensive, 1=aggressive, 2=patrol, 3=guard, 4=passive, 5=berserker
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CreepStats {
    pub hp: f32,
    pub armor: f32,
    pub magic_resistance: f32,
    pub damage: f32,
    pub attack_range: f32,
    pub move_speed: f32,
    pub enemy_type: u8,
    pub ai_type: u8,
    pub exp_reward: i32,
    pub gold_reward: i32,
}

/// Summon intrinsic stats — 對應 templates.json summons[i]。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SummonStats {
    pub hp: f32,
    pub damage: f32,
    pub duration: f32,
    pub move_speed: f32,
}

include!(concat!(env!("OUT_DIR"), "/template_ids_gen.rs"));
