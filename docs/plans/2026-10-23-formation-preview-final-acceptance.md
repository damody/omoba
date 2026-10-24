# 自身朝向陣形施放提示：最終驗收

完成complete-facing-formation-preview。新增明確facing_formation作者契約與標準逐rank數量／行列間距／前距欄位，特殊handler幾何不由英雄名稱、cue形狀或碰巧有extras推斷。generated metadata經bridge縮放為typed UI spec；owned overlay公開目前有效朝向，瞄準繪製最多128個請求位置，tooltip顯示排／列與地形／友軍佔位調整規則。保留原point指令、距離、mana/CD/control與取消規則。

## 驗收

- Rust **1973通過／10既有ignored／零失敗**，新增共享schema與產生器契約測試；四Lua契約、正式backend及UE5.8建置成功。
- World **60成功／零Failed／NotRun**，含新增FacingFormationPreview：四rank、非指定技能不推斷、旋轉、非法資料／未知朝向、隱藏／重現與退役清除、真tooltip及Slate圖像。4既有RuntimePresentationReset warning保留。
- **3411來源／資產**集合、SHA、size、mtime前後一致；7 staged DLL與import library核對成功。
- 封存 **177份證據**，獨立逐byte／SHA校驗；程序退出後檢查 **21張PNG**。新增圖為使用正式幾何函式的正交意圖圖＋真tooltip，不能當作實際3D遊戲viewport或地形調整後出生點的畫面證明。
- 健康採樣12次、最大間隔172秒；最終及歷史worker／watcher與health程序均退休。

V1全部21圖曾逐張目視；V2其中18張與V1逐位元相同，沿用已目視像素，另3張有變化的V2圖重新目視。每圖review_source與SHA證據另驗。V1測試全成功後才由架構複核發現rows傳遞缺口；等待所有reader退休後補AbilityConst／executor／reload guard，再執行V2，保留兩輪證據。當前兩排技能平衡不變，四排與reload拒絕另有測試。

## 範圍與接續

預覽是目前公開朝向的請求陣形，伺服器可因地形／佔位與釋放時狀態調整。既有first-kill局部HUD裁切、objective窄strip與英雄／焦點交疊仍有限制。錯誤根因／修正／預防見errors.md及E1501。

48h omoba-48沿用期限2026-10-25 00:33:38 Taipei。下一個具體單機缺口：排數已釘住，但既有runtime數值調參與build-time native metadata的rank數量／間距／前距一致性仍需共用政策與guard；不宣稱本批已同步全部幾何數值熱重載。整組召喚settlement、持續引導／區域、其他特殊幾何、combat replica生命期、內容／美術／音效及11外部接受仍pending；MVP33/44不變。未新增真網路對局、quota reset、commit/push或omfx維護。

證據：`../acceptance/2026-10-23-formation-preview/index.json`。
