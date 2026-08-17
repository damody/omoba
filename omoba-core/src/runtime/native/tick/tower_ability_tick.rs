use omoba_sim::Fixed64;
use specs::{Join, World, WorldExt};

use crate::runtime::comp::{PendingTowerAbilityPulse, PendingTowerAbilityPulseQueue, Tower};

/// Advances tower active-ability state once and records pulse opportunities.
/// Script dispatch and acknowledgement intentionally happen at a later runtime
/// boundary so this scheduler remains deterministic and side-effect free.
pub fn tick_tower_abilities(world: &mut World, dt: Fixed64) {
    let entities = world.entities();
    let mut towers = world.write_storage::<Tower>();
    let mut queue = world.write_resource::<PendingTowerAbilityPulseQueue>();

    for (entity, tower) in (&entities, &mut towers).join() {
        let Some(state) = tower.active_ability.as_mut() else {
            continue;
        };
        let opportunity = state.advance(dt);
        if opportunity.pulse_due {
            queue.requests.push(PendingTowerAbilityPulse {
                entity,
                ability_id: state.ability_id.clone(),
                activation_serial: state.activation_serial,
                pulse_index: opportunity.pulse_index,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use omoba_sim::Fixed64;
    use specs::{Builder, World, WorldExt};

    use crate::runtime::comp::{PendingTowerAbilityPulseQueue, Tower, TowerActiveAbilityState};
    use crate::runtime::SimulationTickProfile;

    use super::tick_tower_abilities;

    fn pulse_and_cooldown_for_profile(
        profile: SimulationTickProfile,
    ) -> (Vec<u16>, Fixed64, Fixed64, u16) {
        let mut state = TowerActiveAbilityState::ready("arty_fire_at_will");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(3),
                Fixed64::from_raw(512),
                6,
            )
            .unwrap();
        let mut pulses = Vec::new();
        for tick in 1..=u64::from(profile.ticks_per_game_second()) * 10 {
            let opportunity = state.advance(Fixed64::from_raw(profile.fixed_raw_for_tick(tick)));
            if opportunity.pulse_due {
                pulses.push(opportunity.pulse_index);
                state.acknowledge_pulse(true);
            }
        }
        (
            pulses,
            state.cooldown_remaining,
            state.active_remaining,
            state.pulses_remaining,
        )
    }

    #[test]
    fn new_state_is_ready() {
        let state = TowerActiveAbilityState::ready("arty_fire_at_will");

        assert_eq!(state.ability_id, "arty_fire_at_will");
        assert_eq!(state.cooldown_remaining, Fixed64::ZERO);
        assert_eq!(state.active_remaining, Fixed64::ZERO);
        assert_eq!(state.pulses_remaining, 0);
    }

    #[test]
    fn activation_starts_window_pulses_and_cooldown() {
        let mut state = TowerActiveAbilityState::ready("arty_fire_at_will");

        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(3),
                Fixed64::from_raw(512),
                6,
            )
            .unwrap();

        assert_eq!(state.cooldown_remaining, Fixed64::from_i32(10));
        assert_eq!(state.active_remaining, Fixed64::from_i32(3));
        assert_eq!(state.pulse_interval, Fixed64::from_raw(512));
        assert_eq!(state.pulses_remaining, 6);
        assert_eq!(state.activation_serial, 1);
    }

    #[test]
    fn accepted_activations_advance_state_owned_serial() {
        let mut state = TowerActiveAbilityState::ready("boomerang_turbo_charge");

        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(2),
                Fixed64::ZERO,
                0,
            )
            .unwrap();
        assert_eq!(state.activation_serial, 1);

        state.advance(Fixed64::from_i32(10));
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(2),
                Fixed64::ZERO,
                0,
            )
            .unwrap();
        assert_eq!(state.activation_serial, 2);
    }

    #[test]
    fn activation_serial_wrap_skips_reserved_zero() {
        let mut state = TowerActiveAbilityState::ready("boomerang_turbo_charge");
        state.activation_serial = u32::MAX;

        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(2),
                Fixed64::ZERO,
                0,
            )
            .unwrap();

        assert_eq!(state.activation_serial, 1);
    }

    #[test]
    fn duplicate_activation_is_rejected_without_mutation() {
        let mut state = TowerActiveAbilityState::ready("arty_fire_at_will");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(3),
                Fixed64::from_raw(512),
                6,
            )
            .unwrap();
        let before = state.clone();

        assert!(state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(3),
                Fixed64::from_raw(512),
                6,
            )
            .is_err());
        assert_eq!(state.cooldown_remaining, before.cooldown_remaining);
        assert_eq!(state.active_remaining, before.active_remaining);
        assert_eq!(state.pulses_remaining, before.pulses_remaining);
        assert_eq!(state.activation_serial, before.activation_serial);
    }

    #[test]
    fn zero_dt_pauses_every_timer() {
        let mut state = TowerActiveAbilityState::ready("arty_fire_at_will");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(3),
                Fixed64::from_raw(512),
                6,
            )
            .unwrap();
        let before = state.clone();

        let opportunity = state.advance(Fixed64::ZERO);

        assert!(!opportunity.pulse_due);
        assert_eq!(state.cooldown_remaining, before.cooldown_remaining);
        assert_eq!(state.active_remaining, before.active_remaining);
        assert_eq!(state.pulse_accumulator, before.pulse_accumulator);
    }

    #[test]
    fn scheduler_emits_each_pulse_once_across_tick_boundaries() {
        let mut state = TowerActiveAbilityState::ready("arty_fire_at_will");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(3),
                Fixed64::from_raw(512),
                6,
            )
            .unwrap();
        let mut total = 0;
        let mut indices = Vec::new();
        for _ in 0..30 {
            let opportunity = state.advance(Fixed64::from_raw(103));
            if opportunity.pulse_due {
                indices.push(opportunity.pulse_index);
                state.acknowledge_pulse(true);
                total += 1;
            }
        }
        assert_eq!(total, 6);
        assert_eq!(indices, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(state.pulses_remaining, 0);
    }

    #[test]
    fn oversized_tick_quantizes_and_drains_all_pulses_after_window_expiry() {
        let mut state = TowerActiveAbilityState::ready("arty_fire_at_will");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(3),
                Fixed64::from_raw(512),
                6,
            )
            .unwrap();

        let first = state.advance(Fixed64::from_i32(3));
        assert!(first.pulse_due);
        assert_eq!(first.pulse_index, 0);
        assert_eq!(state.active_remaining, Fixed64::ZERO);
        assert_eq!(state.pending_due, 6);

        let mut indices = vec![first.pulse_index];
        state.acknowledge_pulse(true);
        for _ in 1..6 {
            let opportunity = state.advance(Fixed64::from_raw(1));
            assert!(opportunity.pulse_due);
            indices.push(opportunity.pulse_index);
            state.acknowledge_pulse(true);
        }

        assert_eq!(indices, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(state.pending_due, 0);
        assert_eq!(state.pulses_remaining, 0);
        assert!(!state.advance(Fixed64::from_raw(1)).pulse_due);
    }

    #[test]
    fn one_advance_quantizes_multiple_crossed_intervals_but_emits_once() {
        let mut state = TowerActiveAbilityState::ready("arty_fire_at_will");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(3),
                Fixed64::from_raw(512),
                6,
            )
            .unwrap();

        let first = state.advance(Fixed64::from_raw(1300));

        assert!(first.pulse_due);
        assert_eq!(state.pending_due, 2);
        assert!(state.opportunity_outstanding);
        assert!(!state.advance(Fixed64::from_raw(1)).pulse_due);
        assert_eq!(state.pending_due, 2);
    }

    #[test]
    fn serde_round_trip_reemits_transient_outstanding_opportunity() {
        let mut state = TowerActiveAbilityState::ready("arty_fire_at_will");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(3),
                Fixed64::from_raw(512),
                6,
            )
            .unwrap();
        assert!(state.advance(Fixed64::from_i32(1)).pulse_due);
        assert_eq!(state.pending_due, 2);
        assert!(state.opportunity_outstanding);

        let encoded = serde_json::to_string(&state).unwrap();
        assert!(!encoded.contains("opportunity_outstanding"));
        let mut restored: TowerActiveAbilityState = serde_json::from_str(&encoded).unwrap();

        assert!(!restored.opportunity_outstanding);
        assert_eq!(restored.pending_due, 2);
        let recovered = restored.advance(Fixed64::from_raw(1));
        assert!(recovered.pulse_due);
        assert_eq!(recovered.pulse_index, 0);
        restored.acknowledge_pulse(true);
        assert_eq!(restored.pending_due, 1);
        assert_eq!(restored.pulses_remaining, 5);
    }

    #[test]
    fn older_serialized_state_defaults_new_backlog_fields() {
        let state = TowerActiveAbilityState::ready("arty_fire_at_will");
        let mut encoded = serde_json::to_value(state).unwrap();
        let object = encoded.as_object_mut().unwrap();
        object.remove("pending_due");
        object.remove("opportunity_outstanding");

        let restored: TowerActiveAbilityState = serde_json::from_value(encoded).unwrap();

        assert_eq!(restored.pending_due, 0);
        assert!(!restored.opportunity_outstanding);
    }

    #[test]
    fn negative_ack_preserves_charge_until_next_interval() {
        let mut state = TowerActiveAbilityState::ready("arty_fire_at_will");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(3),
                Fixed64::from_raw(512),
                6,
            )
            .unwrap();

        let first = state.advance(Fixed64::from_raw(512));
        assert!(first.pulse_due);
        assert_eq!(first.pulse_index, 0);
        state.acknowledge_pulse(false);
        assert_eq!(state.pulses_remaining, 6);
        assert!(!state.advance(Fixed64::from_raw(511)).pulse_due);
        let retry = state.advance(Fixed64::from_raw(1));
        assert!(retry.pulse_due);
        assert_eq!(retry.pulse_index, 0);
    }

    #[test]
    fn negative_ack_consumes_due_attempt_and_expires_unused_charge_after_backlog() {
        let mut state = TowerActiveAbilityState::ready("arty_fire_at_will");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(1),
                Fixed64::from_raw(512),
                2,
            )
            .unwrap();

        let first = state.advance(Fixed64::from_i32(1));
        assert!(first.pulse_due);
        assert_eq!(state.pending_due, 2);
        state.acknowledge_pulse(false);
        assert_eq!(state.pending_due, 1);
        assert_eq!(state.pulses_remaining, 2);

        let retry = state.advance(Fixed64::from_raw(1));
        assert!(retry.pulse_due);
        assert_eq!(retry.pulse_index, 0);
        state.acknowledge_pulse(true);

        assert_eq!(state.pending_due, 0);
        assert_eq!(state.pulses_remaining, 0);
        assert!(!state.advance(Fixed64::from_raw(1)).pulse_due);
    }

    #[test]
    fn window_expiry_without_due_intervals_expires_unused_charges() {
        let mut state = TowerActiveAbilityState::ready("arty_fire_at_will");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_raw(256),
                Fixed64::from_raw(512),
                6,
            )
            .unwrap();

        assert!(!state.advance(Fixed64::from_raw(256)).pulse_due);
        assert_eq!(state.active_remaining, Fixed64::ZERO);
        assert_eq!(state.pending_due, 0);
        assert_eq!(state.pulses_remaining, 0);
    }

    #[test]
    fn active_window_expires_with_saturating_time() {
        let mut state = TowerActiveAbilityState::ready("boomerang_turbo_charge");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(2),
                Fixed64::ZERO,
                0,
            )
            .unwrap();

        state.advance(Fixed64::from_i32(3));

        assert_eq!(state.active_remaining, Fixed64::ZERO);
        assert_eq!(state.pulses_remaining, 0);
    }

    #[test]
    fn cooldown_reaches_zero_without_underflow() {
        let mut state = TowerActiveAbilityState::ready("boomerang_turbo_charge");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(2),
                Fixed64::ZERO,
                0,
            )
            .unwrap();

        state.advance(Fixed64::from_i32(11));

        assert_eq!(state.cooldown_remaining, Fixed64::ZERO);
    }

    #[test]
    fn multi_pulse_and_cooldown_match_at_fifteen_and_one_twenty_hz() {
        let coarse = pulse_and_cooldown_for_profile(SimulationTickProfile::Coarse15Hz);
        let fine = pulse_and_cooldown_for_profile(SimulationTickProfile::Production120Hz);

        assert_eq!(coarse, fine);
        assert_eq!(coarse.0, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(coarse.1, Fixed64::ZERO);
        assert_eq!(coarse.2, Fixed64::ZERO);
        assert_eq!(coarse.3, 0);
    }

    #[test]
    fn ecs_tick_enqueues_deterministic_pulse_record() {
        let mut world = World::new();
        world.register::<Tower>();
        world.insert(PendingTowerAbilityPulseQueue::default());
        let mut tower = Tower::new();
        let mut state = TowerActiveAbilityState::ready("arty_fire_at_will");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(3),
                Fixed64::from_raw(512),
                6,
            )
            .unwrap();
        tower.active_ability = Some(state);
        let entity = world.create_entity().with(tower).build();

        tick_tower_abilities(&mut world, Fixed64::from_raw(512));

        let queue = world.read_resource::<PendingTowerAbilityPulseQueue>();
        assert_eq!(queue.requests.len(), 1);
        let pulse = &queue.requests[0];
        assert_eq!(pulse.entity, entity);
        assert_eq!(pulse.ability_id, "arty_fire_at_will");
        assert_eq!(pulse.activation_serial, 1);
        assert_eq!(pulse.pulse_index, 0);
    }
}
