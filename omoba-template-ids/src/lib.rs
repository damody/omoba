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

#[cfg(all(feature = "compiled-content-only", feature = "runtime-lua-content"))]
compile_error!("compiled-content-only forbids runtime-lua-content; Lua is build-time authoring only");

pub use omoba_sim::Fixed64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommunicationSemanticConst {
    Joke, Challenge, Dance, Laugh, Salute,
    Acknowledge, Applaud, Thanks, Surprise, Resolve, Patience, Celebrate, Regroup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommunicationContentConst {
    pub catalog_id: u16,
    pub id: &'static str,
    pub semantic: CommunicationSemanticConst,
    pub interaction_slot: u8,
    pub wheel_slot: u8,
    pub label: &'static str,
    pub subtitle: &'static str,
    pub duration_raw: u16,
    pub icon: Option<&'static str>,
    pub audio: Option<&'static str>,
    pub animation: Option<&'static str>,
}

#[derive(Clone, Copy, Debug)]
pub struct MobaLaneConst {
    pub id: &'static str,
    pub waypoints: &'static [(i32, i32)],
}

#[derive(Clone, Copy, Debug)]
pub struct MobaMapConst {
    pub base_structures: Option<MobaBaseStructuresConst>,
    pub base_unlock: MobaBaseUnlockConst,
    pub minion_wave: &'static [MobaMinionConst],
    pub wave_interval_seconds: u32,
    pub id: &'static str,
    pub lane_length: i32,
    pub tower_offset: i32,
    pub tower_layers: &'static [i32],
    pub lanes: &'static [MobaLaneConst],
    pub jungle_camps: &'static [MobaJungleConst],
    pub terrain: &'static [MobaTerrainConst],
    pub brushes: &'static [MobaTerrainConst],
    pub presentation_surfaces: &'static [MobaMapSurfaceConst],
}

#[derive(Clone, Copy, Debug)]
pub struct MobaBaseStructuresConst {
    pub minimum_spacing: i32,
    pub inhibitor_offset: i32,
    pub inhibitor_hp: i32,
    pub inhibitor_respawn_seconds: i32,
    pub nexus_tower_offset: i32,
    pub nexus_tower_hp: i32,
    pub nexus_tower_respawn_seconds: i32,
    pub nexus_tower_respawn_hp_percent: i32,
    pub siege_minion_id: &'static str,
    pub super_minion: MobaMinionConst,
}

#[derive(Clone, Copy, Debug)]
pub struct MobaMinionConst {
    pub id: &'static str,
    pub count: u32,
    pub every: u32,
    pub hp: i32,
    pub damage: i32,
    pub attack_range: i32,
    pub move_speed: i32,
    pub attack_interval_ms: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MobaMapSurfaceKind { River = 1, Jungle = 2 }

#[derive(Clone, Copy, Debug)]
pub struct MobaMapSurfaceConst {
    pub id: &'static str,
    pub kind: MobaMapSurfaceKind,
    pub width: i32,
    /// River centerline, or jungle minimum/maximum bounds. Visual only.
    pub points: &'static [(i32, i32)],
}

#[derive(Clone, Copy, Debug)]
pub struct MobaTerrainConst {
    pub id: &'static str,
    pub min: (i32, i32),
    pub max: (i32, i32),
}

#[derive(Clone, Copy, Debug)]
pub struct MobaJungleConst {
    pub death_children: &'static [MobaJungleMemberConst],
    pub additional_objective_buffs: &'static [MobaObjectiveBuffConst],
    pub personal_buff: Option<MobaObjectiveBuffConst>,
    pub objective_buff: Option<MobaObjectiveBuffConst>,
    pub members: &'static [MobaJungleMemberConst],
    pub patience_seconds: i32,
    pub soft_reset_seconds: i32,
    pub id: &'static str,
    pub position: (i32, i32),
    pub hp: i32,
    pub damage: i32,
    pub move_speed: i32,
    pub attack_range: i32,
    pub leash_radius: i32,
    pub attack_interval_seconds: i32,
    pub respawn_seconds: i32,
    pub gold: u32,
    pub xp: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobaObjectiveBuffConst {
    pub minion_aura: Option<MobaMinionAuraConst>,
    pub transfer_on_kill: bool,
    pub buff_id: &'static str,
    pub max_stacks: u32,
    pub duration_seconds: i32,
    pub attack_bonus: i32,
    pub armor_bonus: i32,
    pub lost_on_death: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobaMinionAuraConst {
    pub buff_id: &'static str,
    pub radius: i32,
    pub attack_bonus: i32,
    pub armor_bonus: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobaBaseUnlockConst { AllLaneTowers, AnyLaneTowers }

#[derive(Clone, Copy, Debug)]
pub struct MobaJungleMemberConst {
    pub death_children: &'static [MobaJungleMemberConst],
    pub id: &'static str,
    pub offset: (i32, i32),
    pub hp: i32,
    pub damage: i32,
    pub gold: u32,
    pub xp: u32,
}

impl MobaJungleConst {
    pub fn objective_rewards(&self)->impl Iterator<Item=MobaObjectiveBuffConst>+'_ {
        self.objective_buff.into_iter().chain(self.additional_objective_buffs.iter().copied())
    }
    pub fn home_positions(&self)->impl Iterator<Item=(i32,i32)>+'_ {
        fn collect(children:&[MobaJungleMemberConst],parent:(i32,i32),out:&mut Vec<(i32,i32)>) {
            for child in children {let home=(parent.0+child.offset.0,parent.1+child.offset.1);
                out.push(home);collect(child.death_children,home,out);}
        }
        let mut homes=vec![self.position];
        collect(self.death_children,self.position,&mut homes);
        collect(self.members,self.position,&mut homes);
        homes.into_iter()
    }
}

/// Shared MOBA catalog compiled from Lua. Full price includes recipe
/// materials. Bonuses are quantized once by codegen, not separately per host.
#[derive(Clone, Copy, Debug)]
pub struct MobaItemConst {
    pub catalog_id: u16,
    pub id: &'static str,
    pub name: &'static str,
    pub cost: i32,
    pub atk: Fixed64,
    pub hp: Fixed64,
    pub ms: Fixed64,
    pub armor: Fixed64,
    pub lifesteal: Fixed64,
    pub critical_chance: Fixed64,
    pub unique_passive: Option<MobaItemUniquePassiveConst>,
    pub purchase_group: Option<MobaItemPurchaseGroupConst>,
    pub active: Option<MobaItemActiveConst>,
    pub cooldown: Fixed64,
    pub recipe: &'static [&'static str],
    pub target: MobaItemTargetConst,
}

#[derive(Clone, Copy, Debug)]
pub struct MobaItemUniquePassiveConst {
    pub group: &'static str,
    pub name: &'static str,
    pub priority: u16,
    pub atk: Fixed64,
    pub hp: Fixed64,
    pub ms: Fixed64,
    pub armor: Fixed64,
    pub lifesteal: Fixed64,
    pub critical_chance: Fixed64,
}

#[derive(Clone, Copy, Debug)]
pub struct MobaItemPurchaseGroupConst {pub id:&'static str,pub name:&'static str,pub limit:u8}

/// Authored item target rule. Self ignores attached point and unit bytes.
/// Point and unit reject out of range and never queue or approach.
#[derive(Clone, Copy, Debug)]
pub struct MobaItemTargetConst {
    pub shape: MobaItemTargetShapeConst,
    pub range: Fixed64,
    pub team: MobaItemTeamConst,
    pub requires_visible: bool,
    pub out_of_range: MobaItemOutOfRangeConst,
}

impl MobaItemTargetConst {
    pub const SELF: Self = Self {
        shape: MobaItemTargetShapeConst::Self_,
        range: Fixed64::ZERO,
        team: MobaItemTeamConst::Any,
        requires_visible: false,
        out_of_range: MobaItemOutOfRangeConst::Reject,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobaItemTargetShapeConst {
    Self_,
    Point,
    Unit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobaItemTeamConst {
    Any,
    Enemy,
    Hostile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobaItemOutOfRangeConst {
    Reject,
}

/// Build-time authored effects, executed by the native shared item executor.
#[derive(Clone, Copy, Debug)]
pub enum MobaItemActiveConst {
    SelfControlCleanse,
    ChannelHeal {amount:Fixed64,duration:Fixed64,interval:Fixed64},
    Shield { amount: Fixed64, duration: Fixed64 },
    SprintBuff { ms_bonus: Fixed64, duration: Fixed64 },
    RestoreMana { amount: Fixed64 },
    DamageReduce { percent: Fixed64, duration: Fixed64 },
    HeadshotNext { bonus_damage: Fixed64 },
    PointNova { damage: Fixed64, radius: Fixed64 },
    UnitHex { damage: Fixed64,healing_reduction:Option<MobaHealingReductionConst>,control:Option<MobaItemControlConst> },
}

#[derive(Clone,Copy,Debug)]
pub enum MobaItemControlKindConst {Stun,Root,Silence}
#[derive(Clone,Copy,Debug)]
pub struct MobaItemControlConst {pub kind:MobaItemControlKindConst,pub duration:Fixed64}

#[derive(Clone,Copy,Debug)]
pub struct MobaHealingReductionConst {pub multiplier:Fixed64,pub duration:Fixed64}

/// Generated summoner spell. Balance comes from `templates/moba_summoners.lua`.
/// There is no second rules hash; these fields are part of `CONTENT_CATALOG_DATA_HASH`.
#[derive(Clone, Copy, Debug)]
pub struct SummonerSpellConst {
    pub catalog_id: u16,
    pub id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub cooldown: Fixed64,
    pub target: SummonerTargetConst,
    pub operation: SummonerSpellOperationConst,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummonerTargetConst {
    None,
    Point,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummonerOutOfRangeConst {
    Reject,
    ClampToRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummonerSpellOperationConst {
    PointBlink {
        range: Fixed64,
        out_of_range: SummonerOutOfRangeConst,
    },
    SelfHeal { amount: Fixed64 },
    SelfRestoreMana { amount: Fixed64 },
    SelfShield { amount: Fixed64, duration: Fixed64 },
    SelfControlCleanse,
    SelfSprint { ms_bonus: Fixed64, duration: Fixed64 },
    SelfDamageReduction { reduction: Fixed64, duration: Fixed64 },
}

/// Generated trinket. Balance comes from `templates/moba_trinkets.lua`.
/// There is no second rules hash; these fields are part of `CONTENT_CATALOG_DATA_HASH`.
/// Continuous fields are quantized once. Counts and levels stay integers.
/// This catalog does not place a ward or grant vision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrinketConst {
    pub catalog_id: u16,
    pub id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub placement_range: Fixed64,
    pub initial_charges: u16,
    pub max_charges: u16,
    pub recharge_seconds: Fixed64,
    pub lifetime_seconds: Fixed64,
    pub sight_radius: Fixed64,
    pub detection_level: u16,
    pub stealth_level: u16,
    pub hit_points: u16,
    pub maximum_owned_placements: u16,
    pub direct_attack: TrinketDirectAttackConst,
    pub automatic_acquisition: TrinketAutomaticAcquisitionConst,
}

/// Authored direct-attack policy. Future trinkets add a variant here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrinketDirectAttackConst {
    DisclosedEnemy,
    NotDirectlyAttackable,
}

/// Authored automatic-acquisition policy. Future trinkets add a variant here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrinketAutomaticAcquisitionConst {
    Disabled,
    Enabled,
}

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
    pub moba_loadout: MobaLoadoutConst,
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MobaLoadoutConst {
    pub ranks: [u8; 4],
    pub skill_points: u8,
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

#[repr(u8)]
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum SummonTargetPriority {Nearest,OwnerFocus}

/// Summon intrinsic stats — 對應 templates.lua summons[i]。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SummonStats {
    pub hp: Fixed64,
    pub damage: Fixed64,
    pub duration: Fixed64,
    pub move_speed: Fixed64,
    pub attack_range: Fixed64,
    pub attack_interval: Fixed64,
    pub armor: Fixed64,
    pub magic_resistance: Fixed64,
    pub collision_radius: Fixed64,
    pub vision_radius: Fixed64,
    pub follow_start_distance: Fixed64,
    pub follow_stop_distance: Fixed64,
    pub target_priority: SummonTargetPriority,
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
    pub required_hero_level: u8,
    pub cooldown: Fixed64,
    pub mana_cost: Fixed64,
    pub cast_time: Fixed64,
    pub cast_interrupt_on_damage: bool,
    pub range: Fixed64,
}

/// Exclusive authored host recovery-channel keys, shared with bot admission.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RecoveryChannelConst {
    pub amount_key: &'static str,
    pub duration_key: &'static str,
    pub interval_key: &'static str,
}

/// Ability values and sorted per-rank extras projected from authoring.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AbilityConst {
    /// Validated executable friendly cleansing capability, not tooltip inference.
    pub cleanses_allies: bool,
    pub ability_type: AbilityTypeC,
    pub cast_type: CastTypeC,
    pub target_type: TargetTypeC,
    pub max_level: u8,
    /// Validated authored zone policy; omitted legacy policies compile to false.
    pub zone_independent: bool,
    pub facing_formation_rows: u16,
    /// Quantized executable source-sector half-angle for every rank; zero for non-cones.
    pub cone_half_angle_ticks: &'static [u16],
    /// Executable nonmovement capsule width per rank; zero for other effects.
    pub line_widths: &'static [Fixed64],
    /// Executable exclusive host channel declaration; never a tooltip inference.
    pub recovery_channel: Option<RecoveryChannelConst>,
    /// Shared semantic geometry, identical to build-time native aiming metadata.
    pub aim_preview_signature: &'static str,
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
    fn minion_cadence_generated_profiles_preserve_authored_milliseconds() {
        let map=moba_map_by_name("three_lane_training").unwrap();assert_eq!(map.minion_wave.iter().map(|m|m.attack_interval_ms).collect::<Vec<_>>(),vec![1000,1500,2000]);
        assert_eq!(map.base_structures.unwrap().super_minion.attack_interval_ms,1000);
        for map in MOBA_MAPS {for row in map.minion_wave.iter().chain(map.base_structures.iter().map(|s|&s.super_minion)){assert!((1..=10000).contains(&row.attack_interval_ms));}}
    }

    #[test]
    fn mana_drain_authored_bolt_preserves_damage_cost_range_and_apprentice_clone() {
        let bolt=active_ability_const(ability_by_name("lumen_bolt").unwrap()).unwrap();
        let read=|key|bolt.extras.iter().find(|(name,_)|*name==key).unwrap().1;
        assert_eq!(read("mana_drain"),&[20,30,40,50].map(Fixed64::from_i32));assert_eq!(read("damage"),&[80,125,170,215].map(Fixed64::from_i32));
        assert!(bolt.levels.iter().all(|l|l.range==Fixed64::from_i32(600)));assert_eq!(bolt.levels[0].mana_cost,Fixed64::from_i32(45));
        let clone=active_ability_const(ability_by_name("apprentice_bolt").unwrap()).unwrap();assert!(!clone.extras.iter().any(|(key,_)|*key=="mana_drain"));
    }

    #[test]
    fn ally_shield_support_extras_keep_original_heal_cost_range_and_source_clone_independent() {
        let aid=active_ability_const(ability_by_name("lumen_aid").unwrap()).unwrap();
        let read=|key|aid.extras.iter().find(|(name,_)|*name==key).unwrap().1;
        assert_eq!(read("shield"),&[70,100,130,160].map(Fixed64::from_i32));assert_eq!(read("shield_duration"),&[Fixed64::from_i32(2);4]);
        assert_eq!(read("heal"),&[140,220,300,380].map(Fixed64::from_i32));assert_eq!(aid.levels[0].mana_cost,Fixed64::from_i32(90));
        let original=active_ability_const(ability_by_name("lumen_mend").unwrap()).unwrap();assert!(!original.extras.iter().any(|(key,_)|*key=="shield"));
    }

    #[test]
    fn composed_mana_authored_support_preserves_target_range_and_rank_amounts() {
        let id=ability_by_name("lumen_aid").unwrap();let aid=active_ability_const(id).unwrap();
        assert_eq!(aid.target_type,TargetTypeC::Unit);
        assert_eq!(aid.levels.len(),4);
        assert!(aid.levels.iter().all(|level|level.range==Fixed64::from_i32(600)));
        let mana=aid.extras.iter().find(|(key,_)|*key=="self_mana_restore").unwrap();
        assert_eq!(mana.1,&[10,15,20,25].map(Fixed64::from_i32));
        let original=active_ability_const(ability_by_name("lumen_mend").unwrap()).unwrap();
        assert!(!original.extras.iter().any(|(key,_)|*key=="self_mana_restore"));
    }

    #[test]
    fn communication_catalog_compiles_explicit_ids_slots_and_missing_optional_media() {
        assert_eq!(COMMUNICATION_CONTENT.len(), 13);
        for (slot, id) in [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)] {
            assert_eq!(communication_content(id).unwrap().interaction_slot, slot);
        }
        for slot in 1..=8 {
            let row = communication_content(100 + u16::from(slot)).unwrap();
            assert_eq!(row.wheel_slot, slot);
            assert_eq!(row.interaction_slot, 0);
        }
        assert!(communication_content(0).is_none());
        assert!(communication_content(65535).is_none());
        assert!(COMMUNICATION_CONTENT.iter().all(|row| row.icon.is_none() && row.audio.is_none() && row.animation.is_none()));
    }

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

    #[test]
    fn summoner_generated_rows_mirror_lua_balance_and_join_content_catalog_data_hash() {
        let blink = summoner_spell_by_name("training_point_blink_v1").unwrap();
        let heal = summoner_spell_by_name("training_self_heal_v1").unwrap();
        assert_eq!(SUMMONER_SPELL_CATALOG.len(), 7);
        let mana=summoner_spell_by_name("training_self_mana_v1").unwrap();assert_eq!(mana.catalog_id,7);assert_eq!(mana.target,SummonerTargetConst::None);
        assert_eq!(mana.operation,SummonerSpellOperationConst::SelfRestoreMana {amount:Fixed64::from_i32(200)});assert_eq!(mana.cooldown,Fixed64::from_i32(180));
        let guard=summoner_spell_by_name("training_self_guard_v1").unwrap();assert_eq!(guard.catalog_id,6);assert_eq!(guard.target,SummonerTargetConst::None);
        assert_eq!(guard.operation,SummonerSpellOperationConst::SelfDamageReduction {reduction:Fixed64::from_raw(256),duration:Fixed64::from_i32(3)});
        let sprint=summoner_spell_by_name("training_self_sprint_v1").unwrap();assert_eq!(sprint.catalog_id,5);assert_eq!(sprint.cooldown,Fixed64::from_i32(180));assert_eq!(sprint.target,SummonerTargetConst::None);
        assert_eq!(sprint.operation,SummonerSpellOperationConst::SelfSprint {ms_bonus:Fixed64::from_i32(80),duration:Fixed64::from_i32(6)});
        let cleanse=summoner_spell_by_name("training_control_cleanse_v1").unwrap();
        assert_eq!(cleanse.catalog_id,4);
        assert_eq!(cleanse.cooldown,Fixed64::from_i32(210));
        assert_eq!(cleanse.target,SummonerTargetConst::None);
        assert_eq!(cleanse.operation,SummonerSpellOperationConst::SelfControlCleanse);
        let shield=summoner_spell_by_name("training_self_shield_v1").unwrap();
        assert_eq!(shield.catalog_id,3);
        assert_eq!(shield.cooldown,Fixed64::from_i32(180));
        assert_eq!(shield.target,SummonerTargetConst::None);
        assert_eq!(shield.operation,SummonerSpellOperationConst::SelfShield {amount:Fixed64::from_i32(160),duration:Fixed64::from_i32(2)});
        assert_eq!(blink.catalog_id, 1);
        assert_eq!(heal.catalog_id, 2);
        assert_eq!(blink.display_name, "閃躍");
        assert_eq!(heal.display_name, "復甦");
        assert_eq!(blink.description, "指定地面一點並瞬間移動。超出距離時沿射線夾到範圍內。可跨越牆面，落點必須可站。");
        assert_eq!(heal.description, "立即回復自身生命，不選取任何單位。");
        assert_eq!(blink.target, SummonerTargetConst::Point);
        assert_eq!(heal.target, SummonerTargetConst::None);
        let quant = |value: f32| i64::from((value * 1024.0).round() as i32);
        assert_eq!(blink.cooldown, Fixed64::from_raw(quant(300.0)));
        assert_eq!(heal.cooldown, Fixed64::from_raw(quant(240.0)));
        match blink.operation {
            SummonerSpellOperationConst::PointBlink { range, out_of_range } => {
                assert_eq!(range, Fixed64::from_raw(quant(400.0)));
                assert_eq!(out_of_range, SummonerOutOfRangeConst::ClampToRange);
            }
            _ => panic!("blink operation"),
        }
        match heal.operation {
            SummonerSpellOperationConst::SelfHeal { amount } => {
                assert_eq!(amount, Fixed64::from_raw(quant(200.0)));
            }
            _ => panic!("heal operation"),
        }
        assert_eq!(CONTENT_CATALOG_DATA_HASH.len(), 16);
        assert!(CONTENT_CATALOG_DATA_HASH.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn trinket_generated_row_matches_authored_training_scout_lantern() {
        let row = trinket_by_name("training_scout_lantern_v1").unwrap();
        assert_eq!(TRINKET_CATALOG.len(), 2);
        assert_eq!(row.catalog_id, 1);
        assert_eq!(row.id, "training_scout_lantern_v1");
        assert_eq!(row.display_name, "偵察燈");
        assert_eq!(row.description, "放置後暫時提供友方視野。電荷有限，用盡後需等待回復。");
        let quant = |value: f32| i64::from((value * 1024.0).round() as i32);
        assert_eq!(row.placement_range, Fixed64::from_raw(quant(600.0)));
        assert_eq!(row.recharge_seconds, Fixed64::from_raw(quant(120.0)));
        assert_eq!(row.lifetime_seconds, Fixed64::from_raw(quant(90.0)));
        assert_eq!(row.sight_radius, Fixed64::from_raw(quant(900.0)));
        assert_eq!(row.initial_charges, 1);
        assert_eq!(row.max_charges, 2);
        assert_eq!(row.hit_points, 3);
        assert_eq!(row.maximum_owned_placements, 3);
        assert_eq!(row.detection_level, 0);
        assert_eq!(row.stealth_level, 1);
        assert_eq!(row.direct_attack, TrinketDirectAttackConst::DisclosedEnemy);
        assert_eq!(row.automatic_acquisition, TrinketAutomaticAcquisitionConst::Disabled);
        assert!(MOBA_TRINKET_CATALOG_JSON.contains("training_scout_lantern_v1"));
        assert!(MOBA_TRINKET_CATALOG_JSON.contains("disclosed_enemy"));
        assert!(MOBA_TRINKET_CATALOG_JSON.contains("\"automatic_acquisition\":\"disabled\"")
            || MOBA_TRINKET_CATALOG_JSON.contains("\"automatic_acquisition\": \"disabled\""));
        assert_eq!(CONTENT_CATALOG_DATA_HASH.len(), 16);
    }

    #[test]
    fn detection_trinket_is_independent_and_keeps_visible_detection_tradeoffs() {
        let row=trinket_by_name("training_detection_lantern_v1").unwrap();
        assert_eq!((row.catalog_id,row.display_name),(2,"偵隱燈"));
        assert_eq!((row.detection_level,row.stealth_level,row.hit_points,row.maximum_owned_placements),(2,0,4,1));
        assert_eq!((row.initial_charges,row.max_charges),(1,1));
        assert_eq!((row.placement_range,row.sight_radius,row.lifetime_seconds,row.recharge_seconds),
            (Fixed64::from_i32(600),Fixed64::from_i32(600),Fixed64::from_i32(60),Fixed64::from_i32(180)));
        assert_eq!(row.direct_attack,TrinketDirectAttackConst::DisclosedEnemy);
        assert_eq!(row.automatic_acquisition,TrinketAutomaticAcquisitionConst::Disabled);
        assert!(row.description.contains("地形仍會阻擋視線"));
        assert_eq!(trinket_by_name("training_scout_lantern_v1").unwrap().catalog_id,1);
    }

    #[test]
    fn authored_binding_item_preserves_control_and_existing_hex_identity() {
        let item=MOBA_ITEM_CATALOG.iter().find(|i|i.catalog_id==13).unwrap();
        assert_eq!((item.id,item.cost),("moba_binding_charm",650));
        assert_eq!((item.cooldown,item.target.range),(Fixed64::from_i32(18),Fixed64::from_i32(450)));
        let Some(MobaItemActiveConst::UnitHex {damage,control:Some(control),healing_reduction:None})=item.active else {panic!("authored control")};
        assert_eq!(damage,Fixed64::from_i32(30));assert!(matches!(control.kind,MobaItemControlKindConst::Root));
        assert_eq!(control.duration,Fixed64::from_raw(1536));
        assert!(matches!(MOBA_ITEM_CATALOG.iter().find(|i|i.catalog_id==11).unwrap().active,
            Some(MobaItemActiveConst::UnitHex {control:None,healing_reduction:Some(_),..})));
    }

    #[test]
    fn authored_cleanse_item_has_closed_self_operation_and_stable_prior_identity() {
        let row=MOBA_ITEM_CATALOG.iter().find(|i|i.catalog_id==14).unwrap();assert_eq!((row.id,row.cost),("moba_purification_charm",1400));
        assert_eq!(row.cooldown,Fixed64::from_i32(45));assert!(matches!(row.active,Some(MobaItemActiveConst::SelfControlCleanse)));
        assert_eq!(row.target.shape,MobaItemTargetShapeConst::Self_);assert_eq!(row.target.range,Fixed64::ZERO);
        assert_eq!(MOBA_ITEM_CATALOG.iter().find(|i|i.catalog_id==13).unwrap().id,"moba_binding_charm");
    }
}
