# Editor MCP 最小驗收（2026-10-03）

## 環境與可重跑指令

- Unreal：`D:\UE5.8`，目標 `OmGameEditor Win64 Development`。
- 插件：BpGeneratorUltimate 預編譯版 2.0.6，descriptor 指定 UE 5.8.0。
- 固定工具：Rust 1.95.0、`tools/lua/lua.exe`。
- 完整建置與啟動：`tools/lua/lua.exe scripts/build_ue_moba.lua --full --ue-root D:\UE5.8`，成功。
- MCP 工具探索：`tools/lua/lua.exe scripts/ue_mcp.lua --list`。
- 最小驗收：`tools/lua/lua.exe scripts/ue_pie_smoke.lua`，成功。

## 結果

- HTTP MCP `127.0.0.1:30000/mcp` 可完成初始化、動態工具探索及實際工具呼叫。
- 使用 Editor 本機 session token；token 不放進程序參數、stdout 或報告。
- `begin_play_in_editor` 後取得 `UEDPIE_0_Main`、Standalone、玩家 `BP_RtsCameraPawn_C`。
- `run_pie_test_sequence` 生成 `/Script/OmGenerated.OmHeroTrainingLuminary`，斷言 `location.x == 300`，回報 `PASS`、1 項通過、0 項失敗、0 項不確定。
- 測試 actor 由工具清除；獨立讀取 PIE 日誌並取得 composited Slate 截圖。
- 自動化啟動的 PIE 在完成後停止，`get_pie_status` 驗證 `pie_running:false`。
- 若執行前已有 PIE，驗收入口使用該 session，並保留其執行狀態。

## 執行產物

產物位於 `omfue/Saved/McpAutomation`，不提交暫存資料或圖片：`tools.json`、`pie-before.json`、`pie-running.json`、`pie-native-sequence.json`、`pie-log.json`、`pie-screenshot.json`、`pie-smoke.png`、`pie-after.json`、`pie-smoke-report.json`。報告 `success:true`。

這是 3.3 的 Editor 通道與生成類別最小驗收。現有場景仍啟動 TD 單機玩法；尚未驗收選角到 MOBA 結算、雙隊 filtered world、美術 fallback 完整外觀或效能門檻，相關工作維持 4.x／5.x／6.x 待完成。
