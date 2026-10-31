//! Scripted inventory changes (E561): v3 `give_item`, `remove_item`. A script sends
//! `add_stat_buff(unit, BUFF, 1, payload)`; the host changes the unit's inventory, never stores the
//! row. The item stats follow through `ItemEffects.dirty` (item_tick) and the owner team reads the
//! inventory from the per-tick committed economy, like a purchase. A scripted change closes the
//! owner's open shop undo (its purchases may no longer be in the bag).
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

use super::comp::{Inventory, ItemEffects, ItemInstance, PlayerOwner, INVENTORY_SLOTS};

pub const KEY: &str = "__item_grant_v1";
pub const BUFF: &str = "__item_grant";

/// What a request does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// The first free slot gets a new instance of the item; nothing when the bag is full.
    Give(String),
    /// The first slot holding the item loses it.
    RemoveItem(String),
    /// The slot loses whatever it holds.
    RemoveSlot(usize),
}

pub fn give_payload(item: &str) -> Value { json!({ (KEY): { "give": item } }) }
pub fn remove_item_payload(item: &str) -> Value { json!({ (KEY): { "remove": item } }) }
pub fn remove_slot_payload(slot: usize) -> Value { json!({ (KEY): { "remove_slot": slot } }) }

/// The catalog id of an item named by its catalog id or by its v3 source id (`item_flash_ring`,
/// the catalog row's `source_ref` tail; E569). The committed economy carries catalog ids only.
pub fn catalog_id(id: &str) -> Option<&'static str> {
    omoba_template_ids::MOBA_ITEM_CATALOG.iter().find(|item| item.id == id
        || item.source_ref.and_then(|r| r.rsplit_once(':')).is_some_and(|(_, source)| source == id)).map(|item| item.id)
}

/// The v3 source id of a catalog item (E569); the catalog id itself when it has none.
pub fn source_id(catalog: &str) -> Option<&'static str> {
    let item = omoba_template_ids::MOBA_ITEM_CATALOG.iter().find(|item| item.id == catalog)?;
    Some(item.source_ref.and_then(|r| r.rsplit_once(':')).map_or(item.id, |(_, source)| source))
}

pub fn decode(buff_id: &str, payload: &Value) -> Option<Request> {
    if buff_id != BUFF || payload.as_object()?.len() != 1 { return None; }
    let row = payload.get(KEY)?.as_object()?;
    if row.len() != 1 { return None; }
    let (key, value) = row.iter().next()?;
    match key.as_str() {
        "give" => value.as_str().and_then(catalog_id).map(|id| Request::Give(id.to_owned())),
        "remove" => value.as_str().and_then(catalog_id).map(|id| Request::RemoveItem(id.to_owned())),
        "remove_slot" => value.as_u64().and_then(|slot| usize::try_from(slot).ok()).filter(|slot| *slot < INVENTORY_SLOTS)
            .map(Request::RemoveSlot),
        _ => None,
    }
}

/// Applies a request to `unit`'s inventory. Returns whether the inventory changed.
pub fn apply(world: &mut World, unit: Entity, buff_id: &str, payload: &Value) -> bool {
    let Some(request) = decode(buff_id, payload) else { return false };
    let changed = {
        let mut inventories = world.write_storage::<Inventory>();
        let Some(inventory) = inventories.get_mut(unit) else { return false };
        let slot = match &request {
            Request::Give(_) => inventory.first_free_slot(),
            Request::RemoveItem(id) => inventory.find_item(id),
            Request::RemoveSlot(slot) => inventory.slots[*slot].is_some().then_some(*slot),
        };
        match (slot, request) {
            (Some(slot), Request::Give(id)) => { inventory.slots[slot] = Some(ItemInstance { item_id: id, cooldown_remaining: 0.0 }); true }
            (Some(slot), _) => { inventory.slots[slot] = None; true }
            (None, _) => false,
        }
    };
    if !changed { return false; }
    if let Some(effects) = world.write_storage::<ItemEffects>().get_mut(unit) { effects.dirty = true; }
    let owner = world.read_storage::<PlayerOwner>().get(unit).map(|owner| owner.player_id);
    if let Some(player) = owner { super::shop::retire_owner_shop_undo(world, player); }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    #[test]
    fn a_script_gives_removes_and_never_invents_an_item() {
        let item = omoba_template_ids::MOBA_ITEM_CATALOG[0].id;
        let mut world = World::new();
        world.register::<Inventory>(); world.register::<ItemEffects>(); world.register::<PlayerOwner>();
        let unit = world.create_entity().with(Inventory::default()).with(ItemEffects::default()).build();
        assert!(apply(&mut world, unit, BUFF, &give_payload(item)));
        assert_eq!(world.read_storage::<Inventory>().get(unit).unwrap().find_item(item), Some(0));
        assert!(world.read_storage::<ItemEffects>().get(unit).unwrap().dirty, "its stats follow");
        // An unknown id, another buff or a malformed row changes nothing.
        assert!(!apply(&mut world, unit, BUFF, &give_payload("no_such_item")));
        assert!(!apply(&mut world, unit, "other", &give_payload(item)));
        assert!(!apply(&mut world, unit, BUFF, &json!({ (KEY): { "give": item, "extra": 1 } })));
        // A full bag takes nothing more.
        for _ in 1..INVENTORY_SLOTS { assert!(apply(&mut world, unit, BUFF, &give_payload(item))); }
        assert!(!apply(&mut world, unit, BUFF, &give_payload(item)));
        assert!(apply(&mut world, unit, BUFF, &remove_slot_payload(3)));
        assert!(world.read_storage::<Inventory>().get(unit).unwrap().slots[3].is_none());
        assert!(!apply(&mut world, unit, BUFF, &remove_slot_payload(3)), "an empty slot");
        assert!(apply(&mut world, unit, BUFF, &remove_item_payload(item)));
        assert_eq!(world.read_storage::<Inventory>().get(unit).unwrap().find_item(item), Some(1));
        assert!(!apply(&mut world, unit, BUFF, &remove_slot_payload(INVENTORY_SLOTS)), "no such slot");
        // E569: a v3 source id names the same catalog item, and maps back.
        if let Some(row) = omoba_template_ids::MOBA_ITEM_CATALOG.iter().find(|row| row.source_ref.is_some()) {
            let source = source_id(row.id).unwrap();
            assert_eq!((catalog_id(source), catalog_id(row.id)), (Some(row.id), Some(row.id)));
            assert!(apply(&mut world, unit, BUFF, &give_payload(source)) || world.read_storage::<Inventory>().get(unit).unwrap().first_free_slot().is_none());
        }
        assert_eq!(catalog_id("no_such_item"), None);
    }
}
