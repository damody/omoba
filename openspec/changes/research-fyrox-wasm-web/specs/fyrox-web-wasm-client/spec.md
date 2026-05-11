## ADDED Requirements

### Requirement: WASM executor build workflow
系統 SHALL 提供可重複執行的 `executor-wasm` 建置流程，將 Fyrox client 編譯為 `wasm32-unknown-unknown` / `wasm-pack --target web` 可載入的 package。

#### Scenario: Build command succeeds
- **WHEN** 開發者依文件在 `omfx/executor-wasm` 執行 WASM build 指令
- **THEN** 系統 MUST 產生 browser 可 import 的 `pkg/` 輸出

#### Scenario: Native-only code is excluded from WASM
- **WHEN** WASM build 編譯 `omfx/game`
- **THEN** backend spawn、Windows Job Object、native script DLL 載入、`tokio_kcp` UDP client 與 `std::thread` sim runner path MUST 不被編入 wasm target

### Requirement: Web asset staging and launch
系統 SHALL 提供明確的 web root staging 流程，讓瀏覽器能透過 HTTP 載入 `index.html`、JS glue、WASM package、Fyrox scene 與必要 `omfx/data/` assets。

#### Scenario: Static server loads the page
- **WHEN** 開發者完成 staging 並用本機 HTTP server serve web root
- **THEN** 瀏覽器 MUST 能載入 start page 並在使用者互動後初始化 Fyrox executor

#### Scenario: Missing asset is diagnosable
- **WHEN** 必要 scene 或 texture asset 不存在於 web root
- **THEN** client MUST 顯示或記錄可定位缺失路徑的錯誤，而不是靜默停在空白畫面

### Requirement: Browser-compatible backend transport
系統 SHALL 提供 browser 可使用的 backend transport，讓 WASM client 能連線到原生 `omb` server 或 bridge，並重用既有 protobuf / lockstep message 意義。

#### Scenario: Browser connects without UDP
- **WHEN** WASM client 在標準瀏覽器中啟動
- **THEN** client MUST 不嘗試直接開 UDP/KCP socket，並 MUST 使用 WebSocket、WebTransport 或其他 browser-supported transport

#### Scenario: Endpoint is configurable
- **WHEN** 開發者以 query string、JS 設定或文件化設定指定 backend endpoint
- **THEN** WASM client MUST 使用該 endpoint 建立連線，而不是假設 `127.0.0.1:50061` 的 KCP 位址

#### Scenario: Lockstep payload semantics are preserved
- **WHEN** backend 或 bridge 傳送 lockstep frame 給 browser client
- **THEN** client MUST 將資料解碼成與 native path 等價的 tick、input、state hash 或 game event 資訊

### Requirement: Server-side gameplay content remains authoritative
系統 SHALL 保持 gameplay simulation、story data 與 Rust script content 在原生 backend 端執行；WASM client 不直接載入 native script DLL。

#### Scenario: Browser joins a game
- **WHEN** browser client 連線並加入遊戲
- **THEN** gameplay state MUST 由 `omb` backend 產生，並透過 transport 傳給 browser renderer

#### Scenario: Script DLL is unavailable in browser
- **WHEN** browser client 執行時沒有 `base_content.dll`
- **THEN** client MUST 仍可作為 renderer/輸入端啟動，且不得嘗試在瀏覽器中載入該 DLL

### Requirement: Native desktop behavior is preserved
系統 SHALL 將 Web/WASM path 與 native desktop path 隔離，避免影響既有 Windows 開發與壓測流程。

#### Scenario: Native executor keeps existing behavior
- **WHEN** 開發者執行既有 native `run.bat` 或 desktop executor
- **THEN** native path MUST 保留目前自動 spawn backend、KCP lockstep 與 local sim runner 行為

#### Scenario: WASM changes do not require desktop migration
- **WHEN** Web transport 或 asset staging 尚未啟用
- **THEN** native desktop build MUST 仍可使用既有 features 與 scripts 建置執行
