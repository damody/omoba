# 普通戰鬥鏡頭與公開地表覆蓋

後續極窄／直式視窗相機投影修正及fresh完整驗收已接受：`playable-mvp-portrait-accepted-v1`，見`2026-10-18-camera-portrait-projection.md`。本頁保留公開邊界／地表覆蓋的原始實作與接受紀錄。

Codex自行實作，不使用子代理。本輪接MVP P0.4；當前操作／HUD本機接受紀錄保留，不宣稱已有新權威對局。

## 問題

FocusCameraOnOwnedHero首次以固定±200000cm，再從可見單位擴張並加80000cm設定中心邊界，與公開地圖尺寸無關。裝飾地表固定擴張一個公開網格長邊，極端縮放／視窗比例或靠邊移動沒有實際投影覆蓋保證。這些是呈現範圍，不應用可見敵人位置推導。

## 實作順序

1. 從已通過驗證的公開網格地表取得精確XY界限；同一相機SetCameraBounds供跟隨、拖曳、小地圖、鍵盤平移共用。沒有公開地表的legacy路徑保持相容。
2. 一個世界地表actor於PostUpdateWork讀取已綁定本地玩家的實際投影，固定四角射線與裝飾地表平面相交。將裝飾範圍合併基準外裙與視野範圍，含一像素數值邊界；不修改公開地表、碰撞、導航、迷霧、世界單位或目標。
3. 設定／視窗／縮放變化由同一投影機制處理；未變投影不寫transform；無效資料拒絕，來源reset停tick並清除玩家引用。每frame僅一個有界投影查詢，無逐entity搜尋。
4. 原生驗證公開bounds、實際投影四角、窄／寬視窗、縮放、非法與退休、未變資料不重建。兩張正式compiled地圖實際game viewport在角落拍原圖，核對場景／HUD／小地圖與地表覆蓋。
5. 普通NoEngineChanges／local／NoUBA／nocache建置，凍結相關來源。先驗受影響功能，最後再整合完整當前功能驗收；錯誤原樣保存到register。

## 狀態

當前功能與完整本機整合驗收成功：`.ai-collab/playable-mvp-camera-coverage-accepted-v1/summary.json`。ordinary camera-coverage-build-v4成功15.68秒，153來源於建置、native與完整操作後一致；native-v3真六項Success／0warning，含CameraGroundCoverage／WorldSurfacePresentation／WorldBridgePresentationScale／CameraProjectedDrag／MinimapPreferences／OwnedCameraInputStack。新原生採實際viewport／player projection驗三比例×兩縮放的四角、有效arm、公開ground不變、同投影不重建與註冊／來源退休。兩map×四角八張及兩map×戰鬥／出生四張，共十二張當次原圖已逐張檢視；遠景實際相機高度418847cm、普通77942cm，沒有以DesiredZoom代替有效縮放。

fresh `playable-mvp-camera-coverage-final-v1` workflow.success=true：195 native全部成功（187 clean／8 warning-success，12 warning原樣保留）、三份實際PCM成功、完整458操作／222 wire／157 stage與13筆游標觀察成功、69 Blueprint compiler errors／warnings／load failures皆0。69資產於native階段事先擷取SHA／mtime，完成後逐檔一致；editor／publisher／profile與listener全清理成功。這輪195／458／69／PCM均fresh，沒有沿用前輪194的結果。Rust／ABI無改動，前批Rust測試明示沿用：153綁定輸入僅五個UE來源改變，其餘雜湊相同。10/10權威對局額度不重設，沒有新對局／ACK／LAN或實體OS DPI切換。E1245–E1248與拒絕的原圖／首次失敗原樣保留。

正式frame在ApplyPublicGrid成功後綁定當前本地controller。只有公開網格失效／source reset才撤銷覆蓋，沒有活著的可見單位不撤銷仍有效的公開地圖。reset清除controller並停tick，不沿用舊來源的投影。

本輪五個UE來源：WorldPresentationActor的header／cpp新增公開bounds、單一PostUpdateWork投影覆蓋與退休；WorldBridgeActor改用公開地表限制中心；WorldSurfacePresentationTests補真viewport與生命期；OmCompiledMapSceneCapture補有效縮放與四角原圖。沒有每entity新增controller或widget搜尋。

範圍限制：裝飾覆蓋只接受有限、向下與地平面相交的投影；無效／朝天投影不擴張geometry。compiled整合fixture的team由正式model明示承認、是合成renderer資料，不假裝已協商的權威team；一般真入口仍走原本的已驗證runtime team。此批不變更角色、地圖素材或碰撞／迷霧。
