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
