# C05_old 前田慶次（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C05*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名取自 KV 註解（`// 前田慶次 11.2B [騎術]` 等）。

1. **槽位**：`heromap_skill["C05"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:321`），choose_11 逐一 `AddAbility("C05?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；`heromap_autoskill["11"] = ""`（424 行），沒有 D。
2. **騎術（C05W_old）**：說明依 `npc/C_Neutral/C05.txt:400-482` 還原。`bonus_movement_speed` 與 `bonus_attack_speed` 都用鍵名 `"02"`（418-428 行）；推定 KV 保留重複鍵、`%變數` 依名稱查找，兩者都生效（同 B08R_old、B09E_old 的推定）。
3. **雷舞（C05E_old）**：說明依 `npc/C_Neutral/C05.txt:484-587`、`vscripts/heroes/C_Neutral/C05.lua:3-40`（本體發射函式）與 `198-214`（舊版命中函式）還原。
4. **雷擊（C05R_old）**：說明依 `npc/C_Neutral/C05.txt:589-641` 與 `C05.lua:225-248` 還原。
5. **雷鳴豪槍（C05T_old）**：說明依 `npc/C_Neutral/C05.txt:643-733` 與 `C05.lua:250-314` 還原。
   - 每秒傷害的 `C05T_old_OnIntervalThink` 在宣告 `local target` 之前就用 `target`（287-291 行），讀到全域 `target`。推定該全域通常是 nil（有些腳本如 B01.lua:105 會寫全域 target，若剛好是存活單位則粒子會出現在那個單位身上，之後傷害照常），執行期報錯、每秒傷害沒有發生。依 2026-11-07 主 session 決定照設計意圖修：每秒造成 `damage_per_think` 純粹傷害，quirk `FC-166 D22 舊版`。
   - 狀態的施加者原作是目標自己，資料改以慶次為施加者（傷害來源本來就是慶次），差異記在 quirks。
   - KV 沒寫 RequiredLevel，照 Dota 大招預設 6/12/18。
