# 共用選角本機多人 launcher（2026-10-05）

## 計畫與實作決定

1. 保留單真人流程；配方2..10真人由固定Lua啟動一個Rust選角host及每真人一個Unreal。所有玩法／選角規則仍由compiled Rust與原生C++執行，沒有runtime Lua或英雄專用Blueprint。
2. renderer不自行製造開局配方；每席位存完整合法終局reply收據，追隨者可能收到stale error／null plan，但必須帶相同finalized快照。
3. Lua檢查shared_room握手、固定身分、內容hash、ready／finalized／全部locked及整份配方；只允許真人英雄变更，保護Bot、隊伍、角色与玩法規則。
4. 任一取消／不符／逾時在gameplay前中止；本輪owned程序反向逐一依PID＋expected exe清理，各項錯誤獨立保存，不停止無關程序。每次spawn存不可覆寫owned-process-N.json，失敗紀錄於selection輸出errors.md。

## 入口與交接

沿既有 `tools/lua/lua.exe scripts/run_moba_role_ue.lua --interactive-selection --recipe <配方.lua或.json>`。
配方的bot=false決定真人席位；多真人自動共用loopback動態port，不需要各自複製房間。每席位使用自己的player-ID.json邀請；Lua只確認檔案存在，不解析／記錄token。

選角輸出為原output加 `-selection`：candidate.json、host/ready.json、各席位terminal reply／log／UserDir、ownership紀錄與最終match-plan.json。host是唯一finalized-plan.json來源；全員與host正常退休才更新options.recipe，清空初始hero override，再交给既有prepare／正式60Hz server-runtime-Unreal launcher。沒有在選角期間建gameplay World或偷偷鎖定。

無收據退出立即取消；有合法终局收據但host檔稍晚可見才等最多5秒。握手45秒、renderer終局退出15秒、host退休5秒；真人思考時間不設自動同意timeout。原子host發布仍沿E244，沒有讀半份JSON重試fallback。

## 當前功能確認

- 新Lua13/13：完整配方防竄改、stale終局、成功交接、發布延遲、取消、host退出、舊握手、hash不符、Bot篡改、錯身分、缺收據、發布逾時及cleanup注入。成功案例另核對三份ownership紀錄；各失敗必須是預期原因。
- 所有程序皆為mock，沒有執行Unreal／正式對局；原單真人相鄰6/6成功。
- 原生收據修改的OmRuntime／OmGenerated／OmEditor scoped build：3 actions，5.31秒，Succeeded；日誌 `omfue/Saved/Logs/native-shared-selection-receipt-modules-20261005.log`。只已編譯，不宣稱UI assertions執行。
- 首輪禁止覆寫失敗與修正記E245；負向cleanup錯誤是刻意注入。沒有新增shell fallback／根目錄bat，沒有stage二進位／commit／push，也未維護omfx。

## 剩餘界線

E224的engine/project BuildId仍阻礙完整Editor執行；本輪沒有重跑完整啟動、修改BuildId或取消NoEngineChanges。兩台LAN邀請交付、選角至正式server／runtime、完整畫面／對局／效能仍待最後整合驗收。完整6.2／6.4保持未勾選，OpenSpec仍21/31。
