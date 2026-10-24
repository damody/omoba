//! Bounded point-sector tests. Radius is integer exact; angular comparisons use
//! the engine's shared 4096-tick orientation and Q10 sine/cosine boundaries.
use crate::{Angle,Fixed64,Vec2};
use crate::trig::{sin,cos,TAU_FIXED_RAW,TAU_TICKS};
use crate::terrain::MAX_GEOMETRY_RAW;

/// Nearest shared angle tick, accepting the closed [0, pi] Q10 envelope.
pub fn half_angle_ticks(radians:Fixed64)->Option<u16> {
    let raw=radians.raw();
    if !(0..=TAU_FIXED_RAW/2).contains(&raw) {return None;}
    Some(((raw*i64::from(TAU_TICKS)+TAU_FIXED_RAW/2)/TAU_FIXED_RAW) as u16)
}

/// Includes apex, both angular edges and the radial boundary. A zero half angle
/// is a forward ray; 2048 ticks is a full circle. Invalid dimensions fail closed.
pub fn point_in_sector(point:Vec2,apex:Vec2,axis:Angle,radius:Fixed64,half_ticks:u16)->bool {
    let bounded=|p:Vec2|p.x.raw().unsigned_abs()<=MAX_GEOMETRY_RAW as u64
        && p.y.raw().unsigned_abs()<=MAX_GEOMETRY_RAW as u64;
    if !bounded(point)||!bounded(apex)||radius.raw()<0||radius.raw()>MAX_GEOMETRY_RAW||half_ticks>2048 {return false;}
    let x=i128::from(point.x.raw())-i128::from(apex.x.raw());
    let y=i128::from(point.y.raw())-i128::from(apex.y.raw());
    if x*x+y*y>i128::from(radius.raw()).pow(2) {return false;}
    let (ax,ay)=(i128::from(cos(axis).raw()),i128::from(sin(axis).raw()));
    let dot=x*ax+y*ay;let cross=(ax*y-ay*x).abs();
    if half_ticks==0 {return dot>=0&&cross==0;}
    let half=Angle::from_ticks(i32::from(half_ticks));
    dot*i128::from(sin(half).raw())>=cross*i128::from(cos(half).raw())
}

#[cfg(test)]
mod tests {
 use super::*;
 fn p(x:i64,y:i64)->Vec2 {Vec2::new(Fixed64::from_raw(x),Fixed64::from_raw(y))}
 #[test]
 fn sector_includes_apex_angular_edges_radius_and_excludes_one_raw_outside() {
  let apex=p(10000,-20000);let radius=Fixed64::from_raw(5000);
  for (dx,dy,inside) in [(0,0,true),(3000,3000,true),(3000,-3000,true),(3000,3001,false),
   (3000,-3001,false),(5000,0,true),(5001,0,false),(-1,0,false),(0,1,false)] {
   assert_eq!(point_in_sector(apex+p(dx,dy),apex,Angle::ZERO,radius,512),inside,"{dx} {dy}");
  }
 }
 #[test]
 fn sector_rotations_wide_half_plane_ray_and_full_circle_have_closed_boundaries() {
  for (axis,point) in [(Angle::ZERO,p(100,0)),(Angle::QUARTER_TURN,p(0,100)),
   (Angle::HALF_TURN,p(-100,0)),(Angle::THREE_QUARTER_TURN,p(0,-100))] {
   assert!(point_in_sector(point,Vec2::ZERO,axis,Fixed64::from_raw(100),0));
   assert!(!point_in_sector(-point,Vec2::ZERO,axis,Fixed64::from_raw(100),0));
  }
  for (ticks,point,inside) in [(1024,p(0,100),true),(1024,p(-1,100),false),
   (1536,p(-100,100),true),(1536,p(-101,100),false),(2048,p(-200,0),true)] {
   assert_eq!(point_in_sector(point,Vec2::ZERO,Angle::ZERO,Fixed64::from_raw(200),ticks),inside);
  }
  assert!(point_in_sector(Vec2::ZERO,Vec2::ZERO,Angle::ZERO,Fixed64::ZERO,0));
  for (point,inside) in [(p(100,0),true),(p(0,100),true),(p(-1,100),false)] {
   assert_eq!(point_in_sector(point,Vec2::ZERO,Angle::from_ticks(512),Fixed64::from_raw(200),512),inside);
  }
 }
 #[test]
 fn sector_bounded_extremes_and_nearest_tick_radians_never_overflow() {
  assert_eq!(half_angle_ticks(Fixed64::from_raw(804)),Some(512));
  assert_eq!(half_angle_ticks(Fixed64::from_raw(3217)),Some(2048));
  assert_eq!(half_angle_ticks(Fixed64::ZERO),Some(0));
  for raw in [-1,3218,i64::MIN,i64::MAX] {assert_eq!(half_angle_ticks(Fixed64::from_raw(raw)),None);}
  let m=MAX_GEOMETRY_RAW;let radius=Fixed64::from_raw(m);
  assert!(point_in_sector(p(m,0),p(0,0),Angle::ZERO,radius,512));
  assert!(!point_in_sector(p(m,m),p(-m,-m),Angle::ZERO,radius,2048));
  for raw in [m+1,i64::MIN,i64::MAX] {
   assert!(!point_in_sector(p(raw,0),Vec2::ZERO,Angle::ZERO,radius,512));
   assert!(!point_in_sector(Vec2::ZERO,p(raw,0),Angle::ZERO,radius,512));
  }
  assert!(!point_in_sector(Vec2::ZERO,Vec2::ZERO,Angle::ZERO,Fixed64::from_raw(-1),512));
  assert!(!point_in_sector(Vec2::ZERO,Vec2::ZERO,Angle::ZERO,Fixed64::from_raw(m+1),512));
  assert!(!point_in_sector(Vec2::ZERO,Vec2::ZERO,Angle::ZERO,radius,2049));
 }
}
