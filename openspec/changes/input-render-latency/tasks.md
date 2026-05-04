## 0. Pre-flight audit

- [x] 0.1 Audit `omfx/game/src/lockstep_client.rs` 確認 `LockstepEvent` enum 結構 + 既有 `send_lockstep_input` 函式簽章；列出可加 `input_id` 的注入點（建議在 `send_lockstep_input` 內 assign id 後返回給 caller，或 caller 預先 assign）；no commit
- [x] 0.2 Audit `omfx/game/src/sim_runner.rs::extract_snapshot` 跟 `dispatch_loop` — 確認 sim worker 怎麼從 lockstep_client 接 TickBatch、TickBatch.inputs[] 怎麼變成 sim 內 PendingMoveQueue / PendingTowerSpawnQueue 等 push（這條路徑要記錄 input_id → 該 tick）；no commit
- [x] 0.3 Audit `omb/src/transport/kcp_transport.rs` 跟 `omb/src/lockstep/` 路徑 — 找 `InputSubmit` 解碼點跟 `InputForPlayer` 編碼點，確認 echo 路徑乾淨（input_id 解碼後存哪、編碼時從哪取）；no commit

## 1. Wire schema 擴展

- [x] 1.1 改 `proto/game.proto`：(a) `InputSubmit` 加 `uint32 input_id = 4;`、(b) `InputForPlayer` 加 `uint32 input_id = 3;`；(c) 跑 `cargo build` 在 `omb/` / `omfx/` / `omoba-core/` 三邊（prost build.rs 自動 codegen）；verify 三邊都 clean build；commit `proto: add input_id to InputSubmit + InputForPlayer for end-to-end latency metric`

## 2. omb echo 路徑

- [x] 2.1 omb 收到 `InputSubmit` 時保留 `input_id` 進 PendingPlayerInputs：(a) `omb/src/transport/kcp_transport.rs` `InputSubmit::decode` 後把 `input_id` 連 `target_tick + input` 一起塞進 `PendingPlayerInputs` resource（看 `comp::PendingPlayerInputs` 結構，加 `input_id: u32` 欄位 — 這是 omfx-bound metadata 不影響 sim 邏輯）；(b) verify `cargo build` clean + 145 lib tests；commit `omb: stash input_id alongside target_tick in PendingPlayerInputs`

- [x] 2.2 omb broadcast TickBatch 時把 `input_id` echo 進 `InputForPlayer`：(a) `omb/src/lockstep/tick_broadcaster.rs` 或 `kcp_transport.rs` 對應 broadcast 路徑，建 `InputForPlayer` 時從 PendingPlayerInputs 讀 input_id 放進去；(b) verify omfx 收到 `TickBatch.inputs[].input_id` 跟原 submit 一致（簡單 println 或 lib test）；commit `omb: echo input_id back through InputForPlayer in TickBatch`

## 3. omfx submit 端 — input_id assign + PendingInputBook

- [x] 3.1 加 `input_id_counter: AtomicU32` 跟 helper：(a) `omfx/game/src/lockstep_client.rs` 加 `pub struct LockstepClient { ..., input_id_counter: AtomicU32 }`，初始為 1；(b) `pub fn next_input_id(&self) -> u32` 用 `fetch_add(1, Ordering::Relaxed)`；(c) `send_lockstep_input` 改回傳 `(submit_result, assigned_input_id: u32)`；commit `omfx: input_id_counter for end-to-end latency tagging`

- [x] 3.2 加 `PendingInputBook` 結構：(a) `omfx/game/src/lib.rs` 加 `struct PendingInput { submit_wall_clock_us: u64, target_tick: u32, action_kind: InputActionKind }` + `pub enum InputActionKind { TowerPlace, TowerSell, TowerUpgrade, ItemUse, StartRound, MoveTo, AttackTarget, CastAbility, NoOp }`；(b) `Game` struct 加 `pending_inputs: HashMap<u32, PendingInput>` + `pending_inputs_evict_at: Instant`（每秒 housekeeping）；(c) housekeeping 邏輯：每秒 evict `submit_wall_clock_us > 5000ms ago` 的項；commit `omfx: PendingInputBook with 5s eviction`

- [x] 3.3 5 個 click handler / 鍵盤 handler 全部 wrap 加 input_id：(a) `lib.rs:3160-3181` TowerSell、`:3185-3231` TowerUpgrade、`:3234-3253` TowerPlace、`:3478-3496` ItemUse、`:1414-1428` StartRound 自動 smoke；對每處：呼叫 `let id = lockstep_client.next_input_id()`、`pending_inputs.insert(id, PendingInput { ... action_kind, target_tick: ?, ... })`；(b) `target_tick` 怎麼拿？走 audit 0.2 結果決定（可能是 `lockstep_client.expected_target_tick()` 或從 send 動作得 ack）；(c) verify omfx build clean + 手動 smoke：點塔後 `pending_inputs.len() > 0`；commit `omfx: assign input_id at every PlayerInput submit site`

## 4. snapshot.applied_input_ids + sim_runner 收集

- [x] 4.1 `SimWorldSnapshot` 加欄位：(a) `omfx/game/src/sim_runner.rs::SimWorldSnapshot` 加 `pub applied_input_ids: Vec<u32>`；(b) 加註解「omfx-only metadata，sim ECS 不讀此欄位」；commit `omfx: snapshot.applied_input_ids field`

- [x] 4.2 sim_runner 收集 input_id：(a) audit `sim_runner.rs` 收 TickBatch 的 channel handoff 點（`run_sim_dispatch_loop` 之類），確認怎麼拿到 `TickBatch.inputs[]`；(b) 在 sim worker 內 per-tick 收集 `inputs[].input_id` filter 掉 `== 0` 的，存成 `current_tick_input_ids: Vec<u32>`；(c) `extract_snapshot(...)` 內把 `current_tick_input_ids` clone 進 snapshot 然後 clear 內部 buffer；(d) verify omfx 跑 TD_1 點塔後下一 snapshot 含對應 input_id；commit `omfx: collect applied_input_ids per tick into snapshot`

## 5. InputLatencyMeter + HUD + log

- [x] 5.1 加 `InputLatencyMeter`：(a) `omfx/game/src/lib.rs` 加 `pub struct InputLatencyMeter { samples: VecDeque<LatencySample>, last_compute_at: Instant, cached_p50_ms: u32, cached_p99_ms: u32, cached_max_ms: u32, cached_latest_ms: u32 }`；(b) `LatencySample { input_id: u32, action_kind: InputActionKind, total_ms: u32, submitted_at: Instant }`；(c) `pub fn push(&mut self, sample: LatencySample)`：push_back，超過 capacity (120) pop_front；(d) `pub fn maybe_recompute(&mut self, now: Instant)`：throttled 1Hz 重算 p50/p99/max/latest 進 cached fields；(e) lib test：push 200 筆 → cap=120、push 已知分布 → p50/p99 算對；commit `omfx: InputLatencyMeter with rolling window p50/p99`

- [x] 5.2 接 PendingInputBook ↔ Meter：(a) snapshot 進 render thread 後，對 `snapshot.applied_input_ids` 每個 id：if let Some(pending) = pending_inputs.remove(&id) → 算 `total_ms = (now_us - pending.submit_wall_clock_us) / 1000` → `meter.push(LatencySample { input_id, action_kind: pending.action_kind, total_ms, submitted_at: ... })` + `log::debug!("input_render_latency: id=... kind=... target_tick=... submit_us=... render_us=... total_ms=...")`；(b) 每秒呼叫 `meter.maybe_recompute(now)` 重算 cached；commit `omfx: pair PendingInput → InputLatencyMeter on snapshot consume`

- [x] 5.3 HUD 顯示：(a) `lib.rs:2585-2611` 既有 status string 中 `Ping: {} ms` 後加 `| Lag: p50 {} / p99 {} ms`，空 meter 時 `Lag: —`；(b) 手動 smoke：TD_1 點塔後 HUD 段出現有限數字；commit `omfx: HUD shows Lag p50/p99 alongside Ping`

## 6. Verify gates

- [x] 6.1 omoba-sim determinism tests：`cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features` → 69 全綠（input_id 不影響任何 pin）
- [x] 6.2 omb lib tests：`cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib` → 145 全綠
- [x] 6.3 omfx lib tests：`cargo test --manifest-path D:/omoba/omfx/Cargo.toml -p omfx --lib` 全綠（含新加的 PendingInputBook + InputLatencyMeter unit tests）
- [ ] 6.4 TD_1 60s smoke：點塔 / 升塔 / 賣塔 / Q 升 / 撿物 各幾次，HUD `Lag: p50 ... / p99 ...` 顯示有限數字（typical ~50-150ms localhost）；`omfx_app.log` `grep "input_render_latency:" | wc -l` ≥ 5
- [ ] 6.5 TD_STRESS 60s smoke：跑滿 60 秒，期間定期點塔（至少 30 次），verify：(a) `Lag: p99 < 200ms` HUD 數字穩定；(b) `pending_inputs.len()` 不持續成長（housekeeping 有效）；(c) `omfx_app.log` `grep -c "input_render_latency:"` ≥ 30；(d) 無 desync / panic
- [x] 6.6 grep guard — `input_id` 不出現在 sim 路徑：grep `input_id` 在 `omb/src/comp/`、`omb/src/tick/`、`omoba-sim/src/` — 預期 0 命中（除 `comp/lockstep_resources.rs` 等 wire-edge metadata 模組）
- [x] 6.7 graphify update：`graphify update .` 把本 change 變動同步進 graph；commit `chore: graphify update post input-render-latency`
