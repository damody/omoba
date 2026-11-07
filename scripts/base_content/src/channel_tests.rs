//! G-2 pulse channels with the real generated content and production step:
//! cast settlement, host-owned pulses through `on_channel_pulse`, and
//! interruption without a ghost pulse or a second charge.

use omoba_core::game_proto::{player_input::Action, CastAbility, PlayerInput, Vec2I};
use omoba_core::runtime::*;
use omoba_sim::{Fixed64, Vec2};
use specs::{Entity, World, WorldExt};

fn world() -> (World, SimulationDriver) {
    let pool = StateInitializer::create_thread_pool();
    let mut ecs = StateInitializer::setup_campaign_ecs_world(&pool);
    ecs.insert(TowerTemplateRegistry::default());
    populate_tower_upgrade_registry(&mut ecs);
    let mut scripts = ScriptRegistry::new();
    scripts.insert_manifest(crate::get_manifest());
    populate_ability_registry(&mut ecs, &scripts);
    ecs.insert(scripts);
    let config = SingleLaneConfig {
        seed: 0x20261025,
        warmup: Fixed64::ZERO,
        wave_interval: Fixed64::from_i32(1_000_000),
        heroes: ["training_channeler".into(), "training_luminary".into()],
        ..SingleLaneConfig::default()
    };
    setup_single_lane_match(&mut ecs, config).expect("match setup");
    let driver = SimulationDriver::from_world(&mut ecs, SimulationTickProfile::Production60Hz).unwrap();
    (ecs, driver)
}

fn hero(world: &World, player: u32) -> Entity {
    world.read_resource::<MobaMatch>().heroes.iter().find(|slot| slot.player_id == player).and_then(|slot| slot.entity).expect("hero")
}

fn hp(world: &World, entity: Entity) -> Fixed64 {
    world.read_storage::<CProperty>().get(entity).unwrap().hp
}

fn stunned(world: &World, entity: Entity) -> bool {
    world.read_resource::<BuffStore>().is_stunned(entity)
}

/// Caster at the origin, a sturdy enemy hero 400 units away.
fn staged() -> (World, SimulationDriver, Entity, Entity) {
    let (mut world, mut driver) = world();
    driver.step(&mut world, []).unwrap();
    let (caster, enemy) = (hero(&world, 1), hero(&world, 2));
    world.write_storage::<Pos>().get_mut(caster).unwrap().0 = Vec2::ZERO;
    world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = Vec2::new(Fixed64::from_i32(400), Fixed64::ZERO);
    {
        let mut properties = world.write_storage::<CProperty>();
        let enemy_hp = properties.get_mut(enemy).unwrap();
        enemy_hp.mhp = Fixed64::from_i32(100_000);
        enemy_hp.hp = Fixed64::from_i32(100_000);
    }
    // Only channel pulses may hurt the enemy: neither hero auto-attacks here.
    world.write_storage::<TAttack>().remove(caster);
    world.write_storage::<TAttack>().remove(enemy);
    for entity in [caster, enemy] {
        let mut heroes = world.write_storage::<Hero>();
        let state = heroes.get_mut(entity).unwrap();
        let ids = state.abilities.clone();
        for id in ids { state.ability_levels.insert(id, 1); }
    }
    (world, driver, caster, enemy)
}

fn cast(world: &mut World, driver: &mut SimulationDriver, slot: u32, point: Option<Vec2>, unit: Option<Entity>) {
    let cast = CastAbility {
        ability_index: slot,
        target_pos: point.map(|p| Vec2I { x: p.x.raw() as i32, y: p.y.raw() as i32 }),
        target_entity: unit.map(|entity| entity.id()),
        ..Default::default()
    };
    driver.step(world, [(1, PlayerInput { action: Some(Action::CastAbility(cast)) })]).unwrap();
}

#[test]
fn tremor_pulses_damage_and_stun_on_the_fixed_clock_and_ends_after_its_pulse_count() {
    let id = omoba_template_ids::ability_by_name("lumen_tremor").unwrap();
    let definition = crate::get_manifest().abilities()().into_iter()
        .find(|row| serde_json::from_str::<omoba_core::ability_meta::AbilityDef>(row.def_json.as_str()).unwrap().id == id.as_str())
        .map(|row| serde_json::from_str::<omoba_core::ability_meta::AbilityDef>(row.def_json.as_str()).unwrap()).unwrap();
    assert!(matches!(definition.host_execution, Some(omoba_core::ability_meta::HostAbilityExecution::PulseChannel {..})));
    let (mut world, mut driver, caster, enemy) = staged();
    let before = hp(&world, enemy);
    cast(&mut world, &mut driver, 3, Some(Vec2::new(Fixed64::from_i32(400), Fixed64::ZERO)), None);
    assert!(world.read_resource::<BuffStore>().has(caster, "ability_channel"), "public channel bar");
    assert!(world.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("lumen_tremor"));
    // Each wave stuns 0.2 s, shorter than the 0.25 s spacing: one rising edge per wave.
    let mut waves = Vec::new();
    let mut was = stunned(&world, enemy);
    // 12 waves * 0.25 s = 3 s = 180 ticks; run past the end.
    for tick in 0..240 {
        driver.step(&mut world, []).unwrap();
        let now = stunned(&world, enemy);
        if now && !was { waves.push(tick); }
        was = now;
    }
    assert_eq!(waves.len(), 12, "one stun per wave: {waves:?}");
    for pair in waves.windows(2) { assert!((14..=16).contains(&(pair[1] - pair[0])), "0.25 s spacing: {waves:?}"); }
    assert!(hp(&world, enemy) < before);
    assert!(!world.read_resource::<BuffStore>().has(caster, "ability_channel"), "ends after the last wave");
}

#[test]
fn a_stunned_caster_stops_tremor_without_another_wave_and_tether_follows_its_unit() {
    let (mut world, mut driver, caster, enemy) = staged();
    cast(&mut world, &mut driver, 3, Some(Vec2::new(Fixed64::from_i32(400), Fixed64::ZERO)), None);
    let mut waves = 0;
    let mut was = false;
    while waves < 3 {
        driver.step(&mut world, []).unwrap();
        let now = stunned(&world, enemy);
        if now && !was { waves += 1; }
        was = now;
    }
    world.write_resource::<BuffStore>().add(caster, "stun", Fixed64::from_i32(1), serde_json::json!({}));
    let mut late = 0;
    for _ in 0..120 {
        driver.step(&mut world, []).unwrap();
        let now = stunned(&world, enemy);
        if now && !was { late += 1; }
        was = now;
    }
    assert_eq!(late, 0, "no ghost wave after the interruption");
    assert!(!world.read_resource::<BuffStore>().has(caster, "ability_channel"));

    // Unit anchor: the enemy walks; pulses keep landing on it until it dies.
    let (mut world, mut driver, caster, enemy) = staged();
    cast(&mut world, &mut driver, 1, None, Some(enemy));
    assert!(world.read_resource::<BuffStore>().has(caster, "ability_channel"));
    let start = hp(&world, enemy);
    for _ in 0..40 { driver.step(&mut world, []).unwrap(); }
    world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = Vec2::new(Fixed64::from_i32(450), Fixed64::from_i32(100));
    for _ in 0..40 { driver.step(&mut world, []).unwrap(); }
    assert!(hp(&world, enemy) < start, "tether damage lands");
    assert!(world.read_resource::<BuffStore>().has(caster, "ability_channel"), "a moving unit anchor keeps the channel");
    world.write_storage::<CProperty>().get_mut(enemy).unwrap().hp = Fixed64::ZERO;
    for _ in 0..5 { driver.step(&mut world, []).unwrap(); }
    assert!(!world.read_resource::<BuffStore>().has(caster, "ability_channel"), "a dead anchor ends the channel");
}
