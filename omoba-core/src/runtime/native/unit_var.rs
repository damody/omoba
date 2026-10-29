//! unit_var：v3 `unit_state` 的單位狀態變數（產生的技能 Rust 裡的 `set_state`／`state.x`）。
//!
//! A script writes one by sending the unit an ordinary permanent buff
//! `add_stat_buff(unit, spec.buff_id(), PERMANENT, spec.payload())` and clears it with
//! `remove_buff`; it reads the row back through `GameWorld::get_buff_payload`. The value lives in
//! the unit's `BuffStore` row, so it is part of the replicated, hashed world like any other
//! marker, and a script DLL keeps nothing between calls.
//!
//! Lifetime: a unit's rows go with its buffs (death clears them). A `hero` row (v3 scope `hero`,
//! 跨死亡保留) is saved from the dying hero by [`persisting`] and put back on the next life by
//! [`restore`] (moba_match hero retire / respawn).
use omb_script_abi::types::EntityHandle;
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::Entity;

use crate::runtime::BuffStore;

pub const KEY: &str = "__unit_var_v1";
/// Buff id prefix: `unit_var:<key>`.
pub const PREFIX: &str = "unit_var:";
pub const MAX_KEY_LEN: usize = 96;
/// Rows one unit may hold (a bound on what scripts can grow).
pub const MAX_PER_UNIT: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VarValue {
    Num(Fixed64),
    Unit(EntityHandle),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    /// `<package>:<var>`, or a registry name shared between documents.
    pub key: String,
    pub value: VarValue,
    /// Kept across the hero's death (v3 scope `hero`).
    pub hero: bool,
}

pub fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= MAX_KEY_LEN
        && key.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b':')
}

impl Spec {
    pub fn checked(self) -> Option<Self> { valid_key(&self.key).then_some(self) }
    pub fn buff_id(&self) -> String { format!("{PREFIX}{}", self.key) }
    pub fn payload(&self) -> Value {
        let row = match self.value {
            VarValue::Num(v) => json!({ "n": v.raw(), "hero": self.hero }),
            VarValue::Unit(u) => json!({ "u": [u.id, u.gen], "hero": self.hero }),
        };
        json!({ (KEY): row })
    }
    /// Strict: only the exact id and payload this module writes.
    pub fn from_buff(buff_id: &str, payload: &Value) -> Option<Self> {
        let key = buff_id.strip_prefix(PREFIX)?;
        if payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 2 { return None; }
        let hero = row.get("hero")?.as_bool()?;
        let value = if let Some(n) = row.get("n") {
            VarValue::Num(Fixed64::from_raw(n.as_i64()?))
        } else {
            let u = row.get("u")?.as_array()?;
            if u.len() != 2 { return None; }
            VarValue::Unit(EntityHandle { id: u32::try_from(u[0].as_u64()?).ok()?, gen: u32::try_from(u[1].as_u64()?).ok()? })
        };
        Self { key: key.to_owned(), value, hero }.checked()
    }
}

/// Admission of a write: a permanent row with a valid key and payload, within the unit's bound
/// (an existing row of the same key is a rewrite and always fits).
pub fn admitted(buffs: &BuffStore, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> bool {
    if duration != crate::runtime::native::ability_runtime::buff_store::PERMANENT_BUFF_DURATION { return false; }
    if Spec::from_buff(buff_id, payload).is_none() { return false; }
    buffs.has(target, buff_id)
        || buffs.iter_for(target).filter(|(id, _)| id.starts_with(PREFIX)).count() < MAX_PER_UNIT
}

/// The `hero` rows of `entity`, in buff id order, to keep across its death.
pub fn persisting(buffs: &BuffStore, entity: Entity) -> Vec<(String, Value)> {
    let mut rows: Vec<(String, Value)> = buffs.iter_for(entity)
        .filter(|(id, entry)| Spec::from_buff(id, &entry.payload).is_some_and(|s| s.hero))
        .map(|(id, entry)| (id.to_owned(), entry.payload.clone()))
        .collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}

/// Put saved `hero` rows on the hero's new life.
pub fn restore(buffs: &mut BuffStore, entity: Entity, rows: &[(String, Value)]) {
    for (id, payload) in rows {
        if Spec::from_buff(id, payload).is_some() {
            buffs.add(entity, id, crate::runtime::native::ability_runtime::buff_store::PERMANENT_BUFF_DURATION, payload.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::native::ability_runtime::buff_store::PERMANENT_BUFF_DURATION as P;
    use specs::{Builder, World, WorldExt};

    fn num(key: &str, v: i64, hero: bool) -> Spec { Spec { key: key.into(), value: VarValue::Num(Fixed64::from_raw(v)), hero } }

    #[test]
    fn payload_round_trips_strictly() {
        let s = num("c08:c08t_use_count", 3072, true);
        assert_eq!(Spec::from_buff(&s.buff_id(), &s.payload()), Some(s.clone()));
        let u = Spec { key: "a28:a28e_cur".into(), value: VarValue::Unit(EntityHandle { id: 7, gen: 2 }), hero: false };
        assert_eq!(Spec::from_buff(&u.buff_id(), &u.payload()), Some(u.clone()));
        assert!(Spec::from_buff("unit_var:Bad Key", &s.payload()).is_none(), "key characters");
        assert!(Spec::from_buff("other:c08:x", &s.payload()).is_none(), "prefix");
        let mut extra = s.payload();
        extra[KEY]["x"] = json!(1);
        assert!(Spec::from_buff(&s.buff_id(), &extra).is_none(), "no extra fields");
    }

    #[test]
    fn admission_is_permanent_valid_and_bounded() {
        let mut world = World::new();
        let e = world.create_entity().build();
        let mut buffs = BuffStore::default();
        let s = num("a:x", 1, false);
        assert!(admitted(&buffs, e, &s.buff_id(), P, &s.payload()));
        assert!(!admitted(&buffs, e, &s.buff_id(), Fixed64::from_i32(5), &s.payload()), "permanent only");
        for i in 0..MAX_PER_UNIT {
            let r = num(&format!("a:v{i}"), 1, false);
            buffs.add(e, &r.buff_id(), P, r.payload());
        }
        assert!(!admitted(&buffs, e, &s.buff_id(), P, &s.payload()), "bounded");
        let again = num("a:v0", 2, false);
        assert!(admitted(&buffs, e, &again.buff_id(), P, &again.payload()), "a rewrite always fits");
    }

    #[test]
    fn hero_rows_survive_a_new_life() {
        let mut world = World::new();
        let old = world.create_entity().build();
        let new = world.create_entity().build();
        let mut buffs = BuffStore::default();
        for s in [num("a:keep", 5, true), num("a:drop", 6, false), num("a:also", 7, true)] {
            buffs.add(old, &s.buff_id(), P, s.payload());
        }
        let saved = persisting(&buffs, old);
        assert_eq!(saved.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(), ["unit_var:a:also", "unit_var:a:keep"]);
        buffs.remove_all_for(old);
        restore(&mut buffs, new, &saved);
        let read = |id: &str| buffs.get(new, id).and_then(|b| Spec::from_buff(id, &b.payload)).map(|s| s.value);
        assert_eq!(read("unit_var:a:keep"), Some(VarValue::Num(Fixed64::from_raw(5))));
        assert_eq!(read("unit_var:a:drop"), None);
    }
}
