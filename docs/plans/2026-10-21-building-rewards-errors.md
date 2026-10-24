# 建築獎勵錯誤紀錄

- B01：OpenSpec CLI既有環境不可用，手動維護本輪proposal／design／spec／tasks；不宣稱CLI validate。
- B02：查詢曾猜測damage.rs、native/mod.rs、comp/faction.rs與runtime/outcome.rs，實際傷害入口為native/game_processor.rs，元件為native/comp/unit.rs與comp/outcome.rs。已改先rg定位，不建立fallback或重複查錯誤路徑。

- B03：RustV1 core824通過／4失敗；新測試使用process_outcomes但未像正式fixture安裝TowerTemplateRegistry，於Death流程fetch時panic。補fixture資源，不更動正式Death流程；V1保留並以新cohort重跑。另移除新測試的unused mut。

最終RustV3 1670通過／10既有ignored（core828、base247）；UE5.8／後端build與World30成功。V1／V2保留為失敗／取代紀錄，未混入最終cohort。完整結果見2026-10-21-building-rewards-final-acceptance.md。

- B04：RustV2 core828通過，但base release link報LNK1104，V1同名測試EXE仍執行中。Windows不可在舊測試process存活時重連結同一EXE；後續cohort必須等待前兩批完整退休再啟動，不能僅等待Cargo編譯鎖釋放。此為驗收排程錯誤，不修改遊戲規則、不刪除執行中EXE；保留V1／V2並改用串行V3作最終接受。
- B05：Saved World helper複用時曾保留舊supervise_workflow self path；本輪在既有owned workflow job內執行，該path參數未用於重啟，實際新目錄／新engine log與30項結果已產生。已修正獨立啟動時的self path；後續複用同時檢查自我路徑、輸出目錄與外部capture旗標，不能一律全域替換。
- B06：封存helper靜態核對發現path library提供parent而非dirname，執行前修正；UTC欄位也明確tonumber。先核對固定Lua API，不以別的語言工具庫名稱猜測。
