//! Authenticated ward command, private owner projection, and disclosed body.
//! These tests are source for the transport foundation. This worker does not run them.

use std::collections::BTreeMap;

use omoba_core::comp::run_now;
use omoba_core::game_proto::{FilteredTeamSnapshot, TeamGameStart, ViewEpoch};
use omoba_core::runtime::ward_transport::{
    decode_disclosed_ward_body, decode_owner_ward_private, decode_owner_ward_projection,
    disclosed_automatic_acquisition_allowed, disclosed_direct_attack_allowed, encode_disclosed_ward_body,
    negotiate_ward_protocol, owner_projection_for_session, owner_ward_session_binding,
    stamp_owner_ward_admission, OwnerWardProjection, OWNER_WARD_OUTCOME_REJECTED, OWNER_WARD_OUTCOME_SETTLED,
    WARD_PROTOCOL_VERSION,
};
use omoba_core::runtime::*;
use omoba_sim::{Fixed64, Vec2};
use omoba_template_ids::TrinketConst;
use specs::{Builder, Entity, Join, World, WorldExt};

const EMPTY_SHA256: [u8; 32] = [
    0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
    0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
];

fn catalog_row() -> &'static TrinketConst {
    let named = omoba_template_ids::trinket_by_name("training_scout_lantern_v1").expect("training trinket");
    omoba_template_ids::TRINKET_CATALOG.iter().find(|row| row.catalog_id == named.catalog_id).expect("catalog id")
}

#[test]
fn admitted_detection_trinket_uses_production_step_and_game_time_deadlines() {
    let row = omoba_template_ids::trinket_by_name("training_detection_lantern_v1").unwrap();
    let (mut world, mut driver) = world();playing(&mut world, &mut driver);
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let actor_life = trusted_actor_generation(&world, 1);
    let target = pos_of(&world, hero(&world, 1)) + Vec2::new(Fixed64::ZERO, row.placement_range);
    let placement = omoba_core::game_proto::WardPlacement {slot: u32::from(WARD_LOCAL_SLOT),
        x_raw: Some(target.x.raw()), y_raw: Some(target.y.raw())};
    let before = (vision_count(&world), ward_count(&world));
    route_admitted_ward_placement(1, 4, &placement, 191, actor_life,
        &mut world.write_resource::<PendingWardPlacementQueue>());
    driver.step(&mut world, []).unwrap();
    assert!(world.read_resource::<PendingWardPlacementQueue>().requests.is_empty());
    assert_eq!(last_receipt(&world).code, omoba_core::runtime::ward_transport::RECEIPT_SETTLED);
    assert_eq!(last_receipt(&world).admitted_correlation, 191);
    assert_eq!((vision_count(&world), ward_count(&world)), (before.0+1, before.1+1));
    let roster = copied_ward_roster(&world, 1).unwrap();
    assert_eq!(roster.charges(), 0);assert_eq!(roster.placements().len(), 1);
    let due = roster.next_recharge_at().unwrap().raw();
    let expiry = roster.placements()[0].expires_at_raw;
    let private = decode_owner_ward_projection(&project_private_owner_ward(&world, 1, 4)).unwrap();
    assert_eq!((private.catalog_id, private.charges, private.max_charges), (row.catalog_id, 0, 1));
    let elapsed = world.read_resource::<MobaMatch>().elapsed.raw();
    assert_eq!(private.recharge_remaining_raw, due-elapsed);
    world.write_resource::<GamePause>().is_paused = true;
    for _ in 0..12 {driver.step(&mut world, []).unwrap();}
    assert_eq!(world.read_resource::<MobaMatch>().elapsed.raw(), elapsed);
    assert_eq!(copied_ward_roster(&world, 1).unwrap().placements()[0].expires_at_raw, expiry);
    world.write_resource::<GamePause>().is_paused = false;
    world.write_resource::<DeltaTime>().0 = Fixed64::from_raw(expiry-elapsed-1);
    assert!(begin_moba_match_tick(&mut world));world.maintain();
    assert_eq!(ward_count(&world), before.1+1, "alive one raw unit before authored expiry");
    driver.step(&mut world, []).unwrap();
    assert_eq!(ward_count(&world), before.1, "production step retires expired body");
    assert_eq!(copied_ward_roster(&world, 1).unwrap().charges(), 0);
    let now = world.read_resource::<MobaMatch>().elapsed.raw();
    world.write_resource::<DeltaTime>().0 = Fixed64::from_raw(due-now);
    assert!(begin_moba_match_tick(&mut world));world.maintain();
    assert_eq!(copied_ward_roster(&world, 1).unwrap().charges(), 1);
    assert_eq!(ward_count(&world), before.1, "recharge never resurrects a retired detector");
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

fn playing(world: &mut World, driver: &mut SimulationDriver) {
    driver.step(world, []).unwrap();
}

fn hero(world: &World, player: u32) -> Entity {
    world.read_resource::<MobaMatch>().heroes.iter().find(|slot| slot.player_id == player).and_then(|slot| slot.entity).expect("hero")
}

fn pos_of(world: &World, entity: Entity) -> Vec2 {
    world.read_storage::<Pos>().get(entity).expect("position").0
}

fn generation(entity: Entity) -> u32 {
    entity.gen().id() as u32
}

fn vision_count(world: &World) -> usize {
    (&world.entities(), &world.read_storage::<VisionSource>()).join().count()
}

fn ward_count(world: &World) -> usize {
    (&world.entities(), &world.read_storage::<PlacedWard>()).join().count()
}

fn sees(world: &World, team: u32, entity: Entity) -> bool {
    let id = canonical_entity_id(entity);
    world.read_resource::<TeamVisibilityRuntime>().teams.get(&team).is_some_and(|state| state.index.current.contains(&id))
}

fn project(world: &mut World) {
    let tick = world.read_resource::<Tick>().0;
    run_committed_visibility_wave_b(world, tick, 0);
}

fn queue_in_range(world: &mut World, player: u32, generation: Option<u32>, correlation: u32, slot: u8) {
    let target = in_range(world, player);
    queue_ward(world, player, generation, correlation, target, slot);
}

fn queue_ward(world: &mut World, player: u32, generation: Option<u32>, correlation: u32, target: Vec2, slot: u8) {
    world.write_resource::<PendingWardPlacementQueue>().requests.push(PendingWardPlacement {
        owner_pid: player,
        local_slot: slot,
        x_raw: target.x.raw(),
        y_raw: target.y.raw(),
        source_generation: generation,
        admitted_correlation: correlation,
    });
}

fn admit_in_range(world: &mut World, player: u32, correlation: u32, actor_life: Option<u32>) {
    let target = in_range(world, player);
    let placement = omoba_core::game_proto::WardPlacement {
        slot: u32::from(WARD_LOCAL_SLOT),
        x_raw: Some(target.x.raw()),
        y_raw: Some(target.y.raw()),
    };
    let mut queue = world.write_resource::<PendingWardPlacementQueue>();
    route_admitted_ward_placement(player, 4, &placement, correlation, actor_life, &mut queue);
}

fn last_receipt(world: &World) -> WardCommandReceipt {
    world.read_resource::<PendingWardPlacementQueue>().receipts.last().expect("receipt").clone()
}

fn in_range(world: &World, player: u32) -> Vec2 {
    let row = catalog_row();
    pos_of(world, hero(world, player)) + Vec2::new(Fixed64::ZERO, row.placement_range)
}

#[test]
fn admitted_route_keeps_receipt_identity_across_duplicate_recharge_and_replacement() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    playing(&mut world, &mut driver);
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let actor = hero(&world, 1);
    let actor_life = trusted_actor_generation(&world, 1);
    assert_eq!(actor_life, Some(generation(actor)));
    admit_in_range(&mut world, 1, 0, actor_life);
    assert_eq!(last_receipt(&world).code, "INVALID_WARD_REQUEST_ID");
    assert!(world.read_resource::<PendingWardPlacementQueue>().requests.is_empty());
    let charges = copied_ward_roster(&world, 1).unwrap().charges();
    admit_in_range(&mut world, 1, 80, None);
    drain_pending_ward_placements(&mut world);
    let missing = last_receipt(&world);
    assert_eq!(missing.admitted_correlation, 80);
    assert_eq!(missing.code, "DEAD");
    assert_eq!(copied_ward_roster(&world, 1).unwrap().charges(), charges);
    admit_in_range(&mut world, 1, 81, actor_life);
    drain_pending_ward_placements(&mut world);
    let settled = last_receipt(&world);
    assert_eq!(settled.admitted_correlation, 81);
    assert_eq!(settled.code, omoba_core::runtime::ward_transport::RECEIPT_SETTLED);
    let spent = copied_ward_roster(&world, 1).unwrap().charges();
    assert_eq!(spent, charges - 1);
    admit_in_range(&mut world, 1, 81, actor_life);
    drain_pending_ward_placements(&mut world);
    assert_eq!(last_receipt(&world).code, "DUPLICATE_WARD_INPUT");
    assert_eq!(last_receipt(&world).admitted_correlation, 81);
    let elsewhere = in_range(&world, 1) + Vec2::new(Fixed64::from_i32(1), Fixed64::ZERO);
    let placement = omoba_core::game_proto::WardPlacement {
        slot: u32::from(WARD_LOCAL_SLOT),
        x_raw: Some(elsewhere.x.raw()),
        y_raw: Some(elsewhere.y.raw()),
    };
    {
        let mut queue = world.write_resource::<PendingWardPlacementQueue>();
        route_admitted_ward_placement(1, 4, &placement, 81, actor_life, &mut queue);
    }
    drain_pending_ward_placements(&mut world);
    assert_eq!(last_receipt(&world).code, "DUPLICATE_WARD_INPUT");
    assert_eq!(copied_ward_roster(&world, 1).unwrap().charges(), spent);
    world.write_resource::<DeltaTime>().0 = row.recharge_seconds;
    begin_moba_match_tick(&mut world);
    let recharged = copied_ward_roster(&world, 1).unwrap().charges();
    assert!(recharged > spent, "recharge restores a charge before the old command is retried");
    admit_in_range(&mut world, 1, 81, actor_life);
    drain_pending_ward_placements(&mut world);
    assert_eq!(last_receipt(&world).code, "DUPLICATE_WARD_INPUT");
    assert_eq!(copied_ward_roster(&world, 1).unwrap().charges(), recharged);
    let replaced = hero(&world, 1);
    let replaced_life = trusted_actor_generation(&world, 1);
    admit_in_range(&mut world, 1, 82, replaced_life);
    let killer = hero(&world, 2);
    let pos = pos_of(&world, replaced);
    world.write_resource::<Vec<Outcome>>().push(Outcome::Damage {
        pos, phys: Fixed64::ZERO, magi: Fixed64::ZERO, real: Fixed64::from_i32(10_000),
        source: killer, target: replaced, damage_profile: 0, predeclared: false,
    });
    let mut sink = RuntimeEventVecSink::default();
    process_outcomes(&mut world, &mut sink).unwrap();
    let mut sink = RuntimeEventVecSink::default();
    process_outcomes(&mut world, &mut sink).unwrap();
    world.maintain();
    assert!(world.read_resource::<MobaMatch>().heroes.iter().any(|slot| slot.player_id == 1 && slot.entity.is_none()));
    let before_step = copied_ward_roster(&world, 1).unwrap().charges();
    driver.step(&mut world, []).unwrap();
    let lived = hero(&world, 1);
    assert_ne!(generation(lived), replaced_life.unwrap());
    assert_eq!(last_receipt(&world).code, "STALE_SOURCE");
    assert_eq!(last_receipt(&world).admitted_correlation, 82);
    assert_eq!(copied_ward_roster(&world, 1).unwrap().charges(), before_step);
}

#[test]
fn honest_ward_command_spends_one_charge_into_one_vision_source_and_settled_receipt() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    playing(&mut world, &mut driver);
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    assert_eq!(copied_ward_roster(&world, 1).unwrap().charges(), row.initial_charges);
    let before_vision = vision_count(&world);
    let before_wards = ward_count(&world);
    let hero_entity = hero(&world, 1);
    queue_in_range(&mut world, 1, Some(generation(hero_entity)), 11, WARD_LOCAL_SLOT);
    drain_pending_ward_placements(&mut world);
    let receipt = last_receipt(&world);
    assert_eq!(receipt.stage, omoba_core::runtime::ward_transport::WardReceiptStage::Settled);
    assert_eq!(receipt.code, omoba_core::runtime::ward_transport::RECEIPT_SETTLED);
    assert_ne!(receipt.code, "ADMITTED");
    let roster = copied_ward_roster(&world, 1).unwrap();
    assert_eq!(roster.charges(), row.initial_charges - 1);
    assert_eq!(vision_count(&world), before_vision + 1);
    assert_eq!(ward_count(&world), before_wards + 1);
    assert_eq!(roster.placements().len(), 1);
}

#[test]
fn rejected_ward_command_leaves_charges_vision_and_entities_unchanged() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    playing(&mut world, &mut driver);
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let charges = copied_ward_roster(&world, 1).unwrap().charges();
    let vision = vision_count(&world);
    let wards = ward_count(&world);
    let hero_entity = hero(&world, 1);
    let too_far = pos_of(&world, hero_entity) + Vec2::new(Fixed64::ZERO, row.placement_range + Fixed64::from_i32(1));
    queue_ward(&mut world, 1, Some(generation(hero_entity)), 21, too_far, WARD_LOCAL_SLOT);
    drain_pending_ward_placements(&mut world);
    let rejected = last_receipt(&world);
    assert_eq!(rejected.stage, omoba_core::runtime::ward_transport::WardReceiptStage::SettlementRejected);
    assert_eq!(rejected.code, "OUT_OF_RANGE");
    queue_in_range(&mut world, 1, Some(generation(hero_entity)), 22, 1);
    drain_pending_ward_placements(&mut world);
    assert_eq!(last_receipt(&world).code, "WRONG_SLOT");
    assert_eq!(copied_ward_roster(&world, 1).unwrap().charges(), charges);
    assert_eq!(vision_count(&world), vision);
    assert_eq!(ward_count(&world), wards);
}

#[test]
fn duplicate_replayed_and_stale_ward_requests_cannot_double_spend() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    playing(&mut world, &mut driver);
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let hero_entity = hero(&world, 1);
    let generation = generation(hero_entity);
    queue_in_range(&mut world, 1, Some(generation), 31, WARD_LOCAL_SLOT);
    drain_pending_ward_placements(&mut world);
    assert_eq!(last_receipt(&world).code, omoba_core::runtime::ward_transport::RECEIPT_SETTLED);
    let charges = copied_ward_roster(&world, 1).unwrap().charges();
    let vision = vision_count(&world);
    let wards = ward_count(&world);
    queue_in_range(&mut world, 1, Some(generation), 31, WARD_LOCAL_SLOT);
    drain_pending_ward_placements(&mut world);
    assert_eq!(last_receipt(&world).code, "DUPLICATE_WARD_INPUT");
    queue_in_range(&mut world, 1, Some(u32::MAX), 32, WARD_LOCAL_SLOT);
    drain_pending_ward_placements(&mut world);
    assert_eq!(last_receipt(&world).code, "STALE_SOURCE");
    assert_eq!(copied_ward_roster(&world, 1).unwrap().charges(), charges);
    assert_eq!(vision_count(&world), vision);
    assert_eq!(ward_count(&world), wards);
}

#[test]
fn ward_charges_persist_through_hero_absence_and_reconnect_without_reinstall() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    playing(&mut world, &mut driver);
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let before = copied_ward_roster(&world, 1).unwrap();
    let charges = before.charges();
    let revision = before.revision();
    let victim = hero(&world, 1);
    let killer = hero(&world, 2);
    let pos = pos_of(&world, victim);
    let hp = world.read_storage::<CProperty>().get(victim).expect("hp").hp;
    assert!(hp > Fixed64::from_i32(10), "a chip must be smaller than the live hero");
    let chip = Outcome::Damage {
        pos, phys: Fixed64::ZERO, magi: Fixed64::ZERO, real: Fixed64::from_i32(10),
        source: killer, target: victim, damage_profile: 0, predeclared: false,
    };
    world.write_resource::<Vec<Outcome>>().push(chip);
    let mut sink = RuntimeEventVecSink::default();
    process_outcomes(&mut world, &mut sink).unwrap();
    let mut sink = RuntimeEventVecSink::default();
    process_outcomes(&mut world, &mut sink).unwrap();
    world.maintain();
    assert!(world.read_resource::<MobaMatch>().heroes.iter().any(|slot| slot.player_id == 1 && slot.entity.is_some()));
    let chipped = decode_owner_ward_projection(&project_private_owner_ward(&world, 1, 4)).unwrap();
    assert!(chipped.placement_capable, "non-lethal damage must leave placement available");
    assert_eq!(chipped.charges, charges);
    let victim = hero(&world, 1);
    let pos = pos_of(&world, victim);
    world.write_resource::<Vec<Outcome>>().push(Outcome::Damage {
        pos, phys: Fixed64::ZERO, magi: Fixed64::ZERO, real: Fixed64::from_i32(10_000),
        source: killer, target: victim, damage_profile: 0, predeclared: false,
    });
    let mut sink = RuntimeEventVecSink::default();
    process_outcomes(&mut world, &mut sink).unwrap();
    let mut sink = RuntimeEventVecSink::default();
    process_outcomes(&mut world, &mut sink).unwrap();
    world.maintain();
    let absent = world.read_resource::<MobaMatch>().heroes.iter().find(|slot| slot.player_id == 1).unwrap().clone();
    assert!(absent.entity.is_none(), "lethal damage must clear the live hero");
    assert!(absent.deaths >= 1);
    let roster = copied_ward_roster(&world, 1).unwrap();
    assert_eq!(roster.charges(), charges);
    assert_eq!(roster.revision(), revision);
    assert_eq!(roster.catalog_id(), Some(row.catalog_id));
    let bytes = project_private_owner_ward(&world, 1, 4);
    let decoded = decode_owner_ward_projection(&bytes).unwrap();
    assert_eq!(decoded.charges, charges);
    assert_eq!(decoded.catalog_id, row.catalog_id);
    assert!(!decoded.placement_capable);
    driver.step(&mut world, []).unwrap();
    let respawned = hero(&world, 1);
    assert_ne!(generation(respawned), generation(victim));
    let restored = copied_ward_roster(&world, 1).unwrap();
    assert_eq!(restored.charges(), charges);
    assert_eq!(restored.catalog_id(), Some(row.catalog_id));
    let reconnected = project_private_owner_ward(&world, 1, 4);
    let reconnected_row = decode_owner_ward_projection(&reconnected).unwrap();
    assert_eq!(reconnected_row.charges, charges);
    assert_eq!(reconnected_row.catalog_id, row.catalog_id);
    assert!(reconnected_row.placement_capable);
    let start = owner_start(1, 4, reconnected.clone());
    let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
        &start, secure_replica_component_allowlist(), secure_replica_resource_allowlist(),
    ).unwrap();
    assert_eq!(replica.owner_ward_projection(), reconnected.as_slice());
}

#[test]
fn private_owner_ward_projection_does_not_leak_to_ally_or_enemy_and_hides_hidden_bodies() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    playing(&mut world, &mut driver);
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let hero_entity = hero(&world, 1);
    queue_in_range(&mut world, 1, Some(generation(hero_entity)), 41, WARD_LOCAL_SLOT);
    drain_pending_ward_placements(&mut world);
    project(&mut world);
    let placed = copied_ward_roster(&world, 1).unwrap().placements()[0];
    let ward = specs::world::Entity::new(placed.entity_id, specs::world::Generation::new(placed.generation as i32));
    assert!(sees(&world, 1, ward));
    assert!(!sees(&world, 2, ward), "enemy index must not contain the hidden ward");
    let owner = project_private_owner_ward(&world, 1, 3);
    let ally = project_private_owner_ward(&world, 2, 3);
    let mut side = BTreeMap::new();
    side.insert(1, owner.clone());
    assert_eq!(owner_projection_for_session(&side, 1), owner);
    assert!(owner_projection_for_session(&side, 2).is_empty());
    assert!(owner_projection_for_session(&side, 0).is_empty());
    assert_ne!(owner, ally);
    let runtime = world.read_resource::<TeamVisibilityRuntime>();
    let baseline = runtime.latest_disclosed_baseline_by_canonical.get(&placed.canonical_id).expect("authority baseline");
    assert!(disclosed_component(baseline, DISCLOSED_WARD_BODY_COMPONENT_SCHEMA_ID).is_some());
    assert!(!baseline.windows(owner.len()).any(|window| window == owner.as_slice()));
    let enemy_has_body = runtime.teams.get(&2).is_some_and(|state| state.index.current.contains(&placed.canonical_id));
    assert!(!enemy_has_body);
}

#[test]
fn disclosed_ward_body_stays_distinguishable_and_clears_when_hidden_or_removed() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    playing(&mut world, &mut driver);
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let hero_entity = hero(&world, 1);
    queue_in_range(&mut world, 1, Some(generation(hero_entity)), 51, WARD_LOCAL_SLOT);
    drain_pending_ward_placements(&mut world);
    project(&mut world);
    let placed = copied_ward_roster(&world, 1).unwrap().placements()[0];
    let ward = specs::world::Entity::new(placed.entity_id, specs::world::Generation::new(placed.generation as i32));
    assert!(world.read_storage::<Unit>().get(ward).is_none());
    assert!(world.read_storage::<Creep>().get(ward).is_none());
    let baseline = world.read_resource::<TeamVisibilityRuntime>().latest_disclosed_baseline_by_canonical.get(&placed.canonical_id).unwrap().clone();
    let bytes = disclosed_component(&baseline, DISCLOSED_WARD_BODY_COMPONENT_SCHEMA_ID).unwrap().to_vec();
    let body = decode_disclosed_ward_body(&bytes).unwrap();
    assert_eq!(body.catalog_id, row.catalog_id);
    assert_eq!(body.generation, placed.generation);
    assert!(disclosed_direct_attack_allowed(body.direct_attack));
    assert!(!disclosed_automatic_acquisition_allowed(body.automatic_acquisition));
    let render = decode_demo_render_state(disclosed_component(&baseline, DEMO_RENDER_COMPONENT_SCHEMA_ID).unwrap()).unwrap();
    assert_eq!(render.kind, 0);
    let mut components = BTreeMap::new();
    components.insert(DISCLOSED_WARD_BODY_COMPONENT_SCHEMA_ID, bytes.clone());
    world.insert(DisclosedAttackabilityGate::default());
    let filtered = world.create_entity().build();
    apply_disclosed_ward_component(&mut world, filtered, &components).unwrap();
    let stored = world.read_storage::<DisclosedWardBody>().get(filtered).copied().unwrap();
    assert_eq!(stored.catalog_id, row.catalog_id);
    assert_eq!(stored.generation, placed.generation);
    assert!(world.read_storage::<Unit>().get(filtered).is_none());
    assert!(world.read_resource::<DisclosedAttackabilityGate>().ward_automatic_disabled.contains(&filtered));
    assert!(!world.read_resource::<DisclosedAttackabilityGate>().ward_direct_denied.contains(&filtered));
    components.clear();
    apply_disclosed_ward_component(&mut world, filtered, &components).unwrap();
    assert!(world.read_storage::<DisclosedWardBody>().get(filtered).is_none());
    components.insert(DISCLOSED_WARD_BODY_COMPONENT_SCHEMA_ID, vec![1, 2, 3]);
    assert!(apply_disclosed_ward_component(&mut world, filtered, &components).is_err());
    retire_owner_wards(&mut world, 1);
    assert!(world.read_storage::<VisionSource>().get(ward).is_none(), "vision retires in the removal call");
    assert!(world.read_storage::<StealthProfile>().get(ward).is_none());
    assert!(world.read_storage::<PlacedWard>().get(ward).is_none());
    world.maintain();
    project(&mut world);
    assert!(world.read_resource::<TeamVisibilityRuntime>().latest_disclosed_baseline_by_canonical.get(&placed.canonical_id).is_none());
    assert!(!sees(&world, 1, ward), "a removed ward is not visible after the committed frame");
    assert!(!sees(&world, 2, ward));
    let legacy = world.create_entity().build();
    apply_disclosed_ward_component(&mut world, legacy, &BTreeMap::new()).unwrap();
    assert!(world.read_storage::<DisclosedWardBody>().get(legacy).is_none());
    let _ = encode_disclosed_ward_body(row.catalog_id, 1, 2, 2).unwrap();
}

#[test]
fn bootstrap_and_continuing_owner_ward_projection_agree_and_retirement_clears() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    playing(&mut world, &mut driver);
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    let epoch = 6_u64;
    let bytes = project_private_owner_ward(&world, 1, epoch);
    let continuing = project_private_owner_ward(&world, 1, epoch);
    assert_eq!(bytes, continuing);
    assert_eq!(decode_owner_ward_projection(&bytes).unwrap().player_id, 1);
    let start = owner_start(1, epoch, bytes.clone());
    let sealed = start.owner_ward_envelope.as_ref().expect("sealed bootstrap");
    assert_eq!(sealed.projection, bytes);
    assert_eq!(
        sealed.projection_hash.as_slice(),
        omoba_core::runtime::ward_transport::hash_owner_ward_projection(&bytes),
    );
    let mut replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
        &start, secure_replica_component_allowlist(), secure_replica_resource_allowlist(),
    ).unwrap();
    assert_eq!(replica.owner_ward_projection(), bytes.as_slice());
    replica.apply_continuing_owner_ward_projection(1, &continuing).unwrap();
    assert_eq!(replica.owner_ward_projection(), continuing.as_slice());
    replica.apply_continuing_owner_ward_projection(1, &[]).unwrap();
    assert!(replica.owner_ward_projection().is_empty());
    replica.apply_continuing_owner_ward_projection(1, &bytes).unwrap();
    assert!(replica.apply_continuing_owner_ward_projection(2, &bytes).is_err());
    assert!(replica.owner_ward_projection().is_empty());
    let mut stale = bytes.clone();
    assert_eq!(&bytes[..4], &omoba_core::runtime::ward_transport::OWNER_WARD_SCHEMA.to_be_bytes());
    stale[3] = 1;
    assert_ne!(stale, bytes);
    let bad = owner_start(1, epoch, stale);
    assert!(SelectiveReplicaRuntime::bootstrap_from_team_game_start(
        &bad, secure_replica_component_allowlist(), secure_replica_resource_allowlist(),
    ).is_err());
    let ally = owner_start(2, epoch, Vec::new());
    let ally_replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
        &ally, secure_replica_component_allowlist(), secure_replica_resource_allowlist(),
    ).unwrap();
    assert!(ally_replica.owner_ward_projection().is_empty());
    assert_eq!(negotiate_ward_protocol(0, ""), Ok(false));
    assert_eq!(negotiate_ward_protocol(WARD_PROTOCOL_VERSION, "old"), Err("WARD_RULES_HASH_MISMATCH"));
    retire_owner_wards(&mut world, 1);
    let retired = project_private_owner_ward(&world, 1, epoch);
    let retired_row = decode_owner_ward_projection(&retired).unwrap();
    assert_eq!(retired_row.charges, decode_owner_ward_projection(&bytes).unwrap().charges);
    assert!(copied_ward_roster(&world, 1).unwrap().placements().is_empty());
    let _ = OwnerWardProjection { player_id: 1, view_epoch: epoch, catalog_id: 0, charges: 0, max_charges: 0, recharge_remaining_raw: 0, revision: 1, placement_capable: false, elapsed_raw: 1 };
}

#[test]
fn private_projection_carries_settled_and_rejected_outcomes_for_the_same_epoch() {
    let row = catalog_row();
    let (mut world, mut driver) = world();
    playing(&mut world, &mut driver);
    set_authoritative_trinket_loadout(&mut world, 1, Some(row.catalog_id)).unwrap();
    world.write_resource::<WardAdmissionEpoch>().by_player.insert(1, 4);
    let actor_life = trusted_actor_generation(&world, 1);
    let charges = copied_ward_roster(&world, 1).unwrap().charges();
    let rejected = omoba_core::game_proto::WardPlacement { slot: 0, x_raw: None, y_raw: Some(1) };
    {
        let mut queue = world.write_resource::<PendingWardPlacementQueue>();
        route_admitted_ward_placement(1, 4, &rejected, 70, actor_life, &mut queue);
    }
    drain_pending_ward_placements(&mut world);
    assert!(world.read_resource::<PendingWardPlacementQueue>().receipts.is_empty());
    let (dead_air, rejected_block) = decode_owner_ward_private(&project_private_owner_ward(&world, 1, 4)).unwrap();
    assert_eq!(dead_air.charges, charges);
    assert_eq!(rejected_block.count, 1);
    assert_eq!(rejected_block.rows[0].admitted_correlation, 70);
    assert_eq!(rejected_block.rows[0].stage, OWNER_WARD_OUTCOME_REJECTED);
    assert_eq!(rejected_block.rows[0].actor_generation, actor_life.unwrap_or(0));
    admit_in_range(&mut world, 1, 71, actor_life);
    drain_pending_ward_placements(&mut world);
    let (spent, settled) = decode_owner_ward_private(&project_private_owner_ward(&world, 1, 4)).unwrap();
    assert_eq!(spent.charges, charges - 1);
    assert_eq!(settled.rows[1].admitted_correlation, 71);
    assert_eq!(settled.rows[1].stage, OWNER_WARD_OUTCOME_SETTLED);
    assert_eq!(settled.rows[1].actor_generation, actor_life.unwrap());
    let (other_epoch, cleared) = decode_owner_ward_private(&project_private_owner_ward(&world, 1, 9)).unwrap();
    assert_eq!(other_epoch.charges, charges - 1);
    assert_eq!(cleared.count, 0);
    assert_eq!(cleared.uncertain_through, 0);
    let (_ally, ally_outcomes) = decode_owner_ward_private(&project_private_owner_ward(&world, 2, 4)).unwrap();
    assert_eq!(ally_outcomes.count, 0);
}

#[test]
fn bootstrap_view_epoch_reaches_the_outcome_journal_without_a_manual_player_insert() {
    let row = catalog_row();
    let (mut world, _driver) = world();
    world.write_resource::<DeltaTime>().0 = Fixed64::from_raw(1);
    world.write_resource::<MobaMatch>().config.warmup = Fixed64::from_i32(1);
    assert!(!begin_moba_match_tick(&mut world), "warmup holds the match before the owner epoch is installed");
    let start = owner_start(1, 1, Vec::new());
    let epoch = start.view_epoch.as_ref().map(|value| value.value).unwrap_or(0);
    install_ward_admission_epochs(&mut world, [(start.player_id, epoch), (0, epoch), (start.player_id, 0)]);
    {
        let installed = world.read_resource::<WardAdmissionEpoch>();
        assert_eq!(installed.by_player.get(&start.player_id).copied(), Some(epoch));
        assert!(!installed.by_player.contains_key(&0));
    }
    world.write_resource::<MobaMatch>().config.warmup = Fixed64::ZERO;
    assert!(begin_moba_match_tick(&mut world));
    set_authoritative_trinket_loadout(&mut world, start.player_id, Some(row.catalog_id)).unwrap();
    let actor_life = trusted_actor_generation(&world, start.player_id);
    admit_in_range(&mut world, start.player_id, 901, actor_life);
    drain_pending_ward_placements(&mut world);
    let (projection, outcomes) = decode_owner_ward_private(&project_private_owner_ward(&world, start.player_id, epoch)).unwrap();
    assert_eq!(projection.view_epoch, epoch);
    assert_eq!(outcomes.count, 1);
    assert_eq!(outcomes.rows[0].admitted_correlation, 901);
    assert_eq!(outcomes.rows[0].stage, OWNER_WARD_OUTCOME_SETTLED);
    let (stale, stale_outcomes) = decode_owner_ward_private(&project_private_owner_ward(&world, start.player_id, 0)).unwrap();
    assert_eq!(stale.view_epoch, 0);
    assert_eq!(stale_outcomes.count, 0);
}

fn owner_start(player: u32, epoch: u64, bytes: Vec<u8>) -> TeamGameStart {
    let disclosed_world = Vec::new();
    let mut start = TeamGameStart {
        protocol_version: omoba_core::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION,
        player_id: player,
        team_id: 1,
        replica_start_tick: 1,
        next_team_sequence: 1,
        tick_rate_hz: 60,
        view_epoch: Some(ViewEpoch { value: epoch }),
        ward_protocol_version: if bytes.is_empty() { 0 } else { WARD_PROTOCOL_VERSION },
        ward_rules_hash: if bytes.is_empty() { String::new() } else { omoba_template_ids::CONTENT_CATALOG_DATA_HASH.to_owned() },
        owner_ward_projection: bytes.clone(),
        filtered_snapshot: Some(FilteredTeamSnapshot {
            team_id: 1,
            authoritative_tick: 1,
            view_epoch: Some(ViewEpoch { value: epoch }),
            disclosed_world,
            filtered_snapshot_hash: EMPTY_SHA256.to_vec(),
            ..FilteredTeamSnapshot::default()
        }),
        ..TeamGameStart::default()
    };
    if !bytes.is_empty() {
        stamp_owner_ward_admission(
            &mut start,
            true,
            owner_ward_session_binding("ward-bootstrap-test", player),
            &bytes,
        );
    }
    start
}
