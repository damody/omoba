# 移動與生命值音效目前frame／綁定來源接受

2026-10-20。當前功能修正及完整分段本機驗收成功；OpenSpec仍33/44，十一父項待權威／LAN／效能或實際OS DPI證據。未新增對局、重設10/10或commit／push。

## 缺口與通用修正

移動音效入口只比較epoch，外來audio component雖不能播目前角色音效，仍先改寫walking baseline，吃掉合法起步；舊idle亦可使持續移動假起步。生命值入口先前雖隔離entity／epoch，同身分future tick仍可能在cue被拒絕後污染HP clock。有效red以真world／source／兩個bridge音效元件驗證，唯一失敗Foreign audio owner cannot consume current movement transition（預期2實際1）。

共用IsCurrentPresentationFrame以O(1)驗正epoch及精確epoch／tick，health與locomotion都在副作用前使用。移動入口另驗目前綁定元件及同world／bridge owner。沒有新actor時鐘欄位、timer、掃描、重送或Rust／FFI變更。green驗外來元件不吃起步、舊idle不假起步、落後audio frame但高於HP基準的狀態及future狀態不改baseline、fresh起步、同entity future／past health不污染clock／HP並保留恢復caption。前輪entity／epoch、死亡與新timeline測試保留。NoSound只驗真admission／history／caption且零voice，實際音訊由五PCM另驗。

## 驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local無action cache | UBT22.80秒成功，stage SHA一致 |
| 專項 | 26成功，四筆既有preview-world warning保留 |
| 完整native | fresh206成功，既有warning保留 |
| 分段viewport | 458案例／222 wire／157 stages／13 cursor |
| Blueprint／音訊 | 69零compiler／load診斷，五PCM成功 |
| Lua契約 | fresh140 owned＋69 local acceptance成功 |
| 來源／資產 | 3335檔SHA／size／mtime及集合前後一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust明示沿用2026-10-19新機接受 |

foundation294案例／120wire／103stages，tail164／102／54及13cursor；原860／900秒、90 proof frames、8fps不變。strict union重新驗原始報告、七binary cohort、案例清單與清理，兩段AVI驗RIFF／索引／timeline／幀數／SHA。三分鐘監測至root退出，publisher／editor／profile均清理。兩公開map另驗144幀／12片段，六HUD與兩map原圖逐張檢視；圖片僅作呈現回歸，不以字幕或cue數冒稱device播放。

38份SHA／size證據保留有效red；E1431記錄產品缺口與防重犯。歷史證據原狀保留。execution_mode=phased、single_session_full_success=false；受控資料與跨session案例覆蓋不冒稱權威ACK、真LAN、單session固定效能或實際100%／150% OS DPI配對。
