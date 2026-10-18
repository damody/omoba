# 正式 bridge 道路比例與單位綁定

自行實作。已接受世界表面及單位視覺比例，但正式道路仍使用32cm，而hero高度12000cm；原route hash也未包含WorldUnitsToCm，尺度變更可能重用舊座標。

1. 新增明確MapRouteWidthWorldUnits（140遊戲單位正式預設）；0明確選擇既有MapRouteWidthCm，保留作者cm設定。finite／上限／轉換後寬均驗證。
2. route快取納入world scale，寬／offset／material變更亦重建，沒有變更則沿用components。只沿公開points，無碰撞或導航；不把裝飾寬宣稱權威走廊。
3. 以真正ProcessSyntheticFrameForAutomation進入bridge，驗生成英雄實際mesh高度、原世界位置與actor root、pool重用、公開route寬度／座標／join／control frame保留／empty清理。可變尺度及cm相容用測試子類明確配置，避免改使用者設定檔。
4. ordinary nocache build、精確受影響native與當前world camera原圖驗證。完整三路／river／jungle／塔基地與最終驗收仍待後續。


2026-10-12 正式bridge道路與單位尺度當前範圍接受：`.ai-collab/world-route-scale-accepted-v1/summary.json`。ordinary nocache build-v3成功10.04秒，fresh native-v2 exact24 Success／零warning；兩張當輪original原圖直接目視，正式bridge產生的石道路寬140world units，英雄與轉角接合存在。真正publication測試涵蓋native初始化／高度與世界位置、WorldUnitsToCm切換、cm相容模式、寬變更、相同幾何component重用、control保留、complete empty清理及pool reuse。原world／動畫／材質packages SHA／mtime再驗不變，stage contract通過。E892／E893保留；此為synthetic公開frame與當前呈現整合，不是完整三路／river／jungle／塔基地或完整對局驗收。父項12/44、6.1／6.2／6.3保持pending，無authority game／ACK。
