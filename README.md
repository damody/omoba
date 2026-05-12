# omoba

`omoba` 是以 Rust 實作的 MOBA / TD 雙模式遊戲專案。整體採 monorepo 管理，但主要子系統分成多個 Cargo workspace 與 git submodule：後端 server (`omb`)、Fyrox 前端 renderer (`omfx`)、script DLL (`scripts/base_content.dll`)、共用 schema/client (`omoba-core`) 與 deterministic sim / template id codegen 等支援 crate。

這份 README 是給新開發者快速理解整體架構與常用流程使用；更細的 agent / 維護注意事項可參考 [`CLAUDE.md`](CLAUDE.md)，設計紀錄可參考 [`docs/plans/`](docs/plans/)。

## Quick Start

此專案主要在 Windows 上開發，根目錄為 `D:\omoba`。`.bat` 腳本請從 Windows `cmd.exe` 在 repo 根目錄執行。

| 情境 | 指令 | 說明 |
|---|---|---|
| 第一次 clone 後拉 submodule | `git submodule update --init --recursive` | `omb`、`omfx`、`specs`、`log4rs` 等是獨立 repo |
| 一般開發啟動 | `run.bat` | debug build script DLL + backend + frontend，frontend 會 spawn backend child |
| 自動 smoke run | `run_smoke.bat` | 2 秒自動 Start Round，10 秒自動退出 |
| 較長 smoke run | `run_smoke_long.bat` | 2 秒自動 Start Round，60 秒自動退出 |
| TD stress 壓測 | `run_stress.bat` | release build，產生 stress map，暫時切 `omb/game.toml` 到 stress variant，結束後還原 |
| 產生 unit / script API catalog | `gen_docs.bat` | 產出並開啟 `omb/target/docs/index.html` |
| 後端測試 | `cargo test --manifest-path omb/Cargo.toml -p omobab` | 測 `omb` workspace |
| Script ABI 測試 | `cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi` | 測 host / DLL 共用的 ABI crate |
| base_content 測試 | `cargo test --manifest-path scripts/Cargo.toml -p base_content` | 測 script workspace |

## Toolchain

- Rust 版本固定在 `rust-toolchain.toml` 的 Rust `1.91.0`。
- `abi_stable` 要求 host (`omb`) 與 script DLL (`scripts/base_content.dll`) 使用同一個 rustc，不能只在其中一個 workspace 升級 toolchain。
- 一次完整啟動通常會碰到三個 Cargo workspace：`scripts/Cargo.toml` 先編 `base_content.dll`，再編 `omb/Cargo.toml` 與 `omfx/Cargo.toml`。
- `.bat` 檔必須使用 CRLF 行尾。LF-only 會讓 Windows `cmd.exe` 把每行首字吃掉，常見錯誤是 `'M' is not recognized`。

若新建或重寫 `.bat`，可用以下 PowerShell 轉 CRLF：

```powershell
$p = 'D:\omoba\xxx.bat'; $c = (Get-Content -Raw $p) -replace "(?<!`r)`n","`r`n"; [System.IO.File]::WriteAllText($p, $c, (New-Object System.Text.UTF8Encoding $false))
```

## Repository Layout

### Git Submodules

| 路徑 | 說明 |
|---|---|
| `omb/` | 後端 server，package / bin 名稱為 `omobab`，repo 為 `github.com/damody/open_moba_backend` |
| `omfx/` | Fyrox 前端 renderer，實際執行入口為 `executor` |
| `map_editor/` | 地圖編輯器 |
| `specs/` | forked `specs` ECS，包含本專案需要的 derive / serde 調整 |
| `log4rs/` | forked `log4rs`，包含 MQTT appender 等專案需求 |
| `mqtt_log_viewer/` | MQTT log viewer 工具 |

Submodule 內的修改要先在該 submodule repo commit，再回到 monorepo bump submodule pointer。

### Monorepo Directories

| 路徑 | 說明 |
|---|---|
| `omoba-core/` | client / server 共用 schema、transport client、quantization、tower / ability meta |
| `omoba-sim/` | deterministic simulation crate，供 server / client lockstep 方向共用 |
| `omoba-template-ids/` | build-time 從 Lua / template data 產生 typed template IDs |
| `scripts/script-abi/` | host 與 script DLL 唯一共用的 stable ABI contract crate |
| `scripts/base_content/` | 內建塔、英雄、召喚物與技能 script 實作，編成 `base_content.dll` |
| `scripts/lua_data/` | story / map / template Lua source，例如 `TD_1`、`TD_STRESS`、`templates/heroes.lua` |
| `eui/` | omfx 使用的 immediate-mode GUI library |
| `omb-mcp/` | MCP server，以 KCP query-only 方式查詢 `omb`，避免訂閱 event 洪水 |
| `proto/game.proto` | prost / tonic 共用 wire schema，依 Cargo feature 由 build script 產生 |
| `docs/plans/` | 架構設計與實作計畫紀錄 |

`omb/` 單獨 clone 無法完整 build，因為它依賴 monorepo 內的 `../scripts/script-abi`、`../omoba-core`、`../omoba-sim` 等 path dependencies。

## Runtime Architecture

```text
player input
  -> omfx executor / game client
  -> omoba-core transport client (default KCP)
  -> omb transport layer
  -> omb ECS world + tick systems
  -> scripts/base_content.dll via abi_stable FFI
  -> omb emits GameEvent / snapshots
  -> omfx render bridge updates Fyrox scene + EUI HUD
```

核心流程：

1. `run.bat` 先確認 `scripts/base_content.dll` 是否新鮮，必要時編 `scripts/Cargo.toml -p base_content`。
2. Script DLL 會被 stage 到 `omb/scripts/base_content.dll`，讓後端 host 載入。
3. `omb` 啟動後讀取 `omb/game.toml`，依 `STORY` 選擇 generated story / map / template data。
4. `omfx/target/debug/executor.exe` 啟動前端，前端會 spawn `target/debug/omobab.exe` 作為 child process。
5. 玩家指令經 transport 進入後端，後端在 ECS tick 中處理移動、碰撞、戰鬥、TD wave、技能與 script callbacks。
6. 後端發送 typed / JSON event、heartbeat snapshot 與 hero stats snapshot，前端轉成 render state。

## Backend (`omb`)

`omb` 是 authoritative server。主要 crate 是 `omobab`，入口為 `omb/src/main.rs`，常見模組如下：

| 模組 | 職責 |
|---|---|
| `comp/` | specs ECS components，例如位置、攻擊、英雄、技能、塔、小兵等狀態 |
| `tick/` | 每 tick 的 gameplay systems，例如移動、技能、傷害、碰撞、波次與死亡處理 |
| `state/` | resource management、event emission、hero stats payload builder 等狀態整合 |
| `ability_runtime/` | buff store、stat aggregation、ability dispatcher 與 runtime helper |
| `scripting/` | 載入 script DLL，將 host world 包成 ABI-safe API 給 script 呼叫 |
| `transport/` | MQTT / gRPC / KCP 的共同抽象與具體實作 |
| `vision/`、`aoi.rs` | 視野、AOI 與可見性相關邏輯 |
| `lockstep/` | lockstep / deterministic simulation 方向的 server-side support |
| `config/` | `game.toml` 與 server config |

後端使用 `specs` ECS。`specs/` 是 forked submodule，`Entity` serde 與 derive 使用方式已依本專案調整。新增 gameplay state 時應優先思考它是 ECS component、resource、script payload，還是 transport-only data。

## Frontend (`omfx`)

`omfx` 是 Fyrox 前端 workspace，主要執行入口是 `omfx/executor`，game logic bridge 位於 `omfx/game`。

| 模組 | 職責 |
|---|---|
| `executor/` | native desktop launcher，啟動 Fyrox engine 並 spawn backend |
| `game/` | client-side game crate，包含 `sim_runner`、`render_bridge`、`lockstep_client` |
| `editor/` | Fyrox editor integration |
| `export-cli/` | asset / scene export 工具 |
| `executor-wasm/`、`executor-android/` | 其他平台 executor |

目前 `omfx` spawn backend 的路徑 hard-coded 到 debug backend。`run_stress.bat` 為了確保壓測使用 release backend，會在 release build 後把 release `omobab.exe` stage 到 debug spawn path。

## Script ABI And Content

Gameplay content 分成兩層：Lua template/story data 與 Rust script DLL。

| 層級 | 位置 | 說明 |
|---|---|---|
| Lua story / templates | `scripts/lua_data/` | 定義 story、map、hero/tower/creep templates；`omb/game.toml` 的 `STORY` 指向其中一個 generated story id |
| Template ID codegen | `omoba-template-ids/` | build-time 讀 Lua / template data，產生 typed IDs 給 Rust 端使用 |
| Stable ABI contract | `scripts/script-abi/` | `UnitScript`、`AbilityScript`、`GameWorld`、`stat_keys` 等 ABI-safe 型別與 trait |
| Base content DLL | `scripts/base_content/` | 內建塔、英雄、召喚物與技能實作，編成 cdylib 給 `omb` host 載入 |

`scripts/script-abi` 是 host 與 cdylib 的唯一共用 crate，只能放 `abi_stable` 可跨 DLL 邊界安全傳遞的型別。不要在這裡引入 `specs`、`serde_json` 或 host-only dependency。

英雄頭像由 `scripts/lua_data/templates/heroes.lua` 的 `portrait` 欄位指定，預設檔放在 `omfx/data/hero_portraits/`。技能圖示由 `scripts/lua_data/templates/abilities.lua` 的 `icon` 欄位指定；雜賀孫市沿用既有 `omfx/data/hero1_1.png` 到 `hero1_4.png`，新 placeholder 放在 `omfx/data/ability_icons/`。企劃要換圖時直接替換同名 PNG；每個英雄與技能都有獨立檔名，例如 `hero_saika_magoichi_portrait.png`、`ability_flame_blade.png`。

主要 script traits：

| Trait | 用途 |
|---|---|
| `UnitScript` | 塔 / 英雄 / creep / summon 的 tick、attack hook 與生命週期行為 |
| `AbilityScript` | Q/W/E/R 等技能施放、升級與效果邏輯 |
| `GameWorld` | script 回呼 host 的 ABI-safe world API，例如查詢單位、套 buff、產生 projectile / summon |

## Ability Runtime And Buffs

`omb/src/ability_runtime/` 負責 script 與 ECS stats 的 runtime glue。

- `BuffStore` 管理 entity -> buff list。
- buff payload 使用 JSON-like key/value 慣例，`*_bonus` 表示 additive，`*_multiplier` 表示 multiplicative。
- stat aggregation 透過 `sum_add` / `product_mult` 類型 helper 聚合。
- `UnitStats` 提供 script 查詢與操作單位屬性的 helper。
- `Dispatcher` 快取 ability / unit script entry，避免 hot path 重複查找。

Hero stats 由後端約每 0.3 秒對每個 hero 廣播一次 `hero.stats` snapshot。payload 內含已套用 buff 的最終屬性，以及 `buffs` 陣列；`remaining = -1` 表示 toggle / infinite 類型。前端會本地倒數顯示，下一次 server snapshot 會重新校正。

## Transport And Protocol

`omb` 與 `omoba-core` 共有三種 transport feature：`mqtt`、`grpc`、`kcp`。預設是 `kcp`，port 來自 `omb/game.toml`，目前預設 `50061`。

| Feature | 用途 |
|---|---|
| `kcp` | 預設 runtime transport，使用 `tokio_kcp`、`prost` 與 LZ4 |
| `grpc` | tonic / prost server-streaming 與 query API |
| `mqtt` | legacy / tooling 用途，仍保留 feature |

KCP frame 格式為 `[1B tag][4B len BE][prost payload]`。常用 tag：

| Tag | Payload |
|---|---|
| `0x01` | `PlayerCommand` |
| `0x02` | `GameEvent` |
| `0x03` | `CommandAck` |
| `0x04` | `SubscribeRequest` |
| `0x05` | `GameStateRequest` |
| `0x06` | `GameStateResponse` |

`proto/game.proto` 同時保留 typed payload 與 JSON fallback。新 event 若進入熱路徑，應優先考慮 typed payload 與 quantized scalar，避免高頻 JSON allocation。

## Game Modes And Data Flow

`omb/game.toml` 的 `[server].STORY` 決定目前載入的 story，例如 `TD_1`、`MVP_1`、`DEBUG_1`、`TD_STRESS`。Lua source 位於 `scripts/lua_data/{STORY}/` 與 `scripts/lua_data/templates/`。

TD 模式由 backend resources 控制 wave、player lives、tower kind、漏怪與減速等行為。MOBA / hero 相關狀態則透過 hero components、ability runtime、buff store 與 hero stats snapshot 串起前後端 HUD。

`game.toml` 也包含 spatial index 設定：

| 區域 | 預設 | 說明 |
|---|---|---|
| vision | `quadtree` | 視野 shadow casting 查詢 |
| tower collision | `bvh` | tower 變更少、查詢多，適合 BVH |
| creep collision | `sap` | creep 每 tick 大量 rebuild / sort，使用 SAP |
| hero collision | `sap` | hero 數量少，沿用 creep 風格 |
| region collision | `bvh` | region init 後幾乎只查詢 |

## Build Details

手動分步 build 可用：

```bat
cargo build --manifest-path scripts\Cargo.toml -p base_content
copy /y scripts\target\debug\base_content.dll omb\scripts\base_content.dll
cargo build --manifest-path omb\Cargo.toml -p omobab
cargo build --manifest-path omfx\Cargo.toml -p executor
```

Release stress build 的概念：

```bat
cargo build --release --manifest-path scripts\Cargo.toml -p base_content
copy /y scripts\target\release\base_content.dll omb\scripts\base_content.dll
cargo build --release --manifest-path omb\Cargo.toml -p omobab
cargo build --release --manifest-path omfx\Cargo.toml -p executor
```

實務上優先使用 `run.bat`、`run_stress.bat` 與 `gen_docs.bat`，因為它們會做 freshness check、DLL staging、process cleanup 與 stress config restore。

## Unit And Script API Catalog

想看目前 `base_content` 有哪些 towers / heroes / creeps / abilities、script API 長什麼樣、哪些 hook 有 override，使用：

```bat
gen_docs.bat
```

產物：`omb/target/docs/index.html`。

內容包含：

- Towers / Heroes / Creeps 完整屬性。
- `UnitScript` / `AbilityScript` / `GameWorld` API reference 與 doc comments。
- Stat Keys 分類說明。
- Coverage Matrix，列出每個 unit override 了哪些 `UnitScript` hook。

手動產生流程：

```bat
cargo build --manifest-path scripts\Cargo.toml -p base_content --release
copy /y scripts\target\release\base_content.dll omb\scripts\base_content.dll
cd omb
cargo run -p omobab --bin gen-docs --features gen-docs --release
```

Gen-docs smoke test：

```bat
cargo test --manifest-path omb\Cargo.toml -p omobab --features gen-docs -- --ignored
```

## Performance Notes

Stress 場景已驗證過 1000 towers x 1000 creeps 的熱路徑。新增高頻 gameplay 或 UI 行為時，請優先確認是否在每 entity / 每 frame 產生額外成本。

已知重要節流點：

- `omfx` collision ring 預設以 `COLLISION_RING_ENABLED: bool` 關閉，避免 24 segments x 1000 entities 形成大量 scene nodes。
- name label 的 `ui.send` 有 diff 節流，位置差小於 1 px 且文字未變時不送 UI update。
- 高頻 event 應使用 typed prost payload 與 quantized scalar，少走 JSON。
- `run_stress.bat` 會使用 release build 並產生 `TD_STRESS` map，適合檢查 CPU / render / network 熱點。

## Logs And Debugging

常見輸出位置：

| 路徑 | 說明 |
|---|---|
| `omfx_app.log` | omfx / sim_runner 側 log |
| `omfx.log` | Fyrox / frontend 相關 log |
| `omb/log/requests.log` | omb host side request / event log，可能很大 |
| `omb/target/docs/index.html` | `gen_docs.bat` 產生的 catalog |

`omb/game.toml` 支援 debug speed multiplier：`SPEED_MULT = 1`。執行中可在 `omb` stdin 輸入 `:speed 4`、`:speed 1` 等指令切換 1..=16 倍模擬速度；硬體跟不上時有效速度會低於設定值。

## Transient Fyrox Patches

`omfx` 跑在 Fyrox `1.0.1` 上，有兩個小 bug / 限制必須直接編輯 Cargo registry cache 的 Fyrox source 才能修。這些 patch 不會 commit 到任何 repo；每次 `cargo clean`、`cargo update`、換 machine 或清 cargo registry cache 都可能需要重新 patch 並重 build。

### Patch 1: 強制 vsync OFF

Upstream Fyrox `1.0.1` 的 `vsync: false` 在 Windows 是 no-op。`fyrox-graphics-gl/src/server.rs` 只在 `vsync=true` 時 `set_swap_interval(Wait(1))`，`false` 完全不 set，OS 預設常讓 Windows DWM compositor 鎖 60 Hz。

檔案：`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-graphics-gl-1.0.1/src/server.rs` 約第 664 行。

```rust
                if vsync {
                    Log::verify(gl_surface.set_swap_interval(
                        &gl_context,
                        SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
                    ));
                } else {
                    // Force vsync OFF (override OS/driver default which is usually
                    // vsync on for Windows DWM windowed apps).
                    Log::verify(gl_surface.set_swap_interval(
                        &gl_context,
                        SwapInterval::DontWait,
                    ));
                }
```

### Patch 2: 每 frame sleep 1 ms

vsync off 後 stress 場景能跑 280+ fps，但 CPU 會被佔滿。在 Fyrox 主 event loop 的 `Event::AboutToWait` 結尾加 `thread::sleep(1ms)` 把 CPU 讓出來。

檔案：`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-impl-1.0.1/src/engine/executor.rs` 約第 406 行，`run_normal` 函式內。

```rust
            Event::AboutToWait => {
                game_loop_iteration(
                    &mut engine,
                    ApplicationLoopController::ActiveEventLoop(active_event_loop),
                    &mut previous,
                    &mut lag,
                    fixed_time_step,
                    throttle_threshold,
                    throttle_frame_interval,
                    frame_counter,
                    &mut last_throttle_frame_number,
                );
                // omfx-local patch: 每 frame 強制 sleep 1ms 把 CPU 從 ~100% 降下來。
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
```

實測：280 fps -> 183 fps。大致組成是 render 3.15 ms + sleep 1 ms + Windows scheduler overshoot 約 1.5 ms，也就是約 5.5 ms/frame。

如果要更穩的 fps cap，例如 144 fps，可以把 `from_millis(1)` 改大。Windows `thread::sleep` 通常 overshoot 1 到 2 ms，每加 1 ms 大約再降 50 fps，但不是精準控制。

### timeBeginPeriod(1)

Windows 預設 timer granularity 是 15.6 ms。若只加 `thread::sleep(1ms)`，在乾淨或 idle 的 Windows 上可能實際 sleep 約 15 ms，導致 fps 鎖到 60。

`executor/src/main.rs` 已 commit 呼叫 `timeBeginPeriod(1)`，將 system-wide timer resolution 降到 1 ms。這通常只是保險；桌面環境常已有 Chrome / 遊戲等 process request 1 ms timer。

注意這只是 timer granularity 上限，不代表真正 1 ms sleep。Windows scheduler context switch latency 大約仍有 1 到 2 ms。若需要更精準的 1 ms 級 timing，才需要考慮 `CreateWaitableTimerEx(STATE_HIGH_RESOLUTION)` 或 spin-wait，目前不需要。

### 重 Patch 流程

```bat
rem 1. 編輯 ~/.cargo/registry/.../fyrox-graphics-gl-1.0.1/src/server.rs (Patch 1)
rem 2. 編輯 ~/.cargo/registry/.../fyrox-impl-1.0.1/src/engine/executor.rs (Patch 2)
rem 3. 強制重編 affected crates
cargo clean --release -p fyrox-graphics-gl --manifest-path omfx\Cargo.toml
cargo clean --release -p fyrox-impl --manifest-path omfx\Cargo.toml
rem 4. 重 build
cargo build --release --manifest-path omfx\Cargo.toml -p executor
```

### 持久化選項

目前選擇 transient patch，因為維護成本最低。

另一個選項是 fork `fyrox-graphics-gl` 與 `fyrox-impl`，在 `omfx/Cargo.toml` 用 `[patch.crates-io]` 指向 forked 路徑。但每次 Fyrox 升版都要 rebase patch，暫時不值得。
