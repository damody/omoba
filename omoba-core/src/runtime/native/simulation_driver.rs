//! Shared, uncapped deterministic simulation driver.
//!
//! Both profiles execute the same authoritative ECS phases. The coarse profile
//! changes only elapsed simulation time per tick; it never skips systems and it
//! never sleeps. Consequently 240 ticks/wall-second is an informational
//! throughput measurement, not a pass condition or a 240 Hz simulation clock.

use failure::{err_msg, Error};
use specs::{Dispatcher, World, WorldExt};

use crate::comp::{DeltaTime, GamePause, GameSpeed, PendingPlayerInputs, Tick, Time};
use crate::lockstep_timing::fixed_raw_for_tick_at_fps;
use crate::runtime::{
    build_phase3_dispatcher, drain_pending_ability_casts, drain_pending_ability_upgrades,
    drain_pending_hero_command_clears, drain_pending_item_uses, drain_pending_moves,
    drain_pending_tower_ability_callbacks, drain_pending_tower_ability_casts,
    drain_pending_tower_sells, drain_pending_tower_spawns, drain_pending_tower_target_priorities,
    drain_pending_tower_upgrades, process_outcomes, run_script_dispatch, tick_tower_abilities,
    PlayerInput, RuntimeEventVecSink, RuntimeEvents, ScriptRegistry,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimulationTickProfile {
    Production120Hz,
    Coarse15Hz,
}

impl SimulationTickProfile {
    pub const fn ticks_per_game_second(self) -> u32 {
        match self {
            Self::Production120Hz => 120,
            Self::Coarse15Hz => 15,
        }
    }

    pub fn seconds_per_tick(self) -> f64 {
        1.0 / f64::from(self.ticks_per_game_second())
    }

    pub fn fixed_raw_for_tick(self, tick: u64) -> i64 {
        fixed_raw_for_tick_at_fps(tick, u64::from(self.ticks_per_game_second()))
    }
}

#[derive(Debug, Default)]
pub struct SimulationTickResult {
    pub tick: u64,
    pub events: RuntimeEvents,
}

pub struct SimulationDriver {
    profile: SimulationTickProfile,
    tick: u64,
    dispatcher: Dispatcher<'static, 'static>,
    scripts: ScriptRegistry,
}

impl SimulationDriver {
    /// Takes ownership of the runtime script registry installed by world
    /// initialization. Keeping it beside the dispatcher avoids removing and
    /// reinserting a resource on every tick.
    pub fn from_world(world: &mut World, profile: SimulationTickProfile) -> Result<Self, Error> {
        let scripts = world
            .remove::<ScriptRegistry>()
            .ok_or_else(|| err_msg("simulation world is missing ScriptRegistry"))?;
        Ok(Self {
            profile,
            tick: world.read_resource::<Tick>().0,
            dispatcher: build_phase3_dispatcher()?,
            scripts,
        })
    }

    pub fn profile(&self) -> SimulationTickProfile {
        self.profile
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Advances exactly one fixed simulation tick without sleeping.
    pub fn step(
        &mut self,
        world: &mut World,
        inputs: impl IntoIterator<Item = (u32, PlayerInput)>,
    ) -> Result<SimulationTickResult, Error> {
        self.tick = self.tick.saturating_add(1);
        {
            let mut pending = world.write_resource::<PendingPlayerInputs>();
            pending.tick = u32::try_from(self.tick)
                .map_err(|_| err_msg("simulation tick exceeds PlayerInput u32 range"))?;
            pending.inputs.extend(inputs);
        }
        world.write_resource::<Tick>().0 = self.tick;

        let paused = world.read_resource::<GamePause>().is_paused;
        if paused {
            world.write_resource::<DeltaTime>().0 = omoba_sim::Fixed64::ZERO;
        } else {
            let speed = world.read_resource::<GameSpeed>().multiplier();
            world.write_resource::<Time>().0 += self.profile.seconds_per_tick() * f64::from(speed);
            world.write_resource::<DeltaTime>().0 = omoba_sim::Fixed64::from_raw(
                self.profile
                    .fixed_raw_for_tick(self.tick)
                    .saturating_mul(i64::from(speed)),
            );
        }

        self.dispatcher.dispatch(world);
        world.maintain();

        // RuntimeEvents is the intra-tick system channel. Outcomes below are
        // returned through the same result in their exact phase order.
        let mut events = std::mem::take(&mut *world.write_resource::<RuntimeEvents>());

        drain_pending_hero_command_clears(world);
        world.maintain();
        drain_pending_tower_spawns(world);
        world.maintain();
        drain_pending_tower_sells(world);
        world.maintain();
        drain_pending_tower_target_priorities(world);
        world.maintain();
        drain_pending_item_uses(world);
        world.maintain();
        drain_pending_ability_upgrades(world);
        world.maintain();
        drain_pending_ability_casts(world);
        world.maintain();
        drain_pending_moves(world);
        world.maintain();

        let mut sink = RuntimeEventVecSink::default();
        process_outcomes(world, &mut sink)?;
        world.maintain();
        events.append(&mut sink.events);

        drain_pending_tower_upgrades(world);
        drain_pending_tower_ability_casts(world);
        let scaled_dt = world.read_resource::<DeltaTime>().0;
        tick_tower_abilities(world, scaled_dt);
        drain_pending_tower_ability_callbacks(world, &self.scripts, self.tick);
        run_script_dispatch(world, &self.scripts, self.tick, scaled_dt);

        let mut sink = RuntimeEventVecSink::default();
        process_outcomes(world, &mut sink)?;
        world.maintain();
        events.append(&mut sink.events);

        Ok(SimulationTickResult {
            tick: self.tick,
            events,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_profiles_sum_to_exactly_one_game_second() {
        for profile in [
            SimulationTickProfile::Production120Hz,
            SimulationTickProfile::Coarse15Hz,
        ] {
            let total: i64 = (1..=u64::from(profile.ticks_per_game_second()))
                .map(|tick| profile.fixed_raw_for_tick(tick))
                .sum();
            assert_eq!(total, crate::lockstep_timing::LOCKSTEP_FIXED_SCALE);
        }
    }

    #[test]
    fn coarse_profile_is_fifteen_hz_not_two_hundred_forty_hz() {
        let profile = SimulationTickProfile::Coarse15Hz;
        assert_eq!(profile.ticks_per_game_second(), 15);
        assert!((profile.seconds_per_tick() - 1.0 / 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn elapsed_occurrences_match_across_profiles_with_identical_remainders() {
        fn drain_one_second(profile: SimulationTickProfile, interval_raw: i64) -> (u32, i64) {
            let mut elapsed = 0i64;
            let mut count = 0u32;
            for tick in 1..=u64::from(profile.ticks_per_game_second()) {
                elapsed += profile.fixed_raw_for_tick(tick);
                while elapsed >= interval_raw {
                    elapsed -= interval_raw;
                    count += 1;
                    assert!(count <= 1_024, "occurrence guard");
                }
            }
            (count, elapsed)
        }

        // Representative attack, DoT, pulse/cooldown and spawn intervals.
        for interval_raw in [51i64, 128, 205, 341] {
            assert_eq!(
                drain_one_second(SimulationTickProfile::Coarse15Hz, interval_raw),
                drain_one_second(SimulationTickProfile::Production120Hz, interval_raw),
                "interval_raw={interval_raw}"
            );
        }
    }
}
