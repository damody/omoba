# 原生小地圖座標輸入：60Hz 部分完成

## 本段計畫與決策

1. 在已完成的安全小地圖上補座標逆投影；保留原有公開route bounds／aspect／Y inversion，拒絕留白、NaN、零尺寸、無map，不clamp成假邊緣位置。
2. 通用SOmMinimap只接受右鍵Move、Shift排隊，不做entity點選或actor查詢。所有小地圖按下與放開都Handled以避免穿透；只有有效right-down呼叫一次委派，左鍵／release不提交。widget改Visible以接收座標事件。
3. 原生bar使用configured player與WorldUnitsToCm，runtime Connected才走原有SubmitGameplayInputEvent。Point target沒有EntityId，GameplayInputEvent.bConsumed仍false（它不是Slate的Handled）。合法性／地形仍由Rust權威處理；示意bounds不是導航資料。
4. 非Shipping opt-in以真實cached geometry合成right-down，呼叫既有Slate handler→UObject callback；不得直接呼叫玩法API替代。要求一次回呼、原input ID accepted／status0，並核對實際逆轉點與預期誤差≤0.05 backend units、UE移動與移動後三方hash。dispatch失敗停止，不換ID重送。
5. UBT cwd大小寫切換會先進TEMP，本機共用Temp鎖定造成首次build17732失敗。防護收進既有Lua build workflow，僅om_restart build子程序使用Saved/BuildTemp/ubt，未改系統環境／引擎來源、未刪Temp／殺其他專案程序。詳細錯誤E099。

這是通用前端支援，不新增角色專屬C++／Blueprint graph或Rust測試專用入口；本段未修改Rust玩法。

## 最終驗證

- 固定Lua：新minimap observer14 assertions（含JSON round-trip）；既有雙隊6、ability7、parity3、match8 scenarios；shop12、shop button12及JSON；bridge stage一致／不一致／缺檔fixture通過。修改workflow皆loadfile parse通過。
- 完整build60172 exit0，UBT10.36秒、content identity de9c7fcfc98d6479；新C4701警告已明確初始化修正，最後C++編譯未再出現。既有Rust unused／plugin dependency warnings仍在，不宣稱全專案零warning。
- staged bridge SHA-256：d99f2df046655fe2060f6b4a66b13580809badf19b341c6c8db5f0467d2f44b3。最終stage-only再次核對。Rust測試沿用前段core322／base73／runtime49+3／bridge47證據，本段沒有重跑這些suite，不把舊結果算新結果。
- Editor91416同session兩輪各11/11，包含新Om.Runtime.NativeMinimapInput：inverse／letterbox／NaN／非法player／scale、configured Point輸入、不Consumed／無entity、Shift、真實Slate handler單回呼／left／release／offline。現有EditorPreview DestroyActor無context warnings仍保留，無test error／skip。
- 串行PIE exit0，已停止；截圖已檢視，小地圖／bar／shop可見。本次fixture HUD為120Hz／120FPS，非正式60Hz雙UE效能證明，不混用兩種runtime模式。Editor正常Save／QUIT且CIM查無。
- 最終正式run：interactive-ue-1791058752，launcher53155 exit0，success=true／cleanup_verified=true；release server＋2 filtered runtime＋2 UE -game，network60Hz、presentation30Hz、UE cap60。

| 證據 | 玩家1 | 玩家2 |
| --- | --- | --- |
| 實際回呼 backend point | (480,288) | (1920,-288) |
| handled／callbacks／accepted／原input ID | 1／1／1／1 | 1／1／1／1 |
| 原ID applied status／tick | 0／2452 | 0／3050 |
| UE own-only movement觀察frames | 21 | 16 |
| 最後safe tick／Consumed snapshot | 3212／795 | 3218／1388 |
| 三方checkpoint PASS／last tick | 54／3240 | 54／3240 |
| FAIL／非checkpoint UNVERIFIED | 0／6543 | 0／6543 |

- 双native map／owner HUD／economy／own-only movement通過，UE與runtime抵達回呼座標；checkpoint3240晚於雙隊移動觀察。UNVERIFIED沒有expected，不能算PASS或宣稱每tick權威驗證。
- 五PID89504／6312／20172／636／72452皆CIM查無；前一run1791058594亦success／cleanup，hash55／56 PASS至3360，但它早於最後座標log補強，只保留為前一版本證據。
- 指定變更檔root與omfue repo的diff --check通過；既有LF／CRLF warnings未大範圍改寫。不提交暫存產物。

## 重現與證據

固定Lua依序執行scripts/build_ue_moba.lua --full、scripts/ue_native_visual_smoke.lua（等兩輪complete）、scripts/ue_pie_smoke.lua，再正常Save／QUIT Editor。不要把啟動Editor的整個workflow包在另一層capturing process.run中，以免繼承pipe延後返回。

正式雙玩家設定OMOBA_UE_MINIMAP_SMOKE=1、OMOBA_UE_MINIMAP_MOVE_SMOKE=1、OMOBA_UE_SMOKE_SECONDS=90、OMOBA_UE_STEP_FPS=60、OMOBA_RELEASE=1、OMOBA_SKIP_UE_BUILD=1，再執行tools/lua/lua.exe scripts/run_2player_ue.lua --single-lane。不能同開ability／match／shop smoke。此次OMOBA_SKIP_BUILD=1僅因前段release已重建相同Rust來源；Rust變更後不可沿用。

證據位於target/interactive-runs/interactive-ue-1791058752/unreal-ipc-smoke-report.json、logs/ue-p1.stdout.log與ue-p2.stdout.log、server/three-way-checkpoints.jsonl；Editor報告omfue/Saved/McpAutomation/NativeVisual/report.json、pie-smoke-report.json。這些皆暫存，不提交。

## 尚未完成

6.2只增加部分進展，總數仍17/30；未驗收OS／真人滑鼠或完整hit-test grid、Shift權威多命令序列、攝影機拖曳／entity目標、fog／memory、正式導航bounds、五人隊伍／三路、選角到結算與計分板、LAN與完整frame-time效能。60Hz網路整合成功不等於全框架完成或每frame達60FPS。

後續優先補玩家端完整操作與選角／結算UI，再處理正式地圖資料與LAN；錯誤與防重犯記錄見unreal-moba-error-register.md E099。
