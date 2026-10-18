# 鏡頭／小地圖實作進度

完整操作規格：`2026-10-07-moba-controls-presentation-plan.md`，OpenSpec `complete-moba-controls-and-presentation`。整套操作仍未完成；目前先修使用者實際指出的camera/minimap斷線。舊authority10/10不重設，本輪沒有新增真對局。

最新已驗收批次：profile／HUD formal build-v5 exit0（UBT40.21s），full controls-profile-hud-full-v2 native81（failed0/notRun0、13舊warnings）及69Blueprint compile/load零錯誤；render-v3 realSlate PNG2400×1353、方形icon實際geometry PASS，獨立看圖確認名稱不裁切與冷卻遮罩。完整原創atlas prompt與SHA位於project UI/README。此scope無server，positive combat ACK／livePIE與其餘完整操作仍pending。

新runtime改鍵adapter候選（run-muy5l35h-bjx7qc，783s／USD1.85741048）只4檔未promotion；Primary讀全部diff發現identity失效仍可save、camera revision跨owner碰撞、clear不還原settings、apply失敗卻cache成功、改A鍵重設Y lock等，交同thread run-muy67wpz-d3gyta bounded修正。另Primary加入操作設定widget／component source（目前未接入、未編譯），可用action才顯示改鍵／解除／還原；modal採exact player input-stack barrier加camera／owner additive guard，避免使用UIOnly攔住共享viewport的其他玩家。新source不援用上述81PASS背書。

## 已加入source

- SOmMinimap左鍵開始／focus／結束delegate，真Slate capture、drag／release／capture loss與double click；右鍵MoveTo及Shift保留。無效map／owner切換取消inspection。
- UOmCommandBarWidget使用GetOwningPlayer的exact local view target，依公開minimap bounds／設定比例換算camera focus，沒有送gameplay command；HUD retirement／重建／view target切換回收inspection。
- Camera以local PC持有的單一raw input component綁MouseWheelAxis、Space、Y，支持unpossessed view target及native／plain controller；失效owner與EndPlay pop/destroy，不再依賴缺失named axis mapping。
- Zoom按目前distance比例10%且有minimum step，先拒絕非finite值；Space回hero後release回復原free視點；MinimapInspect暫停follow，放開恢復原lock。
- HUD使用actual laid-out geometry攔截panel／shop殘餘wheel，pointer與keyboard gate分離。Keyboard文字focus／結果modal會停camera keys。Camera viewport框由本local projection四角與ground plane交點產生，再以公開bounds裁切polygon，不是固定裝飾框。

這些是實作內容，尚須新測試結果確認，不拿上一版71PASS作新版本證據。Native routed fixture即使成功也不等於真人滑鼠／live PIE。

## Grok4.7Fast與獨立審查

| 工作 | job／thread | 結果 |
|---|---|---|
| source audit | run-muxzys74-heed23／8826a5c0-2d9a-4a23-9d2d-93dc09c0cd21 | 166s／USD0.43520544，唯讀 |
| broad P0 | run-muy0acws-plucgx | Primary859s取消，無source修改；usage不可用 |
| minimap pointer | run-muy0u0pf-l853w3／26d0bc0c-d0db-4074-84d6-3d36af12f7f0 | 312s／12calls／USD0.36035784，Primary review後接consumer |
| camera input | run-muy16o0t-brqg09／aa42d2a8-1f7e-44b9-80a1-21b5652b8a4e | 619s／42calls／USD1.51963816，Primary review並補UPROPERTY／held cleanup |
| next action audit | run-muy1mf72-pegfib／b34ca39d-8c55-4d23-842a-4a6bd1474eb2 | 166s／11calls／USD0.35406104，確認plain controller不能用native-only cursor helper |
| A owner candidate | run-muy21csy-3ogrmq／3a4295c5-e5b7-4629-bf23-93ac8db31e2b | candidate-only執行中，尚未promotion／審查／驗收 |

Primary另做camera模式／local resolution、HUD consumers／guard／footprint與三項native tests。Candidate隔離於`.ai-collab/candidates`避免與formal build競寫source，不當成已安裝功能。

## 正式建置

第一次normal build v1於219.75s因EngineMetadata既有outputs被NoEngineChanges拒絕，保存原log。正式refresh v2預設executor長時間未完成action，以exactowned supervisor／creation token回收私有Job，cleanup驗證通過，原checkpoint保持。

已補通用Lua流程：明確local executor也可套用source-engine initialization／refresh，SCW與OmGameEditor均帶官方-NoUBA；正常NoEngineChanges與SHA檢查保留。Source-engine admission／command tests實際exit0，local+verify實際拒絕。

`controls-camera-refresh-local-v3` **exit0**：SCW2.31s、source OmGameEditor302.28s、normal NoEngineChanges3.82s。stage SHA：

- bridge：`6cb1145683ccc10d4a21e8dc73e416c164bbbfa9ac041ebb03ce5aa9d0b8bd0d`
- content：`1fc65608a4fb46d3aa9db5f531289256f55e35ac3a606abe3e9b3d6c951b249c`

Log：`.ai-collab/.bridge-data/worker-pool-machine-v1/controls-camera-refresh-local-v3.log`。沒有手改Engine source/config/BuildId或關掉正常gate；正式source build自己產生一致metadata。舊動作退出／failed logs與E362／E363保留。

## 本機驗收進行中

固定Lua `ue_local_acceptance.lua --run-id controls-camera-p0-v1`，從source動態找到74native tests，再驗既有69Blueprint。新三項是OwnedCameraInputStack、MinimapCameraRoutedInput與MinimapCameraFootprintClip。Native stack檢查raw key→effective camera component距離；Slate fixture使用production HUD real widget path／實際layout→cameraXY／capture／follow／retirement，range polygon裁切檢查enclosing-view與diagonal-edge。

原full-v1／v2及scoped-v3..v6的fixture失敗／crash均保留，原因與修正列E364；不可當PASS。正式v9建置exit0後，`controls-camera-native-v7` exact3測試全部Success、0warnings、0authority games。

`controls-camera-p0-v3` 完整驗收 **success=true**：74native全部完成（67Success、7SuccessWithWarnings，共13個既有warning；failed0、not_run0），69Blueprint載入與編譯全部通過（compiler errors0、warnings0、failed loads0），native及Blueprint程序exit0。依exact report完成tasks2.1／2.2／2.3；actual viewport／DPI、完整camera shortcuts與A／combat／HUD／visual仍pending，不以native fixtures代替真人PIE。

完整報告：`omfue/Saved/Automation/LocalAcceptance/controls-camera-p0-v3/summary.json`。範圍仍是local-native-and-existing-assets，未使用新authority場次；original10/10保持不變。新A候選run-muy21csy-3ogrmq完成1758秒、173calls、USD7.85498056，未編譯、不等於安裝；後續run-muy37g1c-rhec6x接入與具體修正中。

## A owner、快捷鏡頭與當前完整驗收

Grok integration run-muy37g1c-rhec6x完成734秒、58calls、USD2.84768768；Primary獨立讀source並修正式編譯錯誤E365。已promote UOmPlayerActionOwnerComponent與WorldBridge consumers，不改BP_PlayerController父類或Engine。A arm／world左confirm／右cancel／Esc、Shift+A即時unqueued、公開owned hero range ring、focus／view target／death／connection gate、單一input component与terminal retirement均已有source。不能把未執行的positive frontend command／server ACK說成已驗收。

另補raw方向鍵平移與F1回自己、Y／文字keyboard guard回歸、Native HUD wrapper及root overlay的SelfHitTestInvisible，保留interactive children。完整2.5仍缺middle drag／F2–F5；2.4／2.7的actual viewport與DPI矩陣未完成。

Owner ID舊解析抽digits並fallback1已改為正整數／player_<id>完整驗證，invalid／zero／negative／overflow返回0；正常player_1預設保留。新增ConfiguredPlayerIdentity及ActionOwnerLifetime（真正game-preview lifecycle／registered local player／real UPlayerInput與stack，沒有initialized runtime或server），並補EndPlay清理，不mute新warning。

正式 `controls-a-owner-build-v4` local／NoEngineChanges **exit0／326.57s**；stage SHA與上列一致。`controls-a-camera-native-v2` exact5全部Success、0warnings、0games；`controls-a-camera-full-v2` **success=true**，76native全部完成（69Success、7SuccessWithWarnings／13既有warnings、failed0／not_run0）、69Blueprint compiler errors0／warnings0／failed loads0，兩程序exit0。此次包括新的Y／arrow／F1、owner生命周期與ID回歸；不取代真人PIE或positive A ACK證據。

版本化profile候選 run-muy3pp7q-lnqtly／thread8153b5de-9bd7-4984-aa87-d4ab8e2f8f2f 執行中，僅寫候選，尚未promote／編譯／接入；全套戰鬥／DF／ward／chat／presentation其餘task保持pending。
# 後續 HUD／profile 批次（2026-10-07）

完整功能清單與44项驗收仍以 `2026-10-07-moba-controls-presentation-plan.md`／新change tasks為準；本紀錄不把候選或native fixture標成全部對局完成。

Grok4.7Fast profile candidate run-muy3pp7q-lnqtly完成：1549秒、88calls、USD4.24580848；Primary發現save搬走原檔造成非原子missing-file窗口，已啟動同thread bounded repair run-muy53kmy-vzv7am，修OS atomic replace及unique temp、補production回歸，尚待正式編譯。原78-action catalog不是78個已接線功能，Stop／D-F／ward／chat權威backend仍pending。

原創4×4 HUD atlas已存project `omfue/Content/Omoba/UI/omoba-hud-icons-v1.png`，完整prompt與SHA記README。Primary新consumer從typed AbilityHudState取ID／rank／cooldown，原文字、Ctrl提示和長名稱保持；未知ID不借用別技能圖，invalid／retired狀態清除。RawPNG以UFS stage設定保存。另native controller將H／B／Ctrl upgrade的keyboard guard與pointer region分離，text／modal仍阻擋。兩批source和新增native tests尚未編譯；新fixture的實際Slate PNG亦待執行，不能援用前版76PASS替新source背書。

# 當前增量：runtime profile／操作設定接線（尚待驗收）

Grok4.7 Fast修正run-muy67wpz-d3gyta（907s／USD1.24993792）已獨立review及guarded promotion。新增設定widget/component：Esc優先讓A瞄準取消，否則開設定；HUD「操作設定」按鈕保留解除Esc後的入口；僅列真實接線十個action，支援keyboard／mouse chord、modifier、解除、衝突拒絕、原子保存後apply、restore defaults。modal以本地input stack barrier阻擋遊戲，camera guard獨立於每tick HUD setter，runtime身份失效不寫舊player檔。新增CameraProfileInput實際InputKey→input stack→camera regression。current正式build pending，不以之前81/69替此增量背書。

下一Grokrun-muy7402y-z238jl／thread6bccc016-0d22-472a-804c-08d09892b766候選中鍵drag與F2–F5，同profile與exact local viewport，隊友位置僅來源已公開scoreboard／current minimap marker。candidate-only，無production覆写／真對局。原10/10未重設，完整44項仍4勾選，未宣稱整套操作已完。

