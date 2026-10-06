# 選角完成期限：共用取消與交接邊界

## 問題與決定

原有單人與多人選角只限制服務／renderer handshake 的 45 秒。UI 就緒後若不產生 finalized artifact，可以無限等待；`--finish-timeout-seconds` 只在遊戲啟動後生效，無法限制卡住的自動選角。這是流程缺口，不應以自動選英雄、鎖定或啟動對局掩蓋。

新增 `--selection-timeout-seconds 1..7200`，必須與 `--interactive-selection` 使用。人工選角未設定時維持不限時；既有 `--selection-smoke-hero` 自動選角預設 120 秒，可明確覆寫。共用 `moba_selection_deadline` 使用一次 session deadline，從選角程序開始到 final handoff，不每位玩家／每輪 poll 重設。多人服務與所有選角視窗共享一個 deadline；終局等待各 PID 退出時也只能使用剩餘預算。

逾時只沿既有 cleanup 清理本次 owned PIDs、保存 errors.md，不修改配方／CLI choice、不啟動 gameplay。原 readiness45秒、terminal receipt／host catalog／roster／player綁定檢查全部保持。此 Lua 是開局工具，不是遊戲內 Lua runtime。

## 當前功能確認

- 純期限測試通過：manual/default automation/explicit override、deadline boundary、wait 剩餘 clamp、非法／非互動選角 options。
- 單人 mock 選角 7/7；新增 UI ready 卻不 final 的 1 秒案例，精確逾時並清理，沒有 match output。
- 多人 mock 選角 14/14；新增 shared UI 全 ready 卻不 final 的期限案例，全 owned host/renderer 清理，不合成 consent 或最後配方。
- 未啟動真實 Unreal／server／runtime，未跑模擬或完整驗收，不勾整項6.2。

## 共用工作流時鐘補強

`tools/lua/lib/time.lua` 舊 `monotonic_ms` 實際為 os.time()*1000，UTC校時會改deadline長度。現由lua-host `monotonic_ms` operation返回Windows GetTickCount64的機器uptime，跨helper程序使用同一clock domain，Lua拒非法／倒退回傳。UTC timestamp仍用原os.date；native gameplay clock、tick/hash不改。GetTickCount64是粗粒度wall-elapsed而不是CPU時間，包含休眠時間，不用來計perf phase。

目前正式平台Windows以既有windows-sys增加SystemInformation feature，沒有system install；非Windows明確unsupported，沒有UTC／shellfallback。每次clock query有helper程序成本，不能把這次工具時鐘修正說成client step尖峰修復。

Rust精準clock測試1/1、固定Lua注入非法／倒退／相同讀值與真跨helper sleep40ms時鐘前進檢查通過。共用期限pure、單人7/shared14再次確認通過（非第二次遊戲場次）；未做完整suite。

## 並行工作

Grok Build job `run-muw9jh35-ydrn8m`／thread `98965d83-a59a-428b-85fa-5d8b9be25f39` 獨立負責 Rust replica step 的有界分段診斷，固定原 OM_PERF 與門檻。不與本批 Lua 檔案重疊；主 agent 後續必須審查 diff 與局部確認才接受，不能以 Grok 摘要算完成。
