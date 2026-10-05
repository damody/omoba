# MOBA 選角前建置與部署拷貝一致性

## 計畫與決定

1. 實際缺口：既有 verify-staged-only 只檢查 bridge；server 載入 scripts/base_content.dll、Unreal 載入 plugin/base_content.dll，兩份都可能和剛建置的 script DLL 不同。
2. 新共用 moba_stage_contract 契約，以 SHA-256 檢查 bridge built→plugin 一份、base_content built→server／plugin 兩份；缺少 built／staged 或不一致即拒絕，回傳有 scope 的結構化結果。它不偷偷拷貝、切 profile 或換 DLL。
3. build_ue_moba.lua 的建置後與 verify-staged-only 都用同一契約。正式前端建置固定使用 debug bridge／script DLL；server／client／selection CLI 保留使用者指定的 debug／release profile，沒有 fallback。
4. 共用 moba_launch_workflow 統一順序：解析 Editor→建置前端→建置 selection CLI／server／client runtime→部署檢查→選角→準備權威配方→再次部署檢查→開局。
5. 不讓玩家選完才等待後端建置或發現部署错误；第二次檢查防止選角期間有人更換部署副本。這是正常開局 guard，不是重跑完整驗收。
6. --no-build 跳過建置但不跳過部署檢查；--prepare-only 只產生設定，不解析 Editor、建置前端或啟動程序。
7. 輸出與選角目錄若已存在，主入口在建置前先拒絕，不寫失敗紀錄到被拒絕的既有目錄。

## 當前確認

- 固定 Lua 的 scripts/test_moba_launch_contract.lua 8/8 成功：真正 fixture SHA-256、三份 copy、舊 server script DLL 拒絕、built／staged 缺失、精確階段順序、建置失敗不選角、不一致在選角前拒絕、no-build guard、prepare-only 無程序。
- 流程部分使用 mock callbacks，沒有執行完整建置、部署 DLL、啟動遊戲或重跑 Unreal 選角。
- 主入口與建置入口 --help 可執行，主 repo whitespace check 通過；openspec validate build-unreal-rust-moba-framework --strict 通過。

## 邊界

- SHA 相同只代表部署拷貝一致，不證明 Cargo feature、ABI、DLL 內容 catalog、來源新鮮度、引擎版本或玩法正確。正式 compiled-content-only build／既有 ABI 與內容 handshake 仍必須保留；不把 copy report 當完整版本／功能驗收。
- 尚未在此批做正式「選角→server／runtime→完整對局」確認；完整對局、LAN 與效能最後集中驗收。
- 6.2 全項仍待，整體 21/31；只維護 omfue，不修／建／驗 omfx。
- 缺口與防錯規則記 E214。
