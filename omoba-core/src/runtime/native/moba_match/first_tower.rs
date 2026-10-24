//! One match milestone, reduced once per tick in canonical tower identity order.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Claim {
    tick:u64,
    target:u64,
    side:Option<usize>,
    local:BTreeSet<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct FirstTowerState {
    pending:Option<Claim>,
    settled:Option<Claim>,
}

impl FirstTowerState {
    pub(super) fn observe(&mut self,tick:u64,target:u64,side:Option<usize>,local:BTreeSet<u32>) {
        if self.settled.is_none() && self.pending.as_ref().is_none_or(|claim|(tick,target)<(claim.tick,claim.target)) {
            self.pending=Some(Claim {tick,target,side,local});
        }
    }

    pub(super) fn settle(&mut self)->Option<(usize,BTreeSet<u32>)> {
        if self.settled.is_some() {return None;}
        let claim=self.pending.take()?;
        let award=claim.side.map(|side|(side,claim.local.clone()));
        self.settled=Some(claim);award
    }
}

pub(super) fn settle(world:&World,state:&mut MobaMatch) {
    if state.phase!=MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused {return;}
    if let Some((side,local))=state.first_tower.settle() {
        let bounty=state.config.building_reward_rules.first_tower;
        building_rewards::credit(world,state,side,&local,bounty);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{process_outcomes,RuntimeEventVecSink,TowerTemplateRegistry};
    use specs::Join;

    fn fixture()->(World,[Entity;2],[Entity;2]) {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        world.insert(TowerTemplateRegistry::default());
        setup_single_lane_match(&mut world,SingleLaneConfig {building_reward_rules:BuildingRewardRules::training(),..Default::default()}).unwrap();
        let (heroes,towers)={let mut state=world.write_resource::<MobaMatch>();state.phase=MobaMatchPhase::Playing;
            ([state.heroes[0].entity.unwrap(),state.heroes[1].entity.unwrap()],state.towers.map(Option::unwrap))};
        for gold in (&mut world.write_storage::<Gold>()).join() {gold.0=0;}
        for entity in heroes.into_iter().chain(towers) {world.write_storage::<Pos>().get_mut(entity).unwrap().0=Vec2::ZERO;}
        (world,heroes,towers)
    }
    fn hit(source:Entity,target:Entity)->Outcome {
        Outcome::Damage {pos:Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,
            real:Fixed64::from_i32(100_000),source,target,damage_profile:0,predeclared:false}
    }
    fn flush(world:&mut World,events:Vec<Outcome>) {
        world.write_resource::<Vec<Outcome>>().extend(events);
        process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();
        process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();
    }
    #[test]
    fn canonical_candidate_reduction_is_permutation_stable_and_keeps_earliest_tick() {
        for reverse in [true,false] {
            let mut state=FirstTowerState::default();
            let mut rows=[(9,20,0),(9,10,1)];if reverse {rows.reverse();}
            for (tick,key,side) in rows {state.observe(tick,key,Some(side),BTreeSet::from([side as u32+1]));}
            assert_eq!(state.settle(),Some((1,BTreeSet::from([2]))));
            state.observe(10,1,Some(0),BTreeSet::from([1]));assert_eq!(state.settle(),None);
        }
        let mut state=FirstTowerState::default();state.observe(1,100,Some(0),BTreeSet::new());
        state.observe(2,1,Some(1),BTreeSet::new());assert_eq!(state.settle(),Some((0,BTreeSet::new())));
        let mut state=FirstTowerState::default();state.observe(1,100,None,BTreeSet::new());assert_eq!(state.settle(),None);
        assert!(state.settled.is_some());state.observe(2,1,Some(1),BTreeSet::new());assert_eq!(state.settle(),None);
    }
    #[test]
    fn actual_same_tick_both_teams_get_normal_gold_but_one_gets_bonus_independent_of_order() {
        for reverse in [false,true] {
            let (mut world,heroes,towers)=fixture();let mut hits=vec![hit(heroes[0],towers[1]),hit(heroes[1],towers[0])];
            if reverse {hits.reverse();}flush(&mut world,hits);
            let winner=if entity_key(towers[0])<entity_key(towers[1]) {1}else{0};
            let before=single_lane_replay_digest(&world);
            finish_moba_match_tick(&mut world);
            assert_ne!(single_lane_replay_digest(&world),before);
            for side in 0..2 {assert_eq!(world.read_storage::<Gold>().get(heroes[side]).unwrap().0,if side==winner {500}else{350});}
            finish_moba_match_tick(&mut world);
            assert_eq!(world.read_storage::<Gold>().get(heroes[winner]).unwrap().0,500,"finish is idempotent for the milestone");
        }
    }
    #[test]
    fn pending_bonus_preserves_earned_recipients_through_death_pause_and_respawn() {
        let (mut world,heroes,towers)=fixture();flush(&mut world,vec![hit(heroes[0],towers[1])]);
        record_moba_death(&mut world,heroes[0]);
        world.write_resource::<GamePause>().is_paused=true;finish_moba_match_tick(&mut world);
        assert_eq!(world.read_resource::<MobaMatch>().heroes[0].gold.0,350);
        world.write_resource::<GamePause>().is_paused=false;finish_moba_match_tick(&mut world);
        assert_eq!(world.read_resource::<MobaMatch>().heroes[0].gold.0,500);
        let mut state=world.remove::<MobaMatch>().unwrap();state.spawn_hero(&mut world,0);let next=state.heroes[0].entity.unwrap();world.insert(state);
        assert_eq!(world.read_storage::<Gold>().get(next).unwrap().0,500);
    }
    #[test]
    fn forced_and_anonymous_first_tower_consume_milestone_and_disabled_policy_stays_zero() {
        for forced in [true,false] {
            let (mut world,heroes,towers)=fixture();
            if forced {flush(&mut world,vec![Outcome::Death {ent:towers[0],pos:Vec2::ZERO}]);}
            else {flush(&mut world,vec![Outcome::ScriptDirectDamage {target:towers[0],amount:Fixed64::from_i32(100_000)}]);}
            finish_moba_match_tick(&mut world);
            flush(&mut world,vec![hit(heroes[0],towers[1])]);finish_moba_match_tick(&mut world);
            assert_eq!(world.read_storage::<Gold>().get(heroes[0]).unwrap().0,350,"no delayed first-tower claim");
        }
        let (mut world,heroes,towers)=fixture();world.write_resource::<MobaMatch>().config.building_reward_rules=BuildingRewardRules::default();
        flush(&mut world,vec![hit(heroes[0],towers[1])]);finish_moba_match_tick(&mut world);
        assert_eq!(world.read_storage::<Gold>().get(heroes[0]).unwrap().0,0);
    }
}
