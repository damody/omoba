//! Bounded team leases. Actual modifiers live in the normal BuffStore.
use super::*;
use crate::runtime::BuffStore;
use omoba_template_ids::MobaObjectiveBuffConst;

#[derive(Clone, Debug)]
pub(super) struct TeamObjectiveLease {
    expires:Option<Fixed64>,
    stacks:u32,
    rule:MobaObjectiveBuffConst,
    eligible:BTreeSet<u32>,
}

fn live(world:&World,slot:&MobaHeroSlot)->bool {
    !slot.lethal_pending && slot.entity.is_some_and(|entity|world.entities().is_alive(entity)
        && world.read_storage::<CProperty>().get(entity).is_some_and(|p|p.hp>Fixed64::ZERO))
}

#[cfg(test)]
pub(super) fn grant(world:&World,state:&mut MobaMatch,rule:MobaObjectiveBuffConst,killer:u32) {
    let Some(side)=state.heroes.iter().find(|slot|slot.player_id==killer).map(|slot|slot.side) else {return};
    grant_side(world,state,rule,side);
}

pub(super) fn grant_side(world:&World,state:&mut MobaMatch,rule:MobaObjectiveBuffConst,side:usize) {
    if state.phase!=MobaMatchPhase::Playing || state.elapsed<Fixed64::ZERO
        || side>=state.config.teams.len() || !(1..=16).contains(&rule.max_stacks)
        || !(0..=3600).contains(&rule.duration_seconds) || rule.transfer_on_kill
        || (rule.duration_seconds==0 && rule.lost_on_death)
        || rule.minion_aura.is_some_and(|a|a.buff_id.is_empty() || a.buff_id==rule.buff_id || !(1..=3000).contains(&a.radius)
            || !(0..=1000).contains(&a.attack_bonus) || !(0..=1000).contains(&a.armor_bonus) || (a.attack_bonus==0 && a.armor_bonus==0))
        || world.read_resource::<GamePause>().is_paused {return;}
    let eligible=state.heroes.iter().filter(|slot|slot.side==side && (!rule.lost_on_death || live(world,slot)))
        .map(|slot|slot.player_id).collect();
    // At most two teams times the map's bounded objective-bearing camps.
    let key=(side,rule.buff_id);
    if !state.objective_buffs.contains_key(&key) && state.objective_buffs.len()>=64 {return;}
    let stacks=state.objective_buffs.get(&key)
        .filter(|lease|lease.expires.is_none_or(|at|at>state.elapsed) && lease.rule==rule)
        .map_or(1,|lease|lease.stacks.saturating_add(1).min(rule.max_stacks));
    {
        let mut buffs=world.write_resource::<BuffStore>();
        for slot in state.heroes.iter().filter(|slot|slot.side==side) {
            if let Some(entity)=slot.entity {buffs.remove(entity,rule.buff_id);}
        }
    }
    state.objective_buffs.insert(key,TeamObjectiveLease {
        expires:(rule.duration_seconds>0).then(||Fixed64::from_raw(state.elapsed.raw().saturating_add(i64::from(rule.duration_seconds)*1024))),stacks,rule,eligible});
    synchronize(world,state,Fixed64::ZERO);
}

/// The lease's BuffStore payload: every stat per stack times the stacks. The E187 stats (the
/// original bosses' modifiers) appear only when set: percents become engine fractions (rounded raw),
/// magic resistance stays in percent points like the KV MAGICAL_RESISTANCE_BONUS.
pub(super) fn payload(rule:&MobaObjectiveBuffConst,stacks:u32,id:&str)->serde_json::Value {
    use omb_script_abi::stat_keys::StatKey;
    let n=i64::from(stacks);
    let mut payload=serde_json::json!({
        "baseattack_bonus_damage":i64::from(rule.attack_bonus)*n*1024,
        "physical_armor_bonus":i64::from(rule.armor_bonus)*n*1024,
        "objective_stacks":stacks,
        "stacks":stacks,
        "__aggregation_family":id});
    let s=rule.stats;
    let fraction=|percent:i32|(i64::from(percent)*1024+50)/100*n;
    let flat=|value:i32|i64::from(value)*1024*n;
    for (key,value) in [(StatKey::BaseDamageOutgoingPercentage,fraction(s.damage_percent)),
        (StatKey::SpellAmplifyPercentage,fraction(s.spell_amp_percent)),
        (StatKey::AttackSpeedBonusConstant,flat(s.attack_speed)),
        (StatKey::MoveSpeedBonusPercentage,fraction(s.move_speed_percent)),
        (StatKey::MagicalResistanceBonus,flat(s.magic_resistance)),
        (StatKey::HealthRegenMaxPercentage,fraction(s.hp_regen_max_percent)),
        (StatKey::ManaRegenConstant,flat(s.mana_regen))] {
        if value!=0 {payload[key.as_str()]=serde_json::json!(value);}
    }
    payload
}

pub(super) fn retire_hero(state:&mut MobaMatch,player:u32) {
    for lease in state.objective_buffs.values_mut() {
        if lease.rule.lost_on_death {lease.eligible.remove(&player);}
    }
}

/// `pending_tick` compensates only newly materialized respawn modifiers for
/// the normal BuffStore tick that will run later in this same simulation step.
pub(super) fn synchronize(world:&World,state:&mut MobaMatch,pending_tick:Fixed64) {
    let terminal=matches!(state.phase,MobaMatchPhase::Finished {..});
    let mut buffs=world.write_resource::<BuffStore>();
    state.objective_buffs.retain(|(side,id),lease| {
        let expired=terminal || lease.expires.is_some_and(|at|state.elapsed>=at);
        for slot in state.heroes.iter().filter(|slot|slot.side==*side) {
            let Some(entity)=slot.entity else {continue};
            if expired || !lease.eligible.contains(&slot.player_id) || !live(world,slot) {
                buffs.remove(entity,id);continue;
            }
            if !buffs.has(entity,id) {
                let remaining=lease.expires.map_or(crate::runtime::ability_runtime::buff_store::PERMANENT_BUFF_DURATION,
                    |at|Fixed64::from_raw((at-state.elapsed).raw().saturating_add(pending_tick.raw().max(0))));
                buffs.add(entity,id,remaining,payload(&lease.rule,lease.stacks,id));
            }
        }
        !expired
    });
    drop(buffs);
    minion_aura::synchronize(world,state);
}

pub(super) fn aura_emitters(world:&World,state:&MobaMatch)->Vec<(usize,omoba_template_ids::MobaMinionAuraConst,Vec<Vec2>)> {
    if state.phase!=MobaMatchPhase::Playing {return Vec::new();}
    let positions=world.read_storage::<Pos>();
    state.objective_buffs.iter().filter_map(|(&(side,_),lease)| {
        let rule=lease.rule.minion_aura?;
        if lease.expires.is_some_and(|at|at<=state.elapsed) {return None;}
        let origins:Vec<_>=state.heroes.iter().filter(|slot|slot.side==side && lease.eligible.contains(&slot.player_id) && live(world,slot))
            .filter_map(|slot|positions.get(slot.entity?).map(|p|p.0)).collect();
        (!origins.is_empty()).then_some((side,rule,origins))
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{process_outcomes,RuntimeEventVecSink,TowerTemplateRegistry};
    use crate::runtime::ability_runtime::UnitStats;

    fn fixture()->(World,Entity,Entity) {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        world.insert(TowerTemplateRegistry::default());
        let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        setup_single_lane_match(&mut world,SingleLaneConfig {map_id:Some(map.id.into()),
            lane_length:Fixed64::from_i32(map.lane_length),additional_players:vec![SingleLanePlayerConfig {
                player_id:3,team_id:1,hero:"training_luminary".into()}],..Default::default()}).unwrap();
        let (source,ally)={let mut state=world.write_resource::<MobaMatch>();state.phase=MobaMatchPhase::Playing;
            (state.heroes[0].entity.unwrap(),state.heroes[2].entity.unwrap())};
        // The bosses appear on the match clock (E187); these fixtures fight them at the start.
        spawn_waiting_jungle_camps_for_test(&mut world);
        // The stack tests respawn the stacking objective 90 s after its kill, inside its 120 s
        // training lease; the map's (original) respawn is longer.
        for camp in &mut world.write_resource::<MobaMatch>().jungle_camps {
            if camp.definition.objective_rewards().any(|rule|rule.max_stacks>1) {camp.definition.respawn_seconds=90;}
        }
        (world,source,ally)
    }
    fn target(world:&World,lost_on_death:bool)->Entity {
        world.read_resource::<MobaMatch>().jungle_camps.iter().find(|c|c.definition.objective_buff
            .is_some_and(|b|b.lost_on_death==lost_on_death)).unwrap().entity.unwrap()
    }
    fn lethal(source:Entity,target:Entity)->Outcome {
        Outcome::Damage {pos:Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,
            real:Fixed64::from_i32(100_000),source,target,damage_profile:0,predeclared:false}
    }
    fn flush(world:&mut World,events:impl IntoIterator<Item=Outcome>) {
        world.write_resource::<Vec<Outcome>>().extend(events);
        process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();
        process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();
    }
    fn sync(world:&mut World,elapsed:i32,pending:Fixed64) {
        let mut state=world.remove::<MobaMatch>().unwrap();state.elapsed=Fixed64::from_i32(elapsed);
        synchronize(world,&mut state,pending);world.insert(state);
    }
    #[test]
    fn actual_epic_death_grants_team_modifiers_once_and_refreshes_without_stacking() {
        let (mut world,source,ally)=fixture();let victim=target(&world,true);
        flush(&mut world,[lethal(source,victim),lethal(source,victim)]);
        let enemy=world.read_resource::<MobaMatch>().heroes[1].entity.unwrap();
        {
            let buffs=world.read_resource::<BuffStore>();
            assert!(buffs.has(source,"objective_might") && buffs.has(ally,"objective_might"));
            assert!(!buffs.has(enemy,"objective_might"));
            let stats=UnitStats::from_refs(&buffs,false);
            assert_eq!(stats.normal_attack_physical(Fixed64::from_i32(50),source),Fixed64::from_i32(70));
            assert_eq!(stats.final_armor(Fixed64::ZERO,source),Fixed64::from_i32(10));
        }
        assert_eq!(world.read_resource::<MobaMatch>().objective_buffs.len(),1);
        let rule=world.read_resource::<MobaMatch>().objective_buffs.values().next().unwrap().rule;
        let before=single_lane_replay_digest(&world);
        {let mut state=world.write_resource::<MobaMatch>();state.elapsed=Fixed64::from_i32(30);grant(&world,&mut state,rule,1);}
        assert_ne!(single_lane_replay_digest(&world),before);
        let buffs=world.read_resource::<BuffStore>();
        assert_eq!(buffs.iter_for(source).filter(|(id,_)|*id=="objective_might").count(),1);
        assert_eq!(buffs.iter_for(source).find(|(id,_)|*id=="objective_might").unwrap().1.remaining,Fixed64::from_i32(180));
        assert_eq!(world.read_resource::<MobaMatch>().objective_buffs.values().next().unwrap().expires,Some(Fixed64::from_i32(210)));
    }
    #[test]
    fn persistent_dead_roster_respawns_with_remaining_time_lost_on_death_never_returns() {
        for lost in [true,false] {
            let (mut world,source,ally)=fixture();let victim=target(&world,lost);
            if !lost {record_moba_death(&mut world,ally);}
            flush(&mut world,[lethal(source,victim)]);
            if lost {record_moba_death(&mut world,ally);}
            let id=if lost {"objective_might"}else{"objective_guard"};
            world.maintain();
            let mut state=world.remove::<MobaMatch>().unwrap();state.elapsed=Fixed64::from_i32(60);
            state.spawn_hero(&mut world,2);let next=state.heroes[2].entity.unwrap();
            assert_ne!(next,ally);synchronize(&world,&mut state,Fixed64::from_raw(17));world.insert(state);
            world.write_resource::<BuffStore>().tick(Fixed64::from_raw(17));
            let buffs=world.read_resource::<BuffStore>();
            assert_eq!(buffs.has(next,id),!lost);
            if !lost {assert_eq!(buffs.iter_for(next).find(|(buff,_)|*buff==id).unwrap().1.remaining,Fixed64::from_i32(60));}
        }
    }
    #[test]
    fn pause_deadline_terminal_and_fresh_world_do_not_leak_team_lease() {
        let (mut world,source,_)=fixture();let victim=target(&world,true);flush(&mut world,[lethal(source,victim)]);
        world.write_resource::<GamePause>().is_paused=true;
        world.write_resource::<DeltaTime>().0=Fixed64::from_i32(500);
        let before=world.read_resource::<MobaMatch>().elapsed;
        assert!(!begin_moba_match_tick(&mut world));assert_eq!(world.read_resource::<MobaMatch>().elapsed,before);
        assert!(world.read_resource::<BuffStore>().has(source,"objective_might"));
        world.write_resource::<GamePause>().is_paused=false;
        sync(&mut world,179,Fixed64::ZERO);assert!(world.read_resource::<BuffStore>().has(source,"objective_might"));
        sync(&mut world,180,Fixed64::ZERO);assert!(!world.read_resource::<BuffStore>().has(source,"objective_might"));
        assert!(world.read_resource::<MobaMatch>().objective_buffs.is_empty());
        let rule=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap().jungle_camps.iter().find_map(|c|c.objective_buff).unwrap();
        {let mut state=world.write_resource::<MobaMatch>();grant(&world,&mut state,rule,1);state.phase=MobaMatchPhase::Finished {winner:Some(0),tick:1};}
        assert!(!begin_moba_match_tick(&mut world));assert!(world.read_resource::<MobaMatch>().objective_buffs.is_empty());
        assert!(!world.read_resource::<BuffStore>().has(source,rule.buff_id));
        let (fresh,hero,_)=fixture();assert!(fresh.read_resource::<MobaMatch>().objective_buffs.is_empty());
        assert!(!fresh.read_resource::<BuffStore>().has(hero,rule.buff_id));
    }
    #[test]
    fn actual_epic_respawns_stack_to_authored_cap_once_per_life() {
        let (mut world,source,ally)=fixture();
        let enemy=world.read_resource::<MobaMatch>().heroes[1].entity.unwrap();
        for n in 1..=5 {
            let victim=target(&world,false);flush(&mut world,[lethal(source,victim),lethal(source,victim)]);
            let state=world.read_resource::<MobaMatch>();let lease=&state.objective_buffs[&(0,"objective_guard")];
            assert_eq!(lease.stacks,n.min(4));
            {let buffs=world.read_resource::<BuffStore>();
                for entity in [source,ally] {
                    assert_eq!(buffs.iter_for(entity).filter(|(id,_)|*id=="objective_guard").count(),1);
                    assert_eq!(UnitStats::from_refs(&buffs,false).final_armor(Fixed64::ZERO,entity),Fixed64::from_i32(15*n.min(4) as i32));
                }
                assert!(!buffs.has(enemy,"objective_guard"));}
            drop(state);
            if n<5 {let mut state=world.remove::<MobaMatch>().unwrap();state.elapsed+=Fixed64::from_i32(90);
                synchronize(&world,&mut state,Fixed64::ZERO);jungle::tick(&mut world,&mut state,Fixed64::ZERO);world.insert(state);}
        }
    }
    #[test]
    fn stacks_refresh_expire_reset_and_respect_death_eligibility() {
        let (mut world,source,ally)=fixture();
        let mut rule=world.read_resource::<MobaMatch>().jungle_camps.iter().find_map(|c|c.definition.objective_buff)
            .unwrap();rule.max_stacks=3;
        {let mut state=world.write_resource::<MobaMatch>();grant(&world,&mut state,rule,1);grant(&world,&mut state,rule,1);}
        record_moba_death(&mut world,ally);world.maintain();
        let mut state=world.remove::<MobaMatch>().unwrap();state.elapsed=Fixed64::from_i32(60);state.spawn_hero(&mut world,2);
        let next=state.heroes[2].entity.unwrap();synchronize(&world,&mut state,Fixed64::ZERO);
        assert!(!world.read_resource::<BuffStore>().has(next,rule.buff_id));
        grant(&world,&mut state,rule,1);assert_eq!(state.objective_buffs[&(0,rule.buff_id)].stacks,3);
        assert!(world.read_resource::<BuffStore>().has(next,rule.buff_id));
        assert_eq!(state.objective_buffs[&(0,rule.buff_id)].expires,Some(Fixed64::from_i32(240)));
        state.elapsed=Fixed64::from_i32(240);synchronize(&world,&mut state,Fixed64::ZERO);
        assert!(!world.read_resource::<BuffStore>().has(source,rule.buff_id));
        grant(&world,&mut state,rule,1);assert_eq!(state.objective_buffs[&(0,rule.buff_id)].stacks,1);
        rule.attack_bonus+=1;grant(&world,&mut state,rule,1);
        assert_eq!(state.objective_buffs[&(0,rule.buff_id)].stacks,1,"policy change never merges incompatible effects");
        world.insert(state);
    }
    #[test]
    fn stack_state_digest_pause_phase_invalid_limit_and_enemy_are_explicit() {
        let (world,_,_)=fixture();let mut rule=world.read_resource::<MobaMatch>().jungle_camps.iter()
            .find_map(|c|c.definition.objective_buff).unwrap();rule.max_stacks=3;
        {let mut state=world.write_resource::<MobaMatch>();grant(&world,&mut state,rule,1);}
        let before=single_lane_replay_digest(&world);
        world.write_resource::<MobaMatch>().objective_buffs.get_mut(&(0,rule.buff_id)).unwrap().stacks=2;
        assert_ne!(single_lane_replay_digest(&world),before);
        world.write_resource::<GamePause>().is_paused=true;
        {let mut state=world.write_resource::<MobaMatch>();grant(&world,&mut state,rule,1);
            assert_eq!(state.objective_buffs[&(0,rule.buff_id)].stacks,2);}
        world.write_resource::<GamePause>().is_paused=false;
        {let mut state=world.write_resource::<MobaMatch>();state.phase=MobaMatchPhase::Warmup;grant(&world,&mut state,rule,1);
            assert_eq!(state.objective_buffs[&(0,rule.buff_id)].stacks,2);state.phase=MobaMatchPhase::Playing;
            for max in [0,17] {let bad=MobaObjectiveBuffConst {max_stacks:max,..rule};grant(&world,&mut state,bad,1);}
            assert_eq!(state.objective_buffs[&(0,rule.buff_id)].stacks,2);
            grant(&world,&mut state,rule,2);assert_eq!(state.objective_buffs[&(1,rule.buff_id)].stacks,1);}
    }
    #[test]
    fn generic_stack_metadata_tracks_authority_for_timed_and_permanent_rewards() {
        let (mut world,source,ally)=fixture();
        for n in 1..=5 {
            let victim=target(&world,false);flush(&mut world,[lethal(source,victim)]);
            let buffs=world.read_resource::<BuffStore>();
            for hero in [source,ally] {
                for id in ["objective_guard","objective_legacy"] {
                    let buff=buffs.iter_for(hero).find(|(name,_)|*name==id).unwrap().1;
                    assert_eq!(buff.payload["stacks"],serde_json::json!(n.min(4)));
                    assert_eq!(buff.payload["stacks"],buff.payload["objective_stacks"]);
                }
            }
            drop(buffs);
            if n<5 {let mut state=world.remove::<MobaMatch>().unwrap();state.elapsed+=Fixed64::from_i32(90);
                jungle::tick(&mut world,&mut state,Fixed64::ZERO);world.insert(state);}
        }
    }
    #[test]
    fn actual_multiple_objective_rewards_stack_permanently_and_keep_finite_policy() {
        let (mut world,source,ally)=fixture();
        for n in 1..=5 {
            let victim=target(&world,false);flush(&mut world,[lethal(source,victim),lethal(source,victim)]);
            let s=world.read_resource::<MobaMatch>();let lease=&s.objective_buffs[&(0,"objective_legacy")];
            assert_eq!(lease.stacks,n.min(4));assert_eq!(lease.expires,None);
            let buffs=world.read_resource::<BuffStore>();
            for hero in [source,ally] {
                assert_eq!(buffs.iter_for(hero).filter(|(id,_)|*id=="objective_legacy").count(),1);
                assert_eq!(UnitStats::from_refs(&buffs,false).normal_attack_physical(Fixed64::from_i32(50),hero),Fixed64::from_i32(50+5*n.min(4) as i32));
            }
            assert!(s.objective_buffs[&(0,"objective_guard")].expires.is_some());drop(buffs);drop(s);
            if n<5 {let mut s=world.remove::<MobaMatch>().unwrap();s.elapsed+=Fixed64::from_i32(90);
                jungle::tick(&mut world,&mut s,Fixed64::ZERO);world.insert(s);}
        }
        sync(&mut world,1_000_000,Fixed64::ZERO);world.write_resource::<BuffStore>().tick(Fixed64::from_i32(1_000_000));
        let buffs=world.read_resource::<BuffStore>();assert!(buffs.has(source,"objective_legacy"));assert!(!buffs.has(source,"objective_guard"));
    }
    #[test]
    fn permanent_dead_roster_respawns_and_terminal_cleanup_is_authoritative() {
        let (mut world,source,ally)=fixture();record_moba_death(&mut world,ally);
        let target=target(&world,false);flush(&mut world,[lethal(source,target)]);world.maintain();
        let before=single_lane_replay_digest(&world);
        let mut s=world.remove::<MobaMatch>().unwrap();s.elapsed=Fixed64::from_i32(1_000_000);s.spawn_hero(&mut world,2);
        let next=s.heroes[2].entity.unwrap();synchronize(&world,&mut s,Fixed64::from_raw(17));world.insert(s);
        world.write_resource::<BuffStore>().tick(Fixed64::from_raw(17));
        assert_ne!(before,single_lane_replay_digest(&world));
        let buffs=world.read_resource::<BuffStore>();
        assert_eq!(buffs.iter_for(next).find(|(id,_)|*id=="objective_legacy").unwrap().1.remaining,
            crate::runtime::ability_runtime::buff_store::PERMANENT_BUFF_DURATION);drop(buffs);
        world.write_resource::<GamePause>().is_paused=true;let elapsed=world.read_resource::<MobaMatch>().elapsed;
        assert!(!begin_moba_match_tick(&mut world));assert_eq!(world.read_resource::<MobaMatch>().elapsed,elapsed);
        world.write_resource::<GamePause>().is_paused=false;
        let mut s=world.remove::<MobaMatch>().unwrap();s.phase=MobaMatchPhase::Finished {winner:Some(0),tick:1};
        synchronize(&world,&mut s,Fixed64::ZERO);assert!(s.objective_buffs.is_empty());world.insert(s);
        assert!(!world.read_resource::<BuffStore>().has(next,"objective_legacy"));
        assert!(fixture().0.read_resource::<MobaMatch>().objective_buffs.is_empty());
    }
    #[test]
    fn permanent_invalid_policy_is_rejected_without_changing_existing_stacks() {
        let (world,_,_)=fixture();let rule=world.read_resource::<MobaMatch>().jungle_camps.iter()
            .flat_map(|c|c.definition.additional_objective_buffs).copied().next().unwrap();
        let mut s=world.write_resource::<MobaMatch>();
        for bad in [MobaObjectiveBuffConst {lost_on_death:true,..rule},MobaObjectiveBuffConst {transfer_on_kill:true,..rule},
            MobaObjectiveBuffConst {duration_seconds:-1,..rule}] {grant_side(&world,&mut s,bad,0);}
        assert!(s.objective_buffs.is_empty());grant_side(&world,&mut s,rule,0);grant_side(&world,&mut s,rule,1);
        assert_eq!(s.objective_buffs[&(0,rule.buff_id)].stacks,1);assert_eq!(s.objective_buffs[&(1,rule.buff_id)].stacks,1);
    }
    #[test]
    fn anonymous_forced_and_member_deaths_do_not_grant_and_enemy_lease_is_independent() {
        for forced in [true,false] {
            let (mut world,source,_)=fixture();let victim=target(&world,true);
            if forced {flush(&mut world,[Outcome::Death {ent:victim,pos:Vec2::ZERO}]);}
            else {flush(&mut world,[Outcome::ScriptDirectDamage {target:victim,amount:Fixed64::from_i32(100_000)},
                Outcome::Heal {target:victim,pos:Vec2::ZERO,amount:Fixed64::from_i32(100)},lethal(source,victim)]);}
            assert!(world.read_resource::<MobaMatch>().objective_buffs.is_empty());
        }
        let (mut world,source,_)=fixture();let member=world.read_resource::<MobaMatch>().jungle_camps.iter()
            .flat_map(|c|&c.members).find_map(|m|m.entity).unwrap();
        flush(&mut world,[lethal(source,member)]);assert!(world.read_resource::<MobaMatch>().objective_buffs.is_empty());
        let victim=target(&world,true);flush(&mut world,[lethal(source,victim)]);
        let rule=world.read_resource::<MobaMatch>().objective_buffs.values().next().unwrap().rule;
        {let mut state=world.write_resource::<MobaMatch>();grant(&world,&mut state,rule,2);}
        assert_eq!(world.read_resource::<MobaMatch>().objective_buffs.len(),2);
    }
}
