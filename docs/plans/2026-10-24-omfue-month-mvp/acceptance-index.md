# 驗收索引

各 gate 的證據路徑、build identity，以及 fresh／沿用／未執行狀態。gate 定義見 README §8，只在里程碑（schedule.md §2）判定。

| Gate | 狀態 | 證據 | build identity | 備註 |
|---|---|---|---|---|
| G0 基準 | 進行中 | [execution-status.md](execution-status.md) 2026-10-25 | 主 repo b1f00b9、omb 7f26187、omfue d311ad4 | A3、A5 已完成（A5：`target/ue-package/a5-first/package-report.json`）；待 A6 |
| G1 英雄薄切片 | 達成 | split50（H01 雜賀孫市 4/4 招實機施放，`check_hero_live_acceptance`）、split51（E083 後 0 mismatch） | build-40／41 | 從來源描述 → package → hero_author → 選角 → UE 施法 |
| G2 可玩基線 | 達成（L3h 除外） | split19、29、30（自然結束）；split55-single（1 人類席＋9 bot）；rematch-1（重開一局，check_managed_rematch 26/26） | build-45 | L3h（使用者親自試玩）只能由使用者進行 |
| G3 製作能力 | 達成 | split46（H05／H06）、split49（H03／H04）、split50／51（H01／H02），全部由 `scripts/check_hero_live_acceptance.lua` 逐招判定 | build-39～41 | 6 套完整 kit 實機逐招通過（最低 4 套） |
| G4 美術交付 | 部分 | [unreal-art-guide.md](unreal-art-guide.md)（含 §0 新英雄換模型最短步驟）；UE 測試 ArtSwapTwoRigs、PublicHeroCombatRoster、ItemIconResources | build-44 | 工具、指南與 Editor 換模型的自動驗收都已完成；6 位新英雄（H03～H06、I1、I2）的正式模型要人工在 Editor 指定，目前是 fallback 外觀 |
| G5 泛化能力 | 達成 | split53（I1 玉子 4/4、I2 村上義清 4/4，0 mismatch）；blind/ 盲測紀錄 | build-43 | 無上下文 agent 只依 hero-authoring-guide 製作；guide 回饋 18 條已修正 |
| G6 發佈 | 部分 | split44（重連）、split43（效能 PASS）、package-dev-4＋split-dev-4-smoke-2（打包版自然結束）、package-shipping-4（Shipping）、split31／35（netem 與斷線） | build-38～45 | 缺真雙機 LAN、OS DPI 150%（都需要人工）；2 小時 soak 依使用者規定改為多局 ≤10 分鐘 |

## 近期實機證據（L3a-loopback，B＋C＋C，p1 隊伍 1、p6 隊伍 2、8 bot）

| run | 內容 | 結果 | 證據 |
|---|---|---|---|
| split19（build-23） | 標準地圖 M1，p6 雜賀孫市 | 36:15 自然結束（隊伍 2 勝）；mismatch 3 次皆 rebase 恢復；late input 0 | `target/mvp-a3/split19/p1/autopilot-p1-shot-037-result.png`、`target/mvp-a3/split19/server/server.stdout.log` |
| split23 | J6 打包版，兩個 OmGame.exe 對 dedicated server | 可連線開打，草地與英雄模型正常 | `target/mvp-a3/split23-packaged/` |
| split27 | quick 地圖，E056 修正後 | 跑滿 10 分鐘，mismatch 0、terminated 0、late input 0 | `target/mvp-a3/split27-quick/` |
| split29（build-31） | quick 地圖，每路一座塔 | 7:00 自然結束（隊伍 1 勝），mismatch 0 | `target/mvp-a3/split29-quick/`（`p1/autopilot-p1-shot-007-result.png`） |
| split30 | quick 地圖，p1 training_channeler | 約 10 分鐘內自然結束；引導 completed 5；mismatch 0 | `target/mvp-a3/split30-quick/` |
| split31 | J1 netem wan-80-20-loss1（seed 7） | 約 3.6 分鐘自然結束；mismatch 0、terminated 0、late input 0 | `target/mvp-a3/split31-netem/`（`netem/`） |
| split32 | J4 1920x1080 | 約 5 分鐘自然結束；mismatch 0；HUD 沒有裁切或重疊 | `target/mvp-a3/split32-1080p/`（`p1/autopilot-p1-shot-006-result.png`） |
| split33 | J1 斷線（1:90:10、1:240:10、2:420:40），build-32 | 10/11：一筆召喚物 ATTACK mismatch（rebase 恢復，促成診斷強化） | `target/mvp-a3/split33-outage/` |
| split35（build-34） | J1 斷線，同參數 | `check_netem_outage` 11/11 PASS；0 mismatch、0 terminated；stalled→recovered ×2；p6 與 server 各判定 link lost 一次 | `target/mvp-a3/split35-outage/` |
| split-shipping-smoke-1 | Shipping 打包版（package-shipping-4）冒煙，quick 地圖，沒有 autopilot，120 秒 | 兩個 Shipping client 進場並撐滿 time box，沒有 crash；client-runtime ready；match 有在進行；0 mismatch、0 terminated | `target/mvp-a5/split-shipping-smoke-1/`、`target/mvp-a5/package-shipping-4/package-report.json` |
| J5 效能彙總 | split19、27～32、35、36 的 OM_PERF（不另開局） | UE 平均 59 FPS PASS；伺服器 tick 平均 9～12.5 ms、p99 視窗 22～27 ms FAIL；split35／36 被建置污染，不採計 | [perf-report.md](perf-report.md)、`target/mvp-a5/perf-summary.md`、`scripts/summarize_moba_perf.lua` |

## 沿用的既有證據

| 來源 | 內容 | 適用範圍 |
|---|---|---|
| [2026-10-25-minion-cadence 最終驗收](../2026-10-25-minion-cadence-final-acceptance.md) | 九套 Rust 共 2,163 passed／0 failed／9 ignored；UE5.8 建置、DLL staging | L1 基準；不含 b1f00b9 之前後續 commit 的重跑 |

## 原 11 主項

| 原 ID | 狀態 | 證據 |
|---|---|---|
| 3.1、3.3、3.4、3.5、3.9、4.3、4.4、4.7、7.1、7.3、7.4 | 未關閉 | — |

## 最終完整驗收（2026-10-26 11:00，build-45）

| 項目 | 結果 | 證據 |
|---|---|---|
| Rust：`cargo test -p omobab` | 196 passed、0 failed（另有子套件全部通過） | `target/mvp-a3/final-rust.log` |
| Rust：`cargo test -p omb-script-abi` | 15 passed、0 failed | 同上 |
| Rust：`cargo test -p base_content`（debug，包含所有 parity soak 與完整三路對局） | 538 passed、0 failed（2217 秒） | 同上 |
| UE：全量 `Om.`，`-d3d12 -NoSound` | 287/287 Success、EXIT 0 | `target/mvp-a3/ue-tests-final-rhi.log` |
| UE：全量 `Om.`，`-nullrhi`（build-44） | 287/287 Success | ward 回報（E088） |
| 8 位 authored 英雄實機逐招 | 全部通過，0 mismatch | split46、49、50＋51、53 |
| 斷線重連／效能／打包 | PASS | split44、split43、split-dev-4-smoke-2 |
