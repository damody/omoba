//! Server-side 48 個 tower upgrade 配表，存為 ECS resource。
//! 在 state/core.rs 初始化時 insert。
//!
//! 數值來源：`scripts/lua_data/templates.lua` 的 `towers[].upgrades`，由
//! `omoba-template-ids/build.rs` 編譯期讀取生 `TOWER_<NAME>_UPGRADES` const +
//! `tower_upgrades(id)` lookup。本檔案只負責把 const POD 轉成 runtime
//! `TowerUpgradeDef`（含 String / Vec）並塞入 HashMap 供查詢。

use crate::tower_meta::{StatOp, TowerUpgradeDef, UpgradeEffect};
use omoba_template_ids::{
    active_tower_upgrades, StatOpC, UpgradeEffectConst, UpgradeEffectKindC, TOWER_BOMB, TOWER_DART,
    TOWER_ICE, TOWER_TACK,
};
use std::collections::HashMap;

pub struct TowerUpgradeRegistry {
    /// key = (塔類型、路徑、等級)
    defs: HashMap<(String, u8, u8), TowerUpgradeDef>,
}

impl TowerUpgradeRegistry {
    pub fn new() -> Self {
        let mut defs = HashMap::new();
        for &tid in &[TOWER_DART, TOWER_TACK, TOWER_BOMB, TOWER_ICE] {
            let kind = tid.as_str();
            let Some(paths) = active_tower_upgrades(tid) else {
                continue;
            };
            for (path_idx, path) in paths.iter().enumerate() {
                for (lvl_idx, c) in path.iter().enumerate() {
                    let lvl = (lvl_idx + 1) as u8;
                    let def = TowerUpgradeDef {
                        tower_kind: kind.into(),
                        path: path_idx as u8,
                        level: lvl,
                        name: c.name.into(),
                        description: c.description.into(),
                        cost: c.cost,
                        effects: c.effects.iter().map(upgrade_effect_from_const).collect(),
                    };
                    let prev = defs.insert((kind.into(), path_idx as u8, lvl), def);
                    debug_assert!(
                        prev.is_none(),
                        "duplicate upgrade def for {} path {} level {}",
                        kind,
                        path_idx,
                        lvl
                    );
                }
            }
        }
        Self { defs }
    }

    pub fn get(&self, kind: &str, path: u8, level: u8) -> Option<&TowerUpgradeDef> {
        self.defs.get(&(kind.to_string(), path, level))
    }

    pub fn iter_all(&self) -> impl Iterator<Item = &TowerUpgradeDef> {
        self.defs.values()
    }
}

fn upgrade_effect_from_const(c: &UpgradeEffectConst) -> UpgradeEffect {
    match c.kind {
        UpgradeEffectKindC::StatMod => UpgradeEffect::StatMod {
            key: c.key.into(),
            value: c.value.to_f32_for_render(),
            op: match c.op {
                StatOpC::Add => StatOp::Add,
                StatOpC::Mul => StatOp::Mul,
            },
        },
        UpgradeEffectKindC::BehaviorFlag => UpgradeEffect::BehaviorFlag { flag: c.key.into() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tower_meta::{upgrade_cost, StatOp, UpgradeEffect};
    use omb_script_abi::stat_keys::{Aggregation, StatKey, ALL as ALL_STAT_KEYS};
    use omoba_template_ids::{
        TOWER_BOMB_STATS, TOWER_DART_STATS, TOWER_ICE_STATS, TOWER_TACK_STATS,
    };
    use std::collections::BTreeSet;

    #[test]
    fn dart_has_12_upgrades() {
        let reg = TowerUpgradeRegistry::new();
        for path in 0..3 {
            for level in 1..=4 {
                assert!(
                    reg.get(TOWER_DART.as_str(), path, level).is_some(),
                    "dart path {} level {}",
                    path,
                    level
                );
            }
        }
    }

    #[test]
    fn all_four_towers_have_12_upgrades_each() {
        let reg = TowerUpgradeRegistry::new();
        for kind in &[
            TOWER_DART.as_str(),
            TOWER_BOMB.as_str(),
            TOWER_TACK.as_str(),
            TOWER_ICE.as_str(),
        ] {
            for path in 0..3 {
                for level in 1..=4 {
                    assert!(
                        reg.get(kind, path, level).is_some(),
                        "{} path {} level {}",
                        kind,
                        path,
                        level
                    );
                }
            }
        }
    }

    #[test]
    fn costs_match_formula() {
        use crate::tower_meta::upgrade_cost;
        use omoba_template_ids::{
            TOWER_BOMB_STATS, TOWER_DART_STATS, TOWER_ICE_STATS, TOWER_TACK_STATS,
        };
        let reg = TowerUpgradeRegistry::new();
        let bases = [
            (TOWER_DART.as_str(), TOWER_DART_STATS.cost),
            (TOWER_BOMB.as_str(), TOWER_BOMB_STATS.cost),
            (TOWER_TACK.as_str(), TOWER_TACK_STATS.cost),
            (TOWER_ICE.as_str(), TOWER_ICE_STATS.cost),
        ];
        for (kind, base) in bases {
            for path in 0..3u8 {
                for level in 1..=4 {
                    let def = reg.get(kind, path, level).unwrap();
                    assert_eq!(
                        def.cost,
                        upgrade_cost(base, level),
                        "{} path {} L{}",
                        kind,
                        path,
                        level
                    );
                }
            }
        }
    }

    #[test]
    fn no_duplicate_keys() {
        let reg = TowerUpgradeRegistry::new();
        assert_eq!(reg.defs.len(), 48);
    }

    #[test]
    fn all_upgrade_metadata_passes_strict_lint() {
        let reg = TowerUpgradeRegistry::new();
        validate_all_upgrade_metadata(&reg);
    }

    fn validate_all_upgrade_metadata(reg: &TowerUpgradeRegistry) {
        validate_registry_shape(reg);
        validate_text_and_costs(reg);
        validate_stat_effects(reg);
    }

    fn expected_towers() -> [(&'static str, i32); 4] {
        [
            (TOWER_DART.as_str(), TOWER_DART_STATS.cost),
            (TOWER_TACK.as_str(), TOWER_TACK_STATS.cost),
            (TOWER_BOMB.as_str(), TOWER_BOMB_STATS.cost),
            (TOWER_ICE.as_str(), TOWER_ICE_STATS.cost),
        ]
    }

    fn validate_registry_shape(reg: &TowerUpgradeRegistry) {
        assert_eq!(
            reg.defs.len(),
            48,
            "tower upgrade registry must contain exactly 4 towers * 3 paths * 4 levels"
        );

        let expected_tower_ids: BTreeSet<&str> =
            expected_towers().into_iter().map(|(kind, _)| kind).collect();
        let actual_tower_ids: BTreeSet<&str> =
            reg.iter_all().map(|def| def.tower_kind.as_str()).collect();
        assert_eq!(
            actual_tower_ids, expected_tower_ids,
            "tower upgrade registry contains unexpected tower ids"
        );

        for (kind, _) in expected_towers() {
            for path in 0..=2u8 {
                for level in 1..=4u8 {
                    let def = reg.get(kind, path, level).unwrap_or_else(|| {
                        panic!("missing upgrade def for {kind} path {path} level {level}")
                    });
                    assert_eq!(
                        def.tower_kind, kind,
                        "{kind} path {path} level {level}: tower_kind mismatch"
                    );
                    assert_eq!(
                        def.path, path,
                        "{kind} path {path} level {level}: path mismatch"
                    );
                    assert_eq!(
                        def.level, level,
                        "{kind} path {path} level {level}: level mismatch"
                    );
                }
            }
        }

        for def in reg.iter_all() {
            assert!(
                expected_tower_ids.contains(def.tower_kind.as_str()),
                "{} path {} level {}: unexpected tower_kind",
                def.tower_kind,
                def.path,
                def.level
            );
            assert!(
                def.path <= 2,
                "{} path {} level {}: path must be 0..=2",
                def.tower_kind,
                def.path,
                def.level
            );
            assert!(
                (1..=4).contains(&def.level),
                "{} path {} level {}: level must be 1..=4",
                def.tower_kind,
                def.path,
                def.level
            );
        }
    }

    fn validate_text_and_costs(reg: &TowerUpgradeRegistry) {
        for (kind, base_cost) in expected_towers() {
            for path in 0..=2u8 {
                for level in 1..=4u8 {
                    let def = reg
                        .get(kind, path, level)
                        .expect("shape validation guarantees every upgrade exists");
                    let label = upgrade_label(kind, path, level);

                    assert!(!def.name.trim().is_empty(), "{label}: name must not be empty");
                    assert!(
                        !def.description.trim().is_empty(),
                        "{label}: description must not be empty"
                    );
                    assert_ne!(
                        def.name.trim(),
                        def.description.trim(),
                        "{label}: description must not be identical to name"
                    );
                    assert!(def.cost > 0, "{label}: cost must be positive");
                    assert_eq!(
                        def.cost,
                        upgrade_cost(base_cost, level),
                        "{label}: cost must match upgrade_cost(base_cost, level)"
                    );
                    assert!(
                        !def.effects.is_empty(),
                        "{label}: upgrade must contain at least one effect"
                    );
                }
            }
        }
    }

    fn validate_stat_effects(reg: &TowerUpgradeRegistry) {
        for def in reg.iter_all() {
            let label = upgrade_label(&def.tower_kind, def.path, def.level);
            for effect in &def.effects {
                let UpgradeEffect::StatMod { key, value, op } = effect else {
                    continue;
                };

                assert!(!key.trim().is_empty(), "{label}: stat key must not be empty");
                assert!(value.is_finite(), "{label}: stat key {key} value must be finite");
                assert_ne!(*value, 0.0, "{label}: stat key {key} value must not be zero");

                validate_stat_op_matches_key(&label, key, *op);
            }
        }
    }

    fn validate_stat_op_matches_key(label: &str, key: &str, op: StatOp) {
        if key.ends_with("_bonus") || is_absolute_stat_key(key) {
            assert_eq!(
                op,
                StatOp::Add,
                "{label}: suffix-style stat key `{key}` must use StatOp::Add"
            );
            return;
        }
        if key.ends_with("_multiplier") {
            assert_eq!(
                op,
                StatOp::Mul,
                "{label}: suffix-style stat key `{key}` must use StatOp::Mul"
            );
            return;
        }

        let Some(stat_key) = stat_key_by_metadata_key(key) else {
            panic!(
                "{label}: stat key `{key}` must use suffix style, be explicitly allowlisted, or match a StatKey variant/as_str value"
            );
        };
        match (op, stat_key.aggregation()) {
            (
                StatOp::Add,
                Aggregation::SumAdd | Aggregation::SumAddThenMul1Plus | Aggregation::Chance,
            ) => {}
            (StatOp::Add, Aggregation::PassThrough) if is_pass_through_upgrade_stat(key) => {}
            (StatOp::Mul, Aggregation::ProductMult) => {}
            (StatOp::Add, aggregation) => {
                panic!(
                    "{label}: StatOp::Add key `{key}` maps to {:?} with unsupported {:?} aggregation",
                    stat_key, aggregation
                );
            }
            (StatOp::Mul, aggregation) => {
                panic!(
                    "{label}: StatOp::Mul key `{key}` maps to {:?} with unsupported {:?} aggregation",
                    stat_key, aggregation
                );
            }
        }
    }

    fn stat_key_by_metadata_key(key: &str) -> Option<StatKey> {
        ALL_STAT_KEYS
            .iter()
            .copied()
            .find(|stat_key| format!("{:?}", stat_key) == key || stat_key.as_str() == key)
    }

    fn is_absolute_stat_key(key: &str) -> bool {
        matches!(
            key,
            "crit_chance" | "crit_bonus" // Dart crit path uses absolute values in suffix-style content.
        )
    }

    fn is_pass_through_upgrade_stat(key: &str) -> bool {
        matches!(
            key,
            "PreattackCriticalStrike" // Dart crit path exposes an absolute critical-strike proc value.
                | "SlowFactorOverride" // Ice path writes an override factor through tower upgrade metadata.
        )
    }

    fn upgrade_label(kind: &str, path: u8, level: u8) -> String {
        format!("{kind} path {path} level {level}")
    }
}
