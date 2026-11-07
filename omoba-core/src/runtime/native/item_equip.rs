//! v3 item `passive.triggers.on_created` / `on_removed` (and statuses with `remove_on = unequip`):
//! the item script hears each copy of its item a unit gains or loses.
//!
//! One ledger, not a hook in every inventory path: [`changes`] compares each unit's inventory
//! with what it held when last read and reports one change per copy gained or lost, in canonical
//! (unit, item id) order: losses before gains for a unit, so a swap of the same item is quiet.
//! Buying, selling, combining, dropping, picking up and a script's own inventory change all show
//! up the same way. A unit that is gone loses its row without a change (its statuses went with
//! it). Dispatch then calls the item's `AbilityScript::on_equip` / `on_unequip` (scripting).
use std::collections::BTreeMap;

use specs::{Entity, Join, World, WorldExt};

use super::comp::Inventory;

/// Item ids (inventory ids) and copies each unit held when last read.
#[derive(Clone, Debug, Default)]
pub struct EquipLedger(pub BTreeMap<Entity, BTreeMap<String, u32>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub owner: Entity,
    /// The inventory item id.
    pub item_id: String,
    pub equipped: bool,
}

/// Whether [`changes`] has anything to compare: a unit holds an item, or the ledger remembers one
/// (a loss since the last read). The script dispatch never skips a tick that may hold a change (E413).
pub fn pending(world: &World) -> bool {
    world.try_fetch::<EquipLedger>().is_some_and(|ledger| !ledger.0.is_empty())
        || (world.has_value::<specs::storage::MaskedStorage<Inventory>>()
            && world.read_storage::<Inventory>().join().any(|inventory| inventory.items().next().is_some()))
}

/// The copies gained and lost since the last call, and the ledger brought up to date.
pub fn changes(world: &mut World) -> Vec<Change> {
    // A world without inventories (a tower or test world) holds no items.
    if !world.has_value::<specs::storage::MaskedStorage<Inventory>>() { return Vec::new(); }
    let now: BTreeMap<Entity, BTreeMap<String, u32>> = {
        let (entities, inventories) = (world.entities(), world.read_storage::<Inventory>());
        (&entities, &inventories).join().map(|(entity, inventory)| {
            let mut held = BTreeMap::new();
            for (_, item) in inventory.items() { *held.entry(item.item_id.clone()).or_insert(0) += 1; }
            (entity, held)
        }).filter(|(_, held)| !held.is_empty()).collect()
    };
    if world.try_fetch::<EquipLedger>().is_none() {
        if now.is_empty() { return Vec::new(); }
        world.insert(EquipLedger::default());
    }
    let mut ledger = world.write_resource::<EquipLedger>();
    let alive = world.entities();
    let mut out = Vec::new();
    let owners: std::collections::BTreeSet<Entity> = ledger.0.keys().chain(now.keys()).copied().collect();
    let mut owners: Vec<Entity> = owners.into_iter().filter(|e| alive.is_alive(*e)).collect();
    owners.sort_by_key(|e| (e.id(), e.gen().id()));
    for owner in owners {
        let empty = BTreeMap::new();
        let (before, after) = (ledger.0.get(&owner).unwrap_or(&empty), now.get(&owner).unwrap_or(&empty));
        for (item, count) in before {
            let left = after.get(item).copied().unwrap_or(0);
            for _ in left..*count { out.push(Change { owner, item_id: item.clone(), equipped: false }); }
        }
        for (item, count) in after {
            let had = before.get(item).copied().unwrap_or(0);
            for _ in had..*count { out.push(Change { owner, item_id: item.clone(), equipped: true }); }
        }
    }
    ledger.0 = now;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::comp::ItemInstance;
    use specs::Builder;

    /// A world that never registered inventories (tower projectile tests) has no equip changes.
    #[test]
    fn a_world_without_inventories_has_no_changes() {
        let mut world = World::new();
        world.create_entity().build();
        assert!(changes(&mut world).is_empty());
    }

    fn put(world: &mut World, owner: Entity, slot: usize, item: Option<&str>) {
        world.write_storage::<Inventory>().get_mut(owner).unwrap().slots[slot] =
            item.map(|id| ItemInstance { item_id: id.into(), cooldown_remaining: 0.0 });
    }

    #[test]
    fn each_copy_gained_or_lost_is_one_change_and_a_gone_unit_is_quiet() {
        let mut world = World::new();
        world.register::<Inventory>();
        let hero = world.create_entity().with(Inventory::new()).build();
        assert!(changes(&mut world).is_empty());
        put(&mut world, hero, 0, Some("hannya"));
        put(&mut world, hero, 1, Some("hannya"));
        let gained = changes(&mut world);
        assert_eq!(gained, vec![Change { owner: hero, item_id: "hannya".into(), equipped: true }; 2], "two copies");
        assert!(changes(&mut world).is_empty(), "nothing new");
        put(&mut world, hero, 0, None);
        put(&mut world, hero, 2, Some("boots"));
        assert_eq!(changes(&mut world), vec![Change { owner: hero, item_id: "hannya".into(), equipped: false },
            Change { owner: hero, item_id: "boots".into(), equipped: true }], "losses first, then gains");
        put(&mut world, hero, 2, None);
        put(&mut world, hero, 3, Some("boots"));
        assert!(changes(&mut world).is_empty(), "moving a copy between slots is no change");
        world.delete_entity(hero).unwrap();
        world.maintain();
        assert!(changes(&mut world).is_empty(), "a gone unit is quiet");
    }
}
