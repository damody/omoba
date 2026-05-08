## 1. Shared Cadence Constants

- [x] 1.1 新增 client/server 共用的 lockstep timing 模組，定義 `LOCKSTEP_TPS = 120`、tick duration 與 seconds-per-tick helper。
- [x] 1.2 將 `omb/src/lockstep/tick_broadcaster.rs` 的 `TickBroadcasterConfig` 改用共享 timing helper，並更新 120Hz log 文字與 unit tests。
- [x] 1.3 將 `omfx/game/src/sim_runner.rs` 的 `SIM_DT_S`、`Time`、`DeltaTime` 與 script dispatch dt 改用 `LOCKSTEP_TPS`。
- [x] 1.4 將 `omfx/game/src/lib.rs` 的 HUD `game_time`、TickBatch sampling、`APPLIED_INPUT_ID_RETENTION_TICKS` 與相關註解改用 `LOCKSTEP_TPS`。
- [x] 1.5 將 `omb/src/main.rs` authoritative `TPS` 改為 120，並確認 `Clock::new` 仍使用 `1.0 / TPS`。
- [x] 1.6 將 `omb/src/state/core.rs` 中以 tick 表示秒數的 intervals 依 120Hz 重新換算，保留 state hash 約 10 秒、snapshot 約 30 秒與 visibility diff 原 wall-clock cadence。

## 2. Server Edge Metadata

- [x] 2.1 擴充 `proto/game.proto` 的 `InputForPlayer`，加入 `server_receive_tick`、`server_drain_tick` 與 `server_queue_us` 或等價欄位。
- [x] 2.2 更新 omb 與 `omoba-core` prost 產生路徑，確保新增欄位在 server/client 兩邊型別可用。
- [x] 2.3 擴充 `omb/src/lockstep/input_buffer.rs::BufferedPlayerInput`，保存收到 input 時的 current tick 與 server receive timestamp。
- [x] 2.4 在 `omb/src/transport/kcp_transport.rs` 收到 `TAG_INPUT_SUBMIT` 時填入 receive metadata，late input log 加上 `input_id`。
- [x] 2.5 在 `TickBroadcaster::fire_one_tick()` drain input 時填入 drain tick 與 server queue duration，並 echo 到 `TickBatch.inputs[]`。
- [x] 2.6 更新 lockstep broadcaster/input buffer tests，驗證 metadata echo 且不影響 input 排序。

## 3. Client Phase Trace

- [x] 3.1 將 `omfx/game/src/lockstep_client.rs` 的 outgoing input tuple 改成具名 struct，攜帶 `target_tick`、`PlayerInput`、`input_id` 與 trace timestamps。
- [x] 3.2 在 `omfx/game/src/lib.rs` 為 OS event input 捕捉 `on_os_event` timestamp；auto input 使用 submit call timestamp 並標記 origin kind。
- [x] 3.3 擴充 `PendingInput`，保存 per-phase timestamps、origin kind、server queue metadata 與 action kind。
- [x] 3.4 在 lockstep client submit 前後記錄 `submit_start` / `submit_done`，收到 `TickBatch` 時記錄 client receive timestamp。
- [x] 3.5 在 `Game::update` 轉送 TickBatch 到 sim_runner 時記錄 forward timestamp，並把 server metadata 與 client receive timestamp 合併回 pending book。
- [x] 3.6 在 `sim_runner` publish snapshot 時讓 applied input metadata 可回到 render thread，讓 `Game pair applied` 能補齊 `sim_publish_snapshot` 與 pair timestamps。
- [x] 3.7 擴充 `InputLatencyMeter` sample，保留既有 total latency p50/p99，並新增 phase duration 計算與 debug log。
- [x] 3.8 更新 input latency unit tests，涵蓋 trace lifecycle、auto input、stale eviction 與 existing `input_render_latency:` grep 相容性。

## 4. Determinism Guardrails

- [ ] 4.1 確認 `input_id`、phase timestamps 與 server queue metadata 不寫入 gameplay ECS components、resources、outcomes 或 state hash payload。
- [ ] 4.2 加入或更新 grep guard 測試，限制 phase metadata 只出現在 transport、lockstep wire-edge、input buffer metadata、omfx pending book 與 tests。
- [ ] 4.3 檢查 120Hz 後 movement、buff、projectile、attack cooldown、wave timing 與 script dt 使用，修正任何 tick-dependent speed regression。

## 5. Verification

- [ ] 5.1 執行 `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib`。
- [ ] 5.2 執行 `cargo test --manifest-path D:/omoba/omoba-core/Cargo.toml` 或等價 omoba-core test command。
- [ ] 5.3 執行 `cargo test --manifest-path D:/omoba/omfx/Cargo.toml -p omfx` 或最接近的 omfx game crate tests。
- [ ] 5.4 執行 `cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features`，確認 determinism tests 與 pin hash 不受影響。
- [ ] 5.5 跑 TD_1 smoke，確認 lockstep healthy log 約 600 TickBatch frames / 5s、`Lag` p50/p99 下降、late input log 沒有異常增加。
- [ ] 5.6 跑 TD_STRESS 或既有 smoke long，確認 120Hz 不造成 TickBatch starvation、KCP session removal 或 server tick profile 超出預算。
