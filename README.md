# omoba

目前進度更新至 **2026-10-26（Asia/Taipei）**。本機專案為 `D:\omoba`，Unreal 為 `D:\UE5.8`。換機／接續開發請先讀 [handoff.md](handoff.md)；完整功能盤點見 [整體完整度盤點](docs/plans/2026-10-21-whole-game-completeness-audit.md)，各批驗收與錯誤紀錄位於 [docs/plans](docs/plans/) 和 [docs/acceptance](docs/acceptance/)。[2026-10-18 可攜交接](docs/acceptance/2026-10-18-handoff/README.md)保留為歷史基準，最新狀態以本頁及各批報告為準。

## 目前實作與驗收進度

### omfue 月度 MVP（2026-10-24 起，進行中）

計畫與每日紀錄在 [docs/plans/2026-10-24-omfue-month-mvp/](docs/plans/2026-10-24-omfue-month-mvp/README.md)：[執行狀態](docs/plans/2026-10-24-omfue-month-mvp/execution-status.md)、[錯誤紀錄](docs/plans/2026-10-24-omfue-month-mvp/errors.md)、[發佈說明](docs/plans/2026-10-24-omfue-month-mvp/release-notes.md)、[玩家指南](docs/plans/2026-10-24-omfue-month-mvp/player-guide.md)、[英雄製作指南](docs/plans/2026-10-24-omfue-month-mvp/hero-authoring-guide.md)、[Unreal 美術指南](docs/plans/2026-10-24-omfue-month-mvp/unreal-art-guide.md)、[效能報告](docs/plans/2026-10-24-omfue-month-mvp/perf-report.md)、[下個月](docs/plans/2026-10-24-omfue-month-mvp/next-month.md)。以下證據都是本機 B＋C＋C（1 個權威 server、2 個 client runtime、2 個 UE，另有 8 個 bot）實機局，由 omfue autopilot 透過真實輸入管線操作，log 位於 `target/mvp-a3/`。

| 項目 | 狀態與證據 |
|---|---|
| 可玩對局 | 標準地圖與 quick 測試地圖都能自然結束並顯示結算。追查到的 replica parity 問題（召喚物攻擊計時 E083、英雄 HP 套用順序 E087）都已修正，並在 split51／split53 實機驗證為 0 mismatch。 |
| 英雄（描述驅動製作） | H01 雜賀孫市、H02 伊達政宗、H03 阿松、H04 阿市、H05 稻姬、H06 淺井長政實機逐招驗收通過（split46／49／50／51，`scripts/check_hero_live_acceptance.lua`）；盲測 I1 玉子、I2 村上義清 8/8 招通過、0 mismatch（split53；replica HP 套用順序 E087 由可重播輸入紀錄 E086 定位）。 |
| 網路 | netem 80±20 ms、1% 掉包；10 秒斷線自動恢復；30 秒以上斷線後重新加入同一場（split44，E077）。 |
| 效能 | 伺服器 tick 平均 13.21→6.18 ms、p99 視窗 35.37→12.41 ms，UE 59.5 FPS（split37 對 split43）。 |
| 打包 | Development 與 Shipping 打包，從乾淨目錄啟動對戰。 |
| UE 測試 | 全量 `Om.` 286/286，`-d3d12 -NoSound` 與 `-nullrhi` 兩種模式都通過。 |
| HUD 美術 | 每個英雄技能都有原創圖示、每位英雄都有正式頭像（Grok 生成，E091／E092）；沒有正式模型的英雄顯示共用人形 fallback（E093）。 |
| 尚未完成 | 真雙機 LAN、OS 150% DPI、使用者親自試玩（需要使用者）；6 位新英雄的正式 3D 模型要在 Editor 指定。 |

原 MVP 清單維持 **33／44 完成、11 個父項待驗收**。這是原清單的接受狀態，不代表整個遊戲或所有 LOL 類玩法已完成。近期單機功能增量不會自動勾完要求真權威回覆、跨機或實體環境證據的父項；詳見 [MVP 功能核對](docs/plans/2026-10-18-mvp-functional-task-audit.md)。

### 各系統目前狀態

| 系統 | 已實作並具單機證據的內容 |
|---|---|
| 正式架構與生命期 | Rust 權威 ECS、60Hz 正式流程、隊伍 filtered replica、Unreal presentation-only；選角、開始／結束、死亡／重生、回城、暫停及來源／世代退休。 |
| 地圖 | 兩張 MOBA map；左下／右上基地、三路、河流、野區、牆體、導航／碰撞及公開地形呈現，已取代原先左右對稱配置。 |
| 視野與草叢 | LOS／範圍、草叢連通、隱身與偵測、普攻揭露、ward、探索霧格；隱藏或退休單位不當成可選目標。 |
| 小兵與兵線 | 近戰／遠程／砲車／超級兵、波次組成、路線、索敵、護主與推塔例外、成長及強化兵線；受控與屬性修正共用正式規則。新增資料驅動基礎攻擊間隔：近戰／超級兵 1 秒、遠程兵 1.5 秒、砲車 2 秒。 |
| 塔與基地建築 | 分層防禦塔、前置開門、兵營／基地塔、摧毀／重生、同英雄連續攻擊壓力；首塔／分層獎勵、限時塔皮分段金／護甲、名牌與權威公告。 |
| 野區 | 16 營地、32 初始主體；群組仇恨、耐心、軟／硬回頭、回血、重生、LOS、每個體與召喚鏈歸因；個人／隊伍／史詩獎勵、Buff 傳承／疊層／光環與有界死亡分裂。 |
| NPC 戰鬥 | 小兵與每個野怪個體共用暈眩／定身／移速、攻速／射程與上限、傷害修正及一次普攻加傷；拒絕出手不消耗加傷，已提交恢復週期不因增益到期重算。塔保留獨立規則。 |
| 英雄與技能 | 九個英雄配裝；rank／技能點、Mana、有效成本／冷卻／前搖、傷害／治療／護盾／控制／淨化／削魔、普攻被動、位移、圓形／扇形／沿路效果、召喚、引導／脈衝與獨立地面區域。 |
| 施法操作與提示 | 三種施法模式、改鍵、自用、升級；單位／圓形／走廊／陣形／扇形預覽，逐級數值、公開控制／資源／射程／方向提示及 tooltip；預覽不冒充權威落點接受。 |
| 商店與經濟 | 19 項道具，吸血／暴擊、引導／控制／淨化、唯一被動與購買群組、合成、購買／出售撤銷；被動金、尾刀、附近 XP、擊殺／助攻、首殺、終結賞金及目標經濟補償。 |
| 召喚師與 Bot | 五種召喚師法術及 ward 生命期；Bot 沿正式輸入操作，包含友方治療／淨化、自用淨化、引導回血道具及召喚師淨化策略。 |
| HUD 與選取 | 技能／道具／召喚師欄、計分板、隊友卡、商店、設定、聊天／ping 介面；護盾、控制／緩速倒數、塔皮、野怪返回狀態；名牌畫面間距已完成。選取卡片新增目前公開攻擊範圍、到自身英雄的距離及自身普攻範圍內／外提示。 |
| 動畫、音效與呈現 | 目錄驅動角色與技能呈現、攻擊／施法／引導進度、傷害／MISS／CRIT、護盾與 Buff 視覺、公開目標播報、獨立音量與字幕；跨 world、隱藏、死亡、重生及來源退休有回歸證據。 |

上述為目前已接線的機制與內容，不表示每個英雄／兵種／營地已具專用美術、完整平衡或所有競技規則。完整歷史增量與各自限制以 [整體盤點](docs/plans/2026-10-21-whole-game-completeness-audit.md)及其連結的接受報告為準。

兵種間隔欄位為 `templates/moba_maps.lua` 中的 `attack_interval_ms`，單位為整數毫秒，接受範圍 `1..10000`；舊內容省略時預設 `1000`，明確填入非法值會拒絕。生成時保留整數，出生時一次量化為 Q10，再沿共用攻速規則計算每次提交的週期。

### 最近完成的批次

| 批次 | 完成內容與證據 |
|---|---|
| 兵種攻擊節奏 | Lua 嚴格 schema → 生成常數 → 兵線 AI；兵種基礎間隔、canonical 攻速、鎖定恢復週期、暫停／到期／公開進度與重播。[報告](docs/plans/2026-10-25-minion-cadence-final-acceptance.md)／[證據索引](docs/acceptance/2026-10-25-minion-cadence/index.json)。 |
| 選取距離與普攻範圍提示 | 目前公開位置、來源守門、平面世界單位換算、位置或自身射程單獨更新、未知資料清除及窄卡片換行。[報告](docs/plans/2026-10-25-selected-distance-final-acceptance.md)。 |
| 選取目標攻擊範圍 | 公開 frame → bridge → typed payload → 實際 Slate 卡片；射程更新、非法值與生命期清除。[報告](docs/plans/2026-10-25-selected-range-final-acceptance.md)。 |
| 共用公開射程 | 英雄、小兵、野怪的 resolved range 接通兩份權威快照、隊伍公開 baseline／update、唯讀 ECS 及 CABI；不建立假的攻擊元件或公開私有 Buff。[報告](docs/plans/2026-10-25-public-range-final-acceptance.md)。 |
| NPC 傷害／攻擊／控制 | 正式屬性修正、一次加傷、已提交攻擊時鐘、每個野怪個體控制與回營政策。[傷害](docs/plans/2026-10-25-npc-damage-final-acceptance.md)／[攻擊](docs/plans/2026-10-25-npc-attack-final-acceptance.md)／[控制](docs/plans/2026-10-25-npc-control-final-acceptance.md)。 |
| 敵對野怪道具 | 共用 hostile 陣營契約，可指定可見敵對野怪；被動中立、隱藏、無敵回營、建築及非法距離仍守門。[報告](docs/plans/2026-10-25-hostile-item-final-acceptance.md)。 |
| 支援與淨化內容 | 友方護盾／淨化、正式緩速淨化、削魔，以及 Bot 友方／自用／召喚師淨化與引導道具策略。[交接及各批報告](handoff.md)。 |

### 最新集中驗收

最新完整 Rust／建置基準為 `2026-10-25-minion-cadence`：

- 九套 Rust：**2,163 passed、0 failed、9 項既有 ignored**；另有本批重點測試 4 項。
- 正式後端、UE5.8 建置、DLL staging／匯入庫一致性、兩份既有離線計畫及 catalog 核對通過。
- 本批原生回歸 **4 項通過、0 項測試警告**，並檢視新產生的實際 Slate 圖；這不是宣稱所有歷史 UE 操作測試都在本批重跑。
- **3,496 份來源一致性檢查**：3,487 未變更、9 份計畫內變更；**307 份證據獨立重核**，保留原始失敗、來源副本及程序退出／退休／清理紀錄。
- 正式版本：selective 協定 **18**、presentation 協定 **14**、Unreal bridge CABI **36**。
  - selective 18 追加（不升版）：S→C KCP tag `0x36` `MatchTerminated`（`[u64 BE last_safe_sequence][u8 len][ascii reason]`），安全終止時通知 client；舊 client 遇到未知 tag 只 warn（errors.md E050）。
  - selective 18：新增 `CommittedCharge` fact（39）；filtered replica 套用與權威端相同的施法前搖規則（`FormalCastRules`）；投射物每一步都發 `Movement` fact，flight 元件改為 v2（errors.md E037）。
  - selective 16／presentation 13：新增技能投射物，render kind 5 與 flight component `0x464f4718`（`work-G1.md`）。
  - selective 17／presentation 14：新增技能儲存次數。`Hero.ability_charges` 公開；HUD schema 5 的 `AbilityHudPresentation` 增加欄位 9–12（`work-G2.md`）。
- 本輪沒有新增真網路對局、重設 quota、commit／push，亦未維護歷史 `omfx`。

完整結果與適用範圍見 [最新驗收報告](docs/plans/2026-10-25-minion-cadence-final-acceptance.md)。錯誤與防止重犯方式見 [兵種節奏錯誤紀錄](docs/plans/2026-10-25-minion-cadence-errors.md)、[選取距離紀錄](docs/plans/2026-10-25-selected-distance-errors.md)及 [docs/plans](docs/plans/) 中各批 `errors.md`。

### 尚未完成與接續方向

| 類別 | 仍待完成或接受 |
|---|---|
| 原 MVP 11 個父項 | 真權威 ACK／對局完整流程、LAN／跨環境、實際 OS DPI／視窗模式、實體輸入與固定硬體效能等完整證據；單機合成 frame 或部分子功能通過不能代替。 |
| 戰鬥與技能內容 | NPC 普攻前搖／彈體、更多追蹤／碰撞／多段技能與召喚控制、更多特殊營地／道具／召喚師內容，以及完整戰鬥公開複本與真連線生命期接受。 |
| 地圖、美術與平衡 | 任意多邊形地圖編輯、跳牆／高低地規則、角色／建築／場景專用美術與動畫、更多目標名稱／音效、兵種及整體競技平衡。 |
| 產品流程 | 觀戰／回放產品化、完整大廳／重連／溝通往返驗收，以及長時間單 session 的固定硬體測量。 |

`complete-nameplate-screen-spacing`、公開射程、選取射程／距離及兵種攻擊節奏的 OpenSpec tasks 均已完成；未將整個遊戲或原 MVP 宣告完成。使用者要求本次收尾後，無人值守排程 `omoba-48` 已暫停。接續工作從 [handoff.md](handoff.md)、[完整度盤點](docs/plans/2026-10-21-whole-game-completeness-audit.md)及對應 OpenSpec change 開始，不重複實作已接受功能。

## 專案概觀

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

四個 `.bat` 以 `%~dp0` 定位此 clone 的 `tools\lua\lua.exe` 與對應腳本、轉送參數並回傳 exit code，不需修改 clone 絕對路徑。請從 `cmd.exe` 啟動 `.bat`。

| 情境 | 指令 | 實際腳本 |
|---|---|---|
| 第一次 clone 後拉 submodule | `git submodule update --init --recursive` | `omb`、`omfx`、`omfue`、`erps`、`specs`、`log4rs`、`map_editor`、`mqtt_log_viewer` |
| 正式 MOBA | `run_ue.bat --moba` | `scripts/run_moba_session_ue.lua` 重用正式 role workflow。預設真人選角；全本機真人可於結算準備下一局 |
| Headless MOBA | 見下方 Lua 指令 | `scripts/run_moba_headless.lua` |
| Unreal TD | `run_ue.bat` | `scripts/run_ue.lua`。預設單機 `TD_1`，環境關閉 runtime Lua；加 `--networked` 才另起 `compiled-content-only` backend |
| 雙隊 Unreal IPC | `tools\lua\lua.exe scripts\run_2player_ue.lua` | `scripts/run_2player_ue.lua` |
| 歷史 Fyrox TD | `run.bat` | `scripts/run.lua`。`runtime-lua-content`、`TD_1`、`OMB_NO_HEROES=1`。不再維護 |
| 歷史 release TD | `run_10000.bat` | `scripts/run_10000.lua`。release `TD_1` 加 `OMB_TD_STARTING_GOLD=10000` |
| 歷史雙玩家 fog | `run_2player.bat` | `scripts/run_2player_interactive.lua`。兩份 Fyrox renderer |

維護範圍：`omfue` 是唯一維護的前端。`omfx`／Fyrox 與 `run.bat`、`run_10000.bat`、`run_2player.bat` 只留作歷史參考，不再新增功能、修復相容、建置或驗收。共用 Rust API 變更不再替 `omfx` 補接線；舊檔案沒有刪除。

正式 MOBA：

```bat
run_ue.bat --moba
run_ue.bat --moba --help
run_ue.bat --moba --interactive-selection --no-build --ue-root D:/UE5.8
tools\lua\lua.exe scripts\run_moba_role_ue.lua --help
tools\lua\lua.exe scripts\run_moba_headless.lua
tools\lua\lua.exe scripts\run_moba_role_ue.lua --hero 1=training_ranger
tools\lua\lua.exe scripts\run_moba_role_ue.lua --interactive-selection
tools\lua\lua.exe scripts\run_moba_role_ue.lua --recipe target\role-ue-runs\...\session\match-plan.json
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

從 clone 根目錄執行 Lua 工具測試：`tools\lua\lua.exe scripts\test_lua_tooling.lua`。

## Toolchain

- Rust 固定為 `rust-toolchain.toml` 的 `1.95.0`。`abi_stable` 要求 host 與 `scripts/base_content.dll` 使用同一個 rustc。
- 工作流 Lua 固定為 `tools/lua/lua.exe`（Lua 5.4）。標準庫沒有的平台能力由 `tools/lua-host` 提供。不要為同一件事再加 PowerShell、Python 或 shell fallback。
- 根目錄沒有單一 Cargo workspace。常見 manifest：
  - `scripts/Cargo.toml`：`omb-script-abi`、`base_content`
  - `omb/Cargo.toml`：`omobab`
  - `omfx/Cargo.toml`：Fyrox `executor`
  - `omoba-client-runtime/Cargo.toml`
  - `omoba-core/Cargo.toml`、`omoba-sim/Cargo.toml`、`omoba-template-ids/Cargo.toml`、`omoba-content-model/Cargo.toml` 是 path dependency
- 正式 MOBA 建置加 `--features compiled-content-only`。`runtime-lua-content` 用於作者期內容驗證及歷史 Fyrox `run.lua`，兩個 feature 不能同時開。build dependency 裡的 mlua 只負責生成，不算遊戲執行期依賴。
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

`omoba-client-runtime` 的呈現幀是 4-byte big-endian length 加 protobuf。`PRESENTATION_MAGIC` 是 `0x4f4d5254`，`PRESENTATION_PROTOCOL_VERSION` 目前是 `14`，單一 frame 上限 8 MiB。常數以 `omoba-core/src/renderer_protocol.rs` 為準。`render_id` 只在單一 team replica 與 disclosure epoch 內有效，不是 Specs entity ID。

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

1. 用 KCP 連權威 server，協商 selective lockstep V2 系列的目前協定版本 `15`。
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

正式對局加 `-om-presentation-only`，不代入 KCP endpoint。`run_ue.bat --moba` 轉到正式 MOBA launcher；沒有 `--moba` 的既有 TD 路徑不加 `--networked` 時仍是 `-om-single-player`，那是單機 TD。

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

`SELECTIVE_LOCKSTEP_PROTOCOL_VERSION` 目前是 `15`，定義於 `omoba-core/src/transport.rs`；「selective V2」是協定系列名稱，並非目前數字版本。未知／舊版本不得混入目前 session。`proto/game.proto` 同時保留 typed payload 與 JSON fallback。熱路徑優先用 typed payload 與 quantized scalar。

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
tools\lua\lua.exe scripts\build_ue_moba.lua --ue-root D:/UE5.8 --build-only
tools\lua\lua.exe scripts\test_compiled_content_contract.lua
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

## 歷史 Fyrox Frame Pacing（僅留存）

這一節只解釋還留在 repo 裡的 Fyrox 前端。不要為了現行 MOBA 去改這些檔。

`omfx` 使用 Fyrox `1.0.1`。`omfx/Cargo.toml` 用 `[patch.crates-io]` 把 `fyrox-impl` 指到 `third_party/fyrox-impl-1.0.1`。frame pacing 已經在這份 vendor source 裡，不要再去改 Cargo registry 的 `fyrox-impl`。

`third_party/fyrox-impl-1.0.1/src/engine/executor.rs` 的 `Event::AboutToWait` 會看離下一個 fixed step 還有多久：

- 剩餘時間大於 2 ms 時，最多 `sleep` 1 ms，並留下 2 ms spin window。
- 進入 spin window 後用 `spin_loop`，在剩餘約 250 µs 時返回。
- 這樣是為了在 Windows 上靠近 120 Hz，而不是每 frame 固定睡 1 ms。

`fyrox-graphics-gl` 仍來自 crates.io，沒有 vendor。Upstream `1.0.1` 的 `vsync: false` 在 Windows 是 no-op：`server.rs` 只在 `vsync=true` 時 `set_swap_interval(Wait(1))`，`false` 什麼都不設，DWM 常把視窗鎖在 60 Hz。若本機仍被鎖 60 Hz，要改 registry 裡的 `fyrox-graphics-gl-1.0.1/src/server.rs`，在 `else` 分支呼叫 `SwapInterval::DontWait`。這份修改不進 git；`cargo clean`、`cargo update` 或清 registry 之後要重做，並重編 `fyrox-graphics-gl`。

`omobab`、`omfx/executor` 與 `omoba-client-runtime` 在 Windows 上都會呼叫 `timeBeginPeriod(1)`。沒有它時，`sleep(1ms)` 在乾淨的 Windows 上可能實際睡約 15 ms。這只把 timer granularity 降到 1 ms，不代表 sleep 本身精準到 1 ms。
