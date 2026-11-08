//! status_event：產生的技能 Rust 的狀態觸發（v3 `status.triggers`：on_removed／on_expired、
//! 持有者的 on_attack_hit／on_attacked_hit／on_take_damage／on_deal_damage）。
//!
//! A status the generated code applies carries one more payload key, [`KEY`]: the ability that
//! owns it, the rank, its v3 id, the source, and a mask of the events it listens to. The host then
//! calls `AbilityScript::on_status_event(holder, status, event)` of that ability:
//! - when the row leaves the holder: natural expiry ([`EXPIRED`]), removal or dispel
//!   ([`REMOVED`]), or all of the holder's buffs cleared, e.g. its death ([`CLEARED`]). `BuffStore`
//!   records these ([`BuffStore::drain_status_ended`]);
//! - for this tick's attack hits (rules::attack `AttackEvent::Hit`: ATTACK_HIT for the attacker,
//!   ATTACKED for the target) and the damage it took or dealt (`handle_damage` notes it in
//!   [`DamageNotes`] after the hit lands, only when a holder listens), in event order. (The
//!   legacy `ScriptEvent::Damage` is a damage request that deals its amount, not a notice.)
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
/// The holder started an attack (its windup; rules::attack AttackEvents Windup); `other` is the target.
pub const ATTACK_START: u8 = 10;
/// The holder's hit killed `other` (the packet's credit is the holder; v3 `on_kill`).
pub const KILL: u8 = 11;
/// The holder's attack left (rules::attack AttackEvents Launch: after the windup, as the projectile
/// leaves or an instant attack strikes; v3 `on_attack_launch`); `other` is the target.
pub const ATTACK_LAUNCH: u8 = 12;
/// The status's absorb (absorb.rs) took part of a hit (v3 `on_absorb`); `other` is the damage's
/// source, `amount` what it absorbed (E393).
pub const ABSORB: u8 = 13;
/// The status's absorb was used up by a hit (v3 `on_broken`), before the status is removed;
/// `other` is the damage's source, `amount` what that hit absorbed (E393).
pub const BROKEN: u8 = 14;
/// A chase summon (summon_chase, v3 `ai = chase_target`) reached its target: its reach ability's
/// hook with holder = the summon, status = its template id, `other` = the target and `source` =
/// its owner. Host-sent, never a mask bit (E406).
pub const CHASE_REACH: u8 = 15;
/// The holder moved since the status events last ran (v3 `on_moved`, 原作 OnUnitMoved: walking,
/// being pushed or teleported alike); `amount` is the straight distance between the two positions
/// (E424).
pub const MOVED: u8 = 16;
/// The holder spent mana on an ability or item (v3 `on_mana_spent`, 原作 OnSpentMana); `amount`
/// is the mana spent (E449).
pub const MANA_SPENT: u8 = 17;
/// The holder was made the target of an accepted order (v3 `on_order_targeted`, 原作
/// eventfororder): `other` is the unit ordered, `damage_kind` 0 an attack order, 1 a cast; a cast's
/// ability is `event.order.ability` (EVENT_ABILITY_ATTR, E465).
pub const ORDER_TARGETED: u8 = 18;
/// The holder was healed (v3 `on_heal_received`, 原作 OnHealthGained): `amount` is the health it
/// gained; the healer is not tracked (`other` invalid, E478).
pub const HEAL_RECEIVED: u8 = 19;
/// The holder's attack missed `other` (v3 `on_attack_miss`, rules::attack AttackEvent::Miss, E496):
/// `damage_kind` 0 when the holder missed at launch (blind, accuracy), 1 when the target evaded
/// or was attack-immune on arrival.
pub const ATTACK_MISS: u8 = 20;
/// An enemy ordered `other` to use a unit ability or item on the holder, and the order has not
/// executed yet (v3 `on_order_targeted` whose actions `cancel_cast`, 原作 ExecuteOrderFilter):
/// the hook may refuse it (ext CANCEL_ORDER); `event.order.ability` is that ability (EVENT_ABILITY_*).
/// The authority runs it; a team replica takes the refusal from the authority (order_filter).
pub const ORDER_FILTER: u8 = 21;

/// `Spec.mask` bits: which events the status listens to.
pub const ON_END: u32 = 1 << 0;
pub const ON_ATTACK_HIT: u32 = 1 << 1;
pub const ON_ATTACKED: u32 = 1 << 2;
pub const ON_TAKE_DAMAGE: u32 = 1 << 3;
pub const ON_DEAL_DAMAGE: u32 = 1 << 4;
pub const ON_TAKE_DAMAGE_PRE: u32 = 1 << 5;
pub const ON_DEAL_DAMAGE_PRE: u32 = 1 << 6;
pub const ON_ABILITY_EXECUTED: u32 = 1 << 7;
pub const ON_ATTACK_START: u32 = 1 << 8;
pub const ON_KILL: u32 = 1 << 9;
pub const ON_ATTACK_LAUNCH: u32 = 1 << 10;
pub const ON_ABSORB: u32 = 1 << 11;
pub const ON_BROKEN: u32 = 1 << 12;
pub const ON_MOVED: u32 = 1 << 13;
pub const ON_MANA_SPENT: u32 = 1 << 14;
pub const ON_ORDER_TARGETED: u32 = 1 << 15;
pub const ON_HEAL_RECEIVED: u32 = 1 << 16;
pub const ON_ATTACK_MISS: u32 = 1 << 17;
pub const ON_ORDER_FILTER: u32 = 1 << 18;
pub const ALL: u32 = ON_END | ON_ATTACK_HIT | ON_ATTACKED | ON_TAKE_DAMAGE | ON_DEAL_DAMAGE | ON_TAKE_DAMAGE_PRE | ON_DEAL_DAMAGE_PRE
    | ON_ABILITY_EXECUTED | ON_ATTACK_START | ON_KILL | ON_ATTACK_LAUNCH | ON_ABSORB | ON_BROKEN | ON_MOVED | ON_MANA_SPENT
    | ON_ORDER_TARGETED | ON_HEAL_RECEIVED | ON_ATTACK_MISS | ON_ORDER_FILTER;

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
            ATTACK_START => ON_ATTACK_START,
            KILL => ON_KILL,
            ATTACK_LAUNCH => ON_ATTACK_LAUNCH,
            ABSORB => ON_ABSORB,
            BROKEN => ON_BROKEN,
            MOVED => ON_MOVED,
            MANA_SPENT => ON_MANA_SPENT,
            ORDER_TARGETED => ON_ORDER_TARGETED,
            HEAL_RECEIVED => ON_HEAL_RECEIVED,
            ATTACK_MISS => ON_ATTACK_MISS,
            ORDER_FILTER => ON_ORDER_FILTER,
            _ => 0,
        };
        self.mask & bit != 0
    }
    /// The event for the BuffStore row `row` (its independent instance, if any, rides along).
    pub fn event(&self, row: &str, kind: u8, other: EntityHandle, amount: Fixed64, damage_kind: u8) -> StatusEvent {
        StatusEvent { kind, level: self.level, source: self.source, other, amount, damage_kind, instance: instance_of(row) }
    }
}

/// The `n` of an independent instance row `<runtime id>#<n>`; 0 for any other row.
pub fn instance_of(row: &str) -> u32 {
    omoba_template_ids::instance_number(row).and_then(|n| u32::try_from(n).ok()).unwrap_or(0)
}

/// Whether some status on `holder` listens to `kind` (a damage event is queued only then).
pub fn listening(buffs: &crate::runtime::BuffStore, holder: specs::Entity, kind: u8) -> bool {
    buffs.stat_rows(holder).any(|(_, entry)| entry.payload.get(KEY).is_some()
        && Spec::from_payload(&entry.payload).is_some_and(|spec| spec.listens(kind)))
}

/// A hit that landed, for on_take_damage / on_deal_damage (`amount`: the hp it took); `killer`:
/// the packet's credit when the hit was lethal (on_kill).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageNote { pub source: specs::Entity, pub target: specs::Entity, pub amount: Fixed64, pub damage_kind: u8,
    pub killer: Option<specs::Entity>,
    /// The packet's total before the target's mitigation (v3 `event.damage.raw`, E478).
    pub raw: Fixed64 }

/// The heals since status events last ran: the healed unit and the health it gained (E478).
#[derive(Clone, Debug, Default)]
pub struct HealNotes(pub Vec<(specs::Entity, Fixed64)>);

/// A heal that `target` gained `amount` health from, noted only when a status of it listens.
pub fn note_heal(world: &mut specs::World, target: specs::Entity, amount: Fixed64) {
    use specs::WorldExt;
    if amount <= Fixed64::ZERO || !world.try_fetch::<crate::runtime::BuffStore>().is_some_and(|buffs| listening(&buffs, target, HEAL_RECEIVED)) { return; }
    if world.try_fetch::<HealNotes>().is_none() { world.insert(HealNotes::default()); }
    world.write_resource::<HealNotes>().0.push((target, amount));
}

pub fn take_heal_notes(world: &mut specs::World) -> Vec<(specs::Entity, Fixed64)> {
    world.try_fetch_mut::<HealNotes>().map(|mut notes| std::mem::take(&mut notes.0)).unwrap_or_default()
}

/// The landed hits since status events last ran.
#[derive(Clone, Debug, Default)]
pub struct DamageNotes(pub Vec<DamageNote>);

pub fn note_damage(world: &mut specs::World, note: DamageNote) {
    use specs::WorldExt;
    if world.try_fetch::<DamageNotes>().is_none() { world.insert(DamageNotes::default()); }
    world.write_resource::<DamageNotes>().0.push(note);
}

pub fn take_damage_notes(world: &mut specs::World) -> Vec<DamageNote> {
    world.try_fetch_mut::<DamageNotes>().map(|mut notes| std::mem::take(&mut notes.0)).unwrap_or_default()
}

/// A status absorb that took part of a hit (ABSORB, and BROKEN when it was used up). `spec` is read
/// when it absorbed: a broken status is gone by the time the events run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbsorbNote { pub holder: specs::Entity, pub row: String, pub spec: Spec, pub source: specs::Entity,
    pub amount: Fixed64, pub broken: bool }

/// The absorbs since status events last ran.
#[derive(Clone, Debug, Default)]
pub struct AbsorbNotes(pub Vec<AbsorbNote>);

pub fn note_absorbs(world: &mut specs::World, notes: Vec<AbsorbNote>) {
    use specs::WorldExt;
    if notes.is_empty() { return; }
    if world.try_fetch::<AbsorbNotes>().is_none() { world.insert(AbsorbNotes::default()); }
    world.write_resource::<AbsorbNotes>().0.extend(notes);
}

pub fn take_absorb_notes(world: &mut specs::World) -> Vec<AbsorbNote> {
    world.try_fetch_mut::<AbsorbNotes>().map(|mut notes| std::mem::take(&mut notes.0)).unwrap_or_default()
}

/// Where each unit holding an `on_moved` status stood when status events last ran (E424).
#[derive(Clone, Debug, Default)]
pub struct MovedLedger(pub std::collections::BTreeMap<specs::Entity, omoba_sim::Vec2>);

/// The holders of an `on_moved` status that moved since the last call and how far, in entity
/// order; the ledger then holds their current positions. A holder seen for the first time, or
/// one that holds no such status any more, moves nothing.
pub fn take_moves(world: &mut specs::World) -> Vec<(specs::Entity, Fixed64)> {
    use specs::WorldExt;
    let holders: Vec<(specs::Entity, omoba_sim::Vec2)> = {
        let buffs = world.read_resource::<crate::runtime::BuffStore>();
        let positions = world.read_storage::<super::comp::Pos>();
        let mut holders: Vec<specs::Entity> = buffs.entities_with_key(KEY)
            .filter(|holder| world.entities().is_alive(*holder)
                && buffs.iter_for(*holder).any(|(_, b)| Spec::from_payload(&b.payload).is_some_and(|s| s.listens(MOVED))))
            .collect();
        holders.sort_by_key(|e| (e.id(), e.gen().id()));
        holders.dedup();
        holders.into_iter().filter_map(|holder| positions.get(holder).map(|p| (holder, p.0))).collect()
    };
    if holders.is_empty() {
        if let Some(mut ledger) = world.try_fetch_mut::<MovedLedger>() { ledger.0.clear(); }
        return Vec::new();
    }
    if world.try_fetch::<MovedLedger>().is_none() { world.insert(MovedLedger::default()); }
    let mut ledger = world.write_resource::<MovedLedger>();
    let mut moves = Vec::new();
    let mut next = std::collections::BTreeMap::new();
    for (holder, at) in holders {
        if let Some(before) = ledger.0.get(&holder).filter(|before| **before != at) { moves.push((holder, (at - *before).length())); }
        next.insert(holder, at);
    }
    ledger.0 = next;
    moves
}

/// The `damage_kind` of a damage event: the largest-first type share (physical, magical, pure)
/// and the reflection bit of the packet's profile (read before its flags are stripped).
pub fn damage_kind(by_type: [Fixed64; 3], profile: u32) -> u8 {
    let kind = if by_type[0] > Fixed64::ZERO { 0 } else if by_type[1] > Fixed64::ZERO { 1 } else { 2 };
    let reflected = profile & omb_script_abi::types::DamageProfile::REFLECTION.bits() != 0;
    kind | if reflected { StatusEvent::REFLECTION } else { 0 }
}

/// How far status events have read rules::attack's AttackEvents: the (tick, count) of events
/// already delivered, so every windup and hit reaches its holder once whatever the system order.
#[derive(Clone, Copy, Debug, Default)]
pub struct WindupCursor { pub tick: u64, pub seen: usize }

/// An attack event a status hears.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackSeen {
    /// The holder started an attack (ATTACK_START).
    Windup { attacker: specs::Entity, target: Option<specs::Entity> },
    /// The attack left (ATTACK_LAUNCH).
    Launch { attacker: specs::Entity, target: specs::Entity },
    /// The attack hit (ATTACK_HIT for the attacker, ATTACKED for the target).
    Hit { attacker: specs::Entity, target: specs::Entity },
    /// The attack missed (ATTACK_MISS for the attacker); `evaded` when the target avoided it.
    Miss { attacker: specs::Entity, target: specs::Entity, evaded: bool },
}

/// The windups and hits not delivered yet, in order; advances the cursor.
pub fn take_attack_events(world: &mut specs::World) -> Vec<AttackSeen> {
    use specs::WorldExt;
    use super::rules::attack::{AttackEvent, AttackEvents};
    let Some(events) = world.try_fetch::<AttackEvents>().map(|e| (e.tick, e.events.clone())) else { return Vec::new() };
    if world.try_fetch::<WindupCursor>().is_none() { world.insert(WindupCursor::default()); }
    let mut cursor = world.write_resource::<WindupCursor>();
    let (tick, list) = events;
    let start = if cursor.tick == tick { cursor.seen } else { 0 };
    let out = list.iter().skip(start).filter_map(|e| match e {
        AttackEvent::Windup { attacker, target, .. } => Some(AttackSeen::Windup { attacker: *attacker, target: *target }),
        AttackEvent::Launch { attacker, target, .. } => Some(AttackSeen::Launch { attacker: *attacker, target: *target }),
        AttackEvent::Hit { attacker, target, .. } => Some(AttackSeen::Hit { attacker: *attacker, target: *target }),
        AttackEvent::Miss { attacker, target, reason, .. } => Some(AttackSeen::Miss { attacker: *attacker, target: *target,
            evaded: *reason == super::rules::attack::MissReason::Evaded }),
        _ => None }).collect();
    *cursor = WindupCursor { tick, seen: list.len() };
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::BuffStore;
    use specs::{Builder, World, WorldExt};

    fn spec(mask: u32) -> Spec {
        Spec { ability: "b16_beast_union".into(), level: 2, status: "b16t_union".into(), source: EntityHandle { id: 4, gen: 1 }, mask }
    }

    /// E424: on_moved holders report how far they moved since the last read, once per change.
    #[test]
    fn moved_holders_report_the_distance_since_the_last_read() {
        let mut world = World::new();
        world.register::<super::super::comp::Pos>();
        world.insert(BuffStore::new());
        let at = |x: i32, y: i32| super::super::comp::Pos(omoba_sim::Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)));
        let holder = world.create_entity().with(at(0, 0)).build();
        let deaf = world.create_entity().with(at(0, 0)).build();
        let marked = |mask: u32| json!({ KEY: spec(mask).row() });
        world.write_resource::<BuffStore>().add(holder, "b16t_union", Fixed64::from_i32(9), marked(ON_MOVED));
        world.write_resource::<BuffStore>().add(deaf, "b16t_union", Fixed64::from_i32(9), marked(ON_END));
        assert!(take_moves(&mut world).is_empty(), "the first read only records");
        *world.write_storage::<super::super::comp::Pos>().get_mut(holder).unwrap() = at(3, 4);
        *world.write_storage::<super::super::comp::Pos>().get_mut(deaf).unwrap() = at(3, 4);
        assert_eq!(take_moves(&mut world), vec![(holder, Fixed64::from_i32(5))]);
        assert!(take_moves(&mut world).is_empty(), "standing still");
        assert!(spec(ON_MOVED).listens(MOVED) && !spec(ON_END).listens(MOVED) && ALL & ON_MOVED != 0);
        world.write_resource::<BuffStore>().remove(holder, "b16t_union");
        assert!(take_moves(&mut world).is_empty());
        assert!(world.read_resource::<MovedLedger>().0.is_empty(), "a holder without the status leaves the ledger");
    }

    #[test]
    fn an_independent_instance_row_names_its_instance_in_the_event() {
        // v3 stacking = "independent": every row `<runtime id>#<n>` hears its own events, and the
        // hook learns which one (so `remove_status` of this instance removes only it).
        let s = spec(ON_END);
        assert_eq!(s.event("b16t_union#3", EXPIRED, EntityHandle::INVALID, Fixed64::ZERO, 0).instance, 3);
        assert_eq!(s.event("b16t_union", EXPIRED, EntityHandle::INVALID, Fixed64::ZERO, 0).instance, 0);
        assert_eq!(s.event("b16t_union#x", EXPIRED, EntityHandle::INVALID, Fixed64::ZERO, 0).instance, 0);
        assert_eq!(instance_of("#4"), 0, "an instance needs a runtime id");
    }

    #[test]
    fn payload_round_trips_strictly() {
        let s = spec(ON_END | ON_TAKE_DAMAGE);
        let payload = json!({ "magic_resist": 1, (KEY): s.row() });
        assert_eq!(Spec::from_payload(&payload), Some(s.clone()));
        assert!(s.listens(EXPIRED) && s.listens(CLEARED) && s.listens(TAKE_DAMAGE) && !s.listens(ATTACK_HIT));
        assert!(!s.listens(ABILITY_EXECUTED) && spec(ON_ABILITY_EXECUTED).listens(ABILITY_EXECUTED));
        assert!(!s.listens(MANA_SPENT) && spec(ON_MANA_SPENT).listens(MANA_SPENT) && !spec(ON_MANA_SPENT).listens(MOVED), "E449");
        assert!(!s.listens(ATTACK_START) && spec(ON_ATTACK_START).listens(ATTACK_START));
        assert!(!s.listens(KILL) && spec(ON_KILL).listens(KILL));
        assert!(!s.listens(ABSORB) && spec(ON_ABSORB).listens(ABSORB) && spec(ON_BROKEN).listens(BROKEN));
        assert!(!s.listens(MOVED) && spec(ON_MOVED).listens(MOVED));
        assert!(!s.listens(ORDER_TARGETED) && spec(ON_ORDER_TARGETED).listens(ORDER_TARGETED), "E465");
        assert!(!s.listens(HEAL_RECEIVED) && spec(ON_HEAL_RECEIVED).listens(HEAL_RECEIVED), "E478");
        assert!(!s.listens(ORDER_FILTER) && spec(ON_ORDER_FILTER).listens(ORDER_FILTER) && !spec(ON_ORDER_FILTER).listens(ORDER_TARGETED));
        let mut bad = payload.clone(); bad[KEY]["mask"] = json!(1 << 19);
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
        let drained = buffs.drain_status_ended();
        let left: Vec<_> = drained.iter().map(|(_, id, _, _, remaining)| (id.clone(), *remaining)).collect();
        let ended: Vec<_> = drained.into_iter().map(|(e, id, _, kind, _)| (e, id, kind)).collect();
        assert_eq!(ended, vec![(a, "short".to_owned(), EXPIRED), (b, "gone".to_owned(), REMOVED), (b, "dead".to_owned(), CLEARED)]);
        let five_less_tick = Fixed64::from_raw(Fixed64::from_i32(5).raw() - 17);
        assert_eq!(left, vec![("short".to_owned(), Fixed64::ZERO), ("gone".to_owned(), five_less_tick), ("dead".to_owned(), five_less_tick)],
            "an end carries the duration left; an expiry none");
        assert!(buffs.drain_status_ended().is_empty(), "drained once");
    }

    #[test]
    fn attack_windups_and_hits_are_taken_once_in_order_and_damage_kinds_carry_reflection() {
        use super::super::rules::attack::{AttackEvent, AttackEvents};
        let mut world = World::new();
        let (a, b) = (world.create_entity().build(), world.create_entity().build());
        world.insert(AttackEvents { tick: 3, events: vec![
            AttackEvent::Windup { attacker: a, seq: 1, target: Some(b), critical: false },
            AttackEvent::Launch { attacker: a, seq: 1, target: b, critical: false, missed: false },
            AttackEvent::Hit { attacker: a, target: b, projectile: None },
        ] });
        assert_eq!(take_attack_events(&mut world), vec![AttackSeen::Windup { attacker: a, target: Some(b) },
            AttackSeen::Launch { attacker: a, target: b }, AttackSeen::Hit { attacker: a, target: b }]);
        assert!(spec(ON_ATTACK_LAUNCH).listens(ATTACK_LAUNCH) && !spec(ON_ATTACK_HIT).listens(ATTACK_LAUNCH));
        assert!(take_attack_events(&mut world).is_empty(), "taken once");
        world.write_resource::<AttackEvents>().events.push(AttackEvent::Hit { attacker: b, target: a, projectile: None });
        assert_eq!(take_attack_events(&mut world), vec![AttackSeen::Hit { attacker: b, target: a }], "only the new one");
        let n = Fixed64::from_i32;
        let reflection = omb_script_abi::types::DamageProfile::REFLECTION.bits();
        assert_eq!(damage_kind([n(0), n(5), n(1)], 0), 1);
        assert_eq!(damage_kind([n(0), n(0), n(1)], reflection), 2 | StatusEvent::REFLECTION);
        let mut buffs = BuffStore::new();
        buffs.add(a, "mark", Fixed64::ONE, json!({ (KEY): spec(ON_TAKE_DAMAGE).row() }));
        assert!(listening(&buffs, a, TAKE_DAMAGE) && !listening(&buffs, a, DEAL_DAMAGE) && !listening(&buffs, b, TAKE_DAMAGE));
    }
}
