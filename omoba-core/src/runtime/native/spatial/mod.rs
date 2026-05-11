//! Deterministic spatial index implementations shared by server and native clients.

pub mod bvh;
pub mod hash_grid;
pub mod quadtree;
pub mod sweep_and_prune;

use std::fmt::Debug;
use std::hash::Hash;

use specs::Entity;
use vek::Vec2;

pub use bvh::Bvh;
pub use hash_grid::SpatialHashGrid;
pub use quadtree::QuadTree;
pub use sweep_and_prune::SweepAndPrune;

#[derive(Debug, Clone)]
pub struct Entry<Id, Item> {
    pub id: Id,
    pub item: Item,
    pub position: Vec2<f32>,
    pub bounding_radius: f32,
}

impl<Id, Item> Entry<Id, Item> {
    pub fn new(id: Id, item: Item, position: Vec2<f32>, bounding_radius: f32) -> Self {
        Self {
            id,
            item,
            position,
            bounding_radius,
        }
    }

    pub fn point(id: Id, item: Item, position: Vec2<f32>) -> Self {
        Self {
            id,
            item,
            position,
            bounding_radius: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bounds {
    pub min: Vec2<f32>,
    pub max: Vec2<f32>,
}

impl Bounds {
    pub fn new(min: Vec2<f32>, max: Vec2<f32>) -> Self {
        Self { min, max }
    }

    pub fn contains_point(&self, point: Vec2<f32>) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
}

#[derive(Debug, Clone)]
pub struct SpatialIndexParams {
    pub quadtree_max_depth: usize,
    pub quadtree_max_per_node: usize,
    pub hash_grid_cell_size: f32,
    pub bvh_max_leaf: usize,
}

impl Default for SpatialIndexParams {
    fn default() -> Self {
        Self {
            quadtree_max_depth: 8,
            quadtree_max_per_node: 10,
            hash_grid_cell_size: 128.0,
            bvh_max_leaf: 4,
        }
    }
}

pub trait SpatialIndex<Id, Item>: Send + Sync
where
    Id: Clone + Eq + Hash + Ord + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    fn initialize(&mut self, bounds: Bounds, entries: Vec<Entry<Id, Item>>);
    fn insert(&mut self, entry: Entry<Id, Item>);
    fn remove(&mut self, id: &Id) -> bool;
    fn update(&mut self, entry: Entry<Id, Item>);
    fn query_in_range(&self, center: Vec2<f32>, radius: f32) -> Vec<Entry<Id, Item>>;

    fn query_with_distance(&self, center: Vec2<f32>, radius: f32) -> Vec<(Entry<Id, Item>, f32)> {
        let mut out: Vec<_> = self
            .query_in_range(center, radius)
            .into_iter()
            .map(|e| {
                let d = e.position.distance(center);
                (e, d)
            })
            .collect();
        out.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    fn bulk_replace(&mut self, bounds: Bounds, entries: Vec<Entry<Id, Item>>) {
        self.initialize(bounds, entries);
    }

    fn count_nodes(&self) -> usize;
    fn name(&self) -> &'static str;
}

pub fn build_spatial_index<Id, Item>(
    kind: &str,
    params: SpatialIndexParams,
) -> Box<dyn SpatialIndex<Id, Item>>
where
    Id: Clone + Eq + Hash + Ord + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    match kind {
        "quadtree" => {
            log::info!(
                "SpatialIndex initialized: quadtree (depth={}, per_node={})",
                params.quadtree_max_depth,
                params.quadtree_max_per_node
            );
            Box::new(QuadTree::new(
                params.quadtree_max_depth,
                params.quadtree_max_per_node,
            ))
        }
        "hash_grid" => {
            log::info!(
                "SpatialIndex initialized: hash_grid (cell_size={})",
                params.hash_grid_cell_size
            );
            Box::new(SpatialHashGrid::new(params.hash_grid_cell_size))
        }
        "bvh" => {
            log::info!(
                "SpatialIndex initialized: bvh (max_leaf={})",
                params.bvh_max_leaf
            );
            Box::new(Bvh::new(params.bvh_max_leaf))
        }
        "sap" => {
            log::info!("SpatialIndex initialized: sap");
            Box::new(SweepAndPrune::new())
        }
        other => {
            log::warn!(
                "Unknown SPATIAL_INDEX = {:?}, falling back to quadtree",
                other
            );
            Box::new(QuadTree::new(
                params.quadtree_max_depth,
                params.quadtree_max_per_node,
            ))
        }
    }
}

pub fn build_entity_index(
    kind: &str,
    params: SpatialIndexParams,
) -> Box<dyn SpatialIndex<Entity, ()>> {
    match kind {
        "quadtree" | "hash_grid" | "bvh" | "sap" => build_spatial_index(kind, params),
        other => {
            log::warn!(
                "Unknown SPATIAL_INDEX = {:?} for entity index, falling back to sap",
                other
            );
            build_spatial_index("sap", params)
        }
    }
}
