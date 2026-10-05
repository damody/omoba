# Unreal 選角入口原生功能確認與單前端維護範圍

## 本批計畫與決定

1. 不再把 E177 當永久阻礙：只讀確認現有 Editor／引擎基線，再以既有 restart build、保留 -NoEngineChanges 做原生增量編譯。引擎四個現有來源修改全部保留。
2. 修正真正 C4458：AOmPlayerController 的區域 Player 改 SelectionPlayerId；沒有取消嚴格警告。
3. 選角 Initialize 不 LoadBridge，StartRuntime gate 與 world actor 自動生成 gate 仍保留，前置畫面不依賴 gameplay DLL 或現有對局。
4. 用非 Shipping、明確 opt-in 的單项確認，經真正 Slate 按鈕／cached geometry／pointer down-up 回呼觸發既有 Send，再由 Rust 選角服務處理。沒有直接呼叫 kernel、虛構 geometry、強制 enable 或建立對局。
5. 首次 touch 回呼未觸發請求：預設 DownAndUp 需全域 capture。選角採與既有商店一致的 PreciseTap，支援捲動區精確觸控釋放；滑鼠策略不變。自動化失敗會退出，正式使用者錯誤仍留在畫面。
6. 使用者新增範圍要求：後續只有 omfue 前端需要維護。AGENTS.md／README／OpenSpec 同步；omfx／Fyrox 保留歷史檔案，不修、不建、不驗收，不新增 shared API 相容接線。

## 當前功能結果

- 第一次 build：已通過引擎閘門，因 C4458 失敗；紀錄 `omfue/Saved/Logs/selection-build-1791181104.log`。
- 修正後原生 compile／link 成功；最後 PreciseTap 版本建置 `selection-build-1791181625.log`，4 個 project actions，Result: Succeeded。沒有建置或還原共享引擎來源。
- 首次單項 run `target/unreal-selection-tests/1791181448-1` 失敗：handshake 成功但 request=0；120 秒總期限後只清理自己的 renderer，保留 errors.md 與 log。
- 修正後單項 run `target/unreal-selection-tests/1791181633-1` 成功：指定真人 1／protocol1 ready；select／lock／finalize 各一次、request 1／2／3、pressed／released／submitted_once 全為 1；最终 training_ranger 配方與來源保護通過，gameplay_started=false。
- 原生確認入口：`tools/lua/lua.exe scripts/test_unreal_hero_selection.lua [HERO_ID]`。它只確認本功能，不啟動 server／client runtime／正式對局，且保留獨立輸出；不是全套驗收入口。
- OpenSpec strict validation 與主 repo／omfue whitespace check 通過。

## 限制與下一段工作

- 成功的是原生選角程序→Rust service→配方交接，不是 OS 真人滑鼠、整個 hit-test grid、美術像素驗收或正式選角到結算對局。
- 單真人＋九 Bot 本機入口已確認；共享 LAN 多真人選角仍未實作。
- 先前 E212「C++ 尚未編譯」與 E177 阻礙是歷史狀態，現由本批結果更新。正式 DLL 部署／完整遊戲建置與對局、LAN／效能驗收不在本批，最後集中執行。
- 6.2 全項仍未完成，整體 21/31。不維護 omfx、不刪使用者檔案、不 commit／push。
- 錯誤與預防規則記 E213。
