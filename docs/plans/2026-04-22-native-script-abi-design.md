# Native Script ABI 設計（abi_stable + DLL/SO 腳本）

**日期**：2026-04-22（PoC-2 完成 2026-04-23）
**狀態**：PoC-1（DartTower）+ PoC-2（8 個英雄技能 + 4 個塔）皆已落地。
**目的**：把 omb 的怪物、英雄、塔的**行為邏輯**抽成 native 腳本（編譯為 `.dll`/`.so`），以 `abi_stable` 達成跨 DLL 邊界 ABI 穩定。數值與資料仍走既有 config。

**實作狀態**（2026-04-23）：
- ✅ PoC-1：`scripts/base_content` DartTower（`on_attack_hit` hook）
- ✅ PoC-2：8 個英雄技能（Saika×4、Date×4）搬進 `scripts/base_content/src/heroes/`，走新增的 `AbilityScript` sabi_trait
- ✅ 塔擴充：Bomb / Tack / Ice 已腳本化
- ✅ `omb/ability-system` sub-crate 解散、主 crate 平行 skill/ability 系統刪除
- ✅ `omoba-core/ability_meta` 共享 schema 讓 client 可查 `list_abilities` metadata
- ⏳ 待接：`AbilityScript::execute` 的 skill dispatch 路徑（玩家 cast skill → DLL handler）
- ⏳ 待接：`host-side` buff/stat modifier 計算（目前 `add_buff`/`has_buff` 仍是 log stub）

---

## 1. 核心決策摘要

| 題目 | 選擇 | 理由 |
|------|------|------|
| 腳本化範圍 | **A+C**：只腳本化行為/技能邏輯；事件回調模型 | 數值仍走 config，host 保留 ECS 與主 tick 迴圈 |
| ECS 存取方式 | **Handle-based `#[sabi_trait] GameWorld`** | 最大彈性；腳本透過 `EntityHandle` + trait method 讀寫 ECS |
| 熱重載 | **H1：不支援**，啟動載入、結束卸載 | 避免 use-after-unload 地雷；release 用簡單穩定 |
| DLL 分組 | **D3：一 DLL = 任意單位集合**（mod 友好） | `base_content.dll` + 未來 `expansion_*.dll` + mod DLL |
| 執行模型 | **E1：腳本只在序列 dispatch 階段執行** | 獨佔 `&mut World`，無鎖無並發風險 |
| 腳本位置 | **W1：獨立 workspace**（`D:/omoba/scripts/`） | mod 作者可獨立複製開工 |
| Panic 策略 | **P1：log + skip** | 開發時修、release 容錯 |
| Config 綁定 | unit 定義新增 `script = "<id>"` 欄位（可選） | 增量遷移、與現有單位相容 |
| PoC 起點 | **arrow_tower**（單一 `on_attack_hit`） | 最小驗證面 |
| rustc 鎖定 | 雙 workspace 共用 `rust-toolchain.toml`（如 `1.85.0`） | abi_stable 要求相同 rustc |

---

## 2. Hook 清單（v1）

`UnitScript` trait 提供的事件回調（單元 script 方法）：

| Hook | 觸發方 | 用途 |
|------|--------|------|
| `on_spawn` | 自己 | 登場初始化、音效、buff |
| `on_death` | 自己 | 死亡爆炸、掉落、連鎖 |
| `on_damage_taken` | victim | 護盾、減傷、反射；可改寫 `amount` |
| `on_damage_dealt` | attacker | 吸血、附加效果、爆擊加成（看到最終 amount） |
| `on_skill_cast` | caster | 英雄技能主體邏輯 |
| `on_attack_hit` | attacker | 塔攻擊命中（濺射、穿透、特殊彈） |

v2 候選（有需求再加）：`on_kill` / `on_level_up` / `on_buff_tick` / `on_projectile_hit`。

**明確拒絕**：`on_tick(per-unit per-tick)`、`on_enter_vision` / `on_exit_vision`。

### 事件派發語意

```text
DamageEvent { attacker, victim, amount, kind }
  → victim.on_damage_taken(...)          // 先，可改寫 amount
  → attacker.on_damage_dealt(final)      // 後，看到最終 amount
  → host 套用最終 amount 到 HP

SkillCastEvent { caster, skill_id, target }
  → caster.on_skill_cast(...)            // 唯一觸發；腳本在裡面呼叫 GameWorld 實作效果

DeathEvent { victim, killer }
  → victim.on_death(...)
  → (v2) killer.on_kill(...)
```

---

## 3. Crate 結構

### omb workspace 新增

```
D:/omoba/omb/
├── script-abi/                   NEW — 純 ABI 契約 crate（host + DLL 都依賴）
│   ├── Cargo.toml                   crate-type = ["rlib"]
│   └── src/
│       ├── lib.rs                   re-export
│       ├── types.rs                 EntityHandle, Vec2f, DamageInfo, DamageKind, Target
│       ├── world.rs                 #[sabi_trait] GameWorld
│       ├── script.rs                #[sabi_trait] UnitScript + hook 方法
│       └── manifest.rs              #[export_root_module] Manifest + UnitDef
│
├── script-host/                  NEW — host 端載入/dispatch
│   ├── Cargo.toml                   depends: omb, script-abi, abi_stable
│   └── src/
│       ├── lib.rs
│       ├── loader.rs                掃 scripts/*.dll、load_root_module、建 registry
│       ├── registry.rs              HashMap<UnitId, UnitScript_TO>
│       ├── world_impl.rs            impl GameWorld for WorldAdapter<'a>
│       └── dispatch_system.rs       specs System — drain EventQueue → 呼叫 hook
│
├── src/
│   └── comp/script_event.rs      NEW — EventQueue<ScriptEvent> resource
│
└── ability-system/               (既有)
```

### scripts/ 獨立 workspace（W1）

```
D:/omoba/scripts/
├── Cargo.toml                    workspace root
├── rust-toolchain.toml           鎖定 rustc（與 omb 同版）
├── base_content/
│   ├── Cargo.toml                crate-type = ["cdylib"]；depends: omb-script-abi, abi_stable
│   └── src/
│       ├── lib.rs                #[export_root_module] 註冊所有單位
│       ├── heroes/
│       │   ├── mod.rs
│       │   ├── lancer.rs         impl UnitScript for Lancer
│       │   └── archer.rs
│       ├── towers/
│       │   └── arrow_tower.rs    PoC-1
│       └── creeps/
│           └── goblin.rs
└── target/                       腳本自己的 build 產物
```

### ABI crate 約束

- **只**用 abi_stable 型別（`RVec`, `RString`, `ROption`, `RBox`, `#[sabi_trait]`）
- **不得**依賴 `specs` / `omb` 主 crate
- `EntityHandle` = `#[repr(C)] struct { id: u32, gen: u32 }`，host 轉 `specs::Entity`

---

## 4. 核心型別草圖

### `script-abi/src/types.rs`

```rust
use abi_stable::{StableAbi, std_types::{RString, ROption, RVec}};

#[repr(C)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityHandle { pub id: u32, pub gen: u32 }

#[repr(C)]
#[derive(StableAbi, Copy, Clone, Debug)]
pub struct Vec2f { pub x: f32, pub y: f32 }

#[repr(u8)]
#[derive(StableAbi, Copy, Clone, Debug)]
pub enum DamageKind { Physical, Magical, Pure }

#[repr(C)]
#[derive(StableAbi, Clone, Debug)]
pub struct DamageInfo {
    pub attacker: ROption<EntityHandle>,
    pub amount: f32,
    pub kind: DamageKind,
}

#[repr(C)]
#[derive(StableAbi, Clone, Debug)]
pub enum Target {
    Entity(EntityHandle),
    Point(Vec2f),
    None,
}
```

### `script-abi/src/world.rs`（節錄）

```rust
use abi_stable::{sabi_trait, std_types::{RVec, RStr, ROption}};
use crate::types::*;

#[sabi_trait]
pub trait GameWorld: Send {
    // 查詢
    fn get_pos(&self, e: EntityHandle) -> ROption<Vec2f>;
    fn get_hp(&self, e: EntityHandle) -> ROption<f32>;
    fn get_max_hp(&self, e: EntityHandle) -> ROption<f32>;
    fn is_alive(&self, e: EntityHandle) -> bool;
    fn faction_of(&self, e: EntityHandle) -> ROption<RString>;
    fn query_enemies_in_range(&self, center: Vec2f, radius: f32, of: EntityHandle) -> RVec<EntityHandle>;

    // 修改
    fn set_pos(&mut self, e: EntityHandle, p: Vec2f);
    fn deal_damage(&mut self, target: EntityHandle, amount: f32, kind: DamageKind, source: ROption<EntityHandle>);
    fn heal(&mut self, target: EntityHandle, amount: f32);
    fn add_buff(&mut self, target: EntityHandle, buff_id: RStr<'_>, duration: f32);
    fn remove_buff(&mut self, target: EntityHandle, buff_id: RStr<'_>);
    fn spawn_projectile(&mut self, from: Vec2f, to: EntityHandle, speed: f32, dmg: f32, owner: EntityHandle) -> EntityHandle;
    fn despawn(&mut self, e: EntityHandle);

    // 特效/音效（非遊戲狀態）
    fn play_vfx(&mut self, id: RStr<'_>, at: Vec2f);
    fn play_sfx(&mut self, id: RStr<'_>, at: Vec2f);

    // 日誌（跨 FFI 後導回 host 的 log4rs）
    fn log_info(&self, msg: RStr<'_>);
    fn log_warn(&self, msg: RStr<'_>);
}
```

增加 component → 加 method；ABI 以**不改既有 method 簽名**為前提穩定。

### `script-abi/src/script.rs`

```rust
use abi_stable::{sabi_trait, std_types::{RStr, ROption}};
use crate::types::*;
use crate::world::GameWorld_TO;

#[sabi_trait]
pub trait UnitScript: Send + Sync {
    fn unit_id(&self) -> RStr<'_>;

    fn on_spawn(&self, _e: EntityHandle, _w: &mut GameWorld_TO<'_, ()>) {}
    fn on_death(&self, _e: EntityHandle, _killer: ROption<EntityHandle>, _w: &mut GameWorld_TO<'_, ()>) {}
    fn on_damage_taken(&self, _e: EntityHandle, _dmg: &mut DamageInfo, _w: &mut GameWorld_TO<'_, ()>) {}
    fn on_damage_dealt(&self, _attacker: EntityHandle, _victim: EntityHandle, _final_amount: f32, _w: &mut GameWorld_TO<'_, ()>) {}
    fn on_skill_cast(&self, _caster: EntityHandle, _skill_id: RStr<'_>, _target: Target, _w: &mut GameWorld_TO<'_, ()>) {}
    fn on_attack_hit(&self, _attacker: EntityHandle, _victim: EntityHandle, _w: &mut GameWorld_TO<'_, ()>) {}
}
```

- `on_damage_taken` 拿 `&mut DamageInfo` → **腳本可改寫 amount**（護盾／減傷）
- 所有 hook 都有 default impl → 腳本只實作自己要的

### `script-abi/src/manifest.rs`

```rust
use abi_stable::{
    library::RootModule, package_version_strings,
    std_types::{RString, RVec}, sabi_trait::TD_Opaque, StableAbi,
};
use crate::script::UnitScript_TO;

#[repr(C)]
#[derive(StableAbi)]
pub struct UnitDef {
    pub unit_id: RString,
    pub script: UnitScript_TO<'static, ()>,
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = "Manifest_Ref")))]
#[sabi(missing_field(panic))]
pub struct Manifest {
    pub units: extern "C" fn() -> RVec<UnitDef>,
}

impl RootModule for Manifest_Ref {
    abi_stable::declare_root_module_statics! { Manifest_Ref }
    const BASE_NAME: &'static str = "omb_script";
    const NAME: &'static str = "omb_script";
    const VERSION_STRINGS: abi_stable::sabi_types::VersionStrings = package_version_strings!();
}
```

---

## 5. 執行流程（單 tick）

```text
[並行 tick 階段] — 現有 tick systems 照跑
  HeroMoveTick / TowerTick / CreepTick / ProjectileTick / DamageTick ...
    ↳ 產生 ScriptEvent 推入 EventQueue<ScriptEvent> resource

[序列 ScriptDispatchSystem]（E1 — 單執行緒、獨佔 World）
  let events = world.fetch_mut::<EventQueue>().drain();
  for ev in events {
      match ev {
          ScriptEvent::Damage { attacker, victim, mut info } => {
              // 1. victim.on_damage_taken（可改 info.amount）
              if let Some(s) = registry.get(unit_id_of(victim)) {
                  catch_unwind(|| s.on_damage_taken(handle(victim), &mut info, &mut adapter));
              }
              // 2. attacker.on_damage_dealt
              if let Some(s) = registry.get(unit_id_of(attacker)) {
                  catch_unwind(|| s.on_damage_dealt(handle(attacker), handle(victim), info.amount, &mut adapter));
              }
              // 3. host 套用最終 amount
              world.write_storage::<Hp>().get_mut(victim).unwrap().current -= info.amount;
          }
          ScriptEvent::SkillCast { caster, skill_id, target } => { ... }
          ScriptEvent::Death { victim, killer } => { ... }
          ScriptEvent::AttackHit { attacker, victim } => { ... }
          ScriptEvent::Spawn { e } => { ... }
      }
  }
```

- `WorldAdapter<'a>` 實作 `GameWorld`，內含 `&'a mut specs::World`（序列階段獨佔 → 無鎖）
- `catch_unwind` 對應 **P1**：panic 攔截 → `log::error!` + 丟棄該 hook，遊戲繼續

---

## 6. 載入流程（啟動時）

```rust
// script-host/src/loader.rs
pub fn load_all(scripts_dir: &Path) -> Registry {
    let mut reg = Registry::default();
    for dll in list_dynlibs(scripts_dir) {
        let manifest = Manifest_Ref::load_from_file(&dll)
            .expect(&format!("load script {dll:?} failed"));
        for def in manifest.units()() {
            let id: String = def.unit_id.into();
            if reg.contains(&id) {
                log::warn!("duplicate unit_id {id} — overriding");
            }
            reg.insert(id, def.script);
        }
    }
    reg
}
```

`Registry` 一旦建好就**不動**（H1），整個 process lifetime 持有 DLL handle → 不會 unload。

---

## 7. Config 整合

現有 `game.toml` / campaign JSON 的單位定義新增一欄：

```toml
[[heroes]]
id = "hero_lancer"
name = "槍兵"
hp = 800
atk = 45
script = "hero_lancer"    # 可選；對應腳本 manifest 裡的 unit_id
```

**Host 行為**
- `script` 欄缺省 → 純 host 邏輯（與現況相容）
- `script` 欄填了但 `Registry.get()` 回 `None` → `log::warn!`、fallback 到 host 預設
- `script` 欄填了且找到 → spawn 時 `ScriptUnitTag { unit_id }` 掛到 entity

**dispatch 取 script_id**
- `ScriptDispatchSystem` 從 `WriteStorage<ScriptUnitTag>` 取 `unit_id`
- 沒掛 tag 的 entity → 跳過該 hook（不是錯誤）

---

## 8. rustc / toolchain 鎖定

abi_stable 要求 host 與所有 DLL **相同 rustc 版本**。

- `D:/omoba/rust-toolchain.toml`（新）
- `D:/omoba/scripts/rust-toolchain.toml`（新）
- 兩份內容相同，例如：

```toml
[toolchain]
channel = "1.85.0"
components = ["rustc", "cargo", "rust-std"]
profile = "minimal"
```

升版規則：**同步升**；若 host 先升必須同步重編所有 DLL。

---

## 9. 錯誤 / Panic 策略（P1 細節）

- 所有 `UnitScript` 方法呼叫包 `std::panic::catch_unwind`
- panic → `log::error!("script panic in on_xxx of {unit_id}: {payload}")`
- 不標記 entity、不禁用 DLL、遊戲照跑
- 多次 panic 同一 entity → 後續由 v2 `panic_budget` 機制處理（暫不做）

---

## 10. 實作路徑（incremental）

1. **建 `omb/script-abi` crate**：型別、trait、manifest
2. **建 `omb/script-host` crate**：loader + registry + WorldAdapter 骨架（先只實作 3~5 個 GameWorld method 夠 PoC 用）
3. **加 `ScriptEvent` / `EventQueue` / `ScriptDispatchSystem`**：在 ecs_setup 裡註冊為最後一個系統
4. **加 `ScriptUnitTag` component**，spawn 流程從 config 讀 `script` 欄掛 tag
5. **建 `D:/omoba/scripts/` workspace + `base_content` crate**
6. **PoC-1：`arrow_tower`**
   - 單一 `on_attack_hit` 邏輯（例：10% 機率造成額外 50 傷害）
   - 現有 `arrow_tower` config 加 `script = "tower_arrow"`
   - 驗證：load → dispatch → GameWorld.deal_damage → HP 下降 → 全鏈路通
7. **PoC-2：一個 active skill 英雄**（驗證 `on_skill_cast` + 多步 GameWorld 呼叫）
8. **擴充 GameWorld API**（按實際腳本需求逐步加 method）
9. **遷移文件 + 範例腳本模板**

---

## 11. 已知風險 / 之後再想

| 風險 | 緩解 |
|------|------|
| `GameWorld` method 爆炸成百行 | 以「最小夠用」原則加；按 PoC 需求驅動；可分 `GameWorld` / `GameWorldQuery` / `GameWorldMut` 多個 trait |
| 腳本跨 DLL 版本不符 | abi_stable 啟動時 `RootModule::load_from_file` 會檢查 `VERSION_STRINGS`；不符直接 fail-fast |
| rustc 升版破 ABI | toolchain.toml 鎖死；升版 CI 一次全重編 |
| 腳本被惡意/壞掉：讀寫非自己 entity | v1 不做沙盒；mod 內容即信任內容。若要做 → 給「縮窄版 GameWorld」只開自己 entity |
| lockstep/replay 相容性 | A+C 模型下腳本是 deterministic（只操作 ECS、不做 I/O）；GameWorld 內部若用 RNG 必須走 host 提供的 seeded RNG（**未來 API 要加 `fn rng(&mut self) -> u64`**） |
| 浮點確定性（跨平台） | 後續議題；PoC 階段不處理 |

---

## 12. 不在本次範圍

- 資料驅動腳本化（方案 B：數值也在 DLL 裡）
- 熱重載（H2/H3）
- 沙盒/權限模型
- 多語言腳本（wasm/lua/rhai）
- 存檔對腳本 state 的序列化（腳本目前設計成無 state）

---

## 13. 附錄：PoC-1 範例程式草稿

### 腳本端（`scripts/base_content/src/towers/arrow_tower.rs`）

```rust
use omb_script_abi::prelude::*;

pub struct ArrowTower;

impl UnitScript for ArrowTower {
    fn unit_id(&self) -> RStr<'_> { rstr!("tower_arrow") }

    fn on_attack_hit(&self, attacker: EntityHandle, victim: EntityHandle, w: &mut GameWorld_TO<'_, ()>) {
        let roll: f32 = /* host-provided RNG — v2 */ 0.0;
        if roll < 0.10 {
            w.deal_damage(victim, 50.0, DamageKind::Physical, ROption::RSome(attacker));
            w.play_vfx(rstr!("vfx_crit"), w.get_pos(victim).unwrap_or(Vec2f { x:0.0, y:0.0 }));
        }
    }
}
```

### manifest（`scripts/base_content/src/lib.rs`）

```rust
use abi_stable::{export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn, std_types::RVec, sabi_trait::prelude::*};
use omb_script_abi::{manifest::{Manifest, Manifest_Ref, UnitDef}, script::UnitScript_TO};

mod towers;

#[export_root_module]
fn get_manifest() -> Manifest_Ref {
    Manifest { units }.leak_into_prefix()
}

#[sabi_extern_fn]
fn units() -> RVec<UnitDef> {
    let mut v = RVec::new();
    v.push(UnitDef {
        unit_id: "tower_arrow".into(),
        script: UnitScript_TO::from_value(towers::arrow_tower::ArrowTower, TD_Opaque),
    });
    v
}
```

### host 載入點（`omb/src/main.rs` 或 ecs_setup）

```rust
let registry = script_host::loader::load_all(Path::new("./scripts/target/release"));
world.insert(registry);
dispatcher_builder.add(script_host::dispatch_system::ScriptDispatchSystem, "script_dispatch", &[/* after all tick systems */]);
```

---

**設計定案。下一步：建立 `omb/script-abi` crate，寫第一版 ABI 契約。**
