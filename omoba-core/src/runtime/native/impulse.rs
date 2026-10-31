//! v3 `impulse { who, direction, speed, decay, duration }` (E514, 原作 B25 本願寺顯如 Fiend's Grip
//! through the Physics library's AddPhysicsVelocity): a velocity added to a unit that decays every
//! physics frame, several adding up, while the unit still moves by itself. One physics frame is
//! 1/30 s: each frame the unit moves by the sum of its impulses' velocities (units per frame) and
//! each velocity is multiplied by its `decay`. An impulse ends when it is below [`MIN_SPEED`] or
//! its `duration` runs out. Walls stop the displacement; buildings and dead units take none.
//! One-shot AddBuff packet on the moved unit, never stored as a buff.
use std::collections::BTreeMap;

use omoba_sim::{Fixed64, Vec2};
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

use crate::comp::{BlockedRegions, CProperty, DeltaTime, IsBuilding, Pos};

pub const KEY: &str = "__impulse_v1";
pub const BUFF: &str = "__impulse";
/// Physics frames per second (原作 Physics library THINK 0.0333).
pub const FRAMES_PER_SECOND: i64 = 30;
/// Units per frame an impulse may start with.
pub const MAX_SPEED: i32 = 1_000;
/// Below this speed (units per frame) an impulse is spent.
pub const MIN_SPEED: Fixed64 = Fixed64::from_raw(10);
/// The longest `duration`.
pub const MAX_DURATION: i32 = 30;
/// Live impulses one unit may hold.
pub const MAX_PER_UNIT: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    /// Units per physics frame.
    pub velocity: Vec2,
    /// The share of the velocity each frame keeps, in [0, 1).
    pub decay: Fixed64,
    /// Seconds it lasts at most; None until it is spent.
    pub duration: Option<Fixed64>,
}

impl Spec {
    pub fn checked(self) -> Option<Self> {
        let max = Fixed64::from_i32(MAX_SPEED);
        (self.velocity.x.raw().abs() <= max.raw() && self.velocity.y.raw().abs() <= max.raw()
            && self.decay >= Fixed64::ZERO && self.decay < Fixed64::ONE
            && self.duration.is_none_or(|d| d > Fixed64::ZERO && d <= Fixed64::from_i32(MAX_DURATION))).then_some(self)
    }
    pub fn payload(self) -> Value {
        let mut row = json!({ "vx_raw": self.velocity.x.raw(), "vy_raw": self.velocity.y.raw(), "decay_raw": self.decay.raw() });
        if let Some(duration) = self.duration { row["duration_raw"] = duration.raw().into(); }
        json!({ (KEY): row })
    }
    pub fn from_payload(buff_id: &str, payload: &Value) -> Option<Self> {
        if buff_id != BUFF || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        let raw = |key: &str| row.get(key).and_then(Value::as_i64);
        let duration = match row.get("duration_raw") { None => None, Some(v) => Some(Fixed64::from_raw(v.as_i64()?)) };
        if row.len() != 3 + usize::from(duration.is_some()) { return None; }
        Self { velocity: Vec2::new(Fixed64::from_raw(raw("vx_raw")?), Fixed64::from_raw(raw("vy_raw")?)),
            decay: Fixed64::from_raw(raw("decay_raw")?), duration }.checked()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Row { velocity: Vec2, decay: Fixed64, left: Option<Fixed64> }

/// One unit's impulses and its physics clock (seconds since its first impulse, raw; frames done).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Unit { rows: Vec<Row>, elapsed_raw: i64, frames: i64 }

/// The live impulses, by unit.
#[derive(Clone, Debug, Default)]
pub struct Impulses(BTreeMap<Entity, Unit>);

impl Impulses {
    /// Replay digest rows (moba_match hash).
    pub fn hash_value(&self) -> Value {
        Value::Array(self.0.iter().map(|(entity, unit)| json!([entity.id(), entity.gen().id(), unit.elapsed_raw, unit.frames,
            unit.rows.iter().map(|row| json!([row.velocity.x.raw(), row.velocity.y.raw(), row.decay.raw(),
                row.left.map(Fixed64::raw)])).collect::<Vec<_>>()])).collect())
    }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
}

fn movable(world: &World, entity: Entity) -> bool {
    world.entities().is_alive(entity)
        && world.read_storage::<CProperty>().get(entity).is_some_and(|p| p.hp > Fixed64::ZERO)
        && world.read_storage::<IsBuilding>().get(entity).is_none()
        && world.read_storage::<Pos>().get(entity).is_some()
}

/// A well-formed packet on a live unit that can be moved.
pub fn admitted(world: &World, target: Entity, buff_id: &str, payload: &Value) -> Option<Spec> {
    let spec = Spec::from_payload(buff_id, payload)?;
    movable(world, target).then_some(spec)
}

/// Adds the impulse to the unit (a full unit keeps its impulses and takes no more).
pub fn apply(world: &mut World, target: Entity, buff_id: &str, payload: &Value) -> bool {
    let Some(spec) = admitted(world, target, buff_id, payload) else { return false };
    if world.try_fetch::<Impulses>().is_none() { world.insert(Impulses::default()); }
    let mut impulses = world.write_resource::<Impulses>();
    let unit = impulses.0.entry(target).or_default();
    if unit.rows.len() >= MAX_PER_UNIT { return false; }
    unit.rows.push(Row { velocity: spec.velocity, decay: spec.decay, left: spec.duration });
    true
}

/// One tick: the physics frames due since the last tick move each unit (moba_match, after the
/// forced motions).
pub fn step(world: &mut World) {
    let dt = world.read_resource::<DeltaTime>().0;
    let Some(mut units) = world.try_fetch_mut::<Impulses>().map(|mut rows| std::mem::take(&mut rows.0)) else { return };
    if dt <= Fixed64::ZERO || units.is_empty() { world.write_resource::<Impulses>().0 = units; return; }
    let mut moved = Vec::new();
    units.retain(|entity, unit| {
        if !movable(world, *entity) { return false; }
        unit.elapsed_raw = unit.elapsed_raw.saturating_add(dt.raw());
        let due = unit.elapsed_raw.saturating_mul(FRAMES_PER_SECOND) / 1024;
        let mut shift = Vec2::ZERO;
        for _ in unit.frames..due {
            for row in &mut unit.rows {
                shift = shift + row.velocity;
                row.velocity = Vec2::new(row.velocity.x * row.decay, row.velocity.y * row.decay);
            }
        }
        unit.frames = due;
        for row in &mut unit.rows { row.left = row.left.map(|left| left - dt); }
        unit.rows.retain(|row| row.left.is_none_or(|left| left > Fixed64::ZERO)
            && (row.velocity.x.raw().abs() >= MIN_SPEED.raw() || row.velocity.y.raw().abs() >= MIN_SPEED.raw()));
        if shift != Vec2::ZERO {
            let from = world.read_storage::<Pos>().get(*entity).expect("movable position").0;
            let to = from + shift;
            let radius = world.read_storage::<crate::comp::CollisionRadius>().get(*entity).map_or(Fixed64::ZERO, |r| r.0);
            let end = world.try_fetch::<BlockedRegions>().and_then(|regions| super::forced_motion::wall_contact(from, to, radius, &regions))
                .unwrap_or(to);
            if end != from {
                world.write_storage::<Pos>().get_mut(*entity).expect("movable position").0 = end;
                moved.push((*entity, end));
            }
        }
        !unit.rows.is_empty()
    });
    world.write_resource::<Impulses>().0 = units;
    super::forced_motion::publish(world, moved);
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn spec(speed: i32, decay_pct: i64, duration: Option<i32>) -> Spec {
        Spec { velocity: Vec2::new(Fixed64::from_i32(speed), Fixed64::ZERO), decay: Fixed64::from_raw(decay_pct * 1024 / 100),
            duration: duration.map(Fixed64::from_i32) }
    }

    fn world() -> (World, Entity, Entity) {
        let mut world = World::new();
        world.register::<Pos>(); world.register::<CProperty>(); world.register::<IsBuilding>();
        world.register::<crate::comp::CollisionRadius>();
        world.insert(DeltaTime(Fixed64::from_raw(1024 / 60 + 1)));
        world.insert(crate::comp::Tick(1));
        world.insert(crate::runtime::ObservableFactBuffer::default());
        let life = CProperty { hp: Fixed64::ONE, mhp: Fixed64::ONE, msd: Fixed64::ZERO, def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO };
        let unit = world.create_entity().with(Pos(Vec2::ZERO)).with(life.clone()).build();
        let tower = world.create_entity().with(Pos(Vec2::ZERO)).with(life).with(IsBuilding).build();
        (world, unit, tower)
    }

    /// Packets round-trip; bad ones and buildings are refused.
    #[test]
    fn impulse_packets_round_trip_and_buildings_take_none() {
        let (world, unit, tower) = world();
        for s in [spec(30, 95, None), spec(30, 95, Some(2))] {
            assert_eq!(Spec::from_payload(BUFF, &s.payload()), Some(s));
            assert_eq!(admitted(&world, unit, BUFF, &s.payload()), Some(s));
        }
        assert_eq!(admitted(&world, tower, BUFF, &spec(30, 95, None).payload()), None, "a building");
        assert!(spec(30, 100, None).checked().is_none() && spec(MAX_SPEED + 1, 50, None).checked().is_none());
        assert_eq!(Spec::from_payload("other", &spec(30, 95, None).payload()), None);
    }

    /// One 30-per-frame impulse decaying 0.95 moves about 30 / (1 - 0.95) = 600 in all (raw decay 972 / 1024: about 590) (原作 dir*30 from
    /// several frames); two add up; the motion is spread over frames, 30 a second.
    #[test]
    fn impulses_decay_per_frame_and_add_up() {
        let (mut world, unit, _) = world();
        assert!(apply(&mut world, unit, BUFF, &spec(30, 95, None).payload()));
        let x = |world: &World| world.read_storage::<Pos>().get(unit).unwrap().0.x;
        for _ in 0..2 { step(&mut world); }
        assert_eq!(x(&world), Fixed64::from_i32(30), "one frame in two ticks at 60 Hz");
        for _ in 0..600 { step(&mut world); }
        let total = x(&world);
        assert!(total > Fixed64::from_i32(580) && total <= Fixed64::from_i32(600), "{total:?}");
        assert!(world.read_resource::<Impulses>().is_empty(), "spent");
        assert!(apply(&mut world, unit, BUFF, &spec(30, 0, None).payload()));
        assert!(apply(&mut world, unit, BUFF, &spec(-10, 0, None).payload()));
        for _ in 0..4 { step(&mut world); }
        assert_eq!(x(&world), total + Fixed64::from_i32(20), "30 - 10 in the first frame, then spent");
    }
}
