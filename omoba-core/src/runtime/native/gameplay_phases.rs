/// The only production ordering table for deterministic gameplay work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum DeterministicGameplayPhase {
    Dispatcher,
    RuntimeEventBoundary,
    HeroCommandClears,
    TowerSpawns,
    TowerSells,
    TowerTargetPriorities,
    ItemUses,
    AbilityUpgrades,
    AbilityCasts,
    SummonerCasts,
    Moves,
    PreScriptOutcomes,
    TowerUpgrades,
    TowerAbilityCasts,
    TowerAbilityScheduler,
    TowerAbilityCallbacks,
    ScriptDispatch,
    CreepWave,
    PostScriptOutcomes,
}

pub const DETERMINISTIC_GAMEPLAY_PHASES: &[DeterministicGameplayPhase] = &[
    DeterministicGameplayPhase::Dispatcher,
    DeterministicGameplayPhase::RuntimeEventBoundary,
    DeterministicGameplayPhase::HeroCommandClears,
    DeterministicGameplayPhase::TowerSpawns,
    DeterministicGameplayPhase::TowerSells,
    DeterministicGameplayPhase::TowerTargetPriorities,
    DeterministicGameplayPhase::ItemUses,
    DeterministicGameplayPhase::AbilityUpgrades,
    DeterministicGameplayPhase::AbilityCasts,
    DeterministicGameplayPhase::SummonerCasts,
    DeterministicGameplayPhase::Moves,
    DeterministicGameplayPhase::PreScriptOutcomes,
    DeterministicGameplayPhase::TowerUpgrades,
    DeterministicGameplayPhase::TowerAbilityCasts,
    DeterministicGameplayPhase::TowerAbilityScheduler,
    DeterministicGameplayPhase::TowerAbilityCallbacks,
    DeterministicGameplayPhase::ScriptDispatch,
    DeterministicGameplayPhase::CreepWave,
    DeterministicGameplayPhase::PostScriptOutcomes,
];

pub trait DeterministicGameplayContext {
    type Error;
    fn run_phase(&mut self, phase: DeterministicGameplayPhase) -> Result<(), Self::Error>;
}

impl<E, F> DeterministicGameplayContext for F
where
    F: FnMut(DeterministicGameplayPhase) -> Result<(), E>,
{
    type Error = E;

    fn run_phase(&mut self, phase: DeterministicGameplayPhase) -> Result<(), Self::Error> {
        self(phase)
    }
}

pub fn run_deterministic_gameplay_phases<C: DeterministicGameplayContext>(
    context: &mut C,
) -> Result<(), C::Error> {
    for phase in DETERMINISTIC_GAMEPLAY_PHASES {
        context.run_phase(*phase)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summoner_phase_index_matches_table_between_ability_casts_and_moves() {
        let phases = DETERMINISTIC_GAMEPLAY_PHASES;
        assert_eq!(phases.len(), 19);
        let casts = phases.iter().position(|phase| *phase == DeterministicGameplayPhase::AbilityCasts).unwrap();
        let summoners = phases.iter().position(|phase| *phase == DeterministicGameplayPhase::SummonerCasts).unwrap();
        let moves = phases.iter().position(|phase| *phase == DeterministicGameplayPhase::Moves).unwrap();
        assert_eq!(summoners, casts + 1);
        assert_eq!(moves, summoners + 1);
        assert_eq!(DeterministicGameplayPhase::SummonerCasts as usize, summoners);
        assert_eq!(phases.first(), Some(&DeterministicGameplayPhase::Dispatcher));
        assert_eq!(phases.last(), Some(&DeterministicGameplayPhase::PostScriptOutcomes));
    }
}
