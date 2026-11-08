//! v3 `status.absorb` (a status that absorbs damage up to an amount: 頑強戰法護盾、直江兼續的護盾…).
//! The script applies the status, then sends the one-shot packet `__absorb` naming that status's
//! BuffStore key; the host keeps the row `__absorb:<status key>` (same duration) with what is left.
//! Every damage packet after mitigation and before the plain `damage_shield` is absorbed by these
//! rows in key order, each taking only the share of the damage types it lists (all types when it
//! lists none). A row whose status is gone (dispelled, expired) absorbs nothing and goes; a row
//! that is used up removes its status (with the shared states only it held). `hp_removal` damage
//! is never absorbed. Supported: `amount`, `types`, `phase`, and the status's `on_absorb` /
//! `on_broken` triggers (status_event ABSORB / BROKEN, E393); `fraction` and `lethal_only` are not.
//! The status may be an independent instance (`<status>#<n>`): each instance has its own row.
//!
//! `phase = "pre_mitigation"` (原作 nobu_modifier_magical_shield, which drains its pool by the
//! damage before reductions): the row's amount is in damage before armor and resistance. It takes
//! its share at the same stage as the others (before them), but each point it takes off the packet
//! drains the pool by what that point was before mitigation: the packet's listed raw share over its
//! listed mitigated share. A packet that came from a raw one ([`RawPacket`]) knows its raw shares;
//! one a producer made already mitigated (an attack's) drains one for one.
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
pub struct Spec {
    pub status: String,
    pub amount: Fixed64,
    pub types: Types,
    /// `phase = "pre_mitigation"`: `amount` is in damage before armor and resistance.
    pub pre_mitigation: bool,
}

const PRE_MITIGATION: &str = "pre_mitigation";

impl Spec {
    pub fn payload(&self) -> Value {
        let mut row = json!({ "status": self.status, "amount_raw": self.amount.raw(), "types": self.types.names() });
        if self.pre_mitigation { row["phase"] = json!(PRE_MITIGATION); }
        json!({ (KEY): row })
    }
    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Spec> {
        if buff_id != BUFF || duration <= Fixed64::ZERO || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        let pre_mitigation = match row.get("phase") {
            None => false,
            Some(phase) => { (phase.as_str()? == PRE_MITIGATION).then_some(())?; true }
        };
        if row.len() != 3 + usize::from(pre_mitigation) { return None; }
        let status = row.get("status")?.as_str()?;
        if status.is_empty() || status.len() > 128 || status.starts_with("__") { return None; }
        let amount = Fixed64::from_raw(row.get("amount_raw")?.as_i64()?);
        if amount <= Fixed64::ZERO || amount > Fixed64::from_i32(1_000_000) { return None; }
        let names: Vec<&str> = row.get("types")?.as_array()?.iter().map(Value::as_str).collect::<Option<_>>()?;
        Some(Spec { status: status.to_owned(), amount, types: Types::from_names(&names)?, pre_mitigation })
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
    let mut row = json!({ "remaining_raw": spec.amount.raw(), "types": spec.types.names() });
    if spec.pre_mitigation { row["phase"] = json!(PRE_MITIGATION); }
    buffs.add(target, &key, duration, row);
}

/// The raw packet the damage being settled came from (`RawDamage`, before block, armor and
/// resistance): (source, target, its settled shares, its raw shares). Set when the raw packet is
/// settled, taken by the damage handler of the same packet.
#[derive(Clone, Copy, Debug, Default)]
pub struct RawPacket(pub Option<(Entity, Entity, [Fixed64; 3], [Fixed64; 3])>);

/// The raw shares of the packet `source` -> `target` with settled shares `settled` (taken), or the
/// settled shares themselves when it was not made from a raw packet.
pub fn raw_shares(world: &World, source: Entity, target: Entity, settled: [Fixed64; 3]) -> [Fixed64; 3] {
    let taken = world.try_fetch_mut::<RawPacket>().and_then(|mut packet| packet.0.take());
    match taken {
        Some((s, t, shares, raw)) if s == source && t == target && shares == settled => raw,
        _ => settled,
    }
}

/// `value * numerator / denominator`, rounded down (0 for a zero denominator).
fn scale(value: Fixed64, numerator: Fixed64, denominator: Fixed64) -> Fixed64 {
    if denominator <= Fixed64::ZERO { return Fixed64::ZERO; }
    Fixed64::from_raw(i64::try_from(i128::from(value.raw()) * i128::from(numerator.raw()) / i128::from(denominator.raw())).unwrap_or(i64::MAX))
}

/// Absorbs `total` (the mitigated damage, `by_type` its physical / magical / pure shares, from
/// `source`; `raw` those shares before mitigation, see [`raw_shares`]) with the target's absorb
/// rows, the pre-mitigation ones first, each kind in key order; returns what is left, and the
/// notes for the statuses listening to their absorbs (status_event ABSORB / BROKEN).
pub fn absorb(world: &World, source: Entity, target: Entity, total: Fixed64, by_type: [Fixed64; 3], raw: [Fixed64; 3])
    -> (Fixed64, Vec<super::status_event::AbsorbNote>) {
    let mut notes = Vec::new();
    if total <= Fixed64::ZERO { return (total, notes); }
    let rows: Vec<(String, Fixed64, Types, bool)> = {
        let buffs = world.read_resource::<BuffStore>();
        let mut rows: Vec<_> = buffs.iter_for(target).filter_map(|(key, entry)| {
            key.strip_prefix(ROW_PREFIX)?;
            let remaining = Fixed64::from_raw(entry.payload.get("remaining_raw")?.as_i64()?);
            let names: Vec<&str> = entry.payload.get("types")?.as_array()?.iter().filter_map(Value::as_str).collect();
            let pre = entry.payload.get("phase").and_then(Value::as_str) == Some(PRE_MITIGATION);
            Some((key.to_owned(), remaining, Types::from_names(&names)?, pre))
        }).collect();
        rows.sort_by(|a, b| (!a.3, &a.0).cmp(&(!b.3, &b.0)));
        rows
    };
    let mut left = total;
    let mut by_type = by_type;
    let mut raw = raw;
    for (row, remaining, types, pre) in rows {
        let status = &row[ROW_PREFIX.len()..];
        if !world.read_resource::<BuffStore>().has(target, status) {
            world.write_resource::<BuffStore>().remove(target, &row);
            continue;
        }
        let listed = types.share(by_type);
        // A pre-mitigation row's pool, in this packet's mitigated damage, and what each point drains.
        let listed_raw = if pre { types.share(raw).max(listed) } else { listed };
        let capacity = if listed_raw > listed { scale(remaining, listed, listed_raw) } else { remaining };
        let taken = listed.min(capacity).min(left);
        if taken <= Fixed64::ZERO { continue; }
        let drained = if taken >= capacity { remaining } else if listed_raw > listed { scale(taken, listed_raw, listed).min(remaining) } else { taken };
        // Take the absorbed amount from the listed types' shares in order (their raw shares too).
        let (mut owed, mut owed_raw) = (taken, drained);
        for (index, on) in [types.physical, types.magical, types.pure].into_iter().enumerate() {
            if !on { continue; }
            let t = by_type[index].min(owed); by_type[index] = by_type[index] - t; owed = owed - t;
            let t = raw[index].min(owed_raw); raw[index] = raw[index] - t; owed_raw = owed_raw - t;
        }
        left = left - taken;
        let rest = remaining - drained;
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
        let spec = Spec { status: "a18r_shield".into(), amount: Fixed64::from_i32(100), types: Types::from_names(&["magical"]).unwrap(), pre_mitigation: false };
        apply(&mut world, hero, BUFF, Fixed64::from_i32(10), &spec.payload());
        let n = Fixed64::from_i32;
        // 60 physical + 40 magical: only the 40 magical is absorbed.
        assert_eq!(absorb(&world, hero, hero, n(100), [n(60), n(40), n(0)], [n(60), n(40), n(0)]).0, n(60));
        // 80 magical: 60 left in the shield, 20 goes through and the status is removed.
        assert_eq!(absorb(&world, hero, hero, n(80), [n(0), n(80), n(0)], [n(0), n(80), n(0)]).0, n(20));
        assert!(!world.read_resource::<BuffStore>().has(hero, "a18r_shield"));
        assert!(!world.read_resource::<BuffStore>().has(hero, "__absorb:a18r_shield"));
        // A row whose status was dispelled absorbs nothing.
        world.write_resource::<BuffStore>().add(hero, "wall", Fixed64::from_i32(10), json!({}));
        let all = Spec { status: "wall".into(), amount: n(50), types: Types::ALL, pre_mitigation: false };
        apply(&mut world, hero, BUFF, Fixed64::from_i32(10), &all.payload());
        world.write_resource::<BuffStore>().remove(hero, "wall");
        assert_eq!(absorb(&world, hero, hero, n(30), [n(30), n(0), n(0)], [n(30), n(0), n(0)]).0, n(30));
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &all.payload()), Some(all));
    }

    /// `phase = "pre_mitigation"`: the pool is in damage before mitigation, so a hit halved by
    /// resistance drains it by twice what it takes; such rows go before the others; the phase
    /// round-trips and an unknown one is refused.
    #[test]
    fn a_pre_mitigation_shield_drains_by_the_damage_before_mitigation() {
        let mut world = World::new();
        world.insert(BuffStore::new());
        let hero = world.create_entity().build();
        let n = Fixed64::from_i32;
        let magical = Types::from_names(&["magical"]).unwrap();
        for status in ["ward", "wall"] { world.write_resource::<BuffStore>().add(hero, status, n(10), json!({})); }
        let ward = Spec { status: "ward".into(), amount: n(150), types: magical, pre_mitigation: true };
        apply(&mut world, hero, BUFF, n(10), &ward.payload());
        apply(&mut world, hero, BUFF, n(10), &Spec { status: "wall".into(), amount: n(100), types: Types::ALL, pre_mitigation: false }.payload());
        // 50 magical, 100 before resistance: the ward (first, though "wall" sorts before it) takes
        // all 50 and drains 100.
        assert_eq!(absorb(&world, hero, hero, n(50), [n(0), n(50), n(0)], [n(0), n(100), n(0)]).0, n(0));
        let left = |world: &World, status: &str| world.read_resource::<BuffStore>().get(hero, &format!("{ROW_PREFIX}{status}"))
            .and_then(|row| row.payload.get("remaining_raw")?.as_i64()).map(Fixed64::from_raw);
        assert_eq!((left(&world, "ward"), left(&world, "wall")), (Some(n(50)), Some(n(100))));
        // 40 magical from 80: 50 before mitigation holds 25 of it; the wall takes the other 15.
        assert_eq!(absorb(&world, hero, hero, n(40), [n(0), n(40), n(0)], [n(0), n(80), n(0)]).0, n(0));
        assert!(!world.read_resource::<BuffStore>().has(hero, "ward"), "the ward broke");
        assert_eq!(left(&world, "wall"), Some(n(85)));
        // A packet with no raw shares known drains one for one.
        world.write_resource::<BuffStore>().add(hero, "ward", n(10), json!({}));
        apply(&mut world, hero, BUFF, n(10), &ward.payload());
        assert_eq!(absorb(&world, hero, hero, n(30), [n(0), n(30), n(0)], [n(0), n(30), n(0)]).0, n(0));
        assert_eq!(left(&world, "ward"), Some(n(120)));
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &ward.payload()), Some(ward.clone()));
        let mut bad = ward.payload(); bad[KEY]["phase"] = json!("post_mitigation");
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &bad), None, "post_mitigation is the default, never written");
    }

    /// Independent instances (`<status>#<n>`) each keep their own shield; a used-up instance goes
    /// alone.
    #[test]
    fn each_independent_instance_keeps_its_own_shield() {
        let mut world = World::new();
        world.insert(BuffStore::new());
        let hero = world.create_entity().build();
        let n = Fixed64::from_i32;
        for (instance, amount) in [(1, 30), (2, 50)] {
            let status = format!("magic_talisman_shield#{instance}");
            world.write_resource::<BuffStore>().add(hero, &status, n(10), json!({}));
            apply(&mut world, hero, BUFF, n(10), &Spec { status, amount: n(amount), types: Types::ALL, pre_mitigation: false }.payload());
        }
        assert_eq!(absorb(&world, hero, hero, n(40), [n(40), n(0), n(0)], [n(40), n(0), n(0)]).0, n(0));
        let buffs = world.read_resource::<BuffStore>();
        assert!(!buffs.has(hero, "magic_talisman_shield#1") && buffs.has(hero, "magic_talisman_shield#2"));
        assert_eq!(buffs.get(hero, "__absorb:magic_talisman_shield#2").and_then(|row| row.payload.get("remaining_raw")?.as_i64()), Some(n(40).raw()));
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
        let spec = Spec { status: "a18r_shield".into(), amount: n(100), types: Types::ALL, pre_mitigation: false };
        apply(&mut world, hero, BUFF, n(10), &spec.payload());
        let note = |amount: i32, broken: bool| AbsorbNote { holder: hero, row: "a18r_shield".into(), spec: listen.clone(),
            source: foe, amount: n(amount), broken };
        assert_eq!(absorb(&world, foe, hero, n(70), [n(70), n(0), n(0)], [n(70), n(0), n(0)]), (n(0), vec![note(70, false)]));
        assert_eq!(absorb(&world, foe, hero, n(50), [n(0), n(50), n(0)], [n(0), n(50), n(0)]), (n(20), vec![note(30, true)]));
        assert!(!world.read_resource::<BuffStore>().has(hero, "a18r_shield"), "broken: the status goes");
        // A status that does not listen leaves no note.
        world.write_resource::<BuffStore>().add(hero, "wall", n(10), json!({}));
        apply(&mut world, hero, BUFF, n(10), &Spec { status: "wall".into(), amount: n(50), types: Types::ALL, pre_mitigation: false }.payload());
        assert_eq!(absorb(&world, foe, hero, n(10), [n(10), n(0), n(0)], [n(10), n(0), n(0)]), (n(0), vec![]));
    }
}
