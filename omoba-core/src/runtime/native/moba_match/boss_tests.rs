//! The map bosses (E187): timed first appearance, respawn growth, KV stats, the boss tag and the
//! original kill reward, through the production outcome pipeline. Source: damody/history_nobu @
//! f966720b, game/dota_addons/nobu/scripts/ (GameRulesStateChange.lua:482-533,
//! library/events/eventforkill.lua:438-539, items/Addon_Items/tofar_goback.lua:210-389,
//! npc/npc_units_custom_load.txt:15430-15569).
use super::*;
use specs::Join;
use crate::runtime::{process_outcomes,RuntimeEventVecSink,TowerTemplateRegistry,BuffStore,UnitStats};

const KING:&str="south_epic_guardian";
const SOULS:&str="north_epic_guardian";

/// Players 1, 3, 5 on side 0 and 2, 4, 6 on side 1, far apart, no gold, the three-lane map.
fn fixture(original:bool,warmup:i32)->World {
    let pool=crate::runtime::StateInitializer::create_thread_pool();
    let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
    world.insert(TowerTemplateRegistry::default());
    let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
    setup_single_lane_match(&mut world,SingleLaneConfig {warmup:Fixed64::from_i32(warmup),
        game_rules:original.then(||omoba_template_ids::GAME_RULES.unwrap()),
        map_id:Some(map.id.into()),lane_length:Fixed64::from_i32(map.lane_length),
        additional_players:[(3,1),(5,1),(4,2),(6,2)].map(|(player_id,team_id)|
            SingleLanePlayerConfig {player_id,team_id,hero:"training_luminary".into()}).to_vec(),
        ..Default::default()}).unwrap();
    world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
    for gold in (&mut world.write_storage::<Gold>()).join() {gold.0=0;}
    for (index,player) in [1,3,5,2,4,6].into_iter().enumerate() {
        let entity=hero(&world,player);
        world.write_storage::<Pos>().get_mut(entity).unwrap().0=Vec2::new(Fixed64::from_i32(50_000+index as i32*5000),Fixed64::ZERO);
    }
    world
}
fn index(world:&World,player:u32)->usize {world.read_resource::<MobaMatch>().heroes.iter().position(|s|s.player_id==player).unwrap()}
fn hero(world:&World,player:u32)->Entity {world.read_resource::<MobaMatch>().heroes[index(world,player)].entity.unwrap()}
fn camp_index(world:&World,id:&str)->usize {world.read_resource::<MobaMatch>().jungle_camps.iter().position(|c|c.definition.id==id).unwrap()}
fn body(world:&World,id:&str)->Option<Entity> {world.read_resource::<MobaMatch>().jungle_camps[camp_index(world,id)].entity}
fn balance(world:&World,player:u32)->i32 {
    let state=world.read_resource::<MobaMatch>();let slot=&state.heroes[index(world,player)];
    slot.entity.map_or(slot.gold.0,|e|world.read_storage::<Gold>().get(e).unwrap().0)
}
fn total_xp(world:&World,player:u32)->i64 {
    let rules=omoba_template_ids::GAME_RULES.unwrap();
    let state=world.read_resource::<MobaMatch>();let slot=&state.heroes[index(world,player)];
    let hero=slot.entity.and_then(|e|world.read_storage::<Hero>().get(e).cloned()).unwrap_or_else(||slot.hero.clone());
    i64::from(rules.experience_total_to_reach[(hero.level-1) as usize])+i64::from(hero.experience)
}
fn kill(world:&mut World,source:Entity,target:Entity) {
    world.write_resource::<Vec<Outcome>>().push(Outcome::Damage {pos:Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,
        real:Fixed64::from_i32(1_000_000),source,target,damage_profile:0,predeclared:false});
    process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();
    process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();world.maintain();
}
/// Runs the camp tick at a match time (seconds of play after warmup).
fn camp_tick_at(world:&mut World,seconds:Fixed64) {
    let mut state=world.remove::<MobaMatch>().unwrap();
    state.elapsed=state.config.warmup+seconds;
    jungle::tick(world,&mut state,Fixed64::ZERO);world.insert(state);
}
fn respawn_now(world:&mut World,id:&str) {
    let at=world.read_resource::<MobaMatch>().jungle_camps[camp_index(world,id)].respawn_at.expect("respawn scheduled");
    let warmup=world.read_resource::<MobaMatch>().config.warmup;
    camp_tick_at(world,at-warmup);
}

#[test]
fn bosses_first_appear_on_the_playing_clock_and_respawn_with_growth_and_kv_stats() {
    let mut world=fixture(true,30);
    let definitions:Vec<_>=[KING,SOULS].iter().map(|id|world.read_resource::<MobaMatch>().jungle_camps[camp_index(&world,id)].definition).collect();
    for definition in &definitions {
        let state=world.read_resource::<MobaMatch>();let camp=&state.jungle_camps[camp_index(&world,definition.id)];
        assert!(camp.entity.is_none() && camp.first_appearance_pending,"{} waits for its time",definition.id);
        assert_eq!(camp.respawn_at,Some(Fixed64::from_i32(30+definition.first_spawn_seconds)),"after the warmup");
    }
    assert!(world.read_resource::<MobaMatch>().jungle_camps.iter().filter(|c|c.definition.first_spawn_seconds==0).all(|c|c.entity.is_some()));
    // A pause stops the clock the deadline is on.
    {let mut state=world.write_resource::<MobaMatch>();state.elapsed=Fixed64::from_i32(30+179);}
    world.write_resource::<GamePause>().is_paused=true;world.write_resource::<DeltaTime>().0=Fixed64::from_i32(10);
    begin_moba_match_tick(&mut world);
    assert!(body(&world,KING).is_none(),"paused: no appearance");
    world.write_resource::<GamePause>().is_paused=false;
    camp_tick_at(&mut world,Fixed64::from_i32(180)-Fixed64::from_raw(1));
    assert!(body(&world,KING).is_none());
    camp_tick_at(&mut world,Fixed64::from_i32(180));
    let king=body(&world,KING).expect("the king at 180 s");
    assert!(body(&world,SOULS).is_none());
    {
        let state=world.read_resource::<MobaMatch>();let camp=&state.jungle_camps[camp_index(&world,KING)];
        assert_eq!(camp.respawns,0,"the first appearance is no respawn");assert!(!camp.first_appearance_pending);
        assert_eq!(camp.attack_duration_for_test(),Fixed64::ONE);
    }
    let check=|world:&World,entity:Entity,hp:i32,damage:i32,armor:i32,resist:i32| {
        let unit=world.read_storage::<Unit>().get(entity).cloned().unwrap();
        assert_eq!((unit.max_hp,unit.base_damage,unit.base_armor,unit.magic_resistance),
            (hp,damage,Fixed64::from_i32(armor),Fixed64::from_i32(resist)));
        let property=world.read_storage::<CProperty>().get(entity).cloned().unwrap();
        assert_eq!((property.hp,property.mhp,property.def_physic,property.def_magic),
            (Fixed64::from_i32(hp),Fixed64::from_i32(hp),Fixed64::from_i32(armor),Fixed64::from_i32(resist)));
        assert!(is_boss(world,entity));
    };
    check(&world,king,5000,205,20,30);
    // Death schedules the respawn 300 s later; respawn n adds n x (+1000 HP, +30 attack).
    let killer=hero(&world,1);
    for n in 1..=2 {
        let target=body(&world,KING).unwrap();
        kill(&mut world,killer,target);
        let state=world.read_resource::<MobaMatch>();let camp=&state.jungle_camps[camp_index(&world,KING)];
        assert_eq!(camp.respawn_at,Some(state.elapsed+Fixed64::from_i32(300)));drop(state);
        respawn_now(&mut world,KING);
        let next=body(&world,KING).unwrap();assert_ne!(next,target);
        assert_eq!(world.read_resource::<MobaMatch>().jungle_camps[camp_index(&world,KING)].respawns,n);
        check(&world,next,5000+1000*n as i32,205+30*n as i32,20,30);
    }
    // The souls: 1200 s, 1.2 s attacks, 600 s respawn, +5000 HP and +75 attack per respawn.
    camp_tick_at(&mut world,Fixed64::from_i32(1200));
    let souls=body(&world,SOULS).expect("the souls at 1200 s");
    check(&world,souls,15000,500,40,50);
    assert_eq!(world.read_resource::<MobaMatch>().jungle_camps[camp_index(&world,SOULS)].attack_duration_for_test().raw(),1200*1024/1000);
    kill(&mut world,killer,souls);
    {let state=world.read_resource::<MobaMatch>();
        assert_eq!(state.jungle_camps[camp_index(&world,SOULS)].respawn_at,Some(state.elapsed+Fixed64::from_i32(600)));}
    respawn_now(&mut world,SOULS);
    check(&world,body(&world,SOULS).unwrap(),20000,575,40,50);
    // The camp state (first-appearance flag, respawns) is replay state.
    let before=single_lane_replay_digest(&world);
    let souls=camp_index(&world,SOULS);world.write_resource::<MobaMatch>().jungle_camps[souls].first_appearance_pending=true;
    assert_ne!(before,single_lane_replay_digest(&world));
}

#[test]
fn only_the_boss_bodies_carry_the_boss_tag() {
    let mut world=fixture(false,0);
    spawn_waiting_jungle_camps_for_test(&mut world);
    let state=world.read_resource::<MobaMatch>();
    let mut bosses=Vec::new();
    for camp in &state.jungle_camps {
        let leader=camp.entity.unwrap();
        if is_boss(&world,leader) {bosses.push(camp.definition.id);}
        assert_eq!(is_boss(&world,leader),camp.definition.has_tag(omoba_template_ids::MobaUnitTag::Boss),"{}",camp.definition.id);
        for member in &camp.members {assert!(!is_boss(&world,member.entity.unwrap()),"members carry no tags");}
    }
    bosses.sort();
    assert_eq!(bosses,[SOULS,KING]);
    for slot in &state.heroes {assert!(!is_boss(&world,slot.entity.unwrap()));}
    // A world that never registered the tag storage has no boss.
    let mut bare=World::new();let entity=bare.create_entity().build();
    assert!(!is_boss(&bare,entity));
}

#[test]
fn original_rules_pay_the_killing_side_and_rotate_the_kings_buff() {
    let mut world=fixture(true,0);
    spawn_waiting_jungle_camps_for_test(&mut world);
    let seed=world.read_resource::<MobaMatch>().config.seed;
    let reward=world.read_resource::<MobaMatch>().jungle_camps[camp_index(&world,KING)].definition.original_reward.unwrap();
    let (h1,h3,h2)=(hero(&world,1),hero(&world,3),hero(&world,2));
    // A dead ally still gets the gold and XP; the lost-on-death buff only goes to the living.
    let h5=hero(&world,5);record_moba_death(&mut world,h5);world.maintain();
    let king=body(&world,KING).unwrap();
    kill(&mut world,h1,king);
    assert_eq!((balance(&world,1),balance(&world,3),balance(&world,5),balance(&world,2)),(150+300,300,300,0),
        "KV 150 to the last hitter, 300 to every hero of the side");
    assert_eq!((total_xp(&world,1),total_xp(&world,3),total_xp(&world,5),total_xp(&world,2)),(150+200,200,200,0));
    let first=reward.buffs_for_kill(seed,0)[0];
    {
        let buffs=world.read_resource::<BuffStore>();
        assert!(buffs.has(h1,first.buff_id) && buffs.has(h3,first.buff_id) && !buffs.has(h2,first.buff_id));
        for training in ["objective_guard","objective_legacy"] {assert!(!buffs.has(h1,training),"no training buff under the original rules");}
        assert_eq!(buffs.get(h1,first.buff_id).unwrap().remaining,Fixed64::from_i32(180));
    }
    // The next kill takes the next buff in the rotation.
    respawn_now(&mut world,KING);
    let king=body(&world,KING).unwrap();kill(&mut world,h1,king);
    let second=reward.buffs_for_kill(seed,1)[0];
    assert_ne!(second.buff_id,first.buff_id);
    assert!(world.read_resource::<BuffStore>().has(h3,second.buff_id));
    // Each type's modifiers (npc_abilities_custom_load.txt:821-852).
    let lease=|id:&str|*reward.buffs.iter().find(|b|b.buff_id==id).unwrap();
    let weapons=objective_buffs::payload(&lease("robbers_king_weapons"),1,"robbers_king_weapons");
    assert_eq!((weapons["basedamageoutgoing_percentage"].as_i64(),weapons["spell_amplify_percentage"].as_i64()),(Some(205),Some(256)));
    let armor=objective_buffs::payload(&lease("robbers_king_armor"),1,"robbers_king_armor");
    assert_eq!((armor["physical_armor_bonus"].as_i64(),armor["magical_resistance_bonus"].as_i64()),(Some(10*1024),Some(25*1024)));
    let remedy=objective_buffs::payload(&lease("robbers_king_remedy"),1,"robbers_king_remedy");
    assert_eq!((remedy["health_regen_max_percentage"].as_i64(),remedy["mana_regen_constant"].as_i64()),(Some(31),Some(10*1024)));
    assert!(remedy.get("basedamageoutgoing_percentage").is_none(),"unset stats stay out of the payload");
    // The souls: 1000 gold each, no team XP, a 180 s lease a respawned hero gets back.
    let before=(balance(&world,1),balance(&world,3),total_xp(&world,3));
    let souls=body(&world,SOULS).unwrap();kill(&mut world,h1,souls);
    assert_eq!((balance(&world,1)-before.0,balance(&world,3)-before.1,total_xp(&world,3)-before.2),(450+1000,1000,0));
    let buffs=world.read_resource::<BuffStore>();
    let morale=buffs.get(h3,"warrior_souls_morale").unwrap();
    assert_eq!(morale.remaining,Fixed64::from_i32(180));
    for (key,value) in [("basedamageoutgoing_percentage",307),("spell_amplify_percentage",307),("attackspeed_bonus_constant",20*1024),
        ("movespeed_bonus_percentage",102),("magical_resistance_bonus",15*1024),("physical_armor_bonus",10*1024)] {
        assert_eq!(morale.payload[key].as_i64(),Some(value),"{key}");
    }
    let stats=UnitStats::from_refs(&buffs,false);
    assert_eq!(stats.final_armor(Fixed64::ZERO,h3),Fixed64::from_i32(10)+if buffs.has(h3,"robbers_king_armor") {Fixed64::from_i32(10)} else {Fixed64::ZERO});
    assert!(!buffs.has(h2,"warrior_souls_morale"));
    assert!(world.read_resource::<MobaMatch>().objective_buffs.contains_key(&(0,"warrior_souls_morale")));
}

#[test]
fn without_the_original_rules_bosses_keep_the_training_buffs_and_pay_only_the_last_hitter() {
    let mut world=fixture(false,0);
    spawn_waiting_jungle_camps_for_test(&mut world);
    let (h1,h3)=(hero(&world,1),hero(&world,3));
    let king=body(&world,KING).unwrap();kill(&mut world,h1,king);
    let buffs=world.read_resource::<BuffStore>();
    assert!(buffs.has(h1,"objective_guard") && buffs.has(h3,"objective_legacy"));
    assert!(["robbers_king_weapons","robbers_king_armor","robbers_king_remedy"].iter().all(|id|!buffs.has(h3,id)));
    drop(buffs);
    assert_eq!(balance(&world,3),0,"no team gold");
}
