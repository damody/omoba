# 持續 buff 視覺狀態同步（2026-10-05）

## 計畫與決定

1. 使用既有 C ABI `active_buffs` 快照，不重新播放歷史 BuffAdded／BuffRefreshed，也不新增 Lua runtime 或英雄專屬分支。
2. 新 `OnBuffState` 專門同步持續呈現；通用 fallback 以 visual instance key 重用同一文字 component，更新顯示名稱／剩餘時間而不播放 add sphere。一般 BuffAdded 的一次性效果仍保留，持續標記改共用狀態入口。
3. 原生 bridge 在 actor／buff events 處理後，同步当前披露的 active state；只接受當前 live actor、正確 disclosure generation、已知 buff catalog、非零 visual key 與有限時間。同 target/key 的重複記錄只同步一次。
4. 完整 presentation snapshot 可移除已不存在的 persistent visual；control-only frame 沒有 baseline 不清除。舊 embedded 路徑無完整旗標時只同步已提供 state，移除仍由既有事件處理，不錯把缺欄位當成空集合。
5. 文字不變時不呼叫 SetText；自訂持續 component 不由通用文字 fallback 覆蓋。

## 範圍

- 不新增 buff 身分披露。正式 MOBA IPC→render data 目前沒有完整 typed per-entity buff identity baseline，不能從 raw BuffStore、敵方位置或匿名效果補資料。
- 此批只補「既有合法 active_buffs 到原生 actor」的恢復契約，不宣稱正式 MOBA 全部 buff 視覺／UI 已完成。下一步需要權威端明確決定可披露的 visual-only buff 狀態。
- 原生 UI 的完整列表退役契約、宣告式美術資產及 Blueprint 自訂效果恢復仍待後續；此批不重播 UI 或 gameplay 事件。

## 局部確認

- 強化 `Om.Generated.ActorContentRebinding` native 斷言：換類別後從 state 恢復、重複 baseline 重用同一 object、pool 重用後恢復、control-only 保留與空完整 baseline 清除。
- 首次 native 編譯 C2248：測試直接存取 protected ActiveBuffEffects。修為只在 WITH_DEV_AUTOMATION_TESTS 的唯讀 accessor，不改正式封裝。
- scoped native 修復編譯 15 actions 成功：`omfue/Saved/Logs/buff-state-modules-repair-20261005.log`。新斷言仍未執行：E224 BuildId 基線不變，不重試相同啟動失敗條件、不手改 manifest。
- OpenSpec strict 與主 repo／omfue whitespace 檢查通過。
- 不 stage DLL、不維護 omfx、不跑完整對局驗收；整體 21/31 保持。

## 防錯紀錄

見 `unreal-moba-error-register.md` E229。
