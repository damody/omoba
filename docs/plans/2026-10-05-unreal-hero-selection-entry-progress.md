# Unreal 原生選角入口與開局交接

## 本批計畫與決定

1. 接續 6.2，以通用原生 Slate／UUserWidget 顯示 Rust 回報的英雄目錄與席位。没有英雄專屬 C++ 或 Blueprint graph，也不新增 Lua VM。
2. `-om-hero-selection` 是獨立前置模式：controller 不建立戰鬥 HUD／不綁 gameplay bridge，world subsystem 不自動生成 bridge actor，`StartRuntimeFromSettings` 即使被其他呼叫也拒絕啟動。選角不建立 Rust gameplay World。
3. `UOmHeroSelectionWidget` 啟動可信路徑的 `moba-config --selection-session`，三組匿名 pipe 分隔 stdin、stdout JSON 與 stderr。使用 byte framing 再轉 UTF-8，避免中文字跨 pipe chunk 損壞；每份回覆最多 1 MiB，等待回覆逾時 10 秒失敗封閉。
4. 按鈕只發 select／lock／finalize；英雄有效性、鎖定、全員 ready 與配方由 Rust 決定。UI 依回覆關閉不適用按鈕，仍由 Rust 做最終拒絕。
5. Rust 回覆新增 decimal `revision_token`／`request_id_token`；舊 numeric 欄位保留。C++ 不將 u64 revision 經 double 轉換；驗證 token 並直接輸出 JSON 整數，請求 ID 使用有上限的 u32、逐則相關 ID 檢查。協定仍是相容的 protocol1。
6. 只有明確 finalize 回傳 plan 才以 NoReplaceExisting 寫 `finalized-reply.json`，之後退出選角 renderer。關閉視窗沒有結果，不自動鎖定、不猜預設英雄、不開局。只關閉 widget 自己建立的 process handle。
7. Lua launcher 加入 opt-in `--interactive-selection`：先建置正式選角 binary 與 Unreal，再啟動選角 renderer；等它退出後讀回覆，檢查 protocol／身分／hash／ready／finalized，及除了該真人 hero 外整份配方保持不變。保存一般 JSON、清除舊 CLI 英雄 override，沿既有 Rust preflight／server／client runtime／Unreal 正式開局流程。
8. 一位真人＋Bot 是此本機入口的明確限制；多真人 recipe 拒絕，不用各開一個獨立狀態冒充共享大廳。沒有改既有預先准备模式為強制互動，避免未驗證 C++ 阻塞其他工作。
9. 發現既有 lifecycle loop 使用工具庫不存在的 `time.sleep`；修正成 `time.sleep_ms`，並同步 mock API。這是 API 名稱修正，不新增平台 fallback。
10. 舊 Unreal binary 可能不認得選角旗標；額外要求 presentation-only 防止 legacy embedded World，並在 45 秒內讀到含真人身分／protocol1 的 OM_SELECTION_READY。握手未到則只停止本批 renderer 並記錄錯誤，不一直等；成功後的玩家選角時間不設此限制。

## 當前功能確認

- Rust service 4/4 局部測試成功（新增 token 輸出相容欄位）。
- 正式 compiled-content-only 的 moba-config debug binary build 成功。
- 新 `omb/tests/moba_selection_process.rs` 真正啟動該 native binary：初始 catalog／flush、逐次 select／lock／finalize、相關 token、最後配方及 EOF 退出，首輪 1/1 成功；不是 mock，也沒有啟動 gameplay World 或 Unreal。
- 固定 Lua 的 `scripts/test_moba_hero_selection.lua` 初輪 5/5 成功：單真人更換與配方保護、renderer 退出後交接、清除 CLI override、取消不開局、篡改配方拒絕並記 MD、互斥模式。新增第六項檢查舊 renderer 未握手的啟動逾時與只清理本批程序；最終結果補於下方。renderer 部分是 mock，不能當 Unreal 畫面證據。
- 主 repo／omb／omfue whitespace check 成功。
- 最終 Lua 交接 6/6 成功（含舊 renderer 啟動握手逾時）；Rust native binary 測試補強 EOF 退出等待後再次 1/1 成功。OpenSpec strict validation 成功。spawn 與 owned-process 記錄也納入同一清理範圍，清理失敗一併記 MD，不讓原始錯誤蓋掉清理診斷。

## 尚未完成與限制

- 新 C++ widget／controller／bridge gate 尚未編譯或在畫面執行。上次已知共享引擎 E177 仍是這段的驗證限制；本批不重跑完整建置／驗收，不修改或清除共享引擎變更。
- 本批不是完整選角到结算驗收，也不是 LAN 共享大廳。6.2 仍不勾選，整體 21/31。
- 後續解除原生建置限制後，確認此入口一次，再於全部功能完成時做完整對局／LAN／效能驗收。
- 問題與預防見 E212；沒有 commit、push、stage DLL 或清理使用者檔案。
