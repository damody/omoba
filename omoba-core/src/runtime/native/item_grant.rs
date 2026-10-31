//! Scripted inventory changes (E561): v3 `give_item`, `remove_item`. A script sends
//! `add_stat_buff(unit, BUFF, 1, payload)`; the host changes the unit's inventory, never stores the
//! row. The item stats follow through `ItemEffects.dirty` (item_tick) and the owner team reads the
//! inventory from the per-tick committed economy, like a purchase. A scripted change closes the
//! owner's open shop undo (its purchases may no longer be in the bag).
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

use omb_script_abi::types::EntityHandle;
use omoba_sim::Fixed64;

use super::ability_runtime::buff_store::PERMANENT_BUFF_DURATION;
use super::comp::{Inventory, ItemEffects, ItemInstance, PlayerOwner, INVENTORY_SLOTS};
use super::unit_var::VarValue;
use crate::runtime::BuffStore;

pub const KEY: &str = "__item_grant_v1";
pub const BUFF: &str = "__item_grant";

/// What a request does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// The first free slot gets a new instance of the item with these instance vars (v3
    /// `give_item.vars`, E571); nothing when the bag is full.
    Give(String, Vec<(String, VarValue)>),
    /// The first slot holding the item loses it.
    RemoveItem(String),
    /// The slot loses whatever it holds.
    RemoveSlot(usize),
}

pub fn give_payload(item: &str) -> Value { json!({ (KEY): { "give": item } }) }
/// `give_payload` whose new instance holds `vars` (v3 `give_item.vars`, E571).
pub fn give_with_vars_payload(item: &str, vars: &[(&str, VarValue)]) -> Value {
    let vars: serde_json::Map<String, Value> = vars.iter().map(|(key, value)| ((*key).to_owned(), var_json(*value))).collect();
    json!({ (KEY): { "give": item, "vars": vars } })
}

/// An item instance's v3 vars (`scope = "item_instance"`) are its holder's unit vars
/// `item:<slot>:<var>` (unit_var rows, replicated and hashed like any other); a scripted give
/// or removal of the slot clears the slot's rows first. Inventory slots never move.
pub fn instance_var_key(slot: usize, var: &str) -> String { format!("item:{slot}:{var}") }

/// A v3 `item_name` value as a number (E571): the catalog row's position + 1, 0 for none.
pub fn item_name_number(id: &str) -> Fixed64 {
    catalog_id(id).and_then(|id| omoba_template_ids::MOBA_ITEM_CATALOG.iter().position(|row| row.id == id))
        .map_or(Fixed64::ZERO, |n| Fixed64::from_i32(i32::try_from(n + 1).unwrap_or(0)))
}
/// The v3 source id an [`item_name_number`] names.
pub fn item_name_of(number: Fixed64) -> Option<&'static str> {
    let n = usize::try_from(number.raw() >> 10).ok()?.checked_sub(1)?;
    omoba_template_ids::MOBA_ITEM_CATALOG.get(n).and_then(|row| source_id(row.id))
}

fn var_json(value: VarValue) -> Value {
    match value { VarValue::Num(n) => json!({ "n": n.raw() }), VarValue::Unit(u) => json!({ "u": [u.id, u.gen] }) }
}
fn var_of(value: &Value) -> Option<VarValue> {
    let row = value.as_object().filter(|row| row.len() == 1)?;
    if let Some(n) = row.get("n") { return Some(VarValue::Num(Fixed64::from_raw(n.as_i64()?))); }
    let u = row.get("u")?.as_array().filter(|u| u.len() == 2)?;
    Some(VarValue::Unit(EntityHandle { id: u32::try_from(u[0].as_u64()?).ok()?, gen: u32::try_from(u[1].as_u64()?).ok()? }))
}
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
    if let (Some(item), Some(vars), 2) = (row.get("give"), row.get("vars"), row.len()) {
        let item = catalog_id(item.as_str()?)?;
        let vars = vars.as_object()?.iter().map(|(key, value)| Some((key.clone(), var_of(value)?)))
            .collect::<Option<Vec<_>>>()?;
        // each var must be a valid unit var key in every slot, and within one unit's bound
        if vars.len() > super::unit_var::MAX_PER_UNIT / INVENTORY_SLOTS
            || !vars.iter().all(|(key, _)| super::unit_var::valid_key(&instance_var_key(INVENTORY_SLOTS - 1, key))) { return None; }
        return Some(Request::Give(item.to_owned(), vars));
    }
    if row.len() != 1 { return None; }
    let (key, value) = row.iter().next()?;
    match key.as_str() {
        "give" => value.as_str().and_then(catalog_id).map(|id| Request::Give(id.to_owned(), Vec::new())),
        "remove" => value.as_str().and_then(catalog_id).map(|id| Request::RemoveItem(id.to_owned())),
        "remove_slot" => value.as_u64().and_then(|slot| usize::try_from(slot).ok()).filter(|slot| *slot < INVENTORY_SLOTS)
            .map(Request::RemoveSlot),
        _ => None,
    }
}

/// Applies a request to `unit`'s inventory. Returns whether the inventory changed.
pub fn apply(world: &mut World, unit: Entity, buff_id: &str, payload: &Value) -> bool {
    let Some(request) = decode(buff_id, payload) else { return false };
    let (slot, vars) = {
        let mut inventories = world.write_storage::<Inventory>();
        let Some(inventory) = inventories.get_mut(unit) else { return false };
        let slot = match &request {
            Request::Give(..) => inventory.first_free_slot(),
            Request::RemoveItem(id) => inventory.find_item(id),
            Request::RemoveSlot(slot) => inventory.slots[*slot].is_some().then_some(*slot),
        };
        match (slot, request) {
            (Some(slot), Request::Give(id, vars)) => {
                inventory.slots[slot] = Some(ItemInstance { item_id: id, cooldown_remaining: 0.0 });
                (slot, vars)
            }
            (Some(slot), _) => { inventory.slots[slot] = None; (slot, Vec::new()) }
            (None, _) => return false,
        }
    };
    if let Some(mut buffs) = world.try_fetch_mut::<BuffStore>() {
        let prefix = format!("{}{}", super::unit_var::PREFIX, instance_var_key(slot, ""));
        let stale: Vec<String> = buffs.iter_for(unit).filter(|(id, _)| id.starts_with(&prefix)).map(|(id, _)| id.to_owned()).collect();
        for id in stale { buffs.remove(unit, &id); }
        for (key, value) in vars {
            // kept across the holder's death, like the item (unit_var `hero` rows)
            let spec = super::unit_var::Spec { key: instance_var_key(slot, &key), value, hero: true };
            buffs.add(unit, &spec.buff_id(), PERMANENT_BUFF_DURATION, spec.payload());
        }
    }
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

    /// E571: a given instance's vars are its slot's unit vars; the slot's next give or removal
    /// clears them, and an item name round-trips through its number.
    #[test]
    fn a_given_instance_keeps_its_vars_in_its_slot() {
        let item = omoba_template_ids::MOBA_ITEM_CATALOG[0].id;
        let mut world = World::new();
        world.register::<Inventory>(); world.register::<ItemEffects>(); world.register::<PlayerOwner>();
        world.insert(BuffStore::default());
        let unit = world.create_entity().with(Inventory::default()).with(ItemEffects::default()).build();
        let var = |world: &World, key: &str| {
            let id = format!("{}{key}", super::super::unit_var::PREFIX);
            world.read_resource::<BuffStore>().get(unit, &id).map(|entry| entry.payload.clone())
                .and_then(|payload| super::super::unit_var::Spec::from_buff(&id, &payload)).map(|spec| spec.value)
        };
        let named = item_name_number(item);
        let damage = VarValue::Num(Fixed64::from_i32(75));
        assert!(apply(&mut world, unit, BUFF, &give_with_vars_payload(item, &[("c06e_damage", damage), ("c06e_swapped", VarValue::Num(named))])));
        assert_eq!(var(&world, "item:0:c06e_damage"), Some(damage));
        assert_eq!(item_name_of(named), source_id(item));
        assert_eq!((item_name_number("no_such_item"), item_name_of(Fixed64::ZERO)), (Fixed64::ZERO, None));
        assert!(apply(&mut world, unit, BUFF, &remove_slot_payload(0)));
        assert_eq!(var(&world, "item:0:c06e_damage"), None, "removal clears the slot's vars");
        assert!(apply(&mut world, unit, BUFF, &give_with_vars_payload(item, &[("c06e_damage", damage)])));
        assert!(apply(&mut world, unit, BUFF, &remove_item_payload(item)));
        assert!(apply(&mut world, unit, BUFF, &give_payload(item)));
        assert_eq!(var(&world, "item:0:c06e_damage"), None, "a plain give starts with none");
        // A var that is not a valid unit var key refuses the whole give.
        assert!(!apply(&mut world, unit, BUFF, &give_with_vars_payload(item, &[("Bad Key", damage)])));
    }
}
