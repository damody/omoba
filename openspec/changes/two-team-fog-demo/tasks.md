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

- [ ] 1.1.1 列出 `FOG_2TEAM_DEMO` 需要的 Lua package 檔案清單。
- [ ] 1.1.2 找出既有 story selection 的環境變數入口。
- [ ] 1.1.3 找出 Lua map loader 的回傳型別。
- [ ] 1.1.4 找出 campaign scene 建立英雄的函式入口。
- [ ] 1.1.5 找出一般單位可重用的 template ID。
- [ ] 1.1.6 找出 player ownership component 的既有型別。
- [ ] 1.1.7 找出 `VisionSource` 與 `ReplicationScope` 的建立入口。
- [ ] 1.1.8 找出 `RememberPolicy::LastKnown` 的建立入口。
- [ ] 1.1.9 找出 deterministic movement/patrol 可重用的 tick system。
- [ ] 1.1.10 找出 omfx filtered snapshot 的消費入口。
- [ ] 1.1.11 找出 omfx remembered render cache 的消費入口。
- [ ] 1.1.12 找出 server team observer 的啟動與診斷入口。
- [ ] 1.1.13 記錄不得重用的 viewport/legacy visibility path。
- [ ] 1.1.14 將 inventory 寫入 phase1 evidence。

### 1.2 Demo constants 與 evidence contract

**目的：** 將不應分散的場景常數與證據格式固定為單一來源。
**輸入：** 1.1 inventory、核准數量與視野決策。
**產出：** Demo constants contract、evidence index/schema。
**依賴：** 1.1。
**Owner／Wave：** Primary integrator／Wave 1B。
**Gate／Evidence：** G-DEMO-CARDINALITY、G-DEMO-DETERMINISM／`evidence/index.jsonl`。
**完成門檻：** 100、2、33／33／34、16、220、700 與 stable ordering 各有單一權威定義。

- [ ] 1.2.1 定義 grid rows 常數為 10。
- [ ] 1.2.2 定義 grid columns 常數為 10。
- [ ] 1.2.3 定義 grid spacing 常數為 220。
- [ ] 1.2.4 定義 grid unit count 常數為 100。
- [ ] 1.2.5 定義 player hero count 常數為 2。
- [ ] 1.2.6 定義 Team 1 grid count 常數為 33。
- [ ] 1.2.7 定義 Team 2 grid count 常數為 33。
- [ ] 1.2.8 定義 Neutral grid count 常數為 34。
- [ ] 1.2.9 定義 patrol unit count 常數為 16。
- [ ] 1.2.10 定義 vision radius 常數為 700。
- [ ] 1.2.11 定義 row-major stable spawn key 格式。
- [ ] 1.2.12 定義 deterministic team assignment 規則。
- [ ] 1.2.13 定義 deterministic patrol index 清單。
- [ ] 1.2.14 建立空的 append-only evidence index。
- [ ] 1.2.15 記錄 A／B／C adjustment 與 stale replacement 規則。

## 2. Lua content 與 validated descriptor

### 2.1 建立 `FOG_2TEAM_DEMO` package

**目的：** 建立只使用既有 assets 的 opt-in demo story。
**輸入：** 1.2 constants、既有 Lua package 範例。
**產出：** `scripts/lua_data/FOG_2TEAM_DEMO/`。
**依賴：** 1.2。
**Owner／Wave：** Content implementer／Wave 2A。
**Gate／Evidence：** G-DEMO-CARDINALITY／content hash 與 loader evidence。
**完成門檻：** Package 可被 runtime Lua loader 找到，且不修改其他 story。

- [ ] 2.1.1 建立 `FOG_2TEAM_DEMO` package 目錄。
- [ ] 2.1.2 建立 package 的 `map.lua` 入口。
- [ ] 2.1.3 宣告 demo map bounds。
- [ ] 2.1.4 宣告 grid origin。
- [ ] 2.1.5 宣告 10×10 grid dimensions。
- [ ] 2.1.6 宣告 220-unit spacing。
- [ ] 2.1.7 宣告 P1 hero spawn。
- [ ] 2.1.8 宣告 P2 hero spawn。
- [ ] 2.1.9 宣告 vision radius 700。
- [ ] 2.1.10 宣告 `LastKnown` remember policy。
- [ ] 2.1.11 宣告 16 個 patrol stable indexes。
- [ ] 2.1.12 宣告 patrol endpoint offset。
- [ ] 2.1.13 宣告 patrol speed。
- [ ] 2.1.14 引用一個既有 hero render template。
- [ ] 2.1.15 引用一個既有 grid unit render template。
- [ ] 2.1.16 確認 package 不引用不存在的 asset path。

### 2.2 擴充 Lua/import descriptor

**目的：** 將 demo 宣告轉成明確、可驗證的 Rust descriptor。
**輸入：** 2.1 package、現有 Lua map loader。
**產出：** Demo descriptor types 與 parser。
**依賴：** 2.1。
**Owner／Wave：** Backend content integrator／Wave 2B。
**Gate／Evidence：** G-DEMO-CARDINALITY、G-DEMO-DETERMINISM／focused parser tests。
**完成門檻：** 合法 package 完整 parse；每個不合法欄位在 scene mutation 前失敗。

- [ ] 2.2.1 新增 grid descriptor Rust type。
- [ ] 2.2.2 新增 hero spawn descriptor Rust type。
- [ ] 2.2.3 新增 patrol descriptor Rust type。
- [ ] 2.2.4 新增 demo visibility descriptor Rust type。
- [ ] 2.2.5 將 Lua grid dimensions parse 到 descriptor。
- [ ] 2.2.6 將 Lua spacing 與 origin parse 到 descriptor。
- [ ] 2.2.7 將兩個 hero spawn parse 到 descriptor。
- [ ] 2.2.8 將 patrol indexes parse 到 descriptor。
- [ ] 2.2.9 將 vision radius 與 remember policy parse 到 descriptor。
- [ ] 2.2.10 驗證 rows×columns 等於 100。
- [ ] 2.2.11 驗證 hero count 等於 2。
- [ ] 2.2.12 驗證所有座標與速度是 finite。
- [ ] 2.2.13 驗證 patrol index 唯一且位於 0..100。
- [ ] 2.2.14 驗證 stable spawn key 不重複。
- [ ] 2.2.15 驗證 team assignment 只產生 Team 1、Team 2、Neutral。
- [ ] 2.2.16 讓 validation error 包含欄位與實際值。
- [ ] 2.2.17 保證 validation 失敗前不建立 ECS entity。

## 3. Authoritative demo world

### 3.1 建立 100 個 grid units

**目的：** 以 deterministic row-major order 建立精確的 grid population。
**輸入：** 2.2 validated descriptor、existing unit templates。
**產出：** Demo grid scene builder。
**依賴：** 2.2。
**Owner／Wave：** Backend gameplay implementer／Wave 3A。
**Gate／Evidence：** G-DEMO-CARDINALITY、G-DEMO-DETERMINISM／scene tests。
**完成門檻：** 100 個單位、33／33／34 分布、位置與 stable keys 完全符合 descriptor。

- [ ] 3.1.1 新增 demo scene selection branch。
- [ ] 3.1.2 建立 row-major index helper。
- [ ] 3.1.3 建立 index 到 world position helper。
- [ ] 3.1.4 建立 index 到 team assignment helper。
- [ ] 3.1.5 建立 index 到 stable spawn key helper。
- [ ] 3.1.6 建立單一 grid unit entity builder。
- [ ] 3.1.7 在 builder 加入 position component。
- [ ] 3.1.8 在 builder 加入 unit/template component。
- [ ] 3.1.9 在 builder 加入 faction/team component。
- [ ] 3.1.10 在 builder 加入 stable identity input。
- [ ] 3.1.11 在 builder 加入 `TeamVision` replication scope。
- [ ] 3.1.12 在 builder 加入 `LastKnown` remember policy。
- [ ] 3.1.13 以固定順序建立全部 100 個 grid units。
- [ ] 3.1.14 在 scene build 後 assert grid count 為 100。
- [ ] 3.1.15 在 scene build 後 assert team counts 為 33／33／34。

### 3.2 建立兩個玩家英雄與 ownership

**目的：** 建立不計入 grid count 的兩個可控制 vision-source heroes。
**輸入：** 3.1 scene builder、authenticated bindings 1→1 與 2→2。
**產出：** 兩個 hero entities 與 player/team ownership。
**依賴：** 3.1。
**Owner／Wave：** Backend gameplay implementer／Wave 3B。
**Gate／Evidence：** G-DEMO-CARDINALITY、G-DEMO-TEAM-ISOLATION／hero ownership tests。
**完成門檻：** 總 gameplay unit count 102，兩個英雄分別只能由正確玩家控制。

- [ ] 3.2.1 建立 P1 hero stable spawn key。
- [ ] 3.2.2 建立 P2 hero stable spawn key。
- [ ] 3.2.3 以 descriptor 位置建立 P1 hero。
- [ ] 3.2.4 以 descriptor 位置建立 P2 hero。
- [ ] 3.2.5 將 P1 owner player ID 設為 1。
- [ ] 3.2.6 將 P2 owner player ID 設為 2。
- [ ] 3.2.7 將 P1 hero team 設為 1。
- [ ] 3.2.8 將 P2 hero team 設為 2。
- [ ] 3.2.9 為 P1 hero 建立半徑 700 `VisionSource`。
- [ ] 3.2.10 為 P2 hero 建立半徑 700 `VisionSource`。
- [ ] 3.2.11 讓兩個 hero 使用既有 player movement input path。
- [ ] 3.2.12 阻止 P1 input 控制 P2 hero。
- [ ] 3.2.13 阻止 P2 input 控制 P1 hero。
- [ ] 3.2.14 在 scene build 後 assert hero count 為 2。
- [ ] 3.2.15 在 scene build 後 assert gameplay unit total 為 102。

### 3.3 實作 16 個 deterministic patrol units

**目的：** 持續產生不依賴玩家操作的 visibility transitions。
**輸入：** 3.1 grid entities、2.2 patrol descriptor。
**產出：** Patrol state 與 authoritative tick update。
**依賴：** 3.1。
**Owner／Wave：** Backend gameplay implementer／Wave 3C。
**Gate／Evidence：** G-DEMO-DETERMINISM／patrol trace hashes。
**完成門檻：** 精確 16 個單位以 fixed-point、stable order 往返，重跑 trace 相同。

- [ ] 3.3.1 定義 demo patrol component/state。
- [ ] 3.3.2 將 patrol endpoints 量化為 fixed-point。
- [ ] 3.3.3 將 patrol speed 量化為 fixed-point。
- [ ] 3.3.4 只為固定 index 清單加入 patrol state。
- [ ] 3.3.5 以 stable spawn key 排序 patrol update。
- [ ] 3.3.6 實作朝目前 endpoint 的 fixed-step movement。
- [ ] 3.3.7 實作到達 endpoint 的 deterministic clamp。
- [ ] 3.3.8 實作 endpoint direction reversal。
- [ ] 3.3.9 防止 zero-distance path 產生除零。
- [ ] 3.3.10 將 patrol update 放入 authoritative Wave A。
- [ ] 3.3.11 確保 patrol 不讀 wall clock。
- [ ] 3.3.12 確保 patrol 不使用 runtime RNG。
- [ ] 3.3.13 確保非 patrol units 保持靜止。
- [ ] 3.3.14 產生固定 tick patrol position trace。

## 4. Team projection 與 observer 整合

### 4.1 接上 team visibility projection

**目的：** 讓 demo entities 使用既有 secure V2 disclosure pipeline。
**輸入：** 3.1–3.3 authoritative world、existing `TeamViewProjector`。
**產出：** Team 1／Team 2 filtered frames。
**依賴：** 3.1、3.2、3.3。
**Owner／Wave：** Selective projection integrator／Wave 4A。
**Gate／Evidence：** G-DEMO-TEAM-ISOLATION／projection scenario evidence。
**完成門檻：** 每隊只收到 700-unit shared view；Neutral 不會 global disclose。

- [ ] 4.1.1 將 demo hero `VisionSource` 納入 team visibility input。
- [ ] 4.1.2 將 grid unit scope 納入 projection classification。
- [ ] 4.1.3 讓 Neutral grid units 使用 `TeamVision` 而非 `Public`。
- [ ] 4.1.4 驗證 force-hide precedence 仍優先於 demo vision。
- [ ] 4.1.5 使用既有 visibility commit delay。
- [ ] 4.1.6 在 reveal effective tick 擷取 fresh baseline。
- [ ] 4.1.7 在 hide effective tick 停止 gameplay delta。
- [ ] 4.1.8 為 hide 產生去敏感化 `LastKnown` presentation。
- [ ] 4.1.9 保持每隊獨立 `ReplicaEntityId` mapping。
- [ ] 4.1.10 拒絕 stale disclosure epoch input。
- [ ] 4.1.11 確保 viewport update 不觸發 visibility transition。
- [ ] 4.1.12 確保 camera state 不進 authoritative hash。
- [ ] 4.1.13 確保 hidden grid unit 不進 team snapshot。
- [ ] 4.1.14 確保 hidden-only patrol movement 不改變 team frame bytes。

### 4.2 接上兩隊 observer replica

**目的：** 驗算兩隊實際送出 bytes，而不阻塞 outbound。
**輸入：** 4.1 encoded team streams、existing observer worker。
**產出：** Team 1／Team 2 observer diagnostics。
**依賴：** 4.1。
**Owner／Wave：** Observer integrator／Wave 4B。
**Gate／Evidence：** G-DEMO-OBSERVER／observer smoke report。
**完成門檻：** 兩個 observer 只消費自己的 stream，正常 demo run 無 mismatch 或未處理 gap。

- [ ] 4.2.1 在 Team 1 active 時建立 Team 1 observer state。
- [ ] 4.2.2 在 Team 2 active 時建立 Team 2 observer state。
- [ ] 4.2.3 確認 encoded frame 先進 session send queue。
- [ ] 4.2.4 將同一份 `Arc<[u8]>` 非阻塞 tap 給 observer。
- [ ] 4.2.5 禁止 observer 讀 authoritative Specs world。
- [ ] 4.2.6 禁止 Team 1 observer 讀 Team 2 bootstrap/frame。
- [ ] 4.2.7 禁止 Team 2 observer 讀 Team 1 bootstrap/frame。
- [ ] 4.2.8 記錄 per-team observer sequence 與 tick。
- [ ] 4.2.9 記錄 per-team hash mismatch diagnostics。
- [ ] 4.2.10 記錄 validation queue coverage gap。
- [ ] 4.2.11 保留 overflow 後 filtered rebootstrap path。
- [ ] 4.2.12 將 demo observer 狀態輸出到 server-only log。

## 5. omfx fog demo presentation

### 5.1 建立 demo session 與 HUD model

**目的：** 讓每個 frontend 清楚顯示自己的身份與 filtered counts。
**輸入：** 4.1 filtered snapshot、launcher environment contract。
**產出：** Demo presentation state 與 HUD labels。
**依賴：** 4.1。
**Owner／Wave：** Frontend implementer／Wave 5A。
**Gate／Evidence：** G-DEMO-PRESENTATION／frontend focused tests。
**完成門檻：** HUD 不讀 full-world count，且兩視窗 team/player 資訊正確。

- [ ] 5.1.1 新增 fog demo session detection。
- [ ] 5.1.2 從 secure start binding 取得 local team ID。
- [ ] 5.1.3 將 Team 1 presentation color 固定為藍色。
- [ ] 5.1.4 將 Team 2 presentation color 固定為紅色。
- [ ] 5.1.5 建立固定文字 `Demo grid units: 100`。
- [ ] 5.1.6 建立固定文字 `Player heroes: 2`。
- [ ] 5.1.7 從 filtered live entities 計算 `Currently disclosed`。
- [ ] 5.1.8 從 remembered cache 計算 `Remembered ghosts`。
- [ ] 5.1.9 顯示 local player ID。
- [ ] 5.1.10 顯示 local team ID。
- [ ] 5.1.11 顯示 replica tick。
- [ ] 5.1.12 顯示點擊移動與觀察提示。
- [ ] 5.1.13 阻止 HUD 讀 authoritative total entity count。

### 5.2 繪製圓形視野與 fog overlay

**目的：** 以 renderer-only 圖層清楚表示 local team 的已知視野。
**輸入：** 5.1 local hero snapshot、既有 circle drawing helper。
**產出：** Vision circle 與 fog overlay render pass。
**依賴：** 5.1。
**Owner／Wave：** Frontend implementer／Wave 5B。
**Gate／Evidence：** G-DEMO-PRESENTATION／render model tests、人工畫面。
**完成門檻：** 圓與 fog 跟隨自有英雄；renderer state 不回寫 gameplay authority。

- [ ] 5.2.1 從 filtered snapshot 找出 local owned hero。
- [ ] 5.2.2 將 demo radius 700 轉成 render units。
- [ ] 5.2.3 重用既有 batched circle drawing helper。
- [ ] 5.2.4 使用 local team color 畫 circle outline。
- [ ] 5.2.5 建立半透明 fog overlay material/state。
- [ ] 5.2.6 以 local hero position 更新 fog opening center。
- [ ] 5.2.7 以 radius 700 更新 fog opening size。
- [ ] 5.2.8 將 fog layer 排在 terrain 之上、units 之下或採等價可讀順序。
- [ ] 5.2.9 在 local hero 尚未 disclosed 時安全隱藏 demo overlay。
- [ ] 5.2.10 避免每 entity 每 frame 建立新 material。
- [ ] 5.2.11 避免 circle draw 產生 per-frame heap churn。
- [ ] 5.2.12 確保 camera movement 不改變 server visibility input。

### 5.3 呈現並隔離 LastKnown ghosts

**目的：** 顯示已去敏感化的 remembered state且不污染 simulation。
**輸入：** Existing remembered cache、4.1 hide transition。
**產出：** Low-opacity ghost render records。
**依賴：** 4.1、5.1。
**Owner／Wave：** Frontend implementer／Wave 5C。
**Gate／Evidence：** G-DEMO-PRESENTATION、G-DEMO-TEAM-ISOLATION／cache isolation tests。
**完成門檻：** Ghost 可見但無法被 target、碰撞或計入 hash/count。

- [ ] 5.3.1 將 hide presentation 寫入獨立 remembered cache。
- [ ] 5.3.2 保存 last disclosed render position。
- [ ] 5.3.3 保存 render-safe unit kind/color。
- [ ] 5.3.4 不保存 hidden current position。
- [ ] 5.3.5 不保存 hidden HP 或其他敏感 component。
- [ ] 5.3.6 以低透明度繪製 ghost。
- [ ] 5.3.7 Reveal 同 identity 時移除對應 ghost。
- [ ] 5.3.8 Forget transition 時移除對應 ghost。
- [ ] 5.3.9 從 gameplay target lookup 排除 remembered cache。
- [ ] 5.3.10 從 collision query 排除 remembered cache。
- [ ] 5.3.11 從 team hash 排除 remembered cache。
- [ ] 5.3.12 從 `Currently disclosed` 排除 remembered cache。

## 6. Windows 雙 process launcher

### 6.1 修正 `run_2player.bat` 啟動拓撲

**目的：** 建立可重複的一 server、兩 frontend 開發入口。
**輸入：** Existing freshness helpers、omb/omfx binaries、demo story。
**產出：** CRLF `run_2player.bat` 與必要的非根目錄 helper。
**依賴：** 3.2、5.1。
**Owner／Wave：** Primary integrator／Wave 6A。
**Gate／Evidence：** G-DEMO-LAUNCHER／launcher static/smoke evidence。
**完成門檻：** 不依賴遺失 helper；三個 process identity、環境與 PID 都可追蹤。

- [ ] 6.1.1 將 launcher story 設為 `FOG_2TEAM_DEMO`。
- [ ] 6.1.2 保留 script DLL freshness check。
- [ ] 6.1.3 保留 backend freshness check。
- [ ] 6.1.4 保留 frontend freshness check。
- [ ] 6.1.5 在啟動前驗證 demo Lua package 存在。
- [ ] 6.1.6 移除對 `run_2player_client.bat` 的呼叫。
- [ ] 6.1.7 建立 P1 獨立 process environment。
- [ ] 6.1.8 建立 P2 獨立 process environment。
- [ ] 6.1.9 設定 P1 player ID/name/lockstep name。
- [ ] 6.1.10 設定 P2 player ID/name/lockstep name。
- [ ] 6.1.11 設定 P1 team/window/log suffix。
- [ ] 6.1.12 設定 P2 team/window/log suffix。
- [ ] 6.1.13 保存 server PID。
- [ ] 6.1.14 保存 P1 executor PID。
- [ ] 6.1.15 保存 P2 executor PID。
- [ ] 6.1.16 確認 P1 與 P2 executor PID 不同。
- [ ] 6.1.17 正常結束只停止本次 server PID。
- [ ] 6.1.18 Server 提前退出只停止本次兩個 executor PID。
- [ ] 6.1.19 將缺 artifact 情況轉成非零 exit code。
- [ ] 6.1.20 將 `run_2player.bat` 正規化為 CRLF。

### 6.2 視窗辨識與左右排列

**目的：** 讓人工驗收可直接並排比較兩個 team view。
**輸入：** 6.1 process identity、omfx executor window initialization。
**產出：** Per-process title 與 best-effort window position。
**依賴：** 6.1。
**Owner／Wave：** Frontend launcher integrator／Wave 6B。
**Gate／Evidence：** G-DEMO-LAUNCHER、G-DEMO-MANUAL／window evidence。
**完成門檻：** 標題永遠正確；單螢幕可左右排列，定位失敗不終止 match。

- [ ] 6.2.1 定義 P1 window title suffix。
- [ ] 6.2.2 定義 P2 window title suffix。
- [ ] 6.2.3 在 executor title 顯示 P1／Team 1。
- [ ] 6.2.4 在 executor title 顯示 P2／Team 2。
- [ ] 6.2.5 取得 primary monitor work area。
- [ ] 6.2.6 計算左側視窗 bounds。
- [ ] 6.2.7 計算右側視窗 bounds。
- [ ] 6.2.8 對 P1 套用左側 bounds。
- [ ] 6.2.9 對 P2 套用右側 bounds。
- [ ] 6.2.10 將定位失敗記為 warning。
- [ ] 6.2.11 確保定位失敗不更改 secure connection state。

## 7. 集中式最終驗證與人工預覽

### 7.1 Content 與 backend 最終驗證

**目的：** 一次驗證 cardinality、determinism、ownership、patrol 與 visibility。
**輸入：** Phase 2–4 完成實作。
**產出：** `evidence/final/backend/summary.json`。
**依賴：** 2.2、3.1–3.3、4.1–4.2。
**Owner／Wave：** Primary verifier／Wave 7A。
**Gate／Evidence：** G-DEMO-CARDINALITY、G-DEMO-DETERMINISM、G-DEMO-TEAM-ISOLATION、G-DEMO-OBSERVER。
**完成門檻：** 所有 scenario 通過且 hidden disclosure finding 為 0。

- [ ] 7.1.1 執行合法 demo Lua package load test。
- [ ] 7.1.2 執行錯誤 grid count rejection test。
- [ ] 7.1.3 執行 duplicate spawn key rejection test。
- [ ] 7.1.4 執行 non-finite coordinate rejection test。
- [ ] 7.1.5 驗證 grid unit count 為 100。
- [ ] 7.1.6 驗證 hero count 為 2。
- [ ] 7.1.7 驗證 gameplay unit total 為 102。
- [ ] 7.1.8 驗證 team distribution 為 33／33／34。
- [ ] 7.1.9 比對兩次 spawn manifest hash。
- [ ] 7.1.10 比對兩次 patrol trace hash。
- [ ] 7.1.11 執行 P1 ownership input case。
- [ ] 7.1.12 執行 P2 ownership input case。
- [ ] 7.1.13 執行 cross-owner rejection case。
- [ ] 7.1.14 執行 radius-inside reveal case。
- [ ] 7.1.15 執行 radius-outside hide case。
- [ ] 7.1.16 執行 visibility boundary exact-distance case。
- [ ] 7.1.17 執行 viewport non-authority case。
- [ ] 7.1.18 執行 Neutral non-public case。
- [ ] 7.1.19 執行 hidden-only patrol byte non-interference case。
- [ ] 7.1.20 執行兩隊 observer isolation case。
- [ ] 7.1.21 執行 observer outbound-nonblocking case。
- [ ] 7.1.22 寫入 backend final summary 與 hashes。

### 7.2 Frontend 與 launcher 最終驗證

**目的：** 驗證 filtered presentation 與 Windows process topology。
**輸入：** Phase 5–6 完成實作。
**產出：** `evidence/final/frontend-launcher/summary.json`。
**依賴：** 5.1–5.3、6.1–6.2。
**Owner／Wave：** Primary verifier／Wave 7B。
**Gate／Evidence：** G-DEMO-PRESENTATION、G-DEMO-LAUNCHER。
**完成門檻：** Client 不讀 full world；launcher 只建立並清理預期 PID。

- [ ] 7.2.1 執行 Team 1 HUD model test。
- [ ] 7.2.2 執行 Team 2 HUD model test。
- [ ] 7.2.3 執行 disclosed count filtered-source test。
- [ ] 7.2.4 執行 remembered count cache-source test。
- [ ] 7.2.5 執行 fog center follows local hero test。
- [ ] 7.2.6 執行 vision radius render conversion test。
- [ ] 7.2.7 執行 ghost sensitive-field omission test。
- [ ] 7.2.8 執行 ghost targeting exclusion test。
- [ ] 7.2.9 執行 ghost hash exclusion test。
- [ ] 7.2.10 執行 launcher static dependency scan。
- [ ] 7.2.11 驗證 launcher 不引用遺失 helper。
- [ ] 7.2.12 驗證 batch 檔為 CRLF。
- [ ] 7.2.13 執行一 server／兩 executor topology smoke。
- [ ] 7.2.14 驗證三個 PID 互不混淆。
- [ ] 7.2.15 驗證兩個 per-player log 檔分離。
- [ ] 7.2.16 驗證 server early-exit cleanup。
- [ ] 7.2.17 驗證 normal frontend-exit cleanup。
- [ ] 7.2.18 寫入 frontend-launcher final summary 與 hashes。

### 7.3 雙視窗人工驗收與 release review

**目的：** 實際啟動 demo，讓使用者可直接查看不同 team view。
**輸入：** 7.1、7.2 all green。
**產出：** `evidence/final/manual/summary.json`、執行中的雙視窗或驗收截圖、final verdict。
**依賴：** 7.1、7.2。
**Owner／Wave：** Primary integrator／Wave 7C。
**Gate／Evidence：** G-DEMO-MANUAL、全部 blocking gates。
**完成門檻：** 雙視窗可供查看，所有自動 gate passed，無 unresolved P0/P1。

- [ ] 7.3.1 凍結本次 omb binary hash。
- [ ] 7.3.2 凍結本次 omfx executor hash。
- [ ] 7.3.3 凍結 demo Lua package content hash。
- [ ] 7.3.4 凍結 launcher hash。
- [ ] 7.3.5 執行 `run_2player.bat`。
- [ ] 7.3.6 確認 P1／Team 1 視窗已開啟。
- [ ] 7.3.7 確認 P2／Team 2 視窗已開啟。
- [ ] 7.3.8 確認兩視窗初始 disclosed 集合不同。
- [ ] 7.3.9 確認移動英雄可觸發新 reveal。
- [ ] 7.3.10 確認巡邏單位離開後顯示 ghost。
- [ ] 7.3.11 確認 overlap 單位 public state 一致。
- [ ] 7.3.12 確認 server 顯示兩隊 observer healthy。
- [ ] 7.3.13 掃描 player logs 的 hidden/canonical disclosure。
- [ ] 7.3.14 執行 `openspec validate two-team-fog-demo --strict`。
- [ ] 7.3.15 確認每個已勾選 task 有唯一 evidence record。
- [ ] 7.3.16 確認所有 blocking gate 為 passed。
- [ ] 7.3.17 確認 unresolved P0/P1 為 0。
- [ ] 7.3.18 寫入 final release verdict。
