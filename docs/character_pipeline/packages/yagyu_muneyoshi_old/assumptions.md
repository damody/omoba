# C24_old 柳生宗嚴（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C24*_old` 說明，招式名取自 KV 註解（`npc/C_Neutral/C24.txt:336、397、498、572`），說明依 KV 與腳本還原。

1. **槽位**：`heromap_skill["C24"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:340`），沒有 D；自動學習為空。
2. **柳生新陰流．無刀取（C24W_old）**：`C24.txt:336-395`，datadriven 傷害＋引擎暈眩；腳本 `vscripts/heroes/C_Neutral/C24.lua:129-138` 只放特效。英文名 Muto-dori 為音譯。
3. **柳生新陰流．浮舟（C24E_old）**：`C24.txt:397-496`、`C24.lua:141-193`。固定往點的方向滑行 700（不看點的距離），同時發射固定飛 500 的劍氣。每命中一次刷新 8 秒 +20 攻速並加一層；datadriven 屬性不乘層數，照作者指南 §4 當原作行為（不是 bug）。OnSpellStart 套的 modifier_C24E 不存在於此技能，沒有效果；atkspd／stime 是從本體抄來未使用的值，設計意圖不明，沒有補成效果。
4. **柳生新陰流（C24R_old）**：`C24.txt:498-570`、`C24.lua:195-210`。保底「計數 > 9」＝連續 9 次沒暴擊後必中。暴擊照意圖寫成這一擊的 modify_attack＋cannot_miss。
5. **柳生新陰流．激‧雙嵐陣（C24T_old）**：`C24.txt:572-682`、`C24.lua:212-342`。28 次「斬擊」只是移動與動畫，不傷害；資料不逐幀移動宗嚴，只在結束時放到目標旁 150（原作最後位置是固定世界方位，差異屬呈現）。DISABLE_HEALING 用 `heal_reduction_pct` 100% 表示（產生器不支援 `cannot_be_healed`）。緩速 7 秒內被驅散即補回，以 `reapply_if_removed` 表示（原作是以完整 7 秒重套，資料是補回剩餘時間，差異很小）。英文名 Twin Storm Formation 為意譯。
