# 圓形爆發技能錯誤與決策

P01：盤點時誤猜omoba-core/src/runtime/outcome.rs存在，rg回報找不到；實際是runtime/native/comp/outcome.rs。未修改檔案；後續只從已知目錄rg --files／rg -n定位宣告，不再猜短路徑。

P02（build前自查）：新sibling測試檔不能靠point_blast::*轉出其私有use，已顯式import ABI types與ControlEffectKind，未等到編譯失敗才處理。

設計預防：新增PointCircle時，把ability_requires_movement由is_some改為明確SweptPath match，避免所有新幾何都被誤判位移／root限制。PointCircle不與executable effects雙重宣告；讀生成rank常數須checked index，present invalid extras不可fallback。新native fixture從真metadata填MaxLevel與RuntimeSettings比例；formal重用no-regen fixture，避免同tick回魔假失敗。所有技能slot動態查詢，不用歷史WERT註解推測。

本批為instant圓形傷害／控制，無persistent zone、cone、持續引導或通用process_outcomes整組回滾。失敗RErr用既有cast交易撤銷；最終實際失敗與warning會依序追加，不覆寫版本證據。
集中結果進度：base284全通過（含新增7案），WorldV1 54成功（53 Success＋1 SuccessWithWarnings）／0 Failed／0 NotRun。4個warnings均既有RuntimePresentationReset的Untitled World DestroyActor無context清理（2 route、2 fallback），controller重播非新增事件；新PointBlastMetadata無warning。UE build-v1成功UBT48.71秒，backend-v1及四Lua契約通過。
目視14圖：兩個新真SOmHudToolTip分別顯示炎刃range1100／radius100／mana100／CD10、炎襲range700／radius500／mana120／CD14；真Q／E slot與point操作文字可讀，沒有移動root限制。其餘12圖作地圖／HUD／召喚／名牌／沿路與位移提示回歸。first-kill為公告fixture，HUD部分在外；objective為條幅裁切；不當成完整HUD／真對局cast／ACK接受。

Rust最終：1950通過／10既有ignored／0失敗，含generation共12工作全部成功；本批新舊正式測試未發生失敗。source／asset baseline3397；最終source check與封存結果以final-acceptance及index為準。所有驗收腳本的舊capture flag／PNG檔名、test ID、scope均已在執行／封存前讀回校正，不以全域替換後的名稱猜實際輸出。
