//! Shared deterministic summon placement and cached public-terrain navigation.
use omoba_sim::{Fixed64,Vec2};
use super::comp::{BlockedRegions,SummonNavigation};
use super::tick::{hero_move_tick::path_hits_regions,hero_command_tick::static_next_waypoint};

/// Pending allocations belong to the World, across event and on_tick caches.
/// Only canonical serial allocation writes this ledger; hidden enemy occupancy
/// never influences another team's placement. Entries transfer to outcomes.
#[derive(Clone, Default)]
pub(crate) struct PlacementReservations(pub std::sync::Arc<std::sync::Mutex<
    std::collections::BTreeMap<specs::Entity,(i32,Vec2,Fixed64)>>>);
impl PlacementReservations {
    pub(crate) fn release(&self,entity:specs::Entity) {
        self.0.lock().unwrap_or_else(|poisoned|poisoned.into_inner()).remove(&entity);
    }
}

/// Fixed candidate order and bounded displacement. Friendly occupancy only;
/// undisclosed enemies must never affect a player's placement prediction.
pub(crate) fn placement(requested:Vec2,anchor:Option<Vec2>,radius:Fixed64,
    regions:&BlockedRegions,occupied:&[(Vec2,Fixed64)])->Option<Vec2> {
    let legal=|candidate:Vec2| {
        !path_hits_regions(candidate,candidate,radius,regions)
            && occupied.iter().all(|(pos,other)| {
                let sum=radius+*other;
                (candidate-*pos).length_squared()>=sum*sum
            })
            && anchor.is_none_or(|anchor|!path_hits_regions(anchor,candidate,radius,regions)
                || static_next_waypoint(anchor,candidate,radius,regions).is_some())
    };
    if legal(requested) {return Some(requested);}
    for ring in 1..=16 {
        let step=Fixed64::from_i32(ring*32);
        for (x,y) in [(1,0),(0,1),(-1,0),(0,-1),(1,1),(-1,1),(-1,-1),(1,-1)] {
            let candidate=requested+Vec2::new(step*Fixed64::from_i32(x),step*Fixed64::from_i32(y));
            if legal(candidate) {return Some(candidate);}
        }
    }
    None
}

/// Cache blocked-route searches for 250ms, replan on changed goals or arrival.
/// Every actual step still sweeps terrain, so stale cache cannot cross a wall.
pub(crate) fn advance(nav:&mut SummonNavigation,from:Vec2,target:Vec2,budget:Fixed64,
    dt:Fixed64,radius:Fixed64,regions:&BlockedRegions)->Vec2 {
    if budget<=Fixed64::ZERO {return from;}
    nav.replan_in=(nav.replan_in-dt).max(Fixed64::ZERO);
    if !path_hits_regions(from,target,radius,regions) {
        nav.destination=Some(target);nav.waypoint=Some(target);
    } else {
        let changed=nav.destination.is_none_or(|old|(old-target).length()>Fixed64::from_i32(32));
        let arrived=nav.waypoint.is_some_and(|point|(point-from).length()<=Fixed64::ONE);
        if changed || arrived || nav.replan_in<=Fixed64::ZERO {
            nav.destination=Some(target);nav.waypoint=static_next_waypoint(from,target,radius,regions);
            nav.replan_in=Fixed64::from_raw(256);
        }
    }
    let Some(waypoint)=nav.waypoint else {return from;};
    let next=omoba_sim::navigation::step_toward(from,waypoint,budget);
    if path_hits_regions(from,next,radius,regions) {from} else {next}
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::comp::BlockedRegion;
    fn wall()->BlockedRegions {BlockedRegions(vec![BlockedRegion {name:"summon-wall".into(),
        points:vec![vek::Vec2::new(80.0,-30.0),vek::Vec2::new(81.0,-30.0),
            vek::Vec2::new(81.0,30.0),vek::Vec2::new(80.0,30.0)]}])}
    #[test]
    fn bounded_placement_avoids_terrain_and_friendly_volume_and_rejects_enclosure() {
        let wall=wall();let radius=Fixed64::from_i32(20);
        let requested=Vec2::new(Fixed64::from_i32(80),Fixed64::ZERO);
        let first=placement(requested,Some(Vec2::ZERO),radius,&wall,&[]).unwrap();
        assert!(!path_hits_regions(first,first,radius,&wall));assert_ne!(first,requested);
        assert_eq!(placement(requested,Some(Vec2::ZERO),radius,&wall,&[]),Some(first));
        let second=placement(first,Some(Vec2::ZERO),radius,&wall,&[(first,radius)]).unwrap();
        assert!((second-first).length()>=radius+radius);
        let closed=BlockedRegions(vec![BlockedRegion {name:"no-room".into(),points:vec![
            vek::Vec2::new(-1000.0,-1000.0),vek::Vec2::new(1000.0,-1000.0),
            vek::Vec2::new(1000.0,1000.0),vek::Vec2::new(-1000.0,1000.0)]}]);
        assert_eq!(placement(Vec2::ZERO,None,radius,&closed,&[]),None);
    }
    #[test]
    fn cached_navigation_detours_at_60hz_and_never_crosses_a_blocked_edge() {
        let trace=|| {
            let mut nav=SummonNavigation::default();let wall=wall();let radius=Fixed64::from_i32(20);
            let mut from=Vec2::ZERO;let mut trace=Vec::new();let target=Vec2::new(Fixed64::from_i32(192),Fixed64::ZERO);
            for tick in 1..=180 {
                let dt=Fixed64::from_raw(tick*1024/60-(tick-1)*1024/60);
                let next=advance(&mut nav,from,target,Fixed64::from_i32(320)*dt,dt,radius,&wall);
                assert!(!path_hits_regions(from,next,radius,&wall));
                assert!((next-from).length()<=Fixed64::from_i32(320)*dt+Fixed64::from_raw(1));
                from=next;trace.push(next);
            }
            assert_eq!(from,target);assert!(trace.iter().any(|p|p.y!=Fixed64::ZERO));trace
        };
        assert_eq!(trace(),trace());
    }
}
