//! Toggle orb (G-2 `toggle_mana_per_attack`). A toggle ability puts a marker
//! buff `attack_orb:<ability>` with payload `{KEY: {"ability": id}}` on its
//! caster. While present, each ordinary attack spends the rank's mana cost (or
//! switches the toggle off when mana is short) and the launched projectile
//! carries the orb; a hit that is not evaded adds the rank's bonus damage and
//! slows. Values always come from the caster's current rank.
//!
//! Source parity (A02W, A02.lua:A02W_OnAttackStart:22-47 / A02W_OnOrbImpact:49-69):
//! - The charge happens when the attack starts (hero windup, `AttackPhaseCue`),
//!   so an attack cancelled during its windup has already paid. An attack that
//!   launches without a recorded start (no windup cue) pays at launch instead.
//! - Mana short: the toggle switches off and this attack is an ordinary one.
//! - Target is a building: no mana, no bonus damage and no slows.
//! - `attack_orb_attack_speed_slow_constant` (optional extras, attack speed
//!   points): an AttackSpeedBonusConstant reduction in the shared
//!   [`ATTACK_SPEED_SLOW_FAMILY`], so only the strongest source applies
//!   (dummy.lua slow_self: `as_slow` keeps the most negative). Orbs without it
//!   keep the legacy `attack_speed_multiplier` slow.
use omoba_sim::Fixed64;
use serde::Serialize;
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};
use std::collections::BTreeMap;

use super::ability_runtime::BuffStore;
use super::comp::*;

pub const KEY: &str = "__attack_orb_v1";
/// Generated extras key of an orb's per-rank attack speed point reduction.
pub const ATTACK_SPEED_CONSTANT_KEY: &str = "attack_orb_attack_speed_slow_constant";
/// Aggregation family of constant attack speed slows: strongest source only.
pub const ATTACK_SPEED_SLOW_FAMILY: &str = "slow_self_attack_speed";

pub fn buff_id(ability: &str) -> String { format!("attack_orb:{ability}") }

pub fn payload(ability: &str) -> Value { json!({ (KEY): { "ability": ability } }) }

/// Strict marker: exactly `{KEY: {"ability": <id>}}` under the matching buff id.
pub fn orb_ability<'a>(buff: &str, payload: &'a Value) -> Option<&'a str> {
    if payload.as_object()?.len() != 1 { return None; }
    let row = payload.get(KEY)?.as_object()?;
    let ability = row.get("ability")?.as_str()?;
    (row.len() == 1 && buff == buff_id(ability)).then_some(ability)
}

/// Rank-resolved orb carried by one launched attack. Authority-private.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct OrbHit {
    pub ability: String,
    pub amount_raw: i64,
    pub damage_kind: u8,
    pub move_slow_raw: Option<i64>,
    pub attack_slow_raw: Option<i64>,
    pub slow_duration_raw: i64,
    /// Attack speed points removed (positive), from [`ATTACK_SPEED_CONSTANT_KEY`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack_speed_constant_raw: Option<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct OrbAttacks(pub BTreeMap<Entity, OrbHit>);

/// What the attack start of `attack_seq` decided, consumed by its launch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Started { Paid(OrbHit), Ordinary }

/// Authority/replica-local: attacker -> (attack sequence, decision). Both run
/// the same cue and launch settlement; nothing here is disclosed.
#[derive(Clone, Debug, Default)]
pub struct OrbStarts(pub BTreeMap<Entity, (u32, Started)>);

fn values(world: &World, source: Entity, ability: &str) -> Option<omoba_template_ids::AttackOrbValues> {
    let rank = world.read_storage::<Hero>().get(source)?.get_ability_level(ability);
    omoba_template_ids::ability_by_name(ability).and_then(omoba_template_ids::active_ability_const)?
        .attack_orb_at(u8::try_from(rank).ok().filter(|rank| *rank > 0)?)
}

fn attack_speed_constant(world: &World, source: Entity, ability: &str) -> Option<Fixed64> {
    let rank = world.read_storage::<Hero>().get(source)?.get_ability_level(ability);
    let index = usize::try_from(rank).ok()?.checked_sub(1)?;
    let content = omoba_template_ids::ability_by_name(ability).and_then(omoba_template_ids::active_ability_const)?;
    let value = content.extras.iter().find(|(name, _)| *name == ATTACK_SPEED_CONSTANT_KEY)?.1.get(index).copied()?;
    (value > Fixed64::ZERO && value <= Fixed64::from_i32(10_000)).then_some(value)
}

/// Toggle-on admission: the marker targets its own caster, for a learned orb ability.
pub fn admitted(world: &World, target: Entity, buff: &str, payload: &Value, caster: Option<Entity>) -> bool {
    let Some(ability) = orb_ability(buff, payload) else { return false; };
    caster.is_none_or(|caster| caster == target) && values(world, target, ability).is_some()
}

/// Charges the first active orb (orbs never stack; ordered by buff id) against
/// `target`. Mana short: switches it off. Building target: free and ordinary.
fn charge(world: &mut World, source: Entity, target: Option<Entity>) -> Started {
    let orb = world.read_resource::<BuffStore>().iter_for(source)
        .filter_map(|(id, entry)| orb_ability(id, &entry.payload).map(|ability| (id.to_owned(), ability.to_owned())))
        .min();
    let Some((buff, ability)) = orb else { return Started::Ordinary; };
    let Some(values) = values(world, source, &ability) else {
        world.write_resource::<BuffStore>().remove(source, &buff);
        return Started::Ordinary;
    };
    let pool_short = world.read_storage::<Hero>().get(source).and_then(|hero| hero.mana_pool.as_ref())
        .is_some_and(|pool| pool.current() < values.mana_per_attack);
    if pool_short {
        // A02.lua:36-39: checked first, even against a building; this attack is ordinary.
        world.write_resource::<BuffStore>().remove(source, &buff);
        return Started::Ordinary;
    }
    // A02.lua:42-46, 54: buildings cost nothing and receive nothing.
    if target.is_some_and(|target| world.read_storage::<IsBuilding>().get(target).is_some()) { return Started::Ordinary; }
    let paid = {
        let mut heroes = world.write_storage::<Hero>();
        match heroes.get_mut(source).and_then(|hero| hero.mana_pool.as_mut()) {
            // Mana-disabled rulesets have no pool: the orb is free there.
            None => true,
            Some(pool) => pool.spend(values.mana_per_attack).is_ok(),
        }
    };
    if !paid {
        world.write_resource::<BuffStore>().remove(source, &buff);
        return Started::Ordinary;
    }
    let constant = attack_speed_constant(world, source, &ability);
    Started::Paid(OrbHit {
        ability, amount_raw: values.amount.raw(), damage_kind: values.damage_kind,
        move_slow_raw: values.move_slow.map(Fixed64::raw), attack_slow_raw: values.attack_slow.map(Fixed64::raw),
        slow_duration_raw: values.slow_duration.raw(), attack_speed_constant_raw: constant.map(Fixed64::raw),
    })
}

/// Serial attack-start hook (hero windup cue of `attack_seq`): pays now; the
/// launch of the same sequence carries the result. A newer start replaces an
/// older one (a cancelled windup already paid and is not refunded).
pub fn on_attack_start(world: &mut World, source: Entity, target: Option<Entity>, attack_seq: u32) {
    let has_orb = world.try_fetch::<BuffStore>()
        .is_some_and(|buffs| buffs.iter_for(source).any(|(id, entry)| orb_ability(id, &entry.payload).is_some()));
    if !has_orb {
        if let Some(mut starts) = world.try_fetch_mut::<OrbStarts>() { starts.0.remove(&source); }
        return;
    }
    let started = charge(world, source, target);
    if world.try_fetch::<OrbStarts>().is_none() { world.insert(OrbStarts::default()); }
    world.write_resource::<OrbStarts>().0.insert(source, (attack_seq, started));
}

/// Serial attack launch hook. Uses the decision of this attack's start when
/// one was recorded; otherwise charges now (legacy launch-time charge).
pub fn on_launch(world: &mut World, source: Entity, target: Entity, projectile: Entity) {
    let seq = world.read_storage::<TAttack>().get(source).map(|attack| attack.attack_seq);
    let recorded = world.try_fetch_mut::<OrbStarts>().and_then(|mut starts| {
        let matches = starts.0.get(&source).is_some_and(|(started, _)| Some(*started) == seq);
        if matches { starts.0.remove(&source).map(|(_, started)| started) } else { None }
    });
    let started = match recorded { Some(started) => started, None => charge(world, source, Some(target)) };
    let Started::Paid(hit) = started else { return; };
    // Switched off after the start: the source modifier is gone, so OnAttackLanded never fires.
    if !world.read_resource::<BuffStore>().has(source, &buff_id(&hit.ability)) { return; }
    if world.try_fetch::<OrbAttacks>().is_none() { world.insert(OrbAttacks::default()); }
    world.write_resource::<OrbAttacks>().0.insert(projectile, hit);
}

/// On-hit payload for a non-evaded empowered hit, attributed to `source`.
pub fn hit_outcomes(hit: &OrbHit, source: Entity, target: Entity, pos: omoba_sim::Vec2) -> Vec<Outcome> {
    let amount = Fixed64::from_raw(hit.amount_raw);
    let zero = Fixed64::ZERO;
    let (phys, magi, real, profile) = match hit.damage_kind {
        0 => (amount, zero, zero, omb_script_abi::types::DamageProfile::NORMAL),
        1 => (zero, amount, zero, omb_script_abi::types::DamageProfile::ENERGY),
        _ => (zero, zero, amount, omb_script_abi::types::DamageProfile::TRUE),
    };
    // The orb's bonus is the ability's damage, not the attack's (A02W A02.lua:59-66 ApplyDamage
    // with `ability`): the source's spell amplification scales it (E188).
    let profile = profile.union(omb_script_abi::types::DamageProfile::SPELL);
    let mut outcomes = Vec::new();
    if amount > zero {
        outcomes.push(Outcome::RawDamage { pos, phys, magi, real, source, target, damage_profile: profile.bits(), origin: None });
    }
    let duration = Fixed64::from_raw(hit.slow_duration_raw);
    let caster = omb_script_abi::types::EntityHandle { id: source.id(), gen: source.gen().id() as u32 };
    if let Some(reduction) = hit.move_slow_raw.filter(|_| duration > zero) {
        outcomes.push(Outcome::AddBuff { target, buff_id: omb_script_abi::buff_ids::generic_slow_buff_id(&hit.ability, caster),
            duration, payload: json!({"move_speed_bonus": -reduction, "__aggregation_family": "generic_ability_slow"}) });
    }
    if let Some(reduction) = hit.attack_slow_raw.filter(|_| duration > zero) {
        outcomes.push(Outcome::AddBuff { target, buff_id: format!("orb_attack_slow:{}:{}:{}", hit.ability, caster.id, caster.gen),
            duration, payload: json!({"attack_speed_multiplier": 1024 - reduction}) });
    }
    if let Some(points) = hit.attack_speed_constant_raw.filter(|_| duration > zero) {
        let key = omb_script_abi::stat_keys::StatKey::AttackSpeedBonusConstant.as_str();
        outcomes.push(Outcome::AddBuff { target, buff_id: format!("orb_attack_speed_slow:{}:{}:{}", hit.ability, caster.id, caster.gen),
            duration, payload: json!({key: -points, "__aggregation_family": ATTACK_SPEED_SLOW_FAMILY}) });
    }
    outcomes
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;
    #[test]
    fn marker_is_strict_and_bound_to_its_buff_id() {
        assert_eq!(orb_ability("attack_orb:venom", &payload("venom")), Some("venom"));
        assert_eq!(orb_ability("attack_orb:other", &payload("venom")), None);
        let mut extra = payload("venom"); extra[KEY]["mana"] = json!(10);
        assert_eq!(orb_ability("attack_orb:venom", &extra), None);
        let mut sibling = payload("venom"); sibling["move_speed_bonus"] = json!(1);
        assert_eq!(orb_ability("attack_orb:venom", &sibling), None);
    }
    #[test]
    fn hit_outcomes_carry_damage_kind_and_both_slows() {
        let mut world = World::new();
        let (source, target) = (world.create_entity().build(), world.create_entity().build());
        let hit = OrbHit { ability: "venom".into(), amount_raw: 30 * 1024, damage_kind: 1, move_slow_raw: Some(92),
            attack_slow_raw: Some(92), slow_duration_raw: 4 * 1024, attack_speed_constant_raw: None };
        let rows = hit_outcomes(&hit, source, target, omoba_sim::Vec2::ZERO);
        assert!(matches!(&rows[0], Outcome::RawDamage { magi, phys, .. } if magi.raw() == 30 * 1024 && *phys == Fixed64::ZERO));
        assert!(matches!(&rows[1], Outcome::AddBuff { payload, .. } if payload["move_speed_bonus"] == -92));
        assert!(matches!(&rows[2], Outcome::AddBuff { payload, .. } if payload["attack_speed_multiplier"] == 1024 - 92));
        let none = OrbHit { amount_raw: 0, move_slow_raw: None, attack_slow_raw: None, ..hit };
        assert!(hit_outcomes(&none, source, target, omoba_sim::Vec2::ZERO).is_empty());
    }

    #[test]
    fn constant_attack_speed_slows_keep_only_the_strongest_source() {
        let mut world = World::new();
        let (a, b, target) = (world.create_entity().build(), world.create_entity().build(), world.create_entity().build());
        let hit = |points: i64| OrbHit { ability: "venom".into(), amount_raw: 0, damage_kind: 1, move_slow_raw: None,
            attack_slow_raw: None, slow_duration_raw: 4 * 1024, attack_speed_constant_raw: Some(points * 1024) };
        let mut buffs = BuffStore::default();
        for (source, points) in [(a, 9), (b, 21)] {
            for row in hit_outcomes(&hit(points), source, target, omoba_sim::Vec2::ZERO) {
                let Outcome::AddBuff { target, buff_id, duration, payload } = row else { panic!("slow buff") };
                assert!(payload.get("attack_speed_multiplier").is_none(), "points, not a percentage");
                buffs.add(target, &buff_id, duration, payload);
            }
        }
        let key = omb_script_abi::stat_keys::StatKey::AttackSpeedBonusConstant;
        assert_eq!(buffs.sum_add(target, key), Fixed64::from_i32(-21), "two orbs: the most negative only");
    }

    /// Hero with `a02_venom_arrow` rank `rank` (content with the constant slow) and `mana`.
    fn orb_world(rank: i32, mana: i32) -> (World, Entity, Entity, Entity) {
        let mut world = World::new();
        world.register::<Hero>(); world.register::<IsBuilding>(); world.register::<TAttack>();
        world.insert(BuffStore::default());
        let mut hero = Hero::default();
        hero.abilities.push("a02_venom_arrow".into());
        hero.ability_levels.insert("a02_venom_arrow".into(), rank);
        hero.mana_pool = Some(super::super::ability_runtime::ManaPool::new(Fixed64::from_i32(mana), Fixed64::from_i32(500)).unwrap());
        let source = world.create_entity().with(hero).with(TAttack::new(Fixed64::from_i32(50), Fixed64::from_i32(2),
            Fixed64::from_i32(700), Fixed64::from_i32(900))).build();
        let creep = world.create_entity().build();
        let tower = world.create_entity().with(IsBuilding).build();
        world.write_resource::<BuffStore>().add(source, &buff_id("a02_venom_arrow"), Fixed64::from_i32(1000), payload("a02_venom_arrow"));
        (world, source, creep, tower)
    }
    fn mana(world: &World, source: Entity) -> Fixed64 {
        world.read_storage::<Hero>().get(source).unwrap().mana_pool.as_ref().unwrap().current()
    }
    fn seq(world: &World, source: Entity) -> u32 { world.read_storage::<TAttack>().get(source).unwrap().attack_seq }

    #[test]
    fn attack_start_pays_once_buildings_are_free_and_short_mana_switches_off() {
        let (mut w, source, creep, tower) = orb_world(2, 25);
        let projectile = w.create_entity().build();
        // Start pays; the launch of the same attack carries the orb without paying again.
        { let s = seq(&w, source); on_attack_start(&mut w, source, Some(creep), s); }
        assert_eq!(mana(&w, source), Fixed64::from_i32(15));
        on_launch(&mut w, source, creep, projectile);
        assert_eq!(mana(&w, source), Fixed64::from_i32(15));
        let hit = w.read_resource::<OrbAttacks>().0.get(&projectile).cloned().expect("orb on the attack");
        assert_eq!(hit.amount_raw, 50 * 1024);
        assert_eq!(hit.attack_speed_constant_raw, Some(13 * 1024), "rank 2: -13 attack speed points");
        assert_eq!(hit.attack_slow_raw, None, "no percentage attack slow any more");
        // A building: no mana, no orb on the attack.
        w.write_storage::<TAttack>().get_mut(source).unwrap().attack_seq += 1;
        let at_tower = w.create_entity().build();
        { let s = seq(&w, source); on_attack_start(&mut w, source, Some(tower), s); }
        on_launch(&mut w, source, tower, at_tower);
        assert_eq!(mana(&w, source), Fixed64::from_i32(15));
        assert!(!w.read_resource::<OrbAttacks>().0.contains_key(&at_tower));
        // A cancelled windup already paid; the next start pays again.
        w.write_storage::<TAttack>().get_mut(source).unwrap().attack_seq += 1;
        { let s = seq(&w, source); on_attack_start(&mut w, source, Some(creep), s); }
        w.write_storage::<TAttack>().get_mut(source).unwrap().attack_seq += 1;
        { let s = seq(&w, source); on_attack_start(&mut w, source, Some(creep), s); }
        assert_eq!(mana(&w, source), Fixed64::from_i32(5), "15 -> 5, then short");
        assert!(!w.read_resource::<BuffStore>().has(source, &buff_id("a02_venom_arrow")), "short mana switches the toggle off");
        let late = w.create_entity().build();
        on_launch(&mut w, source, creep, late);
        assert!(!w.read_resource::<OrbAttacks>().0.contains_key(&late), "that attack is ordinary");
    }

    #[test]
    fn short_mana_switches_off_even_against_a_building_and_launch_without_start_pays_at_launch() {
        let (mut w, source, creep, tower) = orb_world(1, 5);
        { let s = seq(&w, source); on_attack_start(&mut w, source, Some(tower), s); }
        assert!(!w.read_resource::<BuffStore>().has(source, &buff_id("a02_venom_arrow")), "A02.lua:36-39 runs first");
        let _ = creep;
        let (mut w, source, _, _) = orb_world(1, 30);
        let creep = w.create_entity().build();
        let projectile = w.create_entity().build();
        on_launch(&mut w, source, creep, projectile);
        assert_eq!(mana(&w, source), Fixed64::from_i32(20), "no recorded start: legacy launch-time charge");
        assert!(w.read_resource::<OrbAttacks>().0.contains_key(&projectile));
    }
}
