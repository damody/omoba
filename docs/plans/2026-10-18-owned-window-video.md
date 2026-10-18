# 正式視窗操作錄影

由 Codex 獨立實作；沿用已接受的1193輸入基準，當前27/44，不重設10/10對局額度。

## 計畫

1. 建立可重用的原生 Motion JPEG AVI writer，使用 UE ImageUtils 編碼，不依賴外部程式、系統 codec 或 Engine 修改。最多每秒擷取8張、以8fps封裝並另記原始時間，有界記憶體與1GiB檔案上限，拒絕覆寫、非法尺寸及壞JPEG，完成時寫正確RIFF長度與索引。
2. 在既有 owned viewport 真實 Slate 事件流程加入可選錄影；只擷取其所屬視窗。每張保留單調時間與測試階段；尺寸改變另開片段，不縮放或裁切原畫面。錄影失敗必須使驗收失敗，關閉／失敗路徑皆 finalize。
3. 原生檔案測試解碼實際JPEG、驗索引與壞輸入／覆寫拒絕。再跑實際 owned viewport 輸入錄影並檢查原始畫面、檔案、時間紀錄與程序清理。
4. ordinary NoEngineChanges 建置、來源與資產保護核對、相關測試後最後完整驗收；不是新權威對局、ACK、LAN或整場效能證據。父項6.6仍要求出生／lane／多人combat／fog／HUD／shop完整固定視點，不能只憑工具存在勾完。

## 當前實作與專項證據

- `OmWindowVideoWriter` 使用UE ImageUtils產生JPEG並串流至Motion JPEG AVI，RIFF／stream／movi長度、每張keyframe索引均在close完成；尺寸上限4096且最多8388608 pixels，16000張／合計1GiB硬限制。輸入視窗換尺寸即另開原尺寸片段，不裁切或縮放。
- `moba_owned_viewport_ipc.lua --record-video`與`ue_local_acceptance.lua --record-video`接既有owned Slate事件流程，沒有新增模擬gameplay、成功ACK或system-global截圖。每張紀錄stage、實際單調時間、片段／影格／尺寸；影片每秒8張播放，低於取樣率時不補畫／不複製影格，應以JSONL查看原始經過時間。影片無音軌；真音訊沿獨立PCM驗收。
- Lua `owned_window_video`完整核對所有RIFF長度、JPEG envelope、索引、影格／片段／時間軸／native總數與SHA；原生專項另用真JPEG encoder／decoder確認不同畫面與尺寸。這不宣稱Lua重新解碼每張JPEG。
- ordinary `owned-video-build-v2`成功（入口20秒，無refresh），1197來源／資產已在建置前完成擷取；`owned-video-native-v1`兩項Success／零warnings。新`Om.Editor.WindowVideoContainer`為明確執行的editor專項，既有完整197項自動名單不包含它，不能加算成198項完整原生。
- Lua容器／損壞資料50檢查、既有IPC contract127檢查通過；441個已擷取Rust／ABI輸入相同，明示沿用1168 Rust／五checks／九既有ignored與ABI30，沒有fresh Rust宣稱。

當前功能已完整接受：`.ai-collab/owned-window-video-accepted-v1/summary.json`。ordinary build-v2成功20秒；focused兩Success／零warning，owned-video-final-v1真197 native（12既有warning保留）／458 cases／222 wire／157 stages／13游標觀察／69 Blueprint零compiler／load診斷／五PCM全部成功。1197來源與資產SHA、保護mtime一致；441已擷取Rust／ABI輸入相同，明示沿用1168 Rust／五checks／九ignored與ABI30。所有Editor／publisher／profile／listener已清理，沒有活動編譯或fixture consumer。

已直接查看第一個封存片段的原始JPEG（ArmedAWorld階段、frame227），其800×600像素確實包含正式HUD與當前tooltip；此owned輸入fixture的世界區域為黑底，不能以這段影片宣稱完整三路世界／A範圍美術已驗。錄影不補畫地圖或隱藏tooltip，後續仍須另錄正式世界場景，父項6.6保持待辦。

## 當次錄影與使用方式

11個Motion JPEG AVI在`omfue/Saved/Automation/OwnedViewportIpc/owned-video-final-v1-ipc/owned-window-001.avi`至`owned-window-011.avi`。合計4811張、144845788 bytes（約138 MiB）；原始經過797.450秒，8fps播放約601.375秒，必須用`owned-window-frames.jsonl`讀取真實時間，不稱即時速度或FPS效能。`owned-window-video.json`保留每段SHA與尺寸、影格／長度核對。沒有音軌，五PCM為另外的真音訊驗證。

八個未修改JPEG body在`.ai-collab/owned-video-final-v1-video-originals/`；`review.json`記錄直接看過的八張與限制，沒有宣稱逐張目視4811張。Ground／ArmedAWorld／Accessibility保留當時大tooltip；Minimap／Display可見英雄與slot／小地圖變更；ShopTxn可見正式recipe／購買widget但receipt來自fixture；Selection保留400×300原始像素。黑底世界不是完整地圖驗收，不以換背景／裁切修圖掩蓋。

```bat
tools\lua\lua.exe scripts\moba_owned_viewport_ipc.lua --ue-root D:\UE5.8 --run-id 新的唯一ID --record-video
tools\lua\lua.exe scripts\ue_local_acceptance.lua --ue-root D:\UE5.8 --run-id 新的唯一ID --native-rhi d3d12 --record-video
```

下一範圍仍依父計畫補正式世界／多人combat／fog／HUD／shop的固定視點與事件錄影；本輪沒有重設10/10，也未勾完6.6或其他父項。
