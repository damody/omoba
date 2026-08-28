## 1. 共用揭露契約

目的：先消除server projector、server observer與client runtime的schema差異。主要檔案：`omoba-core/src/runtime/**`、`omb/src/**`、`omfx/game/src/sim_runner.rs`。前置依賴：無。完成門檻：所有production consumer只呼叫同一API；本章不執行測試。

- [x] 1.1 在`omoba-core::runtime`新增`secure_replica_component_allowlist()`函式簽章
- [x] 1.2 將既有核准component schema ID移入共用component allowlist
- [x] 1.3 在`omoba-core::runtime`新增`secure_replica_resource_allowlist()`函式簽章
- [x] 1.4 將既有核准resource schema ID移入共用resource allowlist
- [x] 1.5 讓allowlist回傳具有固定排序的集合
- [x] 1.6 將server team projector改為呼叫共用component allowlist
- [x] 1.7 將server team projector改為呼叫共用resource allowlist
- [x] 1.8 將Team 1 server observer改為呼叫共用allowlist
- [x] 1.9 將Team 2 server observer改為呼叫共用allowlist
- [x] 1.10 移除`omfx/game/src/sim_runner.rs`的局部component ID集合
- [x] 1.11 新增production source guard規則以禁止consumer手寫secure schema集合
- [x] 1.12 在allowlist API文件列出新增schema時唯一允許的更新位置

## 2. 建立 external client runtime crate

目的：建立不依賴Fyrox/Unreal的可執行骨架。主要檔案：workspace `Cargo.toml`、`omoba-client-runtime/Cargo.toml`、`omoba-client-runtime/src/**`。前置依賴：第1章。完成門檻：crate結構、設定與錯誤型別完整；尚不連server。

- [x] 2.1 建立`omoba-client-runtime/Cargo.toml`
- [x] 2.2 將`omoba-client-runtime`加入正確Cargo workspace
- [x] 2.3 新增`src/lib.rs`並公開必要模組
- [x] 2.4 新增`src/main.rs`的單一啟動入口
- [x] 2.5 新增`config.rs`的`ClientRuntimeConfig`
- [x] 2.6 加入`player_id`命令列參數解析
- [x] 2.7 加入只允許1或2的`team_id`參數解析
- [x] 2.8 加入authoritative server位址參數解析
- [x] 2.9 加入presentation bind位址參數解析
- [x] 2.10 拒絕非loopback presentation bind位址
- [x] 2.11 加入evidence目錄與test mode參數解析
- [x] 2.12 加入protocol與content hash設定欄位
- [x] 2.13 新增runtime錯誤enum並區分config/session/replica/IPC錯誤
- [x] 2.14 新增不含hidden payload的結構化startup log
- [x] 2.15 新增`shutdown.rs` cancellation token與退出原因型別

## 3. 抽出secure session與frame barrier

目的：讓external runtime直接消費production KCP secure V2。主要檔案：`omoba-client-runtime/src/session.rs`、`omfx/game/src/sim_runner.rs`、`omoba-core/src/**`。前置依賴：第2章。完成門檻：session可bootstrap並依序輸出完整team frame，錯誤時fail closed。

- [x] 3.1 將secure V2 join request建立邏輯移到可共用模組
- [x] 3.2 在runtime session送出固定player/team binding
- [x] 3.3 驗證server回覆的team與設定team一致
- [x] 3.4 驗證secure V2 capability且禁止legacy downgrade
- [x] 3.5 解碼`TeamGameStart`並保存`global_seed`
- [x] 3.6 解碼filtered bootstrap snapshot
- [x] 3.7 保存bootstrap的下一個team sequence
- [x] 3.8 將frame barrier移入runtime session
- [x] 3.9 拒絕duplicate team sequence
- [x] 3.10 拒絕sequence gap且不輸出部分frame
- [x] 3.11 拒絕wrong team與wrong disclosure epoch
- [x] 3.12 拒絕unknown schema而不略過payload
- [x] 3.13 加入安全replay request狀態轉移
- [x] 3.14 加入安全rebase response狀態轉移
- [x] 3.15 server斷線時停止輸出simulation frame
- [x] 3.16 移除omfx secure mode對authoritative KCP session的ownership

## 4. 建立filtered Specs replica host

目的：每個runtime process只建立一個獨立filtered world。主要檔案：`omoba-client-runtime/src/replica_host.rs`、`omoba-core/src/runtime/**`。前置依賴：第1、3章。完成門檻：bootstrap、transition、step、hash、correction由replica host完整負責。

- [x] 4.1 新增`ReplicaHost`並唯一持有`SelectiveReplicaRuntime`
- [x] 4.2 讓`ReplicaHost`唯一持有`SpecsDisclosedWorldStepper`
- [x] 4.3 以共用allowlist建立空filtered Specs world
- [x] 4.4 禁止filtered world執行完整story/map spawn
- [x] 4.5 從bootstrap建立replica-local entity mapping
- [x] 4.6 將canonical identity限制在wire-edge mapping內
- [x] 4.7 為每個runtime建立獨立ScriptRegistry
- [x] 4.8 為每個runtime建立獨立dispatcher
- [x] 4.9 實作Reveal current baseline建立
- [x] 4.10 實作Hide transition狀態更新
- [x] 4.11 實作Forget刪除Specs entity與target lookup
- [x] 4.12 讓Forget後舊replica ID永久失效
- [x] 4.13 實作LastKnown sanitized remembered cache
- [x] 4.14 排除ghost的Specs component與collision註冊
- [x] 4.15 排除ghost的targeting、script與team hash
- [x] 4.16 注入accepted inputs到指定tick
- [x] 4.17 注入sanitized external effects到指定phase
- [x] 4.18 以`global_seed + tick`建立tick-local RNG
- [x] 4.19 呼叫共用deterministic gameplay phases
- [x] 4.20 計算並保存pre-repair team hash
- [x] 4.21 在套server correction前保存divergence record
- [x] 4.22 實作server repair套用
- [x] 4.23 實作server replace/rebase套用
- [x] 4.24 連續無法收斂時終止該secure session

## 5. Server兩隊observer與三方hash

目的：server內以兩條thread同時驗算Team 1/2，並和external runtime配對。主要檔案：`omb/src/**`、`omoba-core/src/runtime/**`。前置依賴：第1、4章的共用契約。完成門檻：每個checkpoint保存三方pre-repair診斷並驗證post-repair收斂，server仍是唯一authority。

- [x] 5.1 建立固定Team 1 observer worker thread
- [x] 5.2 建立固定Team 2 observer worker thread
- [x] 5.3 為兩條worker建立互不共享的filtered world
- [x] 5.4 讓observer消費正式encoded frame的decode語意
- [x] 5.5 禁止observer從canonical world補未投影component
- [x] 5.6 定義`ReplicaCheckpointKey`四個固定欄位
- [x] 5.7 保存server expected pre-repair hash
- [x] 5.8 保存server observer pre-repair hash
- [x] 5.9 接收external runtime pre-repair hash report
- [x] 5.10 依checkpoint key配對而非arrival order配對
- [x] 5.11 讓兩隊worker completion order不影響authority
- [x] 5.12 將missing external report標記`UNVERIFIED`
- [x] 5.13 將coverage gap標記`UNVERIFIED`
- [x] 5.14 將observer worker crash標記`UNVERIFIED`
- [x] 5.15 三方mismatch時先保存證據再產生server correction
- [x] 5.16 保留pre-repair診斷，blocking verdict改以三方post-repair收斂判定

## 6. Server team frame queue與安全邊界

目的：確保兩隊frame不遺失、不降級、不夾帶global state。主要檔案：`omb/src/transport/**`、`omb/src/lockstep/**`。前置依賴：第5章。完成門檻：queue滿載會阻塞authority且sequence保持連續。

- [x] 6.1 將每隊secure outbound frame queue設為bounded
- [x] 6.2 queue滿載時阻塞authoritative tick
- [x] 6.3 禁止滿載時丟棄必要team frame
- [x] 6.4 禁止只讓其中一隊跳過tick
- [x] 6.5 輸出不含payload的queue blocked diagnostic
- [x] 6.6 輸出不含payload的queue resumed diagnostic
- [x] 6.7 從secure bootstrap移除global snapshot fallback
- [x] 6.8 從secure tick移除legacy完整`TickBatch` fallback
- [x] 6.9 阻止legacy render event進入secure team session
- [x] 6.10 無法安全rebase時關閉單一session而不影響另一隊

## 7. 定義presentation與input protobuf

目的：建立omfx與未來Unreal都能使用的穩定IPC schema。主要檔案：`proto/game.proto`、generated Rust schema、`omoba-core` shared types。前置依賴：第4章資料邊界。完成門檻：每個message有version與明確安全欄位，不含canonical ID。

- [x] 7.1 新增IPC envelope magic與version欄位
- [x] 7.2 新增runtime ready message
- [x] 7.3 新增team identity與tick status message
- [x] 7.4 新增render entity message並使用replica-safe render ID
- [x] 7.5 新增removed render ID message
- [x] 7.6 新增remembered ghost message
- [x] 7.7 新增10×10 fog tile message
- [x] 7.8 新增vision circle message
- [x] 7.9 新增render-safe tree occluder message
- [x] 7.10 新增render-safe polygon blocked region message
- [x] 7.11 新增effect與audio cue message
- [x] 7.12 新增critical input result message
- [x] 7.13 新增session stall與termination message
- [x] 7.14 新增MoveTo input message
- [x] 7.15 新增AttackMove input message
- [x] 7.16 新增AbilityCast input message
- [x] 7.17 新增ItemUse input message
- [x] 7.18 新增Tower action input message
- [x] 7.19 新增renderer ready與consumed sequence message
- [x] 7.20 新增graceful shutdown message
- [x] 7.21 設定IPC frame最大bytes常數
- [x] 7.22 重新產生Rust protobuf型別
- [x] 7.23 加入供未來Unreal生成C++型別的schema說明

## 8. 實作runtime presentation IPC

目的：runtime以安全、有界方式服務renderer。主要檔案：`omoba-client-runtime/src/presentation_bridge.rs`、`input_bridge.rs`。前置依賴：第7章。完成門檻：loopback server可收input、送latest presentation與critical event。

- [x] 8.1 建立loopback TCP listener
- [x] 8.2 實作big-endian length prefix reader
- [x] 8.3 實作big-endian length prefix writer
- [x] 8.4 在配置上限前拒絕過長frame
- [x] 8.5 驗證每個envelope的magic與version
- [x] 8.6 解碼renderer ready並回傳latest full presentation
- [x] 8.7 建立bounded latest-wins snapshot slot
- [x] 8.8 建立獨立reliable ordered critical queue
- [x] 8.9 實作30 Hz presentation cadence
- [x] 8.10 實作預設60 Hz presentation cadence
- [x] 8.11 實作120 Hz presentation cadence
- [x] 8.12 將filtered entities投影成render-safe IDs
- [x] 8.13 將Forget轉成removed render IDs
- [x] 8.14 將LastKnown轉成sanitized ghost
- [x] 8.15 將fog、vision與occluder投影到presentation
- [x] 8.16 保證presentation不輸出canonical Specs ID
- [x] 8.17 解碼input時驗證player owner
- [x] 8.18 解碼input時驗證disclosure epoch
- [x] 8.19 解碼target input時驗證target membership
- [x] 8.20 立即轉送合法input而不等待presentation cadence
- [x] 8.21 保存runtime rejection為critical input result
- [x] 8.22 renderer斷線後啟動bounded grace timer
- [x] 8.23 renderer重連時重用既有filtered world
- [x] 8.24 grace到期後停止presentation但不切換team

## 9. 將omfx切成renderer-only

目的：secure fog renderer不持有simulation或server連線。主要檔案：`omfx/game/src/**`、`omfx/executor/**`。前置依賴：第7、8章。完成門檻：secure mode只由IPC驅動畫面與input。

- [x] 9.1 新增明確的renderer-only啟動參數
- [x] 9.2 新增presentation IPC client模組
- [x] 9.3 將session ready映射到omfx連線狀態
- [x] 9.4 將render entities映射到既有scene cache
- [x] 9.5 將removed render IDs映射到scene清理
- [x] 9.6 將remembered ghost映射到非互動式畫面物件
- [x] 9.7 將10×10 fog tiles畫成LOL式暗色戰爭迷霧
- [x] 9.8 將己隊vision circles只作為debug overlay選項
- [x] 9.9 將tree與polygon occlusion結果映射到fog畫面
- [x] 9.10 將右鍵點擊編成MoveTo IPC message
- [x] 9.11 將AttackMove UI編成IPC message
- [x] 9.12 將技能UI編成AbilityCast IPC message
- [x] 9.13 將item UI編成ItemUse IPC message
- [x] 9.14 將tower UI編成Tower action IPC message
- [x] 9.15 讓input click被處理後不落入其他map handler
- [x] 9.16 secure mode停止建立`SelectiveReplicaRuntime`
- [x] 9.17 secure mode停止建立`SpecsDisclosedWorldStepper`
- [x] 9.18 secure mode停止載入script DLL
- [x] 9.19 secure mode停止建立authoritative KCP client
- [x] 9.20 移除renderer自行計算gameplay visibility的路徑
- [x] 9.21 移除renderer自行驗證target legality的路徑
- [x] 9.22 renderer reconnect後套用latest full presentation
- [x] 9.23 加入source guard禁止renderer-only模組引用Specs/KCP/script loader

## 10. Server與runtime input端到端路由

目的：讓玩家意圖經runtime過濾後仍由server最終裁決。主要檔案：`omoba-client-runtime/src/input_bridge.rs`、`omb/src/tick/player_input_tick.rs`。前置依賴：第3、8、9章。完成門檻：MoveTo等input依target tick套用，client與server拒絕可區分。

- [x] 10.1 為每個renderer input配置唯一input ID
- [x] 10.2 將合法input包成shared `PlayerInput`
- [x] 10.3 保存input送出tick與target tick
- [x] 10.4 將server acceptance映射到critical result
- [x] 10.5 將runtime-local rejection映射到不同result code
- [x] 10.6 將server-authoritative rejection映射到不同result code
- [x] 10.7 讓server重新驗證player ownership
- [x] 10.8 讓server重新驗證target visibility epoch
- [x] 10.9 讓server重新驗證stale/hidden target
- [x] 10.10 讓MoveTo只修改owning hero
- [x] 10.11 讓accepted input隨對應team frame回到runtime
- [x] 10.12 讓runtime只在排定tick套用accepted input
- [x] 10.13 server結果與runtime預期衝突時採server結果
- [x] 10.14 禁止renderer optimistic修改英雄simulation位置

## 11. Demo內容與可視性行為

目的：建立能穩定跨越視野與遮擋邊界的雙隊場景。主要檔案：`scripts/lua_data/**`、`scripts/base_content/**`、story generated IDs。前置依賴：第4、10章。完成門檻：場景資料固定、動作以tick定義且兩隊視野非對稱。

- [x] 11.1 確認`FOG_2TEAM_DEMO`建立10×10排列的100個普通單位
- [x] 11.2 在100個普通單位之外建立Team 1英雄
- [x] 11.3 在100個普通單位之外建立Team 2英雄
- [x] 11.4 將Player 1固定綁定Team 1英雄
- [x] 11.5 將Player 2固定綁定Team 2英雄
- [x] 11.6 讓兩名己方英雄永遠進入各隊初始揭露
- [x] 11.7 配置圓形英雄視野來源
- [x] 11.8 配置10×10 fog tile尺寸與world mapping
- [x] 11.9 配置至少16個deterministic patrol unit
- [x] 11.10 配置至少一個Forget離場路徑
- [x] 11.11 配置至少一個LastKnown離場路徑
- [x] 11.12 新增可阻擋line of sight的tree circle
- [x] 11.13 新增可阻擋line of sight的不規則polygon
- [x] 11.14 配置相同距離但不同遮擋結果的兩個target
- [x] 11.15 配置兩隊在不同tick reveal同一對英雄的路徑
- [x] 11.16 將scripted MoveTo動作固定到authoritative tick
- [x] 11.17 將screenshot trigger固定到authoritative tick
- [x] 11.18 確保故事內容不把另一隊sentinel寫入公開metadata

## 12. Sentinel與安全證據production工具

目的：建立每次run都不同且可跨邊界掃描的hidden canary。主要檔案：`omb/src/**`、`omoba-client-runtime/src/evidence.rs`、既有`scripts/*.ps1`。前置依賴：第5、8、11章。完成門檻：工具能產生、注入、dump、scan並輸出機器可讀結果。

- [x] 12.1 在server test mode以安全random產生Team 1 128-bit sentinel
- [x] 12.2 在server test mode以安全random產生Team 2 128-bit sentinel
- [x] 12.3 將sentinel以hash形式寫入manifest
- [x] 12.4 將Team 1 sentinel注入Team 1-only test entity tag
- [x] 12.5 將Team 2 sentinel注入Team 2-only test entity tag
- [x] 12.6 注入可掃描的test-only property pattern
- [x] 12.7 注入可掃描的test-only position pattern
- [x] 12.8 在server記錄每隊raw application payload capture
- [x] 12.9 在server記錄每隊decoded frame capture
- [x] 12.10 在runtime輸出filtered world evidence dump
- [x] 12.11 新增Windows runtime process memory dump helper
- [x] 12.12 新增Windows renderer process memory dump helper
- [x] 12.13 新增Linux process memory dump helper
- [x] 12.14 新增presentation payload capture
- [x] 12.15 新增玩家可見log sanitizer與scan輸入
- [x] 12.16 實作byte-exact sentinel scanner
- [x] 12.17 實作property與position pattern scanner
- [x] 12.18 讓scan結果記錄PID與binary SHA-256
- [x] 12.19 讓scan結果記錄dump方法與工具版本
- [x] 12.20 讓scan結果記錄false-positive排除理由
- [x] 12.21 任一dump失敗時輸出`UNVERIFIED`而非PASS
- [x] 12.22 任一對方sentinel命中時輸出FAIL

## 13. Evidence彙整與verdict

目的：把功能、安全、hash與生命週期證據整理成單一blocking結果。主要檔案：`omoba-client-runtime/src/evidence.rs`、server evidence模組、comparison helper。前置依賴：第5、12章。完成門檻：每次run有完整固定目錄與`verdict.json`。

- [x] 13.1 定義evidence `manifest.json` schema
- [x] 13.2 在manifest記錄五個可選PID欄位
- [x] 13.3 在manifest記錄binary與content hashes
- [x] 13.4 在manifest記錄rustc版本與global seed hash
- [x] 13.5 在manifest記錄ports與player/team binding
- [x] 13.6 在manifest記錄起訖tick與工具版本
- [x] 13.7 輸出server canonical timeline
- [x] 13.8 輸出每隊expected timeline
- [x] 13.9 輸出每隊filtered timeline且移除對方canonical ID
- [x] 13.10 輸出disclosure matrix
- [x] 13.11 輸出三方checkpoint hash comparison
- [x] 13.12 輸出renderer/runtime lifecycle結果
- [x] 13.13 輸出預定tick的兩隊screenshot索引
- [x] 13.14 列出所有blocking gate與individual status
- [x] 13.15 僅在所有gate通過時產生PASS
- [x] 13.16 evidence缺檔時列出缺少路徑並產生非PASS

## 14. Launcher與程序生命週期

目的：以既有入口可靠啟動三process或五process並安全清理。主要檔案：`run_2player.bat`、既有`scripts/*.ps1`。前置依賴：第2至13章production功能。完成門檻：兩種模式都能依ready marker運行，且只清理本run PID。

- [x] 14.1 在`run_2player.bat`加入headless三process模式參數
- [x] 14.2 在`run_2player.bat`加入visual五process模式參數
- [x] 14.3 檢查script DLL與三種binary freshness
- [x] 14.4 選擇未使用的authoritative server port
- [x] 14.5 選擇未使用的Team 1 presentation port
- [x] 14.6 選擇未使用的Team 2 presentation port
- [x] 14.7 啟動唯一authoritative server並保存PID/path
- [x] 14.8 等待server ready marker且設定timeout
- [x] 14.9 啟動Team 1 runtime並保存PID/path
- [x] 14.10 等待Team 1 secure/Specs ready marker
- [x] 14.11 啟動Team 2 runtime並保存PID/path
- [x] 14.12 等待Team 2 secure/Specs ready marker
- [x] 14.13 visual模式啟動Team 1 renderer-only omfx
- [x] 14.14 visual模式啟動Team 2 renderer-only omfx
- [x] 14.15 將兩個renderer視窗排列為左右顯示
- [x] 14.16 驗證所有PID的executable path與hash
- [x] 14.17 依authoritative tick送出scripted inputs
- [x] 14.18 依authoritative tick觸發同步截圖
- [x] 14.19 支援只對Team 1注入test-only fault
- [x] 14.20 關閉Team 1 renderer並確認runtime仍存活
- [x] 14.21 重啟Team 1 renderer並連回原runtime
- [x] 14.22 關閉Team 1 runtime並保持Team 2/server運作
- [x] 14.23 依序優雅關閉renderer、runtime與server
- [x] 14.24 fallback只終止PID/path都符合manifest的process
- [x] 14.25 禁止使用image-wide `taskkill`
- [x] 14.26 將`run_2player.bat`轉回UTF-8無BOM與CRLF
- [x] 14.27 執行離線comparison並回傳單一process exit code

## 15. 最後建立測試與驗收資產

目的：production實作完成後才建立測試、fixture、guard與量測腳本。主要檔案：各crate `tests/**`、既有`scripts/*.ps1`、change evidence工具。前置依賴：第1至14章全部完成。完成門檻：所有測試資產已寫好但本章不逐項執行。

- [x] 15.1 新增共用allowlist contract test
- [x] 15.2 新增局部schema set source guard test
- [x] 15.3 新增wrong team/epoch bootstrap test
- [x] 15.4 新增duplicate/gap/unknown schema fail-closed test
- [x] 15.5 新增Reveal current baseline test
- [x] 15.6 新增Forget entity與stale ID test
- [x] 15.7 新增LastKnown不更新hidden state test
- [x] 15.8 新增`global_seed + tick` deterministic RNG fixture
- [x] 15.9 新增pre-repair後才correction的fault test
- [x] 15.10 新增兩observer completion order permutation test
- [x] 15.11 新增bounded outbound queue阻塞test
- [x] 15.12 新增IPC framing/version/size limit test
- [x] 15.13 新增presentation canonical-ID absence test
- [x] 15.14 新增latest-wins與critical queue test
- [x] 15.15 新增renderer-only source guard test
- [x] 15.16 新增MoveTo ownership與hidden target test
- [x] 15.17 新增100普通單位加2英雄count fixture
- [x] 15.18 新增圓形視野與10×10 fog fixture
- [x] 15.19 新增tree circle occlusion fixture
- [x] 15.20 新增polygon directional occlusion fixture
- [x] 15.21 新增兩隊非對稱disclosure fixture
- [x] 15.22 新增packet/world/presentation sentinel test
- [x] 15.23 新增Windows runtime/renderer memory scan scenario
- [x] 15.24 新增Linux runtime memory scan scenario
- [x] 15.25 新增三方hash完整coverage test
- [x] 15.26 新增renderer restart lifecycle scenario
- [x] 15.27 新增單隊runtime failure isolation scenario
- [x] 15.28 新增三process headless orchestration scenario
- [x] 15.29 新增五process同步截圖scenario
- [x] 15.30 新增10,000 entity performance scenario
- [x] 15.31 新增30分鐘Reveal/Hide/Forget/reconnect soak scenario

## 16. 最後集中執行完整測試與檢查

目的：所有實作與測試資產完成後才一次執行完整驗收。主要檔案：全workspace與本change evidence。前置依賴：第1至15章全部完成。完成門檻：下列每項有實際command output/evidence且全部通過；任何失敗先修正，再從受影響範圍重跑，最後重跑完整gate。

- [x] 16.1 執行`cargo fmt --check`涵蓋所有受影響workspace
- [x] 16.2 執行`cargo test --manifest-path omoba-core/Cargo.toml`
- [x] 16.3 執行`cargo test --manifest-path omoba-sim/Cargo.toml --no-default-features`
- [x] 16.4 執行`cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi`
- [x] 16.5 執行`cargo test --manifest-path scripts/Cargo.toml -p base_content`
- [x] 16.6 執行`cargo test --manifest-path omb/Cargo.toml -p omobab --lib`
- [x] 16.7 執行`omoba-client-runtime`全部unit與integration tests
- [x] 16.8 建置native `omfx` executor並確認secure renderer-only dependencies
- [x] 16.9 執行所有source guard與protocol contract tests
- [x] 16.10 執行三process headless功能驗證
- [x] 16.11 檢查100個普通單位與另外兩名英雄的count evidence
- [x] 16.12 檢查MoveTo、Reveal、Hide、Forget、LastKnown timeline
- [x] 16.13 檢查tree與polygon occlusion disclosure matrix
- [x] 16.14 執行兩隊packet、world、runtime memory與log sentinel scan
- [x] 16.15 執行presentation與renderer memory sentinel scan
- [x] 16.16 執行無fault三方post-repair收斂驗證並保存pre-repair診斷
- [x] 16.17 執行Team 1 fault與server-authoritative recovery驗證
- [x] 16.18 執行renderer restart與單隊runtime failure驗證
- [x] 16.19 執行五processvisual驗證並人工檢視同步截圖
- [x] 16.20 確認兩隊在非對稱視野下畫面不相同
- [x] 16.21 在Windows Rust 1.95.0產生deterministic fixture hash
- [x] 16.22 在Linux Rust 1.95.0產生deterministic fixture hash
- [x] 16.23 比較Windows/Linux fixture hash完全一致
- [x] 16.24 執行server、兩runtime、兩observer的10,000 entity gate
- [x] 16.25 確認filtered step p99低於tick period的80%
- [x] 16.26 確認presentation頻寬與所有queue維持固定budget
- [x] 16.27 執行30分鐘soak並確認0 gap、0 unintended rebase與無持續記憶體成長
- [x] 16.28 檢查manifest、timeline、scan、hash、lifecycle與screenshots完整
- [x] 16.29 確認`verdict.json`只有在全部blocking gate通過時為PASS
- [x] 16.30 執行`openspec validate extract-client-runtime-three-process-fog-validation --strict`
