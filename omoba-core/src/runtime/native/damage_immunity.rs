//! Damage-kind immunity, damage-to-heal conversion, and status / slow
//! resistance (G-4). All are ordinary BuffStore payload keys on the victim, so
//! authority and replicas that hold the same buffs settle the same way.
//!
//! Damage (settled in `game_processor::handle_damage`, the one place every
//! damage packet — attack, projectile, ability after armor/resistance —
//! reduces HP):
//! - `absolute_no_damage_{physical,magical,pure}` (StatKey, raw > 0.5): that
//!   component is not dealt.
//! - `absorb_{physical,magical,pure}_as_heal` (fraction 0..=1, summed, clamped):
//!   that share of the component heals the victim (ordinary received-healing
//!   policy) and is not dealt. With the kind's immunity the whole component is
//!   still not dealt, so the absorbed share is pure healing (例：A07W 毫髮無傷：
//!   所有傷害無效，非物理傷害的 30% 回復為生命).
//!
//! Debuffs (adjusted in `BuffStore::add`, see [`resist`]):
//! - `status_resistance` (fraction, strongest wins): stuns, roots, silences,
//!   ability slows and ability DoTs land with duration × (1 − r).
//! - `slow_resistance` (fraction, strongest wins): an ability slow lands with
//!   its move-speed reduction × (1 − r), and its attack-speed-constant
//!   reduction likewise (原作 `ms_unslow`，dummy.lua:827-852). Ability slows
//!   then stack per `BuffStore` (move speed 1−Π(1−sᵢ), attack speed strongest).
use super::ability_runtime::BuffStore;
use omb_script_abi::stat_keys::StatKey;
use omoba_sim::Fixed64;
use serde_json::Value;
use specs::Entity;

pub const ABSORB_PHYSICAL: &str = "absorb_physical_as_heal";
pub const ABSORB_MAGICAL: &str = "absorb_magical_as_heal";
pub const ABSORB_PURE: &str = "absorb_pure_as_heal";
pub const STATUS_RESISTANCE: &str = "status_resistance";
pub const SLOW_RESISTANCE: &str = "slow_resistance";

const HALF: Fixed64 = Fixed64::from_raw(512);

/// Sum of a raw payload key over every buff on `entity`.
fn sum_key(buffs: &BuffStore, entity: Entity, key: &str) -> Fixed64 {
    let mut total = 0i128;
    for (_, entry) in buffs.iter_for(entity) {
        if let Some(raw) = entry.payload.get(key).and_then(Value::as_i64) { total += i128::from(raw); }
    }
    Fixed64::from_raw(total.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64)
}

/// Strongest raw payload value of `key` on `entity`, clamped to 0..=1.
fn strongest(buffs: &BuffStore, entity: Entity, key: &str) -> Fixed64 {
    buffs.iter_for(entity).filter_map(|(_, entry)| entry.payload.get(key).and_then(Value::as_i64))
        .max().map_or(Fixed64::ZERO, Fixed64::from_raw).clamp(Fixed64::ZERO, Fixed64::ONE)
}

/// The packet after immunity and conversion: (physical, magical, pure, healing).
pub fn settle(buffs: &BuffStore, victim: Entity, physical: Fixed64, magical: Fixed64, pure: Fixed64)
    -> (Fixed64, Fixed64, Fixed64, Fixed64) {
    let immune = |key: StatKey| buffs.sum_add(victim, key) > HALF;
    // v3.1 `physical_immune` state: the physical part is not dealt (rules::vocab).
    let physical_immune = buffs.has(victim, super::rules::vocab::state::PHYSICAL_IMMUNE);
    let mut heal = Fixed64::ZERO;
    let mut part = |value: Fixed64, immune: bool, absorb_key: &str| -> Fixed64 {
        if value <= Fixed64::ZERO { return value; }
        let share = sum_key(buffs, victim, absorb_key).clamp(Fixed64::ZERO, Fixed64::ONE);
        let absorbed = Fixed64::from_raw((i128::from(value.raw()) * i128::from(share.raw()) / 1024) as i64);
        heal += absorbed;
        if immune { Fixed64::ZERO } else { value - absorbed }
    };
    let physical = part(physical, physical_immune || immune(StatKey::AbsoluteNoDamagePhysical), ABSORB_PHYSICAL);
    let magical = part(magical, immune(StatKey::AbsoluteNoDamageMagical), ABSORB_MAGICAL);
    let pure = part(pure, immune(StatKey::AbsoluteNoDamagePure), ABSORB_PURE);
    (physical, magical, pure, heal)
}

fn is_control(buff_id: &str) -> bool {
    use omb_script_abi::buff_ids::BuffId;
    [BuffId::Stun.as_str(), BuffId::Root.as_str(), BuffId::Silence.as_str()].contains(&buff_id)
}
fn is_ability_slow(payload: &Value) -> bool {
    payload.get("__aggregation_family").and_then(Value::as_str) == Some("generic_ability_slow")
}
fn is_ability_dot(buff_id: &str) -> bool { buff_id.starts_with("generic_dot:") }

/// Status and slow resistance of `entity` applied to a debuff about to land.
/// Returns the duration and payload to store; anything else passes unchanged.
pub fn resist(buffs: &BuffStore, entity: Entity, buff_id: &str, duration: Fixed64, mut payload: Value) -> (Fixed64, Value) {
    let slow = is_ability_slow(&payload);
    if !(slow || is_control(buff_id) || is_ability_dot(buff_id)) { return (duration, payload); }
    let scale = |value: i64, resistance: Fixed64| -> i64 {
        (i128::from(value) * i128::from((Fixed64::ONE - resistance).raw()) / 1024) as i64
    };
    // A status may declare that status resistance does not shorten it (rules::status).
    let status = if super::rules::status::ignores_status_resistance(&payload) { Fixed64::ZERO }
        else { strongest(buffs, entity, STATUS_RESISTANCE) };
    let duration = if status > Fixed64::ZERO && duration < Fixed64::from_raw(i64::MAX) {
        Fixed64::from_raw(scale(duration.raw(), status).max(1))
    } else { duration };
    if slow {
        let resistance = strongest(buffs, entity, SLOW_RESISTANCE);
        if resistance > Fixed64::ZERO {
            if let Some(reduction) = payload.get("move_speed_bonus").and_then(Value::as_i64) {
                payload["move_speed_bonus"] = scale(reduction, resistance).into();
            }
            // 原作 `Slow`（dummy.lua:846-852）同樣以 ms_unslow 削弱攻速減速。
            let attack = StatKey::AttackSpeedBonusConstant.as_str();
            if let Some(reduction) = payload.get(attack).and_then(Value::as_i64).filter(|v| *v < 0) {
                payload[attack] = scale(reduction, resistance).into();
            }
        }
    }
    (duration, payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use specs::{Builder, World, WorldExt};

    fn entity() -> Entity { let mut w = World::new(); w.create_entity().build() }
    fn i(v: i32) -> Fixed64 { Fixed64::from_i32(v) }

    #[test]
    fn nothing_on_the_victim_leaves_the_packet_alone() {
        let (store, e) = (BuffStore::new(), entity());
        assert_eq!(settle(&store, e, i(10), i(20), i(30)), (i(10), i(20), i(30), Fixed64::ZERO));
    }

    #[test]
    fn immunity_ignores_a_kind_and_absorption_heals_its_share() {
        let (mut store, e) = (BuffStore::new(), entity());
        store.add(e, "guard", i(3), json!({"absolute_no_damage_physical": 1024, ABSORB_MAGICAL: 1024, ABSORB_PURE: 512}));
        assert_eq!(settle(&store, e, i(100), i(40), i(20)), (Fixed64::ZERO, Fixed64::ZERO, i(10), i(50)));
    }

    #[test]
    fn immune_kinds_with_a_partial_absorption_heal_only_that_share() {
        let (mut store, e) = (BuffStore::new(), entity());
        store.add(e, "a07w", i(3), json!({"absolute_no_damage_physical": 1024, "absolute_no_damage_magical": 1024,
            "absolute_no_damage_pure": 1024, ABSORB_MAGICAL: 307, ABSORB_PURE: 307}));
        let (p, m, r, heal) = settle(&store, e, i(100), i(100), i(100));
        assert_eq!((p, m, r), (Fixed64::ZERO, Fixed64::ZERO, Fixed64::ZERO));
        assert_eq!(heal, Fixed64::from_raw(2 * 100 * 307), "30% of the non-physical damage only");
    }

    #[test]
    fn absorption_shares_sum_and_clamp_to_the_whole_packet() {
        let (mut store, e) = (BuffStore::new(), entity());
        store.add(e, "a", i(3), json!({ABSORB_MAGICAL: 768}));
        store.add(e, "b", i(3), json!({ABSORB_MAGICAL: 768}));
        assert_eq!(settle(&store, e, Fixed64::ZERO, i(100), Fixed64::ZERO), (Fixed64::ZERO, Fixed64::ZERO, Fixed64::ZERO, i(100)));
    }

    #[test]
    fn status_resistance_shortens_controls_slows_and_dots_and_slow_resistance_weakens_slows() {
        let (mut store, e) = (BuffStore::new(), entity());
        store.add(e, "ult", i(20), json!({STATUS_RESISTANCE: 512, SLOW_RESISTANCE: 512}));
        store.add(e, "stun", i(2), Value::Null);
        assert_eq!(store.get(e, "stun").unwrap().remaining, i(1), "half the stun");
        store.add(e, "generic_slow:x:1:1", i(4), json!({"move_speed_bonus": -768, "__aggregation_family": "generic_ability_slow"}));
        let slow = store.get(e, "generic_slow:x:1:1").unwrap();
        assert_eq!((slow.remaining, slow.payload["move_speed_bonus"].as_i64()), (i(2), Some(-384)), "half as long and half as strong");
        store.add(e, "generic_slow:y:1:1", i(4), json!({(StatKey::AttackSpeedBonusConstant.as_str()): -40 * 1024, "__aggregation_family": "generic_ability_slow"}));
        assert_eq!(store.get(e, "generic_slow:y:1:1").unwrap().payload[StatKey::AttackSpeedBonusConstant.as_str()].as_i64(), Some(-20 * 1024),
            "原作 ms_unslow also weakens the attack-speed slow");
        store.add(e, "generic_dot:x:1:1", i(4), json!({"amount_per_second": 1024}));
        assert_eq!(store.get(e, "generic_dot:x:1:1").unwrap().remaining, i(2));
        store.add(e, "haste", i(4), json!({"attack_speed_bonus_constant": 1024}));
        assert_eq!(store.get(e, "haste").unwrap().remaining, i(4), "other buffs are unaffected");
        store.remove(e, "ult");
        store.add(e, "root", i(2), Value::Null);
        assert_eq!(store.get(e, "root").unwrap().remaining, i(2), "full duration once the resistance ends");
    }

    #[test]
    fn a_status_declaring_ignore_keeps_its_duration_and_slow_resistance_still_applies() {
        use super::super::rules::status::ignoring_status_resistance;
        let (mut store, e) = (BuffStore::new(), entity());
        store.add(e, "ult", i(20), json!({STATUS_RESISTANCE: 512, SLOW_RESISTANCE: 512}));
        // 原作 ultimate_modifiers (events.lua:246-269): e.g. modifier_B29T.
        store.add(e, "stun", i(2), ignoring_status_resistance(Value::Null));
        assert_eq!(store.get(e, "stun").unwrap().remaining, i(2));
        store.add(e, "generic_slow:x:1:1", i(4), ignoring_status_resistance(
            json!({"move_speed_bonus": -768, "__aggregation_family": "generic_ability_slow"})));
        let slow = store.get(e, "generic_slow:x:1:1").unwrap();
        assert_eq!((slow.remaining, slow.payload["move_speed_bonus"].as_i64()), (i(4), Some(-384)));
    }
}
