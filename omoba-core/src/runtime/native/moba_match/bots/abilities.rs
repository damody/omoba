//! Explicit author-owned AI intentions, not tooltip effect inference.
use super::*;
use crate::runtime::CastAbility;
use omoba_template_ids::{AbilityTypeC, CastTypeC, TargetTypeC};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum BotAbilityIntent {
    EnemyUnit,
    EnemyPoint {radius_key:String,min_targets:u16},
    /// Author explicitly opts a point skill into closing on disclosed enemies.
    ApproachEnemyPoint {min_distance:u32},
    SelfHeal { below_hp_per_mille:u16 },
    AllyHeal { below_hp_per_mille:u16 },
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
        let Some(slot)=hero.abilities.iter().position(|id|id==&step.ability).filter(|slot|*slot<4) else {continue;};
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
        if !matches!(def.ability_type,AbilityTypeC::Active | AbilityTypeC::Ultimate)
            || def.cast_type!=CastTypeC::Instant {return Err("bot policy requires an instant active ability");}
        match &policy.intent {
            BotAbilityIntent::EnemyUnit if def.target_type==TargetTypeC::Unit => {},
            BotAbilityIntent::AllyHeal {below_hp_per_mille} if def.target_type==TargetTypeC::Unit
                && (1..=1000).contains(below_hp_per_mille)=>{},
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
            _ => return Err("bot ability intent does not match target type or health threshold"),
        }
    }
    Ok(())
}

pub(super) fn requires_mana(policies:&[BotAbilityPolicy]) -> bool {
    policies.iter().any(|policy|matches!(policy.intent,
        BotAbilityIntent::SelfRecovery {..}|BotAbilityIntent::SelfManaRegeneration {..}))
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
    choose_cast_with_attack_priority(hero,role,team,own,health,seen,policies,focus,false,false,cost,regen)
}

/// Defer offensive intentions only; recovery still follows author ordering.
pub(super) fn choose_cast_with_attack_priority(hero:&Hero,role:BotRole,team:u32,own:Vec2,health:&CProperty,
    seen:&[SeenUnit],policies:&[BotAbilityPolicy],focus:Option<u64>,defer_offense:bool,immobilized:bool,
    cost:impl Fn(&str,u8)->Option<Fixed64>,
    regen:impl Fn(&str,Fixed64)->Option<(Fixed64,Fixed64)>) -> Option<CastAbility> {
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
            |BotAbilityIntent::EnemyPoint {..}|BotAbilityIntent::ApproachEnemyPoint {..}) {continue;}
        let Some(slot)=hero.abilities.iter().position(|id|id==&policy.ability) else {continue;};
        if slot>=4 {continue;} // formal PlayerInput exposes exactly four slots
        if !hero.can_use_ability(&policy.ability) || hero.is_on_cooldown(&policy.ability) {continue;}
        let owner_rank=hero.get_ability_level(&policy.ability);
        if !(1..=255).contains(&owner_rank) {continue;}
        let rank=(owner_rank-1) as usize;
        let Some(def)=omoba_template_ids::ability_by_name(&policy.ability).and_then(omoba_template_ids::active_ability_const) else {continue;};
        let Some(level)=def.levels.get(rank) else {continue;};
        let required_cost=if let Some(pool)=&hero.mana_pool {
            let Some(required)=cost(&policy.ability,owner_rank as u8) else {continue;};
            if required<Fixed64::ZERO || required>pool.current() {continue;}
            Some(required)
        } else {None};
        let mut target_pos=None;
        let target=match &policy.intent {
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
                    .min_by_key(|unit|priority(unit)) else {continue;};
                let (Ok(x),Ok(y))=(i32::try_from(unit.position.x.raw()),i32::try_from(unit.position.y.raw())) else {continue;};
                target_pos=Some(crate::runtime::Vec2I {x,y});None
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

#[cfg(test)]
mod tests {
    use super::*;
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
