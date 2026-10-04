# Renderer 重連安全基準補強

## 計畫與決定

本輪依 OpenSpec `build-unreal-rust-moba-framework` 4.3 檢查一次性效果與新 renderer 連線。先封住重連首個 snapshot 的歷史播放，再驗證既有輸入與視野回歸；不將效果欄位目前為空當成沒有重播風險。

- 第一個完整安全 snapshot 是狀態恢復基準：清除一次性 `effects`／`audio_cues`，保留可見 entities、server-sanitized frozen ghosts、fog、tick、view epoch 等持續狀態。
- 同時處理連線時已有完整 latest snapshot，以及先收到 RuntimeReady、後由 latest 或 critical 送來第一個完整 snapshot。
- 僅基準傳送複製 envelope，不變更共享 watch source；正常 live snapshot 無新增複製，仍保留其一次性資料。
- 舊於完整基準的 critical snapshot/lifecycle 仍由既有 covered sequence 規則排除；terminal input results 不被當成效果刪除。
- 不對 buff/script 狀態事件套用 projectile dedup：它們可能是 actor 重建所需狀態或同 tick 合法多次行為，缺乏穩定事件 ID 時不能任意吞掉。
- RendererConsumed 只能回報本連線實際送出的完整 snapshot 界線；重複／較舊 lease 冪等處理，未來回報明確拒絕並斷線。這不是 UE 消費流程已接通的宣告。
- 發現 Cargo tests 可在 stage 後重產 DLL，已加入 SHA-256 gate：完整 build 在 Editor 啟動前驗證，雙 client launcher 在開局前亦驗證（包含 skip UE build）。新 `--verify-staged-only` 模式不建置、不 stage、不啟動程序。

## 驗證

- runtime 33 library + 2 binary unit passed。
- 真實 TCP 的兩個先後 renderer（新 socket、新握手）在沒有新 simulation frame 時取得相同 persistent view，歷史 VFX/audio 清除，來源 snapshot 未被修改。
- RuntimeReady 後的第一個 snapshot，在 latest/critical 兩條路徑都清除歷史效果；後續 live cue 仍送達。
- 終局 input result 保持原值，只有 snapshot 的兩個 one-shot 陣列被修改。
- 真實 socket 的有效、重複、較舊 consumed 回報不影響新 live cue；未來 consumed 序號 999 被拒絕，連線已關閉。
- 完整 UE build sessions 98625／31119／31227 exit 0；最終 Editor PID 10556、MCP ready。31227 之後沒有再執行會重建 bridge DLL 的 Cargo 指令。
- 最終 stage SHA-256 與 bridge build DLL 一致：`5e1e69ff5f81aed20d8fc75f87e1a44fc73659e4b0d213bb53aef076bd588b00`。
- session 7289 Editor 回歸：同一 Editor 兩輪各 7/7，failed/skipped/not_run=0；PIE native/ghost 都 rendered=true、memory counts=[1,0]，自行啟動 PIE 已停止。
- stage gate 的相同／不同／缺檔三個 pure fixture 通過，沒有啟動任何 process；首次 fixture 的 Lua and/or nil 錯誤已修正並記錄 E041。
- dual run 1791015128 PASS；最終新 gate + 雙 client run **1791015412 exit 0**，兩隊自身 filtered view 與 UE/replica 移動斷言均通過，五個 child processes 已清理、active session 登記移除。完成後 read-only SHA-256 gate 再次通過。
- Lua module tests、6 個 UE observation scenarios、codegen --check、diff whitespace check 通過。

## 剩餘範圍

4.3 保持未勾選。現行正式 IPC snapshot builder 仍未把所有 MOBA effects 對應到 Unreal C ABI；RendererConsumed 不能直接用 UE 本地 frame sequence 代替 IPC sequence。下一段需要明確來源事件 ID、序號對應與 renderer 消費契約，才能驗收所有 retained cue 在持續 live frame 與跨 process 重連不重播。

首個恢復基準不播放 offline 歷史效果，是刻意的 at-most-once 呈現決策，不保證離線期間的效果必定播放。這不影響權威傷害、buff、技能結算，也不放寬視野／輸入驗證。

錯誤與預防：`docs/plans/unreal-moba-error-register.md` E038–E041。
