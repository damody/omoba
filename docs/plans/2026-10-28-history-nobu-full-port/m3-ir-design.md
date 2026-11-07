# M3 設計：behavior IR 與直譯器

> **已取代（2026-10-29）**：使用者決定不用執行期 IR，改成由 v3 產生 Rust，每位英雄一個 DLL，可以熱加載（README D15、D16）。新設計見 [m3-rust-codegen-design.md](m3-rust-codegen-design.md)。本文件只保留作歷史紀錄；其中呼叫 host 元件的對應與揭露政策，會改寫成 `ability_kit`。

建立日期：2026-10-29。負責：工作線 B（規則層、執行器核心 session）。狀態：設計定案中，B01／C17 實作進行中。

對應 README §5 的 M3 與 [v3-content-model-proposal.md](../../character_pipeline/schema/v3-content-model-proposal.md) §4。

## 1. 目標與驗收

- 阿市（C17）與雜賀孫市（B01）的技能完全由資料執行：v3 文件 → 建置期 IR → omoba-core 直譯。
- 用既有實機局的 accepted inputs 重播（`hero-port-8/live-8`，陣容含 oichi 與 saika_magoichi），IR 版與 Rust handler 版**每 tick 的狀態 hash 相同**。
- `run_10000`（或同等的 headless 量測）效能不退步。

## 2. 為什麼直譯器呼叫 `GameWorld` API

- 現行 Rust handler 都實作 script-abi 的 `AbilityScript`（`execute`、`on_attack_hit`、`on_learn`……），效果全部經 `GameWorldDyn`（`add_stat_buff`、`deal_damage`、`spawn_summoned_unit`、tether／reveal 的 envelope……）變成 `Outcome`。
- IR 直譯器實作同一個 trait（`omoba_core::runtime::native::ir::IrAbility`），用同樣的 world 呼叫。輸出的 Outcome 與 handler 相同，所以重播 hash 可以逐 tick 比對；host 的施法交易、冷卻、法力、揭露流程完全不用改。
- 切換方式：base_content 的 `<ability>_ffi()` 由 `build_ability_ffi(ID, Handler, …)` 改成 `build_ability_ffi(ID, IrAbility::new(ID)?, …)`。一招一招切換（計畫 D10）。

## 3. IR 格式

以 [v3-content-model-proposal.md](../../character_pipeline/schema/v3-content-model-proposal.md) §4.1 為準（工作線 A 的 `v3_ir.lua` 產生，`v3_compile.lua` 的 `M.EMIT_IR` 打開時寫進 V3Block 的 `ir`）。重點：

- 索引從 1 起算；運算式是後序 token 陣列（`["slot", 1]`、`["name", "event"]`、`["attr", "target"]`、`["jump_if_false", 2]`……），`select`、`and`、`or` 用相對跳躍。
- slot 只帶名稱、`owner`、`scale = "pct"`、`rounding = "truncate"`；數值本身在 V3Block 的 `values`（或擁有者的 values），載入時取 raw，`pct` 除以 100（有 truncate 就截斷，否則四捨五入）。
- 程式是指令陣列，指令是開放的 map（`op` 加參數，運算式與程式參數是索引）；`entries`（技能事件 → 程式）、`passive.triggers`、`statuses`、`selectors`、`routines`、`templates`。
- content-model `ir.rs`（工作線 B）：頂層欄位固定，指令、狀態、模板保持開放的 JSON map；只檢查版本、token 形狀與索引範圍。
- **`runtime_id`**：M3 期間由 base_content 的切換清單（`ir_switch.rs`）提供，把 v3 狀態 id 換成 handler 的 buff id，讓重播 hash 相同；M5 刪 handler 後移除。

## 4. 執行器

`omoba-core/src/runtime/native/ir.rs`（之後依大小拆成子模組）：

| 部分 | 內容 |
|---|---|
| 載入（`Program::load`） | slot 取值與 pct 換算、token 解析成列舉、指令解析、狀態 stat 經 `rules::vocab` 換成 StatKey；程式用到的運算式必須全部可執行，否則那一招載入失敗 |
| 運算式（`Program::eval`） | 後序 stack machine（固定大小的 stack，不配置記憶體）；值型別：數值、布林、單位、位置、`event` |
| 指令（`Program::run`） | `apply_status`、`damage`、`create_unit`、`cue`（呈現，直譯器略過）、`if`、`let`、`call`（routine）、`for_each`、`tether`、`reveal` |
| `IrAbility` | 實作 `AbilityScript`：`execute`（`entries.on_cast`）、`on_attack_hit`、`on_learn`；施放時照 `target`（type、team、types、`no_target_self`、`self_cast_allowed`、射程）拒絕，與 handler 相同 |
| `IrHeroUnit` | 通用的英雄 UnitScript：`on_skill_cast` 轉給 IR 被動的 `on_ability_executed`（取代阿市專屬的 Rust UnitScript） |
| 可達性 | 只有 entries、觸發、routine 與它們可達的程式要能執行；host 元件自己結算的程式（光鍊的 `on_expire`、定身的 `on_created`）不直接跑 |

### 4.1 host 元件的對應

v3 的動作語意已經有 host 元件實作（計畫 §3.1「重用現有元件」）。直譯器在載入時辨識固定的形狀，執行時呼叫那個元件；形狀不符就載入失敗：

| v3 寫法 | host 元件 | 為什麼 |
|---|---|---|
| `for_each` 敵方圓形 | `query_enemies_in_range`（依揭露視野），依 entity id 排序 | 和 handler 相同的選取與排序 |
| `for_each` 友方圓形，本體只有 `heal`／`restore_mana`／`set_health` 最大值比例 | `ally_area_restore` | world API 沒有友方查詢；host 在串列結算時依中心選取同隊存活單位並揭露 vitals |
| `tether` 的 `on_expire` 是「依距離比例傷害＋依距離內插定身（定身時另受傷害）」 | `tether` | 連結的生命週期、邊緣距離、施法者死亡後的處理都在 tether.rs |
| `reveal` | `reveal_zone` | 視野只在 authority 決定 |

之後的排程器（`delay`、`repeat`、狀態 `periodic`）與一般 selector 會讓這些形狀改成直接執行。

- 狀態的施加：`stats` 依 `rules::vocab` 換成 StatKey payload，與 handler 的 payload 鍵與數值相同；`states` 換成 BuffStore 的狀態 id。
- 狀態觸發（`on_created` 等）在施加後依序執行；`periodic` 由 M3 第二批的排程器處理。
- 事件分派表：載入時建立「事件 → (技能, 入口)」；只對有訂閱的單位處理（計畫 §3.2 第 2 點）。
- 決定性：沒有雜湊表走訪；selector 依 entity id 排序；擲骰用 `rules::chance::roll_key`。

## 5. 揭露政策

- 狀態的 `reveal` 載入時保留；直譯器不讀對方隊伍 `team` 狀態做會影響對方的判定（m2-disclosure-policy.md）。
- M3 先在載入時記錄每個狀態的政策；判定的靜態檢查在第二批加入。

## 6. 進度

| 項目 | 狀態 |
|---|---|
| 設計（本文件） | 完成（2026-10-29） |
| content-model `ir` 欄位 | 完成：`IrBlock`（頂層固定欄位，含 `target`、`requires`） |
| Lua IR 編譯器 | 工作線 A 完成（`v3_ir.lua`）；`v3_compile.lua` 的 `M.EMIT_IR` 隨本批打開 |
| omoba-core 直譯器 | 完成第一批（見 §4） |
| B01、C17 切換 | `scripts/base_content/src/ir_switch.rs`：九招與阿市的 UnitScript 由 IR 執行 |
| 重播比對 | `ir_replay_matches_rust_handlers_every_tick`（實機局 accepted inputs）、`ir_scripted_casts_match_rust_handlers_every_tick`（每招都施放）；結果見 execution-status |

## 7. v3.2 追加：`from_ability` 與 `scope = "match"`（2026-10-29 排程）

主 session 指派（v3.md §12.11）。兩者的讀取端都還沒有被執行的英雄：`from_ability` 由分身 `create_illusion.copy.status_tags` 讀（C02、C16、A27W、B02W），`scope = "match"` 只有 `tsum`（B30T、B27T）。所以先定設計，實作跟著讀取端的元件一起放：

### 7.1 `from_ability`：規則層推導的狀態 tag

- 定義（登錄表 `v3-shared-registry.lua`）：狀態的施加者**還存在**，且來源是**非道具技能**。原作 CreateMirror 的條件是 `IsValidEntity(v:GetCaster()) and not v:GetAbility():IsItem()`。
- 它不是施加時寫死的 tag，是讀取時才推導：施加者死亡後實體還在就算存在，被移除（召喚物消失、幻影過期）就不算。所以 BuffStore 不存 tag 字串，而是存**來源**：
  - `BuffEntry.source: StatusSource { kind: Ability | Item | Host, caster: Option<Entity> }`，施加時由 host 填。
  - `rules::status::has_tag(entry, tag, entities)`：`from_ability` = `kind == Ability` 且 `caster` 仍存在；其他 tag 讀狀態宣告的 `tags`（IR 狀態實例）。
- 來源由 host 決定，不由資料或 handler 宣告，handler 版與 IR 版的結果才會相同：
  - 技能腳本的 hook（`execute`、`on_attack_hit`、`on_learn`、投射物命中等）裡的 `add_buff`／`add_stat_buff` → `Ability`，`caster` 是被呼叫的英雄；IR 也走同一條。
  - 由技能規格驅動的 host 元件（`ability_projectile`、`attack_orb`、`delayed_effect`、`tether`、damage zone）→ `Ability`，`caster` 取規格的施法者。
  - 道具程式（`item.rs`、`item_modifiers`、商店）→ `Item`；地圖、塔、兵、規則層自己的狀態 → `Host`。
- 傳遞：`Outcome::AddBuff` 不加欄位（39 個建構點、其他 session 也在改）；改由 ParallelWorldAdapter 記錄目前 hook 的來源，緊接在 AddBuff 之後送 `Outcome::StatusSource { target, buff_id, source }`，game_processor 寫進已存在的條目（被魔免擋下就沒有條目，也就不寫）。同一 id 被不同來源刷新時以最後一次為準（原作 modifier 重套也換成新的 ability／caster）。
- 不揭露：來源只在 authority 與 replica 的 BuffStore，不進 snapshot 或 HUD。重播比對的 authority 摘要會把它加進每個 buff 的那一行。

### 7.2 `unit_state` 與 `scope = "match"`

- 儲存：resource `UnitStateStore`（`BTreeMap`，決定性走訪）：
  - `scope = "hero"`：鍵是（英雄所屬玩家，名稱），英雄死亡重生保留；
  - `scope = "unit"`：鍵是（entity，名稱），entity 移除時清掉；
  - `scope = "match"`：鍵只有名稱，全場一份，對局開始時清空。
- 第一次讀取前的值是宣告的 `init`；型別照宣告（fixed、int、bool、ability、unit）。
- 跨文件同名就是同一個變數（登錄表保證型別相同）：B30T 與 B27T 的 `tsum` 互相改寫，照原作。
- 揭露：`reveal = "public"` 的值才可以影響對方可見的判定；`team`／`owner` 照 m2-disclosure-policy.md。
- **需要資料端補的欄位**：IR 目前只有 `{"state", name}` token，沒有宣告。請 M0 在 IrBlock 加 `unit_state = [{ name, type, init, scope, reveal }]`，列出這個區塊讀寫到的每個變數（跨文件的從登錄表帶入）。直譯器載入時依名稱建立索引；同名但宣告不同就載入失敗。

### 7.3 排程

| 項目 | 跟著哪一批 |
|---|---|
| `BuffEntry.source`、`StatusSource` outcome、`rules::status::has_tag` | M3 第二批（和排程器、一般 selector 一起；先讓 C17／B01 的重播摘要帶上來源，確認 handler 與 IR 相同） |
| `UnitStateStore`、`set_state`／`{"state"}`、`scope = "match"` | M3 第二批（`repeat` 與 `set_state` 是 M6 第 1 批的元件，B30／B27 在第 1 批） |
| `create_illusion` 的 `copy.status_tags`／`status_except` | M6 第 3 批（第一次出現分身） |
