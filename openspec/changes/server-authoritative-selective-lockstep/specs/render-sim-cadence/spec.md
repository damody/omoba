## MODIFIED Requirements

### Requirement: omfx render pacing follows simulation cadence

omfx native frontend SHALL 由 shared lockstep cadence 與 V2 replica barrier buffer 推導 default render pacing，而不是以 engine 最大速度 busy-render。Pacing source of truth SHALL 使用 `omoba_core::lockstep_timing` 與 `TeamGameStart.replica_buffer_ticks`。Renderer MAY 在 replica 暫停等待 frame 時重繪 UI，但 MUST NOT 推進 deterministic simulation 或猜測 entity state。

#### Scenario: Render target derives from shared timing

- **WHEN** 檢查 omfx render pacing implementation
- **THEN** frame interval 使用 `LOCKSTEP_TPS` 或 shared helper 推導
- **AND** replica display tick 由 negotiated buffer/tick barrier 決定
- **AND** render pacing path 不寫死獨立 FPS/tick interval magic number

#### Scenario: Renderer 不超前 replica

- **WHEN** V2 stream healthy 且 replica buffer 維持設定深度
- **THEN** renderer 顯示已完成的 replica snapshot
- **AND** 不 render future hidden/predicted deterministic state

## ADDED Requirements

### Requirement: Late team frame 停止 simulation 但保留 UI responsiveness

Expected team frame 未在 barrier 前抵達時，omfx SHALL 停止 `SelectiveReplicaRuntime` step，保留 network receive、input/UI、diagnostic 與既有畫面處理。Frame replay 或 rebase 完成後 SHALL 從 server 指定 tick 恢復。

#### Scenario: Frame gap 不造成 speculative render

- **WHEN** client 缺少 expected `team_sequence`
- **THEN** deterministic replica 不跨越 gap
- **AND** UI/network thread 仍 responsive
- **AND** gap recovery 後 render 從新的 authoritative snapshot 繼續
