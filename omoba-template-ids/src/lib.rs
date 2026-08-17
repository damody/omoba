//! Build-time 產生的 template ids。
//!
//! Source of truth：`scripts/lua_data` 下的 Lua builders。
//! Design：`docs/plans/2026-04-25-template-id-codegen-design.md`。
//!
//! 每個 category（tower、hero、ability、buff、summon、creep、projectile_kind）
//! 都有自己的 `#[repr(transparent)]` newtype 包住 `u16`。Id 0 保留為
//! UNSPECIFIED。Forward lookup（`*_by_name`）與 reverse lookup（`*_id_str`、
//! `*_display`）都會產生為 match statements。

#![allow(clippy::too_many_lines)]

pub use omoba_sim::Fixed64;

#[cfg(feature = "runtime-lua-content")]
pub(crate) mod lua_content;
#[cfg(feature = "runtime-lua-content")]
pub mod runtime_content;
pub mod td_rounds;

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
    pub placement_radius: Fixed64,
    pub hp: Fixed64,
    pub turn_speed_deg: Fixed64,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TowerRenderModeC {
    BaseBarrel = 0,
    AnimatedArea = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TowerRotationModeC {
    Targeted = 0,
    Fixed = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TowerBarrelLayoutC {
    Single = 0,
    RadialCountVariants = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TowerRecoilModeC {
    Directional = 0,
    ScalePulse = 1,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerRenderPointConst {
    pub x: Fixed64,
    pub y: Fixed64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerRenderAnimationConst {
    pub fps: Fixed64,
    pub loop_animation: bool,
    pub fire_fps: Fixed64,
    pub fire_once: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerBarrelVariantConst {
    pub min_path: u8,
    pub min_level: u8,
    pub count: u16,
    pub image: &'static str,
    pub frames: &'static [&'static str],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerRecoilConst {
    pub mode: TowerRecoilModeC,
    pub distance: Fixed64,
    pub scale: Fixed64,
    pub duration_ms: u32,
    pub return_ms: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerRenderMetadataConst {
    pub render_mode: TowerRenderModeC,
    pub base: &'static str,
    pub barrel: &'static str,
    pub visual_size: Fixed64,
    pub barrel_frames: &'static [&'static str],
    pub body_frames: &'static [&'static str],
    pub barrel_animation: TowerRenderAnimationConst,
    pub body_animation: TowerRenderAnimationConst,
    pub rotation_mode: TowerRotationModeC,
    pub barrel_layout: TowerBarrelLayoutC,
    pub barrel_variants: &'static [TowerBarrelVariantConst],
    pub barrel_offset: TowerRenderPointConst,
    pub barrel_pivot: TowerRenderPointConst,
    pub muzzle_offset: TowerRenderPointConst,
    pub default_angle_deg: Fixed64,
    pub recoil: TowerRecoilConst,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AttackTimingConst {
    pub windup: u16,
    pub backswing: u16,
}

impl AttackTimingConst {
    pub const fn is_valid(self) -> bool {
        self.windup as u32 + self.backswing as u32 == 1000
    }
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

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HeroRenderModeC {
    Model3d = 1,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct HeroAnimationSourceConst {
    pub key: &'static str,
    pub model: &'static str,
    pub animation: &'static str,
    pub duration_ticks: Fixed64,
    pub ticks_per_second: Fixed64,
    pub timeline_offset_ticks: Fixed64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct HeroAnimationBindingConst {
    pub action: &'static str,
    pub source: &'static str,
    pub start_tick: Fixed64,
    pub end_tick: Fixed64,
    pub repeat_start_tick: Fixed64,
    pub has_impact_tick: bool,
    pub impact_tick: Fixed64,
    pub loop_animation: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct HeroRenderMetadataConst {
    pub render_mode: HeroRenderModeC,
    pub model: &'static str,
    pub texture: &'static str,
    pub scale: Fixed64,
    pub pitch_offset_deg: Fixed64,
    pub roll_offset_deg: Fixed64,
    pub yaw_offset_deg: Fixed64,
    pub z_offset: Fixed64,
    pub muzzle_bone: &'static str,
    pub animation_sources: &'static [HeroAnimationSourceConst],
    pub animations: &'static [HeroAnimationBindingConst],
}

/// Creep / Enemy intrinsic stats — 對應 templates.lua creeps[i]。
/// `enemy_type`：0=caster, 1=melee, 2=ranged, 3=boss。
/// `ai_type`：0=defensive, 1=aggressive, 2=patrol, 3=guard, 4=passive, 5=berserker。
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

/// Generated authoritative TD layer graph. Strings remain stable content ids;
/// runtime state stores them directly so snapshots and diagnostics are readable.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TdLayerMetadataConst {
    pub id: &'static str,
    pub label: &'static str,
    pub hp: u32,
    pub move_speed: u32,
    pub children: &'static [&'static str],
    pub cash: u32,
    pub leak_value: u32,
    pub properties: u32,
    pub accepted_damage: u32,
    pub regrow_eligible: bool,
    pub fortified_eligible: bool,
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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ActiveAbilityConst {
    pub ability_id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub cooldown: Fixed64,
    pub duration: Fixed64,
    pub pulse_interval: Fixed64,
    pub pulse_count: u16,
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
    pub active_ability: Option<ActiveAbilityConst>,
}

/// shipped story content 的 dependency-light generated data tree。
///
/// Runtime crates 可以把它轉接成自己的 local structs，不需讀取 JSON 或 Lua source files，
/// 也不需要讓本 crate 依賴 runtime crate types。
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

pub fn active_tower_stats(id: TowerId) -> Option<&'static TowerStats> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::tower_stats(id) {
            return Some(value);
        }
    }
    tower_stats(id)
}

pub fn active_tower_display(id: TowerId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::tower_display(id) {
            return value;
        }
    }
    tower_display(id)
}

pub fn active_tower_render_metadata(id: TowerId) -> Option<&'static TowerRenderMetadataConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::tower_render_metadata(id) {
            return Some(value);
        }
    }
    tower_render_metadata(id)
}

pub fn active_tower_attack_timing(id: TowerId) -> Option<AttackTimingConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::tower_attack_timing(id) {
            return Some(value);
        }
    }
    tower_attack_timing(id)
}

pub fn active_tower_upgrades(id: TowerId) -> Option<&'static [&'static [UpgradeDefConst]]> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::tower_upgrades(id) {
            return Some(value);
        }
    }
    tower_upgrades(id)
}

pub fn active_hero_stats(id: HeroId) -> Option<&'static HeroStats> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_stats(id) {
            return Some(value);
        }
    }
    hero_stats(id)
}

pub fn active_hero_display(id: HeroId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_display(id) {
            return value;
        }
    }
    hero_display(id)
}

pub fn active_hero_title(id: HeroId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_title(id) {
            return value;
        }
    }
    hero_title(id)
}

pub fn active_hero_portrait(id: HeroId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_portrait(id) {
            return value;
        }
    }
    hero_portrait(id)
}

pub fn active_hero_render_metadata(id: HeroId) -> Option<&'static HeroRenderMetadataConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_render_metadata(id) {
            return Some(value);
        }
    }
    hero_render_metadata(id)
}

pub fn active_hero_abilities(id: HeroId) -> &'static [AbilityId] {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_abilities(id) {
            return value;
        }
    }
    hero_abilities(id)
}

pub fn active_hero_attack_timing(id: HeroId) -> Option<AttackTimingConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_attack_timing(id) {
            return Some(value);
        }
    }
    hero_attack_timing(id)
}

pub fn active_creep_stats(id: CreepId) -> Option<&'static CreepStats> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::creep_stats(id) {
            return Some(value);
        }
    }
    creep_stats(id)
}

pub fn active_creep_display(id: CreepId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::creep_display(id) {
            return value;
        }
    }
    creep_display(id)
}

pub fn active_creep_attack_timing(id: CreepId) -> Option<AttackTimingConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::creep_attack_timing(id) {
            return Some(value);
        }
    }
    creep_attack_timing(id)
}

pub fn active_td_layer_catalog() -> &'static [TdLayerMetadataConst] {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::td_layer_catalog() {
            return value;
        }
    }
    td_layer_catalog()
}

pub fn active_td_layer_by_name(id: &str) -> Option<&'static TdLayerMetadataConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::td_layer_by_name(id) {
            return Some(value);
        }
    }
    td_layer_by_name(id)
}

pub fn active_td_layer_catalog_digest() -> u64 {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::td_layer_digest() {
            return value;
        }
    }
    TD_LAYER_CATALOG_DIGEST
}

pub fn active_summon_stats(id: SummonId) -> Option<&'static SummonStats> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::summon_stats(id) {
            return Some(value);
        }
    }
    summon_stats(id)
}

pub fn active_summon_display(id: SummonId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::summon_display(id) {
            return value;
        }
    }
    summon_display(id)
}

pub fn active_summon_attack_timing(id: SummonId) -> Option<AttackTimingConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::summon_attack_timing(id) {
            return Some(value);
        }
    }
    summon_attack_timing(id)
}

pub fn active_ability_const(id: AbilityId) -> Option<&'static AbilityConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::ability_const(id) {
            return Some(value);
        }
    }
    ability_const(id)
}

pub fn active_ability_display(id: AbilityId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::ability_display(id) {
            return value;
        }
    }
    ability_display(id)
}

pub fn active_ability_description(id: AbilityId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::ability_description(id) {
            return value;
        }
    }
    ability_description(id)
}

pub fn active_story_by_name(name: &str) -> Option<&'static GeneratedStory> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::story_by_name(name) {
            return Some(value);
        }
    }
    story_by_name(name)
}

pub fn active_story_ids() -> &'static [&'static str] {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::story_ids() {
            return value;
        }
    }
    story_ids()
}

pub fn ensure_runtime_lua_content() -> Result<bool, String> {
    #[cfg(feature = "runtime-lua-content")]
    {
        return runtime_content::ensure_loaded().map(|content| content.is_some());
    }
    #[cfg(not(feature = "runtime-lua-content"))]
    {
        if std::env::var("OMB_LUA_CONTENT")
            .ok()
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
        {
            Err(
                "OMB_LUA_CONTENT=1 but omoba-template-ids was built without runtime-lua-content"
                    .into(),
            )
        } else {
            Ok(false)
        }
    }
}

pub fn runtime_lua_content_generation() -> Result<Option<u64>, String> {
    #[cfg(feature = "runtime-lua-content")]
    {
        return runtime_content::runtime_lua_content_generation();
    }
    #[cfg(not(feature = "runtime-lua-content"))]
    {
        Ok(None)
    }
}

pub fn runtime_lua_content_hash() -> Result<Option<String>, String> {
    #[cfg(feature = "runtime-lua-content")]
    {
        return runtime_content::runtime_lua_content_hash();
    }
    #[cfg(not(feature = "runtime-lua-content"))]
    {
        Ok(None)
    }
}

pub fn lua_hot_reload_enabled() -> bool {
    #[cfg(feature = "runtime-lua-content")]
    {
        return runtime_content::lua_hot_reload_enabled();
    }
    #[cfg(not(feature = "runtime-lua-content"))]
    {
        false
    }
}

#[cfg(feature = "runtime-lua-content")]
pub fn reload_runtime_lua_content_dev(
    expected_hash: Option<&str>,
) -> Result<Option<runtime_content::RuntimeContentInfo>, String> {
    runtime_content::reload_runtime_lua_content_dev(expected_hash)
}

#[cfg(feature = "runtime-lua-content")]
pub fn validate_runtime_lua_content_dev(
) -> Result<Option<runtime_content::RuntimeContentInfo>, String> {
    runtime_content::validate_runtime_lua_content_dev()
}

#[cfg(not(feature = "runtime-lua-content"))]
pub fn reload_runtime_lua_content_dev(_expected_hash: Option<&str>) -> Result<Option<()>, String> {
    Ok(None)
}

#[cfg(not(feature = "runtime-lua-content"))]
pub fn validate_runtime_lua_content_dev() -> Result<Option<()>, String> {
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_towers_publish_active_ability_metadata() {
        let cases = [
            (TOWER_DART, 2, "dart_heavy_burst", 12, 5120, 0, 0),
            (TOWER_BOMB, 2, "bomb_cluster_overload", 12, 5120, 0, 0),
            (TOWER_ICE, 2, "ice_crystal_nova", 12, 0, 0, 0),
            (TOWER_TACK, 2, "tack_blade_maelstrom", 12, 410, 102, 4),
            (TOWER_BOOMERANG, 1, "boomerang_turbo_charge", 10, 5120, 0, 0),
            (TOWER_ARTY, 2, "arty_fire_at_will", 10, 3072, 512, 6),
            (
                TOWER_CAKE_SPLASH,
                1,
                "cake_dessert_party",
                10,
                5120,
                512,
                10,
            ),
        ];
        for (tower, path, id, cooldown, duration_raw, pulse_raw, pulse_count) in cases {
            let def = active_tower_upgrades(tower).unwrap()[path][3];
            let ability = def.active_ability.expect("active ability");
            assert_eq!(ability.ability_id, id);
            assert_eq!(ability.cooldown, Fixed64::from_i32(cooldown));
            assert_eq!(ability.duration, Fixed64::from_raw(duration_raw));
            assert_eq!(ability.pulse_interval, Fixed64::from_raw(pulse_raw));
            assert_eq!(ability.pulse_count, pulse_count);
        }

        assert!(active_tower_upgrades(TOWER_BOOMERANG).unwrap()[0][0]
            .active_ability
            .is_none());
    }

    #[test]
    fn generated_td_layer_catalog_covers_all_shipped_rounds() {
        let catalog = td_layer_catalog();
        assert_eq!(catalog.len(), 17);
        assert_ne!(TD_LAYER_CATALOG_DIGEST, 0);
        for round_index in 0..td_rounds::round_count() {
            for balloon in td_rounds::round(round_index) {
                let layer = td_layer_by_name(balloon.base).unwrap_or_else(|| {
                    panic!(
                        "round {} missing TD layer {}",
                        round_index + 1,
                        balloon.base
                    )
                });
                assert!(layer.hp > 0);
                assert!(layer.move_speed > 0);
                assert!(layer.accepted_damage & !td_rounds::damage_profile::KNOWN == 0);
                for child in layer.children {
                    assert!(
                        td_layer_by_name(child).is_some(),
                        "{} child {}",
                        layer.id,
                        child
                    );
                }
            }
        }
        assert_eq!(td_layer_by_name("moab").unwrap().children.len(), 4);
        assert_eq!(td_layer_by_name("ceramic").unwrap().leak_value, 104);
    }
}
