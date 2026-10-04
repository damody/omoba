# Runtime 重連輸入 ID 恢復（2026-10-04）

## 本輪計畫與決定

1. 先處理會阻擋 shop admission journal 的 ID 重用，再補 terminal receipt 重送。商店 secure gate 保持關閉。
2. `InputBuffer` 保存每位玩家、整場對局的 `last_seen_input_id`；合法進入緩衝區的命令（含晚到拒絕）提高 floor，不因 drain、eviction、runtime 正常斷線而清除。零玩家與零 ID 不建立界線；商店形狀／衝突／過期檢查維持既有拒絕規則。這不是 gameplay 狀態，也不是交易成功記錄。
3. `TeamGameStart` tags 21／22 回傳 `input_allocator_version=1` 與 `last_seen_input_id`。正式 server 的 cached／empty bootstrap 兩條路徑都填入；通用 projector 仍為 0，只有 transport 可以宣告 allocator agreement。新版 selective client 拒絕缺少／未知版本，不能默默對舊 server 使用 1。
4. `InputBridge::resume_after` 在 runtime 啟動時使用 server floor；分配採 `checked_add`，耗盡回覆 `INPUT_ID_EXHAUSTED`，重複呼叫仍拒絕、不繞回、不 panic。Renderer 的不合法輸入仍不消耗 ID。
5. 不修改腳本 ABI、C ABI 5 或 Unreal C++；protobuf fallback 由既有 vendored protoc 流程生成，Fyrox 共用協議編譯驗證。沒有新增 Blueprint。

## 驗收

- core `cargo test --manifest-path omoba-core/Cargo.toml --features runtime-lua-content --lib`：320 passed。
- server 全 lib：148 passed／1 ignored；包含新 admission floor 與 bootstrap protobuf roundtrip，覆蓋 player 7／team-independent player 2、out-of-order ID、late rejection、drain／eviction、u32::MAX。
- runtime：46 lib＋3 bin passed；額外 targeted input_bridge 4/4，涵蓋 resume=42 後 43、MAX-1 後 MAX、永久耗盡與合法 renderer 請求拒絕。
- bridge：44 passed／1 real-capture opt-in ignored。
- Fyrox `cargo check --manifest-path omfx/Cargo.toml --tests` 通過。
- targeted rustfmt 與 scoped diff whitespace check 通過。既有 dead-code／unused-world／protoc fallback 與 CRLF 提示仍存在，不宣稱零警告。

### 真實 KCP 正常關閉後重連

重現：設定 `OMOBA_RUNTIME_RECONNECT_SMOKE=1`，由固定 `tools/lua/lua.exe scripts/run_moba_runtime_smoke.lua` 執行。使用實際 debug DLL、server、兩個 filtered runtimes；先讓雙隊移動並產生 checkpoint，再透過既有 shutdown signal／SESSION_CLOSE 停止玩家 1，等待 server cleanup，於同一 server 對局啟動新 runtime。新證據使用獨立 reconnect 目錄，不覆蓋舊 runtime capture。

成功 run：`target/interactive-runs/moba-runtime-1791047918/moba-runtime-smoke-report.json`。

- `success=true`、`cleanup_verified=true`。
- 原 runtime input ID=1；新握手 floor=1，新 input ID=2，後续實際持續送出 3 至 7。
- 恢復 runtime 的移動原點 raw=(921600,716800)，最後 raw=(895998,718132)，正式 RendererInput／KCP／權威 tick 已實際改變位置。
- 雙隊完整三方 pre/post hash：14／12 個 PASS，最後檢查 tick 1680；其中玩家 1 重連後 4 個檢查點 PASS，零 FAIL／無 repair。
- report 恢復觀察 tick=1739，關閉前 move evidence 到 tick 1869。server 持續運行，未因 runtime 重啟而重新建立對局。
- 程序 server 90336、舊 runtime 81136、隊伍 2 runtime 67772、新 runtime 93688 均退出，另以程序查詢核對。
- 本輪未重建或重啟 Unreal，未執行新的 PIE／實際 UE 雙視窗驗收；Unreal 的 IPC／C ABI shape 不變，不能將前輪 UE 證據算本輪。

## 失敗與限制

首次 run 1791047611 hard-stop 後等不到 session cleanup；第二次 run 1791047805 已恢復 floor／送出新 ID，但再次移動到已站立的位置，沒有位移證據。兩次報告保留且程序清理成功；細節集中於 error register E091。不提高 timeout、不直接改 roster、不捏造 movement。

這只驗收正常關閉後重啟，不代表突然 crash／斷網會即時釋放 session。未到達 server 的封包不算已見；沒有持久化到 server 重啟、沒有自動恢復 renderer pending request、沒有交易 exactly-once。新 runtime 仍需由既有 roster 允許加入；不放寬同玩家多 session 規則。

## 下一步（依序）

1. authority 保存真實 terminal shop receipt，完全相同重送只回原結果，不重新交易；明確定義 pending／expired／conflict，不能把 DuplicateShop 當成交。
2. 把 terminal replay 接到 KCP／runtime 自己的 presentation ledger，驗證丟失回覆後重送、正常 runtime 重連與六格／Gold 一致。
3. 才開放 secure buy／sell RendererIntent，接原生 UE 商店操作與實際成功／失敗／非空裝備驗收。
4. 補金錢收入、擊殺／助攻與回城，再擴展三路與完整 UI。

OpenSpec 仍為 17/30；4.1／5.3／6.2 不勾選完成。
