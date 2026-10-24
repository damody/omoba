# 控制效果施法提示錯誤與決策

- P01：PowerShell不展開傳給rg的路徑glob，出現Windows os error 123；改用已確認目錄及rg的-g選項，不再以shell glob猜路徑。
- 防錯：公開Buff清單UI最多64項，不能以被截斷清單判定沒有控制效果；由同一份公開frame有界掃描完整active_buffs，查詢時O(1)。
- 正式技能使用BuffStore::is_silenced，包含silence與stun，root僅禁移動；提示遵循此規則，不改權威admission。
- OpenSpec CLI仍未安裝，依既有spec-driven文件實作，不冒稱CLI驗證成功。沿正式-NoUBTMakefiles與測試限定FEditorScriptExecutionGuard。

- P02：盤點時猜測WorldEntityKey獨立header不存在，已改由Source目錄搜尋實際宣告。另確認BuffIdName對未知ID會回傳buff_N而非None；新摘要直接使用typed catalog Entry，避免誤判無控制。測試改含實際宣告函式的OmRegistry.h，均於啟動建置前修正。

- P03：Saved封存helper批次改名會連帶改到測試CPP基名；啟動封存前逐項核對required路徑，已修正為OmCastControlFeedbackTests.cpp，不更動正式來源。
