# 塔皮呈現錯誤紀錄

沿用前批worker／watcher退休後才重試，consumer尚活著不修改production／test。

- U01：初寫frame key時誤用EntityId／EntityGen，來源檢查發現FOmWorldEntityKey實際是Id／Gen，於建置前修正。複製相鄰型別欄位前先讀定義。
- U02：不能用目前HP比例推算剩餘塔皮，否則治療會錯誤恢復已領獎層數。改用獨立權威status metadata，與護甲Buff分離，包含完整未受損外塔。
- U03：V1 UE測試使用不存在的EOmNameplateRelation::Hostile，造成C2838／C2065。等待整批consumer退休後，依實際enum改為Enemy；不猜測跨型別列舉名稱。保留V1 build log與receipt。
- U04：高負載檢查發現預建16格會讓普通名牌也付出配置成本。改為首次有效分段時才建立，後續快取沿用；新增原生配置數量與重複Apply不重建的回歸。與U03一併於前批完全退休後實作。
- U05：名牌只顯示整秒，不能因權威每frame的小數秒變化重寫Slate。以相同ceil顯示秒數作快取簽章，仍由權威snapshot清除到期狀態，不自行倒數。
- U06：查FrameContract時誤猜Private/OmFrameContract.cpp。改用rg --files定位header／實作；先列檔案再讀，不依類型名稱猜路徑。
