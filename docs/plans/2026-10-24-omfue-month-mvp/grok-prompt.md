# 交給 Grok 的執行指令

以下區塊可直接作為工作開始訊息。這份文件不假設特定 Grok 版本具備自動排程、720 小時連續執行或任何未驗證的工具能力；中斷後按狀態檔續跑即可。

---

請在 `D:\omoba` 執行「omfue 描述驅動英雄製作與可玩 MOBA MVP」一個月計畫。

先閱讀：

1. `AGENTS.md`、根目錄 `README.md`（目前進度、正式入口、建置與內容管線）與 `handoff.md`。
2. `docs/plans/2026-10-24-omfue-month-mvp/README.md`（尤其是 §0 與 §0.1）。
3. 同目錄的 `audit.md`、`schedule.md`、`test-topology.md`。
4. 若已存在，同目錄 `execution-status.md` 與最後一批驗收紀錄。

我的最終需求是：我提供 `https://github.com/damody/nobugg_official` 的英雄技能描述，或直接貼一份新英雄描述，你就能生成所需 Rust script 與資料，把完整英雄接進 **omfue**。我最後只進 Unreal Editor 補模型、材質、動畫、圖示、音效與技能特效，不需要手寫 gameplay Blueprint、C++ 或 Rust。

**最重要的一句：盡快把遊戲做好。** 每天的成果是遊戲本身的進步，不是測試數量、驗收文件或開了幾局；測試只做到防止做好的東西壞掉為止（詳見 README §0）。

請直接實作，不只回覆建議。第一次從 schedule.md 的 A1 baseline 開始；後續從 schedule.md 中最早未完成且前置已滿足的工作項繼續（計畫不排日曆），不要每次重新盤點整個 repository，也不要把每天重跑舊驗收當進度。

## 範圍與優先序

- 唯一前端是 omfue。omfx 未來會移除，完全不維護、不編譯、不做相容接線、不納入驗收；本任務不刪它的檔案。正式 UE 工具目前對它的資產依賴需要解除。
- 後端／共用 Rust／script／Lua 只做支撐 omfue 的必要改動。不要另起框架，不重做已接受功能。
- 第一優先是同一真實對局從選角到自然勝負與再開一局，其次是可重複的英雄生成及 Editor 美術流程，最後才是額外內容數量。
- 本月目標：完整來源 kit 最低 4 套、目標 6 套，加 2 位保留盲測英雄；全來源索引與能力分類。配裝變體不算獨立來源英雄，不能因缺技能就把角色改名算完成。
- 每個里程碑（schedule.md §2 的 M1–M4）交付可展示的整合成果。M4 之後凍結新功能，只修發佈阻塞與完成交接。
- 第二台電腦到位前，多人驗收在單機以 2 個 omfue client＋3 個 Rust 程序（1 omobab＋2 omoba-client-runtime）進行，沿用 `scripts/run_moba_role_ue.lua` 的 `--server-only`／`--connect` 模式，不另寫 launcher。人類席由 omfue client 端的規則式 autopilot 驅動（test-topology.md §4）；它只讀玩家可見資料、經真輸入管線送出操作，不得直接呼叫 server 或沿用權威端 bot。

## 工作方式

1. 先讀 git status、現在的 specs／來源與最近接受，保護既有 dirty work；不要 reset、clean、整批 commit 或終止無關程序。
2. 建立／更新 `execution-status.md`，選一個小而完整的垂直工作。以玩家／作者能做什麼描述目標，不以測試數為目標。
3. 修改前確認相關現有 OpenSpec change；可沿用則更新它。新能力按本 repo 既有提案／實作方式補規格，不重建平行文件系統。
4. 沿現有 `AbilityScript`／`UnitScript`、Lua metadata、`gen_hero_registry.lua`、template IDs、omfue codegen／native actor／asset recipe 實作。必要新 API 要同時接正式 host、replica 與 UE 消費端。
5. 使用根目錄 `tools\lua\lua.exe` 執行所有工作流。Rust 固定 1.95.0；host／script 同 rustc。不得新增 PowerShell／Python／shell fallback 或額外根目錄 bat。
6. 每次只跑必要測試，整合日再跑較廣驗收。測試通過後繼續交付，不無限重跑相同套件。
7. 同一失敗連續三次，寫出根因與改變策略；保留原失敗，不刪樣本、不降低斷言。
8. 只有真正阻擋工作且無法從現有環境推得的問題才集中詢問。普通命名、佔位、實作選擇自行合理決定並記錄；未知技能語意或外部硬體條件不能偽造。
9. 不需要新建聊天、排程或子代理即可執行；這份交接不要求多代理。不要擅自啟動外部模型 API 付費任務或宣稱已跑滿一個月。

## 英雄生成要求

- 固定來源 commit、source hero／ability ID、variant、原文、數值來源與 assumptions。B02 的來源身分與本地 `B02_date_masamune` 不同，禁止按編號猜身分。
- 正規化目標、幾何、時序、資源、rank、效果順序、事件、死亡／取消／視野／重連行為。敘述沒寫清楚時記假設；必要機制未支援就報 `needs_framework`，不能靜默做成普通傷害。
- 原 W/E/R/T/D/F、被動與 `_old` 不可直接當固定 QWER。額外技能要有通用 action 接口或明確阻塞；來源 D/F 不等於本遊戲召喚師技能。
- 一般效果重用通用資料；新機制可生成 Rust handler，但不能每個英雄新增一堆 UE hero-name 特判。
- 先由來源文字列預期行為，再驗生成程式。自訂 handler 必須有實際效果；空的 hook／只生成名稱、圖示不算英雄。
- 正式產物須包含：玩法、註冊、選角、輸入、tooltip、預覽、可見性安全的 cues、佔位模型／圖示、bot 使用提示、測試與製作清單。
- 編譯或驗證失敗不得覆蓋上一個可用版本。新增英雄後內容 ID／catalog／DLL 與前端一致；人工素材不因重新生成被覆蓋。

## Unreal 美術與玩法分離

- 使用者可在 Editor 指派所有外觀槽位，儲存、重開、重新生成、cook 後仍保留。
- 優先序固定為人工外觀 > 生成預設 > 通用佔位。外觀 profile／手做 Blueprint 與生成輸出分開管理。
- 骨架、socket、材質、動畫、VFX、音效與圖示要能驗證；缺美術有 fallback，缺必要玩法不能通過。
- 傷害與命中由 Rust 權威時序決定，不由 AnimNotify、Niagara 或畫面碰撞決定；換模型不能改 gameplay 半徑／速度。
- 以兩種不同模型／骨架證明 Editor-only 流程。不能只在一個作者預先配好的 hero 上演示成功。

## 驗收紀律（里程碑時整理；平日從輕，以 README §0 為準）

- L1 程式測試、L2 合成 UI／native renderer、L3 真實 runtime 對局（L3a autopilot／L3h 使用者試玩）、L4 真 LAN／DPI／效能／打包分開。單機雙 client 標 L3a-loopback，不記成 LAN。
- `submit sent` 不等於權威接受；看見投射物不等於命中；cargo 成功不等於英雄可玩；測試 replay 不等於使用者回放產品。
- 原 checklist 33/44 與 11 未完成主項要逐條對照實證。它們不是整個遊戲完成百分比。
- 使用者 2026-10-25 授權：本機對局不限局數，舊 `10/10` 對局預算不再適用於本計畫。需要真對局證據就直接開局；工具裡若有寫死的局數上限，可改成可設定並記錄。不能用 synthetic 冒稱補完真局。
- 新證據註明 build identity、來源／資產、命令、時間、exit code；沿用結果清楚標沿用。不要把其他工作階段的圖寫成當次畫面。
- 真雙機、OS DPI、固定硬體或兩小時 session 沒做就是未做。沒有第二台機器不能用兩個 process 取代 LAN 驗收。
- 遊玩失敗、生成覆蓋人工素材、內容版本不一致、未知敵人洩漏、重複扣費／傷害都屬 release blocker。

## 每次結束前

更新 `execution-status.md`：完成的功能、修改檔、驗證與未驗證內容、錯誤、gate 狀態、剩餘對局／資源條件、下一個具體 task。確保任何新執行者都能繼續。

本月結束提供可玩包、玩家手冊、英雄生成手冊、Unreal 美術手冊、來源／能力覆蓋表、G0–G6 驗收表與剩餘 backlog。必需 gate 沒過就交候選版並說清楚，不改定義宣稱完整成功。

---

## 後續續跑訊息

```text
繼續執行 D:\omoba\docs\plans\2026-10-24-omfue-month-mvp 的計畫。
先讀 AGENTS.md、execution-status.md 與最後一批驗收。
從最早未完成且相依滿足的任務繼續實作，重點仍是 omfue 可玩與描述生成英雄。
不要重做已接受內容，不碰 omfx，不把文件／測試數當產品進度。
完成本批必要驗證後更新交接，繼續下一個可做任務。
```

## 起手可用的現有入口

下列是已查到的入口，不代表本次已執行通過。執行前遵守當前資源／session／Editor owner 限制。

```bat
run_ue.bat --moba --help
tools\lua\lua.exe scripts\run_moba_role_ue.lua --help
tools\lua\lua.exe scripts\run_moba_pie_ue.lua --help
tools\lua\lua.exe scripts\build_ue_moba.lua --ue-root D:/UE5.8 --build-only
tools\lua\lua.exe scripts\test_compiled_content_contract.lua
tools\lua\lua.exe scripts\test_lua_tooling.lua
cargo test --manifest-path scripts\Cargo.toml -p omb-script-abi
cargo test --manifest-path scripts\Cargo.toml -p base_content
cargo test --manifest-path omb\Cargo.toml -p omobab
cargo test --manifest-path omoba-core\Cargo.toml
cargo test --manifest-path omoba-client-runtime\Cargo.toml
cargo test --manifest-path omoba-template-ids\Cargo.toml
cargo test --manifest-path omfue\codegen\Cargo.toml
```

正式建置一律加 `compiled-content-only`，不要拿 `runtime-lua-content` 編正式 MOBA。`scripts/build_ue_moba.lua` 會把 DLL stage 到 repo 的 `scripts/base_content.dll`，不要複製到 `omb/scripts/`。

`run_ue.bat --moba` 會進入實際 MOBA 流程，不是只顯示 help。不要在還沒確認現有程序與 build 狀態時拿它當無副作用檢查。其他建置／stage／驗收命令先查既有 Lua 工具的 help 與 spec，不猜參數。
