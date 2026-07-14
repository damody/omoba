use std::collections::{BTreeSet, HashMap};
use std::hash::Hash;

use vek::Vec2;

use super::{BoundedQueryResult, Bounds, Entry, SpatialIndex};

const NIL: u32 = u32::MAX;
const T_TRAVERSE: f32 = 1.0;
const T_INTERSECT: f32 = 2.0;

#[derive(Debug, Clone)]
struct Aabb {
    min: Vec2<f32>,
    max: Vec2<f32>,
}

impl Aabb {
    fn empty() -> Self {
        Self {
            min: Vec2::new(f32::INFINITY, f32::INFINITY),
            max: Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    fn from_entry<Id, Item>(entry: &Entry<Id, Item>) -> Self {
        let r = entry.bounding_radius.max(0.0);
        Self {
            min: Vec2::new(entry.position.x - r, entry.position.y - r),
            max: Vec2::new(entry.position.x + r, entry.position.y + r),
        }
    }

    fn union(&self, other: &Aabb) -> Aabb {
        Aabb {
            min: Vec2::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y)),
            max: Vec2::new(self.max.x.max(other.max.x), self.max.y.max(other.max.y)),
        }
    }

    fn perimeter(&self) -> f32 {
        let w = (self.max.x - self.min.x).max(0.0);
        let h = (self.max.y - self.min.y).max(0.0);
        2.0 * (w + h)
    }

    fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    fn from_query(center: Vec2<f32>, radius: f32) -> Self {
        Self {
            min: Vec2::new(center.x - radius, center.y - radius),
            max: Vec2::new(center.x + radius, center.y + radius),
        }
    }
}

#[derive(Debug, Clone)]
struct BvhNode<Id, Item> {
    bounds: Aabb,
    entries: Vec<Entry<Id, Item>>,
    left: u32,
    right: u32,
}

impl<Id, Item> BvhNode<Id, Item> {
    fn is_leaf(&self) -> bool {
        self.left == NIL && self.right == NIL
    }
}

pub struct Bvh<Id, Item> {
    nodes: Vec<BvhNode<Id, Item>>,
    id_index: HashMap<Id, (Item, Vec2<f32>, f32)>,
    bounds: Option<Bounds>,
    max_leaf: usize,
}

impl<Id, Item> Bvh<Id, Item>
where
    Id: Clone + Eq + Hash + Ord + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    pub fn new(max_leaf: usize) -> Self {
        Self {
            nodes: Vec::new(),
            id_index: HashMap::new(),
            bounds: None,
            max_leaf: max_leaf.max(1),
        }
    }

    fn rebuild(&mut self) {
        self.nodes.clear();
        if self.id_index.is_empty() {
            self.nodes.push(BvhNode {
                bounds: Aabb::empty(),
                entries: Vec::new(),
                left: NIL,
                right: NIL,
            });
            return;
        }

        let entries: Vec<Entry<Id, Item>> = self
            .id_index
            .iter()
            .map(|(id, (item, pos, r))| Entry {
                id: id.clone(),
                item: item.clone(),
                position: *pos,
                bounding_radius: *r,
            })
            .collect();

        self.nodes.push(BvhNode {
            bounds: Aabb::empty(),
            entries: Vec::new(),
            left: NIL,
            right: NIL,
        });
        let max_leaf = self.max_leaf;
        Self::build_recursive(&mut self.nodes, 0, entries, max_leaf);
    }

    fn build_recursive(
        nodes: &mut Vec<BvhNode<Id, Item>>,
        node_idx: usize,
        mut entries: Vec<Entry<Id, Item>>,
        max_leaf: usize,
    ) {
        let parent_aabb = entries
            .iter()
            .map(Aabb::from_entry)
            .fold(Aabb::empty(), |acc, aabb| acc.union(&aabb));
        nodes[node_idx].bounds = parent_aabb.clone();

        if entries.len() <= max_leaf {
            nodes[node_idx].entries = entries;
            return;
        }

        let parent_sa = parent_aabb.perimeter().max(1e-6);
        let leaf_cost = entries.len() as f32 * T_INTERSECT;
        let n = entries.len();
        let mut best_cost = f32::INFINITY;
        let mut best_axis = 0usize;
        let mut best_split = 1usize;

        for axis in 0..2 {
            entries.sort_by(|a, b| {
                let av = if axis == 0 {
                    a.position.x
                } else {
                    a.position.y
                };
                let bv = if axis == 0 {
                    b.position.x
                } else {
                    b.position.y
                };
                av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal)
            });

            let aabbs: Vec<Aabb> = entries.iter().map(Aabb::from_entry).collect();
            let mut prefix = Vec::with_capacity(n);
            let mut acc = Aabb::empty();
            for aabb in &aabbs {
                acc = acc.union(aabb);
                prefix.push(acc.clone());
            }
            let mut suffix = vec![Aabb::empty(); n];
            let mut acc = Aabb::empty();
            for i in (0..n).rev() {
                acc = acc.union(&aabbs[i]);
                suffix[i] = acc.clone();
            }

            for split in 1..n {
                let left_sa = prefix[split - 1].perimeter();
                let right_sa = suffix[split].perimeter();
                let cost = T_TRAVERSE
                    + (left_sa / parent_sa) * split as f32 * T_INTERSECT
                    + (right_sa / parent_sa) * (n - split) as f32 * T_INTERSECT;
                if cost < best_cost {
                    best_cost = cost;
                    best_axis = axis;
                    best_split = split;
                }
            }
        }

        if best_cost >= leaf_cost {
            nodes[node_idx].entries = entries;
            return;
        }

        entries.sort_by(|a, b| {
            let av = if best_axis == 0 {
                a.position.x
            } else {
                a.position.y
            };
            let bv = if best_axis == 0 {
                b.position.x
            } else {
                b.position.y
            };
            av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal)
        });

        let right_entries = entries.split_off(best_split);
        let left_entries = entries;
        let left_idx = nodes.len() as u32;
        nodes.push(BvhNode {
            bounds: Aabb::empty(),
            entries: Vec::new(),
            left: NIL,
            right: NIL,
        });
        let right_idx = nodes.len() as u32;
        nodes.push(BvhNode {
            bounds: Aabb::empty(),
            entries: Vec::new(),
            left: NIL,
            right: NIL,
        });
        nodes[node_idx].left = left_idx;
        nodes[node_idx].right = right_idx;

        Self::build_recursive(nodes, left_idx as usize, left_entries, max_leaf);
        Self::build_recursive(nodes, right_idx as usize, right_entries, max_leaf);
    }
}

impl<Id, Item> SpatialIndex<Id, Item> for Bvh<Id, Item>
where
    Id: Clone + Eq + Hash + Ord + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    fn initialize(&mut self, bounds: Bounds, entries: Vec<Entry<Id, Item>>) {
        self.id_index.clear();
        for entry in entries {
            self.id_index.insert(
                entry.id,
                (entry.item, entry.position, entry.bounding_radius),
            );
        }
        self.bounds = Some(bounds);
        self.rebuild();
    }

    fn insert(&mut self, entry: Entry<Id, Item>) {
        self.id_index.insert(
            entry.id,
            (entry.item, entry.position, entry.bounding_radius),
        );
        self.rebuild();
    }

    fn remove(&mut self, id: &Id) -> bool {
        if self.id_index.remove(id).is_some() {
            self.rebuild();
            true
        } else {
            false
        }
    }

    fn update(&mut self, entry: Entry<Id, Item>) {
        self.id_index.insert(
            entry.id,
            (entry.item, entry.position, entry.bounding_radius),
        );
        self.rebuild();
    }

    fn query_in_range(&self, center: Vec2<f32>, radius: f32) -> Vec<Entry<Id, Item>> {
        if self.nodes.is_empty() {
            return Vec::new();
        }

        let query = Aabb::from_query(center, radius);
        let mut results = Vec::new();
        let mut seen: BTreeSet<Id> = BTreeSet::new();
        let mut stack = vec![0];

        while let Some(idx) = stack.pop() {
            let node = &self.nodes[idx as usize];
            if !node.bounds.intersects(&query) {
                continue;
            }
            if node.is_leaf() {
                for entry in &node.entries {
                    if seen.contains(&entry.id) {
                        continue;
                    }
                    let extended = radius + entry.bounding_radius.max(0.0);
                    if entry.position.distance(center) <= extended {
                        seen.insert(entry.id.clone());
                        results.push(entry.clone());
                    }
                }
            } else {
                if node.left != NIL {
                    stack.push(node.left);
                }
                if node.right != NIL {
                    stack.push(node.right);
                }
            }
        }

        results
    }

    fn query_in_range_bounded(
        &self,
        center: Vec2<f32>,
        radius: f32,
        visit_budget: usize,
    ) -> BoundedQueryResult<Id, Item> {
        let mut entries = Vec::new();
        let mut visited_candidates = 0;
        if self.nodes.is_empty() || visit_budget == 0 {
            return BoundedQueryResult {
                entries,
                visited_candidates,
            };
        }

        let query = Aabb::from_query(center, radius);
        let mut seen = BTreeSet::new();
        let mut stack = vec![0];
        'traversal: while let Some(idx) = stack.pop() {
            let node = &self.nodes[idx as usize];
            if !node.bounds.intersects(&query) {
                continue;
            }
            if node.is_leaf() {
                for entry in &node.entries {
                    if seen.contains(&entry.id) {
                        continue;
                    }
                    if visited_candidates == visit_budget {
                        break 'traversal;
                    }
                    seen.insert(entry.id.clone());
                    visited_candidates += 1;
                    let extended = radius + entry.bounding_radius.max(0.0);
                    if entry.position.distance(center) <= extended {
                        entries.push(entry.clone());
                    }
                }
            } else {
                if node.left != NIL {
                    stack.push(node.left);
                }
                if node.right != NIL {
                    stack.push(node.right);
                }
            }
        }

        BoundedQueryResult {
            entries,
            visited_candidates,
        }
    }

    fn count_nodes(&self) -> usize {
        self.nodes.len()
    }

    fn name(&self) -> &'static str {
        "bvh"
    }
}
