# 生命值音效角色與呈現來源隔離接受

2026-10-20。當前功能修正及完整分段本機驗收成功；OpenSpec仍33/44，十一父項待權威／LAN／效能或實際OS DPI證據。未新增對局、重設10/10或commit／push。

## 缺口與通用修正

PresentOriginalAudioFrame原先在更新音效owner／scope／HP／tick前未核對payload entity／generation與presentation epoch，卻使用目前actor身分建立cue key。有效red真world／owner／source的錯世代HP=0 frame產生目前角色死亡cue，唯一失敗Foreign health generation cannot emit a current source audio cue（預期0實際1）。

共用AOmUnitActor入口在任何副作用前驗目前entity id／generation、正且一致的audio epoch、同world／bridge owner；不新增欄位、timer、掃描、網路重試或ABI。green涵蓋錯id／generation／舊epoch不產生cue、外來future tick不污染clock、目前恢復及死亡cue／caption、新epoch低tick先seed而不假重生。NoSound只驗實際admission／history及caption，明示零voice，device輸出另由五PCM驗證。

E1430保留首次MinimapPreferences右拖曳命中失敗與diagnostic-v2未重現；只加日誌不算修復。最終fixture明確帶自己的視窗到前景、關閉前一個tooltip、同步實際cursor與Slate事件，退出時還原cursor；命中、座標及capture斷言不放寬，產品tooltip功能保留。WorldV3與source-v2獨立接受；原失敗的具體遮擋元件未確認，沒有冒稱已找到特定遮擋來源。

## 驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local無action cache | UBT6.17秒成功，stage SHA一致 |
| 專項 | 26成功，四筆既有preview-world warning保留 |
| 完整native | fresh206成功，既有warning保留 |
| 分段viewport | 458案例／222 wire／157 stages／13 cursor |
| Blueprint／音訊 | 69零compiler／load診斷，五PCM成功 |
| Lua契約 | fresh140 owned＋69 local acceptance成功 |
| 來源／資產 | 3335檔SHA／size／mtime及集合前後一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust明示沿用2026-10-19新機接受 |

foundation294案例／120wire／103stages，tail164／102／54及13cursor；原860／900秒、90 proof frames、8fps不變。strict union重新驗原始報告、七binary cohort、案例清單與清理，兩段AVI驗RIFF／索引／timeline／幀數／SHA。三分鐘監測至root退出，publisher／editor／profile均清理。兩公開map另驗144幀／12片段，六HUD與兩map原圖逐張檢視；圖片僅作呈現回歸，音效隔離由cue history／caption及獨立PCM證據驗證。

43份SHA／size證據保留有效red，E1429記錄產品缺口與防重犯。歷史證據原狀保留。execution_mode=phased、single_session_full_success=false；受控資料及跨session案例覆蓋不冒稱權威ACK、真LAN、單session固定效能或實際100%／150% OS DPI配對。
