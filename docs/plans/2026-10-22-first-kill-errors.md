# 首殺獎勵與公告錯誤紀錄

- F01：沿用既有經驗，傷害致死與Death退休是不同步驟，測試必須drain延後事件，不能手動重生插隊。首次額外金與普通擊殺金分開結算，不能由UI或總擊殺推測。
- F02：OpenSpec CLI已確認不可用，手動維護既有格式artifacts；protoc fallback沿既有生成碼同步與codec驗證。不增加工作流fallback或維護歷史omfx。
- F03：WorldV1保留RuntimePresentationReset既有4個preview-world DestroyActor「World has no context」清理警告，37案為36 Success＋1 SuccessWithWarnings／0 Failed／NotRun；FirstKillNotice零警告，三張實際PNG已目視。不得把既有警告寫成零警告接受，也不能把獨立Slate公告圖當真連線畫面。
