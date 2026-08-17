use omoba_sim::Fixed64;

pub const DEFAULT_ATTACK_WINDUP_WEIGHT: u16 = 350;
pub const DEFAULT_ATTACK_BACKSWING_WEIGHT: u16 = 650;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackPhaseStep {
    Charging,
    Ready,
    Impact,
}

pub fn attack_phase_durations(interval: Fixed64) -> (Fixed64, Fixed64) {
    let windup =
        interval * Fixed64::from_i32(DEFAULT_ATTACK_WINDUP_WEIGHT as i32) / Fixed64::from_i32(1000);
    let backswing = interval - windup;
    (windup, backswing)
}

pub fn advance_attack_phase(
    asd_count: &mut Fixed64,
    dt: Fixed64,
    interval: Fixed64,
) -> AttackPhaseStep {
    let (windup, _) = attack_phase_durations(interval);
    if *asd_count < Fixed64::ZERO {
        *asd_count += dt;
        if *asd_count < Fixed64::ZERO {
            return AttackPhaseStep::Charging;
        }
        *asd_count = windup + *asd_count;
        return AttackPhaseStep::Impact;
    }

    if *asd_count < interval {
        *asd_count += dt;
    }
    if *asd_count >= interval {
        AttackPhaseStep::Ready
    } else {
        AttackPhaseStep::Charging
    }
}

pub fn start_attack_windup(asd_count: &mut Fixed64, interval: Fixed64) -> (Fixed64, Fixed64) {
    let (windup, backswing) = attack_phase_durations(interval);
    let over = {
        let count = *asd_count - interval;
        if count > Fixed64::ZERO {
            count
        } else {
            Fixed64::ZERO
        }
    };
    *asd_count = over - windup;
    (windup, backswing)
}

pub fn fixed_secs_to_ms(value: Fixed64) -> u32 {
    (value.to_f32_for_render() * 1000.0).clamp(0.0, u32::MAX as f32) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::SimulationTickProfile;

    fn attack_order_for_profile(profile: SimulationTickProfile) -> Vec<AttackPhaseStep> {
        let interval = Fixed64::from_raw(819);
        let mut asd_count = interval;
        let mut events = Vec::new();
        for tick in 1..=u64::from(profile.ticks_per_game_second()) * 10 {
            match advance_attack_phase(
                &mut asd_count,
                Fixed64::from_raw(profile.fixed_raw_for_tick(tick)),
                interval,
            ) {
                AttackPhaseStep::Ready => {
                    events.push(AttackPhaseStep::Ready);
                    start_attack_windup(&mut asd_count, interval);
                }
                AttackPhaseStep::Impact => events.push(AttackPhaseStep::Impact),
                AttackPhaseStep::Charging => {}
            }
        }
        events
    }

    #[test]
    fn windup_and_backswing_sum_to_interval() {
        let interval = Fixed64::from_i32(1);
        let (windup, backswing) = attack_phase_durations(interval);
        assert_eq!(windup + backswing, interval);
    }

    #[test]
    fn negative_asd_count_charges_until_impact() {
        let interval = Fixed64::from_i32(1);
        let mut asd_count = interval;
        let (windup, _) = start_attack_windup(&mut asd_count, interval);
        assert_eq!(asd_count, -windup);
        assert_eq!(
            advance_attack_phase(&mut asd_count, windup - Fixed64::from_raw(1), interval),
            AttackPhaseStep::Charging
        );
        assert_eq!(
            advance_attack_phase(&mut asd_count, Fixed64::from_raw(1), interval),
            AttackPhaseStep::Impact
        );
        assert_eq!(asd_count, windup);
    }

    #[test]
    fn repeated_attack_order_matches_at_fifteen_and_one_twenty_hz() {
        let coarse = attack_order_for_profile(SimulationTickProfile::Coarse15Hz);
        let fine = attack_order_for_profile(SimulationTickProfile::Production120Hz);

        assert_eq!(coarse, fine);
        assert!(!coarse.is_empty());
        assert!(coarse
            .chunks_exact(2)
            .all(|pair| pair == [AttackPhaseStep::Ready, AttackPhaseStep::Impact]));
    }
}
