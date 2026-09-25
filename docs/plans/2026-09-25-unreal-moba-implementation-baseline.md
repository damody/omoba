# Unreal MOBA 框架實作基線

日期：2026-09-25

## 工作樹保護

開始前主 repo 已有未提交修改，包含 `omoba-client-runtime/src/main.rs`、`scripts/run_ue.lua`、ERPS 文件與 OpenSpec 任務、`erps` 與 `omfue` 子目錄；另有未追蹤的雙人 Unreal 啟動工具與測試證據。`omfue` 內已有 bridge、OmRuntime、restart、Target.cs、專案設定與 UI 的未提交修改。這些都視為既有使用者工作，本次先只修改未重疊的 `omfue/codegen`，並新增獨立建置入口。

## 已驗證基線

| 指令 | 結果 |
|---|---|
| `cargo check --manifest-path omfue/codegen/Cargo.toml` | 通過 |
| `cargo check --manifest-path scripts/Cargo.toml -p base_content` | 通過；`omoba-template-ids` 有既有 dead-code 警告 |
| `cargo check --manifest-path omfue/bridge/Cargo.toml` | 通過；先前 moved-value 錯誤已不再出現 |
| 在 `omfue` 執行 `cargo run --manifest-path restart/Cargo.toml -- status --ue-root D:\UE5.8 --output json` | 通過；當時沒有已開啟的相符 Editor |
| 在 `omfue` 執行 `cargo run --manifest-path restart/Cargo.toml -- build --ue-root D:\UE5.8 --output json` | 通過，回報 `unreal_build: ok` |
| `cargo test --manifest-path omfue/codegen/Cargo.toml` | 17 項測試通過，含新 `--check` 測試 |
| `cargo run --manifest-path omfue/codegen/Cargo.toml -- --content-root scripts/lua_data --out omfue/Plugins/OmRuntime/Source/OmGenerated --check` | 通過；10 個生成檔、13 個 Lua 輸入一致，content hash `ba93ee7c4884c40c` |
| `tools\lua\lua.exe scripts\build_ue_moba.lua --build-only --ue-root D:\UE5.8` | 通過；依序建 DLL、stage、codegen、bridge 與 OmGame |
| `tools\lua\lua.exe scripts\build_ue_moba.lua --full --ue-root D:\UE5.8` | 完整流程通過；Editor 啟動，MCP HTTP 在 port 30000 回報 ready |
| Editor 已開啟時執行 `scripts\build_ue_moba.lua --build-only --ue-root D:\UE5.8` | 預期失敗：`build_bridge.bat` 回傳 4，建置入口立即停止且未進入 OmGame 編譯；Editor 保持開啟 |
| BpGeneratorUltimate MCP `play_test.get_pie_status` | PIE 啟動前回報 `pie_running:false`；啟動後回報 `UEDPIE_0_Main`、Standalone、1 個 world 與玩家 `BP_RtsCameraPawn_C`；測後已停止 PIE |

## 本次決定

1. 先以目前工作樹實測，不把舊錯誤當成仍存在；Rust 與 OmGame 已能編譯，因此不改動使用者正在修改的 bridge。
2. 新增 `scripts/build_ue_moba.lua` 作為完整建置入口，先重建與 stage `base_content.dll`，再沿用 `omfue/restart --with-bridge` 的生成、bridge、OmGame 與 Editor/MCP 流程。
3. `omfue/codegen --check` 對生成檔逐字比對且不寫檔，避免檢查階段偷偷覆蓋使用者改動。
4. `om_restart start` 透過 Lua host 的同步命令會因 Editor 持續持有輸出管線而不返回；啟動階段改用獨立程序與日誌，再等待工具結束。Editor 維持開啟。
5. BpGeneratorUltimate 現版使用 HTTP MCP `127.0.0.1:30000/mcp` 與 `/mcp/health`，舊 `9877` TCP 探針會逾時；`om_restart wait-mcp` 改為先檢查 HTTP，仍保留舊通道相容性。
