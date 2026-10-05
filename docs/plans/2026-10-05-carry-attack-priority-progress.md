# Carry 普攻與技能時序協調

## 計畫與決策

1. 查明 E195 的普攻可擊殺優先為何仍可能被技能輸入搶先。
2. 在技能決策前只計算一次安全尾刀候選，依正式 owner 攻擊時序決定是否暫緩進攻 intention。
3. 保留生存／資源恢復與既有其他角色行為，僅做當前功能確認，完整驗收留最後。

## 實作

- 尾刀候選提前到 choose_cast 前，後續普通 AttackTarget 共用同一結果，不再各算一遍。
- 已在 Windup 的合法、已披露、範圍內可擊殺兵優先於另一可擊殺兵；active target 僅從本人 command 取得身分，生命／HP／位置／倍率仍來自 current 披露。
- `reserve_last_hit_attack`：目前有可擊殺候選，且 Idle／count 達有效攻擊間隔，或負 count 的 Windup 正在攻擊該候選，才保留本次進攻輸入機會。
- 有效 interval 與正式 hero_tick 同樣使用 base asd／UnitStats 攻速（既有 10 raw 下限）；不另造 windup 秒數、預測未來 impact 或改 gameplay 取消規則。
- `choose_cast_with_attack_priority` 跳過 EnemyUnit／EnemyPoint／ApproachEnemyPoint，保留 SelfHeal／AllyHeal／SelfRecovery／SelfManaRegeneration 的作者順序與原有 rank／CD／cost 等驗證。
- 非 Carry gate=false；sustain、回城、shop、學技能既有順序保留；普通攻擊若 command 已是相同目標則不重送。
- 冷卻及 Backswing 不套用保留；沒有合法候選就不保留，也不讓 hidden 目標造成永久停用技能。

## 當前功能確認

- core `--lib carry_attack_priority`：1 passed，Idle ready、冷卻、匹配／不匹配 Windup、無候選、Backswing。
- base_content 同一篩選：1 passed，正常 Production60Hz，ready 尾刀先選正式 AttackTarget；已接受同目標 Windup 無重複輸入；低 HP 即使進攻政策列在前仍選正常 generated lumen_mend 並真正恢復 HP；Backswing 允許 lumen_bolt。
- 當次 TEMP/TMP 沿 E194 指向 D 槽既有建置目錄，退出還原。未發生新編譯／測試失敗；非 test wrapper warning 已修正，見 E196。

## 邊界

只改善 Bot 輸入優先順序，不宣稱所有 Cast 原本都取消普攻，也不保證最後取得尾刀。未完成完整 farming／波控／100場／UE；未修改 Lua內容、hash、ABI、生成C++或 Blueprint，未部署 DLL。OpenSpec 5.5 保持未勾選、20/30；E177保持。
