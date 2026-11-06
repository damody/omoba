# C21_old 宮本武藏（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C21*_old` 說明，招式名取自 KV 註解（`npc/C_Neutral/C21.txt:409、522、605、665`），說明依 KV 與腳本還原（腳本 `vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua`，檔名有空白，只寫在 quirks ref 與數值 source）。

1. **槽位**：`heromap_skill["C21"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:337`），choose_11 逐一加入；自動學習為空，沒有 D。
2. **奧義．弧月斬（C21W_old）**：`C21.txt:409-521`、腳本 346-392。OnCreated 打一次、ThinkInterval 0.2 再打，持續 0.5 秒，一般三跳。只有魔免，沒有沉默。
3. **奧義．連環鋼破（C21E_old）**：`C21.txt:522-604`、腳本 394-450。datadriven 物理傷害＋引擎暈眩；三次「瞬移＋下攻擊命令」（不是立即攻擊），第三跳後就移除無敵（實際約 2 秒）。
4. **二天一流（C21R_old）**：`C21.txt:605-664`、腳本 452-469。機率 RandomInt(1,100) <= N 剛好 N%；保底「計數 > 100/N」換算成連續 5/3/2/2 次沒暴擊後必中。暴擊 modifier 掛在 OnAttackStart，照意圖寫成這一擊的 modify_attack（design_intent）。
5. **秘奧義‧飛翔烈震斬（C21T_old）**：`C21.txt:665-729`、腳本 471-667。Physics 拋物線跳躍，飛行時間隨距離變化；以 leap 固定 1.1 秒表示（依上升段 6000×剩餘比例²、下降段每幀加速度 +3000，在最遠 1300 時估算，`assumed:`）。產生器不支援 `leap.on_end`，改成 leap 後接同長度的 `delay` 在落點結算（效果相同）。
