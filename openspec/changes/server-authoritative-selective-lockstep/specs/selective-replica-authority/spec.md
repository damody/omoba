## ADDED Requirements

### Requirement: 共用 `SelectiveReplicaRuntime`

`omoba-core::runtime` SHALL 提供 mandatory `SelectiveReplicaRuntime`，由 omfx 與 server team observer 共用。Runtime SHALL 等待 expected sequence/tick barrier，依序套用 `PreStep`、執行一個 fixed `Step`、套用 `PostStep` authority，再產生 team hash 與 render snapshot。

#### Scenario: Client 與 observer 使用同一 runtime

- **WHEN** 檢查 omfx sim runner 與 omb validation worker
- **THEN** 兩者都呼叫 `omoba-core::runtime::SelectiveReplicaRuntime`
- **AND** 不存在 server-only duplicate replica step implementation

### Requirement: Server revision 永遠勝出

Client local state 與 authoritative state 衝突時 SHALL 以 server revision 為準。Server SHALL 依範圍使用 `ComponentRepair`、`EntityReplace` 或 `TeamViewRebase`；所有 correction SHALL 通過 team projection/redaction boundary。

#### Scenario: Component conflict 被 authority repair 覆寫

- **WHEN** client disclosed entity HP 與 server team projection 不一致
- **THEN** server 在後續 barrier 發出較新 revision 的 `ComponentRepair`
- **AND** client 覆寫 local HP 並繼續 step

#### Scenario: Repair 不洩漏 hidden source

- **WHEN** repair 原因來自 hidden attacker
- **THEN** correction 只包含 disclosed target 與合法結果
- **AND** 不包含 attacker identity 或 position

### Requirement: Gap replay 與 filtered rebase recovery

Team stream SHALL 維護 bounded encoded-frame replay ring。Replica 發現 sequence gap 時 SHALL 先要求 replay；sequence 已過期時 SHALL 以 filtered `TeamViewRebase` bootstrap，再接續 catch-up frames。Interrupted rebase MUST NOT 在 snapshot ID、chunk hash 與 manifest 全部通過前套用。

#### Scenario: Ring 內 gap 可 replay

- **WHEN** client 缺少仍存在 replay ring 的 frame N
- **THEN** server 重送相同 encoded frame N
- **AND** client idempotently catch up，不需要 global snapshot

#### Scenario: Ring 外 gap 使用 filtered rebase

- **WHEN** client 要求的 sequence 已過期
- **THEN** server 只送該 team 的 filtered rebase
- **AND** client 從 server 指定 sequence 繼續

### Requirement: Late frame 停在 barrier

Replica MUST NOT 猜測或跨越 missing authoritative frame。`replica_buffer_ticks` default SHALL 為 12，允許範圍 3–24 且不得小於 visibility delay。

#### Scenario: Late frame 不造成 speculative step

- **WHEN** expected frame 在 replica tick deadline 前未到
- **THEN** replica 停在該 tick barrier
- **AND** render/network input thread 保持 responsive
- **AND** frame 抵達或 rebase 後才繼續 simulation
