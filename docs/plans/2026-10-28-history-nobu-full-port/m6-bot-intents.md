# M6 bot 施放資料：v3 `bot` 欄位與執行器介面提案

建立日期：2026-10-29。負責：工作線 A（資料與 catalog）。Rust 實作交給工作線 B（omoba-core bot 與 IR）。

## 1. 目的

實機驗收靠 server bot 與 UE autopilot 施放每一招。目前的做法有兩個問題：

- **server bot**：比賽設定（`scripts/lua_data/moba_role_match.lua`）逐招手寫 `ability_policies`，例如 `{ability='c17_light_chain', intent={kind='enemy_hero'}}`，目前只涵蓋 18 招。
- **UE autopilot**（`OmAutoPilot.cpp` 的 `ChooseCast`）：只看 HUD 卡片的目標形狀、敵我、射程。友方技能一律在受傷或敵人接近時自用，攻擊技能打射程內最近的敵方英雄。它分不出治療與增益、範圍技能要幾個目標、切換技、召喚等差別。

87 位尚未上架的英雄要逐批開放（[m6-waves.md](m6-waves.md)），不能每位都手寫，所以 v3 文件要帶通用的施放資訊。

## 2. v3 `bot` 欄位（已放回，schema v3.3）

- 技能與道具 `active` 可以寫 `bot`。
  - 形狀照 omoba-core 的 `BotAbilityIntent`（`moba_match/bots/abilities.rs`），例如 `bot = { kind = "enemy_point", radius_key = "$radius", min_targets = 1 }`。
  - `bot = false` 表示 bot 不施放。
- **沒寫時自動推導**（`docs/character_pipeline/tools/v3_bot.lua` 的 `derive`），依 target、cast、cost 與施放行為判斷，第一條符合的規則勝出：

  | 條件 | 推導出的 intent |
  |---|---|
  | 被動、子技能（`sub`／`hidden`） | `none` |
  | 切換技 | `toggle_engage`（提案） |
  | 引導：會治療自己 | `self_channel_recovery` |
  | 引導：有以施法者為中心的範圍 | `channel_enemy_area`（提案） |
  | 單位目標，敵方：只能選英雄 | `enemy_hero` |
  | 單位目標，敵方：其他 | `enemy_unit` |
  | 單位目標，友方：會治療 | `ally_heal`（會驅散則 `ally_heal_or_cleanse`） |
  | 單位目標，友方：其他 | `ally_engage`（提案） |
  | 點或方向目標：只有位移、沒有傷害 | `approach_enemy_point` |
  | 點或方向目標：有 `aoe_radius` 或圓形選取 | `enemy_point`（`radius_key` = 半徑的值名稱） |
  | 點或方向目標：錐形選取 | `enemy_cone` |
  | 點或方向目標：直線、線段、矩形選取 | `enemy_line` |
  | 點或方向目標：會召喚 | `summon_near_enemy`（提案） |
  | 無目標：回復自己法力 | `self_recovery` |
  | 無目標：治療自己 | `self_heal` |
  | 無目標：會召喚 | `summon_near_enemy`（提案） |
  | 無目標：以自己為中心、打敵人的範圍 | `self_area` |
  | 無目標：其他（增益） | `self_engage` |

  門檻預設值：治療在生命低於 600‰ 時施放，`threat_radius` 900，`min_targets` 1，法力恢復在法力低於 400‰ 時施放，位移最短距離 300。
- 以下情況才手寫：推導不符實際用法（例如增益看起來像治療），或門檻已經過實機驗證。驗證器會檢查種類與參數，以及 `*_key` 指向的值是否存在。
- **catalog**：每招的 V3Block 都有 `bot`（`omoba-content-model` 的 `V3Block.bot`，型別是 `serde_json::Value`），內容如下。
  - intent 的 JSON 形狀：`*_key` 已換成值名稱，也就是編譯後的 extras 鍵。
  - `source`：`authored` 或 `derived`。
  - `proposed: true`：執行器還沒有這個種類。
  - `reason`：只有 `kind = "none"` 時才有。
  - `note`：推導時的備註，可能出現。
  - 若半徑等 `*_key` 指向的值沒有逐 rank 數字（執行期運算式），編譯器會改成 `none` 並寫出原因。

### 2.1 與現行手寫 policy 對齊

`moba_role_match.lua` 的 18 條手寫 policy，v3 的 `bot` 全部一致：

- 12 條靠推導就一致。`radius_key` 是 v3 值名稱，例如 `radius`；手寫的是 v2 host 鍵 `shape_radius`。兩者是同一個值（`v2_key`），extras 裡也兩個鍵都有。
- 6 條在文件裡手寫，照實機驗證過的門檻：a07_unscathed（700）、a07_red_thunder、a07_strongest、b06_brave_warriors（800）、b06_first_warrior、c17_victory_blessing（ally_heal）。

### 2.2 全量分布（97 位英雄、167 件道具，共 553 個 bot 欄位）

| kind | 技能 | 道具 | 執行器 |
|---|---|---|---|
| enemy_unit | 77 | 22 | 已有 |
| enemy_hero | 15 | 1 | 已有 |
| enemy_point | 58 | 12 | 已有 |
| enemy_line | 13 | 1 | 已有（見 §3.2） |
| approach_enemy_point | 5 | 2 | 已有 |
| self_area | 30 | 11 | 已有 |
| self_engage | 100 | 39 | 已有 |
| self_heal | 8 | 0 | 已有 |
| self_recovery | 0 | 2 | 已有 |
| ally_heal | 3 | 4 | 已有 |
| ally_heal_or_cleanse | 1 | 0 | 已有 |
| toggle_engage | 10 | 0 | **提案** |
| channel_enemy_area | 16 | 0 | **提案** |
| summon_near_enemy | 22 | 8 | **提案** |
| ally_engage | 3 | 3 | **提案** |
| none（被動 74、子技能 13） | 87 | 0 | 不施放 |

## 3. 請工作線 B 實作的介面

### 3.1 bot 預設讀 catalog

- `RoleBotMatchPlan` 的 `ability_policies` 改成「覆寫」：
  - 座位英雄的每一招，沒有明確 policy 時，就用該招 V3Block 的 `bot`。
  - `kind = "none"` 或 `proposed: true` 的不施放，直到執行器支援為止。
- 轉換方式：把 `bot` 的 JSON 拿掉 `source`、`proposed`、`reason`、`note`，就是 `BotAbilityIntent` 的 serde 形狀，可以直接 `serde_json::from_value`。之後沿用 `validate_policies`，不通過就跳過並記錄。
- `validate_policies` 目前要求能力是 `active_ability_const` 的 Instant 施放（`SelfChannelRecovery` 除外）。M6 用 IR 執行的英雄不在 v2 `active_ability_const` 裡，需要改從 content-model 的 V3Block 取目標類型與 extras（V3Block.values 的 raw）。

### 3.2 幾何資訊

- `EnemyCone`、`EnemyLine` 目前讀 v2 編譯的 `cone_half_angle_ticks`、`line_widths`。M6 的英雄沒有 v2 targeting_shape。
- 改法：從 IR 施放事件的 selector 取得幾何（`shape = "cone"` 的 `angle`、`length`，`line`／`segment` 的 `width`）。也可以請我在 catalog 的 `bot` 補 `angle_key`、`width_key`、`length_key`。需要哪一種請告訴我，資料端補欄位很快。

### 3.3 提案中的新 intent

| kind | 參數 | 決策規則（建議） |
|---|---|---|
| `toggle_engage` | `threat_radius` | 有已揭露的敵方英雄在半徑內時打開，沒有時關閉（UE autopilot 目前只會打開） |
| `ally_engage` | `threat_radius` | 對半徑內有敵方英雄、離自己最近的友方英雄施放；沒有就對自己施放 |
| `summon_near_enemy` | `threat_radius` | 半徑內有敵人時，在自己與最近敵人之間施放（無目標的召喚直接施放） |
| `channel_enemy_area` | `radius_key`、`min_targets` | 自己周圍半徑內至少 N 個已揭露敵人、且自身不在控制中時開始引導 |
| `escape` | `below_hp_per_mille`、`threat_radius` | 生命低於門檻且有威脅時，往己方基地方向施放位移技能。目前沒有任何推導會產生它，留給作者手寫 |

這些 intent 只能用已揭露的觀察資料，不能讀對方隊伍的 `team` 狀態（m2-disclosure-policy.md）。

### 3.4 法力保留

- 現行規則保持不變：bot 法力不足時不施放；UE autopilot 會「保留給大招」，第二輪攻小兵要有 70% 法力。
- 建議在 policy 層加可選的 `reserve_mana_per_mille`：施放後剩餘法力不得低於這個比例。
- v3 先不加這個欄位，等 B 決定介面後我再補進 `v3_bot.lua` 的 KINDS 共用參數與推導。

## 4. UE autopilot（工作線 C）

- `FOmAutoPilot::ReadSlotCards` 依 `AbilityCatalogId` 取得 catalog 裡該招的 `bot`。
- `ChooseCast` 依 `kind` 分派：
  - `ally_heal`：只在有友方低於門檻時施放。
  - `self_area`、`enemy_point`：需要 `min_targets`，數量從已揭露實體計算。
  - `toggle_engage`：依 `threat_radius` 開或關。
  - `none`：不施放。
  - 沒有 `bot` 時維持現行規則。
- 不需要為個別英雄寫 C++。

## 5. 檢查

- `scripts/tests/package_v3_test.lua` 的 `bot_policy`：推導（C17 的 enemy_hero、enemy_point）、手寫覆寫、驗證錯誤。
- 全量內容（`scripts/check_v3_content.lua`）的 615 個 V3Block 都能被 content-model 載入。其中技能（含被動，被動為 `none`）與有主動的道具，共 553 個帶 `bot`；純被動道具沒有這個欄位。
