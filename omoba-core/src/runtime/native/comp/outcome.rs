use omoba_sim::{Fixed64, Vec2 as SimVec2};
use serde::{Deserialize, Serialize};
use specs::Entity;

use crate::runtime::comp::{CProperty, Creep, CreepStatus, Faction, TAttack, TProperty, Unit};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Outcome {
    /// Host-owned stationary channel, admitted with its cast resource transaction.
    ScriptRecoveryChannel { caster:Entity, owner:u32, amount:Fixed64, duration:Fixed64, interval:Fixed64 },
    /// Host-only scheduled zone. Expand at serial settlement after earlier hits,
    /// rechecking source lifetime and current target state before any cue/damage.
    DamageZonePulse { source:Entity, payload:serde_json::Value },
    /// Raw script/timed damage. Base defenses have NOT been applied.
    /// Convert exactly once at serial settlement before packet modifiers/shields.
    RawDamage {
        pos: SimVec2, phys: Fixed64, magi: Fixed64, real: Fixed64,
        source: Entity, target: Entity, damage_profile: u32,
        #[serde(default)]
        origin: Option<super::ProjectileOrigin>,
    },
    /// Host-local managed cast transaction; no effect or cue escapes admission.
    ScriptCastTransaction {
        caster: Entity,
        skill_id: String,
        rank: u32,
        faction: Option<Faction>,
        player_owner: Option<u32>,
        expected_mana: Option<crate::runtime::ability_runtime::ManaPool>,
        outcomes: Vec<Outcome>,
        cues: Vec<CastSettlementCue>,
        aggression_target: Option<Entity>,
    },
    /// Internal completion marker inserted only after transaction admission.
    ScriptCastCommitted { caster: Entity, cues: Vec<CastSettlementCue> },
    /// Internal provenance envelope. Only damage, hit callbacks and child
    /// projectiles from this exact source are admitted by the processor.
    ProjectileContext {
        origin: super::ProjectileOrigin,
        outcome: Box<Outcome>,
    },
    Damage {
        pos: SimVec2,
        phys: Fixed64,
        magi: Fixed64,
        real: Fixed64,
        source: Entity,
        target: Entity,
        /// Explicit compatibility mask for TD layered enemies.
        #[serde(default)]
        damage_profile: u32,
        #[serde(default)]
        predeclared: bool,
    },
    ProjectileDamage {
        origin: super::ProjectileOrigin,
        pos: SimVec2,
        phys: Fixed64,
        magi: Fixed64,
        real: Fixed64,
        source: Entity,
        target: Entity,
        damage_profile: u32,
        predeclared: bool,
    },
    ProjectileHit {
        source: Entity,
        target: Entity,
        kind_id: u16,
        generation: u8,
    },
    ProjectileLine2 {
        pos: SimVec2,
        source: Option<Entity>,
        target: Option<Entity>,
    },
    Death {
        pos: SimVec2,
        ent: Entity,
    },
    Creep {
        cd: CreepData,
    },
    CreepUpdate {
        entity: Entity,
        pos: SimVec2,
        status: CreepStatus,
        pidx: usize,
        path_remaining_distance: Fixed64,
        facing: omoba_sim::Angle,
        facing_broadcast: Option<f32>,
    },
    CreepStop {
        source: Entity,
        target: Entity,
    },
    CreepWalk {
        target: Entity,
    },
    Tower {
        pos: SimVec2,
        td: TowerData,
    },
    Heal {
        pos: SimVec2,
        target: Entity,
        amount: Fixed64,
    },
    UpdateAttack {
        target: Entity,
        asd_count: Option<Fixed64>,
        cooldown_reset: bool,
    },
    GainExperience {
        target: Entity,
        amount: i32,
    },
    GainGold {
        target: Entity,
        amount: i32,
    },
    SpawnUnit {
        pos: SimVec2,
        unit: Unit,
        faction: Faction,
        duration: Option<Fixed64>,
    },
    SpawnSummoned {
        entity: Entity,
        pos: SimVec2,
        unit_id: String,
        owner: Entity,
        faction: Faction,
        player_owner: Option<u32>,
        duration: Fixed64,
    },
    CreepLeaked {
        ent: Entity,
    },
    AddBuff {
        target: Entity,
        buff_id: String,
        duration: Fixed64,
        #[serde(default)]
        payload: serde_json::Value,
    },
    Explosion {
        pos: SimVec2,
        radius: Fixed64,
        duration: Fixed64,
    },
    ProjectileDirectional {
        pos: SimVec2,
        source: Option<Entity>,
        end_pos: SimVec2,
    },
    AttackPhaseCue {
        entity: Entity,
        attack_seq: u32,
        #[serde(default)]
        is_critical: bool,
        target: Option<Entity>,
        target_pos: Option<SimVec2>,
        windup_ms: u32,
        backswing_ms: u32,
        dir_rad: f32,
    },
    ScriptSetPos {
        entity: Entity,
        pos: SimVec2,
    },
    ScriptSetFacing {
        entity: Entity,
        facing: omoba_sim::Angle,
    },
    ScriptSetAsdCount {
        entity: Entity,
        asd_count: Fixed64,
    },
    ScriptSetTowerAtk {
        entity: Entity,
        value: Fixed64,
    },
    ScriptSetTowerRange {
        entity: Entity,
        value: Fixed64,
    },
    ScriptSetAsdInterval {
        entity: Entity,
        value: Fixed64,
    },
    ScriptSetTowerInternalCooldown {
        entity: Entity,
        duration: Fixed64,
    },
    ScriptDirectDamage {
        target: Entity,
        amount: Fixed64,
    },
    ScriptHeal {
        target: Entity,
        amount: Fixed64,
    },
    ScriptRemoveBuff {
        target: Entity,
        buff_id: String,
    },
    ScriptProjectile {
        #[serde(default)]
        basic_attack: bool,
        #[serde(default)]
        source_missed: bool,
        #[serde(default)]
        source_critical: bool,
        pos: SimVec2,
        owner: Entity,
        target: Option<Entity>,
        tpos: SimVec2,
        radius: Fixed64,
        msd: Fixed64,
        damage_phys: Fixed64,
        damage_magi: Fixed64,
        damage_real: Fixed64,
        damage_profile: u32,
        slow_factor: Fixed64,
        slow_duration: Fixed64,
        hit_radius: Fixed64,
        stun_duration: Fixed64,
        kind_id: u16,
        generation: u8,
    },
    ScriptTowerFireFx {
        entity: Entity,
        dir_rad: f32,
    },
    ScriptAttackPhaseCue {
        entity: Entity,
        target: Option<Entity>,
        target_pos: Option<SimVec2>,
        windup_ms: u32,
        backswing_ms: u32,
        dir_rad: f32,
    },
    ScriptStartCooldown {
        entity: Entity,
        ability_id: String,
        duration: Fixed64,
    },
    ScriptSetMana {
        entity: Entity,
        pool: crate::runtime::ability_runtime::ManaPool,
    },
    ScriptManaCommit {
        changes: Vec<ManaSettlementChange>,
        notifications: Vec<ManaSettlementNotification>,
    },
    ScriptManaGained { entity: Entity, amount: Fixed64 },
    ScriptManaSpent { entity: Entity, amount: Fixed64, ability_id: String },
    EntityRemoved {
        entity: Entity,
    },
    /// Host-only settled native damage with conditional leech. Only an immediate
    /// Damage packet is accepted; no precomputed Heal escapes serial admission.
    DamageWithLeech { damage:Box<Outcome>, physical_leech:Fixed64, magical_leech:Fixed64 },
    /// Appended host-only critical provenance. Immediate damage/leech only.
    DamageWithCritical { damage:Box<Outcome> },
    /// Authority-only lease token. Damage/faction/credit come from the private book.
    IndependentDamageZonePulse { source:Entity, key:String, lease:u64 },
}

/// The successful-cast subset of native presentation, without ABI policy handles.
/// Settlement reconstructs the normal script event and assigns its policy then.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CastSettlementCue {
    pub target: Option<Entity>,
    pub skill_id: Option<String>,
    pub rank: u32,
    pub tick: u64,
    pub relocation: Option<(u32, i64, i64)>,
    pub area: Option<(u32, i64, i64, i64, i64)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ManaSettlementChange {
    pub entity: Entity,
    pub expected: crate::runtime::ability_runtime::ManaPool,
    pub pool: crate::runtime::ability_runtime::ManaPool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ManaSettlementNotification {
    Gained { entity: Entity, amount: Fixed64 },
    Spent { entity: Entity, amount: Fixed64, ability_id: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreepData {
    pub pos: SimVec2,
    pub creep: Creep,
    pub cdata: CProperty,
    #[serde(default)]
    pub faction_name: String,
    #[serde(default = "default_creep_turn_speed_deg")]
    pub turn_speed_deg: Fixed64,
    #[serde(default = "default_creep_cr")]
    pub collision_radius: Fixed64,
}

fn default_creep_cr() -> Fixed64 {
    Fixed64::from_i32(20)
}

fn default_creep_turn_speed_deg() -> Fixed64 {
    Fixed64::from_i32(90)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerData {
    pub tpty: TProperty,
    pub tatk: TAttack,
}
