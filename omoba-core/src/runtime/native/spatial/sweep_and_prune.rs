use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::hash::Hash;

use vek::Vec2;
use voracious_radix_sort::{RadixSort, Radixable};

use super::{Bounds, Entry, SpatialIndex};

#[derive(Copy, Clone, Debug)]
struct AxisRef {
    slot: u32,
    coord: f32,
}

impl PartialEq for AxisRef {
    fn eq(&self, other: &Self) -> bool {
        self.coord == other.coord
    }
}

impl PartialOrd for AxisRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.coord.partial_cmp(&other.coord)
    }
}

impl Radixable<f32> for AxisRef {
    type Key = f32;

    fn key(&self) -> Self::Key {
        self.coord
    }
}

#[derive(Debug, Clone)]
struct Slot<Id, Item> {
    id: Id,
    item: Item,
    position: Vec2<f32>,
    bounding_radius: f32,
}

pub struct SweepAndPrune<Id, Item> {
    slots: Vec<Option<Slot<Id, Item>>>,
    free_slots: Vec<u32>,
    id_to_slot: HashMap<Id, u32>,
    xs: Vec<AxisRef>,
    ys: Vec<AxisRef>,
    max_bounding_radius: f32,
}

impl<Id, Item> SweepAndPrune<Id, Item>
where
    Id: Clone + Eq + Hash + Ord + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_slots: Vec::new(),
            id_to_slot: HashMap::new(),
            xs: Vec::new(),
            ys: Vec::new(),
            max_bounding_radius: 0.0,
        }
    }

    fn alloc_slot(&mut self, slot_data: Slot<Id, Item>) -> u32 {
        if let Some(idx) = self.free_slots.pop() {
            self.slots[idx as usize] = Some(slot_data);
            idx
        } else {
            let idx = self.slots.len() as u32;
            self.slots.push(Some(slot_data));
            idx
        }
    }

    fn free_slot(&mut self, idx: u32) {
        self.slots[idx as usize] = None;
        self.free_slots.push(idx);
    }

    fn axis_insert_sorted(arr: &mut Vec<AxisRef>, item: AxisRef) {
        let pos = arr
            .binary_search_by(|probe| {
                probe
                    .coord
                    .partial_cmp(&item.coord)
                    .unwrap_or(Ordering::Equal)
                    .then(Ordering::Greater)
            })
            .unwrap_or_else(|i| i);
        arr.insert(pos, item);
    }

    fn axis_remove(arr: &mut Vec<AxisRef>, slot: u32, coord: f32) {
        let lower = arr
            .binary_search_by(|probe| probe.coord.partial_cmp(&coord).unwrap_or(Ordering::Equal))
            .unwrap_or_else(|i| i);
        let mut i = lower;
        while i > 0 && arr[i - 1].coord == coord {
            i -= 1;
        }
        while i < arr.len() && arr[i].coord == coord {
            if arr[i].slot == slot {
                arr.remove(i);
                return;
            }
            i += 1;
        }
        if let Some(i) = arr.iter().position(|axis| axis.slot == slot) {
            arr.remove(i);
        }
    }

    fn axis_range(arr: &[AxisRef], lo: f32, hi: f32) -> (usize, usize) {
        let left = arr.partition_point(|p| p.coord < lo);
        let right = arr.partition_point(|p| p.coord <= hi);
        (left, right)
    }

    fn recompute_max_radius(&mut self) {
        self.max_bounding_radius = self
            .slots
            .iter()
            .filter_map(|slot| slot.as_ref().map(|slot| slot.bounding_radius))
            .fold(0.0_f32, f32::max);
    }
}

impl<Id, Item> SpatialIndex<Id, Item> for SweepAndPrune<Id, Item>
where
    Id: Clone + Eq + Hash + Ord + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    fn initialize(&mut self, _bounds: Bounds, entries: Vec<Entry<Id, Item>>) {
        self.slots.clear();
        self.free_slots.clear();
        self.id_to_slot.clear();
        self.xs.clear();
        self.ys.clear();
        self.max_bounding_radius = 0.0;

        self.slots.reserve(entries.len());
        self.xs.reserve(entries.len());
        self.ys.reserve(entries.len());

        for entry in entries {
            let slot = self.slots.len() as u32;
            self.id_to_slot.insert(entry.id.clone(), slot);
            self.xs.push(AxisRef {
                slot,
                coord: entry.position.x,
            });
            self.ys.push(AxisRef {
                slot,
                coord: entry.position.y,
            });
            if entry.bounding_radius > self.max_bounding_radius {
                self.max_bounding_radius = entry.bounding_radius;
            }
            self.slots.push(Some(Slot {
                id: entry.id,
                item: entry.item,
                position: entry.position,
                bounding_radius: entry.bounding_radius,
            }));
        }

        if self.xs.len() >= 2 {
            self.xs.voracious_mt_sort(4);
            self.ys.voracious_mt_sort(4);
        }
    }

    fn insert(&mut self, entry: Entry<Id, Item>) {
        if let Some(&old_slot) = self.id_to_slot.get(&entry.id) {
            if let Some(Some(old)) = self.slots.get(old_slot as usize) {
                Self::axis_remove(&mut self.xs, old_slot, old.position.x);
                Self::axis_remove(&mut self.ys, old_slot, old.position.y);
            }
            self.free_slot(old_slot);
            self.id_to_slot.remove(&entry.id);
            self.recompute_max_radius();
        }

        let radius = entry.bounding_radius;
        let pos = entry.position;
        let id = entry.id.clone();
        let slot = self.alloc_slot(Slot {
            id: entry.id,
            item: entry.item,
            position: pos,
            bounding_radius: radius,
        });
        self.id_to_slot.insert(id, slot);
        Self::axis_insert_sorted(&mut self.xs, AxisRef { slot, coord: pos.x });
        Self::axis_insert_sorted(&mut self.ys, AxisRef { slot, coord: pos.y });
        if radius > self.max_bounding_radius {
            self.max_bounding_radius = radius;
        }
    }

    fn remove(&mut self, id: &Id) -> bool {
        let slot = match self.id_to_slot.remove(id) {
            Some(slot) => slot,
            None => return false,
        };
        let (was_max, old_pos) = match self.slots.get(slot as usize) {
            Some(Some(s)) => (s.bounding_radius >= self.max_bounding_radius, s.position),
            _ => return false,
        };
        Self::axis_remove(&mut self.xs, slot, old_pos.x);
        Self::axis_remove(&mut self.ys, slot, old_pos.y);
        self.free_slot(slot);
        if was_max {
            self.recompute_max_radius();
        }
        true
    }

    fn update(&mut self, entry: Entry<Id, Item>) {
        self.insert(entry);
    }

    fn bulk_replace(&mut self, _bounds: Bounds, entries: Vec<Entry<Id, Item>>) {
        use std::collections::HashSet;

        let new_ids: HashSet<Id> = entries.iter().map(|entry| entry.id.clone()).collect();
        let to_remove: Vec<Id> = self
            .id_to_slot
            .keys()
            .filter(|id| !new_ids.contains(*id))
            .cloned()
            .collect();
        let mut removed_max_radius = false;
        for id in &to_remove {
            if let Some(slot) = self.id_to_slot.remove(id) {
                if let Some(Some(s)) = self.slots.get(slot as usize) {
                    if s.bounding_radius >= self.max_bounding_radius {
                        removed_max_radius = true;
                    }
                }
                self.slots[slot as usize] = None;
                self.free_slots.push(slot);
            }
        }

        for entry in entries {
            let radius = entry.bounding_radius;
            if let Some(&slot) = self.id_to_slot.get(&entry.id) {
                if let Some(slot_ref) = self.slots.get_mut(slot as usize) {
                    if let Some(s) = slot_ref.as_mut() {
                        s.item = entry.item;
                        s.position = entry.position;
                        s.bounding_radius = radius;
                    }
                }
            } else {
                let id = entry.id.clone();
                let slot = self.alloc_slot(Slot {
                    id: entry.id,
                    item: entry.item,
                    position: entry.position,
                    bounding_radius: radius,
                });
                self.id_to_slot.insert(id, slot);
            }
            if radius > self.max_bounding_radius {
                self.max_bounding_radius = radius;
            }
        }

        if removed_max_radius {
            self.recompute_max_radius();
        }

        self.xs.clear();
        self.ys.clear();
        self.xs.reserve(self.id_to_slot.len());
        self.ys.reserve(self.id_to_slot.len());
        for (idx, slot_opt) in self.slots.iter().enumerate() {
            if let Some(slot) = slot_opt {
                self.xs.push(AxisRef {
                    slot: idx as u32,
                    coord: slot.position.x,
                });
                self.ys.push(AxisRef {
                    slot: idx as u32,
                    coord: slot.position.y,
                });
            }
        }
        if self.xs.len() >= 2 {
            self.xs.voracious_mt_sort(4);
            self.ys.voracious_mt_sort(4);
        }
    }

    fn query_in_range(&self, center: Vec2<f32>, radius: f32) -> Vec<Entry<Id, Item>> {
        let extended = radius + self.max_bounding_radius;
        let (lx, rx) = Self::axis_range(&self.xs, center.x - extended, center.x + extended);
        let (ly, ry) = Self::axis_range(&self.ys, center.y - extended, center.y + extended);

        let mut x_slots: BTreeSet<u32> = BTreeSet::new();
        for i in lx..rx {
            x_slots.insert(self.xs[i].slot);
        }

        let mut results = Vec::new();
        for i in ly..ry {
            let slot = self.ys[i].slot;
            if !x_slots.contains(&slot) {
                continue;
            }
            if let Some(Some(s)) = self.slots.get(slot as usize) {
                let extended_r = radius + s.bounding_radius.max(0.0);
                if s.position.distance(center) <= extended_r {
                    results.push(Entry {
                        id: s.id.clone(),
                        item: s.item.clone(),
                        position: s.position,
                        bounding_radius: s.bounding_radius,
                    });
                }
            }
        }
        results
    }

    fn count_nodes(&self) -> usize {
        self.id_to_slot.len()
    }

    fn name(&self) -> &'static str {
        "sap"
    }
}
