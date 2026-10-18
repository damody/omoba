# 世界光照生命週期

Codex直接實作，不使用子代理。接續24／44的當前功能，處理6.3中的通用光照生命週期，不將局部完成視為完整世界美術接受。

最終已接受：`.ai-collab/playable-mvp-lighting-accepted-v1/summary.json`。普通build-v5成功2.38秒，fresh195 native全部成功（12 warning保留）、458實際視窗操作／222 wire／157 stage／13筆游標觀察、69 Blueprint零compiler error／warning／load failure、三份實際PCM、全部程序與profile／listener清理成功。153來源前後一致，69資產SHA／mtime不變；focused6項零warning，15張當次原圖直接檢視。原Rust1055／ABI29明示沿用且綁定輸入未改。6.3光照／陰影／postprocess／camera fit基準已完成，進度25／44；沒有新增權威對局／ACK、LAN、實體搖桿、OS DPI或PIE實測，其他美術與跨環境父項不因此完成。以下執行中描述保留歷史。

## 計畫

1. 移除首次載入後永久不再判斷光源的狀態。公開地表啟用時立即判斷，之後每0.5秒僅枚舉DirectionalLight元件及同類呈現actor，不掃英雄／小兵或改其每frame路徑。
2. 只承認同世界、已註冊、可見、未退休、會影響世界且有有效非零輸出的場景光源；自動補光不冒充場景光源。同世界多個公開地表僅一份補光，來源移除後可恢復；場景自帶曝光與更高優先序postprocess保留。
3. 保持已接受的key／fill方向、色彩、強度與固定曝光，不臆測新的美術值。無公開地表時停止補光、曝光及tick；相機覆蓋仍使用既有實際controller。
4. 擴充WorldSurfacePresentation：場景光源新增、隱藏、元件隱藏、零輸出、移除、來源重載、多地表及多世界隔離，驗每0.5秒節流及零多餘geometry寫入。
5. 凍結153來源，普通NoEngineChanges／local／NoUBA／nocache建置，先驗受影響native及兩map當次renderer原圖，最後完整195 native／458操作／69 Blueprint與三PCM。失敗保留，不修改舊接受結果，不新增權威對局或ACK。

## 狀態

當前功能已通過：lighting-source-v4凍結153來源；lighting-build-v5普通NoEngineChanges／local／NoUBA／nocache成功2.38秒，bootstrap checkpoint已由正式refresh-v2完成並退休。lighting-native-v2六項Success／零warning，全部新增光照生命週期斷言、真實元件註冊／可見性／曝光與key-only陰影、geometry不重寫及來源重載精確+1通過；兩map十二viewport原圖與三張地表／路線／英雄近景共15張均已直接檢視。這些是合成公開幾何renderer，不是新權威對局。

lighting-final-v1正在fresh完整驗收；69 Blueprint已於native階段先擷取SHA／mtime。完整結果出爐前不宣稱整批成功。6.3尚未勾完，完整美術／曝光品質與跨環境父項不以單一生命週期修正代替。

## 光照與鏡頭基準

| 基準 | 當前實作與證據 |
|---|---|
| Key／Fill | 保持既有方向、2.5／0.75強度、暖key／冷fill；只有key投射動態陰影，保持單一forward main light。 |
| Postprocess | 補光使用手動固定曝光、關閉physical camera exposure，priority -1000，場景作者的較高優先序設定保留。場景已有有效directional light時關閉整組native曝光與補光。 |
| 重載／多世界 | 每0.5秒只枚舉光源元件與同類地表；有效同世界光源動態切換，最多一個fallback owner；公開來源退休停止tick與曝光。 |
| 有效Camera | 沿用已接受的公開bounds、投影四角地表覆蓋及MaintainY策略；本輪CameraGroundCoverage再次通過24組真實投影／有效縮放，CameraProjectedDrag與OwnedCameraInputStack再次成功。 |
| 當次畫面 | WorldSurfacePresentation產生768×768地表／路線／英雄陰影；兩正式compiled map另有1280×720、1600×600與400×1000 viewport，共15張原圖逐張檢視。遠距圖用於縮放邊界，不能當一般遊玩視角或效能證據。 |

以上是6.3所需的光照／陰影／postprocess／camera fit基準，不代表6.1完整地圖美術、6.2全部模型、6.4全部動畫VFX、6.6實際事件錄影或7.3跨環境矩陣完成。完整本機回歸成功後才更新6.3勾選。

原碼InitializeLighting只執行一次；ResetPublicView也未清除bLightingInitialized，因此場景新增／移除／隱藏光源或換局會保留過期判斷。已確認本機UE公開Light／SceneComponent API，接續實作與實測。

普通build-v2成功10.78秒，153來源一致。建置後審查補強多地表仲裁：保留現有補光owner再比較穩定ID，避免新actor重用較低UObject ID時在下一tick前短暫重複補光；新增首次Apply即不得重複的斷言。此補強需新來源與新普通建置，不沿用v2作最終結果。
