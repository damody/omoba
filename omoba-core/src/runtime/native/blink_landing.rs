//! Legal blink landing (G-4 `point_walkable`). A blink ignores what lies between
//! the caster and its destination, but it must land where the caster could stand.
//! A script cannot test a point against terrain, so it sends this packet to its
//! own caster: `{KEY: {source, anchor, preferred, cast range}}`. Serial
//! settlement places the caster at the first walkable point, with the caster's
//! collision radius against public terrain (the hero movement test):
//! 1. `preferred` (the script's landing, e.g. beside the target);
//! 2. otherwise the same distance from `anchor`, rotating away from the preferred
//!    direction in 15 degree steps, alternating sides (nearest first);
//! 3. otherwise the caster stays where it is.
//! The caster then faces `anchor`. The result is a pure function of committed
//! positions and terrain, so every world that settles the packet lands the same
//! point. Never stored as a buff.
use omoba_sim::{Angle, Fixed64, Vec2};
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

use super::comp::{BlockedRegions, CProperty, CollisionRadius, Facing, Pos};

pub const KEY: &str = "__blink_landing_v1";
pub const BUFF: &str = "__blink_landing";
/// Rotation steps on each side (15 degrees each, up to 180).
const STEPS: i32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub source_id: u32,
    pub source_generation: u32,
    /// What the blink lands beside (and faces).
    pub anchor: Vec2,
    /// The script's chosen landing point.
    pub preferred: Vec2,
    /// Admission bound from the caster to the anchor.
    pub cast_range: Fixed64,
}

impl Spec {
    pub fn checked(self) -> Option<Self> {
        let bounded = omoba_sim::terrain::MAX_GEOMETRY_RAW;
        let inside = |p: Vec2| p.x.raw().abs() <= bounded && p.y.raw().abs() <= bounded;
        (self.source_id != u32::MAX && self.source_generation > 0 && self.source_generation <= i32::MAX as u32
            && inside(self.anchor) && inside(self.preferred)
            && (self.preferred - self.anchor).length_squared() <= Fixed64::from_i32(1_000) * Fixed64::from_i32(1_000)
            && self.cast_range > Fixed64::ZERO && self.cast_range <= Fixed64::from_i32(10_000)).then_some(self)
    }
    pub fn payload(&self) -> Value {
        json!({ (KEY): { "source_id": self.source_id, "source_generation": self.source_generation,
            "anchor": [self.anchor.x.raw(), self.anchor.y.raw()], "preferred": [self.preferred.x.raw(), self.preferred.y.raw()],
            "range_raw": self.cast_range.raw() } })
    }
    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Self> {
        if buff_id != BUFF || duration != Fixed64::ONE || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 5 { return None; }
        let point = |name: &str| -> Option<Vec2> {
            let xy = row.get(name)?.as_array()?;
            (xy.len() == 2).then_some(())?;
            Some(Vec2::new(Fixed64::from_raw(xy[0].as_i64()?), Fixed64::from_raw(xy[1].as_i64()?)))
        };
        Self { source_id: u32::try_from(row.get("source_id")?.as_u64()?).ok()?,
            source_generation: u32::try_from(row.get("source_generation")?.as_u64()?).ok()?,
            anchor: point("anchor")?, preferred: point("preferred")?,
            cast_range: Fixed64::from_raw(row.get("range_raw")?.as_i64()?) }.checked()
    }
}

/// The packet is valid only on its own live caster, with the anchor in range.
pub fn admitted(world: &World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value, expected_source: Option<Entity>) -> Option<Spec> {
    let spec = Spec::from_payload(buff_id, duration, payload)?;
    let entities = world.entities();
    let source = entities.entity(spec.source_id);
    if source != target || source.gen().id() as u32 != spec.source_generation || !entities.is_alive(source)
        || expected_source.is_some_and(|expected| expected != source) { return None; }
    if world.read_storage::<CProperty>().get(source).is_none_or(|p| p.hp <= Fixed64::ZERO) { return None; }
    let origin = world.read_storage::<Pos>().get(source)?.0;
    ((spec.anchor - origin).length_squared() <= spec.cast_range * spec.cast_range).then_some(spec)
}

/// Single-point walkability: a body of `radius` can stand at `point` (the hero movement terrain test,
/// a zero-length sweep against public blocked regions).
pub fn point_walkable(regions: &BlockedRegions, point: Vec2, radius: Fixed64) -> bool {
    !super::tick::hero_move_tick::path_hits_regions(point, point, radius.max(Fixed64::ZERO), regions)
}

fn rotate(v: Vec2, angle: Angle) -> Vec2 {
    let (sin, cos) = (omoba_sim::trig::sin(angle), omoba_sim::trig::cos(angle));
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

/// Where the caster lands: the first walkable candidate, else `None` (stay).
pub fn landing(world: &World, spec: &Spec) -> Option<Vec2> {
    let source = world.entities().entity(spec.source_id);
    let radius = world.read_storage::<CollisionRadius>().get(source).map_or(Fixed64::ZERO, |r| r.0.max(Fixed64::ZERO));
    let regions = world.try_fetch::<BlockedRegions>();
    let walkable = |p: Vec2| regions.as_ref().is_none_or(|regions| point_walkable(regions, p, radius));
    if walkable(spec.preferred) { return Some(spec.preferred); }
    let offset = spec.preferred - spec.anchor;
    if offset == Vec2::ZERO { return None; }
    for step in 1..=STEPS {
        for side in [1, -1] {
            let candidate = spec.anchor + rotate(offset, Angle::from_degrees_i32(side * step * 15));
            if walkable(candidate) { return Some(candidate); }
            if step == STEPS { break; }
        }
    }
    None
}

/// Settles the packet: move to the landing (or stay) and face the anchor.
pub fn apply(world: &mut World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) {
    let Some(spec) = admitted(world, target, buff_id, duration, payload, None) else { return; };
    let at = match landing(world, &spec) {
        Some(point) => {
            if let Some(pos) = world.write_storage::<Pos>().get_mut(target) { pos.0 = point; }
            point
        }
        None => match world.read_storage::<Pos>().get(target) { Some(pos) => pos.0, None => return },
    };
    let facing = spec.anchor - at;
    if facing != Vec2::ZERO {
        if let Some(face) = world.write_storage::<Facing>().get_mut(target) { face.0 = omoba_sim::trig::atan2(facing.y, facing.x); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::comp::BlockedRegion;
    use specs::Builder;
    type FloatVec2 = vek::Vec2<f32>;

    fn p(x: i32, y: i32) -> Vec2 { Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)) }
    fn world(walls: Vec<BlockedRegion>) -> (World, Entity) {
        let mut world = World::new();
        world.register::<Pos>(); world.register::<Facing>(); world.register::<CProperty>(); world.register::<CollisionRadius>();
        world.insert(BlockedRegions(walls));
        let caster = world.create_entity().with(Pos(p(0, 0))).with(Facing(Angle::ZERO))
            .with(CProperty { hp: Fixed64::from_i32(100), mhp: Fixed64::from_i32(100), msd: Fixed64::ZERO,
                def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO })
            .with(CollisionRadius(Fixed64::from_i32(30))).build();
        (world, caster)
    }
    fn square(min: (f32, f32), max: (f32, f32)) -> BlockedRegion {
        BlockedRegion { name: "wall".into(), points: vec![FloatVec2::new(min.0, min.1), FloatVec2::new(max.0, min.1),
            FloatVec2::new(max.0, max.1), FloatVec2::new(min.0, max.1)] }
    }
    fn spec(caster: Entity, anchor: Vec2, preferred: Vec2) -> Spec {
        Spec { source_id: caster.id(), source_generation: caster.gen().id() as u32, anchor, preferred,
            cast_range: Fixed64::from_i32(1_100) }.checked().unwrap()
    }
    fn settle(world: &mut World, caster: Entity, spec: Spec) -> Vec2 {
        apply(world, caster, BUFF, Fixed64::ONE, &spec.payload());
        world.read_storage::<Pos>().get(caster).unwrap().0
    }

    #[test]
    fn open_ground_lands_on_the_preferred_point_and_faces_the_anchor() {
        let (mut world, caster) = world(vec![]);
        let at = settle(&mut world, caster, spec(caster, p(1_000, 0), p(900, 0)));
        assert_eq!(at, p(900, 0));
        assert_eq!(world.read_storage::<Facing>().get(caster).unwrap().0, Angle::ZERO);
    }

    #[test]
    fn a_preferred_point_inside_a_wall_moves_to_the_nearest_walkable_side_of_the_anchor() {
        // The target hugs a wall that covers the caster's side of it.
        let (mut world, caster) = world(vec![square((850.0, -60.0), (960.0, 60.0))]);
        let landed = settle(&mut world, caster, spec(caster, p(1_000, 0), p(900, 0)));
        assert_ne!(landed, p(900, 0), "never inside the wall");
        assert!(point_walkable(&world.read_resource::<BlockedRegions>(), landed, Fixed64::from_i32(30)),
            "the landing is walkable for the caster's body");
        assert!(!point_walkable(&world.read_resource::<BlockedRegions>(), p(900, 0), Fixed64::from_i32(30)), "the preferred point is not");
        let gap = (landed - p(1_000, 0)).length();
        assert!(gap >= Fixed64::from_i32(99) && gap <= Fixed64::from_i32(101), "still beside the target");
        let facing = p(1_000, 0) - landed;
        assert_eq!(world.read_storage::<Facing>().get(caster).unwrap().0, omoba_sim::trig::atan2(facing.y, facing.x));
        // Same packet, same terrain: every world lands the same point.
        let (mut again, caster2) = world_with_same_ids();
        assert_eq!(settle(&mut again, caster2, spec(caster2, p(1_000, 0), p(900, 0))), landed);
    }
    fn world_with_same_ids() -> (World, Entity) { world(vec![square((850.0, -60.0), (960.0, 60.0))]) }

    #[test]
    fn a_fully_enclosed_anchor_keeps_the_caster_in_place() {
        let (mut world, caster) = world(vec![square((850.0, -150.0), (1_150.0, 150.0))]);
        assert_eq!(settle(&mut world, caster, spec(caster, p(1_000, 0), p(900, 0))), p(0, 0));
    }

    #[test]
    fn foreign_dead_or_out_of_range_packets_are_not_admitted() {
        let (mut world, caster) = world(vec![]);
        let other = world.create_entity().with(Pos(p(5, 5))).build();
        let packet = spec(caster, p(1_000, 0), p(900, 0)).payload();
        assert!(admitted(&world, other, BUFF, Fixed64::ONE, &packet, None).is_none(), "only on its own caster");
        assert!(admitted(&world, caster, BUFF, Fixed64::from_i32(2), &packet, None).is_none(), "instant packet");
        let far = Spec { cast_range: Fixed64::from_i32(500), ..spec(caster, p(1_000, 0), p(900, 0)) };
        assert!(admitted(&world, caster, BUFF, Fixed64::ONE, &far.payload(), None).is_none(), "anchor beyond range");
        world.write_storage::<CProperty>().get_mut(caster).unwrap().hp = Fixed64::ZERO;
        assert!(admitted(&world, caster, BUFF, Fixed64::ONE, &packet, None).is_none(), "dead caster");
        assert_eq!(settle(&mut world, caster, spec(caster, p(1_000, 0), p(900, 0))), p(0, 0));
    }
}
