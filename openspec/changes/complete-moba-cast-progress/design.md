# 設計

前搖表記錄每英雄一筆明確Preparing／Canceled／Released狀態及instance，無目標／位置披露。PostStep沿既有Team HUD metrics輸出slot、state、remaining、total、instance，沒有活英雄／結算時輸出零。正式client-runtime依owner與team嚴格驗證完整tuple並形成typed optional AbilityCastPresentation，proto及checked-in generated fallback保持一致。非Unreal歷史前端不維護。

Rust bridge及C header同步擴充OmAbilityHud，ABI版本遞增；Unreal由owner ability payload驅動每slot進度與提示，取消／釋放／退休／重生清理，進度只採權威值不自行完成施法。資料／UI接線完成後集中驗收。
