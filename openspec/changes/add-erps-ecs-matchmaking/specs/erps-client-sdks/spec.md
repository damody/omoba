## ADDED Requirements

### Requirement: Rust SDK 提供型別安全 async client
`erps-client` SHALL 提供 session、party、queue、ready check、match、reconnect 與 state reconciliation 的 async API及 event stream，並 SHALL 使用與 server 相同的 `erps-proto` contract。

#### Scenario: Rust client 完成配對流程
- **WHEN** Rust client 建立 party、enqueue、接收 proposal、accept 並等到 game instance ready
- **THEN** SDK 以型別安全結果交付 match roster、endpoint 與 connection token

### Requirement: C SDK 使用 poll 而非背景 callback
`erps-client-ffi` SHALL 在內部執行 Rust async runtime 與網路背景執行緒，背景執行緒 MUST NOT 呼叫使用者 callback。C consumer SHALL 透過 `erps_client_poll()` 從 bounded local event queue 取得事件。

#### Scenario: 遊戲主迴圈 poll ready proposal
- **WHEN** 背景 gRPC stream 收到 ready-check proposal
- **THEN** C SDK 將事件放入 bounded queue，並只在主迴圈呼叫 `erps_client_poll()` 時交付

### Requirement: C ABI 明確管理記憶體與 panic
C event SHALL 使用 library-owned opaque handle，並 SHALL 提供 accessor 與唯一 release API。呼叫端 MUST NOT 以自己的 allocator 釋放 library memory。所有 exported functions MUST 捕捉 Rust panic 並轉成明確 error code。

#### Scenario: Event 由唯一 release API 釋放
- **WHEN** C consumer 讀完含動態字串與 roster 的 match event
- **THEN** 呼叫指定 release function 完整釋放資源，且不要求 C consumer 知道 Rust layout

### Requirement: C SDK 執行緒契約可驗證
C SDK SHALL 允許命令 API 從多執行緒呼叫，但每個 client handle 的 `poll` MUST 只有一個 consumer。違反契約 SHALL 回報明確錯誤或由 debug guard 偵測，MUST NOT 造成 data race。

#### Scenario: 並行送出 cancel 與狀態查詢
- **WHEN** 兩個 C threads 對同一 handle 呼叫 thread-safe command API
- **THEN** SDK 安全序列化命令並保持 `request_id` 語意

### Requirement: C SDK 發行 Windows 與 Linux 動態產物
第一版 SHALL 產生 Windows x64 DLL、import library、header，以及 Linux x86_64 shared object、header與範例。macOS 與 static library MUST NOT 是第一版驗收依賴。

#### Scenario: 真實 C compiler 連結 SDK
- **WHEN** Windows x64 或 Linux x86_64 smoke program 使用公開 header 編譯並連結發行產物
- **THEN** 程式可建立 client、enqueue、poll、accept、release event 與 shutdown

