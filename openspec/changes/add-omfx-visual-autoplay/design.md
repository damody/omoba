## Context

`omoba-core` 已有可用 coarse profile 跑完 TD 1–100 的 headless autoplay，但函式只回傳最終 report，沒有中途 snapshot 或取消邊界。`omfx` 已能把 `SimWorldSnapshot` 同步到 Fyrox scene，現有 `sim_runner` 則依賴 KCP tick batches。新模式必須讓 autoplay 快轉與 renderer 解耦，且不得改變正式 120 Hz lockstep。

根目錄 launcher 受既有腳本清單限制，因此使用現有 `run.bat --autoplay-100` 作入口。實作橫跨主 repo 與 `omfx` submodule，但不新增 protocol 或 dependency。

## Goals / Non-Goals

**Goals:**

- 以正式 `PlayerInput`、正式 script 與相同 autoplay policy 跑完第 1 至 100 關。
- simulation uncapped 推進，約每 100 ms 發布最新 snapshot 給 omfx。
- 顯示戰場與 round、cash、lives、tower/enemy 數、tick、進度及結果。
- 視窗關閉時可取消 worker，完成或失敗後保留最後畫面。
- 保持 headless 結果、一般 launcher 與正式 multiplayer 行為不變。

**Non-Goals:**

- 不渲染每個 coarse tick，不保證高速動作平滑。
- 不修改 server cadence、KCP frame 或 TickBroadcaster。
- 不把可視化 viewer 當成 CI pass/fail 入口。

## Decisions

1. **Observed runner 與 headless runner 共用一個核心迴圈。** `omoba-core` 提供 observer/cancellation 邊界，既有 `run_td_autoplay_1_to_100` 以 no-op observer 包裝它。這避免在 omfx 複製 policy 或 phase loop；替代方案是直接在 omfx 重寫 loop，但容易與 headless 測試漂移。
2. **Observer 收到 immutable frame，不接觸 ECS。** frame 包含 `SimWorldSnapshot` 與精簡進度狀態，observer 只能回傳 continue/cancel。wall-clock 發布時機不寫入 deterministic state。
3. **omfx 使用專用背景 worker。** worker 將最新 frame 寫入 `Arc<Mutex<_>>`，render thread 只取最新值，不保存 backlog。這比讓 Fyrox update callback 執行大量 ticks 更不易卡住 UI。
4. **發布節流採 wall-clock 100 ms。** round transition、完成與失敗會強制發布。每次發布後 `yield_now`，兼顧接近 headless throughput與約 10 FPS 可見更新。
5. **重用既有 snapshot-to-scene 路徑。** omfx 只新增模式選擇、worker handle 與狀態 overlay，塔、creep、projectile、血條與效果仍由既有 reconciliation 處理。
6. **完成畫面不自動退出。** worker 停止後保留 final frame；使用者關閉 Fyrox 視窗才結束。失敗摘要與 report path 顯示於 overlay 並寫 log。
7. **既有 `run.bat` 新增參數而非新增根目錄檔案。** `--autoplay-100` 設定 `OMFX_AUTOPLAY_100=1` 與無英雄 TD 環境，沿用 freshness build 與 DLL staging。無參數路徑不變。

## Risks / Trade-offs

- [10 FPS snapshot 造成位置跳動] → 明確定位為高速觀察模式，保留一般 `run.bat` 作平滑互動遊戲。
- [snapshot extraction 降低 throughput] → 只發布最新 frame且限制 10 Hz，不以 wall-clock throughput 作通過條件。
- [worker 佔滿 CPU 影響 renderer] → 每次發布後 yield，render thread不執行 simulation burst。
- [omoba-core API refactor 影響 deterministic 結果] → 新增 headless/observed final report parity 與 cancellation tests。
- [omfx worker panic] → handle 捕捉 worker 結果並轉成 `FAILED` 狀態與 log，不讓 render thread panic。

## Migration Plan

1. 先加入 observed runner 與 parity tests，保留既有 headless API。
2. 在 omfx 加入 visual worker 與 frame/overlay 單元測試。
3. 接上環境模式與 `run.bat` 參數，確認一般 launch 不受影響。
4. 執行 headless 1–100、omfx tests 與可視化整合驗收。
5. 若需回復，可移除 `--autoplay-100` 分支與 omfx worker；headless API wrapper仍可保持相容。

## Open Questions

無；完成後視窗保留與約 10 FPS 更新已由使用者確認。
