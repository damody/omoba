## ADDED Requirements

### Requirement: `input_id` 編號與 wire schema

`omfx/game/src/lockstep_client.rs` SHALL 維護 `input_id_counter: AtomicU32`，每次 `send_lockstep_input` 時 `fetch_add(1, Ordering::Relaxed)` 取下一個 id（從 1 開始；0 保留為 "no metric" sentinel）。`proto/game.proto` 的 `InputSubmit` SHALL 加 `uint32 input_id = 4;`、`InputForPlayer` SHALL 加 `uint32 input_id = 3;`。omb scheduler / broadcaster SHALL 把 `InputSubmit.input_id` 純 echo 進 `InputForPlayer.input_id`，不解析、不寫入 sim ECS。

#### Scenario: 兩次連續 submit input_id 單調遞增

- **WHEN** omfx 連續呼叫 `send_lockstep_input(...)` 兩次
- **THEN** 第一次的 `InputSubmit.input_id` 為 N
- **AND** 第二次為 N+1

#### Scenario: omb echo input_id 不解讀

- **WHEN** omb 收到 `InputSubmit { input_id: 42, target_tick: 1000, ... }`
- **THEN** 對應 tick 1000 broadcast 的 `TickBatch.inputs[].input_id == 42`
- **AND** omb 不對 `input_id` 做任何 ECS 寫入 / Outcome push / hash 計算

#### Scenario: input_id 不影響 sim determinism

- **WHEN** 跑 `cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features`
- **THEN** 69 個 test 全綠（含 8 個 pin hash）
- **AND** 加 / 移除 `input_id` 欄位 OR 改變編號規則對 pin hash 結果**完全不影響**（因 omoba-sim 不依賴 omobab wire schema）

### Requirement: omfx-side `PendingInputBook` 生命週期

`omfx/game/src/lib.rs` SHALL 維護 `pending_inputs: HashMap<u32, PendingInput>`，每筆紀錄 `(submit_wall_clock_us: u64, target_tick: u32, action_kind: InputActionKind)`。submit 時 `insert(input_id, PendingInput { ... })`；snapshot 配對成功（snapshot.applied_input_ids 含此 id）時 `remove(&input_id)` 並把 `(now_us - submit_wall_clock_us) / 1000` 餵進 `InputLatencyMeter`；超過 `MAX_AGE_MS = 5000` 仍未配對的項目 SHALL 在每秒一次的 housekeeping 跑時 evict 避免記憶體洩漏（玩家斷線重連等場景的 input 可能永遠 pair 不到）。

#### Scenario: submit 後 pair 成功並清掉 entry

- **WHEN** omfx submit input_id=42，sim_runner 之後收到 `TickBatch.inputs[].input_id=42` 並 extract snapshot.tick=T 含 `applied_input_ids=[42]`
- **THEN** `pending_inputs.contains_key(&42) == true`（submit 後）
- **AND** snapshot 配對後 `pending_inputs.contains_key(&42) == false`
- **AND** 一筆 `LatencySample { input_id: 42, total_ms: ... }` 進入 `InputLatencyMeter`

#### Scenario: 5 秒未 pair 的項目被 evict

- **WHEN** input_id=99 submit 5.5 秒後仍未配對
- **THEN** 下次 housekeeping 跑時 `pending_inputs.contains_key(&99) == false`
- **AND** **不**進 `InputLatencyMeter`（漏失樣本，不算進 p50/p99）
- **AND** evict 計數加 1（未來 metric expose 用，本 change 不顯示但內部記錄）

### Requirement: `SimWorldSnapshot.applied_input_ids` 蒐集

`omfx/game/src/sim_runner.rs::SimWorldSnapshot` SHALL 加 `applied_input_ids: Vec<u32>` 欄位。`extract_snapshot(tick=T, ...)` SHALL 收集**該 tick 從 TickBatch 處理的所有 `InputForPlayer.input_id`** 進此欄位（id=0 視為 "no metric" 過濾掉不放入）。此欄位純 omfx-side metadata，sim ECS 任何 system / Outcome / Resource SHALL 不讀此欄位。

#### Scenario: tick 處理 N 個 input 反映在 snapshot

- **WHEN** sim_runner 在 tick T 從 `TickBatch.inputs` 處理 3 個 input（id=10, 11, 12）
- **THEN** `SimWorldSnapshot.applied_input_ids` 包含 `[10, 11, 12]`（順序不要求穩定）
- **AND** snapshot.tick == T

#### Scenario: id=0 的 sentinel 被過濾

- **WHEN** TickBatch 內某 input 的 `input_id == 0`（舊 client 沒設 / 未來向下相容）
- **THEN** `applied_input_ids` 不包含 `0`
- **AND** 該 input 仍正常被 sim 處理（不 skip 任何遊戲邏輯）

#### Scenario: 空 batch tick `applied_input_ids` 為空

- **WHEN** sim_runner 在 tick T 處理空 TickBatch（純 heartbeat）
- **THEN** `SimWorldSnapshot.applied_input_ids` 為空 Vec

### Requirement: `InputLatencyMeter` rolling window 計算

`omfx/game/src/lib.rs` SHALL 提供 `InputLatencyMeter` 結構，內含 `samples: VecDeque<LatencySample>`（capacity=120，~2 秒 @ 60Hz）。每筆 sample push_back，超過 capacity pop_front。SHALL 每秒一次（throttled by `last_compute_at: Instant`）clone samples → sort by `total_ms` → 取 `idx_p50 = N/2`、`idx_p99 = ((N as f32) * 0.99) as usize` → 寫進 `cached_p50_ms` / `cached_p99_ms` / `cached_max_ms` / `cached_latest_ms` 4 個 cached field。HUD 讀 cached_*。

#### Scenario: 樣本累積到 capacity 後 ringbuffer 滾動

- **WHEN** 連續 push 130 筆 sample
- **THEN** `samples.len() == 120`（capacity）
- **AND** `samples.front()` 是第 11 筆 sample（前 10 筆已 pop_front）

#### Scenario: 1Hz 重算 p50/p99 cache

- **WHEN** push 第 N 筆 sample 時距離 `last_compute_at` < 1 秒
- **THEN** `cached_p50_ms` / `cached_p99_ms` 不變
- **AND** 距離 ≥ 1 秒則重算 cached 值並更新 `last_compute_at`

#### Scenario: 空 meter HUD 顯示 sentinel

- **WHEN** `samples.is_empty()`
- **THEN** HUD `Lag:` 段顯示 `—`（不是 `0 / 0 ms`）

### Requirement: HUD 整合 + log 輸出

`omfx/game/src/lib.rs` 既有 status string（連線資訊行）SHALL 在 `Ping: ...` 後加 `Lag: p50 {} / p99 {} ms` 段。空 meter 時用 `Lag: —`。每筆 sample pair 成功時 SHALL emit `log::debug!("input_render_latency: id={} kind={:?} target_tick={} submit_us={} render_us={} total_ms={}", ...)` 一行；TD_STRESS smoke 跑完後可 grep `input_render_latency:` 拿到完整樣本集。

#### Scenario: HUD 同行顯示 Ping + Lag

- **WHEN** TD_1 跑一段時間，玩家點過幾次塔
- **THEN** HUD status string 包含 `Ping: ... ms` 跟 `Lag: p50 ... / p99 ... ms` 兩段
- **AND** 兩段在同一行（不換行）
- **AND** 沒任何樣本時 `Lag: —`

#### Scenario: stress log 可被 grep 後分析

- **WHEN** TD_STRESS 60s smoke 跑完
- **AND** `RUST_LOG=omfx::lib=debug` 啟用
- **THEN** `omfx_app.log` 內有多筆 `input_render_latency: id=... kind=... target_tick=... submit_us=... render_us=... total_ms=...` 行
- **AND** `grep -c "input_render_latency:" omfx_app.log` ≥ 樣本數

### Requirement: Determinism 邊界護欄

`input_id` SHALL 嚴禁出現在以下任何路徑：(1) 任何 specs `Component` 結構欄位、(2) 任何 specs `Resource` 結構欄位（除了 omfx-only 的 `pending_inputs` HashMap，但該欄位不在 sim World 內）、(3) 任何 `Outcome` enum variant、(4) 任何 sim 內 system 的讀寫對象、(5) 任何進入 `omoba_sim::state_hash::hash_sorted_by_id` 的 hash payload。

#### Scenario: omoba-sim pin hash 不變

- **WHEN** 加完本 change 後跑 `cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features`
- **THEN** 69 個 test 全綠（含 `fixed64_arithmetic_pin_hash` / `trig_lut_pin_hash` / `rng_sequence_pin_hash` 等 8 個 pin）

#### Scenario: omb lib tests 不破

- **WHEN** 跑 `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib`
- **THEN** 145 個 lib test 全綠

#### Scenario: grep guard — input_id 不在 sim 路徑

- **WHEN** 在 `omb/src/comp/`、`omb/src/tick/`、`omoba-sim/src/` 內 grep `input_id`
- **THEN** 沒有任何匹配（除了 `comp/lockstep_resources.rs` 等純 wire-edge metadata 模組，且該模組的 `input_id` 用法 SHALL 限於 echo back 不進 ECS）
