# 一般操作回覆接線與完整本機驗收

2026-10-20。剩餘11項的本輪實作：完成4.3的一般操作request／result回饋子範圍，同時補移動／攻擊移動／Stop／Hold等操作的回覆呈現。本輪包含正式程式、整合回歸及視窗取樣修正。OpenSpec父項仍33/44，11項包含原權威ACK／新能力對局／LAN／固定效能及實際100%／150% OS DPI證據，沒有勾成全數完成。零新增權威對局，10/10不重設，未commit／push。

## 實作與行為

- 新增共用FOmActionFeedbackTracker，依原request ID、local player、command kind及HUD owner epoch關聯結果。涵蓋Move、AttackMove、AttackTarget、CastAbility、UpgradeAbility、ItemUse、Recall、Hold、Stop及Summoner。
- 原世界／Blueprint共用SubmitGameplayInputEvent與小地圖直接Move入口記錄真正的bridge admission。HUD先顯示「指令已送出」，不能據此確認權威行為。
- 正式OmAppliedInput派送連到同一live HUD；包含沒有presentation snapshot的control-only frame。Ok顯示「指令已確認」、InvalidArgument顯示「指令遭拒絕」，未知具體原因不捏造；Busy及逾時顯示「指令尚未確認」。指令確認不代表已到目的地、造成傷害或完成施法效果。
- 最多64筆、10秒的有界記錄；不自動重送。重複、錯誤ID／player／kind／epoch及已退役結果不能改HUD，舊指令的晚到結果不覆蓋最新操作。死亡／來源退休與清除回饋會清掉關聯，新本機拒絕會取代舊pending通知。
- 商店仍使用原receipt／undo確認，ward使用既有owner投影，方向與偏好沿原狀態契約，不借一般Ok推測退款、插眼或實際技能效果。畫面更新沿既有layout invalidation與三秒通知，不新增per-entity掃描。

## 共用視窗取樣

完整v1曾在原shared deadline逾時，失敗完整保留。讀UE5.8的TakeScreenshot實作確認它自行PrivateDrawWindows，移除helper在前面重複的ForceRedrawWindow／flush。新backbuffer回呼回歸確認四種實際尺寸每次取樣只繪製一次，仍檢查真render extent、完整pixel count及新鮮顏色內容。不改Engine、不減錄影頻率、不丟幀、不放寬原期限。

## 本次驗收

普通UE5.8建置action-feedback-build-v6成功（UBT14.59秒），十六項fresh專項零warning，127 Lua契約通過。完整action-feedback-final-v2：205 native全成功（12筆既有warning保留）、458操作／222 wire／157 stages／13 cursor、69既有Blueprint零診斷且package不變、五份實際submix PCM及全部清理成功。

完整操作錄影3559影格／11片段／113986636 bytes，實際經過590.485秒；原RIFF／JPEG envelope／索引／SHA及逐幀timeline嚴格通過。明示synthetic filtered-feed的實際Slate視窗，並非權威對局、LAN、整場效能或實體音響驗收。

公開世界另有144原始影格／12片段／14643804 bytes，兩map原圖、原AVI JPEG及文字／VFX像素對照通過，來源明示synthetic compiled public map。

四張當次800×600原始視窗逐張檢視，真Slate文字及paint後可見性斷言全過。另從完整實際操作錄影直接擷取Ground／Enemy／MoveOnly／ShiftA四張原始JPEG並逐張檢視，地板右鍵、敵方右鍵、MoveOnly及Shift+A均可見正確指令名稱與「已送出」；來源仍是synthetic filtered-feed，不當成gameplay ACK。模型負例包含錯誤ID／owner／kind／epoch、Busy後終態、重複／亂序、容量、NaN、逾時與退休；production frame測試涵蓋control-only拒絕與確認。這些使用明示synthetic結果，不冒稱新權威ACK。

3334份來源／資產的檔案集合、SHA、size、mtime驗收前後完全一致。527份Rust／ABI及93份內容Lua也與原新機完整接受一致，明示沿用2026-10-19-new-machine的Rust／ABI／TD1–100及all-target結果，不宣稱本輪重跑Rust。DLL最後read-only stage SHA通過。

可攜原圖、報告、build／stage／Lua摘要及原log指紋：`docs/acceptance/2026-10-20-action-feedback/index.json`。原始log留在Saved/BuildTemp，完整報告與AVI留在`omfue/Saved/Automation/LocalAcceptance/action-feedback-final-v2`及`omfue/Saved/Automation/OwnedViewportIpc/action-feedback-final-v2-ipc`。

## 限制與錯誤保留

3.1／3.3／3.4／3.5／3.9／4.3／4.4／7.1／7.4尚含原gameplay ACK、新能力真對局、LAN或固定效能證據；4.7／7.3仍缺實際100%／150% OS DPI配對。當前可不新增對局處理的一般回覆呈現已實作及接受，沒有把測試控制ack當gameplay ACK，也沒有用text scale冒充OS DPI。

E1359–E1365記錄本輪盤點、三次編譯失敗、首次paint前visibility斷言、Saved工具盤點錯誤及完整v1逾時／重複繪製成本。所有失敗log／報告保留；修正後才fresh建置、專項與完整驗收，不覆寫失敗、不增加期限、不跳過可見性斷言。長流程依exact owned identity每3分鐘記錄程序與log metadata，另檢查CPU及公開快照序列有前進，完整監控留Saved，衍生摘要納入可攜證據。
