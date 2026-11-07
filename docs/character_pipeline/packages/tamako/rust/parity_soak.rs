//! Parity soak for 玉子, carried by the package (installed test-only as
//! heroes::C15_tamako::parity_soak). Run (release):
//!   cargo test --release ... -p base_content --lib -- tamako_kit_keeps --nocapture
use crate::single_lane_match_tests::live_soak_replica_parity::{soak_match,cast,cast_at,attack_move};
use crate::single_lane_match_tests::{three_lane_config,hero_pair};
use omoba_core::runtime::*;
use omoba_sim::Fixed64;
use specs::WorldExt;

/// Scatter shot at the enemy, the growing burst-arrow fire line with its
/// lingering burn, and 百箭齊發 (stun + invulnerable arrows locked on the target,
/// plus the every-8th-attack passive arrow), against 村上義清 walking in and out
/// of vision; both team replicas are checked every tick.
#[test]
fn tamako_kit_keeps_team_replicas_identical() {
    let spawns=std::cell::Cell::new(None);
    // Counters prove the kit actually fired; parity alone would pass on an idle hero.
    let (burn_ticks,arrow_ticks,stun_ticks)=(std::cell::Cell::new(0u32),std::cell::Cell::new(0u32),std::cell::Cell::new(0u32));
    let mismatches=soak_match(SingleLaneConfig {mana_enabled:true,heroes:["tamako".into(),"murakami_yoshikiyo".into()],..three_lane_config()},7200,1,
        &|authority|{
            for hero in hero_pair(authority) {
                authority.write_storage::<Gold>().get_mut(hero).unwrap().0=5000;
                let mut heroes=authority.write_storage::<Hero>();let h=heroes.get_mut(hero).unwrap();h.level=12;
                for id in h.abilities.clone() {h.ability_levels.insert(id,1);}
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
            if buffs.iter_for(second).any(|(id,_)|id.starts_with("generic_dot:ground_damage_zone:")) {burn_ticks.set(burn_ticks.get()+1);}
            if buffs.is_stunned(second) {stun_ticks.set(stun_ticks.get()+1);}
            {
                let summons=authority.read_storage::<SummonedUnit>();
                let entities=authority.entities();
                use specs::Join;
                if (&entities,&summons).join().any(|(_,s)|s.summoner==first) {arrow_ticks.set(arrow_ticks.get()+1);}
            }
            match tick {
                // Chase each other, then the enemy walks home out of vision.
                t if t%1800<1300 && t%60==6=>vec![(1,attack_move(enemy)),(2,attack_move(me))],
                t if t%1800==1300=>vec![(1,attack_move(a)),(2,attack_move(b))],
                // Proximity rotation within the mana pool: R, then W, then Q.
                t if t>6 && t%30==10=>{
                    let heroes=authority.read_storage::<Hero>();let h=heroes.get(first).unwrap();
                    let dist=(me-enemy).length();let mana=h.mana_pool.as_ref().map(|m|m.current()).unwrap_or(Fixed64::ZERO);
                    let ready=|slot:usize,cost:i32|h.abilities.get(slot).is_some_and(|id|h.get_cooldown(id)==Fixed64::ZERO) && mana>=Fixed64::from_i32(cost);
                    if dist<=Fixed64::from_i32(650) && ready(3,150) {vec![(1,cast(3,Some(second)))]}
                    else if dist<=Fixed64::from_i32(1500) && ready(1,150) {vec![(1,cast_at(1,enemy))]}
                    else if dist<=Fixed64::from_i32(650) && ready(0,100) {vec![(1,cast_at(0,enemy))]}
                    else {vec![]}
                }
                _=>vec![],
            }
        });
    eprintln!("c15 soak: burn {} arrows {} stun {}",burn_ticks.get(),arrow_ticks.get(),stun_ticks.get());
    assert!(mismatches.is_empty(),"{:#?}",&mismatches[..mismatches.len().min(12)]);
    assert!(burn_ticks.get()>0,"the burst-arrow fire line burned the enemy hero");
    assert!(arrow_ticks.get()>0,"百箭齊發 released lethal arrows");
    assert!(stun_ticks.get()>0,"scatter shot or 百箭齊發 stunned the enemy hero");
}

/// split48 setting: the full role-bot match with 村上義清 (p1) and 玉子 (p6) both
/// learning and casting their kits over the whole match (deaths, respawns, vision
/// changes, items); both team replicas are checked every second.
/// Set SOAK_BOT_TICKS=37000 for the full ten minutes.
#[test]
fn tamako_and_murakami_role_bot_match_keeps_team_replicas_identical() {
    use crate::single_lane_match_tests::live_soak_replica_parity::soak_match;
    use omoba_core::runtime::native::moba_match::bots::*;
    use omoba_core::game_proto::UpgradeAbility;
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let output=std::process::Command::new(root.join("tools/lua/lua.exe")).current_dir(&root)
        .args(["-e",r#"local j=require('tools.lua.lib.json');print(j.encode(assert(loadfile('scripts/lua_data/moba_role_match.lua'))()))"#]).output().unwrap();
    assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    // SOAK_ROLE_PLAN=<match-plan.json> replays a live run's exact plan (map, roster, seed rules).
    let source=std::env::var("SOAK_ROLE_PLAN").ok().map(|path|std::fs::read(path).unwrap()).unwrap_or(output.stdout);
    let mut plan:RoleBotMatchPlan=serde_json::from_slice(&source).unwrap();
    for player in &mut plan.players {player.bot=true;}
    // Seat the two kits; a teammate already on that hero takes the seat's previous hero
    // (one hero per team, E164).
    for (seat,hero) in [(1u32,"murakami_yoshikiyo"),(6,"tamako")] {
        let index=plan.players.iter().position(|p|p.player_id==seat).unwrap();
        let (team,previous)=(plan.players[index].team_id,std::mem::replace(&mut plan.players[index].hero,hero.into()));
        for other in plan.players.iter_mut().filter(|p|p.player_id!=seat && p.team_id==team && p.hero==hero) {other.hero=previous.clone();}
    }
    let (config,bots)=plan.compile(0x5048,omoba_core::runtime::SimulationTickProfile::Production60Hz).unwrap();
    let ticks=std::env::var("SOAK_BOT_TICKS").ok().and_then(|v|v.parse().ok()).unwrap_or(7200);
    let mut inputs=|world:&specs::World,tick:u32|{
        let mut inputs=role_bot_inputs(world,&bots).unwrap();
        let state=world.read_resource::<MobaMatch>();
        let find=|player:u32|state.heroes.iter().find(|slot|slot.player_id==player).and_then(|slot|slot.entity);
        let pos=|e:Option<specs::Entity>|e.and_then(|e|world.read_storage::<Pos>().get(e).map(|p|p.0));
        let (murakami,tamako)=(find(1),find(6));
        match tick%240 {
            0=>for player in [1,6] {inputs.extend((0..4).map(|slot|(player,PlayerInput {action:Some(PlayerInputEnum::UpgradeAbility(UpgradeAbility {ability_index:slot}))})));},
            // 玉子 aims W / Q / R at 村上義清, who answers with Q / W / R toward 玉子.
            60=>if let Some(at)=pos(murakami) {inputs.push((6,cast_at(1,at)));},
            90=>if let Some(at)=pos(tamako) {inputs.push((1,cast_at(1,at)));},
            120=>if let Some(at)=pos(murakami) {inputs.push((6,cast_at(0,at)));},
            150=>inputs.push((1,cast(0,None))),
            180=>inputs.push((6,cast(3,murakami))),
            210=>if let Some(at)=pos(tamako) {inputs.push((1,cast_at(3,at)));},
            _=>{}
        }
        inputs
    };
    let mismatches=soak_match(config,ticks,60,&|_|{},&mut inputs);
    assert!(mismatches.is_empty(),"{:#?}",&mismatches[..mismatches.len().min(12)]);
}
