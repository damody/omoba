# Grok Build 功能收斂計畫（2026-10-06）

## 決策

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
