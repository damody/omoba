# 效能報告（J5，2026-10-26）

資料全部取自既有的 split 證據，沒有另外開局。工具：`tools/lua/lua.exe scripts/summarize_moba_perf.lua --run <dir>...`（fixture 測試：`scripts/tests/summarize_moba_perf_test.lua`），完整表格存在 `target/mvp-a5/perf-summary.md`。

## 1. 量測環境與限制

- **拓撲**：同一台機器（24 個邏輯核心）跑 1 個權威伺服器、2 個 client runtime、2 個 UE（B＋C＋C，L3a-loopback）。伺服器的 ECS worker 是 3 條 thread（`OM_ECS_WORKERS threads=3`，host 預算 `--host-simulations 3`）。兩個 UE 與伺服器共用 CPU，所以數據代表「單機雙 client 的壓力情境」，不是 README §8 所定義的「固定硬體、單一 client 1080p」。
- **兩小時 soak 的取捨**：使用者規定實機局不得超過 10 分鐘，因此 J5 的兩小時 soak 改用「多局 10 分鐘以內的資料彙總」代替。另外加上 split19（36 分鐘、build-23）作為長局的時間趨勢參考。這個做法無法證明「同一 session 連續 2 小時」的資源成長目標，只能證明在每局 10 分鐘內，以及單一 36 分鐘局中，沒有隨時間惡化。
- **資料格式**：OM_PERF 每筆是 60 個樣本的視窗（count、sum、max）。平均值以 sum/count 計算；p99 是對「視窗平均」取 nearest-rank，不是逐樣本的 p99；max 是所有視窗中最大的單一樣本。
- **污染的局**：split36 從頭到尾都與 consistent-build-35 的 UE 編譯同時進行（由 00:47 的 omfue 著陸觸發，編譯持續到 01:13）；split35 的第一分鐘則與 package-shipping-3 重疊。這兩局的伺服器數據不能和其他局比較（見 errors.md E068）。

## 2. 門檻（A3 沒有寫明，以下為本報告提出的值）

| 項目 | 門檻 | 理由 |
|---|---|---|
| 伺服器 tick compute 平均 | ≤ 8 ms | 60 Hz 預算 16.7 ms 的一半，保留尖峰與追趕的餘裕 |
| 伺服器 tick 的 p99 視窗平均 | ≤ 16.7 ms | 任何一秒內的平均都不應超出 60 Hz 預算 |
| 伺服器實際 tick rate | ≥ 57 Hz | 達到 60 Hz 目標的 95% |
| UE 平均 FPS（1280x720） | ≥ 57 | client 有 60 FPS 上限，平均 59.x 代表貼著上限；取上限的 95% |
| UE 卡頓 | 暖機 60 秒後，含 >100 ms frame 的視窗 ≤ 2% | 一開始的 400 ms 是載入與 shader 暖機時的 delta 上限，不是遊戲中的卡頓 |
| 網路 | late input 被拒 0、mismatch 0、terminated 0 | 正確性門檻，與效能一起檢查 |

## 3. 結果

| run | 伺服器 tick 平均／p99 視窗／max (ms) | tick rate (Hz) | UE fps (p1／p6) | UE 卡頓視窗 (p1／p6) | replica step 平均 (ms) | late 被拒／retarget | mismatch／terminated | 判定 |
|---|---|---|---|---|---|---|---|---|
| split19（36 分鐘） | 12.16／31.52／191.5 | 57.7 | 59.2／59.0 | 28/2386／27/2379 | 4.11／4.77 | 0／2 | 3／0 | FAIL：伺服器 tick、mismatch |
| split27 | 10.77／26.59／74.0 | 58.8 | 59.5／59.4 | 0/513／0/513 | 4.15／4.63 | 0／1 | 0／0 | FAIL：伺服器 tick |
| split28 | 10.77／26.50／88.8 | 58.9 | 59.5／59.4 | 2/523／0/523 | 4.32／4.51 | 0／0 | 0／0 | FAIL：伺服器 tick |
| split29 | 11.10／25.42／86.5 | 58.5 | 59.6／59.6 | 0/525／0/524 | 3.44／3.26 | 0／0 | 0／0 | FAIL：伺服器 tick |
| split30 | 9.28／22.33／57.1 | 60.4 | 59.5／59.4 | 0/506／0/500 | 3.36／3.89 | 0／0 | 0／0 | FAIL：伺服器 tick |
| split31（netem） | 11.92／27.33／64.7 | 57.5 | 59.0／58.7 | 0/158／0/151 | 3.78／4.89 | 0／182 | 0／0 | FAIL：伺服器 tick |
| split32（1080p） | 12.49／27.03／59.4 | 57.0 | 59.1／59.0 | 5/239／3/238 | 4.38／5.20 | 0／0 | 0／0 | FAIL：伺服器 tick、p1 卡頓 2.1% |
| split35（斷線，部分污染） | 13.72／44.18／123.3 | 54.8 | 72.5／58.1 | 5/418／4/318 | 5.47／5.37 | 5／295 | 0／0 | FAIL：伺服器 tick、late 被拒（斷線情境預期會有） |
| split36（被編譯污染） | 17.75／47.31／145.0 | 47.7 | 58.2／58.9 | 16/494／7/514 | 7.95／6.63 | 0／1 | 0／0 | 不採計 |

## 4. 結論

- **UE（PASS）**：所有未污染的局平均 59.0～59.6 FPS，貼著 60 的上限；1080p 也維持 59.1／59.0。暖機後的卡頓視窗多在 0～1% 之間，只有 split32 的 p1 是 2.1%（5/239），略超門檻。
- **伺服器（FAIL）**：
  - 未污染的局平均 tick compute 為 9.3～12.5 ms，超過 8 ms 的門檻。
  - p99 視窗平均 22～27 ms，代表每局都有不少秒數的平均超出 60 Hz 預算；實際 tick rate 仍維持在 57～60 Hz，靠的是之後的追趕。
  - split19 的 36 分鐘局，每 1/10 段的平均都在 11～14 ms 之間，沒有隨時間增長，所以不是洩漏，而是穩定的過重負載。
- **client runtime**：每個 frame 的 replica step 平均 3.3～5.5 ms。
- **網路與正確性**：未污染的局 late 被拒 0、10054 為 0；E056 之後 mismatch 為 0。

## 5. 熱點與下一步（errors.md E068）

- **正式 MOBA 路徑沒有分段計時**：伺服器 log 只有 `tick_compute` 一個總數。既有的 `TickProfile`（run／dispatch／outcomes 與前 13 個 system）只在舊路徑記錄，split19／27／29／32／36 的 server log 中都出現 0 次，因此目前無法指出是哪一段吃掉時間。
- **建議的通用修正**（omb，下一輪）：在正式 tick 依 phase 輸出 OM_PERF，至少涵蓋 dispatcher systems、script dispatch、process_outcomes、visibility wave B、team projection（每隊）、observer validation，讓這個工具可以直接列出熱點。
- **候選熱點**（推測，待分段數據確認）：每隊各做一次 team projection 與 wave B 可見度；伺服器同時跑兩個 observer replica；ECS 只有 3 條 worker thread。
- **固定硬體量測**：需要一個沒有建置在跑、只開 1 個 UE 的 10 分鐘時段，才能依 README §8 量測單一 client 1080p 的 p95／p99 frame time。

## 6. 伺服器熱點優化（errors.md E073）

**基準**：split37-perf-baseline（build-35，乾淨時段，quick map，training＋saika，2 UE＋8 bot，10 分鐘）。

- 伺服器 tick 平均 13.21 ms、p99 視窗 35.37 ms、max 326 ms、55.1 Hz。
- phase：dispatcher 8.47 ms（64%）、team_projection 1.64 ms（12%）、security_metrics 0.90 ms（7%）、visibility_wave_b 0.80 ms（6%）、bots 0.66 ms（5%）、bootstrap_store 0.13 ms。
- system：`hero_command` 16.8 ms／tick，其他 system 都在 0.05 ms 以下。

**修正**（輸出完全相同）：
1. 地形尋路：`TerrainIndex` 只量化一次地形；`NavMemo` 跨 tick 快取「格子是否被擋」與「格心到格心的邊是否被擋」，地形指紋改變時清空；BFS 的 parent 改用密集陣列，FIFO 與鄰格順序不變。
2. 可見度歷史改成 `Arc` 共用：`security_metrics` 不再每 tick 深拷貝 2×512 個可見集合；projection 開頭 clone `TeamVisibilityRuntime` 時也只複製指標。

**離線量測**（release）：quick map 上每次尋路查詢 1.236 ms → 0.080 ms（15.5 倍）。新舊 planner 在所有已編譯地圖、每張 600 個查詢 × 3 種半徑下結果完全一致。

**after**：split43-perf-after（build-37，乾淨時段，同一配置，加 `--autopilot lane_v0`，比賽自然結束）。對照指令：`tools\lua\lua.exe scripts\summarize_moba_perf.lua --run target/mvp-a3/split37-perf-baseline --run target/mvp-a3/split43-perf-after`。

| 項目 | 基準 split37 | after split43 |
|---|---|---|
| 伺服器 tick 平均／p99 視窗／max | 13.21／35.37／326.2 ms | 6.18／12.41／99.3 ms |
| tick rate | 55.1 Hz | 61.3 Hz |
| UE fps（p1／p6） | 58.4／58.3 | 59.5／59.6 |
| UE hitch 視窗（p1／p6） | 6/488、5/487 | 0/527、0/522 |
| observer replica step 平均（team1／team2） | 6.06／5.20 ms | 2.37／2.61 ms |
| `hero_command` | 16.78 ms | 4.50 ms |
| dispatcher phase | 8.47 ms（64%） | 2.94 ms（48%） |
| security_metrics phase | 0.90 ms | 0.06 ms |
| mismatch／terminated／10054 | 0／0／0 | 0／0／0 |
| 判定 | FAIL（tick 平均、p99、rate） | PASS（目標：平均 < 8 ms、p99 < 16.7 ms） |

剩下的熱點：dispatcher 2.94 ms（48%，其中大部分是 `hero_command`）、team_projection 1.64 ms（26%，兩隊各約 0.8 ms，優化前後沒變）。目標已經達成，這兩項留到 next-month 再看。split42 漏加 `--autopilot`、英雄全場閒置，不能當對照（數據為 tick 6.09／12.40 ms，僅供參考）。

**正確性附帶修正（errors.md E074）**：移動目標在地形內時，改為走向最近的可達點，或以 UNREACHABLE 拒絕，不再靜默丟棄命令。
