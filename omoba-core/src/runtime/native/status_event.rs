//! status_event：產生的技能 Rust 的狀態觸發（v3 `status.triggers`：on_removed／on_expired、
//! 持有者的 on_attack_hit／on_attacked_hit／on_take_damage／on_deal_damage）。
//!
//! A status the generated code applies carries one more payload key, [`KEY`]: the ability that
//! owns it, the rank, its v3 id, the source, and a mask of the events it listens to. The host then
//! calls `AbilityScript::on_status_event(holder, status, event)` of that ability:
//! - when the row leaves the holder: natural expiry ([`EXPIRED`]), removal or dispel
//!   ([`REMOVED`]), or all of the holder's buffs cleared, e.g. its death ([`CLEARED`]). `BuffStore`
//!   records these ([`BuffStore::drain_status_ended`]);
//! - for this tick's attack hits and damage events of the holder, in event order.
//!
//! `on_created` needs no host: the generated code runs it right after it applies the status.
//! Periodic triggers are schedules (scheduled_effect). Like a delayed effect, a hook may only
//! apply effects; the DLL keeps nothing between calls.
use omb_script_abi::types::{EntityHandle, StatusEvent};
use omoba_sim::Fixed64;
use serde_json::{json, Value};

pub const KEY: &str = "__status_events_v1";

/// `StatusEvent.kind`.
pub const EXPIRED: u8 = 0;
pub const REMOVED: u8 = 1;
pub const CLEARED: u8 = 2;
pub const ATTACK_HIT: u8 = 3;
pub const ATTACKED: u8 = 4;
pub const TAKE_DAMAGE: u8 = 5;
pub const DEAL_DAMAGE: u8 = 6;
/// Before the damage lands (scripting::damage_pre; `on_damage_pre`, not `on_status_event`).
pub const TAKE_DAMAGE_PRE: u8 = 7;
pub const DEAL_DAMAGE_PRE: u8 = 8;
/// The holder executed an ability or used an item (原作 OnAbilityExecuted); `other` is none.
pub const ABILITY_EXECUTED: u8 = 9;

/// `Spec.mask` bits: which events the status listens to.
pub const ON_END: u32 = 1 << 0;
pub const ON_ATTACK_HIT: u32 = 1 << 1;
pub const ON_ATTACKED: u32 = 1 << 2;
pub const ON_TAKE_DAMAGE: u32 = 1 << 3;
pub const ON_DEAL_DAMAGE: u32 = 1 << 4;
pub const ON_TAKE_DAMAGE_PRE: u32 = 1 << 5;
pub const ON_DEAL_DAMAGE_PRE: u32 = 1 << 6;
pub const ON_ABILITY_EXECUTED: u32 = 1 << 7;
pub const ALL: u32 = ON_END | ON_ATTACK_HIT | ON_ATTACKED | ON_TAKE_DAMAGE | ON_DEAL_DAMAGE | ON_TAKE_DAMAGE_PRE | ON_DEAL_DAMAGE_PRE
    | ON_ABILITY_EXECUTED;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    pub ability: String,
    pub level: u8,
    /// The v3 status id (the generated code's match key).
    pub status: String,
    pub source: EntityHandle,
    pub mask: u32,
}

impl Spec {
    pub fn checked(self) -> Option<Self> {
        let name = |s: &str| !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
        (name(&self.ability) && name(&self.status) && self.level > 0 && self.mask != 0 && self.mask & !ALL == 0).then_some(self)
    }
    /// The value of [`KEY`] in the status' payload.
    pub fn row(&self) -> Value {
        json!({ "ability": self.ability, "level": self.level, "status": self.status,
            "source": [self.source.id, self.source.gen], "mask": self.mask })
    }
    /// Strict: the [`KEY`] row of a payload, if it is exactly what this module writes.
    pub fn from_payload(payload: &Value) -> Option<Self> {
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 5 { return None; }
        let source = row.get("source")?.as_array()?;
        if source.len() != 2 { return None; }
        Self {
            ability: row.get("ability")?.as_str()?.to_owned(),
            level: u8::try_from(row.get("level")?.as_u64()?).ok()?,
            status: row.get("status")?.as_str()?.to_owned(),
            source: EntityHandle { id: u32::try_from(source[0].as_u64()?).ok()?, gen: u32::try_from(source[1].as_u64()?).ok()? },
            mask: u32::try_from(row.get("mask")?.as_u64()?).ok()?,
        }.checked()
    }
    pub fn listens(&self, kind: u8) -> bool {
        let bit = match kind {
            EXPIRED | REMOVED | CLEARED => ON_END,
            ATTACK_HIT => ON_ATTACK_HIT,
            ATTACKED => ON_ATTACKED,
            TAKE_DAMAGE => ON_TAKE_DAMAGE,
            DEAL_DAMAGE => ON_DEAL_DAMAGE,
            TAKE_DAMAGE_PRE => ON_TAKE_DAMAGE_PRE,
            DEAL_DAMAGE_PRE => ON_DEAL_DAMAGE_PRE,
            ABILITY_EXECUTED => ON_ABILITY_EXECUTED,
            _ => 0,
        };
        self.mask & bit != 0
    }
    pub fn event(&self, kind: u8, other: EntityHandle, amount: Fixed64, damage_kind: u8) -> StatusEvent {
        StatusEvent { kind, level: self.level, source: self.source, other, amount, damage_kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::BuffStore;
    use specs::{Builder, World, WorldExt};

    fn spec(mask: u32) -> Spec {
        Spec { ability: "b16_beast_union".into(), level: 2, status: "b16t_union".into(), source: EntityHandle { id: 4, gen: 1 }, mask }
    }

    #[test]
    fn payload_round_trips_strictly() {
        let s = spec(ON_END | ON_TAKE_DAMAGE);
        let payload = json!({ "magic_resist": 1, (KEY): s.row() });
        assert_eq!(Spec::from_payload(&payload), Some(s.clone()));
        assert!(s.listens(EXPIRED) && s.listens(CLEARED) && s.listens(TAKE_DAMAGE) && !s.listens(ATTACK_HIT));
        assert!(!s.listens(ABILITY_EXECUTED) && spec(ON_ABILITY_EXECUTED).listens(ABILITY_EXECUTED));
        let mut bad = payload.clone(); bad[KEY]["mask"] = json!(1 << 9);
        assert_eq!(Spec::from_payload(&bad), None, "unknown event bits");
        let mut bad = payload.clone(); bad[KEY]["status"] = json!("B16T");
        assert_eq!(Spec::from_payload(&bad), None, "status ids are v3 ids");
        assert_eq!(Spec::from_payload(&json!({})), None);
    }

    /// The store records how each marked row left: expiry, removal, or the holder's buffs
    /// cleared; unmarked rows and refreshes record nothing; draining is in canonical order.
    #[test]
    fn the_store_records_how_marked_rows_end() {
        let mut world = World::new();
        let (a, b) = (world.create_entity().build(), world.create_entity().build());
        let mut buffs = BuffStore::new();
        let marked = json!({ (KEY): spec(ON_END).row() });
        buffs.add(a, "short", Fixed64::from_raw(10), marked.clone());
        buffs.add(a, "long", Fixed64::from_i32(5), marked.clone());
        buffs.add(a, "long", Fixed64::from_i32(6), marked.clone());
        buffs.add(b, "gone", Fixed64::from_i32(5), marked.clone());
        buffs.add(b, "dead", Fixed64::from_i32(5), marked.clone());
        buffs.add(b, "plain", Fixed64::from_raw(10), json!({ "armor": 1 }));
        assert!(buffs.drain_status_ended().is_empty(), "a refresh is not an end");
        buffs.tick(Fixed64::from_raw(17));
        buffs.remove(b, "gone");
        buffs.remove_all_for(b);
        let ended: Vec<_> = buffs.drain_status_ended().into_iter().map(|(e, id, _, kind)| (e, id, kind)).collect();
        assert_eq!(ended, vec![(a, "short".to_owned(), EXPIRED), (b, "gone".to_owned(), REMOVED), (b, "dead".to_owned(), CLEARED)]);
        assert!(buffs.drain_status_ended().is_empty(), "drained once");
    }
}
