# history_nobu 分析：新的權威來源（2026-10-27）

唯讀分析。除了本檔，沒有修改 repo 內任何檔案。來源只當文字讀，沒有執行裡面的任何腳本。

## 0. 基本資料

| 項目 | 值 |
|---|---|
| 倉庫 | https://github.com/damody/history_nobu |
| 本地 clone | `D:\omoba\target\source\history_nobu`（不在 repo 追蹤路徑） |
| HEAD | `f966720b0956d002099d72aabec7ddc94b16d4bf`，2021-06-01 23:33:31 +0800，訊息「jump money * 3」 |
| 第一個 commit | `946f59cc`，2019-12-21「add game」 |
| commit 數 | 1851，只有 master 一個分支 |
| 作者 | Victor 1198、circus0983 262、damody 232、ts0963187279 111、Sxty1 48 |
| 活躍期 | 2020-02～04（開發高峰）、2020-11～2021-01（第二波平衡與新英雄）、2021-03（道具與 aram），之後零星更新 |
| 對照用的 nobugg_official | `D:\omoba\target\nobugg_src`，`ca6ee8602473f6015a397cc121dac6baeb70b2e5` |

分析時寫的輔助腳本放在 scratchpad，沒有放進 repo：一個 KV 讀取器，以及單位、英雄、道具三種比對腳本。

## 1. 這是什麼

這是 **Dota 2 自訂遊戲 addon `nobu`**（信長之野望，Source 2／Workshop Tools 格式），以 barebones 樣板改寫，是一套完整可玩的遊戲。`addoninfo.txt` 列出兩張地圖：`nobu`（10 人）與 `nobu_aram`。程式裡另外還認得 `nobu_pk` 1v1 模式，但 addoninfo 沒有列出這張地圖。

### 目錄結構

```
content/dota_addons/nobu/      # 原始資產（只有編輯器才用得到）
  maps/nobu.vmap, nobu_aram.vmap, nobu_basic.vmap, prefabs/, content/  # 二進位 DMX（binary 9, vmap 29）
  models/ particles/ materials/ sounds/ soundevents/ panorama/ flash/
game/dota_addons/nobu/         # 執行期資料
  addoninfo.txt
  scripts/npc/                 # 所有 KV
  scripts/shops/nobu_shops.txt # 商店
  scripts/vscripts/            # Lua 實作（遊戲模式、英雄、道具、AI）
  resource/addon_{tchinese,schinese,english,japanese}.txt   # 在地化，UTF-16LE
  panorama/                    # UI（js/xml/css）與 panorama/localization
  itembuilds/
```

### 各類資料的位置

| 類別 | 位置 | 規模 |
|---|---|---|
| 英雄 npc 定義 | `scripts/npc/{A_Oda,B_Unified,C_Neutral}/X_2.txt`，由 `npc_heroes_custom.txt` 以 `#base` 載入；預設值在 `npc_heroes_override.txt`；`herolist.txt` 決定開放哪些英雄 | 97 位英雄（A01–A36、B01–B36、C01–C25） |
| 技能 KV | `scripts/npc/{A_Oda,B_Unified,C_Neutral}/X.txt`；系統技能在 `npc_abilities_custom_load.txt`（6468 行）；另有 `npc_abilities_override.txt` | 每位英雄一檔 |
| 技能實作 | `scripts/vscripts/heroes/{A_Oda,B_Unified,C_Neutral}/*.lua`，加上 `heroes/*.lua` 的共用 modifier | 121 個 Lua 檔、39,184 行；97 位英雄都有對應腳本 |
| 小兵、建築、野怪 | `scripts/npc/npc_units_custom_load.txt`（15,869 行）；`npc_units_override.txt`（18,545 行，Dota 原生單位覆寫） | 塔、兵營、主堡、小兵、野怪、召喚物、dummy |
| 出兵 AI | `vscripts/computer_system/chubing.lua`（762 行）；`surrender.lua` 是投降機制 | — |
| 道具 KV | `scripts/npc/npc_items_custom_load.txt`（18,492 行）；`npc_neutral_items_custom.txt` | 262 個 item 區塊 |
| 道具腳本 | `scripts/vscripts/items/Addon_Items/*.lua` | 103 個檔、11,466 行 |
| 遊戲模式 | `vscripts/addon_game_mode.lua`（入口）→ `require.lua` → `nobu_init.lua`（GameRules）、`GameRulesStateChange.lua`（金錢、出兵、魔王）、`equilibrium_constant.lua`（三圍換算）、`library/events/eventforkill.lua`（重生、經驗、野怪重生）、`library/events/eventforspawned.lua`（英雄出生）、`library/common/dummy.lua`（回復、攻速、魔抗的 tick 系統） | 全部 vscripts 約 90,788 行 |
| 地圖 | `content/.../maps/*.vmap`（二進位 DMX），包含出兵路徑點 `chubinluxian_location_of_{nobu,wl}_{top,middle,button}N`、野怪、魔王、`dota_{goodguys,badguys}_fort`、商店 | 需要 DMX 解析器才能取座標 |
| 在地化 | `game/.../resource/addon_tchinese.txt`（UTF-16LE，4061 行）、`panorama/localization/addon_tchinese.txt` | 說明文字使用 `%duration%` 等佔位符，沒有展開 |

`settings.lua` 與 `gamemode.lua` 是 barebones 樣板，**實際沒有被載入**：`require.lua` 沒有 require 它們，`gamemode.lua` 裡的 require 也都被註解掉。裡面的 `STARTING_GOLD=500`、`GOLD_PER_TICK=100` 不是實際設定，不能採用。

## 2. 遊戲模式常數（實際生效者）

### 2.1 三圍換算（`equilibrium_constant.lua`，由 `eventforspawned.lua` require，所有單位出生時都會掛上）

腳本寫的是「目標值 − Dota 預設值」的差額。加上引擎預設值後，設計意圖如下：

| 項目 | 原作設定（HEAD） | 目前 `attribute_rules.lua` 的推測 |
|---|---|---|
| 每點力量生命 | **25**（引擎 20 + 5） | 20 |
| 每點力量生命回復 | **0.2**（引擎 0.1 + 0.1，經 `HealthRegen_self` 每 0.1 秒以 0.1 為單位疊層） | 0.03 |
| 每點智力魔力 | **18**（12 + 6） | 12 |
| 每點智力魔力回復 | **0.15**（0.05 + 0.1，經 `ManaRegen_self` 以 0.01 為單位疊層） | 0.04 |
| 每點敏捷護甲 | **0.2**（腳本以 0.14 當預設，差額 +0.06）；2020–21 年的引擎實際值可能約 0.16，所以實際約 0.22，**不確定** | 1/7 |
| 每點敏捷攻速 | **1.5**（1 + 0.5，經 `AtkSpeedBonus_self` 疊層） | 1 |
| 主屬性攻擊力 | 1（引擎預設，腳本沒有改） | 1 |
| 移速上限 | **700**（`MODIFIER_PROPERTY_MOVESPEED_LIMIT`；HEAD 前是 1500，最早是 3000） | — |
| 英雄魔抗 | **基礎 30%，上限 70%**：每位英雄的 `attribute_bonusx` 每 0.5 秒呼叫 `dummy.lua:magical_resistance`，設定 `SetBaseMagicalResistanceValue(caster.magical_resistance or 30)`；道具和技能透過 `caster.magical_resistance` 加減。`npc_heroes_override` 的 25 會被這個值覆蓋 | 0（之前判為「腳本不在來源，未知」） |
| 智力魔抗 | 無 | 0 |
| 攻速範圍 | `SetMaximumAttackSpeed(600)`、`SetMinimumAttackSpeed(-1000)` | — |

歷史版本：第一個 commit 的設定是 25／0.2／20／0.15／0.333／2。2020-05 之前改成 25／0.2／18／0.15／0.2／1.5，之後就沒有再變。2021-02-01 移除了近戰英雄額外生命回復（「remove hp regen bouns for melee hero」，程式碼還留在註解裡）。結論：**之前推測的 Dota 2 6.87–7.05 不成立**。原作有自己的一套換算，數值明顯較高。

### 2.2 金錢

- 起始金錢 **2000**：`HERO_SELECTION` 階段 `PlayerResource:SetGold(i,2000)`；`GameRules:SetStartingGold(0)`。
- 被動收入：開局 45 秒後每 2 秒 **+30**，也就是每秒 15 金。這正是 HEAD 那次 commit「jump money * 3」，之前是 +10／2 秒。aram 是每 2 秒 +15。
- 每 60 秒每人 **+150**（`ccpres`，從第 60 秒起）。
- 引擎的 GoldPerTick 是 0；`SetUseBaseGoldBountyOnHeroes(false)`；死亡不掉錢（`SetLoseGoldOnDeath(false)`，最後一次設定為準）；禁止買活。
- 建築賞金：非英雄擊殺時，由 1500 範圍內友方英雄均分 `GetGoldBounty()`。
- 賣出比例：2021-03-05 從 85% 改成 **75%**。

### 2.3 經驗與等級

- 等級上限 **25**；`SetUseCustomHeroXPValues(true)`。
- 升級累計經驗：0, 220, 485, 795, 1150, 1550, 1995, 2485, 3020, 3600, 4225, 4895, 5610, 6370, 7175, 8025, 8875, 9725, 10575, 11425, 12275, 13125, 13975, 14825, 15675。公式是 `xp += 220 + min(i-1,14)*45`。
- 小兵經驗不在 KV（BountyXP 是 0），由腳本給：足輕 30、弓兵 30、鐵炮 50、騎兵 50。1400 範圍內的敵方英雄共享，倍率依人數為 1／0.7／0.6／0.5／0.4（`eventforkill.lua`）。
- 野怪經驗：douchebag／bandit／assassin／wild_bear／tortoise／fear_molang 37、executor／military_adviser／pirates_leader 60、bear 90、sea_people 45、wild_warrior 和 pour_odd_person 75、robbers_remnants 30。低於隊伍平均等級時另有補償。
- 擊殺英雄：2000 範圍內的敵方英雄平分 `等級×30`，再依等級差乘 0.5／1／2／3／4。

### 2.4 重生

- 英雄依等級查表 `die_tim2 = {1,2,4,7,11,16,22,25,28,32,36,46,48,50,52,54,56,66,70,74,78,82,86,90,100}` 秒，超出表格是 140 秒，aram 減半。
- 野怪：`CP_respawn_time = 120` 秒後在原位重生（aram 60 秒）。
- 魔王：「強盜之王」在 180 秒出現，「受詛咒的武士亡靈」在 1200 秒出現。
- 600 秒時，雙方主堡各生成一個 `home_aura` 守家單位。

### 2.5 兵線時程（`chubing.lua` 的 `ShuaGuai`）

- 第 30 秒開始出兵。
- **足輕**：每 35 秒一波，上下路各 3、中路 4。開局 +150 HP，之後每波 +8 HP、+1 攻擊。
- **弓兵**：比足輕晚 2 秒，每 35 秒一波，上下路各 2、中路 3。每波 +8 HP、+2 攻擊。
- 從第 180 秒起：**鐵炮** 上下路各 2、中路 3，**騎兵** 每路 1；每 175 秒一波，1800 秒之後改成每 70 秒。鐵炮每波 +5 HP，騎兵每波 +10 HP，兩者都 +3 攻擊。
- 同一批內每 0.5 秒出一隻。每種兵有 8 種造型（`_1`～`_8`），隨機挑一種。
- 小兵護甲每 300 秒 +1（`_G.armor_bonus`）。
- 兵營被破壞後，對方該路小兵強化 ×1.3。完全破壞後，足輕和弓兵 ×1.3，鐵炮和騎兵 ×2。
- 塔每 45 秒攻擊 +4，基礎護甲重設為 15；開局 10 秒時先設為 5，並掛上 `buff_tower`、`tower_armor`。
- 2020-12-24：隊伍擁有 3 座騎兵營時，大將 +300 攻擊、+30 護甲。
- 1v1、aram 模式只走中路。

### 2.6 其他規則

- **隊伍**：`SetCustomGameTeamMaxPlayers(6,5)`，DOTA_TEAM_CUSTOM_1 是觀戰隊（2021-04「add spectator team」）；雙方各 5 人，也就是 **5v5**。
- **選角**：可重複選同一位英雄（`SetSameHeroSelectionEnabled(true)`）。選角時間 50 秒，策略時間 20 秒，PreGame 0 秒。
- **勝負**：50 殺結束的設定只在樣板裡，沒有生效；實際以推平主堡獲勝。15 分鐘後可以輸入 `-ff` 投降。
- **其他**：攝影機距離 1200；偷塔保護關閉；第一滴血開啟；樹木 300 秒後重生。
- **英雄出生時加上的系統技能**：hero_kill、hero_die、slow_self（自訂緩速疊加）、HealthRegen_self、ManaRegen_self、AtkSpeedBonus_self、Attack_fail、afk_checker。另外給一個 `item_S01`，並為每人建立一隻信使驢。

### 2.7 自動索敵範圍（AttackAcquisitionRange）

| 單位 | 範圍 |
|---|---|
| 英雄預設（`npc_heroes_override`） | 800 |
| 英雄個別（X_2.txt） | B01 1400、A34 1200、C17 1200、B08 900、C15 700、B29 700、B04 500、A02 500、A07 500、B06 500 |
| 塔 `com_tower_*` | 900（攻擊距離 800） |
| 大將塔 `com_general_*` | 1100 |
| 足輕 | 1200 |
| 弓兵 | 600 |
| 鐵炮 | 700 |
| 騎兵 | 600 |
| 魔王 | 300 |

## 3. 塔、建築、小兵、野怪的數值（HEAD）

| 單位 | HP | 護甲 | 魔抗 | 攻擊 | 攻擊間隔 | 射程 | 移速 | 金錢／經驗 |
|---|---|---|---|---|---|---|---|---|
| `com_tower_{1,2,3}_{top,mid,bottom}`、`com_towera` | 2400 | 20（腳本改為 5→15，每 45 秒 +4 攻擊） | 0 | 377–379 | 1.0 | 800 | — | 150／0 |
| `com_general_oda/unified`（大將塔） | 3000，每秒回 20 | 23 | 100 | 300 | 0.5 | 1100 | — | 1300／100 |
| `com_general2_*`（可移動大將） | 4000 | 23 | 0 | 300 | 0.6 | 1100 | 400 | 800／100 |
| `com_soldiercamp_{oda,unified}_{top,mid,down}`（兵營） | 3000 | 17 | 0 | 不攻擊 | — | — | — | **300**／200（nobugg 是 750） |
| `com_soldiercamp_Oda/Unified`（主堡） | 10000，每秒回 20 | 20 | 50 | 不攻擊 | — | — | — | 50／50 |
| `dota_fountain` | — | — | — | **800 純粹傷害**（nobugg 是 4000） | **0.2**（nobugg 是 0.1） | 1400 | — | — |
| `com_infantry_*`（足輕） | 650（+150） | 0 | 0 | 45–49 | 1.58 | 138 | 325 | 25／腳本 30 |
| `com_archer_*`（弓兵） | 650 | 0 | 0 | 70–75 | 1.67 | 500 | 325 | 35／腳本 30 |
| `com_gunner_*`（鐵炮） | 780 | 0 | 0 | 80–95 | 2.0 | 650 | 290 | 60／腳本 50 |
| `com_cavalry_*`（騎兵） | 1500，每秒回 20 | 4 | 0 | 60–68 | 1.0 | 100 | 400 | 60／腳本 50 |
| `npc_dota_the_king_of_robbers`（強盜之王） | 5000，每秒回 100 | 20 | 30 | 180–230 | 1.0 | 400 | 600 | 150／150 |
| `npc_dota_cursed_warrior_souls`（武士亡靈） | 15000，每秒回 200 | 40 | 50 | **500**（nobugg 是 300） | 1.2 | 500 | 550 | 450／450 |
| `npc_dota_neutral_160_douchebag` | 966 | 2 | — | 55–66 | 1.35 | — | 350 | 40／37 |
| `npc_dota_neutral_160_executor` | 1182 | 2 | — | 134–144 | 1.35 | — | 350 | 85／60 |
| `npc_dota_neutral_130_wild_bear` | 900 | 3 | — | 90–100 | 1.35 | — | 270 | 64／37 |
| `npc_dota_neutral_130_bear` | 1920 | 11 | — | 126–133 | 0.9 | — | 350 | 70／90 |
| `npc_dota_neutral_130_wild_warrior` | 2040 | 15 | — | 150–157 | 0.9 | — | 350 | 77／75 |
| `npc_dota_neutral_130_robbers_remnants` | 1260 | 5 | — | 135–141 | 1.35 | — | 350 | 46／30 |

野怪共 14 種（130／160 兩組，加兩隻魔王），每種在 `nobu.vmap` 出現 3 次，表示有多個營地。

**單位 KV 比對（nobugg → history）**：
- `npc_units_custom_load.txt` 共 235 個單位：173 個相同、56 個不同、6 個只在 history。
- 主要差異：
  - 塔的 CombatClassAttack 從 PIERCE 改成 HERO；
  - 兵營賞金 750 → 300；
  - 泉水 4000／0.1 → 800／0.2；
  - 移動大將的索敵範圍 1500 → 1100；
  - 野怪的 BountyXP 從 0 補成與腳本相同的值；
  - 武士亡靈攻擊 300 → 500；
  - 部分召喚物與 dummy 調整。
- 只在 history 的單位：`{oda,unified}_{infantry,archer}_forest`、`com_soldiercamp_*_fake`。
- history 另有 nobugg 沒有的 `npc_units_override.txt`。

## 4. 與 nobugg_official 的對照

### 4.1 版本與年代

- nobugg_official 的 commit 日期是 2026-01-05，但那只是匯入日期；網站 footer 寫 2017。它的 `npc/` **約等於 history_nobu 在 2020-05 中下旬的狀態**：
  - 以 npc 檔逐 commit 比對，最接近的是 `e99b7afd`（2020-05-21），但仍差約 750 行，所以不是單一 commit 的快照；
  - `A30_2.txt`、`B13_2.txt` 等於 history 2020-04-15 的版本，`C22_2.txt` 等於 2020-05-14 的版本；
  - nobugg 有一份 `A18.txt`，與 history 任何版本都差 600 行以上。history 的 A18 是 2021-01-06 才加入，所以 nobugg 的 A18 來自別處。
- history_nobu HEAD（2021-06-01）比那個時間點**多了 766 個 commit**，其中包括：
  - 2020-11-27「adjust many hero's property」：大量英雄的護甲從 0 改成 3／5，攻擊力上下限收斂成單一值；
  - 2020-12～2021-03：一連串道具平衡；
  - 新增英雄的檔案：B02、B03（2020-11）、A18、A30（2021-01）；
  - 2021-03：aram 模式；
  - 2021-06：金錢 ×3。
- **結論：history_nobu 比較新，也比較完整，應以它為準。** nobugg 是網站用的早期衍生快照。

### 4.2 英雄名單與代碼

- 代碼完全一致：A01–A36、B01–B36、C01–C25 共 97 位。名稱與 `addon_game_mode.lua` 的 `hero_name_zh` 一致，例如：
  - A07 本多忠勝、B06 真田幸村；
  - B01 雜賀孫市、B04 伊達政宗、A34 阿松、C17 阿市、A02 稻姬、B08 淺井長政、C15 玉子、B29 村上義清。
- **開放名單不同**：
  - nobugg 的 `herolist.txt` 開放 48 位，`npc_heroes_custom.txt` 只 `#base` 49 位，其餘被註解掉；
  - history 開放 75 位，並 `#base` 全部 109 行（含 C26–C36，但這些檔案不存在）。
  - 目前的名冊收 97 位，不受影響。
- history 的 `C_Neutral/範例.txt` 是範本，不是英雄。

### 4.3 英雄數值

**單位數值（X_2.txt，比對 20 多個欄位與技能槽）**：

- **目前 8 位和 A07、B06 這 10 位完全相同**，三圍、成長、HP、魔力、護甲、攻擊、射程、移速、索敵範圍、技能槽都一樣。所以已產生的 `hero_source_stats` 對這 10 位不需要改。
- 全部 97 位中有 **54 位不同**。典型差異是 `ArmorPhysical` 0 → 3 或 5、攻擊力上下限收斂成單一值、部分三圍與主屬性互換，例如：
  - B35 敏捷 → 力量；
  - C22 敏捷 → 智力；
  - B12、C16、C18 力量和智力的初始值互換。
- 也有技能槽順序的差異：`magical_resistance`、`_precache`、T 的位置不同。
- nobugg 的 B18_2 讀不出單位區塊。

**技能 KV（X.txt）的差異**（`<` 是 nobugg，`>` 是 history）：

| 英雄 | 差異行數 | 重點 |
|---|---|---|
| B01 | 4 | B01T 的 modifier 名稱錯用成 `modifier_B26T`，history 修正為 `modifier_B01T`（2020-12-31 bug fix） |
| B04 | 26 | B04T 改為投射物命中後呼叫 `B04.lua:B04T_OnHit`，新增 `damage 35/50/65` 等 special |
| A34 | 114 | 施法前搖 0.25 → 0.1；新增 `mana_regen 100/150/200`、`magical_resistence_reduce 5/10/15`、Dodgeable、IsStunDebuff |
| C17 | 4 | 冷卻 7 → 7/6/5/4；魔耗 80/100/120/140 → 100/110/120/130 |
| A02 | 19 | 音效與表現，主要是 vo |
| B08 | 28 | W、E 前搖 0.2／0.3 → 0.1；W 加上 `RunScript B08W` 與 0.1 秒的 COMMAND_RESTRICTED |
| C15 | 4 | Dodgeable 0 → 1、IsStunDebuff |
| B29 | 8 | IsStunDebuff |
| A07 | 175 | A07W 魔耗 100/120/140/160 → 100/110/120/130、持續時間 1–4 → 1.25–4.25；另有新增段落：攻速 60–100、heal_percentage 4、unslow 50%、魔抗 50、狀態抗性 50、受到傷害 −50% |
| B06 | 55 | 光環移速 15% → `aura_move_speed 8`、`aura_duration 15`、`move_speed 30` |

### 4.4 道具

- **數量**：history 有 262 個 item 區塊，nobugg 有 245 個；兩邊都有 238 個，只在 history 24 個，只在 nobugg 7 個。
  - 只在 nobugg：`item_frozen_staff`、`item_samurai_cloth_1`、`item_the_spear_of_saddle_2` 及部分配方。
  - 只在 history：千鳥、大千鳥十文字槍、新藤五國光、般若鬼丸、助真、助宗、和弓、赤備具足、冰風暴之書、攻速鋼等。
- **商店**：history 的 `nobu_shops.txt` 列 177 個道具，nobugg 的 `shops.txt` 列 138 個；history 多 43 個，少 4 個。
- **價錢**：有 36 個不同，例如：
  - 茶碗 1600 → **1300**，目前上架的 `tea_bowl` 受影響；
  - 野太刀 2600 → 2100、薩摩銃 3000 → 3500、大炎上 9050 → 8600；
  - 多項合成費調整，例如 `recipe_great_dragon` 1500 → 750、`recipe_nannbann_armor` 2000 → 1000。
- **配方**：有 10 個不同，例如：
  - 長捲：月槍 → 長槍 ×2＋力量鋼；
  - 薩摩銃：另需打刀；
  - 南蠻胴：另需番頭盾；
  - 武士服：另需攻速鋼；
  - 大雷電之書：`energy_ingot + thunderbolt_book`。
- 目前上架的 30 個道具中，**只有 `tea_bowl` 的價錢不同**，配方都沒變。上架前要讓 `item_sources_check` 依新來源重算。
- **腳本**：
  - nobugg 沒有任何道具 vscript；history 有 `items/Addon_Items/*.lua` 103 個檔、11,466 行。
  - 道具 KV 共引用 88 個腳本檔，只缺 `item_samurai_cloth.lua` 一個。
  - **之前標為 needs_framework 的 103 個商店道具**（其中 97 個標 SCRIPT）：**97 個的腳本檔齊全**，RunScript 引用的函式找到 277／279 個（缺的是 `item_soul_adder` 的 1 個函式，以及 `item_samurai_cloth` 整個檔案），4 個本來就沒有腳本，`item_frozen_staff` 在 history 已刪除。
  - 範例：`item_brave_ring` 的爆擊是 15% 機率乘 1.85 倍，並帶「連續 7 次沒觸發就保底觸發」的偽隨機；這個規則只看說明文字是無從得知的。
- **在地化**：
  - history 的 tooltip 是 UTF-16LE，用 `%special%` 佔位符；nobugg 的 `addon_tchinese2.txt` 是 UTF-8，數值已展開，例如 `(1/2/3/4)`，還多收了舊版技能的說明。
  - history 收錄的英雄技能 tooltip 比較少：A07 12 條，nobugg 30 條。
  - 目前 `item_source/active.lua` 的「說明文字的每個數字都要能由 KV 解釋」規則，改用 history 時必須先展開佔位符。

### 4.5 history 多出的資料（nobugg 沒有）

1. **技能實作**：121 個 Lua、39,184 行，97 位英雄都有。技能 KV 引用的 200 個腳本檔中，196 個存在。之前 source-catalog 有 763 個技能標 `depends_on_missing_vscript = true`，改用 history 後大多可解。
2. **道具腳本**：103 個檔，涵蓋 E160 標為 deferred 的主動效果，以及 E155 的 SCRIPT 被動。
3. **遊戲模式常數**：三圍、魔抗、移速上限、金錢、經驗、重生、兵線、塔的成長、魔王時程，見第 2 節。
4. **單位**：`npc_units_override.txt`、`npc_neutral_items_custom.txt`、`OB.txt`／`OB_2.txt`（觀戰英雄）、`portraits.txt`。
5. **地圖**：三張 vmap，以及 prefab（大將、兵營、旗本武士）。
6. **AI**：`ai_core.lua`、`B08_ai.lua` 等，屬於原作 AI，可當 bot 行為參考。
7. **UI**：panorama 的 js、xml、css（計分板、商店等），可當 UE 的參考。

## 5. 對目前工作的影響

### 5.1 三圍換算

- **影響大，而且有明確答案。** `attribute_rules.lua` 的 20／0.03／12／0.04／(1/7)／1／0 應改成原作的 25／0.2／18／0.15／0.2／1.5。另外補上「英雄基礎魔抗 30%、上限 70%」與「移速上限 700」。
- 敏捷護甲建議直接採用設計意圖 0.2。引擎疊加後的實際值約 0.22，但這取決於 Dota 引擎版本，不宜照抄。
- 要注意的副作用：
  - 英雄 HP、回復、攻速都會明顯上升（例如 B01 力量 34 → 生命 +170、回復 +5.1／秒），平衡與「10 分鐘內打完」的驗收要重測；
  - 魔抗 30% 會降低所有法術傷害。
- E158／E162 的 settle 機制不受影響，只是係數改變。魔抗 30% 需要一個「英雄基礎魔抗」欄位，或者讓三圍 settle 一併寫入魔抗。

### 5.2 道具的 SCRIPT 與 ACTIVE 效果

- **可以照原作移植。** 原本缺腳本的道具，現在 97／103 都有實作。E160 那些「數字只出現在說明文字」的 deferred 主動，例如魔免、閃現、連鎖閃電、隱形 2.5 秒，現在可以從腳本讀出實際數值與規則。
- 建議做法沿用 E160：先依機制分群，做成通用能力；轉換器拒絕不支援的部分並說明原因。差別在於證據來源多了「腳本函式」，轉換器可以記下引用的函式與行號，提供 provenance。
- 腳本是 Dota API 的 Lua，例如 `ApplyDataDrivenModifier`、`FindUnitsInRadius`、`caster.xxx` 這類全域狀態，**不能直接執行**，要人工或半自動移植到 Rust 的通用能力。要特別注意原作的偽隨機、引擎內建行為（例如 `PREATTACK_CRITICALSTRIKE`），以及共用的 `caster.magical_resistance`、`ms_slow`、`as_slow` 這類全域欄位。
- 價錢與配方要依 history 重算：`tea_bowl` 1600 → 1300；E155 那份「來源內部價錢不一致」清單也要重算。

### 5.3 英雄技能

- 目前 8 位英雄的 Rust 實作約 5,014 行，是依 KV 與說明文字寫的。原作的 Lua 有：
  - B01 422、B04 283、A34 192、C17 470、A02 154 行；
  - B08 280 行，另有 AI 561 行；
  - C15 214、B29 377 行；
  - 合計約 2,400 行（不含 AI）。
- 差異分三類：
  1. **KV 數值已經變了**：A34 前搖 0.25 → 0.1；C17 冷卻與魔耗；B08 前搖 0.2／0.3 → 0.1；A07W 魔耗與持續時間。這些照 history 改即可。
  2. **機制只寫在腳本裡**：例如 B04T 命中後的 `B04T_OnHit`、B08W 的 `RunScript`、B01 的 W／E／R 狀態機（`B01E_CHECK`、`B01R2`／`R3`／`R4`）。目前 Rust 是從說明推測的，可能和原作不同，要逐招對照。
  3. **全域系統**：緩速疊加（`slow_self`、`ms_slow`）、狀態抗性 `states_res`、魔法護盾（`nobu_modifier_magical_shield`）、Dodgeable 投射物等。這些應做成引擎層的通用規則，不要每位英雄各寫一份。
- **可以做「照原作移植」的流程**：
  1. `hero_source_index export` 的原始資料包加上該英雄的 vscript 原文，包括 RunScript 函式全文與共用 modifier；
  2. D3 requirements 的 `depends_on_missing_vscript` 改成「已有腳本」，並把腳本實際用到的 API 列為能力需求；
  3. 每招寫一張「原作行為 → Rust 實作」對照卡（觸發點、數值來源、目標篩選、持續時間、例外），再補測試；
  4. kit-coverage 增加一個狀態「已對照原作」。

### 5.4 塔、小兵、野怪的數值

- **可以改用原作，但不能只換數字。** 原作的尺度和目前地圖差很多：
  - 原作塔 2400 HP、377 攻擊，每 45 秒 +4 攻擊，護甲 15；
  - 原作小兵 650 HP、45–75 攻擊，每 35 秒一波，每波逐步成長；
  - 目前沿用的 Dota 參考值是：近戰 550 HP、21 攻擊，每 30 秒一波（`moba_maps.lua`）；quick 地圖的塔只有 750 HP。
- 直接套用原作數值，比賽時間會遠超過使用者「每場 10 分鐘內」的驗收上限。建議：
  - 正式地圖用原作數值，包括成長規則，這需要支援每波成長、按時間成長的塔、兵營破壞強化；
  - quick／live check 地圖另外定義倍率，記錄為「原作 × 係數」，不要另外發明數值。
- 原作的兵種是足輕、弓兵、鐵炮、騎兵（第 180 秒起出鐵炮與騎兵），和目前的 melee／caster／siege／super 不同，要重新對應。
- 野怪有 14 種、兩隻魔王（180 秒、1200 秒），重生 120 秒；目前是 15 秒重生的訓練營地。

### 5.5 預設 5v5、地圖、兵線節奏

- **5v5**：與原作一致（10 人，每隊 5 人，另有觀戰隊）。原作允許重複選同一位英雄，所以 E163「10 位不重複來源英雄」是我們自己的選擇，不是原作規則。
- **地圖**：原作是三路，主堡大約在 (7445, −7517) 與 (−7390, 7203)，地圖範圍約 ±8000 Dota 單位。出兵點與路徑點是 vmap 裡的 `chubinluxian_*` 實體，但 vmap 是二進位 DMX，要另寫唯讀解析器才能取得座標。目前只能從 Lua 拿到少量座標，例如魔王、FOW viewer、守家單位。
- **兵線節奏**：見 2.5。目前每 30 秒一波、3 近戰＋3 遠程＋每 3 波 1 攻城，和原作不同。

## 6. 建議的遷移計畫（不在本次動手）

### 第 0 批：固定新來源與索引工具（風險：低到中）

- **來源命名與固定 commit**：`history_nobu@f966720b0956d002099d72aabec7ddc94b16d4bf`，clone 到 `target/source/history_nobu`，維持唯讀、不執行。
- **`scripts/hero_source_index.lua`**：
  - REPOSITORY／COMMIT 改成 history_nobu；
  - 增加 source root 的概念：npc 在 `game/dota_addons/nobu/scripts/npc`，vscripts 在 `.../scripts/vscripts`，tooltip 在 `.../resource/addon_tchinese.txt`；
  - catalog 的 `path` 欄位改用相對於 npc 的路徑，保持和舊 catalog 可比。
- **`hero_source/catalog.lua`**：
  - `TOOLTIP_FILE` 改讀 UTF-16LE。lua-host 若沒有解碼功能，就用 Lua 自己解 UTF-16LE。不要用 PowerShell 或 Python。
  - vscript 解析：記錄每個 RunScript 對應的檔案、函式與行號範圍；`depends_on_missing_vscript` 依新來源重算。
- **`scripts/item_source_index.lua`、`item_source/catalog.lua`**：
  - `PINNED`、`DEFAULT_SOURCE` 換成 history_nobu；
  - `SHOP_FILE` 改成 `scripts/shops/nobu_shops.txt`；
  - tooltip 同上；
  - 新增道具腳本的引用與存在檢查。
- **`item_source/active.lua`**：說明文字要先依 KV special 展開 `%x%` 佔位符，才能套用「數字要能由 KV 解釋」的規則。
- **source_ref 格式**：`HeroDefinition.source_ref`、`MobaItemEntry.source_ref` 改成 `history_nobu@<commit>:<id>`。要改的地方：
  - `omoba-template-ids` 的 `validate_hero_sources` 與 `lib.rs:1314`（目前斷言開頭是 `nobugg_official@`）；
  - `lua_content.rs` 的測試案例；
  - `hero_source/roster.lua` 的 `M.NAME`；
  - `templates/hero_sources.lua`、`source_items.lua`、`item_sources_check.lua`、`hero_source_stats.lua` 的註解與檢查；
  - `scripts/tests/hero_source_roster_test.lua`、`hero_author_test.lua`、`item_sources_check_test.lua`；
  - omfue `item_catalog.rs` 與 `OmUiEventTypes.h` 的註解；
  - 角色 package 的 provenance。
  - 建議一次切換，不保留雙來源。如果需要過渡，validate 可以暫時接受兩種前綴，但同一位英雄只能有一個。
- **重新產生**：
  - `source-catalog.lua`、`source-item-catalog.lua`、`item-compare.md`、`hero_sources.lua`、`source_items.lua`；
  - `d3-requirements.md`、`kit-coverage.md` 的來源欄位；
  - `roster --check` 與 `item_sources_check` 要通過。
- **驗證**：
  - 新增「nobugg → history 對照」測試：10 位已交付或指定英雄的 `hero_source_stats` 不變；`tea_bowl` 價錢的變化要被 `item_sources_check` 擋下，再隨資料更新；
  - 經 `land_to_live`；
  - `lua_content_check`（E159）要涵蓋新的 templates。
- **風險**：tooltip 編碼與佔位符；catalog 很大（目前 7.6 MB）；有些路徑寫死（`target/nobugg_src`）。

### 第 1 批：遊戲模式常數（風險：中，平衡會變）

- `attribute_rules.lua` 改為原作值（25／0.2／18／0.15／0.2／1.5），新增英雄基礎魔抗 30%（上限 70%）與移速上限 700。
- 新增 `templates/source_game_rules.lua`（名稱待定），從 history 以文字萃取並記錄 provenance（檔案與行號）。內容包括：
  - 起始金錢 2000、+30／2 秒（45 秒起）、+150／60 秒；
  - 等級上限 25 與經驗表；
  - 重生表 `die_tim2`；
  - 小兵與野怪經驗、分享倍率；
  - 擊殺經驗；
  - 賣出 75%。
- 驗證：三圍 settle 的單元測試；replica 與 authority 一致（E158、E162 的測試）；10 分鐘 live test。
- 風險：英雄變硬、法術傷害下降，可能讓比賽拉長。必要時給 quick 地圖一個明文記錄的倍率。

### 第 2 批：英雄（風險：中到高）

1. 10 位英雄的 X_2 數值不變，確認 `hero_source_stats` 沒有差異即可。
2. 技能 KV 依 history 更新：A34、C17、B08、A07、B06 等，見 4.3 的表。
3. 「照原作移植」：export pack 附上 vscript，D3 和 kit-coverage 增加「已對照原作」。每位英雄一個 PR，順序是交付中的 8 位，再來是 A07、B06。
4. 通用系統先行：緩速疊加、狀態抗性、魔法護盾、Dodgeable、IsStunDebuff。

風險：Rust 行為會改變，現有測試需要更新；原作可能有 bug（例如 B01T 曾錯用 B26T），移植時以 HEAD 為準，並記錄刻意的偏離。

### 第 3 批：道具（風險：中）

1. 價錢與配方依 history 更新（36 個價錢、10 個配方）。上架的 `tea_bowl` 改成 1300；商店清單改用 `nobu_shops.txt`（177 個）。
2. SCRIPT 被動與 ACTIVE 依機制分群後，照腳本實作成通用能力。優先處理：
   - 爆擊（含偽隨機）；
   - 自身魔免；
   - 單體暈眩加傷害（吹箭）；
   - 範圍傷害；
   - 魔法護盾；
   - 連鎖閃電。
3. `item_samurai_cloth`（缺腳本）、`item_soul_adder` 缺的 1 個函式、`item_frozen_staff`（history 已刪除）標為 deferred 或下架。

### 第 4 批：塔、小兵、野怪、兵線（風險：高，牽涉地圖與比賽時間）

- 新增來源單位表，包括塔、兵營、主堡、四種小兵、14 種野怪、兩隻魔王，並記錄 provenance。
- 引擎需要的新能力：
  - 每波成長（HP、攻擊）；
  - 塔每 45 秒成長；
  - 兵營破壞後強化；
  - 鐵炮與騎兵從 180 秒起出兵，每 175 秒一波，1800 秒後改為每 70 秒；
  - 魔王時程。
- quick 地圖使用明文記錄的倍率，以符合 10 分鐘的驗收。
- 兵種對應：足輕 → melee、弓兵 → ranged、鐵炮 → siege、騎兵 → 新的快速近戰。UE 的外觀目錄要一併更新。

### 第 5 批（選做）：地圖

- 寫一個唯讀的 DMX（binary 9）實體萃取器，取出塔、兵營、出兵點、野怪營地的座標，作為重製地圖的參考。風險高，可以延後。

## 6.5 與使用者 clone（`C:\nobu\history_nobu`）的比對

使用者那份只讀，沒有修改，也沒有執行裡面的任何東西。比對時那份 clone 已經完成：檔案數連續 3 次檢查不變，也沒有 `index.lock`。

| 項目 | 本地 clone（`D:\omoba\target\source\history_nobu`） | 使用者 clone（`C:\nobu\history_nobu`） |
|---|---|---|
| remote | github.com/damody/history_nobu | 同左 |
| HEAD | `f966720b…`，2021-06-01 | 相同 |
| HEAD tree | `5f579efef249cd7c6d2d2f6581179e96b03f4771` | 相同 |
| 分支 | 只有 master | 只有 master |
| 追蹤檔案數 | 9419 | 9419 |
| 工作樹差異（`diff -rq`，排除 .git） | — | **0 個** |
| `git status` | 乾淨 | 乾淨 |
| LFS | 沒有 `.gitattributes`，`git lfs ls-files` 是 0 | 相同，沒有 LFS |
| 大小 | content 1.5 GB、game 44 MB | content 1.5 GB、game 44 MB、.git 507 MB |

結論：**兩份完全相同**。這個 repo 沒有使用 LFS，模型、粒子、音效都以一般 git 物件存放，本地 clone 已經是完整的。

### 資產本身的缺口（上游就缺，兩份相同）

- **沒有編譯後的資源**：沒有任何 `*_c` 檔（`.vmdl_c`、`.vpcf_c`、`.vsnd_c`），只有原始資產，包括 fbx 315、dmx 575、vmdl 175、vpcf 3834、vmat 238、mp3 1799、vmap 133。要在 Dota 工具裡重新編譯才能執行，對我們來說只能當參考。
- **模型**：KV 引用了 301 個 `models/…`，`content/` 裡有 140 個；其餘大多是 Dota 內建模型。確定缺的自訂模型有：
  - `models/A36/A36.vmdl`、`models/a02/a02.vmdl`、`models/c23/c23.vmdl`、`models/maori/maori_model.vmdl`；
  - `models/c19/c19_model.vmdl` 實際在 `models/new/c19/`。
- **粒子**：腳本與 KV 引用了 1004 個 `.vpcf`，`content/` 裡有 568 個；其餘大多是 Dota 內建（`particles/units|econ|generic_gameplay…`）。排除內建前綴後，仍有約 56 個自訂粒子找不到，例如 `particles/A01E4/A01E4.vpcf`、`A03/A03e.vpcf`、`A08t/A08t.vpcf`、`B07r/B07r.vpcf`、`C03t/C03t.vpcf`。
- 影響：本專案在 Unreal 使用自己的 Tripo／CC0 資產，原作的模型與粒子最多只能當外觀參考，不影響數值與機制的移植。

## 7. 未確認與注意事項

- **敏捷護甲**：引擎預設值依 2020–21 年 Dota 版本而定，腳本假設 0.14。這裡採設計意圖 0.2，實際可能約 0.22。
- **nobugg 不是 history 的單一快照**：例如 A18 的來源不明。所以「nobugg 數值 = history 某個 commit」只是近似。
- **未逐一確認**：`buff_tower`、`tower_armor`、`modifier_unit_armor` 的實際效果（定義在 `npc_abilities_custom_load.txt` 與 `dummy.lua`），都還沒有逐行看過。
- **模式差異**：以上數值以 `nobu` 地圖的一般模式為準；aram、pk 另有分支（金錢、重生減半、只走中路）。
- **已鎖定的時間點**：HEAD commit（2021-06）是金錢 ×3 的版本；如果要用更早的平衡版本，要另外指定 commit。
