//! Real match, visibility, damage, and acquisition paths for placed trinkets.
//! These tests install an explicit catalog id. They do not infer one from row 0.

use omoba_core::comp::run_now;
use omoba_core::runtime::combat_acquisition::AdmittedCombatPreferences;
use omoba_core::runtime::combat_controls::{AttackAcquisition, CombatPreferenceUpdate, OwnerCombatPreferences};
use omoba_core::runtime::*;
use omoba_sim::{Fixed64, Vec2};
use omoba_template_ids::TrinketConst;
use specs::{Builder, Entity, Join, World, WorldExt};

fn catalog_row() -> &'static TrinketConst {
    let named = omoba_template_ids::trinket_by_name("training_scout_lantern_v1")
        .expect("accepted training trinket");
    omoba_template_ids::TRINKET_CATALOG
        .iter()
        .find(|row| row.catalog_id == named.catalog_id)
        .expect("catalog id")
}

#[test]
fn detection_trinket_obeys_level_radius_occlusion_team_and_retirement() {
    let row = omoba_template_ids::trinket_by_name("training_detection_lantern_v1").unwrap();
    let (mut world, mut driver) = world();
    driver.step(&mut world, []).unwrap();
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let target = pos_of(&world, hero(&world, 1)) + Vec2::new(Fixed64::ZERO, row.placement_range);
    let mut hidden = |offset: Vec2, level, team| {
        world.create_entity().with(Pos(target + offset))
            .with(ReplicationScope { kind: ReplicationScopeKind::Vision, owner_team: Some(team) })
            .with(StealthProfile { stealth_level: level })
            .with(Faction::new(if team == 1 {FactionType::Player} else {FactionType::Enemy}, i32::try_from(team).unwrap())).build()
    };
    let red1 = hidden(Vec2::new(Fixed64::ZERO, Fixed64::from_i32(100)), 1, 2);
    let red2 = hidden(Vec2::new(Fixed64::ZERO, Fixed64::from_i32(100)), 2, 2);
    let red3 = hidden(Vec2::new(Fixed64::ZERO, Fixed64::from_i32(100)), 3, 2);
    let edge = hidden(Vec2::new(row.sight_radius, Fixed64::ZERO), 2, 2);
    let outside = hidden(Vec2::new(row.sight_radius + Fixed64::from_raw(1), Fixed64::ZERO), 2, 2);
    let blue = hidden(Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO), 2, 1);
    project(&mut world);
    assert!(!sees(&world, 1, red2));
    let placed = place(&mut world, 1, target);
    let ward = entity_at(placed.entity_id, placed.generation);
    world.create_entity().with(Pos(target)).with(VisionSource {
        team: 2, radius: row.sight_radius, detection_level: 0 }).build();
    project(&mut world);
    assert!(sees(&world, 1, red1) && sees(&world, 1, red2));
    assert!(!sees(&world, 1, red3));
    assert!(sees(&world, 1, edge));
    assert!(!sees(&world, 1, outside));
    assert!(sees(&world, 2, ward), "visible detector can be found without enemy detection");
    assert!(!sees(&world, 2, blue), "detector grants no detection to enemies");
    let center = target + Vec2::new(Fixed64::ZERO, Fixed64::from_i32(50));
    let radius = Fixed64::from_i32(20);let extent = Vec2::new(radius, radius);
    world.write_resource::<VisionOccluderSet>().0.push(VisionOccluder::Tree(VisionTreeCircle {
        stable_id: 78, center, radius, aabb: VisionAabb {min: center-extent, max: center+extent} }));
    project(&mut world);
    assert!(!sees(&world, 1, red2), "detection does not bypass terrain");
    world.write_resource::<VisionOccluderSet>().0.clear();project(&mut world);
    assert!(sees(&world, 1, red2));
    let killer = hero(&world, 2);kill_with_damage(&mut world, killer, ward);project(&mut world);
    assert!(!generation_alive(&world, ward));
    assert!(!sees(&world, 1, red2), "retirement removes detection immediately");
}

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
        seed: 0x20261010,
        warmup: Fixed64::ZERO,
        wave_interval: Fixed64::from_i32(1_000_000),
        passive_gold_per_second: 0,
        lane_creep_gold: 0,
        respawn_delay: Fixed64::from_raw(1),
        ..SingleLaneConfig::default()
    };
    setup_single_lane_match(&mut ecs, config).expect("match setup");
    let driver = SimulationDriver::from_world(&mut ecs, SimulationTickProfile::Production60Hz).unwrap();
    (ecs, driver)
}

fn row_of(world: &World, player: u32) -> WardRosterState {
    copied_ward_roster(world, player).expect("roster slot")
}

fn hero(world: &World, player: u32) -> Entity {
    world
        .read_resource::<MobaMatch>()
        .heroes
        .iter()
        .find(|slot| slot.player_id == player)
        .and_then(|slot| slot.entity)
        .expect("living hero")
}

fn pos_of(world: &World, entity: Entity) -> Vec2 {
    world.read_storage::<Pos>().get(entity).expect("position").0
}

fn generation(entity: Entity) -> u32 {
    entity.gen().id() as u32
}

/// Rebuild the entity saved at placement. `entities().entity(id)` after delete
/// and maintain returns the slot's negative generation, and `WorldExt::is_alive`
/// panics on that handle. Allocator liveness compares this positive generation.
fn entity_at(entity_id: u32, generation: u32) -> Entity {
    let generation = i32::try_from(generation).expect("ward generation");
    assert!(generation > 0, "a placed ward generation is positive");
    Entity::new(entity_id, specs::world::Generation::new(generation))
}

fn generation_alive(world: &World, entity: Entity) -> bool {
    entity.gen().is_alive() && world.entities().is_alive(entity)
}

fn request(entity: Entity, target: Vec2) -> WardPlacementRequest {
    WardPlacementRequest {
        local_slot: WARD_LOCAL_SLOT,
        source_generation: generation(entity),
        target,
    }
}

fn place(world: &mut World, player: u32, target: Vec2) -> WardPlacementReceipt {
    let entity = hero(world, player);
    settle_ward_placement(world, player, request(entity, target)).expect("placement")
}

fn project(world: &mut World) {
    let tick = world.read_resource::<Tick>().0;
    run_committed_visibility_wave_b(world, tick, 0);
}

fn sees(world: &World, team: u32, entity: Entity) -> bool {
    let id = canonical_entity_id(entity);
    world
        .read_resource::<TeamVisibilityRuntime>()
        .teams
        .get(&team)
        .is_some_and(|state| state.index.current.contains(&id))
}

fn probe_at(world: &mut World, at: Vec2) -> Entity {
    world
        .create_entity()
        .with(Pos(at))
        .with(ReplicationScope {
            kind: ReplicationScopeKind::Vision,
            owner_team: None,
        })
        .with(StealthProfile { stealth_level: 0 })
        .with(Faction::new(FactionType::Neutral, 0))
        .build()
}

fn vision_count(world: &World) -> usize {
    (&world.entities(), &world.read_storage::<VisionSource>()).join().count()
}

fn fingerprint(world: &World, player: u32) -> String {
    let roster = row_of(world, player);
    format!(
        "{}|{:?}|{}|{}|{}|{}|{:?}",
        single_lane_replay_digest(world),
        roster.catalog_id(),
        roster.charges(),
        roster.next_recharge_at().map(|time| time.raw()).unwrap_or(0),
        roster.revision(),
        vision_count(world),
        roster.placements()
    )
}

fn advance(world: &mut World, seconds: i32) {
    world.write_resource::<DeltaTime>().0 = Fixed64::from_i32(seconds);
    assert!(begin_moba_match_tick(world));
    world.maintain();
}

fn kill_with_damage(world: &mut World, source: Entity, target: Entity) {
    let pos = pos_of(world, target);
    world.write_resource::<Vec<Outcome>>().push(Outcome::Damage {
        pos,
        phys: Fixed64::ZERO,
        magi: Fixed64::ZERO,
        real: Fixed64::from_i32(10),
        source,
        target,
        damage_profile: 0,
        predeclared: false,
    });
    let mut sink = RuntimeEventVecSink::default();
    process_outcomes(world, &mut sink).unwrap();
    let mut sink = RuntimeEventVecSink::default();
    process_outcomes(world, &mut sink).unwrap();
    world.maintain();
}

#[test]
fn allied_sight_appears_after_real_placement_and_clears_on_expiry_death_and_owner_teardown() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    driver.step(&mut world, []).unwrap();
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let origin = pos_of(&world, hero(&world, 1));
    let target = origin + Vec2::new(Fixed64::ZERO, row.placement_range);
    let probe = probe_at(&mut world, origin + Vec2::new(Fixed64::ZERO, Fixed64::from_i32(1400)));
    project(&mut world);
    assert!(!sees(&world, 1, probe), "hero and tower sight must not already cover the probe");

    let placed = place(&mut world, 1, target);
    assert_eq!(placed.catalog_id, row.catalog_id);
    let ward = world.entities().entity(placed.entity_id);
    assert!(world.read_storage::<PlacedWard>().get(ward).is_some());
    assert!(world.read_storage::<Unit>().get(ward).is_none(), "a ward is not a unit");
    assert!(world.read_storage::<Creep>().get(ward).is_none());
    assert!(world.read_storage::<Hero>().get(ward).is_none());
    project(&mut world);
    assert!(sees(&world, 1, probe), "allied ward sight reveals the probe");
    assert!(!sees(&world, 2, ward), "enemy detection 0 does not reveal stealth");

    let charges = row_of(&world, 1).charges();
    retire_owner_wards(&mut world, 1);
    world.maintain();
    project(&mut world);
    assert!(!world.is_alive(ward));
    assert!(!sees(&world, 1, probe), "owner teardown removes vision in that call");
    assert_eq!(row_of(&world, 1).charges(), charges);

    advance(&mut world, 120);
    let placed = place(&mut world, 1, target);
    let ward = world.entities().entity(placed.entity_id);
    project(&mut world);
    assert!(sees(&world, 1, probe));
    advance(&mut world, 91);
    project(&mut world);
    assert!(!world.is_alive(ward), "lifetime expiry deletes the ward");
    assert!(!sees(&world, 1, probe), "expiry removes vision in the same match tick");

    advance(&mut world, 120);
    let placed = place(&mut world, 1, target);
    let ward = world.entities().entity(placed.entity_id);
    project(&mut world);
    assert!(sees(&world, 1, probe));
    let attacker = hero(&world, 2);
    kill_with_damage(&mut world, attacker, ward);
    project(&mut world);
    assert!(!world.is_alive(ward));
    assert!(world.read_storage::<VisionSource>().get(ward).is_none());
    assert!(!sees(&world, 1, probe), "ward death removes vision before the next match tick");
}

#[test]
fn occlusion_and_enemy_stealth_detection_isolate_ward_vision() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    driver.step(&mut world, []).unwrap();
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let origin = pos_of(&world, hero(&world, 1));
    let target = origin + Vec2::new(Fixed64::ZERO, row.placement_range);
    let probe = probe_at(&mut world, origin + Vec2::new(Fixed64::ZERO, Fixed64::from_i32(1400)));
    let placed = place(&mut world, 1, target);
    let ward = world.entities().entity(placed.entity_id);
    project(&mut world);
    assert!(sees(&world, 1, probe));

    let middle = origin + Vec2::new(Fixed64::ZERO, Fixed64::from_i32(1000));
    let radius = Fixed64::from_i32(80);
    let extent = Vec2::new(radius, radius);
    world.write_resource::<VisionOccluderSet>().0.push(VisionOccluder::Tree(VisionTreeCircle {
        stable_id: 77,
        center: middle,
        radius,
        aabb: VisionAabb { min: middle - extent, max: middle + extent },
    }));
    project(&mut world);
    assert!(!sees(&world, 1, probe), "a tree on the segment occludes ward sight");
    assert!(sees(&world, 1, ward), "owner-team scope still discloses the ward to its team");
    world.write_resource::<VisionOccluderSet>().0.clear();
    project(&mut world);
    assert!(sees(&world, 1, probe));

    let hidden = world
        .create_entity()
        .with(Pos(target + Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)))
        .with(ReplicationScope { kind: ReplicationScopeKind::Vision, owner_team: Some(2) })
        .with(StealthProfile { stealth_level: 1 })
        .with(Faction::new(FactionType::Enemy, 2))
        .build();
    project(&mut world);
    assert!(!sees(&world, 1, hidden), "ward detection 0 does not reveal enemy stealth");
    let blind = world
        .create_entity()
        .with(Pos(target))
        .with(VisionSource { team: 2, radius: Fixed64::from_i32(2000), detection_level: 0 })
        .build();
    project(&mut world);
    assert!(!sees(&world, 2, ward), "a zero-detection enemy source does not reveal the ward");
    world.write_storage::<VisionSource>().get_mut(blind).unwrap().detection_level = 1;
    let detector = world
        .create_entity()
        .with(Pos(target))
        .with(VisionSource { team: 1, radius: Fixed64::from_i32(200), detection_level: 1 })
        .build();
    project(&mut world);
    assert!(sees(&world, 2, ward), "detection that meets stealth reveals the ward");
    assert!(sees(&world, 1, hidden), "ally detection that meets stealth reveals the enemy");
    let _ = (detector, blind);
}

#[test]
fn rejected_placement_leaves_charges_timer_entities_order_and_vision_unchanged() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    let check = |world: &mut World, expected: WardReject, call: &mut dyn FnMut(&mut World) -> WardReject| {
        let before = fingerprint(world, 1);
        let entities = world.entities().join().count();
        assert_eq!(call(world), expected);
        assert_eq!(fingerprint(world, 1), before);
        assert_eq!(world.entities().join().count(), entities);
    };

    check(&mut world, WardReject::NotPlaying, &mut |world| {
        settle_ward_placement(world, 1, WardPlacementRequest {
            local_slot: WARD_LOCAL_SLOT,
            source_generation: 1,
            target: Vec2::new(Fixed64::from_i32(120), Fixed64::from_i32(600)),
        }).unwrap_err()
    });
    check(&mut world, WardReject::WrongSlot, &mut |world| {
        settle_ward_placement(world, 1, WardPlacementRequest {
            local_slot: 1,
            source_generation: 1,
            target: Vec2::new(Fixed64::ZERO, Fixed64::ZERO),
        }).unwrap_err()
    });
    driver.step(&mut world, []).unwrap();
    check(&mut world, WardReject::Unequipped, &mut |world| {
        let hero = hero(world, 1);
        let request = request(hero, pos_of(world, hero));
        settle_ward_placement(world, 1, request).unwrap_err()
    });
    check(&mut world, WardReject::UnknownCatalog, &mut |world| {
        set_authoritative_trinket_loadout(world, 1, Some(u16::MAX)).unwrap_err()
    });
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let hero_entity = hero(&world, 1);
    let origin = pos_of(&world, hero_entity);
    check(&mut world, WardReject::Paused, &mut |world| {
        world.write_resource::<GamePause>().is_paused = true;
        let error = settle_ward_placement(world, 1, request(hero_entity, origin)).unwrap_err();
        world.write_resource::<GamePause>().is_paused = false;
        error
    });
    check(&mut world, WardReject::Dead, &mut |world| {
        let saved_hp = world.read_storage::<CProperty>().get(hero_entity).unwrap().hp;
        world.write_storage::<CProperty>().get_mut(hero_entity).unwrap().hp = Fixed64::ZERO;
        let error = settle_ward_placement(world, 1, request(hero_entity, origin)).unwrap_err();
        world.write_storage::<CProperty>().get_mut(hero_entity).unwrap().hp = saved_hp;
        error
    });
    check(&mut world, WardReject::WrongOwner, &mut |world| {
        settle_ward_placement(world, 99, request(hero_entity, origin)).unwrap_err()
    });
    check(&mut world, WardReject::StaleSource, &mut |world| {
        let mut stale = request(hero_entity, origin);
        stale.source_generation = stale.source_generation.wrapping_add(1);
        settle_ward_placement(world, 1, stale).unwrap_err()
    });
    check(&mut world, WardReject::IllegalPoint, &mut |world| {
        settle_ward_placement(world, 1, request(
            hero_entity,
            Vec2::new(Fixed64::from_raw((1 << 29) + 1), Fixed64::ZERO),
        )).unwrap_err()
    });
    check(&mut world, WardReject::OutOfRange, &mut |world| {
        settle_ward_placement(world, 1, request(
            hero_entity,
            origin + Vec2::new(Fixed64::ZERO, row.placement_range + Fixed64::from_i32(1)),
        )).unwrap_err()
    });
    check(&mut world, WardReject::TerrainBlocked, &mut |world| {
        let saved = (*world.read_resource::<BlockedRegions>()).clone();
        let point = origin + Vec2::new(Fixed64::ZERO, row.placement_range);
        world.write_resource::<BlockedRegions>().0.push(BlockedRegion {
            name: "ward-reject".into(),
            points: vec![
                vek::Vec2::new(point.x.to_f32_for_render() - 2.0, point.y.to_f32_for_render() - 2.0),
                vek::Vec2::new(point.x.to_f32_for_render() + 2.0, point.y.to_f32_for_render() - 2.0),
                vek::Vec2::new(point.x.to_f32_for_render() + 2.0, point.y.to_f32_for_render() + 2.0),
                vek::Vec2::new(point.x.to_f32_for_render() - 2.0, point.y.to_f32_for_render() + 2.0),
            ],
        });
        let error = settle_ward_placement(world, 1, request(hero_entity, point)).unwrap_err();
        *world.write_resource::<BlockedRegions>() = saved;
        error
    });
    check(&mut world, WardReject::ApplyFailed, &mut |world| {
        let saved = world.read_resource::<MobaMatch>().elapsed;
        world.write_resource::<MobaMatch>().elapsed = Fixed64::from_raw(i64::MAX - 1000);
        let error = settle_ward_placement(world, 1, request(
            hero_entity,
            origin + Vec2::new(Fixed64::ZERO, row.placement_range),
        )).unwrap_err();
        world.write_resource::<MobaMatch>().elapsed = saved;
        error
    });
    let before_fail = fingerprint(&world, 1);
    let removed = world.remove::<MobaMatch>().expect("match");
    assert_eq!(
        settle_ward_placement(&mut world, 1, request(hero_entity, origin)).unwrap_err(),
        WardReject::CapabilityDisabled
    );
    world.insert(removed);
    assert_eq!(fingerprint(&world, 1), before_fail);

    let target = origin + Vec2::new(Fixed64::ZERO, row.placement_range);
    let receipt = place(&mut world, 1, target);
    assert_eq!(receipt.charges_after, row.initial_charges - 1);
    let after_place = fingerprint(&world, 1);
    assert_eq!(
        settle_ward_placement(&mut world, 1, request(hero_entity, target)).unwrap_err(),
        WardReject::NoCharge
    );
    assert_eq!(fingerprint(&world, 1), after_place);
    assert!(world.is_alive(world.entities().entity(receipt.entity_id)));
}

#[test]
fn owner_limit_retires_oldest_placement_and_stale_generation_is_not_deleted() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    driver.step(&mut world, []).unwrap();
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let origin = pos_of(&world, hero(&world, 1));
    let target = origin + Vec2::new(Fixed64::ZERO, row.placement_range);
    let mut kept = Vec::new();
    for index in 0..row.maximum_owned_placements {
        if index > 0 {
            set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
        }
        let receipt = place(&mut world, 1, target);
        assert!(receipt.retired_canonical_id.is_none());
        kept.push(receipt);
    }
    let before = row_of(&world, 1).placements().to_vec();
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let extra = place(&mut world, 1, target);
    world.maintain();
    let retired = extra.retired_canonical_id.expect("oldest retirement");
    let after = row_of(&world, 1).placements().to_vec();
    assert_eq!(after.len(), usize::from(row.maximum_owned_placements));
    assert!(before.iter().any(|row| row.canonical_id == retired));
    assert!(after.iter().all(|row| row.canonical_id != retired));
    let retired_row = before.iter().find(|row| row.canonical_id == retired).unwrap();
    for row in before.iter().chain(after.iter()) {
        assert!((retired_row.placement_tick, retired_row.canonical_id, retired_row.order)
            <= (row.placement_tick, row.canonical_id, row.order));
    }
    assert!(
        !generation_alive(&world, entity_at(retired_row.entity_id, retired_row.generation)),
        "the retired placement generation is dead"
    );
    for row in &after {
        let entity = entity_at(row.entity_id, row.generation);
        assert!(generation_alive(&world, entity));
        assert!(world.read_storage::<VisionSource>().get(entity).is_some());
        assert_eq!(world.read_storage::<PlacedWard>().get(entity).unwrap().order, row.order);
    }

    let survivor = after[0];
    let old = entity_at(survivor.entity_id, survivor.generation);
    assert_eq!(generation(old), survivor.generation);
    assert!(generation_alive(&world, old));
    world.delete_entity(old).unwrap();
    world.maintain();
    let fresh = world
        .create_entity()
        .with(Pos(target))
        .with(VisionSource { team: 1, radius: Fixed64::from_i32(10), detection_level: 0 })
        .build();
    assert_eq!(fresh.id(), old.id(), "specs recycles the deleted ward id");
    assert_ne!(generation(fresh), survivor.generation);
    assert!(!generation_alive(&world, old), "the retired generation stays dead after its id is reused");
    retire_owner_wards(&mut world, 1);
    world.maintain();
    assert!(generation_alive(&world, fresh), "a reused generation is not deleted");
    assert!(world.read_storage::<VisionSource>().get(fresh).is_some());
    assert!(row_of(&world, 1).placements().is_empty());
}

#[test]
fn recharge_respects_cap_large_delta_pause_results_and_death_respawn() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    assert_eq!(row_of(&world, 1).charges(), row.initial_charges);
    assert!(row_of(&world, 1).next_recharge_at().is_none());
    driver.step(&mut world, []).unwrap();
    let armed = row_of(&world, 1);
    assert_eq!(armed.charges(), row.initial_charges);
    let elapsed = world.read_resource::<MobaMatch>().elapsed.raw();
    assert_eq!(
        armed.next_recharge_at().expect("armed clock").raw(),
        elapsed + row.recharge_seconds.raw()
    );
    let origin = pos_of(&world, hero(&world, 1));
    let target = origin + Vec2::new(Fixed64::ZERO, row.placement_range);
    let probe = probe_at(&mut world, origin + Vec2::new(Fixed64::ZERO, Fixed64::from_i32(1400)));
    project(&mut world);
    assert!(!sees(&world, 1, probe), "hero and tower sight must not already cover the probe");
    let placed = place(&mut world, 1, target);
    let ward = entity_at(placed.entity_id, placed.generation);
    let expires = world.read_storage::<PlacedWard>().get(ward).expect("placed lifetime").expires_at_raw;
    let due = elapsed + row.recharge_seconds.raw();
    assert!(expires < due, "the catalog lifetime ends before this armed recharge");
    assert_eq!(row_of(&world, 1).charges(), row.initial_charges - 1);
    assert_eq!(row_of(&world, 1).next_recharge_at().unwrap().raw(), due);
    let revision = row_of(&world, 1).revision();
    project(&mut world);
    assert!(sees(&world, 1, probe));

    world.write_resource::<GamePause>().is_paused = true;
    driver.step(&mut world, []).unwrap();
    assert_eq!(world.read_resource::<MobaMatch>().elapsed.raw(), elapsed);
    assert_eq!(row_of(&world, 1).charges(), row.initial_charges - 1);
    assert_eq!(row_of(&world, 1).revision(), revision);
    assert_eq!(row_of(&world, 1).next_recharge_at().unwrap().raw(), due);
    project(&mut world);
    assert!(sees(&world, 1, probe), "a paused driver step keeps the placed ward's sight");

    world.write_resource::<DeltaTime>().0 = Fixed64::from_i32(10_000);
    assert!(
        !begin_moba_match_tick(&mut world),
        "a positive delta while paused is an inactive tick"
    );
    assert_eq!(world.read_resource::<MobaMatch>().elapsed.raw(), elapsed, "pause does not add elapsed");
    assert_eq!(row_of(&world, 1).charges(), row.initial_charges - 1);
    assert_eq!(row_of(&world, 1).revision(), revision);
    assert_eq!(row_of(&world, 1).next_recharge_at().unwrap().raw(), due);
    assert!(generation_alive(&world, ward));
    assert!(row_of(&world, 1).placements().iter().any(|row| row.canonical_id == placed.canonical_id));
    project(&mut world);
    assert!(sees(&world, 1, probe), "sight remains while paused time crosses both deadlines");

    world.write_resource::<GamePause>().is_paused = false;
    world.write_resource::<DeltaTime>().0 = Fixed64::from_i32(1);
    assert!(begin_moba_match_tick(&mut world));
    let resumed = world.read_resource::<MobaMatch>().elapsed.raw();
    assert_eq!(resumed, elapsed + Fixed64::from_i32(1).raw());
    assert!(resumed < expires && resumed < due);
    assert!(generation_alive(&world, ward));
    assert_eq!(row_of(&world, 1).charges(), row.initial_charges - 1, "paused time is not granted on resume");
    assert_eq!(row_of(&world, 1).next_recharge_at().unwrap().raw(), due);
    project(&mut world);
    assert!(sees(&world, 1, probe), "the first resumed subthreshold tick keeps sight");

    let life_left = expires - resumed;
    world.write_resource::<DeltaTime>().0 = Fixed64::from_raw(life_left);
    assert!(begin_moba_match_tick(&mut world));
    world.maintain();
    assert_eq!(world.read_resource::<MobaMatch>().elapsed.raw(), expires);
    assert!(!generation_alive(&world, ward), "only unpaused time reaches the lifetime");
    assert!(row_of(&world, 1).placements().is_empty());
    assert_eq!(row_of(&world, 1).charges(), row.initial_charges - 1);
    project(&mut world);
    assert!(!sees(&world, 1, probe), "exact unpaused expiry removes sight");

    let now = world.read_resource::<MobaMatch>().elapsed.raw();
    assert!(now < due);
    world.write_resource::<DeltaTime>().0 = Fixed64::from_raw(due - now);
    assert!(begin_moba_match_tick(&mut world));
    assert_eq!(world.read_resource::<MobaMatch>().elapsed.raw(), due);
    assert_eq!(row_of(&world, 1).charges(), row.initial_charges);
    assert_eq!(
        row_of(&world, 1).next_recharge_at().unwrap().raw(),
        due + row.recharge_seconds.raw(),
        "the next deadline is one interval after the due instant"
    );

    advance(&mut world, 150);
    assert_eq!(row_of(&world, 1).charges(), row.max_charges, "excess delta does not pass the cap");
    assert!(row_of(&world, 1).next_recharge_at().is_none());
    let origin = pos_of(&world, hero(&world, 1));
    let target = origin + Vec2::new(Fixed64::ZERO, row.placement_range);
    let first = place(&mut world, 1, target);
    let serial = row_of(&world, 1).next_recharge_at().expect("spend arms an empty timer");
    let second = place(&mut world, 1, target);
    assert_eq!(row_of(&world, 1).charges(), 0);
    assert_eq!(row_of(&world, 1).next_recharge_at().unwrap().raw(), serial.raw(), "serial timer is not restarted");
    advance(&mut world, 10_000);
    assert_eq!(row_of(&world, 1).charges(), row.max_charges);
    assert!(row_of(&world, 1).next_recharge_at().is_none());
    assert!(row_of(&world, 1).placements().is_empty(), "the large delta also expires both wards");
    assert!(!generation_alive(&world, entity_at(first.entity_id, first.generation)));
    assert!(!generation_alive(&world, entity_at(second.entity_id, second.generation)));

    world.write_resource::<Vec<DamageInstance>>().clear();
    world.write_resource::<Vec<Outcome>>().clear();
    let origin = pos_of(&world, hero(&world, 1));
    let placed = place(&mut world, 1, origin + Vec2::new(Fixed64::ZERO, row.placement_range));
    let ward = entity_at(placed.entity_id, placed.generation);
    let old_hero = hero(&world, 1);
    let old_generation = generation(old_hero);
    let charges_after_place = row_of(&world, 1).charges();
    let timer_after_place = row_of(&world, 1).next_recharge_at();
    world.write_resource::<Vec<Outcome>>().push(Outcome::Death { pos: pos_of(&world, old_hero), ent: old_hero });
    let mut sink = RuntimeEventVecSink::default();
    process_outcomes(&mut world, &mut sink).unwrap();
    world.maintain();
    assert!(!generation_alive(&world, old_hero));
    assert!(generation_alive(&world, ward));
    assert!(world.read_storage::<VisionSource>().get(ward).is_some());
    assert_eq!(row_of(&world, 1).charges(), charges_after_place);
    assert_eq!(row_of(&world, 1).next_recharge_at(), timer_after_place);
    world.write_resource::<Vec<DamageInstance>>().clear();
    world.write_resource::<Vec<Outcome>>().clear();
    driver.step(&mut world, []).unwrap();
    let new_hero = hero(&world, 1);
    assert_ne!(generation(new_hero), old_generation);
    assert!(generation_alive(&world, ward), "hero death and respawn keep a valid ward");
    assert_eq!(row_of(&world, 1).charges(), charges_after_place);
    assert_eq!(
        settle_ward_placement(&mut world, 1, WardPlacementRequest {
            local_slot: WARD_LOCAL_SLOT,
            source_generation: old_generation,
            target: origin,
        }).unwrap_err(),
        WardReject::StaleSource
    );
    assert_eq!(row_of(&world, 1).charges(), charges_after_place);

    world.write_resource::<MobaMatch>().bases = [None, None];
    finish_moba_match_tick(&mut world);
    world.maintain();
    assert!(matches!(world.read_resource::<MobaMatch>().phase, MobaMatchPhase::Finished { .. }));
    assert!(!generation_alive(&world, ward), "results retire placed wards in the finish mutation");
    assert_eq!(row_of(&world, 1).charges(), charges_after_place);
    assert!(row_of(&world, 1).placements().is_empty());
    world.write_resource::<DeltaTime>().0 = Fixed64::from_i32(10_000);
    assert!(!begin_moba_match_tick(&mut world));
    assert_eq!(row_of(&world, 1).charges(), charges_after_place, "results do not keep recharging");
}

#[test]
fn disclosed_direct_attack_hits_auto_disabled_ward_and_attack_move_skips_it() {
    let row = catalog_row();
    assert!(matches!(row.automatic_acquisition, omoba_template_ids::TrinketAutomaticAcquisitionConst::Disabled));
    assert!(matches!(row.direct_attack, omoba_template_ids::TrinketDirectAttackConst::DisclosedEnemy));
    let (mut world, mut driver) = world();
    driver.step(&mut world, []).unwrap();
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let origin = pos_of(&world, hero(&world, 1));
    let placed = place(&mut world, 1, origin + Vec2::new(Fixed64::ZERO, row.placement_range));
    let ward = world.entities().entity(placed.entity_id);
    let attacker = hero(&world, 2);
    let owner = hero(&world, 1);
    let ward_pos = Vec2::new(Fixed64::from_i32(1400), Fixed64::from_i32(600));
    let attacker_pos = Vec2::new(Fixed64::from_i32(1500), Fixed64::from_i32(600));
    world.write_storage::<Pos>().get_mut(ward).unwrap().0 = ward_pos;
    world.write_storage::<Pos>().get_mut(attacker).unwrap().0 = attacker_pos;
    world.write_storage::<Facing>().get_mut(attacker).unwrap().clone_from(&Facing::from_rad_f32(std::f32::consts::PI));
    let creep = (&world.entities(), &world.read_storage::<Creep>(), &world.read_storage::<Faction>())
        .join()
        .find(|(_, _, faction)| faction.team_id == 1)
        .map(|(entity, _, _)| entity)
        .expect("lane creep");
    world.write_storage::<Pos>().get_mut(creep).unwrap().0 =
        Vec2::new(Fixed64::from_i32(1100), Fixed64::from_i32(600));
    set_attack_range(&mut world, attacker, 800);
    set_attack_range(&mut world, owner, 1);
    let detector = world.create_entity().with(Pos(attacker_pos)).with(VisionSource {
        team: 2,
        radius: Fixed64::from_i32(2000),
        detection_level: 1,
    }).build();
    project(&mut world);
    assert!(sees(&world, 2, ward));
    assert!(sees(&world, 2, creep));
    index_bodies(&mut world, &[ward, creep]);
    world.write_resource::<DeltaTime>().0 = Fixed64::from_i32(1);
    prime_attack(&mut world, attacker);
    world.write_resource::<Vec<Outcome>>().clear();
    run_now::<omoba_core::tick::hero_tick::Sys>(&world);
    assert_eq!(cue_from(&world, attacker), Some(creep), "idle acquisition skips the auto-disabled ward");

    command(&mut world, attacker, HeroCommand::AttackMove {
        pos: Vec2::new(Fixed64::ZERO, Fixed64::from_i32(600)),
    });
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(!moving(&world, attacker), "attack-move holds on the ordinary creep");
    index_bodies(&mut world, &[ward]);
    // End the previous creep's animation lock before testing a new acquisition.
    prime_attack(&mut world, attacker);
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(
        moving(&world, attacker) || active(&world, attacker).is_none(),
        "attack-move does not hold on the auto-disabled ward"
    );

    command(&mut world, attacker, HeroCommand::AttackTarget { target: ward, chase_origin: None });
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(matches!(
        active(&world, attacker),
        Some(HeroCommand::AttackTarget { target, .. }) if target == ward
    ));
    assert!(!moving(&world, attacker));

    world.delete_entity(detector).unwrap();
    world.maintain();
    project(&mut world);
    assert!(!sees(&world, 2, ward), "without detection the stealth ward is hidden");
    command(&mut world, attacker, HeroCommand::AttackTarget { target: ward, chase_origin: None });
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(active(&world, attacker).is_none(), "hidden direct attack is refused");

    command(&mut world, owner, HeroCommand::AttackTarget { target: ward, chase_origin: None });
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(active(&world, owner).is_none(), "a friendly ward is refused");

    let stale = world.create_entity().with(Pos(attacker_pos)).with(CProperty {
        hp: Fixed64::from_i32(3),
        mhp: Fixed64::from_i32(3),
        msd: Fixed64::ZERO,
        def_physic: Fixed64::ZERO,
        def_magic: Fixed64::ZERO,
    }).with(Faction::new(FactionType::Enemy, 1)).build();
    world.delete_entity(stale).unwrap();
    world.maintain();
    command(&mut world, attacker, HeroCommand::AttackTarget { target: stale, chase_origin: None });
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(active(&world, attacker).is_none(), "a stale entity is refused");

    let _detector = world.create_entity().with(Pos(attacker_pos)).with(VisionSource {
        team: 2,
        radius: Fixed64::from_i32(2000),
        detection_level: 1,
    }).build();
    world.write_storage::<Pos>().get_mut(owner).unwrap().0 =
        attacker_pos + Vec2::new(Fixed64::from_i32(-80), Fixed64::ZERO);
    project(&mut world);
    assert!(sees(&world, 2, ward));
    assert!(sees(&world, 2, owner));
    let mut admitted = AdmittedCombatPreferences::default();
    admitted.by_owner.insert(2, OwnerCombatPreferences::try_from_raw(CombatPreferenceUpdate {
        owner_id: 2,
        acquisition_raw: AttackAcquisition::Hero.raw(),
        idle_auto_attack: true,
        champion_only_direct: true,
    }).unwrap());
    world.insert(admitted);
    command(&mut world, attacker, HeroCommand::AttackTarget { target: ward, chase_origin: None });
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(matches!(active(&world, attacker),Some(HeroCommand::AttackTarget {target,..}) if target==ward),
        "shadow champion-only preferences cannot override the authority roster");
    command(&mut world, attacker, HeroCommand::AttackTarget { target: owner, chase_origin: None });
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(matches!(
        active(&world, attacker),
        Some(HeroCommand::AttackTarget { target, .. }) if target == owner
    ), "champion-only direct still allows the disclosed hero");
}

fn set_attack_range(world: &mut World, hero: Entity, units: i32) {
    let range = Fixed64::from_i32(units);
    let mut attacks = world.write_storage::<TAttack>();
    let attack = attacks.get_mut(hero).unwrap();
    attack.range = Vf32::new(range);
    attack.asd_count = attack.asd.v;
    attack.attack_phase = AttackSequencePhase::Idle;
}

fn prime_attack(world: &mut World, hero: Entity) {
    let mut attacks = world.write_storage::<TAttack>();
    let attack = attacks.get_mut(hero).unwrap();
    attack.attack_phase = AttackSequencePhase::Idle;
    attack.asd_count = attack.asd.v;
    attack.animation_timing = None;
}

fn command(world: &mut World, hero: Entity, command: HeroCommand) {
    let owner_pid = world
        .read_storage::<PlayerOwner>()
        .get(hero)
        .expect("commanded hero owner")
        .player_id;
    let kind = match command {
        HeroCommand::HoldPosition => PendingHeroCommandKind::HoldPosition,
        HeroCommand::MoveTo { pos } => PendingHeroCommandKind::MoveTo { pos },
        HeroCommand::AttackMove { pos } => PendingHeroCommandKind::AttackMove { pos },
        HeroCommand::AttackTarget { target, .. } => PendingHeroCommandKind::AttackTarget {
            target_entity_id: target.id(),
        },
    };
    world.write_resource::<PendingMoveQueue>().requests.push(PendingHeroCommand { admitted_correlation: 0,
        owner_pid,
        queued: false,
        kind,
    });
    drain_pending_moves(world);
}

fn active(world: &World, hero: Entity) -> Option<HeroCommand> {
    world.read_storage::<HeroCommandQueue>().get(hero).and_then(|queue| queue.active)
}

fn moving(world: &World, hero: Entity) -> bool {
    world.read_storage::<MoveTarget>().get(hero).is_some()
}

#[test]
fn ward_roster_changes_replay_digest_and_identical_matches_agree() {
    let row = catalog_row();
    let (mut left, mut left_driver) = world();
    let (mut right, mut right_driver) = world();
    left_driver.step(&mut left, []).unwrap();
    right_driver.step(&mut right, []).unwrap();
    assert_eq!(single_lane_replay_digest(&left), single_lane_replay_digest(&right));
    set_authoritative_trinket_loadout(&mut left, 1, Some(row.catalog_id)).unwrap();
    assert_ne!(single_lane_replay_digest(&left), single_lane_replay_digest(&right));
    set_authoritative_trinket_loadout(&mut right, 1, Some(row.catalog_id)).unwrap();
    assert_eq!(single_lane_replay_digest(&left), single_lane_replay_digest(&right));
    let origin = pos_of(&left, hero(&left, 1));
    let target = origin + Vec2::new(Fixed64::ZERO, row.placement_range);
    place(&mut left, 1, target);
    assert_ne!(single_lane_replay_digest(&left), single_lane_replay_digest(&right));
    place(&mut right, 1, target);
    assert_eq!(single_lane_replay_digest(&left), single_lane_replay_digest(&right));
    advance(&mut left, 150);
    assert_ne!(
        single_lane_replay_digest(&left),
        single_lane_replay_digest(&right),
        "charge, timer, and placement changes are in the digest"
    );
}

struct ParkedWard {
    ward: Entity,
    attacker: Entity,
    owner: Entity,
    attacker_pos: Vec2,
    owner_origin: Vec2,
    canonical_id: u64,
}

fn park_real_ward(world: &mut World) -> ParkedWard {
    let row = catalog_row();
    let owner = hero(world, 1);
    let owner_origin = pos_of(world, owner);
    let placed = place(world, 1, owner_origin + Vec2::new(Fixed64::ZERO, row.placement_range));
    let ward = world.entities().entity(placed.entity_id);
    assert!(world.read_storage::<Unit>().get(ward).is_none(), "production spawn has no Unit");
    assert!(world.read_storage::<Creep>().get(ward).is_none());
    assert!(world.read_storage::<Hero>().get(ward).is_none());
    let attacker = hero(world, 2);
    let attacker_pos = Vec2::new(Fixed64::from_i32(3000), Fixed64::from_i32(3000));
    world.write_storage::<Pos>().get_mut(ward).unwrap().0 =
        attacker_pos + Vec2::new(Fixed64::from_i32(-40), Fixed64::ZERO);
    world.write_storage::<Pos>().get_mut(attacker).unwrap().0 = attacker_pos;
    world
        .write_storage::<Facing>()
        .get_mut(attacker)
        .unwrap()
        .clone_from(&Facing::from_rad_f32(std::f32::consts::PI));
    set_attack_range(world, attacker, 80);
    set_attack_range(world, owner, 80);
    run_now::<omoba_core::tick::nearby_tick::Sys>(world);
    assert_ward_absent_from_indexes(world, ward, attacker_pos);
    ParkedWard {
        ward,
        attacker,
        owner,
        attacker_pos,
        owner_origin,
        canonical_id: placed.canonical_id,
    }
}

fn assert_ward_absent_from_indexes(world: &World, ward: Entity, center: Vec2) {
    let searcher = world.read_resource::<Searcher>();
    let point = vek::Vec2::new(center.x.to_f32_for_render(), center.y.to_f32_for_render());
    let radius = Fixed64::from_i32(200).to_f32_for_render();
    assert!(!searcher.hero.entities_in_radius(point, radius).contains(&ward));
    assert!(!searcher.creep.entities_in_radius(point, radius).contains(&ward));
    assert!(!searcher.tower.entities_in_radius(point, radius).contains(&ward));
}

fn reveal_for_attacker(world: &mut World, at: Vec2) -> Entity {
    world
        .create_entity()
        .with(Pos(at))
        .with(VisionSource {
            team: 2,
            radius: Fixed64::from_i32(2000),
            detection_level: 1,
        })
        .build()
}

fn idle_target(world: &mut World, attacker: Entity) -> Option<Entity> {
    if let Some(queue) = world.write_storage::<HeroCommandQueue>().get_mut(attacker) {
        queue.clear_all();
    }
    world.write_storage::<MoveTarget>().remove(attacker);
    world.write_resource::<DeltaTime>().0 = Fixed64::from_i32(1);
    prime_attack(world, attacker);
    world.write_resource::<Vec<Outcome>>().clear();
    run_now::<omoba_core::tick::hero_tick::Sys>(world);
    cue_from(world, attacker)
}

fn holds_attack_target(world: &World, attacker: Entity, target: Entity) -> bool {
    matches!(
        active(world, attacker),
        Some(HeroCommand::AttackTarget { target: held, .. }) if held == target
    )
}

fn lane_creep(world: &World) -> Entity {
    (&world.entities(), &world.read_storage::<Creep>(), &world.read_storage::<Faction>())
        .join()
        .find(|(_, _, faction)| faction.team_id == 1)
        .map(|(entity, _, _)| entity)
        .expect("lane creep")
}

fn set_policy(
    world: &mut World,
    canonical_id: u64,
    direct: omoba_template_ids::TrinketDirectAttackConst,
    automatic: omoba_template_ids::TrinketAutomaticAcquisitionConst,
) {
    assert!(
        override_retained_ward_combat_policy(world, canonical_id, direct, automatic),
        "policy override applies only to a retained placed ward"
    );
}

#[test]
fn real_placed_ward_without_unit_is_acquired_when_automatic_acquisition_is_enabled_and_skipped_when_disabled() {
    use omoba_template_ids::{TrinketAutomaticAcquisitionConst, TrinketDirectAttackConst};
    let row = catalog_row();
    assert!(matches!(row.automatic_acquisition, TrinketAutomaticAcquisitionConst::Disabled));
    let (mut world, mut driver) = world();
    driver.step(&mut world, []).unwrap();
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let parked = park_real_ward(&mut world);
    let _detector = reveal_for_attacker(&mut world, parked.attacker_pos);
    project(&mut world);
    assert!(sees(&world, 2, parked.ward));
    set_policy(
        &mut world,
        parked.canonical_id,
        TrinketDirectAttackConst::DisclosedEnemy,
        TrinketAutomaticAcquisitionConst::Enabled,
    );
    assert_eq!(idle_target(&mut world, parked.attacker), Some(parked.ward));
    command(
        &mut world,
        parked.attacker,
        HeroCommand::AttackMove {
            pos: parked.attacker_pos + Vec2::new(Fixed64::from_i32(-400), Fixed64::ZERO),
        },
    );
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(!moving(&world, parked.attacker), "attack-move holds on the auto-enabled ward");
    assert!(matches!(active(&world, parked.attacker), Some(HeroCommand::AttackMove { .. })));

    set_policy(
        &mut world,
        parked.canonical_id,
        TrinketDirectAttackConst::DisclosedEnemy,
        TrinketAutomaticAcquisitionConst::Disabled,
    );
    assert_eq!(idle_target(&mut world, parked.attacker), None, "auto-disabled ward is not acquired");
    command(
        &mut world,
        parked.attacker,
        HeroCommand::AttackMove {
            pos: parked.attacker_pos + Vec2::new(Fixed64::from_i32(-400), Fixed64::ZERO),
        },
    );
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(
        moving(&world, parked.attacker) || active(&world, parked.attacker).is_none(),
        "attack-move does not hold on the auto-disabled ward"
    );
    command(
        &mut world,
        parked.attacker,
        HeroCommand::AttackTarget { target: parked.ward, chase_origin: None },
    );
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(holds_attack_target(&world, parked.attacker, parked.ward), "direct attack stays independent of automatic acquisition");

    let creep = lane_creep(&world);
    world.write_storage::<Pos>().get_mut(creep).unwrap().0 =
        parked.attacker_pos + Vec2::new(Fixed64::from_i32(-30), Fixed64::ZERO);
    run_now::<omoba_core::tick::nearby_tick::Sys>(&world);
    assert_ward_absent_from_indexes(&world, parked.ward, parked.attacker_pos);
    project(&mut world);
    assert!(sees(&world, 2, creep), "moved ordinary creep must be committed as disclosed before acquisition");
    assert_eq!(idle_target(&mut world, parked.attacker), Some(creep), "an ordinary indexed creep is still acquired");
}

#[test]
fn not_directly_attackable_real_ward_stays_on_the_automatic_path() {
    use omoba_template_ids::{TrinketAutomaticAcquisitionConst, TrinketDirectAttackConst};
    let row = catalog_row();
    let (mut world, mut driver) = world();
    driver.step(&mut world, []).unwrap();
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let parked = park_real_ward(&mut world);
    let _detector = reveal_for_attacker(&mut world, parked.attacker_pos);
    project(&mut world);
    assert!(sees(&world, 2, parked.ward));
    set_policy(
        &mut world,
        parked.canonical_id,
        TrinketDirectAttackConst::NotDirectlyAttackable,
        TrinketAutomaticAcquisitionConst::Enabled,
    );
    assert_eq!(idle_target(&mut world, parked.attacker), Some(parked.ward));
    command(
        &mut world,
        parked.attacker,
        HeroCommand::AttackMove {
            pos: parked.attacker_pos + Vec2::new(Fixed64::from_i32(-400), Fixed64::ZERO),
        },
    );
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(!moving(&world, parked.attacker));
    assert!(matches!(active(&world, parked.attacker), Some(HeroCommand::AttackMove { .. })));
    command(
        &mut world,
        parked.attacker,
        HeroCommand::AttackTarget { target: parked.ward, chase_origin: None },
    );
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(active(&world, parked.attacker).is_none(), "NotDirectlyAttackable refuses the direct command");
}

#[test]
fn hidden_friendly_stale_and_champion_only_gates_use_the_real_ward() {
    use omoba_template_ids::{TrinketAutomaticAcquisitionConst, TrinketDirectAttackConst};
    let row = catalog_row();
    let (mut world, mut driver) = world();
    driver.step(&mut world, []).unwrap();
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let parked = park_real_ward(&mut world);
    let detector = reveal_for_attacker(&mut world, parked.attacker_pos);
    project(&mut world);
    assert!(sees(&world, 2, parked.ward));
    set_policy(
        &mut world,
        parked.canonical_id,
        TrinketDirectAttackConst::DisclosedEnemy,
        TrinketAutomaticAcquisitionConst::Enabled,
    );
    let mut admitted = AdmittedCombatPreferences::default();
    admitted.by_owner.insert(
        2,
        OwnerCombatPreferences::try_from_raw(CombatPreferenceUpdate {
            owner_id: 2,
            acquisition_raw: AttackAcquisition::Hero.raw(),
            idle_auto_attack: true,
            champion_only_direct: true,
        })
        .unwrap(),
    );
    world.insert(admitted);
    assert_eq!(
        idle_target(&mut world, parked.attacker),
        Some(parked.ward),
        "champion-only does not remove an auto-enabled ward from idle"
    );
    command(
        &mut world,
        parked.attacker,
        HeroCommand::AttackTarget { target: parked.ward, chase_origin: None },
    );
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(matches!(active(&world, parked.attacker),Some(HeroCommand::AttackTarget {target,..}) if target==parked.ward),
        "shadow champion-only preferences cannot override the authority roster");

    world.write_storage::<Pos>().get_mut(parked.owner).unwrap().0 =
        parked.attacker_pos + Vec2::new(Fixed64::from_i32(-20), Fixed64::ZERO);
    run_now::<omoba_core::tick::nearby_tick::Sys>(&world);
    assert_ward_absent_from_indexes(&world, parked.ward, parked.attacker_pos);
    project(&mut world);
    assert!(sees(&world, 2, parked.owner));
    command(
        &mut world,
        parked.attacker,
        HeroCommand::AttackTarget { target: parked.owner, chase_origin: None },
    );
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(
        holds_attack_target(&world, parked.attacker, parked.owner),
        "champion-only direct still allows the disclosed hero"
    );

    command(
        &mut world,
        parked.owner,
        HeroCommand::AttackTarget { target: parked.ward, chase_origin: None },
    );
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(active(&world, parked.owner).is_none(), "the owner does not directly attack a friendly ward");

    world.write_storage::<Pos>().get_mut(parked.owner).unwrap().0 = parked.owner_origin;
    run_now::<omoba_core::tick::nearby_tick::Sys>(&world);
    world.delete_entity(detector).unwrap();
    world.maintain();
    project(&mut world);
    assert!(!sees(&world, 2, parked.ward), "without detection the stealth ward is hidden");
    assert_eq!(idle_target(&mut world, parked.attacker), None);
    command(
        &mut world,
        parked.attacker,
        HeroCommand::AttackTarget { target: parked.ward, chase_origin: None },
    );
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(active(&world, parked.attacker).is_none(), "a hidden ward is not a direct target");

    let stale = parked.ward;
    let stale_id = stale.id();
    world.delete_entity(stale).unwrap();
    world.maintain();
    assert_eq!(idle_target(&mut world, parked.attacker), None, "a dead placement generation is not acquired");
    command(
        &mut world,
        parked.attacker,
        HeroCommand::AttackTarget { target: stale, chase_origin: None },
    );
    run_now::<omoba_core::tick::hero_command_tick::Sys>(&world);
    assert!(active(&world, parked.attacker).is_none(), "a stale ward handle is refused");
    let fresh = world
        .create_entity()
        .with(Pos(parked.attacker_pos + Vec2::new(Fixed64::from_i32(-40), Fixed64::ZERO)))
        .with(CProperty {
            hp: Fixed64::from_i32(3),
            mhp: Fixed64::from_i32(3),
            msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO,
            def_magic: Fixed64::ZERO,
        })
        .with(Faction::new(FactionType::Enemy, 1))
        .build();
    assert_eq!(fresh.id(), stale_id, "specs recycles the deleted ward id");
    assert_ne!(generation(fresh), generation(stale));
    run_now::<omoba_core::tick::nearby_tick::Sys>(&world);
    assert_ward_absent_from_indexes(&world, fresh, parked.attacker_pos);
    assert_eq!(idle_target(&mut world, parked.attacker), None, "the recycled id is not the retained ward");
}

fn cue_from(world: &World, attacker: Entity) -> Option<Entity> {
    world.read_resource::<Vec<Outcome>>().iter().find_map(|outcome| match outcome {
        Outcome::AttackPhaseCue { entity, target, .. } if *entity == attacker => *target,
        Outcome::ProjectileLine2 { source, target, .. } if *source == Some(attacker) => *target,
        _ => None,
    })
}

fn index_bodies(world: &mut World, bodies: &[Entity]) {
    let positions = world.read_storage::<Pos>();
    let rows: Vec<_> = bodies.iter().copied().filter_map(|entity| {
        positions.get(entity).map(|pos| {
            let (x, y) = pos.xy_f32();
            (entity, vek::Vec2::new(x, y))
        })
    }).collect();
    drop(positions);
    let mut searcher = world.write_resource::<Searcher>();
    searcher.hero.rebuild_from(Vec::<(Entity, vek::Vec2<f32>)>::new());
    searcher.creep.rebuild_from(rows);
    searcher.tower.rebuild_from(Vec::<(Entity, vek::Vec2<f32>)>::new());
}
