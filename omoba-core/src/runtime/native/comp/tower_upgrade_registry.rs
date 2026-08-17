//! Server-side 84 個 tower upgrade 配表，存為 ECS resource。
//! 在 state/core.rs 初始化時 insert。
//!
//! 數值來源：`scripts/lua_data/templates.lua` 的 `towers[].upgrades`，由
//! `omoba-template-ids/build.rs` 編譯期讀取生 `TOWER_<NAME>_UPGRADES` const +
//! `tower_upgrades(id)` lookup。本檔案只負責把 const POD 轉成 runtime
//! `TowerUpgradeDef`（含 String / Vec）並塞入 HashMap 供查詢。

use crate::tower_meta::{StatOp, TowerActiveAbilityDef, TowerUpgradeDef, UpgradeEffect};
use omoba_template_ids::{
    active_tower_upgrades, ActiveAbilityConst, StatOpC, UpgradeEffectConst, UpgradeEffectKindC,
    TOWER_ARTY, TOWER_BOMB, TOWER_BOOMERANG, TOWER_CAKE_SPLASH, TOWER_DART, TOWER_ICE, TOWER_TACK,
};
use std::collections::HashMap;

pub struct TowerUpgradeRegistry {
    /// key = (塔類型、路徑、等級)
    defs: HashMap<(String, u8, u8), TowerUpgradeDef>,
}

impl TowerUpgradeRegistry {
    pub fn new() -> Self {
        Self::new_with_cost_multiplier(1.0)
    }

    pub fn new_with_cost_multiplier(cost_multiplier: f32) -> Self {
        let mut defs = HashMap::new();
        for &tid in &[
            TOWER_DART,
            TOWER_TACK,
            TOWER_BOMB,
            TOWER_ICE,
            TOWER_ARTY,
            TOWER_CAKE_SPLASH,
            TOWER_BOOMERANG,
        ] {
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
                        cost: scaled_cost(c.cost, cost_multiplier),
                        effects: c.effects.iter().map(upgrade_effect_from_const).collect(),
                        active_ability: c.active_ability.map(active_ability_from_const),
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

fn scaled_cost(base_cost: i32, multiplier: f32) -> i32 {
    ((base_cost as f32) * multiplier).round().max(1.0) as i32
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

fn active_ability_from_const(c: ActiveAbilityConst) -> TowerActiveAbilityDef {
    TowerActiveAbilityDef {
        ability_id: c.ability_id.into(),
        display_name: c.display_name.into(),
        description: c.description.into(),
        icon: c.icon.into(),
        cooldown: c.cooldown,
        duration: c.duration,
        pulse_interval: c.pulse_interval,
        pulse_count: c.pulse_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tower_meta::{upgrade_cost, StatOp, UpgradeEffect};
    use omb_script_abi::stat_keys::{Aggregation, StatKey, ALL as ALL_STAT_KEYS};
    use omoba_sim::Fixed64;
    use omoba_template_ids::{
        TOWER_ARTY, TOWER_ARTY_STATS, TOWER_BOMB_STATS, TOWER_BOOMERANG, TOWER_BOOMERANG_STATS,
        TOWER_CAKE_SPLASH, TOWER_CAKE_SPLASH_STATS, TOWER_DART_STATS, TOWER_ICE_STATS,
        TOWER_TACK_STATS,
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
    fn all_seven_towers_have_12_upgrades_each() {
        let reg = TowerUpgradeRegistry::new();
        for kind in &[
            TOWER_DART.as_str(),
            TOWER_BOMB.as_str(),
            TOWER_TACK.as_str(),
            TOWER_ICE.as_str(),
            TOWER_ARTY.as_str(),
            TOWER_CAKE_SPLASH.as_str(),
            TOWER_BOOMERANG.as_str(),
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
    fn cost_multiplier_scales_all_upgrade_costs() {
        let reg = TowerUpgradeRegistry::new_with_cost_multiplier(0.7);
        let def = reg.get(TOWER_DART.as_str(), 0, 1).expect("dart p1 l1");

        assert_eq!(def.cost, 35);
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
        assert_eq!(reg.defs.len(), 84);
    }

    #[test]
    fn all_upgrade_metadata_passes_strict_lint() {
        let reg = TowerUpgradeRegistry::new();
        validate_all_upgrade_metadata(&reg);
    }

    #[test]
    fn all_nine_active_upgrades_match_authored_routes() {
        let reg = TowerUpgradeRegistry::new();
        let expected = BTreeSet::from([
            (
                TOWER_DART.as_str(),
                2,
                4,
                "dart_heavy_burst",
                Fixed64::from_i32(12).raw(),
            ),
            (
                TOWER_BOMB.as_str(),
                2,
                4,
                "bomb_cluster_overload",
                Fixed64::from_i32(12).raw(),
            ),
            (
                TOWER_ICE.as_str(),
                2,
                4,
                "ice_crystal_nova",
                Fixed64::from_i32(12).raw(),
            ),
            (
                TOWER_TACK.as_str(),
                2,
                4,
                "tack_blade_maelstrom",
                Fixed64::from_i32(12).raw(),
            ),
            (
                TOWER_ARTY.as_str(),
                2,
                4,
                "arty_fire_at_will",
                Fixed64::from_i32(10).raw(),
            ),
            (
                TOWER_CAKE_SPLASH.as_str(),
                1,
                4,
                "cake_dessert_party",
                Fixed64::from_i32(10).raw(),
            ),
            (
                TOWER_CAKE_SPLASH.as_str(),
                2,
                4,
                "cake_frosting_lockdown",
                Fixed64::from_i32(12).raw(),
            ),
            (
                TOWER_BOOMERANG.as_str(),
                1,
                4,
                "boomerang_turbo_charge",
                Fixed64::from_i32(10).raw(),
            ),
            (
                TOWER_BOOMERANG.as_str(),
                2,
                4,
                "boomerang_shuriken_storm",
                Fixed64::from_i32(12).raw(),
            ),
        ]);
        let actual = reg
            .iter_all()
            .filter_map(|def| {
                def.active_ability.as_ref().map(|active| {
                    (
                        def.tower_kind.as_str(),
                        def.path,
                        def.level,
                        active.ability_id.as_str(),
                        active.cooldown.raw(),
                    )
                })
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn arty_attack_speed_path_stops_adding_permanent_speed_at_level_four() {
        let reg = TowerUpgradeRegistry::new();
        let expected = [1.25_f32, 1.5, 2.0];
        let mut cumulative = 1.0_f32;

        for (level, expected_total) in (1..=3).zip(expected) {
            let def = reg
                .get(TOWER_ARTY.as_str(), 2, level)
                .expect("Arty attack-speed upgrade");
            let multiplier = def
                .effects
                .iter()
                .find_map(|effect| match effect {
                    UpgradeEffect::StatMod { key, value, op } if key == "AttackSpeedMultiplier" => {
                        assert_eq!(*op, StatOp::Mul);
                        Some(*value)
                    }
                    _ => None,
                })
                .expect("Arty attack-speed level has multiplier");
            cumulative *= multiplier;

            assert!(
                (cumulative - expected_total).abs() < 0.003,
                "Arty path 2 level {level}: expected cumulative {expected_total}, got {cumulative}"
            );
        }

        let active_upgrade = reg
            .get(TOWER_ARTY.as_str(), 2, 4)
            .expect("Arty active upgrade");
        assert!(active_upgrade.active_ability.is_some());
        assert!(!active_upgrade.effects.iter().any(|effect| matches!(
            effect,
            UpgradeEffect::StatMod { key, .. } if key == "AttackSpeedMultiplier"
        )));
    }

    #[test]
    fn arty_heavy_ordnance_path_reaches_authored_final_stats_at_every_level() {
        let reg = TowerUpgradeRegistry::new();
        let expected = [
            (50.0_f32, 200.0_f32, 700.0_f32),
            (75.0, 250.0, 700.0),
            (100.0, 300.0, 700.0),
            (150.0, 400.0, 800.0),
        ];
        let mut damage_bonus = 0.0_f32;
        let mut splash_bonus = 0.0_f32;
        let mut range_bonus = 0.0_f32;

        for (level, (expected_attack, expected_splash, expected_range)) in (1..=4).zip(expected) {
            let def = reg
                .get(TOWER_ARTY.as_str(), 0, level)
                .expect("Arty heavy-ordnance upgrade");
            for effect in &def.effects {
                if let UpgradeEffect::StatMod { key, value, op } = effect {
                    assert_eq!(*op, StatOp::Add);
                    match key.as_str() {
                        "BaseDamageOutgoingPercentage" => damage_bonus += value,
                        "SplashBonus" => splash_bonus += value,
                        "AttackRangeBonus" => range_bonus += value,
                        _ => {}
                    }
                }
            }

            let final_attack = 50.0 * (1.0 + damage_bonus);
            let final_splash = 200.0 + splash_bonus;
            let final_range = 600.0 + range_bonus;
            assert_eq!(
                (final_attack, final_splash, final_range),
                (expected_attack, expected_splash, expected_range),
                "Arty path 1 level {level} final stats"
            );
        }
    }

    #[test]
    fn arty_level_four_control_metadata_authors_a_finite_five_second_slow() {
        let reg = TowerUpgradeRegistry::new();
        let def = reg
            .get(TOWER_ARTY.as_str(), 1, 4)
            .expect("Arty level-four control upgrade");

        assert_eq!(def.name, "深度凍結");
        assert_eq!(def.description, "暈眩時間→3 秒，附加 5 秒 50% 減速");
    }

    fn validate_all_upgrade_metadata(reg: &TowerUpgradeRegistry) {
        validate_registry_shape(reg);
        validate_text_and_costs(reg);
        validate_stat_effects(reg);
        validate_behavior_flags(reg);
        validate_active_abilities(reg);
    }

    fn expected_towers() -> [(&'static str, i32); 7] {
        [
            (TOWER_DART.as_str(), TOWER_DART_STATS.cost),
            (TOWER_TACK.as_str(), TOWER_TACK_STATS.cost),
            (TOWER_BOMB.as_str(), TOWER_BOMB_STATS.cost),
            (TOWER_ICE.as_str(), TOWER_ICE_STATS.cost),
            (TOWER_ARTY.as_str(), TOWER_ARTY_STATS.cost),
            (TOWER_CAKE_SPLASH.as_str(), TOWER_CAKE_SPLASH_STATS.cost),
            (TOWER_BOOMERANG.as_str(), TOWER_BOOMERANG_STATS.cost),
        ]
    }

    fn validate_registry_shape(reg: &TowerUpgradeRegistry) {
        assert_eq!(
            reg.defs.len(),
            84,
            "tower upgrade registry must contain exactly 7 towers * 3 paths * 4 levels"
        );

        let expected_tower_ids: BTreeSet<&str> = expected_towers()
            .into_iter()
            .map(|(kind, _)| kind)
            .collect();
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

                    assert!(
                        !def.name.trim().is_empty(),
                        "{label}: name must not be empty"
                    );
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
                        !def.effects.is_empty() || def.active_ability.is_some(),
                        "{label}: upgrade must contain an effect or active ability"
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

                assert!(
                    !key.trim().is_empty(),
                    "{label}: stat key must not be empty"
                );
                assert!(
                    value.is_finite(),
                    "{label}: stat key {key} value must be finite"
                );
                assert_ne!(
                    *value, 0.0,
                    "{label}: stat key {key} value must not be zero"
                );
                if key == "AttackSpeedMultiplier" {
                    assert_eq!(
                        *op,
                        StatOp::Mul,
                        "{label}: AttackSpeedMultiplier must use StatOp::Mul"
                    );
                    assert!(
                        *value > 1.0,
                        "{label}: AttackSpeedMultiplier must be greater than 1.0, got {value}"
                    );
                }

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

    fn validate_behavior_flags(reg: &TowerUpgradeRegistry) {
        for def in reg.iter_all() {
            let label = upgrade_label(&def.tower_kind, def.path, def.level);
            for effect in &def.effects {
                let UpgradeEffect::BehaviorFlag { flag } = effect else {
                    continue;
                };

                assert!(
                    !flag.trim().is_empty(),
                    "{label}: behavior flag must not be empty"
                );
                assert!(
                    supported_behavior_flags(def.tower_kind.as_str()).contains(&flag.as_str()),
                    "{label}: unsupported behavior flag `{flag}` for {}",
                    def.tower_kind
                );
            }
        }
    }

    fn validate_active_abilities(reg: &TowerUpgradeRegistry) {
        let scoped_towers = [
            TOWER_DART.as_str(),
            TOWER_BOMB.as_str(),
            TOWER_ICE.as_str(),
            TOWER_TACK.as_str(),
            TOWER_ARTY.as_str(),
            TOWER_CAKE_SPLASH.as_str(),
            TOWER_BOOMERANG.as_str(),
        ];
        let mut active_counts = HashMap::<&str, usize>::new();
        let mut ability_ids = BTreeSet::new();

        for def in reg.iter_all() {
            let Some(ability) = def.active_ability.as_ref() else {
                continue;
            };
            let label = upgrade_label(&def.tower_kind, def.path, def.level);

            assert_eq!(
                def.level, 4,
                "{label}: active ability must be declared at level 4"
            );
            assert!(
                scoped_towers.contains(&def.tower_kind.as_str()),
                "{label}: active ability is not scoped to this tower kind"
            );
            assert!(
                !ability.ability_id.trim().is_empty(),
                "{label}: active ability id must not be empty"
            );
            assert!(
                ability_ids.insert(ability.ability_id.as_str()),
                "{label}: duplicate active ability id `{}`",
                ability.ability_id
            );
            assert!(
                ability.cooldown > Fixed64::ZERO,
                "{label}: active ability cooldown must be positive"
            );
            assert!(
                ability.duration >= Fixed64::ZERO,
                "{label}: active ability duration must not be negative"
            );
            let pulses_absent = ability.pulse_interval == Fixed64::ZERO && ability.pulse_count == 0;
            let pulses_present = ability.pulse_interval > Fixed64::ZERO && ability.pulse_count > 0;
            assert!(
                pulses_absent || pulses_present,
                "{label}: pulse interval and count must both be positive or both be zero"
            );
            if pulses_present {
                assert!(
                    ability.duration
                        >= ability.pulse_interval * Fixed64::from_i32(ability.pulse_count as i32),
                    "{label}: active duration must cover every authored pulse"
                );
            }

            *active_counts.entry(def.tower_kind.as_str()).or_default() += 1;
        }

        assert_eq!(
            ability_ids.len(),
            9,
            "expected nine unique tower active abilities"
        );
        for tower_kind in scoped_towers {
            assert!(
                active_counts
                    .get(tower_kind)
                    .is_some_and(|count| *count >= 1),
                "{tower_kind}: expected at least one active ability"
            );
        }
        assert_eq!(active_counts.get(TOWER_CAKE_SPLASH.as_str()), Some(&2));
        assert_eq!(active_counts.get(TOWER_BOOMERANG.as_str()), Some(&2));
    }

    fn supported_behavior_flags(tower_kind: &str) -> &'static [&'static str] {
        match tower_kind {
            "tower_dart" => &[
                "camo_detection",
                "sharp_pierce",
                "spike_o_pult",
                "triple_shot",
                "fan_club",
                "always_crit",
                "mega_crit",
            ],
            "tower_bomb" => &[
                "bomb_stun",
                "missile",
                "missile_speed_tier2",
                "moab_assassin",
                "frag_8",
                "frag_12",
                "frag_recursive",
                "frag_homing",
            ],
            "tower_tack" => &[
                "blade_shooter",
                "burn_tier1",
                "burn_tier2",
                "ring_of_fire",
                "inferno_ring",
                "needles_12",
                "needles_16",
                "needles_32",
            ],
            "tower_ice" => &[
                "deep_freeze",
                "absolute_zero",
                "arctic_aura_20",
                "snowstorm",
                "cryo_cannon",
                "embrittle_15",
                "refreeze",
                "embrittle_25",
                "icicle_impale",
            ],
            "tower_arty" => &["arty_stun", "arty_stun_2", "arty_stun_3", "arty_slow_50"],
            "tower_cake_splash" => &[
                "cake_burn_20",
                "cake_burn_40",
                "cake_secondary_pulse_1",
                "cake_secondary_pulse_2",
                "cake_dessert_party",
                "cake_frost_20",
                "cake_frost_35",
                "cake_frost_vulnerability_15",
                "cake_frost_50_vulnerability_25",
            ],
            "tower_boomerang" => &[
                "glaive_ricochet",
                "glaive_lord",
                "moab_press",
                "faster_rangs",
                "bionic_burst",
                "turbo_charge",
                "shuriken",
                "double_shuriken",
                "storm_shuriken",
            ],
            _ => &[],
        }
    }

    fn upgrade_label(kind: &str, path: u8, level: u8) -> String {
        format!("{kind} path {path} level {level}")
    }
}
