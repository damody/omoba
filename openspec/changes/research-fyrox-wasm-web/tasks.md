## 1. Baseline Audit

- [x] 1.1 確認本機具備 `wasm32-unknown-unknown` target 與 `wasm-pack`，並記錄缺少時的安裝指令
- [x] 1.2 在 `omfx/executor-wasm` 執行 `wasm-pack build --target web --release`，保存第一輪 build blocker 清單
- [x] 1.3 將 build blocker 分類為 dependency、native API、thread/runtime、asset path、Fyrox/web renderer 或 transport 問題
- [x] 1.4 決定第一版 browser transport 採用直接整合 `omb` WebSocket transport 或獨立 bridge process，並更新 `design.md` 的 Open Questions 結論

## 2. WASM-Safe Client Build

- [ ] 2.1 調整 `omfx/game/Cargo.toml`，將 `omobab`、`specs`、native `tokio` runtime、Windows-only dependency 與其他 native-only dependency 放到 target-specific 或 feature-gated dependency
- [ ] 2.2 將 `spawn_backend` 與 Windows Job Object code gate 到 `cfg(not(target_arch = "wasm32"))`，並提供 wasm path 的明確 disconnected / unsupported 狀態
- [ ] 2.3 將 `sim_runner` 與所有直接依賴 `omobab` / script DLL 的 local simulation path gate 到 native target
- [ ] 2.4 將 `lockstep_client` 的 KCP/thread implementation 抽成 native backend，保留 `omfx::Game` 可消費的共用 event/input boundary
- [ ] 2.5 調整 `omoba-core` feature / target 設定，讓 wasm target 不拉入 `tokio_kcp`、UDP socket 或不支援的 tokio net feature
- [ ] 2.6 讓 `omfx/executor-wasm` 在沒有 backend transport 實作時仍能編譯並顯示可診斷的未連線狀態

## 3. Browser Transport

- [ ] 3.1 在 backend 或 bridge 新增 browser-safe endpoint，第一版優先使用 WebSocket 並文件化 bind address / port
- [ ] 3.2 定義 WebSocket frame 格式，重用既有 protobuf / lockstep message payload，並標明與 KCP tag 的對應
- [ ] 3.3 在 `omoba-core` 或 `omfx/game` 新增 wasm WebSocket client，使用 browser API 連線並轉出與 native lockstep path 等價的事件
- [ ] 3.4 支援從 query string、JS 設定或文件化設定指定 backend endpoint，避免 hard-code `127.0.0.1:50061`
- [ ] 3.5 將 browser transport 的連線、斷線、decode error 與 latency 資訊接到既有 HUD / log path

## 4. Asset Staging And Web Launch

- [ ] 4.1 建立 web root staging 流程，複製 `pkg/`、`index.html`、`main.js`、scene 與必要 `omfx/data/` assets
- [ ] 4.2 若新增 `.bat` script，確認檔案為 CRLF 行尾並可從 repo 根目錄執行
- [ ] 4.3 更新 `omfx/executor-wasm/README.md`，說明 build、stage、serve、backend 啟動與 browser URL
- [ ] 4.4 在 WASM client 補上缺少 scene / texture 時的明確錯誤訊息或畫面提示
- [ ] 4.5 確認 browser 啟動流程需要使用者互動來解鎖 audio context，並保留現有 Start button 行為

## 5. Verification

- [ ] 5.1 執行 `wasm-pack build --target web --release` 並確認 `executor-wasm/pkg/` 成功產生
- [ ] 5.2 用本機 HTTP server serve web root，確認瀏覽器能載入頁面並初始化 Fyrox executor
- [ ] 5.3 啟動原生 `omb` backend 與 browser transport，確認 browser client 可連線、加入遊戲並接收第一批 state / tick event
- [ ] 5.4 驗證 browser 端輸入可送到 backend，並由後續 state / tick event 反映在 renderer
- [ ] 5.5 執行 native desktop build 或既有 `run.bat` smoke test，確認自動 spawn backend、KCP lockstep 與 local sim runner 行為未退化
- [ ] 5.6 執行相關 Rust tests，例如 `cargo test --manifest-path omb/Cargo.toml -p omobab` 與必要的 `omoba-core` / `omfx` test
