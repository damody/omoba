## ADDED Requirements

### Requirement: per-input phase trace lifecycle

omfx SHALL 為每個非零 `input_id` 維護 phase trace。trace SHALL 至少記錄 `on_os_event`、`send_lockstep_input`、`lockstep_client submit_start`、`lockstep_client submit_done`、`client receive TickBatch`、`Game forward to sim`、`sim_runner publish snapshot` 與 `Game pair applied` 的 client-local timestamps。沒有 OS event 的自動 input SHALL 將 origin timestamp 設為 submit call 時間，並標記 origin kind。

#### Scenario: UI input records full client-side trace
- **WHEN** player 透過 mouse 或 keyboard 送出一筆 lockstep input
- **THEN** pending input book 內該 `input_id` 有 `on_os_event` 與 `send_lockstep_input` timestamps
- **AND** lockstep client 寫入 `submit_start` 與 `submit_done` timestamps
- **AND** TickBatch 被收到、轉送、publish snapshot、pair applied 時補齊後續 timestamps

#### Scenario: auto input still produces trace
- **WHEN** `OMFX_AUTO_START_AFTER_SEC` 或 `OMFX_AUTO_NOOP_EVERY_MS` 產生 input
- **THEN** 該 input 有非零 `input_id`
- **AND** trace 的 origin kind 表示不是 OS event
- **AND** latency sample 仍可計入 p50/p99

### Requirement: server-side input queue metadata is echoed without clock sync

omb SHALL 在收到 `InputSubmit` 時記錄 server-edge metadata，至少包含 receive current tick 與 receive timestamp。當 `TickBroadcaster` drain target tick 時，`InputForPlayer` SHALL echo `input_id`，並附帶 `server_receive_tick`、`server_drain_tick` 與 `server_queue_us` 或等價的 server-local queue duration。client SHALL NOT 用 server absolute timestamp 與 client timestamp 直接相減。

#### Scenario: server queue time appears in TickBatch input metadata
- **WHEN** omb 收到 `InputSubmit { input_id: 42, target_tick: T }` 並在 tick T drain
- **THEN** 對應 `TickBatch.inputs[]` 包含 `input_id == 42`
- **AND** 同一筆 input metadata 包含 receive tick、drain tick 與 server queue duration

#### Scenario: cross-machine clock offset does not affect phase math
- **WHEN** client 整理 phase latency sample
- **THEN** client-side phases 只由 client-local timestamps 相減
- **AND** server-side queue duration 使用 server 已計算好的 duration 或 tick delta
- **AND** client 不把 server absolute timestamp 當作 client absolute timestamp 使用

### Requirement: phase latency logs preserve existing total latency metric

既有 HUD `Lag: p50 ... / p99 ... ms` SHALL 繼續使用 submit-to-pair total latency。每筆 pair 成功的 sample SHALL 產生可 grep 的 debug log，包含原本的 `input_render_latency:` 欄位，並新增 phase durations 或額外 `input_latency_phase:` log 以顯示各段耗時。

#### Scenario: existing latency grep still works
- **WHEN** TD_1 跑一段時間且 `RUST_LOG=omfx::lib=debug` 啟用
- **THEN** `omfx_app.log` 仍包含 `input_render_latency:` lines
- **AND** 每行仍包含 input id、kind、target tick 與 total latency

#### Scenario: phase trace identifies handoff cost
- **WHEN** 一筆 input 成功 pair 成 latency sample
- **THEN** debug log 可讀出至少 client submit cost、server queue cost、TickBatch receive-to-forward cost、sim publish cost 與 pair cost
- **AND** 各 phase duration 的總和可用來解釋 total latency 的主要來源

### Requirement: phase trace metadata stays outside deterministic sim

phase trace metadata SHALL 只存在 wire-edge structures、omfx pending input book、latency meter 與 logs。omb/omfx gameplay ECS components、gameplay resources、outcomes、script ABI 與 state hash payload SHALL NOT 讀取或儲存 phase timestamps。

#### Scenario: grep guard excludes phase metadata from gameplay state
- **WHEN** 搜尋 `omb/src/comp/`、`omb/src/tick/`、`scripts/script-abi/` 與 `omoba-sim/src/` 中的 phase timestamp 欄位名稱
- **THEN** 不存在 gameplay state 使用
- **AND** 允許的匹配只在 lockstep wire-edge、transport、input buffer metadata、omfx pending input book 或 tests

#### Scenario: determinism tests remain stable
- **WHEN** 執行 `cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features`
- **THEN** determinism tests 通過
- **AND** phase trace metadata 不影響任何 pin hash
