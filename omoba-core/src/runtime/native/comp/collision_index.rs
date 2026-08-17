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
}
