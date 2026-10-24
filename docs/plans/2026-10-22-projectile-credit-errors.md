# 投射物來源歸因錯誤紀錄

- P01：現有projectile僅保留owner Entity，退休後Faction／PlayerOwner消失，擊殺歸因遺失。以每發快照修正，不能暫留死實體或以id查新generation。
- P02：OpenSpec CLI前輪已確認不可用；按技能維護proposal/design/tasks，使用者已授權自主決策。

## P03：工具編輯不可巢狀使用相同 PowerShell here-string 分隔符
首次批次編輯未執行；已改用逐步 apply_patch，讀回確認三個實際工廠皆完成接線。


## P04：不要假設 state initializer 的來源路徑
檢視時引用不存在的 native/state_initializer.rs，rg 回報 os error 2；改以 rg --files 定位，這次檢查直接讀 setup_single_lane_match 的正式入口，確認新 match 要求空 world。不得把推測路徑當作已讀證據。

