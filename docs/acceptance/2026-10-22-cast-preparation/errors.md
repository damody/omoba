# 2026-10-22 施法準備動作錯誤紀錄

- P01：盤點猜測 OmNativeCharacterComponent、scripts/lua_codegen 與 OmCastPreviewFeedback.cpp不存在；查詢沒有變更來源，改依rg --files與既有成功測試定位到OmAnimatedUnitActor、OmHeroActor及scripting/dispatch.rs。後續不再猜測API／路徑。
- 設計防錯：只從實際權威AbilityWindups產生公開狀態，不從本機瞄準、cooldown或動畫名稱推論成功。post-outcome／post-buff-timer同步，避免起始少一tick與過期多一tick；不改傷害、魔力或冷卻交易。
- 設計防錯：instance以十進位字串傳JSON，避免double丟失int64精度；total_raw有界Q10。準備與釋放使用獨立生命期，取消／死亡／來源退休後舊狀態不能復播。
- P02：查詢再次使用Windows位置參數萬用字元與猜測OmEditor插件根路徑，收到os error123／不存在；無來源變更。已改以rg --files列出真實路徑，再讀取OmRuntime/Source/OmEditor。
- P03：Saved驗收helper建立後再寫入時漏設path.write overwrite=true，受到既有保護拒絕；尚未啟動消費者。確認函式簽名後僅修正Saved helper的覆寫旗標並重建，沒有重跑來源編輯。
- P04：複製前批World helper時全域替換誤改既有om-objective-notice-capture旗標；在World啟動前改回真實旗標，輸出資料夾可使用本批名稱。後續複製只替換工作路徑，既有測試ID／CLI旗標維持原值。
- T01：WorldV1 CastPreparationAnimation發現TryGetStringField會容許數字轉字串，使instance:1錯誤通過。改用JSON value Type==EJson::String的顯式檢查後才讀值，拒絕浮點精度來源；先等待V1所有consumer／watcher／sourceguard完全退休，再修來源並重做V2。保留原始失敗log及receipt，不把V1計為成功。
- T02：bridge V1新fixture錯把純remaining倒數預期為BuffUpdated，索引空事件列導致失敗。正式契約僅在payload變更／刷新發事件，每個snapshot仍有最新active_buffs。保留通用節流，修測試改驗remaining_secs=0.25且無額外事件；不是修改production發送更多事件。RustV1因此失敗，source-v1依成功守門拒絕檢查，皆保留為失敗證據。
