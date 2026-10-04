//! Bounded fixed-point waypoint following, independent of renderers and ECS.
//! This is authored lane navigation, not obstacle avoidance or a navmesh.
use crate::{Fixed64, Vec2};

/// Spend a nonnegative movement budget without overshooting a waypoint.
pub fn step_toward(origin: Vec2, target: Vec2, budget: Fixed64) -> Vec2 {
    if budget <= Fixed64::ZERO { return origin; }
    let diff = target - origin;
    let distance = diff.length();
    if distance <= budget { return target; }
    let interpolate = |v: Fixed64| Fixed64::from_raw(
        (i128::from(v.raw()) * i128::from(budget.raw()) / i128::from(distance.raw())) as i64);
    let mut delta = Vec2::new(interpolate(diff.x), interpolate(diff.y));
    // With a one-raw-unit budget both diagonal components can truncate to zero.
    // Advance one dominant-axis quantum instead of stalling forever.
    if delta == Vec2::ZERO {
        if diff.x.raw().abs() >= diff.y.raw().abs() {
            delta.x = Fixed64::from_raw(diff.x.raw().signum());
        } else { delta.y = Fixed64::from_raw(diff.y.raw().signum()); }
    }
    origin + delta
}

/// Cursor is the next waypoint, or route.len() when complete. Duplicate points
/// are safe; each call visits at most route.len() nodes. Remaining budget carries
/// across corners, so high speeds cannot skip directly through the route.
pub fn advance_route(route: &[Vec2], cursor: &mut usize, mut origin: Vec2, mut budget: Fixed64) -> Vec2 {
    if budget < Fixed64::ZERO { return origin; }
    while let Some(&target) = route.get(*cursor) {
        let distance = (target - origin).length();
        if distance > budget { return step_toward(origin, target, budget); }
        origin = target;
        budget -= distance;
        *cursor += 1;
    }
    // Combat can pull a completed follower off the endpoint. Keep returning to
    // that endpoint after aggro ends, rather than stranding it at the chase site.
    route.last().map_or(origin, |&target| step_toward(origin,target,budget))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn point(x: i32, y: i32) -> Vec2 { Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)) }

    #[test]
    fn corners_consume_budget_without_shortcuts() {
        let route = [point(0,0),point(0,10),point(10,10)];
        let mut cursor = 1;
        assert_eq!(advance_route(&route,&mut cursor,route[0],Fixed64::from_i32(15)),point(5,10));
        assert_eq!(cursor,2);
        assert_eq!(advance_route(&route,&mut cursor,point(5,10),Fixed64::from_i32(100)),point(10,10));
        assert_eq!(cursor,route.len());
        assert_eq!(advance_route(&route,&mut cursor,point(12,10),Fixed64::from_i32(2)),point(10,10));
    }

    #[test]
    fn zero_negative_duplicate_and_finished_routes_are_bounded() {
        let route = [point(0,0),point(0,0),point(10,0)];
        let mut cursor = 1;
        assert_eq!(advance_route(&route,&mut cursor,route[0],-Fixed64::ONE),route[0]);
        assert_eq!(cursor,1);
        assert_eq!(advance_route(&route,&mut cursor,route[0],Fixed64::ZERO),route[0]);
        assert_eq!(cursor,2);
        assert_eq!(advance_route(&[],&mut cursor,point(3,4),Fixed64::ONE),point(3,4));
    }

    #[test]
    fn subquantum_diagonal_does_not_stall_or_overshoot() {
        for target in [Vec2::new(Fixed64::from_raw(2),Fixed64::from_raw(2)),
            Vec2::new(Fixed64::from_raw(-2),Fixed64::from_raw(-2))] {
            let mut current = Vec2::ZERO;
            for _ in 0..4 { current = step_toward(current,target,Fixed64::from_raw(1)); }
            assert_eq!(current,target);
        }
    }

    #[test]
    fn authored_routes_reach_both_bases_at_60hz() {
        let route = [point(0,0),point(400,1400),point(2000,1400),point(2400,0)];
        for route in [route.to_vec(),route.into_iter().rev().collect()] {
            let mut cursor = 1;
            let mut current = route[0];
            let mut previous = current;
            for _ in 0..1500 {
                current = advance_route(&route,&mut cursor,current,Fixed64::from_i32(240)*Fixed64::from_raw(17));
                assert!(current != previous || cursor == route.len(),"unexpected stuck waypoint");
                previous = current;
            }
            assert_eq!(cursor,route.len());
            assert_eq!(current,*route.last().unwrap());
        }
    }
}
