# 公開對局播報與獨立語音音量

## 最終接受（2026-10-18）

已完成：`.ai-collab/public-match-voice-accepted-v1/summary.json`。四段[Kenney Voiceover Pack](https://kenney.nl/assets/voiceover-pack) CC0英文錄音已接正式Voice分類；來源及包含授權存於`omfue/Content/Omoba/Audio/VoiceV1/provenance-v1.json`。不宣稱自行錄音或角色專屬配音。既有十三原創音效、27個原始音訊檔與package SHA／mtime未變。

1193來源／資產與保護mtime最終一致。正式refresh-v2解碼與首次四asset保存成功，當次第二次匯入四verified／零save；獨立import-repeat-v1兩次亦四verified／零save，含四derived WAV與四SoundWave的整批資產hash／mtime保持。ordinary build-v5 UBT23.98秒、入口33秒，NoEngineChanges／local／NoUBA／nocache成功。Native-v2六項成功零warning，當次2065×1578設定頁原圖直接檢視；真Slate改Voice為99%、獨立保存與重開值正確，其他音量不變。

最後final-v1完整197 native成功，12既有warning保留；真458操作／222 wire／157 stage／13筆游標觀察，69 Blueprint零compiler error／warning／load failure且package SHA／mtime不變，五份真realtime owned mixer PCM與所有publisher／editor／profile／listener清理成功。音訊捕捉只涵蓋自有submix，不推斷實體喇叭可聽。441個已擷取Rust／ABI輸入一致，明示沿用前批1168 Rust／五checks／九既有ignored及ABI30證據；後續僅UE／Lua／audio差異。

6.5完成，27／44，其餘17項仍待辦；未新開權威對局／ACK，未宣稱LAN／實體控制器／跨OS DPI或全部MVP完成。E1285–E1290與所有原失敗保留。下方計畫／中間失敗說明為歷史，以本段接受為準。

| 實作 | 正式來源 |
|---|---|
| 有界Ogg／manifest、license／來源／derived PCM與package完整性 | `scripts/voice_audio_import.lua` |
| 受管理匯入、第二次零save、portable SHA receipt | `scripts/import_moba_voice.lua` |
| 原生Vorbis解碼、先全批驗證、canonical WAV、SoundFactory及既有目的地拒絕覆寫 | `OmEditor/Private/OmOriginalAudioImportCommandlet.cpp` |
| 四Voice資源、獨立分類／32元件上限、phase／epoch／team／字幕／靜音／退休 | `OmRuntime/Private/OmAudioPresentationComponent.cpp` |
| 真公開HUD消費路徑 | `OmGenerated/Private/OmWorldBridgeActor.cpp` |
| 可用滑桿／語意標籤／真Slate調整與保存 | `OmRuntime/Private/OmControlSettingsWidget.cpp` |
| 正式build ensure與verify-only guard、cook目錄 | `scripts/build_ue_moba.lua`、`omfue/Config/DefaultGame.ini` |

Lua voice34、原音效49、真stage12、portrait28自足檢查成功；native驗已知資源、mute／caption／late join／epoch／rollback／team、真WorldBridge開始與結果接線、結果刷新續播、設定UI與重開。完整fresh PCM涵蓋routed-mix非零、master-muted零、ambient-restored非零、voice-routed非零、voice-muted零。未知／無效公開資料不產生播報，中途加入不補舊語音，解除靜音不重播。


前批完整接受為`.ai-collab/hero-portrait-accepted-v1/summary.json`，1148輸入與資產mtime一致、所有consumer退出。Codex自行接續，不使用子代理。整體26／44；本批不聲稱新權威對局或ACK。

## 缺口與方案

既有13個原創音效、Music／Effects音量與字幕已接通，但Voice永遠回報不可用，沒有實際Voice路由。不能以效果音冒充語音。採用Kenney Voiceover Pack的四個英文女聲公共播報ready／game_over／you_win／you_lose；來源為https://kenney.nl/assets/voiceover-pack，包內CC0授權已閱讀。素材屬第三方CC0錄音，明確標示來源，不宣稱自行錄音或角色專屬配音。95個Ogg經有界ZIP檢查、路徑／大小／CRC／SHA盤點；盤點本身尚不代表解碼或播放成功。

1. 四個原始Ogg byte-identical放入獨立VoiceV1目錄，記錄官方URL、archive／原始檔SHA、包含授權、語句、中文字幕及未加工事實。既有OriginalV1十三音效／packages不覆寫。
2. 新Lua匯入入口與純驗證module產出固定namespace的有界計畫；先驗完整manifest／來源身份／路徑／SHA，使用既有UE SoundFactory與SoundFileIO原生解碼。commandlet在首次asset mutation前解碼全部檔，檢查聲道、取樣率、時長、非靜音PCM；只新增或核對自有metadata，拒絕覆寫既有不一致資產。第二次匯入必須零save且package SHA／mtime不變。沒有PowerShell／Python轉檔fallback。
3. AudioPresentation沿用32元件上限，Voice使用獨立分類與master乘積，新的播報取代舊播報，效果音／音樂各自控制。可用性取決於完整真SoundWave集合，缺檔有明確診斷，字幕即使靜音仍可用。
4. 唯一入口觀察已公開HUD的epoch／tick／phase與本地／勝方team。只播同一scope內已觀察到的準備→開始及進行→結束；初次加入進行中／結果或重連不補播舊事件。每scope每階段至多一次，重複／倒退／非法team或phase不產生播報；未知勝方用中性結束語句。退休清理全部語音與字幕，不從hidden entity衍生資訊。
5. 自足Lua負例驗manifest／SHA／路徑與result契約；native驗實際資源、初次加入／重複／倒退／epoch／隔離／字幕／音量，原有音效測試保留。實際mixer新增Voice非零與Voice靜音PCM，不以函式回傳替代聲音輸出。ordinary build、既有Blueprint與素材SHA／mtime保護；最後fresh完整本機驗收。任何失敗留原檔並記error register，未完成不得勾6.5。

## 準備證據

目前更正：第一版SoundFileIO原生Ogg解碼實際觸發引擎EOF Array assert（E1288），拒絕該批接受；尚無語音資產產生。改為UE公開VorbisAudioDecoder→有界canonical PCM16 WAV→既有SoundFactory WAV匯入，保留取樣率／聲道／sample count，不修Engine或加特殊檔案補丁。原Ogg與四derived WAV／四SoundWave各有SHA，重複解碼與匯入零覆寫。Voice UI舊的強制停用亦已發現並修正，增真Slate調整／獨立保存／重開驗證（E1287）。當前仍待成功建置、匯入、mixer與最後完整接受，不提前勾6.5。

`.ai-collab/cc0-voice-source-v1/summary.json`僅接受來源／授權／格式盤點；archive 1,851,632 bytes。完整license與95檔SHA保留，不提交ZIP／log／可執行暫存。UE 5.8的SoundFactory.cpp明確列出Ogg且經SoundFileIO轉WAV；使用引擎公開API，不改Engine。
