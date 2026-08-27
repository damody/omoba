## Why

目前 server 已能依 team 產生不同的 selective frame，但 process 內的 observer 使用 `NoopDisclosedWorldStepper`，只驗證 frame 能否套用，沒有真正重跑 Specs systems、scripts、輸入與隨機流程；同時每 tick 主動送出的 component repair 會掩蓋 simulation divergence。因此現況不是可驗證 gameplay deterministic parity 的 team lockstep replica。

本修正要讓固定兩隊的 MOBA 對局各自擁有獨立、平行執行的完整 team replica，並保留 server 最終權威、戰爭迷霧資料邊界與不中途降級的 secure match 契約。

## What Changes

- Team 1 與 Team 2 各建立一條獨立 validation thread，以及一份只含該隊 disclosed state 的 Specs world。
- 抽出 authoritative server、server observer 與 omfx replica 共用的 deterministic gameplay phase runner。
- 以真正執行 systems、pending queues、scripts、輸入與 external effects 的 Specs stepper 取代 production `NoopDisclosedWorldStepper`。
- `TeamGameStart` 提供同一個 `global_seed`；每個 tick 以 `global_seed + tick` 重建 deterministic RNG stream。
- 平行 system 不直接競爭 RNG；random request 在 barrier 依 stable order 消耗。同隊內呼叫順序必須固定。
- Hidden entity 不進入未授權 team world；hidden 行為影響 disclosed state 時由 server 投影成 sanitized external effect。
- 移除 steady-state 每 tick 主動 component repair；hash 必須在 authority correction 前比較，才能偵測真正 divergence。
- Server mismatch 時依序使用 `ComponentRepair`、`EntityReplace` 或 filtered team rebase 收斂，且永遠以 server 為主。
- Match 建立時固定 bootstrap 兩隊 observer，不再依賴玩家 session 是否成功收到 `TeamGameStart`。
- **BREAKING**：authoritative tick 對 reliable outbound queue 使用阻塞 enqueue；team frame 不得以 `try_send` 靜默丟棄。
- Queue backpressure 超過 watchdog 上限時安全終止 secure match，不跳過 sequence，也不降級 legacy protocol。
- Team 1 與 Team 2 replica 同時執行；跨隊完成順序不得影響 authoritative state、frame bytes或repair決策。
- 補齊三方 differential、故障注入、資訊隔離、10,000 entity 與雙隊長時間驗證。

## Capabilities

### New Capabilities

- `full-team-replica-simulation`: 規範兩隊 filtered Specs world、共用 deterministic phase runner、Reveal/Hide/Forget、scripts、輸入、external effects與hash時機。
- `team-replica-randomness`: 規範由 `global_seed + tick` 建立的tick-local RNG、stable request order與hidden random跨界處理。
- `parallel-team-observer-validation`: 規範兩條獨立team validation thread、match-owned lifecycle、平行驗算、coverage與server-authoritative recovery。
- `reliable-team-frame-delivery`: 規範不可靜默遺失的阻塞outbound enqueue、watchdog、sequence連續性與安全終止。

### Modified Capabilities

- `lockstep-event-flow`: Team frame必須先可靠進入outbound queue，之後才由network與server observer消費同一份encoded bytes。
- `lockstep-cadence`: Authoritative tick在outbound backpressure時允許阻塞，但必須量測deadline miss並受watchdog限制。

## Impact

- `omb/src/state/core.rs`：authoritative phase、雙隊observer lifecycle、blocking enqueue與repair協調。
- `omb/src/transport/kcp_transport.rs`：reliable team frame delivery、相同encoded frame fan-out與watchdog failure path。
- `omoba-core/src/runtime/observer_validation.rs`：從單一Noop worker改為Team 1、Team 2獨立完整Specs workers。
- `omoba-core/src/runtime/selective_replica.rs`：真正的Specs stepper、pre-repair hash與correction boundary。
- `omoba-core/src/runtime/team_projector.rs`：移除steady-state主動repair，補齊global seed與安全external effect projection。
- `omoba-core/src/runtime/native/`：抽出server、observer與omfx共用的deterministic phase runner與filtered world builder。
- `omfx/game/src/`：改用共用filtered replica runtime與相同hash/RNG契約。
- `proto/game.proto`與generated schema：`TeamGameStart`新增或恢復global seed欄位及相關相容性測試。
- 測試與evidence：舊Noop observer parity evidence失效，由真正simulation differential evidence取代。
