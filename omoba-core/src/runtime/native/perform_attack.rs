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
//!   An `effect_multiplier` other than 1 is not supported (the kit refuses it).
//! - not `instant` (E486): the attack launches an attack projectile at the attacker's projectile
//!   speed instead. The miss and the critical are rolled at launch (`on_attack_launch` fires
//!   then), the target's evasion and attack immunity when it arrives (`on_attack_hit` or a miss
//!   then), like an ordinary ranged attack's; the ordinary attack's record stays untouched.
//!   `cannot_miss` also skips the evasion on arrival ([`CannotMissAttacks`]); `use_orb` pays at
//!   launch, as an ordinary attack does. It needs `trigger_effects` (the kit refuses it without).
//! - `use_orb`: a toggle orb on the attacker pays and adds its hit like an ordinary attack
//!   (attack_orb), only on a hit.
//! Rolls are keyed by the attacker and the target, so several scripted attacks of one tick roll
//! independently and the authority and the replicas roll the same.
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Builder, Entity, World, WorldExt};
use super::comp::*;
use crate::runtime::BuffStore;

pub const BUFF: &str = "__perform_attack_v1";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flags { pub instant: bool, pub cannot_miss: bool, pub fake: bool, pub trigger_effects: bool, pub use_orb: bool }

/// The envelope payload: the target's handle and the flags.
pub fn payload(target_id: u32, target_generation: u32, flags: Flags) -> Value {
    json!({ "target": [target_id, target_generation], "instant": flags.instant, "cannot_miss": flags.cannot_miss, "fake": flags.fake,
        "trigger_effects": flags.trigger_effects, "use_orb": flags.use_orb })
}

/// The scripted attack projectiles that cannot miss (E486): their arrival rolls no evasion.
/// Authority data, part of the state hash; rows of projectiles that are gone are dropped.
#[derive(Clone, Debug, Default)]
pub struct CannotMissAttacks(pub std::collections::BTreeSet<Entity>);

fn parse(world: &World, payload: &Value) -> Option<(Entity, Flags)> {
    // Five keys: a kit from before E486, always instant.
    let row = payload.as_object().filter(|row| row.len() == 5 || row.len() == 6)?;
    let target = row.get("target")?.as_array().filter(|pair| pair.len() == 2)?;
    let (id, generation) = (u32::try_from(target[0].as_u64()?).ok()?, u32::try_from(target[1].as_u64()?).ok()?);
    let entity = world.entities().entity(id);
    if entity.gen().id() as u32 != generation || !world.entities().is_alive(entity) { return None; }
    let flag = |key: &str| row.get(key).and_then(Value::as_bool);
    let instant = if row.len() == 5 { true } else { flag("instant")? };
    Some((entity, Flags { instant, cannot_miss: flag("cannot_miss")?, fake: flag("fake")?, trigger_effects: flag("trigger_effects")?,
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
        // A projectile attack meets the target's immunity and evasion when it arrives.
        let immune = flags.instant && super::rules::vocab::attack_immune(&buffs, target);
        let missed = immune || (!flags.cannot_miss && super::attack_accuracy::source_misses(seed, tick, key, stats.miss_chance(attacker)));
        let evaded = flags.instant && !missed && !flags.cannot_miss
            && super::attack_accuracy::target_evades(seed, tick, key, stats.evasion_against(attacker, target));
        let sources = super::rules::attack::critical_sources(&buffs, attacker, Fixed64::ZERO, Fixed64::from_i32(2));
        let critical = if missed || evaded { None } else { super::rules::attack::roll_critical(&sources, seed, tick, key, attacker, None) };
        (physical, missed, evaded, critical)
    };
    if !flags.instant {
        let amount = if flags.fake { Fixed64::ZERO } else { critical.map_or(physical, |times| physical * times) };
        launch(world, attacker, target, target_pos, flags, amount, missed, critical.is_some());
        return Vec::new();
    }
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

/// The attack projectile of an attack that is not instant: what an ordinary ranged attack's
/// launch makes (game_processor), with the script attack's origin.
#[allow(clippy::too_many_arguments)]
fn launch(world: &mut World, attacker: Entity, target: Entity, target_pos: omoba_sim::Vec2, flags: Flags, physical: Fixed64,
    missed: bool, critical: bool) {
    let Some(from) = world.read_storage::<Pos>().get(attacker).map(|pos| pos.0) else { return };
    let speed = world.read_storage::<TAttack>().get(attacker).map_or(Fixed64::ZERO, |attack| attack.bullet_speed);
    let speed = if speed > Fixed64::ZERO { speed } else { Fixed64::from_i32(900) };
    super::rules::attack::scripted_launch(world, attacker, target, critical, missed);
    let flight = ((target_pos - from).length().to_f32_for_render() / speed.to_f32_for_render()).max(0.01);
    let projectile = world.create_entity().with(Pos(from)).with(Projectile {
        damage_stage: DamageStage::Raw,
        time_left: Fixed64::from_raw(((flight * 3.0 + 3.0) * omoba_sim::fixed::SCALE as f32) as i64),
        owner: attacker, tpos: target_pos, target: Some(target), radius: Fixed64::ZERO, msd: speed,
        damage_phys: physical, damage_magi: Fixed64::ZERO, damage_real: Fixed64::ZERO,
        damage_profile: omb_script_abi::types::DamageProfile::NORMAL.bits(),
        slow_factor: Fixed64::ZERO, slow_duration: Fixed64::ZERO, hit_radius: Fixed64::ZERO, stun_duration: Fixed64::ZERO,
        kind_id: 0, generation: 0,
    }).build();
    if let Some(mut origin) = super::moba_match::capture_projectile_origin(world, attacker) {
        origin.script_attack = !missed;
        origin.attack_missed = missed;
        origin.attack_critical = critical && !missed && physical > Fixed64::ZERO;
        origin.attack_lifesteal = if missed { Fixed64::ZERO } else {
            crate::runtime::UnitStats::from_refs(&world.read_resource::<BuffStore>(), false).lifesteal(attacker)
        };
        if world.try_fetch::<ProjectileOrigins>().is_none() { world.insert(ProjectileOrigins::default()); }
        world.write_resource::<ProjectileOrigins>().0.insert(projectile, origin);
    }
    if flags.cannot_miss {
        if world.try_fetch::<CannotMissAttacks>().is_none() { world.insert(CannotMissAttacks::default()); }
        let kept: std::collections::BTreeSet<Entity> = {
            let (entities, projectiles) = (world.entities(), world.read_storage::<Projectile>());
            world.read_resource::<CannotMissAttacks>().0.iter().copied()
                .filter(|entity| entities.is_alive(*entity) && projectiles.get(*entity).is_some()).collect()
        };
        world.write_resource::<CannotMissAttacks>().0 = kept;
        world.write_resource::<CannotMissAttacks>().0.insert(projectile);
    }
    if flags.use_orb { super::attack_orb::scripted_launch(world, attacker, target, projectile); }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn flags(fake: bool, trigger_effects: bool) -> Flags { Flags { instant: true, cannot_miss: true, fake, trigger_effects, use_orb: false } }
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
        let (mut w, attacker, _) = world();
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
        // A kit from before E486 sends five keys: an instant attack.
        let (mut w, attacker, _) = world();
        let old = json!({ "target": [id, generation], "cannot_miss": true, "fake": false, "trigger_effects": true, "use_orb": false });
        assert_eq!(outcomes(&mut w, attacker, &old).len(), 1);
    }

    /// E486: an attack that is not instant launches an attack projectile at the attacker's
    /// projectile speed; it fires on_attack_launch now and settles nothing yet.
    #[test]
    fn a_scripted_projectile_attack_launches_now_and_is_judged_when_it_arrives() {
        let (mut w, attacker, target) = world();
        w.register::<Projectile>();
        let (id, generation) = handle(target);
        let sure = Flags { instant: false, ..flags(false, true) };
        assert!(outcomes(&mut w, attacker, &payload(id, generation, sure)).is_empty(), "nothing settles at launch");
        use specs::Join;
        let shots: Vec<_> = (&w.entities(), &w.read_storage::<Projectile>()).join()
            .map(|(e, p)| (e, p.owner, p.target, p.msd, p.damage_phys)).collect();
        assert_eq!(shots.len(), 1);
        let (shot, owner, aimed, speed, phys) = shots[0];
        assert_eq!((owner, aimed, speed, phys), (attacker, Some(target), Fixed64::from_i32(900), Fixed64::from_i32(50)));
        let events = w.read_resource::<super::super::rules::attack::AttackEvents>().events.clone();
        assert!(matches!(events.as_slice(), [super::super::rules::attack::AttackEvent::Launch { attacker: a, target: t, missed: false, .. }]
            if *a == attacker && *t == target), "{events:?}");
        assert!(w.read_resource::<CannotMissAttacks>().0.contains(&shot), "a sure attack rolls no evasion on arrival");
        // An attack that can miss is not in the sure set.
        let (mut w, attacker, _) = world();
        w.register::<Projectile>();
        let unsure = Flags { instant: false, cannot_miss: false, ..flags(false, true) };
        outcomes(&mut w, attacker, &payload(id, generation, unsure));
        assert_eq!(w.read_storage::<Projectile>().count(), 1);
        assert!(w.try_fetch::<CannotMissAttacks>().is_none_or(|rows| rows.0.is_empty()));
    }
}
