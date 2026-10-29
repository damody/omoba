//! Owner command rejections keyed by the admitted input id (`correlation`).
//!
//! Authority and the owner's replica run the same admission and cast rules on
//! the same disclosed owner state, so a rejection recorded here is the
//! authority's rejection. A step that admitted an input and recorded no row
//! means "scheduled, nothing rejected it this step", not "the effect landed":
//! a windup that is interrupted on a later tick is not reported here.
use std::collections::VecDeque;

use specs::{Entity, World};

pub const INVALID_SLOT: &str = "INVALID_SLOT";
pub const NO_HERO: &str = "NO_HERO";
pub const ABILITY_NOT_BOUND: &str = "ABILITY_NOT_BOUND";
pub const NOT_LEARNED: &str = "NOT_LEARNED";
pub const ON_COOLDOWN: &str = "ON_COOLDOWN";
pub const NOT_ENOUGH_MANA: &str = "NOT_ENOUGH_MANA";
pub const SILENCED: &str = "SILENCED";
pub const OUT_OF_RANGE: &str = "OUT_OF_RANGE";
pub const INVALID_TARGET: &str = "INVALID_TARGET";
pub const ALREADY_CASTING: &str = "ALREADY_CASTING";
pub const GAME_NOT_ACTIVE: &str = "GAME_NOT_ACTIVE";
pub const HERO_INACTIVE: &str = "HERO_INACTIVE";
pub const CAST_REJECTED: &str = "CAST_REJECTED";
pub const SKILL_POINTS: &str = "SKILL_POINTS";
pub const MAX_LEVEL: &str = "MAX_LEVEL";
pub const LEVEL_REQUIRED: &str = "LEVEL_REQUIRED";
pub const UPGRADE_REJECTED: &str = "UPGRADE_REJECTED";
/// A charge ability with no stored use (G-2 charges).
pub const NO_CHARGE: &str = "NO_CHARGE";
/// A unit or point ability submitted without that target (input shape only).
pub const TARGET_REQUIRED: &str = "TARGET_REQUIRED";
/// A move order whose destination is unreachable and no reachable point is closer.
pub const UNREACHABLE: &str = "UNREACHABLE";

/// Every code a renderer may receive from this journal.
pub const REJECTION_CODES: &[&str] = &[
    INVALID_SLOT, NO_HERO, ABILITY_NOT_BOUND, NOT_LEARNED, ON_COOLDOWN, NOT_ENOUGH_MANA, SILENCED,
    OUT_OF_RANGE, INVALID_TARGET, ALREADY_CASTING, GAME_NOT_ACTIVE, HERO_INACTIVE, CAST_REJECTED,
    SKILL_POINTS, MAX_LEVEL, LEVEL_REQUIRED, UPGRADE_REJECTED, NO_CHARGE, TARGET_REQUIRED,
];

/// Codes a world without authority-only state (the owner replica: no
/// `MobaMatch`, no host lifecycles, partial visibility) may still report,
/// because they come only from the owner's own disclosed hero state. Any
/// other dispatch-stage failure in such a world is not the authority's
/// verdict and is never journaled there.
pub fn owner_state_code(code: &str) -> bool {
    matches!(code, "NOT_LEARNED" | "ON_COOLDOWN" | "NOT_ENOUGH_MANA" | "SILENCED" | "HERO_INACTIVE" | "ALREADY_CASTING" | "NO_CHARGE" | "TARGET_REQUIRED")
}

/// Stable wire number for a renderer ABI field: the 1-based position in
/// `REJECTION_CODES` (append-only). 0 means no rejection or an unknown code.
pub fn rejection_code_number(code: &str) -> u32 {
    REJECTION_CODES.iter().position(|known| *known == code).map_or(0, |index| index as u32 + 1)
}

/// Bounded so an authority that never drains rows cannot grow without limit.
const REJECTION_CAPACITY: usize = 256;
/// More casts than this per tick are not tracked; they stay "scheduled".
const IN_FLIGHT_CAPACITY: usize = 64;

/// One rejected admission with a stable code and a log message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandRejection {
    pub code: &'static str,
    pub message: String,
}

impl CommandRejection {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
}

impl std::fmt::Display for CommandRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnerCommandRejection {
    pub player_id: u32,
    pub correlation: u32,
    pub code: &'static str,
}

#[derive(Clone, Debug)]
struct InFlightCast {
    caster: Entity,
    skill_id: String,
    player_id: u32,
    correlation: u32,
    /// An input-shape problem found at admission; it names any later failure.
    shape: Option<&'static str>,
}

#[derive(Default)]
pub struct OwnerCommandOutcomes {
    rejections: VecDeque<OwnerCommandRejection>,
    in_flight: Vec<InFlightCast>,
}

impl OwnerCommandOutcomes {
    /// Correlation 0 is an unidentified command (bots, legacy) and is not journaled.
    pub fn reject(&mut self, player_id: u32, correlation: u32, code: &'static str) {
        if player_id == 0 || correlation == 0 {
            return;
        }
        if self.rejections.len() == REJECTION_CAPACITY {
            self.rejections.pop_front();
        }
        self.rejections.push_back(OwnerCommandRejection { player_id, correlation, code });
    }

    /// Drains this owner's rows in recording order; other owners keep theirs.
    pub fn take_for(&mut self, player_id: u32) -> Vec<(u32, &'static str)> {
        let mut taken = Vec::new();
        self.rejections.retain(|row| {
            if row.player_id == player_id {
                taken.push((row.correlation, row.code));
                false
            } else {
                true
            }
        });
        taken
    }

    pub fn rejections(&self) -> impl Iterator<Item = &OwnerCommandRejection> {
        self.rejections.iter()
    }

    /// A new cast drain starts a new tick's correlation window.
    pub(crate) fn begin_casts(&mut self) {
        self.in_flight.clear();
    }

    /// Admission passed; later same-tick cast stages may still reject it.
    pub(crate) fn track_cast(&mut self, caster: Entity, skill_id: &str, player_id: u32, correlation: u32, shape: Option<&'static str>) {
        if player_id == 0 || correlation == 0 || self.in_flight.len() >= IN_FLIGHT_CAPACITY {
            return;
        }
        self.in_flight.push(InFlightCast { caster, skill_id: skill_id.to_owned(), player_id, correlation, shape });
    }

    /// Rejects the oldest tracked input for this exact caster and skill. An
    /// admission shape problem names the failure. Without one, `judged` says
    /// whether this world may report `code`; otherwise the input just settles.
    pub(crate) fn reject_cast_if(&mut self, caster: Entity, skill_id: &str, code: &'static str, judged: bool) {
        let Some(index) = self.in_flight.iter().position(|row| row.caster == caster && row.skill_id == skill_id) else {
            return;
        };
        let row = self.in_flight.remove(index);
        match row.shape {
            Some(shape) => self.reject(row.player_id, row.correlation, shape),
            None if judged => self.reject(row.player_id, row.correlation, code),
            None => {}
        }
    }

    pub(crate) fn reject_cast(&mut self, caster: Entity, skill_id: &str, code: &'static str) {
        self.reject_cast_if(caster, skill_id, code, true);
    }

    /// The cast reached a stage that no longer reports per input (windup, success).
    pub(crate) fn settle_cast(&mut self, caster: Entity, skill_id: &str) {
        if let Some(index) = self.in_flight.iter().position(|row| row.caster == caster && row.skill_id == skill_id) {
            self.in_flight.remove(index);
        }
    }
}

pub(crate) fn ensure(world: &mut World) {
    if world.try_fetch::<OwnerCommandOutcomes>().is_none() {
        world.insert(OwnerCommandOutcomes::default());
    }
}

pub(crate) fn reject(world: &World, player_id: u32, correlation: u32, code: &'static str) {
    if let Some(mut outcomes) = world.try_fetch_mut::<OwnerCommandOutcomes>() {
        outcomes.reject(player_id, correlation, code);
    }
}

/// Handler errors are free text. A leading `CODE:` names a known code;
/// otherwise the shared generic-effect wording is classified, else generic.
pub fn classify_handler_error(message: &str) -> &'static str {
    if let Some((head, _)) = message.split_once(':') {
        if let Some(code) = REJECTION_CODES.iter().find(|code| **code == head.trim()) {
            return code;
        }
    }
    let lower = message.to_ascii_lowercase();
    if lower.contains("outside cast range") || lower.contains("out of range") {
        OUT_OF_RANGE
    } else if lower.contains("not alive") || lower.contains("not a live") || lower.contains("invalid target")
        || lower.contains("not an enemy") || lower.contains("not an ally") {
        INVALID_TARGET
    } else {
        CAST_REJECTED
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, WorldExt};

    #[test]
    fn journal_is_owner_scoped_bounded_and_ignores_unidentified_commands() {
        let mut journal = OwnerCommandOutcomes::default();
        journal.reject(1, 0, NOT_LEARNED);
        journal.reject(0, 9, NOT_LEARNED);
        journal.reject(1, 7, NOT_LEARNED);
        journal.reject(2, 8, ON_COOLDOWN);
        assert_eq!(journal.take_for(1), vec![(7, NOT_LEARNED)]);
        assert!(journal.take_for(1).is_empty());
        assert_eq!(journal.take_for(2), vec![(8, ON_COOLDOWN)]);
        for correlation in 1..=(REJECTION_CAPACITY as u32 + 3) {
            journal.reject(1, correlation, CAST_REJECTED);
        }
        let rows = journal.take_for(1);
        assert_eq!(rows.len(), REJECTION_CAPACITY);
        assert_eq!(rows[0].0, 4, "oldest rows are dropped first");
    }

    #[test]
    fn in_flight_casts_resolve_once_per_admitted_input() {
        let mut world = World::new();
        let caster = world.create_entity().build();
        let mut journal = OwnerCommandOutcomes::default();
        journal.track_cast(caster, "bolt", 1, 5, None);
        journal.track_cast(caster, "bolt", 1, 6, None);
        journal.reject_cast(caster, "bolt", NOT_ENOUGH_MANA);
        journal.settle_cast(caster, "bolt");
        journal.reject_cast(caster, "bolt", ON_COOLDOWN);
        assert_eq!(journal.take_for(1), vec![(5, NOT_ENOUGH_MANA)]);
        journal.track_cast(caster, "bolt", 1, 9, None);
        journal.begin_casts();
        journal.reject_cast(caster, "bolt", ON_COOLDOWN);
        assert!(journal.take_for(1).is_empty(), "a new tick forgets the previous window");
    }

    /// Production fixed step with an authenticated-style admission: the input
    /// travels as encoded `InputSubmit` bytes and its id is the correlation row
    /// aligned with `PendingPlayerInputs`, exactly as the KCP admission stages it.
    /// Every bound hero ability gets rank metadata and a handler that either
    /// succeeds or returns `refusal`, so dispatch-stage rejections are reachable.
    #[derive(Clone)]
    struct Handler(Option<&'static str>);
    impl omb_script_abi::ability::AbilityScript for Handler {
        fn ability_id(&self) -> abi_stable::std_types::RStr<'_> { "".into() }
        fn execute(&self, _: omb_script_abi::types::EntityHandle, _: omb_script_abi::types::Target, _: u8,
            _: abi_stable::std_types::RStr<'_>, _: &mut omb_script_abi::world::GameWorldDyn<'_>)
            -> abi_stable::std_types::RResult<(), abi_stable::std_types::RString> {
            match self.0 { Some(refusal) => abi_stable::std_types::RErr(refusal.into()), None => abi_stable::std_types::ROk(()) }
        }
    }

    fn admitted_world() -> (World, crate::runtime::SimulationDriver) {
        admitted_world_with(false, 0.0, None)
    }

    fn admitted_world_with(mana_enabled: bool, mana_cost: f32, refusal: Option<&'static str>) -> (World, crate::runtime::SimulationDriver) {
        admitted_world_targeted(mana_enabled, mana_cost, refusal, "none")
    }

    fn admitted_world_targeted(mana_enabled: bool, mana_cost: f32, refusal: Option<&'static str>, target_type: &str) -> (World, crate::runtime::SimulationDriver) {
        use crate::runtime::{SingleLaneConfig, StateInitializer};
        let pool = StateInitializer::create_thread_pool();
        let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
        crate::runtime::setup_single_lane_match(&mut world, SingleLaneConfig {
            warmup: omoba_sim::Fixed64::ZERO,
            wave_interval: omoba_sim::Fixed64::from_i32(1_000_000),
            mana_enabled,
            ..SingleLaneConfig::default()
        }).unwrap();
        let mut scripts = crate::runtime::ScriptRegistry::new();
        let ids: Vec<String> = specs::Join::join(&world.read_storage::<crate::comp::Hero>())
            .flat_map(|hero| hero.abilities.iter().filter(|id| !id.is_empty()).cloned()).collect();
        let mut abilities = crate::runtime::ability_runtime::AbilityRegistry::new();
        for id in ids {
            let level = serde_json::json!({"cooldown": 5, "mana_cost": mana_cost});
            let def: crate::ability_meta::AbilityDef = serde_json::from_value(serde_json::json!({
                "id": id, "name": id, "ability_type": "active", "target_type": target_type, "max_level": 4,
                "levels": {"1": level, "2": level, "3": level, "4": level},
            })).unwrap();
            abilities.register(def.clone());
            scripts.insert_ability_for_test(def, omb_script_abi::ability::AbilityScript_TO::from_value(
                Handler(refusal), abi_stable::sabi_trait::prelude::TD_Opaque));
        }
        world.insert(abilities);
        world.insert(scripts);
        let driver = crate::runtime::SimulationDriver::from_world(
            &mut world, crate::runtime::SimulationTickProfile::Production60Hz).unwrap();
        (world, driver)
    }

    fn submit(world: &mut World, driver: &mut crate::runtime::SimulationDriver, player_id: u32, input_id: u32,
        action: crate::game_proto::player_input::Action) {
        use prost::Message;
        let bytes = crate::game_proto::InputSubmit {
            player_id, target_tick: 0, input_id,
            input: Some(crate::game_proto::PlayerInput { action: Some(action) }),
        }.encode_to_vec();
        let decoded = crate::game_proto::InputSubmit::decode(bytes.as_slice()).unwrap();
        world.write_resource::<crate::comp::AdmittedCommandIdentities>().rows = vec![crate::comp::AdmittedCommandIdentity {
            player_id: decoded.player_id, correlation: decoded.input_id,
            actor_generation: crate::runtime::trusted_actor_generation(world, decoded.player_id),
        }];
        driver.step(world, [(decoded.player_id, decoded.input.unwrap())]).unwrap();
    }

    fn hero(world: &World, player_id: u32) -> Entity {
        world.read_resource::<crate::runtime::MobaMatch>().heroes.iter()
            .find(|slot| slot.player_id == player_id).and_then(|slot| slot.entity).unwrap()
    }

    #[test]
    fn rejected_owner_cast_and_upgrade_report_codes_by_input_id() {
        use crate::game_proto::{player_input::Action, CastAbility, UpgradeAbility};
        let (mut world, mut driver) = admitted_world();
        driver.step(&mut world, []).unwrap();
        let hero = hero(&world, 1);
        {
            let mut heroes = world.write_storage::<crate::comp::Hero>();
            let state = heroes.get_mut(hero).unwrap();
            let ultimate = state.abilities[3].clone();
            state.ability_levels.remove(&ultimate);
        }
        // split5: synthetic Ctrl+R arrived as a cast of the unlearned R slot.
        submit(&mut world, &mut driver, 1, 41, Action::CastAbility(CastAbility { ability_index: 3, ..Default::default() }));
        assert_eq!(world.write_resource::<OwnerCommandOutcomes>().take_for(1), vec![(41, NOT_LEARNED)]);

        {
            let mut heroes = world.write_storage::<crate::comp::Hero>();
            let state = heroes.get_mut(hero).unwrap();
            state.skill_points = 1;
        }
        let before = { let heroes = world.read_storage::<crate::comp::Hero>(); let state = heroes.get(hero).unwrap(); state.get_ability_level(&state.abilities[0]) };
        submit(&mut world, &mut driver, 1, 42, Action::UpgradeAbility(UpgradeAbility { ability_index: 0 }));
        assert!(world.write_resource::<OwnerCommandOutcomes>().take_for(1).is_empty(), "a valid upgrade is applied, not rejected");
        let ability = world.read_storage::<crate::comp::Hero>().get(hero).unwrap().abilities[0].clone();
        assert_eq!(world.read_storage::<crate::comp::Hero>().get(hero).unwrap().get_ability_level(&ability), before + 1);
        submit(&mut world, &mut driver, 1, 43, Action::UpgradeAbility(UpgradeAbility { ability_index: 0 }));
        assert_eq!(world.write_resource::<OwnerCommandOutcomes>().take_for(1), vec![(43, SKILL_POINTS)]);
        submit(&mut world, &mut driver, 1, 44, Action::UpgradeAbility(UpgradeAbility { ability_index: 9 }));
        assert_eq!(world.write_resource::<OwnerCommandOutcomes>().take_for(1), vec![(44, INVALID_SLOT)]);

        world.write_storage::<crate::comp::Hero>().get_mut(hero).unwrap()
            .start_cooldown(&ability, omoba_sim::Fixed64::from_i32(30));
        submit(&mut world, &mut driver, 1, 45, Action::CastAbility(CastAbility { ability_index: 0, ..Default::default() }));
        assert_eq!(world.write_resource::<OwnerCommandOutcomes>().take_for(1), vec![(45, ON_COOLDOWN)]);
        assert!(world.write_resource::<OwnerCommandOutcomes>().take_for(2).is_empty(), "other owners see nothing");
    }

    #[test]
    fn dispatch_stage_cast_rejections_reach_the_same_input_id() {
        use crate::game_proto::{player_input::Action, CastAbility};
        let cast = || Action::CastAbility(CastAbility { ability_index: 0, ..Default::default() });
        // Admission passes; the managed reservation cannot pay.
        let (mut world, mut driver) = admitted_world_with(true, 100_000.0, None);
        driver.step(&mut world, []).unwrap();
        submit(&mut world, &mut driver, 1, 51, cast());
        assert_eq!(world.write_resource::<OwnerCommandOutcomes>().take_for(1), vec![(51, NOT_ENOUGH_MANA)]);
        // The handler refuses with the shared generic-effect wording.
        let (mut world, mut driver) = admitted_world_with(false, 0.0, Some("ability 'x': point outside cast range"));
        driver.step(&mut world, []).unwrap();
        submit(&mut world, &mut driver, 1, 52, cast());
        assert_eq!(world.write_resource::<OwnerCommandOutcomes>().take_for(1), vec![(52, OUT_OF_RANGE)]);
        // A successful cast leaves no row; the next tick forgets the window.
        let (mut world, mut driver) = admitted_world_with(false, 0.0, None);
        driver.step(&mut world, []).unwrap();
        submit(&mut world, &mut driver, 1, 53, cast());
        assert!(world.write_resource::<OwnerCommandOutcomes>().take_for(1).is_empty());
        let hero = hero(&world, 1);
        let ability = world.read_storage::<crate::comp::Hero>().get(hero).unwrap().abilities[0].clone();
        assert!(world.read_storage::<crate::comp::Hero>().get(hero).unwrap().is_on_cooldown(&ability), "the cast really applied");
    }

    /// A world without authority-only state (the replica) reports only codes
    /// derived from the owner's own hero; it never invents a handler verdict.
    #[test]
    fn a_non_authoritative_world_reports_only_owner_state_rejections() {
        use crate::game_proto::{player_input::Action, CastAbility};
        let cast = || Action::CastAbility(CastAbility { ability_index: 0, ..Default::default() });
        let (mut world, mut driver) = admitted_world_with(false, 0.0, Some("ability 'x': point outside cast range"));
        driver.step(&mut world, []).unwrap();
        world.remove::<crate::runtime::MobaMatch>();
        submit(&mut world, &mut driver, 1, 71, cast());
        assert!(world.write_resource::<OwnerCommandOutcomes>().take_for(1).is_empty(), "no false OUT_OF_RANGE");
        let (mut world, mut driver) = admitted_world_with(true, 100_000.0, None);
        driver.step(&mut world, []).unwrap();
        world.remove::<crate::runtime::MobaMatch>();
        submit(&mut world, &mut driver, 1, 72, cast());
        assert_eq!(world.write_resource::<OwnerCommandOutcomes>().take_for(1), vec![(72, NOT_ENOUGH_MANA)]);
        assert!(owner_state_code(ON_COOLDOWN) && !owner_state_code(CAST_REJECTED) && !owner_state_code(OUT_OF_RANGE));
    }

    /// split15: a unit ability submitted with no target. The shape is input-only,
    /// so the replica reports it too; a supplied target is never second-guessed.
    #[test]
    fn a_missing_unit_target_is_reported_by_input_shape_in_every_world() {
        use crate::game_proto::{player_input::Action, CastAbility};
        let refusal = Some("ability 'x' requires an entity target");
        for authoritative in [true, false] {
            let (mut world, mut driver) = admitted_world_targeted(false, 0.0, refusal, "unit");
            driver.step(&mut world, []).unwrap();
            let enemy = world.read_resource::<crate::runtime::MobaMatch>().heroes.iter()
                .find(|slot| slot.player_id == 2).and_then(|slot| slot.entity).unwrap();
            let caster = hero(&world, 1);
            if !authoritative { world.remove::<crate::runtime::MobaMatch>(); }
            submit(&mut world, &mut driver, 1, 81, Action::CastAbility(CastAbility { ability_index: 0, ..Default::default() }));
            assert_eq!(world.write_resource::<OwnerCommandOutcomes>().take_for(1), vec![(81, TARGET_REQUIRED)], "authoritative={authoritative}");
            world.write_storage::<crate::comp::Hero>().get_mut(caster).unwrap().ability_cooldowns.clear();
            submit(&mut world, &mut driver, 1, 82, Action::CastAbility(CastAbility {
                ability_index: 0, target_entity: Some(enemy.id()), ..Default::default() }));
            let rows = world.write_resource::<OwnerCommandOutcomes>().take_for(1);
            // The enemy stands far beyond the cast range: the authority keeps the order as an
            // approach to cast (E210) instead of reporting anything yet; a world without a match
            // casts it at once and cannot judge the handler.
            let approaching = world.try_fetch::<super::super::cast_approach::CastApproaches>().is_some_and(|a| a.0.contains_key(&caster));
            if authoritative { assert!(rows.is_empty() && approaching, "approach, not a shape refusal: {rows:?}"); }
            else { assert!(rows.is_empty() && !approaching, "the replica cannot judge a handler"); }
        }
    }

    #[test]
    fn handler_errors_map_to_stable_codes() {
        assert_eq!(classify_handler_error("ability 'x': point outside cast range"), OUT_OF_RANGE);
        assert_eq!(classify_handler_error("ability 'x' target is not alive"), INVALID_TARGET);
        assert_eq!(classify_handler_error("ability 'x': projectile target is not a live enemy"), INVALID_TARGET);
        assert_eq!(classify_handler_error("NOT_ENOUGH_MANA: custom script"), NOT_ENOUGH_MANA);
        assert_eq!(classify_handler_error("UNKNOWN_CODE: custom script"), CAST_REJECTED);
        assert_eq!(classify_handler_error("anything else"), CAST_REJECTED);
    }

    #[test]
    fn rejection_numbers_are_append_only() {
        assert_eq!(rejection_code_number(INVALID_SLOT), 1);
        assert_eq!(rejection_code_number(ON_COOLDOWN), 5);
        assert_eq!(rejection_code_number(NOT_ENOUGH_MANA), 6);
        assert_eq!(rejection_code_number(UPGRADE_REJECTED), 17);
        assert_eq!(rejection_code_number(NO_CHARGE), 18);
        assert_eq!(rejection_code_number(TARGET_REQUIRED), 19);
        assert_eq!(rejection_code_number("APPLIED_TO_PRESENTATION"), 0);
    }
}
