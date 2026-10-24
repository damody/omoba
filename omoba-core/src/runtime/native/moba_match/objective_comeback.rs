//! Earned-income ledger and canonical, bounded team objective compensation.
use super::*;
use crate::runtime::BuffStore;
use crate::runtime::ability_runtime::buff_store::PERMANENT_BUFF_DURATION;
const STATUS:&str="objective_comeback";

#[derive(Clone,Debug,Default)]
pub(super) struct ComebackState {
    side:Option<usize>,
    remaining:u32,
    // Sorted life identity makes a limited pool independent of outcome iteration.
    pending:BTreeMap<(u64,u64),(usize,u32)>,
}

impl MobaMatch {
    pub fn team_earned_gold(&self)->[u64;2] {self.earned_gold}
    pub fn objective_comeback(&self)->Option<(usize,u32)> {
        (self.phase==MobaMatchPhase::Playing && self.comeback.remaining>0)
            .then_some(self.comeback.side).flatten().map(|side|(side,self.comeback.remaining))
    }
}

pub(super) fn begin(world:&World,state:&mut MobaMatch) {
    if world.read_resource::<GamePause>().is_paused {return;}
    let rule=state.config.objective_comeback_rules;
    if !rule.is_valid() || rule.activate_gap==0 || state.phase!=MobaMatchPhase::Playing
        || i128::from(state.elapsed.raw())-i128::from(state.config.warmup.raw())<i128::from(rule.starts_seconds)*1024 {
        state.comeback=ComebackState::default();
    } else {
        if state.comeback.side.is_some_and(|side|
            state.earned_gold[1-side].saturating_sub(state.earned_gold[side])<=u64::from(rule.release_gap)) {
            state.comeback.side=None;state.comeback.remaining=0;
        }
        if state.comeback.side.is_none() {
            let side=if state.earned_gold[0]<state.earned_gold[1] {0}else{1};
            if state.earned_gold[1-side].saturating_sub(state.earned_gold[side])>=u64::from(rule.activate_gap) {
                state.comeback.side=Some(side);state.comeback.remaining=rule.activation_pool;
            }
        }
    }
    synchronize(world,state);
}

fn synchronize(world:&World,state:&MobaMatch) {
    let Some(mut buffs)=world.try_fetch_mut::<BuffStore>() else {return;};
    let health=world.read_storage::<CProperty>();
    for slot in &state.heroes {
        let Some(entity)=slot.entity else {continue;};
        let active=state.objective_comeback().is_some_and(|(side,_)|side==slot.side)
            && !slot.lethal_pending && world.entities().is_alive(entity)
            && health.get(entity).is_some_and(|p|p.hp>Fixed64::ZERO);
        if active {
            if !buffs.has(entity,STATUS) {buffs.add(entity,STATUS,PERMANENT_BUFF_DURATION,serde_json::json!({}));}
        } else {buffs.remove(entity,STATUS);}
    }
}

pub(super) fn observe(world:&World,state:&mut MobaMatch,target:Entity,side:usize,epic:bool) {
    if state.phase!=MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused
        || state.objective_comeback().is_none_or(|(eligible,_)|eligible!=side) {return;}
    let rules=state.config.objective_comeback_rules;
    let amount=if epic {rules.epic_gold_per_player}else{rules.tower_gold_per_player};
    if amount>0 {state.comeback.pending.entry((world.read_resource::<Tick>().0,entity_key(target))).or_insert((side,amount));}
}

pub(super) fn settle(world:&World,state:&mut MobaMatch) {
    if world.read_resource::<GamePause>().is_paused {return;}
    if state.phase!=MobaMatchPhase::Playing {state.comeback=ComebackState::default();synchronize(world,state);return;}
    for (_, (side,per_player)) in std::mem::take(&mut state.comeback.pending) {
        if state.comeback.side!=Some(side) {continue;}
        let recipients:Vec<_>=state.heroes.iter().enumerate().filter(|(_,s)|s.side==side)
            .filter(|(_,s)|s.entity.is_some_and(|e|world.entities().is_alive(e) && world.read_storage::<Gold>().get(e).is_some())
                || (s.entity.is_none() && s.admitted_dead_wallet_ref().is_some()))
            .map(|(i,_)|i).collect();
        if recipients.is_empty() {continue;}
        let share=per_player.min(state.comeback.remaining/recipients.len() as u32);
        if share==0 {state.comeback.remaining=0;continue;}
        for index in recipients {
            if credit_moba_player(world,state,index,share,0) {
                state.comeback.remaining=state.comeback.remaining.saturating_sub(share);
            }
        }
    }
    synchronize(world,state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{TowerTemplateRegistry,process_outcomes,RuntimeEventVecSink};
    fn rules()->ObjectiveComebackRules {ObjectiveComebackRules {starts_seconds:1,activate_gap:500,release_gap:200,
        tower_gold_per_player:100,epic_gold_per_player:125,activation_pool:350}}
    fn fixture()->World {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut w=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        w.insert(TowerTemplateRegistry::default());
        setup_single_lane_match(&mut w,SingleLaneConfig {warmup:Fixed64::ZERO,passive_gold_per_second:0,
            objective_comeback_rules:rules(),map_id:Some("three_lane_training".into()),
            lane_length:Fixed64::from_i32(omoba_template_ids::moba_map_by_name("three_lane_training").unwrap().lane_length),
            additional_players:vec![SingleLanePlayerConfig {player_id:3,team_id:1,hero:"training_luminary".into()}],
            ..Default::default()}).unwrap();
        {let mut s=w.write_resource::<MobaMatch>();s.phase=MobaMatchPhase::Playing;s.elapsed=Fixed64::from_i32(1);}
        w
    }
    fn update(w:&mut World,f:impl FnOnce(&World,&mut MobaMatch)) {let mut s=w.remove::<MobaMatch>().unwrap();f(w,&mut s);w.insert(s);}
    fn flush(w:&mut World,source:Option<Entity>,target:Entity) {
        w.write_resource::<Vec<Outcome>>().push(source.map_or(
            Outcome::ScriptDirectDamage {target,amount:Fixed64::from_i32(100_000)},|source|
            Outcome::Damage {source,target,pos:Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,
                real:Fixed64::from_i32(100_000),damage_profile:0,predeclared:false}));
        process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();
        process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();
    }
    #[test]
    fn earned_ledger_ignores_wallet_spending_and_survives_death_saturation_and_respawn() {
        let mut w=fixture();let e=w.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
        update(&mut w,|w,s|{assert!(credit_moba_player(w,s,0,100,0));assert_eq!(s.team_earned_gold(),[100,0]);});
        w.write_resource::<Vec<Outcome>>().extend([Outcome::GainGold {target:e,amount:40},Outcome::GainGold {target:e,amount:-40}]);
        process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
        assert_eq!(w.read_resource::<MobaMatch>().team_earned_gold(),[140,0]);
        w.write_storage::<Gold>().get_mut(e).unwrap().0=0;
        update(&mut w,|w,s|{assert_eq!(s.team_earned_gold(),[140,0]);
            w.write_storage::<Gold>().get_mut(e).unwrap().0=i32::MAX-5;
            credit_moba_player(w,s,0,100,0);assert_eq!(s.earned_gold,[145,0]);});
        record_moba_death(&mut w,e);
        update(&mut w,|w,s|{credit_moba_player(w,s,0,100,0);assert_eq!(s.earned_gold,[145,0],"full dead wallet does not inflate income");
            s.heroes[0].gold.0=0;assert!(credit_moba_player(w,s,0,100,0));assert_eq!(s.earned_gold,[245,0]);});
        let mut s=w.remove::<MobaMatch>().unwrap();s.spawn_hero(&mut w,0);assert_eq!(s.earned_gold,[245,0]);w.insert(s);
    }
    #[test]
    fn elapsed_hysteresis_pause_release_exhaustion_and_terminal_status_are_authoritative() {
        let mut w=fixture();update(&mut w,|w,s|{s.earned_gold=[0,500];s.elapsed=Fixed64::ZERO;begin(w,s);assert_eq!(s.objective_comeback(),None);
            s.elapsed=Fixed64::ONE;begin(w,s);assert_eq!(s.objective_comeback(),Some((0,350)));
            s.earned_gold=[201,500];begin(w,s);assert_eq!(s.objective_comeback(),Some((0,350)));
            s.comeback.remaining=0;begin(w,s);assert_eq!(s.objective_comeback(),None);assert_eq!(s.comeback.side,Some(0));});
        w.write_resource::<GamePause>().is_paused=true;
        update(&mut w,|w,s|{s.earned_gold=[500,500];begin(w,s);assert_eq!(s.comeback.side,Some(0));});
        w.write_resource::<GamePause>().is_paused=false;
        update(&mut w,|w,s|{begin(w,s);assert_eq!(s.comeback.side,None);s.earned_gold=[500,1000];begin(w,s);
            assert_eq!(s.objective_comeback(),Some((0,350)));
            let hero=s.heroes[0].entity.unwrap();assert!(w.read_resource::<BuffStore>().has(hero,STATUS));
            s.phase=MobaMatchPhase::Finished {winner:None,tick:10};begin(w,s);assert_eq!(s.objective_comeback(),None);
            assert!(!w.read_resource::<BuffStore>().has(hero,STATUS));});
    }
    #[test]
    fn canonical_pool_is_shared_with_dead_wallet_once_per_objective_and_not_refilled() {
        for reverse in [false,true] {
            let mut w=fixture();let ally=w.read_resource::<MobaMatch>().heroes[2].entity.unwrap();record_moba_death(&mut w,ally);
            update(&mut w,|w,s|{s.earned_gold=[0,1000];begin(w,s);let mut targets=s.lane_towers.iter().map(|p|p[1].unwrap()).collect::<Vec<_>>();
                if reverse {targets.reverse();}
                for target in targets {observe(w,s,target,0,false);observe(w,s,target,0,false);}
                assert_eq!(s.comeback.pending.len(),3);settle(w,s);assert_eq!(s.earned_gold,[350,1000]);assert_eq!(s.comeback.remaining,0);
                assert_eq!(s.heroes[2].gold.0,175);let e=s.heroes[0].entity.unwrap();assert_eq!(w.read_storage::<Gold>().get(e).unwrap().0,175);
                begin(w,s);assert_eq!(s.comeback.remaining,0);settle(w,s);assert_eq!(s.earned_gold,[350,1000]);});
        }
    }
    #[test]
    fn committed_tower_and_epic_credit_bonus_but_anonymous_forced_and_duplicate_paths_do_not() {
        let mut w=fixture();update(&mut w,|w,s|{s.earned_gold=[0,1000];begin(w,s);});
        let (source,tower,epic,ordinary)={let s=w.read_resource::<MobaMatch>();(s.heroes[0].entity.unwrap(),s.lane_towers[0][1].unwrap(),
            s.jungle_camps.iter().find(|c|c.definition.objective_rewards().next().is_some()).unwrap().entity.unwrap(),
            s.jungle_camps.iter().find(|c|c.definition.objective_rewards().next().is_none()).unwrap().entity.unwrap())};
        flush(&mut w,Some(source),tower);flush(&mut w,Some(source),tower);
        update(&mut w,|w,s|{assert_eq!(s.comeback.pending.len(),1);settle(w,s);assert_eq!(s.earned_gold[0],200);});
        flush(&mut w,Some(source),ordinary);
        update(&mut w,|_,s|assert!(s.comeback.pending.is_empty()));
        flush(&mut w,Some(source),epic);flush(&mut w,Some(source),epic);
        update(&mut w,|w,s|{assert_eq!(s.comeback.pending.len(),1);settle(w,s);assert_eq!(s.comeback.remaining,0);});
        let mut w=fixture();update(&mut w,|w,s|{s.earned_gold=[0,1000];begin(w,s);});
        let target=w.read_resource::<MobaMatch>().towers[1].unwrap();flush(&mut w,None,target);
        update(&mut w,|_,s|assert!(s.comeback.pending.is_empty()));
        let target=w.read_resource::<MobaMatch>().jungle_camps.iter().find(|c|c.definition.objective_rewards().next().is_some()).unwrap().entity.unwrap();
        w.write_resource::<Vec<Outcome>>().push(Outcome::Death {ent:target,pos:Vec2::ZERO});
        process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
        update(&mut w,|_,s|assert!(s.comeback.pending.is_empty()));
    }
}
