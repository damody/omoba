//! Player-held camp leases; kill transfer preserves the earned deadline.
use super::*;
use omoba_template_ids::MobaObjectiveBuffConst;

#[derive(Clone,Debug)]
pub(super) struct PersonalBuffLease {
    expires:Fixed64,
    rule:MobaObjectiveBuffConst,
}
fn living(world:&World,heroes:&[MobaHeroSlot],player:u32)->bool {
    heroes.iter().find(|s|s.player_id==player).is_some_and(|s|!s.lethal_pending
        && s.entity.is_some_and(|e|world.entities().is_alive(e)
            && world.read_storage::<CProperty>().get(e).is_some_and(|p|p.hp>Fixed64::ZERO)))
}
pub(super) fn grant(world:&World,state:&mut MobaMatch,player:u32,rule:MobaObjectiveBuffConst) {
    if state.phase!=MobaMatchPhase::Playing || state.elapsed<Fixed64::ZERO || world.read_resource::<GamePause>().is_paused
        || !state.heroes.iter().any(|s|s.player_id==player) || rule.max_stacks!=1 || rule.minion_aura.is_some()
        || (rule.transfer_on_kill && !rule.lost_on_death) || !(1..=3600).contains(&rule.duration_seconds)
        || !(0..=1000).contains(&rule.attack_bonus) || !(0..=1000).contains(&rule.armor_bonus)
        || (rule.attack_bonus==0 && rule.armor_bonus==0)
        || (rule.lost_on_death && !living(world,&state.heroes,player)) {return;}
    let key=(player,rule.buff_id);
    if !state.personal_buffs.contains_key(&key) && state.personal_buffs.len()>=320 {return;}
    state.personal_buffs.insert(key,PersonalBuffLease {rule,
        expires:Fixed64::from_raw(state.elapsed.raw().saturating_add(i64::from(rule.duration_seconds)*1024))});
    synchronize(world,state,Fixed64::ZERO);
}
pub(super) fn transfer(world:&World,state:&mut MobaMatch,killer:usize,victim:usize) {
    if state.phase!=MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused
        || killer==victim || state.heroes[killer].side==state.heroes[victim].side {return;}
    let receiver=state.heroes[killer].player_id;let owner=state.heroes[victim].player_id;
    let claims:Vec<_>=state.personal_buffs.iter().filter(|((p,_),lease)|*p==owner
        && lease.rule.transfer_on_kill && lease.expires>state.elapsed).map(|((_,id),lease)|(*id,lease.clone())).collect();
    for (id,claim) in claims {
        state.personal_buffs.remove(&(owner,id));
        if let Some(entity)=state.heroes[victim].entity {world.write_resource::<crate::runtime::BuffStore>().remove(entity,id);}
        if !living(world,&state.heroes,receiver) {continue;}
        let key=(receiver,id);
        if let Some(old)=state.personal_buffs.get_mut(&key) {
            if old.rule==claim.rule {old.expires=old.expires.max(claim.expires);}
        } else if state.personal_buffs.len()<320 {state.personal_buffs.insert(key,claim);}
    }
    synchronize(world,state,Fixed64::ZERO);
}
pub(super) fn retire_hero(state:&mut MobaMatch,player:u32) {
    state.personal_buffs.retain(|(p,_),lease|*p!=player || !lease.rule.lost_on_death);
}
pub(super) fn synchronize(world:&World,state:&mut MobaMatch,pending_tick:Fixed64) {
    let terminal=matches!(state.phase,MobaMatchPhase::Finished {..});
    let mut buffs=world.write_resource::<crate::runtime::BuffStore>();
    state.personal_buffs.retain(|(player,id),lease| {
        let expired=terminal || state.elapsed>=lease.expires;
        let slot=state.heroes.iter().find(|s|s.player_id==*player);
        if let Some(entity)=slot.and_then(|s|s.entity) {
            if expired || !living(world,&state.heroes,*player) {buffs.remove(entity,id);}
            else {
                let desired=Fixed64::from_raw((lease.expires-state.elapsed).raw().saturating_add(pending_tick.raw().max(0)));
                // Camp refresh and transfer may change the deadline of an existing entry.
                if buffs.iter_for(entity).find(|(name,_)|*name==*id).is_none_or(|(_,b)|b.remaining!=desired
                    || b.payload["baseattack_bonus_damage"]!=i64::from(lease.rule.attack_bonus)*1024
                    || b.payload["physical_armor_bonus"]!=i64::from(lease.rule.armor_bonus)*1024) {
                    buffs.remove(entity,id);buffs.add(entity,id,desired,serde_json::json!({
                        "baseattack_bonus_damage":i64::from(lease.rule.attack_bonus)*1024,
                        "physical_armor_bonus":i64::from(lease.rule.armor_bonus)*1024,"__aggregation_family":id}));
                }
            }
        }
        !expired && slot.is_some()
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{process_outcomes,RuntimeEventVecSink,TowerTemplateRegistry,BuffStore};
    fn fixture()->World {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut w=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        w.insert(TowerTemplateRegistry::default());
        let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        setup_single_lane_match(&mut w,SingleLaneConfig {map_id:Some(map.id.into()),lane_length:Fixed64::from_i32(map.lane_length),
            additional_players:vec![SingleLanePlayerConfig {player_id:3,team_id:1,hero:"training_luminary".into()}],..Default::default()}).unwrap();
        w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;w
    }
    fn hero(w:&World,i:usize)->Entity {w.read_resource::<MobaMatch>().heroes[i].entity.unwrap()}
    fn rule(w:&World)->MobaObjectiveBuffConst {w.read_resource::<MobaMatch>().jungle_camps.iter().find_map(|c|c.definition.personal_buff).unwrap()}
    fn flush(w:&mut World) {for _ in 0..2 {process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();}w.maintain();}
    fn hit(w:&mut World,source:Entity,target:Entity) {
        w.write_resource::<Vec<Outcome>>().push(Outcome::Damage {source,target,pos:Vec2::ZERO,phys:Fixed64::ZERO,
            magi:Fixed64::ZERO,real:Fixed64::from_i32(100_000),damage_profile:0,predeclared:false});flush(w);
    }
    #[test]
    fn real_camp_holder_transfers_remaining_deadline_once_and_not_to_teammates() {
        let mut w=fixture();let owner=hero(&w,0);let killer=hero(&w,1);let ally=hero(&w,2);let r=rule(&w);
        let camp=w.read_resource::<MobaMatch>().jungle_camps.iter().find(|c|c.definition.personal_buff.is_some()).unwrap().entity.unwrap();
        hit(&mut w,owner,camp);
        assert!(w.read_resource::<BuffStore>().has(owner,r.buff_id));assert!(!w.read_resource::<BuffStore>().has(ally,r.buff_id));
        w.write_resource::<MobaMatch>().elapsed=Fixed64::from_i32(60);hit(&mut w,killer,owner);
        record_moba_hero_damage(&w,Some(killer),owner,true);
        assert!(!w.read_resource::<BuffStore>().has(owner,r.buff_id));
        let mut state=w.remove::<MobaMatch>().unwrap();
        assert!(!state.personal_buffs.contains_key(&(1,r.buff_id)));
        assert_eq!(state.personal_buffs[&(2,r.buff_id)].expires,Fixed64::from_i32(120));
        assert_eq!(w.read_resource::<BuffStore>().iter_for(killer).find(|(id,_)|*id==r.buff_id).unwrap().1.remaining,Fixed64::from_i32(60));
        state.spawn_hero(&mut w,0);assert!(!w.read_resource::<BuffStore>().has(state.heroes[0].entity.unwrap(),r.buff_id));
        state.elapsed=Fixed64::from_i32(120);synchronize(&w,&mut state,Fixed64::ZERO);
        assert!(state.personal_buffs.is_empty());assert!(!w.read_resource::<BuffStore>().has(killer,r.buff_id));w.insert(state);
    }
    #[test]
    fn summon_credit_merges_existing_deadline_without_full_refresh() {
        let mut w=fixture();let owner=hero(&w,0);let killer=hero(&w,1);let r=rule(&w);
        {let mut s=w.write_resource::<MobaMatch>();grant(&w,&mut s,1,r);s.elapsed=Fixed64::from_i32(30);grant(&w,&mut s,2,r);s.elapsed=Fixed64::from_i32(60);}
        let summon=w.create_entity().with(SummonedUnit::new(killer,None,0.0)).with(Faction::new(FactionType::Player,2)).build();
        hit(&mut w,summon,owner);
        assert_eq!(w.read_resource::<MobaMatch>().personal_buffs[&(2,r.buff_id)].expires,Fixed64::from_i32(150));
        assert_eq!(w.read_resource::<BuffStore>().iter_for(killer).filter(|(id,_)|*id==r.buff_id).count(),1);
        assert_eq!(w.read_resource::<BuffStore>().iter_for(killer).find(|(id,_)|*id==r.buff_id).unwrap().1.remaining,Fixed64::from_i32(90));
    }
    #[test]
    fn anonymous_forced_death_and_dead_receiver_do_not_pass_buff() {
        for mode in 0..3 {
            let mut w=fixture();let owner=hero(&w,0);let killer=hero(&w,1);let r=rule(&w);
            {let mut s=w.write_resource::<MobaMatch>();grant(&w,&mut s,1,r);}
            match mode {
                0=>w.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage {target:owner,amount:Fixed64::from_i32(100_000)}),
                1=>w.write_resource::<Vec<Outcome>>().push(Outcome::Death {ent:owner,pos:Vec2::ZERO}),
                _=>{w.write_resource::<MobaMatch>().heroes[1].lethal_pending=true;hit(&mut w,killer,owner);}
            }
            flush(&mut w);
            if mode==0 {
                assert!(w.read_storage::<CProperty>().get(owner).unwrap().hp<=Fixed64::ZERO);
                // ScriptDirectDamage only changes HP; the normal next-step death system retires it.
                record_moba_death(&mut w,owner);
            }
            assert!(w.read_resource::<MobaMatch>().personal_buffs.is_empty(),"mode {mode}");
            assert!(!w.read_resource::<BuffStore>().has(killer,r.buff_id));
        }
    }
    #[test]
    fn policy_pause_phase_persistent_respawn_digest_and_finish_are_explicit() {
        let mut w=fixture();let owner=hero(&w,0);let mut r=rule(&w);
        let before=single_lane_replay_digest(&w);
        {let mut s=w.write_resource::<MobaMatch>();grant(&w,&mut s,1,r);}
        assert_ne!(single_lane_replay_digest(&w),before);
        w.write_resource::<GamePause>().is_paused=true;
        {let mut s=w.write_resource::<MobaMatch>();transfer(&w,&mut s,1,0);assert!(s.personal_buffs.contains_key(&(1,r.buff_id)));}
        w.write_resource::<GamePause>().is_paused=false;
        {let mut s=w.write_resource::<MobaMatch>();s.phase=MobaMatchPhase::Warmup;transfer(&w,&mut s,1,0);
            assert!(s.personal_buffs.contains_key(&(1,r.buff_id)));s.phase=MobaMatchPhase::Playing;
            transfer(&w,&mut s,2,0);assert!(s.personal_buffs.contains_key(&(1,r.buff_id)));}
        r.transfer_on_kill=false;r.lost_on_death=false;
        {let mut s=w.write_resource::<MobaMatch>();grant(&w,&mut s,1,r);}
        record_moba_death(&mut w,owner);w.maintain();
        let mut s=w.remove::<MobaMatch>().unwrap();s.elapsed=Fixed64::from_i32(60);s.spawn_hero(&mut w,0);
        let next=s.heroes[0].entity.unwrap();synchronize(&w,&mut s,Fixed64::ZERO);
        assert!(w.read_resource::<BuffStore>().has(next,r.buff_id));
        s.phase=MobaMatchPhase::Finished {winner:Some(0),tick:1};synchronize(&w,&mut s,Fixed64::ZERO);
        assert!(s.personal_buffs.is_empty());assert!(!w.read_resource::<BuffStore>().has(next,r.buff_id));w.insert(s);
    }
}
