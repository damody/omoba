//! Per-owner live summon cap (source B01E `B01E_CHECK`, B01.lua:116-140).
//!
//! A summon ability may author `owner_cap` on its `summon` effect (generated
//! extras key [`EXTRA_KEY`], one value per rank). After a cast of that ability
//! commits its summons, the caster keeps at most `cap` live summons of the
//! templates this cast spawned; the earliest spawned ones beyond the cap are
//! retired at once (no death, so no bounty and no kill credit).
//!
//! Order is deterministic: (spawn tick, canonical entity id). Expired or dead
//! bodies never count and are never "replaced" (the source would also retire a
//! corpse; the design intent is "at most `cap` live gunners per owner").
use specs::{Entity, Join, World, WorldExt};

use super::comp::*;

/// v3 `create_unit { limit_per_owner, over_limit = "remove_oldest" }` (E418): a one-shot packet on
/// the owner, never stored, that keeps at most `limit` of its live summons of `template`, the
/// earliest beyond it retired at once ([`enforce`]). Any path that spawns may send it after its
/// `create_unit`; a cast's spawns commit before its other effects settle.
pub const LIMIT_BUFF: &str = "__summon_limit";
pub const LIMIT_KEY: &str = "__summon_limit_v1";

/// The packet's payload.
pub fn limit_payload(template: &str, limit: usize) -> serde_json::Value {
    serde_json::json!({ (LIMIT_KEY): { "template": template, "limit": limit } })
}

/// A well-formed packet: a template id and a limit in 1..=[`MAX_CAP`].
pub fn limit_spec(buff_id: &str, payload: &serde_json::Value) -> Option<(String, usize)> {
    if buff_id != LIMIT_BUFF || payload.as_object()?.len() != 1 { return None; }
    let row = payload.get(LIMIT_KEY)?.as_object()?;
    if row.len() != 2 { return None; }
    let template = row.get("template")?.as_str()?;
    let limit = usize::try_from(row.get("limit")?.as_u64()?).ok()?;
    (omoba_template_ids::summon_by_name(template).is_some() && (1..=MAX_CAP).contains(&limit))
        .then(|| (template.to_owned(), limit))
}

/// Settles a [`LIMIT_BUFF`] packet on `owner`; returns the summons it retired (the caller removes them).
pub(crate) fn apply_limit(world: &mut World, owner: Entity, buff_id: &str, payload: &serde_json::Value) -> Vec<Entity> {
    let Some((template, limit)) = limit_spec(buff_id, payload) else { return Vec::new() };
    if !world.entities().is_alive(owner) { return Vec::new(); }
    enforce(world, owner, &[template], limit)
}

/// Generated extras key of the summon effect's per-rank `owner_cap`.
pub const EXTRA_KEY: &str = "summon_owner_cap";
/// Upper bound accepted from content; anything larger is ignored as invalid.
pub const MAX_CAP: usize = 64;

/// The authored cap of `ability` at `rank` (1-based), when it declares one.
pub fn cap_at(ability: &str, rank: u8) -> Option<usize> {
    let content = omoba_template_ids::ability_by_name(ability).and_then(omoba_template_ids::active_ability_const)?;
    let index = usize::from(rank.checked_sub(1)?);
    let value = content.extras.iter().find(|(name, _)| *name == EXTRA_KEY)?.1.get(index).copied()?;
    let raw = value.raw();
    (raw > 0 && raw % omoba_sim::fixed::SCALE == 0)
        .then_some((raw / omoba_sim::fixed::SCALE) as usize)
        .filter(|cap| *cap <= MAX_CAP)
}

/// Live summons of `owner` whose template is one of `units`, earliest first.
fn live_in_spawn_order(world: &World, owner: Entity, units: &[String]) -> Vec<Entity> {
    let entities = world.entities();
    let summons = world.read_storage::<SummonedUnit>();
    let props = world.read_storage::<CProperty>();
    let tags = world.read_storage::<crate::runtime::scripting::ScriptUnitTag>();
    let mut rows: Vec<(u64, u64, Entity)> = (&entities, &summons, &tags).join()
        .filter(|(entity, summon, tag)| summon.summoner == owner && !summon.is_expired()
            && props.get(*entity).is_some_and(|p| p.hp > omoba_sim::Fixed64::ZERO)
            && units.iter().any(|unit| *unit == tag.unit_id))
        .map(|(entity, summon, _)| (summon.spawn_tick, crate::runtime::canonical_entity_id(entity), entity))
        .collect();
    rows.sort_unstable_by_key(|(tick, id, _)| (*tick, *id));
    rows.into_iter().map(|(_, _, entity)| entity).collect()
}

/// Retires the earliest live summons beyond `cap`; returns them in retire order.
/// The caller publishes one `Outcome::EntityRemoved` for each.
pub(crate) fn enforce(world: &mut World, owner: Entity, units: &[String], cap: usize) -> Vec<Entity> {
    let live = live_in_spawn_order(world, owner, units);
    let excess = live.len().saturating_sub(cap);
    let retired: Vec<Entity> = live.into_iter().take(excess).collect();
    let mut summons = world.write_storage::<SummonedUnit>();
    retired.into_iter().filter(|entity| summons.get_mut(*entity).is_some_and(|summon| summon.expire_now())).collect()
}

/// After a committed cast: applies the cast ability's authored cap to the
/// templates it spawned. Nothing happens for abilities without `owner_cap`.
pub(crate) fn after_cast(world: &mut World, caster: Entity, ability: &str, units: &[String]) -> Vec<Entity> {
    if units.is_empty() { return Vec::new(); }
    let rank = world.read_storage::<Hero>().get(caster).map_or(0, |hero| hero.get_ability_level(ability));
    let Some(cap) = u8::try_from(rank).ok().and_then(|rank| cap_at(ability, rank)) else { return Vec::new(); };
    enforce(world, caster, units, cap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_sim::Fixed64;
    use specs::Builder;

    fn world() -> World {
        let mut world = World::new();
        world.register::<SummonedUnit>();
        world.register::<CProperty>();
        world.register::<crate::runtime::scripting::ScriptUnitTag>();
        world
    }

    fn summon(world: &mut World, owner: Entity, unit: &str, tick: u64) -> Entity {
        let mut row = SummonedUnit::new_fixed(owner, Some(Fixed64::from_i32(180)));
        row.spawn_tick = tick;
        world.create_entity().with(row)
            .with(CProperty { hp: Fixed64::from_i32(500), mhp: Fixed64::from_i32(500), msd: Fixed64::ZERO,
                def_physic: Fixed64::ONE, def_magic: Fixed64::ZERO })
            .with(crate::runtime::scripting::ScriptUnitTag { unit_id: unit.into() }).build()
    }

    fn expired(world: &World, entity: Entity) -> bool {
        world.read_storage::<SummonedUnit>().get(entity).unwrap().is_expired()
    }

    #[test]
    fn owner_cap_retires_the_earliest_live_summons_of_the_cast_templates_only() {
        let mut w = world();
        let owner = w.create_entity().build();
        let other = w.create_entity().build();
        let units = vec!["gunner".to_string()];
        // Spawned out of entity order: the tick decides who is oldest.
        let late = summon(&mut w, owner, "gunner", 30);
        let oldest = summon(&mut w, owner, "gunner", 10);
        let middle = summon(&mut w, owner, "gunner", 20);
        let foreign_template = summon(&mut w, owner, "turret", 1);
        let foreign_owner = summon(&mut w, other, "gunner", 1);
        assert!(enforce(&mut w, owner, &units, 3).is_empty(), "at the cap nothing is retired");
        let newest = summon(&mut w, owner, "gunner", 40);
        assert_eq!(enforce(&mut w, owner, &units, 3), vec![oldest]);
        assert!(expired(&w, oldest) && !expired(&w, middle) && !expired(&w, late) && !expired(&w, newest));
        assert!(!expired(&w, foreign_template) && !expired(&w, foreign_owner), "other templates and owners never count");
        // A lower cap (or several spawns at once) retires several, earliest first.
        assert_eq!(enforce(&mut w, owner, &units, 1), vec![middle, late]);
        assert!(!expired(&w, newest));
    }

    #[test]
    fn dead_or_expired_summons_neither_count_nor_get_replaced() {
        let mut w = world();
        let owner = w.create_entity().build();
        let units = vec!["gunner".to_string()];
        let corpse = summon(&mut w, owner, "gunner", 1);
        w.write_storage::<CProperty>().get_mut(corpse).unwrap().hp = Fixed64::ZERO;
        let gone = summon(&mut w, owner, "gunner", 2);
        w.write_storage::<SummonedUnit>().get_mut(gone).unwrap().expire_now();
        let a = summon(&mut w, owner, "gunner", 3);
        let b = summon(&mut w, owner, "gunner", 4);
        assert!(enforce(&mut w, owner, &units, 2).is_empty());
        assert!(!expired(&w, a) && !expired(&w, b) && !expired(&w, corpse));
        // Same tick: canonical identity breaks the tie deterministically.
        let c = summon(&mut w, owner, "gunner", 4);
        let retired = enforce(&mut w, owner, &units, 2);
        assert_eq!(retired, vec![a]);
        let _ = c;
    }

    /// E418: the `__summon_limit` packet of a create_unit with `limit_per_owner`.
    #[test]
    fn a_limit_packet_keeps_the_newest_summons_of_its_template() {
        let mut w = world();
        let owner = w.create_entity().build();
        let oldest = summon(&mut w, owner, "saika_gunner", 1);
        let middle = summon(&mut w, owner, "saika_gunner", 2);
        let newest = summon(&mut w, owner, "saika_gunner", 3);
        let other = summon(&mut w, owner, "lumen_sentry_turret", 0);
        assert_eq!(apply_limit(&mut w, owner, LIMIT_BUFF, &limit_payload("saika_gunner", 2)), vec![oldest]);
        assert!(!expired(&w, middle) && !expired(&w, newest) && !expired(&w, other));
        for bad in [limit_payload("saika_gunner", 0), limit_payload("no_such_template", 1),
            serde_json::json!({ (LIMIT_KEY): { "template": "saika_gunner", "limit": 1, "extra": 1 } })] {
            assert!(apply_limit(&mut w, owner, LIMIT_BUFF, &bad).is_empty());
        }
        assert!(apply_limit(&mut w, owner, "other", &limit_payload("saika_gunner", 1)).is_empty());
        assert_eq!(apply_limit(&mut w, owner, LIMIT_BUFF, &limit_payload("saika_gunner", 1)), vec![middle]);
    }

    /// E431: the removals a limit packet pushes while the outcome batch settles it survive the
    /// batch, and the next batch removes the retired summon.
    #[test]
    fn a_limit_packet_in_an_outcome_batch_removes_its_retired_summon() {
        use crate::runtime::{process_outcomes, RuntimeEventVecSink, StateInitializer, TowerTemplateRegistry};
        use crate::comp::Outcome;
        let mut w = StateInitializer::setup_campaign_ecs_world(&StateInitializer::create_thread_pool());
        w.insert(TowerTemplateRegistry::default());
        let owner = w.create_entity().build();
        let oldest = summon(&mut w, owner, "saika_gunner", 1);
        let newest = summon(&mut w, owner, "saika_gunner", 2);
        w.write_resource::<Vec<Outcome>>().push(Outcome::AddBuff { target: owner, buff_id: LIMIT_BUFF.into(),
            duration: Fixed64::ONE, payload: limit_payload("saika_gunner", 1) });
        process_outcomes(&mut w, &mut RuntimeEventVecSink::default()).unwrap();
        assert!(w.read_resource::<Vec<Outcome>>().iter().any(|o| matches!(o, Outcome::EntityRemoved { entity } if *entity == oldest)));
        process_outcomes(&mut w, &mut RuntimeEventVecSink::default()).unwrap();
        w.maintain();
        assert!(!w.is_alive(oldest) && w.is_alive(newest));
    }

    #[test]
    fn content_cap_reads_whole_positive_rank_values_only() {
        assert_eq!(cap_at("no_such_ability", 1), None);
        // Authored by B01E (`summon.owner_cap` 2/3/4/5).
        for (rank, cap) in [(1u8, 2usize), (2, 3), (3, 4), (4, 5)] {
            assert_eq!(cap_at("b01_saika_corps", rank), Some(cap), "rank {rank}");
        }
        assert_eq!(cap_at("b01_saika_corps", 0), None);
        assert_eq!(cap_at("b01_saika_corps", 5), None);
    }
}
