# 2026-10-22 權威施法準備動作最終驗收

OpenSpec complete-moba-cast-preparation-animation **5/5完成**。接續17系統全遊戲盤點，本批補可見英雄施法前搖動作及中斷生命期。

## 實作

實際AbilityWindups在outcome與buff timer後同步native-only cast_preparation公開Buff，不從本機瞄準、動畫名稱或cooldown推論。每次準備帶有有界Q10 total／remaining及保留int64精度的十進位instance字串；後置檢查取消遲到的控制、移動、rank變更及死亡，終局清理。僅遍歷有效windup與已索引狀態，沒有新增全體entity逐frame掃描。

正式BuffSnapshot沿既有ABI34 active buff傳至Unreal typed FrameState。公開投影及Bridge仍使用完整entity generation與既有視野／來源排序；解碼明確檢查JSON型別、大小、數值界線、十進位overflow及合法剩時，拒絕非英雄、死亡、重複或非法資料。

角色沿既有cast資產對齊權威前搖進度及播放速度。準備優先於walk／hit；死亡、respawn、recall仍較優先。取消恢復最新移動姿態，成功release清準備後啟動獨立瞬時動作，退休instance不可復播。pause停播，snapshot延遲時保持cast末端；隱藏清快取，重新可見須收到新的有效權威frame才可恢復當前進度。epoch／pool generation重設沿既有守門。

不改技能魔力／冷卻／傷害或召喚效果交易，沒有逐英雄特判，沒有新增美術資產。

## 集中驗收

最終RustV2 **1799通過／10既有ignored／0失敗**：template75、core908、codegen58、base268、backend190／1、bridge134／1、client151／8、ABI15；含generation九工作全部成功退休。

新增核心驗證精確第一tick、後續倒數／pause、停止／控制／移動／rank／死亡、未admit view／終局／換代退休。正式Production60Hz generated saika_reinforcements驗證512 raw起始、pause凍結、Stop／stun取消不扣魔力、重施法instance遞增、30tick後僅產生兩個召喚物並進入cooldown。Bridge實際ABI payload保留最大int64 identity，純倒數以active snapshot更新且不重發cue，generation替換與移除退休。

UE5.8 build-v2成功（UBT **5.55秒**），正式後端build-v1成功，Lua三契約成功。WorldV2 **43成功**，0 Failed／NotRun；4既有清理warnings保留。CastPreparationAnimation檢查有界解碼、public ABI、實際skeletal animation資產／25%與50%cursor、移動與hit優先、pause／末端hold、取消／新instance／隱藏／release／死亡／respawn／reset。另回歸兩map的地形、塔、兵線、野怪與既有HUD／技能提示；兩map各74影格／6片段，六張回歸PNG已目視。本批沒有另宣稱施法動作專用截圖或真連線接受。

V1 World42成功／1失敗：Unreal TryGetStringField會將數字轉字串；先檢查EJson::String後讀值修正。RustV1新增bridge fixture將純倒數誤期為BuffUpdated，改驗既有active snapshot節流契約；V1來源檢查因Rust成功守門拒絕。所有consumer／watcher退休後才修來源，V2整批通過。原始失敗證據保留，不加總兩批數字。

**3372來源／資產**接受前後集合、SHA／size／mtime一致，七項DLL與import library另驗。**100份證據**逐byte／SHA封存並獨立複核；全程監測 **13次**、最大間隔 **171秒**，7最終及5先前worker／watcher與全程monitor皆退出。

## 剩餘

持續引導／脈衝、特殊線／扇形／多段／導引技能幾何與提示，召喚完整可見複本生命期及更多控制，目標專屬名稱／音效與平衡，英雄／建築／地圖美術仍有工作。11項外部ACK／LAN／實際OS DPI／固定效能證據維持待驗，MVP33/44不變；未新增真網路對局、quota reset、commit／push或omfx維護。

P01..P04、T01..T02見2026-10-22-cast-preparation-errors.md；總登錄E1480。完整證據索引：../acceptance/2026-10-22-cast-preparation/index.json。
