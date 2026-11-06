# C25_old 茶茶（舊版）假設

原作有繁中說明（`resource/addon_tchinese.txt:3793-3802`，UTF-16），招式名照它；說明依 KV 與實際腳本重寫（原作說明的數字不一定和 KV 一致）。

1. **槽位與自動學習**：原作 11 版茶茶未開放（`vscripts/addon_game_mode.lua:239` `["11"] = false`），`heromap_skill["C25"]["11"] = ""`（341 行），choose_11 不會加任何技能。依 D22 仍做成獨立英雄，技能取 KV 的五招 `C25D_old`、`C25W_old`、`C25E_old`、`C25R_old`、`C25T_old`（`npc/C_Neutral/C25.txt:391-747`）：W/E/R/T → Q/W/E/R，D 放 D。D 的取得方式推定為出生自動學一級（`hero.auto_learn`、`learnable = false`），依據是本體 16 版 `heromap_autoskill ["16"] = "D"`（444 行）與 C25D_old 的 MaxLevel 1、RequiredLevel 0。
2. **五招腳本函式名稱錯誤（D22 修）**：KV 呼叫的 `C25D_Action1`、`C25W_OnSpellStart`、`C25E`、`C25R_OnSpellStart`、`C25T_OnSpellStart`、`modifier_C25T_bleeding_OnIntervalThink` 都不存在，`vscripts/heroes/C_Neutral/C25.lua:187-327` 定義的是加 `_old` 尾碼的版本。原作執行時只剩 datadriven 的部分（R 的致盲、T 的被動加攻）。照主 session 2026-11-07 的決定，五招都照 `_old` 函式的行為寫，quirk `policy = "fixed"`（FC 待編號）。
3. **一刀．瞬（C25D_old）**：`C25.lua:187-198`。SetAbsOrigin 到目標身上＋0.1 秒相位；魔免目標只瞬移不傷害。KV 註解的「50% 機率瞬移」沒有做，不加。
4. **迴旋裂斬（C25W_old）**：`C25.lua:200-233`。半徑是施法距離 200；只打看得見的敵人（FOW_VISIBLE）。
5. **短刃投擲（C25E_old）**：`C25.lua:246-277`。Timer 每幀前進 35，換算速度約 1050（30Hz），寫成不可閃避的追蹤投射物（design_intent）。
6. **炫目（C25R_old）**：`C25.lua:235-244`。25% 機率 ReduceMana（寫成 burn_mana，不造成傷害）。
7. **放血之術（C25T_old）**：`C25.lua:280-327`。腳本寫死 12 秒（KV 12.1 未用）；OnCreated 與每 2 秒各一跳，共六跳（12 秒到期那刻是否再跳依時序）。每跳不論魔免都暈眩 1 秒並給視野，非魔免才傷害並 Stop；茶茶死亡後改以信使 caster.donkey 為來源繼續傷害，寫 `require_source_alive = false`。腳本 0.1 秒 Timer 以剩餘時間重套，寫成 `reapply_if_removed`。被動加攻 40/50/60 寫在名為 "Properties" 的 Passive modifier。
