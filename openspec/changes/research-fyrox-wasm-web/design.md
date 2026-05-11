## Context

`omfx` 是 Fyrox 1.0.1 前端 workspace，已包含 `executor-wasm` crate。該 crate 目前只有基本的 `wasm_bindgen` 入口，會建立 `Executor` 並加入 `omfx::Game` plugin；README 也記錄了 `wasm-pack build --target web --release` 與 `basic-http-server` 的基本流程。

目前阻礙不是 executor 雛形，而是 `omfx/game` 仍耦合多個 native 假設：native executor 會 spawn `omb/target/debug/omobab.exe`，lockstep client 依賴 `tokio_kcp` / UDP，`lockstep_client` 與 `sim_runner` 會建立 `std::thread`，`sim_runner` 連結 `omobab` 並期望能由 server-side script DLL 初始化 gameplay world。這些能力都不能直接在一般 browser WASM 環境中使用。

因此本設計把 Web 版定位為 browser client：WASM 只負責 Fyrox 渲染、UI 與玩家輸入，authoritative backend、story/script DLL、lockstep dispatcher 仍留在原生 `omb` server。瀏覽器透過 browser-safe transport 連線到 backend 或 bridge。

## Goals / Non-Goals

**Goals:**

- 讓 `omfx/executor-wasm` 可以在 `wasm32-unknown-unknown` target 下完成 release build。
- 讓瀏覽器能載入 Fyrox canvas、必要 asset 與 `omfx::Game` plugin，並以明確錯誤顯示未連線或不支援功能。
- 保留既有 protobuf / lockstep payload schema，避免為 Web 版重寫 gameplay protocol。
- 將 native-only 程式碼與相依性隔離，確保 native `run.bat`、`run_stress.bat` 與 desktop executor 行為不退化。
- 建立可驗證的 dev workflow：build WASM、stage assets、serve static files、啟動 backend、用 browser 連線。

**Non-Goals:**

- 不把 `omb` authoritative server 編譯成 browser WASM。
- 不在瀏覽器載入 `scripts/base_content.dll` 或任何 native cdylib。
- 不要求第一版支援離線單機 deterministic simulation。
- 不重寫 Fyrox renderer 或更換 engine。
- 不在第一版解決正式部署、帳號登入、matchmaking 或 CDN cache policy。

## Decisions

### Decision: Web 版採用 browser client + 原生 backend

Web 版 SHALL 保持 backend server-side。瀏覽器只執行 `executor-wasm` 與 `omfx/game` 的 WASM-safe 子集。

替代方案是把 `omb`、`specs` ECS、script ABI 與 content 一起編進 WASM，但這會遇到 UDP socket、native thread、filesystem、dynamic library、abi_stable cdylib 與 build time codegen 等多重限制，也會改變 authoritative server 的信任模型。此方案不適合作為第一階段。

### Decision: 用 `cfg` / feature gate 隔離 native-only path

`spawn_backend`、Windows Job Object、`sim_runner`、`lockstep_client` 的 native thread runtime、`tokio_kcp` 與 `omobab` as-lib dependency 應在 wasm target 排除。`omfx/game` 需要提供 platform boundary，例如 native path 使用既有 KCP lockstep，wasm path 使用 browser transport 並回傳同形狀的 game / lockstep events。

替代方案是複製一個 Web 專用 game crate，但會讓 rendering、UI 與 input mapping 快速分叉。保留單一 `omfx::Game`，只抽出平台 I/O 邊界，比較能維持 native 與 web 行為一致。

### Decision: 第一版優先獨立 WebSocket bridge process

Browser 不能直接開 UDP/KCP socket。第一版優先使用獨立 WebSocket bridge process 作為 browser transport，bridge 對 browser 暴露 WebSocket endpoint，對 `omb` 則連到既有 KCP endpoint，並轉送相同 `[tag][len][payload]` framed protobuf bytes。這避免修改 `omb` 目前單一 `TransportHandle` outbound consumer / KCP session fan-out 架構，也讓 native backend rollback 最小化。

直接把 WebSocket transport 整合到 `omb/src/transport` 仍是後續選項，但需要先重構 outbound broadcast fan-out，否則 KCP 與 WebSocket 會競爭同一個 `out_rx`。WebTransport 可作為後續最佳化選項，特別是若需要 unreliable datagram 或更接近 KCP 的低延遲行為。gRPC-web 也可行，但目前 lockstep path 已圍繞自訂 framed protobuf tag，WebSocket bridge 的遷移面較小。

### Decision: Asset staging 應顯式化

Web build 不應要求手動 clone 整包 repo 到 `executor-wasm`。應提供一個可重複的 staging 流程，把 `pkg/`、`index.html`、`main.js`、Fyrox scene 與 `omfx/data/` 必要資產放到同一個 web root。這個流程可以先是 `.bat` 或文件化指令，後續再整合進 dev script。

### Decision: WASM smoke test 分階段驗證

驗證順序應先確認 compile，再確認 static page load，再確認 asset load，最後才驗證 backend 連線與遊戲互動。這能快速分辨問題是在 Rust target、JS glue、資產路徑、Fyrox renderer，還是 transport。

## Risks / Trade-offs

- Browser transport 與 KCP 的延遲特性不同 -> 第一版保留相同 lockstep payload 並把 latency HUD / input latency metric 接到 WebSocket path，後續用實測決定是否升級 WebTransport。
- `omfx/game` native-only 相依性太深 -> 先以 build error audit 建立 wasm blocker 清單，再用最小 platform boundary 拆分，不一次大改 rendering code。
- Fyrox asset path 在 Web 上與 native 不一致 -> 建立 staging root 與固定相對路徑，並加上缺資產時的明確錯誤訊息。
- WASM bundle 過大或初次載入太慢 -> 第一版接受 debugability 優先，後續再做 release size audit、asset subset 與壓縮。
- Backend WebSocket bridge 增加 server attack surface -> 初期只允許本機或明確設定的 bind address，並在文件中標明 production 前需補上 origin / auth / rate limit。
- Browser thread 支援受限 -> 第一版避免依賴 WASM threads；若 Fyrox 或 sim path 需要 worker，再另開設計處理 `SharedArrayBuffer`、COOP/COEP header 與 worker packaging。

## Migration Plan

1. 先執行 `wasm-pack build --target web --release`，記錄並分類所有 wasm build blocker。
2. 調整 `omfx/game` 與 `omoba-core` 的 target-specific dependency / module gate，使 `executor-wasm` 能完成編譯。
3. 建立 web staging 流程，讓 `executor-wasm/index.html` 能載入 `pkg/` 與必要 assets。
4. 新增獨立 WebSocket bridge process，重用既有 protobuf / lockstep message 的 framed bytes。
5. 在 `omfx/game` 新增 wasm transport path，連線到 WebSocket endpoint，並把 inbound event 轉成既有 render path 可消費的資料。
6. 補上 README / dev script / smoke test，包含 backend 啟動順序、browser URL、限制與常見錯誤。

Rollback 策略：所有 Web path 都以 wasm target 或明確 feature gate 隔離；若 Web 實作失敗，可停用 `executor-wasm` workflow，不應影響 native desktop executor 與既有 KCP path。

## Open Questions

- WebSocket endpoint 第一版採獨立 bridge process，避免先重構 `omb` transport fan-out。
- Web 第一版先以 framed lockstep / game event bytes 打通連線與診斷，完整 renderer state parity 依後續實測補齊。
- `omfx/game` 第一版先在 wasm path 內建立 WebSocket client，待協定穩定後再評估是否把 transport trait 抽到 `omoba-core`。
- Asset staging 第一版先做 repo 根目錄 `.bat`，後續再評估是否由 Rust `export-cli` / build tool 產生 web root。
