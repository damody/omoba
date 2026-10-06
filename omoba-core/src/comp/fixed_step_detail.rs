//! Optional diagnostics, never world state or canonical hash inputs.
use crate::runtime::native::gameplay_phases::{
    DeterministicGameplayPhase as P, DETERMINISTIC_GAMEPLAY_PHASES,
};

pub const PHASE_COUNT: usize = DETERMINISTIC_GAMEPLAY_PHASES.len();

pub fn phase_key(phase: P) -> &'static str {
    match phase {
        P::Dispatcher => "dispatcher_ns",
        P::RuntimeEventBoundary => "runtime_event_boundary_ns",
        P::HeroCommandClears => "hero_command_clears_ns",
        P::TowerSpawns => "tower_spawns_ns",
        P::TowerSells => "tower_sells_ns",
        P::TowerTargetPriorities => "tower_target_priorities_ns",
        P::ItemUses => "item_uses_ns",
        P::AbilityUpgrades => "ability_upgrades_ns",
        P::AbilityCasts => "ability_casts_ns",
        P::Moves => "moves_ns",
        P::PreScriptOutcomes => "pre_script_outcomes_ns",
        P::TowerUpgrades => "tower_upgrades_ns",
        P::TowerAbilityCasts => "tower_ability_casts_ns",
        P::TowerAbilityScheduler => "tower_ability_scheduler_ns",
        P::TowerAbilityCallbacks => "tower_ability_callbacks_ns",
        P::ScriptDispatch => "script_dispatch_ns",
        P::CreepWave => "creep_wave_ns",
        P::PostScriptOutcomes => "post_script_outcomes_ns",
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FixedStepDetail {
    pub preparation_ns: u128,
    pub phases_ns: [u128; PHASE_COUNT],
    pub finalization_ns: u128,
    /// Aggregate kernel + user CPU of all process threads, NOT wait time.
    /// May exceed wall time; unavailable counters are None, not zero.
    pub process_cpu_ns: Option<u128>,
    pub dispatcher: Option<super::dispatcher_detail::DispatcherDetail>,
}

impl FixedStepDetail {
    pub fn residual_ns(&self, fixed_step_ns: u128) -> Option<u128> {
        let sum = self.phases_ns.iter().try_fold(
            self.preparation_ns.checked_add(self.finalization_ns)?,
            |sum, ns| sum.checked_add(*ns),
        )?;
        fixed_step_ns.checked_sub(sum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_step_detail_phase_mapping_and_checked_reconciliation() {
        let mut keys = std::collections::BTreeSet::new();
        for (i, phase) in DETERMINISTIC_GAMEPLAY_PHASES.iter().enumerate() {
            assert_eq!(*phase as usize, i);
            assert!(keys.insert(phase_key(*phase)));
        }
        let mut detail = FixedStepDetail { preparation_ns: 2, finalization_ns: 3,
            process_cpu_ns: Some(999), ..Default::default() };
        detail.phases_ns[0] = 4;
        assert_eq!(detail.residual_ns(10), Some(1)); // CPU > wall is legal.
        assert_eq!(detail.residual_ns(8), None);
        detail.phases_ns[0] = u128::MAX;
        assert_eq!(detail.residual_ns(u128::MAX), None);
    }
}
