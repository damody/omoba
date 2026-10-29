//! 回復（v3.1 §12.10）。
//!
//! - `hp_regen_pct`／`mana_regen_pct`（`StatKey::HealthRegenMaxPercentage`／`ManaRegenMaxPercentage`）：
//!   每秒回復最大值的比例，與固定回復相加後一起吃倍率與增幅。
//! - `hp_regen_amp_pct`（`HpRegenAmplifyPercentage`，原本就有）／`mana_regen_amp_pct`
//!   （`ManaRegenAmplifyPercentage`）：回復量的增幅。
//! - 本遊戲沿用的 `HealthRegenPercentage`、`ManaRegenPercentage` 是「回復倍率」，不是每秒最大值比例，
//!   維持原意（測試以 −1 關掉回復）。
//!
//! 沒有這些新鍵時，結果與原本的 `UnitStats::hp_regen`／`mana_regen` 完全相同（同樣的運算順序）。
use omb_script_abi::stat_keys::StatKey;
use omoba_sim::Fixed64;
use specs::Entity;

use crate::runtime::ability_runtime::UnitStats;

/// 生命每秒回復：`UnitStats::hp_regen` 的固定項加上最大生命 × 每秒比例。
pub fn hp_per_second(stats: &UnitStats<'_>, base: Fixed64, max_hp: Fixed64, e: Entity) -> Fixed64 {
    let per_max = stats.buffs.sum_add(e, StatKey::HealthRegenMaxPercentage);
    let base = if per_max == Fixed64::ZERO { base } else { base + max_hp * per_max };
    stats.hp_regen(base, e)
}

/// 法力每秒回復：`UnitStats::mana_regen` 加上最大法力 × 每秒比例，再乘法力回復增幅。
pub fn mana_per_second(stats: &UnitStats<'_>, base: Fixed64, max_mana: Fixed64, e: Entity) -> Fixed64 {
    let per_max = stats.buffs.sum_add(e, StatKey::ManaRegenMaxPercentage);
    let natural = if per_max == Fixed64::ZERO { stats.mana_regen(base, e) } else {
        stats.checked_mana_regen_with_flat_bonus(base, e, max_mana * per_max).unwrap_or(Fixed64::ZERO)
    };
    let amp = stats.buffs.sum_add(e, StatKey::ManaRegenAmplifyPercentage);
    if amp == Fixed64::ZERO { natural } else { (natural * (Fixed64::ONE + amp)).max(Fixed64::ZERO) }
}

/// 生命回復要不要每次檢查這個單位（regen_tick 的候選鍵）。
pub const HP_REGEN_KEYS: [StatKey; 4] = [StatKey::HealthRegenConstant, StatKey::HealthRegenPercentage,
    StatKey::HpRegenAmplifyPercentage, StatKey::HealthRegenMaxPercentage];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::BuffStore;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn max_share_regen_adds_to_the_flat_regen_and_amp_scales_mana() {
        let mut world = World::new();
        let e = world.create_entity().build();
        let mut buffs = BuffStore::new();
        let i = Fixed64::from_i32;
        let stats = UnitStats::from_refs(&buffs, false);
        assert_eq!(hp_per_second(&stats, i(2), i(1000), e), stats.hp_regen(i(2), e), "no new keys: unchanged");
        assert_eq!(mana_per_second(&stats, i(3), i(500), e), stats.mana_regen(i(3), e));
        // 1/1024 of max HP and 20/1024 of max mana per second; mana amplified by half.
        buffs.add(e, "spring", i(5), serde_json::json!({"health_regen_max_percentage": 10, "mana_regen_max_percentage": 20,
            "mana_regen_amplify_percentage": 512}));
        let stats = UnitStats::from_refs(&buffs, false);
        assert_eq!(hp_per_second(&stats, i(2), i(1000), e), stats.hp_regen(i(2) + i(1000) * Fixed64::from_raw(10), e));
        let natural = stats.checked_mana_regen_with_flat_bonus(i(3), e, i(500) * Fixed64::from_raw(20)).unwrap();
        assert_eq!(mana_per_second(&stats, i(3), i(500), e), natural * Fixed64::from_raw(1536));
        assert!(natural > i(12) && natural < i(13));
    }
}
