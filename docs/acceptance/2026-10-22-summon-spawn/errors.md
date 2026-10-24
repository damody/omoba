# 召喚生成錯誤紀錄

- S01：現有 spawn_summoned_unit 回傳 INVALID，SpawnUnit outcome 未處理。fixture 的召喚元件不代表正式技能已生成；本輪補實際入口。
- S02：PowerShell 不會展開傳給 rg 的檔名 glob，另誤猜 native/mod.rs。後續僅傳實際目錄，使用 rg --files 或 rg --glob 篩選，保留失敗搜尋紀錄。
- S03：OpenSpec CLI 前輪已確認不可用；依現有 skill 維護 proposal/design/tasks，使用者已授權自行決定，不重複詢問。
- S04：apply_patch 同一檔案不能在一個 patch 同時 Delete/Add；改成一次完整內容寫入，沒有刪除來源。
- S05：第一次編譯發現 game_processor 明確 import 清單漏掉 UnitType/BountyType/SummonedUnit，summon_tick 漏 Fixed64。補 import；失敗 CheckV1 原始 log 保留，後續工作等待其 worker/watcher 退休。
- S06：原碼檢查確認 specs 的 atomic delete 直到 World::maintain 才退休；取消測試須在 maintain 後驗證 is_alive，不能假設 delete 立即改 generation。已修正測試順序。
- S07：RustV1 的新 core 測試誤由 runtime 根目錄匯入 TickProfile（實際已在 comp glob）。移除錯誤匯入；保留 V1 失敗，完整後續 cohort 須等待其退休後重新驗收。
- S08：RustV2 core 833 通過／13 失敗，舊 shop／summoner 隔離 fixture 沒註冊 SummonedUnit，新增 digest 讀取時 panic。補齊兩個共用 fixture，不弱化正式 digest；並補召喚 Unit／Facing／ScriptUnitTag 的未來行為狀態。舊 build/world/source 基準標記 superseded，最終完整 cohort／建置／World／來源基準全部更新。
- S09：WorldV1 與最終 WorldV2 均 33 成功，均有 4 筆既有 preview-world DestroyActor context 警告保留，不因警告停用測試。
- S10：CoreCheckV2 新增 digest tag 時漏明確路徑，編譯拒絕 ScriptUnitTag；改用 crate::runtime::scripting 完整型別路徑，保留失敗 log。後續先完成 core 全套再建置，避免把未編譯修正帶入接受。
- S11：CoreCheckV3 為 843 通過／3 失敗，summoner 的最小 stop-command world 另缺 Faction。一次檢查 digest 所有 storage 與 fixture 註冊表後補齊；不以略過 digest 或放寬測試解決。
- S12：額外命令列彙總誤把 Lua pattern 的 `%` 寫成 `%%`，導致匹配不到並 assert；這不是測試失敗。改回單一 pattern escape；封存工具原本使用正確 pattern，原始測試 log 不變。
