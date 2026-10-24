//! Objective auras own only lane-creep modifiers; overlapping emitters never multiply one policy.
use super::*;
use crate::runtime::{BuffStore,ability_runtime::buff_store::PERMANENT_BUFF_DURATION};

pub(super) fn synchronize(world:&World,state:&mut MobaMatch) {
    if state.phase==MobaMatchPhase::Playing && world.read_resource::<GamePause>().is_paused {return;}
    let emitters=objective_buffs::aura_emitters(world,state);
    let mut desired=BTreeMap::new();
    if !emitters.is_empty() {
        let positions=world.read_storage::<Pos>();let health=world.read_storage::<CProperty>();
        let entities=world.entities();
        for unit in state.units.values().filter(|u|u.role==LaneRole::Creep && !u.last_hit_retired) {
            if !entities.is_alive(unit.entity) || !health.get(unit.entity).is_some_and(|p|p.hp>Fixed64::ZERO) {continue;}
            let Some(pos)=positions.get(unit.entity) else {continue};
            for (side,rule,origins) in &emitters {
                let radius=Fixed64::from_i32(rule.radius);
                if usize::from(unit.team)==*side && origins.iter().any(|origin|(pos.0-*origin).length_squared()<=radius*radius) {
                    desired.insert((unit.entity,rule.buff_id),(rule.attack_bonus,rule.armor_bonus));
                }
            }
        }
    }
    let mut buffs=world.write_resource::<BuffStore>();
    for &(entity,id) in state.minion_aura_targets.keys() {
        if !desired.contains_key(&(entity,id)) {buffs.remove(entity,id);}
    }
    for (&(entity,id),&(attack,armor)) in &desired {
        if state.minion_aura_targets.get(&(entity,id))!=Some(&(attack,armor)) || !buffs.has(entity,id) {
            buffs.remove(entity,id);
            buffs.add(entity,id,PERMANENT_BUFF_DURATION,serde_json::json!({
                "baseattack_bonus_damage":i64::from(attack)*1024,"physical_armor_bonus":i64::from(armor)*1024,
                "__aggregation_family":id}));
        }
    }
    state.minion_aura_targets=desired;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{TowerTemplateRegistry,UnitStats};
    fn point(x:i32,y:i32)->Vec2 {Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y))}
    fn fixture()->(World,MobaMatch,Entity,Entity,Entity) {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        world.insert(TowerTemplateRegistry::default());
        let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        setup_single_lane_match(&mut world,SingleLaneConfig {map_id:Some(map.id.into()),lane_length:Fixed64::from_i32(map.lane_length),
            additional_players:vec![SingleLanePlayerConfig {player_id:3,team_id:1,hero:"training_luminary".into()}],..Default::default()}).unwrap();
        let mut state=world.remove::<MobaMatch>().unwrap();state.phase=MobaMatchPhase::Playing;state.spawn_wave(&mut world);
        let hero=state.heroes[0].entity.unwrap();let ally=state.heroes[2].entity.unwrap();
        let creep=state.units.values().find(|u|u.role==LaneRole::Creep && u.team==0).unwrap().entity;
        {let mut pos=world.write_storage::<Pos>();for slot in &state.heroes {pos.get_mut(slot.entity.unwrap()).unwrap().0=point(50_000,50_000);}
            pos.get_mut(hero).unwrap().0=Vec2::ZERO;pos.get_mut(creep).unwrap().0=Vec2::ZERO;}
        (world,state,hero,ally,creep)
    }
    fn reward(state:&MobaMatch)->omoba_template_ids::MobaObjectiveBuffConst {
        state.jungle_camps.iter().find_map(|c|c.definition.objective_buff.filter(|b|b.minion_aura.is_some())).unwrap()
    }
    #[test]
    fn inclusive_allied_lane_scope_overlap_and_normal_stats_are_authoritative() {
        let (world,mut state,hero,ally,creep)=fixture();let rule=reward(&state);let aura=rule.minion_aura.unwrap();
        let enemy=state.units.values().find(|u|u.role==LaneRole::Creep && u.team==1).unwrap().entity;
        let tower=state.units.values().find(|u|u.role==LaneRole::Tower && u.team==0).unwrap().entity;
        {let mut pos=world.write_storage::<Pos>();pos.get_mut(creep).unwrap().0=point(aura.radius,0);
            pos.get_mut(ally).unwrap().0=Vec2::ZERO;pos.get_mut(enemy).unwrap().0=Vec2::ZERO;pos.get_mut(tower).unwrap().0=Vec2::ZERO;}
        objective_buffs::grant_side(&world,&mut state,rule,0);objective_buffs::grant_side(&world,&mut state,rule,0);
        let buffs=world.read_resource::<BuffStore>();let stats=UnitStats::from_refs(&buffs,false);
        assert_eq!(stats.normal_attack_physical(Fixed64::from_i32(50),creep),Fixed64::from_i32(65));
        assert_eq!(stats.final_armor(Fixed64::ZERO,creep),Fixed64::from_i32(10));
        assert_eq!(buffs.iter_for(creep).filter(|(id,_)|*id==aura.buff_id).count(),1);
        for other in [hero,ally,enemy,tower] {assert!(!buffs.has(other,aura.buff_id));}
    }
    #[test]
    fn departure_dead_emitters_eligibility_expiry_and_terminal_clear_owned_modifiers() {
        let (world,mut state,hero,_,creep)=fixture();let rule=reward(&state);let id=rule.minion_aura.unwrap().buff_id;
        objective_buffs::grant_side(&world,&mut state,rule,0);assert!(world.read_resource::<BuffStore>().has(creep,id));
        world.write_storage::<Pos>().get_mut(creep).unwrap().0=point(901,0);synchronize(&world,&mut state);
        assert!(!world.read_resource::<BuffStore>().has(creep,id));
        world.write_storage::<Pos>().get_mut(creep).unwrap().0=Vec2::ZERO;synchronize(&world,&mut state);
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::ZERO;synchronize(&world,&mut state);
        assert!(!world.read_resource::<BuffStore>().has(creep,id));
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::from_i32(100);synchronize(&world,&mut state);
        assert!(world.read_resource::<BuffStore>().has(creep,id));
        objective_buffs::retire_hero(&mut state,1);synchronize(&world,&mut state);assert!(!world.read_resource::<BuffStore>().has(creep,id));
        objective_buffs::grant_side(&world,&mut state,rule,0);state.elapsed+=Fixed64::from_i32(181);
        objective_buffs::synchronize(&world,&mut state,Fixed64::ZERO);assert!(!world.read_resource::<BuffStore>().has(creep,id));
        objective_buffs::grant_side(&world,&mut state,rule,0);state.phase=MobaMatchPhase::Finished {winner:Some(0),tick:9};
        objective_buffs::synchronize(&world,&mut state,Fixed64::ZERO);assert!(state.minion_aura_targets.is_empty());
        assert!(!world.read_resource::<BuffStore>().has(creep,id));
    }
    #[test]
    fn pause_new_waves_removed_generations_and_missing_instances_obey_membership() {
        let (mut world,mut state,hero,_,creep)=fixture();let rule=reward(&state);let id=rule.minion_aura.unwrap().buff_id;
        objective_buffs::grant_side(&world,&mut state,rule,0);
        world.write_resource::<BuffStore>().remove(creep,id);synchronize(&world,&mut state);
        assert!(world.read_resource::<BuffStore>().has(creep,id));
        world.write_resource::<GamePause>().is_paused=true;
        world.write_storage::<Pos>().get_mut(hero).unwrap().0=point(10_000,10_000);synchronize(&world,&mut state);
        assert!(world.read_resource::<BuffStore>().has(creep,id));
        world.write_resource::<GamePause>().is_paused=false;synchronize(&world,&mut state);assert!(!world.read_resource::<BuffStore>().has(creep,id));
        world.write_storage::<Pos>().get_mut(hero).unwrap().0=Vec2::ZERO;
        synchronize(&world,&mut state);assert!(world.read_resource::<BuffStore>().has(creep,id));
        world.entities().delete(creep).unwrap();world.maintain();state.spawn_wave(&mut world);
        let next=state.units.values().filter(|u|u.role==LaneRole::Creep && u.team==0 && u.entity!=creep && world.entities().is_alive(u.entity))
            .last().unwrap().entity;
        world.write_storage::<Pos>().get_mut(next).unwrap().0=Vec2::ZERO;synchronize(&world,&mut state);
        assert!(world.read_resource::<BuffStore>().has(next,id));assert!(!world.read_resource::<BuffStore>().has(creep,id));
        assert!(!state.minion_aura_targets.keys().any(|(target,_)|*target==creep));
    }
}
