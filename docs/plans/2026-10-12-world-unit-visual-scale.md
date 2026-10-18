# 世界單位與角色顯示比例

Codex 自行實作，不使用子代理。WorldUnitsToCm=100，英雄來源約180–240cm，現有出生鏡頭900×WorldUnitsToCm，造成角色在世界視點幾乎不可見。保留後端距離、碰撞、公開座標、匯入骨架及原始動畫，不以修改來源root或全域座標比例解決。

1. 為單位提供明確可覆寫的目標顯示高度（game world units），預設hero120、creep70、tower240、summon70、ward48；projectile／fx不依身體高度放大。
2. 在正式bridge單位綁定建立／重用時依目標高度與實際mesh bounds套用visual component比例，保留作者原本非等比比例與relative offset。統一native skeletal、Blueprint mesh與fallback，重複呼叫／pool／比例切換不可累乘。
3. 初始化native mesh與套用比例的順序要明確，晚載入mesh也能套用既有世界尺度。無效設定拒絕、上限有界；不增加per entity per frame bounds量測。
4. 裝飾VFX尺寸跟隨角色視覺比例；公開技能半徑／range／trajectory已換算為cm，不再乘一次。保持固定pool與回收／暫停／fog生命期。
5. 測試多種來源、world scale往返、pool identity／原actor transform不變、實際selection bounds與attachment；以正式world視距產生原圖並目視。正式ordinary nocache編譯與受影響原生回歸，記錄錯誤與範圍，不提前勾完整6.2／6.3。

實作複核追加：既有技能距離轉換將大於10000cm的公開半徑截短，VFX ribbon／size亦固定20000cm；在world scale100的正式世界會錯縮或拒絕合法cue。完成本輪未變C++ ordinary後，統一有界renderer尺寸契約，保留公開距離原值、不再縮小為診斷尺度；增加大範圍公開area／cone／trajectory與越界拒絕測試。固定pool512及每cue bounded幾何數不變。


2026-10-12 世界單位顯示比例當前範圍接受：`.ai-collab/world-unit-scale-accepted-v1/summary.json`。ordinary build-v4 nocache成功11.22秒；fresh native-v3 exact23 Success／零warning，49 codegen tests真正通過。兩native骨架、五fallback種類、兩既有Blueprint驗目標高度／比例往返／pool清理／原actor root與offset／selection body bounds；正式camera component90000cm視距原圖已直接目視，英雄與回城圈可見。公開30000cm area與cone14段、90000cm literal projectile及越界拒絕已驗，不縮成診斷cm或再次乘display比例。八authored及11combat原圖SHA同舊目視證據；cast_secondary當次重新直接目視。原動畫、skeleton、材質與world四packages SHA／mtime不變，stage contract再驗。E889–E891失敗保留；這是synthetic world固定相機及當前功能驗證，非完整地圖／正式對局或最終畫質接受。父項12/44、6.2／6.3仍pending；無新authority game／ACK。
