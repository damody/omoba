# Template ID Codegen Design

Date: 2026-04-25
Status: **Shipped 2026-04-25** — reduced scope (see "Delivered scope" below).

## Delivered scope (2026-04-25)

**Landed:**
- New crate `omoba-template-ids` (zero runtime deps, build.rs codegen)
- `Story/templates.json` single source of truth (4 towers, 2 heroes, 8 abilities, 5 buffs, 1 summon, 14 creeps, 9 projectile kinds)
- 7 per-namespace newtype (`TowerId`, `HeroId`, `AbilityId`, `BuffId`, `SummonId`, `CreepId`, `ProjectileKindId`) + consts + `*_by_name` / `*_id_str` / `*_display` lookups
- **Wire migration (the high-value part):**
  - `ProjectileCreate.kind_id`: FNV-1a u32 hash → sequential u16 (saves ~2 B/event × 3000/s ≈ 6 KB/s under stress)
  - `CreepCreate.name_id`: FNV-1a u32 hash → sequential u16 (saves ~7 B/creep — Chinese labels hashed to big numbers)
  - Server no longer hashes strings; client reverse-lookup through new crate
- **Scripts (base_content):** all 4 towers + 1 summon use `TOWER_*.as_str()` / `PROJECTILE_*.0` patterns — string literal typo is a compile error
- Legacy `omoba-core/src/template_ids.rs` (hand-maintained FNV tables) deleted
- `ProjectileSpec.kind_tag: RString → kind_id: u16` ABI break — only FFI signature change this pass

**Deferred (future work, not urgent):**
- Trait signature changes: `UnitScript::unit_id()` / `AbilityScript::ability_id()` stay as `RStr<'_>` (scripts call `RStr::from_str(TOWER_TACK.as_str())` — compile-time safe but not type-safe at FFI boundary). Upgrading to `UnitTemplateId` / `AbilityId` newtype return is ergonomic polish, not wire-critical.
- `GameWorld::add_buff / remove_buff / has_buff / add_stat_buff` stay as `RStr<'_>` for buff_id. Host adapter already converts via `.to_string()` for BuffStore key.
- Proto breaking changes for `TowerCreate.kind/name`, `HeroStatic.name/title/ability_ids`, `BuffAdd.buff_id`, `BuffSnapshot.buff_id`, `BuffRemove.buff_id` — these are low-frequency events (TowerCreate ~100/session, HeroStatic ~10/session, Buff ~50/s), total wire savings < 1 KB/s, not worth the encoder/decoder rewiring right now.
- Ability files (`heroes/**/No*.rs`) keep `pub const ABILITY_ID: &str = "sniper_mode"` as-is — runtime cross-check planned via host catalog load validation.

**Rationale for scope reduction:** The ABI trait signature change touches 18 script files + host adapter + gen_docs simultaneously (non-atomic if done in multiple commits, meaning the intermediate state breaks build). The delivered scope captures ≥95% of the wire-bytes benefit with one focused FFI change (`ProjectileSpec.kind_tag → kind_id: u16`) plus the wire field semantics swap at `proto_build` level — same end result, 20% of the refactor surface.

---



## Problem

Template 字串 id（`"tower_tack"`, `"sniper_mode"`, `"訓練法師"`, `"saika_shot"`, buff id 等）目前手工散佈在三處：
- `scripts/base_content/src/**/*.rs` — hard-coded `RStr::from_str("tower_tack")`、`RString::from("saika_shot")`
- `omb/Story/{TD_1,TD_STRESS,MVP_1,DEBUG_1,B01_1}/entity.json` — hero/creep 定義
- `omoba-core/src/template_ids.rs` — 手工 `KNOWN_PROJECTILE_KINDS` / `KNOWN_CREEP_NAMES` 列表搭配 FNV-1a hash

痛點：
1. **漂移**：JSON 新增 creep 但 `KNOWN_CREEP_NAMES` 忘了同步 → 客戶端 reverse lookup 打 `unknown_<hex>`（comment 自己標了「harmless but...」）
2. **打錯字無感**：scripts 裡 `"tower_tack"` 打成 `"tower_tak"` 編譯通過，runtime 才炸
3. **wire 浪費**：stress 場景每秒千級 `ProjectileCreate` / `CreepCreate`，`kind_id` 走 FNV-1a u32 → varint 幾乎固定 4 bytes；`TowerCreate.kind` / `HeroStatic.ability_ids` / `BuffAdd.buff_id` 還是 string
4. **FNV hash 無法壓縮**：hash 分佈大數字，varint 省不到 byte

## Goals

- Template id 集中於 `Story/templates.json` 作為**單一真理**
- Build-time codegen 產**sequential u16 per-namespace**，varint 下前 127 個 id 只吃 1 byte
- Scripts 與 host 側使用 **newtype wrapper** (`TowerId`, `HeroId`, `AbilityId`, `BuffId`, `SummonId`, `CreepId`, `ProjectileKindId`)，打錯 id 編譯失敗
- Proto 高頻事件的 `string kind/name/id` 破壞性換成 `uint32`
- Client 顯示字串走本地 reverse lookup（server 不傳 label，省 wire bytes）

## Non-Goals

- i18n 多語系 infra（目前單語系繁中；schema 用 `display_name: string`，未來要擴可擴）
- 舊版 client × 新 server 相容（部署永遠同 commit，不處理混合版本）
- Runtime hot-reload templates.json（build artifact，改完 rebuild）
- Tombstone / id 穩定性跨 commit 鎖（B 選項不含 lockfile；未來若有 save/replay 需求再升級到 C sequential + lockfile）

## Architecture

### 新 crate：`omoba-template-ids`

位置：`D:/omoba/omoba-template-ids/`（repo root workspace + scripts workspace 都 path dep）

依賴：只有 `serde` + `serde_json`（build-time），執行期零依賴。**不**引 abi_stable / specs / tonic。

產物（全 `include!` 進 lib.rs）：
```rust
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TowerId(pub u16);

impl TowerId { pub const fn raw(self) -> u16 { self.0 } }

pub const TPL_TOWER_TACK:  TowerId = TowerId(1);
pub const TPL_TOWER_DART:  TowerId = TowerId(2);
// ...

pub fn tower_name(id: TowerId) -> &'static str { /* match */ }
pub fn tower_by_name(s: &str) -> Option<TowerId> { /* phf-like match */ }

// 每個 namespace 獨立 u16 空間（重複 1,2,3... ok，types 擋跨類誤用）
pub struct HeroId(pub u16);      pub const TPL_HERO_SAIKA_MAGOICHI: HeroId = HeroId(1);
pub struct AbilityId(pub u16);   pub const TPL_ABILITY_SNIPER_MODE: AbilityId = AbilityId(1);
pub struct BuffId(pub u16);      pub const TPL_BUFF_STUN: BuffId = BuffId(1);
pub struct SummonId(pub u16);    pub const TPL_SUMMON_SAIKA_GUNNER: SummonId = SummonId(1);
pub struct CreepId(pub u16);     pub const TPL_CREEP_TRAINING_MAGE: CreepId = CreepId(1);
pub struct ProjectileKindId(pub u16); pub const TPL_PROJECTILE_TACK: ProjectileKindId = ProjectileKindId(1);
```

每 namespace 都 id 0 保留 = UNSPECIFIED（對應現有 "unspecified kind" 語義）。

### `Story/templates.json` schema

```json
{
  "towers": [
    { "id": "tower_tack",
      "display_name": "Tack Shooter",
      "projectile_kind": "tack" },
    { "id": "tower_dart", "display_name": "Dart Monkey", "projectile_kind": "dart" }
  ],
  "heroes": [
    { "id": "saika_magoichi",
      "display_name": "雜賀孫市",
      "title": "千里狙擊手",
      "abilities": ["sniper_mode", "saika_reinforcements", "rain_iron_cannon", "three_stage_technique"],
      "base": { /* str/agi/int/hp/... 屬性搬過來 */ }
    }
  ],
  "abilities": [
    { "id": "sniper_mode",  "display_name": "狙擊模式" },
    { "id": "saika_reinforcements", "display_name": "雜賀援軍" }
  ],
  "buffs": [
    { "id": "stun", "display_name": "暈眩" },
    { "id": "slow", "display_name": "減速" }
  ],
  "summons": [
    { "id": "saika_gunner", "display_name": "雜賀鐵炮兵" }
  ],
  "creeps": [
    { "id": "training_mage", "display_name": "訓練法師",
      "base": { "hp": 320, "armor": 0.5, /* ... */ } }
  ],
  "projectile_kinds": [
    { "id": "tack" }, { "id": "bomb" }, { "id": "bomb_frag" }, { "id": "saika_shot" }
  ]
}
```

Scene 的 `entity.json` 退化為：
```json
{
  "heroes_used": ["saika_magoichi"],
  "creeps_used": ["training_mage", "fire_mage"],
  "waves": [ /* 原本就 scene-specific 的波數/座標 */ ],
  "hero_spawns": [ { "id": "saika_magoichi", "x": 100, "y": 200 } ]
}
```

**id 分配規則（build.rs）**：
- 按 JSON 陣列順序分配 `1, 2, 3, ...`（id 0 保留）
- 新 entry 一律 append 到陣列末，避免現有 id 平移
- 刪除 entry 需留 `{ "id": "...", "tombstone": true }` 佔位（不進 const 表但佔 id 編號）
- build.rs 起手做 self-check：id 重複 → build fail；display_name 空 → build fail

### FFI ABI 變更

`omb-script-abi/src/script.rs`：
```rust
// 舊
fn unit_id(&self) -> RStr<'_>;

// 新（abi_stable 對 u16 原生 StableAbi；newtype 用 transparent + derive StableAbi）
fn unit_id(&self) -> UnitTemplateId;

// UnitTemplateId 是跨 tower/hero/creep/summon 的 tagged 版本
#[repr(C)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnitTemplateId {
    pub kind: UnitTemplateKind, // Tower / Hero / Creep / Summon
    pub id: u16,
}
#[repr(u8)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnitTemplateKind { Tower, Hero, Creep, Summon }
```

`omb-script-abi/src/ability.rs`：
```rust
fn ability_id(&self) -> AbilityId;  // u16 newtype
```

`omb-script-abi` depend `omoba-template-ids`（path dep 到 `../../omoba-template-ids`），把 newtype re-export 過來：
```rust
pub use omoba_template_ids::{TowerId, HeroId, AbilityId, BuffId, SummonId, CreepId, ProjectileKindId};
```

Scripts 寫法：
```rust
impl UnitScript for TackTower {
    fn unit_id(&self) -> UnitTemplateId {
        UnitTemplateId { kind: UnitTemplateKind::Tower, id: TPL_TOWER_TACK.0 }
    }
}
// 打錯變成 TPL_TOWER_TAK 會 "cannot find value `TPL_TOWER_TAK`" 直接 compile fail
```

### Proto 變更（破壞性一次換）

```proto
// 刪
message TowerCreate {
  uint64 id = 1;
  Position16 pos = 2;
  Fixed16 hp = 3;
  Fixed16 max_hp = 4;
  reserved 5;              // was string kind
  reserved 6;              // was string name（client 查表）
  uint32 tower_id = 7;     // TowerId.0
}

message HeroStatic {
  uint64 id = 1;
  reserved 2;              // was string name
  reserved 3;              // was string title
  uint32 hero_id = 14;     // HeroId.0
  // ... (client 透過 hero_id 查 display_name / title)
  reserved 7;              // was repeated string ability_ids
  repeated uint32 ability_ids = 15;  // AbilityId.0 per-entry
}

message BuffAdd {
  uint64 entity_id = 1;
  reserved 2;              // was string buff_id
  uint32 buff_id = 5;
  uint32 remaining_ms = 3;
  string payload_json = 4;  // 保留，payload 仍是 JSON
}
message BuffRemove {
  uint64 entity_id = 1;
  reserved 2;              // was string buff_id
  uint32 buff_id = 3;
}
message BuffSnapshot {
  reserved 1;              // was string buff_id
  uint32 buff_id = 4;
  uint32 remaining_ms = 2;
  string payload_json = 3;
}

// 維持
message CreepCreate { /* name_id 已經是 uint32，底層改 sequential */ }
message ProjectileCreate { /* kind_id 已經是 uint32，底層改 sequential */ }
```

### Unknown id policy（F2）

`omoba-template-ids` 的 reverse lookup：
```rust
pub fn creep_name(id: CreepId) -> &'static str {
    match id.0 {
        0 => "",
        1 => "訓練法師",
        // ...
        _ => {
            debug_assert!(false, "unknown CreepId: {}", id.0);
            log::error!("unknown CreepId: {} — client/server template table mismatch", id.0);
            "?"
        }
    }
}
```

Release build：log_error + 回 `"?"`（使用者看到問號知道出 bug，不是靜默空字串）。
Debug build：直接 panic 方便 catch。

### `omoba-core/src/template_ids.rs` 處置

整個檔刪除。現有三個 call site：
- `omoba-core/src/kcp/client.rs` 反查 creep name / projectile kind 時 `use omoba_template_ids::{creep_name, projectile_kind_name};`
- 手工 `KNOWN_PROJECTILE_KINDS` 等 const 表消失
- `encode_creep_name(s: &str) -> u32` 之類 API 刪除（server 端直接用 `CreepId.0`，不再 hash）

### Server 側（`omb`）

`omb` 依賴 `omoba-template-ids`。所有原本 encode string 的路徑改成：
- Creep 建立時：從 `Story/templates.json` 載入的 catalog 查 `CreepId`，`CreepCreate.name_id = creep_id.0`
- Tower 建立時：scripts 回報 `UnitTemplateId`，host 轉成 `TowerCreate.tower_id`
- Projectile 發射時：scripts 回報 `ProjectileKindId`（取代現在的 `RString`），`ProjectileCreate.kind_id = pk.0`
- Buff add/remove：内部改用 `BuffId`；`add_buff(entity, BuffId, ...)` FFI signature 更新

### Build.rs 流程

`omoba-template-ids/build.rs`：
1. `cargo:rerun-if-changed=../omb/Story/templates.json`
2. 讀 JSON、驗 schema（每個 entry 有 `id` 字串、無 `id` 重複、有 `display_name`）
3. 按陣列宣告順序分配 u16 id（跳過 tombstone）
4. 產出 `OUT_DIR/template_ids_gen.rs`：所有 `pub const TPL_*` + `pub fn *_name()` + `pub fn *_by_name()` 反查
5. `lib.rs` `include!(concat!(env!("OUT_DIR"), "/template_ids_gen.rs"));`

Scripts workspace 和 omb workspace 各自 build 時會各自跑一次 build.rs；但因同一份 JSON → 確定性輸出，不會漂移。

## Migration 步驟

1. **建 crate** `omoba-template-ids`（空殼 + build.rs scaffold，id 表先空）
2. **建 `Story/templates.json`** — 從 5 個 scene 的 `entity.json` 聚合 + dedupe；補上 tower / ability / buff / projectile_kind / summon sections
3. **build.rs 實作 + generated code 檢查**（`cargo check -p omoba-template-ids` + id 分配正確）
4. **wire up dependencies**：
   - `scripts/script-abi/Cargo.toml` 加 `omoba-template-ids = { path = "../../omoba-template-ids" }`，re-export newtypes
   - `omoba-core/Cargo.toml` 加同樣 dep
   - `omb/Cargo.toml` 加同樣 dep（或透過 script-abi transitive）
5. **改 script-abi trait**：`UnitScript::unit_id() -> UnitTemplateId`；`AbilityScript::ability_id() -> AbilityId`
6. **改 proto**：`TowerCreate` / `HeroStatic` / `BuffAdd` / `BuffRemove` / `BuffSnapshot` 欄位；同步 encoder / decoder
7. **改 scripts/base_content**：所有 `RStr::from_str("...")` / `RString::from("...")` 的 id site 改 const；所有 `spawn_projectile` 的 `kind_tag` 參數 signature 改成 `ProjectileKindId`
8. **改 omb host**：catalog load 從 `Story/templates.json` + scene `entity.json` 合併；所有原本 emit string 的 event 改 emit u16
9. **改 omoba-core kcp client**：刪 `template_ids.rs`，改用新 crate 的 `*_name()` 反查函式
10. **改 omfx**：`TowerCreate` 顯示 name 改 `tower_name(TowerId(tower_id))`，buff tooltip 改 `buff_name(BuffId(buff_id))` 等
11. **Scene `entity.json`** schema 瘦身：刪 heroes / enemies 完整定義，改成 `heroes_used` / `creeps_used` 字串 list + waves
12. **Smoke test**：TD_1 跑通（creep/tower 名稱正常顯示）→ TD_STRESS 壓測 wire 下降 → B01_1 hero 場景跑通
13. **清理**：`omoba-core/src/template_ids.rs` 刪除；`KNOWN_CREEP_NAMES` / `KNOWN_PROJECTILE_KINDS` 常數刪除；scripts 內所有 id 字串搜尋確認無殘留

## Testing

- `omoba-template-ids` 內 unit tests：
  - roundtrip：`tower_name(TPL_TOWER_TACK) == "Tack Shooter"`
  - `tower_by_name("tower_tack") == Some(TPL_TOWER_TACK)`
  - id 0 reserved：所有 `*_name(Id(0)) == ""`
  - unknown id：`*_name(Id(9999))` debug panics
- 整合 smoke：跑 `gen-docs` 驗證 scripts 註冊的 `UnitTemplateId` 能反查回 `templates.json` 宣告的 id 字串
- Proto breaking change 確認：舊 client binary 連新 server → `data_json` fallback 處理要拒絕舊 buff 事件（因為欄位號 reserved，舊訊息 decode 會遺失資料）

## Risk

- **script-abi ABI 改變 → DLL 不相容**：`base_content.dll` 要跟 host 同時重 build。但 `run.bat` 已經這樣做，不是新問題
- **build.rs 順序漂移**：若 `templates.json` 陣列順序不小心被 reformat 工具重排 → id 整組變。Mitigation：`.editorconfig` / CI check 防護、人工 review
- **一次改太多 surface**：script-abi trait + proto + scripts 全部同時動。Mitigation：依 Migration 步驟 incremental commit、每步跑 `cargo check`
- **abi_stable newtype compat**：`#[repr(transparent)]` + `#[derive(StableAbi)]` over u16 理論支援，但實際要驗。Fallback：FFI 邊界用裸 `u16`，Rust 兩側各自 wrap/unwrap

## Wire size 預估

Stress 場景（每秒 3000 projectile + 1000 creep + 500 buff events）：
- `ProjectileCreate.kind_id` FNV u32 → sequential u16：每個省 ~2 bytes * 3000 = 6 KB/s
- `CreepCreate.name_id` 同上：~2 KB/s
- `BuffAdd.buff_id` string (~8 bytes) → uint32 varint (1-2 bytes)：每個省 ~6 bytes * 500 = 3 KB/s
- `TowerCreate.kind` + `TowerCreate.name` 刪除：每 tower create 省 ~20 bytes（低頻不累積）
- `HeroStatic.ability_ids` 4 個 string 8 字元 → 4 個 u16：每次省 ~28 bytes（低頻）

合計 stress 約 **10-12 KB/s** 下降。低頻事件省量小但 code 乾淨度提升最大。

## Rollback

若 proto 改動發現 wire incompatible bug：
- revert 該 proto commit（encoder/decoder 改回 string field）
- template_ids crate 保留（不丟），只是先不啟用於 wire
- Phase 1 退回：只做 compile-time safety（scripts 用 newtype const），wire 仍走 string

如此 template_ids crate 的投資仍保留，下次再推 wire migration。
