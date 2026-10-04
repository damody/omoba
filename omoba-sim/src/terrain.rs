//! Exact, bounded fixed-point swept-circle tests against public static polygons.
//! Coordinates/radii outside the supported raw range fail closed. This keeps all
//! cross products and squared-distance comparisons inside i128 without floats.
use crate::{Fixed64, Vec2};

const MAX_RAW: i64 = 1 << 29;
type Point = (i128, i128);
fn point(v: Vec2) -> Point { (i128::from(v.x.raw()), i128::from(v.y.raw())) }
fn cross(a: Point, b: Point, p: Point) -> i128 {
    (b.0-a.0)*(p.1-a.1)-(b.1-a.1)*(p.0-a.0)
}
fn on_segment(a: Point, b: Point, p: Point) -> bool {
    cross(a,b,p)==0 && p.0>=a.0.min(b.0) && p.0<=a.0.max(b.0)
        && p.1>=a.1.min(b.1) && p.1<=a.1.max(b.1)
}
fn intersects(a: Point, b: Point, c: Point, d: Point) -> bool {
    let (ac,ad,ca,cb)=(cross(a,b,c),cross(a,b,d),cross(c,d,a),cross(c,d,b));
    (ac.signum()*ad.signum()<0 && ca.signum()*cb.signum()<0)
        || on_segment(a,b,c) || on_segment(a,b,d)
        || on_segment(c,d,a) || on_segment(c,d,b)
}
fn near_segment(p: Point, a: Point, b: Point, radius_sq: i128) -> bool {
    let (dx,dy)=(b.0-a.0,b.1-a.1);
    let (px,py)=(p.0-a.0,p.1-a.1);
    let length_sq=dx*dx+dy*dy;
    let projection=px*dx+py*dy;
    if projection<=0 { return px*px+py*py<=radius_sq; }
    if projection>=length_sq {
        let (x,y)=(p.0-b.0,p.1-b.1);
        return x*x+y*y<=radius_sq;
    }
    let area=dx*py-dy*px;
    area*area<=radius_sq*length_sq
}
fn inside(p: Point, polygon: &[Vec2]) -> bool {
    let mut inside=false;
    for i in 0..polygon.len() {
        let (a,b)=(point(polygon[i]),point(polygon[(i+1)%polygon.len()]));
        if on_segment(a,b,p) { return true; }
        if (a.1>p.1)!=(b.1>p.1) {
            let side=cross(a,b,p);
            if (b.1>a.1 && side>0) || (b.1<a.1 && side<0) { inside=!inside; }
        }
    }
    inside
}

/// Touching counts as blocked. Handles concave/degenerate edges and zero moves;
/// tests the whole segment, so a thin wall cannot be crossed by a large step.
pub fn swept_circle_hits_polygon(from: Vec2, to: Vec2, radius: Fixed64, polygon: &[Vec2]) -> bool {
    if polygon.len()<3 { return false; }
    let bounded=|v: &Vec2| v.x.raw().unsigned_abs()<=MAX_RAW as u64
        && v.y.raw().unsigned_abs()<=MAX_RAW as u64;
    if radius.raw()<0 || radius.raw()>MAX_RAW || !bounded(&from) || !bounded(&to)
        || polygon.iter().any(|p| !bounded(p)) { return true; }
    let (from,to)=(point(from),point(to));
    // Integer broad phase: distant public walls must not pay for narrow-phase
    // segment distance tests on every BFS edge. Inclusive bounds retain tangency.
    let (mut min_x,mut min_y)=(i128::MAX,i128::MAX);
    let (mut max_x,mut max_y)=(i128::MIN,i128::MIN);
    for p in polygon {
        let (x,y)=point(*p);
        min_x=min_x.min(x);min_y=min_y.min(y);max_x=max_x.max(x);max_y=max_y.max(y);
    }
    let r=i128::from(radius.raw());
    if from.0.max(to.0)+r<min_x || from.0.min(to.0)-r>max_x
        || from.1.max(to.1)+r<min_y || from.1.min(to.1)-r>max_y { return false; }
    if inside(from,polygon) || inside(to,polygon) { return true; }
    let radius_sq=i128::from(radius.raw())*i128::from(radius.raw());
    (0..polygon.len()).any(|i| {
        let (a,b)=(point(polygon[i]),point(polygon[(i+1)%polygon.len()]));
        intersects(from,to,a,b) || near_segment(from,a,b,radius_sq)
            || near_segment(to,a,b,radius_sq) || near_segment(a,from,to,radius_sq)
            || near_segment(b,from,to,radius_sq)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(x:i32,y:i32)->Vec2 { Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y)) }
    fn wall()->[Vec2;4] { [p(50,-100),p(51,-100),p(51,100),p(50,100)] }
    #[test]
    fn thin_wall_and_diagonal_corner_cannot_be_tunnelled() {
        assert!(swept_circle_hits_polygon(p(0,0),p(100,0),Fixed64::ZERO,&wall()));
        let square=[p(40,40),p(60,40),p(60,60),p(40,60)];
        assert!(swept_circle_hits_polygon(p(0,0),p(100,100),Fixed64::ONE,&square));
        assert!(!swept_circle_hits_polygon(p(0,0),p(100,0),Fixed64::ONE,&square));
    }
    #[test]
    fn exact_radius_boundary_zero_move_and_winding() {
        let mut wall=wall();
        for _ in 0..2 {
            assert!(swept_circle_hits_polygon(p(40,0),p(40,0),Fixed64::from_i32(10),&wall));
            assert!(!swept_circle_hits_polygon(p(39,0),p(39,0),Fixed64::from_i32(10),&wall));
            assert!(swept_circle_hits_polygon(p(50,0),p(50,0),Fixed64::ZERO,&wall));
            wall.reverse();
        }
    }
    #[test]
    fn concave_duplicate_edges_and_capsule_endpoints() {
        let poly=[p(0,0),p(100,0),p(100,20),p(20,20),p(20,100),p(0,100),p(0,100)];
        assert!(!swept_circle_hits_polygon(p(40,40),p(90,90),Fixed64::ONE,&poly));
        assert!(swept_circle_hits_polygon(p(40,40),p(10,40),Fixed64::ONE,&poly));
        assert!(swept_circle_hits_polygon(p(0,110),p(100,110),Fixed64::from_i32(10),&wall()));
    }
    #[test]
    fn extreme_inputs_fail_closed_without_overflow() {
        let extreme=Vec2::new(Fixed64::from_raw(i64::MIN),Fixed64::ZERO);
        assert!(swept_circle_hits_polygon(extreme,p(0,0),Fixed64::ONE,&wall()));
        assert!(swept_circle_hits_polygon(p(0,0),p(1,1),Fixed64::from_raw(i64::MAX),&wall()));
        assert!(swept_circle_hits_polygon(p(0,0),p(1,1),-Fixed64::ONE,&wall()));
    }
}
