//! v3 `set_ability_active` and `swap_abilities` (one-shot AddBuff packets from generated Rust,
//! never stored as the packet itself).
//!
//! - **Inactive**: a hero's ability that is not active cannot be cast (Dota SetActivated(false));
//!   the hero carries the permanent row `__ability_inactive:<ability>`, which cast admission reads
//!   ([`inactive`]). Activating removes it.
//! - **Swap**: the slot holding `from` holds `to` instead (Hero::abilities is the slot order, so
//!   the same key casts `to`); `to` takes `from`'s rank when it has none (a sub ability is not
//!   learned on its own). Cooldowns stay per ability (`continue`); `reset` clears `to`'s.
//!   With a duration the hero carries `__ability_swap:<to>` (payload: the swap) and the timer
//!   row `__ability_swap_timer:<to>`; once the timer row expires, [`settle`] swaps back. Both
//!   rows are host state (`__` rows are never dispelled).
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Entity, Join, World, WorldExt};

use super::ability_runtime::BuffStore;
use super::comp::Hero;

pub const ACTIVE_KEY: &str = "__ability_active_v1";
pub const ACTIVE_BUFF: &str = "__ability_active";
pub const SWAP_KEY: &str = "__ability_swap_v1";
pub const SWAP_BUFF: &str = "__ability_swap";
const INACTIVE_PREFIX: &str = "__ability_inactive:";
const SWAPPED_PREFIX: &str = "__ability_swap:";
const TIMER_PREFIX: &str = "__ability_swap_timer:";

/// Whether the unit's ability is set inactive (cast admission refuses it).
pub fn inactive(buffs: &BuffStore, entity: Entity, ability: &str) -> bool {
    buffs.has(entity, &format!("{INACTIVE_PREFIX}{ability}"))
}

fn ability_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 96 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveSpec {
    pub ability: String,
    pub active: bool,
}

impl ActiveSpec {
    pub fn payload(&self) -> Value {
        json!({ (ACTIVE_KEY): { "ability": self.ability, "active": self.active } })
    }
    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Self> {
        if buff_id != ACTIVE_BUFF || duration != Fixed64::ONE || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(ACTIVE_KEY)?.as_object()?;
        if row.len() != 2 { return None; }
        let ability = row.get("ability")?.as_str()?;
        if !ability_id(ability) { return None; }
        Some(ActiveSpec { ability: ability.to_owned(), active: row.get("active")?.as_bool()? })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CooldownPolicy { Continue, Reset }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwapSpec {
    pub from: String,
    pub to: String,
    pub policy: CooldownPolicy,
    /// Swap back after this long; `None` keeps the swap.
    pub duration: Option<Fixed64>,
}

impl SwapSpec {
    pub fn payload(&self) -> Value {
        json!({ (SWAP_KEY): { "from": self.from, "to": self.to,
            "policy": match self.policy { CooldownPolicy::Continue => "continue", CooldownPolicy::Reset => "reset" },
            "duration_raw": self.duration.map(Fixed64::raw) } })
    }
    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Self> {
        if buff_id != SWAP_BUFF || duration != Fixed64::ONE || payload.as_object()?.len() != 1 { return None; }
        Self::from_row(payload.get(SWAP_KEY)?)
    }
    fn from_row(row: &Value) -> Option<Self> {
        let row = row.as_object()?;
        if row.len() != 4 { return None; }
        let (from, to) = (row.get("from")?.as_str()?, row.get("to")?.as_str()?);
        if !ability_id(from) || !ability_id(to) || from == to { return None; }
        let policy = match row.get("policy")?.as_str()? { "continue" => CooldownPolicy::Continue, "reset" => CooldownPolicy::Reset, _ => return None };
        let duration = match row.get("duration_raw")? {
            Value::Null => None,
            raw => Some(Fixed64::from_raw(raw.as_i64()?)).filter(|d| *d > Fixed64::ZERO && *d <= Fixed64::from_i32(3600)),
        };
        if !row.get("duration_raw")?.is_null() && duration.is_none() { return None; }
        Some(SwapSpec { from: from.to_owned(), to: to.to_owned(), policy, duration })
    }
}

/// Admission (cast settlement): a well-formed packet for a live hero.
pub fn admitted(world: &World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> bool {
    let well_formed = if buff_id == ACTIVE_BUFF { ActiveSpec::from_payload(buff_id, duration, payload).is_some() }
        else { SwapSpec::from_payload(buff_id, duration, payload).is_some() };
    well_formed && world.entities().is_alive(target) && world.read_storage::<Hero>().get(target).is_some()
}

/// Applies an `__ability_active` or `__ability_swap` packet.
pub fn apply(world: &mut World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) {
    if !admitted(world, target, buff_id, duration, payload) { return; }
    if let Some(spec) = ActiveSpec::from_payload(buff_id, duration, payload) {
        let key = format!("{INACTIVE_PREFIX}{}", spec.ability);
        let mut buffs = world.write_resource::<BuffStore>();
        if spec.active { buffs.remove(target, &key); } else { buffs.add(target, &key, Fixed64::from_raw(i64::MAX), json!({})); }
        return;
    }
    let Some(spec) = SwapSpec::from_payload(buff_id, duration, payload) else { return };
    if !swap(world, target, &spec.from, &spec.to, spec.policy) { return; }
    if let Some(duration) = spec.duration {
        let mut buffs = world.write_resource::<BuffStore>();
        buffs.add(target, &format!("{SWAPPED_PREFIX}{}", spec.to), Fixed64::from_raw(i64::MAX), spec.payload()[SWAP_KEY].clone());
        buffs.add(target, &format!("{TIMER_PREFIX}{}", spec.to), duration, json!({}));
    }
}

/// Puts `to` in `from`'s slot; false when the hero has no `from` slot.
fn swap(world: &mut World, hero_entity: Entity, from: &str, to: &str, policy: CooldownPolicy) -> bool {
    let mut heroes = world.write_storage::<Hero>();
    let Some(hero) = heroes.get_mut(hero_entity) else { return false };
    let Some(slot) = hero.abilities.iter().position(|id| id == from) else { return false };
    hero.abilities[slot] = to.to_owned();
    let rank = hero.get_ability_level(from);
    if hero.get_ability_level(to) <= 0 && rank > 0 { hero.ability_levels.insert(to.to_owned(), rank); }
    if policy == CooldownPolicy::Reset { hero.ability_cooldowns.remove(to); }
    true
}

/// Swaps back every timed swap whose timer ran out (every batch, after its outcomes).
pub fn settle(world: &mut World) {
    // Worlds without heroes (tests, TD) hold no swaps.
    if !world.has_value::<specs::storage::MaskedStorage<Hero>>() { return; }
    let due: Vec<(Entity, String, SwapSpec)> = {
        let buffs = world.read_resource::<BuffStore>();
        let entities = world.entities();
        let heroes = world.read_storage::<Hero>();
        let mut due = Vec::new();
        for (entity, _) in (&entities, &heroes).join() {
            for (id, entry) in buffs.iter_for(entity) {
                let Some(to) = id.strip_prefix(SWAPPED_PREFIX) else { continue };
                if buffs.has(entity, &format!("{TIMER_PREFIX}{to}")) { continue; }
                if let Some(spec) = SwapSpec::from_row(&entry.payload) { due.push((entity, id.to_owned(), spec)); }
            }
        }
        due.sort_by(|a, b| (a.0.id(), &a.1).cmp(&(b.0.id(), &b.1)));
        due
    };
    for (entity, row, spec) in due {
        world.write_resource::<BuffStore>().remove(entity, &row);
        swap(world, entity, &spec.to, &spec.from, spec.policy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    #[test]
    fn inactive_abilities_and_timed_swaps_round_trip() {
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
        let mut hero = Hero::default();
        hero.abilities = vec!["b24_q".into(), "b24_earth_prison".into()];
        hero.ability_levels.insert("b24_earth_prison".into(), 3);
        hero.start_cooldown("b24_shatter", Fixed64::from_i32(5));
        let h = world.create_entity().with(hero).build();
        let active = |a: bool| ActiveSpec { ability: "b24_q".into(), active: a }.payload();
        apply(&mut world, h, ACTIVE_BUFF, Fixed64::ONE, &active(false));
        assert!(inactive(&world.read_resource::<BuffStore>(), h, "b24_q"));
        apply(&mut world, h, ACTIVE_BUFF, Fixed64::ONE, &active(true));
        assert!(!inactive(&world.read_resource::<BuffStore>(), h, "b24_q"));
        let spec = SwapSpec { from: "b24_earth_prison".into(), to: "b24_shatter".into(), policy: CooldownPolicy::Reset,
            duration: Some(Fixed64::from_i32(4)) };
        apply(&mut world, h, SWAP_BUFF, Fixed64::ONE, &spec.payload());
        let slots = |w: &World| w.read_storage::<Hero>().get(h).unwrap().abilities.clone();
        assert_eq!(slots(&world), vec!["b24_q".to_string(), "b24_shatter".to_string()]);
        {
            let heroes = world.read_storage::<Hero>();
            let hero = heroes.get(h).unwrap();
            assert_eq!(hero.get_ability_level("b24_shatter"), 3, "the sub ability takes the rank");
            assert!(!hero.is_on_cooldown("b24_shatter"), "reset");
        }
        settle(&mut world);
        assert_eq!(slots(&world)[1], "b24_shatter", "the timer still runs");
        world.write_resource::<BuffStore>().remove(h, &format!("{TIMER_PREFIX}b24_shatter"));
        settle(&mut world);
        assert_eq!(slots(&world)[1], "b24_earth_prison", "swapped back");
        assert!(!world.read_resource::<BuffStore>().has(h, &format!("{SWAPPED_PREFIX}b24_shatter")));
        // A swap from an ability the hero does not hold changes nothing.
        let stray = SwapSpec { from: "nope".into(), to: "b24_shatter".into(), policy: CooldownPolicy::Continue, duration: None };
        apply(&mut world, h, SWAP_BUFF, Fixed64::ONE, &stray.payload());
        assert_eq!(slots(&world)[1], "b24_earth_prison");
    }
}
