# 不新增對局範圍收尾與剩餘驗收條件

2026-10-20，Unreal 前端 `D:/omoba/omfue`，引擎 `D:/UE5.8`。

目前功能的本機實作與完整分段驗收已有通過證據。OpenSpec 仍為 **33/44，11 項待驗**，完整 MVP 尚未完成。這 11 項不能只靠修改程式或重跑同一組本機測試勾完；本輪沒有新增權威對局，也沒有重設既有 10/10 額度。

## 剩餘 11 項逐項收尾條件

| 項目 | 已有本機實作／證據 | 還缺的完成條件 |
|---|---|---|
| 3.1 | 世界移動／攻擊、按住節流與 UI／焦點取消；完整操作事件已驗 | 對原權威命令觀察 MoveTo／AttackTarget ACK 與實際權威結果 |
| 3.3 | AttackMoveClick、MoveOnlyClick、AttackOnlyClick、游標與目標規則已接線並回歸 | 各模式原權威命令 ACK 與移動／攻擊行為對應 |
| 3.4 | Stop／Hold 分開命令、按鍵節流與 queue；Rust／本機事件已驗 | 真權威 ACK 及 Stop／Hold 實際差異 |
| 3.5 | hero-only、精確修飾鍵、auto attack 設定與可見目標限制；本機／Rust 已驗 | 權威端採用設定、明確攻擊與自動尋敵的實際結果 |
| 3.9 | ward 資料／充能／生命週期、transport、HUD／hotkey 與 Runtime 退休已接線 | 原權威插眼 ACK、實際充能與隊伍視野結果 |
| 4.3 | 命令關聯、成功／拒絕／不確定提示、游標、選取與公開 HP 數字已驗 | 原 gameplay ACK 對應可見回饋，不以受控 feed 取代 |
| 4.4 | shop 搜尋／分類／recipe、purchase／sell／undo 關聯與 receipt 生命週期已驗 | 真權威 purchase／sell／undo receipt 與經濟結果 |
| 4.7 | HUD／minimap、字型／配色／可改鍵顯示及多尺寸本機測試已驗 | 同功能在實際 OS 100% 與 150% DPI 的配對證據 |
| 7.1 | native／原 Blueprint 輸入、camera、UI 與原命令 wire 接線已驗 | 全 action 原權威 ACK 與實際結果的完整對照 |
| 7.3 | 實際 standalone PIE native／原 Blueprint、停止重啟與兩種視窗模式；Runtime／生命週期回歸已驗 | 實際 OS 100%／150% DPI 配對完成矩陣 |
| 7.4 | 可用的本機分段完整操作、錄影與清理證據 | 真對局／LAN 及原門檻下的單 session 固定效能證據 |

不得以調整 Slate application scale、DPI 曲線或合成幾何取代實際 OS DPI；不得以兩個本機 publisher session 的案例聯集冒充單 session 效能或 LAN。後端／Rust helper 成功也不等於已觀察原權威對局 ACK。

## 當前完整接受基準

- [完整報告](2026-10-20-audio-locomotion-final-acceptance.md)：UE5.8 普通 local、無 action cache 建置 22.80 秒；26 專項、206 native、458 操作案例／222 wire／157 stages／13 cursor、69 Blueprint、五份 PCM、140＋69 Lua 契約成功。
- [封存 index](../acceptance/2026-10-20-audio-locomotion/index.json)：38 份 SHA／size 證據；四筆專項與完整 native 的既有警告保留。
- 527 Rust／ABI 與 93 內容 Lua 輸入未變，完整 Rust 測試明示沿用既有接受，並非本輪重新執行。
- 實際 PIE 記錄在 `omfue/Saved/Automation/AudioLocomotionWorldV1/engine.log`：四輪 native／原 Blueprint、windowed／borderless 均成功，實測 `os_dpi=1.5000`，並明示沒有不同的實際 100%／150% monitor pair。

本輪只核對目前來源與這份已通過的完整接受是否仍一致，並重新核對封存與原始操作報告。沒有宣稱 fresh 全部測試重跑。獨立來源比對與收尾核對結果分別存於 `omfue/Saved/BuildTemp/nonmatch-closeout-source-verified-v2.json`、`omfue/Saved/BuildTemp/nonmatch-closeout-verified-v1.json`。

## 後續接續規則

一般「繼續」應先查這張清單及最新 handoff。若外部執行條件與授權未變，不應用沒有直接對應缺證的新局部修正，持續取代剩餘 11 項；也不應為程式未變的狀態反覆重建與重跑完整測試。若發現可重現的新缺陷，仍應另記問題、回歸與修正。現有對局額度與不新增對局的限制保持，未取得真證據的父項不得勾選或歸檔。

E1432 記錄本輪來源複驗輸出命名疏漏及改用獨立輸出的處理；先前錯誤均保留於 [錯誤紀錄](unreal-moba-error-register.md)。
