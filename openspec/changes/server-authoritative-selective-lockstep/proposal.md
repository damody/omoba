## Why

目前 `omb` 會向所有 lockstep client 廣播相同的 `TickBatch`、global `StateHash`、global snapshot 與 `master_seed`；即使 renderer 隱藏敵方單位，未授權資訊仍存在於封包與 client 記憶體，無法提供真正的戰爭迷霧。現在需要把 lockstep 擴充成 server-authoritative selective lockstep，讓每個隊伍只取得可合法得知的 deterministic subworld，同時保留低頻寬、本地 step 與 server 最終裁決權。

## What Changes

- 新增 deterministic team-shared visibility model，整合 fixed-point 自動視野、stealth/detection、script override、scheduled reveal/hide/forget 與 remembered presentation policy。
- 新增 team-scoped opaque `ReplicaEntityId`，player wire 不再暴露 canonical ECS identity。
- 新增 protocol V2 `TeamGameStart`、`TeamTickFrame`、filtered snapshot、random tape、external effect、authority repair 與 filtered rebase。
- 將 Specs tick 重構為兩個 parallel wave：Wave A 同時計算 gameplay `Outcome` 與 `ObservableFact`，commit 後 Wave B 按 team 平行計算 `V[T+1]` 與 projection。
- 將共用 `SelectiveReplicaRuntime` 放入 `omoba-core`，由 omfx client 與 server-local team observer replica 共用。
- Team frame 編碼後立即進 outbound queue；同 process 的 validation worker 旁路消費同一份 encoded bytes，非阻塞驗算每個 active team。
- 新增 server-authoritative `ComponentRepair`、`EntityReplace` 與 `TeamViewRebase`；client/server conflict 一律以 server revision 為準。
- 新增 anti-probing、non-interference、payload padding、redacted diagnostics 與 secure-match no-downgrade 規則。
- **BREAKING** Secure fog match 不再向 player session 傳送 global `TickBatch`、global `StateHash`、global `WorldSnapshot`、global `master_seed` 或 raw ECS ID。
- **BREAKING** Client target input 改以 team-scoped replica ID 與 disclosure/view epoch 表示，server 依 input tick 的 visibility history 驗證。
- 完整 unit、differential、fault、security、packet inspection、10,000-entity stress 與 30 分鐘 soak 集中到所有 server/client integration 完成後的 final verification phase，不在每個 implementation phase 重複執行。

## Capabilities

### New Capabilities

- `team-visibility-projection`: 定義 team-shared visibility、ECS visibility component、override precedence、scheduled transition、projection policy 與跨 visibility boundary 行為。
- `selective-lockstep-protocol`: 定義 protocol V2 team bootstrap、team tick frame、team-scoped identity、filtered snapshot、canonical ordering 與 secure match negotiation。
- `selective-replica-authority`: 定義 `SelectiveReplicaRuntime`、server-wins revision、repair、rebase、barrier buffer、gap replay 與 rejoin recovery。
- `team-observer-validation`: 定義同 process 非阻塞 validation worker、每隊 observer replica、encoded-frame tap、coverage gap 與 mismatch reporting。
- `secure-fog-information-boundary`: 定義 hidden-data non-interference、anti-probing、randomness isolation、payload side-channel control、redacted diagnostics 與 no-downgrade security invariant。

### Modified Capabilities

- `lockstep-event-flow`: 由 global event flow 改成 Wave A `Outcome`/`ObservableFact` 與 per-team projected event/external effect flow。
- `lockstep-cadence`: 保留 shared 120Hz authoritative cadence，新增 visibility commitment delay、replica buffer lead 與 team frame barrier semantics。
- `player-input-routing`: Target input 改用 replica ID/view epoch，並新增 team binding、visibility-history validation 與 generalized anti-probing rejection。
- `frontend-backend-decoupling`: `omoba-core::runtime` 的共用 replica boundary 改為 `SelectiveReplicaRuntime`，server observer 與 omfx 必須使用同一實作。
- `sim-snapshot-rendering`: Global sim snapshot rendering 改為 filtered team snapshot、disclosed entity render state與獨立 remembered render cache。
- `render-sim-cadence`: Render/sim handoff 新增 replica barrier buffer、scheduled transition 與 late-frame stall 行為。

## Impact

- `proto/game.proto` 與 `omoba-core/src/generated/game.rs`：新增 protocol V2 wire schema，舊 global player wire 進入遷移／移除流程。
- `omoba-core/src/runtime/**`：新增 selective replica runtime、team hash、transition、repair/rebase、filtered snapshot 與 deterministic projection contract。
- `omb/src/lockstep/**`、`omb/src/state/**`、`omb/src/transport/kcp_transport.rs`：改為 team stream、visibility history、per-team replay ring、repair coordinator 與 validation sidecar。
- `omb/src/vision/**`：既有 wall-clock/floating-point vision 不再作為 authority，改為 deterministic fixed-point visibility path。
- `omb/src/comp/**`、`omb/src/tick/**`、`scripts/base_content/**`：gameplay outcome 同步產生 stable `ObservableFact`，並為每個跨 visibility 行為聲明 projection policy。
- `omfx/game/**`：global local replica 改成 team selective replica，新增 barrier buffer、remembered cache、transition、repair/rebase 與 filtered snapshot bootstrap。
- Network/security：secure fog match 與 legacy V1 以 match-level capability negotiation 隔離；active secure match 不允許降級。
- Performance：10,000 entity 場景需同時承載 authoritative world、per-team projection 與每隊 observer replica；outbound path 不等待 validation。
- Release：Phase 1–5 完成整合後，Phase 6 才集中執行完整驗證、shadow/dogfood、secure-default cutover 與 global player path cleanup。
