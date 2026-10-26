//! Parity soak for 村上義清, carried by the package (installed test-only as
//! heroes::B29_murakami_yoshikiyo::parity_soak). Run (release):
//!   cargo test --release ... -p base_content --lib -- murakami_yoshikiyo_kit_keeps --nocapture
use crate::single_lane_match_tests::live_soak_replica_parity::{soak_match,cast,cast_at,attack_move};
use crate::single_lane_match_tests::{three_lane_config,hero_pair};
use omoba_core::runtime::*;
use omoba_sim::Fixed64;
use specs::WorldExt;

/// Self-centred slam stun, leap + slow + the 破軍靈光 armor aura and the 士氣提升
/// ally aura refreshed by the hero UnitScript tick, and the random-wave point
/// channel after its windup, aimed at the enemy hero while both walk in and out
/// of each other's vision; both team replicas are checked every tick.
#[test]
fn murakami_yoshikiyo_kit_keeps_team_replicas_identical() {
    let spawns=std::cell::Cell::new(None);
    // R has a 0.3 s windup: any new order cancels it, so the caster holds for 0.5 s after casting R.
    let hold_until=std::cell::Cell::new(0u32);
    // Counters prove the kit actually fired; parity alone would pass on an idle hero.
    let (aura_ticks,morale_ticks,stun_ticks,channel_ticks)=(std::cell::Cell::new(0u32),std::cell::Cell::new(0u32),
        std::cell::Cell::new(0u32),std::cell::Cell::new(0u32));
    let mismatches=soak_match(SingleLaneConfig {mana_enabled:true,heroes:["murakami_yoshikiyo".into(),"training_luminary".into()],..three_lane_config()},7200,1,
        &|authority|{
            for hero in hero_pair(authority) {
                authority.write_storage::<Gold>().get_mut(hero).unwrap().0=5000;
                let mut heroes=authority.write_storage::<Hero>();let h=heroes.get_mut(hero).unwrap();h.level=12;
                for id in h.abilities.clone() {h.ability_levels.insert(id,1);}
            }
            // Ranks are set directly, so on_learn never ran: place the passive's rank
            // marker on the authority (replicas receive it with the bootstrap) so the
            // 士氣提升 ally aura is exercised.
            let hero=hero_pair(authority)[0];
            let handle=omoba_core::runtime::scripting::parallel_world_adapter::ParallelWorldAdapter::entity_to_handle(hero);
            authority.write_resource::<BuffStore>().add(hero,&super::No3_morale_boost::rank_marker(handle,1),Fixed64::from_raw(i64::MAX),serde_json::json!({}));
        },
        &mut |authority,tick|{
            let state=authority.read_resource::<MobaMatch>();
            let pair=[0,1].map(|index|state.heroes[index].entity);
            let positions=authority.read_storage::<Pos>();
            if spawns.get().is_none() {spawns.set(Some(pair.map(|hero|positions.get(hero.unwrap()).unwrap().0)));}
            let (Some(first),Some(second))=(pair[0],pair[1]) else {return Vec::new();};
            let [a,b]=spawns.get().unwrap();
            let enemy=positions.get(second).map(|p|p.0).unwrap_or(b);
            let buffs=authority.read_resource::<BuffStore>();
            let caster=omoba_core::runtime::scripting::parallel_world_adapter::ParallelWorldAdapter::entity_to_handle(first);
            if buffs.has(second,&format!("b29_breaker_armor:{}:{}",caster.id,caster.gen)) {aura_ticks.set(aura_ticks.get()+1);}
            if buffs.has(first,&format!("b29_morale:{}:{}",caster.id,caster.gen)) {morale_ticks.set(morale_ticks.get()+1);}
            if buffs.is_stunned(second) {stun_ticks.set(stun_ticks.get()+1);}
            let channeling=buffs.iter_for(first).any(|(id,_)|id.contains("ability_channel"));
            if channeling {channel_ticks.set(channel_ticks.get()+1);}
            let me=positions.get(first).map(|p|p.0).unwrap_or(a);
            if tick<hold_until.get() {return if tick%60==6 {vec![(2,attack_move(me))]} else {vec![]};}
            match tick {
                // Chase each other (re-aimed every second), then the enemy walks home out of
                // vision. A new order ends a channel, so the caster holds while channeling.
                t if t%1800<1300 && t%60==6=>if channeling {vec![(2,attack_move(me))]} else {vec![(1,attack_move(enemy)),(2,attack_move(me))]},
                t if t%1800==1300=>vec![(1,attack_move(a)),(2,attack_move(b))],
                // Proximity rotation within the mana pool: R, then W, then Q.
                t if t>6 && t%30==10 && !channeling=>{
                    let heroes=authority.read_storage::<Hero>();let h=heroes.get(first).unwrap();
                    let dist=(me-enemy).length();let mana=h.mana_pool.as_ref().map(|m|m.current()).unwrap_or(Fixed64::ZERO);
                    let ready=|slot:usize,cost:i32|h.abilities.get(slot).is_some_and(|id|h.get_cooldown(id)==Fixed64::ZERO) && mana>=Fixed64::from_i32(cost);
                    if dist<=Fixed64::from_i32(900) && ready(3,200) {hold_until.set(tick+30);vec![(1,cast_at(3,enemy))]}
                    else if dist<=Fixed64::from_i32(600) && ready(1,105) {vec![(1,cast_at(1,enemy))]}
                    else if dist<=Fixed64::from_i32(450) && ready(0,100) {vec![(1,cast(0,None))]}
                    else {vec![]}
                }
                _=>vec![],
            }
        });
    assert!(mismatches.is_empty(),"{:#?}",mismatches);
    eprintln!("b29 soak: armor aura {} morale aura {} stun {} channel {}",aura_ticks.get(),morale_ticks.get(),stun_ticks.get(),channel_ticks.get());
    assert!(aura_ticks.get()>0,"the 破軍靈光 armor debuff reached the enemy hero");
    assert!(stun_ticks.get()>0,"a slam or tremor wave stunned the enemy hero");
    assert!(channel_ticks.get()>=120,"the earth tremor host channel ran for at least 2 s");
    assert!(morale_ticks.get()>0,"the 士氣提升 ally aura reached the caster");
}
