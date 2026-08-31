# Late input grace 與玩家 replica 追趕設計

## 背景

本機雙玩家長時間執行時，一開始可以移動，之後所有 `MoveTo` 失效。實機 log 證明 server 仍維持 120 Hz，client runtime 送出的 `target_tick` 卻從落後 11～14 tick 擴大到 71～72 tick。Server 現行 late grace 固定為 64 ms，120 Hz 下只有 8 tick，因此 Player 1 前 5 筆、Player 2 前 46 筆成功後，後續輸入全部成為 `RejectedLate`。

使用者選擇方案 A：將 server late grace 擴大到 1 秒。同時，tick 長期落後視為另一個必須修正的 bug，不能只靠放寬 grace 掩蓋。

## 目標

- 120 Hz 下允許最多 1 秒、也就是 120 tick 的短暫 late input，並由 server retarget 到下一個 authoritative tick。
- Client replica 發生短暫 stall 後必須追上 server，不能永久保留固定 backlog。
- 玩家輸入優先於 presentation、checkpoint 與 catch-up 工作。
- 所有 authoritative frame 仍逐 tick、按順序執行；不得跳過 deterministic simulation。
- 一般 presentation snapshot 可以合併，但 Hide、Forget、ResetView 與 input result 不得丟失。
- Server authoritative、team 資訊隔離與 replica hash 驗算規則不變。

## 非目標

- 不加入 client-side movement prediction。
- 不允許 client 決定實際生效 tick、移動速度、碰撞或路徑。
- 不靠無上限 grace 接受任意陳舊輸入。
- 不降低技能、物品或 entity-targeting secure input 的驗證強度。

## 已確認根因

### Late grace 小於實際 pipeline lag

`LATE_INPUT_GRACE_MS` 為 64 ms。`input_lookahead_ticks()` 只有 2 tick，而 runtime 以自己已處理到的 `TeamTickFrame.server_tick` 計算 target tick。只要 replica pipeline 落後超過約 10 tick，輸入便超過 grace。

### Client runtime 將所有工作串在單一 frame apply 路徑

每個 frame 依序執行 Specs replica、checkpoint 寫出、presentation extraction、IPC publish。Checkpoint 與部分 critical presentation 會 `await` I/O。當單次 stall 產生 backlog 後，runtime 雖然不主動 sleep，卻沒有明確的 catch-up batch，也沒有在 catch-up 時降低非必要的 per-frame presentation 成本，因此處理速率容易只等於輸入速率，無法消除 backlog。

### 現有自動恢復不處理時間落後

Replay 與 rebase 只處理 sequence gap、unsafe frame 或 hash mismatch。Late input 根本沒有進入 authoritative simulation，因此 replica hash 仍正確，不會觸發任何恢復。Server 的 coordinate `InputSubmit` late rejection 目前只寫 log，client 也無法把 transport success 與 authoritative acceptance 區分。

## 設計決策

### 1. Server late grace 固定為 1 秒

將 `LATE_INPUT_GRACE_MS` 從 64 改為 1000。`late_input_grace_ticks(step_fps)` 繼續依 server tick rate 換算，因此 60、90、120 Hz 都代表相同牆鐘時間。

在 grace 內的 late input 一律由 server 排到 `current_tick + 1`。超過 1 秒仍拒絕，避免無上限接收陳舊操作。生效 tick 只由 server 決定。

### 2. 增加可驗證的 late input 結果

Coordinate input 的 server 處理結果必須可觀察：Accepted、Retargeted、RejectedLate。至少記錄 player、input ID、original tick、effective/current tick 與 late-by ticks。若現行 wire contract 沒有通用 input result，新增 server-to-runtime result；runtime 再傳給 renderer。Transport write success 不得再被當成 authoritative acceptance。

### 3. 將 frame apply 與非同步輸出分離

Runtime 的 deterministic frame apply 保持單執行緒與嚴格順序，但 checkpoint 寫出改走 bounded FIFO worker，避免每 tick等待 KCP writer。Queue 滿載時才 backpressure，不能默默丟 checkpoint。

Presentation 保持兩種 lane：

- lifecycle／input result：critical FIFO，不可丟。
- 連續 snapshot：replace-latest，可合併。

### 4. 明確的 catch-up batch

Runtime 每次收到 frame 後，先從 inbound queue 非阻塞收集已到達的連續 TeamTickFrame，再依序套用。Catch-up batch 仍執行每一個 Specs step，但只在 batch 尾端抽取一般 presentation snapshot。

若 batch 內包含 Hide／Forget，directive 必須累積並按原順序送入 lifecycle FIFO。若包含本機 input，對應 input-bearing presentation 與 applied result 必須保持 FIFO，不能被一般 snapshot 越過。

每批設定時間或 frame 數上限，避免長時間 catch-up 餓死 renderer input、shutdown、rebase 或 secure input result。建議起始上限為 32 frame 或 4 ms，任一先到即讓出執行權。

### 5. 輸入優先權

Runtime select loop 必須優先處理 renderer input，再處理有限 catch-up batch。輸入 target tick 仍依方案 A 使用現有 authoritative frame base 加固定 lookahead；1 秒 grace 負責吸收短暫 backlog。Catch-up 修正完成後，正常 lag 應回到 grace 內的小範圍。

### 6. Lag telemetry

新增以下資料：

- inbound queue depth；
- latest received server tick 與 last applied replica tick；
- catch-up batch size、耗時與剩餘 backlog；
- checkpoint queue depth；
- Accepted、Retargeted、RejectedLate 計數；
- late-by tick 最大值與分位數。

只有超過門檻時寫 warning，避免 debug log 本身造成負載。

## 錯誤處理

- Checkpoint worker 中斷：runtime 安全結束 session，不繼續未驗算的 replica。
- Checkpoint queue 暫時滿載：施加 backpressure並記錄 deadline miss。
- Catch-up 遇到 sequence gap：停止 batch，沿用既有 replay。
- Catch-up 遇到 hash mismatch：停止 batch，沿用既有 targeted repair／rebase。
- Late input 超過 1 秒：拒絕並回報 client，不自動重送技能或物品。

## 測試策略

所有完整測試集中在實作最後執行。

- `late_input_grace_ticks` 在 60／90／120 Hz 都約等於 1 秒。
- 落後 119／120 tick 的 MoveTo retarget，落後 121 tick 拒絕。
- Retarget 後只在 `current_tick + 1` 出現在 TickBatch。
- 模擬 inbound backlog 10、72、120 frame，runtime 逐 tick套用且最後追到最新 tick。
- Catch-up 中 lifecycle FIFO 順序不變，latest snapshot 可合併。
- Catch-up 中本機 input result 不得被 snapshot 越過。
- Checkpoint worker 保序、滿載 backpressure、斷線安全停止。
- 長時間 120 Hz 雙玩家 release 測試，至少持續 3 分鐘並週期性輸入。
- 驗收期間 `RejectedLate` 必須為 0；若發生短暫 stall，必須觀察到 Retargeted 後恢復。
- 兩隊 server、client replica 與 observer hash 驗算維持一致，且玩家看不到敵方視野外資訊。

## 驗收標準

- `run_2player.bat` 連續執行至少 3 分鐘，兩位玩家全程可持續下達 MoveTo。
- 正常本機 input-to-presentation 保持在 3 tick 目標附近。
- 人為注入 0.5 秒 runtime stall 後，輸入不永久失效，replica backlog 會回到正常範圍。
- 1 秒內 late input 由 server retarget；超過 1 秒明確拒絕並回報。
- 無 deterministic frame 遺失、無 lifecycle 遺失、無英雄分身、無 team 資訊洩漏。
