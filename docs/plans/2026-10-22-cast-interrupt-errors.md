# 2026-10-22 施法受傷中斷錯誤紀錄

- P01：盤點猜測templates/skills.lua不存在；無來源變更，改由rg --files定位abilities.lua。之後以真實路徑／既有呼叫點操作，不以名稱猜路徑。
- 工具：openspec CLI未安裝，沿既有手動spec-driven artifact流程執行，直接依使用者授權決策，不要求確認。
- 防錯：JSON bool不做字串／數字轉型；傷害政策判斷放在shield之後，沿既有cancel，不把命中／護盾消耗冒稱HP損失。已release的交易不回滾。
- P02：core測試草稿使用尚未核對的digest函式名；啟動consumer前用rg定位single_lane_replay_digest並修正，未造成編譯失敗。新增API呼叫先查宣告，避免憑名稱猜測。
- P03：原生tooltip測試初稿附加在既有WITH_DEV_AUTOMATION_TESTS區塊之外；建置前檢查#if／#endif並移回guard內，避免非測試設定暴露無定義型別。
- T01：WorldV1 CastDamageInterruptionTooltip fixture漏設bTargetShapeAvailable與Point，使既有OmAbilityTargetGuidance的未知形狀守門隱藏整段提示。metadata四rank與正式60Hz政策均通過；這是fixture前提缺漏，未修改production放寬守門。等待V1全部consumer／watcher／sourceguard退休後，補合法形狀及未知／未學習負例，重新UE／World及來源guard；RustV1有效結果不重跑或加總。
