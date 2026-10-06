# fixed_step 內部分段與同一步 CPU（2026-10-06）

## 實作與審查

主 agent 沿現有 Specs step 實作 opt-in preparation／18 production gameplay phase／finalization 固定陣列計時。Trait 新 hook 有預設相容實作；plain 路徑不查新 CPU／clock。Detail 直接隨成功 apply profile 傳回，不在 stepper/world 保留，error／stall／duplicate 不留 stale。世界／hash／wire／ABI／Lua native content 執行不變。

60-success-sample ReplicaStageWindow 同一 outer strict maximum 保留完整 detail，tie 保留第一次；checked reconciliation 拒絕 phase sum 超 fixed_step、溢位，不推進窗口。reset 全清。舊 record API 與 OM_REPLICA_STAGE v1 keys 不變；正式 main 在 outer 計時停止後呼叫 record_linked 並分開 log。

Grok CPU 子任務 run-muwc1hp7-rqf7ez，thread d2b45c92-3a9e-4362-a80a-3cde3ccbb81f，4m55s terminal completed。只寫 process_cpu.rs 與其 progress；主 agent 審查 FFI／checked arithmetic／平台 gate 並加入 comp module。原始 child/pseudo-handle 不 close，不操作其他程序。

## OM_FIXED_STEP v1 契約

Flat JSON，component=client_runtime，metric=fixed_step_detail，unit=ns，scope=fixed_step，phases_are=same_outer_slowest_sample，samples=60。player_id／team_id／replica_tick／team_sequence／outer_ns／fixed_step_ns 必須與相鄰 OM_REPLICA_STAGE 窗口完全相同。

Wall fields：preparation_ns、18 個既有 phase 的 snake_case `_ns`、finalization_ns、residual_ns。18 個 phase 的 mapping 由既有 enum exhaustive match，執行順序仍唯一 DETERMINISTIC_GAMEPLAY_PHASES table；不是另一套 gameplay scheduler。wall sum + residual = fixed_step_ns。

process_cpu_ns 是 optional integer 或明確 JSON null；cpu_scope=all_process_threads_kernel_plus_user，cpu_not=wait_time_or_phase_attribution。CPU > wall 合法，不能加進 wall sum、不能相減宣稱等待。查詢失敗／counter regression／non-Windows unavailable；Windows counter 粒度 100 ns，不保證短 step 產生非零 CPU delta。

Lua 入口：`tools/lua/lua.exe scripts/collect_fixed_step.lua RUNTIME_LOG PLAYER_ID TEAM_ID`。有界 8192 chunk／65536 line，共用 scanner；只保留 pending stage／last／whole peak，不存歷史。flat canonical keys、duplicate／escaped／unknown／invalid 整數、未配對／錯身分／倒退／重複 timeline、讀錯／截斷 fail closed。輸出 stdout JSON，不修改原 log／原門檻。旧只有 OM_REPLICA_STAGE 的 log 不會被冒充完整新診斷。

## 已執行的局部確認

- Core replica_stage filter：9 passed、485 filtered。
- Core fixed_step_detail filter：2 passed、492 filtered，含真 Specs plain/profile 同 disclosed world 與同 phase trace。
- Core comp::process_cpu::tests：7 passed、487 filtered，含 Windows current process readonly API。
- Runtime compiled-content-only binary cargo check exit0。
- 固定 Lua 舊 stage collector：11/11；新 linked collector：7/7，含長 stream 1000 窗口純資料 fixture，不是遊戲場次。
- 已有 td_rounds 三項 dead_code warnings 保留，沒有新增警告；不把 warning 當不存在。

新 fixture import 曾缺 DisclosedWorldStepper／DisclosedReplicaWorld／StepInjections 與 specs::WorldExt，編譯失敗後補確切 import 才成功。Lua nil and/or fixture 已修正，詳 E324。沒有跳過測試、提高門檻或刪舊失敗證據。

## 委派用量與限制

Completed job reported USD0.13307872；input83684／cached181248／full input264932／output22236／reasoning18631／total287168，7 calls／7 turns，API duration unavailable。先前廣範圍 run-muwbt1eq-sk1khc 在 6m13s 無 code delta 後主 agent 取消，cost unavailable，不算 0、不接受為完工。

## 真實短程配對結果

`fixed-step-detail-20261006-v1` 是唯一這批遊戲啟動，120 秒上限、release／compiled-content-only／single_lane／60Hz，launcher exit0、雙隊 movement／owner HUD／consumed 與 cleanup_verified 成功。主 agent 獨立 strict owned_alive 驗五個原始 creation-token lifetime 都已退出、active session 檔已退役。不是 renderer reconnect／自然終局／全部功能驗收。

Lua collector 對真正 Rust log 成功配對兩隊各 50 完整窗口／3000 成功 samples；新產物 `target/interactive-runs/fixed-step-detail-20261006-v1/linked-fixed-step-diagnostic.json`，未覆寫原始 log 或先前失敗證據。

| 同一 outer peak | p1 tick1367 | p2 tick1353 |
|---|---:|---:|
| outer wall ms | 93.7459 | 102.1183 |
| fixed_step wall ms | 93.6647 | 102.0598 |
| dispatcher wall ms | 93.0694 | 101.7626 |
| preparation wall ms | 0.2024 | 0.0963 |
| finalize wall ms | 0.3189 | 0.1659 |
| process CPU delta ns | 0 | 0 |

兩隊最大 wall bucket 是 dispatcher，而不是 script dispatch／內容生成／JSON／repair／hash。CPU 查詢成功但兩次計數未觀察到增量，不能把 0 當 unavailable，也不能推論精確等待時間或 OS 根因。仍超固定 client50ms；與前一次不同 tick 的351/475ms都保留，不挑樣本當效能通過。下一個修正範圍限 dispatcher／它執行的系統與排程，先區分內部 jobs／等待，不能只憑 wall 改 worker／priority。

對實際依賴 shred0.16.1 的只讀核對：dispatch_par 明確會阻塞執行者；dispatch_seq 不跑 thread-local，需要另調 dispatch_thread_local；多個 gameplay systems 本身仍使用 par_join。直接替換成 dispatch_seq 不是保證 inline／不等待的通用解，可能把 nested join 送到 global pool，不能草率替換當作已修。

這批診斷功能已實作並真路徑確認，不宣稱 50 ms 或全框架完成；OpenSpec 25/31 保持。未跑 BP／PIE／全場 batch，未維護 omfx／改引擎／commit／push。

最終檢查 `git diff --check` exit0（正常 LF→CRLF 提示保留）。CPU zero-total fixture修後獨立 current_process_query_readonly_sanity 1 passed、493 filtered。工作期間有外部 damody提交（15:03:24）使HEAD由c5211e83變ca34c786，已讀log／stat並保留；本agent未commit／push，baseline只用作歷史起點，沒有reset他人更新。詳E324。
