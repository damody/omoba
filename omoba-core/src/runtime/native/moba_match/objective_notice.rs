//! Public objective milestones from committed damage, never renderer inference.
use super::*;
use std::collections::VecDeque;
pub const OBJECTIVE_TEAM_METRIC_ID:u64=0x4d4f5445414d3031;
pub const OBJECTIVE_KIND_METRIC_ID:u64=0x4d4f4b494e443031;
pub const OBJECTIVE_SEQUENCE_METRIC_ID:u64=0x4d4f534551553031;
pub const OBJECTIVE_REMAINING_METRIC_ID:u64=0x4d4f54494d453031;
const DURATION:Fixed64=Fixed64::from_raw(4*1024);
#[derive(Clone,Copy,Debug)]
struct Notice {team:u32,kind:u32}
#[derive(Clone,Debug,Default)]
pub(super) struct ObjectiveNotices {
    pending:BTreeMap<(u64,u64),Notice>,
    queue:VecDeque<Notice>,
    current:Option<Notice>,
    until:Fixed64,
    sequence:i64,
}
impl ObjectiveNotices {
    fn values(&self,state:&MobaMatch)->[i64;4] {
        if state.phase!=MobaMatchPhase::Playing || state.elapsed>=self.until {return [0;4];}
        self.current.map_or([0;4],|n|[i64::from(n.team),i64::from(n.kind),self.sequence,(self.until-state.elapsed).raw()])
    }
}
pub(super) fn observe(world:&World,state:&mut MobaMatch,target:Entity,side:usize,kind:u32) {
    if state.phase!=MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused
        || side>=2 || !(1..=4).contains(&kind) {return;}
    state.objective_notices.pending.entry((world.read_resource::<Tick>().0,entity_key(target)))
        .or_insert(Notice {team:state.config.teams[side],kind});
}
pub(super) fn settle(world:&World,state:&mut MobaMatch) {
    if state.phase!=MobaMatchPhase::Playing {state.objective_notices=ObjectiveNotices::default();return;}
    if world.read_resource::<GamePause>().is_paused {return;}
    let notices=&mut state.objective_notices;
    for (_,notice) in std::mem::take(&mut notices.pending) {
        // Keep the current notice uninterrupted and at most seven recent waiters.
        if notices.queue.len()==7 {notices.queue.pop_front();}
        notices.queue.push_back(notice);
    }
    if notices.current.is_none() || state.elapsed>=notices.until {
        notices.current=notices.queue.pop_front();
        if notices.current.is_some() {
            notices.sequence=notices.sequence.saturating_add(1);
            notices.until=Fixed64::from_raw(state.elapsed.raw().saturating_add(DURATION.raw()));
        }
    }
}
pub(super) fn emit(world:&World,state:&MobaMatch,phase:crate::runtime::FactPhase) {
    for (index,(id,value)) in [OBJECTIVE_TEAM_METRIC_ID,OBJECTIVE_KIND_METRIC_ID,OBJECTIVE_SEQUENCE_METRIC_ID,OBJECTIVE_REMAINING_METRIC_ID]
        .into_iter().zip(state.objective_notices.values(state)).enumerate() {
        world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(crate::runtime::OrderedFact {
            key:crate::runtime::FactOrderingKey {tick:world.read_resource::<Tick>().0,phase,canonical_source_order:0,
                local_ordinal:180+index as u32,fact_kind:crate::runtime::FactKind::Hud},
            audience:crate::runtime::FactAudience::AllPlayers,
            fact:crate::runtime::ObservableFact::Hud {team:0,metric_id:id,value},
        }).expect("valid public objective notice");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{TowerTemplateRegistry,process_outcomes,RuntimeEventVecSink};
    fn fixture()->World {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut w=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);w.insert(TowerTemplateRegistry::default());
        let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        setup_single_lane_match(&mut w,SingleLaneConfig {map_id:Some(map.id.into()),lane_length:Fixed64::from_i32(map.lane_length),..Default::default()}).unwrap();
        w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;w
    }
    fn update(w:&mut World,f:impl FnOnce(&World,&mut MobaMatch)) {let mut s=w.remove::<MobaMatch>().unwrap();f(w,&mut s);w.insert(s);}
    fn lethal(w:&mut World,source:Option<Entity>,target:Entity) {
        w.write_resource::<Vec<Outcome>>().push(source.map_or(Outcome::ScriptDirectDamage {target,amount:Fixed64::from_i32(100_000)},|source|
            Outcome::Damage {source,target,pos:Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,real:Fixed64::from_i32(100_000),damage_profile:0,predeclared:false}));
        process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();
    }
    #[test]
    fn simultaneous_committed_objectives_are_canonical_once_and_preserve_current_notice() {
        let mut results=Vec::new();
        for reverse in [false,true] {
            let mut w=fixture();let mut targets={let s=w.read_resource::<MobaMatch>();
                vec![(s.lane_towers[0][1].unwrap(),s.heroes[0].entity.unwrap()),
                    (s.lane_towers[1][1].unwrap(),s.heroes[0].entity.unwrap()),
                    (s.lane_towers[2][0].unwrap(),s.heroes[1].entity.unwrap())]};
            let expected_team={let source=targets.iter().min_by_key(|(target,_)|entity_key(*target)).unwrap().1;
                i64::from(w.read_storage::<Faction>().get(source).unwrap().team_id)};
            if reverse {targets.reverse();}
            for (tower,source) in targets {lethal(&mut w,Some(source),tower);lethal(&mut w,Some(source),tower);}
            update(&mut w,|w,s|{assert_eq!(s.objective_notices.pending.len(),3);settle(w,s);
                assert_eq!(s.objective_notices.values(s),[expected_team,1,1,4096]);assert_eq!(s.objective_notices.queue.len(),2);
                let current=s.objective_notices.values(s);settle(w,s);assert_eq!(s.objective_notices.values(s),current);
                results.push(format!("{:?}",s.objective_notices));});
        }
        assert_eq!(results[0],results[1]);
    }
    #[test]
    fn queue_is_bounded_pause_expiry_sequence_digest_and_terminal_are_explicit() {
        let mut w=fixture();let before=single_lane_replay_digest(&w);
        update(&mut w,|w,s|{let target=s.towers[1].unwrap();observe(w,s,target,0,1);settle(w,s);});
        assert_ne!(single_lane_replay_digest(&w),before);
        update(&mut w,|w,s|{let target=s.towers[1].unwrap();
            for tick in 1..30 {s.objective_notices.pending.insert((tick,entity_key(target)),Notice {team:2,kind:4});}
            settle(w,s);assert_eq!(s.objective_notices.queue.len(),7);assert_eq!(s.objective_notices.values(s),[1,1,1,4096]);});
        w.write_resource::<GamePause>().is_paused=true;
        update(&mut w,|w,s|{let at=s.objective_notices.until;settle(w,s);assert_eq!(s.objective_notices.until,at);});
        w.write_resource::<GamePause>().is_paused=false;
        update(&mut w,|w,s|{s.elapsed+=DURATION;settle(w,s);assert_eq!(s.objective_notices.values(s),[2,4,2,4096]);
            for _ in 0..7 {s.elapsed+=DURATION;settle(w,s);}assert_eq!(s.objective_notices.values(s),[0;4]);
            let target=s.towers[1].unwrap();observe(w,s,target,0,1);settle(w,s);assert!(s.objective_notices.sequence>2);
            s.phase=MobaMatchPhase::Finished {winner:None,tick:99};settle(w,s);assert_eq!(s.objective_notices.values(s),[0;4]);assert!(s.objective_notices.queue.is_empty());});
    }
    #[test]
    fn building_roles_and_epic_credit_exclude_ordinary_anonymous_allied_and_forced_death() {
        for (role,kind) in [(LaneRole::Tower,1),(LaneRole::Inhibitor,2),(LaneRole::NexusTower,3),(LaneRole::Base,0),(LaneRole::Creep,0)] {
            let mut w=fixture();let (source,target)={let mut s=w.write_resource::<MobaMatch>();let target=s.towers[1].unwrap();s.units.get_mut(&entity_key(target)).unwrap().role=role;(s.heroes[0].entity.unwrap(),target)};
            record_moba_building_damage(&w,Some(source),target,false);assert!(w.read_resource::<MobaMatch>().objective_notices.pending.is_empty());
            record_moba_building_damage(&w,Some(source),target,true);record_moba_building_damage(&w,Some(source),target,true);
            update(&mut w,|w,s|{settle(w,s);assert_eq!(s.objective_notices.values(s)[1],kind);});
        }
        for mode in 0..5 {
            let mut w=fixture();let (source,target)={let s=w.read_resource::<MobaMatch>();(s.heroes[if mode==1 {1}else{0}].entity.unwrap(),s.towers[1].unwrap())};
            if mode==2 {w.write_resource::<GamePause>().is_paused=true;}
            if mode==3 {w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Warmup;}
            if mode==4 {w.write_resource::<Vec<Outcome>>().push(Outcome::Death {ent:target,pos:Vec2::ZERO});process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();}
            else {record_moba_building_damage(&w,if mode==0 {None}else{Some(source)},target,true);}
            assert!(w.read_resource::<MobaMatch>().objective_notices.pending.is_empty());
        }
        let mut w=fixture();let (source,epic,ordinary)={let s=w.read_resource::<MobaMatch>();(s.heroes[0].entity.unwrap(),
            s.jungle_camps.iter().find(|c|c.definition.objective_rewards().next().is_some()).unwrap().entity.unwrap(),
            s.jungle_camps.iter().find(|c|c.definition.objective_rewards().next().is_none()).unwrap().entity.unwrap())};
        lethal(&mut w,Some(source),ordinary);assert!(w.read_resource::<MobaMatch>().objective_notices.pending.is_empty());
        lethal(&mut w,Some(source),epic);lethal(&mut w,Some(source),epic);
        update(&mut w,|w,s|{assert_eq!(s.objective_notices.pending.len(),1);settle(w,s);assert_eq!(s.objective_notices.values(s),[1,4,1,4096]);});
    }
}
