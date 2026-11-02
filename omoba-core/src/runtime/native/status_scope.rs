//! v3 `status.scope = "hero"` / `"permanent"`: a status the holder hero keeps across its death
//! (原作 RemoveOnDeath false). The generated code marks such a row with [`KEY`]; when the hero dies,
//! moba_match saves those rows with their expiry on the match clock, and the respawned hero gets
//! back the ones that have not run out (a status keeps counting down while its hero is dead, as in
//! the original). A row that never expires comes back unchanged. Other units lose every status
//! when they die (`BuffStore::remove_all_for`).
use omoba_sim::Fixed64;
use serde_json::Value;
use specs::Entity;
use crate::runtime::BuffStore;

pub const KEY: &str = "__status_scope_v1";
/// Remaining durations at or past this never run out (the kit's FOREVER and the BuffStore's permanent).
const NEVER: Fixed64 = Fixed64::from_raw(i32::MAX as i64);

/// The payload value of a kept scope; None for `unit` (the default) and unknown names.
pub fn marker(scope: &str) -> Option<&'static str> {
    match scope { "hero" => Some("hero"), "permanent" => Some("permanent"), _ => None }
}

fn kept(payload: &Value) -> bool {
    payload.get(KEY).and_then(Value::as_str).is_some_and(|scope| marker(scope).is_some())
}

/// One status kept across a death: its row, payload and the match time it runs out (None: never).
#[derive(Clone, Debug, PartialEq)]
pub struct KeptStatus { pub id: String, pub payload: Value, pub until: Option<Fixed64>, pub body: Entity }

/// The kept-scope rows of `entity` at match time `now`, in buff id order.
pub fn persisting(buffs: &BuffStore, entity: Entity, now: Fixed64) -> Vec<KeptStatus> {
    let mut rows: Vec<KeptStatus> = buffs.iter_for(entity).filter(|(_, entry)| kept(&entry.payload))
        .map(|(id, entry)| KeptStatus { id: id.to_owned(), payload: entry.payload.clone(), body: entity,
            until: (entry.remaining < NEVER).then(|| now + entry.remaining.max(Fixed64::ZERO)) })
        .collect();
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    rows
}

/// A removal aimed at the dead body after its rows were saved: an on_death trigger runs then
/// (FC-102: A03's melee form came back on the respawn, E657).
pub fn forget(rows: &mut Vec<KeptStatus>, body: Entity, id: &str) {
    rows.retain(|row| row.body != body || row.id != id);
}

/// Put the saved rows that have not run out by `now` on the hero's new life.
pub fn restore(buffs: &mut BuffStore, entity: Entity, rows: &[KeptStatus], now: Fixed64) {
    for row in rows {
        let remaining = match row.until { None => crate::runtime::native::ability_runtime::buff_store::PERMANENT_BUFF_DURATION,
            Some(until) => until - now };
        if remaining > Fixed64::ZERO && kept(&row.payload) { buffs.add(entity, &row.id, remaining, row.payload.clone()); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn a_removal_aimed_at_the_dead_body_takes_only_that_row_of_that_body() {
        let mut world = World::new();
        let (dead, other) = (world.create_entity().build(), world.create_entity().build());
        let mut buffs = BuffStore::default();
        for hero in [dead, other] {
            buffs.add(hero, "melee", Fixed64::from_i32(30), json!({ (KEY): "hero" }));
            buffs.add(hero, "oath", Fixed64::from_i32(30), json!({ (KEY): "hero" }));
        }
        let mut rows = persisting(&buffs, dead, Fixed64::ZERO);
        let mut others = persisting(&buffs, other, Fixed64::ZERO);
        forget(&mut rows, dead, "melee");
        forget(&mut others, dead, "melee");
        assert_eq!(rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(), ["oath"]);
        assert_eq!(others.len(), 2, "another hero's row with that id stays");
    }

    #[test]
    fn hero_and_permanent_statuses_outlive_a_death_and_keep_counting_down() {
        let mut world = World::new();
        let (dead, born) = (world.create_entity().build(), world.create_entity().build());
        let s = Fixed64::from_i32;
        let mut buffs = BuffStore::default();
        buffs.add(dead, "oath", s(30), json!({ (KEY): "hero", "damage_bonus": 1024 }));
        buffs.add(dead, "brand", s(5), json!({ (KEY): "permanent" }));
        buffs.add(dead, "forever", crate::runtime::native::ability_runtime::buff_store::PERMANENT_BUFF_DURATION,
            json!({ (KEY): "hero" }));
        buffs.add(dead, "slow", s(30), json!({ "move_speed_pct": -100 }));
        buffs.add(dead, "odd", s(30), json!({ (KEY): "unit" }));
        let rows = persisting(&buffs, dead, s(100));
        assert_eq!(rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(), ["brand", "forever", "oath"]);
        // Respawned 10 s later: `brand` ran out, `oath` has 20 s left, `forever` never ends.
        restore(&mut buffs, born, &rows, s(110));
        assert!(!buffs.has(born, "brand") && !buffs.has(born, "slow") && !buffs.has(born, "odd"));
        assert_eq!(buffs.get(born, "oath").map(|row| row.remaining), Some(s(20)));
        assert_eq!(buffs.get(born, "oath").map(|row| row.payload["damage_bonus"].clone()), Some(json!(1024)));
        assert_eq!(buffs.get(born, "forever").map(|row| row.remaining),
            Some(crate::runtime::native::ability_runtime::buff_store::PERMANENT_BUFF_DURATION));
        assert_eq!((marker("hero"), marker("permanent"), marker("unit"), marker("item_instance")),
            (Some("hero"), Some("permanent"), None, None));
    }
}
