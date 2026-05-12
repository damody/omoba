## ADDED Requirements

### Requirement: sim runner targets shared 120 cadence when healthy

omfx native frontend SHALL drive its local sim_runner from the shared lockstep cadence and SHALL verify healthy TD_1 execution against the shared 120 TPS target. The implementation SHALL NOT introduce an independent hard-coded sim FPS constant outside `omoba_core::lockstep_timing` or equivalent shared helpers.

#### Scenario: sim target derives from shared lockstep timing
- **WHEN** 檢查 sim_runner tick scheduling、render pacing 與相關 diagnostics
- **THEN** cadence 來源使用 `LOCKSTEP_TPS`、`LOCKSTEP_TICK_PERIOD_US` 或等價 shared helper
- **AND** 不以獨立 magic number 寫死 `120`、`8_333` 或其他 tick interval 作為第二套 source of truth

#### Scenario: TD_1 healthy path reaches target cadence
- **WHEN** TD_1 在 backend、lockstep client 與 sim_runner 都健康接收 TickBatch 的狀態下執行至少 5 秒
- **THEN** diagnostics 顯示 sim_runner processed TPS 接近 shared lockstep cadence，允許一般 timer jitter
- **AND** render FPS 接近 shared cadence，或 diagnostics 清楚標示瓶頸不是 runtime full snapshot extraction

### Requirement: cadence diagnostics prove full snapshot is not in runtime path

omfx diagnostics SHALL expose low-volume counters that distinguish init static seed construction from runtime tick processing and runtime lightweight publish. Runtime profile windows SHALL NOT report `extract_snapshot` calls from sim_runner.

#### Scenario: profile window separates init seed from runtime updates
- **WHEN** omfx runs for at least one profile window after sim_runner initialization
- **THEN** logs include processed sim ticks per second or equivalent tick count
- **AND** logs include runtime lightweight publish count or latest published tick
- **AND** logs do not show sim_runner `extract_snapshot` calls

#### Scenario: diagnostics remain low-volume under stress
- **WHEN** TD_STRESS 或一般 dev run 持續執行
- **THEN** cadence diagnostics follow existing profile-window cadence
- **AND** diagnostics do not log once per tick, once per full snapshot, or once per entity
