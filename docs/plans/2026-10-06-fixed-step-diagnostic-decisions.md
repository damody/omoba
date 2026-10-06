# 固定步進尖峰：本批決策與實作邊界

## 目標與計畫

OpenSpec build-unreal-rust-moba-framework 仍為 25/31。先處理 6.5 的 client 固定步進尖峰；不重跑完整 UI／BP／PIE／對局驗收。50 ms 門檻、既有失敗資料及其他十二項門檻不改。

1. Grok Build 以 fresh、有追蹤的單一 writer 補 fixed_step 內部分段與同次程序 CPU 差值。
2. 主 agent 審查實際 diff、接線與針對性測試；診斷契約確定後補唯讀 Lua 收集器。
3. 依證據選根因修正，不以 worker hint、任意 timeout 或排除冷啟動當作修復。正式效能驗收與 LAN 跨實機仍未完成。

## 問題、決定與理由

- 最近真實採樣兩隊 outer 351.1038 / 475.328 ms，fixed_step 351.065 / 475.2844 ms，同為 tick / sequence 227。現有 wall time 不能區別 step 內系統成本或排程等待，先細分 preparation、既有 18 phase 與 finalize，沿唯一 production phase table，不建立第二套玩法順序。
- 程序 CPU 是全部執行緒的累積 CPU；平行時可高於 wall time。只並列同一步差值，不能用 wall minus CPU 宣稱精確等待；查詢失敗／計數倒退／平台不支援明確 unavailable，不補 0。依據：[Microsoft GetProcessTimes](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes)。
- 舊 OM_REPLICA_STAGE v1 為嚴格 flat schema；直接添欄位會破壞 collector。保留舊記錄，另輸出 linked OM_FIXED_STEP，與同一 60 筆窗口的 first strict outer maximum 全筆相連，不各自挑 phase／CPU 最大值。
- 新資訊只在 opt-in profiler 路徑，固定容量、不存逐 tick 歷史；不進 world/hash/ABI/wire/content。格式化在原 outer 計時之外。
- 最近 dispatcher reconfigure 修正是 API correctness，正式 game 沒有呼叫該 API；不列為已證明的尖峰原因。
- 舊大段歷史 task 容易讓代理誤接舊任務；本次用獨立 compact packet 與 fresh thread，保留原 dirty。未提交、推送、改引擎或操作其他專案。

## 委派與狀態

Job run-muwbt1eq-sk1khc，thread 2e98ff85-4aad-4d1d-9640-869df265c7a5。本節啟動時為 running，尚未接受，terminal 後由主 agent 審查補記。代理只處理限定 Rust／自己的 progress MD，主 agent 處理 Lua／中央計畫／錯誤紀錄。

後續：該廣範圍 job 在6m13s沒有code delta，由主 agent取消、確認tracked handles null，cost未知，不接受。主 agent接手接線，小CPU模組job run-muwc1hp7-rqf7ez completed4m55s／reportedUSD0.13307872，primary实际FFI審查＋獨立7 tests接受；其餘stage9／detail2／Lua11+7／compiled-onlycheck与release build通過。一次real60Hz雙UE paired50windows/3000samples與五原始lifetime清理另驗成功，dispatcher wall93/101ms仍超50ms，不勾6.5。詳fixed-step-detail-progress／E324。

## 證據限制

局部測試通過只證明診斷功能可用，不等於 client 已低於 50 ms。正式 60 Hz 失敗資料不覆寫；原始短程採樣是各 59 完整窗口／3540 成功 sample，不包括最後不足 60 筆窗口。
