## Why

目前 `omfx/executor-wasm` 已有 Fyrox WASM executor 雛形，但 `omfx/game` 仍假設 native desktop 環境：會自動 spawn `omobab.exe`、使用 KCP/UDP client、啟動背景 thread，並在本機 sim runner 中連結 `omobab` 與 native script DLL。瀏覽器不能直接啟動子行程、開 UDP/KCP socket 或載入 Windows DLL，因此需要先建立一個可落地的 WASM/Web 遷移方案，避免直接嘗試 build 時被平台限制與相依性阻斷。

## What Changes

- 建立 Fyrox Web/WASM client 的目標架構與 MVP 範圍：瀏覽器負責渲染與輸入，backend 仍由原生 `omb` server 執行。
- 定義 `executor-wasm` 的建置、資產佈署與本機 HTTP serve 流程，讓 `wasm-pack build --target web` 成為可重複驗證的 dev path。
- 規劃將 `omfx/game` 中 native-only 行為以 `cfg(not(target_arch = "wasm32"))` 或 feature gate 隔離，包括 backend spawn、local sim runner、native thread/runtime 與 DLL 相關路徑。
- 規劃 browser-safe transport，優先研究 WebSocket bridge 或 WebTransport 作為 KCP/UDP 的替代入口，並保留既有 lockstep payload / protobuf schema。
- 補上 WASM build smoke test、執行文件與限制說明，讓後續實作能逐步驗證 build、載入資產、連線、輸入與渲染。

## Capabilities

### New Capabilities
- `fyrox-web-wasm-client`: 定義 Fyrox client 在瀏覽器中建置、載入、連線 backend、處理輸入並渲染遊戲狀態的行為需求。

### Modified Capabilities
- 無。

## Impact

- `omfx/executor-wasm`: 補齊建置、啟動與資產載入流程。
- `omfx/game`: 隔離 native-only 程式碼，新增或抽象 WASM client path。
- `omoba-core`: 可能新增 browser transport client，重用 protobuf / lockstep 型別，避免在 wasm target 拉入 `tokio_kcp`。
- `omb/src/transport`: 可能新增 WebSocket 或 WebTransport bridge，將 browser 連線轉成既有 inbound/outbound / lockstep flow。
- `scripts/base_content` 與 script DLL: 保持 server-side；WASM client 不直接載入 native DLL。
- 文件與 dev script: 新增 WASM build/run 指引與 smoke test 流程。
