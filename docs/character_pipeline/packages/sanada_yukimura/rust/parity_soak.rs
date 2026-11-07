//! Parity soak for 真田幸村, carried by the package (installed test-only as
//! heroes::B06_sanada_yukimura::parity_soak). Run (release):
//!   cargo test --release ... -p base_content --lib -- sanada_yukimura_kit_keeps --nocapture
use crate::single_lane_match_tests::live_soak_replica_parity::{soak_match,cast,attack_move};
use crate::single_lane_match_tests::{three_lane_config,hero_pair};
use omoba_core::runtime::*;
use omoba_sim::Fixed64;
use specs::WorldExt;

/// The ally warriors buff, the bouncing flame stun, the evasion / spell guard
/// passive (tripped by the enemy luminary's bolt) and the ultimate with its
/// team-wide aura from the hero UnitScript tick, while both heroes walk in and
/// out of vision; both team replicas are checked every tick.
#[test]
fn sanada_yukimura_kit_keeps_team_replicas_identical() {
    let spawns=std::cell::Cell::new(None);
    let hold_until=std::cell::Cell::new(0u32);
    let (warriors,stun,guard,ultimate,aura)=(std::cell::Cell::new(0u32),std::cell::Cell::new(0u32),std::cell::Cell::new(0u32),
        std::cell::Cell::new(0u32),std::cell::Cell::new(0u32));
    let mismatches=soak_match(SingleLaneConfig {mana_enabled:true,heroes:["sanada_yukimura".into(),"training_luminary".into()],..three_lane_config()},7200,1,
        &|authority|{
            for hero in hero_pair(authority) {
                authority.write_storage::<Gold>().get_mut(hero).unwrap().0=5000;
                let mut heroes=authority.write_storage::<Hero>();let h=heroes.get_mut(hero).unwrap();h.level=12;
                for id in h.abilities.clone() {h.ability_levels.insert(id,1);}
            }
            // Ranks are set directly, so on_learn never ran: place the passive's row
            // (evasion and the spell guard) on the authority; replicas get it at bootstrap.
            let hero=hero_pair(authority)[0];
            let payload=super::No3_mind_eye::passive_payload(1).unwrap();
            authority.write_resource::<BuffStore>().add(hero,super::No3_mind_eye::PASSIVE_BUFF,Fixed64::from_raw(i64::MAX),
                serde_json::from_str(&payload).unwrap());
        },
        &mut |authority,tick|{
            let state=authority.read_resource::<MobaMatch>();
            let pair=[0,1].map(|index|state.heroes[index].entity);
            let positions=authority.read_storage::<Pos>();
            if spawns.get().is_none() {spawns.set(Some(pair.map(|hero|positions.get(hero.unwrap()).unwrap().0)));}
            let (Some(first),Some(second))=(pair[0],pair[1]) else {return Vec::new();};
            let [a,b]=spawns.get().unwrap();
            let enemy=positions.get(second).map(|p|p.0).unwrap_or(b);
            let me=positions.get(first).map(|p|p.0).unwrap_or(a);
            let buffs=authority.read_resource::<BuffStore>();
            let caster=omoba_core::runtime::scripting::parallel_world_adapter::ParallelWorldAdapter::entity_to_handle(first);
            if buffs.has(first,&super::No1_brave_warriors::buff_id(caster)) {warriors.set(warriors.get()+1);}
            if buffs.is_stunned(second) {stun.set(stun.get()+1);}
            if buffs.has(first,omoba_core::runtime::native::spell_guard::ACTIVE) {guard.set(guard.get()+1);}
            if buffs.has(first,&super::No4_first_warrior::buff_id(caster)) {ultimate.set(ultimate.get()+1);}
            if buffs.has(first,&super::No4_first_warrior::aura_buff(caster)) {aura.set(aura.get()+1);}
            if tick<hold_until.get() {return if tick%60==6 {vec![(2,attack_move(me))]} else {vec![]};}
            match tick {
                // Chase each other (re-aimed every second), then walk home out of vision.
                t if t%1800<1300 && t%60==6=>vec![(1,attack_move(enemy)),(2,attack_move(me))],
                t if t%1800==1300=>vec![(1,attack_move(a)),(2,attack_move(b))],
                // The luminary bolts him (trips the spell guard).
                t if t>6 && t%90==45=>vec![(2,cast(0,Some(first)))],
                // Rotation (E186): every 30 ticks the first ready, in-range ability starting from a
                // slot that advances each time, so none starves the others. W is unit-targeted at
                // 250 (0.25 s windup: hold the chase so the windup is not cancelled).
                t if t>6 && t%30==10=>{
                    let heroes=authority.read_storage::<Hero>();let h=heroes.get(first).unwrap();
                    let dist=(me-enemy).length();let mana=h.mana_pool.as_ref().map(|m|m.current()).unwrap_or(Fixed64::ZERO);
                    let ready=|slot:usize,cost:i32|h.abilities.get(slot).is_some_and(|id|h.get_cooldown(id)==Fixed64::ZERO) && mana>=Fixed64::from_i32(cost);
                    // (slot, reach, rank-1 mana): R 日本第一武士, W 連牙炎突, Q 真田勇士.
                    const ROTATION:[(usize,i32,i32);3]=[(3,700,300),(1,240,120),(0,700,100)];
                    let start=(t/30) as usize%ROTATION.len();
                    match (0..ROTATION.len()).map(|i|ROTATION[(start+i)%ROTATION.len()])
                        .find(|&(slot,reach,cost)|dist<=Fixed64::from_i32(reach) && ready(slot,cost)) {
                        Some((1,_,_))=>{hold_until.set(tick+30);vec![(1,cast(1,Some(second)))]}
                        Some((slot,_,_))=>vec![(1,cast(slot as u32,None))],
                        None=>vec![],
                    }
                }
                _=>vec![],
            }
        });
    assert!(mismatches.is_empty(),"{:#?}",mismatches);
    eprintln!("b06 soak: warriors {} stun {} guard {} ultimate {} aura {}",warriors.get(),stun.get(),guard.get(),ultimate.get(),aura.get());
    assert!(warriors.get()>0,"真田勇士 buffed the caster");
    assert!(stun.get()>0,"連牙炎突 stunned the enemy hero");
    assert!(guard.get()>0,"心眼 guard tripped by the enemy bolt");
    assert!(ultimate.get()>0,"日本第一武士 was active");
    assert!(aura.get()>0,"the team-wide aura reached the caster");
}
