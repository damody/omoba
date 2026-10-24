# 技能預覽與熱重載幾何一致性：最終驗收

完成complete-aim-preview-reload-coherence。共用內容投影同時供Unreal產生器與compiled/runtime簽章使用，涵蓋作者幾何kind/key、target/action語意、每rank射程、有效圓形範圍、衝刺寬度、陣形排數／數量／間距／前距與movement語意。移除產生器重複解讀邏輯。啟動及reload拒絕幾何不一致，要求重建所有端；不把damage／duration／cosmetic等非幾何值一律鎖死。

## 驗收

- Rust **1976通過／10既有ignored／零失敗**；新增共用投影測試、8種正式內容幾何變更矩陣及實際reload交易測試。錯誤reload後generation/hash與active資料不變，隨後damage調參正常接受。
- 四Lua契約、正式backend與UE5.8建置成功；World60成功、零Failed／NotRun，4既有RuntimePresentationReset warning保留。
- **3411來源／資產**集合、SHA、size、mtime一致；7 staged DLL與import library核對。
- **112份證據**獨立逐byte／SHA驗證；21張PNG沿已接受像素比對與差異目視，逐圖review_source/SHA可追溯。原生fixture／意圖圖不替代3D最終出生點、真ACK或實際OS DPI接受。
- 健康採樣7次，最大間隔171秒；全部worker／watcher及health程序退休。

## 範圍與接續

本批關閉幾何數值reload一致性缺口，並非把全部非幾何UI metadata／敘述同步重做。錯誤根因／預防見errors.md與E1502。既有first-kill局部HUD裁切、objective窄strip及英雄／焦點名牌限制仍記錄。

48h omoba-48沿用期限2026-10-25 00:33:38 Taipei。接續優先盤點現有authored技能可補的持續區域／引導與實際玩家可見玩法；整組召喚settlement、combat replica生命期、內容／美術／音效及11外部接受仍pending，MVP33/44不變。沒有新增真網路對局、quota reset、commit/push或omfx維護。

證據：`../acceptance/2026-10-23-aim-coherence/index.json`。
