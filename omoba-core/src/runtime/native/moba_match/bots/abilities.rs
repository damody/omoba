//! Explicit author-owned AI intentions, not tooltip effect inference.
use super::*;
use crate::runtime::CastAbility;
use omoba_template_ids::{AbilityTypeC, CastTypeC, TargetTypeC};
use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path="cone_tests.rs"]
mod cone_tests;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum BotAbilityIntent {
    EnemyUnit,
    /// Like EnemyUnit, but only disclosed enemy heroes (a skill whose source targets
    /// DOTA_UNIT_TARGET_HERO only, e.g. C17W; E170).
    EnemyHero,
    EnemyPoint {radius_key:String,min_targets:u16},
    /// A tree-targeted ability (v3 `target.type = "tree"`, E403; B22 樹人召喚／樹覺醒, E473): the
    /// standing tree nearest the bot within cast range. With `enemy_radius` > 0, only while a
    /// disclosed enemy hero is within that radius of the bot. Trees are public map state.
    NearestTree {#[serde(default)] enemy_radius:u32},
    /// A point area with no cast range that leaps toward its aim (E281, e.g. B29W 震地斧): the
    /// compiled `reach_key` extra (the package's targeting reach) is how far it reaches; aimed at
    /// the disclosed enemy within that reach whose `radius_key` area covers the most enemies.
    LeapEnemyPoint {radius_key:String,reach_key:String,min_targets:u16},
    /// A point skill cast at any distance (every rank's cast range 0, E387, e.g. C05T 瞬雷閃): aimed
    /// at the disclosed enemy hero whose `radius_key` area covers the most enemy heroes, at least
    /// `min_targets`, the nearest first.
    GlobalEnemyPoint {radius_key:String,min_targets:u16},
    /// A unit skill on an allied hero joining its fight (E387, e.g. C24R 最期一生): cast on another
    /// allied hero with a disclosed enemy hero within `threat_radius` of it, the most hurt first;
    /// within the cast range, or at any distance when every rank's range is 0.
    AllyEngage {threat_radius:u32},
    /// Actual compiled source-sector geometry; no authored radius/angle-key guess.
    EnemyCone {min_targets:u16},
    /// Actual compiled nonmoving capsule geometry, over disclosed observations.
    EnemyLine {min_targets:u16},
    /// Author explicitly opts a point skill into closing on disclosed enemies, farther than
    /// `min_distance` and within its cast range, or within its compiled `reach_key` extra for a
    /// pure move with no cast range (E438, e.g. B13 縮地: teleport max_distance).
    ApproachEnemyPoint {min_distance:u32,#[serde(default,skip_serializing_if="Option::is_none")] reach_key:Option<String>},
    /// A toggle (E263, e.g. A02W 蠍毒箭): switched on while a disclosed enemy hero is within
    /// `threat_radius` and off when none is. `on_status` is the owner's own status the toggle's
    /// on-switch applies (the generated program's own on/off test), read from the owner only.
    ToggleEngage {threat_radius:u32,on_status:String},
    /// A point summon (E260, e.g. B01E 雜賀眾): cast when a disclosed enemy hero or lane unit is
    /// within `threat_radius`, at the nearest one, or toward it at the cast range when farther.
    /// An untargeted summon (E419, e.g. C02E 幻影步, B18R 土爪) is cast then, with no point.
    SummonNearEnemy {threat_radius:u32},
    SelfHeal { below_hp_per_mille:u16 },
    /// Untargeted area around the caster (E163): cast when at least `min_targets`
    /// disclosed enemies stand within the compiled radius extra.
    SelfArea { radius_key:String, min_targets:u16 },
    /// [`BotAbilityIntent::SelfArea`] that counts only the disclosed enemies showing `status`
    /// (its public icon; E605, 原作 C25T detonates the bleeding: the cast cancels itself without one).
    SelfAreaStatus { radius_key:String, min_targets:u16, status:String },
    /// [`BotAbilityIntent::SelfArea`] over corpses (E605, 原作 B23D raises a ghost from one): the combat
    /// units of either team that died within the radius in the last minute (include_dead).
    SelfAreaCorpses { radius_key:String, min_targets:u16 },
    /// A point skill that must land beside an allied unit (E605, 原作 C19D teleports to an ally
    /// creep, summon or building): at the allied unit of `types` ("creep", "summon", "building";
    /// never a hero) nearest the closest disclosed enemy hero; within the cast range, or anywhere
    /// when every rank's range is 0. `radius_key` is the area the script checks around the point.
    AllyAnchorPoint { radius_key:String, types:Vec<String> },
    /// A unit move to an allied non-hero unit (an item's, 千里大劍): of the allied units of
    /// `types` (creep, summon, building), the one nearest a disclosed enemy hero, at least
    /// `min_distance` from the user (worth the trip).
    AllyAnchorUnit { types:Vec<String>, min_distance:u32 },
    /// Untargeted self/ally buff for a fight (E163): cast when a disclosed enemy
    /// hero is within `threat_radius`.
    SelfEngage { threat_radius:u32 },
    /// Explicit stationary host channel, started only outside disclosed threats.
    SelfChannelRecovery { below_hp_per_mille:u16, threat_radius:u32 },
    AllyHeal { below_hp_per_mille:u16 },
    /// Explicit compiled friendly cleanse; control choices use public observation only.
    AllyHealOrCleanse { below_hp_per_mille:u16 },
    /// Explicit immediate-recovery hint. HP is optional (zero disables it);
    /// the mana branch must improve the owner's balance after host cost.
    SelfRecovery { below_hp_per_mille:u16, below_mana_per_mille:u16, restore_key:String },
    /// New additive mana_regen_constant buff. Not a capacity or percentage buff.
    SelfManaRegeneration { below_hp_per_mille:u16, below_mana_per_mille:u16,
        rate_key:String, duration_key:String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotAbilityPolicy {
    pub ability:String,
    pub intent:BotAbilityIntent,
    /// v3 `requires.has_status` on the caster (a runtime status id): cast only while the caster
    /// holds it (原作 A12 秘矢 only in a12t_form, B36 footsteps with b36w_mark; E619).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caster_status:Option<String>,
    /// v3 `requires.not_status` on the caster: cast only while the caster does not hold it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caster_no_status:Option<String>,
    /// v3 `requires` on a summon the caster created (`exists(state.V) and alive(state.V) [and
    /// edge_distance(caster, state.V) <= R]`): cast only while the caster owns a live one (原作 B16
    /// 相移 needs its moon within 1000; E648).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caster_summon:Option<BotCasterSummon>,
}

/// A live summon of the caster's own, by its unit template, within `within` edge distance when given.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotCasterSummon {
    pub template:String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within:Option<u32>,
}

/// Each entry spends one point, in author order when its prerequisites permit.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotAbilityLearningStep {
    pub ability:String,
    pub rank:u8,
}

/// `{ability:"*",rank:0}`: every seated hero learns its own kit (expanded by the plan compile).
pub const KIT_LEARNING:&str="*";

/// Explicit steps for a hero kit: rank by rank, the ultimate first within a rank. Level-gated
/// steps are skipped at runtime until allowed, so the ultimate is taken as soon as its level is.
pub(super) fn kit_learning(abilities:&[omoba_template_ids::AbilityId],skip:&std::collections::BTreeSet<String>) -> Vec<BotAbilityLearningStep> {
    let defs:Vec<_>=abilities.iter().filter_map(|id|{
        let name=id.as_str();
        if skip.contains(name) {return None;}
        omoba_template_ids::active_ability_const(*id).map(|def|(name,def))
    }).collect();
    let top=defs.iter().map(|(_,def)|def.max_level.min(def.levels.len() as u8)).max().unwrap_or(0);
    let mut steps=Vec::new();
    for rank in 1..=top {
        for ultimate in [true,false] {
            for (name,def) in &defs {
                if (def.ability_type==AbilityTypeC::Ultimate)==ultimate && rank<=def.max_level && usize::from(rank)<=def.levels.len() {
                    steps.push(BotAbilityLearningStep {ability:(*name).into(),rank});
                }
            }
        }
    }
    steps
}

pub(super) fn validate_learning(steps:&[BotAbilityLearningStep]) -> Result<(), &'static str> {
    // Authored plans stay small; expanded kits of up to ten different heroes fit in 256.
    if steps.len()>256 {return Err("bot learning plan exceeds 256 steps");}
    let mut ranks=std::collections::BTreeMap::new();
    for step in steps {
        if step.ability==KIT_LEARNING {
            if step.rank!=0 {return Err("kit learning step must have rank 0");}
            continue;
        }
        let def=omoba_template_ids::ability_by_name(&step.ability)
            .and_then(omoba_template_ids::active_ability_const).ok_or("unknown active compiled bot learning ability")?;
        let previous=ranks.get(&step.ability).copied().unwrap_or(0u8);
        if step.rank==0 || step.rank>def.max_level || usize::from(step.rank)>def.levels.len()
            || step.rank!=previous+1 {return Err("bot learning ranks must be consecutive from one within compiled maximum");}
        ranks.insert(&step.ability,step.rank);
    }
    Ok(())
}

pub(super) fn choose_upgrade(hero:&Hero,steps:&[BotAbilityLearningStep]) -> Option<crate::game_proto::UpgradeAbility> {
    if hero.skill_points<=0 {return None;}
    for step in steps {
        let Some(slot)=hero.abilities.iter().position(|id|id==&step.ability).filter(|slot|*slot<HERO_ABILITY_SLOTS) else {continue;};
        let rank=hero.get_ability_level(&step.ability);
        if rank<0 || rank.checked_add(1)!=Some(i32::from(step.rank)) {continue;}
        let Some(def)=omoba_template_ids::ability_by_name(&step.ability)
            .and_then(omoba_template_ids::active_ability_const) else {continue;};
        if rank>=i32::from(def.max_level) {continue;}
        let Some(level)=def.levels.get(rank as usize) else {continue;};
        if hero.level<i32::from(level.required_hero_level) {continue;}
        return Some(crate::game_proto::UpgradeAbility {ability_index:slot as u32});
    }
    None
}

/// Area casts are kept for heroes (E398): an area that covers a disclosed enemy hero needs
/// `min_targets` units; one that covers only lane units needs at least a wave of them and no
/// disclosed enemy hero within [`HERO_RESERVE_RADIUS`] of the caster, so the cooldown is there
/// when the heroes come. A jungler clears its camps with `min_targets` alone.
const WAVE_TARGETS:usize=3;
const HERO_RESERVE_RADIUS:i32=1200;
/// `lane`: a lane role (not a jungler); `hero_near`: a disclosed enemy hero within the reserve.
fn area_worth(heroes:usize,count:usize,min_targets:u16,lane:bool,hero_near:bool)->bool {
    let min=usize::from(min_targets);
    if heroes>0 || !lane {count>=min} else {!hero_near && count>=min.max(WAVE_TARGETS)}
}

/// A line skill's width at `rank`: the compiled line damage width, else the authored targeting
/// width `shape_width` (E260: generated-Rust skills carry no LineDamage effect, so their compiled
/// widths are zero).
fn line_width(def:&omoba_template_ids::AbilityConst,rank:usize)->Option<Fixed64> {
    def.line_widths.get(rank).copied().filter(|width|*width>Fixed64::ZERO)
        .or_else(||def.extras.iter().find(|(key,_)|*key=="shape_width").and_then(|(_,widths)|widths.get(rank).copied()))
        .filter(|width|*width>Fixed64::ZERO)
}

/// How far a line reaches: its cast range, or for a range-0 (unlimited) line its compiled
/// `distance` extra, the length the projectile travels (E410).
fn line_reach(def:&omoba_template_ids::AbilityConst,rank:usize,range:Fixed64)->Option<Fixed64> {
    if range>Fixed64::ZERO {return Some(range);}
    def.extras.iter().find(|(key,_)|*key=="distance").and_then(|(_,values)|values.get(rank).copied())
        .filter(|distance|*distance>Fixed64::ZERO)
}

pub(super) fn validate_policies(policies:&[BotAbilityPolicy]) -> Result<(), &'static str> {
    if policies.len()>64 {return Err("bot ability policies exceed 64");}
    let mut ids=std::collections::BTreeSet::new();
    for policy in policies {
        if !ids.insert(&policy.ability) {return Err("duplicate bot ability policy");}
        let status_id=|id:&String|!id.is_empty() && id.len()<=96 && id.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b':');
        if policy.caster_status.iter().chain(&policy.caster_no_status).any(|id|!status_id(id)) {return Err("invalid bot caster status");}
        if policy.caster_summon.as_ref().is_some_and(|summon|!status_id(&summon.template) || summon.template.contains(':')
            || summon.within.is_some_and(|within|within==0 || within>100_000)) {return Err("invalid bot caster summon");}
        let def=omoba_template_ids::ability_by_name(&policy.ability)
            .and_then(omoba_template_ids::active_ability_const).ok_or("unknown active compiled bot ability")?;
        let supported_cast=if matches!(policy.intent,BotAbilityIntent::SelfChannelRecovery {..}) {
            def.cast_type==CastTypeC::Channeled && def.recovery_channel.is_some()
                && def.levels.iter().all(|rank|rank.cast_time==Fixed64::ZERO)
        } else {def.cast_type==CastTypeC::Instant
            || def.cast_type==CastTypeC::Channeled && def.recovery_channel.is_none()
                && !matches!(policy.intent,BotAbilityIntent::ToggleEngage {..})};
        let toggle=matches!(policy.intent,BotAbilityIntent::ToggleEngage {..});
        if !(matches!(def.ability_type,AbilityTypeC::Active | AbilityTypeC::Ultimate)
            || toggle && def.ability_type==AbilityTypeC::Toggle) || !supported_cast {
            return Err("bot policy requires a supported explicit active execution type");
        }
        match &policy.intent {
            BotAbilityIntent::SelfChannelRecovery {below_hp_per_mille,threat_radius}
                if def.target_type==TargetTypeC::None && (1..=1000).contains(below_hp_per_mille)
                    && (1..=10_000).contains(threat_radius)=>{},
            BotAbilityIntent::EnemyUnit|BotAbilityIntent::EnemyHero if def.target_type==TargetTypeC::Unit => {},
            BotAbilityIntent::NearestTree {enemy_radius} if def.targets_tree && def.target_type==TargetTypeC::Point
                && *enemy_radius<=10_000 && def.levels.iter().all(|rank|rank.range>Fixed64::ZERO&&rank.range<=Fixed64::from_i32(10_000))=>{},
            BotAbilityIntent::AllyHeal {below_hp_per_mille} if def.target_type==TargetTypeC::Unit
                && (1..=1000).contains(below_hp_per_mille)=>{},
            BotAbilityIntent::AllyHealOrCleanse {below_hp_per_mille} if def.target_type==TargetTypeC::Unit
                && def.cleanses_allies && (1..=1000).contains(below_hp_per_mille)
                && def.levels.iter().all(|rank|rank.range>Fixed64::ZERO&&rank.range<=Fixed64::from_i32(10_000))=>{},
            BotAbilityIntent::ApproachEnemyPoint {min_distance,reach_key:None} if def.target_type==TargetTypeC::Point
                && (1..=10_000).contains(min_distance)
                && def.levels.iter().all(|level|level.range>Fixed64::from_i32(*min_distance as i32))=>{},
            BotAbilityIntent::ApproachEnemyPoint {min_distance,reach_key:Some(reach_key)} if def.target_type==TargetTypeC::Point
                && (1..=10_000).contains(min_distance)=>{
                let Some((_,reach))=def.extras.iter().find(|(name,_)|*name==reach_key) else {return Err("bot approach reach_key requires compiled reach extras");};
                if reach.len()!=def.levels.len() || reach.iter().any(|r|*r<=Fixed64::from_i32(*min_distance as i32) || *r>Fixed64::from_i32(10_000)) {
                    return Err("bot approach reach must exceed min_distance per rank, bounded");
                }
            },
            BotAbilityIntent::ToggleEngage {threat_radius,on_status} if def.target_type==TargetTypeC::None
                && def.ability_type==AbilityTypeC::Toggle && (1..=10_000).contains(threat_radius)
                && !on_status.is_empty() && on_status.len()<=128=>{},
            BotAbilityIntent::SummonNearEnemy {threat_radius} if (1..=10_000).contains(threat_radius)
                && (def.target_type==TargetTypeC::None || def.target_type==TargetTypeC::Point
                    && def.levels.iter().all(|level|level.range>Fixed64::ZERO&&level.range<=Fixed64::from_i32(10_000)))=>{},
            BotAbilityIntent::SelfHeal {below_hp_per_mille} if def.target_type==TargetTypeC::None
                && *below_hp_per_mille>0 && *below_hp_per_mille<=1000 => {},
            BotAbilityIntent::SelfEngage {threat_radius} if def.target_type==TargetTypeC::None
                && (1..=10_000).contains(threat_radius) => {},
            BotAbilityIntent::SelfArea {radius_key,min_targets} if def.target_type==TargetTypeC::None
                && (1..=128).contains(min_targets)=>{
                let Some((_,radii))=def.extras.iter().find(|(key,_)|*key==radius_key) else {return Err("bot self-area intent requires compiled radius extras");};
                if radii.len()!=def.levels.len() || radii.iter().any(|r|*r<=Fixed64::ZERO || *r>Fixed64::from_i32(10_000)) {
                    return Err("bot self-area radius must be positive bounded per-rank data");
                }
            },
            BotAbilityIntent::SelfAreaStatus {radius_key,min_targets,..}|BotAbilityIntent::SelfAreaCorpses {radius_key,min_targets}
                if def.target_type==TargetTypeC::None && (1..=128).contains(min_targets)
                    && !matches!(&policy.intent,BotAbilityIntent::SelfAreaStatus {status,..} if status.is_empty() || status.len()>128)=>{
                let Some((_,radii))=def.extras.iter().find(|(key,_)|*key==radius_key) else {return Err("bot self-area intent requires compiled radius extras");};
                if radii.len()!=def.levels.len() || radii.iter().any(|r|*r<=Fixed64::ZERO || *r>Fixed64::from_i32(10_000)) {
                    return Err("bot self-area radius must be positive bounded per-rank data");
                }
            },
            BotAbilityIntent::AllyAnchorPoint {radius_key,types} if def.target_type==TargetTypeC::Point
                && !types.is_empty() && types.len()<=3 && types.iter().all(|kind|matches!(kind.as_str(),"creep"|"summon"|"building"))
                && def.levels.iter().all(|rank|rank.range>=Fixed64::ZERO&&rank.range<=Fixed64::from_i32(10_000))=>{
                let Some((_,radii))=def.extras.iter().find(|(key,_)|*key==radius_key) else {return Err("bot anchor intent requires compiled radius extras");};
                if radii.len()!=def.levels.len() || radii.iter().any(|r|*r<=Fixed64::ZERO || *r>Fixed64::from_i32(10_000)) {
                    return Err("bot anchor radius must be positive bounded per-rank data");
                }
            },
            BotAbilityIntent::SelfRecovery {below_hp_per_mille,below_mana_per_mille,restore_key}
                if def.target_type==TargetTypeC::None && *below_hp_per_mille<=1000
                    && (1..=1000).contains(below_mana_per_mille)=>{
                let Some((_,amounts))=def.extras.iter().find(|(key,_)|*key==restore_key) else {
                    return Err("bot recovery intent requires compiled restore extras");
                };
                if amounts.len()!=def.levels.len() || amounts.iter().any(|amount|
                    *amount<Fixed64::ZERO || *amount>Fixed64::from_i32(1_000_000)) {
                    return Err("bot restore amounts must be nonnegative bounded per-rank data");
                }
            },
            BotAbilityIntent::SelfManaRegeneration {below_hp_per_mille,below_mana_per_mille,rate_key,duration_key}
                if def.target_type==TargetTypeC::None && *below_hp_per_mille<=1000
                    && (1..=1000).contains(below_mana_per_mille)=>{
                for (key,maximum) in [(rate_key,1_000_000),(duration_key,60)] {
                    let Some((_,values))=def.extras.iter().find(|(name,_)|*name==key) else {
                        return Err("bot regeneration intent requires compiled rate and duration extras");
                    };
                    if values.len()!=def.levels.len() || values.iter().any(|value|
                        *value<=Fixed64::ZERO || *value>Fixed64::from_i32(maximum)) {
                        return Err("bot regeneration extras require positive bounded per-rank data");
                    }
                }
            },
            BotAbilityIntent::EnemyPoint {radius_key,min_targets} if def.target_type==TargetTypeC::Point
                && (1..=128).contains(min_targets)=>{
                let Some((_,radii))=def.extras.iter().find(|(key,_)|*key==radius_key) else {return Err("bot point intent requires compiled radius extras");};
                if radii.len()!=def.levels.len() || radii.iter().any(|r|*r<=Fixed64::ZERO || *r>Fixed64::from_i32(10_000)) {
                    return Err("bot point radius must be positive bounded per-rank data");
                }
            },
            BotAbilityIntent::LeapEnemyPoint {radius_key,reach_key,min_targets} if def.target_type==TargetTypeC::Point
                && (1..=128).contains(min_targets)=>{
                for key in [radius_key,reach_key] {
                    let Some((_,values))=def.extras.iter().find(|(name,_)|*name==key) else {return Err("bot leap intent requires compiled radius and reach extras");};
                    if values.len()!=def.levels.len() || values.iter().any(|v|*v<=Fixed64::ZERO || *v>Fixed64::from_i32(10_000)) {
                        return Err("bot leap radius and reach must be positive bounded per-rank data");
                    }
                }
            },
            BotAbilityIntent::GlobalEnemyPoint {radius_key,min_targets} if def.target_type==TargetTypeC::Point
                && (1..=128).contains(min_targets) && def.levels.iter().all(|rank|rank.range==Fixed64::ZERO)=>{
                let Some((_,radii))=def.extras.iter().find(|(key,_)|*key==radius_key) else {return Err("bot global point intent requires compiled radius extras");};
                if radii.len()!=def.levels.len() || radii.iter().any(|r|*r<=Fixed64::ZERO || *r>Fixed64::from_i32(10_000)) {
                    return Err("bot global point radius must be positive bounded per-rank data");
                }
            },
            BotAbilityIntent::AllyEngage {threat_radius} if def.target_type==TargetTypeC::Unit
                && (1..=10_000).contains(threat_radius)
                && def.levels.iter().all(|rank|rank.range>=Fixed64::ZERO&&rank.range<=Fixed64::from_i32(10_000))=>{},
            BotAbilityIntent::EnemyLine {min_targets} if def.target_type==TargetTypeC::Point
                && (1..=128).contains(min_targets)
                && (0..def.levels.len()).all(|rank|line_width(def,rank).is_some_and(|width|width<=Fixed64::from_i32(5_000)))=>{},
            BotAbilityIntent::EnemyCone {min_targets} if def.target_type==TargetTypeC::Point
                && (1..=128).contains(min_targets)
                && def.cone_half_angle_ticks.len()==def.levels.len()
                && def.cone_half_angle_ticks.iter().all(|ticks|(1..=1024).contains(ticks))
                && def.levels.iter().all(|rank|rank.range>Fixed64::ZERO&&rank.range<=Fixed64::from_i32(10_000))=>{},
            _ => return Err("bot ability intent does not match target type or health threshold"),
        }
    }
    Ok(())
}

pub(super) fn requires_mana(policies:&[BotAbilityPolicy]) -> bool {
    policies.iter().any(|policy|matches!(policy.intent,
        BotAbilityIntent::SelfRecovery {..}|BotAbilityIntent::SelfManaRegeneration {..}|BotAbilityIntent::SelfChannelRecovery {..}))
}

#[cfg(test)]
fn choose_cast(hero:&Hero,role:BotRole,team:u32,own:Vec2,health:&CProperty,
    seen:&[SeenUnit],policies:&[BotAbilityPolicy]) -> Option<CastAbility> {
    choose_cast_with_mana_budget(hero,role,team,own,health,seen,policies,|_,_|None)
}

/// Cost provider uses the host's exact script metadata, not a second rounded
/// compiled copy. The planner never reserves or spends the owner's mana.
#[cfg(test)]
pub(super) fn choose_cast_with_mana_budget(hero:&Hero,role:BotRole,team:u32,own:Vec2,health:&CProperty,
    seen:&[SeenUnit],policies:&[BotAbilityPolicy],cost:impl Fn(&str,u8)->Option<Fixed64>) -> Option<CastAbility> {
    choose_cast_with_resources(hero,role,team,own,health,seen,policies,None,cost,|_,_|None)
}

/// Regeneration provider returns rates without/with a NEW flat buff, or None
/// for an active source, unavailable or invalid owner-only resource metadata.
#[cfg(test)]
pub(super) fn choose_cast_with_resources(hero:&Hero,role:BotRole,team:u32,own:Vec2,health:&CProperty,
    seen:&[SeenUnit],policies:&[BotAbilityPolicy],focus:Option<u64>,cost:impl Fn(&str,u8)->Option<Fixed64>,
    regen:impl Fn(&str,Fixed64)->Option<(Fixed64,Fixed64)>) -> Option<CastAbility> {
    choose_cast_with_attack_priority(hero,role,team,own,health,seen,policies,focus,false,false,cost,regen, |_|true)
}

/// Defer offensive intentions only; recovery still follows author ordering.
#[cfg(test)]
pub(super) fn choose_cast_with_attack_priority(hero:&Hero,role:BotRole,team:u32,own:Vec2,health:&CProperty,
    seen:&[SeenUnit],policies:&[BotAbilityPolicy],focus:Option<u64>,defer_offense:bool,immobilized:bool,
    cost:impl Fn(&str,u8)->Option<Fixed64>,
    regen:impl Fn(&str,Fixed64)->Option<(Fixed64,Fixed64)>,
    relocation_clear:impl Fn(Vec2)->bool) -> Option<CastAbility> {
    choose_cast_with_control_observation(hero,role,team,own,health,seen,policies,focus,defer_offense,immobilized,cost,regen,relocation_clear,|_|None,&|_|false,&|_|0,&|_,_|None,&|_,_|false,&|_,_|0)
}

/// How far past its cast range a tree-target ability looks for a tree to walk to.
pub(super) const TREE_APPROACH_SEARCH:i32=600;
/// Short of the cast range the bot stands, so the arrival is in range.
const TREE_APPROACH_MARGIN:i32=3;

/// Where to stand for a ready tree-target ability with no tree in its range: a face of the nearest
/// standing tree within the search, at the cast range from its centre (the authority measures to
/// the centre; a tree blocks an axis square, so a face is the closest a body gets). On nobu the
/// lanes never passed that close and B22 自然之力／木靈展勢 were never cast.
pub(super) fn tree_approach(hero:&Hero,team:u32,own:Vec2,seen:&[SeenUnit],policies:&[BotAbilityPolicy],
    cost:impl Fn(&str,u8)->Option<Fixed64>,tree_near:&dyn Fn(Vec2,Fixed64)->Option<Vec2>,
    reachable:&dyn Fn(Vec2)->bool) -> Option<Vec2> {
    for policy in policies {
        let BotAbilityIntent::NearestTree {enemy_radius}=policy.intent else {continue;};
        let Some(slot)=hero.abilities.iter().position(|id|id==&policy.ability) else {continue;};
        if slot>=crate::runtime::HERO_ABILITY_SLOTS {continue;}
        if !hero.can_use_ability(&policy.ability) || !hero.cast_ready(&policy.ability) {continue;}
        let owner_rank=hero.get_ability_level(&policy.ability);
        if !(1..=255).contains(&owner_rank) {continue;}
        let Some(def)=omoba_template_ids::ability_by_name(&policy.ability).and_then(omoba_template_ids::active_ability_const) else {continue;};
        let Some(level)=def.levels.get((owner_rank-1) as usize) else {continue;};
        if !def.targets_tree || level.range<=Fixed64::ZERO {continue;}
        if let Some(pool)=&hero.mana_pool {
            let Some(required)=cost(&policy.ability,owner_rank as u8) else {continue;};
            if required<Fixed64::ZERO || required>pool.current() {continue;}
        }
        let radius=Fixed64::from_i32(enemy_radius as i32);
        if enemy_radius>0 && !seen.iter().any(|unit|unit.hp_raw>0 && unit.team!=team && unit.team!=0 && unit.kind==1
            && (unit.position-own).length_squared()<=radius*radius) {continue;}
        // A tree in range is the cast's own (choose_cast).
        if tree_near(own,level.range).is_some_and(|tree|(tree-own).length_squared()<=level.range*level.range) {continue;}
        let Some(tree)=tree_near(own,level.range+Fixed64::from_i32(TREE_APPROACH_SEARCH)) else {continue;};
        let stand=level.range-Fixed64::from_i32(TREE_APPROACH_MARGIN);
        if stand<=Fixed64::ZERO {continue;}
        let mut faces=[Vec2::new(stand,Fixed64::ZERO),Vec2::new(-stand,Fixed64::ZERO),
            Vec2::new(Fixed64::ZERO,stand),Vec2::new(Fixed64::ZERO,-stand)].map(|offset|tree+offset);
        faces.sort_by_key(|face|((*face-own).length_squared().raw(),face.x.raw(),face.y.raw()));
        if let Some(face)=faces.into_iter().find(|face|reachable(*face)) {return Some(face);}
    }
    None
}

pub(super) fn choose_cast_with_control_observation(hero:&Hero,role:BotRole,team:u32,own:Vec2,health:&CProperty,
    seen:&[SeenUnit],policies:&[BotAbilityPolicy],focus:Option<u64>,defer_offense:bool,immobilized:bool,
    cost:impl Fn(&str,u8)->Option<Fixed64>,regen:impl Fn(&str,Fixed64)->Option<(Fixed64,Fixed64)>,
    relocation_clear:impl Fn(Vec2)->bool,control_priority:impl Fn(u64)->Option<u8>,
    own_status:&dyn Fn(&str)->bool,target_states:&dyn Fn(u64)->u8,
    tree_near:&dyn Fn(Vec2,Fixed64)->Option<Vec2>,
    unit_status:&dyn Fn(u64,&str)->bool,corpses_near:&dyn Fn(Vec2,Fixed64)->usize) -> Option<CastAbility> {
    let enemy=|unit:&&SeenUnit|unit.hp_raw>0 && unit.team!=team
        && if role==BotRole::Jungle {unit.kind==3
            || (unit.kind==1 && unit.team!=0 && Some(unit.canonical_id)==focus)}
        else {unit.team!=0 && matches!(unit.kind,1|2)};
    let prioritizes_focus=matches!(role,BotRole::Jungle|BotRole::Support);
    // Lane units alone are worth an area only without enemy heroes about (E398).
    let lane=role!=BotRole::Jungle;
    let hero_near=seen.iter().filter(enemy).any(|unit|unit.kind==1
        && (unit.position-own).length_squared()<=Fixed64::from_i32(HERO_RESERVE_RADIUS)*Fixed64::from_i32(HERO_RESERVE_RADIUS));
    let priority=|unit:&SeenUnit|(prioritizes_focus && Some(unit.canonical_id)!=focus,
        (unit.position-own).length_squared().raw(),unit.canonical_id);
    // Author declaration order is the explicit priority. Slot is looked up in
    // this owner's current loadout; skill names never imply Q/W/E/R.
    for policy in policies {
        // Root blocks voluntary relocation, not ordinary damage/recovery.
        if immobilized && matches!(policy.intent,BotAbilityIntent::ApproachEnemyPoint {..}) {continue;}
        if defer_offense && matches!(policy.intent,BotAbilityIntent::EnemyUnit|BotAbilityIntent::EnemyHero
            |BotAbilityIntent::SelfArea {..}|BotAbilityIntent::SelfAreaStatus {..}|BotAbilityIntent::SelfAreaCorpses {..}
            |BotAbilityIntent::AllyAnchorPoint {..}|BotAbilityIntent::SelfEngage {..}
            |BotAbilityIntent::EnemyPoint {..}|BotAbilityIntent::LeapEnemyPoint {..}|BotAbilityIntent::GlobalEnemyPoint {..}
            |BotAbilityIntent::AllyEngage {..}|BotAbilityIntent::EnemyCone {..}|BotAbilityIntent::EnemyLine {..}|BotAbilityIntent::ApproachEnemyPoint {..}
            |BotAbilityIntent::SummonNearEnemy {..}|BotAbilityIntent::ToggleEngage {..}) {continue;}
        let Some(slot)=hero.abilities.iter().position(|id|id==&policy.ability) else {continue;};
        // Every slot the authority admits (E297: the fifth, e.g. A07 奮起 / B08 無盡牙, was never cast).
        if slot>=crate::runtime::HERO_ABILITY_SLOTS {continue;}
        if !hero.can_use_ability(&policy.ability) || !hero.cast_ready(&policy.ability) {continue;}
        let owner_rank=hero.get_ability_level(&policy.ability);
        if !(1..=255).contains(&owner_rank) {continue;}
        let rank=(owner_rank-1) as usize;
        let Some(def)=omoba_template_ids::ability_by_name(&policy.ability).and_then(omoba_template_ids::active_ability_const) else {continue;};
        let Some(level)=def.levels.get(rank) else {continue;};
        // v3 requires.has_status (E472): without the status its own hero must hold, the cast is refused.
        if def.requires_status.is_some_and(|status|!own_status(status)) {continue;}
        // A unit cast the authority refuses (dispatch, E460) is never aimed: the target's public
        // states rule out a hostile magic-immune or any unselectable unit, a hero-only ability anything but a hero.
        let aimable:Vec<SeenUnit>;
        let seen:&[SeenUnit]=if def.target_type==TargetTypeC::Unit {
            let pierces=crate::runtime::native::magic_immunity::ability_pierces(&policy.ability);
            aimable=seen.iter().filter(|unit|{
                let states=target_states(unit.canonical_id);
                !(states&crate::runtime::TARGET_STATE_MAGIC_IMMUNE!=0 && !pierces && unit.team!=team)
                    && states&crate::runtime::TARGET_STATE_UNSELECTABLE==0
                    && !(def.targets_heroes_only && unit.kind!=1)
            }).copied().collect();
            &aimable
        } else {seen};
        if matches!(policy.intent,BotAbilityIntent::SelfChannelRecovery {..})
            && (hero.mana_pool.is_none() || def.cast_type!=CastTypeC::Channeled || def.recovery_channel.is_none()) {continue;}
        let required_cost=if let Some(pool)=&hero.mana_pool {
            let Some(required)=cost(&policy.ability,owner_rank as u8) else {continue;};
            if required<Fixed64::ZERO || required>pool.current() {continue;}
            Some(required)
        } else {None};
        let mut target_pos=None;
        let target=match &policy.intent {
            // an item's intent (bots::items); validation admits it for no ability
            BotAbilityIntent::AllyAnchorUnit {..}=>continue,
            BotAbilityIntent::AllyHealOrCleanse {below_hp_per_mille}=>{
                if !def.cleanses_allies || def.target_type!=TargetTypeC::Unit || !(1..=1000).contains(below_hp_per_mille)
                    || seen.len()>512 || level.range<=Fixed64::ZERO || level.range>Fixed64::from_i32(10_000){continue;}
                let Some((unit,_))=seen.iter().filter(|u|u.team==team && team!=0 && u.kind==1 && u.hp_raw>0 && u.max_hp_raw>0
                    && (u.position-own).length_squared()<=level.range*level.range)
                    .filter_map(|unit|{
                        let priority=control_priority(unit.canonical_id).filter(|p|(1..=4).contains(p)).unwrap_or(5);
                        (priority<5 || i128::from(unit.hp_raw)*1000<i128::from(unit.max_hp_raw)*i128::from(*below_hp_per_mille)).then_some((unit,priority))
                    }).min_by(|(a,pa),(b,pb)|pa.cmp(pb)
                        .then_with(||(i128::from(a.hp_raw)*i128::from(b.max_hp_raw)).cmp(&(i128::from(b.hp_raw)*i128::from(a.max_hp_raw))))
                        .then_with(||(a.position-own).length_squared().raw().cmp(&(b.position-own).length_squared().raw()))
                        .then_with(||a.canonical_id.cmp(&b.canonical_id)))else{continue;};
                Some(unit.canonical_id as u32)
            },
            BotAbilityIntent::SelfChannelRecovery {below_hp_per_mille,threat_radius}=>{
                if !(1..=1000).contains(below_hp_per_mille) || !(1..=10_000).contains(threat_radius)
                    || seen.len()>512 || health.hp<=Fixed64::ZERO || health.mhp<=Fixed64::ZERO
                    || i128::from(health.hp.raw())*1000>=i128::from(health.mhp.raw())*i128::from(*below_hp_per_mille) {continue;}
                let radius=Fixed64::from_i32(*threat_radius as i32);
                let supported=|pos:Vec2|pos.x.raw().unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64
                    && pos.y.raw().unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64;
                if !supported(own) {continue;}
                // Neutral monsters are threats too. Only committed disclosed
                // combatants participate; private positions/targets are irrelevant.
                if seen.iter().any(|unit|unit.hp_raw>0 && unit.team!=team && matches!(unit.kind,1|2|3|4|5|6)
                    && (!supported(unit.position)||omoba_sim::terrain::point_in_swept_circle(unit.position,own,own,radius))) {continue;}
                None
            }
            BotAbilityIntent::AllyHeal {below_hp_per_mille}=>{
                let Some(unit)=seen.iter().filter(|u|u.team==team && u.kind==1 && u.hp_raw>0 && u.max_hp_raw>0
                    && i128::from(u.hp_raw)*1000<i128::from(u.max_hp_raw)*i128::from(*below_hp_per_mille)
                    && (u.position-own).length_squared()<=level.range*level.range)
                    .min_by(|a,b|(i128::from(a.hp_raw)*i128::from(b.max_hp_raw))
                        .cmp(&(i128::from(b.hp_raw)*i128::from(a.max_hp_raw)))
                        .then_with(||(a.position-own).length_squared().raw().cmp(&(b.position-own).length_squared().raw()))
                        .then_with(||a.canonical_id.cmp(&b.canonical_id))) else {continue;};
                Some(unit.canonical_id as u32)
            }
            BotAbilityIntent::AllyEngage {threat_radius}=>{
                if health.hp<=Fixed64::ZERO || seen.len()>512 {continue;}
                let radius=Fixed64::from_i32(*threat_radius as i32);
                let fighting=|ally:&SeenUnit|seen.iter().any(|u|u.hp_raw>0 && u.team!=team && u.team!=0 && u.kind==1
                    && (u.position-ally.position).length_squared()<=radius*radius);
                // Another allied hero (not the caster itself), in reach, in a disclosed fight.
                let Some(unit)=seen.iter().filter(|u|u.team==team && team!=0 && u.kind==1 && u.hp_raw>0 && u.max_hp_raw>0
                    && u.position!=own && (level.range==Fixed64::ZERO || (u.position-own).length_squared()<=level.range*level.range)
                    && fighting(u))
                    .min_by(|a,b|(i128::from(a.hp_raw)*i128::from(b.max_hp_raw))
                        .cmp(&(i128::from(b.hp_raw)*i128::from(a.max_hp_raw)))
                        .then_with(||(a.position-own).length_squared().raw().cmp(&(b.position-own).length_squared().raw()))
                        .then_with(||a.canonical_id.cmp(&b.canonical_id))) else {continue;};
                Some(unit.canonical_id as u32)
            }
            BotAbilityIntent::GlobalEnemyPoint {radius_key,min_targets}=>{
                if health.hp<=Fixed64::ZERO || level.range!=Fixed64::ZERO {continue;}
                let Some(radius)=def.extras.iter().find(|(key,_)|*key==radius_key).and_then(|(_,radii)|radii.get(rank)).copied() else {continue;};
                let mut heroes:Vec<_>=seen.iter().filter(enemy).filter(|u|u.kind==1).collect();
                heroes.sort_by_key(|u|priority(u));
                heroes.truncate(32);
                let center=heroes.iter().filter_map(|u|{
                    let count=heroes.iter().filter(|v|(v.position-u.position).length_squared()<=radius*radius).count();
                    (count>=usize::from(*min_targets)).then_some((std::cmp::Reverse(count),(u.position-own).length_squared().raw(),u.canonical_id,u.position))
                }).min_by_key(|(count,distance,id,_)|(*count,*distance,*id));
                let Some((_,_,_,center))=center else {continue;};
                let (Ok(x),Ok(y))=(i32::try_from(center.x.raw()),i32::try_from(center.y.raw())) else {continue;};
                target_pos=Some(crate::runtime::Vec2I {x,y});None
            }
            BotAbilityIntent::SelfHeal {below_hp_per_mille} => {
                if health.hp<=Fixed64::ZERO || health.mhp<=Fixed64::ZERO
                    || i128::from(health.hp.raw())*1000 >= i128::from(health.mhp.raw())*i128::from(*below_hp_per_mille) {continue;}
                None
            }
            BotAbilityIntent::SelfRecovery {below_hp_per_mille,below_mana_per_mille,restore_key}=>{
                if health.hp<=Fixed64::ZERO || health.mhp<=Fixed64::ZERO {continue;}
                let needs_hp=i128::from(health.hp.raw())*1000
                    <i128::from(health.mhp.raw())*i128::from(*below_hp_per_mille);
                let needs_mana=hero.mana_pool.as_ref().zip(required_cost).is_some_and(|(pool,required)|{
                    pool.maximum()>Fixed64::ZERO
                        && i128::from(pool.current().raw())*1000
                            <i128::from(pool.maximum().raw())*i128::from(*below_mana_per_mille)
                        && def.extras.iter().find(|(key,_)|*key==restore_key)
                            .and_then(|(_,amounts)|amounts.get(rank)).is_some_and(|amount|*amount>required)
                });
                if !needs_hp && !needs_mana {continue;}
                None
            }
            BotAbilityIntent::SelfManaRegeneration {below_hp_per_mille,below_mana_per_mille,rate_key,duration_key}=>{
                if health.hp<=Fixed64::ZERO || health.mhp<=Fixed64::ZERO {continue;}
                let needs_hp=i128::from(health.hp.raw())*1000
                    <i128::from(health.mhp.raw())*i128::from(*below_hp_per_mille);
                let extra=|key:&str|def.extras.iter().find(|(name,_)|*name==key)
                    .and_then(|(_,values)|values.get(rank)).copied();
                let needs_mana=hero.mana_pool.as_ref().zip(required_cost).is_some_and(|(pool,required)|{
                    if pool.maximum()<=Fixed64::ZERO || i128::from(pool.current().raw())*1000
                        >=i128::from(pool.maximum().raw())*i128::from(*below_mana_per_mille) {return false;}
                    let Some((rate,duration))=extra(rate_key).zip(extra(duration_key)) else {return false;};
                    let Some((before,after))=regen(&policy.ability,rate) else {return false;};
                    if before<Fixed64::ZERO || after<=before || duration<=Fixed64::ZERO {return false;}
                    // Compare capped balances with/without casting; ordinary
                    // recovery is not credited to the skill's marginal gain.
                    let current=i128::from(pool.current().raw());let maximum=i128::from(pool.maximum().raw());
                    let baseline=(current+i128::from(before.raw())*i128::from(duration.raw())/1024).min(maximum);
                    let candidate=(current-i128::from(required.raw())
                        +i128::from(after.raw())*i128::from(duration.raw())/1024).min(maximum);
                    candidate>baseline
                });
                if !needs_hp && !needs_mana {continue;}
                None
            }
            BotAbilityIntent::SelfArea {radius_key,min_targets}=>{
                if health.hp<=Fixed64::ZERO {continue;}
                let Some(radius)=def.extras.iter().find(|(key,_)|*key==radius_key).and_then(|(_,radii)|radii.get(rank)).copied() else {continue;};
                let inside:Vec<_>=seen.iter().filter(enemy).filter(|unit|omoba_sim::terrain::point_in_swept_circle(unit.position,own,own,radius)).collect();
                let heroes=inside.iter().filter(|unit|unit.kind==1).count();
                if !area_worth(heroes,inside.len(),*min_targets,lane,hero_near) {continue;}
                None
            }
            BotAbilityIntent::SelfAreaStatus {radius_key,min_targets,status}=>{
                if health.hp<=Fixed64::ZERO {continue;}
                let Some(radius)=def.extras.iter().find(|(key,_)|*key==radius_key).and_then(|(_,radii)|radii.get(rank)).copied() else {continue;};
                let inside:Vec<_>=seen.iter().filter(enemy).filter(|unit|omoba_sim::terrain::point_in_swept_circle(unit.position,own,own,radius)
                    && unit_status(unit.canonical_id,status)).collect();
                let heroes=inside.iter().filter(|unit|unit.kind==1).count();
                if !area_worth(heroes,inside.len(),*min_targets,lane,hero_near) {continue;}
                None
            }
            BotAbilityIntent::SelfAreaCorpses {radius_key,min_targets}=>{
                if health.hp<=Fixed64::ZERO {continue;}
                let Some(radius)=def.extras.iter().find(|(key,_)|*key==radius_key).and_then(|(_,radii)|radii.get(rank)).copied() else {continue;};
                if corpses_near(own,radius)<usize::from(*min_targets) {continue;}
                None
            }
            BotAbilityIntent::AllyAnchorPoint {types,..}=>{
                if health.hp<=Fixed64::ZERO {continue;}
                // Disclosed kinds: 2 lane creep, 4 summon or attacking structure, 5/6 other structures.
                let wanted=|kind:u8|types.iter().any(|name|match name.as_str() {
                    "creep"=>kind==2,"summon"=>kind==4,"building"=>matches!(kind,4|5|6),_=>false});
                let within=|unit:&&SeenUnit|level.range==Fixed64::ZERO || (unit.position-own).length_squared()<=level.range*level.range;
                let anchors:Vec<_>=seen.iter().filter(|unit|unit.team==team && unit.hp_raw>0 && unit.kind!=1 && wanted(unit.kind))
                    .filter(within).collect();
                let Some(goal)=seen.iter().filter(enemy).filter(|unit|unit.kind==1)
                    .min_by_key(|unit|((unit.position-own).length_squared().raw(),unit.canonical_id)).map(|unit|unit.position) else {continue;};
                let Some(anchor)=anchors.iter().min_by_key(|unit|((unit.position-goal).length_squared().raw(),unit.canonical_id)) else {continue;};
                // Not where the bot already stands (a teleport there does nothing).
                if (anchor.position-own).length_squared()<=Fixed64::from_i32(600)*Fixed64::from_i32(600) {continue;}
                let (Ok(x),Ok(y))=(i32::try_from(anchor.position.x.raw()),i32::try_from(anchor.position.y.raw())) else {continue;};
                target_pos=Some(crate::runtime::Vec2I {x,y});None
            }
            BotAbilityIntent::SelfEngage {threat_radius}=>{
                if health.hp<=Fixed64::ZERO {continue;}
                let radius=Fixed64::from_i32(*threat_radius as i32);
                // A jungler's threat is also the camp it fights, its focus (E585): it farms most of the game.
                let threat=|unit:&&SeenUnit|unit.kind==1
                    || (role==BotRole::Jungle && unit.kind==3 && Some(unit.canonical_id)==focus);
                if !seen.iter().filter(enemy).filter(threat).any(|unit|
                    omoba_sim::terrain::point_in_swept_circle(unit.position,own,own,radius)) {continue;}
                None
            }
            BotAbilityIntent::NearestTree {enemy_radius}=>{
                if !def.targets_tree || level.range<=Fixed64::ZERO {continue;}
                let radius=Fixed64::from_i32(*enemy_radius as i32);
                if *enemy_radius>0 && !seen.iter().filter(enemy).any(|unit|unit.kind==1
                    && (unit.position-own).length_squared()<=radius*radius) {continue;}
                let Some(tree)=tree_near(own,level.range) else {continue;};
                // The authority measures the cast to the tree's centre (windup_target_allowed); the
                // lookup admits a square that only reaches the range (B22 自然之力 OUT_OF_RANGE 11 of 11 on nobu).
                if (tree-own).length_squared()>level.range*level.range {continue;}
                let (Ok(x),Ok(y))=(i32::try_from(tree.x.raw()),i32::try_from(tree.y.raw())) else {continue;};
                target_pos=Some(crate::runtime::Vec2I {x,y});None
            }
            BotAbilityIntent::EnemyUnit|BotAbilityIntent::EnemyHero => {
                if level.range<=Fixed64::ZERO {continue;}
                let heroes_only=matches!(policy.intent,BotAbilityIntent::EnemyHero);
                let Some(unit)=seen.iter().filter(enemy)
                    .filter(|unit|!heroes_only || unit.kind==1)
                    .filter(|unit|(unit.position-own).length_squared()<=level.range*level.range)
                    .min_by_key(|unit|priority(unit)) else {continue;};
                Some(unit.canonical_id as u32)
            }
            BotAbilityIntent::ApproachEnemyPoint {min_distance,reach_key}=>{
                let minimum=Fixed64::from_i32(*min_distance as i32);
                let reach=match reach_key {
                    Some(key)=>{let Some(reach)=def.extras.iter().find(|(name,_)|*name==key).and_then(|(_,values)|values.get(rank).copied()) else {continue;}; reach}
                    None=>level.range,
                };
                let Some(unit)=seen.iter().filter(enemy)
                    .filter(|unit|{let distance=(unit.position-own).length_squared();
                        distance>minimum*minimum && distance<=reach*reach})
                    // Relocation requires a clear swept segment, not a route
                    // around the obstacle. Do not inspect hidden dynamic units.
                    .filter(|unit|relocation_clear(unit.position))
                    .min_by_key(|unit|priority(unit)) else {continue;};
                let (Ok(x),Ok(y))=(i32::try_from(unit.position.x.raw()),i32::try_from(unit.position.y.raw())) else {continue;};
                target_pos=Some(crate::runtime::Vec2I {x,y});None
            }
            BotAbilityIntent::ToggleEngage {threat_radius,on_status}=>{
                if health.hp<=Fixed64::ZERO {continue;}
                let radius=Fixed64::from_i32(*threat_radius as i32);
                let threatened=seen.iter().filter(enemy).any(|unit|unit.kind==1
                    && omoba_sim::terrain::point_in_swept_circle(unit.position,own,own,radius));
                // Cast only to change the switch: on for a fight, off when it is over.
                if threatened==own_status(on_status) {continue;}
                None
            }
            BotAbilityIntent::SummonNearEnemy {threat_radius}=>{
                let untargeted=def.target_type==TargetTypeC::None;
                if health.hp<=Fixed64::ZERO || (!untargeted && level.range<=Fixed64::ZERO) {continue;}
                let radius=Fixed64::from_i32(*threat_radius as i32);
                let Some(unit)=seen.iter().filter(enemy).filter(|unit|matches!(unit.kind,1|2))
                    .filter(|unit|(unit.position-own).length_squared()<=radius*radius)
                    .min_by_key(|unit|priority(unit)) else {continue;};
                if untargeted {None} else {
                let offset=unit.position-own;
                let point=if offset.length_squared()<=level.range*level.range {unit.position}
                    else {own+offset*(level.range/offset.length().max(Fixed64::from_raw(1)))};
                let (Ok(x),Ok(y))=(i32::try_from(point.x.raw()),i32::try_from(point.y.raw())) else {continue;};
                target_pos=Some(crate::runtime::Vec2I{x,y});None
                }
            }
            BotAbilityIntent::EnemyLine {min_targets}=>{
                if health.hp<=Fixed64::ZERO{continue;}
                let Some(width)=line_width(def,rank)else{continue;};
                let Some(reach)=line_reach(def,rank,level.range)else{continue;};
                let Some(point)=choose_line_target(own,reach,width,*min_targets,seen.iter().filter(enemy).collect(),if prioritizes_focus{focus}else{None})else{continue;};
                let (Ok(x),Ok(y))=(i32::try_from(point.x.raw()),i32::try_from(point.y.raw()))else{continue;};
                target_pos=Some(crate::runtime::Vec2I{x,y});None
            }
            BotAbilityIntent::EnemyCone {min_targets}=>{
                if health.hp<=Fixed64::ZERO {continue;}
                let Some(ticks)=def.cone_half_angle_ticks.get(rank).copied() else {continue;};
                let Some(point)=choose_cone_target(own,level.range,ticks,*min_targets,
                    seen.iter().filter(enemy).collect(),if prioritizes_focus {focus}else{None}) else {continue;};
                let (Ok(x),Ok(y))=(i32::try_from(point.x.raw()),i32::try_from(point.y.raw())) else {continue;};
                target_pos=Some(crate::runtime::Vec2I{x,y});None
            }
            BotAbilityIntent::EnemyPoint {radius_key,min_targets}
            |BotAbilityIntent::LeapEnemyPoint {radius_key,min_targets,..}=>{
                let Some((_,radii))=def.extras.iter().find(|(key,_)|*key==radius_key) else {continue;};
                let Some(radius)=radii.get(rank).copied() else {continue;};
                // A leap reaches its compiled reach extra; a placed area its cast range.
                let reach=if let BotAbilityIntent::LeapEnemyPoint {reach_key,..}=&policy.intent {
                    let Some(reach)=def.extras.iter().find(|(key,_)|*key==reach_key).and_then(|(_,values)|values.get(rank).copied()) else {continue;};
                    reach
                } else {level.range};
                let mut eligible:Vec<_>=seen.iter().filter(enemy).collect();
                eligible.sort_by_key(|u|priority(u));
                eligible.truncate(512);
                // Candidate centers are disclosed enemy poses; no hidden targets,
                // inferred motion or private aggro. Most covered targets wins.
                let center=eligible.iter().filter(|u|(u.position-own).length_squared()<=reach*reach).take(32)
                    .filter_map(|u|{
                        let covered:Vec<_>=eligible.iter().filter(|v|(v.position-u.position).length_squared()<=radius*radius).collect();
                        let heroes=covered.iter().filter(|v|v.kind==1).count();
                        area_worth(heroes,covered.len(),*min_targets,lane,hero_near).then_some((prioritizes_focus && Some(u.canonical_id)!=focus,
                            std::cmp::Reverse(heroes),std::cmp::Reverse(covered.len()),(u.position-own).length_squared().raw(),u.canonical_id,u.position))
                    }).min_by_key(|(assist,heroes,count,distance,id,_)|(*assist,*heroes,*count,*distance,*id));
                let Some((_,_,_,_,_,center))=center else {continue;};
                let (Ok(x),Ok(y))=(i32::try_from(center.x.raw()),i32::try_from(center.y.raw())) else {continue;};
                target_pos=Some(crate::runtime::Vec2I {x,y});None
            }
        };
        return Some(CastAbility {ability_index:u32::try_from(slot).ok()?,target_entity:target,target_pos});
    }
    None
}

// Bounded planning over committed observations only. Candidate axes point at
fn choose_line_target(own:Vec2,range:Fixed64,width:Fixed64,min_targets:u16,
    mut eligible:Vec<&SeenUnit>,focus:Option<u64>)->Option<Vec2> {
    if width<=Fixed64::ZERO||width>Fixed64::from_i32(5_000)||range<=Fixed64::ZERO||range>Fixed64::from_i32(10_000)
        ||!(1..=128).contains(&min_targets){return None;}
    let radius=width/Fixed64::from_i32(2);
    eligible.retain(|u|u.hp_raw>0&&omoba_sim::terrain::point_in_swept_circle(u.position,own,own,range+radius));
    eligible.sort_by_key(|u|(u.canonical_id,u.position.x.raw(),u.position.y.raw()));eligible.dedup_by_key(|u|u.canonical_id);
    if eligible.len()>512{return None;}
    let distance=|u:&SeenUnit|(u.position-own).length_squared().raw();
    eligible.sort_by_key(|u|(focus.is_some()&&Some(u.canonical_id)!=focus,distance(u),u.canonical_id));
    eligible.iter().filter(|u|u.position!=own).take(32).filter_map(|aim|{
        let end=omoba_sim::terrain::checked_line_endpoint(own,aim.position,range)?;
        let mut count=0;let mut covers_focus=false;
        for victim in &eligible {if omoba_sim::terrain::point_in_swept_circle(victim.position,own,end,radius){
            count+=1;covers_focus|=Some(victim.canonical_id)==focus;if count>128{return None;}
        }}
        (count>=usize::from(min_targets)).then_some((focus.is_some()&&!covers_focus,std::cmp::Reverse(count),distance(aim),aim.canonical_id,end))
    }).min_by_key(|(missing,count,distance,id,_)|(*missing,*count,*distance,*id)).map(|(_,_,_,_,end)|end)
}

// Bounded planning over committed observations only. Candidate axes point at
// disclosed enemy centers (32 aims,512 unique observations), not inferred motion.
// The same fixed-point sector predicate as execution scores each full-range aim.
fn choose_cone_target(own:Vec2,range:Fixed64,half_ticks:u16,min_targets:u16,
    mut eligible:Vec<&SeenUnit>,focus:Option<u64>)->Option<Vec2> {
    let supported=|p:Vec2|p.x.raw().unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64
        &&p.y.raw().unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64;
    if !supported(own)||range<=Fixed64::ZERO||range>Fixed64::from_i32(10_000)
        || !(1..=1024).contains(&half_ticks)||!(1..=128).contains(&min_targets){return None;}
    eligible.retain(|u|u.hp_raw>0&&supported(u.position)
        &&omoba_sim::terrain::point_in_swept_circle(u.position,own,own,range));
    eligible.sort_by_key(|u|(u.canonical_id,u.position.x.raw(),u.position.y.raw()));
    eligible.dedup_by_key(|u|u.canonical_id);
    // Never undercount a known crowded sector by silently dropping observations.
    if eligible.len()>512{return None;}
    let distance=|u:&SeenUnit|(u.position-own).length_squared().raw();
    eligible.sort_by_key(|u|(focus.is_some()&&Some(u.canonical_id)!=focus,distance(u),u.canonical_id));
    eligible.iter().filter(|u|u.position!=own
        &&i32::try_from(u.position.x.raw()).is_ok()&&i32::try_from(u.position.y.raw()).is_ok()).take(32)
        .filter_map(|aim|{
            let delta=aim.position-own;let axis=omoba_sim::trig::atan2(delta.y,delta.x);
            let mut count=0usize;let mut covers_focus=false;
            for victim in &eligible {
                if omoba_sim::sector::point_in_sector(victim.position,own,axis,range,half_ticks){
                    count+=1;covers_focus|=Some(victim.canonical_id)==focus;
                    if count>128{return None;}
                }
            }
            (count>=usize::from(min_targets)).then_some((focus.is_some()&&!covers_focus,
                std::cmp::Reverse(count),distance(aim),aim.canonical_id,aim.position))
        }).min_by_key(|(missing,count,distance,id,_)|(*missing,*count,*distance,*id))
        .map(|(_,_,_,_,point)|point)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn channel_intent_is_closed_explicit_bounded_and_requires_managed_mana() {
        let valid=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"apprentice_mend".into(),intent:BotAbilityIntent::SelfChannelRecovery {below_hp_per_mille:650,threat_radius:1000}};
        assert!(validate_policies(&[valid.clone()]).is_ok());assert!(requires_mana(&[valid.clone()]));
        let mut bad=valid.clone();bad.ability="lumen_mend".into();assert!(validate_policies(&[bad]).is_err());
        let mut bad=valid.clone();bad.intent=BotAbilityIntent::SelfHeal {below_hp_per_mille:650};assert!(validate_policies(&[bad]).is_err());
        for (hp,radius) in [(0,1000),(1001,1000),(650,0),(650,10001)] {
            let mut bad=valid.clone();bad.intent=BotAbilityIntent::SelfChannelRecovery {below_hp_per_mille:hp,threat_radius:radius};assert!(validate_policies(&[bad]).is_err());
        }
        assert!(validate_policies(&[valid.clone(),valid]).is_err());
        assert!(serde_json::from_str::<BotAbilityIntent>(r#"{"kind":"self_channel_recovery","below_hp_per_mille":650,"threat_radius":1000,"hidden_enemy":true}"#).is_err());
    }
    #[test]
    fn channel_intent_uses_disclosed_threat_boundary_resources_root_and_bounded_geometry() {
        use crate::runtime::ability_runtime::ManaPool;
        let policy=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"apprentice_mend".into(),intent:BotAbilityIntent::SelfChannelRecovery {below_hp_per_mille:650,threat_radius:1000}};
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["lumen_bolt".into(),"apprentice_mend".into()];hero.ability_levels.insert("apprentice_mend".into(),1);
        hero.mana_pool=Some(ManaPool::new(Fixed64::from_i32(100),Fixed64::from_i32(200)).unwrap());
        let mut hp=property(Fixed64::from_i32(200),Fixed64::ZERO);hp.hp=Fixed64::from_i32(50);hp.mhp=Fixed64::from_i32(200);
        let decide=|hero:&Hero,hp:&CProperty,seen:&[SeenUnit],cost:Option<Fixed64>|choose_cast_with_attack_priority(hero,BotRole::Mid,1,Vec2::ZERO,hp,seen,&[policy.clone()],None,true,true,|_,_|cost,|_,_|None,|_|false);
        let cost=Some(Fixed64::from_i32(90));let cast=decide(&hero,&hp,&[],cost).unwrap();
        assert_eq!((cast.ability_index,cast.target_entity,cast.target_pos),(1,None,None));
        let enemy=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(1000),Fixed64::ZERO),team:2,kind:1,owner_player_id:2,hp_raw:1,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        assert!(decide(&hero,&hp,&[enemy],cost).is_none(),"inclusive authored threat boundary");
        let far=SeenUnit {position:Vec2::new(Fixed64::from_raw(1000*1024+1),Fixed64::ZERO),..enemy};
        assert_eq!(decide(&hero,&hp,&[far],cost).unwrap(),cast);
        let neutral=SeenUnit {team:0,kind:3,..enemy};assert!(decide(&hero,&hp,&[neutral],cost).is_none());
        for kind in [2,4,5,6]{assert!(decide(&hero,&hp,&[SeenUnit {kind,..enemy}],cost).is_none());}
        let ally=SeenUnit {team:1,..enemy};let dead=SeenUnit {hp_raw:0,..enemy};
        assert_eq!(decide(&hero,&hp,&[ally,dead,far],cost),decide(&hero,&hp,&[far,dead,ally],cost));
        let malformed=SeenUnit {position:Vec2::new(Fixed64::from_raw(omoba_sim::terrain::MAX_GEOMETRY_RAW+1),Fixed64::ZERO),..enemy};
        assert!(decide(&hero,&hp,&[malformed],cost).is_none());assert!(decide(&hero,&hp,&vec![ally;513],cost).is_none());
        assert!(decide(&hero,&hp,&[],None).is_none());
        hero.mana_pool=Some(ManaPool::new(Fixed64::from_i32(89),Fixed64::from_i32(200)).unwrap());assert!(decide(&hero,&hp,&[],cost).is_none());
        hero.mana_pool=None;assert!(decide(&hero,&hp,&[],cost).is_none());
        hero.mana_pool=Some(ManaPool::full(Fixed64::from_i32(200)).unwrap());hp.hp=Fixed64::from_i32(130);assert!(decide(&hero,&hp,&[],cost).is_none());
        hp.hp=Fixed64::from_i32(50);hero.ability_cooldowns.insert("apprentice_mend".into(),Fixed64::ONE);assert!(decide(&hero,&hp,&[],cost).is_none());
    }
    #[test]
    fn support_guard_focus_prioritizes_offensive_intents_without_overriding_heal_order() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["lumen_bolt".into(),"vanguard_resolve".into(),"ranger_volley".into(),"lumen_aid".into()];
        for ability in hero.abilities.clone() {hero.ability_levels.insert(ability,1);}
        let hp=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let enemy=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(400),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:2,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let close=SeenUnit {canonical_id:8,position:Vec2::new(Fixed64::from_i32(50),Fixed64::ZERO),..enemy};
        let cast=|seen:&[SeenUnit],policies:&[BotAbilityPolicy]|choose_cast_with_resources(&hero,BotRole::Support,
            1,Vec2::ZERO,&hp,seen,policies,Some(7),|_,_|None,|_,_|None);
        for (ability,intent,slot) in [
            ("lumen_bolt",BotAbilityIntent::EnemyUnit,0),
            ("vanguard_resolve",BotAbilityIntent::ApproachEnemyPoint {min_distance:100,reach_key:None},1),
            ("ranger_volley",BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:1},2),
        ] {
            let policy=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:ability.into(),intent};
            let result=cast(&[close,enemy],&[policy.clone()]).unwrap();
            assert_eq!(result.ability_index,slot);
            if slot==0 {assert_eq!(result.target_entity,Some(7));}
            else {assert_eq!(result.target_pos.unwrap().x,enemy.position.x.raw() as i32);}
            for invalid in [SeenUnit {team:1,..enemy},SeenUnit {hp_raw:0,..enemy},
                SeenUnit {position:Vec2::new(Fixed64::from_i32(10_001),Fixed64::ZERO),..enemy}] {
                assert!(cast(&[invalid],&[policy.clone()]).is_none());
            }
        }
        let heal=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"lumen_aid".into(),intent:BotAbilityIntent::AllyHeal {below_hp_per_mille:600}};
        let damage=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"lumen_bolt".into(),intent:BotAbilityIntent::EnemyUnit};
        let ally=SeenUnit {canonical_id:9,team:1,hp_raw:20,owner_player_id:3,..close};
        let result=cast(&[enemy,ally],&[heal,damage]).unwrap();
        assert_eq!((result.ability_index,result.target_entity),(3,Some(9)),"author heal-first priority must survive guard focus");
    }

    #[test]
    fn bot_cleanse_policy_prioritizes_public_control_then_health_without_target_private_queries() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());hero.abilities=vec!["lumen_aid".into()];hero.ability_levels.insert("lumen_aid".into(),1);
        let policy=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"lumen_aid".into(),intent:BotAbilityIntent::AllyHealOrCleanse {below_hp_per_mille:600}};
        assert!(validate_policies(&[policy.clone()]).is_ok());
        let hp=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let ally=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(250),Fixed64::ZERO),team:1,kind:1,owner_player_id:3,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let injured=SeenUnit {canonical_id:8,hp_raw:20,..ally};let second=SeenUnit {canonical_id:9,..ally};
        let decide=|seen:&[SeenUnit],observed:&[(u64,u8)]|choose_cast_with_control_observation(&hero,BotRole::Support,1,Vec2::ZERO,&hp,seen,&[policy.clone()],None,true,true,|_,_|None,|_,_|None,|_|panic!("nonmovement"),|id|observed.iter().find(|(key,_)|*key==id).map(|(_,p)|*p),&|_|false,&|_|0,&|_,_|None,&|_,_|false,&|_,_|0);
        assert!(decide(&[ally],&[]).is_none());
        assert_eq!(decide(&[injured,ally],&[(7,1)]).unwrap().target_entity,Some(7));
        assert_eq!(decide(&[ally,injured],&[(7,1)]).unwrap().target_entity,Some(7));
        assert_eq!(decide(&[ally,second],&[(7,4),(9,2)]).unwrap().target_entity,Some(9));
        assert_eq!(decide(&[second,ally],&[(7,1),(9,1)]).unwrap().target_entity,Some(7));
        assert_eq!(decide(&[injured],&[]).unwrap().target_entity,Some(8));
        for invalid in [SeenUnit {team:2,..ally},SeenUnit {kind:2,..ally},SeenUnit {hp_raw:0,..ally},SeenUnit {position:Vec2::new(Fixed64::from_i32(601),Fixed64::ZERO),..ally}] {
            let result=choose_cast_with_control_observation(&hero,BotRole::Support,1,Vec2::ZERO,&hp,&[invalid],&[policy.clone()],None,false,false,|_,_|None,|_,_|None,|_|true,|_|panic!("invalid candidate must not query public status"),&|_|false,&|_|0,&|_,_|None,&|_,_|false,&|_,_|0);assert!(result.is_none());}
        assert!(decide(&[],&[(7,1)]).is_none());assert!(decide(&[ally],&[(7,0),(9,1)]).is_none());assert!(decide(&[ally],&[(7,255)]).is_none());
        for value in [0,1001]{let mut bad=policy.clone();bad.intent=BotAbilityIntent::AllyHealOrCleanse{below_hp_per_mille:value};assert!(validate_policies(&[bad]).is_err());}
        for ability in ["lumen_bolt","vanguard_strike","lumen_touch"]{let mut bad=policy.clone();bad.ability=ability.into();assert!(validate_policies(&[bad]).is_err());}
        assert!(serde_json::from_str::<BotAbilityIntent>(r#"{"kind":"ally_heal_or_cleanse","below_hp_per_mille":600,"hidden_control":true}"#).is_err());
    }

    #[test]
    fn ally_heal_bot_uses_disclosed_relative_health_and_stable_range() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["lumen_aid".into()];hero.ability_levels.insert("lumen_aid".into(),1);
        let policy=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"lumen_aid".into(),intent:BotAbilityIntent::AllyHeal {below_hp_per_mille:600}};
        assert!(validate_policies(&[policy.clone()]).is_ok());
        let ally=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(250),Fixed64::ZERO),
            team:1,kind:1,owner_player_id:3,hp_raw:30,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let lower=SeenUnit {canonical_id:8,hp_raw:40,max_hp_raw:200,..ally};
        let hp=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let decide=|seen:&[SeenUnit]|choose_cast(&hero,BotRole::Support,1,Vec2::ZERO,&hp,seen,&[policy.clone()]);
        assert_eq!(decide(&[ally,lower]).unwrap().target_entity,Some(8));
        assert_eq!(decide(&[lower,ally]).unwrap().target_entity,Some(8));
        for invalid in [SeenUnit {team:2,..ally},SeenUnit {team:0,..ally},SeenUnit {hp_raw:0,..ally},
            SeenUnit {kind:2,..ally},SeenUnit {hp_raw:60,..ally},
            SeenUnit {position:Vec2::new(Fixed64::from_i32(601),Fixed64::ZERO),..ally}] {
            assert!(decide(&[invalid]).is_none());
        }
        assert!(decide(&[]).is_none());
    }
    /// E473：以樹為目標的技能施放在施法距離內最近的立著的樹；設定 enemy_radius 時要附近有敵方英雄；
    /// 沒有樹就不施放。
    #[test]
    fn a_tree_ability_aims_at_the_nearest_standing_tree_in_range() {
        let ability = "b22_force_of_nature";
        let def = omoba_template_ids::ability_by_name(ability).and_then(omoba_template_ids::active_ability_const).expect("tree ability");
        assert!(def.targets_tree);
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec![ability.into()];hero.ability_levels.insert(ability.into(),1);
        let hp=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let tree=Vec2::new(Fixed64::from_i32(120),Fixed64::from_i32(40));
        let enemy=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:2,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let decide=|enemy_radius:u32,seen:&[SeenUnit],trees:&dyn Fn(Vec2,Fixed64)->Option<Vec2>|choose_cast_with_control_observation(&hero,BotRole::Mid,1,Vec2::ZERO,&hp,seen,
            &[BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:ability.into(),intent:BotAbilityIntent::NearestTree {enemy_radius}}],None,false,false,
            |_,_|None,|_,_|None,|_|true,|_|None,&|_|false,&|_|0,trees,&|_,_|false,&|_,_|0).map(|cast|cast.target_pos);
        let reach=def.levels[0].range;
        let near=|from:Vec2,within:Fixed64|(from==Vec2::ZERO && within==reach).then_some(tree);
        let at=Some(crate::runtime::Vec2I {x:tree.x.raw() as i32,y:tree.y.raw() as i32});
        assert_eq!(decide(0,&[],&near),Some(at),"the nearest standing tree within the cast range");
        assert_eq!(decide(0,&[],&|_,_|None),None,"no tree, no cast");
        let beyond=Vec2::new(reach,Fixed64::from_i32(40));
        assert_eq!(decide(0,&[],&|_,_|Some(beyond)),None,"a tree whose centre is out of the cast range");
        // Out of range: walk to the tree's nearest reachable face, at the cast range from its centre.
        let policy=[BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:ability.into(),intent:BotAbilityIntent::NearestTree {enemy_radius:0}}];
        let far=Vec2::new(Fixed64::from_i32(600),Fixed64::from_i32(40));
        let stand=reach-Fixed64::from_i32(TREE_APPROACH_MARGIN);
        let only_far=|_:Vec2,within:Fixed64|(within>reach).then_some(far);
        let approach=|reachable:&dyn Fn(Vec2)->bool|tree_approach(&hero,1,Vec2::ZERO,&[],&policy,|_,_|None,&only_far,reachable);
        assert_eq!(approach(&|_|true),Some(far-Vec2::new(stand,Fixed64::ZERO)),"the face toward the hero");
        let west=far-Vec2::new(stand,Fixed64::ZERO);
        assert_eq!(approach(&|p|p!=west),Some(far+Vec2::new(Fixed64::ZERO,-stand)),"the next reachable face");
        assert_eq!(approach(&|_|false),None,"no reachable face");
        assert_eq!(tree_approach(&hero,1,Vec2::ZERO,&[],&policy,|_,_|None,&near,&|_|true),None,"a tree in range is cast at, not walked to");
        assert_eq!(tree_approach(&hero,1,Vec2::ZERO,&[],&policy,|_,_|None,&|_,_|None,&|_|true),None,"no tree near");
        let waits=[BotAbilityPolicy {intent:BotAbilityIntent::NearestTree {enemy_radius:500},..policy[0].clone()}];
        assert_eq!(tree_approach(&hero,1,Vec2::ZERO,&[],&waits,|_,_|None,&only_far,&|_|true),None,"no enemy hero about");
        assert!(tree_approach(&hero,1,Vec2::ZERO,&[enemy],&waits,|_,_|None,&only_far,&|_|true).is_some());
        assert_eq!(decide(500,&[],&near),None,"no enemy hero about");
        assert_eq!(decide(500,&[enemy],&near),Some(at),"an enemy hero within the radius");
        assert!(validate_policies(&[BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:ability.into(),intent:BotAbilityIntent::NearestTree {enemy_radius:0}}]).is_ok());
        assert!(validate_policies(&[BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"vanguard_strike".into(),intent:BotAbilityIntent::NearestTree {enemy_radius:0}}]).is_err());
    }
    /// E460：authority 會拒絕的單位技能目標（敵方魔免、不可選取）從公開的目標狀態排除，
    /// 改選下一個合法目標；友方魔免不受影響。
    #[test]
    fn a_unit_cast_skips_targets_whose_public_states_the_authority_refuses() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["vanguard_strike".into()];hero.ability_levels.insert("vanguard_strike".into(),1);
        let hp=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let near=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:2,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let far=SeenUnit {canonical_id:8,position:Vec2::new(Fixed64::from_i32(250),Fixed64::ZERO),..near};
        let policy=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"vanguard_strike".into(),intent:BotAbilityIntent::EnemyUnit};
        let decide=|states:&dyn Fn(u64)->u8|choose_cast_with_control_observation(&hero,BotRole::Mid,1,Vec2::ZERO,&hp,&[near,far],
            &[policy.clone()],None,false,false,|_,_|None,|_,_|None,|_|true,|_|None,&|_|false,states,&|_,_|None,&|_,_|false,&|_,_|0).map(|cast|cast.target_entity);
        assert_eq!(decide(&|_|0),Some(Some(7)),"the nearest without states");
        for refused in [crate::runtime::TARGET_STATE_MAGIC_IMMUNE,crate::runtime::TARGET_STATE_UNSELECTABLE] {
            assert_eq!(decide(&|id|if id==7 {refused} else {0}),Some(Some(8)),"states {refused}: the next legal target");
            assert_eq!(decide(&|_|refused),None,"no legal target, no cast");
        }
        assert_eq!(decide(&|id|if id==7 {crate::runtime::TARGET_STATE_INVULNERABLE} else {0}),Some(Some(7)),
            "the authority admits a unit cast at an invulnerable unit");
    }
    #[test]
    fn jungle_assist_skills_share_focus_but_never_accept_hidden_or_friendly_targets() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["vanguard_strike".into()];hero.ability_levels.insert("vanguard_strike".into(),1);
        let hp=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let enemy=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(250),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:2,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let neutral=SeenUnit {canonical_id:8,position:Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO),
            team:0,kind:3,owner_player_id:0,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let policy=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"vanguard_strike".into(),intent:BotAbilityIntent::EnemyUnit};
        let decide=|seen:&[SeenUnit],assist|choose_cast_with_resources(&hero,BotRole::Jungle,1,Vec2::ZERO,
            &hp,seen,&[policy.clone()],assist,|_,_|None,|_,_|None);
        assert_eq!(decide(&[enemy,neutral],Some(7)).unwrap().target_entity,Some(7));
        assert_eq!(decide(&[neutral,enemy],Some(7)).unwrap().target_entity,Some(7));
        assert_eq!(decide(&[enemy,neutral],None).unwrap().target_entity,Some(8));
        assert!(decide(&[neutral],Some(7)).unwrap().target_entity==Some(8),"hidden assist cannot be reconstructed");
        for invalid in [SeenUnit {team:1,..enemy},SeenUnit {hp_raw:0,..enemy},
            SeenUnit {position:Vec2::new(Fixed64::from_i32(301),Fixed64::ZERO),..enemy}] {
            assert_eq!(decide(&[invalid,neutral],Some(7)).unwrap().target_entity,Some(8));
        }
        let mut point_hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        point_hero.abilities=vec!["vanguard_resolve".into(),"ranger_volley".into()];
        for id in ["vanguard_resolve","ranger_volley"] {point_hero.ability_levels.insert(id.into(),1);}
        let enemy=SeenUnit {position:Vec2::new(Fixed64::from_i32(350),Fixed64::ZERO),..enemy};
        let neutral=SeenUnit {position:Vec2::new(Fixed64::from_i32(320),Fixed64::ZERO),..neutral};
        for policy in [
            BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"vanguard_resolve".into(),intent:BotAbilityIntent::ApproachEnemyPoint {min_distance:300,reach_key:None}},
            BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"ranger_volley".into(),intent:BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:1}},
        ] {
            let cast=choose_cast_with_resources(&point_hero,BotRole::Jungle,1,Vec2::ZERO,&hp,
                &[neutral,enemy],&[policy.clone()],Some(7),|_,_|None,|_,_|None).unwrap();
            assert_eq!(cast.target_pos.unwrap().x,Fixed64::from_i32(350).raw() as i32);
            let cast=choose_cast_with_resources(&point_hero,BotRole::Jungle,1,Vec2::ZERO,&hp,
                &[neutral,enemy],&[policy],None,|_,_|None,|_,_|None).unwrap();
            assert_eq!(cast.target_pos.unwrap().x,Fixed64::from_i32(320).raw() as i32);
        }
    }
    /// E170: a hero-only unit skill (source DOTA_UNIT_TARGET_HERO, e.g. C17W) never picks a
    /// creep or camp, however close; with no disclosed enemy hero in range it is not cast.
    #[test]
    fn enemy_hero_intent_targets_only_disclosed_enemy_heroes() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["vanguard_strike".into()];hero.ability_levels.insert("vanguard_strike".into(),1);
        let hp=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let enemy_hero=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(250),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:2,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let creep=SeenUnit {canonical_id:9,kind:2,owner_player_id:0,position:Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO),..enemy_hero};
        let policy=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"vanguard_strike".into(),intent:BotAbilityIntent::EnemyHero};
        assert!(validate_policies(&[policy.clone()]).is_ok());
        let decide=|seen:&[SeenUnit]|choose_cast_with_resources(&hero,BotRole::Mid,1,Vec2::ZERO,
            &hp,seen,&[policy.clone()],None,|_,_|None,|_,_|None);
        assert_eq!(decide(&[creep,enemy_hero]).unwrap().target_entity,Some(7));
        let unit_policy=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"vanguard_strike".into(),intent:BotAbilityIntent::EnemyUnit};
        assert_eq!(choose_cast_with_resources(&hero,BotRole::Mid,1,Vec2::ZERO,&hp,&[creep,enemy_hero],&[unit_policy],
            None,|_,_|None,|_,_|None).unwrap().target_entity,Some(9),"enemy_unit takes the nearer creep");
        assert!(decide(&[creep]).is_none(),"creeps are never targeted");
        assert!(decide(&[SeenUnit {position:Vec2::new(Fixed64::from_i32(301),Fixed64::ZERO),..enemy_hero}]).is_none());
    }
    #[test]
    fn mana_regeneration_bot_compares_marginal_gain_and_blocks_active_source() {
        use crate::runtime::ability_runtime::ManaPool;
        let policy=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"ranger_patch".into(),intent:BotAbilityIntent::SelfManaRegeneration {
            below_hp_per_mille:600,below_mana_per_mille:1000,
            rate_key:"mana_buff_value".into(),duration_key:"mana_buff_duration".into()}};
        assert!(validate_policies(&[policy.clone()]).is_ok());assert!(requires_mana(&[policy.clone()]));
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["ranger_patch".into()];hero.ability_levels.insert("ranger_patch".into(),1);
        hero.mana_pool=Some(ManaPool::new(Fixed64::from_i32(60),Fixed64::from_i32(280)).unwrap());
        let mut hp=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let decide=|hero:&Hero,hp:&CProperty,cost:Fixed64,rates:Option<(Fixed64,Fixed64)>|
            choose_cast_with_resources(hero,BotRole::Carry,1,Vec2::ZERO,hp,&[],&[policy.clone()],None,
                |_,_|Some(cost),|id,bonus|{assert_eq!(id,"ranger_patch");assert_eq!(bonus,Fixed64::from_i32(2));rates});
        let rates=Some((Fixed64::from_i32(5),Fixed64::from_i32(7)));
        assert!(decide(&hero,&hp,Fixed64::from_i32(45),rates).is_none());
        assert!(decide(&hero,&hp,Fixed64::from_i32(12),rates).is_none(),"equal gain/cost is not useful");
        let before=hero.mana_pool.clone();
        assert!(decide(&hero,&hp,Fixed64::from_i32(11),rates).is_some());
        assert_eq!(hero.mana_pool,before);
        assert!(decide(&hero,&hp,Fixed64::ZERO,None).is_none(),"active or unknown source fails closed");
        hero.mana_pool=Some(ManaPool::new(Fixed64::from_i32(250),Fixed64::from_i32(280)).unwrap());
        assert!(decide(&hero,&hp,Fixed64::from_i32(11),rates).is_none(),"baseline already fills the pool");
        hp.hp=Fixed64::from_i32(59);
        assert!(decide(&hero,&hp,Fixed64::from_i32(45),None).is_some(),"healing can still refresh its buff");
        hp.hp=hp.mhp;hero.mana_pool=None;
        assert!(decide(&hero,&hp,Fixed64::ZERO,rates).is_none());
        for (rate_key,duration_key) in [("missing","mana_buff_duration"),("mana_buff_value","missing"),
            ("mana_buff_duration","heal")] {
            assert!(validate_policies(&[BotAbilityPolicy {intent:BotAbilityIntent::SelfManaRegeneration {
                below_hp_per_mille:600,below_mana_per_mille:500,rate_key:rate_key.into(),duration_key:duration_key.into()},
                ..policy.clone()}]).is_err());
        }
    }
    #[test]
    fn mana_recovery_bot_requires_positive_gain_and_preserves_healing() {
        use crate::runtime::ability_runtime::ManaPool;
        let policy=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"lumen_touch".into(),intent:BotAbilityIntent::SelfRecovery {
            below_hp_per_mille:600,below_mana_per_mille:500,restore_key:"mana_restore".into()}};
        assert!(validate_policies(&[policy.clone()]).is_ok());
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["lumen_touch".into()];
        hero.ability_levels.insert("lumen_touch".into(),1);
        hero.mana_pool=Some(ManaPool::new(Fixed64::from_i32(40),Fixed64::from_i32(100)).unwrap());
        let mut health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let decide=|hero:&Hero,health:&CProperty,required:Fixed64|choose_cast_with_mana_budget(
            hero,BotRole::Mid,1,Vec2::ZERO,health,&[],&[policy.clone()],|_,_|Some(required));
        let before=hero.mana_pool.clone();
        assert!(decide(&hero,&health,Fixed64::from_i32(20)).is_none(),"zero net gain is not recovery");
        assert!(decide(&hero,&health,Fixed64::from_i32(40)).is_none(),"negative net gain is not recovery");
        assert_eq!(decide(&hero,&health,Fixed64::from_i32(19)).unwrap().ability_index,0);
        assert_eq!(hero.mana_pool,before,"planning must not spend");
        hero.mana_pool=Some(ManaPool::new(Fixed64::from_i32(50),Fixed64::from_i32(100)).unwrap());
        assert!(decide(&hero,&health,Fixed64::ZERO).is_none(),"threshold is strict");
        health.hp=Fixed64::from_i32(59);
        assert!(decide(&hero,&health,Fixed64::from_i32(50)).is_some(),"HP branch retains priority");
        assert!(decide(&hero,&health,Fixed64::from_i32(51)).is_none(),"HP does not bypass budget");
        hero.mana_pool=None;
        assert!(decide(&hero,&health,Fixed64::ZERO).is_some());
        health.hp=health.mhp;
        assert!(decide(&hero,&health,Fixed64::ZERO).is_none(),"None never invents a mana need");
        hero.mana_pool=Some(ManaPool::full(Fixed64::ZERO).unwrap());
        assert!(decide(&hero,&health,Fixed64::ZERO).is_none());
        health.hp=Fixed64::ZERO;
        assert!(decide(&hero,&health,Fixed64::ZERO).is_none());
        for intent in [
            BotAbilityIntent::SelfRecovery {below_hp_per_mille:1001,below_mana_per_mille:500,restore_key:"mana_restore".into()},
            BotAbilityIntent::SelfRecovery {below_hp_per_mille:0,below_mana_per_mille:0,restore_key:"mana_restore".into()},
            BotAbilityIntent::SelfRecovery {below_hp_per_mille:0,below_mana_per_mille:1001,restore_key:"mana_restore".into()},
            BotAbilityIntent::SelfRecovery {below_hp_per_mille:0,below_mana_per_mille:500,restore_key:"missing".into()},
        ] {assert!(validate_policies(&[BotAbilityPolicy {intent,..policy.clone()}]).is_err());}
        let mana_only=BotAbilityPolicy {intent:BotAbilityIntent::SelfRecovery {
            below_hp_per_mille:0,below_mana_per_mille:500,restore_key:"mana_restore".into()},..policy};
        assert!(validate_policies(&[mana_only.clone()]).is_ok());
        assert!(requires_mana(&[mana_only]));
        assert!(serde_json::from_str::<BotAbilityIntent>(r#"{"kind":"self_recovery","below_hp_per_mille":0,"below_mana_per_mille":500,"restore_key":"mana_restore","hidden_enemy":true}"#).is_err());
    }
    #[test]
    fn mana_budget_bot_skips_unaffordable_or_invalid_cost_without_spending() {
        use crate::runtime::ability_runtime::{checked_mana_cost,ManaPool};
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["lumen_touch".into(),"lumen_bolt".into()];
        hero.ability_levels.insert("lumen_touch".into(),1);
        hero.ability_levels.insert("lumen_bolt".into(),1);
        let mut health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        health.hp=Fixed64::from_i32(20);
        let unit=SeenUnit {canonical_id:(1<<32)|77,position:Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:2,hp_raw:1,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let policies=[damage(),heal()];
        let decide=|hero:&Hero,multiplier:Fixed64|choose_cast_with_mana_budget(
            hero,BotRole::Mid,1,Vec2::ZERO,&health,&[unit],&policies,|id,rank| {
                assert_eq!(rank,1);
                checked_mana_cost(if id=="lumen_bolt" {70.0} else {45.0},multiplier).ok()
            });
        hero.mana_pool=Some(ManaPool::new(Fixed64::from_i32(45),Fixed64::from_i32(100)).unwrap());
        let before=hero.mana_pool.clone();
        assert_eq!(decide(&hero,Fixed64::ONE).unwrap().ability_index,0);
        assert_eq!(hero.mana_pool,before);
        hero.mana_pool.as_mut().unwrap().spend(Fixed64::from_raw(1)).unwrap();
        assert!(decide(&hero,Fixed64::ONE).is_none());
        assert_eq!(decide(&hero,Fixed64::from_raw(512)).unwrap().ability_index,1);
        assert!(decide(&hero,Fixed64::from_i32(2)).is_none());
        hero.mana_pool=Some(ManaPool::full(Fixed64::ZERO).unwrap());
        assert_eq!(decide(&hero,Fixed64::ZERO).unwrap().ability_index,1);
        assert!(decide(&hero,Fixed64::from_raw(-1)).is_none());
        assert!(choose_cast_with_mana_budget(&hero,BotRole::Mid,1,Vec2::ZERO,
            &health,&[unit],&policies,|_,_|None).is_none());
        hero.mana_pool=None;
        assert_eq!(choose_cast_with_mana_budget(&hero,BotRole::Mid,1,Vec2::ZERO,
            &health,&[unit],&policies,|_,_|panic!("legacy must not ask for mana")).unwrap().ability_index,1);
        hero.ability_levels.insert("lumen_bolt".into(),256);
        assert_eq!(decide(&hero,Fixed64::ONE).unwrap().ability_index,0);
    }
    fn learn(ability:&str,rank:u8) -> BotAbilityLearningStep {BotAbilityLearningStep {ability:ability.into(),rank}}
    #[test]
    fn role_bot_learning_validates_bounded_consecutive_catalog_ranks() {
        assert!(validate_learning(&[learn("lumen_lance",1),learn("lumen_bolt",1),learn("lumen_lance",2)]).is_ok());
        for steps in [vec![learn("missing",1)],vec![learn("lumen_bolt",0)],vec![learn("lumen_bolt",2)],
            vec![learn("lumen_bolt",1),learn("lumen_bolt",1)],
            (1..=5).map(|rank|learn("lumen_bolt",rank)).collect(),vec![learn("lumen_bolt",1);257],vec![learn("*",1)]] {
            assert!(validate_learning(&steps).is_err());
        }
        assert!(serde_json::from_str::<BotAbilityLearningStep>(r#"{"ability":"lumen_bolt","rank":1,"slot":0}"#).is_err());
    }
    #[test]
    fn role_bot_learning_skips_gated_missing_completed_and_invalid_owner_ranks() {
        let steps=[learn("lumen_lance",1),learn("lumen_lance",2),learn("lumen_bolt",1)];
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["lumen_bolt".into(),"lumen_lance".into()];
        hero.level=1;hero.skill_points=1;
        hero.ability_levels.insert("lumen_lance".into(),1);
        hero.ability_levels.insert("lumen_bolt".into(),0);
        assert_eq!(choose_upgrade(&hero,&steps).unwrap().ability_index,0);
        hero.level=6;
        assert_eq!(choose_upgrade(&hero,&steps).unwrap().ability_index,1);
        hero.abilities.swap(0,1);
        assert_eq!(choose_upgrade(&hero,&steps).unwrap().ability_index,0);
        hero.skill_points=0;assert!(choose_upgrade(&hero,&steps).is_none());
        hero.skill_points=1;
        hero.ability_levels.insert("lumen_lance".into(),4);
        hero.ability_levels.insert("lumen_bolt".into(),1);
        assert!(choose_upgrade(&hero,&steps).is_none());
        hero.ability_levels.insert("lumen_lance".into(),i32::MAX);
        assert!(choose_upgrade(&hero,&steps).is_none());
        hero.ability_levels.insert("lumen_bolt".into(),0);
        hero.abilities.clear();assert!(choose_upgrade(&hero,&steps).is_none());
    }
    fn damage() -> BotAbilityPolicy {BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"lumen_bolt".into(),intent:BotAbilityIntent::EnemyUnit}}
    fn heal() -> BotAbilityPolicy {BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"lumen_touch".into(),intent:BotAbilityIntent::SelfHeal {below_hp_per_mille:600}}}
    #[test]
    fn dash_bot_skips_blocked_candidates_and_falls_through_without_blocking_area() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["vanguard_resolve".into(),"lumen_touch".into(),"ranger_volley".into()];
        for ability in hero.abilities.clone() {hero.ability_levels.insert(ability,1);}
        let mut health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        health.hp=Fixed64::from_i32(10);
        let near=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(350),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:2,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let far=SeenUnit {canonical_id:8,position:Vec2::new(Fixed64::ZERO,Fixed64::from_i32(400)),..near};
        let dash=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"vanguard_resolve".into(),
            intent:BotAbilityIntent::ApproachEnemyPoint {min_distance:300,reach_key:None}};
        let policies=[dash.clone(),heal()];
        let choose=|seen:&[SeenUnit],policies:&[BotAbilityPolicy],clear:fn(Vec2)->bool|
            choose_cast_with_attack_priority(&hero,BotRole::Top,1,Vec2::ZERO,&health,seen,policies,
                None,false,false,|_,_|None,|_,_|None,clear);
        let cast=choose(&[near,far],&policies,|p|p.x==Fixed64::ZERO).unwrap();
        assert_eq!(cast.ability_index,0);assert_eq!(cast.target_pos.unwrap().y,far.position.y.raw() as i32);
        assert_eq!(choose(&[near],&policies, |_|false).unwrap().ability_index,1,
            "blocked relocation must release author priority to recovery");
        assert!(choose(&[near],&[dash], |_|false).is_none());
        let area=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"ranger_volley".into(),
            intent:BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:1}};
        assert_eq!(choose(&[near],&[area], |_|panic!("area is not relocation")).unwrap().ability_index,2);
        assert!(hero.ability_cooldowns.is_empty(),"planning must not reserve cooldown");
    }

    #[test]
    fn fifth_slot_is_cast_like_any_other_e297() {
        let mut hero=Hero::new("asai_nagamasa".into(),"asai_nagamasa".into(),"asai_nagamasa".into());
        hero.abilities=vec!["b08_fang_slash".into(),"b08_flash_cut".into(),"b08_living_soul".into(),"b08_sky_fang".into(),"b08_endless_fang".into()];
        hero.ability_levels.insert("b08_endless_fang".into(),1);
        let policy=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"b08_endless_fang".into(),intent:BotAbilityIntent::EnemyUnit};
        assert!(validate_policies(&[policy.clone()]).is_ok());
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let enemy=SeenUnit {canonical_id:(1<<32)|77,position:Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:6,hp_raw:1,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let cast=choose_cast(&hero,BotRole::Top,1,Vec2::ZERO,&health,&[enemy],&[policy]).expect("the fifth slot casts");
        assert_eq!(cast.ability_index,4);
    }

    #[test]
    fn leap_area_reaches_its_compiled_reach_and_channel_ultimate_casts_e281() {
        // B29W 震地斧 has no cast range: a placed-area intent never finds a target (never cast in
        // M6 batches); the leap intent reaches the compiled targeting reach (dash_max_distance).
        let mut hero=Hero::new("murakami_yoshikiyo".into(),"murakami_yoshikiyo".into(),"murakami_yoshikiyo".into());
        hero.abilities=vec!["b29_ground_slam".into(),"b29_quake_axe".into(),"b29_morale_boost".into(),"b29_earth_tremor".into()];
        hero.ability_levels.insert("b29_quake_axe".into(),1);
        hero.ability_levels.insert("b29_earth_tremor".into(),1);
        let leap=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"b29_quake_axe".into(),intent:BotAbilityIntent::LeapEnemyPoint {
            radius_key:"radius".into(),reach_key:"dash_max_distance".into(),min_targets:1}};
        let placed=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"b29_quake_axe".into(),intent:BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:1}};
        assert!(validate_policies(&[leap.clone()]).is_ok());
        for bad in [BotAbilityIntent::LeapEnemyPoint {radius_key:"radius".into(),reach_key:"missing".into(),min_targets:1},
            BotAbilityIntent::LeapEnemyPoint {radius_key:"radius".into(),reach_key:"dash_max_distance".into(),min_targets:0}] {
            assert!(validate_policies(&[BotAbilityPolicy {intent:bad,..leap.clone()}]).is_err());
        }
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let at=|x:i32|SeenUnit {canonical_id:(1<<32)|77,position:Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:6,hp_raw:1,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let choose=|seen:&[SeenUnit],policy:&BotAbilityPolicy|choose_cast(&hero,BotRole::Top,1,Vec2::ZERO,&health,seen,&[policy.clone()]);
        let cast=choose(&[at(280)],&leap).expect("an enemy within the leap reach");
        assert_eq!(cast.ability_index,1);
        assert_eq!(cast.target_pos.unwrap().x,Fixed64::from_i32(280).raw() as i32);
        assert!(choose(&[at(400)],&leap).is_none(),"beyond the rank-1 reach (300)");
        assert!(choose(&[at(280)],&placed).is_none(),"a placed area with range 0 never casts");
        // B29R 大地震撼: a channeled point ultimate without a recovery rule casts like an instant.
        let tremor=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"b29_earth_tremor".into(),intent:BotAbilityIntent::EnemyPoint {radius_key:"area_radius".into(),min_targets:1}};
        assert!(validate_policies(&[tremor.clone()]).is_ok());
        assert_eq!(choose(&[at(900)],&tremor).expect("within the cast range").ability_index,3);
    }

    /// E438: B13 縮地 has no cast range; its approach reaches the teleport's blink_range (800).
    #[test]
    fn an_approach_without_cast_range_reaches_its_compiled_reach_e438() {
        let mut hero=Hero::new("momochi_sandayu".into(),"momochi_sandayu".into(),"momochi_sandayu".into());
        hero.abilities=vec!["b13_shukuchi".into()];
        hero.ability_levels.insert("b13_shukuchi".into(),1);
        let approach=|reach_key:Option<&str>|BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"b13_shukuchi".into(),
            intent:BotAbilityIntent::ApproachEnemyPoint {min_distance:300,reach_key:reach_key.map(str::to_owned)}};
        assert!(validate_policies(&[approach(None)]).is_err(),"range 0 never exceeds min_distance");
        assert!(validate_policies(&[approach(Some("missing"))]).is_err());
        let reach=approach(Some("blink_range"));
        assert!(validate_policies(&[reach.clone()]).is_ok());
        let parsed:BotAbilityIntent=serde_json::from_value(serde_json::json!({"kind":"approach_enemy_point","min_distance":300,"reach_key":"blink_range"})).unwrap();
        assert!(matches!(parsed,BotAbilityIntent::ApproachEnemyPoint {min_distance:300,reach_key:Some(ref key)} if key=="blink_range"));
        let plain:BotAbilityIntent=serde_json::from_value(serde_json::json!({"kind":"approach_enemy_point","min_distance":300})).unwrap();
        assert!(matches!(plain,BotAbilityIntent::ApproachEnemyPoint {reach_key:None,..}));
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let at=|x:i32|SeenUnit {canonical_id:(1<<32)|77,position:Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:6,hp_raw:1,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let choose=|x:i32|choose_cast(&hero,BotRole::Top,1,Vec2::ZERO,&health,&[at(x)],&[reach.clone()]);
        assert_eq!(choose(700).expect("within the blink range").target_pos.unwrap().x,Fixed64::from_i32(700).raw() as i32);
        assert!(choose(900).is_none(),"beyond the blink range");
        assert!(choose(200).is_none(),"within min_distance");
    }

    /// E398: a lane-unit area waits for a wave and for no enemy hero nearby; a hero in it is enough.
    #[test]
    fn areas_are_kept_for_heroes_unless_a_wave_stands_alone_e398() {
        let unit=|id:u64,x:i32,kind:u8|SeenUnit {canonical_id:(1<<32)|id,position:Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO),
            team:2,kind,owner_player_id:id as u32,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let mut hero=Hero::new("murakami_yoshikiyo".into(),"murakami_yoshikiyo".into(),"murakami_yoshikiyo".into());
        hero.abilities=vec!["b29_ground_slam".into(),"b29_quake_axe".into(),"b29_morale_boost".into(),"b29_earth_tremor".into()];
        hero.ability_levels.insert("b29_earth_tremor".into(),1);
        let tremor=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"b29_earth_tremor".into(),intent:BotAbilityIntent::EnemyPoint {radius_key:"area_radius".into(),min_targets:1}};
        let choose=|role:BotRole,seen:&[SeenUnit]|choose_cast(&hero,role,1,Vec2::ZERO,&health,seen,&[tremor.clone()]);
        assert!(choose(BotRole::Top,&[unit(1,500,2)]).is_none(),"one creep: keep it");
        assert!(choose(BotRole::Top,&[unit(1,500,2),unit(2,520,2),unit(3,540,2)]).is_some(),"a wave alone");
        // An enemy hero within the reserve radius: never spent on the wave (only on the hero, if in reach).
        let coming=choose(BotRole::Top,&[unit(1,500,2),unit(2,520,2),unit(3,540,2),unit(9,-1100,1)]);
        assert!(coming.is_none_or(|cast|cast.target_pos.unwrap().x==Fixed64::from_i32(-1100).raw() as i32),
            "a wave, but an enemy hero is coming");
        let cast=choose(BotRole::Top,&[unit(9,-600,1)]).expect("the hero itself");
        assert_eq!(cast.target_pos.unwrap().x,Fixed64::from_i32(-600).raw() as i32);
        assert!(choose(BotRole::Jungle,&[SeenUnit {team:0,kind:3,..unit(4,500,3)}]).is_some(),"a jungler clears its camp");
    }

    /// E387: the any-distance ultimates C05T 瞬雷閃 (channeled point) and C24R 最期一生 (ally hero).
    #[test]
    fn any_distance_ultimates_aim_at_disclosed_heroes_e387() {
        let unit=|id:u64,x:i32,team:u32,kind:u8,hp:i64|SeenUnit {canonical_id:(1<<32)|id,position:Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO),
            team,kind,owner_player_id:id as u32,hp_raw:hp,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let mut keiji=Hero::new("maeda_keiji".into(),"maeda_keiji".into(),"maeda_keiji".into());
        keiji.abilities=vec!["c05_flash_thunder".into()];
        keiji.ability_levels.insert("c05_flash_thunder".into(),1);
        let thunder=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"c05_flash_thunder".into(),intent:BotAbilityIntent::GlobalEnemyPoint {radius_key:"radius".into(),min_targets:1}};
        assert!(validate_policies(&[thunder.clone()]).is_ok());
        assert!(validate_policies(&[BotAbilityPolicy {intent:BotAbilityIntent::GlobalEnemyPoint {radius_key:"missing".into(),min_targets:1},..thunder.clone()}]).is_err());
        // A cast-range skill is not an any-distance one.
        assert!(validate_policies(&[BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"b29_earth_tremor".into(),
            intent:BotAbilityIntent::GlobalEnemyPoint {radius_key:"area_radius".into(),min_targets:1}}]).is_err());
        let cast=choose_cast(&keiji,BotRole::Top,1,Vec2::ZERO,&health,
            &[unit(5,3000,2,2,100),unit(6,5000,2,1,100),unit(7,5400,2,1,100),unit(8,4000,2,1,100)],&[thunder.clone()]).expect("a far enemy hero");
        assert_eq!(cast.target_pos.unwrap().x,Fixed64::from_i32(5000).raw() as i32,"the hero whose area covers two");
        assert!(choose_cast(&keiji,BotRole::Top,1,Vec2::ZERO,&health,&[unit(5,300,2,2,100)],&[thunder]).is_none(),"creeps alone: no");
        let mut muneyoshi=Hero::new("yagyu_muneyoshi".into(),"yagyu_muneyoshi".into(),"yagyu_muneyoshi".into());
        muneyoshi.abilities=vec!["c24_last_life".into()];
        muneyoshi.ability_levels.insert("c24_last_life".into(),1);
        let life=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"c24_last_life".into(),intent:BotAbilityIntent::AllyEngage {threat_radius:900}};
        assert!(validate_policies(&[life.clone()]).is_ok());
        let choose=|seen:&[SeenUnit]|choose_cast(&muneyoshi,BotRole::Top,1,Vec2::ZERO,&health,seen,&[life.clone()]);
        assert!(choose(&[unit(1,6000,1,1,30)]).is_none(),"an ally out of any fight");
        assert!(choose(&[unit(0,0,1,1,30),unit(9,500,2,1,100)]).is_none(),"the caster itself");
        let cast=choose(&[unit(1,6000,1,1,80),unit(2,-7000,1,1,40),unit(9,6500,2,1,100),unit(10,-7200,2,1,100)]).expect("an ally in a fight");
        assert_eq!(cast.target_entity,Some(((1u64<<32)|2) as u32),"the most hurt ally, at any distance");
    }

    /// E410: A12 大谷吉繼's 神秘箭 is a range-0 line: it reaches its `distance` extra (1850 at rank 1).
    #[test]
    fn a_range_zero_line_reaches_its_distance_extra_e409() {
        let unit=|id:u64,x:i32|SeenUnit {canonical_id:(1<<32)|id,position:Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:id as u32,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let mut yoshitsugu=Hero::new("otani_yoshitsugu".into(),"otani_yoshitsugu".into(),"otani_yoshitsugu".into());
        yoshitsugu.abilities=vec!["a12_mystic_arrow".into()];
        yoshitsugu.ability_levels.insert("a12_mystic_arrow".into(),1);
        let arrow=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"a12_mystic_arrow".into(),intent:BotAbilityIntent::EnemyLine {min_targets:1}};
        assert_eq!(validate_policies(&[arrow.clone()]),Ok(()));
        // The arrow is cast in the 刑部 form only (content requires_status a12t_form, E472).
        let form=|status:&str|status=="a12t_form";
        let choose=|seen:&[SeenUnit],own_status:&dyn Fn(&str)->bool|choose_cast_with_control_observation(&yoshitsugu,BotRole::Top,1,Vec2::ZERO,
            &health,seen,std::slice::from_ref(&arrow),None,false,false,|_,_|None,|_,_|None,|_|true,|_|None,own_status,&|_|0,&|_,_|None,&|_,_|false,&|_,_|0);
        let cast=choose(&[unit(5,1500)],&form).expect("a hero in reach");
        assert_eq!(cast.target_pos.unwrap().x,Fixed64::from_i32(1500).raw() as i32,"aimed at the hero");
        assert!(choose(&[unit(5,4000)],&form).is_none(),"beyond the distance");
        assert!(choose(&[unit(5,1500)],&|_|false).is_none(),"not in the form");
    }

    #[test]
    fn dash_effect_bot_approaches_only_disclosed_living_enemies_in_band() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["lumen_bolt".into(),"vanguard_resolve".into()];
        hero.ability_levels.insert("vanguard_resolve".into(),1);
        let policy=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"vanguard_resolve".into(),
            intent:BotAbilityIntent::ApproachEnemyPoint {min_distance:300,reach_key:None}};
        assert!(validate_policies(&[policy.clone()]).is_ok());
        for min_distance in [0,450,10_001] {
            let bad=BotAbilityPolicy {intent:BotAbilityIntent::ApproachEnemyPoint {min_distance,reach_key:None},..policy.clone()};
            assert!(validate_policies(&[bad]).is_err());
        }
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let enemy=SeenUnit {canonical_id:(1<<32)|77,position:Vec2::new(Fixed64::from_i32(350),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:2,hp_raw:1,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let choose=|hero:&Hero,seen:&[SeenUnit]|choose_cast(hero,BotRole::Top,1,Vec2::ZERO,&health,seen,&[policy.clone()]);
        let cast=choose(&hero,&[enemy]).unwrap();
        assert_eq!(cast.ability_index,1);assert_eq!(cast.target_entity,None);
        assert_eq!(cast.target_pos.unwrap().x,Fixed64::from_i32(350).raw() as i32);
        assert!(choose(&hero,&[]).is_none(),"no disclosure means no hidden target");
        for invalid in [SeenUnit {team:1,..enemy},SeenUnit {hp_raw:0,..enemy},
            SeenUnit {position:Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO),..enemy},
            SeenUnit {position:Vec2::new(Fixed64::from_raw(450*1024+1),Fixed64::ZERO),..enemy}] {
            assert!(choose(&hero,&[invalid]).is_none());
        }
        let other=SeenUnit {canonical_id:(1<<32)|78,..enemy};
        assert_eq!(choose(&hero,&[other,enemy]),choose(&hero,&[enemy,other]));
        hero.ability_cooldowns.insert("vanguard_resolve".into(),Fixed64::ONE);
        assert!(choose(&hero,&[enemy]).is_none());
        hero.ability_cooldowns.clear();hero.ability_levels.insert("vanguard_resolve".into(),0);
        assert!(choose(&hero,&[enemy]).is_none());
    }
    #[test]
    fn role_bot_abilities_validate_explicit_intents_not_previews() {
        assert!(validate_policies(&[heal(),damage()]).is_ok());
        assert!(validate_policies(&[damage(),damage()]).is_err());
        let mut bad=damage();bad.ability="missing".into();assert!(validate_policies(&[bad]).is_err());
        let mut bad=damage();bad.intent=BotAbilityIntent::SelfHeal {below_hp_per_mille:600};assert!(validate_policies(&[bad]).is_err());
        let mut bad=heal();bad.intent=BotAbilityIntent::SelfHeal {below_hp_per_mille:1001};assert!(validate_policies(&[bad]).is_err());
    }

    #[test]
    fn area_effects_bot_selects_disclosed_point_cluster_without_entity_target() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["ranger_volley".into()];hero.ability_levels.insert("ranger_volley".into(),1);
        let mut policies=vec![BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"ranger_volley".into(),intent:BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:2}}];
        assert!(validate_policies(&policies).is_ok());
        let units=[(3,820,2),(2,600,2),(1,600,1),(4,950,2)].map(|(id,x,team)|SeenUnit {
            canonical_id:id,position:Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO),team,kind:1,owner_player_id:id as u32,hp_raw:1,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO});
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let cast=choose_cast(&hero,BotRole::Carry,1,Vec2::ZERO,&health,&units,&policies).unwrap();
        assert_eq!(cast.target_entity,None);assert_eq!(cast.target_pos.unwrap().x,Fixed64::from_i32(600).raw() as i32);
        assert_eq!(cast,choose_cast(&hero,BotRole::Carry,1,Vec2::ZERO,&health,&units.into_iter().rev().collect::<Vec<_>>(),&policies).unwrap());
        assert!(choose_cast(&hero,BotRole::Carry,1,Vec2::ZERO,&health,&units[..1],&policies).is_none());
        policies[0].intent=BotAbilityIntent::EnemyPoint {radius_key:"missing".into(),min_targets:1};
        assert!(validate_policies(&policies).is_err());
        policies[0].intent=BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:129};
        assert!(validate_policies(&policies).is_err());
    }
    #[test]
    fn role_bot_abilities_use_current_slots_rank_cooldown_and_fixed_range() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["lumen_touch".into(),"lumen_bolt".into()];
        hero.ability_levels.insert("lumen_touch".into(),1);hero.ability_levels.insert("lumen_bolt".into(),1);
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let unit=SeenUnit {canonical_id:(1<<32)|77,position:Vec2::new(Fixed64::from_i32(590),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:2,hp_raw:1,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let cast=choose_cast(&hero,BotRole::Mid,1,Vec2::ZERO,&health,&[unit],&[damage()]).unwrap();
        assert_eq!(cast.ability_index,1);assert_eq!(cast.target_entity,Some(77));
        hero.ability_levels.insert("lumen_bolt".into(),0);
        assert!(choose_cast(&hero,BotRole::Mid,1,Vec2::ZERO,&health,&[unit],&[damage()]).is_none());
        hero.ability_levels.insert("lumen_bolt".into(),1);
        hero.ability_cooldowns.insert("lumen_bolt".into(),Fixed64::ONE);
        assert!(choose_cast(&hero,BotRole::Mid,1,Vec2::ZERO,&health,&[unit],&[damage()]).is_none());
        hero.ability_cooldowns.clear();
        let far=SeenUnit {position:Vec2::new(Fixed64::from_i32(601),Fixed64::ZERO),..unit};
        assert!(choose_cast(&hero,BotRole::Mid,1,Vec2::ZERO,&health,&[far],&[damage()]).is_none());
        let mut low=health.clone();low.hp=Fixed64::from_i32(20);
        assert_eq!(choose_cast(&hero,BotRole::Mid,1,Vec2::ZERO,&low,&[],&[heal()]).unwrap().ability_index,0);
        assert!(choose_cast(&hero,BotRole::Mid,1,Vec2::ZERO,&health,&[],&[heal()]).is_none());
    }
    #[test]
    fn self_area_and_self_engage_cast_only_with_disclosed_enemies_in_reach() {
        let area=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"a07_red_thunder".into(),intent:BotAbilityIntent::SelfArea {radius_key:"shape_radius".into(),min_targets:2}};
        let engage=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"lumen_touch".into(),intent:BotAbilityIntent::SelfEngage {threat_radius:700}};
        assert!(validate_policies(&[area.clone(),engage.clone()]).is_ok());
        assert!(validate_policies(&[BotAbilityPolicy {intent:BotAbilityIntent::SelfArea {radius_key:"missing".into(),min_targets:1},..area.clone()}]).is_err());
        assert!(validate_policies(&[BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"lumen_bolt".into(),..engage.clone()}]).is_err(),"targeted abilities are not self engage");
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["a07_red_thunder".into(),"lumen_touch".into()];
        for ability in hero.abilities.clone() {hero.ability_levels.insert(ability,1);}
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let unit=|id:u64,x:i32,kind:u8|SeenUnit {canonical_id:id,position:Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO),
            team:2,kind,owner_player_id:2,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let choose=|seen:&[SeenUnit],policies:&[BotAbilityPolicy]|choose_cast(&hero,BotRole::Top,1,Vec2::ZERO,&health,seen,policies);
        assert!(choose(&[unit(7,300,2)],&[area.clone()]).is_none(),"one target is below min_targets");
        // Creeps alone need a wave (E398): three inside 525, not two.
        assert!(choose(&[unit(7,300,2),unit(8,525,2)],&[area.clone()]).is_none(),"two creeps are not a wave");
        assert_eq!(choose(&[unit(6,200,2),unit(7,300,2),unit(8,525,2)],&[area.clone()]).unwrap().ability_index,0,"three inside 525");
        assert!(choose(&[unit(6,200,2),unit(7,300,2),unit(8,526,2)],&[area.clone()]).is_none());
        assert_eq!(choose(&[unit(7,300,1),unit(8,525,2)],&[area.clone()]).unwrap().ability_index,0,"a hero and a creep");
        assert!(choose(&[unit(7,300,2)],&[engage.clone()]).is_none(),"creeps do not start a fight");
        assert_eq!(choose(&[unit(7,700,1)],&[engage.clone()]).unwrap().ability_index,1);
        assert!(choose(&[unit(7,701,1)],&[engage]).is_none());
    }
    /// E605: the area counts only enemies showing the status, or the corpses near the bot; the
    /// anchor point lands on the allied unit nearest the closest enemy hero, never a hero.
    #[test]
    fn preconditioned_areas_and_ally_anchors_follow_what_the_cast_needs() {
        let status=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"a07_red_thunder".into(),intent:BotAbilityIntent::SelfAreaStatus {
            radius_key:"shape_radius".into(),min_targets:1,status:"c25e_dot".into()}};
        let corpses=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"a07_red_thunder".into(),intent:BotAbilityIntent::SelfAreaCorpses {
            radius_key:"shape_radius".into(),min_targets:1}};
        assert!(validate_policies(std::slice::from_ref(&status)).is_ok() && validate_policies(std::slice::from_ref(&corpses)).is_ok());
        assert!(validate_policies(&[BotAbilityPolicy {intent:BotAbilityIntent::SelfAreaStatus {radius_key:"shape_radius".into(),
            min_targets:1,status:String::new()},..status.clone()}]).is_err(),"a status is named");
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["a07_red_thunder".into()];hero.ability_levels.insert("a07_red_thunder".into(),1);
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let unit=|id:u64,x:i32,team:u32,kind:u8|SeenUnit {canonical_id:id,position:Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO),
            team,kind,owner_player_id:2,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let choose=|seen:&[SeenUnit],policy:&BotAbilityPolicy,marked:u64,dead:usize|choose_cast_with_control_observation(&hero,BotRole::Top,1,
            Vec2::ZERO,&health,seen,std::slice::from_ref(policy),None,false,false,|_,_|None,|_,_|None,|_|true,|_|None,&|_|false,&|_|0,
            &|_,_|None,&|id,name|id==marked && name=="c25e_dot",&|_,_|dead);
        assert!(choose(&[unit(7,300,2,1)],&status,0,0).is_none(),"no enemy bleeds");
        assert!(choose(&[unit(7,300,2,1)],&status,7,0).is_some(),"the hero bleeds");
        assert!(choose(&[],&corpses,0,0).is_none(),"no corpse");
        assert!(choose(&[],&corpses,0,1).is_some(),"a corpse in reach");
    }

    /// E260: a point summon is cast near the nearest disclosed enemy within the threat radius,
    /// at the enemy when it is in cast range, else toward it at the cast range.
    #[test]
    fn summon_near_enemy_casts_at_or_toward_the_nearest_threat() {
        assert!(matches!(serde_json::from_str::<BotAbilityIntent>(r#"{"kind":"summon_near_enemy","threat_radius":900}"#).unwrap(),
            BotAbilityIntent::SummonNearEnemy {threat_radius:900}));
        let summon=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"b01_saika_corps".into(),intent:BotAbilityIntent::SummonNearEnemy {threat_radius:900}};
        assert!(validate_policies(&[summon.clone()]).is_ok());
        assert!(validate_policies(&[BotAbilityPolicy {intent:BotAbilityIntent::SummonNearEnemy {threat_radius:0},..summon.clone()}]).is_err());
        assert!(validate_policies(&[BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"b01_sniper_mode".into(),..summon.clone()}]).is_ok(),"an untargeted skill is cast with no point (E419)");
        {
            // E419: an untargeted summon is cast when a threat is near, with no point.
            let mut kojiro=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
            kojiro.abilities=vec!["c02_phantom_move".into()];
            kojiro.ability_levels.insert("c02_phantom_move".into(),1);
            let phantom=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"c02_phantom_move".into(),intent:BotAbilityIntent::SummonNearEnemy {threat_radius:900}};
            assert!(validate_policies(&[phantom.clone()]).is_ok());
            let near=SeenUnit {canonical_id:(1<<32)|9,position:Vec2::new(Fixed64::from_i32(500),Fixed64::ZERO),team:2,kind:1,
                owner_player_id:9,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
            let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
            let cast=choose_cast(&kojiro,BotRole::Top,1,Vec2::ZERO,&health,&[near.clone()],&[phantom.clone()]).expect("a threat near");
            assert!(cast.target_pos.is_none() && cast.target_entity.is_none());
            let far=SeenUnit {position:Vec2::new(Fixed64::from_i32(2000),Fixed64::ZERO),..near};
            assert!(choose_cast(&kojiro,BotRole::Top,1,Vec2::ZERO,&health,&[far],&[phantom]).is_none());
        }
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["b01_saika_corps".into()];
        hero.ability_levels.insert("b01_saika_corps".into(),1);
        let range=omoba_template_ids::ability_by_name("b01_saika_corps").and_then(omoba_template_ids::active_ability_const).unwrap().levels[0].range;
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let unit=|id:u64,x:i32,kind:u8|SeenUnit {canonical_id:id,position:Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO),
            team:2,kind,owner_player_id:2,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let choose=|seen:&[SeenUnit]|choose_cast(&hero,BotRole::Carry,1,Vec2::ZERO,&health,seen,&[summon.clone()]);
        assert!(choose(&[]).is_none());
        assert!(choose(&[unit(7,901,1)]).is_none(),"no threat within 900");
        let near=choose(&[unit(7,300,2),unit(8,600,1)]).unwrap();
        assert_eq!(near.target_pos.unwrap().x,Fixed64::from_i32(300).raw() as i32,"the nearest enemy, inside cast range");
        let far=choose(&[unit(8,850,1)]).unwrap();
        let x=i64::from(far.target_pos.unwrap().x);
        assert!(x<=range.raw() && x>range.raw()-1024,"toward a farther one, within the cast range: {x}");
    }
    /// E263: a toggle switches on while a disclosed enemy hero is within the threat radius and
    /// off when none is; it reads only the owner's own on-marker and casts only to change it.
    #[test]
    fn toggle_engage_switches_by_threat_and_the_owners_own_marker() {
        let toggle=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"a02_venom_arrow".into(),
            intent:BotAbilityIntent::ToggleEngage {threat_radius:900,on_status:"attack_orb:a02_venom_arrow".into()}};
        assert!(validate_policies(&[toggle.clone()]).is_ok());
        assert!(validate_policies(&[BotAbilityPolicy {intent:BotAbilityIntent::ToggleEngage {threat_radius:900,on_status:String::new()},..toggle.clone()}]).is_err());
        assert!(validate_policies(&[BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"b01_sniper_mode".into(),..toggle.clone()}]).is_err(),"an active skill is no toggle");
        assert!(validate_policies(&[BotAbilityPolicy {intent:BotAbilityIntent::SelfEngage {threat_radius:900},..toggle.clone()}]).is_err(),
            "a toggle takes only the toggle intent");
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["a02_venom_arrow".into()];
        hero.ability_levels.insert("a02_venom_arrow".into(),1);
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let unit=|id:u64,x:i32,kind:u8|SeenUnit {canonical_id:id,position:Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO),
            team:2,kind,owner_player_id:2,hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let decide=|seen:&[SeenUnit],on:bool|choose_cast_with_control_observation(&hero,BotRole::Carry,1,Vec2::ZERO,&health,seen,
            &[toggle.clone()],None,false,false,|_,_|None,|_,_|None,|_|true,|_|None,
            &|status:&str|on && status=="attack_orb:a02_venom_arrow",&|_|0,&|_,_|None,&|_,_|false,&|_,_|0).map(|cast|cast.ability_index);
        assert_eq!(decide(&[unit(7,800,1)],false),Some(0),"off with an enemy hero near: switch on");
        assert_eq!(decide(&[unit(7,800,1)],true),None,"already on");
        assert_eq!(decide(&[],true),Some(0),"on with no threat: switch off");
        assert_eq!(decide(&[unit(8,300,2)],false),None,"a creep is no fight");
        assert_eq!(decide(&[unit(7,901,1)],false),None,"beyond the threat radius");
    }
}

#[cfg(test)]
mod jungle_self_engage_tests {
    use super::*;

    /// E585: a jungler's self_engage also takes the camp it fights (its focus) as the threat; a
    /// laner's still needs an enemy hero.
    #[test]
    fn a_jungler_engages_the_camp_it_fights() {
        let mut hero=Hero::default();
        hero.abilities=vec!["a06_red_power".into()];hero.ability_levels.insert("a06_red_power".into(),1);
        let hp=CProperty {hp:Fixed64::from_i32(100),mhp:Fixed64::from_i32(100),msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO};
        let camp=SeenUnit {canonical_id:9,position:Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO),team:0,kind:3,owner_player_id:0,
            hp_raw:100,max_hp_raw:100,threat:Fixed64::ZERO,radius:Fixed64::ZERO};
        let policy=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"a06_red_power".into(),intent:BotAbilityIntent::SelfEngage {threat_radius:900}};
        let decide=|role:BotRole,focus:Option<u64>|choose_cast_with_attack_priority(&hero,role,1,Vec2::ZERO,&hp,&[camp],&[policy.clone()],
            focus,false,false,|_,_|None,|_,_|None,|_|false);
        assert!(decide(BotRole::Jungle,Some(9)).is_some(),"the camp it fights");
        assert!(decide(BotRole::Jungle,None).is_none(),"a camp it does not fight");
        assert!(decide(BotRole::Mid,Some(9)).is_none(),"a laner needs an enemy hero");
    }
}
