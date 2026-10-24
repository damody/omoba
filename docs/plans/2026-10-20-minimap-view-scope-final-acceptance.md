# 小地圖公開視圖切換接受

2026-10-20。修正同一玩家切換公開視圖或地圖座標時保留舊拖曳與overlay的缺口。二十專項與完整分段本機驗收成功；OpenSpec仍33/44、十一父項待驗。沒有新增權威對局、重設10/10、commit或push。

## 共用修正與實際證據

SOmMinimap::SetState以O(1)比較本機玩家、payload公開player／team與Minimum／Maximum。不可用或context改變時，沿既有RetirePointerCapture只釋放自己記錄的Slate user／pointer，結束inspect並清communication markers／clock與camera footprint。一般marker／fog更新保持同視圖的capture，不逐entity新增工作，也不改OwnerEpoch原有來源綁定與選角契約。

擴充既有MinimapPreferences的真port-zero Runtime及actual Slate路由fixture；同時讓另一使用者與本使用者另一指標持有獨立probe。有效red唯一失敗為Minimap view change immediately retires owned capture。green分別驗公開team、payload player、map bounds變更；每次先驗stable更新保留capture，再驗切換立即取消捕捉／inspect／舊鏡頭框、舊move不移新視圖，fresh press／release仍可操作。其他user／pointer在退休及fresh操作後均保留捕捉。資料初始化為明示受控公開minimap payload，不冒稱真網路換隊、權威ACK或新增對局。

## 驗收工具缺口與保留失敗

首輪二十項只有ResponsiveHudLayout失敗，新minimap測試已成功。原menu wheel fixture只驗事件handled，沒有驗命中預期scroll；新增strict path後仍失敗。修正取點使用paint viewport／window可見交集及UE window-to-desktop幾何轉換，但沒有把轉換當完整根因。before／after BringToFront的actual path最終證明是SOmHudToolTip→OmHudToolTipScroll攔截，長提示本來就可讀可捲動，屬正常行為。

HUD scroll案例現在明確結束前一個互動提示，同步application cursor與真FPointerEvent，從可見捲軸區進行move／wheel，且必須命中指定OmCombatScroll。原游標於fixture結束恢復，tooltip功能未停用；原16次上限、實際offset增加、完整menu label與camera不zoom斷言保留，沒有直接設定scroll offset或放寬字體／裁切條件。讀tooltip與捲HUD分開驗證，完整native仍回歸既有tooltip行為。

四份failed-green及診斷程式FArrangedChildren不支援range-for的失敗建置全部保留；唯讀查找再次使用位置glob的錯誤也已記錄。詳E1403–E1409，未把任何失敗輪稱為接受。

## 完整驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local、無action cache | 正式minimap修正9.11秒；最終fixture版本17.82秒，stage SHA一致 |
| 專項 | 20成功，四筆既有preview-world warning |
| 完整native | fresh206成功，既有warning保留 |
| 完整分段viewport | 458案例／222 wire／157 stages／13 cursor |
| Blueprint／實際音訊 | 69零compiler error／warning／load failure，五PCM成功 |
| Lua契約 | fresh140 owned＋69 local acceptance成功 |
| 來源與資產 | 3335檔前後SHA／size／mtime及集合一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust明示沿用2026-10-19新機接受 |

foundation294案例／120wire／103stages，tail164／102／54及13cursor，保留原860／900秒期限、90 proof frames、8fps與完整清理。strict union再次核對原始native／wire／history／UE index／cleanup、七binary cohort與原始清單；兩段AVI驗RIFF／索引／timeline／幀數／SHA。所有publisher／editor／profile清理成功，三分鐘監測至root退出。公開map另驗144幀／12片段，六HUD與兩map圖逐張檢視；圖只作呈現回歸，視圖取消由actual Slate／camera斷言證明。

45份SHA／size封存，另保留400x300／text2.0實際捲動後選單圖。execution_mode=phased、single_session_full_success=false；不將跨session覆蓋冒稱權威ACK、真LAN、單session效能或實際100%／150% OS DPI配對。十一父項缺證不勾選，歷史接受及失敗包保留原樣。
