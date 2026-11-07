# C06_old 石川五右衛門（舊版）假設

路徑相對 `game/dota_addons/nobu/scripts/`。原作 `resource/addon_tchinese.txt:3380-3389` 有五招的繁中說明，說明以它為底，數字換成佔位符；和 KV、腳本不符的地方以實際行為為準並記在 quirks。

1. **槽位與 D**：`heromap_skill["C06"]["11"] = "WERDT"`（`vscripts/addon_game_mode.lua:322`），choose_11 逐一 `AddAbility("C06?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；`heromap_autoskill["11"] = "D"`（425 行），`C06D_old`（NOT_LEARNABLE）出生學一級，寫 `hero.auto_learn`，槽位照本體放 D。
2. **狂氣（C06W_old）**：`attack_speed` 與 `move_speed` 都用鍵名 `"01"`（`npc/C_Neutral/C06.txt:563-574`），推定兩者都生效（同 B08R_old 的推定）。說明寫「%攻擊速度」，KV 是攻速點數，照 KV。
3. **殺人越貨（C06E_old）**：
   - 原作 `C06.lua:305` 在給錢、擊殺之前呼叫 `ExecuteOrderFromTable(order)`，`order` 是未宣告的全域變數，通常是 nil → 執行期報錯，整招只剩粒子與音效。依 2026-11-07 主 session 決定照設計意圖修（`FC-151 D22 舊版`）：殺死目標並給四倍賞金。
   - 詞彙表沒有單位的金錢賞金（GetGoldBounty），整招宣告為 hook `c06e_old_murder_and_loot_hook`（`kind = "ability"`，需要手寫 `scripts/heroes/ishikawa_goemon_old/src/hooks/c06_old_murder_and_loot.rs`）；本體沒有可沿用的 hook（本體 C06E 是偷物品）。若之後詞彙表加了單位賞金屬性（例如 `it.bounty_gold`），可以改回資料。
   - 產生器的目標類型沒有「只限非英雄」的組合（`creep+neutral+summon` 會被擋），所以 `target.types` 寫 BASIC 清單、行為裡跳過英雄；差異是點英雄時會被接受並花掉法力與冷卻。
   - 擊殺本身照常給正常賞金，加上額外四倍，總共五倍。KV `level` 3/3/3/4 沒有被使用。
4. **野蠻之力（C06R_old）**：說明寫「攻擊力」，KV 是 `MODIFIER_PROPERTY_DAMAGEOUTGOING_PERCENTAGE`，照 KV 寫 `outgoing_damage_pct`。
5. **搶劫（C06T_old）**：說明寫「傷害」，KV 寫 PURE，腳本 `C06.lua:328` 用 AMHC 魔法傷害，以腳本為準。扣錢對象是 `target:GetPlayerOwnerID()`，對非英雄單位通常無效；資料以 `target.is_hero` 決定是否扣錢。金錢重複計入照本體 FC-129／FC-131 修正。
6. **盜賊藏匿術（C06D_old）**：白天施放會在前搖被 Interrupt（`C06.lua:277-283`），寫成 `requires.condition = "game.is_night"`。OnOrder 解除（`C06.txt:452-464`）在產生器不支援狀態的 `on_order` 觸發，改用 `remove_on = { "attack", "cast", "move" }` 近似：停止／原地待命指令不會解除；自動攻擊會解除（原作推定不會）。`modifier_C06D_old_check` 沒有任何地方套用。KV 的 `A13D_*` 數值沒有被使用。
