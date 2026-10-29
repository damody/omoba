//! hero_command：產生的技能 Rust 改英雄元件的指令（v3 `modify_cooldown`、`set_ability_level`、
//! `set_health`）。A script cannot write the hero component, so it sends the unit a closed
//! envelope `add_stat_buff(unit, BUFF, ONE, payload)`; serial settlement (`handle_add_buff`)
//! applies it at once and never stores it as a buff, like ally_cleanse and blink_landing.
//!
//! - cooldown: `start` (the given seconds), `set` (same), `reduce` (shorter by the seconds, keeping
//!   the admitted total), `end`, on the hero component at once. Team replicas run the same
//!   script and settle the same envelope.
//! - level: the ability's rank on the hero (0..=its max), then `SkillLearn` so the ability's
//!   `on_learn` (passive stats, markers) sees the new rank.
//! - health: the unit's health becomes the value, within (0, max]; no damage or heal event
//!   (原作 SetHealth).
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

pub const KEY: &str = "__hero_command_v1";
pub const BUFF: &str = "__hero_command";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    CooldownStart { ability: String, seconds: Fixed64 },
    CooldownReduce { ability: String, seconds: Fixed64 },
    CooldownEnd { ability: String },
    Level { ability: String, level: i32 },
    Health { value: Fixed64 },
}

fn ability_ok(a: &str) -> bool {
    !a.is_empty() && a.len() <= 96 && a.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b':')
}

impl Command {
    pub fn payload(&self) -> Value {
        let row = match self {
            Command::CooldownStart { ability, seconds } => json!({ "op": "cooldown_start", "ability": ability, "raw": seconds.raw() }),
            Command::CooldownReduce { ability, seconds } => json!({ "op": "cooldown_reduce", "ability": ability, "raw": seconds.raw() }),
            Command::CooldownEnd { ability } => json!({ "op": "cooldown_end", "ability": ability, "raw": 0 }),
            Command::Level { ability, level } => json!({ "op": "level", "ability": ability, "raw": level }),
            Command::Health { value } => json!({ "op": "health", "ability": "", "raw": value.raw() }),
        };
        json!({ (KEY): row })
    }
    pub fn from_payload(buff_id: &str, payload: &Value) -> Option<Self> {
        if buff_id != BUFF || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 3 { return None; }
        let ability = row.get("ability")?.as_str()?.to_owned();
        let raw = row.get("raw")?.as_i64()?;
        let limit = 1_000_000 * 1024;
        let cmd = match row.get("op")?.as_str()? {
            "cooldown_start" if (0..=limit).contains(&raw) => Command::CooldownStart { ability, seconds: Fixed64::from_raw(raw) },
            "cooldown_reduce" if (0..=limit).contains(&raw) => Command::CooldownReduce { ability, seconds: Fixed64::from_raw(raw) },
            "cooldown_end" if raw == 0 => Command::CooldownEnd { ability },
            "level" if (0..=100).contains(&raw) => Command::Level { ability, level: raw as i32 },
            "health" if (0..=limit).contains(&raw) && ability.is_empty() => return Some(Command::Health { value: Fixed64::from_raw(raw) }),
            _ => return None,
        };
        match &cmd {
            Command::CooldownStart { ability, .. } | Command::CooldownReduce { ability, .. } | Command::CooldownEnd { ability }
            | Command::Level { ability, .. } => ability_ok(ability).then_some(cmd),
            Command::Health { .. } => Some(cmd),
        }
    }
}

/// Settle one command on `target` (handle_add_buff); a level change queues `SkillLearn`.
pub fn apply(world: &mut World, target: Entity, buff_id: &str, payload: &Value) -> bool {
    let Some(cmd) = Command::from_payload(buff_id, payload) else { return false };
    if !world.entities().is_alive(target) { return false; }
    match cmd {
        Command::Health { value } => {
            let mut props = world.write_storage::<super::comp::CProperty>();
            let Some(p) = props.get_mut(target) else { return false };
            if p.hp <= Fixed64::ZERO { return false; }
            p.hp = value.max(Fixed64::from_raw(1)).min(p.mhp);
            true
        }
        Command::Level { ability, level } => {
            let changed = {
                let mut heroes = world.write_storage::<super::comp::Hero>();
                let Some(hero) = heroes.get_mut(target) else { return false };
                if !hero.abilities.iter().any(|id| *id == ability) { return false; }
                let max = omoba_template_ids::ability_by_name(&ability).and_then(omoba_template_ids::ability_const)
                    .map_or(level, |c| i32::from(c.max_level));
                let level = level.clamp(0, max.max(0));
                let old = hero.get_ability_level(&ability);
                if old == level { false } else {
                    if level == 0 { hero.ability_levels.remove(&ability); } else { hero.ability_levels.insert(ability.clone(), level); }
                    true
                }
            };
            if changed {
                let new_level = world.read_storage::<super::comp::Hero>().get(target).map_or(0, |h| h.get_ability_level(&ability));
                world.write_resource::<super::scripting::ScriptEventQueue>().push(super::scripting::ScriptEvent::SkillLearn {
                    caster: target, skill_id: ability, new_level: u8::try_from(new_level).unwrap_or(0) });
            }
            changed
        }
        Command::CooldownStart { ability, seconds } => with_hero_ability(world, target, &ability, |h| h.start_cooldown(&ability, seconds)),
        Command::CooldownEnd { ability } => with_hero_ability(world, target, &ability, |h| h.start_cooldown(&ability, Fixed64::ZERO)),
        Command::CooldownReduce { ability, seconds } => with_hero_ability(world, target, &ability, |h| { h.reduce_cooldown(&ability, seconds); }),
    }
}

fn with_hero_ability(world: &mut World, target: Entity, ability: &str, f: impl FnOnce(&mut super::comp::Hero)) -> bool {
    let mut heroes = world.write_storage::<super::comp::Hero>();
    let Some(hero) = heroes.get_mut(target) else { return false };
    if !hero.abilities.iter().any(|id| id == ability) { return false; }
    f(hero);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_round_trips_strictly() {
        for c in [
            Command::CooldownStart { ability: "a34_frost_bolt".into(), seconds: Fixed64::from_i32(3) },
            Command::CooldownReduce { ability: "a34_frost_bolt".into(), seconds: Fixed64::from_raw(512) },
            Command::CooldownEnd { ability: "a34_frost_bolt".into() },
            Command::Level { ability: "a07_rouse".into(), level: 3 },
            Command::Health { value: Fixed64::from_i32(250) },
        ] {
            assert_eq!(Command::from_payload(BUFF, &c.payload()), Some(c.clone()));
        }
        let c = Command::Level { ability: "a07_rouse".into(), level: 3 };
        assert_eq!(Command::from_payload("other", &c.payload()), None);
        let mut bad = c.payload(); bad[KEY]["raw"] = json!(500);
        assert_eq!(Command::from_payload(BUFF, &bad), None, "levels are bounded");
        let mut bad = c.payload(); bad[KEY]["ability"] = json!("a b");
        assert_eq!(Command::from_payload(BUFF, &bad), None);
        let mut bad = Command::CooldownEnd { ability: "x".into() }.payload(); bad[KEY]["raw"] = json!(3);
        assert_eq!(Command::from_payload(BUFF, &bad), None);
    }
}
