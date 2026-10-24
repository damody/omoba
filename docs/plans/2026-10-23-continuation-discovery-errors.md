# 後續盤點的查找錯誤

自用淨化批次封存期間，唯讀查找誤用不存在的 `OmHealthNameplateWidget.cpp` 與 OmRuntime 目錄下的 `OmWorldBridgeActor.cpp`。實際檔案為 `OmRuntime/Private/OmUnitNameplateWidget.cpp` 與 `OmGenerated/Private/OmWorldBridgeActor.cpp`。兩次 rg 都只回報找不到檔案，未修改來源，也未啟動其他建置。

防止重犯：先以 `rg --files omfue/Plugins/OmRuntime/Source` 取得實際路徑，再搜尋內容；生成 Actor 的正式修改入口另從 codegen 實際來源追查，不直接假定生成檔是手寫檔。既有英雄保留區允許英雄互疊及焦點例外，後續修正必須保留英雄可讀性、量測與有界熱路徑，不能只取消例外而把英雄名牌隱藏。

淨化批次已完成封存與獨立校驗，這份後續唯讀紀錄不改寫其封存錯誤證據。尚未開始新的名牌實作，也不宣稱重疊已修復。
