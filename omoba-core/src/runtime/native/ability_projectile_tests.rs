//! L1 contract for generic ability projectiles: flight time, hit rules,
//! deterministic order, target/caster loss, walls and fog projection.
use super::*;
use crate::comp::run_now;
use crate::runtime::native::tick::ability_projectile_tick::Sys as FlightSys;

fn world() -> World {
    let mut w = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
    w.insert(crate::runtime::TowerTemplateRegistry::default());
    w
}

fn at(x: i32, y: i32) -> Vec2 { Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)) }

fn body(w: &mut World, x: i32, y: i32, team: i32) -> Entity {
    w.create_entity().with(Pos(at(x, y)))
        .with(Faction::new(if team == 1 { FactionType::Player } else { FactionType::Enemy }, team))
        .with(CProperty { hp: Fixed64::from_i32(1000), mhp: Fixed64::from_i32(1000), msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO })
        .build()
}

fn linear(caster: Entity, aim: Vec2) -> Spec {
    Spec { source_id: caster.id(), source_generation: caster.gen().id() as u32, ability: "fixture_bolt".into(),
        motion: Motion::Linear { aim }, speed: Fixed64::from_i32(1200), radius: Fixed64::from_i32(10),
        distance: Fixed64::from_i32(1000), lifetime: Fixed64::from_i32(2), max_hits: 1, wall: WallPolicy::Pass,
        target_lost: TargetLostPolicy::Expire, caster_death: CasterDeathPolicy::Continue,
        amount: Fixed64::from_i32(100), kind: DamageKind::Pure, profile: DamageProfile::TRUE.bits(), slow: None, control: None,
        dot: None, buildings: false, forced: None }
}

fn homing(caster: Entity, target: Entity) -> Spec {
    Spec { motion: Motion::Homing { target_id: target.id(), target_generation: target.gen().id() as u32 },
        distance: Fixed64::ZERO, ..linear(caster, at(1, 0)) }
}

fn settle(w: &mut World) { crate::runtime::process_outcomes(w, &mut crate::runtime::RuntimeEventVecSink::default()).unwrap(); }

fn cast(w: &mut World, caster: Entity, spec: &Spec) {
    w.write_resource::<Vec<Outcome>>().push(Outcome::AddBuff { target: caster, buff_id: BUFF.into(),
        duration: spec.lifetime, payload: spec.payload() });
    settle(w);
}

fn hp(w: &World, e: Entity) -> i32 { (w.read_storage::<CProperty>().get(e).unwrap().hp.raw() / 1024) as i32 }

fn live(w: &World) -> usize { w.try_fetch::<AbilityProjectiles>().map_or(0, |rows| rows.0.len()) }

/// Fixed `hz` steps; returns the first tick on which each watched entity lost HP.
fn fly(w: &mut World, hz: i64, ticks: u64, watch: &[Entity]) -> Vec<Option<u64>> {
    let mut first = vec![None; watch.len()];
    let mut elapsed = 0;
    for tick in 1..=ticks {
        let next = tick as i64 * 1024 / hz;
        w.insert(DeltaTime(Fixed64::from_raw(next - elapsed)));
        elapsed = next;
        w.write_resource::<Tick>().0 = tick;
        run_now::<FlightSys>(w);
        settle(w);
        for (slot, entity) in watch.iter().enumerate() {
            if first[slot].is_none() && hp(w, *entity) < 1000 { first[slot] = Some(tick); }
        }
    }
    first
}

#[test]
fn near_and_far_targets_are_hit_on_different_flight_ticks_at_every_rate() {
    for hz in [15, 60, 120] {
        // Linear piercing shot: arrival time follows distance / speed.
        let mut w = world();
        let caster = body(&mut w, 0, 0, 1);
        let near = body(&mut w, 300, 0, 2);
        let far = body(&mut w, 900, 0, 2);
        cast(&mut w, caster, &Spec { max_hits: 2, ..linear(caster, at(1, 0)) });
        assert_eq!(hp(&w, near), 1000, "launch is never instant damage");
        let hits = fly(&mut w, hz, hz as u64 * 2, &[near, far]);
        let (near_tick, far_tick) = (hits[0].unwrap(), hits[1].unwrap());
        // Contact after 290 / 890 units at 1200 u/s on the exact raw-time schedule
        // (each tick's dt is the integer partition of 1024 raw per second).
        let expected = |distance: i64| (1..).find(|t: &i64| (t * 1024 / hz) * 1200 >= distance * 1024).unwrap();
        assert_eq!(near_tick as i64, expected(290), "{hz}Hz near");
        assert_eq!(far_tick as i64, expected(890), "{hz}Hz far");
        assert!(far_tick > near_tick);
        assert_eq!((hp(&w, near), hp(&w, far)), (900, 900));
        // Homing: two separate casts at different ranges.
        let mut w = world();
        let caster = body(&mut w, 0, 0, 1);
        let near = body(&mut w, 0, 240, 2);
        let far = body(&mut w, 0, 960, 2);
        cast(&mut w, caster, &homing(caster, near));
        cast(&mut w, caster, &homing(caster, far));
        let hits = fly(&mut w, hz, hz as u64 * 2, &[near, far]);
        assert!(hits[1].unwrap() > hits[0].unwrap() + hz as u64 / 4, "{hz}Hz homing near {:?} far {:?}", hits[0], hits[1]);
        assert_eq!(live(&w), 0);
    }
}

#[test]
fn stop_on_hit_takes_first_contact_and_ends_at_contact_point() {
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let far = body(&mut w, 600, 0, 2);
    let near = body(&mut w, 400, 0, 2);
    cast(&mut w, caster, &linear(caster, at(1, 0)));
    let projectile = *w.read_resource::<AbilityProjectiles>().0.keys().next().unwrap();
    fly(&mut w, 60, 120, &[]);
    assert_eq!((hp(&w, near), hp(&w, far)), (900, 1000));
    assert_eq!(live(&w), 0);
    assert!(!w.entities().is_alive(projectile));
}

#[test]
fn piercing_hits_each_target_once_even_while_overlapping_for_many_steps() {
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let target = body(&mut w, 500, 0, 2);
    // Radius 300 at 120 u/s keeps the target inside the capsule for ~5 s of steps.
    cast(&mut w, caster, &Spec { max_hits: 8, speed: Fixed64::from_i32(120), radius: Fixed64::from_i32(300),
        lifetime: Fixed64::from_i32(8), ..linear(caster, at(1, 0)) });
    fly(&mut w, 60, 60 * 9, &[]);
    assert_eq!(hp(&w, target), 900, "dedupe by exact entity life");
}

#[test]
fn equal_contacts_resolve_by_canonical_identity_and_respect_hit_budget() {
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    // Symmetric about the axis: identical contact time.
    let first = body(&mut w, 300, 5, 2);
    let second = body(&mut w, 300, -5, 2);
    let behind = body(&mut w, 700, 0, 2);
    cast(&mut w, caster, &Spec { max_hits: 2, ..linear(caster, at(1, 0)) });
    let order: Vec<_> = {
        let mut seen = Vec::new();
        for tick in 1..=120 {
            w.insert(DeltaTime(Fixed64::from_raw(17)));
            w.write_resource::<Tick>().0 = tick;
            run_now::<FlightSys>(&w);
            for outcome in w.read_resource::<Vec<Outcome>>().iter() {
                if let Outcome::AbilityProjectileStep { .. } = outcome { seen.push(tick); }
            }
            settle(&mut w);
            if live(&w) == 0 { break; }
        }
        seen
    };
    assert!(!order.is_empty());
    assert_eq!((hp(&w, first), hp(&w, second), hp(&w, behind)), (900, 900, 1000));
    // Budget of one: the lower canonical identity wins a tie.
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let first = body(&mut w, 300, 5, 2);
    let second = body(&mut w, 300, -5, 2);
    cast(&mut w, caster, &linear(caster, at(1, 0)));
    fly(&mut w, 60, 60, &[]);
    assert_eq!((hp(&w, first), hp(&w, second)), (900, 1000));
}

#[test]
fn allies_buildings_and_the_caster_are_never_hit() {
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let ally = body(&mut w, 200, 0, 1);
    let building = body(&mut w, 400, 0, 2);
    w.write_storage::<IsBuilding>().insert(building, IsBuilding).unwrap();
    let enemy = body(&mut w, 600, 0, 2);
    cast(&mut w, caster, &linear(caster, at(1, 0)));
    fly(&mut w, 60, 90, &[]);
    assert_eq!((hp(&w, caster), hp(&w, ally), hp(&w, building), hp(&w, enemy)), (1000, 1000, 1000, 900));
}

#[test]
fn homing_target_loss_follows_the_authored_policy_without_damage() {
    for policy in [TargetLostPolicy::Expire, TargetLostPolicy::LastPosition] {
        let mut w = world();
        let caster = body(&mut w, 0, 0, 1);
        let target = body(&mut w, 900, 0, 2);
        let bystander = body(&mut w, 900, 0, 2);
        cast(&mut w, caster, &Spec { target_lost: policy, ..homing(caster, target) });
        fly(&mut w, 60, 10, &[]);
        let projectile = *w.read_resource::<AbilityProjectiles>().0.keys().next().unwrap();
        w.delete_entity(target).unwrap();
        w.maintain();
        let replacement = body(&mut w, 900, 0, 2);
        assert_eq!(replacement.id(), target.id(), "a reused index is a different life");
        fly(&mut w, 60, 120, &[]);
        assert_eq!((hp(&w, bystander), hp(&w, replacement)), (1000, 1000), "{policy:?}");
        assert_eq!(live(&w), 0);
        assert!(!w.entities().is_alive(projectile));
    }
}

#[test]
fn caster_death_continues_with_launch_allegiance_or_expires_by_policy() {
    for (policy, mode) in [(CasterDeathPolicy::Continue, 0), (CasterDeathPolicy::Continue, 1), (CasterDeathPolicy::Expire, 0)] {
        let mut w = world();
        let caster = body(&mut w, 0, 0, 1);
        let enemy = body(&mut w, 800, 0, 2);
        cast(&mut w, caster, &Spec { caster_death: policy, ..linear(caster, at(1, 0)) });
        fly(&mut w, 60, 5, &[]);
        if mode == 0 {
            w.write_storage::<CProperty>().get_mut(caster).unwrap().hp = Fixed64::ZERO;
        } else {
            w.delete_entity(caster).unwrap();
            w.maintain();
            let reuse = body(&mut w, 0, 0, 2);
            assert_eq!(reuse.id(), caster.id());
        }
        fly(&mut w, 60, 90, &[]);
        let expected = if policy == CasterDeathPolicy::Continue { 900 } else { 1000 };
        assert_eq!(hp(&w, enemy), expected, "{policy:?} mode{mode}");
        assert_eq!(live(&w), 0);
    }
}

#[test]
fn wall_policy_stop_ends_at_public_terrain_and_pass_ignores_it() {
    for (wall, expected) in [(WallPolicy::Stop, 1000), (WallPolicy::Pass, 900)] {
        let mut w = world();
        let block = |x: f32, y: f32| vek::Vec2::new(x, y);
        w.insert(BlockedRegions(vec![BlockedRegion { name: "wall".into(),
            points: vec![block(300., -50.), block(320., -50.), block(320., 50.), block(300., 50.)] }]));
        let caster = body(&mut w, 0, 0, 1);
        let enemy = body(&mut w, 500, 0, 2);
        cast(&mut w, caster, &Spec { wall, ..linear(caster, at(1, 0)) });
        let projectile = *w.read_resource::<AbilityProjectiles>().0.keys().next().unwrap();
        let mut last = Vec2::ZERO;
        for tick in 1..=90 {
            w.insert(DeltaTime(Fixed64::from_raw(17)));
            w.write_resource::<Tick>().0 = tick;
            run_now::<FlightSys>(&w);
            if let Some(pos) = w.read_storage::<Pos>().get(projectile) { last = pos.0; }
            settle(&mut w);
        }
        assert_eq!(hp(&w, enemy), expected, "{wall:?}");
        if wall == WallPolicy::Stop {
            assert!(last.x <= Fixed64::from_i32(300) && last.x > Fixed64::from_i32(280), "stopped at wall face, got {last:?}");
        }
    }
}

#[test]
fn on_hit_slow_and_control_apply_only_to_hit_targets() {
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let enemy = body(&mut w, 300, 0, 2);
    let missed = body(&mut w, 300, 400, 2);
    cast(&mut w, caster, &Spec { slow: Some((Fixed64::from_raw(512), Fixed64::ONE)),
        control: Some((ControlEffectKind::Stun, Fixed64::ONE)), ..linear(caster, at(1, 0)) });
    fly(&mut w, 60, 30, &[]);
    let buffs = w.read_resource::<crate::runtime::BuffStore>();
    assert!(buffs.is_stunned(enemy) && !buffs.is_stunned(missed));
    let handle = omb_script_abi::types::EntityHandle { id: caster.id(), gen: caster.gen().id() as u32 };
    assert!(buffs.has(enemy, &omb_script_abi::buff_ids::generic_slow_buff_id("fixture_bolt", handle)));
}

#[test]
fn launch_admission_rejects_invalid_shapes_allies_and_live_budget() {
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let ally = body(&mut w, 300, 0, 1);
    let enemy = body(&mut w, 300, 0, 2);
    let valid = linear(caster, at(1, 0));
    assert_eq!(Spec::from_payload(BUFF, valid.lifetime, &valid.payload()), Some(valid.clone()));
    assert!(Spec::from_payload(BUFF, Fixed64::ONE, &valid.payload()).is_none(), "duration must equal lifetime");
    assert!(Spec::from_payload("other", valid.lifetime, &valid.payload()).is_none());
    for (field, value) in [("speed", json!(0)), ("radius", json!(-1)), ("max_hits", json!(65)), ("wall", json!("bounce")),
        ("kind", json!("fire")), ("profile", json!(0)), ("ability", json!("../x")), ("lifetime", json!(31 * 1024)), ("extra", json!(1))] {
        let mut bad = valid.payload();
        bad[KEY][field] = value;
        assert!(Spec::from_payload(BUFF, valid.lifetime, &bad).is_none(), "{field}");
    }
    let homing_wall = Spec { wall: WallPolicy::Stop, ..homing(caster, enemy) };
    assert!(Spec::from_payload(BUFF, homing_wall.lifetime, &homing_wall.payload()).is_none());
    assert!(admitted(&w, caster, BUFF, valid.lifetime, &homing(caster, ally).payload(), Some(caster)).is_none());
    assert!(admitted(&w, caster, BUFF, valid.lifetime, &homing(caster, enemy).payload(), Some(caster)).is_some());
    assert!(admitted(&w, ally, BUFF, valid.lifetime, &valid.payload(), None).is_none(), "envelope targets its own source");
    assert!(admitted(&w, caster, BUFF, valid.lifetime, &linear(caster, at(0, 0)).payload(), None).is_none(), "zero axis");
    for _ in 0..MAX_LIVE_PER_SOURCE { cast(&mut w, caster, &valid); }
    assert_eq!(live(&w), MAX_LIVE_PER_SOURCE);
    cast(&mut w, caster, &valid);
    assert_eq!(live(&w), MAX_LIVE_PER_SOURCE, "per-source budget");
}

#[test]
fn fog_projection_discloses_flight_only_through_owner_team_or_vision() {
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let enemy = body(&mut w, 2000, 0, 2);
    cast(&mut w, caster, &homing(caster, enemy));
    let projectile = *w.read_resource::<AbilityProjectiles>().0.keys().next().unwrap();
    fly(&mut w, 60, 6, &[]);
    let view = crate::runtime::build_wave_b_read_view(&w, 6);
    let row = view.entities.iter().find(|row| row.canonical_id == crate::runtime::canonical_entity_id(projectile)).unwrap();
    assert_eq!((row.scope, row.owner_team, row.team), (ReplicationScopeKind::OwnerTeam, Some(1), 1));
    let render = crate::runtime::decode_demo_render_state(
        crate::runtime::disclosed_component(&row.disclosed_baseline, crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID).unwrap()).unwrap();
    assert_eq!(render.kind, RENDER_KIND);
    let flight = decode_disclosed_flight(crate::runtime::disclosed_component(&row.disclosed_baseline, DISCLOSED_SCHEMA_ID).unwrap()).unwrap();
    assert_eq!((flight.motion, flight.speed_raw, flight.radius_raw), (2, 1200 * 1024, 10 * 1024));
    let identity = crate::runtime::disclosed_component(&row.disclosed_baseline, crate::runtime::DISCLOSED_PRESENTATION_IDENTITY_SCHEMA_ID).unwrap();
    assert_eq!(crate::runtime::decode_disclosed_presentation_identity(identity), Some("fixture_bolt"));
    // The homing target's identity/position is not part of the projectile's components.
    assert!(!row.disclosed_baseline.windows(8).any(|w| w == crate::runtime::canonical_entity_id(enemy).to_be_bytes()));
    let brushes = Default::default();
    let source = |team: u32, x: i32| crate::runtime::CommittedVisionSource { canonical_id: 99, team, position: at(x, 0),
        radius: Fixed64::from_i32(300), detection_level: 0 };
    assert!(crate::runtime::visibility::entity_visible_with_brush(row, 1, &[], &[], &brushes), "owner team always sees it");
    assert!(!crate::runtime::visibility::entity_visible_with_brush(row, 2, &[source(2, 2000)], &[], &brushes), "fogged for enemy");
    assert!(crate::runtime::visibility::entity_visible_with_brush(row, 2, &[source(2, 100)], &[], &brushes), "enemy vision reveals it");
}

#[test]
fn flight_wire_is_bounded_and_rejects_malformed_frames() {
    let flight = DisclosedFlight { motion: 1, speed_raw: 1024, radius_raw: 0 };
    let bytes = encode_disclosed_flight(flight);
    assert_eq!(bytes.len(), 18);
    assert_eq!(decode_disclosed_flight(&bytes), Some(flight));
    for end in 0..bytes.len() { assert!(decode_disclosed_flight(&bytes[..end]).is_none()); }
    for (index, value) in [(0, 1), (0, 3), (1, 0), (1, 3)] { let mut bad = bytes.clone(); bad[index] = value; assert!(decode_disclosed_flight(&bad).is_none()); }
    assert!(crate::runtime::secure_replica_component_allowlist().contains(&DISCLOSED_SCHEMA_ID));
}

#[test]
fn authority_hash_state_is_canonical() {
    let mut a = world();
    let caster = body(&mut a, 0, 0, 1);
    body(&mut a, 900, 0, 2);
    cast(&mut a, caster, &linear(caster, at(1, 0)));
    let first = a.read_resource::<AbilityProjectiles>().hash_value();
    assert_eq!(first, a.read_resource::<AbilityProjectiles>().hash_value());
    fly(&mut a, 60, 1, &[]);
    assert_ne!(first, a.read_resource::<AbilityProjectiles>().hash_value(), "flight progress is hashed");
}

#[test]
fn homing_reaches_enemy_buildings_only_when_the_spec_allows_it() {
    for allow in [false, true] {
        let mut w = world();
        let caster = body(&mut w, 0, 0, 1);
        let tower = body(&mut w, 300, 0, 2);
        w.write_storage::<IsBuilding>().insert(tower, IsBuilding).unwrap();
        let spec = Spec { buildings: allow, ..homing(caster, tower) };
        cast(&mut w, caster, &spec);
        assert_eq!(live(&w), usize::from(allow), "building target admitted only with buildings=true");
        fly(&mut w, 60, 60, &[]);
        assert_eq!(hp(&w, tower), if allow { 900 } else { 1000 });
    }
    // Linear flights never take the building flag; buildings stay unhittable for them.
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    assert!(!Spec { buildings: true, ..linear(caster, at(1, 0)) }.valid());
}

#[test]
fn on_hit_dot_applies_the_generic_periodic_damage_to_each_hit_target() {
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let first = body(&mut w, 200, 0, 2);
    let second = body(&mut w, 400, 0, 2);
    let missed = body(&mut w, 300, 400, 2);
    let spec = Spec { max_hits: 8, dot: Some((Fixed64::from_i32(50), Fixed64::from_i32(5))), kind: DamageKind::Magical,
        ..linear(caster, at(1, 0)) };
    // The envelope round-trips with the optional fields and stays canonical.
    let payload = spec.payload();
    assert_eq!(Spec::from_payload(BUFF, spec.lifetime, &payload), Some(spec.clone()));
    let mut bad = payload.clone();
    bad[KEY]["buildings"] = serde_json::json!(false);
    assert_eq!(Spec::from_payload(BUFF, spec.lifetime, &bad), None, "buildings=false is not canonical");
    cast(&mut w, caster, &spec);
    fly(&mut w, 60, 30, &[]);
    let handle = omb_script_abi::types::EntityHandle { id: caster.id(), gen: caster.gen().id() as u32 };
    let key = omb_script_abi::buff_ids::generic_dot_buff_id("fixture_bolt", handle);
    let buffs = w.read_resource::<crate::runtime::BuffStore>();
    assert!(buffs.has(first, &key) && buffs.has(second, &key) && !buffs.has(missed, &key));
    let row = buffs.get(first, &key).unwrap();
    assert_eq!(row.remaining.raw() > 0, true);
    assert_eq!(row.payload, crate::runtime::ability_runtime::periodic_damage::PeriodicDamageSpec::payload(
        Fixed64::from_i32(50), handle, DamageKind::Magical));
}

#[test]
fn retarget_on_loss_locks_the_nearest_live_hostile_or_expires() {
    // The locked target dies mid-flight: the nearest hostile non-building within
    // the radius of the projectile is hit instead (canonical id breaks ties).
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let target = body(&mut w, 900, 0, 2);
    let near = body(&mut w, 700, 150, 2);
    let far = body(&mut w, 700, 900, 2);
    let tower = body(&mut w, 650, 0, 2);
    w.write_storage::<IsBuilding>().insert(tower, IsBuilding).unwrap();
    let ally = body(&mut w, 600, 50, 1);
    // After 10 ticks at speed 1200 the flight is near (200, 0): `near` is 522 away, `far` 1029.
    let spec = Spec { target_lost: TargetLostPolicy::Retarget(Fixed64::from_i32(600)), ..homing(caster, target) };
    assert_eq!(Spec::from_payload(BUFF, spec.lifetime, &spec.payload()), Some(spec.clone()), "canonical round trip");
    cast(&mut w, caster, &spec);
    fly(&mut w, 60, 10, &[]);
    w.write_storage::<CProperty>().get_mut(target).unwrap().hp = Fixed64::ZERO;
    fly(&mut w, 60, 120, &[]);
    assert_eq!((hp(&w, near), hp(&w, far), hp(&w, tower), hp(&w, ally)), (900, 1000, 1000, 1000));
    assert_eq!(live(&w), 0);
    // Nothing within the radius: the flight expires without damage.
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let target = body(&mut w, 900, 0, 2);
    let distant = body(&mut w, 300, 1500, 2);
    cast(&mut w, caster, &Spec { target_lost: TargetLostPolicy::Retarget(Fixed64::from_i32(300)), ..homing(caster, target) });
    fly(&mut w, 60, 10, &[]);
    w.delete_entity(target).unwrap();
    w.maintain();
    fly(&mut w, 60, 120, &[]);
    assert_eq!((hp(&w, distant), live(&w)), (1000, 0));
    // Only homing flights may retarget, within a bounded radius.
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    assert!(!Spec { target_lost: TargetLostPolicy::Retarget(Fixed64::from_i32(300)), ..linear(caster, at(1, 0)) }.valid());
    let other = body(&mut w, 100, 0, 2);
    assert!(!Spec { target_lost: TargetLostPolicy::Retarget(Fixed64::from_i32(2_001)), ..homing(caster, other) }.valid());
    let mut bad = homing(caster, other).payload();
    bad[KEY]["retarget_radius"] = serde_json::json!(1024);
    assert_eq!(Spec::from_payload(BUFF, homing(caster, other).lifetime, &bad), None, "radius without the retarget policy");
}

fn cast_bounce(w: &mut World, caster: Entity, spec: &Spec, hits: u16, radius: i32) {
    w.write_resource::<Vec<Outcome>>().push(Outcome::AddBuff { target: caster, buff_id: BUFF.into(),
        duration: spec.lifetime, payload: with_bounce(spec.payload(), hits, Fixed64::from_i32(radius)) });
    settle(w);
}

#[test]
fn bounce_chain_hits_at_most_its_total_and_only_units_in_radius_of_the_last_hit() {
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let first = body(&mut w, 200, 0, 2);
    let second = body(&mut w, 600, 0, 2);
    let far = body(&mut w, 1400, 0, 2);
    let ally = body(&mut w, 250, 0, 1);
    let spec = Spec { lifetime: Fixed64::from_i32(5), ..homing(caster, first) };
    cast_bounce(&mut w, caster, &spec, 5, 500);
    fly(&mut w, 60, 600, &[]);
    // Only first and second are in reach of each other: the chain alternates
    // between them (only the unit just hit is excluded) for 5 hits in total.
    let (a, b) = (1000 - hp(&w, first), 1000 - hp(&w, second));
    assert_eq!(a + b, 500, "five hits of 100: first {a} second {b}");
    assert!(a >= 200 && b >= 200, "alternates: first {a} second {b}");
    assert_eq!((hp(&w, far), hp(&w, ally)), (1000, 1000));
    assert_eq!(live(&w), 0, "retires after the last hit");
}

#[test]
fn bounce_chain_ends_early_without_another_unit_and_rejects_bad_rows() {
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let lone = body(&mut w, 200, 0, 2);
    cast_bounce(&mut w, caster, &homing(caster, lone), 5, 500);
    fly(&mut w, 60, 240, &[]);
    assert_eq!(hp(&w, lone), 900, "nothing else in reach: one hit");
    assert_eq!(live(&w), 0);
    let spec = homing(caster, lone);
    for (hits, radius) in [(1u16, 500), (17, 500), (5, 0), (5, 2001)] {
        assert!(Spec::from_payload(BUFF, spec.lifetime, &with_bounce(spec.payload(), hits, Fixed64::from_i32(radius))).is_none(), "{hits} {radius}");
    }
    let linear_spec = linear(caster, at(1, 0));
    assert!(bounce_of(&with_bounce(linear_spec.payload(), 5, Fixed64::from_i32(500))).is_none(), "homing only");
    assert_eq!(bounce_of(&with_bounce(spec.payload(), 5, Fixed64::from_i32(500))), Some((5, Fixed64::from_i32(500))));
}

fn cast_payload(w: &mut World, caster: Entity, spec: &Spec, payload: Value) {
    assert!(Spec::from_payload(BUFF, spec.lifetime, &payload).is_some(), "admitted envelope: {payload}");
    w.write_resource::<Vec<Outcome>>().push(Outcome::AddBuff { target: caster, buff_id: BUFF.into(), duration: spec.lifetime, payload });
    settle(w);
}

fn request_dodge(w: &mut World, unit: Entity) {
    w.write_resource::<Vec<Outcome>>().push(Outcome::AddBuff { target: unit, buff_id: DODGE_BUFF.into(), duration: Fixed64::ONE,
        payload: dodge_payload(unit.id(), unit.gen().id() as u32) });
    settle(w);
}

#[test]
fn dodge_retires_only_dodgeable_homing_flights_on_the_dodger_without_damage() {
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let dodger = body(&mut w, 900, 0, 2);
    let other = body(&mut w, 0, 900, 2);
    let dodgeable = homing(caster, dodger);
    cast_payload(&mut w, caster, &dodgeable, with_dodgeable(dodgeable.payload()));
    cast(&mut w, caster, &homing(caster, dodger));
    cast_payload(&mut w, caster, &homing(caster, other), with_dodgeable(homing(caster, other).payload()));
    // A widening linear flight is never dodged (Dota: tracking projectiles only).
    let line = Spec { max_hits: 4, ..linear(caster, at(1, 0)) };
    cast(&mut w, caster, &line);
    fly(&mut w, 60, 10, &[]);
    assert_eq!(live(&w), 4);
    // Another unit's dodge request is refused; the dodger's own retires one flight.
    w.write_resource::<Vec<Outcome>>().push(Outcome::AddBuff { target: other, buff_id: DODGE_BUFF.into(), duration: Fixed64::ONE,
        payload: dodge_payload(dodger.id(), dodger.gen().id() as u32) });
    settle(&mut w);
    assert_eq!(live(&w), 4);
    request_dodge(&mut w, dodger);
    assert_eq!(live(&w), 3);
    fly(&mut w, 60, 120, &[]);
    // The non-dodgeable bolt and the line still hit the dodger; the dodged one never does.
    assert_eq!((hp(&w, dodger), hp(&w, other)), (800, 900));
    assert_eq!(live(&w), 0);
    // Canonical rows: homing only, present only when true.
    assert!(!dodgeable_of(&with_dodgeable(line.payload())));
    assert!(Spec::from_payload(BUFF, line.lifetime, &with_dodgeable(line.payload())).is_none());
    let mut bad = homing(caster, dodger).payload();
    bad[KEY]["dodgeable"] = serde_json::json!(false);
    assert!(Spec::from_payload(BUFF, dodgeable.lifetime, &bad).is_none());
}

#[test]
fn dodge_retires_normal_attack_projectiles_tracking_the_dodger_only() {
    let mut w = world();
    let attacker = body(&mut w, 0, 0, 1);
    let dodger = body(&mut w, 600, 0, 2);
    let other = body(&mut w, 0, 600, 2);
    let shoot = |w: &mut World, target: Entity, normal: bool| {
        let projectile = w.create_entity().with(Pos(at(10, 0))).with(Projectile { damage_stage: Default::default(),
            time_left: Fixed64::from_i32(3), owner: attacker, target: Some(target), tpos: at(600, 0), radius: Fixed64::ZERO,
            msd: Fixed64::from_i32(900), damage_phys: Fixed64::from_i32(50), damage_magi: Fixed64::ZERO, damage_real: Fixed64::ZERO,
            damage_profile: DamageProfile::NORMAL.bits(), slow_factor: Fixed64::ZERO, slow_duration: Fixed64::ZERO,
            hit_radius: Fixed64::ZERO, stun_duration: Fixed64::ZERO, kind_id: 0, generation: 0 }).build();
        if w.try_fetch::<ProjectileOrigins>().is_none() { w.insert(ProjectileOrigins::default()); }
        w.write_resource::<ProjectileOrigins>().0.insert(projectile, ProjectileOrigin { source: attacker, normal_attack: normal,
            script_attack: false, attack_missed: false, attack_critical: false, attack_lifesteal: Fixed64::ZERO,
            launch_position: None, credit_team: None, credit_player: None, script: None });
        projectile
    };
    let attack = shoot(&mut w, dodger, true);
    let script_shot = shoot(&mut w, dodger, false);
    let elsewhere = shoot(&mut w, other, true);
    let outcomes = {
        let payload = dodge_payload(dodger.id(), dodger.gen().id() as u32);
        dodge(&mut w, dodger, DODGE_BUFF, &payload)
    };
    assert_eq!(outcomes.len(), 1);
    assert!(matches!(outcomes[0], Outcome::Death { ent, .. } if ent == attack));
    assert_eq!(w.read_storage::<Projectile>().get(attack).unwrap().time_left, Fixed64::ZERO, "frozen before its Death settles");
    assert!(w.read_storage::<Projectile>().get(script_shot).unwrap().time_left > Fixed64::ZERO);
    assert!(w.read_storage::<Projectile>().get(elsewhere).unwrap().time_left > Fixed64::ZERO);
    request_dodge(&mut w, other);
    w.maintain();
    assert!(!w.entities().is_alive(elsewhere) && w.entities().is_alive(script_shot));
}

#[test]
fn widening_line_hits_by_the_radius_reached_at_each_distance() {
    // 10 → 310 over 1000: about 70 at x=200, 250 at x=800.
    for (end, expected) in [(None, (1000, 1000)), (Some(310), (1000, 900))] {
        let mut w = world();
        let caster = body(&mut w, 0, 0, 1);
        let early = body(&mut w, 200, 120, 2);
        let late = body(&mut w, 800, 200, 2);
        let spec = Spec { max_hits: 8, ..linear(caster, at(1, 0)) };
        match end {
            Some(end) => cast_payload(&mut w, caster, &spec, with_end_radius(spec.payload(), Fixed64::from_i32(end))),
            None => cast(&mut w, caster, &spec),
        }
        fly(&mut w, 60, 120, &[]);
        assert_eq!((hp(&w, early), hp(&w, late)), expected, "end {end:?}");
    }
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let target = body(&mut w, 300, 0, 2);
    assert_eq!(end_radius_of(&with_end_radius(homing(caster, target).payload(), Fixed64::ONE)), None, "linear only");
    assert_eq!(end_radius_of(&with_end_radius(linear(caster, at(1, 0)).payload(), Fixed64::from_i32(1_001))), None);
    // The radius is an exact lerp on the distance flown, capped at the end.
    let mut state = State { spec: linear(caster, at(1, 0)), owner: caster, faction: Faction::new(FactionType::Player, 1), team: 1,
        origin: super::super::moba_match::capture_projectile_origin(&w, caster).unwrap(), end: at(1000, 0), target: None,
        target_lost: false, age: Fixed64::ZERO, hits: BTreeSet::new(), heading: Vec2::ZERO, bounce: None, dodgeable: false,
        end_radius: Some(Fixed64::from_i32(310)), dot_interval: None, hit_stack: None, delay_left: Fixed64::ZERO, healing_reduction: None };
    assert_eq!(state.hit_radius(), Fixed64::from_i32(10));
    state.age = Fixed64::from_raw(512);
    assert_eq!(state.hit_radius(), Fixed64::from_i32(10 + 300 * 600 / 1000));
    state.age = Fixed64::from_i32(2);
    assert_eq!(state.hit_radius(), Fixed64::from_i32(310));
}

#[test]
fn on_hit_dot_interval_rides_the_periodic_payload_and_requires_a_dot() {
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let first = body(&mut w, 200, 0, 2);
    let spec = Spec { max_hits: 8, dot: Some((Fixed64::from_i32(50), Fixed64::from_i32(5))), kind: DamageKind::Magical,
        ..linear(caster, at(1, 0)) };
    assert_eq!(dot_interval_of(&with_dot_interval(linear(caster, at(1, 0)).payload(), Fixed64::ONE)), None, "needs a dot");
    cast_payload(&mut w, caster, &spec, with_dot_interval(spec.payload(), Fixed64::ONE));
    fly(&mut w, 60, 20, &[]);
    let handle = omb_script_abi::types::EntityHandle { id: caster.id(), gen: caster.gen().id() as u32 };
    let key = omb_script_abi::buff_ids::generic_dot_buff_id("fixture_bolt", handle);
    let row = w.read_resource::<crate::runtime::BuffStore>().get(first, &key).unwrap().payload.clone();
    let parsed = crate::runtime::ability_runtime::periodic_damage::PeriodicDamageSpec::from_payload(&row).unwrap();
    assert_eq!((parsed.interval, parsed.pulse_amount()), (Some(Fixed64::ONE), Some(Fixed64::from_i32(50))));
}

#[test]
fn hit_stack_applies_the_caster_rule_only_while_the_caster_has_it() {
    use super::super::hit_stack::{Rule, RULE_KEY};
    let rule = Rule { buff: "fixture_shred".into(), stat: omb_script_abi::stat_keys::StatKey::MagicalResistanceBonus,
        per_stack: Fixed64::from_i32(-5), max: 3, duration: Fixed64::from_i32(3), refresh_at_max: false, heroes_only: true };
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let hero = body(&mut w, 200, 0, 2);
    w.write_storage::<Hero>().insert(hero, Hero::default()).unwrap();
    let creep = body(&mut w, 300, 0, 2);
    let spec = Spec { max_hits: 8, ..linear(caster, at(1, 0)) };
    // Without the caster buff: no stack.
    cast_payload(&mut w, caster, &spec, with_hit_stack(spec.payload(), "fixture_rule"));
    fly(&mut w, 60, 60, &[]);
    assert!(!w.read_resource::<crate::runtime::BuffStore>().has(hero, "fixture_shred"));
    w.write_resource::<crate::runtime::BuffStore>().add(caster, "fixture_rule", Fixed64::from_i32(22),
        serde_json::json!({ (RULE_KEY): rule.value() }));
    for stacks in [1, 2, 3, 3] {
        cast_payload(&mut w, caster, &spec, with_hit_stack(spec.payload(), "fixture_rule"));
        fly(&mut w, 60, 30, &[]);
        let buffs = w.read_resource::<crate::runtime::BuffStore>();
        assert_eq!(buffs.sum_add(hero, omb_script_abi::stat_keys::StatKey::MagicalResistanceBonus), Fixed64::from_i32(-5 * stacks));
        assert!(!buffs.has(creep, "fixture_shred"), "heroes only");
    }
    assert!(hit_stack_of(&with_hit_stack(spec.payload(), "__envelope")).is_none());
}

#[test]
fn launch_delay_waits_at_the_launch_point_then_flies_the_same_line() {
    let hz = 60;
    let mut w = world();
    let caster = body(&mut w, 0, 0, 1);
    let target = body(&mut w, 300, 0, 2);
    let spec = Spec { max_hits: 2, ..linear(caster, at(1, 0)) };
    // Same shot twice: once now, once 0.1 s later, both from the launch point.
    cast(&mut w, caster, &spec);
    let delayed = with_delay(spec.payload(), Fixed64::from_raw(102));
    assert_eq!(delay_of(&delayed), Fixed64::from_raw(102));
    assert_eq!(Spec::from_payload(BUFF, spec.lifetime, &delayed), Some(spec.clone()), "delay is an optional row");
    w.write_resource::<Vec<Outcome>>().push(Outcome::AddBuff { target: caster, buff_id: BUFF.into(), duration: spec.lifetime, payload: delayed });
    settle(&mut w);
    // The caster moving away afterwards does not move the waiting shot.
    w.write_storage::<Pos>().get_mut(caster).unwrap().0 = at(0, 500);
    let mut damage_ticks = Vec::new();
    let mut elapsed = 0;
    for tick in 1..=hz as u64 * 2 {
        let next = tick as i64 * 1024 / hz;
        w.insert(DeltaTime(Fixed64::from_raw(next - elapsed)));
        elapsed = next;
        w.write_resource::<Tick>().0 = tick;
        let before = hp(&w, target);
        run_now::<FlightSys>(&mut w);
        settle(&mut w);
        if hp(&w, target) < before { damage_ticks.push(tick); }
    }
    assert_eq!(hp(&w, target), 800, "both shots hit");
    assert_eq!(damage_ticks.len(), 2);
    // 0.1 s at 60 Hz is 6 waiting ticks.
    assert_eq!(damage_ticks[1] - damage_ticks[0], 6, "{damage_ticks:?}");
    assert_eq!(live(&w), 0);
    // Out-of-range delays are refused at admission.
    for bad in [Fixed64::ZERO, MAX_LAUNCH_DELAY + Fixed64::from_raw(1)] {
        let payload = with_delay(spec.payload(), bad);
        assert!(Spec::from_payload(BUFF, spec.lifetime, &payload).is_none(), "{bad:?}");
    }
}
