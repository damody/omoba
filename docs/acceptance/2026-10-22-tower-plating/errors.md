# 塔皮錯誤與設計修正

目前沒有建置／測試失敗。沿用前批worker／watcher完全退休才重試的守門。

- P01：實作檢查發現跨多層一次合併pool後均分，會比逐層捨餘多領金額。於建置前改成每層先floor再乘層數，回歸比較分開與合併傷害。
- P02：強制退役可能在entity尚存活時清Buff，下一tick又被ledger重加。同步額外檢查仍在state.units且未last-hit退休，避免只檢查ECS alive／HP。
- P03：查魔抗定義時誤猜stat_key.rs、units.lua、hero.rs路徑。改用已定位的unit_stats.rs與全目錄符號搜尋；不得猜測模組檔名。
- P04：防禦路徑檢查發現前輪將UnitStats的normalized魔抗直接交給DamageInstance百分比結算，會縮小100倍。加入明確percent→normalized→percent邊界，驗證25%基礎保留、與20%額外魔抗組合約40%、實際100魔傷剩約60；不混用UI浮點換算。
