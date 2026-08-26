## ADDED Requirements

### Requirement: Hidden-state non-interference

若兩個 authoritative world 對某 team 的 disclosed state 完全相同，且只在 hidden state 有差異，則在該差異造成規格允許的 public effect 前，該 team 的 encoded frames SHALL byte-identical。

#### Scenario: Hidden movement 不改變 team frame

- **WHEN** world A/B 只有 team A 看不見的 enemy position 不同
- **THEN** team A 在 public causal effect 前收到 byte-identical frames
- **AND** payload length/padding bucket 也相同

### Requirement: Hidden target probing 防護

Client target input SHALL 使用 replica ID 與 view/disclosure epoch。Server SHALL 依 session/team binding、ownership 與 input tick visibility history 驗證。Invalid/stale/unknown target SHALL 使用 generalized rejection class、uniform processing timing 與 rate limit，不得透露 hidden canonical entity 是否存在。

#### Scenario: 猜測 replica ID 不形成 existence oracle

- **WHEN** client 提交未授權或不存在的 replica ID
- **THEN** response class 與 timing 不區分 hidden-existing 與 nonexistent target
- **AND** server 不回傳 canonical ID 或 visibility detail

### Requirement: Player wire 與診斷資料去敏感化

Player frame、filtered snapshot、player-visible log/replay/crash bundle MUST NOT 包含 global seed、canonical ID、其他 team mask、hidden component value 或 full authoritative diagnostic。Full diagnostic SHALL 需要獨立 server-admin capability 與 transport boundary。

#### Scenario: Packet 與 client memory inspection

- **WHEN** final verification 擷取 secure match 封包並檢查 client replica memory/export
- **THEN** 找不到 canonical ID mapping、global seed 或 hidden entity state

### Requirement: Fixed cadence 與 payload padding

Team stream SHALL 以 fixed tick cadence 發送，包含 empty frame。Sensitive payload SHALL 使用 configured size bucket/padding；mass reveal/rebase SHALL chunk/rate-limit 並與 steady-state metrics 分開。

#### Scenario: Hidden-only activity 不造成精確 size oracle

- **WHEN** hidden-only activity 未產生 public effect
- **THEN** team frame 維持相同 cadence 與 padding bucket
- **AND** observer 無法從精確 payload size 推回 hidden entity count

### Requirement: Secure match 禁止 runtime downgrade

Active secure match 發生 gap、mismatch、validator failure 或 rebase failure 時 MUST NOT 改送 global protocol。無法安全收斂時 SHALL 結束 match 並保存 redacted diagnostics。

#### Scenario: Rebase 無法恢復

- **WHEN** secure match 的 filtered rebase 持續驗證失敗
- **THEN** match 安全中止
- **AND** server 不傳 global `WorldSnapshot`、global `TickBatch` 或 `master_seed`
