//! Parity soak for 本多忠勝, carried by the package (installed test-only as
//! heroes::A07_honda_tadakatsu::parity_soak). Run (release):
//!   cargo test --release ... -p base_content --lib -- honda_tadakatsu_kit_keeps --nocapture
use crate::single_lane_match_tests::live_soak_replica_parity::{soak_match,cast,attack_move};
use crate::single_lane_match_tests::{three_lane_config,hero_pair};
use omoba_core::runtime::*;
use omoba_sim::Fixed64;
use specs::WorldExt;

/// The physical-immunity / spell-to-heal guard, the self-centred slow slam, the
/// rouse heal and haste and the ultimate (attack-speed multiplier, strength,
/// magic immunity and the attack cleave from the hero UnitScript) while the
/// enemy luminary casts its bolt at him and both walk in and out of vision;
/// both team replicas are checked every tick.
#[test]
fn honda_tadakatsu_kit_keeps_team_replicas_identical() {
    let spawns=std::cell::Cell::new(None);
    let (guard_ticks,slow_ticks,ultimate_ticks,rouse_ticks)=(std::cell::Cell::new(0u32),std::cell::Cell::new(0u32),
        std::cell::Cell::new(0u32),std::cell::Cell::new(0u32));
    let (crit_ticks,unslow_ticks)=(std::cell::Cell::new(0u32),std::cell::Cell::new(0u32));
    let mismatches=soak_match(SingleLaneConfig {mana_enabled:true,heroes:["honda_tadakatsu".into(),"training_luminary".into()],..three_lane_config()},7200,1,
        &|authority|{
            for hero in hero_pair(authority) {
                authority.write_storage::<Gold>().get_mut(hero).unwrap().0=5000;
                let mut heroes=authority.write_storage::<Hero>();let h=heroes.get_mut(hero).unwrap();h.level=12;
                for id in h.abilities.clone() {h.ability_levels.insert(id,1);}
            }
            // Ranks are set directly, so on_learn never ran: place the rank markers the
            // transform, the rouse table and the crit hook read (replicas get them at bootstrap).
            let hero=hero_pair(authority)[0];
            let handle=omoba_core::runtime::scripting::parallel_world_adapter::ParallelWorldAdapter::entity_to_handle(hero);
            for id in [super::No1_unscathed::ID,super::No2_red_thunder::ID,super::No3_shattering_spear::ID] {
                authority.write_resource::<BuffStore>().add(hero,&super::a07_ranks::marker(id,handle,1),Fixed64::from_raw(i64::MAX),serde_json::json!({}));
            }
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
            if buffs.has(first,&format!("a07_unscathed:{}:{}",caster.id,caster.gen)) {guard_ticks.set(guard_ticks.get()+1);}
            if buffs.iter_for(second).any(|(id,_)|id.contains("a07_red_thunder")) {slow_ticks.set(slow_ticks.get()+1);}
            if buffs.iter_for(first).any(|(id,_)|id.starts_with("a07_strongest_r")) {ultimate_ticks.set(ultimate_ticks.get()+1);}
            if buffs.has(first,&super::No3_shattering_spear::crit_buff(caster)) {crit_ticks.set(crit_ticks.get()+1);}
            if buffs.has(first,&super::No2_red_thunder::unslow_pending(caster)) {unslow_ticks.set(unslow_ticks.get()+1);}
            if buffs.has(first,&format!("a07_rouse:{}:{}",caster.id,caster.gen)) {rouse_ticks.set(rouse_ticks.get()+1);}
            match tick {
                // Chase each other (re-aimed every second), then walk home out of vision.
                t if t%1800<1300 && t%60==6=>vec![(1,attack_move(enemy)),(2,attack_move(me))],
                t if t%1800==1300=>vec![(1,attack_move(a)),(2,attack_move(b))],
                // The luminary bolts him (magic damage to absorb or ignore).
                t if t>6 && t%90==45=>vec![(2,cast(0,Some(first)))],
                // Proximity rotation within the mana pool (E186): every 30 ticks the first ready,
                // in-range ability starting from a slot that advances each time, so no ability
                // (or a refused cast that never cools down) starves the others whatever the
                // opponent's stats make of distance and mana.
                t if t>6 && t%30==10=>{
                    let heroes=authority.read_storage::<Hero>();let h=heroes.get(first).unwrap();
                    let dist=(me-enemy).length();let mana=h.mana_pool.as_ref().map(|m|m.current()).unwrap_or(Fixed64::ZERO);
                    let ready=|slot:usize,cost:i32|h.abilities.get(slot).is_some_and(|id|h.get_cooldown(id)==Fixed64::ZERO) && mana>=Fixed64::from_i32(cost);
                    // (slot, reach, rank-1 mana): R 戰國最強, W 赤雷擊 (525 radius), Q 毫髮無傷, D 奮起.
                    const ROTATION:[(usize,i32,i32);4]=[(3,700,150),(1,500,100),(0,500,100),(4,400,70)];
                    let start=(t/30) as usize%ROTATION.len();
                    (0..ROTATION.len()).map(|i|ROTATION[(start+i)%ROTATION.len()])
                        .find(|&(slot,reach,cost)|dist<=Fixed64::from_i32(reach) && ready(slot,cost))
                        .map(|(slot,_,_)|vec![(1,cast(slot as u32,None))]).unwrap_or_default()
                }
                _=>vec![],
            }
        });
    assert!(mismatches.is_empty(),"{:#?}",mismatches);
    eprintln!("a07 soak: guard {} slow {} ultimate {} rouse {} crit {} unslow {}",guard_ticks.get(),slow_ticks.get(),ultimate_ticks.get(),rouse_ticks.get(),crit_ticks.get(),unslow_ticks.get());
    assert!(guard_ticks.get()>0,"毫髮無傷 guarded the caster");
    assert!(slow_ticks.get()>0,"赤雷擊 slowed the enemy hero");
    assert!(ultimate_ticks.get()>0,"戰國最強 was active");
    assert!(rouse_ticks.get()>0,"奮起 hasted the caster");
    assert!(crit_ticks.get()>0,"碎擊槍 armed a critical attack");
    assert!(unslow_ticks.get()>0,"赤雷擊 scheduled the self unslow");
}
