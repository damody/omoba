# 主 session 調度紀錄

排程觸發的回合不能開新 session，所以新工作改交給閒置的 session。

| 時間 | 事件 |
|---|---|
| 2026-10-28 21:50 | M0 完成，已核對：v3.0 凍結，66 項測試通過，12 份範例 OK |
| 2026-10-28 21:55 | M0 session 接著做 M1 資料線：content-model 加 values／text、catalog、出處比對、10 位英雄與 45 件道具改模板 |
| 2026-10-28 21:55 | 法術增幅 session（已完成）改名為「網站產生器」，接手工作線 E |
| 2026-10-28 21:55 | M4 普查第一波子 agent：H1～H6、I0、I1（見 census-v3/BATCHES.md） |
| 2026-10-28 22:25 | 網站 MVP 完成並核對（29 筆共用向量、78 頁）；UE TooltipRenderer 改交給同一個 session（它寫了共用渲染向量），改名為「網站產生器與 UE 說明渲染器」 |
| 2026-10-28 22:35 | schema v3.1 完成（70 項測試），RESULTS.md 的缺口全部處理；已通知執行中的普查子 agent |
| 2026-10-28 22:35 | M2 第一批放回 live（E193，27 檔）。同一個 session 改名為「規則層、執行器核心（M2、M3）」，先完成 M2 剩餘項目與 v3.1 規則，再做 M3 |
| 2026-10-28 23:00 | E187 原作塔兵野怪數值放回 live（quick 倍率 3.5，中位數 13.4／10.4 分，1/4 停在兵營前）。請它繼續修 bot 推進到中位數 ≤10 分鐘，並補原作剩下的規則（塔成長、騎兵、兵營破壞強化） |
| 2026-10-28 23:00 | E187 後 `three_lane_60hz_full_filtered_match_lifecycle` 在 release 失敗（replica 多記 lumen_bolt 冷卻，entity id 同 tick 重用）→ 交給冷卻不同步 session |
| 2026-10-28 23:30 | **M4 普查完成**：97 位英雄與全部商店道具共 105 份 v3 文件，全部通過驗證；614 筆中 545 筆可表達、4 筆需要 hook、65 筆待確認（見 census-v3/RESULTS.md 總計） |
| 2026-10-28 23:45 | v3.2 完成（登錄表 v3-shared-registry.lua、chain、偷技能、chase_target AI 等，72 項測試）。開 4 個清理子 agent：道具、A、B、C；登錄表新增項目先寫到 census-v3/registry-additions-*.lua |
| 2026-10-29 00:20 | 清理：C 完成（pending 15→0）、A 完成（17→5，剩下的要實測）、B 完成（11→3，正在收最後 3 筆）。schema 補了 chase.max_distance／anchor |
| 2026-10-29 00:40 | 清理全部完成（A 17→5、B 11→0、C 15→0、道具 23→1；hook 全 0）。合併交給 schema 負責人（M0 session，登錄表與測試的擁有者） |
| 2026-10-29 00:50 | **M4 完成**：登錄表合併、全檔測試打開登錄表檢查；主 session 重驗 105 份全部通過；598／614 可表達、0 hook、6 pending（5 筆要實測、1 筆 fix_candidate） |
| 2026-10-29 00:01 | 發現兩組實機局同時在跑（ue-run/55d93… 與 44aab…）。已提醒所有 session；機器層級的「實機局鎖」交給冷卻不同步 session 做成通用機制 |
| 2026-10-29 00:45 | **M1 工作線 A 放回 live**（116 檔，check_live_consistent --ubt 通過；實機 9/9 PASS；V3Block、V3_CATALOG_JSON）。M3 拆分：Lua IR 編譯器與全量 catalog 交給 M0 session（改名「技能 schema、說明資料與 IR 編譯器」），Rust 直譯器留在 M2／M3 session |
| 2026-10-29 00:55 | live 編譯失敗（super_minion_replaces_siege，比賽長度那條只放回一半）→ 通知修正；01:00 一致性檢查恢復通過 |
| 2026-10-29 01:05 | 實機局鎖完成（E197）。冷卻不同步 session：無爭用重跑阿市局、確認 lumen_bolt 測試、列出可清的 target。M0：v3 文件移成正式作者來源、測試情境產生器 |
| 2026-10-29 01:40 | e199 live-1 渲染端消失：原以為是依名稱停程序，查明是 E201（TooltipRenderer 自行載入又釋放 bridge DLL，把遊戲中的 bridge 卸載）。有問題的版本在 live 的時間：10-28 23:52:49～10-29 01:31:49，已修好；已通知所有 session 重建 UE，並撤回指控 |
| 2026-10-29 01:50 | E200 核對：v3.lua 成為正式作者來源（97 位英雄＋166 件道具），kit 合併，測試情境產生器（29,412 條），310 檔放回。M0 下一步：使用頻率統計與執行器元件優先序 → 交給 B；依 D13 處理 M5 前的決定；呈現 cue 目錄 |
| 2026-10-29 02:00 | 使用頻率統計完成（usage-stats.md，308 種元件；55 個解鎖 B01＋C17，277 個解鎖 92 位英雄）並交給 B。網站 session：M1 C 收尾（全部顯示位置、截圖）→ M8 全量網站 |
| 2026-10-29 02:15 | 冷卻不同步 session 全部完成（E196 tick_start_hp、E197／E202 鎖與異常中止標記、光鍊 mismatch 結案、可清約 184G 清單）。lifecycle 測試推不動原作塔 → 轉給比賽長度。該 session 改名「已移植英雄 conformance 測試（M5 前置）」：從情境產生 10 位已移植英雄的 Rust conformance 測試 |
| 2026-10-29 02:25 | E187 第二批放回：quick 中位 7.5／8.1 分、24/24 結束；UE quick 實機 7:32 自然結束、mismatch 0、小兵面向正確。M0 三項完成（usage、D13 決定、cue 目錄）；下一步：以推定解開 6 筆 pending、97 位英雄全量內容管線（unlisted） |
| 2026-10-29 02:40 | M1 C 收尾（技能欄、商店、選角接 TooltipRenderer；720p／1080p 截圖）與 **M8 全量網站**（512 頁，97 英雄＋156 道具）完成。主 session 看截圖：tooltip 下半部仍接舊 runtime 區塊（名稱重複、「L1 CD 0s」、「魔力消耗未提供」與「耗魔 140」矛盾）→ 請 C 整併。M0：6 pending 解開（FC-132～134）、全量內容管線 615 區塊；下一步 E204 狀態說明 |
| 2026-10-29 03:05 | tooltip 整併完成（v2 截圖乾淨），網站與 UE 的 content hash 相同；E204 狀態說明放回（schema 3.3）。M2 確認完成（E198）。C：狀態圖示、召喚師技能、守衛接 TooltipRenderer。M0：M6 批次規劃、bot 使用資料 |
| 2026-10-29 03:25 | M0：M6 批次規劃完成（m6-waves.md，9 批）、bot 施放資料（E206）。空檔任務：原作地圖 nobu.vmap 的 DMX 解析、地圖資料與比較報告（不改 live 地圖） |
| 2026-10-29 03:35 | C：狀態說明接上 TooltipRenderer，修正道具 id 對應（網站上架數 45／111）。C 改名「UE 通用機制與呈現」，新任務：動態技能欄、向量施法、路徑投射物與連鎖、cue 通用佔位特效（依 m6-waves 順序） |
| 2026-10-29 03:45 | conformance 基準（E207：2289 情境，560 過、38 已知差異、1691 unsupported）。下一步：host 走近再施法、harness 擴充。原作地圖解析完成（dmx.lua，32 塔、12 兵營、24 營、地形網格與 heightmap）；下一步：nobu_map template（不接線）＋ 9 項引擎能力規格 → 之後交給遊戲模式 session |
| 2026-10-29 04:45 | C：E208 三批放回（充能 HUD、CABI 40、cue 佔位特效與 Editor 覆寫）；開關、重施放、換技能等 B 的 fact。空檔任務：原作圖示（頭像、技能、道具）匯入 UE 與網站、bridge 商店測試 45 對 64。M3 在 target/m3n 進行中（ir.rs、ir_switch.rs） |
| 2026-10-29 05:55 | E209：原作圖示匯入（英雄 67／97、技能 303／446、道具 140／176，排除 46 張 Dota 肖像）、商店 43／45 件有圖、bridge 商店測試修好。manifest 格式改變 → 已通知 e187、conformance、B 重建影子 UE。C 可以加 JPEG 解碼 crate |
| 2026-10-29 06:25 | **M3 第一批完成（E212）**：B01、C17 改由 IR 執行，5 場重播每 tick 相同，效能沒有退步；604 個 IR 全部可載入。E210 走近再施法、E211 狀態情境（conformance 805 過）。M0：ir-blockers.md 優先序分析；conformance：effects／可見性；B：排程器、selector、M5 切換，並送 fact 給 C |
| 2026-10-29 06:50 | E187 第三批完成：中路加兵、兵營破後強化與追加波、大將塔加成、魔王兩隻（boss tag）、home_aura、經驗金錢規則、投降（伺服器端）；quick 中位 6.3／5.4 分，UE 實機 04:43 自然結束、mismatch 0。M0 打開 EMIT_UNIT_STATE（103 區塊帶 unit_state）；ir-blockers：604 個 IR 只有 13 個能執行。**主 session 發訊達上限（10 則），B、M0、e187、C 都閒置，要等使用者輸入才能恢復派工** |
| 2026-10-29 07:05 | 所有 session 閒置的事件觸發後，發訊恢復。B：M3 第二批（排程器、selector、cue／toggle fact）→ M5 切換。e187：原作 nobu 地圖接線（9 項能力）＋剩餘規則。conformance：effects／可見性、E213 驅散。C、M0 等 B 的放回 |
| 2026-10-29 10:30 | 使用者：「全部繼續」。發現 07:28～10:27 所有 session 閒置約 3 小時，各自等的背景工作中斷、沒有被喚醒（機器上沒有任何程序）。已逐一叫醒：B 重跑稻姬比對並繼續 M3／M5；e187 繼續 nobu 地圖；conformance 重跑測試；M0 做英文說明審查；C 做向量施法與預覽的 UE 端、選角自動上架。重新申請 keep-awake |
| 2026-10-29 11:10 | M0 英文說明審查完成（100 份、en-glossary、v3_en_lint）。主 session 決定 3 項：Voodoo Power 依中文直譯、玩家說明不得含開發用語（加 lint）、羅馬字不加長音。B 稻姬 IR：omoba-core 1317 過，等重播比對；e187 nobu 地圖兩個副本已從中斷處重啟 |
| 2026-10-29 12:15 | conformance：E213 傷害量、E214 host 驅散、E216 可見性、E217 套上狀態放回；1005 通過、17 (b)、104 (c)、1163 unsupported；IR 與 handler 無差異。請它把 104 (c) 分成 c1／c2／c3 處理。C：向量施法輸入、路徑與彈跳預覽、選角頭像放回，排隊實機確認 |
| 2026-10-29 12:45 | C 的 720p 實機檢查通過（選角、技能欄、商店）。發現 bug：人類選到 bot 預設英雄時，選角服務依 E164 改 bot，啟動器的 selection_result 檢查過時，對局沒有開始 → 請 C 修成通用的 roster 合法性驗證 |
| 2026-10-29 01:10 | fix_candidate 決策清單完成：`fix-candidates.md`，131 項（FC-001～FC-131），影響高 21 項；預設建議 124 修、7 不修（層數只顯示那組修了數值會倍增）。等使用者回覆，在那之前資料照原作（D13） |
| 舊待辦 | 道具與 B 收尾完成後開「合併」子 agent：合併 4 份 additions 進登錄表；A29T、B10W、C16R 改用 chase.max_distance＋anchor；打開 census 全檔測試的登錄表檢查；105 份全部重新驗證 |
| 待辦（原始） | 清理完成後：合併 additions 進登錄表，打開 census 全檔測試的登錄表檢查（registry = true），全部重新驗證 |
| 舊待辦（已由上列取代） | v3.2（跨文件登錄表、chain、偷技能等）完成後，開清理子 agent：已完成批次的 pending 改用 v3.1 詞彙；修 32 筆 kv 不一致（v3_provenance.lua） |
| 待辦 | 整理 fix_candidate 清單給使用者（M0 的 19 筆，加上普查新增的） |

- 2026-10-29 13:35：M0 已完成法術增幅普查（60 份放回）。派 M0 和 B 協調，分擔 ir-blockers 裡可以獨立處理的直譯器項目（fn:*、stat:*、target.types、on_learn／on_respawn、傷害 flags），以加快 M5；B 不需要分工時改做 M7 前置（商店由 v3 values 產生）。
- 2026-10-29 14:03：conformance 的 (c) 已清零（1117 通過、17 筆 (b)、1155 筆 unsupported），下一步派它擴充 harness，降低 unsupported。UE 已確認敵方狀態圖示不需要改；下一步派它量測狀態視覺上限（512），並準備 M6 第 1 批的呈現。
- 2026-10-29 14:12：B 與 M0 已分好 ir.rs 的工作（M0 負責 STATS 四列、fn:direction／offset／position、target.type direction、on_learn／on_respawn）。M0 要等 B 的 m3d 放回（大約一小時），這段時間先做 M7 前置：商店價錢、配方、屬性改由 v3 values 產生。
- 2026-10-29 14:55：M0 完成 M7 前置。商店 45 件的價錢、配方與屬性改由 v3 values 產生（`item_shop_v3.lua`，與原作的差異 0 件）；網站 45 件價錢與遊戲一致。新增 m7-items.md 道具解鎖順序：今天能載入 13 件；接下來是 stat 的 attack_damage／int／str／agi 四列，可再解鎖 15 件上架道具。修了一直失敗的 item_sources_check_test（E222）。
- 2026-10-29 14:33：UE 完成 teamfight stress（狀態視覺峰值 23／512，沒有因為達到上限而被丟棄）與 M6 第 1 批呈現準備（兩項通用 fallback）。下一步派它做第 2～9 批的呈現覆蓋、彙整 m6-missing-art.md。conformance 已在 14:28 放回 harness 擴充。
- 2026-10-29 16:20：M0 的 ir.rs 分工已放回（codegen＋cargo）：fn direction／offset／position、target type direction、target types hero+creep（含 BASIC 與帶 building 的組合）、on_learn 一般程式、on_respawn 被動、STATS 新增四列（attack_damage、move_speed_pct、incoming_physical_pct、lifesteal_pct）。ir.rs 新增 4 個對照 handler 的測試，runtime::native 共 946 個測試通過。可載入的條目 55→71，M5 阻擋項 66→49，道具可載入 13→30。b29_ground_slam 與 c15_precise_light 已能完整載入，是否切換由 B 決定。slow_pct 與 vision／outgoing 留給 B 和規則層。影子測試樹缺檔記為 E225。
- 2026-10-29 15:53：UE 完成第 2～9 批的呈現報告（9 份）與通用狀態外觀；缺圖清單是 m6-missing-art.md（頭像 26、圖示 130、cue 129），待使用者決定；團戰重測峰值 50／512。下一步派 UE 做 M7 道具呈現：商店、合成樹、說明框只讀 item_shop_v3，未開放的道具通用標示。
- 2026-10-29 16:22：使用者指示「全部停止」：已取消 cron，5 個 session 都已停止，背景程序已結束（第二次結束時被使用者中斷，留下 1 組 M0 的 debug cargo test）。16:35 寫好交接計畫 handover-2026-10-29.md。
- 2026-10-29 16:50：使用者決定：134 項 bug 全部修、缺圖用 Grok 生成、515G 暫存檔要刪（已刪除）、要 commit；不用 IR，改成產生 Rust，每位英雄一個 DLL、可熱加載。已改 README（D13～D17、架構、M3'）、新增 m3-rust-codegen-design.md，並在 m3-ir-design、fix-candidates、m6-missing-art、v3-content-model-proposal 加上註記。
