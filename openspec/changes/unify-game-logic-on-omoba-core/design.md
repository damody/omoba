## Context

`omoba-core` 已包含 shared protocol、ECS components、runtime tick、outcome processing、snapshot extraction 與 script ABI integration。`omb` 是 authoritative backend，native `omfx` 是 render frontend，但兩者都需要 deterministic gameplay simulation 的同一份邏輯。

既有 spec 已要求 `omfx` 不依賴 `omobab`，且 `omoba-core::runtime` 是 mandatory local lockstep replica boundary。本 change 的重點不是建立新 runtime，而是盤點目前是否仍有歷史轉接層、duplicate type conversion 或 frontend/backend 專用 shim，並在安全範圍內移除，讓兩端直接使用 `omoba-core` 的 API 與型別。

## Goals / Non-Goals

**Goals:**

- 確認 `omb` 與 native `omfx` 的 gameplay logic dependency direction 都是透過 `omoba-core`。
- 移除沒有明確職責的 adapter、bridge、duplicate gameplay type conversion 或 backend-specific wrapper。
- 保留必要邊界：transport/wire protocol、render-only projection、launcher process lifecycle、script DLL ABI 與 platform-specific frontend rendering。
- 讓驗證方式可重複：搜尋 dependency/import/conversion pattern，並執行前後端相關 build/test。

**Non-Goals:**

- 不把 `omfx` rendering、UI layout、asset loading 或 Fyrox integration 移到 `omoba-core`。
- 不改變 gameplay rules、lockstep cadence、TD wave 行為、tower upgrade 規則或 hero ability 行為。
- 不新增 `omoba-runtime` 或其他中介 crate。
- 不為移除 internal adapter 保留相容 shim，除非發現有 persisted data、外部 API 或 launcher contract 需要。

## Decisions

1. `omoba-core::runtime` 作為唯一 deterministic gameplay runtime boundary。

   理由：`omb` 與 `omfx` 已共同依賴 `omoba-core`，直接統一在這個 crate 可降低漂移風險，也避免新增 crate 造成更複雜的 feature/toolchain 管理。

   替代方案：新增 `omoba-runtime` crate。否決原因是現有 specs 明確要求不新增，且會增加 dependency graph 與 script ABI/toolchain 協調成本。

2. 先盤點再移除 adapter，只刪除職責重複或只做 identity conversion 的層。

   理由：部分轉接層可能仍有合法職責，例如跨程序 protocol、render-facing snapshot projection、thread boundary ownership 或 UI mirror。盲目刪除會提高 regression 風險。

   替代方案：一次性大規模 flatten module hierarchy。否決原因是風險過高，也不利於定位行為差異。

3. 型別轉換優先改成共用來源型別，而不是保留 encode/decode roundtrip。

   理由：同 process 內的 `omfx` sim runner 與 `omoba-core::runtime` 不應透過 prost serialize/deserialize 轉換兩份等價 Rust type；這會增加 CPU 成本並掩蓋真實 ownership/API 邊界。

   替代方案：保留 roundtrip 作為相容層。否決原因是這正是本 change 要移除的歷史轉接成本。

4. 後端與前端驗證都要覆蓋。

   理由：這是跨 crate 架構收斂，單測一側無法保證 dependency boundary 與 gameplay 行為都沒破壞。

   替代方案：只跑 `cargo check`。否決原因是 check 無法涵蓋 deterministic tests、lockstep client event flow 或 runtime smoke links。

## Risks / Trade-offs

- [Risk] 某些 adapter 雖看似多餘，但實際承擔 thread ownership 或 render-facing cache 更新。→ Mitigation：移除前先確認 call sites 與資料生命週期，只刪除職責可由 `omoba-core` 直接承擔的層。
- [Risk] Cargo feature 不一致導致 `omb` 與 `omfx` 使用不同 runtime/protocol shape。→ Mitigation：檢查 manifests 與 feature gates，必要時把共用 gameplay API 放在 mandatory path。
- [Risk] 移除 conversion 後暴露 ownership/lifetime 差異。→ Mitigation：優先調整 function signatures 使用 shared owned structs 或 references，不新增複雜 generic abstraction。
- [Risk] 行為沒有改但 trace/perf baseline 改變。→ Mitigation：保留既有 diagnostics，驗證 smoke/stress 可啟動，並用 targeted tests 確認 lockstep/snapshot paths。

## Migration Plan

1. 盤點 `omb`、`omfx/game` 與 `omoba-core` 的 gameplay imports、adapter modules 與 conversion helpers。
2. 標記每個 adapter 的職責：必要邊界、可直接替代、或待確認。
3. 對可直接替代的 call sites 改用 `omoba-core` public API/type。
4. 移除 dead adapter code 與 manifest dependency edge。
5. 執行搜尋驗證與 cargo check/test。

Rollback 策略：若某個 adapter 移除造成難以快速修復的行為 regression，先回復該 adapter 的刪除 patch，保留已完成的盤點與其他低風險收斂。

## Open Questions

- 是否存在尚未被 specs 覆蓋、但實際由 frontend 保留的 gameplay prediction/shim？實作盤點時確認。
- 是否有 adapter 是為了 wasm 或非 native target 保留？若有，需避免 native 收斂破壞其他 target。
