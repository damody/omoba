# 糖球砲與回力鏢塔第三路線主動技能 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 保留糖球砲與回力鏢塔第二路線既有技能，並在兩塔第三路線第四級分別新增「糖霜封鎖」與「手裡劍風暴」。

**Architecture:** 沿用單一 `TowerActiveAbilityState`、既有升級安裝與無目標施放流程；高階路線互斥保證每座實際塔只會解鎖一個技能。糖霜封鎖以一次權威範圍查詢產生傷害、暈眩與同族糖霜 buff，手裡劍風暴以三次 pulse 產生有上限且可沿用既有彈射 hook 的決定性環形投射物。

**Tech Stack:** Rust 1.95.0、abi_stable script ABI、specs ECS、Fixed64 決定性模擬、Lua tower metadata、Cargo test、gen-docs smoke test。

## Global Constraints

- Rust toolchain 固定為 1.95.0；scripts DLL 與 host 必須使用相同 rustc。
- `cake_dessert_party` 與 `boomerang_turbo_charge` 必須留在第二路線第四級，數值與行為不變。
- 新技能只能位於第三路線第四級，且不取代 `cake_frost_50_vulnerability_25` 或 `storm_shuriken` 的永久效果。
- 兩個新技能皆為按鈕即時施放，不新增目標／地點選擇、網路訊息、script ABI method、多技能狀態或相依套件。
- 兩個新技能冷卻皆為 12 秒；糖霜封鎖為瞬發，手裡劍風暴為 0.6 秒內三次 pulse。
- 糖霜封鎖造成最終攻擊力兩倍魔法傷害、1.5 秒 `stun`，以及五秒 50% 減速與 25% 易傷。
- 手裡劍風暴每 pulse 12 枚、共 36 枚初始投射物；既有兩代彈射使整次施放最多再產生 72 枚投射物。
- 所有模擬數值使用 `Fixed64`；放射方向使用既有決定性 `Angle`、`sin`、`cos`。
- 不新增圖示圖片，只設定 `assets/ui/abilities/cake_frosting_lockdown.png` 與 `assets/ui/abilities/boomerang_shuriken_storm.png` metadata 路徑。
- 保留工作樹既有的 `omfue` 狀態與 `omoba-core/src/runtime/native/game_processor.rs` 未提交修改；本計畫不得 stage 或 commit 它們。

## 檔案配置

- Modify: `scripts/lua_data/templates/towers.lua` — 兩份第三路線第四級 active metadata。
- Modify: `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs` — 由每塔恰好一技改為九個精確升級位置與 ID 的 strict contract。
- Modify: `scripts/base_content/src/towers/cake_splash.rs` — 糖霜封鎖的傷害、暈眩、糖霜與測試。
- Modify: `scripts/base_content/src/towers/boomerang.rs` — 手裡劍風暴三次 pulse、環形投射物與測試。
- Modify in submodule: `omb/tests/gen_docs_smoke.rs` — catalog 從七個 active 升為九個，確認 ID 與冷卻分布。
- Do not modify: `omoba-core/src/runtime/native/game_processor.rs` — 既有通用安裝邏輯與測試目前位於使用者未提交變更中。

---

### Task 1: 九個主動升級的 metadata 與 registry 契約

**Files:**
- Modify: `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs:213-276,611-674`
- Modify: `scripts/lua_data/templates/towers.lua:1420-1427,1577-1585`

**Interfaces:**
- Consumes: `TowerUpgradeDef.active_ability: Option<TowerActiveAbilityDef>` 與 Lua `active_ability` schema。
- Produces: `cake_frosting_lockdown` 與 `boomerang_shuriken_storm` 兩份 runtime metadata；strict lint 允許同一 tower kind 在互斥路線各有一個定義。

- [ ] **Step 1: 把 registry 測試改成精確期待九個 active 位置與 ID**

將 `all_seven_towers_have_exactly_one_valid_active_ability` 改名為 `all_nine_active_upgrades_match_authored_routes`，並以完整 key 集合比較：

```rust
#[test]
fn all_nine_active_upgrades_match_authored_routes() {
    let reg = TowerUpgradeRegistry::new();
    let expected = BTreeSet::from([
        (TOWER_DART.as_str(), 2, 4, "dart_heavy_burst", Fixed64::from_i32(12).raw()),
        (TOWER_BOMB.as_str(), 2, 4, "bomb_cluster_overload", Fixed64::from_i32(12).raw()),
        (TOWER_ICE.as_str(), 2, 4, "ice_crystal_nova", Fixed64::from_i32(12).raw()),
        (TOWER_TACK.as_str(), 2, 4, "tack_blade_maelstrom", Fixed64::from_i32(12).raw()),
        (TOWER_ARTY.as_str(), 2, 4, "arty_fire_at_will", Fixed64::from_i32(10).raw()),
        (TOWER_CAKE_SPLASH.as_str(), 1, 4, "cake_dessert_party", Fixed64::from_i32(10).raw()),
        (TOWER_CAKE_SPLASH.as_str(), 2, 4, "cake_frosting_lockdown", Fixed64::from_i32(12).raw()),
        (TOWER_BOOMERANG.as_str(), 1, 4, "boomerang_turbo_charge", Fixed64::from_i32(10).raw()),
        (TOWER_BOOMERANG.as_str(), 2, 4, "boomerang_shuriken_storm", Fixed64::from_i32(12).raw()),
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
```

- [ ] **Step 2: 執行 registry 測試確認因缺少兩份 metadata 而失敗**

Run: `cargo test --manifest-path omb/Cargo.toml -p omoba-core all_nine_active_upgrades_match_authored_routes`

Expected: FAIL；actual 只有七筆，缺少 `cake_frosting_lockdown` 與 `boomerang_shuriken_storm`。

- [ ] **Step 3: 在兩個第三路線第四級加入 active metadata**

```lua
-- Cake Splash path 3 level 4
active_ability = {
  ability_id = "cake_frosting_lockdown", display_name = "糖霜封鎖",
  description = "凍結範圍內敵人，並延長極寒糖霜效果",
  icon = "assets/ui/abilities/cake_frosting_lockdown.png",
  cooldown = 12.0, duration = 0.0,
},

-- Boomerang path 3 level 4
active_ability = {
  ability_id = "boomerang_shuriken_storm", display_name = "手裡劍風暴",
  description = "0.6 秒內發射 3 圈可彈射手裡劍",
  icon = "assets/ui/abilities/boomerang_shuriken_storm.png",
  cooldown = 12.0, duration = 0.6, pulse_interval = 0.2, pulse_count = 3,
},
```

- [ ] **Step 4: 將 strict lint 的數量條件改成九筆精確總數，不再限制每塔恰好一筆**

保留逐筆 level、ID 唯一、cooldown、duration 與 pulse 視窗檢查；把函式尾端換成：

```rust
assert_eq!(ability_ids.len(), 9, "expected nine unique tower active abilities");
for tower_kind in scoped_towers {
    assert!(
        active_counts.get(tower_kind).is_some_and(|count| *count >= 1),
        "{tower_kind}: expected at least one active ability"
    );
}
assert_eq!(active_counts.get(TOWER_CAKE_SPLASH.as_str()), Some(&2));
assert_eq!(active_counts.get(TOWER_BOOMERANG.as_str()), Some(&2));
```

- [ ] **Step 5: 執行完整 registry 測試**

Run: `cargo test --manifest-path omb/Cargo.toml -p omoba-core tower_upgrade_registry::tests`

Expected: PASS；84 份升級 metadata 仍通過 strict lint，九份 active 的位置、ID、冷卻與 pulse 欄位吻合。

- [ ] **Step 6: 提交 metadata 與契約**

```powershell
git add -- scripts/lua_data/templates/towers.lua omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs
git commit -m "feat: define cake and boomerang third-path actives"
```

### Task 2: 糖球砲「糖霜封鎖」

**Files:**
- Modify: `scripts/base_content/src/towers/cake_splash.rs:7-84,146-190,205-390`

**Interfaces:**
- Consumes: `GameWorld::{query_enemies_in_range,deal_damage_splash,add_buff,add_stat_buff,emit_explosion}`、既有 `source_buff_id()` 與 `cake_frosting` aggregation family。
- Produces: `on_tower_ability_activate_with_access` 對 `cake_frosting_lockdown` 產生一次有上限的範圍效果。

- [ ] **Step 1: 寫糖霜封鎖的失敗測試**

在 inline tests import 共用 `invoke_activation`，並加入：

```rust
#[test]
fn frosting_lockdown_damages_stuns_and_applies_five_second_max_frosting() {
    let fixture = fixture(
        &["cake_frost_50_vulnerability_25"],
        &[
            Vec2::new(Fixed64::from_i32(100), Fixed64::ZERO),
            Vec2::new(Fixed64::from_i32(600), Fixed64::ZERO),
        ],
    );
    let outcomes = invoke_activation(
        &fixture.world,
        &CakeSplashTower,
        fixture.tower,
        "cake_frosting_lockdown",
    );
    assert!(outcomes.iter().any(|outcome| matches!(outcome,
        Outcome::ScriptDirectDamage { target, amount }
            if *target == fixture.enemies[0] && *amount == Fixed64::from_i32(20)
    )));
    assert!(!outcomes.iter().any(|outcome| matches!(outcome,
        Outcome::ScriptDirectDamage { target, .. } if *target == fixture.enemies[1]
    )));
    assert!(outcomes.iter().any(|outcome| matches!(outcome,
        Outcome::AddBuff { target, buff_id, duration, .. }
            if *target == fixture.enemies[0]
                && buff_id == "stun"
                && *duration == Fixed64::from_raw(1536)
    )));
    assert!(outcomes.iter().any(|outcome| matches!(outcome,
        Outcome::AddBuff { target, buff_id, duration, payload }
            if *target == fixture.enemies[0]
                && buff_id.starts_with("cake_frosting_lockdown:")
                && *duration == Fixed64::from_i32(5)
                && payload["__aggregation_family"] == "cake_frosting"
                && payload["movespeed_bonus_percentage"] == -512
                && payload["incoming_damage_percentage"] == 256
    )));
}

#[test]
fn frosting_lockdown_emits_without_enemies_and_ignores_other_ids() {
    let fixture = fixture(&["cake_frost_50_vulnerability_25"], &[]);
    let valid = invoke_activation(
        &fixture.world, &CakeSplashTower, fixture.tower, "cake_frosting_lockdown",
    );
    assert!(valid.iter().any(|outcome| matches!(outcome, Outcome::Explosion { .. })));
    assert!(invoke_activation(
        &fixture.world, &CakeSplashTower, fixture.tower, "wrong",
    ).is_empty());
}
```

- [ ] **Step 2: 執行 Cake 測試確認 callback 尚未產生效果**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::cake_splash::tests`

Expected: FAIL；找不到 20 傷害、`stun`、五秒糖霜與 explosion outcomes。

- [ ] **Step 3: 新增技能常數與專用糖霜 payload helper**

```rust
const FROSTING_LOCKDOWN_ABILITY_ID: &str = "cake_frosting_lockdown";
const FROSTING_LOCKDOWN_DAMAGE_FACTOR: Fixed64 = Fixed64::from_i32(2);
const FROSTING_LOCKDOWN_STUN: Fixed64 = Fixed64::from_raw(1536);
const FROSTING_LOCKDOWN_FROST_DURATION: Fixed64 = Fixed64::from_i32(5);

fn frosting_lockdown_payload() -> String {
    serde_json::json!({
        "__aggregation_family": "cake_frosting",
        "movespeed_bonus_percentage": FROST_50.raw(),
        "incoming_damage_percentage": VULNERABILITY_25.raw(),
    })
    .to_string()
}
```

- [ ] **Step 4: 實作 activation callback**

```rust
fn on_tower_ability_activate_with_access(
    &self,
    tower: EntityHandle,
    ability_id: RStr<'_>,
    _access: &TowerActiveAbilityAccessDyn<'_>,
    w: &mut GameWorldDyn<'_>,
) {
    if ability_id.as_str() != FROSTING_LOCKDOWN_ABILITY_ID {
        return;
    }
    let pos = match w.get_pos(tower) {
        RSome(pos) => pos,
        RNone => return,
    };
    let range = w.get_final_attack_range(tower);
    let victims: Vec<EntityHandle> = w.query_enemies_in_range(pos, range, tower).into();
    let damage = w.get_final_atk(tower) * FROSTING_LOCKDOWN_DAMAGE_FACTOR;
    w.deal_damage_splash(pos, range, damage, DamageKind::Magical, RSome(tower));
    w.emit_explosion(pos, range, Fixed64::from_raw(512));

    let frost_id = source_buff_id("cake_frosting_lockdown", tower);
    let frost_payload = frosting_lockdown_payload();
    for victim in victims {
        w.add_buff(
            victim,
            RStr::from_str("stun"),
            FROSTING_LOCKDOWN_STUN,
        );
        w.add_stat_buff(
            victim,
            RStr::from_str(&frost_id),
            FROSTING_LOCKDOWN_FROST_DURATION,
            RStr::from_str(&frost_payload),
        );
    }
}
```

- [ ] **Step 5: 執行 Cake 測試與完整 base_content 測試**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::cake_splash::tests`

Expected: PASS，既有一般糖霜、灼燒、追加脈衝與甜點狂歡測試仍通過。

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content`

Expected: PASS。

- [ ] **Step 6: 提交糖霜封鎖**

```powershell
git add -- scripts/base_content/src/towers/cake_splash.rs
git commit -m "feat: add cake frosting lockdown active"
```

### Task 3: 回力鏢塔「手裡劍風暴」

**Files:**
- Modify: `scripts/base_content/src/towers/boomerang.rs:17-35,192-210,286-390`

**Interfaces:**
- Consumes: `on_tower_ability_pulse_with_access`、`PROJECTILE_SHURIKEN`、既有 `storm_shuriken` 的 `on_projectile_hit` 兩代彈射。
- Produces: pulse index 0、1、2 各產生 12 枚不同相位的 straight shuriken；其他 ID 或 index 回傳 `false`。

- [ ] **Step 1: 寫三圈手裡劍的失敗測試**

在 inline tests import 共用 `invoke_pulse`，並加入：

```rust
#[test]
fn shuriken_storm_emits_three_rotated_rings_of_twelve_shuriken() {
    let fixture = fixture(&["storm_shuriken"], &[]);
    let mut endpoints = Vec::new();
    for pulse_index in 0..3 {
        let (consumed, outcomes) = invoke_pulse(
            &fixture.world,
            &BoomerangTower,
            fixture.tower,
            "boomerang_shuriken_storm",
            pulse_index,
        );
        assert!(consumed);
        let shots: Vec<_> = outcomes.iter().filter_map(|outcome| match outcome {
            Outcome::ScriptProjectile {
                tpos, msd, damage_phys, hit_radius, kind_id, generation, ..
            } if *kind_id == PROJECTILE_SHURIKEN.0 => {
                assert_eq!(*msd, Fixed64::from_i32(1500));
                assert_eq!(*damage_phys, Fixed64::from_i32(10));
                assert_eq!(*hit_radius, Fixed64::from_i32(90));
                assert_eq!(*generation, 0);
                Some(*tpos)
            }
            _ => None,
        }).collect();
        assert_eq!(shots.len(), 12);
        endpoints.push(shots);
    }
    assert_ne!(endpoints[0], endpoints[1]);
    assert_ne!(endpoints[1], endpoints[2]);
}

#[test]
fn shuriken_storm_rejects_unknown_id_and_out_of_range_pulse() {
    let fixture = fixture(&["storm_shuriken"], &[]);
    for (ability_id, pulse_index) in [("wrong", 0), ("boomerang_shuriken_storm", 3)] {
        let (consumed, outcomes) = invoke_pulse(
            &fixture.world, &BoomerangTower, fixture.tower, ability_id, pulse_index,
        );
        assert!(!consumed);
        assert!(outcomes.is_empty());
    }
}
```

- [ ] **Step 2: 執行 Boomerang 測試確認 default pulse callback 失敗**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::boomerang::tests`

Expected: FAIL；合法 pulse 的 `consumed` 為 false 且沒有 shuriken outcomes。

- [ ] **Step 3: 新增技能與環形投射物常數**

```rust
const SHURIKEN_STORM_ABILITY_ID: &str = "boomerang_shuriken_storm";
const SHURIKEN_STORM_PULSES: u16 = 3;
const SHURIKEN_STORM_PROJECTILES: u32 = 12;
const SHURIKEN_STORM_PHASE_DEGREES: i32 = 10;
```

- [ ] **Step 4: 實作 pulse callback**

```rust
fn on_tower_ability_pulse_with_access(
    &self,
    tower: EntityHandle,
    ability_id: RStr<'_>,
    pulse_index: u16,
    _access: &TowerActiveAbilityAccessDyn<'_>,
    w: &mut GameWorldDyn<'_>,
) -> bool {
    if ability_id.as_str() != SHURIKEN_STORM_ABILITY_ID
        || pulse_index >= SHURIKEN_STORM_PULSES
    {
        return false;
    }
    let pos = match w.get_pos(tower) {
        RSome(pos) => pos,
        RNone => return false,
    };
    let range = w.get_final_attack_range(tower);
    let speed = if w.has_tower_flag(tower, RStr::from_str("faster_rangs")) {
        STATS.bullet_speed * Fixed64::from_raw(1536)
    } else {
        STATS.bullet_speed
    };
    let damage = w.get_final_atk(tower);
    for i in 0..SHURIKEN_STORM_PROJECTILES {
        let degrees = i as i32 * 30 + pulse_index as i32 * SHURIKEN_STORM_PHASE_DEGREES;
        let angle = Angle::from_degrees_i32(degrees);
        let end = Vec2 {
            x: pos.x + cos(angle) * range,
            y: pos.y + sin(angle) * range,
        };
        w.spawn_projectile_ex(ProjectileSpec {
            from: pos,
            owner: tower,
            path: PathSpec::Straight { end_pos: end },
            speed,
            damage,
            hit_radius: Fixed64::from_i32(90),
            splash_radius: Fixed64::ZERO,
            slow_factor: Fixed64::ZERO,
            slow_duration: Fixed64::ZERO,
            stun_duration: Fixed64::ZERO,
            kind_id: PROJECTILE_SHURIKEN.0,
        });
    }
    true
}
```

- [ ] **Step 5: 加入 cross-path 彈速與既有兩代彈射不退化測試**

```rust
#[test]
fn shuriken_storm_inherits_faster_rangs_and_existing_ricochet_bound() {
    let fixture = fixture(
        &["storm_shuriken", "faster_rangs"],
        &[
            Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
            Vec2::new(Fixed64::from_i32(20), Fixed64::ZERO),
        ],
    );
    let (_, outcomes) = invoke_pulse(
        &fixture.world, &BoomerangTower, fixture.tower,
        "boomerang_shuriken_storm", 0,
    );
    assert!(outcomes.iter().filter(|outcome| matches!(outcome,
        Outcome::ScriptProjectile { msd, kind_id, .. }
            if *kind_id == PROJECTILE_SHURIKEN.0
                && *msd == Fixed64::from_i32(2250)
    )).count() == 12);

    let generation_two = invoke(
        &fixture.world,
        &BoomerangTower,
        fixture.tower,
        fixture.enemies[0],
        ProjectileHitContext { kind_id: PROJECTILE_SHURIKEN.0, generation: 2 },
    );
    assert!(generation_two.is_empty());
}
```

- [ ] **Step 6: 執行 Boomerang 與完整 base_content 測試**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::boomerang::tests`

Expected: PASS；渦輪充能的五秒 marker、攻速與額外兩枚投射物測試仍通過。

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content`

Expected: PASS。

- [ ] **Step 7: 提交手裡劍風暴**

```powershell
git add -- scripts/base_content/src/towers/boomerang.rs
git commit -m "feat: add boomerang shuriken storm active"
```

### Task 4: Catalog 與雙 workspace 完整驗證

**Files:**
- Modify in `omb` submodule: `omb/tests/gen_docs_smoke.rs:70-96`
- Generated/staged but not committed: `omb/scripts/base_content.dll`

**Interfaces:**
- Consumes: release `base_content.dll`、七座塔的 84 份升級與九份 active metadata。
- Produces: catalog regression test，確認九個 active ID 及其三種 cooldown 分布。

- [ ] **Step 1: 先把 gen-docs smoke assertions 更新為九個技能**

```rust
assert_eq!(
    html.matches("class=\"tower-active-ability\"").count(),
    9,
    "catalog must expose all nine route-specific tower actives"
);
for ability_id in [
    "dart_heavy_burst",
    "bomb_cluster_overload",
    "ice_crystal_nova",
    "tack_blade_maelstrom",
    "boomerang_turbo_charge",
    "boomerang_shuriken_storm",
    "arty_fire_at_will",
    "cake_dessert_party",
    "cake_frosting_lockdown",
] {
    assert!(html.contains(ability_id), "missing active {ability_id}");
}
assert_eq!(html.matches("cooldown 10s").count(), 3);
assert_eq!(html.matches("cooldown 12s").count(), 6);
```

- [ ] **Step 2: 在 `omb` submodule 執行 smoke test，確認舊 DLL／舊期待值尚未通過**

Run from repo root: `cargo test --manifest-path omb/Cargo.toml -p omobab --features gen-docs --test gen_docs_smoke -- --ignored`

Expected: FAIL，直到新 DLL staged 後 catalog 才會出現九份 active。

- [ ] **Step 3: 格式化並驗證 scripts workspace**

Run: `cargo fmt --manifest-path scripts/Cargo.toml --all -- --check`

Expected: PASS；若 formatter 回報差異，執行 `cargo fmt --manifest-path scripts/Cargo.toml --all`，再重跑 `--check`。

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content`

Expected: PASS，包含 Cake、Boomerang 與既有五塔的全部 script tests。

- [ ] **Step 4: 驗證 omoba-core metadata**

Run: `cargo test --manifest-path omb/Cargo.toml -p omoba-core tower_upgrade_registry::tests`

Expected: PASS，九個 active 的 strict contract 全部通過。

- [ ] **Step 5: Release build script DLL 並 stage 到 backend**

```powershell
cargo build --manifest-path scripts/Cargo.toml -p base_content --release
Copy-Item -LiteralPath 'scripts/target/release/base_content.dll' -Destination 'omb/scripts/base_content.dll' -Force
```

Expected: `base_content` release build 成功，`omb/scripts/base_content.dll` 的時間戳更新。

- [ ] **Step 6: 執行新的 gen-docs smoke test**

Run: `cargo test --manifest-path omb/Cargo.toml -p omobab --features gen-docs --test gen_docs_smoke -- --ignored`

Expected: PASS；catalog 含七座塔、每塔十二個升級、九個 active、三個 10 秒與六個 12 秒 cooldown 標籤。

- [ ] **Step 7: 在 `omb` submodule 提交 catalog 契約**

```powershell
git -C omb status --short
git -C omb add -- tests/gen_docs_smoke.rs
git -C omb commit -m "test: catalog cake and boomerang route actives"
```

Expected: 只提交 `tests/gen_docs_smoke.rs`；若 staged DLL 被忽略或顯示為修改，不納入 commit。

- [ ] **Step 8: 在 root repo 提交 `omb` submodule 指標**

```powershell
git add -- omb
git commit -m "chore: bump omb tower active catalog"
```

Expected: root commit 只包含 `omb` gitlink 更新，不包含 `omfue` 或 `game_processor.rs`。

- [ ] **Step 9: 最終 diff 與工作樹確認**

Run: `git diff --check`

Expected: 無 whitespace error。

Run: `git status --short`

Expected: 本計畫修改均已提交；只保留實作前既有的 `? omfue` 與 `M omoba-core/src/runtime/native/game_processor.rs`。
