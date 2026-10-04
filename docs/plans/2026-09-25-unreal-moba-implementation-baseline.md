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
| `cargo test --manifest-path omfue/codegen/Cargo.toml` | 19 項測試通過，含 `--check`、跨 Rust／UE 身分雜湊、通用英雄 metadata 測試 |
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
6. Rust template-id 與 UE codegen 共用 `omoba-content-model` 的英雄／技能身分檢查與完整模板 canonical hash；跨生成器測試確認目前 Lua 來源的身分雜湊同為 `403bb740f4d84738`，加入 `rust_module` 後的完整資料雜湊同為 `d56db98ae0c18953`。兩邊仍各自解析型別 schema，整併列入 2.1c。
7. 通用 `FOmGeneratedHeroMetadata` 與 `FOmGeneratedAbilityMetadata` 由 Lua 為每位英雄生成基本數值、技能槽及技能等級資料；Saika 舊事件維持相容。生成器版本升至 2。重跑完整流程後 OmGame UHT 與 C++ 編譯成功，Editor 啟動且 MCP PIE smoke 通過；測後停止 PIE，Editor 保持開啟。
8. 第二次完整建置遇到另一專案 `OpenKoikatsu.uproject` 的 Unreal 程序；原 `build_bridge.bat` 全域程序檢查誤擋。修正為 `om_restart` 先檢查本專案 Editor，再允許其子程序略過全域檢查；沒有停止另一專案。此情況下完整建置、Editor 啟動、MCP 與 PIE 均通過；反向驗證本專案 Editor 開啟時 `om_restart build --with-bridge` 仍以 exit code 4 拒絕 staging。
9. 完整模板雜湊加入後，`base_content` 45 項測試、`omfue/bridge` 編譯、`omfue/codegen` 19 項測試及 10 個生成檔的只讀 `--check` 均通過；這次只更新生成 JSON，沒有再次改動 Unreal C++ 輸出。
10. 資產配方 `om_asset_recipe.json` 目前有 2 位英雄；Saika 的模型、貼圖、頭像及 7 個動畫用途（5 份不重複 FBX）來源均存在，Date 只有頭像，未虛構模型。產生器目前 20 項測試，11 個生成檔的 `--check` 通過。Editor MCP 驗證既有兩個英雄 Blueprint 均為 valid/compiled，Saika 模型匯入到新 `/Game/OmGenerated/Heroes/saika_magoichi` 目錄成功。
11. 首次 `import_animation` 回報 AnimSequence 成功，但實際第一個物件是 SkeletalMesh，並附帶 PhysicsAsset；生成的 `b01_ani_stand_Anim` 綁定原 Saika Skeleton。已依 UE 5.8 本機原始碼將 MCP 匯入工具設為不自動偵測 FBX 類型，並要求回傳物件必須為 AnimSequence；OmGame 編譯通過。之後 Editor 兩次啟動都出現 HTTP 30000 已監聽但健康請求無回應，且插件外部授權／設定請求逾時；因此沒有批次匯入其餘動畫，修正後的 Editor 端匯入尚未驗證。沒有刪除或覆蓋既有 Blueprint。
12. `scripts/gen_hero_registry.lua` 由 Lua 英雄 `rust_module` 與技能宣告生成 `base_content` 註冊清單；新增測試比對 8 個匯出 handler ID 與定義 JSON。`base_content` 46 項測試通過。`om_codegen` 重新生成 11 檔並通過只讀檢查；技能 effect 宣告／通用執行器仍未實作。
13. `BpGeneratorUltimate` 在目前工作樹為未初始化 git submodule，直接修改其 `ImportTools.cpp` 不會出現在上層版本差異。新增受版控的 `scripts/ensure_bpgu_animation_import.lua`，由 `scripts/build_ue_moba.lua` 及 `omfue/build_bridge.bat` 建置前執行；已有修正則不寫檔，原版固定位置則套用修正，未知版型會失敗，不靜默覆蓋。
14. MCP 故障複測：Editor PID 56948 與重啟後 PID 47008 均監聽 HTTP 30000、broker 9876，但 TCP 連線後皆無 HTTP 回應，`wait-mcp` 逾時。Editor 日誌顯示 BpGeneratorUltimate 的授權心跳與設定服務請求逾時；這是觀測到的相關現象，尚不能判定它是唯一原因。完成診斷後已關閉本專案 Editor，未把 3.1 資產匯入驗證標為完成。
15. 首次匯入只有兩個 `.uasset` 實際落盤，Skeleton 等附帶資產沒有；為避免載入不完整的生成資產，將本次新建的 `saika_magoichi.uasset`、`b01_ani_stand.uasset` 從 `Content/OmGenerated/Heroes/saika_magoichi` 移到 `Saved/RejectedImports/2026-09-25-saika-mcp`。可移回復原；沒有刪除原有資產。
16. Saika 原有 `BP_SaikaMagoichi.uasset` 仍含專屬事件名稱；直接移除生成的 Saika 事件可能破壞既有 Blueprint，2.2b 前保留此相容入口。將 `AbilityDefinition`、`AbilityLevel`、`HeroDefinition`、`HeroLevelGrowth`、`HeroRender` 與動畫型別的共用解析移入 `omoba-content-model`；兩個生成器只保留各自的 `ue` metadata 或攻擊時序等擴充欄位。權威整數數值只在 Unreal C++ 生成邊界轉為浮點數。動畫 metadata 型別化後補齊預設欄位，生成輸出 hash 變成 `5c48767ecea22927`；原始 Lua canonical hash 維持 `d56db98ae0c18953`，身分雜湊維持 `403bb740f4d84738`。`omfue/codegen` 20 項、`omoba-template-ids` runtime Lua 模式 55 項、`omoba-content-model` 6 項、`base_content` 46 項測試通過；11 個生成檔 `--check` 通過；`scripts/build_ue_moba.lua --build-only` 完成 DLL、bridge 與 OmGame 編譯。OpenSpec 2.1c 完成。
17. 新增原創測試英雄 `training_luminary` 與四個 Lua 宣告式技能。技能註冊生成器在沒有 `rust_module` 時，根據 `effects` 建立通用 Rust handler；僅接受即時敵方指定目標傷害、自身治療，並驗證效果類型、目標類型、每級數值和非負有限值。特殊技能繼續使用明確 Rust handler。`base_content` 48 項測試通過，含四技能無畫面效果執行、非法等級／目標拒絕、非敵方目標拒絕與避免部分效果生效；完整 headless 對局輸入及結算尚未驗證，2.3b 保持未完成。
18. 檢查權威 adapter 發現 `faction_of` 目前回傳 `RNone`，不能拿來做敵我驗證；通用執行器改在任何世界變更前用 `query_enemies_in_range` 以目標當前位置、零半徑驗證目標確實是敵人。這沿用現有確定性敵方查詢；完整對局測試仍須檢查視野及合法施放距離的上游驗證。
19. 新英雄初次生成時仍帶不存在的 Blueprint 路徑，會讓 Editor 測試把 native fallback 誤判為資產缺失。加入顯式 `ue.native_only`，使新英雄及四個技能的 Blueprint 路徑為空，渲染器走已存在的 generated native class fallback；舊英雄仍驗證原 Blueprint。`omfue/codegen` 21 項、template-id 英雄測試 8 項通過，11 個生成檔 `--check` 通過，`--build-only` 依序完成 DLL、bridge、OmGame 編譯。這次生成內容 hash 為 `b2053976b557c57a`；尚未經 PIE 驗證美術 fallback。
20. 後端回歸執行 `cargo test --manifest-path omb/Cargo.toml -p omobab` 通過：單元測試 135 項通過、1 項忽略；`combat_damage_integration` 3 項與 `delete_entity_outcome_only` 1 項通過。幾個需外部程序或較長時間的既有 integration test 被標記 ignored，本次沒有把它們視為已驗證。這證明新英雄資料沒有破壞既有後端測試，但仍不足以宣稱四技能已走完整對局施放路徑。
21. 新增 `base_content` 的權威 ECS headless 整合測試：用正式 `StateInitializer` 建立世界、由生成的 manifest 載入 12 個技能，四次 `ScriptEvent::SkillCast` 經 `run_script_dispatch` 與 `process_outcomes`，驗證兩招傷害、兩招治療的最終血量；清除既有冷卻後再對同隊目標施放，驗證不產生傷害或冷卻。測試 fixture 初次缺少 `TowerTemplateRegistry`，依既有後端戰鬥測試補上預設 registry 後通過。`base_content` 完整 49 項、`omfue/codegen` 21 項測試通過。2.3b 完成；此測試不涵蓋網路輸入、施法距離或完整生命週期，保留在 4.x／5.x。
22. IPC 第一段：`RendererReady` 加入 player/team ID，renderer IPC 版本升為 3；client runtime 在送出任何投影前要求並核對握手。錯隊連線無法取得預先排隊的 snapshot；測試亦涵蓋錯玩家。loopback 自報身分只防止意外接錯端點，不是本機程序認證。`omoba-client-runtime` 24+1 項測試通過，`omfue/bridge` 24 項單元與 2 項單機 smoke 通過；bridge 測試原有兩處 `publish_snapshot` 舊呼叫缺少 `PresentationExtras`，已補預設值，正式路徑未變。`scripts/build_ue_moba.lua --build-only --ue-root D:\UE5.8` 亦完成 DLL、codegen、bridge 與 OmGame 建置。4.1 尚未完成 MOBA 專用輸入與跨隊視野驗收，維持未勾選。
