# 連通草叢實作錯誤紀錄

- C01：沿用錯誤路徑 scripts/lua_data/moba_maps.lua、OmRuntime/Private/OmPointAreaPreviewTests.cpp 與不存在的 OmControlSettings.h。已先列實際檔案再定位，正確為 templates/moba_maps.lua、OmEditor/Private/OmPointAreaPreviewTests.cpp 及 OmPlayerControlProfile.h。後續不猜測路徑。
- C02：OpenSpec CLI 未安裝。手動維護 proposal／design／spec／tasks，不宣稱執行 CLI validate。
- C03：RustV1 編譯時對 specs Fetch 呼叫 clone，推導成複製借用包裝；改為 (*regions).clone()，明確複製 immutable Arc topology。V1 失敗證據保留，不當成通過。
- C04：來源 capture 工具成功訊息為 fingerprint succeeded，但通用 launcher 的 summary='-' 只辨識 exit=0，會產生 false receipt。保留 capture 原始證據，最終另用 check 產出 success JSON 驗證；不可將 capture launcher false 宣稱為通過。後續無 JSON 的工作需用明確 exit wrapper。
- C05：RustV2 通過 core811 後，新增跨元件 fog／disclosure 與 live targeting 整合測試；主動退休其餘工作，改以凍結後 RustV3 為本輪完整接受依據。
- C06：tooltip review 發現未綁定快捷鍵會顯示「按下 —」；已改為明確未綁定提示，補上 rebind／unbound 測試。第一次來源 capture 時尚在 review，正式基準改用凍結後 source-before-v2；原 capture 不採用。
- C07：build-v1 完成 14:40:09，最後 UE 測試修改 14:40:11，不能作最終接受版本；凍結後執行 build-v2，再 staging 與 World／full 接受。
- C08：視覺複核沿用不存在的 fog.png 名稱，已先列本輪實際 PNG 清單再選取；缺圖不當作已目視，camp-0 原始圖已成功讀取。
- C09：沿用上一輪影格850／1000沒有拍到展開tooltip；本輪依實際stage索引及原始JPEG大小選取V2／V3。已目視350改鍵F10／放開確認、590指定敵方／放開確認、802射程外提示。這些是施法預覽，不冒稱新版tooltip已目視；新增tooltip模式／rebind／unbound由Om.Runtime.AbilityTargetTooltip及完整native通過證明，錄影未取得展開狀態的限制保留。
