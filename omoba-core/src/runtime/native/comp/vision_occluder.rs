//! Server-authoritative deterministic vision occluders.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};

use omoba_sim::{Fixed64, Vec2};

use crate::runtime::native::scene::import_map::{VisionOccluderPolygonJD, VisionTreeJD};

static GEOMETRY_OVERFLOW_COUNT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VisionAabb {
    pub min: Vec2,
    pub max: Vec2,
}

impl VisionAabb {
    pub fn from_segment(a: Vec2, b: Vec2) -> Self {
        Self {
            min: Vec2::new(a.x.min(b.x), a.y.min(b.y)),
            max: Vec2::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    pub fn intersects(self, other: Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisionTreeCircle {
    pub stable_id: u64,
    pub center: Vec2,
    pub radius: Fixed64,
    pub aabb: VisionAabb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisionTerrainPolygon {
    pub stable_id: u64,
    pub name: String,
    pub vertices: Vec<Vec2>,
    pub aabb: VisionAabb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VisionOccluder {
    Tree(VisionTreeCircle),
    Terrain(VisionTerrainPolygon),
}

impl VisionOccluder {
    pub fn stable_key(&self) -> (u8, u64) {
        match self {
            Self::Tree(value) => (0, value.stable_id),
            Self::Terrain(value) => (1, value.stable_id),
        }
    }

    pub fn aabb(&self) -> VisionAabb {
        match self {
            Self::Tree(value) => value.aabb,
            Self::Terrain(value) => value.aabb,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VisionOccluderSet(pub Vec<VisionOccluder>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LosResult {
    Clear,
    Blocked,
}

fn fixed_from_map(value: f32, label: &str) -> Result<Fixed64, String> {
    if !value.is_finite() {
        return Err(format!("{label} must be finite, got {value}"));
    }
    let scaled = value as f64 * omoba_sim::fixed::SCALE as f64;
    if scaled < i64::MIN as f64 || scaled > i64::MAX as f64 {
        return Err(format!("{label} is outside fixed-point range: {value}"));
    }
    Ok(Fixed64::from_raw(scaled.round() as i64))
}

fn point_from_map(x: f32, y: f32, label: &str) -> Result<Vec2, String> {
    Ok(Vec2::new(
        fixed_from_map(x, &format!("{label}.X"))?,
        fixed_from_map(y, &format!("{label}.Y"))?,
    ))
}

fn cross(a: Vec2, b: Vec2, c: Vec2) -> Option<i128> {
    let abx = i128::from(b.x.raw()).checked_sub(i128::from(a.x.raw()))?;
    let aby = i128::from(b.y.raw()).checked_sub(i128::from(a.y.raw()))?;
    let acx = i128::from(c.x.raw()).checked_sub(i128::from(a.x.raw()))?;
    let acy = i128::from(c.y.raw()).checked_sub(i128::from(a.y.raw()))?;
    abx.checked_mul(acy)?.checked_sub(aby.checked_mul(acx)?)
}

fn point_on_segment(point: Vec2, a: Vec2, b: Vec2) -> Option<bool> {
    Ok::<_, ()>(
        cross(a, b, point)? == 0
            && point.x >= a.x.min(b.x)
            && point.x <= a.x.max(b.x)
            && point.y >= a.y.min(b.y)
            && point.y <= a.y.max(b.y),
    )
    .ok()
}

fn segments_intersect(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Option<bool> {
    let o1 = cross(a, b, c)?;
    let o2 = cross(a, b, d)?;
    let o3 = cross(c, d, a)?;
    let o4 = cross(c, d, b)?;
    if o1 == 0 && point_on_segment(c, a, b)?
        || o2 == 0 && point_on_segment(d, a, b)?
        || o3 == 0 && point_on_segment(a, c, d)?
        || o4 == 0 && point_on_segment(b, c, d)?
    {
        return Some(true);
    }
    Some((o1 < 0) != (o2 < 0) && (o3 < 0) != (o4 < 0))
}

fn point_in_polygon_or_boundary(point: Vec2, vertices: &[Vec2]) -> Option<bool> {
    let mut inside = false;
    for index in 0..vertices.len() {
        let a = vertices[index];
        let b = vertices[(index + 1) % vertices.len()];
        if point_on_segment(point, a, b)? {
            return Some(true);
        }
        let crosses_y = (a.y > point.y) != (b.y > point.y);
        if crosses_y {
            // Division-free even/odd test. Normalize sign using dy.
            let dx = i128::from(b.x.raw()) - i128::from(a.x.raw());
            let py = i128::from(point.y.raw()) - i128::from(a.y.raw());
            let dy = i128::from(b.y.raw()) - i128::from(a.y.raw());
            let px = i128::from(point.x.raw()) - i128::from(a.x.raw());
            let lhs = dx.checked_mul(py)?;
            let rhs = px.checked_mul(dy)?;
            if (dy > 0 && lhs > rhs) || (dy < 0 && lhs < rhs) {
                inside = !inside;
            }
        }
    }
    Some(inside)
}

fn polygon_is_simple(vertices: &[Vec2]) -> Option<bool> {
    let n = vertices.len();
    for i in 0..n {
        let a = vertices[i];
        let b = vertices[(i + 1) % n];
        for j in (i + 1)..n {
            if j == i || j == (i + 1) % n || (j + 1) % n == i {
                continue;
            }
            if segments_intersect(a, b, vertices[j], vertices[(j + 1) % n])? {
                return Some(false);
            }
        }
    }
    Some(true)
}

fn polygon_aabb(vertices: &[Vec2]) -> VisionAabb {
    let mut min = vertices[0];
    let mut max = vertices[0];
    for point in &vertices[1..] {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
    }
    VisionAabb { min, max }
}

impl VisionOccluderSet {
    pub fn from_descriptors(
        trees: &[VisionTreeJD],
        polygons: &[VisionOccluderPolygonJD],
    ) -> Result<Self, String> {
        let mut ids = BTreeSet::new();
        let mut values = Vec::with_capacity(trees.len() + polygons.len());
        for tree in trees {
            if tree.StableId == 0 || !ids.insert(tree.StableId) {
                return Err(format!(
                    "VisionTree StableId {} must be unique and non-zero",
                    tree.StableId
                ));
            }
            if !tree.Radius.is_finite() || tree.Radius <= 0.0 {
                return Err(format!(
                    "VisionTree StableId {} Radius must be finite and positive",
                    tree.StableId
                ));
            }
            let center = point_from_map(tree.X, tree.Y, &format!("VisionTree {}", tree.StableId))?;
            let radius =
                fixed_from_map(tree.Radius, &format!("VisionTree {}.Radius", tree.StableId))?;
            let extent = Vec2::new(radius, radius);
            values.push(VisionOccluder::Tree(VisionTreeCircle {
                stable_id: tree.StableId,
                center,
                radius,
                aabb: VisionAabb {
                    min: center - extent,
                    max: center + extent,
                },
            }));
        }
        for polygon in polygons {
            if polygon.StableId == 0 || !ids.insert(polygon.StableId) {
                return Err(format!(
                    "VisionOccluderPolygon StableId {} must be unique and non-zero",
                    polygon.StableId
                ));
            }
            if polygon.Points.len() < 3 {
                return Err(format!(
                    "VisionOccluderPolygon StableId {} must have at least 3 points",
                    polygon.StableId
                ));
            }
            let vertices: Vec<_> = polygon
                .Points
                .iter()
                .enumerate()
                .map(|(index, point)| {
                    point_from_map(
                        point.X,
                        point.Y,
                        &format!("VisionOccluderPolygon {} point {index}", polygon.StableId),
                    )
                })
                .collect::<Result<_, _>>()?;
            if vertices.windows(2).any(|pair| pair[0] == pair[1])
                || vertices.first() == vertices.last()
            {
                return Err(format!(
                    "VisionOccluderPolygon StableId {} has duplicate adjacent/closing points",
                    polygon.StableId
                ));
            }
            let mut area = 0i128;
            for index in 0..vertices.len() {
                let a = vertices[index];
                let b = vertices[(index + 1) % vertices.len()];
                let term = i128::from(a.x.raw())
                    .checked_mul(i128::from(b.y.raw()))
                    .and_then(|lhs| {
                        i128::from(a.y.raw())
                            .checked_mul(i128::from(b.x.raw()))
                            .and_then(|rhs| lhs.checked_sub(rhs))
                    })
                    .ok_or_else(|| {
                        format!(
                            "VisionOccluderPolygon StableId {} area overflow",
                            polygon.StableId
                        )
                    })?;
                area = area.checked_add(term).ok_or_else(|| {
                    format!(
                        "VisionOccluderPolygon StableId {} area overflow",
                        polygon.StableId
                    )
                })?;
            }
            if area == 0 {
                return Err(format!(
                    "VisionOccluderPolygon StableId {} has zero area",
                    polygon.StableId
                ));
            }
            if !polygon_is_simple(&vertices).ok_or_else(|| {
                format!(
                    "VisionOccluderPolygon StableId {} intersection overflow",
                    polygon.StableId
                )
            })? {
                return Err(format!(
                    "VisionOccluderPolygon StableId {} is self-intersecting",
                    polygon.StableId
                ));
            }
            let aabb = polygon_aabb(&vertices);
            values.push(VisionOccluder::Terrain(VisionTerrainPolygon {
                stable_id: polygon.StableId,
                name: polygon.Name.clone(),
                vertices,
                aabb,
            }));
        }
        values.sort_by_key(VisionOccluder::stable_key);
        Ok(Self(values))
    }

    pub fn line_of_sight(&self, source: Vec2, target: Vec2) -> LosResult {
        line_of_sight(&self.0, source, target)
    }
}

pub fn line_of_sight(occluders: &[VisionOccluder], source: Vec2, target: Vec2) -> LosResult {
    let segment_aabb = VisionAabb::from_segment(source, target);
    for occluder in occluders {
        if !segment_aabb.intersects(occluder.aabb()) {
            continue;
        }
        let result = match occluder {
            VisionOccluder::Tree(tree) => tree_blocks(source, target, tree),
            VisionOccluder::Terrain(polygon) => polygon_blocks(source, target, polygon),
        };
        match result {
            Some(true) => return LosResult::Blocked,
            Some(false) => {}
            None => {
                let count = GEOMETRY_OVERFLOW_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                if count.is_power_of_two() {
                    log::warn!("vision geometry overflow; failing closed count={count}");
                }
                return LosResult::Blocked;
            }
        }
    }
    LosResult::Clear
}

/// Broad phase bucket edge of [`IndexedVisionOccluders`]: 256 world units.
const OCCLUDER_BUCKET_RAW: i64 = 256 * omoba_sim::fixed::SCALE;
/// Lists up to this long are scanned directly (every pre-E187 map stays below it).
const OCCLUDER_SCAN_LIMIT: usize = 32;
/// Occluder and query coordinates (raw) the index handles; anything beyond takes the linear scan.
/// Below it the exact tests never overflow, so a test that blocks always meets the segment
/// inside the occluder's bounds.
const OCCLUDER_INDEX_BOUND_RAW: i64 = 1 << 29;
/// An occluder covering more buckets than this is tested for every query instead.
const OCCLUDER_BUCKET_LIMIT: i64 = 4096;

#[derive(Debug, Default)]
struct OccluderBuckets {
    buckets: std::collections::HashMap<(i64, i64), Vec<u32>>,
    always: Vec<u32>,
}

/// An immutable occluder list with a uniform-grid broad phase (E187 §5.5: the nobu map's 1,564
/// VisionTrees). [`IndexedVisionOccluders::line_of_sight`] answers exactly what the linear
/// [`line_of_sight`] answers for the same list: a blocking occluder meets the segment inside its
/// bounds, so it is registered in a bucket the segment crosses; the candidates are then tested in
/// list order with the linear scan's own bounds rejection and exact test.
#[derive(Clone, Debug, Default)]
pub struct IndexedVisionOccluders {
    list: std::sync::Arc<[VisionOccluder]>,
    index: Option<std::sync::Arc<OccluderBuckets>>,
}

impl PartialEq for IndexedVisionOccluders {
    fn eq(&self, other: &Self) -> bool { self.list == other.list }
}

impl Eq for IndexedVisionOccluders {}

impl std::ops::Deref for IndexedVisionOccluders {
    type Target = [VisionOccluder];
    fn deref(&self) -> &[VisionOccluder] { &self.list }
}

impl From<Vec<VisionOccluder>> for IndexedVisionOccluders {
    fn from(list: Vec<VisionOccluder>) -> Self { Self::new(list.into()) }
}

impl From<std::sync::Arc<[VisionOccluder]>> for IndexedVisionOccluders {
    fn from(list: std::sync::Arc<[VisionOccluder]>) -> Self { Self::new(list) }
}

fn within_index(value: i64) -> bool { value.unsigned_abs() <= OCCLUDER_INDEX_BOUND_RAW as u64 }

impl IndexedVisionOccluders {
    pub fn new(list: std::sync::Arc<[VisionOccluder]>) -> Self {
        if list.len() <= OCCLUDER_SCAN_LIMIT { return Self { list, index: None }; }
        let mut index = OccluderBuckets::default();
        for (position, occluder) in list.iter().enumerate() {
            let aabb = occluder.aabb();
            let radius_ok = match occluder { VisionOccluder::Tree(tree) => within_index(tree.radius.raw()), _ => true };
            let bounded = radius_ok && [aabb.min.x, aabb.min.y, aabb.max.x, aabb.max.y].iter().all(|v| within_index(v.raw()));
            let cells = [aabb.min.x.raw().div_euclid(OCCLUDER_BUCKET_RAW), aabb.min.y.raw().div_euclid(OCCLUDER_BUCKET_RAW),
                aabb.max.x.raw().div_euclid(OCCLUDER_BUCKET_RAW), aabb.max.y.raw().div_euclid(OCCLUDER_BUCKET_RAW)];
            if !bounded || (cells[2] - cells[0] + 1) * (cells[3] - cells[1] + 1) > OCCLUDER_BUCKET_LIMIT {
                index.always.push(position as u32);
                continue;
            }
            for cx in cells[0]..=cells[2] {
                for cy in cells[1]..=cells[3] {
                    index.buckets.entry((cx, cy)).or_default().push(position as u32);
                }
            }
        }
        Self { list, index: Some(std::sync::Arc::new(index)) }
    }

    pub fn line_of_sight(&self, source: Vec2, target: Vec2) -> LosResult {
        let Some(index) = self.index.as_deref() else { return line_of_sight(&self.list, source, target) };
        if ![source.x, source.y, target.x, target.y].iter().all(|v| within_index(v.raw())) {
            return line_of_sight(&self.list, source, target);
        }
        thread_local! { static SCRATCH: std::cell::RefCell<Vec<u32>> = const { std::cell::RefCell::new(Vec::new()) }; }
        SCRATCH.with(|scratch| {
            let Ok(mut candidates) = scratch.try_borrow_mut() else { return line_of_sight(&self.list, source, target) };
            candidates.clear();
            candidates.extend_from_slice(&index.always);
            // Column sweep: per bucket column the segment's y extent, one bucket of margin each way.
            let (from, to) = if source.x.raw() <= target.x.raw() { (source, target) } else { (target, source) };
            let (x0, x1, y0, y1) = (i128::from(from.x.raw()), i128::from(to.x.raw()), i128::from(from.y.raw()), i128::from(to.y.raw()));
            let bucket = i128::from(OCCLUDER_BUCKET_RAW);
            let y_at = |x: i128| if x1 == x0 { y0 } else { y0 + (x - x0) * (y1 - y0) / (x1 - x0) };
            for column in x0.div_euclid(bucket)..=x1.div_euclid(bucket) {
                let (xa, xb) = ((column * bucket).max(x0), ((column + 1) * bucket).min(x1));
                let (ya, yb) = if x1 == x0 { (y0, y1) } else { (y_at(xa), y_at(xb)) };
                for row in ya.min(yb).div_euclid(bucket) - 1..=ya.max(yb).div_euclid(bucket) + 1 {
                    if let Some(entries) = index.buckets.get(&(column as i64, row as i64)) {
                        candidates.extend_from_slice(entries);
                    }
                }
            }
            candidates.sort_unstable();
            candidates.dedup();
            let segment_aabb = VisionAabb::from_segment(source, target);
            for &position in candidates.iter() {
                let occluder = &self.list[position as usize];
                if !segment_aabb.intersects(occluder.aabb()) { continue; }
                let result = match occluder {
                    VisionOccluder::Tree(tree) => tree_blocks(source, target, tree),
                    VisionOccluder::Terrain(polygon) => polygon_blocks(source, target, polygon),
                };
                if result != Some(false) { return line_of_sight(std::slice::from_ref(occluder), source, target); }
            }
            LosResult::Clear
        })
    }
}

/// Line of sight through an occluder collection: the linear scan for plain lists, the bucketed
/// one for [`IndexedVisionOccluders`] (E187 §5.5); both give the same answer.
pub trait LineOfSight {
    fn line_of_sight(&self, source: Vec2, target: Vec2) -> LosResult;
}

impl LineOfSight for [VisionOccluder] {
    fn line_of_sight(&self, source: Vec2, target: Vec2) -> LosResult { line_of_sight(self, source, target) }
}

impl<const N: usize> LineOfSight for [VisionOccluder; N] {
    fn line_of_sight(&self, source: Vec2, target: Vec2) -> LosResult { line_of_sight(self, source, target) }
}

impl LineOfSight for Vec<VisionOccluder> {
    fn line_of_sight(&self, source: Vec2, target: Vec2) -> LosResult { line_of_sight(self, source, target) }
}

impl LineOfSight for IndexedVisionOccluders {
    fn line_of_sight(&self, source: Vec2, target: Vec2) -> LosResult { IndexedVisionOccluders::line_of_sight(self, source, target) }
}

impl VisionOccluderSet {
    /// The committed, indexed form of this set (E187 §5.5). A match's set is fixed, so the index
    /// is built once per distinct set and shared: each Wave B view clones the Arc.
    pub fn committed(&self) -> IndexedVisionOccluders {
        thread_local! {
            static CACHE: std::cell::RefCell<Option<IndexedVisionOccluders>> = const { std::cell::RefCell::new(None) };
        }
        CACHE.with(|cache| {
            if let Some(cached) = cache.borrow().as_ref().filter(|cached| cached.list[..] == self.0[..]) {
                return cached.clone();
            }
            let built = IndexedVisionOccluders::from(self.0.clone());
            *cache.borrow_mut() = Some(built.clone());
            built
        })
    }
}

fn squared_distance(a: Vec2, b: Vec2) -> Option<i128> {
    let x = i128::from(a.x.raw()).checked_sub(i128::from(b.x.raw()))?;
    let y = i128::from(a.y.raw()).checked_sub(i128::from(b.y.raw()))?;
    x.checked_mul(x)?.checked_add(y.checked_mul(y)?)
}

fn tree_blocks(source: Vec2, target: Vec2, tree: &VisionTreeCircle) -> Option<bool> {
    let r2 = i128::from(tree.radius.raw()).checked_mul(i128::from(tree.radius.raw()))?;
    if squared_distance(source, tree.center)? <= r2 {
        return Some(false);
    }
    if squared_distance(target, tree.center)? <= r2 {
        return Some(true);
    }
    let dx = i128::from(target.x.raw()).checked_sub(i128::from(source.x.raw()))?;
    let dy = i128::from(target.y.raw()).checked_sub(i128::from(source.y.raw()))?;
    let cx = i128::from(tree.center.x.raw()).checked_sub(i128::from(source.x.raw()))?;
    let cy = i128::from(tree.center.y.raw()).checked_sub(i128::from(source.y.raw()))?;
    let length2 = dx.checked_mul(dx)?.checked_add(dy.checked_mul(dy)?)?;
    if length2 == 0 {
        return Some(false);
    }
    let projection = cx.checked_mul(dx)?.checked_add(cy.checked_mul(dy)?)?;
    if projection <= 0 || projection >= length2 {
        return Some(false);
    }
    let cross_value = dx.checked_mul(cy)?.checked_sub(dy.checked_mul(cx)?)?;
    let lhs = cross_value.checked_mul(cross_value)?;
    let rhs = r2.checked_mul(length2)?;
    Some(lhs <= rhs)
}

fn polygon_blocks(source: Vec2, target: Vec2, polygon: &VisionTerrainPolygon) -> Option<bool> {
    if point_in_polygon_or_boundary(source, &polygon.vertices)? {
        return Some(false);
    }
    if point_in_polygon_or_boundary(target, &polygon.vertices)? {
        return Some(true);
    }
    for index in 0..polygon.vertices.len() {
        if segments_intersect(
            source,
            target,
            polygon.vertices[index],
            polygon.vertices[(index + 1) % polygon.vertices.len()],
        )? {
            return Some(true);
        }
    }
    Some(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::native::scene::import_map::PointJD;

    fn p(x: i32, y: i32) -> Vec2 {
        Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y))
    }

    fn tree(x: f32, y: f32, radius: f32) -> VisionTreeJD {
        VisionTreeJD {
            StableId: 1,
            X: x,
            Y: y,
            Radius: radius,
        }
    }

    fn polygon(stable_id: u64, points: &[(f32, f32)]) -> VisionOccluderPolygonJD {
        VisionOccluderPolygonJD {
            StableId: stable_id,
            Name: format!("polygon-{stable_id}"),
            Points: points
                .iter()
                .map(|&(x, y)| PointJD { X: x, Y: y })
                .collect(),
        }
    }

    #[test]
    fn tree_blocks_behind_tangent_and_inside_but_not_front_or_beyond_target() {
        let set = VisionOccluderSet::from_descriptors(&[tree(5.0, 0.0, 2.0)], &[]).unwrap();
        assert_eq!(set.line_of_sight(p(0, 0), p(10, 0)), LosResult::Blocked);
        assert_eq!(set.line_of_sight(p(0, 2), p(10, 2)), LosResult::Blocked);
        assert_eq!(set.line_of_sight(p(0, 0), p(5, 0)), LosResult::Blocked);
        assert_eq!(set.line_of_sight(p(0, 0), p(2, 0)), LosResult::Clear);
        assert_eq!(set.line_of_sight(p(5, 0), p(10, 0)), LosResult::Clear);
    }

    #[test]
    fn convex_and_concave_polygons_block_only_segments_crossing_the_shape() {
        let square = polygon(10, &[(2.0, -2.0), (6.0, -2.0), (6.0, 2.0), (2.0, 2.0)]);
        let concave = polygon(
            11,
            &[
                (2.0, 3.0),
                (8.0, 3.0),
                (8.0, 8.0),
                (5.0, 8.0),
                (5.0, 5.0),
                (2.0, 5.0),
            ],
        );
        let set = VisionOccluderSet::from_descriptors(&[], &[square, concave]).unwrap();
        assert_eq!(set.line_of_sight(p(0, 0), p(10, 0)), LosResult::Blocked);
        assert_eq!(set.line_of_sight(p(0, 6), p(4, 6)), LosResult::Clear);
        assert_eq!(set.line_of_sight(p(0, 4), p(10, 4)), LosResult::Blocked);
        assert_eq!(set.line_of_sight(p(3, 4), p(10, 4)), LosResult::Clear);
    }

    #[test]
    fn winding_does_not_change_polygon_result() {
        let points = [(2.0, -2.0), (6.0, -2.0), (6.0, 2.0), (2.0, 2.0)];
        let mut reversed = points.to_vec();
        reversed.reverse();
        let a = VisionOccluderSet::from_descriptors(&[], &[polygon(20, &points)]).unwrap();
        let b = VisionOccluderSet::from_descriptors(&[], &[polygon(21, &reversed)]).unwrap();
        assert_eq!(
            a.line_of_sight(p(0, 0), p(10, 0)),
            b.line_of_sight(p(0, 0), p(10, 0))
        );
    }

    /// E187 §5.5: the bucketed scan answers exactly as the linear one over many trees and walls,
    /// short and long, axis-aligned and diagonal segments, endpoints inside occluders and beyond
    /// the indexed range.
    #[test]
    fn indexed_line_of_sight_matches_the_linear_scan() {
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        let mut next = |span: i64| { seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17; (seed % (2 * span as u64 + 1)) as i64 - span };
        let raw = |x: i64, y: i64| Vec2::new(Fixed64::from_raw(x), Fixed64::from_raw(y));
        let mut list = Vec::new();
        for id in 0..600u64 {
            let center = raw(next(9000) * 1024, next(9000) * 1024);
            let radius = Fixed64::from_i32(1 + (next(80).unsigned_abs() as i32));
            let corner = Vec2::new(radius, radius);
            list.push(VisionOccluder::Tree(VisionTreeCircle { stable_id: id + 1, center, radius,
                aabb: VisionAabb { min: center - corner, max: center + corner } }));
        }
        for id in 0..80u64 {
            let (x, y, w, h) = (next(9000) * 1024, next(9000) * 1024, (1 + next(600).abs()) * 1024, (1 + next(600).abs()) * 1024);
            let vertices = vec![raw(x, y), raw(x + w, y), raw(x + w, y + h), raw(x, y + h)];
            list.push(VisionOccluder::Terrain(VisionTerrainPolygon { stable_id: id + 1, name: format!("wall_{id}"),
                aabb: polygon_aabb(&vertices), vertices }));
        }
        let far = raw(1 << 40, 0);
        list.push(VisionOccluder::Tree(VisionTreeCircle { stable_id: 9999, center: far, radius: Fixed64::ONE,
            aabb: VisionAabb { min: far, max: far } }));
        let indexed = IndexedVisionOccluders::from(list.clone());
        assert!(indexed.index.is_some());
        let (mut blocked, mut clear) = (0, 0);
        for case in 0..20_000 {
            let source = raw(next(9500) * 1024 + next(1023), next(9500) * 1024 + next(1023));
            let target = match case % 4 {
                0 => raw(source.x.raw() + next(1500) * 1024, source.y.raw()),
                1 => raw(source.x.raw(), source.y.raw() + next(1500) * 1024),
                2 => raw(source.x.raw() + next(400) * 1024, source.y.raw() + next(400) * 1024),
                _ => raw(next(9500) * 1024, next(9500) * 1024),
            };
            let want = line_of_sight(&list, source, target);
            assert_eq!(indexed.line_of_sight(source, target), want, "{source:?} -> {target:?}");
            if want == LosResult::Blocked { blocked += 1 } else { clear += 1 }
        }
        assert!(blocked > 1000 && clear > 1000, "both outcomes are exercised: {blocked} / {clear}");
        // Out-of-range queries and short lists take the linear scan.
        assert_eq!(indexed.line_of_sight(raw(0, 0), far), line_of_sight(&list, raw(0, 0), far));
        assert!(IndexedVisionOccluders::from(list[..3].to_vec()).index.is_none());
        // The committed form of a set is shared until the set changes.
        let set = VisionOccluderSet(list.clone());
        assert_eq!(set.committed(), indexed);
        assert!(std::sync::Arc::ptr_eq(&set.committed().list, &set.committed().list));
        let mut changed = list.clone();
        changed.pop();
        assert_eq!(VisionOccluderSet(changed.clone()).committed().len(), changed.len());
    }

    #[test]
    fn invalid_descriptors_are_rejected_without_repair() {
        assert!(VisionOccluderSet::from_descriptors(&[tree(0.0, 0.0, 0.0)], &[]).is_err());
        let duplicate = [tree(0.0, 0.0, 1.0), tree(3.0, 0.0, 1.0)];
        assert!(VisionOccluderSet::from_descriptors(&duplicate, &[]).is_err());
        assert!(
            VisionOccluderSet::from_descriptors(&[], &[polygon(2, &[(0.0, 0.0), (1.0, 1.0)])])
                .is_err()
        );
        assert!(VisionOccluderSet::from_descriptors(
            &[],
            &[polygon(
                3,
                &[(0.0, 0.0), (2.0, 2.0), (0.0, 2.0), (2.0, 0.0)]
            )]
        )
        .is_err());
        assert!(VisionOccluderSet::from_descriptors(
            &[],
            &[polygon(4, &[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)])]
        )
        .is_err());
    }
}
