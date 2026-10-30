//! 原作 dummy／majia 單位（`summons.lua` 的 `dummy = true`，所有英雄與道具共用）：只用來掛狀態與
//! 光環的身體。沒有生命、攻擊、碰撞與指令；存活時間到就移除（summon_tick）。spawn 時
//! （game_processor `commit_summoned_spawn`）不建立 TAttack、碰撞半徑為 0，並在整個存活時間掛上
//! [`STATES`]：不受傷害、不會被普攻或單位技能選到、範圍選取也選不到（無敵）、不擋其他單位、
//! 擁有者不能下指令、不顯示血條。呈現只播 cue，不顯示模型（前端讀 `SummonStats::dummy`）。
use omoba_sim::Fixed64;
use specs::Entity;
use super::rules::vocab::state;
use crate::runtime::BuffStore;

/// The states a dummy holds for its whole life.
pub const STATES: [&str; 6] = [state::INVULNERABLE, state::UNSELECTABLE, state::ATTACK_IMMUNE, state::COMMAND_RESTRICTED,
    state::NO_HEALTHBAR, super::ability_runtime::NO_UNIT_COLLISION];

/// A summon template a spawn admits: a dummy (its zero body is checked by the catalog), or a
/// body with life and an attack.
pub fn body_valid(stats: &omoba_template_ids::SummonStats) -> bool {
    stats.dummy || (stats.hp > Fixed64::ZERO && stats.attack_interval > Fixed64::ZERO && stats.attack_range > Fixed64::ZERO)
}

/// The dummy's states for `lifetime` (zero = until it is removed).
pub fn hold_states(buffs: &mut BuffStore, entity: Entity, lifetime: Fixed64) {
    let life = if lifetime > Fixed64::ZERO { lifetime } else { super::ability_runtime::buff_store::PERMANENT_BUFF_DURATION };
    for id in STATES { buffs.add(entity, id, life, serde_json::Value::Null); }
}
