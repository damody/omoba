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
            BotAbilityIntent::ApproachEnemyPoint {min_distance} if def.target_type==TargetTypeC::Point
                && (1..=10_000).contains(min_distance)
                && def.levels.iter().all(|level|level.range>Fixed64::from_i32(*min_distance as i32))=>{},
            BotAbilityIntent::SelfHeal {below_hp_per_mille} if def.target_type==TargetTypeC::None
                && *below_hp_per_mille>0 && *below_hp_per_mille<=1000 => {},
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

#[cfg(test)]
fn choose_cast(hero:&Hero,role:BotRole,team:u32,own:Vec2,health:&CProperty,
    seen:&[SeenUnit],policies:&[BotAbilityPolicy]) -> Option<CastAbility> {
    choose_cast_with_mana_budget(hero,role,team,own,health,seen,policies,|_,_|None)
}

/// Cost provider uses the host's exact script metadata, not a second rounded
/// compiled copy. The planner never reserves or spends the owner's mana.
pub(super) fn choose_cast_with_mana_budget(hero:&Hero,role:BotRole,team:u32,own:Vec2,health:&CProperty,
    seen:&[SeenUnit],policies:&[BotAbilityPolicy],cost:impl Fn(&str,u8)->Option<Fixed64>) -> Option<CastAbility> {
    // Author declaration order is the explicit priority. Slot is looked up in
    // this owner's current loadout; skill names never imply Q/W/E/R.
    for policy in policies {
        let Some(slot)=hero.abilities.iter().position(|id|id==&policy.ability) else {continue;};
        if slot>=4 {continue;} // formal PlayerInput exposes exactly four slots
        if !hero.can_use_ability(&policy.ability) || hero.is_on_cooldown(&policy.ability) {continue;}
        let owner_rank=hero.get_ability_level(&policy.ability);
        if !(1..=255).contains(&owner_rank) {continue;}
        let rank=(owner_rank-1) as usize;
        let Some(def)=omoba_template_ids::ability_by_name(&policy.ability).and_then(omoba_template_ids::active_ability_const) else {continue;};
        let Some(level)=def.levels.get(rank) else {continue;};
        if let Some(pool)=&hero.mana_pool {
            let Some(required)=cost(&policy.ability,owner_rank as u8) else {continue;};
            if required<Fixed64::ZERO || required>pool.current() {continue;}
        }
        let mut target_pos=None;
        let target=match &policy.intent {
            BotAbilityIntent::SelfHeal {below_hp_per_mille} => {
                if health.hp<=Fixed64::ZERO || health.mhp<=Fixed64::ZERO
                    || i128::from(health.hp.raw())*1000 >= i128::from(health.mhp.raw())*i128::from(*below_hp_per_mille) {continue;}
                None
            }
            BotAbilityIntent::EnemyUnit => {
                if level.range<=Fixed64::ZERO {continue;}
                let Some(unit)=seen.iter().filter(|unit|unit.hp_raw>0 && unit.team!=team
                    && if role==BotRole::Jungle {unit.kind==3} else {unit.team!=0 && matches!(unit.kind,1|2)})
                    .filter(|unit|(unit.position-own).length_squared()<=level.range*level.range)
                    .min_by_key(|unit|((unit.position-own).length_squared().raw(),unit.canonical_id)) else {continue;};
                Some(unit.canonical_id as u32)
            }
            BotAbilityIntent::ApproachEnemyPoint {min_distance}=>{
                let minimum=Fixed64::from_i32(*min_distance as i32);
                let Some(unit)=seen.iter().filter(|unit|unit.hp_raw>0 && unit.team!=team
                    && if role==BotRole::Jungle {unit.kind==3} else {unit.team!=0 && matches!(unit.kind,1|2)})
                    .filter(|unit|{let distance=(unit.position-own).length_squared();
                        distance>minimum*minimum && distance<=level.range*level.range})
                    .min_by_key(|unit|((unit.position-own).length_squared().raw(),unit.canonical_id)) else {continue;};
                let (Ok(x),Ok(y))=(i32::try_from(unit.position.x.raw()),i32::try_from(unit.position.y.raw())) else {continue;};
                target_pos=Some(crate::runtime::Vec2I {x,y});None
            }
            BotAbilityIntent::EnemyPoint {radius_key,min_targets}=>{
                let Some((_,radii))=def.extras.iter().find(|(key,_)|*key==radius_key) else {continue;};
                let Some(radius)=radii.get(rank).copied() else {continue;};
                let mut eligible:Vec<_>=seen.iter().filter(|u|u.hp_raw>0 && u.team!=team
                    && if role==BotRole::Jungle {u.kind==3} else {u.team!=0 && matches!(u.kind,1|2)}).collect();
                eligible.sort_by_key(|u|((u.position-own).length_squared().raw(),u.canonical_id));
                eligible.truncate(512);
                // Candidate centers are disclosed enemy poses; no hidden targets,
                // inferred motion or private aggro. Most covered targets wins.
                let center=eligible.iter().filter(|u|(u.position-own).length_squared()<=level.range*level.range).take(32)
                    .filter_map(|u|{
                        let count=eligible.iter().filter(|v|(v.position-u.position).length_squared()<=radius*radius).count();
                        (count>=usize::from(*min_targets)).then_some((std::cmp::Reverse(count),(u.position-own).length_squared().raw(),u.canonical_id,u.position))
                    }).min_by_key(|(count,distance,id,_)|(*count,*distance,*id));
                let Some((_,_,_,center))=center else {continue;};
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
    fn mana_budget_bot_skips_unaffordable_or_invalid_cost_without_spending() {
        use crate::runtime::ability_runtime::{checked_mana_cost,ManaPool};
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["lumen_touch".into(),"lumen_bolt".into()];
        hero.ability_levels.insert("lumen_touch".into(),1);
        hero.ability_levels.insert("lumen_bolt".into(),1);
        let mut health=property(Fixed64::from_i32(100),Fixed64::ZERO);
        health.hp=Fixed64::from_i32(20);
        let unit=SeenUnit {canonical_id:(1<<32)|77,position:Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO),
            team:2,kind:1,owner_player_id:2,hp_raw:1};
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
            team:2,kind:1,owner_player_id:2,hp_raw:1};
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
            canonical_id:id,position:Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO),team,kind:1,owner_player_id:id as u32,hp_raw:1});
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
            team:2,kind:1,owner_player_id:2,hp_raw:1};
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
