//! 英雄魔抗池（原作 `vscripts/library/common/dummy.lua:690-700`、`:1036-1069`）。
//!
//! - 原作每位英雄有一個 `magical_resistance` 值，預設 30。`Add_magical_resistance` 的道具與技能
//!   （例如 A07T 戰國最強、魔抗類道具）、直接改這個值的技能（A34T 減抗、新藤五國光）都加在它上面。
//! - 被動 `magical_resistance` 每 0.2 秒把 `min(值, 70)` 寫成英雄的**基礎**魔抗。
//! - KV 的 `MODIFIER_PROPERTY_MAGICAL_RESISTANCE_BONUS` 由引擎再以 1 − Π(1 − rᵢ) 乘法合成，
//!   **不受 70 的限制**。
//!
//! 這裡：池 = 英雄的基礎魔抗（`CProperty::def_magic`，來自 `game_rules.lua` 的
//! `hero.base_magic_resistance`）＋ Σ`StatKey::MagicResistancePool`，上限是
//! `hero.max_magic_resistance`；之後照舊與其他魔抗合成（`UnitStats::final_magic_resist`）。
//! 沒有比賽規則的世界（測試、TD）不設上限。只有英雄有魔抗池；其他單位的魔抗不變。
use omb_script_abi::stat_keys::StatKey;
use omoba_sim::Fixed64;
use specs::Entity;

use crate::runtime::BuffStore;

/// 英雄魔抗池（百分點）。`base_percent` 是英雄的 `CProperty::def_magic`。
pub fn hero_pool(buffs: &BuffStore, hero: Entity, base_percent: Fixed64,
    rules: Option<&omoba_template_ids::GameRulesConst>) -> Fixed64 {
    let pool = base_percent + buffs.sum_add(hero, StatKey::MagicResistancePool);
    match rules {
        Some(rules) => pool.min(Fixed64::from_i32(rules.hero_max_magic_resistance.min(100) as i32)),
        None => pool,
    }
}

/// 英雄的最終魔抗（百分點），給呈現用的屬性面板。結算用 `damage::defenses` 的 checked 版本。
pub fn hero_percent(stats: &crate::runtime::ability_runtime::UnitStats<'_>, hero: Entity, base_percent: Fixed64,
    rules: Option<&omoba_template_ids::GameRulesConst>) -> Fixed64 {
    let hundred = Fixed64::from_i32(100);
    stats.final_magic_resist(hero_pool(stats.buffs, hero, base_percent, rules) / hundred, hero) * hundred
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, World, WorldExt};

    fn rules() -> omoba_template_ids::GameRulesConst {
        omoba_template_ids::GAME_RULES.expect("game_rules.lua carries hero magic resistance")
    }

    #[test]
    fn the_pool_caps_at_the_rule_and_other_resistance_multiplies_past_it() {
        let rules = rules();
        let (base, max) = (rules.hero_base_magic_resistance as i32, rules.hero_max_magic_resistance as i32);
        let mut world = World::new();
        let hero = world.create_entity().build();
        let mut buffs = BuffStore::new();
        let i = Fixed64::from_i32;
        assert_eq!(hero_pool(&buffs, hero, i(base), Some(&rules)), i(base));
        // A07T 戰國最強 +50 → min(30 + 50, 70).
        buffs.add(hero, "a07t", i(10), serde_json::json!({"magic_resistance_pool": 50 * 1024}));
        assert_eq!(hero_pool(&buffs, hero, i(base), Some(&rules)), i(max));
        assert_eq!(hero_pool(&buffs, hero, i(base), None), i(base + 50), "no cap without match rules");
        // A KV bonus multiplies on top of the capped pool: 1 − (1 − 0.7)(1 − 0.5) = 85%.
        buffs.add(hero, "kv_bonus", i(10), serde_json::json!({"magical_resistance_bonus": 50 * 1024}));
        let stats = crate::runtime::ability_runtime::UnitStats::from_refs(&buffs, false);
        let percent = hero_percent(&stats, hero, i(base), Some(&rules));
        assert!((percent.to_f32_for_render() - (100.0 - (100 - max) as f32 * 0.5)).abs() < 0.2, "{percent:?}");
        // A34T 減抗 lowers the pool below the base.
        buffs.add(hero, "a34t", i(10), serde_json::json!({"magic_resistance_pool": -60 * 1024}));
        assert_eq!(hero_pool(&buffs, hero, i(base), Some(&rules)), i(base - 10));
    }
}
