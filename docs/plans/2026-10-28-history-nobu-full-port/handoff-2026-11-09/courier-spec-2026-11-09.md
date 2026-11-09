# 信使、儲藏處與商店範圍規格（2026-11-09，E 線主導）

來源細節與每一項的 path:line 在同目錄的 `courier-source-notes.md`。下文的 `UNITS` 是原作
`scripts/npc/npc_units_custom_load.txt`，`VS` 是 `scripts/vscripts/`。原作是 history_nobu，commit
f966720，路徑 C:/nobu/history_nobu。

## 0. 範圍與開關

- 新開關 `shop_range_rule`，寫在 game.toml／recipe，傳到 MobaMatchConfig。
  - **關（第一階段預設）：** 維持現狀。nobu 地圖在商店範圍外購買會回 `OutsideShop`，沒有商店的地圖用自家基地半徑。不生成信使，也沒有 stash。
  - **開：** 照原作運作，見 §2–§4。
- 上線順序：
  1. 規則與開關先 land，預設關閉，各段都有測試。
  2. bot 與信使都完成後，實機局與 nobu 模式預設打開，全面驗證。
  3. 舊的 headless 驗證 recipe 逐步轉為開。
- 不做：飛行信使、速度升級、商店賣信使、透過信使賣道具、秘密商店。
  - 原作把 item_courier 與 item_flying_courier 都移除了，也用 modifier 過濾器禁止飛行與變形。
  - 秘密商店的 catalog 目前沒有 secret 標記。

## 1. 信使單位（照原作 KV）

原作的信使是 `npc_dota_courier2`，中文名「馬」（`UNITS:3306-3379`）。

| 欄位 | 值 | 來源 |
|---|---|---|
| 移動 | 地面 | UNITS MovementCapabilities |
| 移速 | 550（見下方 D13） | VS courier_speed MOVESPEED_ABSOLUTE；KV MovementSpeed 是 750 |
| 轉向 | 0.5 | UNITS MovementTurnRate |
| 生命 | 500，回血 10/s | UNITS StatusHealth／StatusHealthRegen |
| 視野 | 日夜都是 100 | UNITS VisionDaytimeRange／Nighttime |
| 擊殺賞金 | 150 金 | UNITS BountyGoldMin/Max |
| 攻擊 | 沒有 | 原作無攻擊能力 |
| 護甲、魔抗 | 0 | KV 沒寫，用引擎預設（rules:） |
| 碰撞 | 無碰撞（phased），不擋單位 | phased_dummy／for_no_collision |
| 魔法 | 魔免 | for_magic_immune |
| 道具 | 只能攜帶，不能使用（courier_mute） | VS courier_mute |

- **D13 修正：**
  - 原作生成時沒有把 courier_speed 和 for_no_collision 設成 1 級，所以第一次死亡前移速是 750、有碰撞；第一次重生後才變成 550、無碰撞（`eventforkill.lua:540-557`）。
  - 這是原作 bug。照設計意圖，信使從生成起就是 550 移速、無碰撞。
  - 這筆會寫進 fix-candidates.md，policy fixed。
- **保護規則：**
  - 離自家泉水 1200 以內，物理傷害無效。
  - 離自家泉水 1300 以內是 phased。由於信使一律無碰撞，這條只需要照原作記錄。
  - 魔免本來就擋法術。
- **數量與控制：**
  - 每位玩家一隻，英雄第一次生成 0.2 秒後在該英雄出生點生成，主人是該英雄。
  - 只有主人能下指令（`eventforspawned.lua:38-57`）。
  - 不能被友方技能當成英雄（courier tag）。
- **死亡：**
  - 被殺時給擊殺方 150 金。
  - 沒有重生計時，立刻在出生點重生（`eventforkill.lua:540-557`）。
  - 死亡時身上的道具留在信使上，重生後還在。原作是 Dota 單位重生，物品欄保留。
- **斷線處理：** 原作會丟道具並殺掉信使，我們沒有斷線重建流程，所以不做。

## 2. 儲藏處（stash）

- 每位玩家一個 stash，6 格。
  - 來源是引擎預設（rules:dota_stash_slots），repo 沒有 KV。原作那個時期的 Dota stash 是 6 格。
- 開關打開時的購買規則：
  - 英雄在自家或中立商店範圍內購買，道具進物品欄；物品欄滿了就進 stash。
  - 英雄在範圍外購買，道具進 stash；stash 也滿了就拒絕（`StashFull`）。
- 合成：英雄在範圍內時，用物品欄加 stash 的零件合成。範圍外只能用 stash 裡的零件合成，結果留在 stash。這是 Dota 預設行為。
- 自動取貨：英雄在自家商店範圍內時，stash 的道具自動移到物品欄的空格。
- 賣出：只能在商店範圍內賣物品欄或 stash 的道具，照現有退款規則。
- 商店範圍沿用現有的 nobu 地圖 `shop_radius` 720，以及地圖的 shops 清單（E187 §5.9），不改數值。

## 3. 信使指令與運送

原作 7 個 `courier_*` 都是 Dota 內建，我們只實作運送需要的兩個：

| 指令 | 行為 |
|---|---|
| 送貨（courier_take_stash_and_transfer_items） | 信使回自家商店範圍取出 stash 全部道具，最多到信使 6 格滿為止，再走到主人身邊（交付距離 150，rules:assumed），把道具放進主人的空格。主人物品欄滿了，剩下的留在信使身上。主人死亡時停在原地等待。 |
| 回家（courier_return_to_base） | 信使走回出生點，身上的道具在商店範圍內時放回 stash。 |

- 信使沒有指令時停在原地。
- 信使在主人身邊時，主人可以直接把物品欄的道具交給信使，或從信使拿道具。這一項在 UE 呈現完成前可以先只做 bot 和測試介面。
- 千里大劍：目標是信使時，前搖開始就中止（`item_the_great_sword_of_teleport.lua:25-28`）。v3 文件已經寫好（has_tag courier → stop），實機要驗證確實中止、沒有傳送。

## 4. bot

開關打開時，bot 的行為：
- 在商店範圍外照原本的出裝清單買進 stash。
- stash 有道具，而且信使空閒時，下送貨指令。
- 回到泉水時自動取貨。

## 5. 分工

每段單獨 land 並附測試；碰到同步的要做 replica soak。

| 線 | 工作 |
|---|---|
| E | `shop_range_rule` 開關（game.toml／recipe → MobaMatchConfig，預設關）；信使單位模板（照 §1 數值，lua_data 加 codegen）；每位玩家信使的生成與立即重生、擊殺賞金 150；stash 狀態定義（每位玩家 6 格，掛在玩家或英雄上）；fix-candidates 登記；最後的實機驗證與 recipe 切換。 |
| B | 購買進 stash、範圍內自動取貨、stash 合成與賣出（§2）；信使送貨與回家指令、移動和交付（§3）；泉水附近的物理傷害免疫；stash 與信使物品欄的 replica 同步加 soak；bot 行為（§4）；千里大劍對信使中止的實測。 |
| A | 目前沒有必要工作：courier tag 已經接好，千里大劍的 v3 文件已經處理信使。若 B 要把信使模板放進 v3 summons，或需要新的 v3 欄位，再找 A。 |
| C | 信使呈現：沿用現有模型，不花 Tripo 點數，例如現有的馬或坐騎類模型；stash 格子顯示在商店面板；選取自己的信使，送貨與回家兩個按鈕；信使血條與小地圖圖示。 |

## 6. 原作沒有定義、這裡暫定的數值

- stash 6 格（引擎預設）。
- 交付距離 150。
- 信使護甲、魔抗 0。
- 這幾項在程式裡都標為 rules:／assumed:，之後有新證據再改。
