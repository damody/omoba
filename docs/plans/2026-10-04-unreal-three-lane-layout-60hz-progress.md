# Unreal 三路公開地圖／60Hz 進度

## 本輪計畫

- [x] 核對現有共用route actor／小地圖與IPC缺口。
- [x] 公開map/moba-layout schema1包含compiled map id與catalog hash；runtime bootstrap與bridge ready皆拒絕未知ID／錯配hash／重複metadata／超量／錯版資料。
- [x] RuntimeReady additive protobuf7／8與checked-in prost fallback一致；正式ready_envelope_for_start接startup／retained ready，晚連線與重連測試走同一路徑。
- [x] bridge以相同Lua生成常數還原完整三路，full與lifecycle重建共用函式，HUD lane_length不同則拒絕，缺HUD清route，legacy None保留舊單路。
- [x] 增加--three-lane的獨立60Hz雙UE移動／路線／小地圖驗收模式；其他單路fixture明確拒絕混用。
- [x] 完整回歸、真實雙UE、建置／stage獨立SHA核對。
- [x] 保存MD／OpenSpec最後結果。

## 決策與範圍

Unreal已有共用MapRouteActor、frame routes與Slate小地圖，不需要新增角色C++或Blueprint graph。原bridge只根據lane_length畫直線；改以後端公開bootstrap指定的id／hash認定正在使用的map，再從共同compiled Lua catalog讀lane waypoints。地圖身分不能根據hero位置／player ID／story猜測。

map catalog已包含terrain與camp定義，但本輪只使用靜態路線；不傳camp entity／aggro／timer，collision terrain與vision occluder維持分離。前端沒有第二份MobaMatch或NPC AI。world路線為示意標線，仍不是導航mesh、美術地表或完整正式地圖bounds。

--three-lane支援一般互動，bounded smoke只驗60Hz基本移動與map／minimap。商店／升級／回城／終局等既有single_lane煙霧劇本本輪不混用，不把改名當作那些三路操作已驗收。

## 已確認結果

- core340 passed；server156 passed／1 ignored。
- runtime61 passed／8 opt-in ignored＋integration3；晚連線／重連真實TCP保留新ready byte-for-byte。
- bridge54 passed／1 opt-in ignored＋integration2／1 opt-in ignored；三路正負Y／waypoints精確、無hero位置推導、full/lifecycle／缺HUD清除／長度拒絕通過。
- Lua三路route observer8 assertions、minimap observer14與既有two-team觀測器全部通過。
- base_content最後完整回歸108 passed，214.43秒。
- OmGameEditor build-only Result:Succeeded；獨立stage bridge SHA-256 `90bfc4a4456a27f891413dca0c79615e33b21d2b544fd6ccba5cf4e2d6ba2bbd`。
- scripts/target/debug、scripts根目錄、UE Binaries/Win64三份base_content.dll SHA-256一致：`65d6e154e434517a38497f7c09f26e791f896e4512da84c2ebb173e3647ac50c`。

## 真實雙 Unreal 60Hz 結果

`target/interactive-runs/interactive-ue-1791115215/unreal-ipc-smoke-report.json`：success=true、cleanup_verified=true、gameplay_mode=three_lane、tick_rate_hz=60、debug／D3D11。報告SHA-256 `6fe6066bd11a04b7d07f54a940086edb4a6ccd816ddda6dfeb36dfb0eefdb37d`。

- 雙隊實際UE皆建立route_0／1／2，點數4／2／4；native小地圖routes=3、owned=1，bounds=(-480,-1680,2880,1680)。
- team1呈現座標(120,0)→(-400,-400)，team2(2280,0)→(1887,0)；均消費正式快照與owner HUD／economy。team2只證明移動，不聲稱已抵達目標。
- 保存three-way-checkpoints.jsonl獨立核對：team1 62 PASS rows／46 unique ticks至5520；team2 70 PASS rows／45 unique ticks至5400，零FAIL且所有已驗checkpoint pre/post parity=true。不是每個網路tick均有checkpoint。
- 五owned PID（server40444、runtime87328／107584、UE78276／78352）另以Win32_Process核對皆已退出。
- 使用--three-lane、OMOBA_UE_STEP_FPS=60、OMOBA_UE_SMOKE_SECONDS=60、OMOBA_UE_MINIMAP_SMOKE=1；一般移動smoke走正式delegate，不是OS鍵鼠。未取PNG、未測LAN或穩定60FPS，沒有營地／地形美術完整驗收。

初次run1791114846失敗，保留原報告：第一位玩家的KCP空bootstrap漏map metadata，第二位玩家第三條route建立時unique-name fatal。修正空bootstrap外／內metadata同時帶compiled map與collision terrain；共用route actor將MakeUniqueObjectName outer與SpawnParams.OverrideLevel統一為PersistentLevel。重新build／stage後上述雙UE通過，未靠調高timeout或刪除第三路。

## 錯誤

新增protobuf欄位時舊RuntimeReady fixture缺欄位E0063，補Default；錯誤與防重犯規則集中E131。保存歷史未完成說明，不據局部編譯成功勾選完整5.4／6.1／6.2。
