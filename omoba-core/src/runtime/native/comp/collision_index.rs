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
        self.index.count_nodes()
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
                out.push(DisIndex {
                    e: entry.id,
                    dis: d2,
                });
            }
        }
        out.sort_by(|a, b| {
            a.dis
                .partial_cmp(&b.dis)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out.truncate(n);
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
        let mut inner = Vec::new();
        let mut outer = Vec::new();
        for entry in self.index.query_in_range(pos, r_outer) {
            let d2 = entry.position.distance_squared(pos);
            if d2 < r2_inner {
                inner.push(DisIndex {
                    e: entry.id,
                    dis: d2,
                });
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
        Self::from_index_kinds("sap", "sap", "sap", "sap", SpatialIndexParams::default())
    }
}
