# 極窄／直式視窗的戰鬥相機投影

Codex自行實作，不使用子代理。接續已接受的公開鏡頭邊界／地表覆蓋，補真實視窗比例缺項。

已完成當前功能完整本機驗收：`.ai-collab/playable-mvp-portrait-accepted-v1/summary.json`。普通建置13.38秒；fresh195 native全部成功（12 warning保留）、458實際視窗操作／222 wire／157 stage／13筆游標觀察、69 Blueprint零compiler error／warning／load failure、三份實際PCM成功。153來源前後一致，69資產SHA／mtime不變，全部程序與profile／listener清理成功。六項focused native零warning，三axis×四比例×兩有效縮放共24組通過；兩map十二張當次原圖直接檢視，包括兩張真400×1000 PNG。E1249／E1250及原失敗紀錄保留。

## 計畫

1. 在實際CameraGroundCoverage原生測試增加400×1000比例，保留六個既有比例／縮放組合。先取得當前行為，不修改覆蓋的合法投影守衛。
2. 依本機UE5.8正式投影API選擇共用策略：沿用既有16:9參考aspect與FOV，明確固定垂直戰鬥尺度，直式避免把垂直視野拉過地平線。保持既有camera／local player／小地圖與deprojection路徑，不另寫座標轉換，不改Engine或全域設定。
3. 實際投影四角、有效縮放、公開ground不變、未變資料不重寫、玩家與來源退休均須通過。兩張正式compiled地圖以極窄直式game viewport拍原圖，確認世界／HUD／小地圖仍可用。
4. 普通NoEngineChanges／local／NoUBA／nocache建置、來源雜湊凍結，先驗當前功能，最後fresh完整原生／458操作／69 Blueprint與三PCM。保留所有失敗與拒絕畫面。

## 狀態

初步400×1000預設投影重現測試成功；後續portrait-repro-v2真實重現MaintainX、400×1000、zoom7000向上角落遭拒。已修RTS camera明確使用MaintainYFOV，沿用既有16:9參考aspect／FOV，在同一相機上固定垂直戰鬥尺度；不改LocalPlayer全域配置。新測試以三axis×四比例×兩有效縮放直接驗projection矩陣與四角，整合portrait原圖改400×1000。E1249保留原失敗，修後尚待驗證。前輪camera-coverage-accepted-v1為歷史來源快照，不沿用為本輪production結果。無新權威對局／ACK／LAN／OS DPI切換，不重設10/10額度。

修後當前功能已驗：ordinary portrait-build-v3 NoEngineChanges／local／NoUBA／nocache成功13.38秒；portrait-native-v1六項Success／0warning。24組真投影／有效縮放、未變geometry不重寫、公開ground不變、LocalPlayer與來源退休全通；兩map十二張當次原圖直接檢視，包括400×1000、1600×600及1280×720，153來源於建置與native後一致。新production只在AOmRtsCameraPawn建立相機時明確選擇既有UE MaintainY策略，不引入另一份deprojection或改Engine。

fresh portrait-final-v1已完成上述完整驗收；最終接受綁定本輪來源、建置、實測與原圖，不沿用舊195／69結果。Rust1055與ABI29明示沿用先前已接受且綁定輸入未變的結果；這是當前本機功能接受，未新增權威對局／ACK、實體搖桿、LAN或OS DPI／PIE實測，整份44項計畫尚未完成。
