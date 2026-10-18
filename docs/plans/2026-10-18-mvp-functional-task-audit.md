2026-10-18正式視窗取樣錄影工具已接受：`.ai-collab/owned-window-video-accepted-v1/summary.json`，詳`docs/plans/2026-10-18-owned-window-video.md`。共4811原始影格／11原尺寸片段／144845788 bytes，逐張RIFF／JPEG envelope／索引／時間紀錄與總數核對成功，八原始JPEG直接檢視；ordinary build-v2入口20秒，focused兩Success／零warning，fresh完整197 native（12既有warning）／458事件／222 wire／157 stage／13游標觀察／69未變Blueprint／五PCM與所有清理成功。1197來源／資產SHA及保護mtime一致，441已擷取Rust／ABI輸入相同，明示沿用1168 Rust／五checks／九ignored與ABI30。影片為既有synthetic filtered feed的真Slate視窗，世界區域黑底；不是新權威對局／ACK、完整三路美術或效能證據。此工具不勾完6.6，整體仍27/44、17項待辦；以下語音及更早段落保留歷史。

# MVP 當前功能與待辦核對

2026-10-18公開對局語音與獨立音量完整接受：`.ai-collab/public-match-voice-accepted-v1/summary.json`，見`2026-10-18-public-match-voice.md`。四段CC0英文播報、原生Vorbis→PCM16 WAV→SoundWave來源鏈與零覆寫匯入、可用Voice滑桿／獨立保存、公開phase轉換／重連不補播／結果續播／字幕與清理已實作。ordinary build-v5（UBT23.98秒、入口33秒）、1193來源／資產與mtime一致；focused6零warning及當次設定原圖已驗，fresh完整197 native成功（12既有warning保留）、458操作／222 wire／157 stage／13游標觀察、69未變Blueprint、五份實際PCM及所有清理成功。441個已擷取Rust／ABI輸入一致，明示沿用1168 Rust／五checks／九既有ignored與ABI30；不冒稱fresh Rust或新權威對局／ACK。6.5完成，整體27／44；其餘17項維持待辦，以下26／44與更早紀錄保留歷史。


2026-10-18七位正式英雄頭像覆蓋完整接受：`.ai-collab/hero-portrait-accepted-v1/summary.json`，見`2026-10-18-hero-portrait-coverage.md`。新增兩張原創圖片及完整目錄守衛，1148輸入／資產mtime一致；fresh1168 Rust／五checks／九既有ignored、ABI30 MSVC、ordinary27.10秒、focused8零warning／九當次1280×720原圖、完整197 native（12既有warning）／458操作／222 wire／157 stage／13游標觀察／69未變Blueprint／三PCM及清理成功。仍26／44，不藉兩張圖勾完4.1或6.2；無新權威對局／ACK。下一批依`2026-10-18-public-match-voice.md`接獨立Voice分類、公開狀態轉換與真PCM，以下舊紀錄保留歷史。


最新進度26／44：6.1公開三路地面／lane／river／jungle／阻擋輪廓／塔與base呈現已依`2026-10-18-public-map-biomes.md`及`.ai-collab/map-biomes-accepted-v1/summary.json`完成。1024來源、ordinary15.49秒、fresh1168 Rust／五checks／9既有ignored、ABI30 MSVC、197 native（12既有warning）、full458／222 wire／157 stage／13游標觀察、69未變Blueprint與三PCM／全部清理成功；focused8零warning、17當次原圖已驗。資料未改lane／碰撞／fog權威邏輯，原始失敗與破面拒絕保留。其餘18項仍未完成；立即依`2026-10-18-hero-portrait-coverage.md`補兩位正式英雄的缺圖與coverage guard，不把scope成功冒稱44項完成。下方25／44與更早紀錄為歷史。

最新進度25／44：6.3已依`2026-10-18-world-lighting-lifecycle.md`完成光照／陰影／postprocess／camera fit基準，接受紀錄`playable-mvp-lighting-accepted-v1`。fresh195 native（12 warning保留）、full458／222 wire／157 stage／13筆游標觀察、69未變Blueprint、三PCM及清理成功；153來源一致，focused6項零warning及15當次原圖已驗。完整世界美術、語音、錄影與跨環境等19項仍未完成。下方24／44是前輪核對紀錄。

Codex自行核對，不使用子代理。計畫：先完成直式投影修正及fresh完整驗收，再把已具實作與證據的操作項目同步至OpenSpec，保留未實測與未實作範圍。這是狀態核對，不把既有功能當成本輪新增功能。

本輪接受：`.ai-collab/playable-mvp-portrait-accepted-v1/summary.json`。153來源、普通NoEngineChanges／local／NoUBA／nocache建置13.38秒、fresh195 native（12 warning保留）、完整458案例／222 wire／157 stage／13筆游標觀察、69 Blueprint零compiler／load診斷且資產SHA／mtime不變、三PCM及全部清理成功。24組真實相機投影及兩map十二原圖另有focused6項零warning；不是新權威對局或ACK。

依既有功能及本輪完整回歸，以下11項由未勾同步為已完成，總進度由13／44變為24／44：

| 項目 | 實作與接受依據 |
|---|---|
| 1.4 | 版本9設定、衝突／遷移／還原／保存及模式與英雄繼承；ControlProfile、ControlProfileSet及AccessibleInput.Profile測試。 |
| 2.4 | UI wheel／pointer守衛、透明HUD、F9與改鍵保存；ActionOwner及OwnedViewportIpc實際按鍵回歸。 |
| 2.7 | full458中的小地圖左右配置、點／拖、有效鏡頭、wheel、Space／Y、移動／施法偏好及ping／aim仲裁；CameraProjectedDrag與MinimapPreferences。 |
| 3.2 | A瞄準、範圍、左鍵確認、右鍵／Esc取消及HUD負例；共用瞄準原生與實際視窗事件。 |
| 3.7 | Ctrl升級、Alt自用、Shift覆寫、B回城、六格道具模式與獨立ward4／舊設定遷移；完整操作ordered prefix及目前native。 |
| 3.10 | 點擊／鍵盤模式、英雄／模式／全域設定、Scout Ahead／45度、切換時限／重連及游標／搖桿映射；AccessibleInput及full458實際Slate事件。搖桿為合成軸事件，未宣稱實體控制器實測。 |
| 4.2 | 正式slot圖示、按鍵、冷卻、rank／公開charges、不可用原因與完整tooltip；Hotbar／Ward／Summoner／HUD native及完整操作。 |
| 4.6 | Esc優先序、商店／聊天focus、透明HUD與modal；native及實際視窗負例。 |
| 4.8 | 死亡／回城／重生、來源失效／斷線、結算及舊input／cue退休；native、full458及已接受且來源未變的ManagedNextMatch證據。 |
| 7.2 | 當前普通建置、195 native、69 Blueprint、stage／來源SHA及資產檢查；Rust1055與ABI29僅明示沿用先前接受且本輪輸入未變的結果，沒有宣稱重新執行。 |
| 7.5 | 更新操作手冊、原始交接、當前驗收、投影計畫及E1249／E1250，列出證據與限制；本輪沒有新增按鍵。 |

舊段落的8／44、13／44及「尚未接完」保留為各次歷史，不能拿來覆蓋本頁當前狀態。完整功能證據以本輪接受及各功能明示引用的已接受紀錄為準。

未完成項目仍維持20／44：完整世界／河道／野區美術、全部動畫與VFX、語音素材、完整固定視點與實際事件錄影，以及要求權威ACK／跨PIE／OS DPI／實體環境／LAN／效能的驗收。本輪沒有重設既有10／10對局額度。功能部分通過不代表這些較廣父項已完成，也不歸檔整個change。
