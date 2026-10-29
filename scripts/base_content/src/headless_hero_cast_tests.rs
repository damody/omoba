//! The generated four-skill hero must survive the authoritative ECS cast and
//! outcome phases, not just the effect executor's isolated sink tests.

use omoba_core::ability_runtime::ManaPool;
use omoba_core::runtime::comp::tower_registry::TowerTemplateRegistry;
use omoba_core::runtime::{
    handle_ability_cast_from_input, process_outcomes, run_script_dispatch, RuntimeEventVecSink,
    ScriptEvent, ScriptEventQueue, ScriptRegistry, ScriptVisualEventKind, ScriptVisualEventQueue,
    SkillTarget, StateInitializer,
};
use omoba_core::{CProperty, Faction, FactionType, Hero, Outcome, PlayerOwner, Pos, TargetType};
use omoba_sim::{Fixed64, Vec2};
use specs::{Builder, Entity, World, WorldExt};

fn hp(world: &World, entity: Entity) -> Fixed64 {
    world.read_storage::<CProperty>().get(entity).unwrap().hp
}

fn cast(
    world: &mut World,
    registry: &ScriptRegistry,
    caster: Entity,
    skill_id: &str,
    target: SkillTarget,
) {
    world
        .write_resource::<ScriptEventQueue>()
        .push(ScriptEvent::SkillCast {
            caster,
            skill_id: skill_id.to_owned(),
            target,
        });
    run_script_dispatch(world, registry, 41, Fixed64::ZERO);
    assert!(
        !world.read_resource::<Vec<Outcome>>().is_empty(),
        "{skill_id} must emit an authoritative outcome"
    );
    process_outcomes(world, &mut RuntimeEventVecSink::default()).unwrap();
    world.maintain();
}

#[test]
fn lua_generated_four_skill_hero_casts_and_settles_in_headless_match() {
    let pool = StateInitializer::create_thread_pool();
    let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
    world.insert(TowerTemplateRegistry::default());
    let mut registry = ScriptRegistry::new();
    registry.insert_manifest(crate::get_manifest());

    let ability_ids = ["lumen_bolt", "lumen_touch", "lumen_lance", "lumen_mend"];
    let mut hero = Hero::new(
        "training_luminary".to_owned(),
        "Training Luminary".to_owned(),
        String::new(),
    );
    hero.abilities = ability_ids.iter().map(|id| (*id).to_owned()).collect();
    hero.ability_levels = ability_ids.iter().map(|id| ((*id).to_owned(), 1)).collect();
    let caster = world
        .create_entity()
        .with(Pos::from_xy_f32(0.0, 0.0))
        .with(Faction::new(FactionType::Player, 0))
        .with(CProperty {
            hp: Fixed64::from_i32(400),
            mhp: Fixed64::from_i32(500),
            msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO,
            def_magic: Fixed64::ZERO,
        })
        .with(hero)
        .build();
    let enemy = world
        .create_entity()
        .with(Pos::from_xy_f32(10.0, 0.0))
        .with(Faction::new(FactionType::Enemy, 1))
        .with(CProperty {
            hp: Fixed64::from_i32(1000),
            mhp: Fixed64::from_i32(1000),
            msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO,
            def_magic: Fixed64::ZERO,
        })
        .build();

    // get_manifest() also registers the hero DLL crates (hero_dlls.lua and test_only heroes).
    assert_eq!(registry.ability_count(), (crate::get_manifest().abilities())().len());
    for ability_id in ability_ids {
        assert!(registry.get_ability(ability_id).is_some(), "{ability_id}");
    }

    cast(
        &mut world,
        &registry,
        caster,
        "lumen_bolt",
        SkillTarget::Entity(enemy),
    );
    assert_eq!(hp(&world, enemy), Fixed64::from_i32(920));
    cast(
        &mut world,
        &registry,
        caster,
        "lumen_touch",
        SkillTarget::None,
    );
    assert_eq!(hp(&world, caster), Fixed64::from_i32(470));
    cast(
        &mut world,
        &registry,
        caster,
        "lumen_lance",
        SkillTarget::Entity(enemy),
    );
    assert_eq!(hp(&world, enemy), Fixed64::from_i32(740));
    cast(
        &mut world,
        &registry,
        caster,
        "lumen_mend",
        SkillTarget::None,
    );
    assert_eq!(hp(&world, caster), Fixed64::from_i32(500));

    // The same authoritative adapter must reject allies without producing damage.
    let ally = world
        .create_entity()
        .with(Pos::from_xy_f32(5.0, 0.0))
        .with(Faction::new(FactionType::Player, 0))
        .with(CProperty {
            hp: Fixed64::from_i32(500),
            mhp: Fixed64::from_i32(500),
            msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO,
            def_magic: Fixed64::ZERO,
        })
        .build();
    // A previously used skill is on cooldown. Clear only that cooldown so
    // this cast reaches the effect executor's enemy preflight.
    world
        .write_storage::<Hero>()
        .get_mut(caster)
        .unwrap()
        .ability_cooldowns
        .remove("lumen_lance");
    world
        .write_resource::<ScriptEventQueue>()
        .push(ScriptEvent::SkillCast {
            caster,
            skill_id: "lumen_lance".to_owned(),
            target: SkillTarget::Entity(ally),
        });
    run_script_dispatch(&mut world, &registry, 42, Fixed64::ZERO);
    assert!(world.read_resource::<Vec<Outcome>>().is_empty());
    assert_eq!(hp(&world, ally), Fixed64::from_i32(500));
    assert_eq!(
        world
            .read_storage::<Hero>()
            .get(caster)
            .unwrap()
            .get_cooldown("lumen_lance"),
        Fixed64::ZERO
    );
}

fn mana(world: &World, entity: Entity) -> Fixed64 {
    world
        .read_storage::<Hero>()
        .get(entity)
        .unwrap()
        .mana_pool
        .as_ref()
        .unwrap()
        .current()
}

fn cooldown(world: &World, entity: Entity, ability_id: &str) -> Fixed64 {
    world
        .read_storage::<Hero>()
        .get(entity)
        .unwrap()
        .get_cooldown(ability_id)
}

fn skill_cast_cues(world: &mut World) -> Vec<(String, Option<Entity>)> {
    world
        .write_resource::<ScriptVisualEventQueue>()
        .drain()
        .into_iter()
        .filter(|event| event.kind == ScriptVisualEventKind::SkillCast)
        .map(|event| (event.skill_id.unwrap_or_default(), event.secondary))
        .collect()
}

/// The public player-input handler always enqueues whatever position or entity
/// the client attached. Settlement is the same later dispatch and outcome pass
/// used by a match, not a direct `SkillTarget::None` insert.
fn settle_player_cast(
    world: &mut World,
    registry: &ScriptRegistry,
    slot: u32,
    target_pos: Option<Vec2>,
    target_entity: Option<u32>,
) {
    handle_ability_cast_from_input(world, slot, target_pos, target_entity, 1).unwrap();
    run_script_dispatch(world, registry, 43, Fixed64::ZERO);
    process_outcomes(world, &mut RuntimeEventVecSink::default()).unwrap();
    world.maintain();
}

#[test]
fn player_input_no_target_cast_normalizes_before_settlement() {
    // Source-level route: AbilityCast supplies a position even for a no-target
    // skill, and a live entity id wins over that position in the input handler.
    // This is not a replay of the missing tick-83725 capture.
    let pool = StateInitializer::create_thread_pool();
    let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
    world.insert(TowerTemplateRegistry::default());
    let mut registry = ScriptRegistry::new();
    registry.insert_manifest(crate::get_manifest());
    assert_eq!(
        registry.get_ability("lumen_bolt").unwrap().0.target_type,
        TargetType::Unit
    );
    assert_eq!(
        registry.get_ability("ranger_volley").unwrap().0.target_type,
        TargetType::Point
    );
    assert_eq!(
        registry.get_ability("lumen_touch").unwrap().0.target_type,
        TargetType::None
    );
    assert_eq!(
        registry.get_ability("lumen_mend").unwrap().0.target_type,
        TargetType::None
    );
    assert!(registry.get_ability("unregistered_cast").is_none());

    let ability_ids = ["lumen_bolt", "ranger_volley", "lumen_touch", "lumen_mend"];
    let mut hero = Hero::new(
        "training_luminary".to_owned(),
        "Training Luminary".to_owned(),
        String::new(),
    );
    hero.abilities = ability_ids.iter().map(|id| (*id).to_owned()).collect();
    hero.ability_levels = ability_ids.iter().map(|id| ((*id).to_owned(), 1)).collect();
    hero.mana_pool = Some(ManaPool::new(Fixed64::from_i32(400), Fixed64::from_i32(800)).unwrap());
    let caster = world
        .create_entity()
        .with(Pos::from_xy_f32(0.0, 0.0))
        .with(Faction::new(FactionType::Player, 0))
        .with(PlayerOwner::new(1))
        .with(CProperty {
            hp: Fixed64::from_i32(200),
            mhp: Fixed64::from_i32(500),
            msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO,
            def_magic: Fixed64::ZERO,
        })
        .with(hero)
        .build();
    let enemy = world
        .create_entity()
        .with(Pos::from_xy_f32(0.0, 0.0))
        .with(Faction::new(FactionType::Enemy, 1))
        .with(CProperty {
            hp: Fixed64::from_i32(1000),
            mhp: Fixed64::from_i32(1000),
            msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO,
            def_magic: Fixed64::ZERO,
        })
        .build();
    let _ = world.write_resource::<ScriptVisualEventQueue>().drain();

    let away = Vec2::new(Fixed64::from_i32(48), Fixed64::from_i32(-17));
    settle_player_cast(&mut world, &registry, 0, Some(away), None);
    assert_eq!(hp(&world, enemy), Fixed64::from_i32(1000));
    assert_eq!(hp(&world, caster), Fixed64::from_i32(200));
    assert_eq!(mana(&world, caster), Fixed64::from_i32(400));
    assert_eq!(cooldown(&world, caster, "lumen_bolt"), Fixed64::ZERO);
    assert!(skill_cast_cues(&mut world).is_empty());

    settle_player_cast(&mut world, &registry, 0, Some(away), Some(enemy.id()));
    assert_eq!(hp(&world, enemy), Fixed64::from_i32(920));
    assert_eq!(mana(&world, caster), Fixed64::from_i32(355));
    assert_eq!(cooldown(&world, caster, "lumen_bolt"), Fixed64::from_i32(7));
    assert_eq!(
        skill_cast_cues(&mut world),
        vec![("lumen_bolt".to_owned(), Some(enemy))]
    );

    settle_player_cast(&mut world, &registry, 1, Some(Vec2::ZERO), None);
    assert_eq!(hp(&world, enemy), Fixed64::from_i32(810));
    assert_eq!(hp(&world, caster), Fixed64::from_i32(200));
    assert_eq!(mana(&world, caster), Fixed64::from_i32(310));
    assert_eq!(cooldown(&world, caster, "ranger_volley"), Fixed64::from_i32(11));
    assert_eq!(
        skill_cast_cues(&mut world),
        vec![("ranger_volley".to_owned(), None)]
    );

    settle_player_cast(&mut world, &registry, 2, Some(away), None);
    assert_eq!(hp(&world, caster), Fixed64::from_i32(270));
    assert_eq!(hp(&world, enemy), Fixed64::from_i32(810));
    assert_eq!(mana(&world, caster), Fixed64::from_i32(275));
    assert_eq!(cooldown(&world, caster, "lumen_touch"), Fixed64::from_i32(12));
    assert_eq!(
        skill_cast_cues(&mut world),
        vec![("lumen_touch".to_owned(), None)]
    );

    settle_player_cast(&mut world, &registry, 3, Some(away), Some(enemy.id()));
    assert_eq!(hp(&world, caster), Fixed64::from_i32(410));
    assert_eq!(hp(&world, enemy), Fixed64::from_i32(810));
    assert_eq!(mana(&world, caster), Fixed64::from_i32(185));
    assert_eq!(cooldown(&world, caster, "lumen_mend"), Fixed64::from_i32(25));
    assert_eq!(
        skill_cast_cues(&mut world),
        vec![("lumen_mend".to_owned(), None)]
    );

    {
        let mut heroes = world.write_storage::<Hero>();
        let hero = heroes.get_mut(caster).unwrap();
        hero.abilities[0] = "unregistered_cast".to_owned();
        hero.ability_levels.insert("unregistered_cast".to_owned(), 1);
    }
    settle_player_cast(&mut world, &registry, 0, Some(away), Some(enemy.id()));
    assert_eq!(hp(&world, caster), Fixed64::from_i32(410));
    assert_eq!(hp(&world, enemy), Fixed64::from_i32(810));
    assert_eq!(mana(&world, caster), Fixed64::from_i32(185));
    assert_eq!(cooldown(&world, caster, "unregistered_cast"), Fixed64::ZERO);
    assert_eq!(cooldown(&world, caster, "lumen_bolt"), Fixed64::from_i32(7));
    assert!(skill_cast_cues(&mut world).is_empty());
}
