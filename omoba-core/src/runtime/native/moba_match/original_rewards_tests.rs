//! The original reward model through the production outcome pipeline (E187).
use super::*;
use specs::Join;
use crate::runtime::{process_outcomes,RuntimeEventVecSink,TowerTemplateRegistry,BuffStore};

fn rules()->GameRulesConst {omoba_template_ids::GAME_RULES.expect("the catalog carries game_rules")}

/// Players 1, 3, 5 on side 0 and 2, 4, 6 on side 1, far apart, no gold, the original rules.
fn fixture(map:bool)->World {
    let pool=crate::runtime::StateInitializer::create_thread_pool();
    let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
    world.insert(TowerTemplateRegistry::default());
    let mut config=SingleLaneConfig {warmup:Fixed64::ZERO,game_rules:Some(rules()),
        additional_players:[(3,1),(5,1),(4,2),(6,2)].map(|(player_id,team_id)|
            SingleLanePlayerConfig {player_id,team_id,hero:"training_luminary".into()}).to_vec(),
        ..Default::default()};
    if map {
        let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        config.map_id=Some(map.id.into());config.lane_length=Fixed64::from_i32(map.lane_length);
    }
    setup_single_lane_match(&mut world,config).unwrap();
    world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
    // The bosses appear on the match clock (E187); these fixtures fight them at the start.
    spawn_waiting_jungle_camps_for_test(&mut world);
    for gold in (&mut world.write_storage::<Gold>()).join() {gold.0=0;}
    for (index,player) in [1,3,5,2,4,6].into_iter().enumerate() {
        let entity=hero(&world,player);place(&mut world,entity,50_000+index as i32*5000,0);
    }
    world
}
fn index(world:&World,player:u32)->usize {world.read_resource::<MobaMatch>().heroes.iter().position(|s|s.player_id==player).unwrap()}
fn hero(world:&World,player:u32)->Entity {world.read_resource::<MobaMatch>().heroes[index(world,player)].entity.unwrap()}
fn place(world:&mut World,entity:Entity,x:i32,y:i32) {
    world.write_storage::<Pos>().get_mut(entity).unwrap().0=Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
}
fn set_level(world:&mut World,player:u32,level:i32) {
    let entity=hero(world,player);let mut heroes=world.write_storage::<Hero>();let hero=heroes.get_mut(entity).unwrap();
    hero.level=level;hero.experience=0;hero.experience_to_next=Hero::moba_experience_to_next(level,Some(&rules()));
}
/// Total experience of a player's hero (live body, else its retired progression).
fn total_xp(world:&World,player:u32)->i64 {
    let state=world.read_resource::<MobaMatch>();let slot=&state.heroes[index(world,player)];
    let hero=slot.entity.and_then(|e|world.read_storage::<Hero>().get(e).cloned()).unwrap_or_else(||slot.hero.clone());
    i64::from(rules().experience_total_to_reach[(hero.level-1) as usize])+i64::from(hero.experience)
}
fn balance(world:&World,player:u32)->i32 {
    let state=world.read_resource::<MobaMatch>();let slot=&state.heroes[index(world,player)];
    slot.entity.map_or(slot.gold.0,|e|world.read_storage::<Gold>().get(e).unwrap().0)
}
fn damage(source:Entity,target:Entity,amount:i32)->Outcome {
    Outcome::Damage {pos:Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,
        real:Fixed64::from_i32(amount),source,target,damage_profile:0,predeclared:false}
}
fn flush(world:&mut World,outcomes:impl IntoIterator<Item=Outcome>) {
    world.write_resource::<Vec<Outcome>>().extend(outcomes);
    process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();
    process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();world.maintain();
}
/// A tracked side-1 lane creep renamed to a map minion id, at the origin.
fn creep(world:&mut World,id:&str)->Entity {
    let mut state=world.remove::<MobaMatch>().unwrap();state.spawn_wave(world);
    let entity=state.units.values().filter(|u|u.role==LaneRole::Creep && u.team==1).map(|u|u.entity).last().unwrap();
    world.insert(state);
    world.write_storage::<Creep>().get_mut(entity).unwrap().name=id.into();
    place(world,entity,0,0);entity
}

#[test]
fn lane_minion_xp_goes_whole_to_each_nearby_enemy_hero_and_gold_and_killer_xp_follow_the_type() {
    let r=rules();
    for id in ["lane_melee","lane_caster","lane_siege","lane_cavalry"] {
        let row=r.lane_minion(id).unwrap();
        let mut world=fixture(false);let target=creep(&mut world,id);
        let (h1,h3,h5,h2)=(hero(&world,1),hero(&world,3),hero(&world,5),hero(&world,2));
        place(&mut world,h1,100,0);place(&mut world,h3,1400,0);place(&mut world,h5,1401,0);place(&mut world,h2,0,100);
        flush(&mut world,[damage(h1,target,100_000)]);
        // Two enemy heroes in 1400: each gets the 2-hero percent of the whole XP (30 -> 21, 50 -> 35).
        let share=i64::from(r.lane_xp_share(row.xp,2));
        assert_eq!(share,i64::from(row.xp)*70/100,"{id}");
        assert_eq!(total_xp(&world,1),share+i64::from(row.killer_xp),"{id}: last hitter adds the killer XP");
        assert_eq!((total_xp(&world,3),total_xp(&world,5),total_xp(&world,2)),(share,0,0),"{id}");
        assert_eq!((balance(&world,1),balance(&world,3)),(row.gold as i32,0),"{id}: gold to the last hitter only");
    }
    // Infantry 25 / archer 35 / gunner and cavalry 60 gold; gunner and cavalry +50 killer XP.
    assert_eq!(["lane_melee","lane_caster","lane_siege","lane_cavalry"].map(|id|{let m=r.lane_minion(id).unwrap();(m.gold,m.killer_xp)}),
        [(25,0),(35,0),(60,50),(60,50)]);
}

#[test]
fn lane_minion_xp_pays_any_killer_scales_by_map_and_skips_unlisted_units() {
    let r=rules();let row=r.lane_minion("lane_siege").unwrap();
    // A tower kill: heroes in range still get the XP, nobody gets gold or killer XP.
    let mut world=fixture(false);let target=creep(&mut world,"lane_siege");
    let tower=world.read_resource::<MobaMatch>().towers[0].unwrap();let h1=hero(&world,1);place(&mut world,h1,0,0);
    flush(&mut world,[damage(tower,target,100_000)]);
    assert_eq!((total_xp(&world,1),balance(&world,1)),(i64::from(row.xp),0));
    // The quick map multiplies every hero XP award (hero_xp_scale), after the type's share.
    let mut world=fixture(false);world.write_resource::<MobaMatch>().config.hero_xp_scale_permille=3500;
    let target=creep(&mut world,"lane_siege");let h1=hero(&world,1);place(&mut world,h1,0,0);
    flush(&mut world,[damage(h1,target,100_000)]);
    assert_eq!(total_xp(&world,1),i64::from((row.xp+row.killer_xp)*35/10));
    assert_eq!(balance(&world,1),row.gold as i32,"gold is never scaled");
    // A unit without a lane_minions row gives nothing under the original rules.
    let mut world=fixture(false);let target=creep(&mut world,"single_lane_creep");let h1=hero(&world,1);place(&mut world,h1,0,0);
    flush(&mut world,[damage(h1,target,100_000)]);
    assert_eq!((total_xp(&world,1),balance(&world,1)),(0,0));
}

#[test]
fn hero_kill_xp_is_shared_by_level_gap_among_enemies_in_range_and_a_lower_killer_gets_more() {
    let r=rules();let kill=r.hero_kill;
    let mut world=fixture(false);
    for (player,level) in [(2,6),(1,6),(3,4),(5,8),(4,6)] {set_level(&mut world,player,level);}
    let (h1,h2,h3,h5,h4)=(hero(&world,1),hero(&world,2),hero(&world,3),hero(&world,5),hero(&world,4));
    place(&mut world,h2,0,0);place(&mut world,h1,500,0);place(&mut world,h3,2000,0);place(&mut world,h5,0,1999);place(&mut world,h4,0,10);
    let before:Vec<_>=[1,3,5,4].map(|p|total_xp(&world,p)).to_vec();
    flush(&mut world,[damage(h1,h2,100_000)]);
    let gained:Vec<_>=[1,3,5,4].iter().zip(&before).map(|(p,b)|total_xp(&world,*p)-b).collect();
    // N = 3 enemies in 2000 (the level 8 hero counts but gets nothing): 6 * 30 / 3 = 60, x0.5 same
    // level, x2 two levels below the victim; the victim's ally gets nothing.
    assert_eq!(gained,vec![i64::from(kill.xp_share(6,6,3)),i64::from(kill.xp_share(6,4,3)),0,0]);
    assert_eq!(gained,vec![30,120,0,0]);
    // A killer of lower level: its proximity share plus victim level * 20, even out of range.
    for near in [true,false] {
        let mut world=fixture(false);
        for (player,level) in [(2,6),(1,3),(3,6)] {set_level(&mut world,player,level);}
        let (h1,h2,h3)=(hero(&world,1),hero(&world,2),hero(&world,3));
        place(&mut world,h2,0,0);place(&mut world,h3,100,0);if near {place(&mut world,h1,200,0);}
        let (b1,b3)=(total_xp(&world,1),total_xp(&world,3));
        flush(&mut world,[damage(h1,h2,100_000)]);
        let n=if near {2} else {1};
        let proximity=if near {i64::from(kill.xp_share(6,3,n))} else {0};
        assert_eq!(total_xp(&world,1)-b1,proximity+i64::from(kill.killer_lower_level_xp(6,3)),"near {near}");
        assert_eq!(total_xp(&world,3)-b3,i64::from(kill.xp_share(6,6,n)));
        assert_eq!(kill.killer_lower_level_xp(6,3),120);
    }
    // A hero killed by a tower still pays the proximity XP.
    let mut world=fixture(false);set_level(&mut world,2,4);
    let (h2,h1,tower)=(hero(&world,2),hero(&world,1),world.read_resource::<MobaMatch>().towers[0].unwrap());
    place(&mut world,h2,0,0);place(&mut world,h1,100,0);
    flush(&mut world,[damage(tower,h2,100_000)]);
    assert_eq!(total_xp(&world,1),i64::from(kill.xp_share(4,1,1)));
    assert_eq!(balance(&world,1),0);
}

#[test]
fn hero_kill_gold_adds_streak_multi_kill_and_hoard_terms_and_assists_get_the_extra_gold() {
    let r=rules();let kill=r.hero_kill;
    let mut world=fixture(false);
    // Training rules in the config are ignored under the original rules.
    {let mut state=world.write_resource::<MobaMatch>();state.config.kill_bounty_rules=KillBountyRules::training();
        state.config.death_streak_rules=DeathStreakRules::training();state.config.first_kill_bonus_gold=100;}
    let (h1,h2,h3)=(hero(&world,1),hero(&world,2),hero(&world,3));
    let victim=index(&world,2);world.write_resource::<MobaMatch>().heroes[victim].kill_streak=5;
    world.write_storage::<Gold>().get_mut(h2).unwrap().0=4500;
    assert_eq!(world.read_resource::<MobaMatch>().shutdown_bounty(2),300,"the positive streak term");
    flush(&mut world,[damage(h3,h2,1)]);
    flush(&mut world,[damage(h1,h2,100_000)]);
    // 100 + 300 + (5 - 2) * 100 + hoard (2 + 1) * 50; the assist 300 + 300.
    assert_eq!(balance(&world,1),kill.killer_gold(5,0,1,4500) as i32);
    assert_eq!(balance(&world,1),850);
    assert_eq!(balance(&world,3),kill.assist_gold(5,0) as i32);assert_eq!(balance(&world,3),600);
    assert_eq!((balance(&world,5),balance(&world,2)),(0,4500));
    {let state=world.read_resource::<MobaMatch>();
        assert_eq!((state.heroes[index(&world,1)].kills,state.heroes[index(&world,3)].assists),(1,1));}
    // A second kill within 15 s is the chain's second: +2 * 50; one more 16 s later starts over.
    world.write_resource::<MobaMatch>().elapsed=Fixed64::from_i32(kill.multi_kill_seconds as i32);
    let h4=hero(&world,4);flush(&mut world,[damage(h1,h4,100_000)]);
    assert_eq!(balance(&world,1),850+(kill.killer_gold(0,0,2,0) as i32));assert_eq!(kill.killer_gold(0,0,2,0),500);
    world.write_resource::<MobaMatch>().elapsed=Fixed64::from_i32(2*kill.multi_kill_seconds as i32+1);
    let h6=hero(&world,6);flush(&mut world,[damage(h1,h6,100_000)]);
    assert_eq!(balance(&world,1),850+500+400);
    // A victim on consecutive deaths is worth less: -50 per death down to -200.
    let mut world=fixture(false);
    let (h1,h2,h3)=(hero(&world,1),hero(&world,2),hero(&world,3));
    let victim=index(&world,2);world.write_resource::<MobaMatch>().heroes[victim].unanswered_deaths=3;
    flush(&mut world,[damage(h3,h2,1)]);flush(&mut world,[damage(h1,h2,100_000)]);
    assert_eq!((balance(&world,1),balance(&world,3)),(250,150));
    assert_eq!(world.read_resource::<MobaMatch>().heroes[victim].unanswered_deaths,4);
}

#[test]
fn structure_bounty_goes_to_a_hero_killer_or_splits_for_npc_kills_and_the_general_shares_with_allies() {
    let r=rules();
    let setup=|role:LaneRole,positions:[(i32,i32);3]|{
        let mut world=fixture(false);let target=world.read_resource::<MobaMatch>().towers[1].unwrap();
        world.write_resource::<MobaMatch>().units.get_mut(&entity_key(target)).unwrap().role=role;
        place(&mut world,target,0,0);
        for (player,(x,y)) in [1,3,5].into_iter().zip(positions) {let e=hero(&world,player);place(&mut world,e,x,y);}
        (world,target)
    };
    // A hero killer takes the whole bounty; nearby allies get nothing (towers, camps, base).
    for (role,gold) in [(LaneRole::Tower,r.building_tower_gold),(LaneRole::Inhibitor,r.building_inhibitor_gold),(LaneRole::Base,r.building_base_gold)] {
        let (world,target)=setup(role,[(0,0),(1200,0),(1600,0)]);
        record_moba_building_damage(&world,Some(hero(&world,1)),target,true);
        assert_eq!([1,3,5].map(|p|balance(&world,p)),[gold as i32,0,0],"{role:?}");
    }
    // The general: 1300 to the killer, and every other ally within 2000 of the killer 300 (N <= 2)
    // or 300 * N / 2 (N > 2, killer counted).
    let (world,target)=setup(LaneRole::NexusTower,[(0,0),(1200,0),(1600,0)]);
    record_moba_building_damage(&world,Some(hero(&world,1)),target,true);
    assert_eq!([1,3,5].map(|p|balance(&world,p)),[r.building_nexus_tower_gold as i32,450,450]);
    let (world,target)=setup(LaneRole::NexusTower,[(0,0),(1200,0),(2001,0)]);
    record_moba_building_damage(&world,Some(hero(&world,1)),target,true);
    assert_eq!([1,3,5].map(|p|balance(&world,p)),[1300,300,0]);
    // A non-hero killer: equal floor shares for the heroes within 1500 of the structure.
    for (role,gold) in [(LaneRole::Tower,r.building_tower_gold),(LaneRole::Inhibitor,r.building_inhibitor_gold)] {
        let (world,target)=setup(role,[(0,0),(1500,0),(1501,0)]);
        let tower=world.read_resource::<MobaMatch>().towers[0].unwrap();
        record_moba_building_damage(&world,Some(tower),target,true);
        assert_eq!([1,3,5].map(|p|balance(&world,p)),[(gold/2) as i32,(gold/2) as i32,0],"{role:?}");
    }
    assert_eq!((r.building_tower_gold,r.building_inhibitor_gold,r.building_base_gold,r.building_nexus_tower_gold),(150,300,50,1300));
}

#[test]
fn original_rules_have_no_tower_plating_or_first_tower_bonus() {
    let mut world=fixture(false);
    {let mut state=world.write_resource::<MobaMatch>();state.config.tower_plating_rules=Some(TowerPlatingRules::training());
        state.config.building_reward_rules=BuildingRewardRules::training();}
    assert!(world.read_resource::<MobaMatch>().config.active_tower_plating_rules().is_none());
    let target=world.read_resource::<MobaMatch>().towers[1].unwrap();let h1=hero(&world,1);
    place(&mut world,target,0,0);place(&mut world,h1,0,0);
    let hp=(world.read_storage::<CProperty>().get(target).unwrap().mhp.raw()/1024) as i32;
    flush(&mut world,[damage(h1,target,hp*3/4)]);
    assert_eq!(balance(&world,1),0,"no plate gold");
    assert!(!world.read_resource::<BuffStore>().has(target,"tower_plating"));
    flush(&mut world,[damage(h1,target,100_000)]);
    let mut state=world.remove::<MobaMatch>().unwrap();first_tower::settle(&world,&mut state);world.insert(state);
    assert_eq!(balance(&world,1),rules().building_tower_gold as i32,"no first-tower bonus");
    // The same config without the original rules keeps plating active.
    world.write_resource::<MobaMatch>().config.game_rules=None;
    assert!(world.read_resource::<MobaMatch>().config.active_tower_plating_rules().is_some());
}

#[test]
fn jungle_comeback_xp_follows_the_exact_enemy_average_level_for_ordinary_camps_only() {
    let r=rules();
    let camp=|world:&World,id:&str|{let state=world.read_resource::<MobaMatch>();
        let c=state.jungle_camps.iter().find(|c|c.definition.id==id).unwrap();
        (c.entity.unwrap(),c.definition.xp,c.definition.original_reward.map_or(0,|r|r.team_xp))};
    for (killer_level,id) in [(3,"north_guardian"),(6,"north_guardian"),(5,"north_guardian"),(3,"south_epic_guardian")] {
        let mut world=fixture(true);
        // Enemy levels 7, 4 and 5: average 5.33 (floor(5.33 - 3) = 2; 5.33 - 5 < 1 gives nothing).
        for (player,level) in [(1,killer_level),(2,7),(4,4),(6,5)] {set_level(&mut world,player,level);}
        let (target,xp,team_xp)=camp(&world,id);let before=total_xp(&world,1);
        let h1=hero(&world,1);flush(&mut world,[damage(h1,target,1_000_000)]);
        let below=if killer_level==3 {2} else {0};
        // A boss gives no comeback XP; its original reward adds the side's XP (E187).
        let expected=if id=="north_guardian" {xp+r.jungle_comeback_xp(xp,below)} else {xp+team_xp};
        assert_eq!(total_xp(&world,1)-before,i64::from(expected),"level {killer_level} {id}");
    }
    assert_eq!(r.jungle_comeback_xp(75,2),22);
}

#[test]
fn comeback_gold_never_activates_under_the_original_rules() {
    // The training objective comeback (gold for the side behind) has no counterpart in the
    // original game; with game_rules it stays off whatever the gap (E187).
    let rule:ObjectiveComebackRules=serde_json::from_str(omoba_template_ids::MOBA_OBJECTIVE_COMEBACK_JSON).unwrap();
    assert!(rule.is_valid() && rule.activate_gap>0);
    let mut world=fixture(true);
    let mut state=world.remove::<MobaMatch>().unwrap();
    state.config.objective_comeback_rules=rule;
    state.elapsed=state.config.warmup+Fixed64::from_i32(rule.starts_seconds as i32+60);
    state.earned_gold=[0,u64::from(rule.activate_gap)*10];
    objective_comeback::begin(&world,&mut state);
    assert!(state.objective_comeback().is_none());
    state.config.game_rules=None;
    objective_comeback::begin(&world,&mut state);
    assert_eq!(state.objective_comeback().map(|(side,_)|side),Some(0),"control: the training rule does activate");
    world.insert(state);
}
