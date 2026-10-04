# Unreal 通用碰撞地形呈現

## 實作與驗收節奏（依使用者最新指示）

後續以功能實作為主。每批只做必要的編譯或直接相關的功能確認；已有成功結果不反覆重跑。完整 MCP automation、雙 UE 60Hz 長測、跨系統回歸與效能驗收統一留到功能整合完成後執行。建置成功不等於完整驗收，兩者分開記錄，不提前勾選大型未完成項目。

本批接續：ABI10／Unreal 建置確認 → 修正當前編譯問題 → 繼續通用功能實作。以下兩項完整驗收移入最後整合清單，現在不執行。

## 本輪計畫

- [x] compiled public map獨立傳入PresentationExtras，不用vision polygon帶碰撞地形。
- [x] C ABI10 frame-owned terrain_rects／string table；full與lifecycle共用安全map來源。
- [x] 單一可替換mesh的ISMC共用呈現，依mesh bounds調整footprint／center，visual height可配置；關閉UE collision／overlap／nav，不產生第二份玩法。
- [x] 相同位置／尺寸不重建；control保留、完整空view與Stop清除、fresh view重建。
- [x] Rust正負座標任意地圖fixture、Unreal automation與Lua任意數量觀測器，--three-lane必須terrain exact expected/instances而非只有route。
- [ ] 最後回歸、完整Unreal建置、MCP兩輪automation。
- [ ] 真實雙UE60Hz與獨立stage／PID核對、保存證據。

## 決策與限制

資料來源是Lua→compiled catalog→server public map id/hash→validated RuntimeReady→bridge，沒有依地圖名稱猜測。獨立terrain_rects含public矩形，不含camp狀態／仇恨／重生計時器。模型可替換且共用高度是純呈現參數，不是Rust碰撞高度；未新增每map Blueprint graph或C++。ABI9拒絕，必須同步header／DLL／Unreal重建。

缺美術使用engine cube可執行fallback；不是完整地表美術、camp標記、minimap terrain/fog、完整建築、LAN或穩定60FPS。不能以局部編譯／log當作PNG或完整5.4／6.1驗收。

## 當前功能確認

- Rust bridge：55 passed／1 opt-in ignored；integration 2 passed／1 ignored。terrain fixture確認frame-owned資料與vision分離。
- Lua terrain觀測器13個斷言通過；先前相關route與two-team觀測器回歸已通過，不再重跑。
- `tools/lua/lua.exe scripts/build_ue_moba.lua --build-only` exit0，Lua codegen、ABI10 cbindgen、bridge與OmGameEditor建置成功；新的Unreal terrain automation程式已編譯，但尚未執行。
- 本次stage SHA-256：`554332de168326f01ac36999ae7fa8af1fa40ef57a35f549515b5f76c1903f40`。
- 依使用者最新指示，不啟動完整MCP／雙UE／60Hz長測；保留到最後整合驗收。当前確認資料與程式建置成功，不宣稱實際畫面已驗收。
