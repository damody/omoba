use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use specs::Entity;
use vek::Vec2;
use voracious_radix_sort::Radixable;

use crate::runtime::spatial::{
    build_entity_index, Bounds, Entry, SpatialIndex, SpatialIndexParams,
};

const DEFAULT_WORLD_MIN: f32 = -10000.0;
const DEFAULT_WORLD_MAX: f32 = 10000.0;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct DisIndex {
    pub e: Entity,
    pub dis: f32,
}

impl Eq for DisIndex {}

impl Ord for DisIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dis.partial_cmp(&other.dis).unwrap()
    }
}

impl PartialOrd for DisIndex {
    fn partial_cmp(&self, other: &DisIndex) -> Option<Ordering> {
        self.dis.partial_cmp(&other.dis)
    }
}

impl PartialEq for DisIndex {
    fn eq(&self, other: &Self) -> bool {
        self.dis == other.dis
    }
}

impl Radixable<f32> for DisIndex {
    type Key = f32;

    fn key(&self) -> Self::Key {
        self.dis
    }
}

pub struct CollisionIndex {
    index: Box<dyn SpatialIndex<Entity, ()>>,
    bounds: Bounds,
    dirty: bool,
    kind: &'static str,
    item_count: usize,
}

fn compare_hits(a: &DisIndex, b: &DisIndex) -> Ordering {
    a.dis
        .partial_cmp(&b.dis)
        .unwrap_or(Ordering::Equal)
        .then_with(|| a.e.id().cmp(&b.e.id()))
        .then_with(|| a.e.gen().id().cmp(&b.e.gen().id()))
}

impl CollisionIndex {
    pub fn new(kind: &str, params: SpatialIndexParams) -> Self {
        let bounds = Bounds::new(
            Vec2::new(DEFAULT_WORLD_MIN, DEFAULT_WORLD_MIN),
            Vec2::new(DEFAULT_WORLD_MAX, DEFAULT_WORLD_MAX),
        );
        let index = build_entity_index(kind, params);
        let kind_static = index.name();
        let mut idx = Self {
            index,
            bounds,
            dirty: false,
            kind: kind_static,
            item_count: 0,
        };
        idx.index.initialize(idx.bounds.clone(), Vec::new());
        idx
    }

    pub fn rebuild_from<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = (Entity, Vec2<f32>)>,
    {
        let entries: Vec<Entry<Entity, ()>> = items
            .into_iter()
            .map(|(entity, pos)| Entry::point(entity, (), pos))
            .collect();
        self.item_count = entries.len();
        self.index.bulk_replace(self.bounds.clone(), entries);
        self.dirty = false;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn count(&self) -> usize {
        self.item_count
    }

    pub fn kind(&self) -> &'static str {
        self.kind
    }

    pub fn search_nn(&self, pos: Vec2<f32>, radius: f32, n: usize) -> Vec<DisIndex> {
        let r2 = radius * radius;
        let mut out = Vec::with_capacity(n);
        for entry in self.index.query_in_range(pos, radius) {
            let d2 = entry.position.distance_squared(pos);
            if d2 < r2 {
                let hit = DisIndex {
                    e: entry.id,
                    dis: d2,
                };
                if out.len() < n {
                    out.push(hit);
                } else if let Some((max_i, max)) = out
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| compare_hits(a, b))
                {
                    if compare_hits(&hit, max) == Ordering::Less {
                        out[max_i] = hit;
                    }
                }
            }
        }
        out.sort_by(compare_hits);
        out.truncate(n);
        out
    }

    /// Select one admitted spatial candidate by a caller-defined key. Unlike
    /// midpoint-nearest truncation, every result of the same range query is
    /// considered. Only one best result is retained after the index query.
    pub fn select_in_range_by_key<K: Ord>(
        &self,pos:Vec2<f32>,radius:f32,mut key:impl FnMut(Entity)->Option<K>,
    )->Option<(Entity,K)> {
        if !pos.x.is_finite()||!pos.y.is_finite()||!radius.is_finite()||radius<0.0{return None;}
        let mut best:Option<(Entity,K)>=None;
        for entry in self.index.query_in_range(pos,radius) {
            let Some(value)=key(entry.id) else {continue;};
            let replace=best.as_ref().is_none_or(|(entity,old)|value.cmp(old)
                .then_with(||entry.id.id().cmp(&entity.id()))
                .then_with(||entry.id.gen().id().cmp(&entity.gen().id()))==Ordering::Less);
            if replace {best=Some((entry.id,value));}
        }
        best
    }

    pub fn search_nn_bounded(
        &self,
        pos: Vec2<f32>,
        radius: f32,
        visit_budget: usize,
    ) -> Vec<DisIndex> {
        let r2 = radius * radius;
        let mut out: Vec<_> = self
            .index
            .query_in_range_bounded(pos, radius, visit_budget)
            .entries
            .into_iter()
            .filter_map(|entry| {
                let dis = entry.position.distance_squared(pos);
                (dis < r2).then_some(DisIndex { e: entry.id, dis })
            })
            .collect();
        out.sort_by(compare_hits);
        out
    }

    pub fn search_nn_two_radii(
        &self,
        pos: Vec2<f32>,
        r_inner: f32,
        r_outer: f32,
        n: usize,
    ) -> (Vec<DisIndex>, Vec<DisIndex>) {
        let r2_inner = r_inner * r_inner;
        let r2_outer = r_outer * r_outer;
        let mut inner = Vec::with_capacity(n);
        let mut outer = Vec::new();
        for entry in self.index.query_in_range(pos, r_outer) {
            let d2 = entry.position.distance_squared(pos);
            if d2 < r2_inner {
                let hit = DisIndex {
                    e: entry.id,
                    dis: d2,
                };
                if inner.len() < n {
                    inner.push(hit);
                } else if let Some((max_i, max)) = inner
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.dis.partial_cmp(&b.dis).unwrap_or(Ordering::Equal))
                {
                    if hit.dis < max.dis {
                        inner[max_i] = hit;
                    }
                }
            } else if d2 < r2_outer {
                outer.push(DisIndex {
                    e: entry.id,
                    dis: d2,
                });
            }
        }
        inner.sort_by(|a, b| {
            a.dis
                .partial_cmp(&b.dis)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        inner.truncate(n);
        (inner, outer)
    }

    /// Pad so Fixed64→f32 rounding of the center, the stored point, and the
    /// radius cannot drop a point that is inside the exact raw disk.
    ///
    /// The pad is 16 ULPs of the local magnitude plus two fixed units
    /// (`2 / SCALE`). It is not an attack-range extension: callers still accept
    /// or reject with raw `Fixed64` positions. `SCALE` is 1024.
    pub fn conservative_query_radius(center: Vec2<f32>, radius: f32) -> f32 {
        let magnitude = center.x.abs().max(center.y.abs()).max(radius.abs()).max(1.0);
        let scale_pad = magnitude * (16.0 * f32::EPSILON);
        let unit_pad = 2.0 / (omoba_sim::fixed::SCALE as f32);
        radius + scale_pad + unit_pad
    }

    /// Every index identity inside a conservative envelope of `radius`.
    ///
    /// Uses [`SpatialIndex::query_in_range`]. Does not truncate to ten and does
    /// not return stored float distances. A non-finite center or a negative
    /// radius returns nothing. A positive non-finite radius queries a large
    /// finite envelope so a huge fixed range is not dropped at the f32 cast.
    pub fn entities_in_radius(&self, pos: Vec2<f32>, radius: f32) -> Vec<Entity> {
        if !pos.x.is_finite() || !pos.y.is_finite() || !(radius >= 0.0) {
            return Vec::new();
        }
        let query_radius = if radius.is_finite() {
            Self::conservative_query_radius(pos, radius)
        } else {
            f32::MAX / 4.0
        };
        let mut entities: Vec<Entity> = self
            .index
            .query_in_range(pos, query_radius)
            .into_iter()
            .map(|entry| entry.id)
            .collect();
        entities.sort_by_key(|entity| (entity.id(), entity.gen().id()));
        entities.dedup();
        entities
    }
}

pub struct Searcher {
    pub tower: CollisionIndex,
    pub creep: CollisionIndex,
    pub hero: CollisionIndex,
    pub region: CollisionIndex,
}

impl Searcher {
    pub fn from_index_kinds(
        tower_kind: &str,
        creep_kind: &str,
        hero_kind: &str,
        region_kind: &str,
        params: SpatialIndexParams,
    ) -> Self {
        let searcher = Self {
            tower: CollisionIndex::new(tower_kind, params.clone()),
            creep: CollisionIndex::new(creep_kind, params.clone()),
            hero: CollisionIndex::new(hero_kind, params.clone()),
            region: CollisionIndex::new(region_kind, params),
        };
        log::info!(
            "Searcher initialized: tower={}, creep={}, hero={}, region={}",
            searcher.tower.kind(),
            searcher.creep.kind(),
            searcher.hero.kind(),
            searcher.region.kind()
        );
        searcher
    }

    pub fn search_collidable(&self, pos: Vec2<f32>, radius: f32, n: usize) -> Vec<DisIndex> {
        let mut out = Vec::with_capacity(n * 4);
        out.extend(self.hero.search_nn(pos, radius, n));
        out.extend(self.creep.search_nn(pos, radius, n));
        out.extend(self.tower.search_nn(pos, radius, n));
        out.extend(self.region.search_nn(pos, radius, n));
        out
    }
}

impl Default for Searcher {
    fn default() -> Self {
        // Creep/unit positions rebuild every tick and receive many tower script
        // radius queries in TD stress. hash_grid keeps query cost local while
        // retaining deterministic insertion/query order from sorted rebuilds.
        Self::from_index_kinds(
            "sap",
            "hash_grid",
            "sap",
            "sap",
            SpatialIndexParams::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::world::Builder;
    use specs::{World, WorldExt};

    #[test]
    fn projectile_contact_selector_uses_all_candidates_and_stable_ties_on_every_index() {
        let mut world=World::new();let entities:Vec<_>=(0..40).map(|_|world.create_entity().build()).collect();
        let rows:Vec<_>=entities.iter().enumerate().map(|(i,e)|(*e,Vec2::new(i as f32,0.))).collect();
        for kind in ["hash_grid","sap","quadtree","bvh"] {
            let mut index=CollisionIndex::new(kind,SpatialIndexParams::default());
            for reverse in [false,true] {
                let mut ordered=rows.clone();if reverse {ordered.reverse();}index.rebuild_from(ordered);
                assert_eq!(index.select_in_range_by_key(Vec2::zero(),100.,|e|Some(if e==entities[39]{0_u64}else{1})).unwrap().0,entities[39]);
                assert_eq!(index.select_in_range_by_key(Vec2::zero(),100.,|_|Some(0_u64)).unwrap().0,entities[0]);
                assert!(index.select_in_range_by_key(Vec2::zero(),100.,|_|None::<u64>).is_none());
                assert!(index.select_in_range_by_key(Vec2::zero(),f32::NAN,|_|Some(0_u64)).is_none());
            }
        }
    }

    #[test]
    fn collision_index_count_tracks_entities_not_spatial_nodes() {
        let mut world = World::new();
        let a = world.create_entity().build();
        let b = world.create_entity().build();
        let mut index = CollisionIndex::new("hash_grid", SpatialIndexParams::default());

        index.rebuild_from([(a, Vec2::new(10.0, 0.0)), (b, Vec2::new(90.0, 0.0))]);

        assert_eq!(index.count(), 2);
        assert_eq!(
            index
                .search_nn(Vec2::new(0.0, 0.0), 100.0, index.count())
                .len(),
            2
        );
    }

    fn f32_of(raw: i64) -> f32 {
        omoba_sim::Fixed64::from_raw(raw).to_f32_for_render()
    }

    #[test]
    fn local_radius_keeps_fixed_boundary_and_does_not_truncate_to_ten() {
        // Just inside the ±10000 index bounds. At this magnitude every raw
        // step is an exact f32, so the boundary sits on radius^2 rather than
        // outside it. The strict `<` nearest-neighbour path still drops it.
        let source_raw = 10_000_001i64;
        let range_raw = 3i64;
        let boundary_raw = source_raw + range_raw;
        let center = Vec2::new(f32_of(source_raw), 0.0);
        let radius = f32_of(range_raw);
        let beyond = Vec2::new(f32_of(boundary_raw), 0.0);
        // Inside the ±10000 index, raw/1024 values in this binade are exact, so
        // the unpadded squared distance equals radius^2. search_nn uses a strict
        // `<` and drops that boundary. The padded local query keeps it, and it
        // also keeps the next raw unit. Exact eligibility is the caller's job.
        let next_raw = boundary_raw + 1;
        let next_point = Vec2::new(f32_of(next_raw), 0.0);
        let boundary_d2 = beyond.distance_squared(center);
        assert_eq!(boundary_d2, radius * radius);
        assert!(next_point.distance_squared(center) > radius * radius);

        for kind in ["hash_grid", "sap", "quadtree", "bvh"] {
            let mut world = World::new();
            let boundary = world.create_entity().build();
            let next = world.create_entity().build();
            let mut extras = Vec::new();
            let mut rows = vec![
                (boundary, Vec2::new(f32_of(boundary_raw), 0.0)),
                (next, next_point),
            ];
            for index in 0..12 {
                let entity = world.create_entity().build();
                extras.push(entity);
                rows.push((
                    entity,
                    Vec2::new(f32_of(source_raw) + (index as f32 + 1.0), 0.0),
                ));
            }
            let reversed: Vec<_> = rows.iter().rev().copied().collect();
            for insertion in [rows.clone(), reversed] {
                let mut index = CollisionIndex::new(kind, SpatialIndexParams::default());
                index.rebuild_from(insertion);
                let found = index.entities_in_radius(center, radius);
                assert!(
                    found.contains(&boundary),
                    "{kind} dropped the exact fixed boundary"
                );
                assert!(
                    found.contains(&next),
                    "{kind} pad excluded the next raw unit; exact checks must reject it"
                );
                assert!(
                    extras.iter().all(|entity| !found.contains(entity)),
                    "{kind} tiny query was not local"
                );
                assert!(index
                    .search_nn(center, radius, 10)
                    .iter()
                    .all(|hit| hit.e != boundary && hit.e != next));
                let wide = index.entities_in_radius(center, 50.0);
                assert_eq!(wide.len(), 14, "{kind} truncated or dropped a local entity");
                assert!(wide.contains(&boundary) && wide.contains(&next));
                assert!(extras.iter().all(|entity| wide.contains(entity)));
                assert_eq!(index.search_nn(center, 50.0, 10).len(), 10);
            }
        }
    }
}
