# C14_old 齋藤義龍（舊版）假設

原作 `panorama/localization/addon_tchinese.txt`（UTF-16）沒有任何 `C14*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位**：`heromap_skill["C14"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:330`），choose_11 逐一 `AddAbility("C14?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；`heromap_autoskill["C14"]["11"] = ""`（`addon_game_mode.lua:433`）。沒有 D，也沒有 C14E2 子技能。
2. **火靈詛咒（C14W_old）**：說明依 `npc/C_Neutral/C14.txt:554-611` 與 `vscripts/heroes/C_Neutral/C14.lua:4-21` 還原。和本體共用腳本，數值不同。
3. **火龍吐息術（C14E_old）**：說明依 `npc/C_Neutral/C14.txt:613-707` 與 `C14.lua:223-253` 還原。舊版沒有 0.1 秒指令限制。
4. **龍脈強韌（C14R_old）**：說明依 `npc/C_Neutral/C14.txt:708-758` 還原。`MODIFIER_PROPERTY_INCOMING_DAMAGE_PERCENTAGE` 寫正值（737、749 行），實際讓受到的傷害提高。依主 session 2026-11-07 決定照設計意圖修（quirk `fixed`，FC-180）：推定原意是減傷，資料寫成受到的傷害降低 6%～12%。另一種讀法是原意為增傷（變數名 `damage_bonus`、本體 C14R 是攻擊加成），這是不確定的判斷。被動 modifier 和 W 的減益同名 `modifier_C14W`：雙方都是舊版義龍時，敵方義龍身上的被動會讓 `C14E_OnProjectileHitUnit`（`C14.lua:227`）判定為已詛咒而灼燒；資料用獨立狀態 id，這個邊角 bug 自然不會發生。
5. **火龍化身（C14T_old）**：說明依 `npc/C_Neutral/C14.txt:760-861`、`C14.lua:84-114`、`vscripts/AON_Cleave.lua:274-298` 還原。`C14T_OnSpellStart` 在 96 行找不到 `C14E` 報錯，推定只中斷該 RunScript，KV 之後的 `ApplyModifier modifier_C14T` 照常執行（datadriven 動作各自獨立）。`RequiredLevel` 沒寫，依 ULTIMATE 預設 6/12/18。`%attack_damage` 推定為攻擊事件的護甲前傷害（同本體）。
