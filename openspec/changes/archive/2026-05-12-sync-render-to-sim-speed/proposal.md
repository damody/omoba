## Why

目前 omfx renderer 會盡可能快地 render，即使 sim lockstep 只以固定 cadence 前進，造成 GPU/CPU 浪費、frame profile 失真，並讓動畫與 HUD 倒數在 snapshot 之間看起來比 sim 更新頻率更快。這個 change 要讓 render frame pacing 跟 sim 速度同步，避免「畫太快」而不改變 authoritative simulation cadence。

## What Changes

- omfx native frontend SHALL 使用 shared lockstep cadence 推導預設 render pacing target，讓 render update 不超過 sim-facing cadence。
- render loop SHALL 在 sim snapshot 沒有新 tick 且已達 pacing budget 時避免持續 busy-render 同一個 snapshot。
- render pacing SHALL 保留輸入與網路事件處理的反應性，不能因等待 frame cap 而阻塞 lockstep event drain、input submit 或 sim_runner worker。
- frame diagnostics SHALL 顯示實際 render fps/cap 狀態，方便確認 renderer 已被 sim cadence 限速。
- 不變更 backend authoritative tick rate、lockstep wire protocol 或 gameplay simulation dt。

## Capabilities

### New Capabilities
- `render-sim-cadence`: 定義 omfx render frame pacing 如何與 simulation cadence 對齊，包含 snapshot 重用、frame cap、輸入反應性與 diagnostics。

### Modified Capabilities

## Impact

- 主要影響 `omfx/game/src/native.rs` 的 `Plugin::update` frame pacing、snapshot consumption 與 frame profiling。
- 可能需要使用 Fyrox engine/window frame cap 或在 frontend update path 增加輕量 sleep/yield，但不得阻塞 sim_runner thread。
- 會依賴 `omoba_core::lockstep_timing` 的 shared cadence constants/helpers。
- 不新增 transport protocol 欄位，不修改 `omb` authoritative simulation loop。
