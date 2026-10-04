# 通用小地圖公開地形層

## 計畫與決策

- [x] 同一份 ABI10 terrain_rects 轉成 frame lease 外仍有效的 UI 自有資料。
- [x] 公開地形與 route 共同決定示意範圍，保留等比例、Y 反轉與 10% 留白；不從單位位置推算範圍。
- [x] Slate 共用矩形繪製；地形在下、路線與安全可見單位在上，不新增地圖專屬 C++ 或 Blueprint graph。
- [x] control 保留、full empty/reset 清空；非法／非有限／倒置／超容量資料整批清空，不留下部分資料。
- [x] 加入獨立 MinimapTerrain 功能測試與指定測試單輪入口，與完整驗收輸出隔離。
- [x] 編譯與本功能單輪成功確認。

## 問題與處理

原本小地圖只從 route 建立範圍且缺少 route pointer 就提早返回，因此公開地形即使已在世界呈現，小地圖仍無法顯示。改為兩類公開靜態幾何共同建立範圍，也支援只有 terrain 的完整 view。

這是唯讀公開碰撞地形示意，不是 fog／可走邊界／導航判定。小地圖右鍵仍使用同一投影逆轉與正式 Rust 輸入；不在 UI 判定可走或讀取 actor 隱藏資訊。全套 MCP、雙 UE、60Hz 長測與像素驗收留到最後。

原有驗收工具每次固定全套兩輪，不適合功能開發節奏。新增 --test／--runs／--out-dir，指定測試只能選已知清單，拒絕重複與未知名稱，須另外指定報告目錄避免覆寫完整驗收；默认完整清單兩輪不變。

## 當前成功確認

- `tools/lua/lua.exe scripts/build_ue_moba.lua --build-only` exit0、OmGameEditor Succeeded；bridge/header ABI10未再變動，stage SHA `554332de168326f01ac36999ae7fa8af1fa40ef57a35f549515b5f76c1903f40`。
- Editor透過restart啟動，MCP ready；僅執行 `Om.Runtime.MinimapTerrain` 一輪，exit0／success=true，報告 `omfue/Saved/McpAutomation/MinimapTerrain/report.json`。
- 功能fixture驗正負座標／terrain-only view／Y反轉與正pixel尺寸／input round-trip／lease外自有複製／control保留／非法整批清空／缺pointer／超容量／fresh恢復／full empty清除。
- 不執行其他20項全套測試、不執行PIE／雙UE／60Hz長測；沒有將模型測試稱為實戰像素驗收。大型清單維持20/30，完整6.1／6.2尚未完成。
