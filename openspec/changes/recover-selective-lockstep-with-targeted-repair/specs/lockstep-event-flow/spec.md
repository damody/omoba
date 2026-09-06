## ADDED Requirements

### Requirement: Team stream 必須支援原地修復與有序續播

Team lockstep stream SHALL 在 client 進入 `AwaitingRepair` 時暫停提交後續增量幀，以有界佇列保存可重播資料，並在修復成功後重試失敗 sequence 及依序續播。正常 repair 或 filtered rebase MUST NOT 被 launcher 視為 process failure。

#### Scenario: 修復後重試失敗幀

- **WHEN** client 成功驗證並套用 server repair response
- **THEN** client 先重試原失敗 team sequence
- **AND** 成功後依序套用暫存的後續 frame
- **AND** renderer 與 runtime process 全程保持存活

#### Scenario: Filtered rebase 取代舊佇列

- **WHEN** server 將恢復升級為 verified filtered rebase
- **THEN** client 原子替換 team replica world
- **AND** client 丟棄 manifest resume sequence 之前的增量資料
- **AND** client 從 server 指定的 resume sequence 繼續

### Requirement: Repair 期間 outbound queue 必須有界

Server 與 client SHALL 對 repair 期間累積的 team frame 使用既有 replay window 或明確容量上限。容量不足時 SHALL 升級 filtered rebase 或依 authoritative outbound policy 阻塞，不得靜默跳過 sequence。

#### Scenario: Repair 等待超過 replay window

- **WHEN** active recovery 尚未完成且失敗 sequence 已離開 replay window
- **THEN** server 升級 filtered rebase
- **AND** client 不套用有缺口的後續 frame
