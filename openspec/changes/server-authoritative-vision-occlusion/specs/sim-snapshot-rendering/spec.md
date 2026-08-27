## ADDED Requirements

### Requirement: team snapshot 只渲染 server 仍揭露的 entity

omfx SHALL 以 server 的 team-filtered snapshot 與 Forget transition 作為 entity 存在的唯一權威。敵方 entity 不再被揭露時，前端 MUST 移除其 scene node、render mirror、label、血條、選取狀態與 replica identity；不得保留 LastKnown 或 remembered presentation。己方英雄 SHALL 依 owner-team 規則持續存在。

#### Scenario: 遮蔽後 snapshot 移除敵方 entity
- **WHEN** 前一 snapshot 含某敵方 entity，而 server 因 LOS 遮蔽在新 team snapshot 中 Forget 該 entity
- **THEN** omfx 在處理新 snapshot 後不再渲染該 entity 的任何視覺或 UI
- **AND** 該 entity 不再能被 hit-test、選取或 target

#### Scenario: client 不自行補畫隱藏單位
- **WHEN** client 依本地位置估計某敵方單位可能仍在原處，但 server snapshot 未揭露該 entity
- **THEN** omfx 不建立或保留該 entity presentation
- **AND** server 後續 Reveal 時才用新的 authoritative baseline 重建

### Requirement: demo 遮蔽物與視野圖形只作為 presentation

omfx MAY 從 public map presentation data 繪製樹木、多邊形輪廓與己方視野邊界，但這些圖形 MUST NOT 參與 disclosure、重新揭露 entity 或覆寫 server 結果。內部 gameplay identity 或未公開 entity state MUST NOT 藉由 presentation descriptor 洩漏。

#### Scenario: 前端估計與 server 結果衝突
- **WHEN** omfx 的 presentation-only 幾何估計認為 target 可見，但目前 server team snapshot 未揭露 target
- **THEN** target 保持不顯示
- **AND** debug UI 可標示 server 結果，但不得補送、補建或補畫 target
