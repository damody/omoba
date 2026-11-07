# 數值單一來源：玩法、說明、網站共用同一份數值

使用者要求（2026-10-28）：暈眩秒數、耗魔、傷害、回復、攻速這類數值，**只能寫在一個地方**。遊戲結算、遊戲內說明、網站，以及之後的文件與測試，都讀同一份，不能像 Dota 那樣同一個數字寫在 KV、說明文字、腳本好幾處，造成不同步。

## 1. 現況：同一個數字寫在哪些地方

| 位置 | 例子 | 問題 |
|---|---|---|
| 原作 | KV 的 `AbilitySpecial`、說明文字、腳本常數 | 試點每位英雄都有三者不一致，例如明智光秀 E 的暈眩：KV 寫 1～2.5 秒，腳本實際是 0.5～2 秒，說明文字照 KV 寫 |
| 舊官網 `damody/nobugg_official` | 執行時用 regex 讀 KV 的 cooldown、mana、radius、range，再讀**數字已經展開好**的說明文字（`addon_tchinese2.txt`） | 說明文字裡的數字是另外一份 |
| omoba package | `abilities.lua` 的 `description` 是寫死數字的長句；`resources`、`effects` 又各寫一次 | 例如阿市光鍊：description 與 effects 各寫一次 350/520/690/860 |
| omoba Rust handler | 以 `extras` 讀數值（好的做法），但檔頭註解又抄一次 | 註解會過期 |
| content | `AbilityDefinition.description` 是一般字串，UE 直接顯示 | 引擎沒辦法檢查它和數值一致 |
| 道具 | 說明文字用原作 `%x%` 佔位符展開後存成字串 | 展開之後就和數值脫鉤了 |

## 2. 原則

1. **每個數值只定義一次**：放在技能、狀態、道具或單位的 `values` 表裡，有名稱、單位與出處。
2. **其他地方一律用名稱引用**：
   - 行為資料寫 `$stun`；
   - 說明模板寫 `{stun}`；
   - 網站、bot、預覽範圍都讀同一份 `values`。
3. **說明文字裡不能出現數字。** 驗證器檢查模板內不能有半形或全形數字（佔位符除外）。
4. **行為資料裡不能出現未命名的數字。** 只允許 0、1、-1 這類結構常數；其他數字即使不給玩家看，也要命名，並標記 `hidden`。
5. **原作說明只當出處保存。** 原作的說明文字原文與 sha 存在 `source.lua`，不再顯示給玩家。產生一份「原作說明與實際數值的差異報告」，給使用者檢視。

## 3. 資料格式（package schema v3 的一部分）

```lua
{
  id = "c01_ground_flame", source_ability_id = "C01E",
  values = {
    -- 標準欄位也是 values，不另寫一份
    cooldown   = { 14, 13, 12, 11, unit = "s",     source = "kv:AbilityCooldown" },
    mana_cost  = { 100, 110, 120, 130, unit = "mana", source = "kv:AbilityManaCost" },
    cast_range = { 1500, unit = "dist",            source = "kv:AbilityCastRange" },
    radius     = { 300, unit = "dist",             source = "kv:C01E_Radius" },
    damage     = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:C01E_Damage" },
    stun       = { 0.5, 1, 1.5, 2, unit = "s",
                   source = "script:C01_Mitsuhide_Akechi.lua:107-111 level*0.5",
                   source_differs = "KV C01E_Duration=1/1.5/2/2.5 沒有被使用" },
    wave_damage = { 100, unit = "dmg", damage_type = "magical", source = "script:C01_Mitsuhide_Akechi.lua:83-87" },
    wave_count  = { 3, unit = "count" },
    wave_interval = { 0.4, unit = "s" },
    wave_stack_cost = { 1, unit = "count" },
    -- 衍生值：只寫運算式，建置期逐 rank 求值；說明與結算共用
    max_total = { expr = "$damage + $wave_count * $wave_damage", unit = "dmg" },
  },
  text = {
    zh_TW = {
      name = "地焰召來",
      body = "{delay} 秒後在目標地點 {radius} 範圍內爆發地焰，造成 {damage} 魔法傷害並暈眩 {stun} 秒。"
          .. "之後若有血火共刃層數，每 {wave_interval} 秒消耗 {wave_stack_cost} 層再爆發一次（{wave_damage} 傷害），最多 {wave_count} 次。",
    },
  },
  behavior = { ... },   -- 行為只引用 $radius、$damage、$stun……
}
```

### 3.1 單位與格式

| `unit` | 例子 | 顯示 |
|---|---|---|
| `s` | 暈眩、持續、冷卻 | `0.5/1/1.5/2`，目前等級加粗 |
| `pct` | 機率、加成 | `25%` |
| `dmg`、`heal`、`mana` | 傷害、回復、耗魔 | 依 `damage_type` 加上類型顏色或圖示 |
| `dist` | 射程、半徑 | 原作距離。換算成世界單位只在一個地方做（v1 §0 規則層） |
| `atk_speed`、`move_speed`、`armor` | 屬性加成 | 帶正負號 |
| `count` | 次數、層數 | 整數 |
| `chance` | 偽隨機 | `p` 加上保底，例如「約 19%，連續 5 次未觸發則必定觸發」 |

- 佔位符可以指定格式：`{damage}`、`{damage:current}`（只顯示目前等級的值）、`{bonus:signed}`、`{chance:pct}`。
- 引用規則層：`{rules.hero.base_magic_resistance}`。
- 引用狀態或子技能的值：`{status.c01e_stun.duration}`。

### 3.2 自動產生的資訊列

冷卻、耗魔、射程、半徑、前搖、傷害類型、能否穿透魔免、可驅散類型，這些**不寫在說明文字裡**，由 `values` 與 flags 自動產生固定格式的資訊列（舊官網的 cd、mana、radius、range 欄位就是這一類）。

### 3.3 道具

- 屬性加成直接由 `passive.stats` 產生「+20 攻擊力」這類行。
- 主動與被動效果的說明使用同一套模板。
- 價錢與合成配方也只有一份，商店、網站、合成樹共用。

### 3.4 多語言

- `text` 依語言分開，數值共用。
- 驗證器檢查各語言模板的佔位符集合一致。
- 至少要有 zh_TW。舊官網有 zh-TW、zh-CN、en、jp 四種語言；本專案要支援幾種，待使用者決定。

## 4. 誰讀這份數值

```text
package values＋text（Lua，唯一來源）
        │  建置期：驗證、求出衍生值、轉成 Fixed64 raw（決定性）
        ▼
compiled content catalog（JSON；有 content hash）
   ├─▶ Rust：建置期由同一份 values 產生 Fixed64 常數，技能程式碼直接引用（D15；見 m3-rust-codegen-design.md）
   ├─▶ UE：bridge 內嵌同一份 catalog；通用 TooltipRenderer 依目前等級與即時屬性產生說明
   ├─▶ 網站產生器：scripts/build_site.lua 輸出靜態 HTML（英雄、技能、道具、商店、合成樹）
   ├─▶ bot：bot_usage 引用 $cast_range 等值
   ├─▶ 預覽與指示器：範圍、半徑、形狀
   └─▶ 文件與驗收報告：hero-authoring、acceptance 自動產生
```

- **決定性**：數值在建置期由十進位字串直接轉成 Fixed64 raw，不經過 f32。現行 `extras: BTreeMap<String, Vec<f32>>` 要改。
- **同一版本**：網站頁尾、遊戲設定頁都顯示 content hash。網站和遊戲是否同一版，一眼就能看出。
- **版本差異自動產生**：比對兩個 content catalog 的 `values`，就能產生「改版說明」，例如「明智光秀 E 暈眩 1/1.5/2/2.5 → 0.5/1/1.5/2」，不必手寫。

## 5. 防止不同步的檢查（建置期，失敗就不產生 content）

| 檢查 | 內容 |
|---|---|
| 說明沒有數字 | 模板內不能有數字（佔位符除外） |
| 佔位符都存在 | 每個 `{x}` 都要對應到 values、規則層或子項 |
| 數值有人用 | 每個 value 都要被行為、說明或資訊列引用，不能留下沒人用的值 |
| 玩家看得到 | 不是 `hidden` 的 value 必須出現在說明或資訊列 |
| 行為沒有未命名數字 | 行為資料只允許結構常數 |
| 出處一致 | `source = "kv:<name>"` 的值，在匯入或更新原作時與釘選的原作 KV 比對；不一致就要寫 `source_differs` 說明理由 |
| 語言一致 | 各語言模板的佔位符集合相同 |
| Rust hook 不寫數字 | hook 只能讀 values，Rust 程式碼與註解不能抄數值（lint 檢查 hook 檔內有沒有像數值的常數） |

## 6. 網站

- **取代舊官網的資料來源**：舊站在執行時讀 Dota 的 KV 與說明檔，需要 OpenResty 加 Redis。新站在建置期由 Lua 從 content catalog 產生靜態 HTML，可以放在任何靜態主機上。
- **頁面**（對照舊站）：
  - 英雄列表；
  - 英雄頁：三圍、成長、各技能的資訊列與說明、各等級數值表；
  - 道具與商店分類、道具頁（價錢、資訊列、說明、合成樹）；
  - 改版說明。
- **舊站的其他功能**：banpick 工具、英雄預覽影片、勝率統計。要不要做，待使用者決定。
- **版型**：可以參考舊站的 etlua 版型與 CSS。素材只用 nobu 自製的圖示。

## 7. 遷移

1. schema v3 加入 `values` 與 `text`，同時保留舊的 `description`，標為 deprecated。
2. 把已移植的 10 位英雄與已上架道具的說明改成模板。數值改由 `values` 提供，`extras` 由 `values` 產生；Rust handler 改讀新名稱，並刪掉註解裡抄的數字。
3. UE 改用 TooltipRenderer，刪掉直接顯示 `description` 的程式碼。
4. 網站產生器 MVP：先產生這 10 位英雄與已上架道具的頁面，確認數字和遊戲內一致。
5. 檢查全部開啟後，移除 `description` 欄位。
