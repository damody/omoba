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

include!(concat!(env!("OUT_DIR"), "/template_ids_gen.rs"));
