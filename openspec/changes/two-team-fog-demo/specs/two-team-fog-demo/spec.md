## ADDED Requirements

### Requirement: Demo 建立精確且 deterministic 的單位配置

`FOG_2TEAM_DEMO` SHALL 以 row-major stable order 建立 10×10、間距 220 world units 的 100 個一般單位，另外建立 Player 1／Team 1 與 Player 2／Team 2 的兩個英雄。一般單位 SHALL 精確分配 Team 1 共 33、Team 2 共 33、Neutral 共 34；16 個固定 stable index 單位 SHALL 使用 deterministic 往返巡邏。

#### Scenario: Demo cardinality 與 team distribution
- **WHEN** server 載入 `FOG_2TEAM_DEMO`
- **THEN** authoritative world 含精確 100 個 grid units 與額外 2 個英雄
- **AND** grid unit team 分布為 33／33／34

#### Scenario: 重跑配置相同
- **WHEN** 使用相同 content hash 建立場景兩次
- **THEN** stable spawn key、位置、team、巡邏集合與初始方向逐項相同

#### Scenario: 不合法 descriptor fail fast
- **WHEN** demo descriptor 含重複 spawn key、非 finite coordinate、錯誤數量或未知 team
- **THEN** loader 回傳明確錯誤且不建立部分場景

### Requirement: 英雄提供 team-shared 圓形 visibility

每個玩家英雄 SHALL 是所屬 team 半徑 700 world units 的 `VisionSource`。一般與 Neutral grid units SHALL 依 `TeamVision` scope 授權；Neutral MUST NOT 自動視為 `Public`。Viewport、camera 與 renderer state MUST NOT 改變 gameplay visibility。

#### Scenario: 兩隊初始集合不同
- **WHEN** 兩位英雄位於相對的初始出生區
- **THEN** Team 1 與 Team 2 filtered bootstrap 的 disclosed entity 集合不同

#### Scenario: 英雄移動觸發 reveal
- **WHEN** 一個 grid unit 進入某隊英雄 700-unit 圓形視野且通過既有 commitment delay
- **THEN** server 以 effective tick fresh baseline reveal 該單位
- **AND** replica 從 server 指定的即時 tick 接續同步

#### Scenario: 離開視野停止 gameplay 同步
- **WHEN** 單位離開 Team 1 視野但仍位於 Team 2 視野
- **THEN** Team 1 不再收到該單位 gameplay state
- **AND** Team 2 繼續收到該單位的 team frame 更新

#### Scenario: Viewport 不影響 disclosure
- **WHEN** Player 1 只移動 camera 或改變 viewport
- **THEN** Team 1 與 Team 2 的 visibility decision 都不改變

### Requirement: Demo 提供可重複的 reveal/hide 人工驗收

16 個巡邏單位 SHALL 沿固定兩點路徑反覆穿越 visibility boundary。人工驗收 SHALL 能觀察不同 team 集合、單隊 reveal/hide、雙隊 overlap 與 LastKnown ghost，且 server diagnostics SHALL 能區分兩隊 observer 狀態。

#### Scenario: 巡邏單位跨越邊界
- **WHEN** 任一指定巡邏單位跨入再跨出 Team 1 視野
- **THEN** P1 視窗依序顯示 live entity 與低透明度 LastKnown ghost
- **AND** ghost 不參與 gameplay query 或 team hash

#### Scenario: 雙隊 overlap 顯示相同 public state
- **WHEN** 同一單位同時位於兩隊視野
- **THEN** 兩隊在各自 opaque identity 下顯示相同 public component state

