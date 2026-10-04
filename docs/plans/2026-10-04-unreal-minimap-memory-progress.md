# 通用小地圖安全記憶標記

## 計畫

- [x] 只讀既有 Rust server-sanitized frozen ghost，複製位置／kind 到獨立 UI 記憶陣列。
- [x] UI 記憶型別不含 live entity reference／owner，不參與目標選取。
- [x] 依 render ID 去重，live 優先；非法身分／epoch／座標與範圍外資料不呈現。
- [x] 不從 memory 擴張地圖、不加前端 TTL、不推算 hidden 位置。
- [x] 共用 Slate 空心暗色標記；control 保留、Reveal／Forget／Reset／Stop 依正式快照清理。
- [x] 新增 MinimapMemory 功能測試，包含 WorldBridge 整合。
- [x] 建置與本功能單輪成功確認。

## 決策與問題

原小地圖在 entities pointer 為空時提早返回，因此 ghost-only 安全快照沒有呈現機會。將 live 與 remembered 分成獨立迴圈；記憶只使用 Rust 已核准資料，不從 actor 補值。去重規則與世界 ghost adapter 相同：render_id／disclosure_epoch 非零，同 render_id 有 live entity 即不顯示記憶。

記憶與 live 使用不同型別及圖層；前者沒有可攻擊的 entity reference，不猜 owner／隊伍，也不以此改右鍵 Point Move 行為。完整新快照替換記憶，沒有前端自行更新／倒數，重新連線沿既有安全完整快照恢復。

本批只確認此功能；完整 fog shading／實戰像素／双 UE／60Hz長測與全框架驗收仍留最後。沒有將此記憶標記稱為完整戰爭迷霧或所有 cue 完成。

## 當前功能確認

- `tools/lua/lua.exe scripts/build_ue_moba.lua --build-only` exit0，OmGameEditor Succeeded；ABI10／bridge未改，stage SHA仍為 `554332de168326f01ac36999ae7fa8af1fa40ef57a35f549515b5f76c1903f40`。
- 透過restart啟動新版Editor與MCP，僅跑 `Om.Runtime.MinimapMemory` 一輪：exit0、success=true、1 passed／0 failed／error_count0。報告另存 `omfue/Saved/McpAutomation/MinimapMemory/report.json`，沒有覆寫完整驗收或上一批地形報告。
- fixture涵蓋ghost-only／非法epoch／NaN／重複／自有複製／control／Reveal live優先／範圍外排除／bounds不變；WorldBridge測試確認Forget／fresh恢复／ResetView／Stop。
- 本批diff check通過。沒有執行PIE／其他全套測試／雙UE／60Hz長測；總清單20/30，仍有完整UI／fog shading／全部cue／Bot／LAN等工作。
