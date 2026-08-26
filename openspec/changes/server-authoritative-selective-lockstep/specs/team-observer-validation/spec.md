## ADDED Requirements

### Requirement: Outbound 不等待 observer validation

omb SHALL 將 encoded team frame 立即 enqueue 到 team sessions，並以相同 `Arc<[u8]>` 非阻塞 tap 到 bounded validation channel。Validation worker MUST NOT 位於 outbound critical path。

#### Scenario: Validator slowdown 不阻塞送包

- **WHEN** validation worker 被故意放慢
- **THEN** outbound queue 仍依 cadence 接收 team frames
- **AND** player stream latency 不等待 observer step/hash

### Requirement: Team observer replica 隔離

同 process validation worker SHALL 為每個 active team 維護一份 observer replica。Observer SHALL 只透過該 team 的 filtered bootstrap 與 encoded stream 取得資料，MUST NOT 讀 authoritative Specs world、canonical ID mapping 或其他 team state。

#### Scenario: Observer 消費實際 wire bytes

- **WHEN** team frame enqueue
- **THEN** observer 從相同 encoded bytes decode frame
- **AND** 不使用 pre-encode object shortcut

#### Scenario: 兩個 team observer 相互隔離

- **WHEN** match 有 team A 與 team B
- **THEN** A observer 無法查詢 B snapshot、frame 或 replica ID mapping
- **AND** B observer 亦同

### Requirement: Mismatch 非同步回報與收斂

Observer hash mismatch SHALL 記錄 first-divergence tick、team、frame sequence、hash、view/disclosure epoch 與安全 component path，再透過 control channel 回報 `AuthorityRepairCoordinator`。Server SHALL 在後續 frame 發 repair/rebase，observer 與 remote client SHALL 消費相同 correction。

#### Scenario: Mismatch 不改寫已送出的 frame

- **WHEN** observer 在 frame N 發現 mismatch
- **THEN** frame N 已照常送出且不被撤回
- **AND** server 以 N 之後的 authority frame 收斂

### Requirement: Coverage gap 必須可觀察並 rebootstrap

Validation channel overflow SHALL NOT 阻塞 outbound。Server SHALL 記錄 verification coverage gap、丟棄 stale observer、以 filtered snapshot rebootstrap 並從 retained frame 恢復。Coverage gap MUST NOT 記為 validation pass。

#### Scenario: Validator queue overflow

- **WHEN** bounded validation channel 滿
- **THEN** outbound frames 繼續送出
- **AND** metric/diagnostic 記錄 team、gap sequence range 與 rebootstrap result
- **AND** final evidence 不把 gap range 標成已驗算
