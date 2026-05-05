## 為什麼

目前 omoba 有 `Ping: {ms}` 顯示單向 RTT，但**沒有端到端 input-to-render 延遲量測** — 玩家點下 click 到看見畫面變化的真實感受值是黑盒。這個值是「網路 RTT + lockstep buffer + render frame」總和，比 RTT 更直觀，也是 stress 場景優化的關鍵指標：玩家抱怨「卡」時問題可能在 lockstep buffer 而非 RTT。本 change 加入端到端量測：UI click 時給 input 一個 `input_id` + wall-clock 戳，配對 sim 推進到該 tick 的 snapshot 進入 render 的時刻，算出總延遲，HUD 顯示最近 N 筆的 p50 / p99，並 log 出來給 TD_STRESS 分析。

## 變更內容

- **PlayerInput wire schema 擴展**：`InputSubmit` 加 `input_id: u32`、`InputForPlayer` 加 `input_id: u32`，server 端純 echo back（不影響 sim determinism — id 不進 sim ECS）
- **omfx 端 `PendingInputBook`**：HashMap<input_id, (submit_wall_clock_us, target_tick)>，submit 時 record，sim 跑到該 tick 後配對
- **`SimWorldSnapshot.applied_input_ids: Vec<u32>`**：sim_runner 跑 tick T 時把該 tick 內收到的所有 input 的 id 寫進 snapshot — render 看到 snapshot.tick = T 時就知道哪些 id 該量測
- **`InputLatencyMeter`**：rolling window（最近 N=120 筆，~2 秒）算 p50 / p99 / max，配套 latest 樣本
- **HUD 顯示**：既有 `Ping: ... ms` 行擴成 `Ping: 12 ms | Input→Render p50: 65 ms p99: 120 ms`，跟既有 wire bytes / fps 同列
- **Stress log**：每 sample 的 `(input_id, submit_us, render_us, total_ms, action_kind)` 寫 `omfx_app.log` debug level，可 grep 後算分布
- **Determinism 不變**：`input_id` 嚴格不進 sim ECS / Outcome / 任何影響 hash 的路徑；只走 omfx-only 的 `PendingInputBook` 跟 `SimWorldSnapshot.applied_input_ids` 兩個 metadata channel

## Capabilities

### New Capabilities

- `input-latency-metric`：定義 input_id 編號規則、PendingInputBook 生命週期、sim_runner→render pairing、p50/p99 滾動視窗計算、HUD 顯示格式、stress log 結構；強制 determinism invariant（id 不影響 sim hash）

### Modified Capabilities

（無 — `openspec/specs/` 目前無既有 capability。`input-render-latency` 跟同期的 `lockstep-cleanup-and-hud` 平行進行，後者的 `player-input-routing` capability 不被本 change 改動 — 4 個 PlayerInput 端到端流程不變，只是 wire 上多帶一個 metadata 欄位）

## 影響範圍

- **Wire 協定**：BREAKING — `InputSubmit` 跟 `InputForPlayer` proto schema 加 `input_id: u32`；舊 client 與新 server 不相容（client/server 同步發行）
- **Code**：
  - `proto/game.proto` — `InputSubmit` 加 `input_id = 4`、`InputForPlayer` 加 `input_id = 3`
  - `omfx/game/src/lockstep_client.rs` — submit 時 assign id（`AtomicU32` counter）+ wall-clock；echo back 時 emit `LockstepEvent::InputApplied { input_id, target_tick }`
  - `omfx/game/src/sim_runner.rs` — `SimWorldSnapshot.applied_input_ids: Vec<u32>` 欄位 + `extract_snapshot` 收集當 tick 處理過的 input id
  - `omfx/game/src/lib.rs` — 加 `PendingInputBook` (HashMap) + `InputLatencyMeter` + HUD 顯示 + stress log；4 個 click handler（TowerPlace/Sell/Upgrade/ItemUse）+ ItemUse 鍵盤 + StartRound 全部 wrap submit 加 id
  - `omb/src/transport/kcp_transport.rs` — InputSubmit 解碼後保留 `input_id`，TickBatch broadcast 時帶在 `InputForPlayer.input_id` echo back
  - `omb/src/lockstep/tick_broadcaster.rs` — schedule input 到 target_tick 時保留 input_id metadata
- **Determinism**：sim crate 不動；`input_id` 純 omfx-side metadata + omb-side echo channel，不進 sim ECS；omoba-sim 69 pin tests 全綠（已多次確認 omoba-sim 不依賴 wire schema）
- **Testing**：
  - omfx lib test 加 `PendingInputBook` 生命週期 unit test（submit / pair / evict 過期項）
  - `InputLatencyMeter` rolling window p50/p99 算法 unit test（property-based 測 sorted 不變式）
  - 手動 smoke：TD_1 跑放塔 / 升塔 / Q 升 / 撿物 → HUD `Input→Render` 數字穩定 60-120ms 區間
  - TD_STRESS smoke：1000 entity 場景下 p99 不超 200ms（pass / fail 條件）
- **Dependencies**：無新 crate（既有 `std::time::Instant` + `Vec` 滾動視窗即可）；不需 `histogram` / `hdrhistogram` 等專業 lib（N=120 的 sorted Vec 算 p50/p99 是 O(N log N)，每秒一次更新可接受）
