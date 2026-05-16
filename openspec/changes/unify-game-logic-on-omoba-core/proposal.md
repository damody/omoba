## Why

`omb` 與 native `omfx` 都需要執行相同的 deterministic gameplay simulation；若任一側保留獨立遊戲邏輯或重複轉接層，會造成行為漂移、維護成本上升，也讓效能分析更難判斷真正瓶頸。

現在 `omoba-core::runtime` 已承擔共享 runtime 的角色，這個 change 要把邊界明確收斂：確認兩端是否都直接共用 `omoba-core`，若仍有重複或不必要 bridge，移除並改成直接使用 `omoba-core` 的 API 與型別。

## What Changes

- 盤點 `omb` 與 native `omfx` 的 gameplay runtime、state、input、snapshot 與 script dispatch 依賴路徑，確認是否仍存在 duplicated gameplay logic 或 backend-specific adapter。
- 將前後端共用的 deterministic gameplay logic 收斂到 `omoba-core::runtime`，並讓 `omb` 與 `omfx` 直接呼叫同一組 entrypoints。
- 移除不再必要的轉接層、prost roundtrip bridge、duplicate type conversion 或 frontend/backend 專用 shim；保留仍有明確跨程序、render-only 或 wire protocol 職責的邊界。
- **BREAKING**：若有 internal-only adapter module 或 wrapper API 只為歷史架構存在，將移除或改名；不承諾保留內部 crate API 相容性。
- 保持玩家可見行為不變，包含 lockstep cadence、TD tower 行為、hero ability、snapshot rendering 與 smoke/stress launcher flows。

## Capabilities

### New Capabilities

- 無。

### Modified Capabilities

- `frontend-backend-decoupling`: 強化 requirement，要求 `omb` 與 native `omfx` 的 deterministic gameplay logic 直接共用 `omoba-core::runtime`，且移除無明確職責的轉接層與 duplicated gameplay logic。

## Impact

- 影響 code path：`omb/src/**`、`omfx/game/src/**`、`omoba-core/src/runtime/**`、`omoba-core/src/comp/**`、shared protocol/input/snapshot modules。
- 影響 dependency boundary：維持 `omb -> omoba-core` 與 `omfx -> omoba-core`，避免新增 `omfx -> omb` 或新的 gameplay runtime crate。
- 影響驗證：需要 cargo check/test 前後端相關 crates，並以搜尋確認 duplicate bridge、backend imports、prost roundtrip conversion 或 obsolete adapter 已移除。
