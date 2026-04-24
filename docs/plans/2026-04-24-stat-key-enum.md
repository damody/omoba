# Stat Key Enum 遷移 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把散落在腳本裡的 stat key 字串（`"crit_chance"`、`"health_bonus"` 等 165 項）改成 `enum StatKey`，編譯期擋下拼字錯誤；同時把 `omb-script-abi` crate 從 `omb/script-abi/` 搬到 `scripts/script-abi/`，讓 mod 作者日後 clone 公開 `scripts` repo 即可拿到 SDK 與範例。

**Architecture:**
- `StatKey` 用 `#[repr(u16)] #[derive(StableAbi, ...)]` + variant discriminant 顯式寫死，對 abi_stable FFI 安全。
- `GameWorld` trait 的三個 stat method（`sum_stat` / `product_stat` / `get_stat_bonus`）改收 `StatKey`；`add_stat_buff` 的 `modifiers_json` payload 仍是 JSON 字串，但 field key 透過 `StatKey::as_str()` 產生。
- 主程式 `omb/` 未來轉閉源；`scripts/script-abi` 先以 path 依賴 (`omb-script-abi = { path = "../scripts/script-abi" }`) 和 mod 作者一起 clone monorepo；crates.io publish 以後另議。

**Tech Stack:** Rust 1.91.0、abi_stable 0.11、specs 0.20（ECS）、serde_json（BuffStore payload）、syn（gen-docs AST 掃描，本 plan 改為 runtime call）。

**Pre-flight（開工前請確認）：**
- 目前 `git status` 有未提交的 omb submodule bump + 3 個 hero 檔案變更（`No1_sniper_mode.rs` / `No2_saika_reinforcements.rs` / `No4_matchlock_gun.rs`）。**這些變更與本 plan 無關**；先 `git stash` 或 `git commit -m "wip: 既有未完成工作"`，避免和 Phase A-0 搬移混在一起。
- 這個 plan 沒跑在 dedicated worktree 裡；若你想要隔離開發分支，呼叫 superpowers:using-git-worktrees 先開 worktree 再動手。

---

## Task 1: 遷移 `omb-script-abi` crate 到 `scripts/script-abi`（Phase A-0）

**為什麼先做**：之後每個 task 都會碰 `script-abi` 的程式碼，先把位置定案才不會改兩次。

**Files:**
- Move: `D:\omoba\omb\script-abi\*` → `D:\omoba\scripts\script-abi\*`
- Modify: `D:\omoba\omb\Cargo.toml:25-29, 57`
- Modify: `D:\omoba\scripts\Cargo.toml:3-5`
- Modify: `D:\omoba\scripts\base_content\Cargo.toml:17`
- Modify: `D:\omoba\scripts\script-abi\Cargo.toml`（新位置，補 license）
- Modify: `D:\omoba\CLAUDE.md`（更新架構描述的路徑）

**Step 1: 在 omb submodule 內移除 script-abi（保留本地暫存）**

```bash
cd D:/omoba/omb
# 先用 git mv 到 repo 外會失敗（跨 repo），用 git rm 搭配複製：
cp -r script-abi /tmp/script-abi-backup
git rm -r script-abi
git status   # 應看到 "deleted: script-abi/Cargo.toml" 等
```
Expected: 工作目錄無 `omb/script-abi/`；索引有 deletion 待 commit。

**Step 2: 複製到 scripts/script-abi**

```bash
mkdir -p D:/omoba/scripts/script-abi
cp -r /tmp/script-abi-backup/* D:/omoba/scripts/script-abi/
ls D:/omoba/scripts/script-abi/src/   # 應看到 ability.rs, lib.rs, manifest.rs, script.rs, stat_keys.rs, types.rs, world.rs
```

**Step 3: 改 `scripts/Cargo.toml` 把 `script-abi` 加入 members**

Edit `D:\omoba\scripts\Cargo.toml:3-5`：
```toml
members = [
    "script-abi",
    "base_content",
]
```
把 `script-abi` 放第一位讓 build 順序直觀。

**Step 4: 改 `scripts/base_content/Cargo.toml` path 依賴**

Edit line 17:
```toml
# 舊：omb-script-abi = { path = "../../omb/script-abi" }
omb-script-abi = { path = "../script-abi" }
```

**Step 5: 改 `omb/Cargo.toml` 移除 script-abi member + 修依賴**

Edit `D:\omoba\omb\Cargo.toml:25-29`：
```toml
[workspace]
members = [
    ".",
]
```

Edit line 57:
```toml
# 舊：omb-script-abi = { path = "script-abi" }
omb-script-abi = { path = "../scripts/script-abi" }
```

**Step 6: 在新位置 `scripts/script-abi/Cargo.toml` 補開源 metadata**

追加到既有 `[package]` 段：
```toml
license = "MIT OR Apache-2.0"
# 等 scripts repo push 到 github 再解註：
# repository = "https://github.com/damody/omoba-scripts"
publish = false   # 本階段先不推 crates.io；mod 作者走 path/git 依賴
```

`license` 值請使用者 review 後確認；目前 omoba 專案未宣告過 license，選 MIT OR Apache-2.0 是 Rust 生態系標準雙授權。

**Step 7: 三個 workspace 單獨 cargo check 驗證**

```bash
cd D:/omoba/scripts && cargo check -p omb-script-abi
cd D:/omoba/scripts && cargo check -p base_content
cd D:/omoba/omb && cargo check -p omobab
```
Expected: 三個都編得過。若 `omobab` 說 `omb-script-abi` 找不到，回頭檢查 `omb/Cargo.toml` 的 path 是否是 `../scripts/script-abi`（注意 `omb/` 是 submodule，它的 Cargo.toml 改動要在 submodule 內 commit）。

**Step 8: 更新 CLAUDE.md 的路徑描述**

Edit `D:\omoba\CLAUDE.md` 把 `omb/script-abi` 的字串全部改為 `scripts/script-abi`：
- 目錄架構段 `omoba-core/` 附近有 `omb/script-abi`
- 「Traits / 生命週期」段落的 script-abi 描述
- 補一行：「omb 單獨 clone 需搭配 scripts workspace（path 依賴 `../scripts/script-abi`）」放在「## 目錄架構」末尾

**Step 9: 兩階段 commit**

`omb` submodule 先 commit：
```bash
cd D:/omoba/omb
git add -A
git commit -m "$(cat <<'EOF'
refactor: move script-abi out to external scripts workspace

准备 omb 未來轉閉源；script-abi 改由公開的 scripts repo 提供，
omb 透過 path 依賴 ../scripts/script-abi 取用。

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

monorepo 根目錄 commit：
```bash
cd D:/omoba
git add omb scripts CLAUDE.md
git commit -m "$(cat <<'EOF'
refactor(script-abi): relocate to scripts/script-abi

- scripts workspace 納入 script-abi 成員
- base_content / omb 的 path 依賴指向新位置
- CLAUDE.md 更新架構路徑

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: 建立 StatKey enum 骨架（TDD - 3 個 variant 驗證模式）

**Files:**
- Modify: `D:\omoba\scripts\script-abi\src\stat_keys.rs`（加 enum 定義到檔案最前面，暫不動既有 const）
- Test: `D:\omoba\scripts\script-abi\src\stat_keys.rs`（檔尾加 `#[cfg(test)] mod tests`）

**Step 1: 先寫 failing test — 3 個代表性 variant 的 as_str round-trip**

Edit `stat_keys.rs` 檔尾追加：
```rust
#[cfg(test)]
mod tests {
    use super::StatKey;

    #[test]
    fn as_str_roundtrip_smoke() {
        assert_eq!(StatKey::PreattackBonusDamage.as_str(), "preattack_bonus_damage");
        assert_eq!(StatKey::AttackspeedBonusConstant.as_str(), "attackspeed_bonus_constant");
        assert_eq!(StatKey::DamageoutgoingPercentage.as_str(), "damageoutgoing_percentage");
    }

    #[test]
    fn from_str_roundtrip_smoke() {
        assert_eq!(StatKey::from_str_key("preattack_bonus_damage"), Some(StatKey::PreattackBonusDamage));
        assert_eq!(StatKey::from_str_key("not_a_real_key"), None);
    }
}
```

**Step 2: 跑測試驗證失敗**

```bash
cd D:/omoba/scripts && cargo test -p omb-script-abi --lib stat_keys
```
Expected: FAIL，`cannot find type StatKey in this scope`。

**Step 3: 寫最小實作 — 3 個 variant + 兩個 method**

在 `stat_keys.rs` 開頭（在所有 const 宣告之前）加：
```rust
use abi_stable::StableAbi;

/// Script ABI 的 stat key 枚舉。
///
/// # SAFETY
/// Variant 順序 = FFI ABI 契約：新增只能 **追加到尾端**，絕不可在中間 insert
/// 或更動 discriminant 值，否則 host 與 script DLL 版本不同步會 UB。
/// 每個 variant 顯式寫 `= N` 以鎖定值。
#[repr(u16)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StatKey {
    PreattackBonusDamage = 0,
    AttackspeedBonusConstant = 1,
    DamageoutgoingPercentage = 2,
    // ... Task 5 補齊其餘 162 個
}

impl StatKey {
    pub fn as_str(self) -> &'static str {
        match self {
            StatKey::PreattackBonusDamage => "preattack_bonus_damage",
            StatKey::AttackspeedBonusConstant => "attackspeed_bonus_constant",
            StatKey::DamageoutgoingPercentage => "damageoutgoing_percentage",
        }
    }

    pub fn from_str_key(s: &str) -> Option<StatKey> {
        match s {
            "preattack_bonus_damage" => Some(StatKey::PreattackBonusDamage),
            "attackspeed_bonus_constant" => Some(StatKey::AttackspeedBonusConstant),
            "damageoutgoing_percentage" => Some(StatKey::DamageoutgoingPercentage),
            _ => None,
        }
    }
}
```

**Step 4: 跑測試驗證通過**

```bash
cd D:/omoba/scripts && cargo test -p omb-script-abi --lib stat_keys
```
Expected: 兩個 test PASS。

**Step 5: Commit**

```bash
cd D:/omoba/scripts
git add script-abi/src/stat_keys.rs
git commit -m "$(cat <<'EOF'
feat(stat-key): add StatKey enum skeleton with 3 smoke variants

TDD foundation for full 165-variant migration.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: StatSection enum + `section()` method

**Files:**
- Modify: `D:\omoba\scripts\script-abi\src\stat_keys.rs`

**Step 1: 寫失敗 test**

追加到 `mod tests`：
```rust
#[test]
fn section_classification() {
    use super::StatSection;
    assert_eq!(StatKey::PreattackBonusDamage.section(), StatSection::All);
    // Task 5 把 MovespeedBonusConstant 加完後會是 NonBuilding
    // Task 5 完成後追加 Visual section test
}
```

**Step 2: 跑測試（FAIL — `StatSection` / `section` 不存在）**

```bash
cargo test -p omb-script-abi --lib stat_keys
```

**Step 3: 實作 StatSection + method**

在 `stat_keys.rs` 加：
```rust
#[repr(u8)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StatSection {
    All = 0,
    NonBuilding = 1,
    Visual = 2,
}

impl StatKey {
    pub fn section(self) -> StatSection {
        match self {
            StatKey::PreattackBonusDamage
            | StatKey::AttackspeedBonusConstant
            | StatKey::DamageoutgoingPercentage => StatSection::All,
            // Task 5 會把 NonBuilding / Visual 的 variant 補上
        }
    }
}
```
這個 match 現在只有 3 個 variant 所以 exhaustive，Task 5 擴充時 compiler 會強制把 match arm 補齊。

**Step 4: 跑測試通過**

```bash
cargo test -p omb-script-abi --lib stat_keys
```

**Step 5: Commit**

```bash
git add script-abi/src/stat_keys.rs
git commit -m "feat(stat-key): add StatSection enum + section() classifier

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Aggregation enum + `aggregation()` method

**Files:**
- Modify: `D:\omoba\scripts\script-abi\src\stat_keys.rs`

**Step 1: 寫失敗 test（4 種聚合方式各一個代表）**

```rust
#[test]
fn aggregation_by_suffix() {
    use super::Aggregation;
    assert_eq!(StatKey::PreattackBonusDamage.aggregation(), Aggregation::SumAdd);           // _Bonus
    assert_eq!(StatKey::AttackspeedBonusConstant.aggregation(), Aggregation::SumAdd);       // _Constant
    assert_eq!(StatKey::DamageoutgoingPercentage.aggregation(), Aggregation::SumAddThenMul1Plus); // _Percentage
    // Task 5 加完 MULTIPLIER / CHANCE variant 後追加
}
```

**Step 2: 跑測試驗證 FAIL**

```bash
cargo test -p omb-script-abi --lib stat_keys
```

**Step 3: 實作 Aggregation + method**

```rust
#[repr(u8)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Aggregation {
    SumAdd = 0,            // _Bonus / _Constant / _Stacking → sum
    SumAddThenMul1Plus = 1,// _Percentage → (1 + sum)
    ProductMult = 2,       // _Multiplier → product
    Chance = 3,            // _Chance → [0..=1]
    PassThrough = 4,       // 視覺/其他
}

impl StatKey {
    pub fn aggregation(self) -> Aggregation {
        match self {
            StatKey::PreattackBonusDamage => Aggregation::SumAdd,
            StatKey::AttackspeedBonusConstant => Aggregation::SumAdd,
            StatKey::DamageoutgoingPercentage => Aggregation::SumAddThenMul1Plus,
        }
    }
}
```

**Step 4: 跑測試通過**

```bash
cargo test -p omb-script-abi --lib stat_keys
```

**Step 5: Commit**

```bash
git commit -am "feat(stat-key): add Aggregation enum + aggregation() method

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: 填完 162 個既有 variant + 7 個新 variant（共 165）

這是整個 plan 最機械式的一步，但因為 match exhaustiveness compile-time check，錯一個就編不過，安全。

**Files:**
- Modify: `D:\omoba\scripts\script-abi\src\stat_keys.rs`（整個重寫）

**Step 1: 寫 ALL round-trip test（強迫 variant 數量一致）**

加到 tests 模組：
```rust
#[test]
fn all_variants_roundtrip() {
    // 所有 variant 列舉；每次加 variant 要同步加這裡（失去 Hint 可用 strum::EnumIter
    // 但不拉進 script-abi，故手動維護）。
    const ALL: &[StatKey] = &[
        StatKey::PreattackBonusDamage,
        StatKey::PreattackBonusDamageProc,
        // ... 全部 165 個
    ];
    assert_eq!(ALL.len(), 165);
    for &v in ALL {
        assert_eq!(StatKey::from_str_key(v.as_str()), Some(v), "round-trip failed for {:?}", v);
    }
}
```

**Step 2: 跑測試驗證 FAIL（variant 數不夠或 round-trip 不通）**

```bash
cargo test -p omb-script-abi --lib stat_keys
```

**Step 3: 產生完整 enum / as_str / from_str_key / section / aggregation / ALL**

這步建議用小 Python 腳本從現有 `stat_keys.rs` 的 const 清單自動產生，避免手抖。腳本邏輯：

1. 讀 `stat_keys.rs`（備份版本，或用 `git show HEAD~N:script-abi/src/stat_keys.rs`）
2. 對每個 `pub const FOO_BAR: &str = "foo_bar";`：
   - variant 名 = `FooBar`（SNAKE → PascalCase）
   - string 值 = `"foo_bar"`
3. 根據 SECTION 1/2/3 行號分組得 `section()`
4. 根據 suffix 分類 `aggregation()`：
   - `_bonus` / `_constant` / `_stacking` → SumAdd
   - `_percentage` → SumAddThenMul1Plus
   - `_multiplier` → ProductMult
   - `_chance` → Chance
   - 其他 → PassThrough
5. 最後補 7 個新 variant（全為 Section 1）：
   - `CritChance` = `"crit_chance"` → Chance
   - `CritBonus` = `"crit_bonus"` → SumAdd
   - `SplashBonus` = `"splash_bonus"` → SumAdd
   - `SlowFactorOverride` = `"slow_factor_override"` → PassThrough（覆蓋值不聚合）
   - `SlowDurationBonus` = `"slow_duration_bonus"` → SumAdd
   - `AttackStunChance` = `"attack_stun_chance"` → Chance
   - `AttackStunDuration` = `"attack_stun_duration"` → SumAdd
6. discriminant 依序編號 `= 0, 1, ... 164`
7. 輸出完整 `stat_keys.rs`

執行：
```bash
cd D:/omoba/scripts/script-abi
python3 gen_stat_keys.py > src/stat_keys.rs.new
mv src/stat_keys.rs.new src/stat_keys.rs
```

**Step 4: 跑所有測試確認 165 variant 都通**

```bash
cargo test -p omb-script-abi --lib stat_keys
```
Expected: `all_variants_roundtrip` PASS（含 `ALL.len() == 165`）；前面 3 個 smoke test 仍 PASS。

**Step 5: 把 gen_stat_keys.py 放到 docs/tools/ 給後人維護參考**

```bash
mkdir -p D:/omoba/docs/tools
cp gen_stat_keys.py D:/omoba/docs/tools/gen_stat_keys.py
```

**Step 6: Commit**

```bash
cd D:/omoba
git add scripts/script-abi/src/stat_keys.rs docs/tools/gen_stat_keys.py
git commit -m "$(cat <<'EOF'
feat(stat-key): complete 165-variant StatKey enum

- 158 variants migrated from existing pub const &str definitions
- 7 new variants for tower/hero stat keys that were hardcoded as strings
  (crit_chance/bonus, splash_bonus, slow_factor_override,
   slow_duration_bonus, attack_stun_chance/duration)
- ALL round-trip test enforces as_str ↔ from_str_key consistency

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: `is_building_excluded()` 取代舊 `BUILDING_EXCLUDED_KEYS` 陣列

**Files:**
- Modify: `D:\omoba\scripts\script-abi\src\stat_keys.rs`

**Step 1: 寫 failing test**

```rust
#[test]
fn building_excluded_flags() {
    // Section 2 的 variant 應該被建築物排除
    assert!(StatKey::MovespeedBonusConstant.is_building_excluded());
    // Section 1 通用 variant 建築物吃
    assert!(!StatKey::PreattackBonusDamage.is_building_excluded());
}
```

**Step 2: 跑測試驗證 FAIL**

```bash
cargo test -p omb-script-abi --lib stat_keys
```

**Step 3: 實作**

預設行為：Section 2 (`NonBuilding`) 的都 excluded，加個覆寫白名單（舊 `BUILDING_EXCLUDED_KEYS` 陣列的語義）。因為 section() 已分好，直接 delegate：

```rust
impl StatKey {
    pub fn is_building_excluded(self) -> bool {
        self.section() == StatSection::NonBuilding
    }
}
```

**Step 4: 跑測試通過**

```bash
cargo test -p omb-script-abi --lib stat_keys
```

**Step 5: 刪舊 `pub const BUILDING_EXCLUDED_KEYS: &[&str]` 陣列**

Grep `BUILDING_EXCLUDED_KEYS` 全 repo，把所有引用點（可能在 host UnitStats 裡）改為呼叫 `key.is_building_excluded()`。這個在 Task 9 批次遷移時一起處理，這裡只刪宣告。

**Step 6: Commit**

```bash
git commit -am "feat(stat-key): replace BUILDING_EXCLUDED_KEYS slice with is_building_excluded()

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: 修改 `GameWorld` trait 簽名改收 `StatKey`

**Files:**
- Modify: `D:\omoba\scripts\script-abi\src\world.rs:137, 141, 224`

**Step 1: 改三個 method signature**

```rust
// world.rs:137
fn sum_stat(&self, e: EntityHandle, stat_key: StatKey) -> f32;

// world.rs:141
fn product_stat(&self, e: EntityHandle, stat_key: StatKey) -> f32;

// world.rs:224
fn get_stat_bonus(&self, e: EntityHandle, key: StatKey) -> f32;
```

在檔頭 `use super::stat_keys::StatKey;`（或 `crate::stat_keys::StatKey;`）。

**Step 2: 驗證 script-abi 獨立 compile**

```bash
cd D:/omoba/scripts && cargo check -p omb-script-abi
```
Expected: PASS（script-abi 自己沒有 GameWorld impl，只有 trait 定義）。

**Step 3: 先不跑 host / scripts 的 check — 接下來 Task 8-13 會讓它們跟上**

**Step 4: Commit**

```bash
cd D:/omoba
git add scripts/script-abi/src/world.rs
git commit -m "feat(script-abi): GameWorld stat methods take StatKey instead of RStr

Breaks host and base_content; fixed in follow-up tasks.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 8: host BuffStore 改收 `StatKey`

**Files:**
- Modify: `D:\omoba\omb\src\ability_runtime\buff_store.rs`（`sum_add`, `product_mult` sig）

**Step 1: 改兩個 method 的 parameter 型別**

```rust
use omb_script_abi::stat_keys::StatKey;

impl BuffStore {
    pub fn sum_add(&self, entity: Entity, stat: StatKey) -> f32 {
        let key = stat.as_str();
        self.iter_for(entity)
            .filter_map(|(_, e)| e.payload.get(key).and_then(|v| v.as_f64()))
            .sum::<f64>() as f32
    }

    pub fn product_mult(&self, entity: Entity, stat: StatKey) -> f32 {
        let key = stat.as_str();
        self.iter_for(entity)
            .filter_map(|(_, e)| e.payload.get(key).and_then(|v| v.as_f64()))
            .fold(1.0f64, |acc, v| acc * v) as f32
    }
}
```

**Step 2: 此時 omobab 編譯會爆掉在 UnitStats 等呼叫處（傳 `&str` 給 `StatKey` 參數）**

故意的 — Task 9 批次修。先確認 `buff_store.rs` 自己的測試（若有）還過。

```bash
cd D:/omoba/omb && cargo check -p omobab 2>&1 | head -30
```
Expected: 錯誤出現在 unit_stats.rs 大量行，不在 buff_store.rs。

**Step 3: Commit（破窗狀態，用來 bisect 定位）**

```bash
git add -A
git commit -m "refactor(buff-store): take StatKey enum; callers broken pending next task

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 9: 批次遷移 host UnitStats 與其他 sk:: 呼叫點

**Files:**
- Modify: `D:\omoba\omb\src\ability_runtime\unit_stats.rs`（~100 個呼叫）
- Modify: 所有 `Grep "sk::" omb/src/` 命中的檔案
- Modify: 所有 `Grep "stat_keys::" omb/src/` 命中的檔案

**Step 1: 找出所有呼叫點**

```bash
cd D:/omoba/omb
# 用 Grep tool 不是 bash grep，但 plan 裡示意：
grep -rn "sk::" src/ | tee /tmp/sk-callers.txt
grep -rn "stat_keys::" src/ | tee -a /tmp/sk-callers.txt
```

**Step 2: 產生 sed 批次替換 script**

寫個 `docs/tools/sk_const_to_enum.py`（或 bash）：
- 讀 `scripts/script-abi/src/stat_keys.rs` 的 enum 定義
- 產生每行 `s/sk::PREATTACK_BONUS_DAMAGE/StatKey::PreattackBonusDamage/g` 等 165 條 sed rule
- 對 `/tmp/sk-callers.txt` 列的檔案跑 `sed -i -f rules.sed <file>`

另外把這些 import 更新：
- `use omb_script_abi::stat_keys as sk;` → `use omb_script_abi::stat_keys::StatKey;`

Step 3: 補 `BUILDING_EXCLUDED_KEYS` 參照：
`grep -n BUILDING_EXCLUDED_KEYS omb/src/` 每處改為 `key.is_building_excluded()` 呼叫。

**Step 4: cargo check**

```bash
cd D:/omoba/omb && cargo check -p omobab
```
Expected: 如果還剩錯誤大多是 `GameWorld` impl 自身（Task 10 處理）。UnitStats / buff_store 的呼叫全部乾淨。

**Step 5: Commit**

```bash
git add -A
git commit -m "$(cat <<'EOF'
refactor(unit-stats): migrate all sk:: const to StatKey enum

Automated via docs/tools/sk_const_to_enum.py. Also replaces
BUILDING_EXCLUDED_KEYS slice lookups with StatKey::is_building_excluded().

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: 修 host 端 `GameWorld` trait impl

**Files:**
- Grep 找到位置：`Grep "impl GameWorld" omb/src/`（預期命中 `omb_game_world.rs` 或類似名字）

**Step 1: 找到 impl**

```bash
cd D:/omoba/omb
grep -rn "impl.*GameWorld" src/
```

**Step 2: 改三個 stat method impl**

```rust
fn sum_stat(&self, e: EntityHandle, stat_key: StatKey) -> f32 {
    let entity = /* EntityHandle → Entity 轉換，現有程式已有 */;
    self.buff_store.sum_add(entity, stat_key)
}

fn product_stat(&self, e: EntityHandle, stat_key: StatKey) -> f32 {
    let entity = /* ... */;
    self.buff_store.product_mult(entity, stat_key)
}

fn get_stat_bonus(&self, e: EntityHandle, key: StatKey) -> f32 {
    // 依現有語義實作（通常等價於 sum_stat 或是 tower upgrade 專用的 accessor）
    // 以目前程式碼為準
}
```

**Step 3: cargo check 綠**

```bash
cd D:/omoba/omb && cargo check -p omobab
```
Expected: PASS（host side 完全乾淨）。

**Step 4: cargo test（host 單元測試，不含 scripts）**

```bash
cargo test -p omobab --lib 2>&1 | tail -20
```
Expected: 全部 PASS（若既有測試不吃 stat_key 字串的，就是純建置通過就好）。

**Step 5: Commit**

```bash
git add -A
git commit -m "refactor(game-world): GameWorld impl accepts StatKey enum parameters

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 11: gen-docs 改為解析 enum 而非 const

**Files:**
- Modify: `D:\omoba\omb\src\bin\gen_docs_lib\api_scan.rs:156-202`（`scan_stat_keys`）
- Modify: `D:\omoba\omb\src\bin\gen_docs_lib\model.rs:99-115`（`StatKey` 結構加 `aggregation` 欄）
- Modify: `D:\omoba\omb\src\bin\gen_docs_lib\render.rs:279-307`（表格加一欄）

**Step 1: `api_scan.rs` — 改用 runtime call**

舊版用 regex + syn 從 const 宣告抽資料。新版：link script-abi，直接呼叫 method。但是沒有 `EnumIter`，所以要手維護一份 ALL array — 在 `scripts/script-abi/src/stat_keys.rs` 新增 `pub const ALL: &[StatKey] = &[...]`（把 Task 5 tests 裡的 `ALL` 移到 public 位置）。

```rust
// api_scan.rs
pub fn scan_stat_keys() -> Vec<StatKey> {
    omb_script_abi::stat_keys::ALL.iter().map(|&v| StatKey {
        const_name: format!("{:?}", v), // PascalCase 變體名
        string_value: v.as_str().to_string(),
        section: v.section(),
        aggregation: v.aggregation(),
        doc: String::new(), // 若要保留 doc，在 stat_keys.rs 放一個 doc() method，用 match 回傳
    }).collect()
}
```

如果要保留 `/// doc` 文字，多加 `pub fn doc(self) -> &'static str` 同 match（gen_stat_keys.py 產生時把 source 行上方的 `///` 註解抓下來當對應 variant 的 doc）。

**Step 2: `model.rs` — 結構多一個欄位**

```rust
pub struct StatKey {
    pub const_name: String,
    pub string_value: String,
    pub doc: String,
    pub section: StatSection,
    pub aggregation: Aggregation,  // 新
}
```
(這 `StatKey` 是 gen-docs 內部 model，和 script-abi 的 StatKey 同名但不同型別；如要避免衝突，rename 為 `StatKeyDoc`。)

**Step 3: `render.rs` — HTML 表格多一欄**

```rust
thead { tr { th{"variant"} th{"string"} th{"section"} th{"aggregation"} th{"doc"} } }
tbody {
    @for s in ... {
        tr {
            td { code { (s.const_name) } }
            td { code { "\"" (s.string_value) "\"" } }
            td { (format!("{:?}", s.section)) }
            td { (format!("{:?}", s.aggregation)) }   // 新
            td { (s.doc) }
        }
    }
}
```

**Step 4: 跑 gen-docs 產出 HTML**

```bash
cd D:/omoba/omb
cargo run -p omobab --bin gen-docs --features gen-docs --release
# 開 D:/omoba/omb/target/docs/index.html 肉眼檢查 165 個 stat key 的 aggregation 欄
```

**Step 5: Commit**

```bash
git add -A
git commit -m "feat(gen-docs): render StatKey enum with aggregation column

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 12: 遷移 scripts/base_content towers

**Files:**
- Modify: `D:\omoba\scripts\base_content\src\towers\dart.rs:158,171`
- Modify: `D:\omoba\scripts\base_content\src\towers\ice.rs:83,90,93`
- Modify: `D:\omoba\scripts\base_content\src\towers\bomb.rs:80`

**Step 1: 改 dart.rs**

```rust
use omb_script_abi::stat_keys::StatKey;

// 舊：w.get_stat_bonus(attacker, RStr::from_str("crit_chance"))
// 新：
w.get_stat_bonus(attacker, StatKey::CritChance)
// 同理 crit_bonus → StatKey::CritBonus
```

**Step 2: 改 ice.rs 3 處**

```rust
w.get_stat_bonus(e, StatKey::SlowFactorOverride)
w.get_stat_bonus(e, StatKey::SlowDurationBonus)
w.get_stat_bonus(e, StatKey::SplashBonus)
```

**Step 3: 改 bomb.rs 1 處**

```rust
w.get_stat_bonus(e, StatKey::SplashBonus)
```

**Step 4: cargo build base_content**

```bash
cd D:/omoba/scripts && cargo build -p base_content
```
Expected: PASS。

**Step 5: Commit**

```bash
cd D:/omoba
git add scripts/base_content/src/towers/
git commit -m "refactor(towers): use StatKey enum for get_stat_bonus calls

Replaces 6 hardcoded string keys (crit_chance, crit_bonus, splash_bonus,
slow_factor_override, slow_duration_bonus).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 13: 遷移 scripts/base_content heroes

**Files:**
- Modify: `D:\omoba\scripts\base_content\src\heroes\B02_date_masamune\No4_matchlock_gun.rs:57-58, 101-102`
- Modify: 其他 `Grep "sk::" scripts/base_content/src/heroes/` 命中的檔案（No1_sniper_mode 等）
- Modify: 同理 `Grep "modifiers.insert" scripts/base_content/src/` 的檔案

**Step 1: 改 matchlock_gun 非標準 key**

```rust
// 舊 line 57：modifiers.insert("attack_stun_chance".into(), ...)
// 新：
modifiers.insert(StatKey::AttackStunChance.as_str().into(), serde_json::json!(get_f("stun_chance")));
modifiers.insert(StatKey::AttackStunDuration.as_str().into(), serde_json::json!(get_f("stun_duration")));
// preview_mods line 101-102 同理
```

**Step 2: 批次改既有 `sk::XXX.into()` → `StatKey::Xxx.as_str().into()`**

用跟 Task 9 類似的 sed 腳本，但這次 target 是 scripts，且要把 `sk::PREATTACK_BONUS_DAMAGE.into()` 轉 `StatKey::PreattackBonusDamage.as_str().into()`（因為 modifiers map 的 key 是 `String`，必須走 `as_str()`）。

**Step 3: cargo build**

```bash
cd D:/omoba/scripts && cargo build -p base_content
```
Expected: PASS。

**Step 4: Commit**

```bash
cd D:/omoba
git add scripts/base_content/src/heroes/
git commit -m "$(cat <<'EOF'
refactor(heroes): use StatKey enum in modifier payload keys

- matchlock_gun attack_stun_chance/duration 改走 enum
- sniper_mode/saika/date 其餘 modifiers.insert 用 StatKey::X.as_str()

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 14: 全鏈路建置 + in-game 驗證

**Step 1: 清乾淨後重 build 兩個 workspace**

```bash
cd D:/omoba/scripts && cargo build -p base_content --release
cp target/release/base_content.dll ../omb/scripts/base_content.dll

cd D:/omoba/omb && cargo build -p omobab --release
```
Expected: 兩個都 PASS，`omb/scripts/base_content.dll` 更新時間是剛剛。

**Step 2: gen-docs smoke test**

```bash
cd D:/omoba/omb
cargo test -p omobab --features gen-docs -- --ignored
# 若 CLAUDE.md 記的指令有效
```

**Step 3: 開 HTML 肉眼檢查**

開 `D:/omoba/omb/target/docs/index.html`：
- Stat Keys section 有 165 項（Section 1 + 2 + 3 總和）
- 每項 aggregation 欄有值
- 7 個新 variant（CritChance 等）在 Section 1

**Step 4: run.bat 啟動 MVP_1**

```bash
cd D:/omoba
./run.bat
# 前端起來後選 date_masamune，放 No4 火繩銃
```
- 觀察：敵人 debuff 上有 stun（來自 `AttackStunChance/Duration` 透過新 enum 寫入 BuffStore）
- 觀察：英雄 `hero.stats` 廣播 payload 欄位名仍然是舊字串（因為 payload JSON 的 field name 是 `StatKey::X.as_str()` 產生）

**Step 5: run_stress.bat 跑壓測**

```bash
./run_stress.bat
# 30 秒後按 Esc 結束
```
- 觀察：ice tower 的 slow debuff 仍然生效於 creep
- 觀察：dart tower 升級 crit 仍然生效（看實際傷害數字偶爾高於基礎）
- 控制台無 panic / serde 警告

**Step 6: 最終 Commit（submodule bump）**

```bash
cd D:/omoba
git add omb scripts
git commit -m "$(cat <<'EOF'
feat(stat-key): full enum migration end-to-end

- omb submodule bump with GameWorld StatKey API + gen-docs update
- scripts/script-abi exports 165-variant enum
- scripts/base_content fully converted

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## 驗收

- [ ] `scripts/script-abi/` 位於新位置、有 `license` 欄位
- [ ] `cargo test -p omb-script-abi --lib stat_keys` 全過（含 `all_variants_roundtrip`，`ALL.len() == 165`）
- [ ] `cargo build` on both workspaces PASS（release）
- [ ] gen-docs HTML 列出 165 個 stat key 含 aggregation 欄
- [ ] MVP_1 / TD_STRESS in-game 不 crash，stun / slow / crit 實際生效
- [ ] `scripts/base_content` 裡 `grep -rn 'RStr::from_str("' src/` 無剩餘 stat key 字串字面值（extra/buff_id 類非 stat key 不動）
- [ ] `omb` 裡 `grep -rn 'sk::' src/` 無殘留（或只剩 import alias）

## 風險對照

| 風險（計畫中記錄） | 緩解 | Task |
|---|---|---|
| abi_stable sabi_trait 對 bare enum 參數支援？ | Task 7 完成即驗證；若爆錯用 `#[repr(C)] struct StatKeyArg { key: StatKey }` 包一層 | 7 |
| Variant 順序 = ABI 契約，誤改 = UB | 顯式 `= 0, 1, ...` discriminant + `stat_keys.rs` 檔頭 SAFETY 註解 | 5 |
| 165 variant 手抖寫錯 | `gen_stat_keys.py` 從舊 const 自動產；ALL round-trip test 鎖 consistency | 5 |
| host transfer 中間 break 難 bisect | 每 task 獨立 commit，bisect 容易 | 全 |
| hero.stats 廣播 payload 欄位名變了會破壞 omfx 前端 | 刻意保留 `as_str()` 原字串不變，只改 trait 層；Task 14 step 4 in-game 驗證 | 14 |
