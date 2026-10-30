//! 非英雄單位（召喚物）的魔力：模板的 `mana`／`mana_regen` 給它一個 [`UnitMana`]（出生滿魔）。
//!
//! A hero keeps its pool on `Hero::mana_pool`; any other unit with mana holds this component. Every
//! reader and writer of a unit's mana goes through [`pool`] / [`with_pool_mut`], so burns, drains,
//! restores, sets and reads reach a summon's mana like a hero's. A script's spend or restore on a
//! summon settles as a delta ([`Outcome::ScriptUnitManaDelta`]) in serial order, so two hooks in one
//! tick never overwrite each other. The authority regenerates it ([`regenerate`]) and discloses it to
//! the unit's team like a hero's mana: the JSON component [`SCHEMA_ID`] and a CommittedMana fact.
use omoba_sim::Fixed64;
use serde::{Deserialize, Serialize};
use specs::storage::VecStorage;
use specs::{Component, Entity, Join, World, WorldExt};

use super::ability_runtime::ManaPool;
use super::comp::{CProperty, Hero};

/// Disclosed JSON component of a non-hero unit's mana (the unit's own team).
pub const SCHEMA_ID: u32 = 0x464f471a;

/// A non-hero unit's mana and its own regeneration per second (buffs add to it).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitMana {
    pub pool: ManaPool,
    pub regen: Fixed64,
}

impl Component for UnitMana {
    type Storage = VecStorage<Self>;
}

impl UnitMana {
    /// A template's mana, full at birth; none without mana.
    pub fn of_template(mana: Fixed64, regen: Fixed64) -> Option<Self> {
        (mana > Fixed64::ZERO).then(|| ManaPool::full(mana).ok()).flatten().map(|pool| Self { pool, regen: regen.max(Fixed64::ZERO) })
    }
}

/// Whether this world holds unit mana at all (a bare test world may not register it).
pub fn registered(world: &World) -> bool {
    world.has_value::<specs::storage::MaskedStorage<UnitMana>>()
}

/// The unit's mana pool: a hero's, else a non-hero unit's.
pub fn pool(world: &World, entity: Entity) -> Option<ManaPool> {
    world.read_storage::<Hero>().get(entity).and_then(|hero| hero.mana_pool.clone())
        .or_else(|| registered(world).then(|| world.read_storage::<UnitMana>().get(entity).map(|mana| mana.pool.clone())).flatten())
}

/// Changes the unit's mana pool in place (a hero's, else a non-hero unit's); None without one.
pub fn with_pool_mut<R>(world: &World, entity: Entity, change: impl FnOnce(&mut ManaPool) -> R) -> Option<R> {
    if let Some(pool) = world.write_storage::<Hero>().get_mut(entity).and_then(|hero| hero.mana_pool.as_mut()) {
        return Some(change(pool));
    }
    if !registered(world) { return None; }
    world.write_storage::<UnitMana>().get_mut(entity).map(|mana| change(&mut mana.pool))
}

/// A script's spend (negative) or restore (positive) on a non-hero unit; returns what changed.
pub fn apply_delta(world: &World, entity: Entity, amount: Fixed64) -> Fixed64 {
    if !registered(world) { return Fixed64::ZERO; }
    let mut units = world.write_storage::<UnitMana>();
    let Some(mana) = units.get_mut(entity) else { return Fixed64::ZERO };
    if amount >= Fixed64::ZERO { mana.pool.restore(amount).unwrap_or(Fixed64::ZERO) }
    else { -mana.pool.drain(-amount).unwrap_or(Fixed64::ZERO) }
}

/// Natural regeneration of every living non-hero unit with mana over `dt` (its own rate plus
/// the mana regen buffs on it, `rules::regen::mana_per_second`).
pub fn regenerate(world: &World, dt: Fixed64) {
    if dt <= Fixed64::ZERO || !registered(world) { return; }
    let buffs = world.read_resource::<crate::runtime::BuffStore>();
    let stats = crate::runtime::UnitStats::from_refs(&buffs, false);
    let properties = world.read_storage::<CProperty>();
    for (entity, mana) in (&world.entities(), &mut world.write_storage::<UnitMana>()).join() {
        if !properties.get(entity).is_some_and(|property| property.hp > Fixed64::ZERO) { continue; }
        let rate = super::rules::regen::mana_per_second(&stats, mana.regen, mana.pool.maximum(), entity).max(Fixed64::ZERO);
        let _ = mana.pool.regenerate(rate, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    #[test]
    fn a_summon_holds_template_mana_that_regenerates_and_takes_deltas_and_sets() {
        let mut world = World::new();
        world.register::<Hero>();
        world.register::<UnitMana>();
        world.register::<CProperty>();
        world.insert(crate::runtime::BuffStore::new());
        assert!(UnitMana::of_template(Fixed64::ZERO, Fixed64::ONE).is_none(), "no mana, no pool");
        let mana = UnitMana::of_template(Fixed64::from_i32(400), Fixed64::from_i32(10)).unwrap();
        assert_eq!(mana.pool.current(), Fixed64::from_i32(400), "full at birth");
        let life = CProperty { hp: Fixed64::ONE, mhp: Fixed64::ONE, msd: Fixed64::ZERO, def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO };
        let summon = world.create_entity().with(mana).with(life).build();
        assert_eq!(apply_delta(&world, summon, Fixed64::from_i32(-150)), Fixed64::from_i32(-150));
        assert_eq!(apply_delta(&world, summon, Fixed64::from_i32(-300)), Fixed64::from_i32(-250), "never below zero");
        regenerate(&world, Fixed64::from_i32(2));
        assert_eq!(pool(&world, summon).unwrap().current(), Fixed64::from_i32(20), "10 per second");
        assert_eq!(with_pool_mut(&world, summon, |pool| pool.restore(Fixed64::from_i32(1000)).unwrap()), Some(Fixed64::from_i32(380)));
        assert_eq!(pool(&world, summon).unwrap().current(), Fixed64::from_i32(400));
        let plain = world.create_entity().build();
        assert!(pool(&world, plain).is_none() && with_pool_mut(&world, plain, |_| ()).is_none());
        assert_eq!(apply_delta(&world, plain, Fixed64::ONE), Fixed64::ZERO);
    }
}
