# Tower Upgrade Paths 實作計畫

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** 為 TD 4 塔新增 Bloons 風格 3 條強化路線 × 每路線 4 級系統，含後端升級邏輯、script 行為 flag、右側 UI 面板。

**Architecture:**
- **資料層**：client/server 共用 `TowerUpgradeDef` schema（`omoba-core/tower_meta.rs`）；server 側 48 個升級寫進 `TowerUpgradeRegistry`（ECS resource）。
- **執行層**：`Tower` component 擴 `upgrade_levels:[u8;3]` + `upgrade_flags:Vec<String>`；stat modifier 走既有 `BuffStore`（永久 buff，`u32::MAX` duration）；行為改變走 `upgrade_flags`，script `on_tick`/`on_attack_hit` 讀 flag 分支。
- **網路層**：玩家送 `player/tower/upgrade {tower_id, path}` → 後端驗 2.5 規則 → 扣錢 → 套 effects → 廣播 `tower/upgrade {stats}`；前端 eui 點塔顯示右面板。

**Tech Stack:** Rust (specs ECS, abi_stable FFI, KCP transport), eui (immediate-mode GUI, 由 omfx 使用), serde_json, BuffStore payload aggregation。

**Design doc:** `C:\Users\damod\.claude\plans\3-ui-4-humming-meteor.md`（含 48 個升級完整表）。

---

## Phase 1: Schema + Core Data

### Task 1: 新增 `tower_meta.rs` schema 型別

**Files:**
- Create: `D:/omoba/omoba-core/src/tower_meta.rs`
- Modify: `D:/omoba/omoba-core/src/lib.rs`（export）

**Step 1: 建檔**

內容：
```rust
//! Tower upgrade metadata — client/server 共用 schema。
//!
//! 四塔 × 3 路線 × 4 級 = 48 個 TowerUpgradeDef。
//! server 側 registry 見 omb/src/comp/tower_upgrade_registry.rs。

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TowerUpgradeDef {
    pub tower_kind: String,  // "tower_dart" / "tower_bomb" / "tower_tack" / "tower_ice"
    pub path: u8,            // 0, 1, 2
    pub level: u8,           // 1..=4
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
    Add,  // _bonus 後綴，BuffStore sum_add
    Mul,  // _multiplier 後綴，BuffStore product_mult
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
```

**Step 2: lib.rs export**

在 `omoba-core/src/lib.rs` 中加 `pub mod tower_meta;`。

**Step 3: 單元測試**

在 `tower_meta.rs` 底部加：
```rust
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
        let def = TowerUpgradeDef {
            tower_kind: "tower_dart".into(), path: 0, level: 1,
            name: "Long Range Darts".into(),
            description: "射程 +50".into(),
            cost: 50,
            effects: vec![UpgradeEffect::StatMod {
                key: "range_bonus".into(), value: 50.0, op: StatOp::Add,
            }],
        };
        let s = serde_json::to_string(&def).unwrap();
        let _: TowerUpgradeDef = serde_json::from_str(&s).unwrap();
    }
}
```

**Step 4: 跑測試**

```bash
cd D:/omoba/omoba-core && cargo test tower_meta
```
預期：2 passed。

**Step 5: commit**

```bash
cd D:/omoba && git add omoba-core/src/tower_meta.rs omoba-core/src/lib.rs
git commit -m "feat(omoba-core): tower_meta schema (TowerUpgradeDef/UpgradeEffect/StatOp)"
```

---

### Task 2: 擴 `Tower` component

**Files:**
- Modify: `D:/omoba/omb/src/comp/tower.rs:7-21`

**Step 1: 修改 struct 與 new()**

把 `Tower` 改成：
```rust
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tower {
    pub nearby_creeps: Vec<NearbyEnt>,
    pub block_creeps: Vec<Entity>,
    pub buffs: Vec<TModify>,
    #[serde(default)]
    pub upgrade_levels: [u8; 3],
    #[serde(default)]
    pub upgrade_flags: Vec<String>,
    #[serde(default)]
    pub ultimate_cooldown: f32,
}
impl Tower {
    pub fn new() -> Self {
        Self {
            nearby_creeps: vec![],
            block_creeps: vec![],
            buffs: vec![],
            upgrade_levels: [0; 3],
            upgrade_flags: vec![],
            ultimate_cooldown: 0.0,
        }
    }
}
```

**Step 2: 建置驗證**

```bash
cd D:/omoba/omb && cargo check
```
預期：無新增 error。`#[serde(default)]` 讓舊存檔仍相容。

**Step 3: commit**

```bash
git add omb/src/comp/tower.rs
git commit -m "feat(omb): Tower 擴 upgrade_levels/upgrade_flags/ultimate_cooldown"
```

---

## Phase 2: 2.5 規則 Validator + Registry

### Task 3: 2.5 規則驗證函式（純函式，可單元測試）

**Files:**
- Create: `D:/omoba/omb/src/comp/tower_upgrade_rules.rs`
- Modify: `D:/omoba/omb/src/comp/mod.rs`（加 `pub mod tower_upgrade_rules;`）

**Step 1: 寫純函式**

```rust
//! Bloons 2.5 upgrade rule validator.
//!
//! 對任一 tower 的 `[u8; 3]` path levels，要求升第 i 條路線：
//! - 主路線（level ≥ 3）最多 1 條
//! - 副路線（1 ≤ level ≤ 2）最多 1 條
//! - 第三路線必須 0

#[derive(Debug, PartialEq, Eq)]
pub enum UpgradeRejection {
    AlreadyMaxed,
    TwoPrimaryPaths,
    TwoSecondaryPaths,
    ThirdPathLocked,
}

pub fn validate_upgrade(levels: [u8; 3], path: u8) -> Result<(), UpgradeRejection> {
    if path >= 3 {
        return Err(UpgradeRejection::ThirdPathLocked);
    }
    let i = path as usize;
    if levels[i] >= 4 {
        return Err(UpgradeRejection::AlreadyMaxed);
    }
    let mut next = levels;
    next[i] += 1;

    let primary = next.iter().filter(|&&l| l >= 3).count();
    let secondary = next.iter().filter(|&&l| l >= 1 && l <= 2).count();

    if primary > 1 {
        return Err(UpgradeRejection::TwoPrimaryPaths);
    }
    if primary == 1 && secondary > 1 {
        return Err(UpgradeRejection::TwoSecondaryPaths);
    }
    if primary == 0 && secondary > 2 {
        return Err(UpgradeRejection::TwoSecondaryPaths);
    }
    Ok(())
}
```

**Step 2: 測試**

在同檔底部：
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn empty_any_ok() {
        assert!(validate_upgrade([0,0,0], 0).is_ok());
        assert!(validate_upgrade([0,0,0], 1).is_ok());
        assert!(validate_upgrade([0,0,0], 2).is_ok());
    }

    #[test] fn max_rejected() {
        assert_eq!(validate_upgrade([4,0,0], 0), Err(UpgradeRejection::AlreadyMaxed));
    }

    #[test] fn two_primary_rejected() {
        // Path 0 已 L3（primary），升 Path 1 到 L3
        assert_eq!(validate_upgrade([3,2,0], 1), Err(UpgradeRejection::TwoPrimaryPaths));
    }

    #[test] fn two_secondary_when_primary() {
        // Path 0 L3 primary，Path 1 L1 secondary，要把 Path 2 升 → 會變 2 個 secondary
        assert_eq!(validate_upgrade([3,1,0], 2), Err(UpgradeRejection::TwoSecondaryPaths));
    }

    #[test] fn three_secondary_no_primary() {
        // 無主路線時不能三條都升
        assert_eq!(validate_upgrade([2,1,0], 2), Err(UpgradeRejection::TwoSecondaryPaths));
    }

    #[test] fn path_upgrade_to_primary_ok() {
        // Path 0 L2 → L3（升主），Path 1 L2 副 — 合法
        assert!(validate_upgrade([2,2,0], 0).is_ok());
    }

    #[test] fn full_build_ok() {
        // 主 L4 + 副 L2 能達成
        assert!(validate_upgrade([3,2,0], 0).is_ok());  // 升主 L4
    }
}
```

**Step 3: 跑測試**

```bash
cd D:/omoba/omb && cargo test tower_upgrade_rules
```
預期：7 passed。

**Step 4: commit**

```bash
git add omb/src/comp/tower_upgrade_rules.rs omb/src/comp/mod.rs
git commit -m "feat(omb): 2.5 rule validator + unit tests"
```

---

### Task 4: `TowerUpgradeRegistry` ECS resource + Dart 12 個升級

**Files:**
- Create: `D:/omoba/omb/src/comp/tower_upgrade_registry.rs`
- Modify: `D:/omoba/omb/src/comp/mod.rs`

**Step 1: 建檔（只含 Dart 12 個先驗證結構）**

```rust
//! Server-side 48 個 tower upgrade 配表，存為 ECS resource。
//! 在 state/core.rs 初始化時 insert。

use std::collections::HashMap;
use omoba_core::tower_meta::{TowerUpgradeDef, UpgradeEffect, StatOp};

pub struct TowerUpgradeRegistry {
    /// key = (tower_kind, path, level)
    defs: HashMap<(String, u8, u8), TowerUpgradeDef>,
}

impl TowerUpgradeRegistry {
    pub fn new() -> Self {
        let mut reg = Self { defs: HashMap::new() };
        reg.register_dart();
        reg.register_bomb();
        reg.register_tack();
        reg.register_ice();
        reg
    }

    pub fn get(&self, kind: &str, path: u8, level: u8) -> Option<&TowerUpgradeDef> {
        self.defs.get(&(kind.to_string(), path, level))
    }

    fn insert(&mut self, def: TowerUpgradeDef) {
        self.defs.insert((def.tower_kind.clone(), def.path, def.level), def);
    }

    // Dart Monkey (base 200): Path 0 Sharp, Path 1 Quick, Path 2 Crit
    fn register_dart(&mut self) {
        let kind = "tower_dart";
        // Path 0 — Sharp Shots
        self.insert(TowerUpgradeDef {
            tower_kind: kind.into(), path: 0, level: 1,
            name: "Long Range Darts".into(),
            description: "射程 350→400".into(),
            cost: 50,
            effects: vec![UpgradeEffect::StatMod { key: "range_bonus".into(), value: 50.0, op: StatOp::Add }],
        });
        self.insert(TowerUpgradeDef {
            tower_kind: kind.into(), path: 0, level: 2,
            name: "Enhanced Eyesight".into(),
            description: "射程 →450, damage 10→15".into(),
            cost: 100,
            effects: vec![
                UpgradeEffect::StatMod { key: "range_bonus".into(), value: 50.0, op: StatOp::Add },
                UpgradeEffect::StatMod { key: "damage_bonus".into(), value: 0.5, op: StatOp::Add },
            ],
        });
        self.insert(TowerUpgradeDef {
            tower_kind: kind.into(), path: 0, level: 3,
            name: "Razor Sharp Shots".into(),
            description: "穿透 +1, damage →20".into(),
            cost: 200,
            effects: vec![
                UpgradeEffect::BehaviorFlag { flag: "sharp_pierce".into() },
                UpgradeEffect::StatMod { key: "damage_bonus".into(), value: 0.5, op: StatOp::Add },
            ],
        });
        self.insert(TowerUpgradeDef {
            tower_kind: kind.into(), path: 0, level: 4,
            name: "Spike-o-pult".into(),
            description: "改投巨釘：splash 100, damage 40, 彈速減半".into(),
            cost: 500,
            effects: vec![UpgradeEffect::BehaviorFlag { flag: "spike_o_pult".into() }],
        });
        // Path 1 — Quick Shots
        self.insert(TowerUpgradeDef {
            tower_kind: kind.into(), path: 1, level: 1,
            name: "Quick Shots".into(),
            description: "攻速 +20%".into(),
            cost: 50,
            effects: vec![UpgradeEffect::StatMod { key: "attack_speed_multiplier".into(), value: 0.83, op: StatOp::Mul }],
        });
        self.insert(TowerUpgradeDef {
            tower_kind: kind.into(), path: 1, level: 2,
            name: "Very Quick Shots".into(),
            description: "攻速再 +30%".into(),
            cost: 100,
            effects: vec![UpgradeEffect::StatMod { key: "attack_speed_multiplier".into(), value: 0.70, op: StatOp::Mul }],
        });
        self.insert(TowerUpgradeDef {
            tower_kind: kind.into(), path: 1, level: 3,
            name: "Triple Shot".into(),
            description: "一發變 3 發扇形 ±15°".into(),
            cost: 200,
            effects: vec![UpgradeEffect::BehaviorFlag { flag: "triple_shot".into() }],
        });
        self.insert(TowerUpgradeDef {
            tower_kind: kind.into(), path: 1, level: 4,
            name: "Super Monkey Fan Club".into(),
            description: "5 發扇形 + 彈速×2 + 攻速再 +30%".into(),
            cost: 500,
            effects: vec![
                UpgradeEffect::BehaviorFlag { flag: "fan_club".into() },
                UpgradeEffect::StatMod { key: "attack_speed_multiplier".into(), value: 0.70, op: StatOp::Mul },
            ],
        });
        // Path 2 — Crit Master
        self.insert(TowerUpgradeDef {
            tower_kind: kind.into(), path: 2, level: 1,
            name: "Keen Eyes".into(),
            description: "爆率 25→40%, 爆傷 30→40".into(),
            cost: 50,
            effects: vec![
                UpgradeEffect::StatMod { key: "crit_chance".into(), value: 0.40, op: StatOp::Add },
                UpgradeEffect::StatMod { key: "crit_bonus".into(), value: 40.0, op: StatOp::Add },
            ],
        });
        self.insert(TowerUpgradeDef {
            tower_kind: kind.into(), path: 2, level: 2,
            name: "Crossbow".into(),
            description: "爆率 →50%, 爆傷 →60, 射程 +30".into(),
            cost: 100,
            effects: vec![
                UpgradeEffect::StatMod { key: "crit_chance".into(), value: 0.10, op: StatOp::Add },
                UpgradeEffect::StatMod { key: "crit_bonus".into(), value: 20.0, op: StatOp::Add },
                UpgradeEffect::StatMod { key: "range_bonus".into(), value: 30.0, op: StatOp::Add },
            ],
        });
        self.insert(TowerUpgradeDef {
            tower_kind: kind.into(), path: 2, level: 3,
            name: "Sharp Shooter".into(),
            description: "必爆 (100%), base dmg +30%".into(),
            cost: 200,
            effects: vec![
                UpgradeEffect::BehaviorFlag { flag: "always_crit".into() },
                UpgradeEffect::StatMod { key: "damage_bonus".into(), value: 0.3, op: StatOp::Add },
            ],
        });
        self.insert(TowerUpgradeDef {
            tower_kind: kind.into(), path: 2, level: 4,
            name: "Ultra-Juggernaut".into(),
            description: "爆擊 100 dmg + splash 60".into(),
            cost: 500,
            effects: vec![UpgradeEffect::BehaviorFlag { flag: "mega_crit".into() }],
        });
    }

    // Task 5 會補：register_bomb / register_tack / register_ice
    fn register_bomb(&mut self) { /* TODO Task 5 */ }
    fn register_tack(&mut self) { /* TODO Task 5 */ }
    fn register_ice(&mut self)  { /* TODO Task 5 */ }
}
```

**Step 2: mod.rs export**

```rust
pub mod tower_upgrade_registry;
pub mod tower_upgrade_rules;
```

**Step 3: 單元測試**

檔底部加：
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dart_has_12_upgrades() {
        let reg = TowerUpgradeRegistry::new();
        for path in 0..3 {
            for level in 1..=4 {
                assert!(reg.get("tower_dart", path, level).is_some(),
                    "dart path {} level {}", path, level);
            }
        }
    }
}
```

**Step 4: 跑測試**

```bash
cd D:/omoba/omb && cargo test tower_upgrade_registry::tests
```
預期：dart_has_12_upgrades passed。

**Step 5: commit**

```bash
git add omb/src/comp/tower_upgrade_registry.rs omb/src/comp/mod.rs
git commit -m "feat(omb): TowerUpgradeRegistry + Dart 12 個升級"
```

---

### Task 5: 補齊 Bomb / Tack / Ice 36 個升級

**Files:**
- Modify: `D:/omoba/omb/src/comp/tower_upgrade_registry.rs`

**Step 1: 把 `register_bomb / register_tack / register_ice` 完整填好**

內容對照設計文件 `C:\Users\damod\.claude\plans\3-ui-4-humming-meteor.md` 的「48 個升級內容」章節。所有 `StatMod` `key` 命名慣例：
- `_bonus` 後綴 → `StatOp::Add`（加法）
- `_multiplier` 後綴 → `StatOp::Mul`（乘法）

`BehaviorFlag` 對照表（實作在 Task 12-15 script 讀取）：
| Flag | 位置 |
|--|--|
| `sharp_pierce`, `spike_o_pult`, `triple_shot`, `fan_club`, `always_crit`, `mega_crit` | Dart |
| `bomb_stun`, `missile`, `moab_assassin`, `frag_8`, `frag_12`, `frag_recursive`, `frag_homing` | Bomb |
| `blade_shooter`, `burn_tier1`, `burn_tier2`, `ring_of_fire`, `inferno_ring`, `needles_12`, `needles_16`, `needles_32` | Tack |
| `deep_freeze`, `absolute_zero`, `arctic_aura_20`, `snowstorm`, `cryo_cannon`, `embrittle_15`, `refreeze`, `embrittle_25`, `icicle_impale` | Ice |

**Step 2: 擴充測試**

```rust
#[test]
fn all_four_towers_have_12_upgrades_each() {
    let reg = TowerUpgradeRegistry::new();
    for kind in &["tower_dart", "tower_bomb", "tower_tack", "tower_ice"] {
        for path in 0..3 {
            for level in 1..=4 {
                assert!(reg.get(kind, path, level).is_some(),
                    "{} path {} level {}", kind, path, level);
            }
        }
    }
}

#[test]
fn costs_match_formula() {
    use omoba_core::tower_meta::upgrade_cost;
    let reg = TowerUpgradeRegistry::new();
    let bases = [("tower_dart", 200), ("tower_bomb", 650), ("tower_tack", 400), ("tower_ice", 400)];
    for (kind, base) in bases {
        for level in 1..=4 {
            let def = reg.get(kind, 0, level).unwrap();
            assert_eq!(def.cost, upgrade_cost(base, level), "{} path 0 L{}", kind, level);
        }
    }
}
```

**Step 3: 跑測試**

```bash
cd D:/omoba/omb && cargo test tower_upgrade_registry::tests
```
預期：3 passed（加總 48 個 def 齊全）。

**Step 4: commit**

```bash
git add omb/src/comp/tower_upgrade_registry.rs
git commit -m "feat(omb): 補齊 Bomb/Tack/Ice 36 個升級定義（共 48）"
```

---

### Task 6: 在 state/core.rs 註冊 TowerUpgradeRegistry 為 resource

**Files:**
- Modify: `D:/omoba/omb/src/state/core.rs`（找 `populate_ability_registry` 或 TowerTemplateRegistry 初始化處附近）

**Step 1: 加 insert**

在初始化 world resources 的位置加：
```rust
world.insert(crate::comp::tower_upgrade_registry::TowerUpgradeRegistry::new());
```

**Step 2: 建置**

```bash
cd D:/omoba/omb && cargo check
```
預期：無 error。

**Step 3: commit**

```bash
git add omb/src/state/core.rs
git commit -m "feat(omb): 註冊 TowerUpgradeRegistry 為 ECS resource"
```

---

## Phase 3: FFI 擴充

### Task 7: GameWorld trait 擴 3 個 method

**Files:**
- Modify: `D:/omoba/omb/script-abi/src/world.rs`（sabi_trait）
- Modify: `D:/omoba/omb/src/scripting/world_adapter.rs`（實作）

**Step 1: script-abi/world.rs 加 trait method**

在 `GameWorld` trait 內（`get_final_atk` 附近）加：
```rust
/// 取得該塔第 `path` 路線已升級的等級（0..=4）。
fn get_tower_upgrade(&self, e: EntityHandle, path: u8) -> u8;

/// 該塔是否掛了指定的 behavior flag（e.g. "triple_shot"）。
fn has_tower_flag(&self, e: EntityHandle, flag: RStr<'_>) -> bool;

/// 對 tower entity 套一個永久 stat buff（供 upgrade 使用）。
/// `modifiers_json` 應為 `{"key": value}` 形式（與 add_stat_buff 同）。
fn apply_tower_permanent_buff(&self, e: EntityHandle, buff_id: RStr<'_>, modifiers_json: RStr<'_>);
```

**Step 2: world_adapter.rs 實作**

（以現有 `get_tower_atk` 為模板）：
```rust
fn get_tower_upgrade(&self, e: EntityHandle, path: u8) -> u8 {
    let ent = self.resolve(e);
    let towers = self.world.read_storage::<crate::comp::Tower>();
    towers.get(ent).and_then(|t| t.upgrade_levels.get(path as usize)).copied().unwrap_or(0)
}

fn has_tower_flag(&self, e: EntityHandle, flag: RStr<'_>) -> bool {
    let ent = self.resolve(e);
    let towers = self.world.read_storage::<crate::comp::Tower>();
    towers.get(ent)
        .map(|t| t.upgrade_flags.iter().any(|f| f == flag.as_str()))
        .unwrap_or(false)
}

fn apply_tower_permanent_buff(&self, e: EntityHandle, buff_id: RStr<'_>, modifiers_json: RStr<'_>) {
    // 直接轉發到 add_stat_buff，duration 用 f32::MAX
    self.add_stat_buff(e, buff_id, f32::MAX, modifiers_json);
}
```

**Step 3: 建置 (stable ABI 雙端都要 rebuild)**

```bash
cd D:/omoba && cargo build -p omb-script-abi
cd D:/omoba/omb && cargo check
cd D:/omoba/scripts && cargo check
```
預期：無 error。

**Step 4: commit**

```bash
cd D:/omoba && git add omb/script-abi/src/world.rs omb/src/scripting/world_adapter.rs
git commit -m "feat(script-abi): get_tower_upgrade/has_tower_flag/apply_tower_permanent_buff"
```

---

## Phase 4: upgrade_tower() 實作

### Task 8: 後端 upgrade_tower() 主流程

**Files:**
- Modify: `D:/omoba/omb/src/state/resource_management.rs:406-409`（stub 位置）

**Step 1: 完整實作**

取代現有 stub：
```rust
fn upgrade_tower(&self, world: &mut World, pd: &InboundMsg) -> Result<(), Error> {
    use serde_json::json;
    use specs::{Join, WorldExt};
    use crate::comp::tower_upgrade_rules::{validate_upgrade, UpgradeRejection};
    use omoba_core::tower_meta::{UpgradeEffect, StatOp};

    let is_td = world.read_resource::<GameMode>().is_td();
    if !is_td {
        log::warn!("upgrade_tower 指令在非 TD 模式下被忽略");
        return Ok(());
    }

    let tower_id_u32 = pd.d.get("tower_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let path = pd.d.get("path").and_then(|v| v.as_u64()).unwrap_or(255) as u8;
    if path >= 3 {
        log::warn!("upgrade_tower: invalid path {}", path);
        return Ok(());
    }

    // 找 tower entity + 讀當前 levels + kind
    let (tower_entity, levels, kind_str) = {
        let entities = world.entities();
        let towers = world.read_storage::<Tower>();
        let tags = world.read_storage::<crate::scripting::ScriptUnitTag>();
        let mut found = None;
        for (e, t, tag) in (&entities, &towers, &tags).join() {
            if e.id() == tower_id_u32 {
                found = Some((e, t.upgrade_levels, tag.unit_id.to_string()));
                break;
            }
        }
        let Some(x) = found else {
            log::warn!("upgrade_tower: 找不到塔 id={}", tower_id_u32);
            return Ok(());
        };
        x
    };

    // 2.5 規則
    if let Err(rej) = validate_upgrade(levels, path) {
        log::info!("upgrade_tower: 規則拒絕 {:?}", rej);
        let _ = self.mqtx.send(OutboundMsg::new_s("td/all/res", "tower", "upgrade_reject", json!({
            "tower_id": tower_id_u32, "path": path, "reason": format!("{:?}", rej),
        })));
        return Ok(());
    }

    let next_level = levels[path as usize] + 1;

    // 查 TowerUpgradeDef
    let (cost, effects, up_name) = {
        let reg = world.read_resource::<crate::comp::tower_upgrade_registry::TowerUpgradeRegistry>();
        let Some(def) = reg.get(&kind_str, path, next_level) else {
            log::warn!("upgrade_tower: 無對應定義 {} path {} L{}", kind_str, path, next_level);
            return Ok(());
        };
        (def.cost, def.effects.clone(), def.name.clone())
    };

    // 找英雄 + 扣錢
    let hero_entity = {
        let entities = world.entities();
        let heroes = world.read_storage::<Hero>();
        let factions = world.read_storage::<Faction>();
        (&entities, &heroes, &factions).join()
            .find(|(_, _, f)| f.faction_id == FactionType::Player)
            .map(|(e, _, _)| e)
    };
    let Some(hero_entity) = hero_entity else {
        log::warn!("upgrade_tower: 找不到玩家英雄");
        return Ok(());
    };
    {
        let golds = world.read_storage::<Gold>();
        if golds.get(hero_entity).map(|g| g.0).unwrap_or(0) < cost {
            log::info!("upgrade_tower: 金錢不足（需 {}）", cost);
            let _ = self.mqtx.send(OutboundMsg::new_s("td/all/res", "tower", "upgrade_reject", json!({
                "tower_id": tower_id_u32, "path": path, "reason": "not_enough_gold",
            })));
            return Ok(());
        }
    }
    {
        let mut golds = world.write_storage::<Gold>();
        if let Some(g) = golds.get_mut(hero_entity) {
            g.0 -= cost;
        }
    }

    // 套 effects
    for (idx, eff) in effects.iter().enumerate() {
        match eff {
            UpgradeEffect::BehaviorFlag { flag } => {
                let mut towers = world.write_storage::<Tower>();
                if let Some(t) = towers.get_mut(tower_entity) {
                    if !t.upgrade_flags.contains(flag) {
                        t.upgrade_flags.push(flag.clone());
                    }
                }
            }
            UpgradeEffect::StatMod { key, value, op: _ } => {
                // 以 BuffStore 永久 buff 掛載；_bonus 用 sum_add 讀、_multiplier 用 product_mult 讀
                let buff_id = format!("upgrade_{}_{}_{}", path, next_level, idx);
                let payload = json!({ key: value });
                let mut buff_store = world.write_resource::<crate::ability_runtime::BuffStore>();
                buff_store.add(tower_entity, &buff_id, f32::MAX, payload);
            }
        }
    }

    // 更新 upgrade_levels
    {
        let mut towers = world.write_storage::<Tower>();
        if let Some(t) = towers.get_mut(tower_entity) {
            t.upgrade_levels[path as usize] = next_level;
        }
    }

    // 廣播
    let pos = world.read_storage::<Pos>().get(tower_entity).map(|p| p.0).unwrap_or(vek::Vec2::zero());
    let payload = json!({
        "tower_id": tower_id_u32,
        "path": path,
        "level": next_level,
        "name": up_name,
        "levels": {
            "0": {
                let t = world.read_storage::<Tower>();
                t.get(tower_entity).map(|t| t.upgrade_levels[0]).unwrap_or(0)
            },
            "1": { let t = world.read_storage::<Tower>(); t.get(tower_entity).map(|t| t.upgrade_levels[1]).unwrap_or(0) },
            "2": { let t = world.read_storage::<Tower>(); t.get(tower_entity).map(|t| t.upgrade_levels[2]).unwrap_or(0) },
        },
    });
    let _ = self.mqtx.send(OutboundMsg::new_s_at(
        "td/all/res", "tower", "upgrade", payload, pos.x, pos.y,
    ));

    self.push_hero_stats(world, hero_entity);
    log::info!("🔧 塔 id={} path={} L{} 升級成功 ({})", tower_id_u32, path, next_level, up_name);
    Ok(())
}
```

**Step 2: 建置**

```bash
cd D:/omoba/omb && cargo check
```

**Step 3: commit**

```bash
git add omb/src/state/resource_management.rs
git commit -m "feat(omb): upgrade_tower 實作（2.5 規則 + BuffStore effects + 廣播）"
```

---

### Task 9: sell_tower 退還升級費

**Files:**
- Modify: `D:/omoba/omb/src/state/resource_management.rs::sell_tower`（約 line 411）

**Step 1: 在 refund 計算處加升級退款**

找到 `let refund = { ... }` 區塊，在 85% base 之上加上升級費退款：
```rust
let refund = {
    let tags = world.read_storage::<crate::scripting::ScriptUnitTag>();
    let reg = world.read_resource::<crate::comp::tower_registry::TowerTemplateRegistry>();
    let towers = world.read_storage::<Tower>();
    let ureg = world.read_resource::<crate::comp::tower_upgrade_registry::TowerUpgradeRegistry>();
    let base_refund = tags.get(target_entity)
        .and_then(|t| reg.get(&t.unit_id))
        .map(|tpl| (tpl.cost as f32 * 0.85) as i32)
        .unwrap_or(0);
    let upgrade_refund = if let (Some(t), Some(tag)) = (towers.get(target_entity), tags.get(target_entity)) {
        let mut total = 0i32;
        for path in 0..3u8 {
            for level in 1..=t.upgrade_levels[path as usize] {
                if let Some(def) = ureg.get(&tag.unit_id, path, level) {
                    total += (def.cost as f32 * 0.75) as i32;
                }
            }
        }
        total
    } else { 0 };
    base_refund + upgrade_refund
};
```

**Step 2: 建置**

```bash
cd D:/omoba/omb && cargo check
```

**Step 3: commit**

```bash
git add omb/src/state/resource_management.rs
git commit -m "feat(omb): sell_tower 退還 75% 升級費"
```

---

## Phase 5: Script 改寫 — 塔行為 + flag

### Task 10: Dart script 讀 flag 分支

**Files:**
- Modify: `D:/omoba/scripts/base_content/src/towers/dart.rs`

**Step 1: 改 `on_tick` 讀 fan_club/triple_shot/spike_o_pult；改 `on_attack_hit` 讀 crit_chance/always_crit/mega_crit**

完整新版：
```rust
use omb_script_abi::prelude::*;

pub struct DartTower;

const ATK: f32 = 10.0;
const ASD_INTERVAL: f32 = 0.8;
const RANGE: f32 = 350.0;
const BULLET_SPEED: f32 = 1200.0;

impl UnitScript for DartTower {
    fn unit_id(&self) -> RStr<'_> { RStr::from_str("tower_dart") }

    fn on_spawn(&self, e: EntityHandle, w: &mut GameWorldDyn<'_>) {
        w.set_tower_atk(e, ATK);
        w.set_tower_range(e, RANGE);
        w.set_asd_interval(e, ASD_INTERVAL);
    }

    fn tower_metadata(&self) -> ROption<TowerMetadata> {
        RSome(TowerMetadata {
            atk: ATK, asd_interval: ASD_INTERVAL, range: RANGE,
            bullet_speed: BULLET_SPEED,
            splash_radius: 0.0, hit_radius: 0.0,
            slow_factor: 0.0, slow_duration: 0.0,
            cost: 200, footprint: 40.0, hp: 1.0,
            turn_speed_deg: 360.0,
            label: RString::from("Dart Monkey"),
        })
    }

    fn on_tick(&self, e: EntityHandle, dt: f32, w: &mut GameWorldDyn<'_>) {
        let asd_interval = w.get_asd_interval(e);
        if asd_interval <= 0.0 { return; }
        let mut asd_count = w.get_asd_count(e);
        if asd_count < asd_interval {
            asd_count += dt;
            w.set_asd_count(e, asd_count);
        }
        if asd_count < asd_interval { return; }

        let pos = match w.get_pos(e) { RSome(p) => p, RNone => return };
        let range = w.get_final_attack_range(e);  // 含 upgrade buff
        let target = match w.query_nearest_enemy(pos, range, e) {
            RSome(t) => t, RNone => return,
        };
        w.set_asd_count(e, asd_count - asd_interval);

        let atk = w.get_final_atk(e);

        // 判斷要發幾發
        let (count, spread_deg) = if w.has_tower_flag(e, RStr::from_str("fan_club")) {
            (5, 30.0)
        } else if w.has_tower_flag(e, RStr::from_str("triple_shot")) {
            (3, 15.0)
        } else {
            (1, 0.0)
        };

        // Spike-o-pult：改成 splash 釘球
        let spike = w.has_tower_flag(e, RStr::from_str("spike_o_pult"));
        let (bullet_speed, damage, splash) = if spike {
            (BULLET_SPEED * 0.5, 40.0, 100.0)
        } else {
            (BULLET_SPEED * if w.has_tower_flag(e, RStr::from_str("fan_club")) {2.0} else {1.0}, atk, 0.0)
        };

        w.log_info(RStr::from_str("[tower_dart] fire!"));

        // 計算目標方向
        let t_pos = match w.get_pos(target) { RSome(p) => p, RNone => return };
        let dx = t_pos.x - pos.x;
        let dy = t_pos.y - pos.y;
        let base_angle = dy.atan2(dx);

        for i in 0..count {
            let angle = if count == 1 {
                base_angle
            } else {
                let offset = spread_deg * core::f32::consts::PI / 180.0;
                let step = (2.0 * offset) / (count as f32 - 1.0);
                base_angle - offset + step * (i as f32)
            };

            let path_spec = if spike || count > 1 {
                // Fan club / triple shot / spike — 用直線彈道保留扇形
                let end = Vec2f::new(pos.x + angle.cos() * range * 1.5, pos.y + angle.sin() * range * 1.5);
                PathSpec::Straight { end_pos: end }
            } else {
                PathSpec::Homing { target }
            };

            w.spawn_projectile_ex(ProjectileSpec {
                from: pos,
                owner: e,
                path: path_spec,
                speed: bullet_speed,
                damage,
                hit_radius: 0.0,
                splash_radius: splash,
                slow_factor: 0.0,
                slow_duration: 0.0,
                stun_duration: 0.0,
                kind_tag: RString::from(if spike {"spike_opult"} else {"dart"}),
            });
        }
    }

    fn on_attack_hit(
        &self,
        attacker: EntityHandle,
        victim: EntityHandle,
        w: &mut GameWorldDyn<'_>,
    ) {
        // Crit chance: always_crit flag 或擲骰 < crit_chance stat
        let always = w.has_tower_flag(attacker, RStr::from_str("always_crit"));
        let chance = w.get_stat_bonus(attacker, RStr::from_str("crit_chance"));
        let roll = w.rand_f32();
        if !always && roll >= chance { return; }

        let bonus = w.get_stat_bonus(attacker, RStr::from_str("crit_bonus"));
        let bonus = if bonus > 0.0 { bonus } else { 30.0 }; // 無升級時保留原 30 bonus

        w.log_info(RStr::from_str("[tower_dart] crit!"));
        w.deal_damage(victim, bonus, DamageKind::Physical, RSome(attacker));
        if let RSome(at) = w.get_pos(victim) {
            w.play_vfx(RStr::from_str("vfx_dart_crit"), at);
        }

        // Mega crit: 附加 60 splash
        if w.has_tower_flag(attacker, RStr::from_str("mega_crit")) {
            if let RSome(at) = w.get_pos(victim) {
                w.play_vfx(RStr::from_str("vfx_explosion"), at);
                // 查 splash 半徑內 creep 扣血（host 端 deal_damage_splash 若無則省略）
                w.deal_damage_splash(at, 60.0, 60.0, DamageKind::Physical, RSome(attacker));
            }
        }
    }
}
```

**Step 2: 驗證 FFI 是否存在**

若 `get_stat_bonus` / `get_final_attack_range` / `deal_damage_splash` 不存在於 `GameWorld` trait，需補；否則用現有 `get_tower_range + BuffStore.sum_add` 替代。查：

```bash
cd D:/omoba && grep -n "fn get_stat_bonus\|fn get_final_attack_range\|fn deal_damage_splash" omb/script-abi/src/world.rs
```

若缺，在本 task 一併補進 `world.rs` + `world_adapter.rs`（各 3 行轉發到 BuffStore.sum_add / 半徑搜尋）。

**Step 3: 建置**

```bash
cd D:/omoba/scripts && cargo build -p base_content --release
```

**Step 4: 打包 DLL**

base_content.dll 自動 stage 到 `omb/scripts/base_content.dll`（參考 CLAUDE.md gen-docs 章節）。

**Step 5: commit**

```bash
cd D:/omoba && git add omb/script-abi/src/world.rs omb/src/scripting/world_adapter.rs scripts/base_content/src/towers/dart.rs
git commit -m "feat(base_content): Dart 3 路線 12 flag 完整實作"
```

---

### Task 11: Bomb script 讀 flag 分支

**Files:**
- Modify: `D:/omoba/scripts/base_content/src/towers/bomb.rs`

實作重點：
- `missile`: 彈速 +50%（runtime 讀 flag 決定）
- `moab_assassin`: 用 `w.get_ultimate_cooldown(e) / set_ultimate_cooldown(e, v)` FFI（若無要補）實現 15s 冷卻的超級彈
- `frag_8 / frag_12 / frag_recursive / frag_homing`: 在 `on_attack_hit` 以 `spawn_projectile_ex(Straight)` 生 8/12/16 方向碎片，方向均分 TAU
- `bomb_stun`: `stun_duration: 0.5`

BehaviorFlag 優先順序：frag_recursive > frag_12 > frag_8（取最高等級），frag_homing 決定 path 是 Homing 還是 Straight。

**Step 1-5 同 Task 10 pattern**。

**commit**:
```bash
git commit -m "feat(base_content): Bomb 3 路線 12 flag 完整實作"
```

---

### Task 12: Tack script 讀 flag 分支

**Files:**
- Modify: `D:/omoba/scripts/base_content/src/towers/tack.rs`

實作重點：
- `needles_N`: `NEEDLE_COUNT` 從 const 改 runtime：
  ```rust
  let count = if w.has_tower_flag(e, RStr::from_str("needles_32")) { 32 }
              else if w.has_tower_flag(e, RStr::from_str("needles_16")) { 16 }
              else if w.has_tower_flag(e, RStr::from_str("needles_12")) { 12 }
              else { 8 };
  ```
- `blade_shooter`: hit_radius 80→110, damage base 8→20（直接寫進 ProjectileSpec）, 穿透 +2（FFI 補 `puncture: u32` 若需要）
- `burn_tier1/tier2`: 命中後 `add_stat_buff("burn", duration, {"dot_damage": 5/10})`
- `ring_of_fire / inferno_ring`: 每次開火時額外 `deal_damage_splash(塔位, 200/半徑, 20/50 damage)`

同 Task 10 pattern 改寫 + commit。

---

### Task 13: Ice script 讀 flag 分支

**Files:**
- Modify: `D:/omoba/scripts/base_content/src/towers/ice.rs`

實作重點：
- `slow_factor_override` stat 由 Permafrost L1 寫入 → script 讀 `get_stat_bonus("slow_factor_override")`，若 >0 則替換 `SLOW_FACTOR`
- `deep_freeze`: `stun_duration: 1.0`
- `arctic_aura_20 / snowstorm`: 每 tick 對射程內所有 creep `add_stat_buff("move_speed_bonus", -0.2/-0.35, 0.5s)` 持續刷新
- `absolute_zero / cryo_cannon`: 用 `ultimate_cooldown` 計時器，每 15s / 10s 觸發
- `embrittle_15 / embrittle_25`: 命中 creep 時對 target `add_stat_buff("damage_taken_bonus", 0.15/0.25, slow_duration)`
- `refreeze`: 命中時若 target 已有 slow 立刻刷新 slow buff duration（用 `remove_buff` + `add_stat_buff`）
- `icicle_impale`: 改 projectile damage + splash 150，path 改 Straight 穿透

同 Task 10 pattern + commit。

---

## Phase 6: 額外基礎建設

### Task 14: handle_damage 聚合 damage_taken_bonus

**Files:**
- Modify: `D:/omoba/omb/src/comp/game_processor.rs`（找 `handle_damage` 或 `process_damage`）

**Step 1:** 在最終傷害計算前加 BuffStore query：
```rust
let damage_taken_bonus = {
    let bs = world.read_resource::<crate::ability_runtime::BuffStore>();
    bs.sum_add(target_entity, "damage_taken_bonus")
};
let final_damage = final_damage * (1.0 + damage_taken_bonus);
```

**Step 2-4:** 建置 + commit。

---

### Task 15: DoT tick 系統（burn / ring_of_fire 用）

**Files:**
- Create: `D:/omoba/omb/src/tick/dot_tick.rs` 或擴 `buff_tick.rs`
- Modify: `D:/omoba/omb/src/tick/mod.rs`

**Step 1:** 新增 system：每 1s 對所有掛了 `dot_damage > 0` payload 的 entity 扣 HP。
```rust
pub fn dot_tick(world: &mut World) {
    // accumulate 1s，每秒統一 damage
    // 遍歷 BuffStore：對每個 entity 算 sum_add("dot_damage")，>0 則 outcomes.push(Outcome::Damage{...})
}
```

**Step 2-4:** 建置 + 註冊進 main tick loop + commit。

---

## Phase 7: 前端 UI

### Task 16: omfx 接 tower/upgrade 廣播 + selected_tower_entity 狀態

**Files:**
- Modify: `D:/omoba/omfx/game/src/lib.rs`（找 tower handler 區塊）
- Modify: `D:/omoba/omfx/game/src/state/` 下的 TowerView struct（可能叫 `td_towers` 之類）

**Step 1:** TowerView 加 `upgrade_levels: [u8; 3]`、`upgrade_flags: Vec<String>`、`current_stats: TowerStats`。

**Step 2:** `lib.rs` 收到 `tower/upgrade` msg 時更新本地 TowerView。

**Step 3:** 加 `selected_tower_entity: Option<u32>` 狀態到 Game struct。

**Step 4:** 滑鼠點擊處理：若點到塔位置（距離 < collision_radius）→ 設為 selected；點背景/空地 → clear。

**Step 5:** commit。

---

### Task 17: eui 右側塔升級面板

**Files:**
- Create: `D:/omoba/omfx/game/src/ui/tower_panel.rs`
- Modify: `D:/omoba/omfx/game/src/ui/mod.rs`
- Modify: `D:/omoba/omfx/game/src/lib.rs`（render loop 加 panel.draw）

**Step 1:** 面板結構（偽碼，實際用 eui API）：
```rust
pub fn draw_tower_panel(ui: &mut Ui, game: &mut Game, tower: &TowerView) {
    ui.panel("tower_panel", 320.0)
      .label(&tower.name)
      .separator()
      .label(&format!("DMG {:.0}  RNG {:.0}  ASD {:.2}s", tower.atk, tower.range, tower.asd));
    for path in 0..3 {
        let path_name = PATH_NAMES[&tower.kind][path as usize];
        ui.label(&format!("Path {}: {}", path+1, path_name));
        // 4 格 ■■□□
        let level = tower.upgrade_levels[path as usize];
        ui.label(&"■".repeat(level as usize) + &"□".repeat((4 - level) as usize));
        // 升級按鈕
        let next = game.lookup_upgrade(&tower.kind, path, level + 1);
        let can = game.can_upgrade(tower, path);
        if ui.button(&format!("升級 ${}", next.cost)).enabled(can).clicked() {
            game.send_upgrade(tower.id, path);
        }
    }
    if ui.button("賣塔").clicked() {
        game.send_sell(tower.id);
    }
}
```

**Step 2:** 客戶端複製 2.5 規則驗證邏輯（或 call `omoba_core::tower_meta` helper）→ `can_upgrade` 判斷。

**Step 3:** 送出 upgrade command：
```rust
fn send_upgrade(&self, tower_id: u32, path: u8) {
    let payload = json!({"tower_id": tower_id, "path": path});
    self.send_player_msg("tower", "upgrade", payload);
}
```

**Step 4:** 建置 + 視覺驗證。

**Step 5:** commit。

---

## Phase 8: 端到端驗收

### Task 18: 跑 run.bat 手動驗收

**Step 1: 啟動**
```bash
cd D:/omoba && ./run.bat
```

**Step 2: 依序驗收**（照設計文件 E2E 流程）：
1. 建 1 座 Dart 塔 → 扣 200 gold
2. 攢 50 gold → 點塔 → 右面板出現 → 按 Path 2 L1 → 扣 50，`■□□□`，攻速提升
3. 升 Path 2 到 L3 → Triple Shot，看到 1 tick 3 發 projectile
4. 嘗試升 Path 1 L3 → reject `TwoPrimaryPaths`
5. 嘗試升 Path 3 L1 → reject `TwoSecondaryPaths`
6. 賣塔 → 返還 200 × 0.85 + (50+100+200)×0.75 = 170 + 262 = 432
7. 建 Bomb + Cluster L1 → 爆炸後出現 8 個小碎片 projectile
8. Ice + Arctic Wind L2 → creep 在範圍內 HP 移動速度 -20%
9. Tack + Ring of Fire L3 → 每次射針時塔周 200 半徑 creep 一起扣血

**Step 3: 更新 graphify**

```bash
cd D:/omoba && graphify update .
```

**Step 4: 最終 commit（總結）**

```bash
git log --oneline -20
```
確認歷史乾淨，所有 phase 都有 commit 紀錄。

---

## 回滾策略

若中途出包：
- 各 task 都是獨立 commit，`git revert <sha>` 可回退單一 task
- `tower.rs` 的 3 個新欄位都用 `#[serde(default)]`，舊存檔仍相容
- DLL 若不相容，從 staging `omb/scripts/base_content.dll` 還原先前 blob 即可（git）

## 後續可選強化（非本次 scope）

- UI tooltip 顯示 upgrade 的完整 `description`
- upgrade animation（塔變色 / 光環）
- 音效 hook
- 存檔 / 讀檔（目前 serde 已支援，但 save system 是否可用 TBD）
