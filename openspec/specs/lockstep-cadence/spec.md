# lockstep-cadence Specification

## Purpose
TBD - created by archiving change reduce-lockstep-input-latency. Update Purpose after archive.
## Requirements
### Requirement: shared 120Hz lockstep cadence

系統 SHALL 定義單一共享 `LOCKSTEP_TPS = 120`，並由此推導 lockstep tick period 與 seconds-per-tick。omb lockstep broadcaster、omfx sim_runner、HUD tick-to-time 轉換、log sampling 與 tick-based retention windows SHALL 使用共享 cadence，不得各自寫死 60Hz 常數。

#### Scenario: TickBatch broadcaster emits about 120Hz
- **WHEN** TD_1 執行 5 秒且 lockstep client connected
- **THEN** `omfx_app.log` 的 healthy log 顯示約 600 個 `TickBatch` frames in last 5s
- **AND** log 文字不再宣稱 `Lockstep TickBroadcaster spawned at 60Hz`

#### Scenario: stale 60Hz constants removed from cadence paths
- **WHEN** 搜尋 `omfx/game/src/sim_runner.rs`、`omfx/game/src/lib.rs`、`omb/src/lockstep/tick_broadcaster.rs` 與 `omb/src/main.rs`
- **THEN** lockstep cadence paths 不再使用 `16_667`、`1.0 / 60.0` 或 `snapshot.tick as f64) / 60.0`
- **AND** 對應邏輯改用 `LOCKSTEP_TPS` 或由它推導的 helper

### Requirement: server authoritative dispatcher runs at 120Hz

omb authoritative game loop SHALL 以 120Hz 執行 `State::tick()`，讓 gameplay input apply、host-side pending input drain、script dispatch、outcome processing 與 lockstep broadcaster cadence 對齊。以 tick 數表示 real-time interval 的 constants SHALL 依 120Hz 重新換算，以保留原本秒數語意。

#### Scenario: server TPS constant is 120
- **WHEN** 檢查 `omb/src/main.rs`
- **THEN** authoritative loop 的 `TPS` 為 120
- **AND** `Clock::new` 使用 `1.0 / TPS`

#### Scenario: second-based intervals keep same wall-clock duration
- **WHEN** 檢查 state hash、snapshot 與 visibility diff intervals
- **THEN** state hash interval 仍代表約 10 秒
- **AND** snapshot interval 仍代表約 30 秒
- **AND** visibility diff interval 仍代表原本設計的 wall-clock cadence，而不是因 120Hz 變成 4 倍頻繁

### Requirement: omfx simulation time uses 120Hz dt

omfx sim_runner SHALL 以 `1.0 / LOCKSTEP_TPS` 寫入 `Time`、`DeltaTime` 與 script dispatch dt。HUD `game_time` SHALL 使用 `snapshot.tick / LOCKSTEP_TPS`，避免 120Hz 後顯示時間或 gameplay mirror 速度錯誤。

#### Scenario: one second of ticks equals one second of game time
- **WHEN** sim_runner 處理連續 120 個 TickBatch
- **THEN** `Time` 增加約 1.0 秒
- **AND** HUD `game_time` 增加約 1.0 秒

#### Scenario: gameplay speed does not double
- **WHEN** TD_1 在 120Hz cadence 下跑 10 秒
- **THEN** creep movement、buff countdown 與 projectile travel 使用的 wall-clock 速度與 60Hz 版本等價
- **AND** 沒有因 tick rate 提升而變成 2 倍或 4 倍速度

### Requirement: input lookahead uses 120Hz budget

omfx input submission SHALL 保留 `INPUT_LOOKAHEAD_TICKS = 2` 作為預設，且 target tick SHALL 由 latest lockstep tick 加上 lookahead 計算。在 `LOCKSTEP_TPS = 120` 下，2 tick lookahead SHALL 代表約 16.7ms 的排程緩衝。omb SHALL 保留 late input rejection，但 SHALL expose late input count 或 log 以供評估是否能改成 1 tick。

#### Scenario: target tick is two 120Hz ticks ahead
- **WHEN** omfx 在 latest lockstep tick N submit input
- **THEN** `InputSubmit.target_tick == N + 2`
- **AND** 預估固定 lookahead budget 約為 16.7ms

#### Scenario: late input is observable
- **WHEN** omb reject `target_tick <= current_tick` 的 `InputSubmit`
- **THEN** log 或 metric 記錄 player id、input id、target tick 與 current tick
- **AND** 該 late input 不進入 `InputBuffer`

### Requirement: tick-based diagnostics scale with cadence

所有以 tick 數表示保留時間、sample window 或 log frequency 的 diagnostics SHALL 依 `LOCKSTEP_TPS` 換算，避免 120Hz 後保留時間減半或 log 量翻倍。包含 applied input id retention、每秒 TickBatch sampling、state hash interval 與 snapshot interval。

#### Scenario: applied input retention remains about five seconds
- **WHEN** sim_runner 在 120Hz cadence 下保留 recently applied input ids
- **THEN** retention window 約為 5 秒
- **AND** 對應 tick count 約為 600 ticks

#### Scenario: once-per-second logs remain once per second
- **WHEN** TickBatch sampling log 使用 modulo 判斷
- **THEN** modulo 使用 `LOCKSTEP_TPS` 或等價 helper
- **AND** log 頻率約為每秒一次

