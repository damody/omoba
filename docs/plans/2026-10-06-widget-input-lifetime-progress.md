# 原生 Widget 輸入退役

## 本批計畫與決策

1. Controller ending guard 不涵蓋獨立商店／小地圖提交入口；retained widget／Slate callback 不得在退場後利用仍連線的 runtime 送出操作。
2. 使用 widget-local、可重複呼叫且每個 instance 終止性的 RetireGameplayInput。NativeDestruct 與 Controller EndPlay 同源退役，不因 RebuildWidget／late setter 恢復輸入。新的 HUD 使用新的 widget。
3. 退役入口先於 runtime query／全域 HUD invalidation，以免舊 widget 反而清掉新 controller 的 HUD。不停止後端或 shared runtime、不改 ABI／生成內容／Blueprint。
4. Grok Build job `run-muwgj9gv-20tqlf`，thread `c231d947-aadf-4c24-b18b-8df4516a02d2`；限定 widget h/cpp、Controller 單一清理呼叫及一個原生 fixture。Primary 保留上一批三檔變更、獨立 review／scoped compile／紀錄。

## 確認邊界

只確認目前功能；不執行對局模擬、完整 PIE、效能採樣或雙實機 LAN。不勾完整 6.2。

## 實作與實際結果

- Grok 3m35s 無差異，取消後 follower cancelled／三tracked PID null／原 agent107540與bridge29632不存在；primary才接手。成本／API／確切停滯根因未知，不算Grok交付。
- Primary實作RetireGameplayInput／NativeDestruct／Controller顯式退役、CanSubmitShop及兩個submit入口early gate、晚到economy／minimap更新只保留空baseline。純builder／Rust玩法／hash／ABI不改，不查或停止其他玩家runtime。Review將值型temporary改const reference，正常HUD不增加整份陣列copy。
- UE5.8 `-Module=OmRuntime+OmGenerated+OmEditor -NoEngineChanges` 11actions成功20.53s；copy修正3actions成功6.62s；fixture修正3actions成功11.04s。固定Lua runner語法與root／omfue whitespace成功；既有plugin dependency warnings保留。
- 首次原Editor32884已ready，但既有runner把absolute out-dir接在root後而失敗，native尚未執行。正式runner改path.absolute並依解析後路徑擋覆寫完整驗收目錄。
- 第二次原Editor78528的新fixture漏Initialize而crash。真正CrashContext＋已安裝llvm-symbolizer/PDB指向FOmWidgetInputLifetimeTest:2287與UMG WidgetTree，不當MCP暫時斷線重試。修fixture初始化，不改正式UMG／輸入gate。
- 最後`WidgetInputLifetime-1791278313`原Editor78512，透過project-bound MCP實際執行 `Om.Runtime.ControllerOwnedHudLifetime`／`Om.Runtime.WidgetInputLifetime`，單輪2/2 passed、failed/skipped/not_run=0。前者explicit EndPlay／parent attachment，後者idempotent retirement／late state／初始化後Slate construction／商店及小地圖拒絕／其他instance保留；不是viewport／Actor Destroy／完整對局證明。
- 最後Editor正常LogExit，但cleanup原身分查詢在退出期間出access denied，原report保持success=false／原始錯誤不改。之後獨立成功original-token query回false、OS查原PID不存在，另存`retirement-confirmation.json` success=true；不把failed query當退出，不再跑已通過的native。

原始證據：`omfue/Saved/McpAutomation/WidgetInputLifetime-1791278313/native/report.json` 與同目錄 `retirement-confirmation.json`。使用者HEAD02757保留，既有assets／slnx／Publisher檔不改，無commit/push／omfx維護。OpenSpec25/31仍待完整選角至結算、真雙機LAN與固定12項基線封關。
