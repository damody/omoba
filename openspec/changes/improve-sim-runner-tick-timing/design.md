## Context

`omfx` 的 `sim_runner` 透過 `TickBatchPayload` channel 驅動 local lockstep replica，並在 trace 中以 `omfx::sim_runner::wait_tick_batch` 標記等待時間。現況在 channel 空時使用長時間 `recv_timeout` 等待，實際醒來時機完全依賴 channel 或 OS timer，無法穩定貼近目標 frame deadline。

目前多數 lockstep timing path 假設固定 120Hz，因此 120 FPS 對應約 8.33ms。新需求是 server step FPS 可由 backend `game.toml` 設為 120、90 或 60；client 不自行決定 simulation step FPS，而是服從 server 在 lockstep start metadata 宣告的 cadence。frontend 與 backend 最終會分離打包，因此 `omfx` 也需要自己的 `game.toml`，但該檔只應包含 client-owned 設定，不複製 server-authoritative simulation 設定。

Windows 一般 sleep 精度可能晚醒；若每幀都直接 sleep 到 deadline，累積 jitter 會讓 simulation snapshot publication 與 render pacing 不穩。等待策略仍要拆成兩段：大部分剩餘時間交給 sleep，最後保留短窗口用 `yield`/短輪詢貼近 deadline。

## Goals / Non-Goals

**Goals:**

- 在 `omb/game.toml` 的 `[server]` 區段新增 server-owned step FPS 設定，支援 120、90、60，並由 backend authoritative loop 與 TickBatch broadcaster 使用。
- 讓 server 在 `GameStart` 或等價 lockstep start metadata 中宣告實際 step FPS 或 tick period，client 以該值推導 sim dt、wait deadline、tick-to-seconds 與 diagnostics。
- 建立 `D:/omoba/omfx/game.toml` 作為分離打包時的 frontend-local 設定檔；該檔不包含 `STEP_FPS` 這類 server-authoritative simulation 設定。
- 讓 `wait_tick_batch` 以 server 宣告 cadence 推導每個 tick 的目標 deadline，而不是固定假設 8.33ms 或固定 120 FPS。
- 當距離 deadline 超過精準等待窗口時，只 sleep 到 `deadline - yield_window`，預設保留約 2ms 給 yield loop。
- 在 yield window 內持續檢查 channel，若 `TickBatch` 已到達就立刻回傳；若尚未到達則 `thread::yield_now()` 直到 deadline 或 channel 關閉。
- 用可測試的小型 helper 驗證 120/90/60 下的 sleep duration 計算，避免測試依賴實際 OS scheduler 精度。

**Non-Goals:**

- 不讓 `omfx/game.toml` 覆寫 server step FPS；client simulation cadence 只能來自 server 宣告。
- 不用環境變數作為 FPS 設定主要來源，也不新增 `OMFX_*_FPS` 或 `OMB_*_FPS` 類型 override。
- 不改變 `TickBatchPayload` gameplay input payload、input routing 或 snapshot data contract。
- 不導入 busy-spin 到整個 frame；精準等待只允許在短窗口內使用 yield loop。
- 不嘗試修正所有 Windows timer resolution 問題，也不新增平台特定 timer dependency。

## Decisions

- backend config 使用 `omb/game.toml [server].STEP_FPS`，允許值為 `120`、`90`、`60`。
  - 理由：目前 `game.toml` 既有 keys 採 uppercase style，`STEP_FPS` 清楚表示 authoritative server simulation step，而不是 frontend render preference。
  - 替代方案：使用環境變數或 command-line flag；這不符合需求，且分離打包時不如 config file 可追蹤。

- `omfx/game.toml` 只放 client-owned 設定，server step FPS 不複製到 frontend config。
  - 理由：server 決定 authoritative simulation cadence；若 frontend config 也有 step FPS，容易在分離部署時與 server 不一致，造成 local replica dt、input lookahead 或 diagnostics 錯誤。
  - 替代方案：直接複製完整 `omb/game.toml` 到 `omfx/`；此作法會把 server-only 設定帶到 client，模糊設定所有權。

- server 透過 lockstep start metadata 傳遞 cadence。
  - 理由：client 必須服從 server；在 `GameStart` 加入 `step_fps` 或 `tick_period_us` 可讓 client 在建立 local replica 前取得正確 timing。
  - 替代方案：client 從 `omfx/game.toml` 讀同名設定；此作法要求兩端人工同步，與 server-authoritative 設計衝突。

- 將 fixed `LOCKSTEP_TPS` usage 收斂成 runtime timing object 或等價 helper。
  - 理由：`LOCKSTEP_TPS` 作為 compile-time 120 常數無法表達 server runtime 設定。需要讓 tick period、tick-to-seconds、retention windows 與 diagnostics 可依 `step_fps` 推導。
  - 替代方案：到處分支 `if fps == 120/90/60`；這會擴散 timing 邏輯並增加錯誤風險。

- 將 wait 行為拆成可測試 helper 與實際 channel loop。
  - 理由：真實 sleep/yield 的精度受 OS scheduler 影響，單元測試應驗證「剩餘時間如何分配給 sleep 與 yield window」，而不是驗證 wall-clock 精度。
  - 替代方案：只做 end-to-end timing smoke；此作法 flaky，且難以定位計算錯誤。

- 預設精準等待窗口採約 2ms，並限制 sleep 只能睡到 `deadline - yield_window`。
  - 理由：120 FPS 時 frame 約 8.33ms、90 FPS 約 11.11ms、60 FPS 約 16.67ms；最後保留固定短窗口可降低 timer jitter 對醒來時間的影響。
  - 替代方案：依 FPS 比例調整 yield window；較彈性但需要更多調參，先採固定小窗口較直觀。

- `TickBatch` 已在 channel 中時必須優先處理，不為等待 deadline 額外延遲。
  - 理由：sim_runner 需要快速消化 backlog，否則會進一步拉大 frontend local replica 與 backend lockstep 的差距。
  - 替代方案：每個 tick 都強制等到 computed deadline；這會讓 backlog recovery 變慢，不符合既有 `try_recv` fast path。

## Risks / Trade-offs

- [Risk] `GameStart` protocol 加欄位需要同步更新 server/client generated types。→ Mitigation：在同一 change 內更新 proto generation consumer，並以 build/test 驗證。
- [Risk] runtime timing object 取代固定常數會碰到較多 call sites。→ Mitigation：先集中在 authoritative loop、TickBatch broadcaster、client sim dt、wait deadline 與 diagnostics；保留非 runtime-critical 常數直到實際需要。
- [Risk] `omfx/game.toml` 若被誤放 server-only key，部署者可能以為 client 可覆寫 server cadence。→ Mitigation：文件、sample config 與 tests 明確檢查 frontend config 不含 `STEP_FPS`。
- [Risk] yield window 會增加 CPU 使用量。→ Mitigation：將 yield window 限制在短時間，且只在 channel 空、接近 deadline 時使用。
- [Risk] 若 upstream `TickBatch` 本身晚到，sim_runner 仍無法準時處理該 tick。→ Mitigation：deadline wait 只修正本地等待精度，starvation/backlog diagnostics 繼續保留 upstream 可觀察性。
- [Risk] 用 wall-clock 實測 timing 的測試可能 flaky。→ Mitigation：把 duration 計算抽成純函式，測試計算結果；實際 loop 只做輕量整合驗證或沿用既有 build/test。

## Migration Plan

先在 `omb/game.toml` 加入預設 `STEP_FPS = 120`，維持現有預設行為；再讓 server 將該值寫入 lockstep start metadata，client 改用 server 宣告值。新增 `omfx/game.toml` 時只放 frontend-local keys，並調整 native frontend 預設讀取該檔。若出現 regression，可先將 `STEP_FPS` 保持 120 回到既有 cadence，再回退 runtime timing 與 wait loop 改動。

## Open Questions

實作時需確認 cadence metadata 放在 `GameStart.step_fps` 還是 `tick_period_us` 較適合；兩者擇一即可，重點是 client 不從 `omfx/game.toml` 或環境變數決定 server step FPS。
