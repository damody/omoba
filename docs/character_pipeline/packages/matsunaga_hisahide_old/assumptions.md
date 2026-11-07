# B28_old 松永久秀（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `B28*_old` 說明，四招的名稱取 KV 註解（`npc/B_Unified/B28.txt:504,576,707,815`），說明依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位**：`heromap_skill["B28"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:307`），choose_11 逐一 `AddAbility("B28?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；自動學習 `heromap_autoskill["B28"]["11"] = ""`（`addon_game_mode.lua:410`），沒有 D。
2. **鬼謀．火攻（B28W_old）**：依 `npc/B_Unified/B28.txt:504-575` 與 `vscripts/heroes/B_Unified/B28.lua:294-379` 還原。兩段持續傷害的次數（24、16）是照腳本「先累加再比較」與單精度 0.2／0.3 推算的結果，未實測。
3. **鬼謀．綑綁（B28E_old）**：依 `npc/B_Unified/B28.txt:576-706` 與 `B28.lua:5-97` 還原。關鍵推定：舊版 RunScript 沒傳 `duration`，`B28E` 第 22 行對 nil 做運算而報錯，Dota 的 datadriven 在 RunScript 報錯後照常執行後續動作；AddNewModifier 的 duration 為 nil 時 modifier 不限時。所以暈眩等於整個施法時間（不依距離縮短），而分傷從第一次施放起一直生效到久秀死亡，對象是最後一次綁住的目標。這是明顯 bug，D22 比照 D13 已修（fixed）：照本體 B28E 傳入 duration = B28E_duration，分傷只在引導期間、目標被暈眩時生效，暈眩依距離縮短（最短一半），時間到結束引導，施放其他技能解除暈眩。
4. **鬼謀．焰暴（B28R_old）**：依 `npc/B_Unified/B28.txt:707-814` 與 `B28.lua:381-453,507-509` 還原。KV `B28R_old_maxWave` 8 腳本沒用；資料把它當波數上限（施法時間 ÷ 間隔恰為 8）。腳本的 `ability.isChanneling` 旗標以 `caster.is_channeling` 表達。`modifier_stunned` 以 ApplyDataDrivenModifier 套用，照多數既有文件推定套得上。
5. **撕裂大地（B28T_old）**：依 `npc/B_Unified/B28.txt:815-905` 與 `B28.lua:455-509` 還原。KV 沒寫 RequiredLevel，依 AbilityType ULTIMATE 推定 6/12/18。光環殘留 0.5 秒是 Dota 預設（推定）。減速光環只影響英雄與部隊，傷害只打建築。
