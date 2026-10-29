//! Damage trace for conformance tests (work line B). A test world that inserts [`DamageTrace`]
//! gets one row per script damage packet settled at serial settlement: who, how much before and
//! after defenses, and whether it landed, was skipped because its source had died
//! (`require_source_alive`, v3.md §規則層), or was refused for another reason (`cancelled`:
//! invulnerable, removed target, match rules). Generated code tags each damage with its ability
//! and v3 effect path (`GameWorld::ext(DAMAGE_TAG)` → `Outcome::ScriptDamageTag` right before the
//! damage outcome). Without the resource nothing is recorded and scripts send no tags, so live
//! matches and replays are unaffected.
use omoba_sim::Fixed64;
use specs::{Entity, World, WorldExt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageTraceResult {
    Applied,
    SkippedSourceDead,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DamageTraceRow {
    /// ECS `Tick` at settlement.
    pub tick: u64,
    pub source: Entity,
    pub target: Entity,
    /// (physical, magical, pure) as the script dealt it.
    pub before: (Fixed64, Fixed64, Fixed64),
    /// After block, armor and magic resistance; `None` unless applied.
    pub after: Option<(Fixed64, Fixed64, Fixed64)>,
    /// `DamageProfile` bits of the packet.
    pub profile: u32,
    /// The script ability and v3 effect path of the tag, empty for an untagged packet.
    pub ability: String,
    pub path: String,
    pub result: DamageTraceResult,
}

/// Insert into a test world to record script damage (`world.insert(DamageTrace::default())`).
#[derive(Clone, Debug, Default)]
pub struct DamageTrace(pub Vec<DamageTraceRow>);

/// The tag a script attached to its next damage packet.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DamageTag {
    pub ability: String,
    pub path: String,
}

pub fn enabled(world: &World) -> bool {
    world.has_value::<DamageTrace>()
}

pub(crate) fn record(world: &World, row: DamageTraceRow) {
    if let Some(mut trace) = world.try_fetch_mut::<DamageTrace>() {
        trace.0.push(row);
    }
}

pub(crate) fn tick(world: &World) -> u64 {
    world.try_fetch::<crate::runtime::Tick>().map_or(0, |tick| tick.0)
}

pub(crate) fn source_dead_skip(world: &World, source: Entity, profile: u32) -> bool {
    profile & omb_script_abi::types::DamageProfile::REQUIRE_SOURCE_ALIVE.bits() != 0
        && !super::rules::damage::source_alive(world, source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comp::Outcome;
    use omb_script_abi::types::DamageProfile;

    fn world() -> (World, crate::runtime::SimulationDriver, Entity, Entity) {
        use crate::runtime::{SingleLaneConfig, StateInitializer};
        let pool = StateInitializer::create_thread_pool();
        let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
        crate::runtime::setup_single_lane_match(&mut world, SingleLaneConfig {
            warmup: Fixed64::ZERO, wave_interval: Fixed64::from_i32(1_000_000), ..SingleLaneConfig::default()
        }).unwrap();
        let heroes = world.read_resource::<crate::runtime::MobaMatch>().heroes.iter().filter_map(|slot| slot.entity).collect::<Vec<_>>();
        world.insert(crate::runtime::TowerTemplateRegistry::default());
        world.insert(crate::runtime::ScriptRegistry::new());
        world.insert(DamageTrace::default());
        let driver = crate::runtime::SimulationDriver::from_world(&mut world, crate::runtime::SimulationTickProfile::Production60Hz).unwrap();
        (world, driver, heroes[0], heroes[1])
    }

    fn send(world: &mut World, source: Entity, target: Entity, profile: u32, path: &str) {
        let mut outcomes = world.write_resource::<Vec<Outcome>>();
        outcomes.push(Outcome::ScriptDamageTag { ability: "probe".into(), path: path.into() });
        outcomes.push(Outcome::RawDamage { pos: omoba_sim::Vec2::ZERO, phys: Fixed64::ZERO, magi: Fixed64::from_i32(30),
            real: Fixed64::ZERO, source, target, damage_profile: profile, origin: None });
    }

    /// conformance（damage_skipped_if_source_dead）：來源已死時，require_source_alive 的那筆記成
    /// skipped_source_dead，不帶這個旗標的照常結算；兩筆都帶各自的 effect path。
    #[test]
    fn rows_tell_applied_from_skipped_because_the_source_died() {
        let (mut world, mut driver, a, b) = world();
        driver.step(&mut world, []).unwrap();
        let alive = DamageProfile::NORMAL.union(DamageProfile::REQUIRE_SOURCE_ALIVE).bits();
        send(&mut world, a, b, alive, "events.on_cast[1].actions[1]");
        driver.step(&mut world, []).unwrap();
        // Killed earlier in the batch: settle right away, before any regeneration.
        world.write_storage::<crate::runtime::comp::CProperty>().get_mut(a).unwrap().hp = Fixed64::ZERO;
        send(&mut world, a, b, alive, "events.on_cast[1].actions[2]");
        send(&mut world, a, b, DamageProfile::NORMAL.bits(), "statuses[1].events.on_created[1]");
        let mut events = crate::runtime::native::events::RuntimeEventVecSink::default();
        crate::runtime::native::game_processor::process_outcomes(&mut world, &mut events).unwrap();
        let rows: Vec<_> = world.read_resource::<DamageTrace>().0.iter()
            .map(|row| (row.path.clone(), row.result, row.after.is_some())).collect();
        assert_eq!(rows, vec![
            ("events.on_cast[1].actions[1]".into(), DamageTraceResult::Applied, true),
            ("events.on_cast[1].actions[2]".into(), DamageTraceResult::SkippedSourceDead, false),
            ("statuses[1].events.on_created[1]".into(), DamageTraceResult::Applied, true),
        ]);
        assert!(world.read_resource::<DamageTrace>().0.iter().all(|row| row.ability == "probe" && row.source == a && row.target == b));
    }
}
