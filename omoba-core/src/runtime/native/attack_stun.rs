//! Ordinary-attack stun chance (`attack_stun_chance` / `attack_stun_duration`
//! buff payload keys, e.g. matchlock_gun, B04T 種子島).
//!
//! The strongest chance among the attacker's buffs wins; that same buff may
//! declare two options (default off, which keeps the legacy behaviour):
//! - [`ON_LAUNCH_KEY`]: the stun is rolled and applied to the attack target
//!   when the attack launches, hit or miss (source datadriven `OnAttack`,
//!   B04.lua:171-175). Legacy: rolled at launch, applied by a non-missed hit.
//! - [`SKIP_BUILDINGS_KEY`]: buildings are never stunned
//!   (B04.lua:172 `IsHero() or IsCreep()`).
//! - [`super::magic_immunity::PIERCE_KEY`]: the launch-time stun lands on a magic-immune
//!   target (B04T, E184).
use serde_json::Value;
use specs::Entity;

use super::ability_runtime::BuffStore;

pub const ON_LAUNCH_KEY: &str = "attack_stun_on_launch";
pub const SKIP_BUILDINGS_KEY: &str = "attack_stun_skip_buildings";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AttackStun {
    pub chance: f32,
    pub duration: f32,
    pub on_launch: bool,
    pub skip_buildings: bool,
    pub pierces_magic_immunity: bool,
}

fn flag(payload: &Value, key: &str) -> bool {
    match payload.get(key) {
        Some(Value::Bool(value)) => *value,
        Some(value) => value.as_i64().is_some_and(|v| v > 0),
        None => false,
    }
}

/// The attacker's strongest attack stun. Ties keep the first entry in the
/// BuffStore's deterministic order (strictly greater chance replaces).
pub fn select(buffs: &BuffStore, source: Entity) -> AttackStun {
    let mut best = AttackStun::default();
    for (_, entry) in buffs.iter_for(source) {
        let chance = entry.payload.get("attack_stun_chance").and_then(Value::as_f64).unwrap_or(0.0) as f32;
        if chance > best.chance {
            best = AttackStun {
                chance,
                duration: entry.payload.get("attack_stun_duration").and_then(Value::as_f64).unwrap_or(0.0) as f32,
                on_launch: flag(&entry.payload, ON_LAUNCH_KEY),
                skip_buildings: flag(&entry.payload, SKIP_BUILDINGS_KEY),
                pierces_magic_immunity: flag(&entry.payload, super::magic_immunity::PIERCE_KEY),
            };
        }
    }
    best
}

impl AttackStun {
    /// Whether this attack may roll at all against a target (`building` = target is a building).
    pub fn applies(&self, missed: bool, building: bool) -> bool {
        self.chance > 0.0 && self.duration > 0.0 && (self.on_launch || !missed) && !(self.skip_buildings && building)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_sim::Fixed64;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn strongest_chance_carries_its_own_launch_and_building_options() {
        let mut world = World::new();
        let source = world.create_entity().build();
        let mut buffs = BuffStore::default();
        assert_eq!(select(&buffs, source), AttackStun::default());
        buffs.add(source, "matchlock", Fixed64::from_i32(10), serde_json::json!({"attack_stun_chance": 0.2, "attack_stun_duration": 1.0}));
        let legacy = select(&buffs, source);
        assert!(!legacy.on_launch && !legacy.skip_buildings);
        assert!(legacy.applies(false, true) && !legacy.applies(true, false), "legacy: hit only, buildings included");
        buffs.add(source, "gun", Fixed64::from_i32(10), serde_json::json!({"attack_stun_chance": 0.38, "attack_stun_duration": 0.2,
            ON_LAUNCH_KEY: 1, SKIP_BUILDINGS_KEY: 1}));
        let gun = select(&buffs, source);
        assert_eq!((gun.chance, gun.on_launch, gun.skip_buildings), (0.38, true, true));
        assert!(gun.applies(true, false), "launch-time stun ignores a miss");
        assert!(!gun.applies(false, true), "never a building");
        buffs.add(source, "gun", Fixed64::from_i32(10), serde_json::json!({"attack_stun_chance": 0.38, "attack_stun_duration": 0.2,
            ON_LAUNCH_KEY: 0}));
        assert!(!select(&buffs, source).on_launch, "zero is off");
    }
}
