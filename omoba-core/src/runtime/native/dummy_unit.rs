//! 原作 dummy／majia 單位（`summons.lua` 的 `dummy = true`，所有英雄與道具共用）：只用來掛狀態與
//! 光環的身體。沒有生命、攻擊、碰撞與指令；存活時間到就移除（summon_tick）。spawn 時
//! （game_processor `commit_summoned_spawn`）不建立 TAttack、碰撞半徑為 0，並在整個存活時間掛上
//! [`STATES`]：不受傷害、不會被普攻或單位技能選到、範圍選取也選不到（無敵）、不擋其他單位、
//! 擁有者不能下指令、不顯示血條、不上小地圖。模板給了生命（例如可以被打掉的尖刺）時只掛 [`BREAKABLE_STATES`]：
//! 照常可以被攻擊、選取與擊殺，仍不擋路、不聽指令。呈現只播 cue，不顯示模型（前端讀 `SummonStats::dummy`）。
use omoba_sim::Fixed64;
use specs::Entity;
use super::rules::vocab::state;
use crate::runtime::BuffStore;

/// The states a dummy holds for its whole life.
pub const STATES: [&str; 7] = [state::INVULNERABLE, state::UNSELECTABLE, state::ATTACK_IMMUNE, state::COMMAND_RESTRICTED,
    state::NO_HEALTHBAR, state::NOT_ON_MINIMAP, super::ability_runtime::NO_UNIT_COLLISION];

/// The states of a dummy with life: it can be attacked, selected and killed.
pub const BREAKABLE_STATES: [&str; 2] = [state::COMMAND_RESTRICTED, super::ability_runtime::NO_UNIT_COLLISION];

/// A summon template a spawn admits: a dummy (its zero body is checked by the catalog), or a
/// body with life that never attacks (`no_attack`, E546: C16 鬼影) or attacks at a positive
/// interval from a range of 0 or more (a melee range 0 reaches what it touches, edge to edge:
/// A21 鐵扇).
pub fn body_valid(stats: &omoba_template_ids::SummonStats) -> bool {
    stats.dummy || (stats.hp > Fixed64::ZERO
        && (stats.no_attack || (stats.attack_interval > Fixed64::ZERO && stats.attack_range >= Fixed64::ZERO)))
}

/// The dummy's states for `lifetime` (zero = until it is removed): [`BREAKABLE_STATES`] when it
/// has life, else [`STATES`].
pub fn hold_states(buffs: &mut BuffStore, entity: Entity, lifetime: Fixed64, breakable: bool) {
    let life = if lifetime > Fixed64::ZERO { lifetime } else { super::ability_runtime::buff_store::PERMANENT_BUFF_DURATION };
    let states: &[&str] = if breakable { &BREAKABLE_STATES } else { &STATES };
    for id in states { buffs.add(entity, id, life, serde_json::Value::Null); }
}

/// Template stats a summon holds as one permanent stat row (`hp_regen`, night sight).
pub const BASE_ROW: &str = "__summon_base";

/// The template's regen and night sight as [`BASE_ROW`]; nothing when both are zero.
pub fn hold_base_stats(buffs: &mut BuffStore, entity: Entity, stats: &omoba_template_ids::SummonStats) {
    use omb_script_abi::stat_keys::StatKey;
    let mut row = serde_json::Map::new();
    if stats.hp_regen != Fixed64::ZERO { row.insert(StatKey::HealthRegenConstant.as_str().into(), stats.hp_regen.raw().into()); }
    if stats.night_vision_bonus != Fixed64::ZERO { row.insert(StatKey::BonusNightVision.as_str().into(), stats.night_vision_bonus.raw().into()); }
    if !row.is_empty() {
        buffs.add(entity, BASE_ROW, super::ability_runtime::buff_store::PERMANENT_BUFF_DURATION, serde_json::Value::Object(row));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// E546: a body that never attacks (C16 鬼影) and a melee range 0 (A21 鐵扇) spawn.
    #[test]
    fn a_body_without_an_attack_or_with_melee_range_zero_spawns() {
        let stats = |id: &str| omoba_template_ids::summon_by_name(id).and_then(omoba_template_ids::active_summon_stats);
        let ghost = stats("c16e_ghost").expect("c16e_ghost row");
        assert!(ghost.no_attack && body_valid(ghost));
        let mut melee = *ghost; melee.no_attack = false;
        melee.attack_interval = Fixed64::ONE; melee.attack_range = Fixed64::ZERO;
        assert!(body_valid(&melee), "melee range 0");
        melee.attack_interval = Fixed64::ZERO;
        assert!(!body_valid(&melee), "an attack needs an interval");
    }
    use specs::{Builder, World, WorldExt};

    #[test]
    fn a_dummy_with_life_can_be_hit_and_template_regen_and_night_sight_ride_one_row() {
        let mut world = World::new();
        let (holder, spike) = (world.create_entity().build(), world.create_entity().build());
        let mut buffs = BuffStore::new();
        hold_states(&mut buffs, holder, Fixed64::from_i32(3), false);
        hold_states(&mut buffs, spike, Fixed64::from_i32(3), true);
        assert!(STATES.iter().all(|id| buffs.has(holder, id)));
        assert!(!buffs.has(spike, state::INVULNERABLE) && !buffs.has(spike, state::UNSELECTABLE), "a dummy with life can be hit");
        assert!(BREAKABLE_STATES.iter().all(|id| buffs.has(spike, id)));
        let mut stats = *omoba_template_ids::summon_by_name("dummy").and_then(omoba_template_ids::active_summon_stats).unwrap();
        hold_base_stats(&mut buffs, holder, &stats);
        assert!(!buffs.has(holder, BASE_ROW), "nothing to hold");
        stats.hp_regen = Fixed64::from_i32(4);
        stats.night_vision_bonus = Fixed64::from_i32(-200);
        hold_base_stats(&mut buffs, holder, &stats);
        let row = &buffs.get(holder, BASE_ROW).unwrap().payload;
        assert_eq!(row["health_regen_constant"], 4 * 1024);
        assert_eq!(row["bonus_night_vision"], -200 * 1024);
    }
}
