//! 2D geometry helpers shared by deterministic runtime code.

use vek::Vec2;

/// Ray-casting point-in-polygon test. Boundary points are treated as inside.
pub fn point_in_polygon(p: Vec2<f32>, poly: &[Vec2<f32>]) -> bool {
    if poly.len() < 3 {
        return false;
    }
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let pi = poly[i];
        let pj = poly[j];
        let cond = (pi.y > p.y) != (pj.y > p.y)
            && p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y + f32::EPSILON) + pi.x;
        if cond {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Squared shortest distance from point `p` to segment `a-b`.
pub fn point_segment_dist_sq(p: Vec2<f32>, a: Vec2<f32>, b: Vec2<f32>) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let len_sq = ab.x * ab.x + ab.y * ab.y;
    if len_sq < 1e-8 {
        return ap.magnitude_squared();
    }
    let t = (ap.x * ab.x + ap.y * ab.y) / len_sq;
    let t = t.clamp(0.0, 1.0);
    let proj = a + ab * t;
    (p - proj).magnitude_squared()
}

/// Whether a circle overlaps a polygon.
pub fn circle_hits_polygon(center: Vec2<f32>, r: f32, poly: &[Vec2<f32>]) -> bool {
    if poly.len() < 3 {
        return false;
    }
    if point_in_polygon(center, poly) {
        return true;
    }
    let r2 = r * r;
    let n = poly.len();
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        if point_segment_dist_sq(center, a, b) < r2 {
            return true;
        }
    }
    false
}

/// Whether a circle overlaps any unit circle except `self_id`.
pub fn circle_hits_units(
    center: Vec2<f32>,
    r: f32,
    units: &[(u32, Vec2<f32>, f32)],
    self_id: u32,
) -> bool {
    for &(id, other_c, other_r) in units {
        if id == self_id {
            continue;
        }
        let d = center - other_c;
        let min_d = r + other_r;
        if d.magnitude_squared() < min_d * min_d {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<Vec2<f32>> {
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(0.0, 100.0),
        ]
    }

    fn concave_u() -> Vec<Vec2<f32>> {
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(70.0, 100.0),
            Vec2::new(70.0, 30.0),
            Vec2::new(30.0, 30.0),
            Vec2::new(30.0, 100.0),
            Vec2::new(0.0, 100.0),
        ]
    }

    #[test]
    fn point_inside_convex() {
        assert!(point_in_polygon(Vec2::new(50.0, 50.0), &square()));
    }

    #[test]
    fn point_outside_convex() {
        assert!(!point_in_polygon(Vec2::new(200.0, 50.0), &square()));
    }

    #[test]
    fn point_in_concave_arms() {
        assert!(point_in_polygon(Vec2::new(15.0, 50.0), &concave_u()));
        assert!(point_in_polygon(Vec2::new(85.0, 50.0), &concave_u()));
        assert!(!point_in_polygon(Vec2::new(50.0, 80.0), &concave_u()));
    }

    #[test]
    fn circle_inside() {
        assert!(circle_hits_polygon(Vec2::new(50.0, 50.0), 10.0, &square()));
    }

    #[test]
    fn circle_touches_edge() {
        assert!(circle_hits_polygon(Vec2::new(-5.0, 50.0), 10.0, &square()));
        assert!(!circle_hits_polygon(
            Vec2::new(-20.0, 50.0),
            10.0,
            &square()
        ));
    }

    #[test]
    fn circle_separate() {
        assert!(!circle_hits_polygon(
            Vec2::new(300.0, 300.0),
            10.0,
            &square()
        ));
    }

    #[test]
    fn circles_apart() {
        let units = vec![(1u32, Vec2::new(100.0, 0.0), 10.0)];
        assert!(!circle_hits_units(Vec2::new(0.0, 0.0), 10.0, &units, 99));
    }

    #[test]
    fn circles_overlap() {
        let units = vec![(1u32, Vec2::new(15.0, 0.0), 10.0)];
        assert!(circle_hits_units(Vec2::new(0.0, 0.0), 10.0, &units, 99));
    }

    #[test]
    fn circles_ignore_self() {
        let units = vec![(42u32, Vec2::new(0.0, 0.0), 10.0)];
        assert!(!circle_hits_units(Vec2::new(0.0, 0.0), 10.0, &units, 42));
    }
}
