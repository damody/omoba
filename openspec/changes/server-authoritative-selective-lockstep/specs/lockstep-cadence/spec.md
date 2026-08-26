## MODIFIED Requirements

### Requirement: shared 120Hz lockstep cadence

系統 SHALL 定義單一共享 `LOCKSTEP_TPS = 120`，並由此推導 authoritative tick period、seconds-per-tick、team frame cadence、observer replica step 與 render/replica buffer 時間。omb authoritative loop、team frame producer、omfx `SelectiveReplicaRuntime`、server observer、HUD tick-to-time、log sampling 與 tick-based retention windows SHALL 使用共享 cadence，不得各自寫死 60Hz 常數。

#### Scenario: TeamTickFrame producer emits about 120Hz

- **WHEN** secure match 執行 5 秒且 V2 client connected
- **THEN** omfx healthy diagnostic 顯示約 600 個 `TeamTickFrame` in last 5s
- **AND** server observer 對相同 team 消費相同 tick range

#### Scenario: stale cadence constants removed

- **WHEN** 搜尋 omfx sim runner、omb team frame producer、observer worker 與 timing paths
- **THEN** lockstep cadence paths 不使用 `16_667`、`1.0 / 60.0` 或獨立 magic FPS 常數
- **AND** 對應邏輯改用 `LOCKSTEP_TPS` 或 shared helper

## ADDED Requirements

### Requirement: Visibility delay 與 replica buffer 由 match 宣告

`visibility_commit_delay_ticks` default SHALL 為 3，允許 2–4；`replica_buffer_ticks` default SHALL 為 12，允許 3–24 且不得小於 visibility delay。`TeamGameStart` SHALL 宣告兩者，server、client 與 observer SHALL 使用相同設定。

#### Scenario: Default 120Hz buffer semantics

- **WHEN** match 使用 default timing
- **THEN** visibility commitment delay 為 3 ticks
- **AND** replica buffer 為 12 ticks，約 100ms

#### Scenario: Invalid timing negotiation 被拒絕

- **WHEN** match 設定的 replica buffer 小於 visibility delay 或超出 bounds
- **THEN** match startup/negotiation 失敗
- **AND** 不以 client-local fallback 值繼續

### Requirement: 完整 cadence/stress 驗證集中到 Phase 6

Implementation Phase 1–5 SHALL NOT 重複執行完整 cadence、stress 或 soak suite；只允許最低限度 compile/focused smoke。完整 cadence、10,000 entity 與 30 分鐘 soak SHALL 在 end-to-end integration 完成後的 Phase 6 集中執行。

#### Scenario: Final verification 才產生 acceptance evidence

- **WHEN** Phase 1–5 完成 implementation deliverable
- **THEN** 不將局部 smoke 記為 cadence/stress acceptance pass
- **AND** Phase 6 evidence index 才記錄 blocking cadence 與 stress result
