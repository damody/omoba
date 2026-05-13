## MODIFIED Requirements

### Requirement: shared 120Hz lockstep cadence

系統 SHALL 定義單一 server-authoritative lockstep cadence，該 cadence SHALL 由 `omb/game.toml` 的 `[server] STEP_FPS` 設定，允許值為 `120`、`90` 或 `60`。omb lockstep broadcaster、omfx sim_runner、HUD tick-to-time 轉換、log sampling 與 tick-based retention windows SHALL 使用 server 宣告 cadence 或由它建立的 shared runtime timing helper，不得各自寫死 60Hz、90Hz 或 120Hz 常數作為獨立 source of truth。

#### Scenario: TickBatch broadcaster follows configured cadence
- **WHEN** `omb/game.toml` 設定 `[server] STEP_FPS = 90`，且 TD_1 執行 5 秒並有 lockstep client connected
- **THEN** `omfx_app.log` 或等價 diagnostics 顯示約 450 個 `TickBatch` frames in last 5s，允許一般 timer jitter
- **AND** log 文字不宣稱固定 `120Hz` 或 `60Hz`，而是顯示實際 configured/server cadence

#### Scenario: stale fixed cadence constants removed from cadence paths
- **WHEN** 搜尋 `omfx/game/src/sim_runner.rs`、`omfx/game/src/lib.rs`、`omb/src/lockstep/tick_broadcaster.rs` 與 `omb/src/main.rs`
- **THEN** lockstep cadence paths 不再使用 `16_667`、`11_111`、`8_333`、`1.0 / 60.0`、`1.0 / 90.0` 或 `1.0 / 120.0` 作為獨立 source of truth
- **AND** 對應邏輯改用 `STEP_FPS` 解析後的 server cadence、server 宣告 metadata 或 shared runtime timing helper

#### Scenario: server config validates supported FPS values
- **WHEN** `omb/game.toml` 的 `[server] STEP_FPS` 設為 `120`、`90` 或 `60`
- **THEN** backend accepts the value and uses it for authoritative cadence
- **AND** 設為其他值時 backend 以明確錯誤拒絕啟動或回報 config validation failure

### Requirement: server authoritative dispatcher runs at 120Hz

omb authoritative game loop SHALL 依 `omb/game.toml [server].STEP_FPS` 執行 `State::tick()`，讓 gameplay input apply、host-side pending input drain、script dispatch、outcome processing 與 lockstep broadcaster cadence 對齊。以 tick 數表示 real-time interval 的 constants SHALL 依 configured step FPS 重新換算，以保留原本秒數語意。

#### Scenario: server TPS comes from game.toml
- **WHEN** `omb/game.toml` 設定 `[server] STEP_FPS = 60`
- **THEN** authoritative loop 的 runtime TPS 為 60
- **AND** `Clock::new` 或等價 scheduler 使用由 `STEP_FPS` 推導的 tick period

#### Scenario: second-based intervals keep same wall-clock duration
- **WHEN** 在 `STEP_FPS = 90` 下檢查 state hash、snapshot 與 visibility diff intervals
- **THEN** state hash interval 仍代表約 10 秒
- **AND** snapshot interval 仍代表約 30 秒
- **AND** visibility diff interval 仍代表原本設計的 wall-clock cadence，而不是因 FPS 設定改變而縮短或拉長

### Requirement: omfx simulation time uses 120Hz dt

omfx sim_runner SHALL 使用 server 在 lockstep start metadata 宣告的 step FPS 或 tick period 寫入 `Time`、`DeltaTime` 與 script dispatch dt。HUD `game_time` SHALL 使用 `snapshot.tick / server_step_fps` 或等價 runtime timing helper，避免 configured cadence 改變後顯示時間或 gameplay mirror 速度錯誤。`omfx/game.toml` SHALL NOT 提供 simulation step FPS override。

#### Scenario: one second of ticks equals one second of game time
- **WHEN** server 宣告 step FPS 為 90，且 sim_runner 處理連續 90 個 TickBatch
- **THEN** `Time` 增加約 1.0 秒
- **AND** HUD `game_time` 增加約 1.0 秒

#### Scenario: gameplay speed does not change with configured cadence
- **WHEN** TD_1 分別在 `STEP_FPS = 120`、`90` 與 `60` 下各跑 10 秒
- **THEN** creep movement、buff countdown 與 projectile travel 使用的 wall-clock 速度等價
- **AND** 沒有因 tick rate 設定改變而變成更快或更慢的 gameplay speed

### Requirement: input lookahead uses 120Hz budget

omfx input submission SHALL 使用 server 宣告 cadence 推導 lookahead 的 wall-clock budget。若仍保留 `INPUT_LOOKAHEAD_TICKS = 2` 作為預設，log、diagnostics 與 late input 評估 SHALL 依 server step FPS 顯示對應秒數；omb SHALL 保留 late input rejection，但 SHALL expose late input count 或 log 以供評估不同 FPS 下是否需要調整 lookahead ticks。

#### Scenario: target tick uses configured cadence context
- **WHEN** omfx 在 server step FPS 為 60 且 latest lockstep tick N 時 submit input
- **THEN** `InputSubmit.target_tick == N + INPUT_LOOKAHEAD_TICKS`
- **AND** diagnostics 將 2 tick lookahead 顯示為約 33.3ms，而不是固定 16.7ms

#### Scenario: late input is observable
- **WHEN** omb reject `target_tick <= current_tick` 的 `InputSubmit`
- **THEN** log 或 metric 記錄 player id、input id、target tick、current tick 與 server step FPS
- **AND** 該 late input 不進入 `InputBuffer`

### Requirement: tick-based diagnostics scale with cadence

所有以 tick 數表示保留時間、sample window 或 log frequency 的 diagnostics SHALL 依 server step FPS 換算，避免 configured cadence 改變後保留時間、sample window 或 log 量偏離預期。包含 applied input id retention、每秒 TickBatch sampling、state hash interval 與 snapshot interval。

#### Scenario: applied input retention remains about five seconds
- **WHEN** sim_runner 在 server step FPS 為 60 的 cadence 下保留 recently applied input ids
- **THEN** retention window 約為 5 秒
- **AND** 對應 tick count 約為 300 ticks

#### Scenario: once-per-second logs remain once per second
- **WHEN** TickBatch sampling log 使用 modulo 判斷
- **THEN** modulo 使用 server step FPS 或等價 runtime timing helper
- **AND** log 頻率約為每秒一次
