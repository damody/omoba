//! One canonical first-hero-kill milestone, not outcome iteration order.
use super::*;
pub const FIRST_KILL_KILLER_METRIC_ID:u64=0x4d464b494c4c3031;
pub const FIRST_KILL_VICTIM_METRIC_ID:u64=0x4d46564943543031;
pub const FIRST_KILL_GOLD_METRIC_ID:u64=0x4d46474f4c443031;
const NOTICE_DURATION:Fixed64=Fixed64::from_raw(5*1024);

#[derive(Clone, Debug, PartialEq, Eq)]
struct Claim {tick:u64,target:u64,killer:u32,victim:u32,gold:u32}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct FirstKillState {pending:Option<Claim>,settled:Option<Claim>,until:Fixed64}
impl FirstKillState {
    pub(super) fn observe(&mut self,tick:u64,target:u64,killer:u32,victim:u32,gold:u32){
        if gold==0 || self.settled.is_some(){return;}
        if self.pending.as_ref().is_none_or(|c|(tick,target)<(c.tick,c.target)){
            self.pending=Some(Claim {tick,target,killer,victim,gold});
        }
    }
    fn values(&self,state:&MobaMatch)->[u32;3]{
        if state.phase!=MobaMatchPhase::Playing || state.elapsed>=self.until{return [0;3];}
        self.settled.as_ref().map_or([0;3],|c|[c.killer,c.victim,c.gold])
    }
}
pub(super) fn settle(world:&World,state:&mut MobaMatch){
    if state.phase!=MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused || state.first_kill.settled.is_some(){return;}
    let Some(mut claim)=state.first_kill.pending.take()else{return;};
    let recipient=state.heroes.iter().position(|slot|slot.player_id==claim.killer);
    let paid=recipient.is_some_and(|index|credit_moba_player(world,state,index,claim.gold,0));
    if !paid{claim.gold=0;}
    state.first_kill.until=state.elapsed+NOTICE_DURATION;
    state.first_kill.settled=Some(claim);
}
pub(super) fn emit(world:&World,state:&MobaMatch,phase:crate::runtime::FactPhase){
    for (index,(id,value)) in [FIRST_KILL_KILLER_METRIC_ID,FIRST_KILL_VICTIM_METRIC_ID,FIRST_KILL_GOLD_METRIC_ID]
        .into_iter().zip(state.first_kill.values(state)).enumerate(){
        world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(crate::runtime::OrderedFact {
            key:crate::runtime::FactOrderingKey {tick:world.read_resource::<Tick>().0,phase,canonical_source_order:0,
                local_ordinal:160+index as u32,fact_kind:crate::runtime::FactKind::Hud},
            audience:crate::runtime::FactAudience::AllPlayers,
            fact:crate::runtime::ObservableFact::Hud {team:0,metric_id:id,value:i64::from(value)},
        }).expect("valid public first kill notice");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{process_outcomes,RuntimeEventVecSink,TowerTemplateRegistry};
    use specs::Join;
    fn fixture()->World{
        let pool=crate::runtime::StateInitializer::create_thread_pool();let mut w=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        w.insert(TowerTemplateRegistry::default());
        setup_single_lane_match(&mut w,SingleLaneConfig {first_kill_bonus_gold:100,
            additional_players:vec![SingleLanePlayerConfig {player_id:3,team_id:1,hero:"training_luminary".into()},
                SingleLanePlayerConfig {player_id:4,team_id:2,hero:"training_luminary".into()}],..Default::default()}).unwrap();
        w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
        for gold in (&mut w.write_storage::<Gold>()).join(){gold.0=0;}w
    }
    fn heroes(w:&World)->Vec<Entity>{w.read_resource::<MobaMatch>().heroes.iter().map(|s|s.entity.unwrap()).collect()}
    fn lethal(source:Entity,target:Entity)->Outcome{Outcome::Damage {source,target,pos:Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,
        real:Fixed64::from_i32(100_000),damage_profile:0,predeclared:false}}
    fn flush(w:&mut World){for _ in 0..2{process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();}w.maintain();}
    fn respawn(w:&mut World,index:usize){let mut s=w.remove::<MobaMatch>().unwrap();s.spawn_hero(w,index);w.insert(s);}
    #[test]
    fn canonical_candidates_ignore_order_and_keep_earliest_tick(){
        for reverse in [false,true]{let mut s=FirstKillState::default();let mut rows=[(7,20,1,2),(7,10,3,4)];if reverse{rows.reverse();}
            for (tick,key,killer,victim)in rows{s.observe(tick,key,killer,victim,100);}
            s.observe(8,1,1,2,100);let c=s.pending.unwrap();assert_eq!((c.tick,c.target,c.killer),(7,10,3));}
    }
    #[test]
    fn actual_same_tick_hero_kills_pay_one_canonical_bonus_and_never_repeat(){
        for reverse in [false,true]{let mut w=fixture();let h=heroes(&w);let mut hits=vec![lethal(h[0],h[1]),lethal(h[2],h[3])];if reverse{hits.reverse();}
            w.write_resource::<Vec<Outcome>>().extend(hits);flush(&mut w);
            let winner=if entity_key(h[1])<entity_key(h[3]){0}else{2};let digest=single_lane_replay_digest(&w);
            finish_moba_match_tick(&mut w);assert_ne!(single_lane_replay_digest(&w),digest);
            for index in [0,2]{assert_eq!(w.read_storage::<Gold>().get(h[index]).unwrap().0,if index==winner{400}else{300});}
            finish_moba_match_tick(&mut w);assert_eq!(w.read_storage::<Gold>().get(h[winner]).unwrap().0,400);
        }
    }
    #[test]
    fn earned_bonus_uses_dead_wallet_pause_freezes_and_notice_expires_without_resetting_claim(){
        let mut w=fixture();let h=heroes(&w);w.write_resource::<Vec<Outcome>>().push(lethal(h[0],h[1]));flush(&mut w);
        record_moba_death(&mut w,h[0]);w.delete_entity(h[0]).unwrap();w.maintain();
        w.write_resource::<GamePause>().is_paused=true;finish_moba_match_tick(&mut w);
        assert_eq!(w.read_resource::<MobaMatch>().heroes[0].gold.0,300);
        w.write_resource::<GamePause>().is_paused=false;finish_moba_match_tick(&mut w);
        {let s=w.read_resource::<MobaMatch>();assert_eq!(s.heroes[0].gold.0,400);assert_eq!(s.first_kill.values(&s),[1,2,100]);}
        respawn(&mut w,0);let next=w.read_resource::<MobaMatch>().heroes[0].entity.unwrap();assert_eq!(w.read_storage::<Gold>().get(next).unwrap().0,400);
        {let mut s=w.write_resource::<MobaMatch>();s.elapsed+=NOTICE_DURATION;assert_eq!(s.first_kill.values(&s),[0;3]);}
        respawn(&mut w,1);let target=w.read_resource::<MobaMatch>().heroes[1].entity.unwrap();
        w.write_resource::<Vec<Outcome>>().push(lethal(next,target));flush(&mut w);finish_moba_match_tick(&mut w);
        assert_eq!(w.read_storage::<Gold>().get(next).unwrap().0,700);assert_eq!(w.read_resource::<MobaMatch>().first_kill.settled.as_ref().unwrap().victim,2);
    }
    #[test]
    fn uncredited_and_forced_deaths_do_not_consume_first_credited_kill_and_zero_disables(){
        for forced in [false,true]{for disabled in [false,true]{let mut w=fixture();let h=heroes(&w);
            if disabled{w.write_resource::<MobaMatch>().config.first_kill_bonus_gold=0;}
            w.write_resource::<Vec<Outcome>>().push(if forced{Outcome::Death {ent:h[1],pos:Vec2::ZERO}}
                else{Outcome::ScriptDirectDamage {target:h[1],amount:Fixed64::from_i32(100_000)}});flush(&mut w);
            finish_moba_match_tick(&mut w);assert!(w.read_resource::<MobaMatch>().first_kill.settled.is_none());
            respawn(&mut w,1);let target=w.read_resource::<MobaMatch>().heroes[1].entity.unwrap();
            w.write_resource::<Vec<Outcome>>().push(lethal(h[0],target));flush(&mut w);finish_moba_match_tick(&mut w);
            assert_eq!(w.read_storage::<Gold>().get(h[0]).unwrap().0,if disabled{300}else{400});
        }}
        let pool=crate::runtime::StateInitializer::create_thread_pool();let mut invalid=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        assert!(setup_single_lane_match(&mut invalid,SingleLaneConfig {first_kill_bonus_gold:1_000_001,..Default::default()}).is_err());
    }
}
