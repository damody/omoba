use specs::{Join, WriteStorage};

use crate::comp::ecs::{Job, System};
use crate::runtime::native::comp::{DemoPatrol, Pos};

#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    const NAME: &'static str = "demo_patrol";
    type SystemData = (WriteStorage<'a, Pos>, WriteStorage<'a, DemoPatrol>);

    fn run(_job: &mut Job<Self>, (mut positions, mut patrols): Self::SystemData) {
        let mut ordered: Vec<_> = (&mut positions, &mut patrols).join().collect();
        ordered.sort_by_key(|(_, patrol)| patrol.stable_index);
        for (position, patrol) in ordered {
            let target = if patrol.target_b {
                patrol.endpoint_b
            } else {
                patrol.endpoint_a
            };
            let delta = target - position.0;
            let distance_sq = delta.length_squared();
            if distance_sq <= omoba_sim::Fixed64::ZERO {
                patrol.target_b = !patrol.target_b;
                continue;
            }
            let distance = delta.length();
            if distance <= patrol.speed_per_tick {
                position.0 = target;
                patrol.target_b = !patrol.target_b;
            } else {
                position.0 = position.0 + delta * (patrol.speed_per_tick / distance);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn zero_distance_reverses_without_dividing() {
        let mut world = World::new();
        world.register::<Pos>();
        world.register::<DemoPatrol>();
        world.insert(crate::comp::SysMetrics::default());
        world.insert(crate::comp::TickProfile::default());
        let p = omoba_sim::Vec2::new(omoba_sim::Fixed64::ZERO, omoba_sim::Fixed64::ZERO);
        world
            .create_entity()
            .with(Pos(p))
            .with(DemoPatrol {
                stable_index: 1,
                endpoint_a: p,
                endpoint_b: p,
                target_b: true,
                speed_per_tick: omoba_sim::Fixed64::ONE,
            })
            .build();
        crate::comp::ecs::run_now::<Sys>(&world);
        world.maintain();
        assert!(
            !world
                .read_storage::<DemoPatrol>()
                .join()
                .next()
                .unwrap()
                .target_b
        );
    }
}
