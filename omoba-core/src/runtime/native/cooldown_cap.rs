//! Cooldown cap buff (source A34T `A34T_OnIntervalThink`, A34_Matsu.lua:100-113).
//! While a hero carries a buff whose payload has [`CAP_KEY`] with a fraction
//! `x` (0 < x < 1), every running cooldown of the hero's non-ultimate abilities
//! is kept at most `full × (1 − x)`, where `full` is the authored cooldown of
//! the ability's current rank (Dota `GetCooldown(-1)`). This includes cooldowns
//! that started before the buff. Ultimates and items are untouched.
//!
//! [`settle`] runs at serial settlement every tick on the authority and on the
//! hero's own team replica (both simulate the cast that granted the buff), and
//! reports the slots it shortened so opponents receive the settled remaining
//! through `CommittedCooldown`. A clamp of a running cooldown is a reduction: the
//! admitted total is kept, exactly as `Hero::commit_cooldown` reproduces it on a
//! replica. A cooldown clamped the tick it started starts capped (total = cap),
//! since an opponent replica only ever receives that capped remaining.
use omoba_sim::Fixed64;
use omoba_template_ids::{AbilityTypeC, ability_by_name, active_ability_const};
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

use super::comp::Hero;

pub const CAP_KEY: &str = "__cooldown_cap_v1";

/// Payload row for a cap of `fraction` (raw Fixed64 of 0 < x < 1).
pub fn cap_value(fraction: Fixed64) -> Value { json!({ "fraction": fraction.raw(), "scope": "basic" }) }

fn fraction_of(payload: &Value) -> Option<Fixed64> {
    let row = payload.get(CAP_KEY)?.as_object()?;
    if row.len() != 2 || row.get("scope")?.as_str()? != "basic" { return None; }
    let fraction = Fixed64::from_raw(row.get("fraction")?.as_i64()?);
    (fraction > Fixed64::ZERO && fraction < Fixed64::ONE).then_some(fraction)
}

/// Strongest cap fraction among the unit's buffs (order-independent maximum).
pub fn cap_fraction(buffs: &crate::runtime::BuffStore, entity: Entity) -> Option<Fixed64> {
    buffs.iter_for(entity).filter_map(|(_, entry)| fraction_of(&entry.payload)).max()
}

/// Authored cooldown of `id` at its current rank, for non-ultimate abilities.
pub fn basic_full_cooldown(hero: &Hero, id: &str) -> Option<Fixed64> {
    let content = ability_by_name(id).and_then(active_ability_const)?;
    if content.ability_type == AbilityTypeC::Ultimate { return None; }
    let rank = usize::try_from(hero.get_ability_level(id)).ok().filter(|rank| *rank > 0)?;
    content.levels.get(rank - 1).map(|level| level.cooldown).filter(|full| *full > Fixed64::ZERO)
}

/// Clamps one hero; returns the loadout slots it shortened, in slot order.
pub fn clamp(hero: &mut Hero, fraction: Fixed64) -> Vec<(usize, String)> {
    let mut changed = Vec::new();
    for slot in 0..hero.abilities.len() {
        let id = hero.abilities[slot].clone();
        let Some(full) = basic_full_cooldown(hero, &id) else { continue; };
        let cap = Fixed64::from_raw((i128::from(full.raw()) * i128::from((Fixed64::ONE - fraction).raw()) / 1024) as i64);
        let remaining = hero.get_cooldown(&id);
        // A cooldown that started this tick (remaining still equals its admitted total) starts
        // capped: opponents only ever see the committed capped remaining, which a replica
        // commits as a fresh instance whose total is that remaining (E170 live mismatch).
        let fresh = hero.ability_cooldown_totals.get(&id) == Some(&remaining);
        if remaining > cap && hero.reduce_cooldown(&id, remaining - cap) {
            if fresh { hero.ability_cooldown_totals.insert(id.clone(), cap); }
            changed.push((slot, id));
        }
    }
    changed
}

/// Serial settle for every hero carrying a cap, in canonical entity order.
pub fn settle(world: &mut World) -> Vec<(Entity, usize, String)> {
    let mut rows: Vec<(Entity, Fixed64)> = {
        let buffs = world.read_resource::<crate::runtime::BuffStore>();
        let mut entities: Vec<Entity> = buffs.entities_with_key(CAP_KEY).collect();
        entities.sort_by_key(|e| (e.id(), e.gen().id()));
        entities.dedup();
        entities.into_iter().filter_map(|e| cap_fraction(&buffs, e).map(|f| (e, f))).collect()
    };
    rows.retain(|(e, _)| world.entities().is_alive(*e));
    let mut out = Vec::new();
    // Worlds without heroes (script-outcome fixtures) have nothing to cap.
    if rows.is_empty() || !world.has_value::<specs::storage::MaskedStorage<Hero>>() { return out; }
    let mut heroes = world.write_storage::<Hero>();
    for (entity, fraction) in rows {
        let Some(hero) = heroes.get_mut(entity) else { continue; };
        out.extend(clamp(hero, fraction).into_iter().map(|(slot, id)| (entity, slot, id)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: i32) -> Fixed64 { Fixed64::from_i32(v) }

    #[test]
    fn caps_running_and_new_basic_cooldowns_but_not_ultimates_or_unknown_ids() {
        // A34 kit: W cd 14, E cd 7, R cd 10/9/8/7, T (ultimate) cd 100/90/80.
        let mut hero = Hero::default();
        hero.abilities = vec!["a34_frost_bolt".into(), "a34_iceberg_strike".into(), "a34_frost_breath".into(),
            "a34_rapid_cooling".into(), "not_a_real_ability".into()];
        for id in hero.abilities.clone() { hero.ability_levels.insert(id, 1); }
        hero.start_cooldown("a34_frost_bolt", s(14));
        hero.start_cooldown("a34_iceberg_strike", s(3));
        hero.start_cooldown("a34_frost_breath", s(10));
        hero.start_cooldown("a34_rapid_cooling", s(100));
        hero.start_cooldown("not_a_real_ability", s(50));
        // 40%: W ≤ 8.4, E ≤ 4.2 (3 stays), R ≤ 6.
        let changed = clamp(&mut hero, Fixed64::from_raw(410));
        assert_eq!(changed.iter().map(|(slot, _)| *slot).collect::<Vec<_>>(), vec![0, 2]);
        let cap = |full: i32| Fixed64::from_raw(i64::from(full) * 1024 * (1024 - 410) / 1024);
        assert_eq!(hero.get_cooldown("a34_frost_bolt"), cap(14));
        assert_eq!(hero.get_cooldown("a34_iceberg_strike"), s(3));
        assert_eq!(hero.get_cooldown("a34_frost_breath"), cap(10));
        assert_eq!((hero.get_cooldown("a34_rapid_cooling"), hero.get_cooldown("not_a_real_ability")), (s(100), s(50)));
        // A cooldown clamped the tick it started starts capped: an opponent replica that only
        // ever receives the committed capped remaining reproduces the same instance (E170).
        assert_eq!(hero.cooldown_total("a34_frost_bolt", s(14)), Some(cap(14)));
        let mut opponent = Hero::default();
        opponent.commit_cooldown("a34_frost_bolt", cap(14));
        assert_eq!(opponent.ability_cooldowns.get("a34_frost_bolt"), hero.ability_cooldowns.get("a34_frost_bolt"));
        assert_eq!(opponent.ability_cooldown_totals.get("a34_frost_bolt"), hero.ability_cooldown_totals.get("a34_frost_bolt"));
        // A cooldown already running when clamped keeps its admitted total, as a replica that
        // committed it earlier does.
        let mut running = Hero::default();
        running.abilities = vec!["a34_frost_bolt".into()];running.ability_levels.insert("a34_frost_bolt".into(), 1);
        running.start_cooldown("a34_frost_bolt", s(14));
        assert!(running.reduce_cooldown("a34_frost_bolt", s(1)));
        let mut replica = running.clone();
        assert_eq!(clamp(&mut running, Fixed64::from_raw(410)).len(), 1);
        replica.commit_cooldown("a34_frost_bolt", cap(14));
        assert_eq!(running.cooldown_total("a34_frost_bolt", s(14)), Some(s(14)));
        assert_eq!(replica.ability_cooldowns.get("a34_frost_bolt"), running.ability_cooldowns.get("a34_frost_bolt"));
        assert_eq!(replica.ability_cooldown_totals.get("a34_frost_bolt"), running.ability_cooldown_totals.get("a34_frost_bolt"));
        // Idempotent: a second pass changes nothing.
        assert!(clamp(&mut hero, Fixed64::from_raw(410)).is_empty());
        // Rank 4 of R (cd 7) caps at 4.2 under 40%.
        hero.ability_levels.insert("a34_frost_breath".into(), 4);
        assert_eq!(clamp(&mut hero, Fixed64::from_raw(410)).len(), 1);
        assert_eq!(hero.get_cooldown("a34_frost_breath"), cap(7));
    }

    #[test]
    fn settle_reads_the_strongest_cap_buff_and_ignores_malformed_rows() {
        use specs::Builder;
        let mut w = World::new();
        w.register::<Hero>();
        w.insert(crate::runtime::BuffStore::default());
        let mut hero = Hero::default();
        hero.abilities = vec!["a34_frost_bolt".into()];
        hero.ability_levels.insert("a34_frost_bolt".into(), 1);
        hero.start_cooldown("a34_frost_bolt", s(14));
        let e = w.create_entity().with(hero).build();
        {
            let mut buffs = w.write_resource::<crate::runtime::BuffStore>();
            buffs.add(e, "bad", s(5), json!({ (CAP_KEY): { "fraction": 2048, "scope": "basic" } }));
            buffs.add(e, "weak", s(5), json!({ (CAP_KEY): cap_value(Fixed64::from_raw(410)) }));
            buffs.add(e, "strong", s(5), json!({ (CAP_KEY): cap_value(Fixed64::from_raw(614)) }));
        }
        assert_eq!(settle(&mut w).len(), 1);
        assert_eq!(w.read_storage::<Hero>().get(e).unwrap().get_cooldown("a34_frost_bolt"),
            Fixed64::from_raw(14 * 1024 * (1024 - 614) / 1024));
        assert!(settle(&mut w).is_empty());
        w.write_resource::<crate::runtime::BuffStore>().remove(e, "strong");
        w.write_resource::<crate::runtime::BuffStore>().remove(e, "weak");
        assert!(settle(&mut w).is_empty(), "the out-of-range fraction is ignored");
    }
}
