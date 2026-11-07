# 英雄製作指南：從描述到可玩的英雄

本指南說明如何從「原始描述與來源資料」做出一個英雄。成品是能選角、能施放、能通過測試的完整 kit。讀者是 coding agent 或工程師，不需要知道本專案的歷史。

完整範例是 H03 阿松，見 §9。§10 是失敗診斷。

---

## 1. 成品與責任

一個英雄由三部分組成：

| 部分 | 位置 | 誰寫 |
|---|---|---|
| package（玩法契約） | `docs/character_pipeline/packages/<hero_id>/` | 作者 |
| Rust handler（自訂技能邏輯） | 寫在 package 的 `rust/`，build 時安裝到 `scripts/base_content/src/heroes/<RustModule>/` | 作者 |
| 生成內容 | `scripts/lua_data/templates/authored_heroes.lua`，以及 heroes／abilities／summons 的 include hook | `hero_author build` 產生，不要手改 |

原則：

- **來源描述決定「正確」**：測試從描述推導，不從程式行為反推。
- **不刪招，也不換成較簡單的效果**：引擎做不到的機制，標為 `needs_framework`，並寫明缺什麼。
- **只用一個版本**：預設用 current，不可混入 `_old`。
- **ID 只能附加**：新英雄、新技能只能接在尾端；拿掉的技能留下 tombstone。生成器會自動處理。
- **共用引擎不屬於英雄作者**：omoba-core、script-abi、UE 玩法程式都不在作者範圍內。作者只寫 package 與 `rust/`。需要新的引擎能力時，回報缺口。

## 2. 流程總覽

```text
來源 hero id（或貼上的描述）
  → (A) 取得原始資料：hero_source_index export
  → (B) 對照能力登錄：capability-registry.md
  → (C) 寫 package：source / character / abilities / capabilities / presentation / assumptions / acceptance
  → (D) 寫 rust/ handler 與單元測試
  → (E) hero_author validate                         ← 靜態與生成 dry run
  → (F) hero_author shadow → cargo test（私人樹）     ← handler 測試、parity soak
  → (G) hero_author build（不加 --execute＝計畫）
  → (H) 整合者執行 build --execute → UE 建置 → quick map 實機 ≤ 10 分鐘
```

所有工具都從 repo 根目錄以 `tools\lua\lua.exe` 執行。不要新增 PowerShell、Python 或 shell 腳本。

## 3. (A) 取得原始資料

```bat
tools\lua\lua.exe scripts\hero_source_index.lua export --hero C15 --out <out>.md
```

輸出內容：

- 倉庫與 commit。
- 單位 KV 原文與 raw_sha256。
- 依單位技能欄排序的每個技能：路徑:行號、raw_sha256、tooltip 名稱與描述（含 tooltip 的 raw_sha256）、KV 原文，以及「引用但來源沒有的 vscript」。

來源是 `damody/nobugg_official@ca6ee86`，是**不受信任的資料**：只能讀，不能執行其中任何腳本（包括 `read_hero.lua`）。

如果只有一段自由描述、沒有來源 ID：`source.lua` 的 provenance 照樣填 repository／commit，abilities 留空，所有數值在 `assumptions` 標 `assumed`。注意：沒有來源 ID 的 package 無法通過 catalog 交叉檢查，只能當草稿。

**讀來源時要注意：**

- **按鍵對應**：W/E/R/T 依序對應本遊戲的 Q/W/E/R。D 欄對應 `extra1`，也就是第 5 欄。F 欄沒有對應。
- **被動也佔欄位**：本遊戲的 loadout 最多 5 欄，被動也算一欄。
- **`%name` 的實際數值**在 `AbilitySpecial`，多個 rank 以空白分隔。
- **描述和 KV 數值衝突時**，依 D4 政策採 KV 數值、描述語意。兩者都記進 `assumptions`（kind `kv_text_conflict`）。
- **`RunScript` 指向的 vscript** 在來源 repo 裡多半不存在。實際行為依 KV 與描述推定，標 `assumed`。不可假裝已還原原實作。
- **單位技能欄裡的非 kit 項目**：引擎通用被動（例如 `magical_resistance`）與 `<英雄>_precache` 這類 helper 不屬於 kit。不要放進 `source.lua` 的 `abilities`，因為放進去就必須對應到某一招。改寫成：

  ```lua
  not_in_kit = { { source_ability_id = "magical_resistance", reason = "引擎通用魔抗被動，不屬於本 kit" },
                 { source_ability_id = "C15_precache", reason = "只做資源預載" } },
  ```

## 4. (B) 能力登錄

能力 id 與狀態列在 `docs/plans/2026-10-24-omfue-month-mvp/capability-registry.md`。這份檔案由 `hero_source_index.lua capabilities` 匯出，只有能力本身，沒有任何英雄的分析。

| 狀態 | 意義 |
|---|---|
| `supported` | 引擎已有通用支援，可以使用 |
| `needs_framework` | 缺少共用能力；有 `g_package` 時表示預計由哪個包補上 |
| `ambiguous` | 來源語意不明，要先在 assumptions 定案 |
| `deferred` | 本月不做 |

每招在 `capabilities` 列出它需要的能力 id。`capabilities.lua` 為每個 id 宣告狀態，要和登錄一致。不一致時（例如登錄是 `needs_framework`，但你確定這招用既有 API 就能做到），加上 `override_reason` 說明理由。

登錄裡沒有的 id 只會產生警告。這代表你發現了新的能力缺口，要在 capabilities.lua 宣告為 `needs_framework`，並在 assumptions.md 寫明缺什麼。新 id 的命名規則：

- snake_case，描述機制本身，不寫英雄或技能名。例如 `ground_area_line`、`retarget_on_death`、`summon_forced_target`。
- 先寫機制，再寫修飾。例如 `projectile_vs_building`、`stealth_attack_no_reveal`。
- 已有的近似能力加後綴區分，不要另起同義詞。

英雄自己的掛點（普攻命中、施法、受傷、每 tick）用登錄的 `hero_unit_hooks`（另有 `on_cast_trigger`、`aura` 等更具體的 id）。被動技能的 `on_attack_hit` 則是 `on_attack_hook`。

**本月新增、作者可以直接使用的通用能力**（細節見 §7.3）：

- 追蹤／直線投射物（含命中 DoT、以建築為目標）
- 充能
- 法球（toggle、每擊扣魔）
- 引導（每波效果，可有前搖；用 `timing.channel` 宣告，見 §5.3）
- 定點召喚
- 施法縮短其他技能冷卻
- 英雄隱形與無單位碰撞
- 依距離結算的光鍊
- 範圍友軍回復（生命／法力 %）
- 範圍友軍增益（stat buff）
- 光環：敵方或友方，由英雄 UnitScript tick 定期刷新
- 英雄 UnitScript（被動掛點）
- 第 5 欄與自動學習

`capability-registry.md` 由登錄匯出，舊的匯出可能過時。狀態若與本節或程式碼不符，以程式碼為準，並在 capabilities.lua 用 `override_reason` 說明。

## 5. (C) package 檔案

每個 Lua 檔都回傳 `function(ctx) return { schema_version = 2, ... } end`。資料檔在空的沙盒裡載入：不能用 `io`、`os`、`require`，也沒有 `pairs` 以外的全域函式。共用的 helper（例如 `lifecycle`）請寫成檔案內的區域函式。

### 5.1 `source.lua`

```lua
provenance = { repository, commit, catalog, source_hero_id, variant = "current", language = "zh-TW",
  unit_file = { path, line, raw_sha256 } },
abilities = { { source_ability_id, path, line, raw_sha256, tooltip_sha256 }, ... },
assumptions = { { id = "snake_id", kind = "assumed"|"adaptation"|"kv_text_conflict"|"not_applicable", text = "..." }, ... },
```

所有 hash 直接從 export 複製。validate 會核對 catalog：hash 不同就判定 package 過期；variant 或英雄不符也會報錯。

### 5.2 `character.lua`

```lua
hero = { id = "snake_id", display_name, title, role, combat_read },
identity = {
  game_hero_id = <hero.id>,
  integration = "append",            -- 新英雄；改既有英雄才用 "update_existing"
  rust_module = "C15_tamako",        -- Rust 模組名，慣例是 <來源ID>_<hero_id>
  source_hero_id = "C15",
  unit_script = true,                -- 選用：需要英雄 UnitScript（施法／受傷掛點）時才寫，見 §7.4
},
attack = { kind = "ranged"|"melee", range, projectile_speed?, attack_point, base_attack_time },
base_stats = { base_hp, base_mana, base_damage, base_armor, move_speed, turn_speed },
gameplay = { lane, strengths = {...}, weaknesses = {...} },
moba_loadout = { ranks = { q, w, e, r[, d] }, skill_points = n },   -- 選用
```

- 來源的生命與法力常由屬性推得，例如 `StatusMana=0`。基礎數值請沿用同定位的本地英雄，並寫進 assumptions（kind `adaptation`）。
- `moba_loadout` 省略時，出生 rank 由程式推導：第 1 級可學的技能是 1，其他是 0（大招為 0）。自己填的話，任何「第 1 級學不到」的欄位都不能大於 0，否則 validate 會失敗（E029）。

### 5.3 `abilities.lua`

每招一筆，欄位如下（validate 會檢查必填欄位）：

| 欄位 | 說明 |
|---|---|
| `id` | snake_case。慣例是 `<來源小寫>_<名稱>`，例如 `a34_frost_bolt` |
| `source_ability_id` | 對應 `source.lua` 的 abilities |
| `action` | `Q`、`W`、`E`、`R`、`extra1`。`extra1` 是第 5 欄，要列 capability `extra_action_slot` |
| `ability_type` | `active`、`toggle`、`passive`、`ultimate` |
| `name`、`description` | 玩家看到的名稱與 tooltip，寫入各 rank 的實際數值 |
| `max_level` | rank 數 |
| `status` | 由 capabilities 推導最差的狀態；寫錯時 validate 會報錯 |
| `required_hero_level` | 每個 rank 的英雄等級門檻（大招常見 `{6,12,18}`） |
| `auto_learn` | `true` 表示到達門檻就自動學會，不耗技能點，玩家不能手動加點。用於來源 `NOT_LEARNABLE` 的技能 |
| `capabilities` | 這招需要的能力 id |
| `targeting` | `kind`（`none`／`self`／`unit`／`point`／`direction`）、`team`（`enemy`／`ally`／`both`／`self`）、`range`、`shape`（`none`／`circle`／`cone`／`line`／`ring`）、`visibility`（政策文字） |
| `timing` | `cast_point`（前搖）、`cooldown_starts`（`on_cast`／`on_end`）、`channel`（選用，見下方「引導」） |
| `resources` | `cooldown`、`mana_cost`、`charges = { max, recharge_seconds }`（選用） |
| `effects` | 封閉詞彙，見下表 |
| `lifecycle` | `caster_death`、`target_invalid`、`vision_lost`、`cancel`、`expire` 五項都要寫，而且 handler 要真的做到（E036） |
| `implementation` | `{ kind = "rust_handler", module = "No1_xxx", note }`，或 `{ kind = "generic", effects = ... }` |
| `bot_usage` | 對誰、何時使用、何時避免 |
| `verification.scenarios` | 從描述推導的驗收情境 `{ name, expect }` |
| `assumptions` | 引用 `source.lua` 的 assumption id |

數值可以寫單一數字，也可以寫每個 rank 一個值的陣列。數值必須是有限值、0 或絕對值介於 1/1024～1,000,000（以 f32 檢查）。

**effects 詞彙**（`docs/character_pipeline/tools/package_v2.lua` 的 `M.EFFECTS`）：

| kind | 必填 | 用途 |
|---|---|---|
| `damage` | amount, damage_kind | `physical`／`magical`／`pure` |
| `heal` | amount | |
| `shield` | amount, duration | |
| `stat_buff` | stats, duration | stats 的 key 見下方 |
| `stat_bonus_permanent` | stats | 被動常駐 |
| `control` | control, duration | `stun`／`root`／`silence` |
| `slow` | move_speed_pct, duration | |
| `damage_over_time` | amount_per_second, duration, damage_kind | |
| `damage_zone` | radius, amount_per_tick, interval, duration, damage_kind | |
| `summon` | unit, count, lifetime | |
| `dash` | max_distance | |
| `projectile` | motion（`linear`／`homing`）, speed, radius | 追蹤彈另需 lifetime |
| `on_attack_bonus_by_target_class` | classes, damage_kind | |
| `mana_drain` | amount | |
| `attack_orb` | mana_per_attack, amount, damage_kind | 只能用於 toggle、targeting none，且是唯一 effect。選填：`move_slow_pct`、`attack_slow_pct`、`slow_duration` |
| `cooldown_refund_on_cast` | seconds | 學會後，施放其他非大招時，其餘非大招的剩餘冷卻減少 seconds |
| `tether` | duration, max_distance | 光鍊，到期時依距離結算 |
| `custom` | note | 詞彙表達不了的機制（線形地面區域、鎖定目標的召喚物、每 N 次攻擊觸發……）。`note` 寫出機制；只能搭配 `rust_handler`。數值欄位照樣進 extras |

`stats` 可用的 key：`attack_damage_bonus`、`base_damage_outgoing_pct`、`attack_range_bonus`、`attack_speed_bonus`、`attack_speed_multiplier`、`move_speed_bonus`、`move_speed_pct`、`armor_bonus`、`magic_resist_bonus`、`incoming_damage_pct`、`health_regen`、`max_hp_bonus`（最大生命，handler 用 `StatKey::HealthBonus`）、`mana_bonus`、`attack_stun_chance`、`attack_stun_duration`、`cooldown_reduction_pct`、`physical_constant_block`、`invisibility_level`。

**引導（pulse channel）**：在 `timing` 宣告。

```lua
timing = { cast_point = 0.3, cooldown_starts = "on_cast",
  channel = { anchor = "point", interval = 0.1, pulses = { 50, 60, 70 }, radius = { 340, 340, 340 } } },
```

- `anchor` 可為 `self`、`point`、`unit`，必須與 targeting.kind 一致（self 對應 `none` 或 `self`）。
- `self`／`point` 的錨點需要半徑：寫 `channel.radius`，否則用 `targeting.shape.radius`。
- 生成器會輸出 content `channel` 與 extras `channel_interval`、`channel_pulses`（以及 `channel_radius`）。
- host 擁有引導的生命週期：開始、每波計時、中斷（受控、移動、死亡、回城、任何新指令）、只扣一次費用。
- `build_ability_ffi` 會自動把它帶成 host execution。handler 只實作 `on_channel_pulse(caster, target, pulse, world)`，`execute` 不會被呼叫。
- 可以有前搖（`cast_point` > 0）：前搖結束後才開始引導，前搖內取消不扣費。
- 每波 handler 拿到的是 pulse 序號，不是 rank 的 json，要從 content extras 讀值。

effect 裡可以加說明欄位，例如 `applies_to`、`delivered_by`、`note`。數值欄位會被收進 extras；文字欄位只供閱讀。

`needs_framework` 技能的 `implementation`：照樣寫 `{ kind = "rust_handler", module = "<預定的模組名>", note = "缺 <能力 id>" }`，但先不要在 `rust/` 放那個模組檔。切片建置不會要求它；能力補上後再寫 handler，並把狀態改為 supported。

**extras 命名規則**：Rust handler 只靠 extras 名稱讀值，這條規則很重要。每個數值欄位會變成 `extras["<kind>[_序號]_<欄位>"][rank]`。序號從同 kind 的第二個 effect 開始加。stats 會展開成 `<kind>_<stat名>`。另外兩個固定 key：targeting 的 `shape.radius` 是 `shape_radius`，charges 是 `charges_max`／`charges_recharge_seconds`。範例：

| package 寫法 | extras key |
|---|---|
| `{ kind="damage", amount={100,200} }` | `damage_amount` |
| 第二個 `{ kind="damage", amount=100 }` | `damage_2_amount` |
| `{ kind="projectile", speed=2000, radius=16, lifetime=3 }` | `projectile_speed`、`projectile_radius`、`projectile_lifetime` |
| `{ kind="stat_buff", duration=16, stats={ armor_bonus=5 } }` | `stat_buff_duration`、`stat_buff_armor_bonus` |
| `targeting.shape.radius = 335` | `shape_radius` |
| `targeting.range = 0` 且 `{ kind="dash", max_distance=300 }` | `dash_max_distance`，同時宣告為呈現用 reach（`targeting_reach_key`）：指示器把落點夾在 300 內，autopilot 觸及距離為 300＋area 半徑。施放本身不受距離限制，handler 要自己把位移夾在 max_distance 內（E090） |

**單位換算**：extras 存成 Fixed64，1.0 = 1024 raw。百分比原樣保留，`30` 就是 30。handler 要自己轉成引擎用的單位：

- 移速 slow、`base_damage_outgoing_pct`、`incoming_damage_pct`、`cooldown_percentage` 都用 1 的分數。一律用 `(raw + 50) / 100`（正值四捨五入）；負值用 `(raw - 50) / 100`。所有範例都用這條規則。
- 攻速點、魔抗百分比點、護甲、回血、最大生命、法力上限都直接使用 raw。

### 5.4 `capabilities.lua`

```lua
capabilities = { { id = "projectile_tracking", status = "supported" },
                 { id = "x", status = "needs_framework", g_package = "G-4" },
                 { id = "y", status = "supported", override_reason = "..." } },
kit_status = "supported" | "needs_framework" | ...,   -- 全部技能中最差的狀態
catalog_ready = false,                                  -- 只有驗收完成（acceptance.md 寫 status: accepted）才能改成 true
```

### 5.5 `presentation.lua`

```lua
placeholder = { body = "capsule_ranged"|"capsule_melee", team_tint = true, weapon_socket = "hand_r" },
animation_states = { idle, move, attack, cast, channel, death, recall },
abilities = { <ability_id> = { preview = "unit_target"|"ground_circle"|"swept_path"|"none"|..., path_width?, animation = "cast",
  cues = { cast = "...", projectile = "<ability_id>", impact = "...", loop = "...", ["end"] = "..." } } },
```

每招都要有一筆。`preview` 要和 targeting 一致；直線技能用 `swept_path`，並寫 `path_width`。

### 5.6 `assumptions.md` 與 `acceptance.md`

- `assumptions.md`：一節「阻塞」，列出 `needs_framework` 的招式與缺口；另一節「假設」，用人話寫出 source.lua assumptions 的每一項。
- `acceptance.md`：第一行狀態寫 `status: <...>`（例如 `implemented`、`partial`、`accepted`）。內容是技能表，以及每招在 quick map 實機 10 分鐘內要觀察的現象。

## 6. (D) Rust handler

### 6.1 檔案與註冊

- 檔名：`rust/No<n>_<名稱>.rs`。`implementation.module` 寫不含副檔名的檔名。
- 每個檔案必須匯出 `pub fn <ability_id>_ffi()->AbilityDefFFI`。安裝時會在 `mod.rs` 自動加上 `pub use No<n>_<名稱>::<ability_id>_ffi;`。
- 測試 fixture：`rust/<來源小寫>_test_support.rs`，以 `#[cfg(test)]` 載入。可以從 `packages/omatsu/rust/a34_test_support.rs` 複製後改註解。
- 共用 helper 用其他檔名，例如 `b08_fang.rs`，以 `super::b08_fang` 引用。
- 英雄 UnitScript：`rust/hero_unit.rs` 匯出 `pub fn hero_unit_script()->impl UnitScript`，並在 `character.lua` 寫 `identity.unit_script = true`。
- 技能 id 常數由生成器產生：`omoba_template_ids::ABILITY_<ID 大寫>`。數值一律從 `active_ability_const(ID)` 的 extras 讀取，**不要在 Rust 裡寫死數值**。

### 6.2 handler 寫法

以 `packages/omatsu/rust/No2_iceberg_strike.rs` 為範本：

```rust
pub(crate) fn resolve(rank:u8,json:&str)->Result<Spec,&'static str>  // 讀 content，驗證 rank 與數值範圍
pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster,target,spec)->Result<(),&'static str>
impl AbilityScript for Handler { fn execute(...) -> RResult { resolve(..).and_then(|s| execute(..)) } }
```

規則：

1. **所有檢查做完才產生 outcome**。拒絕時不能留下任何 outcome；測試 fixture 會斷言這一點。
2. **目標合法性要明確檢查**：存活、hp>0、敵我（`query_enemies_in_range(at,0,caster)` 是否含目標）、建築（`is_building`）、距離（`omoba_sim::terrain::point_in_swept_circle(at,origin,origin,range)`）。**`query_enemies_in_range` 會回傳敵方建築**；來源只打英雄與一般單位（`HERO | BASIC`）時，要用 `is_building` 過濾掉。
3. **多目標要穩定排序**：`sort_by_key(|e|(e.id,e.gen))` 後再 `dedup`。
4. **被動**：`execute` 回傳錯誤；在 `on_learn(caster,rank,world)` 以同一個 buff id 加上永久 buff。這樣升級時是取代，不會疊加。
5. **buff payload**：用 `StatKey::X.as_str()` 當 key，值用 raw 整數。施法者專屬的 buff id 要帶 `caster.id`／`caster.gen`，例如 `format!("x:{}:{}",caster.id,caster.gen)`。
6. **ABI 限制**：`GameWorld` 的方法表已滿（E045），不能新增方法。新的 host 行為一律走「payload 封包」：送一個特定 buff id 或 payload key，由 host 在 serial settlement 攔截。§7.3 列出現有的封包。

### 6.3 host 封包與通用狀態（作者可用）

| 需求 | 做法 |
|---|---|
| 追蹤／直線投射物 | `omoba_core::runtime::native::ability_projectile::Spec`（`Motion::Homing`／`Linear`、speed、radius、distance、lifetime、max_hits、wall／target_lost／caster_death 政策、amount／kind／profile、slow、control、`dot: Option<(每秒, 秒)>`、`buildings: bool`（只限追蹤彈））。以 `add_stat_buff(caster, host::BUFF, lifetime, spec.payload())` 發射；傷害在命中時才結算。**每位施法者同時最多 `MAX_LIVE_PER_SOURCE`（32）發**：一次施法發出的投射物，加上仍在飛的，超過上限時整個施法會被拒絕。多目標技能要先把目標數截到上限以內 |
| 多目標選取（隱藏目標） | 依「施放當下可見」選目標的技能，覆寫 `AbilityScript::execute_disclosed`，用 `disclosure` 查詢略過霧中單位，再套數量上限，讓權威端與 replica 選到同一批目標（E036、E056）。範例：`packages/date_masamune/rust/No5_flame_trail_dash.rs` |
| 法球 | effect `attack_orb`（content）。handler 只負責切換 `attack_orb::buff_id(ID)` 標記，見 `packages/inahime/rust/No1_venom_arrow.rs` |
| 燒魔 | `mana_drain::Spec::checked(caster.id,caster.gen,amount,range)`，送到 `mana_drain::BUFF`，duration 為 1 |
| 範圍友軍回復（生命／法力 %） | `ally_area_restore::Spec{center,radius,hp_fraction,mana_fraction,cast_range}`，送到施法者自身的 `ally_area_restore::BUFF`，duration 為 1 |
| 範圍友軍增益（stat buff） | `ally_area_buff::Spec{center,radius,buff,seconds,stats:vec![(StatKey 名稱, raw)]}`（最多 8 項），送到施法者自身的 `ally_area_buff::BUFF`，duration 為 1。範圍內同隊、非建築單位（含施法者）套上 `buff`；同一 id 刷新不疊加 |
| 光環（敵方或友方） | 英雄 UnitScript `on_tick`：每 0.5 秒（以施法者身上的 think 標記 buff 節流）。敵方光環用 `query_enemies_in_range` 加短 buff；友方光環送 `ally_area_buff` 封包。buff 持續時間比刷新間隔稍長，例如 0.75 秒。範例：`packages/murakami_yoshikiyo/rust/No2_quake_axe.rs`、`No3_morale_boost.rs` |
| 引導（每波效果） | `timing.channel`（§5.3）。handler 實作 `on_channel_pulse`；每波目標要用決定性選取（例如對施法者、施法點、波序做雜湊），不要用 host 亂數序號。範例：`packages/murakami_yoshikiyo/rust/No4_earth_tremor.rs` |
| 線形持續區域（火牆、沿箭路徑的火焰帶） | `DamageZoneSpec{end:Some(終點),grow,linger,..}`：`end` 讓區域成為 center→end 的膠囊（長度 ≤10000）；`grow:Some(速度)` 讓火焰從 center 以該速度逐段鋪出（只限 `independent:true`）；`linger:Some(秒)` 讓每次 pulse 改為刷新同一個灼燒 DoT（速率 = amount／interval），離開後最多再燒 linger 秒、重疊不疊加。以 `__ground_damage_zone:{ability}:{id}:{gen}` 為 key 用 `add_stat_buff` 送出。範例：`packages/tamako/rust/No2_burst_arrow.rs` |
| 投射物目標死亡後改鎖（retarget_on_death） | 追蹤彈 `target_lost:TargetLostPolicy::Retarget(半徑)`（1～2000）：目標失效時改鎖投射物當下位置半徑內最近的存活敵方非建築單位（id 決勝），找不到才消失。只限 Homing |
| 無敵召喚物（summon_invulnerable） | summons.lua 寫 `invulnerable = true`：生成時登記到 MobaMatch，所有索敵、投射物、區域與傷害結算都跳過它（只在 MOBA 對局生效）。範例：`packages/tamako/summons.lua` |
| 召喚物鎖定指定目標（summon_forced_target） | 生成後對召喚物 `add_stat_buff(召喚物, buff_ids::SUMMON_TARGET_BUFF, 存活時間, buff_ids::summon_target_payload(目標))`。目標存活時優先選它（host 仍檢查射程、視野、可攻擊性）；目標死亡後回到模板的 target_priority（最近）。召喚物射程要打得到目標，或由自己的 UnitScript 移動。範例：`packages/tamako/rust/No4_hundred_arrows.rs` |
| 生成召喚物的施放（結算規則） | 同一筆施放可以：對本筆剛生成的召喚物上 buff（例如指定目標），以及在**最後一個** spawn 之後，對施法者以外的單位放 buff 或傷害（暈眩、傷害）。spawn 之間插入的效果，或作用在施法者身上的效果，會讓整筆被拒絕，即使 handler 回報 cast ok。所以要先依序 spawn，最後才對其他單位放效果。要用 kit soak 驗證召喚真的出現（E085） |
| 每第 N 次普攻觸發 | 英雄 UnitScript `on_attack_hit`，以 `{技能}_hits{n}:{id}:{gen}` 標記 buff 計數（移除舊 n、加上 n+1，到 N 歸零並觸發）。範例：`No4_hundred_arrows.rs` 的 `on_owner_attack_hit` |
| 光鍊（依距離結算） | `tether::Spec{amount,kind,root,range}`，以光鍊持續時間掛在目標身上；自然到期時結算 |
| 施法縮短其他冷卻 | effect `cooldown_refund_on_cast`。handler 不必處理 |
| 冷卻百分比 | buff payload `cooldown_percentage`（負的分數） |
| 隱形 | buff payload `invisibility_level`（整數層 ×1024），攻擊不會破隱 |
| 無單位碰撞 | `add_buff(caster, ability_runtime::NO_UNIT_COLLISION, 秒)` |
| 控制 | `add_buff(target, BuffId::Stun／Root／Silence.as_rstr(), 秒)` |
| 減速 | buff id `omb_script_abi::buff_ids::generic_slow_buff_id(ID,caster)`，payload `{"move_speed_bonus": -分數raw, "__aggregation_family":"generic_ability_slow"}`；攻速減慢用 `attack_speed_multiplier`（1024 − 分數raw） |
| 位移 | 衝刺用 `crate::swept_dash`（`plan`／`plan_filtered`／`commit`）；瞬移用 `set_pos` 加 `set_facing` |
| 第 5 欄 | `action = "extra1"`，capability `extra_action_slot`；handler 寫法與其他欄位相同 |

### 6.4 英雄 UnitScript（施法、受傷等掛點）

需要「每次施放技能時」「受到傷害時」這類被動時：

1. `character.lua`：`identity.unit_script = true`。
2. `rust/hero_unit.rs`：

```rust
pub struct XUnit;
impl UnitScript for XUnit {
    fn unit_id(&self)->RStr<'_> {RStr::from_str("<hero_id>")}   // 必須等於 hero id
    fn on_skill_cast(&self,caster:EntityHandle,skill:RStr<'_>,_:Target,w:&mut GameWorldDyn<'_>) { super::No3_x::on_owner_cast(w,caster,skill.as_str()); }
}
pub fn hero_unit_script()->XUnit {XUnit}
```

3. 被動的 `on_learn` 要掛一個標記 buff（或用 rank 命名的標記，例如 `x_rank:<r>`），讓掛點以 `has_buff` 判斷是否已學會與 rank。`GameWorld` 無法讀取技能等級，也讀不到 payload。

`on_skill_cast` 在施法交易內、`execute` 之前呼叫；施法被拒絕時整筆交易捨棄。範例：`packages/asai_nagamasa/rust/No3_living_soul.rs`、`packages/oichi/rust/No5_devotion.rs`。

## 7. (E)～(H) 驗證與建置

### 7.1 validate

```bat
tools\lua\lua.exe scripts\hero_author.lua validate docs\character_pipeline\packages\<hero_id>
```

validate 依序檢查：

- schema v2
- 對照 catalog 檢查 provenance
- 對照登錄檢查能力狀態
- 缺少的 handler（以 note 列出）
- 在 scratch 目錄複製 templates，並用 `gen_hero_registry` 做 registry／ids 生成 dry run

成功時輸出 `OK: package valid; kit_status ...`，`package installs N Rust files` 表示 build 時會安裝的檔案數。

### 7.2 shadow：在私人樹測試

```bat
tools\lua\lua.exe scripts\hero_author.lua shadow docs\character_pipeline\packages\<hero_id> --tree D:\omoba\target\<你的名字>\shadow
set CARGO_TARGET_DIR=D:\omoba\target\<你的名字>\cargo
cargo test --manifest-path D:\omoba\target\<你的名字>\shadow\scripts\Cargo.toml -p base_content --lib -- heroes::<RustModule>
```

- shadow 會把需要的 crate 複製到私人樹，並在那裡執行真正的內容生成與 `gen_hero_registry`。UE 建置與 staging 會跳過，不會碰到共用 checkout 或已 stage 的 DLL。
- 再次執行時不會覆蓋私人樹裡已有的檔案（E027）。要重新同步，先刪掉整個目錄。例外是 package 驗證工具（`docs/character_pipeline/tools`），每次都會更新。
- kit 不是全部 `supported` 時，要加 `--slice <supported ability ids>`。只有切片內技能用到的 `rust/` 檔會安裝：只實作切片外技能的檔案不安裝，所以 `needs_framework` 技能的 handler 不必能編譯。test support、`hero_unit.rs`、共用 helper 一律安裝。
- 一律使用自己的 `CARGO_TARGET_DIR`，不要和其他人共用 target。
- 只能用自己啟動的工作控制程式。**禁止用映像名稱 taskkill／Stop-Process**（E061、E063）。

### 7.3 必備測試

每個 rust_handler 檔都附 `#[cfg(test)] mod tests`，至少包含：

- **每個 rank 的數值**：以 outcome 檢查，例如 `RawDamage`、`AddBuff` 的 payload 與 duration，或 `ability_projectile::Spec::from_payload` 解出的封包。
- **拒絕情境**：友軍、建築、超距、錯誤目標型別、無效 rank。fixture 會順便確認拒絕時沒有 outcome。
- **邊界**：半徑端點（例如 335 在內、336 在外）、多目標不重複、目標排序。
- **被動**：學習、升級時取代、不能主動施放。

fixture API（`*_test_support.rs`）：`TestWorld::new()`、`body(pos, team)`、`building(pos, team)`、`handle(e)`、`cast(&H, Target, rank, &rank_json(ID,rank))`、`learn(&H, rank)`、`final_position(&outcomes)`、`p(x,y)`。測試中如果要直接改 world，記得 `use specs::WorldExt;`。

注意：

- 不要依賴英雄目前的技能欄或舊 kit（E044）。
- 前搖結束前不要檢查結果（E042）。
- 期望值要用定點數學推算，不要用理想秒數（E024）。

### 7.4 parity soak（權威端與兩隊 replica 逐 tick 一致）

英雄只要用到投射物、DoT、位移、隱形、冷卻、光環、引導這類會影響 replica 重算的機制，就必須跑 parity soak。soak 放在 package 的 `rust/parity_soak.rs`，build／shadow 會以 `#[cfg(test)] mod parity_soak;` 安裝成 `heroes::<RustModule>::parity_soak`，不必修改共用測試檔。

可用的 helper：

- `crate::single_lane_match_tests::live_soak_replica_parity` 下的 `soak`、`soak_report`、`soak_match`、`cast`、`cast_at`、`attack_move`。
- `crate::single_lane_match_tests` 下的 `three_lane_config`、`hero_pair`。
- 型別與元件來自 `omoba_core::runtime::*`。

最簡單的寫法是用 `soak_report`。它會回傳 mismatch，以及每位英雄每招「權威端冷卻開始或充能被花掉」的次數：

```rust
use crate::single_lane_match_tests::live_soak_replica_parity::{soak_report,cast,cast_at,attack_move};

#[test]
fn <hero>_kit_keeps_team_replicas_identical() {
    let schedule=|tick:u32,pair:[specs::Entity;2],a:omoba_sim::Vec2,b:omoba_sim::Vec2|->Vec<(u32,omoba_core::game_proto::PlayerInput)> {
        match tick {
            t if t%600==6=>vec![(1,attack_move(b)),(2,attack_move(a))],     // 走向對方，進入視野
            t if t%600==300=>vec![(1,attack_move(a)),(2,attack_move(b))],   // 各自回頭，離開視野
            t if t>6 && t%120==10=>vec![(1,cast(0,Some(pair[1])))],         // 對單位
            t if t>6 && t%120==90=>vec![(1,cast_at(2,b))],                  // 對點／方向
            _=>vec![],
        }
    };
    let report=soak_report(["<hero_id>","training_luminary"],7200,&schedule);
    assert!(report.mismatches.is_empty(),"{:#?}",report.mismatches);
    // 有效性：每招真的被施放過，否則什麼都沒發生也會通過。
    for id in ["<q_id>","<w_id>","<e_id>","<r_id>"] {assert!(report.casts_of(0,id)>0,"{id} never cast: {:?}",report.casts);}
}
```

在私人樹以 release 執行：`cargo test --release ... -p base_content --lib -- <hero>_kit_keeps`。

注意：

- **一定要做有效性檢查**。只檢查 mismatch 的話，兩位英雄沒碰面、或每次施放都被拒絕，也一樣會通過。至少要斷言每招 `casts_of(...) > 0`。toggle、被動、沒有冷卻的技能，要改用計數器：在 `soak_match` 的輸入 closure 裡讀 authority 的 `BuffStore`、`Hero`，並累加到 `Cell`。範例：`packages/murakami_yoshikiyo/rust/parity_soak.rs`，它統計光環 buff、暈眩與引導 tick。
- **排程要讓兩人碰面**：例如每秒重新瞄準對方目前的位置（`soak_match` 版本可以讀位置），之後再分開，走出彼此視野。
- **任何新指令都會中斷引導**，包括移動與施法。引導中不要對施法者下新指令。
- **soak 不會呼叫 on_learn**：它直接設定 12 級與每招 rank 1。依賴 on_learn 標記的被動，要在 `soak_match` 的 prepare closure 裡手動放上標記，否則不會出現在 soak 中。

### 7.5 build 與整合

```bat
tools\lua\lua.exe scripts\hero_author.lua build docs\character_pipeline\packages\<hero_id>              :: 只印計畫
tools\lua\lua.exe scripts\hero_author.lua build docs\character_pipeline\packages\<hero_id> --slice a,b  :: 只建 supported 的技能
tools\lua\lua.exe scripts\hero_author.lua build docs\character_pipeline\packages\<hero_id> --execute    :: 整合者才執行
```

`--execute` 的流程：

1. 寫入 `authored_heroes.lua` 與 include hook，並安裝 `rust/`。
2. 執行 `gen_hero_registry`（registry／ids；新增的英雄 UnitScript 由 base_content build.rs 以 `units` 模式產生）。
3. 執行 `build_ue_moba.lua`、staged copy 驗證與 compiled-content contract。

任一步失敗時，所有寫入檔與已建置、已 stage 的 DLL 都會回滾，上一版保持可用。

其他規則：

- `kit_status` 不是 `supported` 時，只能用 `--slice` 建置 supported 的技能。
- 切片缺少中間欄位時，後面的技能會往前補位，birth rank 跟著移動；但 D 欄的位置固定，切片中必須包含全部 Q～R 才能放 D（E059）。
- 在多人共用的 repo 裡，`--execute` 由整合者（主 session）執行。作者交付「validate 通過、shadow 測試通過」的 package 即可。
- 修改共用 Rust／content 必須照著陸流程：先在 shadow 完成，再以「取 shadow 當下的共用樹」為 base 做 3-way merge 套回，不可整檔覆蓋（E062）；兩次著陸至少間隔 3 分鐘；`target/BUILD_FREEZE` 存在時不得著陸。

最後由整合者在 quick map 開一局實機（不超過 10 分鐘），依 `acceptance.md` 逐招觀察。

## 8. 檢查清單

- [ ] export 的每個 current 技能都已對應到欄位，沒有遺漏，也沒有混入 `_old`。
- [ ] 每個 KV／描述衝突、vscript 缺失、基礎數值改編，都記在 assumptions。
- [ ] 每招的 `lifecycle` 五項都已寫，而且 handler 真的做到。
- [ ] `capabilities` 與登錄一致；`needs_framework` 都寫出缺口。
- [ ] Rust 的 extras key 與 §5.3 的命名規則一致，沒有寫死數值。
- [ ] 單元測試涵蓋每個 rank、拒絕情境與邊界；需要時有 parity soak。
- [ ] validate 通過；shadow 測試通過。
- [ ] acceptance.md 寫出實機觀察點。

## 9. 完整範例：H03 阿松（來源 A34 current）

成品在 `docs/character_pipeline/packages/omatsu/`，可以邊讀邊對照。

### 9.1 取得資料

```bat
tools\lua\lua.exe scripts\hero_source_index.lua export --hero A34 --out target\a34.md
```

單位檔 `npc/A_Oda/A34_2.txt` 的技能欄是 A34W、A34E、A34R、magical_resistance（通用被動，不屬於本 kit）、A34T。來源另有 `_20` 變體；本 kit 只用 current，並在 assumptions 記錄 `variant_current_only`。

### 9.2 欄位對應與語意

| 來源 | 本遊戲 | 讀出的機制 |
|---|---|---|
| A34W 寒霜彈 | Q `a34_frost_bolt` | 對敵人或**建築**的追蹤彈，速度 2000；命中時造成 100～400 魔法傷害並結凍 1～2.5 秒 |
| A34E 冰山之襲 | W `a34_iceberg_strike` | 主目標受傷害；335 範圍內其他敵人受冰爆傷害；全部移速 −30%、攻速 −20%，持續 5 秒 |
| A34R 冰霜之息 | E `a34_frost_breath` | 直線穿透（速度 1250、飛行 1200）；命中後 5 秒 DoT |
| A34T 急速冷卻 | R `a34_rapid_cooling` | 被動：施放基礎技能時，其他基礎技能冷卻 −2 秒、法力上限 +300～500；主動：22 秒冷卻 −40～60% |

衝突與改編寫進 `source.lua` assumptions：

- `iceberg_damage_kv`：描述與 KV 的傷害不同，採 KV 數值、描述語意。
- `frost_breath_width`：來源寬度 150→500 漸寬，改為固定平均半寬 162.5。
- `frost_breath_distance`：施法距離 800，但投射物飛 1200，取 1200。
- `frost_breath_dot_kv`：DoT 採 KV 的 50～80。
- `cooling_levels`：R 為 6／12／18 級。

### 9.3 abilities.lua 片段

```lua
{ id = "a34_iceberg_strike", source_ability_id = "A34E", action = "W", ability_type = "active",
  name = "冰山之襲", max_level = 4, status = "supported",
  description = "以冰山轟擊指定敵軍造成 100/200/300/400 魔法傷害，並對其周圍 335 內的其他敵人造成 100 冰爆傷害；被擊中的單位移動速度 −30%、攻擊速度 −20%，持續 5 秒。",
  capabilities = { "unit_damage", "aoe_circle", "slow_debuff" },
  targeting = { kind = "unit", team = "enemy", range = { 700, 700, 700, 700 },
    shape = { kind = "circle", radius = { 335, 335, 335, 335 } }, visibility = "target_must_be_visible" },
  timing = { cast_point = 0.25, cooldown_starts = "on_cast" },
  resources = { cooldown = { 7, 7, 7, 7 }, mana_cost = { 125, 145, 165, 185 } },
  effects = {
    { kind = "damage", amount = { 100, 200, 300, 400 }, damage_kind = "magical" },                       -- damage_amount
    { kind = "damage", amount = { 100, 100, 100, 100 }, damage_kind = "magical", applies_to = "others_in_radius" }, -- damage_2_amount
    { kind = "slow", move_speed_pct = { 30, 30, 30, 30 }, attack_speed_reduction = { 20, 20, 20, 20 }, duration = { 5, 5, 5, 5 } },
  },                                                                                                        -- slow_move_speed_pct／slow_attack_speed_reduction／slow_duration
  lifecycle = lifecycle({ target_invalid = "結算時主目標已死亡或不可見則施放失敗，不扣費" }),
  implementation = { kind = "rust_handler", module = "No2_iceberg_strike" },
  ... }
```

R 的被動減冷卻不必寫 Rust，宣告成 effect 即可：

```lua
{ kind = "cooldown_refund_on_cast", seconds = { 2, 2, 2 } },
```

### 9.4 handler 片段（No2_iceberg_strike.rs）

```rust
let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied().ok_or("missing iceberg extra");
let percent=|v:Fixed64|Fixed64::from_raw((v.raw()+50)/100);           // 30 → 0.3（四捨五入，§5.3）
...
// 全部檢查完才寫入：
world.deal_damage(victim,spec.main,DamageKind::Magical,DamageProfile::COLD,RSome(caster));
for e in &splash {world.deal_damage(*e,spec.splash,DamageKind::Magical,DamageProfile::COLD,RSome(caster));}
let slow_key=omb_script_abi::buff_ids::generic_slow_buff_id(ID.as_str(),caster);
world.add_stat_buff(e,slow_key.as_str().into(),spec.duration,move_payload.as_str().into());
```

Q 的建築目標與 E 的命中 DoT 都直接用 G-1 投射物封包的欄位：

```rust
host::Spec { motion:host::Motion::Homing{..}, ..., control:Some((ControlEffectKind::Stun,stun)), dot:None, buildings:true }
host::Spec { motion:host::Motion::Linear{aim}, ..., dot:Some((rate,duration)), buildings:false }
```

### 9.5 測試

```rust
#[test]
fn every_rank_hits_the_target_splashes_others_once_and_slows_all_hit() {
    for (rank,main) in [(1u8,100),(2,200),(3,300),(4,400)] {
        let mut t=TestWorld::new();
        let target=t.body(p(400,0),2);let near=t.body(p(400,335),2);let far=t.body(p(400,336),2);let ally=t.body(p(400,10),1);
        let outcomes=t.cast(&IcebergStrikeHandler,Target::Entity(t.handle(target)),rank,&rank_json(ID,rank)).unwrap();
        // 主目標 rank 傷害、335 端點受冰爆 100、336 與友軍不受影響、移速 −307 raw、攻速 ×(1024−205)
        ...
    }
}
```

parity soak：`omatsu_homing_slow_dot_and_cast_refund_keep_team_replicas_identical`（7200 tick，兩隊 replica 逐 tick 比對），加上 `omatsu_basic_cast_shortens_other_basic_cooldowns`（冷卻 −2 秒，且保留 admitted total）。

### 9.6 執行結果

```text
> tools\lua\lua.exe scripts\hero_author.lua validate docs\character_pipeline\packages\omatsu
OK: package valid; kit_status supported
package installs 7 Rust files into scripts/base_content at build

> tools\lua\lua.exe scripts\hero_author.lua shadow docs\character_pipeline\packages\omatsu --tree ...\shadow
OK: shadow build complete.

> cargo test ... -p base_content --lib -- heroes::A34_omatsu
test result: ok. 8 passed

> cargo test --release ... -- omatsu_
test result: ok. 2 passed
```

驗收觀察點見 `packages/omatsu/acceptance.md`。

## 10. 失敗診斷

| 訊息或現象 | 原因 | 修法 |
|---|---|---|
| `SchemaError: <file> is required for schema v2` | 缺少必要檔案 | 補齊 §5 的七個檔案 |
| `source ability X raw_sha256 differs from catalog (stale package)` | hash 抄錯，或來源版本不同 | 從 export 重新複製 hash |
| `ability X ... status supported but capabilities derive needs_framework` | status 與 capabilities 推導不一致 | 改 status，或在 capabilities.lua 修正狀態 |
| `capability X is needs_framework here but supported in the D3 registry (add override_reason)` | 登錄已更新，package 的狀態過期 | 改成登錄的狀態；確定不同時加 override_reason |
| `birth rank 1 is not learnable at hero level 1`（E029） | 大招或高等級門檻的技能出生 rank 為 1 | moba_loadout 該欄改為 0，或直接省略 moba_loadout |
| `uses extra1 but does not list capability extra_action_slot` | D 欄沒有宣告能力 | 在 capabilities 加上 `extra_action_slot` |
| `the extra1 (D) slot needs all four basic slots in the build`（E059） | 切片沒有包含全部 Q～R | 完整建置，或先不建 D |
| `kit is needs_framework; build a thin slice with --slice ...` | kit 尚未全數支援 | 用 `--slice` 只建 supported 的技能 |
| `X_ffi not exported from .../mod.rs` | `rust/` 缺少該模組，或函式名稱不是 `<ability_id>_ffi` | 檔名、`implementation.module` 與函式名稱三者要一致 |
| `GenerationError: ... missing from template ids` | 技能沒有進入 loadout（舊版 extra 欄的行為） | 確認 action 與切片內容 |
| cargo：`missing extra` panic 或 `Err("missing ... extra")` | extras key 拼錯 | 依 §5.3 規則推出 key，或讀 `authored_heroes.lua` 確認 |
| cargo：`no method named write_storage/write_resource` | 測試沒有匯入 `specs::WorldExt` | 在 tests 模組加 `use specs::WorldExt;` |
| cargo：`missing fields ... in initializer of ability_projectile::Spec` | 投射物封包新增了欄位 | 補上 `dot: None, buildings: false`（或需要的值） |
| `rejected cast emitted outcomes`（fixture 斷言失敗） | 檢查失敗前就呼叫了會寫入的 API | 把所有檢查移到第一個寫入之前 |
| 測試預期在範圍外的單位卻被命中（E030） | 預期用了錯誤的幾何（例如端點包含、半徑加上碰撞半徑） | 依 handler 實際使用的幾何函式推算 |
| 新增方法到 `GameWorld` 後整個 workspace 編不過（E045） | sabi 方法表已滿 | 改用 host payload 封包（§6.3），或回報缺口 |
| parity soak：`tick N team T: ...` 不一致 | 只有權威端算出的狀態沒有揭露給 replica，或 replica 無法重算（E053、E056） | 把狀態放進 host 結算（content／封包），不要只放在 handler 的權威端分支 |
| 共用樹同步後自己的修改消失（E027） | 重新同步覆蓋了私人樹 | shadow 不會覆蓋既有檔案；要完全重來就刪掉整個目錄 |
| `write failed: .../heroes/<Module>/...: No such file or directory`（E058） | 舊版 hero_author 無法建立新模組目錄 | 更新工具（已修正） |
| Lua 長字串被程式碼裡的 `]]` 提早結束（E026） | 在 Lua 字串裡寫 Rust 或 Lua 時遇到 `]]` | 改用 `[==[ ... ]==]` |
| 改 CRLF 檔後整檔行尾改變，或比對失敗（E025、E061） | Git Bash 的 `sed -i` 會把 CRLF 改成 LF | 不要用 `sed -i` 改原始碼。用 Edit 工具，或用 `tools\lua\lua.exe scripts\text_patch.lua <spec.lua> [--dry]`：保留每個檔的行尾；比對時忽略 CR；全部比對成功才寫入；Lua 字串用 `[==[ ... ]==]` |
| `ability 'X': channel ...`（content 驗證） | `timing.channel` 與 targeting 不一致，或缺 interval／pulses／radius | anchor 要與 targeting.kind 一致；self／point 錨點要有半徑；每波 effect 種類不能重複 |
| 引導技施放後什麼都沒發生（soak 的 channel tick 為 0） | 排程在引導中下了新指令，引導被中斷 | 引導中不要對施法者下指令（§7.4） |
| soak 通過，但 `casts_of(...)` 是 0 | 兩人沒碰面、法力不足，或施放都被拒絕 | 讓排程追著對方目前位置；法力不足時降低施放頻率；用計數器斷言（§7.4） |
| 光環沒有作用在友軍或自身 | 自行查詢不到友軍 | 用 `ally_area_buff` 封包（§6.3），不要用 `query_enemies_in_range` |
| 只打英雄與小兵的技能打到塔 | `query_enemies_in_range` 會回傳建築 | 用 `is_building` 過濾 |
| 在 stats 寫了不存在的 key（例如自創 `max_hp`） | stats 是封閉詞彙 | 用 §5.3 列出的 key；最大生命是 `max_hp_bonus` |
| 資料檔裡呼叫 `pairs`／`ipairs` 失敗（E016） | 沙盒環境是空表 | 已開放 `pairs`；其他 helper 請寫成區域函式 |
| 輸出順序每次不同（E005） | 依賴 `pairs()` 的順序 | 先排序 key 再處理 |
| package 宣稱「只鎖定可見敵人」，實機卻會打霧中目標（E036） | lifecycle 寫的規則 handler 沒有做到 | handler 加上相同的篩選（例如 disclosed 查詢），並補測試 |
