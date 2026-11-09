# history_nobu 信使（馬）原作規格

來源：`C:/nobu/history_nobu`，commit `f966720`。以下路徑皆相對於 `game/dota_addons/nobu/`，行號以該 commit 為準。

簡稱：
- `UNITS` = `scripts/npc/npc_units_custom_load.txt`（由 `scripts/npc/npc_units_custom.txt:4` `#base` 載入）
- `ABIL` = `scripts/npc/npc_abilities_custom_load.txt`
- `VS` = `scripts/vscripts`

## 0. 重點結論

- 真正的玩家信使只有一種單位：`npc_dota_courier2`，中文名「馬」（`resource/addon_tchinese.txt:78`，UTF-16LE 檔）。
- 每位玩家（每個英雄）一隻。英雄第一次 spawn 時由 Lua 產生，只歸該玩家所有、只有該玩家能控制（`VS/library/events/eventforspawned.lua:39-57`）。
- Dota 內建信使全部關掉：`SetFreeCourierModeEnabled(false)`（`VS/addon_game_mode.lua:595`），`item_courier`／`item_flying_courier` 都被 REMOVE（`scripts/npc/npc_abilities_override.txt:105`、`:143`）。商店沒有信使可買。
- `UnitLabel "courier"` 出現 13 次，但大多數**不是信使**：那些是戰法／賣兵用的 `npc_dota_building`（ModelScale、移速 0、HasInventory 0），只是複製了信使模板（見 §1.3）。`npc_dota_courier` 也不是信使，它是戰法召喚用的「donkey」佔位單位（見 §1.2）。

## 1. 單位定義

### 1.1 `npc_dota_courier2`（玩家信使「馬」）— UNITS:3306-3379

| 欄位 | 值 | 出處 |
|---|---|---|
| BaseClass | `npc_dota_creature`（不是引擎的 `npc_dota_courier` class） | UNITS:3310 |
| Model | `models/items/courier/white_the_crystal_courier/white_the_crystal_courier.vmdl`（驢子模型那行被註解） | UNITS:3311-3312 |
| Level / IsAncient / ModelScale | 0 / 1 / 1 | UNITS:3315-3317 |
| UnitLabel | `courier` | UNITS:3318 |
| AbilityLayout | 12 | UNITS:3321 |
| Ability1-12 | courier_return_to_base, courier_go_to_secretshop, courier_return_stash_items, courier_take_stash_items, courier_transfer_items, courier_burst, **courier_speed**（不是 courier_morph）, courier_take_stash_and_transfer_items, for_magic_immune, for_no_collision, phased_dummy, courier_mute | UNITS:3323-3334 |
| BoundsHullName | 註解掉（沒有定義） | UNITS:3338 |
| HealthBarOffset | 120 | UNITS:3339 |
| MovementCapabilities | `DOTA_UNIT_CAP_MOVE_GROUND`（地面，KV 沒有飛行） | UNITS:3343 |
| MovementSpeed | 750 | UNITS:3344 |
| MovementTurnRate | 0.5 | UNITS:3345 |
| BountyGoldMin/Max | 150 / 150 | UNITS:3349-3350 |
| StatusHealth | 500 | UNITS:3354 |
| StatusHealthRegen | 10 | UNITS:3355 |
| TeamName | `DOTA_TEAM_GOODGUYS`（建立時用 `caster:GetTeam()` 覆蓋） | UNITS:3359；VS/library/events/eventforspawned.lua:41 |
| CombatClassAttack/Defend | BASIC / BASIC | UNITS:3360-3361 |
| UnitRelationshipClass | `DOTA_NPC_UNIT_RELATIONSHIP_TYPE_COURIER` | UNITS:3362 |
| VisionDaytime/Nighttime | 100 / 100 | UNITS:3366-3367 |
| AttackDesire（bot 用） | 1.5 | UNITS:3371 |
| HasInventory | 1 | UNITS:3374 |
| WakesNeutrals | 0 | UNITS:3378 |
| 護甲、魔抗、攻擊能力、攻擊數值、Mana | **KV 沒有定義**（只能用引擎預設，repo 裡沒有數字） | UNITS:3306-3379 全段沒有這些 key |

### 1.2 `npc_dota_courier`（「馬」註解，但實際上是戰法召喚的佔位單位）— UNITS:3234-3304

| 欄位 | 值 | 出處 |
|---|---|---|
| BaseClass | `npc_dota_creature` | UNITS:3238 |
| Model | `models/props_gameplay/donkey.`（路徑殘缺，少了 `vmdl`） | UNITS:3239 |
| AbilityLayout | 6 | UNITS:3247 |
| Ability1-11 | 6 個 courier_* ＋ courier_speed ＋ courier_take_stash_and_transfer_items ＋ for_magic_immune、for_no_collision、phased_dummy | UNITS:3249-3259 |
| MovementCapabilities / Speed / TurnRate | GROUND / 550 / 0.5 | UNITS:3268-3270 |
| Bounty | 250 / 250 | UNITS:3274-3275 |
| StatusHealth / Regen | 500 / 10 | UNITS:3279-3280 |
| UnitRelationshipClass | COURIER | UNITS:3287 |
| Vision | 600 / 600 | UNITS:3291-3292 |
| HasInventory | 1 | UNITS:3299 |

用途：`VS/items/Addon_Items/war_magic.lua` 的 `to_war_magic_unit2`、`to_soldier_*`、`to_archer_*`、`to_gunner_*`、`to_cavalry_*` 與 `to_sell_ninja_unit` 都用 `CreateUnitByName("npc_dota_courier", ...)`（war_magic.lua:358,423,480,531,581,631,681,732,782,977）。建立後會移除原本的技能、換上戰法／兵營技能（例如 war_magic.lua:360-371、430-440）、換模型（例如 :462-463），多數還會加上 `nobu_modifier_rooted` 和 `modifier_invulnerable`（例如 :464-466）。玩家看到的是「戰法／兵營控制單位」，不是信使。英雄的生成流程從來不建立這個單位（§2）。

### 1.3 其他 `UnitLabel "courier"` 的單位（不是信使）

下面這些都是 `npc_dota_building`、移速 0、HP 1000、Bounty 150、Vision 1、HasInventory 0，帶 8 個 courier_* 技能（含 courier_morph）＋一個 Ability9 的販售／戰法技能，顯然是複製信使模板改出來的：

| 單位 | 起始行 | Ability9 |
|---|---|---|
| to_sell_ninja_unit | UNITS:2350（欄位在 2353-2380） | to_sell_ninja_unit |
| soldier_Unified_unit | UNITS:2494 | to_soldier_Unified |
| soldier_Oda_unit | UNITS:2568 | to_soldier_Oda |
| archer_Unified_unit | UNITS:2642 | to_archer_Unified |
| archer_Oda_unit | UNITS:2716 | to_archer_Oda |
| gunner_Unified_unit | UNITS:2790 | to_gunner_Unified |
| gunner_Oda_unit | UNITS:2864 | to_gunner_Oda |
| cavalry_Unified_unit | UNITS:2938 | to_cavalry_Unified |
| cavalry_Oda_unit | UNITS:3012 | to_cavalry_Oda |
| war_magic_unit | UNITS:3087 | to_war_magic_unit |
| war_magic_unit2 | UNITS:3161 | to_war_magic_unit2 |

（範例欄位：to_sell_ninja_unit 的 BaseClass 在 UNITS:2354、MovementSpeed 0 在 :2384、StatusHealth 1000 在 :2394、HasInventory 0 在 :2416。）

其他只是借用信使模型、與信使無關的單位：`C16D_old_ghost`（UNITS:1900，模型 mlg_courier_wraith 在 :1903）、`findeverything_unit`（UNITS:2423，trapjaw 模型在 :2429）。

### 1.4 為什麼會有好幾個？

- 不分隊伍、不分升級型態：兩隊共用同一個 `npc_dota_courier2`，隊伍在建立時決定（eventforspawned.lua:41）。原作沒有飛行升級的版本（見 §5、§6）。
- `npc_dota_courier` 與 §1.3 的單位都是戰法／賣兵系統借用信使模板，不是信使。
- `scripts/npc/npc_units_override.txt:9280-9336` 只有被註解掉的 `npc_dota_flying_courier`（標記 deprecated），沒有實際覆寫。

## 2. 數量與生成

- **每位玩家一隻**。`npc_spawned` 會觸發 `Nobu.OnHeroIngame`（`VS/events.lua:463`）。英雄 spawn 0.2 秒後，如果是非幻象、隊伍 < 4，而且 `hero.init1 == nil`（只在第一次 spawn），就在英雄位置加 `Vector(100,100,0)` 的地方執行 `CreateUnitByName("npc_dota_courier2", ..., true, caster, caster, caster:GetTeam())`（eventforspawned.lua:38-41）。
- 同時記錄 `donkey.spawn_position`（eventforspawned.lua:42），並把以下技能設成 Lv1：return_to_base、go_to_secretshop、return_stash_items、take_stash_items、transfer_items、burst、take_stash_and_transfer_items、for_magic_immune、phased_dummy、courier_mute（eventforspawned.lua:45-54）。**`courier_speed` 和 `for_no_collision` 沒有設等級。**
- 存成 `hero.donkey`（eventforspawned.lua:55-57）。
- 英雄重生時 `hero.init1` 已經是 true，會走 else 分支（只處理裝備，eventforspawned.lua:140-161），不會再產生信使。
- 斷線重連：`disconnect_checker` 偵測到連線狀態 2 且 `hero.donkey == nil` 時，3 秒後重建一隻 courier2，技能等級設法與上面相同（`VS/items/Addon_Items/tofar_goback.lua:766-798`）。**這裡沒有設 `spawn_position`。**
- 被註解掉的舊做法：開局給 `npc_dota_courier`（`VS/GameRulesStateChange.lua:459-469`）、聊天指令 `-donkey`（`VS/library/events/eventforchat.lua:356-368`）。現在都不會執行。
- 沒有商店購買的生成流程（§5）。

## 3. 控制權

- 每隻信使屬於一位玩家：`donkey:SetOwner(caster)` 加上 `SetControllableByPlayer(caster:GetPlayerID(), true)`（eventforspawned.lua:43-44；重連時 tofar_goback.lua:785-786）。
- 沒有隊伍共用的程式碼。引擎的共用信使已經用 `SetFreeCourierModeEnabled(false)` 關掉（addon_game_mode.lua:595）；這個 API 實際的引擎語意要看 Dota 文件，repo 裡沒有說明。
- 換英雄（`dota_player_pick_hero` → `Nobu:PickHero`，events.lua:462）2 秒後，會清掉 `caster.donkey` 身上大部分 modifier。保留 modifier_for_magic_immune、modifier_courier_transfer_items、modifier_for_move500、modifier_courier_mute 等，另外會移除 Passive_insight_gem（`VS/library/events/eventforpichero.lua:11-28`）。

## 4. 死亡與重生

- **可以被殺**：KV 有 HP 500 和 Bounty 150（UNITS:3349-3354），也沒有無敵 modifier。
- 但防護很多：
  - `for_magic_immune` 讓牠魔免（ABIL:5073-5089；生成時設 Lv1，eventforspawned.lua:52）。
  - 靠近自家泉水時物理傷害也無效：`dota_fountain` 帶 `courier_physical_immune`（UNITS:3395）。這是 1200 範圍的友方光環（ABIL:5166-5181），只對 `npc_dota_courier2` 加上 `modifier_courier_physical_immune2`，效果是 `MODIFIER_PROPERTY_INCOMING_DAMAGE_PERCENTAGE -100`（ABIL:5205-5211；`VS/library/common/dummy.lua:800-814`）。
  - 泉水的 `home_aura`（UNITS:3396；ABIL:3074-3099）每 0.25 秒讓 1300 範圍內的友方 courier2 取得 `modifier_phased` 0.5 秒（dummy.lua:747-759）。
- **死亡處理**（`entity_killed` → `Nobu:OnUnitKill`，events.lua:465）：
  - 單位名稱含 `courier` 時（這也會匹配 `npc_dota_courier`）：如果 `owner.disconnect == false`，**立刻** `RespawnUnit()` 並 `SetOrigin(spawn_position)`。兩隊的分支內容相同（`VS/library/events/eventforkill.lua:540-549`）。
  - 0.5 秒後把**所有**技能設成 Lv1，並把原始模型設回 crystal courier（eventforkill.lua:550-557）。
  - **沒有重生計時器**：程式碼直接立即重生。
  - 沒有自訂公告。在地化檔裡有引擎字串「織田軍信使已遭擊殺。」／「聯合軍信使已遭擊殺。」與重生訊息（`resource/addon_tchinese.txt:129-130,143-144`），但這些是引擎事件（courier_lost/respawned）用的字串；自訂單位會不會觸發，repo 無法確認。
- **金錢**：擊殺 bounty 來自 KV 150（UNITS:3349-3350），由引擎發放。`FilterGold`（events.lua:210-236，在 :455 註冊）把所有金錢設成不可靠金，只擋建築的特定金額和英雄擊殺金，所以信使擊殺金會照常發放。擊殺計數也會照常累加給攻擊者英雄（eventforkill.lua:151-166）。
- **斷線**：連線狀態 3 時，把信使欄位 0-6 的道具丟在英雄出生點，然後 `hero.donkey:ForceKill(true)`、`hero.donkey = nil`（tofar_goback.lua:740-759）。因為這時 `hero.disconnect = true`（:757），死亡處理不會重生（eventforkill.lua:542）。重連後重建（§2）。
- 被註解掉的舊版：只有 2 人以上才重生，並重新套魔免與無碰撞（eventforkill.lua:562-580）。

## 5. 運送道具與購買流程

- **原作依賴 Dota 內建信使技能**來做運送：8 個 `courier_*` 技能名稱都是引擎內建技能。repo 的 `npc_abilities_custom*.txt` 和 `npc_abilities_override.txt` 都**沒有**定義或覆寫它們（grep 只找到 courier_speed、courier_mute、courier_physical_immune 這幾個自訂技能，ABIL:1628、6276、5166）。所以回基地、去秘密商店、拿／放倉庫、送貨給英雄、加速（burst）的行為與數值都是引擎的，**repo 裡沒有數字**。
- 注意：載體是 `npc_dota_creature`，不是引擎的信使 class（UNITS:3310）。內建信使技能在 creature 上的實際表現，repo 無法驗證。
- `HasInventory 1`（UNITS:3374），所以信使可以裝道具。斷線時處理的是欄位 0-6（tofar_goback.lua:742-745）。
- 秘密商店與路邊商店：道具 KV 有 `"SecretShop" "1"`（19 筆）和 `"SideShop" "1"`（13 筆），例如千里大劍 `SecretShop 1`、`SideShop 0`（`scripts/npc/npc_items_custom_load.txt:6407,6420`）。購買走引擎的商店流程。
- **自訂訂單過濾**（`SetExecuteOrderFilter(Nobu.eventfororder)`，events.lua:448）：
  - 購買（ORDER 16）：擋 `item_S01`、`B23T_old`、`B07E_UNIT`，**對信使沒有特別限制**（`VS/library/events/eventfororder.lua:736-760`）。
  - 賣出（ORDER 17）：
    - 非信使單位要檢查自己的英雄活著（eventfororder.lua:797-803）。
    - 買入超過 10 秒才賣時：如果賣的是 `npc_dota_courier2`，主人英雄死了就**拒絕**賣出；活著就額外給 `0.25×itemcost` 不可靠金，並調整 PlayerEarnedGold／SpentGold（eventfororder.lua:808-818）。非信使的處理相同（:819-823）。
    - 買入 10 秒內賣出，只調整 SpentGold（:826-828）。
    - `item_logging` 不能賣（:804-806）。
- `SetItemAddedToInventoryFilter` 直接回傳 true，沒有限制（events.lua:128-136）。
- 商店檔 `scripts/shops/nobu_shops.txt` 沒有任何 courier 道具，`npc_items_custom*.txt` 也沒有 item_courier（grep 無結果）。

## 6. 技能

| 技能 | 內建或自訂 | 數值／效果 | courier2 生成時是否 Lv1 |
|---|---|---|---|
| courier_return_to_base | Dota 內建，沒有覆寫 | 引擎預設，repo 沒有數字 | 是（eventforspawned.lua:45） |
| courier_go_to_secretshop | 內建 | 同上 | 是（:46） |
| courier_return_stash_items | 內建 | 同上 | 是（:47） |
| courier_take_stash_items | 內建 | 同上 | 是（:48） |
| courier_transfer_items | 內建 | 同上 | 是（:49） |
| courier_burst | 內建 | 同上（加速量與 CD 都是引擎值） | 是（:50） |
| courier_take_stash_and_transfer_items | 內建 | 同上 | 是（:51） |
| courier_morph | 內建；**courier2 沒有這個技能**（只在 §1.3 的建築單位上） | — | — |
| courier_speed（自訂，「馬跑速」） | `ability_datadriven` 被動 | `MODIFIER_PROPERTY_MOVESPEED_ABSOLUTE 550`，並且 `MODIFIER_STATE_MUTED`（ABIL:1627-1652） | **否**；死亡重生後所有技能設 Lv1 時才會升級（eventforkill.lua:551-555） |
| for_magic_immune（自訂） | 被動魔免 | ABIL:5073-5089 | 是（:52） |
| for_no_collision（自訂） | 被動無單位碰撞 | ABIL:5247-5262 | **否**（重生後才 Lv1） |
| phased_dummy（自訂） | 每 0.1 秒讓自己（0.3 秒）和 50 範圍內敵我英雄與 basic 單位（0.5 秒）取得 modifier_phased | ABIL:6216-6240；dummy.lua:971-996 | 是（:53） |
| courier_mute（自訂） | 被動 `MODIFIER_STATE_MUTED`（不能使用道具） | ABIL:6276-6292 | 是（:54） |

- 飛行與形態變化被禁止：`ModifierGainedFilter`（events.lua:382，在 :456 註冊）對 `npc_dota_courier2` 拒絕 `modifier_courier_morph_effect`、`modifier_courier_flying`、`modifier_courier_passive_bonus`、`modifier_fountain_aura_buff`（events.lua:239-244、434-439，註解寫「ban 掉馬飛行 及 增加血量」）。所以**原作沒有飛行信使，也沒有速度升級**。
- 推論：datadriven 被動在 Lv0 不會套用。所以初次生成的移速應該是 KV 的 750；第一次死亡重生後 courier_speed 升到 Lv1，移速會被絕對值鎖成 550，並且多了無碰撞。這是程式碼的直接結果，但原作者是不是有意這樣做，無法確認（見 §8）。

## 7. 與英雄技能、道具的互動

- **千里大劍** `item_the_great_sword_of_teleport`（npc_items_custom_load.txt:6395-6470；ItemCost 8500 在 :6417，CD 60 在 :6408，耗魔 60 在 :6410，CastPoint 4 在 :6411，可以指定友方 BASIC／BUILDING 在 :6402-6403）：前搖開始（`Shock0`）時如果目標是 `npc_dota_courier2`，就 `caster:Stop()` 並 return，也就是**不能傳送到信使**（`VS/items/Addon_Items/item_the_great_sword_of_teleport.lua:21-28`）。
- **A13 服部半藏 鉤子**：兩段搜尋都排除 `npc_dota_courier2`（`VS/heroes/A_Oda/A13_Hattori_Hanzo.lua:632`、`:676`）。
- **C08 鉤子**（C08R 相關）：排除 courier2，除非單位名稱在 `target_filter` 裡（`VS/heroes/C_Neutral/C08.lua:337`）。
- **C19（松姬）**：
  - 攻擊連帶的兩個額外目標排除 courier2（`VS/heroes/C_Neutral/C19.lua:59`、`:98`）。
  - 傳送技能判斷落點 300 內有沒有友方單位時，**不把信使算進去**，所以不能以信使為傳送錨點（C19.lua:130-140）。
- **吹雪肩甲／きしょうせん冬月**：信使裝備時不建立護盾粒子，但 modifier 照樣套用（`VS/items/Addon_Items/item_fubuki_shoulders.lua:84`、`VS/items/Addon_Items/item_kisyosennfuyuzuki.lua:9`）。這表示信使身上的道具被動仍然有效。
- **`donkey_back`**：把 1000 範圍內敵方的 courier2 推到 1000 距離外並 phased（tofar_goback.lua:806-818）。KV 裡沒有引用（grep `npc/` 無結果），可能是死碼。
- `nodmg_courier` 原本要讓信使無敵，但那行已註解，函式實際上什麼都不做（dummy.lua:283-299）。
- `_G.EXCLUDE_TARGET_NAME`（dummy.lua:331 起）不包含 courier，信使是在各技能裡逐一用名稱排除的。

## 8. 不確定或原作沒有定義的地方

1. 內建 courier_* 技能的數值（burst 加速、CD、運送規則）不在 repo，屬於引擎預設，不要自己猜數字。
2. 內建信使技能放在 `npc_dota_creature`（不是引擎信使 class）上能不能完整運作，repo 無法驗證。
3. 護甲、魔抗、攻擊能力、Hull 在 KV 都沒有（Hull 被註解，UNITS:3338），只能用引擎預設。
4. 移速：KV 是 750，但 courier_speed 的絕對 550 在重生後才生效（§6 推論）。原作者的意圖不明，可能是 bug。
5. `ModifierGainedFilter` 用的是 `target:GetName()`，不是 `GetUnitName()`（events.lua:436）。對 creature 來說 GetName 可能回傳 class 名稱而不是單位名稱，這樣禁飛判斷可能永遠不成立。實際上 courier2 本來就沒有 courier_morph，所以影響有限。同樣，`Entities:FindAllByClassname('npc_dota_courier2')`（eventforchat.lua:977、986，debug 指令）大概找不到任何東西。
6. 重連後新建的信使沒有設 `spawn_position`（tofar_goback.lua:784-797），之後死亡時 `SetOrigin(nil)` 會發生什麼不確定。
7. 引擎的信使死亡公告與重生訊息（addon_tchinese.txt:129-144）會不會被這個自訂 creature 觸發不確定；Lua 只在 `internal/eventtest.lua:66-67` 列印 courier_lost／respawned 事件。
8. 死亡處理用 `string.match(name, "courier")`（eventforkill.lua:540），也會匹配戰法佔位的 `npc_dota_courier`。不過那些單位多半有 `modifier_invulnerable`。
