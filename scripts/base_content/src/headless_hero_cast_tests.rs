//! The generated four-skill hero must survive the authoritative ECS cast and
//! outcome phases, not just the effect executor's isolated sink tests.

use omoba_core::runtime::comp::tower_registry::TowerTemplateRegistry;
use omoba_core::runtime::{
    process_outcomes, run_script_dispatch, RuntimeEventVecSink, ScriptEvent, ScriptEventQueue,
    ScriptRegistry, SkillTarget, StateInitializer,
};
use omoba_core::{CProperty, Faction, FactionType, Hero, Outcome, Pos};
use omoba_sim::Fixed64;
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

    assert_eq!(registry.ability_count(), crate::abilities().len());
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
