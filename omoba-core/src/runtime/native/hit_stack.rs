//! Conditional on-hit stacking debuff (source A34T `A34T_OnHit`). A caster buff
//! carries a stacking rule under [`RULE_KEY`]. While the caster has that buff,
//! every hit that asks for it adds one stack of the rule's stat buff to the unit
//! hit:
//! - a projectile hit, when its payload names the caster buff
//!   (`ability_projectile::with_hit_stack`), decided at hit settlement;
//! - a script hit, by sending the victim the envelope
//!   `add_stat_buff(victim, APPLY_BUFF, 1, apply_payload(caster, caster_buff))`.
//!
//! Stacking (source A34_Matsu.lua:10-50): the first hit gives one stack; below
//! `max` each hit adds a stack and refreshes the duration; at `max` a hit does
//! nothing (no refresh) unless `refresh_at_max`. The stat value is
//! `per_stack × stacks`, and the whole buff expires at once. `heroes_only`
//! skips non-hero units. The rule is read from the caster buff at the moment of
//! the hit, so a later rank of the caster's ability changes later stacks only.
//! The envelope is never stored; the result is an ordinary BuffStore entry.
use omb_script_abi::stat_keys::StatKey;
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

use super::comp::{CProperty, Hero};

pub const RULE_KEY: &str = "__hit_stack_rule_v1";
pub const APPLY_BUFF: &str = "__hit_stack_apply";
pub const APPLY_KEY: &str = "__hit_stack_apply_v1";
/// Current stack count inside the stacked buff's payload.
pub const STACKS_KEY: &str = "__stacks";
pub const MAX_STACKS: u8 = 16;
pub const MAX_DURATION: i32 = 60;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    /// BuffStore id of the stacked buff on the unit hit.
    pub buff: String,
    pub stat: StatKey,
    pub per_stack: Fixed64,
    pub max: u8,
    pub duration: Fixed64,
    pub refresh_at_max: bool,
    pub heroes_only: bool,
}

/// Buff ids: 1..=64 printable ASCII bytes, never a host envelope (`__` prefix).
pub fn valid_buff_id(id: &str) -> bool {
    (1..=64).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_graphic()) && !id.starts_with("__")
}

impl Rule {
    pub fn valid(&self) -> bool {
        valid_buff_id(&self.buff) && (1..=MAX_STACKS).contains(&self.max)
            && self.per_stack != Fixed64::ZERO && self.per_stack.raw().unsigned_abs() <= 1_000 * 1024
            && self.duration >= Fixed64::from_raw(1) && self.duration <= Fixed64::from_i32(MAX_DURATION)
    }

    /// The rule row to merge into the caster buff's payload under [`RULE_KEY`].
    pub fn value(&self) -> Value {
        json!({ "buff": self.buff, "stat": self.stat.as_str(), "per_stack": self.per_stack.raw(), "max": self.max,
            "duration": self.duration.raw(), "refresh_at_max": self.refresh_at_max, "heroes_only": self.heroes_only })
    }

    pub fn from_value(value: &Value) -> Option<Self> {
        let row = value.as_object()?;
        if row.len() != 7 { return None; }
        let rule = Self {
            buff: row.get("buff")?.as_str()?.to_owned(),
            stat: StatKey::from_str_key(row.get("stat")?.as_str()?)?,
            per_stack: Fixed64::from_raw(row.get("per_stack")?.as_i64()?),
            max: u8::try_from(row.get("max")?.as_u64()?).ok()?,
            duration: Fixed64::from_raw(row.get("duration")?.as_i64()?),
            refresh_at_max: row.get("refresh_at_max")?.as_bool()?,
            heroes_only: row.get("heroes_only")?.as_bool()?,
        };
        rule.valid().then_some(rule)
    }

    /// Stat payload of `stacks` stacks.
    pub fn stacked_payload(&self, stacks: u8) -> Value {
        json!({ (self.stat.as_str()): self.per_stack.raw().saturating_mul(i64::from(stacks)), (STACKS_KEY): stacks })
    }
}

/// The rule a caster buff payload carries, if any.
pub fn rule_of(payload: &Value) -> Option<Rule> { Rule::from_value(payload.get(RULE_KEY)?) }

pub fn apply_payload(source_id: u32, source_generation: u32, caster_buff: &str) -> Value {
    json!({ (APPLY_KEY): { "source_id": source_id, "source_generation": source_generation, "caster_buff": caster_buff } })
}

/// Stacks the next hit leaves, or None when the hit changes nothing.
pub fn next_stacks(rule: &Rule, current: u8) -> Option<u8> {
    if current >= rule.max { return rule.refresh_at_max.then_some(rule.max); }
    Some(current + 1)
}

/// Serial settlement of one hit envelope. Returns whether a stack was applied.
pub fn apply(world: &mut World, target: Entity, buff_id: &str, payload: &Value) -> bool {
    if buff_id != APPLY_BUFF || payload.as_object().is_none_or(|o| o.len() != 1) { return false; }
    let Some(row) = payload.get(APPLY_KEY).and_then(Value::as_object).filter(|row| row.len() == 3) else { return false; };
    let (Some(id), Some(generation), Some(caster_buff)) = (
        row.get("source_id").and_then(Value::as_u64).and_then(|v| u32::try_from(v).ok()),
        row.get("source_generation").and_then(Value::as_u64).and_then(|v| u32::try_from(v).ok()),
        row.get("caster_buff").and_then(Value::as_str)) else { return false; };
    if !valid_buff_id(caster_buff) || generation == 0 { return false; }
    let source = world.entities().entity(id);
    let live = |e: Entity| world.entities().is_alive(e) && world.read_storage::<CProperty>().get(e).is_some_and(|p| p.hp > Fixed64::ZERO);
    if source.gen().id() as u32 != generation || !live(source) || !live(target) { return false; }
    let mut buffs = world.write_resource::<crate::runtime::BuffStore>();
    let Some(rule) = buffs.get(source, caster_buff).and_then(|entry| rule_of(&entry.payload)) else { return false; };
    if rule.heroes_only && world.read_storage::<Hero>().get(target).is_none() { return false; }
    let current = buffs.get(target, &rule.buff)
        .and_then(|entry| entry.payload.get(STACKS_KEY)?.as_u64())
        .map_or(0, |n| u8::try_from(n).unwrap_or(u8::MAX).min(rule.max));
    let Some(stacks) = next_stacks(&rule, current) else { return false; };
    buffs.add(target, &rule.buff, rule.duration, rule.stacked_payload(stacks));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn rule() -> Rule {
        Rule { buff: "a34t_magic_resist_shred".into(), stat: StatKey::MagicalResistanceBonus, per_stack: Fixed64::from_i32(-10),
            max: 3, duration: Fixed64::from_i32(3), refresh_at_max: false, heroes_only: true }
    }

    fn unit(w: &mut World, hero: bool) -> Entity {
        let builder = w.create_entity().with(CProperty { hp: Fixed64::from_i32(100), mhp: Fixed64::from_i32(100),
            msd: Fixed64::ZERO, def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO });
        if hero { builder.with(Hero::default()).build() } else { builder.build() }
    }

    fn world() -> (World, Entity) {
        let mut w = World::new();
        w.register::<CProperty>();w.register::<Hero>();
        w.insert(crate::runtime::BuffStore::default());
        let caster = unit(&mut w, true);
        (w, caster)
    }

    fn hit(w: &mut World, caster: Entity, target: Entity) -> bool {
        apply(w, target, APPLY_BUFF, &apply_payload(caster.id(), caster.gen().id() as u32, "rule_buff"))
    }

    fn stat(w: &World, e: Entity) -> (i64, Option<Fixed64>) {
        let buffs = w.read_resource::<crate::runtime::BuffStore>();
        (buffs.sum_add(e, StatKey::MagicalResistanceBonus).raw(), buffs.get(e, "a34t_magic_resist_shred").map(|b| b.remaining))
    }

    #[test]
    fn stacks_to_the_cap_refresh_below_it_and_never_at_it() {
        let (mut w, caster) = world();
        let victim = unit(&mut w, true);
        assert!(!hit(&mut w, caster, victim), "no rule buff on the caster: nothing");
        w.write_resource::<crate::runtime::BuffStore>().add(caster, "rule_buff", Fixed64::from_i32(22), json!({ (RULE_KEY): rule().value() }));
        assert!(hit(&mut w, caster, victim));
        assert_eq!(stat(&w, victim), (-10 * 1024, Some(Fixed64::from_i32(3))));
        w.write_resource::<crate::runtime::BuffStore>().tick(Fixed64::from_i32(2));
        assert!(hit(&mut w, caster, victim));
        assert_eq!(stat(&w, victim), (-20 * 1024, Some(Fixed64::from_i32(3))), "second stack refreshes");
        assert!(hit(&mut w, caster, victim));
        w.write_resource::<crate::runtime::BuffStore>().tick(Fixed64::from_i32(1));
        assert!(!hit(&mut w, caster, victim), "full stacks: no change, no refresh");
        assert_eq!(stat(&w, victim), (-30 * 1024, Some(Fixed64::from_i32(2))));
        w.write_resource::<crate::runtime::BuffStore>().tick(Fixed64::from_i32(2));
        assert_eq!(stat(&w, victim), (0, None), "all stacks expire together");
        assert!(hit(&mut w, caster, victim));
        assert_eq!(stat(&w, victim).0, -10 * 1024, "starts over");
    }

    #[test]
    fn heroes_only_dead_sources_and_malformed_rows_change_nothing() {
        let (mut w, caster) = world();
        w.write_resource::<crate::runtime::BuffStore>().add(caster, "rule_buff", Fixed64::from_i32(22), json!({ (RULE_KEY): rule().value() }));
        let creep = unit(&mut w, false);
        assert!(!hit(&mut w, caster, creep));
        let victim = unit(&mut w, true);
        let mut bad = apply_payload(caster.id(), caster.gen().id() as u32, "rule_buff");
        bad[APPLY_KEY]["extra"] = json!(1);
        assert!(!apply(&mut w, victim, APPLY_BUFF, &bad));
        assert!(!apply(&mut w, victim, "other", &apply_payload(caster.id(), caster.gen().id() as u32, "rule_buff")));
        w.write_storage::<CProperty>().get_mut(caster).unwrap().hp = Fixed64::ZERO;
        assert!(!hit(&mut w, caster, victim), "a dead caster no longer carries the effect");
        let mut refresh = rule(); refresh.refresh_at_max = true; refresh.max = 1;
        assert_eq!(next_stacks(&refresh, 1), Some(1));
        assert_eq!(Rule::from_value(&rule().value()), Some(rule()));
        for broken in [Rule { max: 0, ..rule() }, Rule { per_stack: Fixed64::ZERO, ..rule() }, Rule { buff: "__x".into(), ..rule() },
            Rule { duration: Fixed64::from_i32(61), ..rule() }] {
            assert!(Rule::from_value(&broken.value()).is_none());
        }
    }
}
