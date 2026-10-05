# omoba

`omoba` 是以 Rust 實作的 MOBA / TD 雙模式遊戲。Repo 是 monorepo，權威模擬、script DLL、Fyrox 前端與 Unreal 前端分在多個 Cargo workspace 與 git submodule。

目前的分工是：

- `omb`（binary `omobab`）是權威 server process，負責設定、KCP／lockstep 連線與對局生命週期。
- 實際 ECS 玩法在 `omoba-core` 的 native runtime。`omb` 的 ability、item、script dispatch 等模組大多重新匯出這份實作，server 與 client replica 才會跑同一套模擬。
- `omoba-client-runtime` 維持每隊一份 filtered Specs replica，只用 loopback TCP 把呈現資料交給 renderer。
- 目前維護的 renderer 是 Unreal `omfue`。正式對局只吃 loopback presentation IPC，不跑權威模擬，也不載入 script DLL。Fyrox `omfx` 仍在 repo 裡，但是歷史前端，不再加功能或修相容。
- 英雄、技能、地圖與經濟的作者來源是 `scripts/lua_data`。`omoba-content-model` 在生成期驗證，`omoba-template-ids` 把結果編成 typed ID。正式遊戲 binary 使用 `compiled-content-only`，不帶執行期 Lua VM。

更細的維護注意事項見 [`AGENTS.md`](AGENTS.md)。進行中的設計與驗收紀錄在 [`docs/plans/`](docs/plans/) 與 [`openspec/`](openspec/)。

## Quick Start

主要在 Windows 上開發。根目錄 `.bat` 只負責呼叫固定的 Lua runtime，建置、DLL staging 與 process lifecycle 都在 `scripts/*.lua`。

這些 `.bat` 目前把 Lua 執行檔寫死成 `D:\code\omoba\tools\lua\lua.exe`，腳本路徑則用 `%~dp0`。Clone 到別的目錄時，要先改這個絕對路徑，或直接用同一支 `lua.exe` 執行對應的 `scripts/*.lua`。請從 `cmd.exe` 啟動 `.bat`。

| 情境 | 指令 | 實際腳本 |
|---|---|---|
| 第一次 clone 後拉 submodule | `git submodule update --init --recursive` | `omb`、`omfx`、`omfue`、`erps`、`specs`、`log4rs`、`map_editor`、`mqtt_log_viewer` |
| 正式 MOBA | 見下方 Lua 指令 | `scripts/run_moba_role_ue.lua`。`compiled-content-only`，Unreal 只呈現 |
| Headless MOBA | 見下方 Lua 指令 | `scripts/run_moba_headless.lua` |
| Unreal TD | `run_ue.bat` | `scripts/run_ue.lua`。預設單機 `TD_1`，環境關閉 runtime Lua；加 `--networked` 才另起 `compiled-content-only` backend |
| 雙隊 Unreal IPC | `run_2player_ue.bat` | `scripts/run_2player_ue.lua` |
| 歷史 Fyrox TD | `run.bat` | `scripts/run.lua`。`runtime-lua-content`、`TD_1`、`OMB_NO_HEROES=1`。不再維護 |
| 歷史 release TD | `run_10000.bat` | `scripts/run_10000.lua`。release `TD_1` 加 `OMB_TD_STARTING_GOLD=10000` |
| 歷史雙玩家 fog | `run_2player.bat` | `scripts/run_2player_interactive.lua`。兩份 Fyrox renderer |

維護範圍：`omfue` 是唯一維護的前端。`omfx`／Fyrox 與 `run.bat`、`run_10000.bat`、`run_2player.bat` 只留作歷史參考，不再新增功能、修復相容、建置或驗收。共用 Rust API 變更不再替 `omfx` 補接線；舊檔案沒有刪除。

正式 MOBA：

```bat
D:\code\omoba\tools\lua\lua.exe scripts\run_moba_role_ue.lua --help
D:\code\omoba\tools\lua\lua.exe scripts\run_moba_headless.lua
D:\code\omoba\tools\lua\lua.exe scripts\run_moba_role_ue.lua --hero 1=training_ranger
D:\code\omoba\tools\lua\lua.exe scripts\run_moba_role_ue.lua --interactive-selection
D:\code\omoba\tools\lua\lua.exe scripts\run_moba_role_ue.lua --recipe target\role-ue-runs\...\session\match-plan.json
```

`run_moba_role_ue.lua` 的預設配方是 `scripts/lua_data/moba_single_player.lua`，profile 是 `release`，KCP port 是 `57061`，story 是 `FOG_2TEAM_DEMO`。準備階段會寫一份新的 `game.toml`，把 `STEP_FPS` 設成 60，並把 `LUA_CONTENT`／`LUA_HOT_RELOAD` 設成 false。server、每位本機真人一份 client runtime，以及 presentation-only 的 Unreal 都吃這份設定。Bot 留在 server process，不拿 KCP 連線。

Lua 只在作者資料、建置生成和這個開局工具裡執行。`compiled-content-only` 與 `runtime-lua-content` 互斥；同時打開時 `omoba-template-ids` 會編譯失敗。server、client runtime、`base_content.dll` 與 Unreal bridge 的正常依賴不連結 Lua VM。bridge 的 catalog 由 `build.rs` 嵌入。

選角不改來源配方。`--hero PLAYER_ID=HERO` 只替換配方裡已經宣告的真人英雄，例如 `--hero 1=training_ranger`。準備後目錄裡有 `match-plan.json`、`selection-candidate.json`、`selection-lock.json` 與 `hero-catalog.json`。之後可以用 `--recipe FILE.json` 重開，JSON 來源本身不被改寫。鎖定規則在 `moba-config --lock-plan`，不在 Unreal 畫面裡重寫。

`--interactive-selection` 先開選角，不啟動 gameplay bridge。一位真人時走 `-om-hero-selection`，與 `moba-config` 用匿名 pipe 交換 compiled catalog、選擇與鎖定。兩位到十位真人走 `scripts/moba_shared_selection.lua`。完成才寫 `match-plan.json` 並沿既有開局繼續；關掉視窗就取消，不會自動開局。這不是 LAN 共享大廳。完整對局驗收仍待，進度在 `docs/plans/`。

常用測試：

```bat
cargo test --manifest-path omb\Cargo.toml -p omobab
cargo test --manifest-path scripts\Cargo.toml -p omb-script-abi
cargo test --manifest-path scripts\Cargo.toml -p base_content
cargo test --manifest-path omoba-core\Cargo.toml
cargo test --manifest-path omoba-client-runtime\Cargo.toml
cargo test --manifest-path omoba-template-ids\Cargo.toml
```

Lua 工具測試入口是 `D:\code\omoba\tools\lua\lua.exe scripts\test_lua_tooling.lua`。

## Toolchain

- Rust 固定為 `rust-toolchain.toml` 的 `1.95.0`。`abi_stable` 要求 host 與 `scripts/base_content.dll` 使用同一個 rustc。
- 工作流 Lua 固定為 `tools/lua/lua.exe`（Lua 5.4）。標準庫沒有的平台能力由 `tools/lua-host` 提供。不要為同一件事再加 PowerShell、Python 或 shell fallback。
- 根目錄沒有單一 Cargo workspace。常見 manifest：
  - `scripts/Cargo.toml`：`omb-script-abi`、`base_content`
  - `omb/Cargo.toml`：`omobab`
  - `omfx/Cargo.toml`：Fyrox `executor`
  - `omoba-client-runtime/Cargo.toml`
  - `omoba-core/Cargo.toml`、`omoba-sim/Cargo.toml`、`omoba-template-ids/Cargo.toml`、`omoba-content-model/Cargo.toml` 是 path dependency
- 正式 MOBA 建置加 `--features compiled-content-only`。`runtime-lua-content` 只留給歷史 Fyrox `run.lua`，兩個 feature 不能同時開。build dependency 裡的 mlua 只負責生成，不算遊戲執行期依賴。
- `.bat` 必須是 CRLF。`.gitattributes` 已固定 `*.bat text eol=crlf`。LF 會讓 `cmd.exe` 把每行首字吃掉，出現 `'M' is not recognized`。

## Repository Layout

### Git Submodules

| 路徑 | 說明 |
|---|---|
| `omb/` | 權威 server。package / bin 名稱是 `omobab`，repo 為 `github.com/damody/open_moba_backend` |
| `omfx/` | Fyrox renderer。執行入口是 `executor` |
| `omfue/` | Unreal renderer。project 是 `omfue/om.uproject`，C++ plugin 是 `OmRuntime`，Rust bridge 在 `omfue/bridge` |
| `erps/` | 獨立的配對、Elo 與信用 process。預設不嵌進 `omb` |
| `map_editor/` | 地圖編輯器 |
| `specs/` | forked `specs` ECS |
| `log4rs/` | forked `log4rs`，含 MQTT appender |
| `mqtt_log_viewer/` | MQTT log viewer |

Submodule 裡的修改要先在該 repo commit，再回到 monorepo 更新 gitlink。`omfue/` 目前不納入一般 cleanup，除非這次工作明確要改 Unreal project。

### Monorepo Directories

| 路徑 | 說明 |
|---|---|
| `omoba-core/` | 共用 schema、KCP/gRPC/MQTT client，以及 native deterministic runtime |
| `omoba-sim/` | `Fixed64`、向量、地形、導航與 deterministic RNG |
| `omoba-content-model/` | Lua 內容的共享驗證模型。技能數值、目標與宣告式 effect 在這裡檢查 |
| `omoba-template-ids/` | build-time 從 `scripts/lua_data` 產生 typed template ID、地圖與經濟常數 |
| `omoba-client-runtime/` | 每隊 filtered replica 與 renderer IPC host |
| `omoba-netem-proxy/` | loopback UDP delay proxy，給延遲與斷線驗證用 |
| `scripts/script-abi/` | host 與 script DLL 唯一共用的 stable ABI crate |
| `scripts/base_content/` | 塔、既有英雄腳本與 generic effect handler，編成 `base_content.dll` |
| `scripts/lua_data/` | story、map、template 與 MOBA 配方的 Lua source |
| `eui/` | `omfx` 使用的 immediate-mode GUI |
| `omb-mcp/` | 以 KCP query-only 查 `omb` 的 MCP server，不訂閱 event 洪水 |
| `omb-ws-bridge/` | 瀏覽器 WebSocket 到既有 KCP backend 的轉送 |
| `proto/game.proto` | prost / tonic 共用 wire schema |
| `tools/lua/` | 受版控的 Lua 5.4 runtime |
| `docs/plans/` | 架構與實作紀錄 |
| `docs/selective-lockstep/` | secure selective lockstep V2 契約 |
| `docs/protocol/` | renderer IPC 說明。程式裡的 magic / version 以 `presentation_bridge.rs` 為準 |
| `openspec/` | 變更提案與驗收證據 |

`omb/` 單獨 clone 無法 build。它依賴 `../omoba-core`、`../omoba-sim`、`../omoba-template-ids`、`../scripts/script-abi` 與 `../specs`。

## Runtime Architecture

正式路徑是 Unreal presentation IPC。歷史 Fyrox TD 才由 `run.bat` 啟動 `omfx/target/<profile>/executor.exe`，再 spawn `omobab.exe`：

```text
player input
  -> omfx executor
  -> omoba-core KCP client
  -> omob transport / lockstep
  -> omoba-core native ECS runtime
  -> scripts/base_content.dll
  -> GameEvent / snapshot
  -> omfx scene + EUI HUD
```

上面這條使用 `runtime-lua-content`，不再維護。正式 MOBA 的 renderer 不連權威 KCP，也不載 script DLL：

```text
Unreal omfue (presentation-only, compiled-content-only)
  -> loopback TCP RendererIpcEnvelope
  -> omoba-client-runtime
       filtered team replica + presentation snapshot
  -> KCP selective lockstep V2
  -> omob
       authoritative SimulationDriver
       team projector / fog / shop / recall facts
  -> base_content.dll
```

`omoba-client-runtime` 的呈現幀是 4-byte big-endian length 加 protobuf。`PRESENTATION_MAGIC` 是 `0x4f4d5254`，`PRESENTATION_PROTOCOL_VERSION` 目前是 `3`，單一 frame 上限 8 MiB。常數在 `omoba-client-runtime/src/presentation_bridge.rs`。`render_id` 只在單一 team replica 與 disclosure epoch 內有效，不是 Specs entity ID。

Renderer 不執行 Specs systems、不載入 script DLL、不連權威 KCP，也不把 remembered ghost 當成可選定的單位。未知版本、過長 frame 或非 loopback endpoint 必須 fail closed。

## Authoritative Simulation

玩法實作在 `omoba-core/src/runtime/native/`，不是只留在 `omb/src`。

| 位置 | 職責 |
|---|---|
| `simulation_driver.rs` | 共用、不 sleep 的 ECS driver。`Production60Hz`、`Production120Hz`、`Coarse15Hz` 只改每 tick 的模擬時間，不跳過 system |
| `tick/` | 移動、攻擊、傷害、投射物、重生、波次、塔、物品與 regen |
| `ability_runtime/` | `BuffStore`、`UnitStats`、法力池與 ability registry |
| `scripting/` | script event queue、dispatch，以及 `GameWorld` adapter |
| `moba_match.rs` | opt-in 的 MOBA 規則：兵線、塔、野怪、擊殺、助攻、回城、商店與勝負 |
| `moba_match/bots/` | server 內的 Bot。只讀已披露狀態，再送正式 `MoveTo` / `CastAbility` / `UpgradeAbility` / `ItemBuy` |
| `spatial/` | quadtree、hash grid、BVH、sweep-and-prune |
| `game_processor.rs` | 把已接受的輸入變成移動、施法、升級、造塔與買賣 |
| `initialization.rs` | 由 story / scene 建立 ECS world |

`omb` 仍負責 process 邊界：`game.toml`、transport、lockstep admission、vision / AOI，以及 `gen-docs`、`moba-headless`、`moba-config` 這幾個 binary。新增玩法狀態時，先決定它是權威 ECS component、filtered fact、script payload，還是只有 renderer 看的呈現資料。

共用 lockstep helper 的 `LOCKSTEP_TPS` 是 120。`SUPPORTED_LOCKSTEP_FPS` 是 120、90、60。倉庫裡的 `omb/game.toml` 仍寫 `STEP_FPS = 120`，V2 契約文件也以 120 Hz 換算 microsecond。正式 role launcher 不改這個檔；它寫出的對局 toml 把 `STEP_FPS` 固定成 60，`moba-config` 的 preflight 也要求 `tick_rate_hz == 60`。`ticks_to_micros` 用整數 tick，不用 wall clock 反推。

`SPEED_MULT` 只影響 debug 快轉。`omb` stdin 可輸入 `:speed 4` 或 `:speed 1`，範圍 `1..=16`。硬體跟不上時，有效速度會低於設定值。

## Client Runtime

`omoba-client-runtime` 是外部 process。啟動後它會：

1. 用 KCP 連權威 server，協商 selective lockstep protocol version 2。
2. 用 `TeamGameStart` 建立該隊的 filtered replica。
3. 在 `--presentation-bind` 上聽 loopback TCP，送 latest-wins snapshot 與 ordered critical result。
4. 把 renderer 的 `RendererInput` 轉成 secure target input。不在 client 上直接改 HP、CD 或視野。
5. 回報 replica checkpoint。hash 不符時走 rebase，不猜 hidden state。

Server 只把該隊看得到的 fact 放進 team stream。Fog grid、商店回條與回城狀態都是這條投影的一部分，不是 renderer 自己算出來的。

## Frontends

### Unreal (`omfue`)

這是目前要維護的前端。`run_ue.bat` 與 `run_moba_role_ue.lua` 找 `UE_5_8_ROOT`、`UE_ROOT`、`UE_5_7_ROOT`，否則依序試 `D:/UE5.8`、`D:/UE_5.8`、`C:/Program Files/Epic Games/UE_5.8`，再試 5.7 的同樣三個位置。預設地圖是 `/Game/Map/Main`，RHI 預設 D3D11。

| 路徑 | 職責 |
|---|---|
| `omfue/bridge` | Rust cdylib。預設 feature 含 `compiled-content-only`。把 presentation IPC 轉成 Unreal frame；catalog 在建置時嵌入 |
| `omfue/codegen` | 作者期從 Lua catalog 產生 C++ 註冊。bridge 的正常依賴關閉 authoring，不在遊戲裡求值 Lua |
| `Plugins/OmRuntime` | 原生 HUD、小地圖、輸入、選角與場景 actor |
| `Plugins/BpGeneratorUltimate` | Editor MCP、資產與 PIE 工具。不是遊戲執行期依賴 |
| `omfue/restart` | 停掉 Editor、編 bridge 與 OmGame 的本機工具。`scripts/build_ue_moba.lua` 會先編它 |

正式對局加 `-om-presentation-only`，不代入 KCP endpoint。`run_ue.bat` 不加 `--networked` 時仍是 `-om-single-player`，那是單機 TD，不是 MOBA 權威模擬。

### Fyrox (`omfx`，不維護)

| 路徑 | 職責 |
|---|---|
| `executor/` | 舊 desktop launcher。Windows 上呼叫 `timeBeginPeriod(1)` |
| `game/` | `sim_runner`、`render_bridge`、lockstep client 與 HUD |
| `editor/`、`export-cli/` | Fyrox editor 與 asset export |
| `executor-wasm/`、`executor-android/` | 其他平台 executor |

`run.bat` 仍由 `executor` spawn debug 或 `OMOBA_BUILD_PROFILE` 指定的 `omobab.exe`。雙隊 fog 腳本會設 `OMFX_RENDERER_ONLY=1` 與 `OMFX_PRESENTATION_ADDR`。這些行為只描述現有檔案，不是現行開發入口。

## Content Pipeline

Lua 是作者來源，不是正式遊戲的腳本 VM。宣告順序就是 ID 順序；ID 0 保留為 `UNSPECIFIED`，不要重排既有條目。通用效果生成 Rust handler；特殊行為仍寫 Rust。Unreal 只用生成出來的 C++ 與嵌入 catalog。

| 層級 | 位置 | 說明 |
|---|---|---|
| Templates | `scripts/lua_data/templates.lua` | 聚合 towers、heroes、abilities、buffs、summons、creeps、projectile kinds、TD layers、MOBA items、economy、maps |
| Stories | `scripts/lua_data/<STORY>/` | `map.lua`、`entity.lua`、`ability.lua`、`mission.lua` |
| MOBA recipes | `scripts/lua_data/moba_*.lua` | 角色配方、archetype、單人與 mana 對局 |
| Validation | `omoba-content-model` | 等級曲線、目標型別與宣告式 effect。非法內容在生成期失敗 |
| Codegen | `omoba-template-ids/build.rs` | typed ID 與編譯期地圖、經濟、野怪常數 |
| Script DLL | `scripts/base_content` | `UnitScript` / `AbilityScript`。正式建置用 `compiled-content-only` |
| Unreal codegen | `omfue/codegen` 與 bridge `build.rs` | 同一份生成資料。遊戲 process 不再依 `content_root` 執行 Lua |

倉庫裡的 `omb/game.toml` 仍是舊 DEV 預設：`LUA_CONTENT = true`、`LUA_HOT_RELOAD = true`。正式 launcher 不改這個檔，而是寫到新的輸出目錄，並在那份 toml 與 process 環境把 Lua 關掉。`OMB_LUA_CONTENT=0`、`OMB_LUA_HOT_RELOAD=0`。

`templates/heroes.lua` 與 `templates/abilities.lua` 會再包含 `templates/moba_archetypes.lua`。英雄 `portrait` 例是 `data/hero_portraits/hero_saika_magoichi_portrait.png`。Fyrox 會從 `omfx/data/` 讀這些圖，技能 fallback 是 `data/ability_icons/ability_default_placeholder.png`；那是舊前端的資料路徑。

`scripts/script-abi` 是 host 與 cdylib 的唯一共用 crate。只能放 `abi_stable` 能跨 DLL 傳遞的型別，不要加 `specs`、`serde_json` 或 host-only dependency。

| Trait | 用途 |
|---|---|
| `UnitScript` | 塔、英雄、creep、summon 的 tick、攻擊與生命週期 |
| `AbilityScript` | 技能施放、升級與效果 |
| `GameWorld` | script 回呼 host：查單位、套 buff、產生 projectile 或 summon |

## Ability Runtime And Buffs

`omoba-core/src/runtime/native/ability_runtime/` 把 script 與 ECS stats 接起來。

- `BuffStore` 管 entity 的 buff list。
- `*_bonus` 是 additive，`*_multiplier` 是 multiplicative。聚合用 `sum_add` / `product_mult` 這類 helper。
- `UnitStats` 是 script 讀寫單位屬性的入口。
- Dispatcher 快取 ability / unit script，避免熱路徑重複查找。
- 法力是 opt-in。`ManaPool` 用 Q10 餘額；未協商或未啟用的對局不會先扣魔。

Hero stats 仍約每 0.3 秒對每個 hero 廣播一次。payload 含套用 buff 後的最終屬性與 `buffs`。`remaining = -1` 表示 toggle 或無限持續。前端可以本地倒數，下一次 server snapshot 會校正。

## Transport And Protocol

`omb` 與 `omoba-core` 的 transport feature 是 `mqtt`、`grpc`、`kcp`，預設 `kcp`。Port 來自 `omb/game.toml`，目前是 `50061`。

| Feature | 用途 |
|---|---|
| `kcp` | 預設 runtime。`tokio_kcp`、`prost`、LZ4 |
| `grpc` | tonic server-streaming 與 query API |
| `mqtt` | legacy / tooling |

KCP frame 是 `[1B tag][4B len BE][payload]`。payload 至少 64 bytes 且 LZ4 後更小時，tag 會 OR `0x80`。`omb-ws-bridge` 對瀏覽器送出前會把壓縮幀解開。

| Tag | 方向與用途 |
|---|---|
| `0x01` | `PlayerCommand` |
| `0x02` | `GameEvent` |
| `0x03` | `CommandAck` |
| `0x04` | `SubscribeRequest` |
| `0x05` / `0x06` | `GameStateRequest` / `GameStateResponse` |
| `0x07` | `ViewportUpdate` |
| `0x10`–`0x18` | 舊 lockstep：input、tick batch、hash、join、snapshot、ping |
| `0x20`–`0x2C` | selective lockstep V2：team start、team frame、rebase、replay、hash mismatch、secure input、checkpoint、session close、shop receipt |

`SELECTIVE_LOCKSTEP_PROTOCOL_VERSION` 是 2。V1 與 V2 以對局為單位協商，同一條 player session 不能混用。`proto/game.proto` 同時保留 typed payload 與 JSON fallback。熱路徑優先用 typed payload 與 quantized scalar。

`game.toml` 目前設 `MATCH_LOCKSTEP_MODE = "secure_v2_required"`。Secure match 不得攜帶 global master seed、raw ECS ID 或其他隊伍的 hidden state。契約全文在 [`docs/selective-lockstep/protocol-v2-contract.md`](docs/selective-lockstep/protocol-v2-contract.md)。

## Game Modes And Data

`[server].STORY` 選擇 story。Lua source 在 `scripts/lua_data/<STORY>/`。

| Story | 用途 |
|---|---|
| `TD_1` | `run.bat` 與 `game.toml` 的預設 TD |
| `TD_FARMSTEAD_BENDS`、`TD_FROZEN_BRIDGE`、`TD_GREEN_CROSSROADS`、`TD_MINE_CORRIDOR`、`TD_MOLTEN_FORK`、`TD_RIVERSIDE_PATH`、`TD_TIDAL_HARBOR`、`TD_TWILIGHT_MAZE`、`TD_TWIN_GATE_OUTPOST` | 其餘 TD 地圖 |
| `TD_STRESS` | 壓測地圖。由 `scripts/gen_stress_map.lua` 依 `omb/game_stress.toml` 產生，不是 `run_10000.bat` 的預設 |
| `MVP_1` | 早期 MVP story |
| `FOG_2TEAM_DEMO` | `run_2player` 使用的雙隊 fog story |

MOBA 不是另一個 ECS。`SingleLaneConfig` 為 `None` 時維持 TD / 舊行為；有設定時才啟用兵線、塔、野怪、經濟與 Bot。地圖、物品與經濟在 `templates/moba_maps.lua`、`moba_items.lua`、`moba_economy.lua`、`moba_archetypes.lua`。

`game.toml` 的 spatial index：

| 區域 | 預設 | 原因 |
|---|---|---|
| vision | `quadtree` | 視野 shadow casting |
| tower collision | `bvh` | 變更少、查詢多 |
| creep collision | `sap` | 每 tick 大量 rebuild |
| hero collision | `sap` | 數量少，沿用 creep |
| region collision | `bvh` | 初始化後幾乎只查詢 |

可選實作是 `quadtree`、`hash_grid`、`bvh`、`sap`。

## Build

正式產物用 `compiled-content-only`。`scripts/build_ue_moba.lua` 會編 `base_content`、把 DLL stage 到 `scripts/base_content.dll`，再經 `omfue/restart` 編 bridge 與 OmGame。不要把 DLL 複製到 `omb/scripts/`。`game.toml` 的 `DLL_PATH = "../scripts/base_content.dll"` 指的是 repo 的 `scripts/base_content.dll`。

```bat
cargo build --manifest-path scripts\Cargo.toml -p base_content --features compiled-content-only
cargo build --manifest-path omb\Cargo.toml -p omobab --features compiled-content-only
cargo build --manifest-path omoba-client-runtime\Cargo.toml --features compiled-content-only
D:\code\omoba\tools\lua\lua.exe scripts\build_ue_moba.lua --build-only
D:\code\omoba\tools\lua\lua.exe scripts\test_compiled_content_contract.lua
```

最後一條檢查 server、client runtime、script DLL 與 Unreal bridge 的 normal dependency 沒有 `mlua`。Release 配方 launcher 預設 `--profile release`。

歷史 Fyrox `run.lua` 仍用 `--features runtime-lua-content` 編 `base_content`、`omobab` 與 `omfx` 的 `executor`。不要拿這組 feature 編正式 MOBA。

ERPS 是另一個 process，listen 預設 `127.0.0.1:50051`。只有 `omb` 加上 `--features erps-game-server` 才會接配對與結算。操作說明在 [`docs/erps/README.md`](docs/erps/README.md)。

WebSocket bridge：

```bat
cargo run --manifest-path omb-ws-bridge\Cargo.toml -- 127.0.0.1:50062 127.0.0.1:50061
```

## Unit And Script API Catalog

```bat
cargo build --manifest-path scripts\Cargo.toml -p base_content --release
copy /y scripts\target\release\base_content.dll scripts\base_content.dll
cargo run --manifest-path omb\Cargo.toml -p omobab --bin gen-docs --features gen-docs --release
```

`gen-docs` 依序找 `scripts/base_content.dll`、`../scripts/base_content.dll`，然後才是 `scripts/target/<profile>/base_content.dll`。從 repo 根目錄執行時，上面的 stage 路徑就是第一個候選。

產物是 `omb/target/docs/index.html`。內容有 towers / heroes / creeps、`UnitScript` / `AbilityScript` / `GameWorld`、stat keys，以及每個 unit override 了哪些 hook。

Gen-docs smoke test：

```bat
cargo test --manifest-path omb\Cargo.toml -p omobab --features gen-docs -- --ignored
```

## Performance Notes

高頻 event 用 typed prost payload 與 quantized scalar。正式 MOBA 的 presentation snapshot 可以降頻，權威 tick 仍由 server 的 `STEP_FPS` 決定；role launcher 寫進對局 toml 的值是 60。

下面是舊 Fyrox stress 路徑留下的限制，不是 Unreal 的現行驗收：

- stress 場景曾跑過約 1000 towers x 1000 creeps。
- `omfx` 的 `COLLISION_RING_ENABLED` 預設 `false`。
- name label 的 UI send 有 diff 節流；位移小於 1 px 且文字沒變就不送。
- `run_10000.bat` 是 release `TD_1` 加上 10000 起始金錢，不是 `TD_STRESS`。大地圖腳本是 `scripts/gen_stress_map.lua`，讀 `omb/game_stress.toml`。

## Logs And Debugging

| 路徑 | 說明 |
|---|---|
| `omfx_app.log` | omfx / sim_runner |
| `omfx.log` | Fyrox / frontend |
| `omb/log/requests.log` | omb request / event，可能很大 |
| `omoba-client-runtime/target/runtime-logs/` | client runtime 的 team stdout / stderr |
| `openspec/changes/.../evidence/` | `run_2player` 這類驗證留下的 manifest 與 log |
| `omfue/Saved/Logs/` | Unreal |

`omobab`、`executor` 與 `omoba-client-runtime` 在 Windows 上都會呼叫 `timeBeginPeriod(1)`，把 timer granularity 從 15.6 ms 降到 1 ms。這只是上限，不是精準的 1 ms sleep。

## Fyrox Frame Pacing

這一節只解釋還留在 repo 裡的 Fyrox 前端。不要為了現行 MOBA 去改這些檔。

`omfx` 使用 Fyrox `1.0.1`。`omfx/Cargo.toml` 用 `[patch.crates-io]` 把 `fyrox-impl` 指到 `third_party/fyrox-impl-1.0.1`。frame pacing 已經在這份 vendor source 裡，不要再去改 Cargo registry 的 `fyrox-impl`。

`third_party/fyrox-impl-1.0.1/src/engine/executor.rs` 的 `Event::AboutToWait` 會看離下一個 fixed step 還有多久：

- 剩餘時間大於 2 ms 時，最多 `sleep` 1 ms，並留下 2 ms spin window。
- 進入 spin window 後用 `spin_loop`，在剩餘約 250 µs 時返回。
- 這樣是為了在 Windows 上靠近 120 Hz，而不是每 frame 固定睡 1 ms。

`fyrox-graphics-gl` 仍來自 crates.io，沒有 vendor。Upstream `1.0.1` 的 `vsync: false` 在 Windows 是 no-op：`server.rs` 只在 `vsync=true` 時 `set_swap_interval(Wait(1))`，`false` 什麼都不設，DWM 常把視窗鎖在 60 Hz。若本機仍被鎖 60 Hz，要改 registry 裡的 `fyrox-graphics-gl-1.0.1/src/server.rs`，在 `else` 分支呼叫 `SwapInterval::DontWait`。這份修改不進 git；`cargo clean`、`cargo update` 或清 registry 之後要重做，並重編 `fyrox-graphics-gl`。

`omobab`、`omfx/executor` 與 `omoba-client-runtime` 在 Windows 上都會呼叫 `timeBeginPeriod(1)`。沒有它時，`sleep(1ms)` 在乾淨的 Windows 上可能實際睡約 15 ms。這只把 timer granularity 降到 1 ms，不代表 sleep 本身精準到 1 ms。
