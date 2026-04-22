# Ability System 腳本化整合設計

**日期：** 2026-04-22
**狀態：** 設計已批准，Phase 1 待實作
**相關文件：** `2026-04-22-native-script-abi-design.md`（native script ABI PoC-1）

## Context

專案目前有**三套平行的「能力/技能/效果」實作**，加上剛落地的 native script ABI，合起來有冗餘也有錯位：

1. **`omb/ability-system` sub-crate**（~1748 行）— 當初為了讓 client 能在本地計算而拆出。有 `AbilityHandler` trait（Rust 原生、非 FFI）、`AbilityRegistry`、`AbilityProcessor`、`EffectManager`、`AbilityEffect` enum。硬編碼 8 個英雄技能（Saika×4、Date×4）在 `src/heroes/`。
2. **`omb/src/comp/ + tick/skill_system/`** — 主 crate 的 Skill/SkillEffect/SkillInput/SkillState，其中 `skill_system/abilities.rs` 硬編碼重複了 sub-crate 的 4 個 Saika 技能。
3. **TD 塔**（`tower_template.rs::SlowBuff` + `projectile_tick` + `slow_buff_tick`）— 完全不走技能系統，自建一套 `Outcome::ApplySlow` → `SlowBuff` component。

加上 **`omb/script-abi`** + **`scripts/base_content`** 這條 abi_stable DLL 腳本路徑（已跑通 DartTower PoC），但目前只有 `UnitScript` sabi_trait 的 6 個 hook，**沒有 `AbilityScript` FFI trait**，所以 `ability-system/heroes/` 的 8 個 handler 雖然邏輯位置正確（屬於「內容」），但物理位置錯（躺在 framework crate 裡）。

**整合目標**：
- **`scripts/base_content` 成為所有「內容（content）」的唯一家**：8 個英雄技能 + 4 個塔（Dart 已有 + 新增 Bomb/Tack/Ice）全部作為 DLL 腳本
- **`omb/ability-system` sub-crate 解散**：runtime infrastructure 搬回 `omb` 主 crate、資料型別搬到 `omoba-core`、handler 搬去 `scripts/base_content`
- **`omb/script-abi` 擴充 `AbilityScript` sabi_trait**：讓技能邏輯能跨 DLL 邊界
- **Client 只讀 metadata**：透過 KCP/gRPC 取得 `AbilityDef` 用於 tooltip，不載入 DLL、不跑 handler
- **消滅主 crate 三套重複實作**，Effect 統一為 `ability-system` 的 `ActiveEffect` / `EffectManager`

## Target Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ omoba-core（client + server 共享）                           │
│   + 新模組 ability_meta::{AbilityDef, EffectSpec,           │
│                            TargetType, CostSpec, LevelData} │
│   純資料 schema，Serde + prost 皆可                          │
└─────────────────────────────────────────────────────────────┘
                        ▲                       ▲
                        │                       │
 ┌──────────────────────┴──────────┐   ┌────────┴────────────┐
 │ omb/script-abi（擴充）           │   │ omfx / omb-mcp      │
 │  + #[sabi_trait] AbilityScript  │   │ 只讀 AbilityDef     │
 │    (execute, on_tick)           │   │ 作為 UI tooltip     │
 │  + Manifest::abilities()        │   └─────────────────────┘
 │    回傳 RVec<AbilityDef_FFI>    │
 │  既有 UnitScript / 6 hooks 不動 │
 └─────────────────────────────────┘
                        ▲
                        │
 ┌──────────────────────┴──────────────────────────────────────┐
 │ omb 主 crate（runtime framework）                            │
 │  - src/ability/ (新) ← 從 ability-system 搬回的 runtime：   │
 │      · registry.rs (AbilityRegistry)                         │
 │      · processor.rs (AbilityProcessor)                       │
 │      · effects.rs (EffectManager, ActiveEffect) 唯一一套    │
 │      · types.rs (AbilityRequest/Result, WorldAccess)         │
 │  - comp/ability_comp.rs（ECS 唯一 binding）                  │
 │  - tick/skill_tick.rs（驅動 Processor + EffectManager）      │
 │  - grpc/kcp server：新增 ListAbilities / GetAbilityDetail    │
 │                                                              │
 │  刪除：                                                       │
 │  · comp/skill.rs, comp/ability.rs                            │
 │  · tick/skill_system/abilities.rs                            │
 │  · tick/skill_system/effects.rs                              │
 │  · tick/slow_buff_tick.rs                                    │
 │  · comp/tower_template.rs::SlowBuff                          │
 └──────────────────────────────────────────────────────────────┘
                        ▲
                        │ abi_stable load_from_file
 ┌──────────────────────┴──────────────────────────────────────┐
 │ scripts/base_content（cdylib，所有 content）                 │
 │  - src/heroes/B01_saika_magoichi/ (4 檔，從 ability-system   │
 │      搬來並改為 impl AbilityScript)                          │
 │  - src/heroes/B02_date_masamune/  (4 檔，同上)               │
 │  - src/towers/dart.rs（既有 DartTower，保留 UnitScript）     │
 │  - src/towers/{bomb, tack, ice}.rs（新增）                   │
 │  - src/lib.rs 的 Manifest.abilities() 列出所有技能           │
 └──────────────────────────────────────────────────────────────┘

整份刪除：omb/ability-system/（sub-crate 解散）
```

## Phased Plan

### Phase 1：建立 metadata 共享層 + 擴充 script-abi

**1a. `omoba-core/src/ability_meta.rs`（新）**

從 `omb/ability-system/src/ability.rs` 移出資料結構並純化（去掉執行期狀態）：

```rust
pub struct AbilityDef {
    pub id: String,
    pub name: String, pub description: String,
    pub ability_type: AbilityType,
    pub target_type: TargetType,
    pub icon: Option<String>,
    pub max_level: u8,
    pub levels: Vec<AbilityLevelData>,
    pub effects_preview: Vec<EffectSpec>,
    pub conditions: Vec<Condition>,
}
```

Serde + `prost-build` 雙支援（feature flag 切）。放 `omoba-core` 讓 client/server 共享。

**1b. `omb/script-abi` 擴充**

新增 `src/ability.rs`：

```rust
#[sabi_trait]
pub trait AbilityScript: Send + Sync {
    fn execute(&self, request: AbilityRequestFFI, world: &mut GameWorldDyn)
        -> RVec<AbilityEffectFFI>;
    fn on_tick(&self, state: ActiveEffectStateFFI, world: &mut GameWorldDyn)
        -> RVec<AbilityEffectFFI>;
}

pub struct Manifest {
    pub units: extern "C" fn() -> RVec<UnitDef>,
    pub abilities: extern "C" fn() -> RVec<AbilityDef_FFI>,
}

pub struct AbilityDef_FFI {
    pub def: AbilityDef_Serialized,
    pub script: AbilityScript_TO<'static, RBox<()>>,
}
```

`AbilityDef` 跨 FFI 用序列化字串傳遞（避免 StableAbi 大量衍生），DLL 端 deserialize 回 Rust-native 結構。

**1c. Client metadata 查詢**

- `proto/game.proto` 新增 `ListAbilitiesRequest/Response`、`GetAbilityDetailRequest/Response`
- KCP tags 0x07（ListReq）/ 0x08（ListResp）/ 0x09（DetailReq）/ 0x0A（DetailResp）
- `omb/src/grpc/server.rs`、`omb/src/kcp/server.rs` 讀 `AbilityRegistry` metadata map，序列化 `AbilityDef` 回傳

**驗收**：三種 feature 組合 build 全過；`omb-mcp` 能查到空 list（DLL 尚未註冊 ability）。

---

### Phase 2：heroes handler 搬去 `scripts/base_content`

**搬遷對象**（`omb/ability-system/src/heroes/` → `scripts/base_content/src/heroes/`）：

```
B01_saika_magoichi/ (4 檔)     B02_date_masamune/ (4 檔)
├── No1_sniper_mode.rs         ├── No1_*.rs
├── No2_saika_reinforcements   ├── No2_*.rs
├── No3_rain_iron_cannon       ├── No3_*.rs
└── No4_three_stage_technique  └── No4_*.rs
```

**每個檔改造**：
- `impl AbilityHandler for X` → `impl AbilityScript for X`
- 對 ECS 的呼叫改走 `GameWorldDyn`
- `AbilityLevelData.extra: HashMap<String, Value>` 讀取邏輯不變

**`scripts/base_content/src/lib.rs` 擴充**：

```rust
extern "C" fn abilities() -> RVec<AbilityDef_FFI> {
    let mut v = RVec::new();
    v.push(make_ability("B01_sniper_mode", SniperModeHandler));
    // ... 8 個
    v
}
```

**配置**：`ability-configs/sniper_abilities.json` 移到 `scripts/base_content/assets/`（DLL 自包含）。

**主 crate 連接**：`omb/src/ability/registry.rs` 啟動時遍歷 `ScriptRegistry._manifests`，呼叫 `abilities()` 拿到 `AbilityDef_FFI`，存 `HashMap<id, (AbilityDef, AbilityScript_TO)>`。

**驗收**：`cargo test -p base_content` 通過；MOBA 關卡 Saika 4 技能功能與遷移前一致。

---

### Phase 3：TD 塔全進 `scripts/base_content`

**新增**：`scripts/base_content/src/towers/{bomb, tack, ice}.rs`

| 塔 | 形式 | 理由 |
|---|---|---|
| Dart | 既有 `UnitScript::on_attack_hit`（不動） | 單目標直接傷害 |
| Bomb | `on_attack_hit` + world.apply_aoe | 範圍傷害 |
| Tack | `on_attack_hit` + 迴圈 8 方向 spawn_projectile | 放射 |
| Ice | `on_attack_hit` + world.apply_buff(slow, duration) | 減速 buff |

**主 crate 改動**：
- `comp/tower_template.rs::SlowBuff` — 刪除
- `tick/slow_buff_tick.rs` — 整份刪除
- `tick/projectile_tick.rs` — 命中改發 `ScriptEvent::AttackHit`，不產 `Outcome::ApplySlow`
- `EffectManager` 接手 buff 倒數（受 `ApplyBuff` outcome 驅動）

**驗收**：TD 模式 4 塔全功能正確；`grep -r "SlowBuff\|slow_buff_tick\|ApplySlow" omb/src` 歸零。

---

### Phase 4：解散 `omb/ability-system` sub-crate

**搬遷對照表**：

| ability-system 原檔 | 新位置 |
|---|---|
| `src/ability.rs` (資料) | → `omoba-core/src/ability_meta.rs`（Phase 1 已部分完成） |
| `src/handler.rs` | → `omb/src/ability/registry.rs`；AbilityHandler 被 AbilityScript FFI trait 取代 |
| `src/processor.rs` + `AbilityProcessor` | → `omb/src/ability/processor.rs` |
| `src/effects.rs` | → `omb/src/ability/effects.rs`（唯一一套） |
| `src/types.rs` | → 請求/結果 → `omoba-core/ability_meta`；WorldAccess 併入 `script-abi/world.rs::GameWorld` |
| `src/config.rs` | → `omb/src/ability/config.rs` |
| `src/heroes/` | → Phase 2 已搬到 `scripts/base_content/src/heroes/` |
| `tests/integration_test.rs` | → 拆分到 `omoba-core/tests/`、`omb/tests/`、`scripts/base_content/tests/` |

**同時刪除主 crate 重複**：
- `omb/src/comp/skill.rs`
- `omb/src/comp/ability.rs`
- `omb/src/tick/skill_system/abilities.rs`
- `omb/src/tick/skill_system/effects.rs`

**改寫**：
- `comp/ability_comp.rs` — import 改 `crate::ability::`
- `tick/skill_tick.rs` — 同上
- `tick/skill_system/processor.rs` — 瘦身為 AbilityEffect → Outcome bridge
- `omb/Cargo.toml` — 移除 `ability-system` 依賴、workspace member

**物理刪除**：`omb/ability-system/` 整個目錄

**驗收**：`grep -r "ability_system\|ability-system" .` 除 git history 外歸零；`cargo build --workspace`、`cargo test --workspace` 全過；MOBA + TD 手動跑一遍。

---

### Phase 5：收尾

- 更新 `docs/plans/2026-04-22-native-script-abi-design.md` 標註 PoC-2 完成
- `run.bat` / `run.sh` 加 `cargo build -p base_content`
- omfx 端保留 TODO：client 呼叫 `ListAbilities` 填 tooltip 表
- 更新 `MEMORY.md`：新增 ability_script_ffi 條目

## Critical Files

**新增**：
- `omoba-core/src/ability_meta.rs`
- `omb/script-abi/src/ability.rs`（AbilityScript sabi_trait + AbilityDef_FFI）
- `omb/src/ability/{registry, processor, effects, types, config}.rs`
- `scripts/base_content/src/heroes/{B01_saika, B02_date}/...`
- `scripts/base_content/src/towers/{bomb, tack, ice}.rs`
- `omb/proto/game.proto`（ListAbilities/GetAbilityDetail）

**整份刪除**：
- `omb/ability-system/`
- `omb/src/comp/{skill, ability}.rs`
- `omb/src/tick/skill_system/{abilities, effects}.rs`
- `omb/src/tick/slow_buff_tick.rs`
- `comp/tower_template.rs::SlowBuff`

**重寫**：
- `omb/script-abi/src/manifest.rs`
- `omb/src/scripting/registry.rs`
- `omb/src/comp/ability_comp.rs`
- `omb/src/tick/skill_tick.rs`
- `omb/src/tick/projectile_tick.rs`
- `scripts/base_content/src/lib.rs`
- `omb/src/grpc/server.rs`、`omb/src/kcp/server.rs`

## Reuse Inventory

- `omb/ability-system/src/effects.rs::EffectManager` → 搬 `omb/src/ability/effects.rs`
- `omb/ability-system/src/types.rs::WorldAccess` trait → 併入 `script-abi/world.rs::GameWorld`
- `omb/src/json_preprocessor.rs` → Phase 2 讀 configs 沿用
- `omb/src/scripting/{loader, registry, dispatch, world_adapter}.rs` → DLL 管線完整
- `omb/script-abi/src/types.rs` → EntityHandle/Vec2f/DamageKind 已備
- `omoba-core/src/{grpc, kcp}/*` → metadata query 擴充 proto

## Verification

**每 Phase 收尾**：
1. `cargo build --features mqtt`、`--no-default-features --features kcp`、`--no-default-features --features grpc` 三種組合通過
2. `cargo test --workspace` 全綠
3. `run.bat` 啟動 KCP，`omb-mcp` 的 `inspect_player_view` + `list_players` 確認：
   - `STORY = "MVP_1"`：Saika 4 技能正常
   - `STORY = "TD_1"`：4 塔射擊、範圍傷害、減速正常
4. Phase 4 完後 `grep -r "ability_system\|comp::skill::\|SlowBuff\|slow_buff_tick\|ApplySlow" .` 歸零
5. Client metadata 查詢：`omb-mcp` 測 command 呼叫 `ListAbilities`，確認 8 英雄 + 4 塔 = 12 筆

**回歸保護**：每 Phase 獨立 branch + PR。Phase 4 最危險，Phase 1-3 綠燈後才動。

## Risks

- **`AbilityDef` 跨 FFI**：含 `HashMap<String, serde_json::Value>` 對 `StableAbi` derive 不友善。採序列化（JSON 字串或 bincode bytes）跨邊界
- **rust-toolchain 鎖定**：`abi_stable` 要求 host 與 DLL 同 rustc 版本；`scripts/base_content` 必須跟 `omb` 一致
- **Phase 2 → 3 中間態**：sub-crate 仍活著時編譯可能失敗；建議 Phase 2 後立即做 4a（先解散 sub-crate，handler 放主 crate 暫存），再進 3
- **Outcome queue 訂閱順序**：Phase 3 `projectile_tick` 改發 AttackHit event，dispatch tick 呼叫時機要早於傷害結算
- **Client schema 版本化**：Phase 1 新增 KCP tags 0x07~0x0A 不與既有 0x01~0x06 衝突
- **mqtt feature 組合**：proto 變更時確認 `--features mqtt` 也跟得上，或明確 metadata query 僅 grpc/kcp 支援
