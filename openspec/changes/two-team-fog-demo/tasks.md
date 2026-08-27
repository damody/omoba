# 實作任務

每個完成的 L3 leaf 必須在 `openspec/changes/two-team-fog-demo/evidence/index.jsonl` 建立唯一 `task_id` record。Record 至少包含 `task_id`、`status`、`artifact_or_command`、`expected`、`actual`、`exit_status_or_reviewer`、`hashes`、`related_gates`、`adjustment_id`、`timestamp`、`subcheck`、`generation`、`replaces` 與 `record_id`。只有 `passed`、有證據的 `not-applicable` 或帶 replacement 的 `superseded` 可勾選。

## 1. 契約與現況固定

### 1.1 Demo content 與 runtime boundary 盤點

**目的：** 在修改前固定可重用的 content/import/runtime 入口。
**輸入：** 核准設計、既有 Lua story、omb scene initialization、selective runtime。
**產出：** `evidence/phase1-inventory.json`。
**依賴：** 無。
**Owner／Wave：** Primary integrator／Wave 1A。
**Gate／Evidence：** G-DEMO-CARDINALITY、G-DEMO-TEAM-ISOLATION／phase1 inventory。
**完成門檻：** 每個將修改的 boundary、型別、測試位置與禁止路徑都有明確記錄。

- [x] 1.1.1 列出 `FOG_2TEAM_DEMO` 需要的 Lua package 檔案清單。
- [x] 1.1.2 找出既有 story selection 的環境變數入口。
- [x] 1.1.3 找出 Lua map loader 的回傳型別。
- [x] 1.1.4 找出 campaign scene 建立英雄的函式入口。
- [x] 1.1.5 找出一般單位可重用的 template ID。
- [x] 1.1.6 找出 player ownership component 的既有型別。
- [x] 1.1.7 找出 `VisionSource` 與 `ReplicationScope` 的建立入口。
- [x] 1.1.8 找出 `RememberPolicy::LastKnown` 的建立入口。
- [x] 1.1.9 找出 deterministic movement/patrol 可重用的 tick system。
- [x] 1.1.10 找出 omfx filtered snapshot 的消費入口。
- [x] 1.1.11 找出 omfx remembered render cache 的消費入口。
- [x] 1.1.12 找出 server team observer 的啟動與診斷入口。
- [x] 1.1.13 記錄不得重用的 viewport/legacy visibility path。
- [x] 1.1.14 將 inventory 寫入 phase1 evidence。

### 1.2 Demo constants 與 evidence contract

**目的：** 將不應分散的場景常數與證據格式固定為單一來源。
**輸入：** 1.1 inventory、核准數量與視野決策。
**產出：** Demo constants contract、evidence index/schema。
**依賴：** 1.1。
**Owner／Wave：** Primary integrator／Wave 1B。
**Gate／Evidence：** G-DEMO-CARDINALITY、G-DEMO-DETERMINISM／`evidence/index.jsonl`。
**完成門檻：** 100、2、33／33／34、16、220、700 與 stable ordering 各有單一權威定義。

- [x] 1.2.1 定義 grid rows 常數為 10。
- [x] 1.2.2 定義 grid columns 常數為 10。
- [x] 1.2.3 定義 grid spacing 常數為 220。
- [x] 1.2.4 定義 grid unit count 常數為 100。
- [x] 1.2.5 定義 player hero count 常數為 2。
- [x] 1.2.6 定義 Team 1 grid count 常數為 33。
- [x] 1.2.7 定義 Team 2 grid count 常數為 33。
- [x] 1.2.8 定義 Neutral grid count 常數為 34。
- [x] 1.2.9 定義 patrol unit count 常數為 16。
- [x] 1.2.10 定義 vision radius 常數為 700。
- [x] 1.2.11 定義 row-major stable spawn key 格式。
- [x] 1.2.12 定義 deterministic team assignment 規則。
- [x] 1.2.13 定義 deterministic patrol index 清單。
- [x] 1.2.14 建立空的 append-only evidence index。
- [x] 1.2.15 記錄 A／B／C adjustment 與 stale replacement 規則。

## 2. Lua content 與 validated descriptor

### 2.1 建立 `FOG_2TEAM_DEMO` package

**目的：** 建立只使用既有 assets 的 opt-in demo story。
**輸入：** 1.2 constants、既有 Lua package 範例。
**產出：** `scripts/lua_data/FOG_2TEAM_DEMO/`。
**依賴：** 1.2。
**Owner／Wave：** Content implementer／Wave 2A。
**Gate／Evidence：** G-DEMO-CARDINALITY／content hash 與 loader evidence。
**完成門檻：** Package 可被 runtime Lua loader 找到，且不修改其他 story。

- [x] 2.1.1 建立 `FOG_2TEAM_DEMO` package 目錄。
- [x] 2.1.2 建立 package 的 `map.lua` 入口。
- [x] 2.1.3 宣告 demo map bounds。
- [x] 2.1.4 宣告 grid origin。
- [x] 2.1.5 宣告 10×10 grid dimensions。
- [x] 2.1.6 宣告 220-unit spacing。
- [x] 2.1.7 宣告 P1 hero spawn。
- [x] 2.1.8 宣告 P2 hero spawn。
- [x] 2.1.9 宣告 vision radius 700。
- [x] 2.1.10 宣告 `LastKnown` remember policy。
- [x] 2.1.11 宣告 16 個 patrol stable indexes。
- [x] 2.1.12 宣告 patrol endpoint offset。
- [x] 2.1.13 宣告 patrol speed。
- [x] 2.1.14 引用一個既有 hero render template。
- [x] 2.1.15 引用一個既有 grid unit render template。
- [x] 2.1.16 確認 package 不引用不存在的 asset path。

### 2.2 擴充 Lua/import descriptor

**目的：** 將 demo 宣告轉成明確、可驗證的 Rust descriptor。
**輸入：** 2.1 package、現有 Lua map loader。
**產出：** Demo descriptor types 與 parser。
**依賴：** 2.1。
**Owner／Wave：** Backend content integrator／Wave 2B。
**Gate／Evidence：** G-DEMO-CARDINALITY、G-DEMO-DETERMINISM／focused parser tests。
**完成門檻：** 合法 package 完整 parse；每個不合法欄位在 scene mutation 前失敗。

- [x] 2.2.1 新增 grid descriptor Rust type。
- [x] 2.2.2 新增 hero spawn descriptor Rust type。
- [x] 2.2.3 新增 patrol descriptor Rust type。
- [x] 2.2.4 新增 demo visibility descriptor Rust type。
- [x] 2.2.5 將 Lua grid dimensions parse 到 descriptor。
- [x] 2.2.6 將 Lua spacing 與 origin parse 到 descriptor。
- [x] 2.2.7 將兩個 hero spawn parse 到 descriptor。
- [x] 2.2.8 將 patrol indexes parse 到 descriptor。
- [x] 2.2.9 將 vision radius 與 remember policy parse 到 descriptor。
- [x] 2.2.10 驗證 rows×columns 等於 100。
- [x] 2.2.11 驗證 hero count 等於 2。
- [x] 2.2.12 驗證所有座標與速度是 finite。
- [x] 2.2.13 驗證 patrol index 唯一且位於 0..100。
- [x] 2.2.14 驗證 stable spawn key 不重複。
- [x] 2.2.15 驗證 team assignment 只產生 Team 1、Team 2、Neutral。
- [x] 2.2.16 讓 validation error 包含欄位與實際值。
- [x] 2.2.17 保證 validation 失敗前不建立 ECS entity。

## 3. Authoritative demo world

### 3.1 建立 100 個 grid units

**目的：** 以 deterministic row-major order 建立精確的 grid population。
**輸入：** 2.2 validated descriptor、existing unit templates。
**產出：** Demo grid scene builder。
**依賴：** 2.2。
**Owner／Wave：** Backend gameplay implementer／Wave 3A。
**Gate／Evidence：** G-DEMO-CARDINALITY、G-DEMO-DETERMINISM／scene tests。
**完成門檻：** 100 個單位、33／33／34 分布、位置與 stable keys 完全符合 descriptor。

- [x] 3.1.1 新增 demo scene selection branch。
- [x] 3.1.2 建立 row-major index helper。
- [x] 3.1.3 建立 index 到 world position helper。
- [x] 3.1.4 建立 index 到 team assignment helper。
- [x] 3.1.5 建立 index 到 stable spawn key helper。
- [x] 3.1.6 建立單一 grid unit entity builder。
- [x] 3.1.7 在 builder 加入 position component。
- [x] 3.1.8 在 builder 加入 unit/template component。
- [x] 3.1.9 在 builder 加入 faction/team component。
- [x] 3.1.10 在 builder 加入 stable identity input。
- [x] 3.1.11 在 builder 加入 `TeamVision` replication scope。
- [x] 3.1.12 在 builder 加入 `LastKnown` remember policy。
- [x] 3.1.13 以固定順序建立全部 100 個 grid units。
- [x] 3.1.14 在 scene build 後 assert grid count 為 100。
- [x] 3.1.15 在 scene build 後 assert team counts 為 33／33／34。

### 3.2 建立兩個玩家英雄與 ownership

**目的：** 建立不計入 grid count 的兩個可控制 vision-source heroes。
**輸入：** 3.1 scene builder、authenticated bindings 1→1 與 2→2。
**產出：** 兩個 hero entities 與 player/team ownership。
**依賴：** 3.1。
**Owner／Wave：** Backend gameplay implementer／Wave 3B。
**Gate／Evidence：** G-DEMO-CARDINALITY、G-DEMO-TEAM-ISOLATION／hero ownership tests。
**完成門檻：** 總 gameplay unit count 102，兩個英雄分別只能由正確玩家控制。

- [x] 3.2.1 建立 P1 hero stable spawn key。
- [x] 3.2.2 建立 P2 hero stable spawn key。
- [x] 3.2.3 以 descriptor 位置建立 P1 hero。
- [x] 3.2.4 以 descriptor 位置建立 P2 hero。
- [x] 3.2.5 將 P1 owner player ID 設為 1。
- [x] 3.2.6 將 P2 owner player ID 設為 2。
- [x] 3.2.7 將 P1 hero team 設為 1。
- [x] 3.2.8 將 P2 hero team 設為 2。
- [x] 3.2.9 為 P1 hero 建立半徑 700 `VisionSource`。
- [x] 3.2.10 為 P2 hero 建立半徑 700 `VisionSource`。
- [x] 3.2.11 讓兩個 hero 使用既有 player movement input path。
- [x] 3.2.12 阻止 P1 input 控制 P2 hero。
- [x] 3.2.13 阻止 P2 input 控制 P1 hero。
- [x] 3.2.14 在 scene build 後 assert hero count 為 2。
- [x] 3.2.15 在 scene build 後 assert gameplay unit total 為 102。

### 3.3 實作 16 個 deterministic patrol units

**目的：** 持續產生不依賴玩家操作的 visibility transitions。
**輸入：** 3.1 grid entities、2.2 patrol descriptor。
**產出：** Patrol state 與 authoritative tick update。
**依賴：** 3.1。
**Owner／Wave：** Backend gameplay implementer／Wave 3C。
**Gate／Evidence：** G-DEMO-DETERMINISM／patrol trace hashes。
**完成門檻：** 精確 16 個單位以 fixed-point、stable order 往返，重跑 trace 相同。

- [x] 3.3.1 定義 demo patrol component/state。
- [x] 3.3.2 將 patrol endpoints 量化為 fixed-point。
- [x] 3.3.3 將 patrol speed 量化為 fixed-point。
- [x] 3.3.4 只為固定 index 清單加入 patrol state。
- [x] 3.3.5 以 stable spawn key 排序 patrol update。
- [x] 3.3.6 實作朝目前 endpoint 的 fixed-step movement。
- [x] 3.3.7 實作到達 endpoint 的 deterministic clamp。
- [x] 3.3.8 實作 endpoint direction reversal。
- [x] 3.3.9 防止 zero-distance path 產生除零。
- [x] 3.3.10 將 patrol update 放入 authoritative Wave A。
- [x] 3.3.11 確保 patrol 不讀 wall clock。
- [x] 3.3.12 確保 patrol 不使用 runtime RNG。
- [x] 3.3.13 確保非 patrol units 保持靜止。
- [x] 3.3.14 產生固定 tick patrol position trace。

## 4. Team projection 與 observer 整合

### 4.1 接上 team visibility projection

**目的：** 讓 demo entities 使用既有 secure V2 disclosure pipeline。
**輸入：** 3.1–3.3 authoritative world、existing `TeamViewProjector`。
**產出：** Team 1／Team 2 filtered frames。
**依賴：** 3.1、3.2、3.3。
**Owner／Wave：** Selective projection integrator／Wave 4A。
**Gate／Evidence：** G-DEMO-TEAM-ISOLATION／projection scenario evidence。
**完成門檻：** 每隊只收到 700-unit shared view；Neutral 不會 global disclose。

- [x] 4.1.1 將 demo hero `VisionSource` 納入 team visibility input。
- [x] 4.1.2 將 grid unit scope 納入 projection classification。
- [x] 4.1.3 讓 Neutral grid units 使用 `TeamVision` 而非 `Public`。
- [x] 4.1.4 驗證 force-hide precedence 仍優先於 demo vision。
- [x] 4.1.5 使用既有 visibility commit delay。
- [x] 4.1.6 在 reveal effective tick 擷取 fresh baseline。
- [x] 4.1.7 在 hide effective tick 停止 gameplay delta。
- [x] 4.1.8 為 hide 產生去敏感化 `LastKnown` presentation。
- [x] 4.1.9 保持每隊獨立 `ReplicaEntityId` mapping。
- [x] 4.1.10 拒絕 stale disclosure epoch input。
- [x] 4.1.11 確保 viewport update 不觸發 visibility transition。
- [x] 4.1.12 確保 camera state 不進 authoritative hash。
- [x] 4.1.13 確保 hidden grid unit 不進 team snapshot。
- [x] 4.1.14 確保 hidden-only patrol movement 不改變 team frame bytes。

### 4.2 接上兩隊 observer replica

**目的：** 驗算兩隊實際送出 bytes，而不阻塞 outbound。
**輸入：** 4.1 encoded team streams、existing observer worker。
**產出：** Team 1／Team 2 observer diagnostics。
**依賴：** 4.1。
**Owner／Wave：** Observer integrator／Wave 4B。
**Gate／Evidence：** G-DEMO-OBSERVER／observer smoke report。
**完成門檻：** 兩個 observer 只消費自己的 stream，正常 demo run 無 mismatch 或未處理 gap。

- [x] 4.2.1 在 Team 1 active 時建立 Team 1 observer state。
- [x] 4.2.2 在 Team 2 active 時建立 Team 2 observer state。
- [x] 4.2.3 確認 encoded frame 先進 session send queue。
- [x] 4.2.4 將同一份 `Arc<[u8]>` 非阻塞 tap 給 observer。
- [x] 4.2.5 禁止 observer 讀 authoritative Specs world。
- [x] 4.2.6 禁止 Team 1 observer 讀 Team 2 bootstrap/frame。
- [x] 4.2.7 禁止 Team 2 observer 讀 Team 1 bootstrap/frame。
- [x] 4.2.8 記錄 per-team observer sequence 與 tick。
- [x] 4.2.9 記錄 per-team hash mismatch diagnostics。
- [x] 4.2.10 記錄 validation queue coverage gap。
- [x] 4.2.11 保留 overflow 後 filtered rebootstrap path。
- [x] 4.2.12 將 demo observer 狀態輸出到 server-only log。

## 5. omfx fog demo presentation

### 5.1 建立 demo session 與 HUD model

**目的：** 讓每個 frontend 清楚顯示自己的身份與 filtered counts。
**輸入：** 4.1 filtered snapshot、launcher environment contract。
**產出：** Demo presentation state 與 HUD labels。
**依賴：** 4.1。
**Owner／Wave：** Frontend implementer／Wave 5A。
**Gate／Evidence：** G-DEMO-PRESENTATION／frontend focused tests。
**完成門檻：** HUD 不讀 full-world count，且兩視窗 team/player 資訊正確。

- [x] 5.1.1 新增 fog demo session detection。
- [x] 5.1.2 從 secure start binding 取得 local team ID。
- [x] 5.1.3 將 Team 1 presentation color 固定為藍色。
- [x] 5.1.4 將 Team 2 presentation color 固定為紅色。
- [x] 5.1.5 建立固定文字 `Demo grid units: 100`。
- [x] 5.1.6 建立固定文字 `Player heroes: 2`。
- [x] 5.1.7 從 filtered live entities 計算 `Currently disclosed`。
- [x] 5.1.8 從 remembered cache 計算 `Remembered ghosts`。
- [x] 5.1.9 顯示 local player ID。
- [x] 5.1.10 顯示 local team ID。
- [x] 5.1.11 顯示 replica tick。
- [x] 5.1.12 顯示點擊移動與觀察提示。
- [x] 5.1.13 阻止 HUD 讀 authoritative total entity count。

### 5.2 繪製圓形視野與 fog overlay

**目的：** 以 renderer-only 圖層清楚表示 local team 的已知視野。
**輸入：** 5.1 local hero snapshot、既有 circle drawing helper。
**產出：** Vision circle 與 fog overlay render pass。
**依賴：** 5.1。
**Owner／Wave：** Frontend implementer／Wave 5B。
**Gate／Evidence：** G-DEMO-PRESENTATION／render model tests、人工畫面。
**完成門檻：** 圓與 fog 跟隨自有英雄；renderer state 不回寫 gameplay authority。

- [x] 5.2.1 從 filtered snapshot 找出 local owned hero。
- [x] 5.2.2 將 demo radius 700 轉成 render units。
- [x] 5.2.3 重用既有 batched circle drawing helper。
- [x] 5.2.4 使用 local team color 畫 circle outline。
- [x] 5.2.5 建立半透明 fog overlay material/state。
- [x] 5.2.6 以 local hero position 更新 fog opening center。
- [x] 5.2.7 以 radius 700 更新 fog opening size。
- [x] 5.2.8 將 fog layer 排在 terrain 之上、units 之下或採等價可讀順序。
- [x] 5.2.9 在 local hero 尚未 disclosed 時安全隱藏 demo overlay。
- [x] 5.2.10 避免每 entity 每 frame 建立新 material。
- [x] 5.2.11 避免 circle draw 產生 per-frame heap churn。
- [x] 5.2.12 確保 camera movement 不改變 server visibility input。

### 5.3 呈現並隔離 LastKnown ghosts

**目的：** 顯示已去敏感化的 remembered state且不污染 simulation。
**輸入：** Existing remembered cache、4.1 hide transition。
**產出：** Low-opacity ghost render records。
**依賴：** 4.1、5.1。
**Owner／Wave：** Frontend implementer／Wave 5C。
**Gate／Evidence：** G-DEMO-PRESENTATION、G-DEMO-TEAM-ISOLATION／cache isolation tests。
**完成門檻：** Ghost 可見但無法被 target、碰撞或計入 hash/count。

- [x] 5.3.1 將 hide presentation 寫入獨立 remembered cache。
- [x] 5.3.2 保存 last disclosed render position。
- [x] 5.3.3 保存 render-safe unit kind/color。
- [x] 5.3.4 不保存 hidden current position。
- [x] 5.3.5 不保存 hidden HP 或其他敏感 component。
- [x] 5.3.6 以低透明度繪製 ghost。
- [x] 5.3.7 Reveal 同 identity 時移除對應 ghost。
- [x] 5.3.8 Forget transition 時移除對應 ghost。
- [x] 5.3.9 從 gameplay target lookup 排除 remembered cache。
- [x] 5.3.10 從 collision query 排除 remembered cache。
- [x] 5.3.11 從 team hash 排除 remembered cache。
- [x] 5.3.12 從 `Currently disclosed` 排除 remembered cache。

## 6. Windows 雙 process launcher

### 6.1 修正 `run_2player.bat` 啟動拓撲

**目的：** 建立可重複的一 server、兩 frontend 開發入口。
**輸入：** Existing freshness helpers、omb/omfx binaries、demo story。
**產出：** CRLF `run_2player.bat` 與必要的非根目錄 helper。
**依賴：** 3.2、5.1。
**Owner／Wave：** Primary integrator／Wave 6A。
**Gate／Evidence：** G-DEMO-LAUNCHER／launcher static/smoke evidence。
**完成門檻：** 不依賴遺失 helper；三個 process identity、環境與 PID 都可追蹤。

- [x] 6.1.1 將 launcher story 設為 `FOG_2TEAM_DEMO`。
- [x] 6.1.2 保留 script DLL freshness check。
- [x] 6.1.3 保留 backend freshness check。
- [x] 6.1.4 保留 frontend freshness check。
- [x] 6.1.5 在啟動前驗證 demo Lua package 存在。
- [x] 6.1.6 移除對 `run_2player_client.bat` 的呼叫。
- [x] 6.1.7 建立 P1 獨立 process environment。
- [x] 6.1.8 建立 P2 獨立 process environment。
- [x] 6.1.9 設定 P1 player ID/name/lockstep name。
- [x] 6.1.10 設定 P2 player ID/name/lockstep name。
- [x] 6.1.11 設定 P1 team/window/log suffix。
- [x] 6.1.12 設定 P2 team/window/log suffix。
- [x] 6.1.13 保存 server PID。
- [x] 6.1.14 保存 P1 executor PID。
- [x] 6.1.15 保存 P2 executor PID。
- [x] 6.1.16 確認 P1 與 P2 executor PID 不同。
- [x] 6.1.17 正常結束只停止本次 server PID。
- [x] 6.1.18 Server 提前退出只停止本次兩個 executor PID。
- [x] 6.1.19 將缺 artifact 情況轉成非零 exit code。
- [x] 6.1.20 將 `run_2player.bat` 正規化為 CRLF。

### 6.2 視窗辨識與左右排列

**目的：** 讓人工驗收可直接並排比較兩個 team view。
**輸入：** 6.1 process identity、omfx executor window initialization。
**產出：** Per-process title 與 best-effort window position。
**依賴：** 6.1。
**Owner／Wave：** Frontend launcher integrator／Wave 6B。
**Gate／Evidence：** G-DEMO-LAUNCHER、G-DEMO-MANUAL／window evidence。
**完成門檻：** 標題永遠正確；單螢幕可左右排列，定位失敗不終止 match。

- [x] 6.2.1 定義 P1 window title suffix。
- [x] 6.2.2 定義 P2 window title suffix。
- [x] 6.2.3 在 executor title 顯示 P1／Team 1。
- [x] 6.2.4 在 executor title 顯示 P2／Team 2。
- [x] 6.2.5 取得 primary monitor work area。
- [x] 6.2.6 計算左側視窗 bounds。
- [x] 6.2.7 計算右側視窗 bounds。
- [x] 6.2.8 對 P1 套用左側 bounds。
- [x] 6.2.9 對 P2 套用右側 bounds。
- [x] 6.2.10 將定位失敗記為 warning。
- [x] 6.2.11 確保定位失敗不更改 secure connection state。

## 7. 集中式最終驗證與人工預覽

### 7.1 Content 與 backend 最終驗證

**目的：** 一次驗證 cardinality、determinism、ownership、patrol 與 visibility。
**輸入：** Phase 2–4 完成實作。
**產出：** `evidence/final/backend/summary.json`。
**依賴：** 2.2、3.1–3.3、4.1–4.2。
**Owner／Wave：** Primary verifier／Wave 7A。
**Gate／Evidence：** G-DEMO-CARDINALITY、G-DEMO-DETERMINISM、G-DEMO-TEAM-ISOLATION、G-DEMO-OBSERVER。
**完成門檻：** 所有 scenario 通過且 hidden disclosure finding 為 0。

- [x] 7.1.1 執行合法 demo Lua package load test。
- [x] 7.1.2 執行錯誤 grid count rejection test。
- [x] 7.1.3 執行 duplicate spawn key rejection test。
- [x] 7.1.4 執行 non-finite coordinate rejection test。
- [x] 7.1.5 驗證 grid unit count 為 100。
- [x] 7.1.6 驗證 hero count 為 2。
- [x] 7.1.7 驗證 gameplay unit total 為 102。
- [x] 7.1.8 驗證 team distribution 為 33／33／34。
- [x] 7.1.9 比對兩次 spawn manifest hash。
- [x] 7.1.10 比對兩次 patrol trace hash。
- [x] 7.1.11 執行 P1 ownership input case。
- [x] 7.1.12 執行 P2 ownership input case。
- [x] 7.1.13 執行 cross-owner rejection case。
- [x] 7.1.14 執行 radius-inside reveal case。
- [x] 7.1.15 執行 radius-outside hide case。
- [x] 7.1.16 執行 visibility boundary exact-distance case。
- [x] 7.1.17 執行 viewport non-authority case。
- [x] 7.1.18 執行 Neutral non-public case。
- [x] 7.1.19 執行 hidden-only patrol byte non-interference case。
- [x] 7.1.20 執行兩隊 observer isolation case。
- [x] 7.1.21 執行 observer outbound-nonblocking case。
- [x] 7.1.22 寫入 backend final summary 與 hashes。

### 7.2 Frontend 與 launcher 最終驗證

**目的：** 驗證 filtered presentation 與 Windows process topology。
**輸入：** Phase 5–6 完成實作。
**產出：** `evidence/final/frontend-launcher/summary.json`。
**依賴：** 5.1–5.3、6.1–6.2。
**Owner／Wave：** Primary verifier／Wave 7B。
**Gate／Evidence：** G-DEMO-PRESENTATION、G-DEMO-LAUNCHER。
**完成門檻：** Client 不讀 full world；launcher 只建立並清理預期 PID。

- [x] 7.2.1 執行 Team 1 HUD model test。
- [x] 7.2.2 執行 Team 2 HUD model test。
- [x] 7.2.3 執行 disclosed count filtered-source test。
- [x] 7.2.4 執行 remembered count cache-source test。
- [x] 7.2.5 執行 fog center follows local hero test。
- [x] 7.2.6 執行 vision radius render conversion test。
- [x] 7.2.7 執行 ghost sensitive-field omission test。
- [x] 7.2.8 執行 ghost targeting exclusion test。
- [x] 7.2.9 執行 ghost hash exclusion test。
- [x] 7.2.10 執行 launcher static dependency scan。
- [x] 7.2.11 驗證 launcher 不引用遺失 helper。
- [x] 7.2.12 驗證 batch 檔為 CRLF。
- [x] 7.2.13 執行一 server／兩 executor topology smoke。
- [x] 7.2.14 驗證三個 PID 互不混淆。
- [x] 7.2.15 驗證兩個 per-player log 檔分離。
- [x] 7.2.16 驗證 server early-exit cleanup。
- [x] 7.2.17 驗證 normal frontend-exit cleanup。
- [x] 7.2.18 寫入 frontend-launcher final summary 與 hashes。

### 7.3 雙視窗人工驗收與 release review

**目的：** 實際啟動 demo，讓使用者可直接查看不同 team view。
**輸入：** 7.1、7.2 all green。
**產出：** `evidence/final/manual/summary.json`、執行中的雙視窗或驗收截圖、final verdict。
**依賴：** 7.1、7.2。
**Owner／Wave：** Primary integrator／Wave 7C。
**Gate／Evidence：** G-DEMO-MANUAL、全部 blocking gates。
**完成門檻：** 雙視窗可供查看，所有自動 gate passed，無 unresolved P0/P1。

- [x] 7.3.1 凍結本次 omb binary hash。
- [x] 7.3.2 凍結本次 omfx executor hash。
- [x] 7.3.3 凍結 demo Lua package content hash。
- [x] 7.3.4 凍結 launcher hash。
- [x] 7.3.5 執行 `run_2player.bat`。
- [x] 7.3.6 確認 P1／Team 1 視窗已開啟。
- [x] 7.3.7 確認 P2／Team 2 視窗已開啟。
- [x] 7.3.8 確認兩視窗初始 disclosed 集合不同。
- [x] 7.3.9 確認移動英雄可觸發新 reveal。
- [x] 7.3.10 確認巡邏單位離開後顯示 ghost。
- [x] 7.3.11 確認 overlap 單位 public state 一致。
- [x] 7.3.12 確認 server 顯示兩隊 observer healthy。
- [x] 7.3.13 掃描 player logs 的 hidden/canonical disclosure。
- [x] 7.3.14 執行 `openspec validate two-team-fog-demo --strict`。
- [x] 7.3.15 確認每個已勾選 task 有唯一 evidence record。
- [x] 7.3.16 確認所有 blocking gate 為 passed。
- [x] 7.3.17 確認 unresolved P0/P1 為 0。
- [x] 7.3.18 寫入 final release verdict。

## 8. 雙隊視野與英雄操作修正

### 8.1 修正可見性權限

**目的：** 普通單位只依本地英雄的圓形視野揭露，自己的英雄則永遠可見。
**完成門檻：** 同隊普通單位在圈外仍隱藏；自己的英雄在圈外仍可見；敵方英雄進圈才揭露。

- [x] 8.1.1 找出同隊單位被無條件揭露的判定。
- [x] 8.1.2 將無條件 owner grant 限制為 `OwnerTeam` scope。
- [x] 8.1.3 保留 `Vision` scope 的幾何半徑判定。
- [x] 8.1.4 將兩位 demo 英雄標成 `OwnerTeam` scope。
- [x] 8.1.5 保持 100 個 grid unit 為 `Vision` scope。
- [x] 8.1.6 新增同隊圈外普通單位隱藏測試。
- [x] 8.1.7 新增自己的英雄永遠可見測試。
- [x] 8.1.8 新增敵方英雄依幾何視野揭露測試。

### 8.2 讓玩家看得見並能辨識自己的英雄

**目的：** 每個前端都以自己的英雄為中心，並用明確圖形標示可控制角色。
**完成門檻：** P1 與 P2 各自看到自己的英雄；鏡頭跟隨；HUD 說明操作方式。

- [x] 8.2.1 從 filtered replica 找出本地玩家擁有的 hero state。
- [x] 8.2.2 放大本地英雄的隊伍色圓環。
- [x] 8.2.3 為本地英雄加上白色外環。
- [x] 8.2.4 為本地英雄加上旗標。
- [x] 8.2.5 保留英雄視野半徑圓。
- [x] 8.2.6 在 demo 模式把鏡頭中心設成本地英雄位置。
- [x] 8.2.7 讓鏡頭在英雄移動後持續跟隨。
- [x] 8.2.8 在 HUD 顯示本地英雄標記說明。
- [x] 8.2.9 在 HUD 顯示右鍵移動提示。

### 8.2A 離開視野後完全隱藏

**目的：** 單位離開本地英雄的圓形視野後，不再保留畫面殘影。
**完成門檻：** Hide transition 使用 Forget；前端 remembered count 維持 0。

- [x] 8.2A.1 確認圈外灰色單位來自 `LastKnown` 記憶策略。
- [x] 8.2A.2 將 fog demo 的 grid unit 記憶策略改為 `Forget`。
- [x] 8.2A.3 保持 remembered presentation 不參與模擬與鎖步 hash。
- [x] 8.2A.4 實機移動英雄並確認離圈單位完全消失。
- [x] 8.2A.5 前端先取得本地英雄位置，再繪製 filtered demo entity。
- [x] 8.2A.6 使用與 server 相同的 700 world-unit 半徑裁切呈現。
- [x] 8.2A.7 確保本地 `OwnerTeam` 英雄不受呈現裁切影響。
- [x] 8.2A.8 圈外 entity 即使處於 transition delay 也不繪製。

### 8.3 集中式修正驗證

**目的：** 所有修正完成後，一次完成自動測試與雙視窗人工驗收。
**完成門檻：** 自動測試通過；兩隊初始畫面不同；兩位英雄可見且至少一位移動成功。

- [x] 8.3.1 執行 visibility 單元測試。
- [x] 8.3.2 執行玩家輸入路由與 hero movement 測試。
- [x] 8.3.3 執行 omoba-core 完整測試。
- [x] 8.3.4 執行 omobab 完整測試。
- [x] 8.3.5 執行 omfx 完整測試。
- [x] 8.3.6 執行 script ABI 與 base content 測試。
- [x] 8.3.7 執行 OpenSpec strict validation。
- [x] 8.3.8 重新建置 demo server 與兩個 frontend。
- [x] 8.3.9 啟動一個 server 與兩個獨立 frontend process。
- [x] 8.3.10 確認 P1 與 P2 的 disclosed 集合不同。
- [x] 8.3.11 確認兩個視窗各自顯示本地英雄標記。
- [x] 8.3.12 對英雄送出右鍵 MoveTo。
- [x] 8.3.13 確認伺服器接受正確玩家的 MoveTo。
- [x] 8.3.14 確認英雄座標改變且鏡頭跟隨。
- [x] 8.3.15 擷取兩個隊伍的最終畫面。
- [x] 8.3.16 寫入本次修正 evidence 與最終結論。

## 9. 修正右鍵輸入使用錯誤時間軸

### 9.1 對齊伺服器權威 tick

**目的：** selective replica 即使因 Wave B 與緩衝而落後，右鍵命令仍排到伺服器尚未執行的 tick。
**完成門檻：** 前端以 `TeamTickFrame.server_tick` 計算輸入基準；不再以延遲的 `replica_tick` 送出而被判定過晚。

- [x] 9.1.1 從伺服器日誌確認右鍵命令被判定為 `late InputSubmit`。
- [x] 9.1.2 比較送出時的 `base_tick` 與伺服器 `current_tick`。
- [x] 9.1.3 確認兩者差距來自 selective replica 的延遲時間軸。
- [x] 9.1.4 將輸入基準改成 team frame 的 `server_tick`。
- [x] 9.1.5 保留既有 2 tick 輸入前瞻。
- [x] 9.1.6 保持伺服器為最終權威，不加入前端預測位移。
- [x] 9.1.7 新增 server tick 與 replica tick 不同的回歸測試。
- [x] 9.1.8 新增超過 u32 範圍時的飽和測試。

### 9.2 集中驗證

**目的：** 程式修正完成後一次執行測試與實機驗收。
**完成門檻：** 測試通過；實機右鍵輸入不再出現 late rejection；英雄權威座標與畫面位置均改變。

- [x] 9.2.1 執行 omfx lockstep client 單元測試。
- [x] 9.2.2 執行 omfx 完整測試。
- [x] 9.2.3 執行 omobab 輸入路由測試。
- [x] 9.2.4 執行 OpenSpec strict validation。
- [x] 9.2.5 重建兩玩家 demo 前端。
- [x] 9.2.6 啟動 server 與兩個獨立 frontend process。
- [x] 9.2.7 對 P1 與 P2 各送出右鍵 MoveTo。
- [x] 9.2.8 確認 server 接受命令且沒有 late rejection。
- [x] 9.2.9 確認兩位英雄的權威座標改變。
- [x] 9.2.10 確認 filtered replica 呈現移動後位置。

## 10. LoL 式戰爭迷霧遮罩

### 10.1 將不可見地圖暗化

**目的：** 畫面直接區分目前可見與不可見區域，不再要求玩家從遮蔽物線框猜測視野。
**完成門檻：** 視野內維持原亮度；半徑外與被樹木／地形擋住的區域覆蓋灰黑半透明迷霧。

- [x] 10.1.1 撤回「只替遮蔽物本體填灰」的錯誤呈現方向。
- [x] 10.1.2 建立覆蓋 demo world-space 的迷霧 tile 網格。
- [x] 10.1.3 將迷霧圖層放在單位上方。
- [x] 10.1.4 使用本地英雄位置與 700 world-unit 半徑切出可見區。
- [x] 10.1.5 使用樹木圓形遮蔽物切出背後陰影。
- [x] 10.1.6 使用不規則地形邊界切出背後陰影。
- [x] 10.1.7 英雄移動時重新更新迷霧 tile。
- [x] 10.1.8 保持 server selective replica 為單位揭露的資料安全邊界。

### 10.2 集中驗證

- [x] 10.2.1 新增樹木遮擋線段測試。
- [x] 10.2.2 執行 omfx 完整測試。
- [x] 10.2.3 執行 OpenSpec strict validation。
- [x] 10.2.4 重建並啟動雙玩家 demo。
- [x] 10.2.5 確認視野外暗化且遮蔽物背後形成陰影。

## 11. 將迷霧取樣細化為 10×10

### 11.1 批次化細網格

**目的：** 消除 100×100 迷霧格造成的粗糙階梯邊界，同時避免大量 scene node 拖慢前端。
**完成門檻：** 每格為 10×10 world units；迷霧維持單一批次 draw；雙視窗仍能穩定執行。

- [x] 11.1.1 將迷霧 tile 尺寸從 100×100 改為 10×10。
- [x] 11.1.2 移除每格一個 scene node 的實作。
- [x] 11.1.3 使用單一 `BatchedSpriteMesh` 容納迷霧格。
- [x] 11.1.4 只建立鏡頭附近 220×180 格的移動視窗。
- [x] 11.1.5 英雄跨過 10-unit cell 時才重算與上傳。
- [x] 11.1.6 保持 server selective replica 為揭露權威。

### 11.2 集中驗證

- [x] 11.2.1 執行迷霧遮擋單元測試。
- [x] 11.2.2 執行 omfx 126 項完整測試。
- [x] 11.2.3 執行 OpenSpec strict validation。
- [x] 11.2.4 重建並啟動雙玩家 demo。
- [x] 11.2.5 確認視野邊界細化且前端約 60 FPS。

## 12. 修正迷霧遮住可見英雄

- [x] 12.1 找出 alpha 0 迷霧 quad 仍寫入 depth buffer 的問題。
- [x] 12.2 將可見格改成零尺寸退化 quad。
- [x] 12.3 保持不可見格的灰黑半透明遮罩。
- [x] 12.4 新增可見格不產生覆蓋幾何的回歸測試。
- [x] 12.5 找出相機跟隨錯誤依賴 team event 順序的問題。
- [x] 12.6 以 filtered local hero 作為最高優先相機依據。
- [x] 12.7 新增英雄位置到相機位置的座標回歸測試。
- [x] 12.8 找出 Windows 高 DPI 虛擬化造成 framebuffer 與視窗座標不一致。
- [x] 12.9 在建立 EventLoop 前固定 legacy renderer 使用 DPI unaware 座標。
- [x] 12.10 將自己的英雄改成實心隊色本體、白色外圈與旗標。
- [x] 12.11 執行 omfx 128 項完整測試與 OpenSpec strict validation。
- [x] 12.12 重建雙玩家 demo，確認英雄位於視野中心且完整顯示。
