# v3 在 omoba-content-model 的表示

狀態：**§2 catalog 格式已定案（2026-10-28，M1 工作線 A）**；§4 的 IR 仍是提案（M3）。UE TooltipRenderer 與網站產生器依 §2 實作；格式有改動時更新本文件，並在 `docs/plans/2026-10-24-omfue-month-mvp/execution-status.md` 記錄。

## 1. 現況與目標

舊的 `AbilityDefinition` 有三個問題：`levels` 與 `extras` 都是 f32，不決定性；`description` 寫死數字，和數值脫鉤；每多一種機制就多一個專用欄位。

M1 的做法是**新舊並存**：

- 每個技能列與道具列多一個 `v3` 區塊，內容是數值、說明模板與資訊列，由 v3 文件編譯而來。
- 舊欄位保留，但改成由 v3 產生：`extras` 取自 values，`levels` 取自資訊列，`description` 是 zh_TW 模板代入所有 rank 的結果。舊欄位標為 deprecated，給還沒改用 v3 的 UI 與 Rust handler。
- 行為仍由 Rust handler 執行（`coverage = "rust_handler"`），M3 才改成 IR。

## 2. catalog 格式（定案）

### 2.1 資料流

```text
v3 文件（docs/character_pipeline/packages/<hero>/v3.lua、docs/character_pipeline/items/*.lua；tools/v3_sources.lua）
   │  tools/package_v3.lua 驗證 → tools/v3_compile.lua 編譯
   ▼
scripts/lua_data/templates/authored_heroes.lua   技能列的 v3 = { ... }（hero_author build 產生）
scripts/lua_data/templates/item_texts_v3.lua     道具 id → v3 區塊（scripts/gen_item_v3.lua 產生）
   │  moba_items.lua 把區塊掛到道具列的 v3，description 換成渲染結果
   ▼
template manifest（content hash = CONTENT_CATALOG_DATA_HASH，涵蓋全部 v3 區塊）
   ├─▶ omoba-content-model：AbilityDefinition.v3、MobaItemEntry.v3（型別 V3Block，載入時驗證）
   ├─▶ omoba-template-ids：V3_CATALOG_JSON 常數（含 content hash）
   │      └─ cargo run -p omoba-template-ids --example export_content_catalog -- <out.json>
   │         → 網站產生器讀這個 JSON
   └─▶ omfue/codegen → UE（bridge 內嵌同一份 manifest）
```

### 2.2 V3Block（技能與道具共用）

Lua（lua_data 裡）與 JSON 的欄位相同：

```lua
v3 = {
  schema = "3.0",
  kind = "ability",                 -- ability | item
  source_id = "C17W",
  max_level = 4,
  coverage = "rust_handler",        -- expressible | needs_hook | pending | rust_handler
  handler = "No1_light_chain",      -- coverage = rust_handler 時，執行行為的 Rust 模組
  values = {                        -- 依名稱排序
    { name = "damage", unit = "dmg", damage_type = "magical",
      ranks = { "350", "520", "690", "860" },          -- 作者寫的十進位或分數字串，長度 = max_level（per = hero_level 時 25）
      raw = { 358400, 532480, 706560, 880640 } },      -- Fixed64 raw（SCALE 1024），由字串以整數運算換算
    { name = "tick", unit = "s", hidden = true, ranks = { "0.2", ... }, raw = { 205, ... } },
    { name = "agi_damage", unit = "dmg", derived = true, expr = "caster.agi * $agi_mul" },  -- 執行期才知道的值：沒有 ranks
    { name = "radius", unit = "dist", ranks = {...}, raw = {...}, v2_key = "shape_radius" },
  },
  info = {                          -- 資訊列，依固定順序；value 指向 values（owner 有值時指向別的擁有者），否則是 expr
    { field = "cast_range", value = "cast_range" },
    { field = "cooldown", value = "cooldown" },
  },
  stat_lines = {                    -- 只有道具：被動屬性加成行
    { stat = "attack_damage", value = "attack" },
  },
  statuses = {                      -- 這個技能或道具擁有的狀態（buff 圖示與說明用）
    { id = "c17w_guard", visibility = "buff", values = {...}, text = {...} },
  },
  text = {
    zh_TW = {
      name = "光鍊罰則",
      body = "……造成 {damage} 魔法傷害……",                -- 原始模板
      pieces = {                                            -- 已解析，消費端只代入與格式化
        { text = "……造成 " },
        { value = "damage" },                               -- format 省略 = all
        { value = "root", format = "current" },
        { value = "radius", owner = "c17_divine_strike" },  -- 其他擁有者的值
        { name_of = "c17_devotion", text = "自我奉獻" },    -- 已代入同語言名稱
        { rules = "hero.base_magic_resistance", text = "30" },
      },
      rendered = "……造成 350/520/690/860 魔法傷害……",      -- 代入所有 rank 的結果（網站、舊 description）
    },
    en = { ... },
  },
}
```

規則：

- **數值**：
  - `ranks` 是顯示用的數字（`pct` 的 25 表示 25%）。`raw` 是同一個數字的 Fixed64：預設四捨五入，`rounding = "truncate"` 改為截斷。
  - 行為裡 `pct` 讀成比例（v3.md §2.1），這個換算由 M3 的 IR 編譯器做；catalog 只存顯示數字。
- **格式化**（`format`，v3.md §9）：
  - `all`：各 rank 用 `/` 連接，全部相同時只顯示一個；`current`：只顯示目前 rank；`signed`：正數加 `+`；`abs`：取絕對值；`int`：四捨五入成整數；`max`：最後一個 rank。
  - `pct` 在最後加一個 `%`（`5/6/8/11%`）。小數最多兩位，去掉尾端的 0。
  - `current` 在網站上和 `all` 相同；遊戲內依目前等級顯示，目前等級要加粗由 UI 決定。
- **資訊列 `field`**：
  - 技能：`cast_range`、`aoe_radius`、`cast_point`、`channel_time`、`mana_cost`、`health_cost`、`gold_cost`、`cooldown`、`charges_max`、`charge_restore`、`required_level`；
  - 道具：另有 `cost`、`recipe_cost`。
  - 顯示文字（例如「冷卻」「Cooldown」）由 UI 與網站各自翻譯，catalog 不帶。
- **說明**：
  - `pieces` 的 `text` 段保證沒有數字（驗證器與 content-model 都檢查）；
  - `rendered` 只給沒有等級概念的地方（網站的總覽、舊的 description）。
- **owner**：`pieces`、`info`、`stat_lines` 的 `owner` 一律是 v3 文件裡的 id，例如技能 `c17_divine_strike`、道具 `item_fire_scroll`、狀態、單位模板，以及來源目錄的 `<文件 id>.shared`。
  - 技能的遊戲 id 和 v3 id 相同。
  - 道具的遊戲 id（catalog `items[].id`）是 v3 id 去掉 `item_` 前綴，例如 `fire_scroll`；45 件上架道具全部成立。
  - 消費端以 v3 id 查值時，道具請先補上 `item_` 前綴。
- **content hash**：`CONTENT_CATALOG_DATA_HASH` 是整個 template manifest 的 hash，網站頁尾與遊戲設定頁顯示同一個值。

### 2.3 舊欄位怎麼產生（deprecated）

| 舊欄位 | 來源 |
|---|---|
| `description` | `text.zh_TW.rendered` |
| `extras[<v3 名稱>]` | 每個有 `ranks` 的 value（顯示數字，f32）。Rust handler 改讀這些名稱 |
| `extras[<v2_key>]` | 有 `v2_key` 的 value 另外以舊鍵名輸出，只給 host 契約欄位用（`charges.max_key`、`channel.interval_key`、`targeting_shape.radius_key` 等）。需要轉換的照舊轉換（錐形半角度→弧度、法球緩速百分比→比例） |
| `levels[i].cooldown/mana_cost/cast_time/range/required_hero_level` | 資訊列的 `cooldown`、`mana_cost`、`cast_point`、`cast_range`、`required_level` |

`hero_author build` 會額外檢查：

- v2 `abilities.lua` 裡同一個數字和 v3 不同時，建置失敗。舊的數字因此不會悄悄分歧，到 M5 才刪掉。
- 英雄 `rust/*.rs` 用 `extra*("key")` 讀的每個鍵，都必須存在於產生的 extras，漏改名會在建置期就被擋下。

### 2.4 道具

- 道具的 v3 文件是 `docs/character_pipeline/items/*.lua`（`kind = "items"`），涵蓋原作全部商店道具；上架的道具由商店決定，其餘只進來源目錄。
- `scripts/gen_item_v3.lua` 把它編成 `templates/item_texts_v3.lua`。
- 道具玩法數值仍讀 `templates/source_items.lua`（由原作 KV 匯出）；v3 道具值的 `kv:` 出處由出處比對工具對原作 KV 檢查，所以兩者不會分歧。

### 2.5 全量來源目錄（網站用，M3 工作線 A）

- `scripts/gen_source_catalog.lua` 從正式位置的 v3 文件（97 份英雄 `packages/<package>/v3.lua`、`items/*.lua`）產生 `target/site/source-catalog.json`，涵蓋 97 位英雄與全部商店道具，不論遊戲是否上架。**不進遊戲的 content**，正式選角與 `V3_CATALOG_JSON` 仍然只放已驗收的英雄。
- 格式：`{ schema, kind = "source", source = { repository, commit }, catalog_hash, heroes = [ { source_id, name = { zh_TW, en }, playable, game_id?, abilities } ], abilities = [ { id, hero, slot, display_name, max_level, v3 } ], items = [ { id, source_id, name, cost?, deferred?, note?, v3? } ] }`；`v3` 與 §2.2 相同（沒有 `ir`）。
- `playable`：`docs/character_pipeline/packages` 有對應的 package（`identity.source_hero_id`）；網站把其他英雄標成「未上架」。已上架英雄的說明以 `V3_CATALOG_JSON` 為準（遊戲實際數值）。
- `catalog_hash`：這份目錄 JSON 的 sha256 前 16 碼，與遊戲的 content hash 分開。

## 3. 移除的欄位（M5）

| 舊欄位 | v3 取代 |
|---|---|
| `description` | `text` |
| `levels[].cooldown/mana_cost/cast_time/range/required_hero_level` | `info` 引用 `values` |
| `extras` | `values` |
| `effects`、`projectile`、`charges`、`summon`、`attack_orb`、`channel`、`zone_lifetime`、`cast_cooldown_refund` | behavior IR |
| `targeting_shape`、`targeting_reach_key`、`aim_geometry` | `target`（`aoe_radius`、`vector_length`）＋ selector 形狀推導指示器 |
| `pierces_magic_immunity`、`unit_target`、`auto_learn` | `flags`、`target.team`、`hero.auto_learn` |

## 4. behavior IR（M3，提案）

> **已取代（2026-10-29）**：catalog 不再帶 behavior IR。行為改在建置期產生 Rust，見 `docs/plans/2026-10-28-history-nobu-full-port/m3-rust-codegen-design.md`。本節只保留作歷史紀錄；Lua 端的 IR 樹可以當成產生器的內部表示。

- 平面化：每個 program 是一段指令陣列；action 與控制流是 opcode，參數是 value slot 索引、Expr 索引或常數 0／1／-1。
- 事件訂閱表：`(event kind, status id) -> program entry`，執行器只處理有訂閱的單位。
- 名稱全部在建置期解析成索引；執行期不查字串、不配置記憶體。
- hook：`Hook { name_id, args: Vec<ExprIdx> }`，名稱對應 base_content DLL 註冊的函式。
- `pct` 的比例換算、衍生值的執行期運算式，都在 IR 編譯器處理。
- **M3 協調註記（工作線 B，2026-10-29）**：IR 放在 V3Block 的新欄位 `ir`（可選；`coverage = rust_handler` 的條目沒有）。型別由工作線 B 在 `omoba-content-model/src/ir.rs` 定義，V3Block 只多一行 `#[serde(default, skip_serializing_if = "Option::is_none")] pub ir: Option<IrBlock>`；Lua 端由 `docs/character_pipeline/tools/v3_ir.lua` 產生，`v3_compile.lua` 在有 behavior 時呼叫它。格式與執行器見 [m3-ir-design.md](../../plans/2026-10-28-history-nobu-full-port/m3-ir-design.md)。M3 期間狀態另有 `runtime_id`（沿用 handler 的 buff id，讓重播 hash 相同），M5 移除。
- **v3.2 協調註記（工作線 B，2026-10-29）**：`from_ability` 由規則層依 BuffStore 記錄的來源（技能／道具／host 與施加者）在讀取時推導，資料不必宣告。`unit_state` 的執行需要宣告：請在 IrBlock 加 `unit_state = [{ name, type, init, scope, reveal }]`，列出區塊讀寫的每個變數（跨文件的從登錄表帶入），`scope = "match"` 是全場一份。設計見 [m3-ir-design.md](../../plans/2026-10-28-history-nobu-full-port/m3-ir-design.md) §7。
- **host 讀值（工作線 A，2026-10-29，應 B 的 B04 切換要求）**：條目可以寫 `host = { component, reads }`，或寫成這種表的陣列。它宣告 coverage 為 expressible 時，host 元件以固定名稱從 catalog 讀的值，例如 attack_orb 的 extras、ability_projectile 的上限、flame_trail 的半徑與壽命。
  - 驗證器把 reads 算成有人用；catalog 與 IR 都不帶這個欄位，host 照常讀 values。
  - `component` 的清單在 v3_vocab 的 `HOST_COMPONENTS`，新元件要先加進清單。規則見 v3.md §12.12。

### 4.1 IR 格式（草案 v1，工作線 A 與 B 對齊中，2026-10-29）

分工：Lua 編譯器 `docs/character_pipeline/tools/v3_ir.lua` 由工作線 A 負責；`omoba-content-model/src/ir.rs` 的型別與 omoba-core 直譯器由工作線 B 負責。以 [m3-ir-design.md](../../plans/2026-10-28-history-nobu-full-port/m3-ir-design.md) §3 為底，補齊細節如下。

**放置**：`V3Block.ir`。只有文件裡有行為的條目才有（`events`、`passive`、`statuses` 其中之一）；`coverage = rust_handler` 的條目沒有。

```lua
ir = {
  version = 1,
  slots = { { name = "damage" }, { name = "slow", scale = "pct" }, { name = "radius", owner = "c17_divine_strike" } },
  exprs = { { { "slot", 1 } }, { { "name", "it" }, { "attr", "base_magic_resist" }, ... } },
  selectors = { { from = 3, shape = "circle", radius = 4, team = "enemy", types = { "hero", "creep" }, flags = {...},
                  where = 5, order = "id", limit = 6, prefer = 7, post_filter = 8, take_while = 9 } },
  programs = { { { op = "damage", to = 2, amount = 6, type = "magical" }, { op = "cue", id = "x", at = 2 } }, ... },
  locals = 3,                                   -- 本區塊用到的區域變數數量（let、for_each、repeat、回呼綁定）
  entries = { on_cast = 1, on_phase_start = 2 },  -- 技能事件 → 程式索引
  passive = { stats = {...}, states = {...}, triggers = { on_attack_hit = { program = 3, filter = 7 } }, aura = {...} },
  statuses = { { id = "c17w_guard", runtime_id = nil, duration = 1, stacking = "refresh", dispel = "basic", visibility = "buff",
                 reveal = "public", stats = { { stat = "incoming_damage_pct", op = "add", expr = 2, when = nil, per_stack = nil } },
                 states = { "root" }, vars = { { name = "absorbed", type = "fixed", init = 4 } },
                 triggers = { on_created = { program = 5 }, periodic = { program = 6, interval = 7, first = nil } },
                 absorb = {...}, aura = {...}, charges = 8, ... } },
  routines = { c01e_wave = { program = 9, params = { "center", "amount", "stun" }, locals = { 1, 2, 3 } } },
  templates = { b01_saika_corps_gunner = { stats = { hp = 10, ... }, attack = {...}, ai = "...", ... } },
  hooks = { "b11_bow_range" },
}
```

規則：

- **索引從 1 開始**（Lua 陣列）。所有 `expr`、`program`、`selector` 欄位都是本區塊陣列的索引；Rust 端載入時轉成 0 起算。
- **slot**：行為讀到的每個 `$名稱` 在本區塊只佔一格，依第一次出現的順序編號。`owner` 不是本條目時，指向別的擁有者（`$c17_divine_strike.radius`）。`scale = "pct"` 表示運算式讀到的是比例，載入時除以 100（v3.md §12.1）；`rounding = "truncate"`（值定義有寫時才出現）表示這次除法截斷，否則四捨五入。數值本身仍在 `V3Block.values`（或對應擁有者的 values）。
- **運算式**：後序（stack machine），每個運算式是一串 token，token 是小陣列：
  - `{"slot", i}`：目前 rank 的值；`{"raw", n}`：Fixed64 raw 常數（結構常數 0／1／-1，以及建置期已知的 `rules.*`）；`{"bool", true}`；
  - `{"name", "caster"}`：根名稱，`caster`、`caster_hero`、`target`、`point`、`direction`、`vector_end`、`self`、`source`、`ability`、`game`、`event`；
  - `{"local", k}`：區域變數（`let`、`for_each` 的 `it`、`repeat` 的 `as`、回呼的 `it`、`here`、`jump`、routine 參數）；
  - `{"state", "c01r_noncrit"}`、`{"var", "absorbed"}`：unit_state 與狀態 vars；
  - `{"attr", "base_magic_resist"}`：取堆疊頂端的屬性或紀錄欄位（`event` → `attack` → `crit` 就是兩個 attr）；
  - `{"sym", "c01r_charge"}`：名稱參數（狀態 id、tag、道具 id、地標、指令種類），給下一個 call；
  - `{"call", "stacks", 2}`：函式與參數個數；
  - `{"op", "+"}`：`+ - * / % == ~= < <= > >= neg not`；
  - `{"jump_if_false", k}`、`{"jump", k}`：相對跳躍（往後 k 個 token），用在 `and`、`or`、`select`，只求值被選中的分支。
- **程式與指令**：程式是指令陣列；指令是具名欄位的 table，`op` 是 action 名稱（v3 詞彙表），其他欄位照 action 參數：
  - 運算式參數 → 運算式索引；`actions`、`otherwise`、`on_hit` 等巢狀 action → 程式索引；`select` → selector 索引；
  - `status`、`template`、`ability`（宣告的 id）保留字串，載入時解析成索引；
  - 列舉、布林、cue id 照原樣；`as`、`let.name` 換成區域變數索引 `bind`；
  - 控制流 `if`、`for_each`、`repeat`、`delay`、`chance`、`random_pick` 以巢狀程式索引表達，不用跳躍（程式陣列本身仍是平面的）。
  - 沒有的參數省略（Lua 空 table 一律省略）。
- **狀態**：本條目擁有的狀態，欄位照 v3 狀態，運算式參數換成索引，觸發換成 `{ program, filter, interval, first, who, phase }`。`runtime_id` 只在 M3 期間出現（由 B 的切換清單提供）。
- **routine**：本區塊用到的 routine 編進 `routines`，參數依序綁到 `locals`。
- **單位模板**：本區塊建立的模板，屬性換成運算式索引（`$x` 讀模板自己的 values，slot 帶 `owner = <模板 id>`）。
- **hook**：用到的 hook 名稱；指令 `{ op = "hook", name, args = { 參數 = 運算式 } }`。
- **target**（技能與道具主動，施放驗證用）：`{ type, team, types, flags, ability_flags, range, min_range, aoe_radius, vector_length }`。
  - `type`、`team`、`types`、`flags` 照 v3 的 `target`；`ability_flags` 是技能（或道具 `active`）的 `flags`，例如 `no_target_self`、`can_target_buildings`、`pierce_magic_immune`。
  - `range` 等距離是運算式索引。
  - 沒寫的欄位省略。
  - 驗證不通過時不扣成本、不進冷卻，與 handler 相同（工作線 B，2026-10-29）。
- **unit_state**（m3-ir-design.md §7.2）：`[{ name, type, scope, reveal, init }]`，依名稱排序。
  - 內容是本區塊讀寫的每個變數，加上本條目自己宣告的變數。來源包括：
    - `{"state", n}` token；
    - `set_state`／`clear_state` 的 `var`；
    - `chance` 的 `counter_var`。
  - 宣告依序找：本條目、文件層、同文件其他條目（routine 的 owner），最後找登錄表。
  - `init` 是運算式索引，沒寫就是該型別的零值（0、false、none）。
  - 跨文件同名的變數宣告一致：`tsum` 在 B27 與 B30 都是 fixed／match／public。
  - 目前是關閉的：`v3_ir.lua` 的 `M.EMIT_UNIT_STATE = false`，或用 `opts.unit_state` 打開。等 ir.rs 的 IrBlock 接受這個欄位才打開。

編譯失敗的條目不產生 `ir`，`hero_author`／`gen_item_v3` 會列出原因（需要直譯器支援但詞彙不認得的部分）。

## 5. 順序

1. M1：V3Block、extras／levels／description 由 v3 產生、網站與 TooltipRenderer 讀 v3。不動行為。
2. M2：status 的 `reveal` 與規則層。
3. M3：IR 編譯器（Lua）＋直譯器（omoba-core）；阿市、雜賀孫市切換，重播 hash 相同。
4. M5：刪舊欄位與對應 handler。

每一步都要依既有規則升 content／presentation／CABI 版本，舊版 fail closed。
