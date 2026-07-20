# 第三路線塔主動技能 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 為飛鏢、炸彈、冰凍、圖釘塔的第三路線第四級升級各加入一個無目標主動技能，讓七座已發布塔都至少具有一個主動技能。

**Architecture:** 沿用 `TowerActiveAbilityDef`、`TowerActiveAbilityState` 與既有 script callback，不增加網路協定或 ABI。持續型技能以塔自身 marker buff 修改既有命中流程，瞬發與 pulse 型技能直接產生有上限的決定性投射物。

**Tech Stack:** Rust 1.95.0、abi_stable script ABI、specs ECS、Fixed64 決定性模擬、Lua template metadata、Cargo test、gen-docs smoke test。

## Global Constraints

- Rust toolchain 固定為 1.95.0；scripts DLL 與 host 必須使用相同 rustc。
- 四個主動技能只能由各塔第三路線第四級解鎖，且不取代原有被動或自動效果。
- 四個技能皆為按鈕即時施放，不新增二次選取目標或位置的 UI。
- 冷卻一律 12 秒；飛鏢與炸彈持續 5 秒，冰凍為瞬發，圖釘為 0.4 秒四次 pulse。
- 不新增網路訊息、script ABI method、相依套件或圖示圖片。
- 所有模擬數值使用 `Fixed64`，放射方向使用既有決定性 Angle/trig API。
- 炸彈不增加碎片數；冰凍每次 16 枚；圖釘每 pulse 16 枚且共四次。
- 保留工作樹中既有的無關修改，尤其不得覆寫 `omoba-core/src/runtime/native/game_processor.rs` 的使用者變更。

## 檔案配置

- Modify: `scripts/lua_data/templates/towers.lua` — 四份 path 3 level 4 active metadata。
- Modify: `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs` — metadata 測試與允許 duration=0 的瞬發技能 lint。
- Modify: `scripts/base_content/src/towers/mod.rs` — 共用 activation、pulse、attack-hit 測試 adapter。
- Modify: `scripts/base_content/src/towers/dart.rs` — 重裝爆裂 marker 與 mega-crit 強化。
- Modify: `scripts/base_content/src/towers/bomb.rs` — 集束超載 marker 與碎片傷害／速度倍率。
- Modify: `scripts/base_content/src/towers/ice.rs` — 16 方向冰晶新星。
- Modify: `scripts/base_content/src/towers/tack.rs` — 四 pulse 刀刃漩渦。
- Modify: `omb/tests/gen_docs_smoke.rs` — 七個主動技能 catalog 驗收。

---

### Task 1: 主動技能 metadata 與 registry 契約

**Files:**
- Modify: `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs:212-230,561-624`
- Modify: `scripts/lua_data/templates/towers.lua:304-314,557-573,796-813,1014-1031`

**Interfaces:**
- Consumes: `TowerUpgradeDef.active_ability: Option<TowerActiveAbilityDef>` 與 Lua `active_ability` schema。
- Produces: 四個 ability ID：`dart_heavy_burst`、`bomb_cluster_overload`、`ice_crystal_nova`、`tack_blade_maelstrom`。

- [ ] **Step 1: 先把 registry 測試改成期待七個技能且限定新增技能位於 path index 2 / level 4**

```rust
#[test]
fn all_seven_towers_have_exactly_one_valid_active_ability() {
    let reg = TowerUpgradeRegistry::new();
    let expected = [
        (TOWER_DART.as_str(), "dart_heavy_burst", Fixed64::from_i32(12)),
        (TOWER_BOMB.as_str(), "bomb_cluster_overload", Fixed64::from_i32(12)),
        (TOWER_ICE.as_str(), "ice_crystal_nova", Fixed64::from_i32(12)),
        (TOWER_TACK.as_str(), "tack_blade_maelstrom", Fixed64::from_i32(12)),
        (TOWER_ARTY.as_str(), "arty_fire_at_will", Fixed64::from_i32(10)),
        (TOWER_CAKE_SPLASH.as_str(), "cake_dessert_party", Fixed64::from_i32(10)),
        (TOWER_BOOMERANG.as_str(), "boomerang_turbo_charge", Fixed64::from_i32(10)),
    ];
    for (kind, id, cooldown) in expected {
        let matches: Vec<_> = reg.iter_all()
            .filter(|def| def.tower_kind == kind && def.active_ability.is_some())
            .collect();
        assert_eq!(matches.len(), 1, "{kind}");
        let def = matches[0];
        assert_eq!((def.path, def.level), (2, 4));
        let active = def.active_ability.as_ref().unwrap();
        assert_eq!(active.ability_id, id);
        assert_eq!(active.cooldown, cooldown);
    }
}
```

- [ ] **Step 2: 執行測試確認因缺少四份 metadata 而失敗**

Run: `cargo test --manifest-path omb/Cargo.toml -p omoba-core all_seven_towers_have_exactly_one_valid_active_ability -- --exact`

Expected: FAIL，Dart/Bomb/Ice/Tack 的 active 數量為 0。

- [ ] **Step 3: 在四個第三路線第四級加入精確 metadata**

```lua
-- Dart path 3 level 4
active_ability = {
  ability_id = "dart_heavy_burst", display_name = "重裝爆裂",
  description = "5 秒內重裝爆擊的爆炸傷害與範圍加倍",
  icon = "assets/ui/abilities/dart_heavy_burst.png",
  cooldown = 12.0, duration = 5.0,
},

-- Bomb path 3 level 4
active_ability = {
  ability_id = "bomb_cluster_overload", display_name = "集束超載",
  description = "5 秒內所有集束碎片傷害與速度提升 50%",
  icon = "assets/ui/abilities/bomb_cluster_overload.png",
  cooldown = 12.0, duration = 5.0,
},

-- Ice path 3 level 4
active_ability = {
  ability_id = "ice_crystal_nova", display_name = "冰晶新星",
  description = "向 16 個方向發射高傷害凍結冰錐",
  icon = "assets/ui/abilities/ice_crystal_nova.png",
  cooldown = 12.0, duration = 0.0,
},

-- Tack path 3 level 4
active_ability = {
  ability_id = "tack_blade_maelstrom", display_name = "刀刃漩渦",
  description = "0.4 秒內連續發射 4 圈高傷害刀刃",
  icon = "assets/ui/abilities/tack_blade_maelstrom.png",
  cooldown = 12.0, duration = 0.4, pulse_interval = 0.1, pulse_count = 4,
},
```

- [ ] **Step 4: 擴充 strict lint 的 scoped towers，允許無 pulse 的瞬發技能 duration=0，並驗證 pulse 視窗足夠**

```rust
let scoped_towers = [
    TOWER_DART.as_str(), TOWER_BOMB.as_str(), TOWER_ICE.as_str(),
    TOWER_TACK.as_str(), TOWER_ARTY.as_str(), TOWER_CAKE_SPLASH.as_str(),
    TOWER_BOOMERANG.as_str(),
];
assert!(ability.cooldown > Fixed64::ZERO, "{label}: cooldown must be positive");
assert!(ability.duration >= Fixed64::ZERO, "{label}: duration must not be negative");
if ability.pulse_count > 0 {
    assert!(ability.duration >= ability.pulse_interval * Fixed64::from_i32(ability.pulse_count as i32));
}
```

- [ ] **Step 5: 執行 registry 測試並確認通過**

Run: `cargo test --manifest-path omb/Cargo.toml -p omoba-core tower_upgrade_registry::tests`

Expected: PASS，包含 strict lint 與七塔 active 契約。

- [ ] **Step 6: 提交 metadata 契約**

```powershell
git add -- scripts/lua_data/templates/towers.lua omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs
git commit -m "feat: define third-path tower active abilities"
```

### Task 2: 共用 tower script 測試 adapter

**Files:**
- Modify: `scripts/base_content/src/towers/mod.rs:246-414`

**Interfaces:**
- Consumes: `UnitScript::{on_attack_hit,on_tower_ability_activate_with_access,on_tower_ability_pulse_with_access}`。
- Produces: `invoke_attack_hit(...) -> Vec<Outcome>`、`invoke_activation(...) -> Vec<Outcome>`、`invoke_pulse(...) -> (bool, Vec<Outcome>)`。

- [ ] **Step 1: 加入編譯期會先失敗的 helper 使用測試**

在 Dart 測試模組暫時 import：

```rust
use crate::towers::projectile_test_support::{invoke_activation, invoke_attack_hit};
```

- [ ] **Step 2: 執行 Dart 測試確認 helper 尚不存在**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::dart::tests`

Expected: FAIL with unresolved imports `invoke_activation` and `invoke_attack_hit`。

- [ ] **Step 3: 在共用 fixture 實作三個 adapter**

```rust
pub fn invoke_attack_hit(
    fixture: &World,
    script: &impl UnitScript,
    tower: specs::Entity,
    victim: specs::Entity,
) -> Vec<Outcome> {
    let cache = ParallelAdapterCache::new(fixture, 1);
    let mut adapter = ParallelWorldAdapter::new(&cache, tower);
    let mut world_dyn = GameWorld_TO::from_ptr(RMut::new(&mut adapter), TD_Opaque);
    script.on_attack_hit(
        EntityHandle { id: tower.id(), gen: tower.gen().id() as u32 },
        EntityHandle { id: victim.id(), gen: victim.gen().id() as u32 },
        &mut world_dyn,
    );
    drop(world_dyn);
    adapter.finish()
}

pub fn invoke_activation(
    fixture: &World,
    script: &impl UnitScript,
    tower: specs::Entity,
    ability_id: &str,
) -> Vec<Outcome> {
    let cache = ParallelAdapterCache::new(fixture, 1);
    let mut adapter = ParallelWorldAdapter::new(&cache, tower);
    let access_adapter = ParallelTowerActiveAbilityAccess::new(&cache);
    let mut world_dyn = GameWorld_TO::from_ptr(RMut::new(&mut adapter), TD_Opaque);
    let access_dyn = TowerActiveAbilityAccess_TO::from_ptr(RRef::new(&access_adapter), TD_Opaque);
    script.on_tower_ability_activate_with_access(
        EntityHandle { id: tower.id(), gen: tower.gen().id() as u32 },
        RStr::from_str(ability_id),
        &access_dyn,
        &mut world_dyn,
    );
    drop(access_dyn);
    drop(world_dyn);
    let mut outcomes = adapter.finish();
    outcomes.extend(access_adapter.finish());
    outcomes
}

pub fn invoke_pulse(
    fixture: &World,
    script: &impl UnitScript,
    tower: specs::Entity,
    ability_id: &str,
    pulse_index: u16,
) -> (bool, Vec<Outcome>) {
    let cache = ParallelAdapterCache::new(fixture, 1);
    let mut adapter = ParallelWorldAdapter::new(&cache, tower);
    let access_adapter = ParallelTowerActiveAbilityAccess::new(&cache);
    let mut world_dyn = GameWorld_TO::from_ptr(RMut::new(&mut adapter), TD_Opaque);
    let access_dyn = TowerActiveAbilityAccess_TO::from_ptr(RRef::new(&access_adapter), TD_Opaque);
    let consumed = script.on_tower_ability_pulse_with_access(
        EntityHandle { id: tower.id(), gen: tower.gen().id() as u32 },
        RStr::from_str(ability_id),
        pulse_index,
        &access_dyn,
        &mut world_dyn,
    );
    drop(access_dyn);
    drop(world_dyn);
    (consumed, adapter.finish())
}
```

每個 adapter 都用 fixture entity 的 `id()` / `gen().id()` 建立 `EntityHandle`；activation 與 pulse 必須建立 `TowerActiveAbilityAccess_TO`，並在 `adapter.finish()` 前 drop ABI wrapper。

- [ ] **Step 4: 執行 base_content 測試確認 helper 編譯**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::`

Expected: PASS。

- [ ] **Step 5: 提交共用測試 adapter**

```powershell
git add -- scripts/base_content/src/towers/mod.rs
git commit -m "test: add tower active ability adapters"
```

### Task 3: 飛鏢重裝爆裂

**Files:**
- Modify: `scripts/base_content/src/towers/dart.rs:12-24,211-224`
- Test: `scripts/base_content/src/towers/dart.rs` inline tests

**Interfaces:**
- Consumes: `TowerActiveAbilityAccessDyn::get_tower_ability_active_remaining` 與 BuffStore-backed `get_buff_remaining`。
- Produces: marker `dart_heavy_burst_active`，只強化 `mega_crit` splash。

- [ ] **Step 1: 寫 activation、錯誤 ID、一般／強化命中測試**

```rust
#[test]
fn heavy_burst_marks_five_second_window_and_doubles_mega_crit_splash() {
    let mut fixture = fixture(
        &["always_crit", "mega_crit"],
        &[
            Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
            Vec2::new(Fixed64::from_i32(90), Fixed64::ZERO),
        ],
    );
    fixture.world.write_resource::<BuffStore>().add(
        fixture.tower, HEAVY_BURST_ACTIVE_BUFF, Fixed64::from_i32(5), serde_json::Value::Null,
    );
    let outcomes = invoke_attack_hit(&fixture.world, &DartTower, fixture.tower, fixture.enemies[0]);
    let splash_hits = outcomes.iter().filter(|outcome| matches!(outcome,
        Outcome::ScriptDirectDamage { amount, .. } if *amount == Fixed64::from_i32(120)
    )).count();
    assert_eq!(splash_hits, 2, "active radius 120 must reach the enemy at distance 90");
}

#[test]
fn heavy_burst_ignores_unknown_ability_id() {
    assert!(invoke_activation(&fixture.world, &DartTower, fixture.tower, "wrong").is_empty());
}
```

- [ ] **Step 2: 跑測試確認缺少 callback 與常數而失敗**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::dart::tests`

Expected: FAIL。

- [ ] **Step 3: 實作 marker 與條件式 splash 常數**

```rust
const HEAVY_BURST_ABILITY_ID: &str = "dart_heavy_burst";
const HEAVY_BURST_ACTIVE_BUFF: &str = "dart_heavy_burst_active";

fn on_tower_ability_activate_with_access(...) {
    if ability_id.as_str() != HEAVY_BURST_ABILITY_ID { return; }
    let remaining = access.get_tower_ability_active_remaining(tower, ability_id);
    if remaining > Fixed64::ZERO {
        w.add_buff(tower, RStr::from_str(HEAVY_BURST_ACTIVE_BUFF), remaining);
    }
}

let active = w.get_buff_remaining(attacker, RStr::from_str(HEAVY_BURST_ACTIVE_BUFF)) > Fixed64::ZERO;
let splash = if active { Fixed64::from_i32(120) } else { Fixed64::from_i32(60) };
w.deal_damage_splash(at, splash, splash, DamageKind::Physical, RSome(attacker));
```

- [ ] **Step 4: 執行 Dart 與完整 base_content 測試**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::dart::tests`

Expected: PASS。

- [ ] **Step 5: 提交飛鏢技能**

```powershell
git add -- scripts/base_content/src/towers/dart.rs
git commit -m "feat: add dart heavy burst active"
```

### Task 4: 炸彈集束超載

**Files:**
- Modify: `scripts/base_content/src/towers/bomb.rs:12-20,135-230`
- Test: `scripts/base_content/src/towers/bomb.rs` inline tests

**Interfaces:**
- Produces: marker `bomb_cluster_overload_active`；`on_projectile_hit` 讀 marker 後只乘 `frag_damage` 與 `frag_speed`。

- [ ] **Step 1: 寫 activation、錯誤 ID、兩代倍率與數量不變測試**

```rust
#[test]
fn cluster_overload_boosts_both_fragment_generations_without_adding_fragments() {
    let mut fixture = fixture(
        &["frag_homing", "frag_recursive"],
        &[Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)],
    );
    fixture.world.write_resource::<BuffStore>().add(
        fixture.tower, CLUSTER_OVERLOAD_ACTIVE_BUFF, Fixed64::from_i32(5), serde_json::Value::Null,
    );
    let first = invoke(&fixture.world, &BombTower, fixture.tower, fixture.enemies[0],
        ProjectileHitContext { kind_id: PROJECTILE_BOMB.0, generation: 0 });
    let first_fragments: Vec<_> = first.iter().filter(|outcome| matches!(outcome,
        Outcome::ScriptProjectile { generation: 1, .. }
    )).collect();
    assert_eq!(first_fragments.len(), 16);
    assert!(first_fragments.iter().all(|outcome| matches!(outcome,
        Outcome::ScriptProjectile { damage_phys, msd, .. }
            if *damage_phys == Fixed64::from_raw(69120) && *msd == Fixed64::from_i32(1200)
    )));
}
```

- [ ] **Step 2: 跑 Bomb 測試確認仍得到 damage=45、speed=800 而失敗**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::bomb::tests`

Expected: FAIL，倍率尚未套用。

- [ ] **Step 3: 實作 5 秒 marker 與 1.5 倍固定點倍率**

```rust
const CLUSTER_OVERLOAD_ABILITY_ID: &str = "bomb_cluster_overload";
const CLUSTER_OVERLOAD_ACTIVE_BUFF: &str = "bomb_cluster_overload_active";
const CLUSTER_OVERLOAD_MULTIPLIER: Fixed64 = Fixed64::from_raw(1536);

let overloaded = w.get_buff_remaining(attacker, RStr::from_str(CLUSTER_OVERLOAD_ACTIVE_BUFF)) > Fixed64::ZERO;
let multiplier = if overloaded { CLUSTER_OVERLOAD_MULTIPLIER } else { Fixed64::ONE };
let frag_damage = base_frag_damage * multiplier;
let frag_speed = Fixed64::from_i32(800) * multiplier;
```

activation callback 與 Dart 相同模式，但使用 Bomb 自己的 ID 與 marker。

- [ ] **Step 4: 執行 Bomb 測試**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::bomb::tests`

Expected: PASS，既有 generation-limit 測試仍通過。

- [ ] **Step 5: 提交炸彈技能**

```powershell
git add -- scripts/base_content/src/towers/bomb.rs
git commit -m "feat: add bomb cluster overload active"
```

### Task 5: 冰凍冰晶新星

**Files:**
- Modify: `scripts/base_content/src/towers/ice.rs:17-45,234-266`
- Test: `scripts/base_content/src/towers/ice.rs` inline tests

**Interfaces:**
- Produces: activation callback 直接產生 16 個 `PROJECTILE_ICICLE` straight projectiles。

- [ ] **Step 1: 寫無敵人、16 方向參數與錯誤 ID 測試**

```rust
#[test]
fn crystal_nova_emits_sixteen_deterministic_icicles_without_an_enemy() {
    let fixture = fixture(&["icicle_impale"], &[]);
    let outcomes = invoke_activation(&fixture.world, &IceTower, fixture.tower, "ice_crystal_nova");
    let shots: Vec<_> = outcomes.iter().filter(|o| matches!(o, Outcome::ScriptProjectile { .. })).collect();
    assert_eq!(shots.len(), 16);
    // 每枚：damage_phys=40（fixture final atk=10）、radius=75、stun=1.5、kind=PROJECTILE_ICICLE。
    // endpoints 必須包含 0° 的 (600,0)、90° 的 (0,600) 等決定性方向。
}
```

- [ ] **Step 2: 執行 Ice 測試確認沒有投射物而失敗**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::ice::tests`

Expected: FAIL，shot count 0。

- [ ] **Step 3: 實作 radial helper 與 activation callback**

```rust
const CRYSTAL_NOVA_ABILITY_ID: &str = "ice_crystal_nova";
const CRYSTAL_NOVA_PROJECTILES: u32 = 16;
const CRYSTAL_NOVA_RANGE: Fixed64 = Fixed64::from_i32(600);
const CRYSTAL_NOVA_SPLASH: Fixed64 = Fixed64::from_i32(75);
const CRYSTAL_NOVA_FREEZE: Fixed64 = Fixed64::from_raw(1536);

for i in 0..CRYSTAL_NOVA_PROJECTILES {
    let angle = Angle::from_degrees_i32(360 / CRYSTAL_NOVA_PROJECTILES as i32 * i as i32);
    let end = pos + Vec2::new(cos(angle), sin(angle)) * CRYSTAL_NOVA_RANGE;
    w.spawn_projectile_ex(ProjectileSpec {
        from: pos, owner: tower, path: PathSpec::Straight { end_pos: end },
        speed: stats.bullet_speed, damage: w.get_final_atk(tower) * Fixed64::from_i32(4),
        hit_radius: Fixed64::ZERO, splash_radius: CRYSTAL_NOVA_SPLASH,
        slow_factor: Fixed64::ZERO, slow_duration: Fixed64::ZERO,
        stun_duration: CRYSTAL_NOVA_FREEZE, kind_id: PROJECTILE_ICICLE.0,
    });
}
```

- [ ] **Step 4: 執行 Ice 測試**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::ice::tests`

Expected: PASS，既有 aura、absolute-zero、cryo-cannon 測試仍通過。

- [ ] **Step 5: 提交冰凍技能**

```powershell
git add -- scripts/base_content/src/towers/ice.rs
git commit -m "feat: add ice crystal nova active"
```

### Task 6: 圖釘刀刃漩渦

**Files:**
- Modify: `scripts/base_content/src/towers/tack.rs:13-40,154-165`
- Test: `scripts/base_content/src/towers/tack.rs` inline tests

**Interfaces:**
- Produces: `on_tower_ability_pulse_with_access(...) -> bool`；合法 pulse 0..3 各產生 16 枚 blade。

- [ ] **Step 1: 寫每 pulse 數量、四 pulse 總量、參數、錯誤 ID／index 測試**

```rust
#[test]
fn blade_maelstrom_emits_four_consumable_rings_of_sixteen_blades() {
    let fixture = fixture(&["needles_32", "burn_tier2"], &[]);
    let mut total = 0;
    for pulse in 0..4 {
        let (consumed, outcomes) = invoke_pulse(&fixture.world, &TackTower, fixture.tower, "tack_blade_maelstrom", pulse);
        assert!(consumed);
        total += outcomes.iter().filter(|o| matches!(o, Outcome::ScriptProjectile { .. })).count();
    }
    assert_eq!(total, 64);
    assert!(!invoke_pulse(&fixture.world, &TackTower, fixture.tower, "wrong", 0).0);
    assert!(!invoke_pulse(&fixture.world, &TackTower, fixture.tower, "tack_blade_maelstrom", 4).0);
}
```

- [ ] **Step 2: 執行 Tack 測試確認 default callback 沒有投射物而失敗**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::tack::tests`

Expected: FAIL，projectile total 0。

- [ ] **Step 3: 實作 pulse callback 與共用刀刃參數**

```rust
const BLADE_MAELSTROM_ABILITY_ID: &str = "tack_blade_maelstrom";
const BLADE_MAELSTROM_PULSES: u16 = 4;
const BLADE_MAELSTROM_BLADES: u32 = 16;
const BLADE_MAELSTROM_RANGE: Fixed64 = Fixed64::from_i32(600);

if ability_id.as_str() != BLADE_MAELSTROM_ABILITY_ID || pulse_index >= BLADE_MAELSTROM_PULSES {
    return false;
}
// 以 360/16 的 Angle 產生 straight projectile；damage=final_atk*3、
// hit_radius=110、kind=PROJECTILE_TACK_BLADE，其餘 status 欄位為 0。
true
```

保留現有 `on_projectile_hit` 對 `PROJECTILE_TACK_BLADE` 呼叫 `apply_burn`，不新增另一套燃燒邏輯。

- [ ] **Step 4: 執行 Tack 測試**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::tack::tests`

Expected: PASS，既有 burn 與 inferno 測試仍通過。

- [ ] **Step 5: 提交圖釘技能**

```powershell
git add -- scripts/base_content/src/towers/tack.rs
git commit -m "feat: add tack blade maelstrom active"
```

### Task 7: Catalog 與完整雙 workspace 驗證

**Files:**
- Modify: `omb/tests/gen_docs_smoke.rs:68-92`

**Interfaces:**
- Consumes: 新建的 `base_content.dll` 與七份 active metadata。
- Produces: catalog regression test 確認七座塔皆有技能。

- [ ] **Step 1: 先更新 smoke assertions**

```rust
assert_eq!(html.matches("class=\"tower-active-ability\"").count(), 7);
for ability_id in [
    "dart_heavy_burst", "bomb_cluster_overload", "ice_crystal_nova",
    "tack_blade_maelstrom", "boomerang_turbo_charge",
    "arty_fire_at_will", "cake_dessert_party",
] {
    assert!(html.contains(ability_id), "missing active {ability_id}");
}
assert_eq!(html.matches("cooldown 10s").count(), 3);
assert_eq!(html.matches("cooldown 12s").count(), 4);
```

- [ ] **Step 2: 格式化並跑完整 scripts workspace 測試**

Run: `cargo fmt --manifest-path scripts/Cargo.toml --all -- --check`

Expected: PASS；若失敗，執行不帶 `--check` 的同一指令後重跑。

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content`

Expected: PASS。

- [ ] **Step 3: 跑 omoba-core metadata/runtime 測試**

Run: `cargo test --manifest-path omb/Cargo.toml -p omoba-core tower_upgrade_registry::tests`

Expected: PASS。

- [ ] **Step 4: 建置並 stage DLL，再跑 gen-docs smoke**

```powershell
cargo build --manifest-path scripts/Cargo.toml -p base_content --release
Copy-Item -LiteralPath 'scripts/target/release/base_content.dll' -Destination 'omb/scripts/base_content.dll' -Force
cargo test --manifest-path omb/Cargo.toml -p omobab --features gen-docs --test gen_docs_smoke -- --ignored
```

Expected: PASS，HTML 超過 50KB、七塔各 12 個升級、七個 active ID 全部存在。

- [ ] **Step 5: 檢查 diff 與提交最後驗收**

Run: `git diff --check`

Expected: 無輸出。

```powershell
git add -- omb/tests/gen_docs_smoke.rs
git commit -m "test: require active abilities for every tower"
```

- [ ] **Step 6: 確認工作樹只剩實作前已存在的使用者修改**

Run: `git status --short`

Expected: 不應出現本計畫已提交檔案的未提交變更；既有 `omfue`、launcher script 或使用者正在修改的 `game_processor.rs` 可保留。
