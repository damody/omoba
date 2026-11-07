//! v3 `status.absorb` (a status that absorbs damage up to an amount: 頑強戰法護盾、直江兼續的護盾…).
//! The script applies the status, then sends the one-shot packet `__absorb` naming that status's
//! BuffStore key; the host keeps the row `__absorb:<status key>` (same duration) with what is left.
//! Every damage packet after mitigation and before the plain `damage_shield` is absorbed by these
//! rows in key order, each taking only the share of the damage types it lists (all types when it
//! lists none). A row whose status is gone (dispelled, expired) absorbs nothing and goes; a row
//! that is used up removes its status (with the shared states only it held). `hp_removal` damage
//! is never absorbed. Supported: `amount`, `types`, `phase = "post_mitigation"` (the default), and
//! the status's `on_absorb` / `on_broken` triggers (status_event ABSORB / BROKEN, E393); `fraction`
//! and `lethal_only` are not.
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

use super::ability_runtime::BuffStore;

pub const KEY: &str = "__absorb_v1";
pub const BUFF: &str = "__absorb";
const ROW_PREFIX: &str = "__absorb:";

/// Damage types an absorb takes: physical, magical, pure.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Types { pub physical: bool, pub magical: bool, pub pure: bool }

impl Types {
    pub const ALL: Types = Types { physical: true, magical: true, pure: true };
    pub fn from_names(names: &[&str]) -> Option<Types> {
        if names.is_empty() { return Some(Types::ALL); }
        let mut types = Types::default();
        for name in names {
            match *name { "physical" => types.physical = true, "magical" => types.magical = true, "pure" => types.pure = true, _ => return None }
        }
        Some(types)
    }
    fn names(self) -> Vec<&'static str> {
        [(self.physical, "physical"), (self.magical, "magical"), (self.pure, "pure")].into_iter().filter(|(on, _)| *on).map(|(_, n)| n).collect()
    }
    fn share(self, by_type: [Fixed64; 3]) -> Fixed64 {
        [self.physical, self.magical, self.pure].into_iter().zip(by_type).filter(|(on, _)| *on).fold(Fixed64::ZERO, |sum, (_, v)| sum + v)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec { pub status: String, pub amount: Fixed64, pub types: Types }

impl Spec {
    pub fn payload(&self) -> Value {
        json!({ (KEY): { "status": self.status, "amount_raw": self.amount.raw(), "types": self.types.names() } })
    }
    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Spec> {
        if buff_id != BUFF || duration <= Fixed64::ZERO || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 3 { return None; }
        let status = row.get("status")?.as_str()?;
        if status.is_empty() || status.len() > 128 || status.starts_with("__") { return None; }
        let amount = Fixed64::from_raw(row.get("amount_raw")?.as_i64()?);
        if amount <= Fixed64::ZERO || amount > Fixed64::from_i32(1_000_000) { return None; }
        let names: Vec<&str> = row.get("types")?.as_array()?.iter().map(Value::as_str).collect::<Option<_>>()?;
        Some(Spec { status: status.to_owned(), amount, types: Types::from_names(&names)? })
    }
}

pub fn admitted(world: &World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> bool {
    Spec::from_payload(buff_id, duration, payload).is_some() && world.entities().is_alive(target)
}

/// Stores (or refreshes) the absorb row of the status.
pub fn apply(world: &mut World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) {
    if !admitted(world, target, buff_id, duration, payload) { return; }
    let Some(spec) = Spec::from_payload(buff_id, duration, payload) else { return };
    let mut buffs = world.write_resource::<BuffStore>();
    let key = format!("{ROW_PREFIX}{}", spec.status);
    buffs.remove(target, &key);
    buffs.add(target, &key, duration, json!({ "remaining_raw": spec.amount.raw(), "types": spec.types.names() }));
}

/// Absorbs `total` (the mitigated damage, `by_type` its physical / magical / pure shares, from
/// `source`) with the target's absorb rows; returns what is left, and the notes for the statuses
/// listening to their absorbs (status_event ABSORB / BROKEN).
pub fn absorb(world: &World, source: Entity, target: Entity, total: Fixed64, by_type: [Fixed64; 3])
    -> (Fixed64, Vec<super::status_event::AbsorbNote>) {
    let mut notes = Vec::new();
    if total <= Fixed64::ZERO { return (total, notes); }
    let rows: Vec<(String, Fixed64, Types)> = {
        let buffs = world.read_resource::<BuffStore>();
        let mut rows: Vec<_> = buffs.iter_for(target).filter_map(|(key, entry)| {
            key.strip_prefix(ROW_PREFIX)?;
            let remaining = Fixed64::from_raw(entry.payload.get("remaining_raw")?.as_i64()?);
            let names: Vec<&str> = entry.payload.get("types")?.as_array()?.iter().filter_map(Value::as_str).collect();
            Some((key.to_owned(), remaining, Types::from_names(&names)?))
        }).collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        rows
    };
    let mut left = total;
    let mut by_type = by_type;
    for (row, remaining, types) in rows {
        let status = &row[ROW_PREFIX.len()..];
        if !world.read_resource::<BuffStore>().has(target, status) {
            world.write_resource::<BuffStore>().remove(target, &row);
            continue;
        }
        let taken = types.share(by_type).min(remaining).min(left);
        if taken <= Fixed64::ZERO { continue; }
        // Take the absorbed amount from the listed types' shares in order.
        let mut owed = taken;
        for (on, share) in [types.physical, types.magical, types.pure].into_iter().zip(by_type.iter_mut()) {
            if on && owed > Fixed64::ZERO { let t = (*share).min(owed); *share = *share - t; owed = owed - t; }
        }
        left = left - taken;
        let rest = remaining - taken;
        let spec = world.read_resource::<BuffStore>().get(target, status)
            .and_then(|entry| super::status_event::Spec::from_payload(&entry.payload));
        if let Some(spec) = spec.filter(|spec| spec.listens(super::status_event::ABSORB) || spec.listens(super::status_event::BROKEN)) {
            notes.push(super::status_event::AbsorbNote { holder: target, row: status.to_owned(), spec, source,
                amount: taken, broken: rest <= Fixed64::ZERO });
        }
        if rest <= Fixed64::ZERO {
            world.write_resource::<BuffStore>().remove(target, &row);
            super::dispel::remove_status(world, target, status);
        } else {
            let mut buffs = world.write_resource::<BuffStore>();
            if let Some(entry) = buffs.get(target, &row).cloned() {
                let mut payload = entry.payload;
                payload["remaining_raw"] = json!(rest.raw());
                buffs.remove(target, &row);
                buffs.add(target, &row, entry.remaining, payload);
            }
        }
        if left <= Fixed64::ZERO { break; }
    }
    (left, notes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    #[test]
    fn absorbs_its_types_until_broken_then_the_status_goes() {
        let mut world = World::new();
        world.insert(BuffStore::new());
        let hero = world.create_entity().build();
        world.write_resource::<BuffStore>().add(hero, "a18r_shield", Fixed64::from_i32(10), json!({}));
        let spec = Spec { status: "a18r_shield".into(), amount: Fixed64::from_i32(100), types: Types::from_names(&["magical"]).unwrap() };
        apply(&mut world, hero, BUFF, Fixed64::from_i32(10), &spec.payload());
        let n = Fixed64::from_i32;
        // 60 physical + 40 magical: only the 40 magical is absorbed.
        assert_eq!(absorb(&world, hero, hero, n(100), [n(60), n(40), n(0)]).0, n(60));
        // 80 magical: 60 left in the shield, 20 goes through and the status is removed.
        assert_eq!(absorb(&world, hero, hero, n(80), [n(0), n(80), n(0)]).0, n(20));
        assert!(!world.read_resource::<BuffStore>().has(hero, "a18r_shield"));
        assert!(!world.read_resource::<BuffStore>().has(hero, "__absorb:a18r_shield"));
        // A row whose status was dispelled absorbs nothing.
        world.write_resource::<BuffStore>().add(hero, "wall", Fixed64::from_i32(10), json!({}));
        let all = Spec { status: "wall".into(), amount: n(50), types: Types::ALL };
        apply(&mut world, hero, BUFF, Fixed64::from_i32(10), &all.payload());
        world.write_resource::<BuffStore>().remove(hero, "wall");
        assert_eq!(absorb(&world, hero, hero, n(30), [n(30), n(0), n(0)]).0, n(30));
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &all.payload()), Some(all));
    }

    /// E393: a status listening to on_absorb / on_broken hears each absorb with its amount and
    /// source, and its breaking, read before the broken status is removed.
    #[test]
    fn a_listening_status_hears_each_absorb_and_its_breaking() {
        use super::super::status_event::{self as se, AbsorbNote};
        use omb_script_abi::types::EntityHandle;
        let mut world = World::new();
        world.insert(BuffStore::new());
        let (hero, foe) = (world.create_entity().build(), world.create_entity().build());
        let n = Fixed64::from_i32;
        let listen = se::Spec { ability: "a18_shield".into(), level: 1, status: "a18r_shield".into(),
            source: EntityHandle { id: 1, gen: 1 }, mask: se::ON_ABSORB | se::ON_BROKEN };
        world.write_resource::<BuffStore>().add(hero, "a18r_shield", n(10), json!({ (se::KEY): listen.row() }));
        let spec = Spec { status: "a18r_shield".into(), amount: n(100), types: Types::ALL };
        apply(&mut world, hero, BUFF, n(10), &spec.payload());
        let note = |amount: i32, broken: bool| AbsorbNote { holder: hero, row: "a18r_shield".into(), spec: listen.clone(),
            source: foe, amount: n(amount), broken };
        assert_eq!(absorb(&world, foe, hero, n(70), [n(70), n(0), n(0)]), (n(0), vec![note(70, false)]));
        assert_eq!(absorb(&world, foe, hero, n(50), [n(0), n(50), n(0)]), (n(20), vec![note(30, true)]));
        assert!(!world.read_resource::<BuffStore>().has(hero, "a18r_shield"), "broken: the status goes");
        // A status that does not listen leaves no note.
        world.write_resource::<BuffStore>().add(hero, "wall", n(10), json!({}));
        apply(&mut world, hero, BUFF, n(10), &Spec { status: "wall".into(), amount: n(50), types: Types::ALL }.payload());
        assert_eq!(absorb(&world, foe, hero, n(10), [n(10), n(0), n(0)]), (n(0), vec![]));
    }
}
