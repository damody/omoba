## 1. Render Pacing State

- [x] 1.1 在 `omfx/game/src/native.rs` 的 `Game` 增加最小 pacing state：上一個可 render 時間、上一個 rendered snapshot tick、paced/skipped frame counters。
- [x] 1.2 使用 `omoba_core::lockstep_timing` 的 shared cadence 推導 render target interval，避免新增獨立 FPS magic number。

## 2. Update Loop Integration

- [x] 2.1 調整 `Plugin::update` 順序，確保 auto hooks、input submission、lockstep event drain 與 TickBatch forwarding 在 pacing 判斷前執行。
- [x] 2.2 在 snapshot tick 未更新且 render interval 未到時，跳過昂貴 render-facing work，並保留下一次可 render frame 會使用最新 snapshot。
- [x] 2.3 確認 pacing 不會阻塞 `sim_runner` worker、不延遲 lockstep event channel drain，也不影響 input target tick 計算。

## 3. Diagnostics

- [x] 3.1 擴充 `FrameProfile` 或等價 log，輸出 target render cadence/interval 與 paced/skipped/capped frame counters。
- [x] 3.2 確認 diagnostics 維持現有 window cadence，不產生 per-entity 或 per-skipped-update log spam。

## 4. Verification

- [x] 4.1 執行 `cargo build --manifest-path D:/omoba/omfx/Cargo.toml -p executor` 確認 frontend build 通過。
- [x] 4.2 執行或觀察一般 dev run，確認 `omfx_render` log 顯示 render FPS 約等於 shared sim cadence。
- [x] 4.3 檢查 render pacing path 沒有新增獨立 `60`、`120`、`16_667`、`8_333` cadence magic number。
