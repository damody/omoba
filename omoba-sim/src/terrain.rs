//! Exact, bounded fixed-point swept-circle tests against public static polygons.
//! Coordinates/radii outside the supported raw range fail closed. This keeps all
//! cross products and squared-distance comparisons inside i128 without floats.
use crate::{Fixed64, Vec2};

pub const MAX_GEOMETRY_RAW: i64 = 1 << 29;
/// Deterministic nonmoving line endpoint. Geometry outside the supported domain
/// and zero axes fail closed; long requests retain direction and clamp to range.
pub fn checked_line_endpoint(origin:Vec2,requested:Vec2,range:Fixed64)->Option<Vec2> {
    if range<=Fixed64::ZERO||range>Fixed64::from_i32(10_000)||origin==requested{return None;}
    checked_travel_endpoint(origin,requested,range).filter(|end|*end!=origin)
}

/// Shared movement endpoint in raw integer distance. Zero budget is a valid
/// stationary step; negative budget or unsupported coordinates fail closed.
/// Ceiling the integer length keeps every rounded step inside its budget.
pub fn checked_travel_endpoint(origin:Vec2,requested:Vec2,budget:Fixed64)->Option<Vec2> {
    let supported=|p:Vec2|p.x.raw().unsigned_abs()<=MAX_GEOMETRY_RAW as u64&&p.y.raw().unsigned_abs()<=MAX_GEOMETRY_RAW as u64;
    if !supported(origin)||!supported(requested)||budget<Fixed64::ZERO{return None;}
    if budget==Fixed64::ZERO||origin==requested{return Some(origin);}
    let delta=requested-origin;let dx=i128::from(delta.x.raw());let dy=i128::from(delta.y.raw());
    let squared=(dx*dx+dy*dy) as u128;
    if squared<=(budget.raw() as u128).pow(2){return Some(requested);}
    // Work in raw integer distance. Fixed64 length_squared truncates subquantum
    // squares and can erase valid short directions. Ceiling keeps the clamped
    // endpoint inside the closed range even for irrational diagonal lengths.
    let floor=squared.isqrt();let length=if floor*floor==squared{floor}else{floor+1};
    let component=|raw:i64|Fixed64::from_raw((i128::from(raw)*i128::from(budget.raw())/length as i128) as i64);
    let end=origin+Vec2::new(component(delta.x.raw()),component(delta.y.raw()));
    supported(end).then_some(end)
}

#[cfg(test)]
mod line_endpoint_tests {
    use super::*;
    #[test]
    fn travel_endpoint_budget_preserves_raw_quanta_and_closed_boundaries() {
        let raw=|x,y|Vec2::new(Fixed64::from_raw(x),Fixed64::from_raw(y));
        for x in -32..=32 {for y in -32..=32 {for amount in [0,1,2,16,64] {
            let end=checked_travel_endpoint(Vec2::ZERO,raw(x,y),Fixed64::from_raw(amount)).unwrap();
            let squared=i128::from(end.x.raw()).pow(2)+i128::from(end.y.raw()).pow(2);
            assert!(squared<=i128::from(amount).pow(2));
            if i128::from(x).pow(2)+i128::from(y).pow(2)<=i128::from(amount).pow(2){assert_eq!(end,raw(x,y));}
        }}}
        assert_eq!(checked_travel_endpoint(Vec2::ZERO,raw(1,0),Fixed64::ZERO),Some(Vec2::ZERO));
        assert_eq!(checked_travel_endpoint(Vec2::ZERO,raw(1,0),Fixed64::from_raw(1)),Some(raw(1,0)));
        assert!(checked_travel_endpoint(Vec2::ZERO,raw(1,0),Fixed64::from_raw(-1)).is_none());
        assert!(checked_travel_endpoint(Vec2::ZERO,raw(i64::MAX,0),Fixed64::ONE).is_none());
    }
    #[test]
    fn checked_line_preserves_short_clamps_axes_and_avoids_unit_vector_quantization() {
        let p=|x,y|Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        assert_eq!(checked_line_endpoint(p(10,20),p(110,20),Fixed64::from_i32(900)),Some(p(110,20)));
        assert_eq!(checked_line_endpoint(p(10,20),p(10_010,20),Fixed64::from_i32(900)),Some(p(910,20)));
        assert_eq!(checked_line_endpoint(p(0,0),p(30_000,40_000),Fixed64::from_i32(1000)),Some(p(600,800)));
        assert_eq!(checked_line_endpoint(p(0,0),p(-30_000,-40_000),Fixed64::from_i32(1000)),Some(p(-600,-800)));
        assert_eq!(Vec2::new(Fixed64::from_raw(1),Fixed64::ZERO).length(),Fixed64::ZERO,"regression: ordinary squared fixed-point length erases the axis");
        for raw in 1..=32 {let aim=Vec2::new(Fixed64::from_raw(raw),Fixed64::ZERO);assert_eq!(checked_line_endpoint(Vec2::ZERO,aim,Fixed64::from_raw(64)),Some(aim));}
        let aim=Vec2::new(Fixed64::from_raw(1),Fixed64::from_raw(1));assert_eq!(checked_line_endpoint(Vec2::ZERO,aim,Fixed64::from_raw(2)),Some(aim));
        assert_eq!(checked_line_endpoint(Vec2::ZERO,Vec2::new(Fixed64::from_raw(2),Fixed64::ZERO),Fixed64::from_raw(1)),Some(Vec2::new(Fixed64::from_raw(1),Fixed64::ZERO)));
    }
    #[test]
    fn checked_line_zero_invalid_range_and_unsupported_coordinates_fail_closed() {
        let origin=Vec2::ZERO;let aim=Vec2::new(Fixed64::ONE,Fixed64::ZERO);
        assert!(checked_line_endpoint(origin,origin,Fixed64::ONE).is_none());
        for range in [Fixed64::ZERO,Fixed64::from_raw(-1),Fixed64::from_i32(10_001)]{assert!(checked_line_endpoint(origin,aim,range).is_none());}
        assert!(checked_line_endpoint(origin,Vec2::new(Fixed64::from_raw(i64::MAX),Fixed64::ZERO),Fixed64::ONE).is_none());
    }
}
const MAX_RAW: i64 = MAX_GEOMETRY_RAW;
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

/// Exact point-in-capsule test. Tangency and endpoint disks are included.
/// Unsupported coordinates/radii fail closed without overflowing i128.
pub fn point_in_swept_circle(p: Vec2, from: Vec2, to: Vec2, radius: Fixed64) -> bool {
    let bounded=|v:Vec2|v.x.raw().unsigned_abs()<=MAX_RAW as u64
        && v.y.raw().unsigned_abs()<=MAX_RAW as u64;
    radius.raw()>=0 && radius.raw()<=MAX_RAW && bounded(p) && bounded(from) && bounded(to)
        && near_segment(point(p),point(from),point(to),i128::from(radius.raw()).pow(2))
}

/// First intersecting prefix of a swept circle, rounded up to Q32 time.
/// Supported raw geometry permits every scaled distance product inside i128.
/// A complete segment is admitted first; prefix intersection is then monotone
/// even after the center passes the point's orthogonal projection.
pub fn swept_circle_contact_q32(p:Vec2,from:Vec2,to:Vec2,radius:Fixed64)->Option<u64> {
    if !point_in_swept_circle(p,from,to,radius){return None;}
    let (a,b,c)=(point(from),point(to),point(p));
    let (dx,dy,px,py)=(b.0-a.0,b.1-a.1,c.0-a.0,c.1-a.1);
    let radius_sq=i128::from(radius.raw()).pow(2);
    if px*px+py*py<=radius_sq{return Some(0);}
    let length_sq=dx*dx+dy*dy;let projection=px*dx+py*dy;
    const SCALE:i128=1_i128<<32;
    let prefix_hits=|time:u64| {
        let t=i128::from(time);
        if projection*SCALE<=length_sq*t{return true;}
        let (x,y)=(px*SCALE-dx*t,py*SCALE-dy*t);
        x*x+y*y<=radius_sq*SCALE*SCALE
    };
    let (mut low,mut high)=(0_u64,1_u64<<32);
    while low<high {let mid=low+(high-low)/2;if prefix_hits(mid){high=mid;}else{low=mid+1;}}
    Some(low)
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
    #[test]
    fn projectile_contact_q32_closed_axes_tangency_and_stationary_cases() {
        let p=|x,y|Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        let ceil=|numerator:u64,denominator:u64|(numerator*(1_u64<<32)).div_ceil(denominator);
        assert_eq!(swept_circle_contact_q32(p(100,0),p(0,0),p(1000,0),Fixed64::from_i32(5)),Some(ceil(95,1000)));
        assert_eq!(swept_circle_contact_q32(p(100,5),p(0,0),p(1000,0),Fixed64::from_i32(5)),Some(ceil(100,1000)));
        assert_eq!(swept_circle_contact_q32(p(102,0),p(0,0),p(100,0),Fixed64::from_i32(5)),Some(ceil(97,100)));
        assert_eq!(swept_circle_contact_q32(p(33,0),p(100,0),p(0,0),Fixed64::from_i32(5)),Some(ceil(62,100)));
        assert_eq!(swept_circle_contact_q32(p(33,0),p(0,0),p(100,0),Fixed64::ZERO),Some(ceil(33,100)));
        assert_eq!(swept_circle_contact_q32(p(1,0),p(0,0),p(0,0),Fixed64::ONE),Some(0));
        assert!(swept_circle_contact_q32(p(2,0),p(0,0),p(0,0),Fixed64::ONE).is_none());
    }
    #[test]
    fn projectile_contact_q32_preserves_raw_boundary_at_large_translation() {
        let p=|x,y|Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        let offset=p(524000,524000);let outside=p(50,1)+Vec2::new(Fixed64::ZERO,Fixed64::from_raw(1));
        assert!(swept_circle_contact_q32(outside,Vec2::ZERO,p(100,0),Fixed64::ONE).is_none());
        assert!(swept_circle_contact_q32(offset+outside,offset,offset+p(100,0),Fixed64::ONE).is_none());
        let tangent=p(50,1);assert_eq!(swept_circle_contact_q32(tangent,Vec2::ZERO,p(100,0),Fixed64::ONE),
            swept_circle_contact_q32(offset+tangent,offset,offset+p(100,0),Fixed64::ONE));
        assert!(swept_circle_contact_q32(p(0,0),p(0,0),p(100,0),Fixed64::from_raw(-1)).is_none());
        assert!(swept_circle_contact_q32(Vec2::new(Fixed64::from_raw(i64::MAX),Fixed64::ZERO),Vec2::ZERO,p(100,0),Fixed64::ONE).is_none());
    }
    #[test]
    fn projectile_contact_q32_matches_capsule_and_translation_on_raw_grid() {
        let raw=|x,y|Vec2::new(Fixed64::from_raw(x),Fixed64::from_raw(y));let shift=raw(-200,145);
        for x in -4..=20 {for y in -8..=8 {
            let (point,start,end,radius)=(raw(x,y),Vec2::ZERO,raw(16,0),Fixed64::from_raw(4));
            let contact=swept_circle_contact_q32(point,start,end,radius);
            assert_eq!(contact.is_some(),point_in_swept_circle(point,start,end,radius));
            assert_eq!(contact,swept_circle_contact_q32(point+shift,start+shift,end+shift,radius));
            if let Some(time)=contact {assert!(time<=1_u64<<32);}
        }}
    }
    fn p(x:i32,y:i32)->Vec2 { Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y)) }
    fn wall()->[Vec2;4] { [p(50,-100),p(51,-100),p(51,100),p(50,100)] }
    #[test]
    fn capsule_tangency_endpoints_and_one_quantum_outside_are_exact() {
        let radius=Fixed64::from_i32(10);
        for point in [p(0,10),p(50,-10),p(-10,0),p(110,0)] {
            assert!(point_in_swept_circle(point,p(0,0),p(100,0),radius));
            assert!(point_in_swept_circle(point,p(100,0),p(0,0),radius));
        }
        assert!(!point_in_swept_circle(Vec2::new(Fixed64::from_i32(50),radius+Fixed64::from_raw(1)),p(0,0),p(100,0),radius));
        assert!(!point_in_swept_circle(p(111,0),p(0,0),p(100,0),radius));
    }
    #[test]
    fn capsule_diagonal_translation_and_zero_length_remain_integer_exact() {
        assert!(point_in_swept_circle(p(47,54),p(0,0),p(100,100),Fixed64::from_i32(5)));
        assert!(!point_in_swept_circle(p(46,54),p(0,0),p(100,100),Fixed64::from_i32(5)));
        let offset=p(100_000,-100_000);
        assert!(point_in_swept_circle(offset+p(47,54),offset,offset+p(100,100),Fixed64::from_i32(5)));
        assert!(point_in_swept_circle(p(3,4),p(0,0),p(0,0),Fixed64::from_i32(5)));
        assert!(!point_in_swept_circle(p(3,5),p(0,0),p(0,0),Fixed64::from_i32(5)));
    }
    #[test]
    fn capsule_unsupported_inputs_fail_closed_without_integer_overflow() {
        let extreme=Vec2::new(Fixed64::from_raw(i64::MIN),Fixed64::ZERO);
        assert!(!point_in_swept_circle(extreme,p(0,0),p(1,1),Fixed64::ONE));
        assert!(!point_in_swept_circle(p(0,0),extreme,p(1,1),Fixed64::ONE));
        assert!(!point_in_swept_circle(p(0,0),p(0,0),p(1,1),-Fixed64::ONE));
        assert!(!point_in_swept_circle(p(0,0),p(0,0),p(1,1),Fixed64::from_raw(i64::MAX)));
    }
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
