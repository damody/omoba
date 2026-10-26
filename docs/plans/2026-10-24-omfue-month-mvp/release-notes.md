# 發佈說明（草稿，2026-10-26 更新）

本文件只列出**有實機或測試證據**的內容；證據路徑寫在每一項後面。沒有證據的內容一律放在「未完成／限制」，不寫成已完成。gate 定義見 [README.md §8](README.md)，詳細紀錄見 [execution-status.md](execution-status.md) 與 [errors.md](errors.md)。

證據層級：L1 為 Rust／Lua 測試；L2 為 UE Editor 測試；L3a 為 omfue autopilot 經真輸入管線的實機對局（本機 B＋C＋C，`L3a-loopback`）；L3h 為使用者親自試玩；L4 為雙機 LAN、OS DPI、固定硬體或打包版。

## 1. 可玩流程

| 內容 | 層級 | 證據 |
|---|---|---|
| 同機 B＋C＋C：1 個權威＋每位人類 1 個 client runtime＋1 個 UE；兩個人類席（p1 隊伍 1、p6 隊伍 2）＋8 bot | L3a | `scripts/run_moba_split_local.lua`；execution-status 2026-10-25（A3、B1／B2） |
| 標準三路地圖，對局自然結束並顯示結算面板（36:15，隊伍 2 勝） | L3a | split19：`target/mvp-a3/split19/p1/autopilot-p1-shot-037-result.png`、`target/mvp-a3/split19/server/server.stdout.log` |
| quick 測試地圖（`three_lane_quick_training`）在 10 分鐘內完成推塔、基地、結束與結算 | L3a | split29（7:00 自然結束，0 mismatch）：`target/mvp-a3/split29-quick/`；split30：`target/mvp-a3/split30-quick/` |
| 真輸入 → ACK → 權威套用：移動、普攻、Stop、Hold、A 移動、佇列、Ctrl＋右鍵只移動、Q／E 指定目標、R 立即、升級 R | L3a | split7：`target/mvp-a3/split7/server/server.stdout.log`（UpgradeAbility 6／6、CastAbility 18／18） |
| 商店購買 | L3a | split8 截圖（execution-status 2026-10-25 H02 安裝） |
| 比賽終止時顯示「比賽已終止」面板並可離開對局 | L3a | split21（execution-status 2026-10-25 split20／21）；[work-match-terminated.md](work-match-terminated.md) |
| 解析度參數 `--resolution WxH`，1920x1080 下 HUD 沒有裁切或重疊 | L3a | split32：`target/mvp-a3/split32-1080p/` |

## 2. desync／parity 修正

權威端與兩隊 filtered replica 必須逐 tick 一致；不一致會觸發 rebase，rebase 期間再次不一致則終止比賽。

| 修正 | 證據 |
|---|---|
| 技能投射物、位移、施法前搖、charges 的 parity（E037） | errors.md E037；`single_lane_match_tests::scripted_motion_replica_parity`、`windup_dual_replica` |
| 同 tick 隱藏效果翻轉施法准入、指令指定隱藏目標（E050）；mismatch 時記錄不一致的元件；終止時通知 client | errors.md E050；`live_soak_replica_parity` |
| 引導技能在 replica 端的 parity（E053） | errors.md E053；split20 引導 start／pulse 兩端一致 |
| 回城只在權威端取消前搖、目標在同一 tick 離開視野（E056）；mismatch log 帶出 canonical、kind 與元件內容 | errors.md E056；10 人 bot soak 130000 tick；split27 起實機 0 mismatch |
| 小兵與野怪的面向（E057） | errors.md E057；split28 creep_facing_probe |
| Windows 上 KCP 10054 造成全體連線停頓（E052） | errors.md E052；split19 late input 0 |

實機結果：split27、28、29、30、31、32 連續 6 局 0 mismatch、0 terminated（execution-status 2026-10-25「測試局 ≤10 分鐘」）。

## 3. 輸入與 HUD

| 內容 | 層級 | 證據 |
|---|---|---|
| 被權威拒絕的施放／升級回傳 `accepted=false` 與原因碼，HUD 顯示「施法指令遭拒絕：冷卻中」等 | L1／L2 | [work-ack-result.md](work-ack-result.md)、[work-reject-reason.md](work-reject-reason.md)；Editor 測試 RejectReasonText |
| W／E／R 送錯欄位（舊 TD 藍圖輸入圖）修正 | L3a | errors.md E051；split16 OM_CAST slot=3 |
| UE 開發版 F1～F5 debug 鍵與遊戲熱鍵衝突修正 | L3a | errors.md E035 |
| owner ward 投影不再被判為 NONCANONICAL | L3a | [work-ward.md](work-ward.md)；split3～7 runtime log 0 筆 |
| 連線中斷時顯示「連線中斷，等待恢復」並暫停操作（client 2 秒無進度發 STALLED，恢復發 READY；30 秒判定斷線） | L1／L2 | errors.md E060；Editor 測試 MatchTerminationRefusesInput；實機見 §5 |

## 4. 打包

| 內容 | 層級 | 證據 |
|---|---|---|
| Windows Development 打包（`scripts/package_ue_moba.lua`），每個檔案記錄 SHA-256 | L4 | `target/ue-package/a5-first/package-report.json` |
| 兩個打包版 OmGame.exe 從獨立目錄連 dedicated server 對戰，地面、英雄模型正常 | L4 | split23：`target/mvp-a3/split23-packaged/`；errors.md E054（cook 範圍） |
| 用最新版本（build-47，含技能圖示 E091、英雄頭像 E092、共用人形 E093）重新打包 package-dev-5，以打包版跑 oichi 對 murakami 到自然結束：0 mismatch，截圖可見頭像、Q～X 技能圖示與人形英雄 | L4 | `target/mvp-a5/package-dev-5/`、`target/mvp-a5/split-dev-5-smoke/`（`p1/autopilot-p1-shot-009-facing.png`、`shot-010-result.png`） |
| 用 build-44（含 E077～E088）重新打包，以打包版跑 autopilot 對局到自然結束 | L4 | `target/mvp-a5/package-dev-4/`（50 個檔案）；冒煙局 `target/mvp-a5/split-dev-4-smoke-2/`：約 4.5 分鐘自然結束、0 mismatch、有結算截圖。`--packaged-game` 現在也接受 Windows 資料夾 |
| Shipping 打包（`--config Shipping`）：autopilot 與各種 smoke 參數都不會編入；連線所需的 `-om-*` 參數照常讀取。**Shipping 不輸出 UE log（未開 bUseLoggingInShipping），診斷請用 Development 版**；冒煙測試以 client-runtime 與 server 的 log 判定 | L4 | `target/mvp-a5/package-shipping-4/package-report.json`：28 個檔案、713 MB，沒有 pdb；exe 不含 autopilot 與 smoke／screenshot 開關（errors.md E064、E065）。冒煙測試 `target/mvp-a5/split-shipping-smoke-1`：兩個 Shipping client 進場（Humans admitted）並撐滿 120 秒，沒有 crash；client-runtime ready；match 有在進行；0 mismatch、0 terminated |

## 5. 網路／netem

| 內容 | 層級 | 證據 |
|---|---|---|
| 每隊各自經 netem proxy，80 ms ±20 ms RTT、1% 掉包，對局自然結束，0 mismatch、0 late input | L3a | split31：`target/mvp-a3/split31-netem/` |
| proxy 排程單隊斷線（`--outage`／`--netem-outage`），雙向全部丟棄，evidence 記錄實際起訖與丟包數；優雅停止會寫出統計 | L1 | errors.md E060；`omoba-netem-proxy` 測試 21 項；本機程序冒煙 |
| 斷線存活判定：client STALLED／READY／30 秒 SERVER_DISCONNECTED；server 30 秒關閉沉默 session；斷線期間的輸入以 INPUT_EXPIRED 回報 | L1 | errors.md E060；`link_liveness` 測試；判定工具 `scripts/check_netem_outage.lua` |
| 斷線恢復實機局：兩次 10 秒斷線都是 stalled→recovered；40 秒斷線時 p6 與 server 各明確判定 link lost 一次；0 mismatch、0 terminated | L3a | split35（build-34）：`target/mvp-a3/split35-outage/`；`scripts/check_netem_outage.lua` 11/11 PASS（參數 `--netem wan-80-20-loss1 --netem-seed 7 --netem-outage 1:90:10 --netem-outage 1:240:10 --netem-outage 2:420:40`） |
| 斷線 30 秒以上後重新加入同一場：server 保留英雄，client 用 resume token 重新加入（兩段式輪替），舊 KCP socket 會關閉；proxy 依 client endpoint 各配一個 upstream（NAT 語意） | L3a | split44（build-38）：`target/mvp-a3/split44-rejoin/`，`check_netem_outage` 8/8 PASS；team 2 在 +120s 斷線 45s → link lost 1 次 → rejoin 1 次，之後串流持續，0 mismatch、0 terminated。修正前 split41 失敗（errors.md E077） |

## 5a. 效能（J5）

| 內容 | 層級 | 證據 |
|---|---|---|
| 伺服器 tick 平均 13.21→6.18 ms、p99 視窗 35.37→12.41 ms、tick rate 55.1→61.3 Hz；UE 59.5 FPS、hitch 視窗 0；hero_command 16.8→4.50 ms（errors.md E073 BFS 地形快取、E074 不可達移動） | L3a | 基準 split37 對 after split43（build-37，quick map，2 autopilot＋8 bot，比賽自然結束）：`tools/lua/lua.exe scripts/summarize_moba_perf.lua --run target/mvp-a3/split37-perf-baseline --run target/mvp-a3/split43-perf-after`，after 判定 PASS；[perf-report.md](perf-report.md) §6 |

## 6. 英雄

| 英雄 | 狀態 | 證據 |
|---|---|---|
| H01 雜賀孫市（來源 B01 current）：Q 狙擊增益、W 儲存制定點雜賀眾、E 依目標類別加成（被動）、R 三段 +200% | kit 完整；實機 Q／W／R 有施放 | [work-H.md](work-H.md) H01；split17（W 召喚雜賀眾實機施放）；base_content 測試 |
| H02 伊達政宗（來源 B04 current）：Q 踏火無痕、W 火焰強襲、E 炎之迅捷、R 種子島 | kit 完整（第一套 supported）；split9 實機 Q 被權威接受，但當時觸發 desync（E037 已修），修正後尚未重跑實機 | work-H.md H02；split9 server log |
| H03～H06（阿松、阿市、稻姬、淺井長政） | 實機逐招驗收通過：split49 阿市 5/5、阿松 4/4；split46 稻姬 4/4、淺井長政 4/4（第 5 招 16 級自動學會，10 分鐘內到不了）；0 mismatch。途中修了 autopilot 升級／unit 目標（E080）、範圍技能觸及距離（E082） | work-H.md H03～H06；[live-acceptance-h01-h06.md](live-acceptance-h01-h06.md)；`scripts/check_hero_live_acceptance.lua` |
| I1 玉子（C15）、I2 村上義清（B29）：盲測英雄，由無上下文的 agent 只依 hero-authoring-guide 製作 | 正式 `hero_author build --execute` 成功；split53 實機逐招 4/4、4/4，0 mismatch。途中修了玉子 R 的結算（E085）、replica 英雄 HP 套用順序（E087，用 E086 的可重播輸入紀錄離線重現） | execution-status 2026-10-26；errors.md E075、E076 |

能力（作者只寫 Lua／package 即可使用）：技能投射物（直線／追蹤，[work-G1.md](work-G1.md)）、儲存次數 charges 與定點召喚物（[work-G2.md](work-G2.md)）、引導技能（[work-G2-channel.md](work-G2-channel.md)）、攻擊法球（每擊扣魔）。英雄製作工具 `scripts/hero_author.lua`（validate／build／回滾保護）。來源全目錄索引：97 個主英雄檔（[work-D.md](work-D.md)）。

## 7. Gate 狀態（G0～G6）

| Gate | 狀態 | 說明 |
|---|---|---|
| G0 基準 | 進行中 | A1、A2、A3、A4、A5 完成；A6（需使用者確認的人工／硬體條件）未做 |
| G1 英雄薄切片 | 達成 | H01 從來源描述 → package → hero_author → 選角 → UE 施法，split50 逐招 4/4（見 [acceptance-index.md](acceptance-index.md)） |
| G2 可玩基線 | 達成（L3h 除外） | 自然結束（split19、29、30）；1 人類席＋9 bot（split55-single）；重開一局（rematch-1，check_managed_rematch 26/26）。L3h（使用者試玩）只能由使用者進行 |
| G3 製作能力 | 達成 | H01～H06 共 6 套完整 kit 實機逐招通過（split46／49／50／51） |
| G4 美術交付 | 部分 | 工具、[unreal-art-guide.md](unreal-art-guide.md) 與 Editor 換模型的自動驗收都已完成；6 位新英雄的正式模型要人工在 Editor 指定 |
| G5 泛化能力 | 達成 | I1 玉子、I2 村上義清由盲測 agent 製作，正式 build 並實機逐招 8/8 通過（split53）；guide 回饋 18 條已修正 |
| G6 發佈 | 未達 | 已有：打包版對戰（split23）、netem（split31）、斷線判定（L1）。缺：真雙機 LAN、OS DPI 矩陣、2 小時 soak 改以多局 10 分鐘彙總代替（J5 split43 PASS）。斷線恢復（split35）與重新加入（split44）實機已通過 |

## 8. 未完成／限制

- **J2 真雙機 LAN**：未執行，需要第二台機器。Loopback 不能代替 LAN。
- **OS DPI 100%／150%**：屬於系統設定，不得自行修改，留給使用者手動切換後驗收。split32 的截圖實際只有 1423x889，原因可能是 DPI 縮放或視窗被限制在工作區內。
- **J5 兩小時 soak 與「實機局 ≤10 分鐘」規則衝突**：README §8 要求同一 session 連續 2 小時，使用者規定實機局不得超過 10 分鐘，需要使用者決定如何取捨。目前最長的實機局是 split19 的 36 分鐘與 split13 的 42.5 分鐘。
- **KCP 退避**：恢復連線後會再凍結數秒；split33 實測 10 秒斷線後分別花了 14.5 秒與 11.25 秒才恢復（kcp nodelay 每次 RTO ×1.5、無上限）。
- **英雄能力缺口**：host 冷卻調整 API（H03）、範圍內友軍查詢與持續連結（H04）、英雄隱形（H05）、英雄 UnitScript 註冊入口（H06）、第 5 技能欄 extra_action_slot（H04、H06）。
- **H02 修正後未重跑實機**；`retained_abilities` 是否會被 UE 當成技能欄顯示，尚未確認。
- **autopilot 打法偏弱**：split19 的 p1 死亡 44 次；只影響驗收效率，不影響規則。
- **打包版剩餘警告**：profiler DLL 與引擎 UnlitText 載入失敗（名牌照常顯示）；Shipping 打包與 2 分鐘冒煙都已通過（package-shipping-4、split-shipping-smoke-1）；沒有 autopilot，所以只驗證到啟動、連線與對局進行，沒有驗證操作流程。
- **效能（J5，見 [perf-report.md](perf-report.md)）**：使用者規定實機局不得超過 10 分鐘，所以兩小時 soak 改用「多局 10 分鐘以內的資料彙總」，另以 split19 的 36 分鐘局作為長局趨勢參考；這無法證明「同一 session 連續 2 小時」的資源成長目標。優化後（split43）伺服器 tick 平均 6.18 ms、p99 視窗 12.41 ms，符合門檻（8 ms／16.7 ms），UE 59.5 FPS，PASS（見 §5a）；剩下的熱點是 dispatcher（48%）與 team_projection（26%）。單一 client 1080p 的固定硬體量測仍未執行。
