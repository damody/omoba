## Why

目前 secure selective client simulation與Fyrox renderer仍共同存在於`omfx` process，無法在未來換成Unreal時保留同一份Rust Specs、scripts、RNG與recovery runtime，也缺少三個獨立simulation process的端到端證據，無法證明玩家process與畫面都不含視野外資訊。

## What Changes

- 新增不依賴Fyrox或Unreal的`omoba-client-runtime` crate與executable，分別承載Team 1及Team 2 filtered Specs world。
- 固定三個獨立Rust simulation backend process：一個authoritative `omb`、Team 1 replica runtime、Team 2 replica runtime。
- 把omfx的secure selective session、frame barrier、Specs stepper ownership、hash與recovery移入獨立client runtime。
- **BREAKING**：secure fog模式的omfx改為renderer-only/input-only，不再直連authoritative KCP server，也不再建立client Specs world或載入script DLL。
- 新增localhost TCP、length-prefixed protobuf presentation/input IPC，作為omfx與未來Unreal共用邊界。
- 將server projector、server observer與external client runtime的component/resource allowlist統一成`omoba-core`單一production API。
- 建立三process headless安全驗收與五process視覺驗收，覆蓋100個普通單位、兩名額外英雄、圓形視野、10×10 fog grid、Reveal／Hide／Forget／LastKnown、樹木與不規則地形遮擋及真實MoveTo。
- 以每run隨機128-bit sentinel掃描team packet、filtered world、client runtime memory、presentation payload、renderer memory及玩家可見log。
- 保存三方pre-repair hash作為divergence證據，並以server correction後的post-repair hash驗證最終收斂；完整世界與filtered world衝突時一律以server為準。
- 修改既有`run_2player.bat`支援三process安全模式與五process畫面模式，維持CRLF且只清理本次驗證過的PID。
- 完成功能後集中執行unit、integration、security、fault、Windows/Linux parity、10,000 entity與30分鐘soak gate。

## Capabilities

### New Capabilities

- `external-client-replica-runtime`: 規範獨立Rust client process的filtered Specs world、secure V2 session、scripts、RNG、hash、recovery及lifecycle。
- `renderer-presentation-ipc`: 規範client runtime與omfx／未來Unreal之間的localhost protobuf presentation/input IPC、cadence與backpressure。
- `three-process-fog-validation`: 規範三個simulation backend、五process視覺展示、runtime sentinel、三方hash、evidence與blocking verdict。

### Modified Capabilities

- `frontend-backend-decoupling`: Secure fog frontend改由獨立client runtime提供presentation與input bridge，不得持有Specs world或直連authoritative server。
- `sim-snapshot-rendering`: Render snapshot改由external client runtime輸出，並明確限制只能包含filtered render state與remembered presentation。
- `player-input-routing`: Renderer input先送external client runtime，再由runtime與authoritative server分層驗證及路由。
- `lockstep-event-flow`: Secure team frames新增external client runtime consumer，且server observer與external runtime必須消費對應team的相同encoded bytes語意。

## Impact

- 新增`omoba-client-runtime/` workspace crate及binary。
- 修改`omoba-core` selective runtime、共用allowlist、presentation/input protobuf與evidence schema。
- 修改`omb` session binding、三方hash report、test-only sentinel及launcher readiness輸出。
- 修改`omfx` network ownership、sim runner、render bridge、input routing與renderer-only lifecycle。
- 修改`proto/game.proto`及generated Rust schema；未來Unreal可由同一proto生成C++型別。
- 修改`run_2player.bat`及既有`scripts/*.ps1` helper，不新增根目錄`.bat`或`.sh`。
- 新增packet/world/process-memory/presentation scan工具、同步截圖與三／五process evidence目錄。
