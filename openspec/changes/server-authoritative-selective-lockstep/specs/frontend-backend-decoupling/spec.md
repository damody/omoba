## MODIFIED Requirements

### Requirement: `omoba-core::runtime` provides the mandatory local lockstep replica boundary

前後端共用的 deterministic selective simulation replica SHALL 位於 `omoba-core::runtime`。`omoba-core::runtime` SHALL 是 mandatory public contract，而不是 optional feature。它 SHALL expose filtered world initialization、V2 team frame transition/input/effect application、fixed tick execution、authority repair/rebase、canonical team hash、render snapshot extraction 與 `SelectiveReplicaRuntime`。omb SHALL consume shared authoritative primitives 並在 validation worker 使用 `SelectiveReplicaRuntime`；native omfx SHALL 使用相同 `SelectiveReplicaRuntime` 執行 team replica。omfx MUST NOT 依賴 `omobab`。

#### Scenario: omb and omfx depend on omoba-core runtime

- **WHEN** 檢查 `D:/code/omoba/omb/Cargo.toml` 與 `D:/code/omoba/omfx/game/Cargo.toml`
- **THEN** 兩者都依賴含 mandatory `runtime` module 的 `omoba-core`
- **AND** dependency direction 是 `omb -> omoba-core` 與 `omfx -> omoba-core`
- **AND** 不存在 `omfx -> omb` dependency edge
- **AND** 不新增 `omoba-runtime` crate

#### Scenario: Client 與 observer 使用 shared selective runtime

- **WHEN** 檢查 native omfx sim runner 與 omb validation worker
- **THEN** filtered bootstrap、transition、tick、repair/rebase 與 team hash 都呼叫 `omoba-core::runtime::SelectiveReplicaRuntime`
- **AND** observer 不使用 authoritative ECS shortcut
