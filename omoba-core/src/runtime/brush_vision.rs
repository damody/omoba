//! Passable concealment, independent of opaque line-of-sight geometry.
use omoba_sim::{Fixed64, Vec2};
use std::sync::Arc;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BrushVisionRegions(Arc<BrushTopology>);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct BrushTopology {
    regions: Vec<BrushVisionRegion>,
    components: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BrushVisionRegion { pub min: Vec2, pub max: Vec2 }

impl BrushVisionRegion {
    pub fn contains(&self, p: Vec2) -> bool {
        if p.x < self.min.x || p.x > self.max.x || p.y < self.min.y || p.y > self.max.y { return false; }
        // Match the public ground renderer's capsule exactly. Doubled raw
        // coordinates preserve half-quantum centers without floating point.
        let width=i128::from(self.max.x.raw())-i128::from(self.min.x.raw());
        let height=i128::from(self.max.y.raw())-i128::from(self.min.y.raw());
        if width<=0 || height<=0 { return false; }
        let diameter=width.min(height);
        let mut dx=(2*i128::from(p.x.raw())-i128::from(self.min.x.raw())-i128::from(self.max.x.raw())).abs();
        let mut dy=(2*i128::from(p.y.raw())-i128::from(self.min.y.raw())-i128::from(self.max.y.raw())).abs();
        if width>=height { dx=(dx-(width-diameter)).max(0); }
        else { dy=(dy-(height-diameter)).max(0); }
        let (dx,dy,radius)=(dx as u128,dy as u128,diameter as u128);
        dx.checked_mul(dx).and_then(|x|dy.checked_mul(dy).and_then(|y|x.checked_add(y)))
            .is_some_and(|distance|distance<=radius*radius)
    }
}

impl BrushVisionRegions {
    pub fn regions(&self) -> &[BrushVisionRegion] { &self.0.regions }

    pub fn from_map(map: &omoba_template_ids::MobaMapConst) -> Self {
        let p = |(x,y)| Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        map.brushes.iter().map(|b| BrushVisionRegion {min:p(b.min),max:p(b.max)}).collect::<Vec<_>>().into()
    }
}

impl From<Vec<BrushVisionRegion>> for BrushVisionRegions {
    fn from(regions: Vec<BrushVisionRegion>) -> Self {
        let mut components: Vec<_> = (0..regions.len()).collect();
        for i in 0..regions.len() {
            for k in 0..i {
                if capsules_intersect(regions[i], regions[k]) {
                    let old = components[i];
                    let new = components[k];
                    for component in &mut components {
                        if *component == old { *component = new; }
                    }
                }
            }
        }
        Self(Arc::new(BrushTopology { regions, components }))
    }
}

// Capsule center segments are axis aligned. Doubled fixed-point endpoints
// preserve half-quantum centers; no floating point or bounding-box proxy.
fn capsule_segment(region: BrushVisionRegion) -> Option<([i128; 4], u128)> {
    let x1=i128::from(region.min.x.raw()); let y1=i128::from(region.min.y.raw());
    let x2=i128::from(region.max.x.raw()); let y2=i128::from(region.max.y.raw());
    let (width,height)=(x2-x1,y2-y1);
    if width<=0 || height<=0 { return None; }
    let diameter=width.min(height);
    Some((if width>=height { [2*x1+diameter,y1+y2,2*x2-diameter,y1+y2] }
        else { [x1+x2,2*y1+diameter,x1+x2,2*y2-diameter] },diameter as u128))
}

// Inputs fit in 65 bits even for the full i64 raw coordinate domain. Keep
// the high limb so squaring and adding never overflow at extreme coordinates.
fn square(value: u128) -> (u128,u128) {
    let (hi,lo)=(value>>64,value & u128::from(u64::MAX));
    let cross=hi*lo;
    let (low,carry)=(lo*lo).overflowing_add(cross<<65);
    (hi*hi+(cross>>63)+u128::from(carry),low)
}

fn capsules_intersect(a: BrushVisionRegion,b: BrushVisionRegion) -> bool {
    let (Some((a,ar)),Some((b,br)))=(capsule_segment(a),capsule_segment(b)) else { return false; };
    let gap=|amin:i128,amax:i128,bmin:i128,bmax:i128| (amin-bmax).max(bmin-amax).max(0) as u128;
    let (dx,dy)=(gap(a[0],a[2],b[0],b[2]),gap(a[1],a[3],b[1],b[3]));
    let radius=ar+br;
    if dx>radius || dy>radius { return false; }
    let (xh,xl)=square(dx); let (yh,yl)=square(dy);
    let (low,carry)=xl.overflowing_add(yl);
    (xh+yh+u128::from(carry),low)<=square(radius)
}

/// Outside targets are never occluded by brush in between. Range, detection
/// and wall LOS remain separate requirements, even for a source inside brush.
pub fn brush_allows_vision(brushes: &BrushVisionRegions, source: Vec2, target: Vec2) -> bool {
    let topology=&brushes.0;
    topology.regions.iter().position(|region|region.contains(target)).is_none_or(|target_index|
        topology.regions.iter().enumerate().any(|(index,region)|
            topology.components[index]==topology.components[target_index] && region.contains(source)))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(x:i32,y:i32)->Vec2 { Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y)) }
    #[test]
    fn concealment_is_not_an_opaque_wall_and_boundaries_are_closed() {
        let patches=vec![BrushVisionRegion {min:p(10,10),max:p(20,20)},BrushVisionRegion {min:p(30,10),max:p(40,20)}].into();
        assert!(!brush_allows_vision(&patches,p(0,15),p(15,15)));
        assert!(brush_allows_vision(&patches,p(10,15),p(20,15)));
        assert!(brush_allows_vision(&patches,p(0,0),p(10,10)),"rounded exterior corner is outside brush");
        assert!(!brush_allows_vision(&patches,p(15,15),p(35,15)));
        assert!(brush_allows_vision(&patches,p(0,15),p(50,15)));
        assert!(brush_allows_vision(&patches,p(15,15),p(50,15)));
        assert!(brush_allows_vision(&BrushVisionRegions::default(),p(0,15),p(15,15)));
    }

    #[test]
    fn capsule_matches_public_renderer_on_both_axes_caps_and_half_quantum_centers() {
        let horizontal=BrushVisionRegion {min:p(0,0),max:p(40,20)};
        for (point,inside) in [(p(0,10),true),(p(40,10),true),(p(10,0),true),(p(20,0),true),
            (p(0,0),false),(p(40,20),false),(p(-1,10),false),(p(5,5),true)] {
            assert_eq!(horizontal.contains(point),inside);
            let vertical=BrushVisionRegion {min:p(0,0),max:p(20,40)};
            assert_eq!(vertical.contains(Vec2::new(point.y,point.x)),inside);
        }
        let odd=BrushVisionRegion {min:Vec2::new(Fixed64::from_raw(0),Fixed64::from_raw(0)),
            max:Vec2::new(Fixed64::from_raw(5),Fixed64::from_raw(3))};
        assert!(!odd.contains(odd.min));
        assert!(odd.contains(Vec2::new(Fixed64::from_raw(2),Fixed64::from_raw(1))));
        let extreme=Vec2::new(Fixed64::from_raw(i64::MAX),Fixed64::from_raw(i64::MIN));
        assert!(!horizontal.contains(extreme));
    }

    #[test]
    fn connected_capsules_are_transitive_order_independent_and_symmetric() {
        let parts=vec![
            BrushVisionRegion {min:p(0,0),max:p(40,20)},
            BrushVisionRegion {min:p(30,0),max:p(50,60)},
            BrushVisionRegion {min:p(40,40),max:p(90,60)},
        ];
        for order in [parts.clone(),vec![parts[2],parts[0],parts[1]],vec![parts[1],parts[2],parts[0]]] {
            let regions=order.into();
            assert!(brush_allows_vision(&regions,p(10,10),p(80,50)));
            assert!(brush_allows_vision(&regions,p(80,50),p(10,10)));
            assert!(!brush_allows_vision(&regions,p(-1,10),p(80,50)));
            assert!(brush_allows_vision(&regions,p(-1,10),p(100,50)));
        }
    }

    #[test]
    fn rounded_corner_overlap_is_not_connectivity_but_tangency_is() {
        let a=BrushVisionRegion {min:p(0,0),max:p(20,20)};
        let corner=BrushVisionRegion {min:p(18,18),max:p(38,38)};
        assert!(!capsules_intersect(a,corner));
        let disjoint=vec![a,corner].into();
        assert!(!brush_allows_vision(&disjoint,p(10,10),p(28,28)));
        let tangent=BrushVisionRegion {min:p(20,0),max:p(40,20)};
        assert!(capsules_intersect(a,tangent));
        assert!(brush_allows_vision(&vec![a,tangent].into(),p(10,10),p(30,10)));
        let invalid=BrushVisionRegion {min:p(0,0),max:p(0,10)};
        assert!(!capsules_intersect(a,invalid));
    }

    #[test]
    fn connectivity_handles_full_raw_domain_and_half_quantum_geometry() {
        let raw=|x,y|Vec2::new(Fixed64::from_raw(x),Fixed64::from_raw(y));
        let a=BrushVisionRegion {min:raw(i64::MIN,i64::MIN),max:raw(i64::MAX,i64::MAX)};
        let corner=BrushVisionRegion {min:raw(i64::MAX-4,i64::MAX-4),max:raw(i64::MAX,i64::MAX)};
        assert!(capsules_intersect(a,a));
        assert!(!capsules_intersect(a,corner));
        let odd=BrushVisionRegion {min:raw(0,0),max:raw(5,3)};
        let tangent=BrushVisionRegion {min:raw(5,0),max:raw(10,3)};
        let separated=BrushVisionRegion {min:raw(6,0),max:raw(11,3)};
        assert!(capsules_intersect(odd,tangent));
        assert!(!capsules_intersect(odd,separated));
    }

    #[test]
    fn compiled_maps_publish_connected_branches_with_matching_capsule_bounds() {
        for map in omoba_template_ids::MOBA_MAPS {
            let brushes=BrushVisionRegions::from_map(map);
            let branch=map.brushes.iter().position(|b|b.id=="top_lane_brush_branch").unwrap();
            let trunk=map.brushes.iter().position(|b|b.id=="top_lane_brush").unwrap();
            assert_eq!(brushes.0.components[branch],brushes.0.components[trunk]);
            assert!(brush_allows_vision(&brushes,p(280,2500),p(600,2980)));
            assert!(!brush_allows_vision(&brushes,p(4500,1800),p(600,2980)));
        }
    }
}
