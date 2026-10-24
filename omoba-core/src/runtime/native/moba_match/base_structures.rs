//! Map-authored base progression. All deadlines use the paused match clock;
//! retirement, never an uncommitted HP observation, opens a prerequisite.
use super::*;
use omoba_template_ids::{MobaBaseStructuresConst, MobaMinionConst};

#[derive(Clone, Debug)]
pub(super) struct RespawningStructure {
    pub team: usize,
    pub lane: usize,
    pub role: LaneRole,
    pub entity: Option<Entity>,
    respawn_at: Option<Fixed64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Join;

    fn fixture()->World {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        setup_single_lane_match(&mut world,SingleLaneConfig {map_id:Some("three_lane_training".into()),
            lane_length:Fixed64::from_i32(omoba_template_ids::moba_map_by_name("three_lane_training").unwrap().lane_length),
            authored_minion_waves:true,..SingleLaneConfig::default()}).unwrap();
        world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
        world
    }
    fn retire(world:&mut World,entity:Entity) {
        record_moba_death(world,entity);
        world.delete_entity(entity).unwrap();world.maintain();
    }
    fn node(world:&World,team:usize,lane:usize,role:LaneRole)->Entity {
        world.read_resource::<MobaMatch>().base_structures.iter()
            .find(|n|n.team==team && n.lane==lane && n.role==role).unwrap().entity.unwrap()
    }
    fn clear_lane(world:&mut World,team:usize,lane:usize) {
        let towers:Vec<_>=world.read_resource::<MobaMatch>().lane_tower_layers[lane].iter().filter_map(|l|l[team]).collect();
        for tower in towers {retire(world,tower)}
    }
    #[test]
    fn base_structures_chain_relocks_after_each_respawn_and_retires_once() {
        let mut world=fixture();
        let inhibitor=node(&world,1,0,LaneRole::Inhibitor);
        let guards=[node(&world,1,0,LaneRole::NexusTower),node(&world,1,2,LaneRole::NexusTower)];
        let base=world.read_resource::<MobaMatch>().bases[1].unwrap();
        assert!(!moba_damage_allowed(&world,inhibitor));
        assert!(guards.iter().all(|&e|!moba_damage_allowed(&world,e)));
        clear_lane(&mut world,1,0);assert!(moba_damage_allowed(&world,inhibitor));
        retire(&mut world,inhibitor);
        let digest=single_lane_replay_digest(&world);
        record_moba_death(&mut world,inhibitor);assert_eq!(single_lane_replay_digest(&world),digest);
        assert!(guards.iter().all(|&e|moba_damage_allowed(&world,e)));
        retire(&mut world,guards[0]);assert!(!moba_damage_allowed(&world,base));
        retire(&mut world,guards[1]);assert!(moba_damage_allowed(&world,base));
        let mut state=world.remove::<MobaMatch>().unwrap();
        state.elapsed=Fixed64::from_i32(180)-Fixed64::from_raw(1);
        state.tick_base_structures(&mut world);
        assert!(state.base_unlocked(1));
        state.elapsed+=Fixed64::from_raw(1);state.tick_base_structures(&mut world);
        assert!(!state.base_unlocked(1));
        let new_guard=state.base_structures.iter().find(|n|n.team==1 && n.role==LaneRole::NexusTower).unwrap().entity.unwrap();
        assert_ne!(new_guard,guards[0]);
        let p=world.read_storage::<CProperty>().get(new_guard).unwrap().clone();
        assert_eq!(p.hp,Fixed64::from_i32(480));assert_eq!(p.mhp,Fixed64::from_i32(1200));
        state.elapsed=Fixed64::from_i32(300);state.tick_base_structures(&mut world);world.insert(state);
        let new_inhibitor=node(&world,1,0,LaneRole::Inhibitor);
        assert_ne!(new_inhibitor,inhibitor);assert!(moba_damage_allowed(&world,new_inhibitor));
        assert!(!moba_damage_allowed(&world,new_guard));assert!(!moba_damage_allowed(&world,base));
    }
    #[test]
    fn base_structures_pause_terminal_and_hash_cover_pending_deadlines() {
        let mut world=fixture();clear_lane(&mut world,1,1);
        let inhibitor=node(&world,1,1,LaneRole::Inhibitor);retire(&mut world,inhibitor);
        let before=single_lane_replay_digest(&world);
        world.write_resource::<MobaMatch>().base_structures.iter_mut().find(|n|n.entity.is_none()).unwrap().respawn_at=Some(Fixed64::from_i32(301));
        assert_ne!(single_lane_replay_digest(&world),before);
        let mut state=world.remove::<MobaMatch>().unwrap();state.elapsed=Fixed64::from_i32(400);
        world.write_resource::<GamePause>().is_paused=true;state.tick_base_structures(&mut world);
        assert!(state.inhibitor_down(1,Some(1)));
        world.write_resource::<GamePause>().is_paused=false;state.phase=MobaMatchPhase::Finished {winner:Some(0),tick:1};
        state.tick_base_structures(&mut world);assert!(state.inhibitor_down(1,Some(1)));
        state.phase=MobaMatchPhase::Playing;state.tick_base_structures(&mut world);assert!(!state.inhibitor_down(1,Some(1)));
    }
    #[test]
    fn base_structures_actual_wave_replaces_siege_only_in_opposing_lane_and_doubles() {
        let mut world=fixture();let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        let normal:Vec<_>=map.minion_wave.iter().flat_map(|m|std::iter::repeat_n(*m,m.count as usize)).collect();
        for lane in 0..3 {
            clear_lane(&mut world,1,lane);let inhibitor=node(&world,1,lane,LaneRole::Inhibitor);retire(&mut world,inhibitor);
            let state=world.read_resource::<MobaMatch>();
            for test_lane in 0..3 {
                let wave=state.reinforced_wave(0,test_lane,&normal);
                assert_eq!(wave.iter().filter(|m|m.id=="lane_super").count(),if lane==2 {2}else{usize::from(test_lane<=lane)});
                assert_eq!(wave.iter().any(|m|m.id=="lane_siege"),test_lane>lane);
                assert_eq!(state.reinforced_wave(1,test_lane,&normal).iter().filter(|m|m.id=="lane_super").count(),0);
            }
        }
        let mut state=world.remove::<MobaMatch>().unwrap();state.waves=2;state.spawn_wave(&mut world);
        let count=(&world.entities(),&world.read_storage::<Unit>()).join().filter(|(_,u)|u.id=="lane_super").count();
        assert_eq!(count,6,"real ECS receives two super minions per attacking lane");
        state.elapsed=Fixed64::from_i32(300);state.tick_base_structures(&mut world);
        for lane in 0..3 {let wave=state.reinforced_wave(0,lane,&normal);assert_eq!(wave.len(),7);assert!(wave.iter().all(|m|m.id!="lane_super"));}
        assert_eq!((&world.entities(),&world.read_storage::<Unit>()).join().filter(|(_,u)|u.id=="lane_super").count(),6,"existing wave survives inhibitor restoration");
    }
}

impl MobaMatch {
    pub(super) fn base_structure_rules(&self) -> Option<MobaBaseStructuresConst> {
        self.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name)
            .and_then(|map|map.base_structures)
    }

    pub(super) fn setup_base_structures(&mut self, world: &mut World) {
        let Some(rules)=self.base_structure_rules() else {return};
        for team in 0..2 {
            for lane in 0..self.routes.len() {
                let entity=self.spawn_structure(world,team,LaneRole::Inhibitor,lane,Fixed64::from_i32(rules.inhibitor_offset));
                self.base_structures.push(RespawningStructure {team,lane,role:LaneRole::Inhibitor,entity:Some(entity),respawn_at:None});
            }
            // The first/last lane leave the common base in different directions.
            for lane in [0,self.routes.len()-1] {
                let entity=self.spawn_structure(world,team,LaneRole::NexusTower,lane,Fixed64::from_i32(rules.nexus_tower_offset));
                self.base_structures.push(RespawningStructure {team,lane,role:LaneRole::NexusTower,entity:Some(entity),respawn_at:None});
            }
        }
    }

    pub(super) fn inhibitor_down(&self,team:usize,lane:Option<usize>)->bool {
        self.base_structures.iter().any(|node|node.team==team && node.role==LaneRole::Inhibitor
            && lane.is_none_or(|lane|node.lane==lane) && node.entity.is_none() && node.respawn_at.is_some())
    }

    pub(super) fn retire_base_structure(&mut self,entity:Entity) {
        let rules=self.base_structure_rules().expect("authored structure");
        if let Some(node)=self.base_structures.iter_mut().find(|node|node.entity==Some(entity)) {
            node.entity=None;
            let delay=if node.role==LaneRole::Inhibitor {rules.inhibitor_respawn_seconds}else{rules.nexus_tower_respawn_seconds};
            node.respawn_at=Some(self.elapsed+Fixed64::from_i32(delay));
        }
    }

    pub(super) fn tick_base_structures(&mut self,world:&mut World) {
        if self.phase!=MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused {return}
        let Some(rules)=self.base_structure_rules() else {return};
        for index in 0..self.base_structures.len() {
            let node=&self.base_structures[index];
            if !node.respawn_at.is_some_and(|deadline|self.elapsed>=deadline) {continue}
            let (team,lane,role)=(node.team,node.lane,node.role);
            let offset=if role==LaneRole::Inhibitor {rules.inhibitor_offset}else{rules.nexus_tower_offset};
            let entity=self.spawn_structure(world,team,role,lane,Fixed64::from_i32(offset));
            if role==LaneRole::NexusTower {
                let mut properties=world.write_storage::<CProperty>();
                let p=properties.get_mut(entity).expect("spawned structure HP");
                p.hp=p.mhp*Fixed64::from_i32(rules.nexus_tower_respawn_hp_percent)/Fixed64::from_i32(100);
                if let Some(unit)=world.write_storage::<Unit>().get_mut(entity) {
                    unit.current_hp=(p.hp.raw()/1024) as i32;
                }
            }
            self.base_structures[index].entity=Some(entity);
            self.base_structures[index].respawn_at=None;
        }
    }

    pub(super) fn reinforced_wave(&self,team:usize,lane:usize,normal:&[MobaMinionConst])->Vec<MobaMinionConst> {
        let Some(rules)=self.base_structure_rules().filter(|_|self.config.authored_minion_waves) else {return normal.to_vec()};
        let defender=1-team;
        if !self.inhibitor_down(defender,Some(lane)) {return normal.to_vec()}
        let all_down=(0..self.routes.len()).all(|lane|self.inhibitor_down(defender,Some(lane)));
        let mut wave:Vec<_>=normal.iter().copied().filter(|m|m.id!=rules.siege_minion_id).collect();
        wave.extend(std::iter::repeat_n(rules.super_minion,if all_down {2}else{1}));
        wave
    }
}
