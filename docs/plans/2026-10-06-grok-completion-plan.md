# Grok Build 功能收斂計畫（2026-10-06）

## 最終收斂

本批後續已完成strict lifetime並真短程60Hz採樣／owned清理；尖峰兩隊同tick227，幾乎全fixed_step，不能再從decode/hash/report clone猜原因。靜態追蹤發現可確定的reconfigure_thread_pool保留舊cached dispatcher API缺陷；以新的fresh Grok thread限定修快取退役與單一pure測試，避免舊resume巨量上下文。它不在當前game呼叫路徑中，不稱修好50ms；真CPU/等待原因仍需定位。完整驗收不提前重跑。

當前正式25/31，剩4.1/4.3/4.4/6.2/6.4/6.5。接續順序：已接受Grok無界report retention刪除＋primary分段collector11/11 → Grok限定修launcher exact process lifetime與native same-handle stop → primary實際diff／局部確認 → 一次短程正式60Hz分段採樣定位既有client max，不反覆完整驗收。固定門檻與歷史失敗不改；完整UI閉環及LAN最後驗收，第二實機缺口不冒充。E323記錄本批決策與操作錯誤。

2026-10-06使用者最新限制：之後每次場次模擬合計最多10場，不用多批拆分規避。batch預設10、CLI及程式化execute雙層拒絕>10（在build/spawn前）；既有100場/25場分批為歷史證據，保留但不再沿用為未來計畫。

後續已恢復當前engine基線的合法OmGame build/stage及Editor啟動，不能沿用舊BuildId阻塞。主agent補通用binary preflight、project-bound MCP（含真正OS listener owner/lifetime），Blueprint11/11已成功。原生全套找到未初始化UMG fixture，修同檔兩處；完整結果續記project-bound-unreal-progress／E313–E315，不重跑已完成100場。

Grok的HUD/量測/通用Bot與診斷修復經主agent審查及局部確認後，唯一正式100場最終batch已完整完成：四組exit0、主agent逐100份report及summary/unique seed/tick digests/side-team/預算/SHA獨立驗證，6004657 replay ticks；額外layered正式十Botseed101亦自然终局152648ticks/replay。5.4/5.5完成，23/31。對局期間不修改或重建loaded artifact；全部證據和歷史失敗保留，不改預設600，不降低guard、不強制勝負。

後續只剩受阻的完整驗收：相容engine/plugin基線→最新正式統一建置/部署→native/PIE/選角至結算/renderer重連/四段效能baseline；第二台實機→雙機LAN。當前共享engine修改與預編譯plugin版本無法合法視為相容，不重複相同UBT、不手改BuildId、不更動其他專案；不冒稱本機雙程序為LAN。這不是全部完成/歸檔，沒有commit/push。舊下列決策按當時狀態保留。

## 決策

第4批續作43m6s由主agent收回，真實seed1仍timeout而未接受策略。實際後半場仍摧塔與傷基地，600秒是歷史工具硬上限，不是spec的勝負規則；不能把逾時單獨當deadlock。第5批新增明示有界game-time budget／objective-stall診斷與不覆蓋證據，預設600秒保留；只有自然Finished＋完整replay才成功，不能強制結束或調弱遊戲。先一場1800秒操作預算、300秒無objective進展失敗，再依實際結果決定100場，不持續修微小退讓策略。

第5批真實seed1自然終局/replay62951ticks。審查發現報告winner_team為side索引與ECS合法entity0被當空slot，沿同Grok thread限定修正，禁止再調Bot。修復審查及獨立局部測試後，使用既有Lua batch四組25場（seed1..25／26..50／51..75／76..100）平行執行，budget1800/stall300明記；每場仍獨立正式60Hz/10Bot/guard/終局/replay。任一組失敗保留證據，不以其他成功組充100場；四組全部核對unique seed及同一content inputs才勾5.5。既有600秒預設不改，Unreal/LAN另有真實環境block。

使用者要求由Grok Build擔任實作子agent、補齊全部MOBA框架功能，功能各自確認，最後完整驗收。主agent持有整合／審查／執行權責，不將子agent完成聲明當驗收。

既有OpenSpec21/31的10個未勾項同時含「功能」及「完整驗收」；本輪先區分真缺功能與只待驗收，避免一直新增防禦式細節而沒有收斂。

1. 接回中斷的Bot追擊導航，完成局部Rust確認（core2＋base4已通過）。
2. 第1批Grok補齊6.2的通用owned HUD／輸入失效流程，包含Buff snapshot、物品及其他UI入口同類真實缺口；限定手寫C++／native fixture，不改Rust、生成ABI或資產。
3. Codex獨立讀diff、審核與一次scoped UE build；新增native assertions只编譯不能宣稱已執行。必要修復沿同Grok session。
4. 對10個未完成項做實作缺口清單，保留已完成內容，不再依過期partial紀錄判定尚缺；將真正缺口分成有明確完成條件的Grok批次串行實作，主agent做非重疊整合。
5. 功能齊備後統一版本／生成／release部署，再完整60Hz／100場、IPC／重連／UE native／PIE／LAN／效能驗收。不能取得實際環境的項目記真實阻礙，不以本機替代外部LAN驗收。

## 範圍與守則

- 唯一omfue，Rust後端／共用內容與Lua建置工具；不維護、編譯或刪除omfx。
- 只建置求值Lua→compiled Rust／Unreal C++，不恢復runtime Lua、不放寬strict ABI／catalog gate、不新增角色程式或Blueprint graph。
- Grok不commit／push／branch／destructive／credentialed／外部變更，必要操作向主agent升級；主agent不要求使用者冗餘確認。
- Bridge記錄job/session、PID及狀態，不繞衝突guard開多個寫入工作；外部工作流不新增PS／Python fallback。Node bridge是技能協作工具，不替代遊戲Lua工作流。
- 錯誤記unreal-moba-error-register.md，Grok E298、主agent E297／後續段落避免同段覆寫。

## 第1批

- job：run-muvyl79j-i29ns7。
- Grok session：5fefe535-4245-4b18-bd6f-9100d529593e。
- 初次bridge check ready=true、Grok1.0.24、已登入。未列出憑證、未修改認證。
- 交接準備時apply_patch同一file同patch Delete＋Add被拒，改單一Update；首次搜尋猜OmGenerated插件根不存在，改rg --files找到OmRuntime/Source/OmGenerated。結果不算建置失敗，後續先定位、每patch同target單一operation。
