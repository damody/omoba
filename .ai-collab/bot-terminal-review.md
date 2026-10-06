# Bounded Grok review: forward lane progress

No tools, no edits, no commands, no credentials, no commits/push/reset/clean/restore or external actions. Analyze only this supplied source. Return concrete correctness findings and a minimal generic algorithm, not a forced winner or game-balance change. Do not claim tests ran.

Current real same-world 60Hz match has reached 22 minutes, still Playing. This alone does not prove a bot defect. Planner uses only committed team disclosures, no hidden enemy HP/aggro. Existing route function is:

```rust
fn route_destination(route: &[Vec2], own: Vec2, active_destination: Option<Vec2>) -> Option<Vec2> {
    let last = *route.last()?;
    let arrival = Fixed64::from_i32(50);
    let index = active_destination.and_then(|point| route.iter().position(|p| *p == point))
        .or_else(|| route.windows(2).enumerate().map(|(index, segment)| {
            let delta = segment[1]-segment[0];
            let length = delta.length_squared();
            let fraction = if length>Fixed64::ZERO {
                ((own-segment[0]).dot(delta)/length).max(Fixed64::ZERO).min(Fixed64::ONE)
            } else {Fixed64::ZERO};
            let closest = segment[0]+delta*fraction;
            ((own-closest).length_squared().raw(), index+1)
        }).min().map(|(_,index)|index)).unwrap_or(0);
    route[index..].iter().copied().find(|point|(*point-own).length_squared()>arrival*arrival)
        .or(Some(last))
}
```

Does stateless nearest segment selection cause backwards progress after combat interrupts at a waypoint or exact crossing? Explain provable cases versus speculative match cause. Do not introduce a persistent cursor unless necessary. Keep active command preservation, deterministic tie handling, arbitrary route geometry, empty/single/coincident routes, public terrain constraints. Exact tests should be pure geometry, no simulated matches. Return at most 500 words; host independently implements and checks.
