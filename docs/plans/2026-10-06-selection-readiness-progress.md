# 選角握手嚴格解析與有界交接

## 決定與範圍

Grok Build job `run-muwdxk9k-vxwgie`，thread `3249609d-bd5b-43c5-b092-f47dbd3734c8` 原委派新的共用 parser／incremental reader 與純 fixture 測試，但初始讀取 3m22s 無 code delta。Primary 取消並確認 follower cancelled、metadata 三 PID null、原 bridge/agent PID 不存在後接手，完整實作與整合由 primary 完成。成本／API duration 與確切停滯根因未知；不歸功 Grok。未啟動無關 Editor、操作憑證、改全域 MCP 或提交／推送。

- 實際 Unreal marker 固定包含 `player`、`protocol=1`、`shared_room=0/1`；完整匹配而非版本前綴。
- 日誌每次 poll 最多 128KiB，單行最多 64KiB；不保存全日誌、事件歷史或每幀無界工作。
- 活著的 renderer 只接受換行完成的 marker；確認原始 child 退出後才允許 EOF 最後一行。
- 在原有 handshake 和 completion deadline 內有界 drain，支援原程序已退出及 shared final artifact 已發布的快速路徑。
- 已捕捉的合法握手不因玩家選角時間增長重讀整份 log；receipt、hash、roster、final plan 與 original-child 驗證仍保持獨立。

## 實際確認

固定 Lua runtime 實際执行：

| 測試 | 結果 | 範圍 |
| --- | --- | --- |
| `scripts/tests/moba_selection_readiness_test.lua` | 11/11 | exact marker、wrong identity/version/room、newline/final EOF、split chunk/backlog、64KiB line、missing/truncation/read/seek/close error、latched IO |
| `scripts/test_moba_hero_selection.lua` | 13/13 | 單人正式入口，含原 child PID 重用、版本前綴／錯 room、已退役 backlog／EOF |
| `scripts/test_moba_shared_selection.lua` | 22/22 | 共享正式入口，含 partial startup／cancel／ownership、protocol/room 前綴及 final artifact 快速路徑 |

新增 test 初次 bootstrap lexical `..` root 錯誤已修並記 E327；其後 11/11 實際通過。單人／共享測試都是 mocked renderer／host，沒有 Cargo preflight、真程序、對局模擬、完整效能或 Unreal/LAN 驗收。保留 OpenSpec 25/31，不因局部功能成功勾選完整 6.2。
