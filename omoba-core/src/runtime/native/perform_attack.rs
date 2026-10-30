//! v3 `attack { attacker, target, instant = true, cannot_miss, fake, trigger_effects, use_orb }`
//! (原作 PerformAttack): a one-shot envelope (never stored) on the attacker,
//! settled in the outcome batch into the hit it makes right now. No windup, no projectile and no
//! cooldown; the attacker's ordinary attack in progress is untouched (its record stays).
//! - damage: the attacker's normal attack physical (`UnitStats::normal_attack_physical`) times its
//!   critical (rolled now, like an attack started outside a windup); `fake` deals none.
//! - miss: the attacker's miss chance and the target's evasion, unless `cannot_miss`; an
//!   attack-immune target is always missed.
//! - `trigger_effects`: the hit is an attack to the status triggers (`on_attack_launch`,
//!   `on_attack_hit`, `on_attacked`; rules::attack events), at the ordinary effect multiplier.
//!   An `effect_multiplier` other than 1 and an attack that is not `instant` (a projectile that
//!   would share the ordinary attack's record) are not supported (the kit refuses them).
//! - `use_orb`: a toggle orb on the attacker pays and adds its hit like an ordinary attack
//!   (attack_orb), only on a hit.
//! Rolls are keyed by the attacker and the target, so several scripted attacks of one tick roll
//! independently and the authority and the replicas roll the same.
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};
use super::comp::*;
use crate::runtime::BuffStore;

pub const BUFF: &str = "__perform_attack_v1";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flags { pub cannot_miss: bool, pub fake: bool, pub trigger_effects: bool, pub use_orb: bool }

/// The envelope payload: the target's handle and the flags.
pub fn payload(target_id: u32, target_generation: u32, flags: Flags) -> Value {
    json!({ "target": [target_id, target_generation], "cannot_miss": flags.cannot_miss, "fake": flags.fake,
        "trigger_effects": flags.trigger_effects, "use_orb": flags.use_orb })
}

fn parse(world: &World, payload: &Value) -> Option<(Entity, Flags)> {
    let row = payload.as_object().filter(|row| row.len() == 5)?;
    let target = row.get("target")?.as_array().filter(|pair| pair.len() == 2)?;
    let (id, generation) = (u32::try_from(target[0].as_u64()?).ok()?, u32::try_from(target[1].as_u64()?).ok()?);
    let entity = world.entities().entity(id);
    if entity.gen().id() as u32 != generation || !world.entities().is_alive(entity) { return None; }
    let flag = |key: &str| row.get(key).and_then(Value::as_bool);
    Some((entity, Flags { cannot_miss: flag("cannot_miss")?, fake: flag("fake")?, trigger_effects: flag("trigger_effects")?,
        use_orb: flag("use_orb")? }))
}

/// The outcomes of one envelope on `attacker` (empty when malformed or either side is not a living
/// unit). The attack events it records feed this tick's status triggers.
pub fn outcomes(world: &mut World, attacker: Entity, payload: &Value) -> Vec<Outcome> {
    let Some((target, flags)) = parse(world, payload) else { return Vec::new() };
    let living = |entity| world.read_storage::<CProperty>().get(entity).is_some_and(|p| p.hp > Fixed64::ZERO);
    if target == attacker || !world.entities().is_alive(attacker) || !living(attacker) || !living(target) { return Vec::new(); }
    let Some(base) = world.read_storage::<TAttack>().get(attacker).map(|attack| attack.atk_physic.v) else { return Vec::new() };
    let Some(target_pos) = world.read_storage::<Pos>().get(target).map(|pos| pos.0) else { return Vec::new() };
    let seed = world.read_resource::<MasterSeed>().0;
    let tick = world.read_resource::<Tick>().0 as u32;
    let key = super::rules::chance::roll_key_of(world, attacker) ^ crate::runtime::canonical_entity_id(target).rotate_left(17);
    let (physical, missed, evaded, critical) = {
        let buffs = world.read_resource::<BuffStore>();
        let stats = crate::runtime::ability_runtime::UnitStats::from_refs(&buffs, world.read_storage::<IsBuilding>().get(attacker).is_some());
        let physical = stats.normal_attack_physical(base, attacker);
        let immune = super::rules::vocab::attack_immune(&buffs, target);
        let missed = immune || (!flags.cannot_miss && super::attack_accuracy::source_misses(seed, tick, key, stats.miss_chance(attacker)));
        let evaded = !missed && !flags.cannot_miss
            && super::attack_accuracy::target_evades(seed, tick, key, stats.evasion_against(attacker, target));
        let sources = super::rules::attack::critical_sources(&buffs, attacker, Fixed64::ZERO, Fixed64::from_i32(2));
        let critical = if missed || evaded { None } else { super::rules::attack::roll_critical(&sources, seed, tick, key, attacker, None) };
        (physical, missed, evaded, critical)
    };
    let hit = !missed && !evaded;
    if flags.trigger_effects { super::rules::attack::scripted(world, attacker, target, critical.is_some(), missed, evaded); }
    let mut out = Vec::new();
    if hit && !flags.fake {
        let amount = critical.map_or(physical, |times| physical * times);
        if amount > Fixed64::ZERO {
            out.push(Outcome::Damage { pos: target_pos, phys: amount, magi: Fixed64::ZERO, real: Fixed64::ZERO, source: attacker,
                target, damage_profile: omb_script_abi::types::DamageProfile::NORMAL.bits(), predeclared: false });
        }
    }
    if flags.use_orb && hit {
        if let Some(orb) = super::attack_orb::scripted_hit(world, attacker, target) {
            out.extend(super::attack_orb::hit_outcomes(&orb, attacker, target, target_pos));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn world() -> (World, Entity, Entity) {
        let mut w = World::new();
        w.register::<CProperty>();w.register::<TAttack>();w.register::<Pos>();w.register::<IsBuilding>();
        w.register::<Hero>();w.register::<PlayerOwner>();
        w.insert(BuffStore::default());w.insert(MasterSeed(7));w.insert(Tick(3));
        let props = |hp| CProperty { hp: Fixed64::from_i32(hp), mhp: Fixed64::from_i32(hp), msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO };
        let attacker = w.create_entity().with(props(100)).with(Pos(omoba_sim::Vec2::ZERO))
            .with(TAttack::new(Fixed64::from_i32(50), Fixed64::ONE, Fixed64::from_i32(150), Fixed64::from_i32(900))).build();
        let target = w.create_entity().with(props(100)).with(Pos(omoba_sim::Vec2::new(Fixed64::from_i32(100), Fixed64::ZERO))).build();
        (w, attacker, target)
    }
    fn flags(fake: bool, trigger_effects: bool) -> Flags { Flags { cannot_miss: true, fake, trigger_effects, use_orb: false } }
    fn handle(e: Entity) -> (u32, u32) { (e.id(), e.gen().id() as u32) }

    #[test]
    fn a_scripted_attack_hits_now_with_the_attackers_attack_and_feeds_the_attack_triggers() {
        let (mut w, attacker, target) = world();
        let (id, generation) = handle(target);
        let out = outcomes(&mut w, attacker, &payload(id, generation, flags(false, true)));
        assert!(matches!(out.as_slice(), [Outcome::Damage { phys, source, target: t, .. }]
            if *phys == Fixed64::from_i32(50) && *source == attacker && *t == target), "{out:?}");
        let events = w.read_resource::<super::super::rules::attack::AttackEvents>().events.clone();
        assert!(events.iter().any(|e| matches!(e, super::super::rules::attack::AttackEvent::Hit { attacker: a, .. } if *a == attacker)));
        // fake: the attack triggers, no damage; no trigger_effects: no events.
        let (mut w, attacker, target) = world();
        assert!(outcomes(&mut w, attacker, &payload(id, generation, flags(true, true))).is_empty());
        assert!(w.try_fetch::<super::super::rules::attack::AttackEvents>().is_some());
        let (mut w, attacker, _) = world();
        assert_eq!(outcomes(&mut w, attacker, &payload(id, generation, flags(false, false))).len(), 1);
        assert!(w.try_fetch::<super::super::rules::attack::AttackEvents>().is_none(), "no trigger_effects, no attack events");
        // An attack-immune target is missed even by a sure attack; malformed and self attacks do nothing.
        let (mut w, attacker, target) = world();
        w.write_resource::<BuffStore>().add(target, super::super::rules::vocab::state::ATTACK_IMMUNE, Fixed64::ONE, serde_json::json!({}));
        assert!(outcomes(&mut w, attacker, &payload(id, generation, flags(false, false))).is_empty());
        assert!(outcomes(&mut w, attacker, &json!({ "target": [id, generation] })).is_empty());
        let (a_id, a_gen) = handle(attacker);
        assert!(outcomes(&mut w, attacker, &payload(a_id, a_gen, flags(false, false))).is_empty());
    }
}
