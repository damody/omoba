use super::*;
use crate::runtime::brush_vision::{BrushVisionRegion,BrushVisionRegions};
use crate::runtime::{StateInitializer,VisibilityOverride,VisibilityOverrideKind,StealthProfile,VisionSource};
use crate::runtime::visibility::{build_wave_b_read_view,entity_visible_with_brush};

fn p(x:i32,y:i32)->Vec2 {Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y))}
fn fixture()->(World,Entity,Entity) {
    let pool=StateInitializer::create_thread_pool();
    let mut world=StateInitializer::setup_campaign_ecs_world(&pool);
    setup_single_lane_match(&mut world,SingleLaneConfig {warmup:Fixed64::ZERO,
        wave_interval:Fixed64::from_i32(10000),..SingleLaneConfig::default()}).unwrap();
    let (attacker,victim)={let mut state=world.write_resource::<MobaMatch>();
        state.phase=MobaMatchPhase::Playing;
        (state.heroes[0].entity.unwrap(),state.heroes[1].entity.unwrap())};
    world.write_storage::<Pos>().get_mut(attacker).unwrap().0=p(15,0);
    world.write_storage::<Pos>().get_mut(victim).unwrap().0=p(0,0);
    world.write_storage::<VisionSource>().clear();
    world.write_storage::<VisionSource>().insert(victim,VisionSource {team:2,
        radius:Fixed64::from_i32(100),detection_level:0}).unwrap();
    world.insert(BrushVisionRegions::from(vec![BrushVisionRegion {min:p(10,-5),max:p(20,5)}]));
    world.insert(crate::runtime::VisionOccluderSet::default());
    (world,attacker,victim)
}
fn visible(world:&World,attacker:Entity,team:u32)->bool {
    let view=build_wave_b_read_view(world,1);
    view.entities.iter().find(|e|e.canonical_id==crate::runtime::canonical_entity_id(attacker)).is_some_and(|entity|
        entity_visible_with_brush(entity,team,&view.vision_sources,&view.vision_occluders,&view.brush_regions))
}

#[test]
fn admitted_native_and_script_attacks_reveal_refresh_and_expire_without_wall_time() {
    for script in [false,true] {
        let (mut world,attacker,victim)=fixture();
        world.create_entity().with(Pos(p(0,0))).with(VisionSource {team:3,
            radius:Fixed64::from_i32(100),detection_level:0}).build();
        assert!(!visible(&world,attacker,2));
        let cue=if script {Outcome::ScriptAttackPhaseCue {entity:attacker,target:Some(victim),target_pos:None,
            windup_ms:100,backswing_ms:100,dir_rad:0.0}}
        else {Outcome::AttackPhaseCue {entity:attacker,attack_seq:1,is_critical:false,target:Some(victim),target_pos:None,
            windup_ms:100,backswing_ms:100,dir_rad:0.0}};
        world.write_resource::<Vec<Outcome>>().push(cue);
        super::super::game_processor::process_outcomes(&mut world,&mut Vec::new()).unwrap();
        assert!(visible(&world,attacker,2));
        assert!(super::super::game_processor::item_entity_visible_to_team(&world,attacker,2));
        assert!(!visible(&world,attacker,3),"grant is not global sight");
        world.write_resource::<MobaMatch>().elapsed=Fixed64::ONE;
        record_brush_attack_reveal(&world,attacker,victim);
        world.write_resource::<MobaMatch>().elapsed=Fixed64::from_i32(2);
        assert!(visible(&world,attacker,2),"second admission refreshes the deadline");
        world.write_resource::<MobaMatch>().elapsed=Fixed64::from_i32(3);
        assert!(!visible(&world,attacker,2),"closed expiry uses simulation time");
        assert!(!super::super::game_processor::item_entity_visible_to_team(&world,attacker,2));
    }
}

#[test]
fn reveal_keeps_range_los_stealth_denials_and_other_brush_occupants_private() {
    let (mut world,attacker,victim)=fixture();
    record_brush_attack_reveal(&world,attacker,victim);
    assert!(visible(&world,attacker,2));
    world.write_storage::<VisionSource>().get_mut(victim).unwrap().radius=Fixed64::ONE;
    assert!(!visible(&world,attacker,2));
    assert!(!super::super::game_processor::item_entity_visible_to_team(&world,attacker,2));
    world.write_storage::<VisionSource>().get_mut(victim).unwrap().radius=Fixed64::from_i32(100);
    world.write_storage::<StealthProfile>().insert(attacker,StealthProfile {stealth_level:1}).unwrap();
    assert!(!visible(&world,attacker,2));
    assert!(!super::super::game_processor::item_entity_visible_to_team(&world,attacker,2));
    world.write_storage::<StealthProfile>().remove(attacker);
    const WALLS:[omoba_template_ids::MobaTerrainConst;1]=[omoba_template_ids::MobaTerrainConst {id:"wall",min:(4,-3),max:(6,3)}];
    world.insert(crate::runtime::moba_map_layout::compiled_vision_occluders(&omoba_template_ids::MobaMapConst {
        terrain:&WALLS,..omoba_template_ids::MOBA_MAPS[0]}));
    assert!(!visible(&world,attacker,2));
    assert!(!super::super::game_processor::item_entity_visible_to_team(&world,attacker,2));
    world.insert(crate::runtime::VisionOccluderSet::default());
    world.write_storage::<VisibilityOverride>().insert(attacker,VisibilityOverride {team:Some(2),
        kind:VisibilityOverrideKind::ForceHide,priority:0,stable_rule_id:1}).unwrap();
    assert!(!visible(&world,attacker,2));
    assert!(!super::super::game_processor::item_entity_visible_to_team(&world,attacker,2));
    world.write_storage::<VisibilityOverride>().remove(attacker);
    world.write_storage::<ReplicationScope>().get_mut(attacker).unwrap().kind=ReplicationScopeKind::ServerOnly;
    assert!(!visible(&world,attacker,2));
    assert!(!super::super::game_processor::item_entity_visible_to_team(&world,attacker,2));
    world.write_storage::<ReplicationScope>().get_mut(attacker).unwrap().kind=ReplicationScopeKind::OwnerTeam;
    let occupant=world.create_entity().with(Pos(p(15,1))).build();
    assert!(!visible(&world,occupant,2));
    assert!(!super::super::game_processor::item_entity_visible_to_team(&world,occupant,2));
    assert!(visible(&world,attacker,2));
}

#[test]
fn grants_freeze_while_paused_retire_on_death_and_participate_in_digest() {
    let (mut world,attacker,victim)=fixture();
    let before=single_lane_replay_digest(&world);
    record_brush_attack_reveal(&world,attacker,victim);
    assert_ne!(single_lane_replay_digest(&world),before);
    let after=single_lane_replay_digest(&world);
    assert_eq!(single_lane_replay_digest(&world),after);
    world.write_resource::<GamePause>().is_paused=true;
    world.write_resource::<DeltaTime>().0=Fixed64::from_i32(10);
    assert!(!begin_moba_match_tick(&mut world));
    assert_eq!(world.read_resource::<MobaMatch>().elapsed,Fixed64::ZERO);
    assert!(visible(&world,attacker,2));
    world.write_resource::<GamePause>().is_paused=false;
    world.write_resource::<DeltaTime>().0=Fixed64::ONE;
    assert!(begin_moba_match_tick(&mut world));
    assert!(visible(&world,attacker,2));
    assert!(begin_moba_match_tick(&mut world));
    assert!(!visible(&world,attacker,2));
    assert!(!world.read_resource::<MobaMatch>().brush_attack_reveals.keys().any(|(key,_)|*key==entity_key(attacker)));
    record_brush_attack_reveal(&world,attacker,victim);
    record_moba_death(&mut world,attacker);
    assert!(!world.read_resource::<MobaMatch>().brush_reveal_teams(attacker).any(|_|true));
}

#[test]
fn invalid_neutral_allied_outside_brush_and_inactive_attacks_never_grant_sight() {
    for case in 0..7 {
        let (world,attacker,victim)=fixture();
        match case {
            0=>world.write_storage::<Faction>().get_mut(victim).unwrap().team_id=0,
            1=>world.write_storage::<Faction>().get_mut(victim).unwrap().team_id=1,
            2=>world.write_storage::<Pos>().get_mut(attacker).unwrap().0=p(30,0),
            3=>world.write_storage::<CProperty>().get_mut(attacker).unwrap().hp=Fixed64::ZERO,
            4=>world.write_storage::<CProperty>().get_mut(victim).unwrap().hp=Fixed64::ZERO,
            5=>world.write_resource::<GamePause>().is_paused=true,
            _=>world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Warmup,
        }
        record_brush_attack_reveal(&world,attacker,victim);
        assert!(world.read_resource::<MobaMatch>().brush_attack_reveals.is_empty(),"case {case}");
    }
}

#[test]
fn lane_ai_actual_attack_admission_uses_the_same_reveal_policy() {
    let (mut world,attacker,victim)=fixture();
    {let mut state=world.write_resource::<MobaMatch>();
        state.units.retain(|_,unit|unit.entity==attacker || unit.entity==victim);
        let unit=state.units.get_mut(&entity_key(attacker)).unwrap();
        unit.role=LaneRole::Creep;unit.attack_remaining=Fixed64::ZERO;
    }
    world.write_resource::<DeltaTime>().0=Fixed64::from_raw(1);
    assert!(!visible(&world,attacker,2));
    assert!(begin_moba_match_tick(&mut world));
    assert_eq!(world.read_resource::<MobaMatch>().units[&entity_key(attacker)].attack_sequence,1);
    assert!(visible(&world,attacker,2));
}

#[test]
fn independent_worlds_agree_and_private_refresh_deadlines_change_replay_digest() {
    let (a,attacker_a,victim_a)=fixture();
    let (b,attacker_b,victim_b)=fixture();
    assert_eq!(single_lane_replay_digest(&a),single_lane_replay_digest(&b));
    record_brush_attack_reveal(&a,attacker_a,victim_a);
    record_brush_attack_reveal(&b,attacker_b,victim_b);
    assert_eq!(single_lane_replay_digest(&a),single_lane_replay_digest(&b));
    a.write_resource::<MobaMatch>().elapsed=Fixed64::ONE;
    b.write_resource::<MobaMatch>().elapsed=Fixed64::ONE;
    record_brush_attack_reveal(&a,attacker_a,victim_a);
    assert_ne!(single_lane_replay_digest(&a),single_lane_replay_digest(&b));
    record_brush_attack_reveal(&b,attacker_b,victim_b);
    assert_eq!(single_lane_replay_digest(&a),single_lane_replay_digest(&b));
}

#[test]
fn real_neutral_jungle_ai_attack_in_brush_exposes_only_its_source_to_defender() {
    let (mut world,_,victim)=fixture();
    let mut state=world.remove::<MobaMatch>().unwrap();
    state.config.map_id=Some("three_lane_training".into());
    jungle::setup(&mut world,&mut state);
    let attacker=state.jungle_camps[0].entity.unwrap();
    let position=world.read_storage::<Pos>().get(attacker).unwrap().0;
    assert_eq!(world.read_storage::<Faction>().get(attacker).unwrap().team_id,0);
    world.write_storage::<Pos>().get_mut(victim).unwrap().0=position+p(30,0);
    world.insert(BrushVisionRegions::from(vec![BrushVisionRegion {min:position-p(10,10),max:position+p(10,10)}]));
    world.insert(state);
    jungle::record_moba_jungle_damage(&world,Some(victim),attacker,false);
    assert!(!visible(&world,attacker,2));
    let mut state=world.remove::<MobaMatch>().unwrap();
    jungle::tick(&mut world,&mut state,Fixed64::from_raw(1));
    assert!(!world.read_resource::<Vec<DamageInstance>>().is_empty(),"real camp AI admitted its basic attack");
    world.insert(state);
    assert!(visible(&world,attacker,2));
    assert!(super::super::game_processor::item_entity_visible_to_team(&world,attacker,2));
    world.write_resource::<MobaMatch>().elapsed=BRUSH_ATTACK_REVEAL_DURATION;
    assert!(!visible(&world,attacker,2));
}

#[test]
fn invisibility_buff_keeps_an_attacking_hero_hidden_until_detected() {
    // H05 inahime R: the timed invisibility buff (BuffStore::stealth_level), not only StealthProfile,
    // denies the brush attack reveal; attacking never removes it.
    let (mut world,attacker,victim)=fixture();
    world.write_resource::<crate::runtime::BuffStore>().add(attacker,"stealth_test",Fixed64::from_i32(20),
        serde_json::json!({"invisibility_level":1024}));
    record_brush_attack_reveal(&world,attacker,victim);
    assert!(!visible(&world,attacker,2),"an invisible attacker stays hidden after attacking");
    assert!(!super::super::game_processor::item_entity_visible_to_team(&world,attacker,2));
    assert_eq!(world.read_resource::<crate::runtime::BuffStore>().stealth_level(attacker),1,"the attack kept the buff");
    world.write_storage::<VisionSource>().get_mut(victim).unwrap().detection_level=1;
    assert!(visible(&world,attacker,2),"detection at the stealth level reveals the attacker");
}
