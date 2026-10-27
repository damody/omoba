# AGENTS.md

This file provides guidance to Codex when working with code in this repository.

## 專案概觀

`omoba` 是 MOBA / TD 雙模式 Rust 遊戲。正式維護範圍是後端 `omb`、共用 Rust／內容生成與 Unreal 前端 `omfue`；場景、英雄、塔與技能行為由 `scripts/base_content.dll` 透過 `abi_stable` FFI 載入。

依使用者 2026-10-05 指示，`omfue` 是唯一需要維護的前端。`omfx`／Fyrox 保留為歷史程式，不再新增功能、修復相容問題、編譯驗證或納入驗收。共用 schema／Rust API 變更只維護正式後端與 Unreal 消費端，不為 `omfx` 補接線；此決定不授權刪除其檔案或 submodule。

正式建置涉及 `scripts/`、`omb/`、`omoba-client-runtime/` 與 `omfue` 的 Rust／Unreal 元件，不包含歷史 `omfx` workspace。所有工作流邏輯使用 Lua 5.4，固定由實際 clone 根目錄的 `tools\lua\lua.exe` 執行；常用入口保留根目錄薄 `.bat` wrapper。

## Toolchain

- 固定使用 Rust `1.95.0`，以 `rust-toolchain.toml` 為準。
- `abi_stable` 要求 host 與 script DLL 使用同一個 rustc，不要單獨升級某個 workspace 的 toolchain。

## 根目錄保留腳本

根目錄只保留這四個 `.bat`：

| 情境 | 指令 |
|---|---|
| 一般 dev run | `run.bat` |
| 高負載 / 10000 entity run | `run_10000.bat` |
| 本機雙玩家 run | `run_2player.bat` |
| Unreal frontend run | `run_ue.bat` |

不要新增根目錄 `.sh` 或其他 `.bat`。四個 `.bat` 只能定位並呼叫對應 Lua、轉送 `%*`、回傳 exit code，不得包含建置或程序邏輯。若修改 `.bat`，必須維持 CRLF 行尾與 UTF-8 無 BOM。

一般工具入口從 clone 根目錄使用 `tools\lua\lua.exe scripts\<tool>.lua`。不得新增 PowerShell、Python 或 shell fallback；Lua 標準庫缺少的平台能力由 `tools/lua-host` 提供。

## 常用手動指令

```bat
cargo test --manifest-path omb/Cargo.toml -p omobab
cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi
cargo test --manifest-path scripts/Cargo.toml -p base_content
```

產生 unit / script API catalog：

```bat
cargo build --manifest-path scripts/Cargo.toml -p base_content --release
copy /y scripts\target\release\base_content.dll scripts\base_content.dll
cargo run --manifest-path omb\Cargo.toml -p omobab --bin gen-docs --features gen-docs --release
```

DLL 一律 stage 到 repo 的 `scripts\base_content.dll`，不要複製到 `omb\scripts\`；`gen-docs` 依序找 `scripts/base_content.dll`、`../scripts/base_content.dll`、`scripts/target/<profile>/base_content.dll`。

產物位於 `omb/target/docs/index.html`。

## 目錄架構

Submodules：

- `omb/`：後端，bin/package 名稱為 `omobab`。
- `omfx/`：Fyrox 前端；`executor` 是 native launcher。
- `map_editor/`：地圖編輯器。
- `specs/`：forked ECS。
- `log4rs/`：forked logging。
- `mqtt_log_viewer/`：MQTT log viewer。

Monorepo 目錄：

- `omoba-core/`：client/server 共用 schema、transport client、deterministic runtime native pieces。
- `omoba-sim/`：deterministic simulation primitives。
- `omoba-template-ids/`：Lua/template data 的 build-time generated IDs 與 story data。
- `scripts/script-abi/`：host 與 script DLL 唯一共用 ABI crate；只能放 `abi_stable` 友善型別。
- `scripts/base_content/`：塔、英雄、召喚物、技能的 script 實作。
- `scripts/lua_data/`：story/map/template Lua source。
- `eui/`：omfx 使用的 immediate-mode GUI。
- `omb-mcp/`：query-only KCP MCP server。
- `proto/game.proto`：prost/tonic 共用 schema。
- `docs/plans/`：設計與實作紀錄。

`omb/` 單獨 clone 無法 build，必須搭配完整 monorepo，因為有多個 `../` path dependencies。

## 核心注意事項

- ECS 使用 `specs` 0.20；`#[derive(Component)]` 來自 `use specs::Component;`。
- `scripts/script-abi` 是 host + cdylib 的共用邊界，不要在此引入 `specs`、`serde_json` 等 runtime-heavy dependency。
- transport feature 互斥：`mqtt` / `grpc` / `kcp`，default 是 `kcp`。
- `omb/game.toml` 的 `STORY` 切換 generated story id。
- `hero.stats` 約每 0.3 秒由後端廣播一次，payload 會聚合 BuffStore 的 additive/multiplicative modifier。
- 新增 per-entity 每 frame UI/scene 操作時，要注意 stress 場景熱路徑，避免破壞既有節流。
- 每個 launch 自帶 base_content（E124）：`moba_role_launch` 從 `scripts/target/{debug,release}` 取最新建置複製到 `<run>/artifacts/scripts/`，server、client runtime、game.toml 與 UE 執行副本（`ue_run_snapshot`）都指向這份；launch preflight 不再檢查共用的 `scripts/base_content.dll`。其他 session 重建 base_content 不會擋下已開始或之後的 run。
- 遊戲程序從副本執行（E125）：`moba_role_launch` 把 omobab 與 client runtime 複製到 `target/run-bin/<key>/` 再啟動，`target/<profile>` 的 exe 永遠不被鎖，cargo 隨時可重建；不要再等別人的程序結束。用自己的 `CARGO_TARGET_DIR` 建置時設 `OM_RUST_RUN_BINARIES=<該 target 目錄>`，exe、moba-config、base_content 來源與 preflight 都跟著它。啟動前比對 server、runtime、renderer bridge 內嵌的 `OM_CONTENT_CATALOG_DATA_HASH` marker，與 authority 的 content hash 不一致就拒絕啟動，錯誤會指出要重建哪一個。
- live 輸入一律經 `land_to_live` 放回（E143）：`scripts/lua_data`、`omfue/codegen`、`omfue/bridge`、omfue 的 C++ 原始碼與 Rust crate 都是其他 session 正在建置的 live 輸入，不要直接改到一半。在影子或副本改好後，用 `tools\lua\lua.exe scripts\land_to_live.lua --change SRC=LIVE@BASE ...` 放回。它會在 build lock 下以「live＋這批變更」跑 om_codegen、cargo check 和 UBT，全部通過才以 3-way merge 一次寫入；失敗就不寫入並說明原因。隨時可以用 `scripts\check_live_consistent.lua [--ubt]` 確認 live 目前能否 codegen 與編譯。
- 影子 UE 專案（E116，`OM_UE_RUN_BINARIES`）一律用 `tools\lua\lua.exe scripts\stage_ue_shadow.lua --project DIR [--sync] [--build]` 同步原始碼、stage bridge／header／base_content 並寫 stage record；不要手動複製 DLL。launch 只檢查專案的 om_bridge.dll 與它一起 stage 的 header 一致（`ue_bridge_stage.verify_launch`），不要求等於共用 target 裡最新的 bridge。

## Submodule 提交

submodule 內有實際變更時，先在該 submodule repo commit/push，再回主 repo bump gitlink。不要只在主 repo commit 指標而漏推 submodule commit。

## 清理原則

- 不提交 `target/`、DLL、EXE、PDB、log、trace、cache 等建置或執行暫存檔；唯一例外是固定 Lua runtime 的 `tools/lua/lua.exe`、`tools/lua/lua54.dll` 與 `tools/lua/lfs.dll`，它們是有 provenance 與 SHA-256 的受版控工具鏈輸入。
- `omfue/` 目前按使用者要求不納入一般 cleanup/commit，除非明確要求處理 Unreal project。
