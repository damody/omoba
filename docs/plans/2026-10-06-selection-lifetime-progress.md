# 選角與正式對局：原始 child ownership

## 計畫與完成的決定

1. 找到單人選角、共享選角與 role 對局 launcher 的 PID-only 啟動／存活／等待／清理，統一沿固定 Lua process API 的 original Child 記錄，不另造程序管理框架。
2. `wait_owned` 與 `poll_owned_ready` 使用 creation_token＋canonical executable＋PID；有界 monotonic 等待，查詢錯誤不假裝退出。readiness predicate 前後都核對原 lifetime，不能用重用 PID 的新程序或舊 log 取得 ready。
3. 啟動後立即持久保存原始身分，後續不重新捕捉 token；單人、共享 host／renderer 與 server／runtime／UE 全部使用同一 API。保留原 ready／finalized／receipt／deadline／取消／無自動 lock 規則；缺 token 拒絕，不回退 legacy API。
4. 原 child 退出後同 PID 被另一程序重用，視為原 child 已退役：不等待／停止新程序。host/runtime 必須仍是原 child 才能通過存活檢查；失敗逆序清理已擁有的其他 child，並保存 errors.md。

## Grok 與責任界線

Grok `run-muwd53b1-fuex10`／thread `7a7bb9d1-20c9-470c-8350-ef4e1c150b62` 在讀來源後 7m15s 未產生修改；primary 取消。follower terminal cancelled，metadata pid／agentPid／bridgePid 全 null。stop exit128 表示追蹤 PID 已消失，不能當代理仍執行。partial metadata 有 ueCP 127.0.0.1:30000 handshake connection refused／transport warning，沒有足夠證據把它斷言為全部等待原因。reported cost/token metrics 未提供，保持 unknown，不填 0。沒有為純 Lua 作業啟動 Editor、掃埠、改全域 MCP／認證或 bridge 安裝。

primary 在確認該 writer 終結後完成全部接線與獨立測試；本段不是 Grok 交付。上一段 Nearby 的成功 Grok job 與成本另見 dispatcher-detail-progress，不混計。

## 實際局部確認

- `scripts/tests/process_owned_wait_test.lua` 10/10：退出、PID重用、0／73ms／預設期限、查詢／legacy失敗、clock regression、ready前後原 lifetime；predicate 返回 true 但已超原期限仍拒絕，不能靠讀取延遲绕過 budget。
- `scripts/test_moba_hero_selection.lua` 9/9：原7項，另 valid handoff／cancel 在原renderer退出且PID重用後不停止新程序，持久 token 不變。
- `scripts/test_moba_shared_selection.lua` 17/17：原14項，另 partial spawn、reused host、reused renderer，其他 child 清理繼續。
- `scripts/test_moba_role_launch.lua` 8/8：原7項，另4種 original server/runtime/renderer PID重用或partial spawn情境。存活／ready不接受新程序，reverse cleanup／錯誤傳遞保持。此檔含既有 Cargo moba-config 配置 preflight；它不是全純 mock，但沒有啟動 server/runtime/Unreal 或跑場次。
- 生產兩個模組已無 `process.spawn/inspect/stop/wait/poll_ready` 的 PID-only 呼叫。`git diff --check` exit0（既有 LF→CRLF warnings 明示），HEAD 保持 ca34c786。

測試 adapter 只在 scripts/tests，提供注入式原／新 lifetime 狀態，production 不依賴它。沒有新增 Lua gameplay runtime、C++／Blueprint graph 或 omfx 功能。沒有再跑對局採樣／完整验收，沒有 commit／push。

## 尚未封關

OpenSpec25/31保持；本次是啟動／退役功能完成，不代表實體選角至自然結算、全部renderer效果或雙機LAN已驗收。完整驗收集中到最後；LAN仍缺第二台實機，不能用本機雙程序冒充。效能最新局部樣本 client 30.7792/24.8499ms，保留歷史失敗與固定十二門檻，不宣稱永久根因修復。
