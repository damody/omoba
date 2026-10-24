use super::*;

fn p(x:i32,y:i32)->Vec2 {Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y))}
fn unit(id:u64,x:i32,y:i32)->SeenUnit {SeenUnit{canonical_id:id,position:p(x,y),team:2,kind:1,owner_player_id:id as u32,hp_raw:100,max_hp_raw:100}}
fn policy(min_targets:u16)->BotAbilityPolicy {BotAbilityPolicy{ability:"ranger_scatter".into(),intent:BotAbilityIntent::EnemyCone{min_targets}}}
fn hero()->Hero {let mut h=Hero::new("unrelated".into(),"unrelated".into(),"unrelated".into());h.abilities=vec!["ranger_scatter".into()];h.ability_levels.insert("ranger_scatter".into(),1);h}

#[test]
fn cone_policy_is_closed_and_requires_actual_compiled_rank_geometry() {
    assert!(validate_policies(&[policy(2)]).is_ok());
    for min in [0,129] {assert!(validate_policies(&[policy(min)]).is_err());}
    for name in ["ranger_volley","lumen_bolt","fire_dash","missing"] {
        let mut bad=policy(2);bad.ability=name.into();assert!(validate_policies(&[bad]).is_err(),"{name}");
    }
    assert!(serde_json::from_str::<BotAbilityIntent>(r#"{"kind":"enemy_cone","min_targets":2,"half_angle_key":"fake"}"#).is_err());
    let parsed:BotAbilityIntent=serde_json::from_str(r#"{"kind":"enemy_cone","min_targets":2}"#).unwrap();
    assert!(matches!(parsed,BotAbilityIntent::EnemyCone{min_targets:2}));
}

#[test]
fn cone_selection_counts_full_source_range_deduplicates_and_is_permutation_stable() {
    let points=[unit(30,-50,0),unit(20,600,0),unit(10,100,0),unit(10,100,0),unit(40,701,0),SeenUnit{hp_raw:0,..unit(50,200,0)}];
    for ticks in omoba_template_ids::active_ability_const(omoba_template_ids::ABILITY_RANGER_SCATTER).unwrap().cone_half_angle_ticks {
        for reverse in [false,true] {let mut seen=points.to_vec();if reverse{seen.reverse();}
            assert_eq!(choose_cone_target(Vec2::ZERO,Fixed64::from_i32(700),*ticks,2,seen.iter().collect(),None),Some(p(100,0)));
            assert!(choose_cone_target(Vec2::ZERO,Fixed64::from_i32(700),*ticks,3,seen.iter().collect(),None).is_none());
        }
    }
}

#[test]
fn cone_focus_means_sector_coverage_and_hidden_focus_never_supplies_a_direction() {
    let seen=[unit(10,100,0),unit(20,600,0),unit(30,-100,0)];
    let choose=|focus,min|choose_cone_target(Vec2::ZERO,Fixed64::from_i32(700),341,min,seen.iter().collect(),focus);
    assert_eq!(choose(Some(20),2),Some(p(100,0)),"cover focus while aiming at nearer enemy");
    assert_eq!(choose(Some(30),1),Some(p(-100,0)),"disclosed guard focus overrides larger unrelated group");
    assert_eq!(choose(Some(999),2),choose(None,2),"missing focus cannot be reconstructed");
}

#[test]
fn cone_planning_enforces_boundaries_zero_direction_and_known_budgets() {
    let seen=[unit(10,0,700),unit(20,0,-700),unit(30,700,0),unit(50,701,0)];
    let chosen=choose_cone_target(Vec2::ZERO,Fixed64::from_i32(700),1024,3,seen.iter().collect(),None).unwrap();
    assert_eq!(chosen,p(700,0),"closed half-plane covers both exact angular/radial edges");
    assert!(choose_cone_target(Vec2::ZERO,Fixed64::from_i32(700),341,1,vec![&unit(1,0,0)],None).is_none());
    for ticks in [0,1025] {assert!(choose_cone_target(Vec2::ZERO,Fixed64::from_i32(700),ticks,1,seen.iter().collect(),None).is_none());}
    let crowded:Vec<_>=(1..=129).map(|id|unit(id,100,0)).collect();
    assert!(choose_cone_target(Vec2::ZERO,Fixed64::from_i32(700),341,1,crowded.iter().collect(),None).is_none());
    let over_budget:Vec<_>=(1..=513).map(|id|unit(id,100,0)).collect();
    assert!(choose_cone_target(Vec2::ZERO,Fixed64::from_i32(700),341,1,over_budget.iter().collect(),None).is_none());
}

#[test]
fn cone_intent_keeps_owner_resource_attack_control_and_role_admission() {
    use crate::runtime::ability_runtime::ManaPool;
    let mut hero=hero();hero.mana_pool=Some(ManaPool::new(Fixed64::from_i32(45),Fixed64::from_i32(280)).unwrap());
    let hp=property(Fixed64::from_i32(100),Fixed64::ZERO);let enemies=[unit(1,100,0),unit(2,600,0)];
    let decide=|hero:&Hero,seen:&[SeenUnit],defer,rooted|choose_cast_with_attack_priority(hero,BotRole::Carry,1,Vec2::ZERO,&hp,
        seen,&[policy(2)],None,defer,rooted,|_,_|Some(Fixed64::from_i32(45)),|_,_|None,|_|panic!("cone is not a relocation"));
    let before=hero.mana_pool.clone();let cast=decide(&hero,&enemies,false,true).unwrap();
    assert_eq!((cast.ability_index,cast.target_entity,cast.target_pos),(0,None,Some(crate::runtime::Vec2I{x:100*1024,y:0})));
    assert_eq!(hero.mana_pool,before);assert!(decide(&hero,&enemies,true,false).is_none());
    for invalid in [SeenUnit{team:1,..enemies[1]},SeenUnit{team:0,kind:3,..enemies[1]},SeenUnit{hp_raw:0,..enemies[1]}] {
        assert!(decide(&hero,&[enemies[0],invalid],false,false).is_none());
    }
    hero.mana_pool.as_mut().unwrap().spend(Fixed64::from_raw(1)).unwrap();assert!(decide(&hero,&enemies,false,false).is_none());
}
