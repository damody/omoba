## ADDED Requirements

### Requirement: deterministic gameplay logic directly uses omoba-core

`omb` 與 native `omfx` 的 deterministic gameplay logic SHALL 直接使用 `omoba-core::runtime`、`omoba-core::comp` 或 shared protocol 型別作為 source of truth。若某個 module 只是在 `omb`、`omfx` 與 `omoba-core` 之間轉傳相同 gameplay type、做 identity conversion、或以 prost encode/decode roundtrip 轉換同 process 內等價型別，該轉接層 MUST 被移除或改成呼叫 `omoba-core` public API。

保留的邊界 MUST 有明確職責，例如 transport wire format、render-only projection、thread ownership transfer、launcher process lifecycle、script ABI boundary 或 UI mirror cache。保留邊界不得重新實作 gameplay rule、target selection、tower upgrade rule、ability behavior 或 lockstep tick semantics。

#### Scenario: omb and omfx use omoba-core as gameplay source
- **WHEN** 檢查 `D:/omoba/omb/src/**/*.rs` 與 `D:/omoba/omfx/game/src/**/*.rs` 的 gameplay tick、input apply、outcome processing、snapshot extraction 與 script dispatch paths
- **THEN** deterministic gameplay logic 來自 `omoba-core::runtime` 或 `omoba-core` shared modules
- **AND** `omfx` 不 import `omobab::*` 或 backend-only gameplay modules
- **AND** `omb` 不維護與 `omoba-core::runtime` 等價但分離的 gameplay implementation

#### Scenario: redundant bridge layers are removed
- **WHEN** 搜尋 `D:/omoba/omfx/game/src/**/*.rs` 與 `D:/omoba/omb/src/**/*.rs` 中的 gameplay adapter、bridge、shim、`convert_*`、`encoded_len()` 與 `prost::Message::decode` usage
- **THEN** 不存在只為同 process 內 duplicate gameplay type conversion 而存在的 prost roundtrip 或 identity adapter
- **AND** 保留的 conversion path 都有 transport、render projection、thread transfer 或 external boundary 職責

#### Scenario: behavior remains unchanged after boundary cleanup
- **WHEN** 完成 adapter cleanup 後執行 backend lib tests、native frontend build 與 sim runner smoke tests
- **THEN** tests/builds pass
- **AND** lockstep cadence、TD tower placement/sell/upgrade、hero ability、snapshot rendering 與 VFX cue behavior 維持既有玩家可見結果
