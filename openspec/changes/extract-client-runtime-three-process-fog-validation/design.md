## Context

目前 secure selective lockstep 的 client replica 與 Fyrox renderer 同處於 `omfx` process。這使 Specs world、script DLL、KCP session、hash/recovery 與畫面生命週期互相綁定，也無法直接讓未來 Unreal frontend 重用 Rust simulation。現有 server observer 與 omfx 還各自維護 component allowlist，已出現 schema 數量不一致；只驗證畫面變暗也不足以證明視野外資料沒有進入玩家 process。

本 change 以既有 [三個獨立 Rust Simulation Process 與戰爭迷霧端到端驗證設計](../../../docs/superpowers/specs/2026-08-28-three-process-client-runtime-fog-validation-design.md) 為完整技術基線。Rust toolchain 固定為 1.95.0；`scripts/base_content.dll` 與 host 必須使用相同 rustc；根目錄仍只保留既有四個 `.bat`，修改後維持 CRLF。

## Goals / Non-Goals

**Goals:**

- 建立一個 authoritative `omb` 加 Team 1、Team 2 各一個獨立 `omoba-client-runtime` 的三 simulation process 架構。
- 每個 client runtime 各自持有 filtered Specs world、scripts、以 `global_seed + tick` 建立的 RNG、hash 與 server-authoritative recovery。
- 讓 omfx 成為 renderer-only/input-only，並以相同 IPC 契約支援未來 Unreal。
- 從 packet、world、process memory、presentation、renderer memory 與 log 證明另一隊 hidden state 未洩漏。
- 保存server expected、server-local observer、external runtime三方pre-repair結果以診斷差異，blocking parity使用server-authoritative repair後的post-repair hash。
- 以 `FOG_2TEAM_DEMO` 重現 100 個普通單位、另外兩名英雄、移動、圓形視野、10×10 fog、樹木與 polygon occlusion。

**Non-Goals:**

- 不在本 change 實作 Unreal frontend。
- 不支援第三隊、任意 team 數量或通用觀戰者。
- 不把 `global_seed` 視為機密，也不讓 client 在 server 無新資料時預測未揭露世界。
- 不改變 server authority；client 與 observer 的結果永遠不能覆寫 server。
- 不建立公開網路用的 renderer protocol；IPC 限定 loopback。

## Decisions

### 1. 固定三個 simulation process，視覺模式再加兩個 renderer

`omb` 保存完整世界並各用一條 observer thread 驗算 Team 1、Team 2；兩個 `omoba-client-runtime` 是不同 PID、不同 world、不同 mutable state。Headless gate 使用三個 process，視覺 gate 使用五個 process。

選擇此方案是因為它同時驗證網路邊界、OS process 隔離與 Unreal 可替換性。否決把 replica 留在 omfx，因為那仍綁定 Fyrox；否決單一 client process 放兩隊 world，因為記憶體掃描無法證明玩家隔離。

### 2. `omoba-client-runtime` 是唯一 client Specs host

新增獨立 workspace crate/binary，擁有 secure V2 KCP session、frame barrier、`SelectiveReplicaRuntime`、`SpecsDisclosedWorldStepper`、ScriptRegistry、replica ID map、remembered cache、hash/recovery 與 IPC。三個 process 不共享 world、queue、RNG 或 script callback state。

替代方案是把 Rust library 嵌入 Unreal；不採用，因為會增加 C ABI、Unreal thread model 與 crash domain 的耦合。

### 3. 共用 allowlist 是第一個阻斷閘門

`omoba-core` 提供唯一 production component/resource allowlist API。server projector、server observer、external runtime 必須呼叫它，source guard 禁止 consumer 手寫 schema set。Filtered world 只能由 filtered snapshot或 Reveal 建立 entity，不執行完整 story spawn。

這避免不同 consumer 對同一 frame 解讀不同，也避免 client 因額外 component 取得 hidden dependency。

### 4. Server-authoritative selective tick pipeline

每 tick 依序執行 frame barrier、PreStep disclosure transition、accepted input/effect 注入、以 `global_seed + tick` 建立 tick-local RNG、共用 deterministic phases、pre-repair hash、checkpoint 比較、記錄 divergence、最後才套 repair/replace/rebase。兩隊同時處理，completion order 不得影響 authority。

不採用長期 client prediction；server 沒送新資訊時 client 無法準確推算未揭露戰場。hash 衝突時一律以 server correction 為準，但原始 mismatch 不得被修復後的 hash 掩蓋。

### 5. Renderer IPC 使用 loopback TCP 與 length-prefixed protobuf

Runtime 對 renderer輸出版本化 presentation snapshot、removed IDs、remembered ghosts、fog tiles、visibility-safe occluders、effects 與 session狀態；renderer輸入只包含玩家意圖。Snapshot queue bounded latest-wins，critical input result 使用獨立 ordered queue。simulation 120 Hz；presentation 預設 60 Hz且可設30/60/120 Hz。

選擇 protobuf 是為了讓 Rust omfx 與未來 Unreal C++ 共用 schema；不採 Rust ABI 或 shared memory，以降低 ABI 與同步風險。

### 6. omfx secure mode 僅負責 render 與 input

Secure fog mode 不建立 Specs world、不載 script DLL、不連 authoritative KCP、不自行判定 visibility/target legality。右鍵與技能 UI 經 IPC 交給 runtime；runtime先做結構與已揭露範圍檢查，server再做最終驗證。Renderer重啟只取得 runtime 最新 presentation，不觸發 server rebootstrap。

### 7. Runtime random sentinel 與三方 hash 都是 blocking evidence

Server 每次 run 產生不同 128-bit Team 1/2 sentinel，注入 test-only hidden fixture；掃描 raw/decoded packet、filtered world、runtime memory、presentation、renderer memory與玩家可見 log。靜態 binary/asset 不得含 sentinel 明文。

每隊checkpoint以`(team_id, replica_tick, team_sequence, authority_revision)`配對。pre-repair hash保留原始分歧；由於filtered world沒有隱藏實體，不能要求它永遠準確預測完整世界，因此blocking gate要求server expected、server observer與external runtime三方post-repair hash相同。coverage gap、缺report、scan失敗或worker crash只能是`UNVERIFIED/FAIL`，不得算PASS。

### 8. Launcher只管理已驗證PID

只修改 `run_2player.bat` 與既有 helper。它配置 server及兩個 presentation port、依序等待 ready marker、啟動三/五 process、執行情境、收集證據、優雅關閉，最後只對本次記錄且 executable path 相符的 PID 做 fallback termination。禁止 image-wide `taskkill`。

## Risks / Trade-offs

- [IPC 增加延遲] → input 不等待 presentation；renderer只對畫面插值，simulation保持120 Hz。
- [protobuf snapshot 過大] → 使用filtered delta、removed IDs、bounded cadence與量測後固定的頻寬門檻。
- [migration期間兩套client路徑] → secure launcher只允許 external runtime；source guard禁止同session雙step。
- [process memory scan誤判] → sentinel每次隨機，記錄PID、binary hash、dump方法與排除依據。
- [兩個runtime與兩條observer增加CPU] → 10,000 entity與30分鐘soak是blocking gate，不以停用驗算通過。
- [renderer斷線遺失狀態] → runtime保留latest full presentation與remembered cache，並繼續bounded grace。
- [server/runtime schema漂移] → 共用allowlist、protocol version、unknown schema fail-closed與contract guard。

## Migration Plan

1. 先建立共用 allowlist API與source guard，修正所有production consumer。
2. 建立 `omoba-client-runtime` crate、設定與生命週期骨架。
3. 從omfx抽出session、replica owner、stepper、scripts、hash/recovery。
4. 接通兩個runtime到authoritative KCP，再實作protobuf presentation/input IPC。
5. 將omfx secure mode切成renderer-only並保留非secure migration path。
6. 接通demo、三方hash、sentinel與evidence pipeline。
7. 修改launcher建立三process安全模式與五process視覺模式。
8. production與測試資產完成後，最後一次集中執行所有unit、integration、security、fault、跨平台、performance與soak gate。

Rollback時讓secure launcher暫時回到既有embedded模式；protocol capability與binary保留版本隔離，不允許同一session混用兩種模式。資料格式 migration失敗時終止該session，不降級到global snapshot或legacy tick。

## Open Questions

無。Team數固定為2、每隊一條observer thread、RNG使用`global_seed + tick`、outbound queue滿載阻塞authoritative tick，以及server衝突時以server為準，均已核准。
