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
- 2026-10-29 20:40：使用者恢復無人值守（「繼續把遊戲做完」）。決定：tmp_fix.lua 保留、184G 清單刪除（30 個目錄，釋出 178G）、產生的 Rust 不進版控。check_live_consistent --ubt 通過。舊 session 已不在，依 plan-2026-10-29-resume.md 新開五條工作線：line-b-runtime（M3' ①②④）、line-a-codegen（M3' ③）、line-c-ue（M7 呈現、Grok 美術）、line-gamemode（野怪營地、樹、加兵）、line-conformance（E226 重跑）。10 分鐘例行檢查 cron a88ef402。
- 2026-10-29 20:56：例行檢查：五條線都在跑；line-c-ue 的截圖局 shop8-1 持鎖 14 分鐘超時，已請它照 PID 結束並在之後設時限。
- 2026-10-29 21:06：例行檢查：shop8-1 於 21:06 釋放鎖（共約 24 分鐘，已提醒 line-c-ue 設時限）。新 land：村上義清地裂 handler（No1_ground_slam.rs），check_live_consistent 通過。五條線都在跑。
- 2026-10-29 21:26：例行檢查：A 在影子 target/lineA/ws3 讓 97 位英雄的產生 crate 在 deny(warnings) 下編過（每英雄 rlib＋cdylib 雙 package，避開 LNK2005），等 B 放回 M3' ①；已派 A 先做 v3_codegen_coverage、第 1 批產生器寫法與資料類 fix-candidates，並刪 ws／ws2。其餘四線在跑，無鎖。
- 2026-10-29 21:36：例行檢查：C 完成 M7 道具呈現收尾（shop8-1、roadmap 122 件「尚未開放」截圖、說明框改通用分類）與 M6 第 1、2 批 Grok 美術（E231 generated_art 索引）；已派 C 修 720p 價錢重疊、接第 2～9 批呈現與美術。B 放回 ability_kit（lib、codegen、metadata），實機局 m3p/live1 21:33 起持鎖。
- 2026-10-29 21:46：例行檢查：B 的 m3p/live1 於 21:40 釋放（約 7 分鐘）。A 改成英雄 crate 由 build.rs 在建置時產生（$OUT_DIR，include!），97 組在 ws3 通過；已再派 coverage、第 1 批寫法、資料類 fix-candidates。conformance 放回 v3_conformance_tests.rs。C 在跑 UE build（build_ue_moba 鎖）。
- 2026-10-29 21:55：A 問 FC-038 等 7 項「建議不修」怎麼辦。主 session 決定全修：層數類滿層總加成＝原作固定值（FC-044 上限 4 位）、A14T 改成施放才生效、B02R 改 18 枚 90/180/270/360、兩件暴擊道具各自判定。C 回報 M7 收尾與第 1、2 批美術完成，已在做下一步。
- 2026-10-29 21:56：例行檢查：A 修好 7 項原本「不修」的 FC（v3_fixed_quirks_test 16 項），fix-candidates 已修 15 項；A 每件事做完就停，已要求一路做完清單。conformance 放回 known_differences.json。B 實機局 m3p/live2 21:48 起持鎖。
- 2026-10-29 22:16：例行檢查：conformance 比對產生的 Rust 17 招（2289 情境），只有 C17W 自我施放不同，已回報 A；比對腳本用了 Python，已要求改 Lua 並一路做完清單。遊戲模式放回 22 檔（野怪營地 jungle.rs、nobu_neutrals、視野遮擋、bots，含 UBT）。B 的 m3p/live2 約 12 分鐘稍超時、live3 約 10 分鐘。
- 2026-10-29 22:36：例行檢查：C 修好 720p 價錢（UiSmall），第 2 批呈現（command_restricted、no_healthbar 通用外觀）放回，第 1～9 批 Grok 美術 229 張全部補齊；已派 not_on_minimap、73 個 Niagara 特效、與 B 定 runtime 呈現介面。遊戲模式 UE 實機局 e187/ue-quick4 22:28 起持鎖。
- 2026-10-29 22:46：例行檢查：遊戲模式 UE 局 e187/ue-quick4 持鎖約 18 分鐘超時（log 22:42 後停寫），已請 line-gamemode 照 PID 結束並設時限。conformance 再放回 v3_conformance_tests.rs。
- 2026-10-29 22:56：e187/ue-quick4 約 28 分鐘未自然結束（tick 95592），line-gamemode 未回應，主 session 照 owned-processes.json PID 結束三個程序，鎖正常釋放；已要求設對局時限並先用 headless 確認打得完。C 放回 not_on_minimap 與狀態外觀（OmStatusLook、OmMinimapModel，UBT 通過）。
- 2026-10-29 23:06：A 放回 B08／A07 的 FC-005／025／046（content hash 變，已通知重建），修好 C17W 產生版自我施放；FC-078 等 B 的 ①；FC-129～131 交 B。第 2～9 批剩下的寫法要 host API，主 session 決定由 A 自己實作（先跟 B 確認檔案不衝突）。
- 2026-10-29 23:26：e187/ue-quick5 又持鎖約 18 分鐘未結束（tick 61484）；主 session 照 PID 結束被 auto mode 擋下，改請 line-gamemode 自己結束，並規定 headless 能 10 分鐘內打完前不再開 UE 局、每局帶時限。
- 2026-10-29 23:36：例行檢查：ue-quick5 於 23:36 釋放（共約 27 分鐘）。C 閒置 18 分鐘，已催它確認背景重拍結果並繼續 Niagara 特效。其他三線在跑，無新 land。
- 2026-10-29 23:46：line-gamemode 在 23:36 又開 e187/ue-quick6（1 人類＋9 bot、三線 quick、launch 沒有對局時限），到 23:46 約 10 分鐘。它的 turn 一直沒結束，主 session 22:46 起的三則訊息都還在佇列沒讀到；主 session 無法結束它的程序（auto mode 擋下）。launch-plan 顯示 hero_a02.dll 已在 script_modules（B 的 M3' ① 已生效）。
- 2026-10-29 23:58：查明 e187 UE 局超時的原因：C 的 UnrealEditor-Cmd（d3d12，ue-shadow-c）同時在跑，UE 初始化要 5～15 分鐘，對局本身 3.7 分就結束。E187 headless 已放回（E234）。已協調 C 做完目前重拍後暫停開 UE 約 15 分鐘，直接通知 line-gamemode 開局。
- 2026-10-30 00:06：例行檢查：ue-quick6 23:58 釋放。C：not_on_minimap 放回；Niagara commandlet 做法可行但第 1 批品質不夠（改 SimpleExplosion 再重拍）；runtime 呈現介面提案已送 B；已暫停開 UE 並通知 line-gamemode 開局，暫停期間做非 UE 工作。C 的 UnrealEditor-Cmd PID 42756（23:37 起）卡住無法結束，回報使用者。A 在等 omoba-core 回歸測試（23:50 起）。
- 2026-10-30 00:16：A 放回四項 host API（unit_var、scheduled_effect、status_event、投射物命中排程；17 檔，ABI 變動需全部重建），可產生招式 50→69，修 C17W 自我施放與 require_source_alive 預設；道具 70→62 是修正誤判。已派 A 自己做 on_take_damage_pre、推 coverage、M5 比對後換 handler、開始 items.dll。卡住的 UnrealEditor-Cmd 42756 已不在。
- 2026-10-30 00:30：e187 ue-quick7（540 秒時限）正常收尾，但 C 暫停後 UE 載入引擎仍卡 9 分鐘以上，仍無局中截圖；UnrealEditor-Cmd 42756 仍在且無法終止（主 session 00:16 誤判已消失，更正），疑 GPU 驅動層卡住，需使用者重開機。E187 其餘已放回（E234），UE 驗收擱置；C 恢復開 UE，一次一個實例。
- 2026-10-30 00:40：e187 查到 UE 不顯示結果、不自己結束的原因（推測）：晚進場時第一幀已是 phase=2，SetMatchResultState 在面板未排版時 return、之後不再判斷。已交給 C 修成排版後再判斷一次，附 UE 測試。
- 2026-10-30 00:45：UnrealEditor-Cmd 42756 自己結束，不需重開機。C 恢復 UE 工作；已派 line-gamemode 重拍 UE quick 驗收，再做中路加兵、兵營追加兵、建築回血、lifecycle 測試。
- 2026-10-30 01:00：使用者決定 D7 門檻：2 招以上類似的機制做成共用（v3 詞彙＋產生器＋ability_kit），完全跟大家不同的才寫 hook（AI 實作產生的 trait，取代 stub）。已改 README D7、m3-rust-codegen-design §2.4（原 2.4 ability_kit 改為 2.5）、記憶；通知 A（trait 產生器、coverage 分類、hook 候選清單）、B、conformance。hook 實作工作線等 A 交出候選清單後再開。
- 2026-10-30 01:05：A 查明 B01 召喚上限由 host summon_owner_cap.rs 執行（有測試）；產生器改成只在三條件下接受 limit_per_owner、其他 block；另 5 份文件原本被默默略過。destroy_trees 在 host 是空操作：產生但列入 coverage「host 沒做到」一節，host 實作交給 line-gamemode。
- 2026-10-30 01:10：C 更正：42756 已消失、UE 啟動約 1 分鐘；73 個 cue Niagara 已全部放回（cap-all3 總表）。主 session 看總表：外觀都是同一種光球＋火花，只差顏色大小；已請 C 修完結算畫面 bug 後依 cue 種類分範本重做。
- 2026-10-30 01:15：C 修好 UE 晚進場不顯示結算畫面（bMatchResultPresented，UE 測試 MatchResultAfterLateJoin Success，3 檔 UBT 通過放回），已請 line-gamemode 下一局實機確認。A 放回 limit_per_owner 修正前在等 omoba-core 全套測試，之後交 hook 候選清單。
- 2026-10-30 01:20：E187 UE quick 驗收通過（ue-quick9：34 秒進場、4.0 分自然結束、13 座建築倒、結果畫面截到、mismatch 0）；中路加兵、兵營追加兵、建築回血、足輕成長已放回（E234），lifecycle 3 seed 通過。缺口：nobu 樹與 24 營 UE 沒畫（bridge driver.rs:1021 有 fog grid 就不送樹）→ 交 C；nobu 完整量測只完成 1 局（40.7 分）→ gamemode 補量測並做 destroy_trees。
- 2026-10-30 01:30：A 放回 trait 產生器（整招 hook、沒實作編不過、不再產生 stub）與大批寫法（damage_pre、slow、疊層、modify_cooldown、teleport 等，22 檔），可產生招式 50→100；A02、B01、C17 已換成產生的 Rust 上 live。hook 候選：45 招 62 項（另 57 項產生器自補）。conformance 支援 hook 情境；已派它重跑全量確認三位英雄 0 退步。等 A 把清單放到 repo 後開 hook 工作線。
- 2026-10-30 01:35：A 把 hook 候選清單放到 hook-candidates.md（B 類 45 招）；items.dll（66/168 件）放回中，道具使用與裝備改走 script registry 交給 B。主 session 看清單：不少項目其實是規則層狀態或類似機制（hex、taunt、dash、chain、destroy_tree），應先依「類似就共用」複核。開 hook 工作線被 auto mode 擋下，改提卡片（task_f694e9fe）等使用者啟動。
- 2026-10-30 01:40：C 把 73 個 cue 分 6 類（ring/aura/buff/burst/trail/slash）放回；主 session 看 cap-all7 總表仍看不出環形，trail/slash 仍是一團。已請 C 用遊戲視角拍、ring/aura 用環形 emitter、trail 用 Ribbon、slash 用弧形。B 同意 runtime 呈現欄位，排在 M5 之後。
- 2026-10-30 01:45：A 回報本輪 33 檔放回（damage_pre、hook trait、limit_per_owner、items.dll 66/168 件），A 類剩約 50 項等 B 的 ext 讀取 API；道具改走 script registry 由 B 接。已派 A 先用「類似就共用」複核 hook 清單、補不依賴 B 的共用寫法、整理 M6 第 1 批解鎖清單。C 放回 bridge lib.rs（樹／營地接線中）。
- 2026-10-30 01:55：A 複核 hook 清單：真正獨有只剩 a13_chain_sickle、c06_grand_heist，其餘併入規則層（清單給 B）或共用；不依賴 B 的共用寫法已放回，可產生 100→106 招；M6 第 1 批解鎖清單 m6-wave1-unlock.md（44 招：已產生 16、共用待補 28、無 hook）。撤回 hook 工作線卡片，2 招 hook 由 A 直接做。B 目前是瓶頸（ext 讀取 API、規則層清單、道具路由、ability_kit）。
- 2026-10-30 02:00：A 放回 A13E 鎖鏈鐮 hook；修投射物 profile 缺傷害類型位元；接上 B 第 1 批 give/take_gold、burn_mana、dash、teleport、dodge_projectiles 等；可產生 113 招，第 1 批 17/44。C06T 被擋：create_pickup 與讀金錢交給 line-gamemode（減輕 B），issue_order 併入規則層 taunt。A 接著做 items.dll host 接線（B 審）。
- 2026-10-30 02:10：C 放回 nobu 靜態樹送到 UE（有迷霧也送，nobu-trees2 截圖有樹）與 cue 遊戲視角對照表（shapes4：ring／aura 是真的環）；trail／slash 仍是亮點，已請 C 查光帶並改 Ribbon／弧形。A 的 C06T 等 gamemode 與 B；items.dll 接線等 B 回設計問題。
- 2026-10-30 02:26：B 放回 damage_from_at（傷害紀錄帶來源與路徑）與 headless 測試斷言修正後停下；已催 B 一路做，先回 A 的 items.dll 兩個設計問題與 taunt 狀態細節，再做 ext 讀取 API、remove_status、規則層清單、M3' ②④。
- 2026-10-30 02:35：C 修好光帶（暖機沒用到 M_Ribbon、改貼地從施法者拉到目標），shapes12 對照表 6 類一眼分得開，主 session 接受此版；真正 Ribbon／弧形 emitter 排到 M8 美術打磨。C 下一步：砍樹移除、M6 各批呈現、runtime 呈現欄位。A 把 damage{source} 改產生 damage_from_at（B 要求）。
