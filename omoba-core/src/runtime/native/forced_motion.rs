//! Forced displacement (G-4 `knockback` / `pull_hook`). A script moves an
//! enemy by a timed buff on the victim whose payload is `{KEY: ..}`:
//! - `knockback` pushes it `distance` straight away from the source;
//! - `pull` drags it straight toward the source, stopping at body contact;
//! - `dash` moves the source itself to a fixed endpoint (a continuous dash, H02):
//!   the packet is on its own caster, carries no stun (the motion buff alone roots
//!   it against commands) and turns it to face the travel direction.
//!
//! The direction and travel are fixed when the serial settlement applies the
//! buff, from the positions at that moment; the victim then moves at a constant
//! speed (travel ÷ duration) over the buff's duration. Terrain stops it at the
//! wall (other units never block it). The victim is stunned for the duration and
//! cannot move by command even if that stun is cleansed (`BuffStore::is_rooted`).
//! The motion ends early on death, on a wall, or when the buff is removed.
//!
//! Only the authority integrates motion (`step`, from the MobaMatch pre-step);
//! every world sees the result as ordinary `PreStepMovement` facts, so team
//! replicas never predict a hidden unit's path.
use omoba_sim::{Fixed64, Vec2};
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};
use std::collections::BTreeMap;

use super::comp::{BlockedRegions, CProperty, CollisionRadius, DeltaTime, IsBuilding, Pos, Tick};
use super::ability_runtime::BuffStore;

pub const KEY: &str = "__forced_motion_v1";
/// The visible status buff; its payload is the spec.
pub const BUFF: &str = "forced_motion";
pub const MAX_DISTANCE: i32 = 3_000;
pub const MAX_DURATION: i32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode { Knockback, Pull, Dash }

impl Mode {
    pub const fn as_str(self) -> &'static str { match self { Self::Knockback => "knockback", Self::Pull => "pull", Self::Dash => "dash" } }
    pub fn parse(text: &str) -> Option<Self> {
        match text { "knockback" => Some(Self::Knockback), "pull" => Some(Self::Pull), "dash" => Some(Self::Dash), _ => None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub source_id: u32,
    pub source_generation: u32,
    pub mode: Mode,
    /// Knockback travel, or the pull's longest travel.
    pub distance: Fixed64,
    /// Admission range from the live source to the victim (a dash: to its endpoint).
    pub cast_range: Fixed64,
    /// A dash's endpoint (`Some` exactly for `Mode::Dash`).
    pub to: Option<Vec2>,
}

impl Spec {
    pub fn checked(self) -> Option<Self> {
        (self.source_id != u32::MAX && self.source_generation > 0 && self.source_generation <= i32::MAX as u32
            && self.distance > Fixed64::ZERO && self.distance <= Fixed64::from_i32(MAX_DISTANCE)
            && self.cast_range > Fixed64::ZERO && self.cast_range <= Fixed64::from_i32(10_000)
            && (self.mode == Mode::Dash) == self.to.is_some()
            && self.to.is_none_or(|to| to.x.raw().abs() <= omoba_sim::terrain::MAX_GEOMETRY_RAW
                && to.y.raw().abs() <= omoba_sim::terrain::MAX_GEOMETRY_RAW)).then_some(self)
    }
    pub fn payload(&self) -> Value {
        let mut row = json!({ "source_id": self.source_id, "source_generation": self.source_generation,
            "mode": self.mode.as_str(), "distance_raw": self.distance.raw(), "range_raw": self.cast_range.raw() });
        if let Some(to) = self.to { row["to"] = json!([to.x.raw(), to.y.raw()]); }
        json!({ (KEY): row })
    }
    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Self> {
        if buff_id != BUFF || !valid_duration(duration) || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        let to = match row.get("to") {
            Some(point) => {
                let xy = point.as_array()?;
                if xy.len() != 2 { return None; }
                Some(Vec2::new(Fixed64::from_raw(xy[0].as_i64()?), Fixed64::from_raw(xy[1].as_i64()?)))
            }
            None => None,
        };
        if row.len() != 5 + usize::from(to.is_some()) { return None; }
        Self { source_id: u32::try_from(row.get("source_id")?.as_u64()?).ok()?,
            source_generation: u32::try_from(row.get("source_generation")?.as_u64()?).ok()?,
            mode: Mode::parse(row.get("mode")?.as_str()?)?,
            distance: Fixed64::from_raw(row.get("distance_raw")?.as_i64()?),
            cast_range: Fixed64::from_raw(row.get("range_raw")?.as_i64()?), to }.checked()
    }
    pub fn source(&self) -> Entity { self.source_entity() }
    fn source_entity(&self) -> Entity { Entity::new(self.source_id, specs::world::Generation::new(self.source_generation as i32)) }
}

pub fn valid_duration(duration: Fixed64) -> bool {
    duration >= Fixed64::from_raw(1) && duration <= Fixed64::from_i32(MAX_DURATION)
}

/// One victim's motion, fixed at settlement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Motion {
    pub source: Entity,
    pub mode: Mode,
    /// Unit direction (Fixed64 components, length ≈ 1).
    pub direction: Vec2,
    pub remaining: Fixed64,
    pub time_left: Fixed64,
    /// A dash's exact endpoint: its last step lands here, free of direction rounding.
    pub to: Option<Vec2>,
}

#[derive(Clone, Debug, Default)]
pub struct ForcedMotions(pub BTreeMap<Entity, Motion>);

impl ForcedMotions {
    /// Canonical authority state for the lockstep desync hash.
    pub fn hash_value(&self) -> Value {
        Value::Array(self.0.iter().map(|(entity, motion)| json!({
            "entity": crate::runtime::canonical_entity_id(*entity),
            "source": crate::runtime::canonical_entity_id(motion.source), "mode": motion.mode.as_str(),
            "direction": [motion.direction.x.raw(), motion.direction.y.raw()],
            "remaining": motion.remaining.raw(), "time_left": motion.time_left.raw(),
            "to": motion.to.map(|to| [to.x.raw(), to.y.raw()]),
        })).collect())
    }
}

fn movable(world: &World, entity: Entity) -> bool {
    world.entities().is_alive(entity)
        && world.read_storage::<CProperty>().get(entity).is_some_and(|p| p.hp > Fixed64::ZERO)
        && world.read_storage::<IsBuilding>().get(entity).is_none()
        && world.read_storage::<Pos>().get(entity).is_some()
}

/// Live hostile target in range of the exact live source, not a building; a dash
/// is on its own live source with its endpoint in range.
pub fn admitted(world: &World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value,
    expected_source: Option<Entity>) -> Option<Spec> {
    let spec = Spec::from_payload(buff_id, duration, payload)?;
    if let Some(to) = spec.to {
        let source = spec.source_entity();
        let at = world.read_storage::<Pos>().get(target).map(|pos| pos.0)?;
        return (target == source && world.entities().entity(spec.source_id) == source
            && expected_source.is_none_or(|expected| expected == source) && movable(world, target)
            && (to - at).length_squared() <= spec.cast_range * spec.cast_range).then_some(spec);
    }
    (super::targeted_effect::admitted(world, target, spec.source_id, spec.source_generation, spec.cast_range, false, expected_source)
        && movable(world, target)).then_some(spec)
}

fn radius(world: &World, entity: Entity) -> Fixed64 {
    world.read_storage::<CollisionRadius>().get(entity).map_or(Fixed64::ZERO, |r| r.0.max(Fixed64::ZERO))
}

/// (unit direction, travel) for `spec` from the current positions, or None
/// when there is nowhere to go (coincident bodies, a pull already in contact).
pub fn plan(world: &World, target: Entity, spec: &Spec) -> Option<(Vec2, Fixed64)> {
    let positions = world.read_storage::<Pos>();
    let (at, from) = (positions.get(target)?.0, positions.get(spec.source_entity())?.0);
    let delta = match (spec.mode, spec.to) {
        (Mode::Dash, Some(to)) => to - at,
        (Mode::Knockback, _) => at - from,
        _ => from - at,
    };
    let length = delta.length();
    if length <= Fixed64::ZERO { return None; }
    let direction = Vec2::new(delta.x / length, delta.y / length);
    let travel = match spec.mode {
        Mode::Knockback => spec.distance,
        Mode::Pull => spec.distance.min(length - radius(world, target) - radius(world, spec.source_entity())),
        Mode::Dash => spec.distance.min(length),
    };
    (travel > Fixed64::ZERO).then_some((direction, travel))
}

/// Serial settlement of the buff: starts (or replaces) the victim's motion and
/// stores the status buff plus a stun of the same duration. False drops it.
pub fn apply(world: &mut World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> bool {
    let Some(spec) = admitted(world, target, buff_id, duration, payload, None) else { return false; };
    let Some((direction, remaining)) = plan(world, target, &spec) else { return false; };
    if world.try_fetch::<ForcedMotions>().is_none() { world.insert(ForcedMotions::default()); }
    {
        let buffs = world.read_resource::<BuffStore>();
        // A world that never steps (a team replica) keeps no stale rows.
        world.write_resource::<ForcedMotions>().0.retain(|entity, _| buffs.has(*entity, BUFF));
    }
    world.write_resource::<ForcedMotions>().0.insert(target, Motion {
        source: spec.source_entity(), mode: spec.mode, direction, remaining, time_left: duration,
        to: spec.to.filter(|_| spec.mode == Mode::Dash) });
    if spec.mode == Mode::Dash {
        if let Some(face) = world.write_storage::<super::comp::Facing>().get_mut(target) {
            face.0 = omoba_sim::trig::atan2(direction.y, direction.x);
        }
    }
    let mut buffs = world.write_resource::<BuffStore>();
    buffs.add(target, BUFF, duration, payload.clone());
    // A dash is the caster's own motion: rooted by the motion buff, not stunned.
    if spec.mode != Mode::Dash { buffs.add(target, omb_script_abi::buff_ids::BuffId::Stun.as_str(), duration, Value::Null); }
    true
}

/// First blocked point of `from -> to` for a body of `radius`; Q16 bisection.
fn wall_contact(from: Vec2, to: Vec2, radius: Fixed64, regions: &BlockedRegions) -> Option<Vec2> {
    let blocked = |end: Vec2| super::tick::hero_move_tick::path_hits_regions(from, end, radius, regions);
    if !blocked(to) { return None; }
    let at = |t: i64| Vec2::new(
        Fixed64::from_raw(from.x.raw() + (i128::from(to.x.raw() - from.x.raw()) * i128::from(t) >> 16) as i64),
        Fixed64::from_raw(from.y.raw() + (i128::from(to.y.raw() - from.y.raw()) * i128::from(t) >> 16) as i64));
    let (mut low, mut high) = (0_i64, 1_i64 << 16);
    // Largest unblocked prefix.
    while low < high { let mid = low + (high - low + 1) / 2; if blocked(at(mid)) { high = mid - 1; } else { low = mid; } }
    Some(at(low))
}

/// Advances every motion by this tick's `DeltaTime`, in canonical entity order,
/// and publishes each new position as a `PreStepMovement` fact (facing kept).
pub fn step(world: &mut World) {
    let dt = world.read_resource::<DeltaTime>().0;
    let Some(mut rows) = world.try_fetch_mut::<ForcedMotions>().map(|mut rows| std::mem::take(&mut rows.0)) else { return; };
    if dt <= Fixed64::ZERO { world.write_resource::<ForcedMotions>().0 = rows; return; }
    let mut moved = Vec::new();
    rows.retain(|entity, motion| {
        if !movable(world, *entity) || !world.read_resource::<BuffStore>().has(*entity, BUFF) { return false; }
        let from = world.read_storage::<Pos>().get(*entity).expect("movable position").0;
        let last = motion.time_left <= dt;
        let travel = if last { motion.remaining } else { motion.remaining.min(motion.remaining * dt / motion.time_left) };
        let to = match motion.to {
            Some(end) if last || travel >= motion.remaining => end,
            _ => from + Vec2::new(motion.direction.x * travel, motion.direction.y * travel),
        };
        let wall = world.try_fetch::<BlockedRegions>().and_then(|regions| wall_contact(from, to, radius(world, *entity), &regions));
        let end = wall.unwrap_or(to);
        if end != from {
            world.write_storage::<Pos>().get_mut(*entity).expect("movable position").0 = end;
            moved.push((*entity, end));
        }
        motion.remaining = motion.remaining - travel;
        motion.time_left = motion.time_left - dt;
        wall.is_none() && !last && motion.remaining > Fixed64::ZERO
    });
    world.write_resource::<ForcedMotions>().0 = rows;
    let tick = world.read_resource::<Tick>().0;
    for (entity, pos) in moved {
        let source = crate::runtime::canonical_entity_id(entity);
        world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(crate::runtime::OrderedFact {
            key: crate::runtime::FactOrderingKey { tick, phase: crate::runtime::FactPhase::PreStep,
                canonical_source_order: source, local_ordinal: 1, fact_kind: crate::runtime::FactKind::PreStepMovement },
            audience: crate::runtime::FactAudience::VisibilityPolicy(
                omb_script_abi::types::projection_policy_ids::MOVEMENT.to_owned()),
            fact: crate::runtime::ObservableFact::Movement { source, x_mm: pos.x.raw(), y_mm: pos.y.raw(), facing_ticks: None },
        }).expect("valid forced movement fact");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn p(x: i32, y: i32) -> Vec2 { Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)) }

    #[test]
    fn a_dash_moves_its_own_caster_continuously_rooted_not_stunned_and_stops_at_a_wall() {
        for wall in [false, true] {
            let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
            world.insert(crate::runtime::ObservableFactBuffer::default());
            world.insert(DeltaTime(Fixed64::from_raw(256)));
            if wall {
                world.insert(BlockedRegions(vec![super::super::comp::BlockedRegion { name: "wall".into(), points: vec![
                    vek::Vec2::new(500.0, -200.0), vek::Vec2::new(600.0, -200.0), vek::Vec2::new(600.0, 200.0), vek::Vec2::new(500.0, 200.0)] }]));
            }
            let caster = world.create_entity().with(Pos(p(0, 0))).with(super::super::comp::Facing(omoba_sim::Angle::from_degrees_i32(90)))
                .with(CProperty { hp: Fixed64::from_i32(100), mhp: Fixed64::from_i32(100), msd: Fixed64::ZERO,
                    def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO })
                .with(CollisionRadius(Fixed64::from_i32(20))).build();
            let spec = Spec { source_id: caster.id(), source_generation: caster.gen().id() as u32, mode: Mode::Dash,
                distance: Fixed64::from_i32(1_000), cast_range: Fixed64::from_i32(1_001), to: Some(p(1_000, 0)) }.checked().unwrap();
            // One second for 1000 units: four 0.25 s steps.
            assert!(apply(&mut world, caster, BUFF, Fixed64::ONE, &spec.payload()), "wall={wall}");
            assert_eq!(world.read_storage::<Pos>().get(caster).unwrap().0, p(0, 0), "not an instant teleport");
            assert_eq!(world.read_storage::<super::super::comp::Facing>().get(caster).unwrap().0, omoba_sim::Angle::ZERO, "faces the dash");
            {
                let buffs = world.read_resource::<BuffStore>();
                assert!(buffs.is_rooted(caster) && !buffs.has(caster, omb_script_abi::buff_ids::BuffId::Stun.as_str()), "rooted, not stunned");
            }
            step(&mut world);
            assert_eq!(world.read_storage::<Pos>().get(caster).unwrap().0, p(250, 0), "a quarter of the way after a quarter second");
            for _ in 0..3 { step(&mut world); }
            let end = world.read_storage::<Pos>().get(caster).unwrap().0;
            if wall {
                assert!(end.x < Fixed64::from_i32(481) && end.x > Fixed64::from_i32(470), "stops at the wall: {end:?}");
            } else {
                assert_eq!(end, p(1_000, 0));
            }
            assert!(world.read_resource::<ForcedMotions>().0.is_empty(), "the motion ends");
        }
    }

    #[test]
    fn a_dash_packet_is_only_admitted_on_its_own_caster() {
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
        let body = |world: &mut World, at: Vec2| world.create_entity().with(Pos(at))
            .with(CProperty { hp: Fixed64::from_i32(100), mhp: Fixed64::from_i32(100), msd: Fixed64::ZERO,
                def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build();
        let (caster, other) = (body(&mut world, p(0, 0)), body(&mut world, p(10, 0)));
        let spec = Spec { source_id: caster.id(), source_generation: caster.gen().id() as u32, mode: Mode::Dash,
            distance: Fixed64::from_i32(500), cast_range: Fixed64::from_i32(501), to: Some(p(500, 0)) }.checked().unwrap();
        assert!(admitted(&world, caster, BUFF, Fixed64::ONE, &spec.payload(), Some(caster)).is_some());
        assert!(admitted(&world, other, BUFF, Fixed64::ONE, &spec.payload(), None).is_none(), "never moves another unit");
        let far = Spec { cast_range: Fixed64::from_i32(400), ..spec };
        assert!(admitted(&world, caster, BUFF, Fixed64::ONE, &far.payload(), None).is_none(), "endpoint beyond range");
    }
    fn spec(mode: Mode) -> Spec {
        Spec { source_id: 3, source_generation: 1, mode, distance: Fixed64::from_i32(400), cast_range: Fixed64::from_i32(600),
            to: (mode == Mode::Dash).then(|| Vec2::new(Fixed64::from_i32(400), Fixed64::ZERO)) }
    }
    #[test]
    fn payload_round_trips_strictly() {
        for mode in [Mode::Knockback, Mode::Pull, Mode::Dash] {
            let s = spec(mode);
            assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &s.payload()), Some(s));
        }
        assert_eq!(Spec { to: None, ..spec(Mode::Dash) }.checked(), None, "a dash names its endpoint");
        assert_eq!(Spec { to: spec(Mode::Dash).to, ..spec(Mode::Pull) }.checked(), None, "only a dash has an endpoint");
        let s = spec(Mode::Knockback);
        assert_eq!(Spec::from_payload("stun", Fixed64::ONE, &s.payload()), None, "only the status buff carries it");
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ZERO, &s.payload()), None);
        assert_eq!(Spec::from_payload(BUFF, Fixed64::from_i32(MAX_DURATION + 1), &s.payload()), None);
        let mut extra = s.payload(); extra[KEY]["bonus"] = 1.into();
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &extra), None);
        let mut mode = s.payload(); mode[KEY]["mode"] = "throw".into();
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &mode), None);
        for (field, value) in [("distance_raw", 0), ("distance_raw", Fixed64::from_i32(MAX_DISTANCE + 1).raw()), ("range_raw", 0)] {
            let mut bad = s.payload(); bad[KEY][field] = value.into();
            assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &bad), None, "{field}={value}");
        }
    }
}
