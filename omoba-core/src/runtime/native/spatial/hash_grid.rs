use std::collections::{BTreeSet, HashMap};
use std::hash::Hash;

use vek::Vec2;

use super::{Bounds, Entry, SpatialIndex};

type Cell = (i32, i32);

pub struct SpatialHashGrid<Id, Item> {
    cell_size: f32,
    cells: HashMap<Cell, Vec<Entry<Id, Item>>>,
    id_cells: HashMap<Id, Vec<Cell>>,
}

impl<Id, Item> SpatialHashGrid<Id, Item>
where
    Id: Clone + Eq + Hash + Ord + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    pub fn new(cell_size: f32) -> Self {
        let cell_size = if cell_size > 0.0 { cell_size } else { 128.0 };
        Self {
            cell_size,
            cells: HashMap::new(),
            id_cells: HashMap::new(),
        }
    }

    fn entry_aabb(entry: &Entry<Id, Item>) -> (Vec2<f32>, Vec2<f32>) {
        let pos = entry.position;
        let r = entry.bounding_radius.max(0.0);
        (
            Vec2::new(pos.x - r, pos.y - r),
            Vec2::new(pos.x + r, pos.y + r),
        )
    }

    fn world_to_cell(&self, p: Vec2<f32>) -> Cell {
        (
            (p.x / self.cell_size).floor() as i32,
            (p.y / self.cell_size).floor() as i32,
        )
    }

    fn cells_for_aabb(&self, min: Vec2<f32>, max: Vec2<f32>) -> Vec<Cell> {
        let (cx0, cy0) = self.world_to_cell(min);
        let (cx1, cy1) = self.world_to_cell(max);
        let mut out = Vec::with_capacity(((cx1 - cx0 + 1) * (cy1 - cy0 + 1)).max(1) as usize);
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                out.push((cx, cy));
            }
        }
        out
    }

    fn insert_internal(&mut self, entry: Entry<Id, Item>) {
        let (min, max) = Self::entry_aabb(&entry);
        let cells = self.cells_for_aabb(min, max);
        for cell in &cells {
            self.cells.entry(*cell).or_default().push(entry.clone());
        }
        self.id_cells.insert(entry.id.clone(), cells);
    }

    fn remove_internal(&mut self, id: &Id) -> bool {
        let cells = match self.id_cells.remove(id) {
            Some(cells) => cells,
            None => return false,
        };
        for cell in cells {
            if let Some(bucket) = self.cells.get_mut(&cell) {
                bucket.retain(|entry| entry.id != *id);
                if bucket.is_empty() {
                    self.cells.remove(&cell);
                }
            }
        }
        true
    }
}

impl<Id, Item> SpatialIndex<Id, Item> for SpatialHashGrid<Id, Item>
where
    Id: Clone + Eq + Hash + Ord + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    fn initialize(&mut self, _bounds: Bounds, entries: Vec<Entry<Id, Item>>) {
        self.cells.clear();
        self.id_cells.clear();
        for entry in entries {
            self.insert_internal(entry);
        }
    }

    fn insert(&mut self, entry: Entry<Id, Item>) {
        self.remove_internal(&entry.id);
        self.insert_internal(entry);
    }

    fn remove(&mut self, id: &Id) -> bool {
        self.remove_internal(id)
    }

    fn update(&mut self, entry: Entry<Id, Item>) {
        self.remove_internal(&entry.id);
        self.insert_internal(entry);
    }

    fn query_in_range(&self, center: Vec2<f32>, radius: f32) -> Vec<Entry<Id, Item>> {
        let qmin = Vec2::new(center.x - radius, center.y - radius);
        let qmax = Vec2::new(center.x + radius, center.y + radius);
        let cells = self.cells_for_aabb(qmin, qmax);

        let mut seen: BTreeSet<Id> = BTreeSet::new();
        let mut results = Vec::new();
        for cell in cells {
            let Some(bucket) = self.cells.get(&cell) else {
                continue;
            };
            for entry in bucket {
                if seen.contains(&entry.id) {
                    continue;
                }
                let extended = radius + entry.bounding_radius.max(0.0);
                if entry.position.distance(center) <= extended {
                    seen.insert(entry.id.clone());
                    results.push(entry.clone());
                }
            }
        }
        results
    }

    fn count_nodes(&self) -> usize {
        self.cells.len()
    }

    fn name(&self) -> &'static str {
        "hash_grid"
    }
}
