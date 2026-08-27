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
