//! Tower upgrade metadata — client/server 共用 schema。
//!
//! 四塔 × 3 路線 × 4 級 = 48 個 TowerUpgradeDef。
//! runtime registry 見 `runtime::comp::tower_upgrade_registry`。

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TowerUpgradeDef {
    pub tower_kind: String, // "tower_dart" / "tower_bomb" / "tower_tack" / "tower_ice"
    pub path: u8,           // 0, 1, 2
    pub level: u8,          // 1..=4
    pub name: String,
    pub description: String,
    pub cost: i32,
    pub effects: Vec<UpgradeEffect>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UpgradeEffect {
    StatMod { key: String, value: f32, op: StatOp },
    BehaviorFlag { flag: String },
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatOp {
    Add, // _bonus 後綴，BuffStore sum_add
    Mul, // _multiplier 後綴，BuffStore product_mult
}

/// 費用公式：base × {0.25, 0.5, 1.0, 2.5}[level-1]
pub fn upgrade_cost(base_cost: i32, level: u8) -> i32 {
    let mul = match level {
        1 => 0.25,
        2 => 0.50,
        3 => 1.00,
        4 => 2.50,
        _ => return 0,
    };
    (base_cost as f32 * mul) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_formula() {
        assert_eq!(upgrade_cost(200, 1), 50);
        assert_eq!(upgrade_cost(200, 2), 100);
        assert_eq!(upgrade_cost(200, 3), 200);
        assert_eq!(upgrade_cost(200, 4), 500);
        assert_eq!(upgrade_cost(650, 4), 1625);
    }

    #[test]
    fn serde_roundtrip() {
        use omoba_template_ids::TOWER_DART;
        let def = TowerUpgradeDef {
            tower_kind: TOWER_DART.as_str().into(),
            path: 0,
            level: 1,
            name: "Long Range Darts".into(),
            description: "射程 +50".into(),
            cost: 50,
            effects: vec![UpgradeEffect::StatMod {
                key: "range_bonus".into(),
                value: 50.0,
                op: StatOp::Add,
            }],
        };
        let s = serde_json::to_string(&def).unwrap();
        let _: TowerUpgradeDef = serde_json::from_str(&s).unwrap();
    }
}
