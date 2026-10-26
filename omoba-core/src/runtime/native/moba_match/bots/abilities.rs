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
    EnemyPoint {radius_key:String,min_targets:u16},
    /// Actual compiled source-sector geometry; no authored radius/angle-key guess.
    EnemyCone {min_targets:u16},
    /// Actual compiled nonmoving capsule geometry, over disclosed observations.
    EnemyLine {min_targets:u16},
    /// Author explicitly opts a point skill into closing on disclosed enemies.
    ApproachEnemyPoint {min_distance:u32},
    SelfHeal { below_hp_per_mille:u16 },
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
}

/// Each entry spends one point, in author order when its prerequisites permit.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotAbilityLearningStep {
    pub ability:String,
    pub rank:u8,
}

pub(super) fn validate_learning(steps:&[BotAbilityLearningStep]) -> Result<(), &'static str> {
    if steps.len()>64 {return Err("bot learning plan exceeds 64 steps");}
    let mut ranks=std::collections::BTreeMap::new();
    for step in steps {
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

pub(super) fn validate_policies(policies:&[BotAbilityPolicy]) -> Result<(), &'static str> {
    if policies.len()>64 {return Err("bot ability policies exceed 64");}
    let mut ids=std::collections::BTreeSet::new();
    for policy in policies {
        if !ids.insert(&policy.ability) {return Err("duplicate bot ability policy");}
        let def=omoba_template_ids::ability_by_name(&policy.ability)
            .and_then(omoba_template_ids::active_ability_const).ok_or("unknown active compiled bot ability")?;
        let supported_cast=if matches!(policy.intent,BotAbilityIntent::SelfChannelRecovery {..}) {
            def.cast_type==CastTypeC::Channeled && def.recovery_channel.is_some()
                && def.levels.iter().all(|rank|rank.cast_time==Fixed64::ZERO)
        } else {def.cast_type==CastTypeC::Instant};
        if !matches!(def.ability_type,AbilityTypeC::Active | AbilityTypeC::Ultimate) || !supported_cast {
            return Err("bot policy requires a supported explicit active execution type");
        }
        match &policy.intent {
            BotAbilityIntent::SelfChannelRecovery {below_hp_per_mille,threat_radius}
                if def.target_type==TargetTypeC::None && (1..=1000).contains(below_hp_per_mille)
                    && (1..=10_000).contains(threat_radius)=>{},
            BotAbilityIntent::EnemyUnit if def.target_type==TargetTypeC::Unit => {},
            BotAbilityIntent::AllyHeal {below_hp_per_mille} if def.target_type==TargetTypeC::Unit
                && (1..=1000).contains(below_hp_per_mille)=>{},
            BotAbilityIntent::AllyHealOrCleanse {below_hp_per_mille} if def.target_type==TargetTypeC::Unit
                && def.cleanses_allies && (1..=1000).contains(below_hp_per_mille)
                && def.levels.iter().all(|rank|rank.range>Fixed64::ZERO&&rank.range<=Fixed64::from_i32(10_000))=>{},
            BotAbilityIntent::ApproachEnemyPoint {min_distance} if def.target_type==TargetTypeC::Point
                && (1..=10_000).contains(min_distance)
                && def.levels.iter().all(|level|level.range>Fixed64::from_i32(*min_distance as i32))=>{},
            BotAbilityIntent::SelfHeal {below_hp_per_mille} if def.target_type==TargetTypeC::None
                && *below_hp_per_mille>0 && *below_hp_per_mille<=1000 => {},
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
            BotAbilityIntent::EnemyLine {min_targets} if def.target_type==TargetTypeC::Point
                && (1..=128).contains(min_targets) && def.line_widths.len()==def.levels.len()
                && def.line_widths.iter().all(|width|*width>Fixed64::ZERO&&*width<=Fixed64::from_i32(5_000))=>{},
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
    choose_cast_with_control_observation(hero,role,team,own,health,seen,policies,focus,defer_offense,immobilized,cost,regen,relocation_clear,|_|None)
}

pub(super) fn choose_cast_with_control_observation(hero:&Hero,role:BotRole,team:u32,own:Vec2,health:&CProperty,
    seen:&[SeenUnit],policies:&[BotAbilityPolicy],focus:Option<u64>,defer_offense:bool,immobilized:bool,
    cost:impl Fn(&str,u8)->Option<Fixed64>,regen:impl Fn(&str,Fixed64)->Option<(Fixed64,Fixed64)>,
    relocation_clear:impl Fn(Vec2)->bool,control_priority:impl Fn(u64)->Option<u8>) -> Option<CastAbility> {
    let enemy=|unit:&&SeenUnit|unit.hp_raw>0 && unit.team!=team
        && if role==BotRole::Jungle {unit.kind==3
            || (unit.kind==1 && unit.team!=0 && Some(unit.canonical_id)==focus)}
        else {unit.team!=0 && matches!(unit.kind,1|2)};
    let prioritizes_focus=matches!(role,BotRole::Jungle|BotRole::Support);
    let priority=|unit:&SeenUnit|(prioritizes_focus && Some(unit.canonical_id)!=focus,
        (unit.position-own).length_squared().raw(),unit.canonical_id);
    // Author declaration order is the explicit priority. Slot is looked up in
    // this owner's current loadout; skill names never imply Q/W/E/R.
    for policy in policies {
        // Root blocks voluntary relocation, not ordinary damage/recovery.
        if immobilized && matches!(policy.intent,BotAbilityIntent::ApproachEnemyPoint {..}) {continue;}
        if defer_offense && matches!(policy.intent,BotAbilityIntent::EnemyUnit
            |BotAbilityIntent::EnemyPoint {..}|BotAbilityIntent::EnemyCone {..}|BotAbilityIntent::EnemyLine {..}|BotAbilityIntent::ApproachEnemyPoint {..}) {continue;}
        let Some(slot)=hero.abilities.iter().position(|id|id==&policy.ability) else {continue;};
        if slot>=4 {continue;} // formal PlayerInput exposes exactly four slots
        if !hero.can_use_ability(&policy.ability) || !hero.cast_ready(&policy.ability) {continue;}
        let owner_rank=hero.get_ability_level(&policy.ability);
        if !(1..=255).contains(&owner_rank) {continue;}
        let rank=(owner_rank-1) as usize;
        let Some(def)=omoba_template_ids::ability_by_name(&policy.ability).and_then(omoba_template_ids::active_ability_const) else {continue;};
        let Some(level)=def.levels.get(rank) else {continue;};
        if matches!(policy.intent,BotAbilityIntent::SelfChannelRecovery {..})
            && (hero.mana_pool.is_none() || def.cast_type!=CastTypeC::Channeled || def.recovery_channel.is_none()) {continue;}
        let required_cost=if let Some(pool)=&hero.mana_pool {
            let Some(required)=cost(&policy.ability,owner_rank as u8) else {continue;};
            if required<Fixed64::ZERO || required>pool.current() {continue;}
            Some(required)
        } else {None};
        let mut target_pos=None;
        let target=match &policy.intent {
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
            BotAbilityIntent::EnemyUnit => {
                if level.range<=Fixed64::ZERO {continue;}
                let Some(unit)=seen.iter().filter(enemy)
                    .filter(|unit|(unit.position-own).length_squared()<=level.range*level.range)
                    .min_by_key(|unit|priority(unit)) else {continue;};
                Some(unit.canonical_id as u32)
            }
            BotAbilityIntent::ApproachEnemyPoint {min_distance}=>{
                let minimum=Fixed64::from_i32(*min_distance as i32);
                let Some(unit)=seen.iter().filter(enemy)
                    .filter(|unit|{let distance=(unit.position-own).length_squared();
                        distance>minimum*minimum && distance<=level.range*level.range})
                    // Relocation requires a clear swept segment, not a route
                    // around the obstacle. Do not inspect hidden dynamic units.
                    .filter(|unit|relocation_clear(unit.position))
                    .min_by_key(|unit|priority(unit)) else {continue;};
                let (Ok(x),Ok(y))=(i32::try_from(unit.position.x.raw()),i32::try_from(unit.position.y.raw())) else {continue;};
                target_pos=Some(crate::runtime::Vec2I {x,y});None
            }
            BotAbilityIntent::EnemyLine {min_targets}=>{
                if health.hp<=Fixed64::ZERO{continue;}
                let Some(width)=def.line_widths.get(rank).copied()else{continue;};
                let Some(point)=choose_line_target(own,level.range,width,*min_targets,seen.iter().filter(enemy).collect(),if prioritizes_focus{focus}else{None})else{continue;};
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
            BotAbilityIntent::EnemyPoint {radius_key,min_targets}=>{
                let Some((_,radii))=def.extras.iter().find(|(key,_)|*key==radius_key) else {continue;};
                let Some(radius)=radii.get(rank).copied() else {continue;};
                let mut eligible:Vec<_>=seen.iter().filter(enemy).collect();
                eligible.sort_by_key(|u|priority(u));
                eligible.truncate(512);
                // Candidate centers are disclosed enemy poses; no hidden targets,
                // inferred motion or private aggro. Most covered targets wins.
                let center=eligible.iter().filter(|u|(u.position-own).length_squared()<=level.range*level.range).take(32)
                    .filter_map(|u|{
                        let count=eligible.iter().filter(|v|(v.position-u.position).length_squared()<=radius*radius).count();
                        (count>=usize::from(*min_targets)).then_some((prioritizes_focus && Some(u.canonical_id)!=focus,
                            std::cmp::Reverse(count),(u.position-own).length_squared().raw(),u.canonical_id,u.position))
                    }).min_by_key(|(assist,count,distance,id,_)|(*assist,*count,*distance,*id));
                let Some((_,_,_,_,center))=center else {continue;};
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
        let valid=BotAbilityPolicy {ability:"apprentice_mend".into(),intent:BotAbilityIntent::SelfChannelRecovery {below_hp_per_mille:650,threat_radius:1000}};
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
        let policy=BotAbilityPolicy {ability:"apprentice_mend".into(),intent:BotAbilityIntent::SelfChannelRecovery {below_hp_per_mille:650,threat_radius:1000}};
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["lumen_bolt".into(),"apprentice_mend".into()];hero.ability_levels.insert("apprentice_mend".into(),1);
        hero.mana_pool=Some(ManaPool::new(Fixed64::from_i32(100),Fixed64::from_i32(200)).unwrap());
        let mut hp=property(Fixed64::from_i32(200),Fixed64::ZERO);hp.hp=Fixed64::from_i32(50);hp.mhp=Fixed64::from_i32(200);
        let decide=|hero:&Hero,hp:&CProperty,seen:&[SeenUnit],cost:Option<Fixed64>|choose_cast_with_attack_priority(hero,BotRole::Mid,1,Vec2::ZERO,hp,seen,&[policy.clone()],None,true,true,|_,_|cost,|_,_|None,|_|false);
        let cost=Some(Fixed64::from_i32(90));let cast=decide(&hero,&hp,&[],cost).unwrap();
        assert_eq!((cast.ability_index,cast.target_entity,cast.target_pos),(1,None,None));
        let enemy=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(1000),Fixed64::ZERO),team:2,kind:1,owner_player_id:2,hp_raw:1,max_hp_raw:100};
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
            team:2,kind:1,owner_player_id:2,hp_raw:100,max_hp_raw:100};
        let close=SeenUnit {canonical_id:8,position:Vec2::new(Fixed64::from_i32(50),Fixed64::ZERO),..enemy};
        let cast=|seen:&[SeenUnit],policies:&[BotAbilityPolicy]|choose_cast_with_resources(&hero,BotRole::Support,
            1,Vec2::ZERO,&hp,seen,policies,Some(7),|_,_|None,|_,_|None);
        for (ability,intent,slot) in [
            ("lumen_bolt",BotAbilityIntent::EnemyUnit,0),
            ("vanguard_resolve",BotAbilityIntent::ApproachEnemyPoint {min_distance:100},1),
            ("ranger_volley",BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:1},2),
        ] {
            let policy=BotAbilityPolicy {ability:ability.into(),intent};
            let result=cast(&[close,enemy],&[policy.clone()]).unwrap();
            assert_eq!(result.ability_index,slot);
            if slot==0 {assert_eq!(result.target_entity,Some(7));}
            else {assert_eq!(result.target_pos.unwrap().x,enemy.position.x.raw() as i32);}
            for invalid in [SeenUnit {team:1,..enemy},SeenUnit {hp_raw:0,..enemy},
                SeenUnit {position:Vec2::new(Fixed64::from_i32(10_001),Fixed64::ZERO),..enemy}] {
                assert!(cast(&[invalid],&[policy.clone()]).is_none());
            }
        }
        let heal=BotAbilityPolicy {ability:"lumen_aid".into(),intent:BotAbilityIntent::AllyHeal {below_hp_per_mille:600}};
        let damage=BotAbilityPolicy {ability:"lumen_bolt".into(),intent:BotAbilityIntent::EnemyUnit};
        let ally=SeenUnit {canonical_id:9,team:1,hp_raw:20,owner_player_id:3,..close};
        let result=cast(&[enemy,ally],&[heal,damage]).unwrap();
        assert_eq!((result.ability_index,result.target_entity),(3,Some(9)),"author heal-first priority must survive guard focus");
    }

    #[test]
    fn bot_cleanse_policy_prioritizes_public_control_then_health_without_target_private_queries() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());hero.abilities=vec!["lumen_aid".into()];hero.ability_levels.insert("lumen_aid".into(),1);
        let policy=BotAbilityPolicy {ability:"lumen_aid".into(),intent:BotAbilityIntent::AllyHealOrCleanse {below_hp_per_mille:600}};
        assert!(validate_policies(&[policy.clone()]).is_ok());
        let hp=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let ally=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(250),Fixed64::ZERO),team:1,kind:1,owner_player_id:3,hp_raw:100,max_hp_raw:100};
        let injured=SeenUnit {canonical_id:8,hp_raw:20,..ally};let second=SeenUnit {canonical_id:9,..ally};
        let decide=|seen:&[SeenUnit],observed:&[(u64,u8)]|choose_cast_with_control_observation(&hero,BotRole::Support,1,Vec2::ZERO,&hp,seen,&[policy.clone()],None,true,true,|_,_|None,|_,_|None,|_|panic!("nonmovement"),|id|observed.iter().find(|(key,_)|*key==id).map(|(_,p)|*p));
        assert!(decide(&[ally],&[]).is_none());
        assert_eq!(decide(&[injured,ally],&[(7,1)]).unwrap().target_entity,Some(7));
        assert_eq!(decide(&[ally,injured],&[(7,1)]).unwrap().target_entity,Some(7));
        assert_eq!(decide(&[ally,second],&[(7,4),(9,2)]).unwrap().target_entity,Some(9));
        assert_eq!(decide(&[second,ally],&[(7,1),(9,1)]).unwrap().target_entity,Some(7));
        assert_eq!(decide(&[injured],&[]).unwrap().target_entity,Some(8));
        for invalid in [SeenUnit {team:2,..ally},SeenUnit {kind:2,..ally},SeenUnit {hp_raw:0,..ally},SeenUnit {position:Vec2::new(Fixed64::from_i32(601),Fixed64::ZERO),..ally}] {
            let result=choose_cast_with_control_observation(&hero,BotRole::Support,1,Vec2::ZERO,&hp,&[invalid],&[policy.clone()],None,false,false,|_,_|None,|_,_|None,|_|true,|_|panic!("invalid candidate must not query public status"));assert!(result.is_none());}
        assert!(decide(&[],&[(7,1)]).is_none());assert!(decide(&[ally],&[(7,0),(9,1)]).is_none());assert!(decide(&[ally],&[(7,255)]).is_none());
        for value in [0,1001]{let mut bad=policy.clone();bad.intent=BotAbilityIntent::AllyHealOrCleanse{below_hp_per_mille:value};assert!(validate_policies(&[bad]).is_err());}
        for ability in ["lumen_bolt","vanguard_strike","lumen_touch"]{let mut bad=policy.clone();bad.ability=ability.into();assert!(validate_policies(&[bad]).is_err());}
        assert!(serde_json::from_str::<BotAbilityIntent>(r#"{"kind":"ally_heal_or_cleanse","below_hp_per_mille":600,"hidden_control":true}"#).is_err());
    }

    #[test]
    fn ally_heal_bot_uses_disclosed_relative_health_and_stable_range() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["lumen_aid".into()];hero.ability_levels.insert("lumen_aid".into(),1);
        let policy=BotAbilityPolicy {ability:"lumen_aid".into(),intent:BotAbilityIntent::AllyHeal {below_hp_per_mille:600}};
        assert!(validate_policies(&[policy.clone()]).is_ok());
        let ally=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(250),Fixed64::ZERO),
            team:1,kind:1,owner_player_id:3,hp_raw:30,max_hp_raw:100};
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
    #[test]
    fn jungle_assist_skills_share_focus_but_never_accept_hidden_or_friendly_targets() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["vanguard_strike".into()];hero.ability_levels.insert("vanguard_strike".into(),1);
        let hp=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let enemy=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(250),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:2,hp_raw:100,max_hp_raw:100};
        let neutral=SeenUnit {canonical_id:8,position:Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO),
            team:0,kind:3,owner_player_id:0,hp_raw:100,max_hp_raw:100};
        let policy=BotAbilityPolicy {ability:"vanguard_strike".into(),intent:BotAbilityIntent::EnemyUnit};
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
            BotAbilityPolicy {ability:"vanguard_resolve".into(),intent:BotAbilityIntent::ApproachEnemyPoint {min_distance:300}},
            BotAbilityPolicy {ability:"ranger_volley".into(),intent:BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:1}},
        ] {
            let cast=choose_cast_with_resources(&point_hero,BotRole::Jungle,1,Vec2::ZERO,&hp,
                &[neutral,enemy],&[policy.clone()],Some(7),|_,_|None,|_,_|None).unwrap();
            assert_eq!(cast.target_pos.unwrap().x,Fixed64::from_i32(350).raw() as i32);
            let cast=choose_cast_with_resources(&point_hero,BotRole::Jungle,1,Vec2::ZERO,&hp,
                &[neutral,enemy],&[policy],None,|_,_|None,|_,_|None).unwrap();
            assert_eq!(cast.target_pos.unwrap().x,Fixed64::from_i32(320).raw() as i32);
        }
    }
    #[test]
    fn mana_regeneration_bot_compares_marginal_gain_and_blocks_active_source() {
        use crate::runtime::ability_runtime::ManaPool;
        let policy=BotAbilityPolicy {ability:"ranger_patch".into(),intent:BotAbilityIntent::SelfManaRegeneration {
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
        let policy=BotAbilityPolicy {ability:"lumen_touch".into(),intent:BotAbilityIntent::SelfRecovery {
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
            team:2,kind:1,owner_player_id:2,hp_raw:1,max_hp_raw:100};
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
            (1..=5).map(|rank|learn("lumen_bolt",rank)).collect(),vec![learn("lumen_bolt",1);65]] {
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
    fn damage() -> BotAbilityPolicy {BotAbilityPolicy {ability:"lumen_bolt".into(),intent:BotAbilityIntent::EnemyUnit}}
    fn heal() -> BotAbilityPolicy {BotAbilityPolicy {ability:"lumen_touch".into(),intent:BotAbilityIntent::SelfHeal {below_hp_per_mille:600}}}
    #[test]
    fn dash_bot_skips_blocked_candidates_and_falls_through_without_blocking_area() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["vanguard_resolve".into(),"lumen_touch".into(),"ranger_volley".into()];
        for ability in hero.abilities.clone() {hero.ability_levels.insert(ability,1);}
        let mut health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        health.hp=Fixed64::from_i32(10);
        let near=SeenUnit {canonical_id:7,position:Vec2::new(Fixed64::from_i32(350),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:2,hp_raw:100,max_hp_raw:100};
        let far=SeenUnit {canonical_id:8,position:Vec2::new(Fixed64::ZERO,Fixed64::from_i32(400)),..near};
        let dash=BotAbilityPolicy {ability:"vanguard_resolve".into(),
            intent:BotAbilityIntent::ApproachEnemyPoint {min_distance:300}};
        let policies=[dash.clone(),heal()];
        let choose=|seen:&[SeenUnit],policies:&[BotAbilityPolicy],clear:fn(Vec2)->bool|
            choose_cast_with_attack_priority(&hero,BotRole::Top,1,Vec2::ZERO,&health,seen,policies,
                None,false,false,|_,_|None,|_,_|None,clear);
        let cast=choose(&[near,far],&policies,|p|p.x==Fixed64::ZERO).unwrap();
        assert_eq!(cast.ability_index,0);assert_eq!(cast.target_pos.unwrap().y,far.position.y.raw() as i32);
        assert_eq!(choose(&[near],&policies, |_|false).unwrap().ability_index,1,
            "blocked relocation must release author priority to recovery");
        assert!(choose(&[near],&[dash], |_|false).is_none());
        let area=BotAbilityPolicy {ability:"ranger_volley".into(),
            intent:BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:1}};
        assert_eq!(choose(&[near],&[area], |_|panic!("area is not relocation")).unwrap().ability_index,2);
        assert!(hero.ability_cooldowns.is_empty(),"planning must not reserve cooldown");
    }

    #[test]
    fn dash_effect_bot_approaches_only_disclosed_living_enemies_in_band() {
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["lumen_bolt".into(),"vanguard_resolve".into()];
        hero.ability_levels.insert("vanguard_resolve".into(),1);
        let policy=BotAbilityPolicy {ability:"vanguard_resolve".into(),
            intent:BotAbilityIntent::ApproachEnemyPoint {min_distance:300}};
        assert!(validate_policies(&[policy.clone()]).is_ok());
        for min_distance in [0,450,10_001] {
            let bad=BotAbilityPolicy {intent:BotAbilityIntent::ApproachEnemyPoint {min_distance},..policy.clone()};
            assert!(validate_policies(&[bad]).is_err());
        }
        let health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let enemy=SeenUnit {canonical_id:(1<<32)|77,position:Vec2::new(Fixed64::from_i32(350),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:2,hp_raw:1,max_hp_raw:100};
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
        let mut policies=vec![BotAbilityPolicy {ability:"ranger_volley".into(),intent:BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:2}}];
        assert!(validate_policies(&policies).is_ok());
        let units=[(3,820,2),(2,600,2),(1,600,1),(4,950,2)].map(|(id,x,team)|SeenUnit {
            canonical_id:id,position:Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO),team,kind:1,owner_player_id:id as u32,hp_raw:1,max_hp_raw:100});
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
            team:2,kind:1,owner_player_id:2,hp_raw:1,max_hp_raw:100};
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
}
