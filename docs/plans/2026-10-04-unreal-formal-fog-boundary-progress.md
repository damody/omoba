# 正式 MOBA 視野呈現邊界

## 原計畫與必要修正

原計畫繪製小地圖可見／已探索／未探索底色；讀取實際程式後發現目前fog_tiles來自DemoFogCache，以固定10-unit grid／700-unit英雄示範半徑推導，ABI只有column／row／visible，沒有geometry、來源或explored契約。因此先修正順序，不能把示範資料轉成正式玩法呈現。

## 本批計畫

- [x] 正式safe HUD phase metric辨识MOBA，停用demo fog／circles／trees／polygons投影。
- [x] 模式界線保留在該PresentationHub session；正式模式後缺HUD／reset不恢復demo推導。
- [x] 一般legacy/demo路徑保留原raycaster與快取，不用全域移除破壞原功能。
- [x] 小地圖明確顯示VISION N/A，不把empty tiles當全可見；既有地圖／markers仍可用。
- [x] Rust production-envelope／cache與Unreal單一功能確認測試。
- [x] 建置與此批必要確認。

## 後續正式 fog 的通用契約

正式迷霧底色尚未實作，不能勾選6.1／6.2。需要authority-team-safe的版本化grid geometry（origin、cell size、coverage）、current-visible與explored定義、完整reset／view epoch及容量上限；filtered runtime傳遞已核准資料，bridge frame自有複製，UI只投影，不從actor、hero位置、collision terrain或remembered ghost補視野。

現在只移除不正確的示範投影，沒有修改server視野隔離／權威visibility／live與ghost披露；ABI10未改。將來不能因VISION N/A字樣存在就宣稱fog完成。

## 成功確認

- Rust只跑 `formal_moba_never_falls_back_to_demo_fog`：1 passed／0 failed；驗legacy仍有demo、formal四種示範資料全空、cache清除、reset缺marker不回落、production envelope辨識phase marker。其他測試未重跑。
- runtime dev executable建置exit0；Lua build-only exit0、OmGameEditor Succeeded。ABI10不變，bridge因共用runtime dependency重建，stage SHA更新為 `c66b56f1702dc80d49c917a44d037d8797e1f92526977e5a07724f5be74b1778`。
- Unreal `Om.Runtime.MinimapFogBoundary` 單輪exit0／success=true；只驗public map仍可用、legacy／empty tiles不冒充權威fog、control保留、reset清狀態。另存 `omfue/Saved/McpAutomation/MinimapFogBoundary/report.json`。
- 本批diff check通過。未跑完整automation／PIE／雙UE／60Hz長測／fog像素驗收。原訂正式迷霧繪製尚未完成，總清單20/30不變。
