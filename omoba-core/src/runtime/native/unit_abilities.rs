//! Abilities a non-hero unit holds (E529, v3 unit_template `abilities`, 原作 a summon's own
//! ability: A36 地刺陷阱's 引爆). A hero keeps its abilities on `Hero::abilities`; any other unit
//! holds one permanent `BuffStore` row `__unit_ability:<ability>` per ability (payload: its rank),
//! written by the creating code (`Cx::grant_unit_ability`). Its cooldown is the row
//! `__unit_ability_cd:<ability>` living for the cooldown: cast admission refuses the ability while
//! it holds. Both rows are host state like every marker (replicated, hashed; `__` rows are never
//! dispelled) and go with the unit's death.
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::Entity;

use super::ability_runtime::BuffStore;

pub const KEY: &str = "__unit_ability_v1";
pub const PREFIX: &str = "__unit_ability:";
pub const COOLDOWN_PREFIX: &str = "__unit_ability_cd:";
/// The highest rank a unit ability takes.
pub const MAX_LEVEL: u8 = 100;
/// Abilities one unit may hold.
pub const MAX_PER_UNIT: usize = 16;

pub fn row_id(ability: &str) -> String { format!("{PREFIX}{ability}") }
pub fn cooldown_id(ability: &str) -> String { format!("{COOLDOWN_PREFIX}{ability}") }

/// The row payload of an ability held at `level`; `None` for a level out of range or a bad id.
pub fn payload(ability: &str, level: u8) -> Option<Value> {
    (valid_ability(ability) && (1..=MAX_LEVEL).contains(&level)).then(|| json!({ (KEY): { "level": level } }))
}

fn valid_ability(ability: &str) -> bool {
    !ability.is_empty() && ability.len() <= 64 && ability.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn level_of(payload: &Value) -> Option<u8> {
    let level = u8::try_from(payload.get(KEY)?.get("level")?.as_u64()?).ok()?;
    (1..=MAX_LEVEL).contains(&level).then_some(level)
}

/// The rank `unit` holds `ability` at, if it holds it as a unit ability.
pub fn level(buffs: &BuffStore, unit: Entity, ability: &str) -> Option<u8> {
    buffs.get(unit, &row_id(ability)).and_then(|row| level_of(&row.payload))
}

/// The unit abilities `unit` holds and their ranks, by ability id.
pub fn abilities(buffs: &BuffStore, unit: Entity) -> Vec<(String, u8)> {
    let mut list: Vec<(String, u8)> = buffs.iter_for(unit)
        .filter_map(|(id, row)| id.strip_prefix(PREFIX).and_then(|a| level_of(&row.payload).map(|l| (a.to_owned(), l))))
        .collect();
    list.sort();
    list
}

/// Whether `unit`'s unit ability `ability` is cooling down.
pub fn on_cooldown(buffs: &BuffStore, unit: Entity, ability: &str) -> bool { buffs.has(unit, &cooldown_id(ability)) }

/// Start `ability`'s cooldown on a unit that holds it as a unit ability (a cast succeeded).
pub fn start_cooldown(buffs: &mut BuffStore, unit: Entity, ability: &str, duration: Fixed64) {
    if duration <= Fixed64::ZERO || level(buffs, unit, ability).is_none() { return; }
    buffs.add(unit, &cooldown_id(ability), duration, json!({}));
}

/// Whether a granted row may be added: a valid payload and room on the unit.
pub fn admits(buffs: &BuffStore, unit: Entity, buff_id: &str, payload: &Value) -> bool {
    let Some(ability) = buff_id.strip_prefix(PREFIX) else { return true };
    valid_ability(ability) && level_of(payload).is_some()
        && (buffs.has(unit, buff_id) || abilities(buffs, unit).len() < MAX_PER_UNIT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn a_unit_ability_has_a_rank_and_a_cooldown_row() {
        let mut world = World::new();
        let unit = world.create_entity().build();
        let mut buffs = BuffStore::default();
        assert_eq!(payload("a36_detonate", 0), None);
        assert_eq!(payload("bad:id", 1), None);
        let row = payload("a36_detonate", 3).unwrap();
        assert!(admits(&buffs, unit, &row_id("a36_detonate"), &row));
        assert!(!admits(&buffs, unit, &row_id("a36_detonate"), &json!({})));
        buffs.add(unit, &row_id("a36_detonate"), Fixed64::from_i32(3600), row);
        assert_eq!(level(&buffs, unit, "a36_detonate"), Some(3));
        assert_eq!(abilities(&buffs, unit), vec![("a36_detonate".to_owned(), 3)]);
        assert!(!on_cooldown(&buffs, unit, "a36_detonate"));
        start_cooldown(&mut buffs, unit, "a36_detonate", Fixed64::from_i32(8));
        assert!(on_cooldown(&buffs, unit, "a36_detonate"));
        start_cooldown(&mut buffs, unit, "other", Fixed64::from_i32(8));
        assert!(!on_cooldown(&buffs, unit, "other"), "only a held ability cools down");
    }
}
