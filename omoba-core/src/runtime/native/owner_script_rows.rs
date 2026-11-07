//! Own-team script rows a team replica cannot rebuild for a new local life.
//!
//! A replica starts a hero's local life (bootstrap, rebase, reveal) from the disclosed baseline,
//! then replays its learn hooks. Controls and channels come from owner_control; the v3 rows the
//! hero's own scripts keep in its `BuffStore` do not: a self-applied status with its stacks, a unit
//! state variable, a passive's periodic schedule (its instance and phase). Without them the
//! replica's learn hook starts a fresh periodic in another phase and its status never exists, so
//! a passive that decides from its own state (kojima_sadaoki_old 法術護盾) acts on the replica
//! where the authority does not.
//!
//! The authority discloses these rows to the hero's own team as a baseline-only component: a new
//! local life seeds them (before the learn replay, whose guards then see them), and the component
//! is never hashed, so its per-tick value needs no event. Only rows whose entity references all
//! name the hero itself are carried; those references travel as 0 and become the replica's own
//! entity. A row naming another unit is left out, as before.
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::Entity;

use super::{scheduled_effect, status_event, unit_var, BuffStore};

pub const SCHEMA_ID: u32 = 0x464f4731;
/// Rows one hero discloses (a bound on what scripts can grow).
const MAX_ROWS: usize = 256;
const SOURCE_ID: &str = "source_entity_id";
const SOURCE_GEN: &str = "source_entity_gen";

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Row {
    pub id: String,
    pub remaining: i64,
    pub payload: Value,
}

fn names(entity: Entity, id: u64, gen: u64) -> bool {
    id == u64::from(entity.id()) && gen == u64::from(entity.gen().id() as u32)
}

fn pair(value: &Value) -> Option<(u64, u64)> {
    let pair = value.as_array()?;
    (pair.len() == 2).then_some(())?;
    Some((pair[0].as_u64()?, pair[1].as_u64()?))
}

/// The row with every reference to `entity` set to `to`, or `None` when it names another unit
/// or is not a kind carried here. `abilities` are the hero's own (its schedules).
fn portable(id: &str, payload: &Value, entity: Entity, abilities: &[String], to: (u32, u32)) -> Option<Value> {
    let mut payload = payload.clone();
    let row = payload.as_object_mut()?;
    if id.starts_with(unit_var::PREFIX) {
        let spec = unit_var::Spec::from_buff(id, &Value::Object(row.clone()))?;
        if let unit_var::VarValue::Unit(u) = spec.value {
            names(entity, u64::from(u.id), u64::from(u.gen)).then_some(())?;
            row.get_mut(unit_var::KEY)?.as_object_mut()?.insert("u".into(), json!([to.0, to.1]));
        }
        return Some(payload);
    }
    if id.starts_with(scheduled_effect::PREFIX) {
        let spec = scheduled_effect::Spec::from_buff(id, &Value::Object(row.clone()))?;
        abilities.iter().any(|ability| *ability == spec.ability).then_some(())?;
        let args = row.get_mut(scheduled_effect::KEY)?.get_mut("args")?.as_array_mut()?;
        for arg in args.iter_mut().filter_map(|arg| arg.get_mut("u")) {
            let (id, gen) = pair(arg)?;
            names(entity, id, gen).then_some(())?;
            *arg = json!([to.0, to.1]);
        }
        return Some(payload);
    }
    // A v3 status the hero put on itself (`source_entity_id/gen`, its full duration).
    if id.starts_with("__") || super::dispel::id_names_applier(id) || row.contains_key("__passive")
        || !row.contains_key(super::illusion::DURATION_KEY)
        || row.contains_key(super::ability_runtime::damage_zone::KEY) || row.contains_key("dot_kind") {
        return None;
    }
    let source = (row.get(SOURCE_ID)?.as_u64()?, row.get(SOURCE_GEN)?.as_u64()?);
    names(entity, source.0, source.1).then_some(())?;
    row.insert(SOURCE_ID.into(), to.0.into());
    row.insert(SOURCE_GEN.into(), to.1.into());
    if let Some(events) = row.get_mut(status_event::KEY) {
        let (id, gen) = pair(events.get("source")?)?;
        names(entity, id, gen).then_some(())?;
        events.as_object_mut()?.insert("source".into(), json!([to.0, to.1]));
    }
    Some(payload)
}

/// The authority's rows of hero `entity`, its references written as 0.
pub fn of(buffs: &BuffStore, entity: Entity, abilities: &[String]) -> Vec<Row> {
    let mut rows: Vec<Row> = buffs.iter_for(entity)
        .filter_map(|(id, entry)| Some(Row { id: id.to_owned(), remaining: entry.remaining.raw(),
            payload: portable(id, &entry.payload, entity, abilities, (0, 0))? }))
        .collect();
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    rows.truncate(MAX_ROWS);
    rows
}

pub fn encode(rows: &[Row]) -> Option<Vec<u8>> {
    (!rows.is_empty()).then(|| crate::runtime::visibility::canonical_disclosed_json(&rows))
}

pub fn decode(bytes: &[u8]) -> Option<Vec<Row>> {
    let rows: Vec<Row> = serde_json::from_slice(bytes).ok()?;
    (rows.len() <= MAX_ROWS).then_some(rows)
}

/// A new local life of `entity` on a replica: the authority's rows, its references made `entity`.
/// A row the replica already holds keeps the replica's.
pub fn seed(buffs: &mut BuffStore, entity: Entity, rows: &[Row]) {
    let to = (entity.id(), entity.gen().id() as u32);
    for row in rows {
        if buffs.has(entity, &row.id) || row.remaining <= 0 { continue; }
        let Some(payload) = recast_zero(&row.id, &row.payload, to) else { continue };
        buffs.seed(entity, &row.id, Fixed64::from_raw(row.remaining), payload);
    }
}

/// `payload` with its zero self references set to `to`; `None` when it is not a carried kind.
fn recast_zero(id: &str, payload: &Value, to: (u32, u32)) -> Option<Value> {
    let mut payload = payload.clone();
    let row = payload.as_object_mut()?;
    if id.starts_with(unit_var::PREFIX) {
        unit_var::Spec::from_buff(id, &Value::Object(row.clone()))?;
        if let Some(unit) = row.get_mut(unit_var::KEY)?.get_mut("u") { *unit = json!([to.0, to.1]); }
        return Some(payload);
    }
    if id.starts_with(scheduled_effect::PREFIX) {
        scheduled_effect::Spec::from_buff(id, &Value::Object(row.clone()))?;
        for arg in row.get_mut(scheduled_effect::KEY)?.get_mut("args")?.as_array_mut()?.iter_mut().filter_map(|arg| arg.get_mut("u")) {
            *arg = json!([to.0, to.1]);
        }
        return Some(payload);
    }
    if !row.contains_key(SOURCE_ID) { return None; }
    row.insert(SOURCE_ID.into(), to.0.into());
    row.insert(SOURCE_GEN.into(), to.1.into());
    if let Some(events) = row.get_mut(status_event::KEY).and_then(Value::as_object_mut) {
        events.insert("source".into(), json!([to.0, to.1]));
    }
    Some(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, World, WorldExt};

    fn handle(e: Entity) -> [u32; 2] { [e.id(), e.gen().id() as u32] }

    #[test]
    fn carries_only_rows_whose_references_name_the_hero_and_seeds_them_on_a_new_entity() {
        let mut world = World::new();
        let (hero, enemy, fresh) = (world.create_entity().build(), world.create_entity().build(), world.create_entity().build());
        let mut buffs = BuffStore::new();
        let me = handle(hero);
        let status = |by: [u32; 2]| json!({ "__v3_stacks": 2, (super::super::illusion::DURATION_KEY): 7, SOURCE_ID: by[0], SOURCE_GEN: by[1],
            (status_event::KEY): { "ability": "kit_q", "level": 1, "status": "kit_shield", "source": by, "mask": 1 } });
        buffs.add(hero, "kit_shield", Fixed64::from_i32(5), status(me));
        buffs.add(hero, "kit_mark", Fixed64::from_i32(5), status(handle(enemy)));
        let schedule = |ability: &str, args: Value| json!({ (scheduled_effect::KEY): { "ability": ability, "level": 1, "step": 1, "instance": 9, "args": args } });
        buffs.add(hero, "scheduled:kit_q:9:1", Fixed64::from_i32(2), schedule("kit_q", json!([{ "u": me }, { "n": 1024 }])));
        buffs.add(hero, "scheduled:kit_w:9:1", Fixed64::from_i32(2), schedule("kit_w", json!([{ "u": handle(enemy) }])));
        buffs.add(hero, "scheduled:foe_r:9:1", Fixed64::from_i32(2), schedule("foe_r", json!([])));
        let var = |value: Value| json!({ (unit_var::KEY): value });
        buffs.add(hero, "unit_var:kit:count", Fixed64::from_i32(9), var(json!({ "n": 3072, "hero": false })));
        buffs.add(hero, "unit_var:kit:me", Fixed64::from_i32(9), var(json!({ "u": me, "hero": true })));
        buffs.add(hero, "unit_var:kit:foe", Fixed64::from_i32(9), var(json!({ "u": handle(enemy), "hero": false })));
        buffs.add(hero, "kit_passive", Fixed64::from_i32(9), json!({ "__passive": true, "level": 1 }));
        let abilities = vec!["kit_q".to_owned(), "kit_w".to_owned()];
        let rows = of(&buffs, hero, &abilities);
        assert_eq!(rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
            ["kit_shield", "scheduled:kit_q:9:1", "unit_var:kit:count", "unit_var:kit:me"]);
        assert_eq!(rows[0].payload[SOURCE_ID], 0, "references travel as 0");
        assert_eq!(decode(&encode(&rows).unwrap()), Some(rows.clone()));
        assert!(encode(&[]).is_none());

        let mut replica = BuffStore::new();
        replica.add(fresh, "unit_var:kit:count", Fixed64::from_i32(1), var(json!({ "n": 1024, "hero": false })));
        seed(&mut replica, fresh, &rows);
        let to = handle(fresh);
        let shield = replica.get(fresh, "kit_shield").unwrap();
        assert_eq!((shield.remaining, shield.payload["__v3_stacks"].clone()), (Fixed64::from_i32(5), json!(2)));
        assert_eq!((shield.payload[SOURCE_ID].clone(), shield.payload[status_event::KEY]["source"].clone()), (json!(to[0]), json!(to)));
        assert_eq!(replica.get(fresh, "scheduled:kit_q:9:1").unwrap().payload[scheduled_effect::KEY]["args"][0]["u"], json!(to));
        assert_eq!(replica.get(fresh, "unit_var:kit:me").unwrap().payload[unit_var::KEY]["u"], json!(to));
        assert_eq!(replica.get(fresh, "unit_var:kit:count").unwrap().payload[unit_var::KEY]["n"], 1024, "the replica's own row stays");
        assert_eq!(of(&replica, fresh, &abilities).len(), rows.len());
    }
}
