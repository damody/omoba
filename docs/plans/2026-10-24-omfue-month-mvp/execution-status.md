# 執行狀態

依 schedule.md §5 格式；新紀錄加在最上面。工作 ID 對照 schedule.md §3。

## 看板

| ID | 狀態 | 負責 | 備註 |
|---|---|---|---|
| A1 | accepted | 主 session | baseline 見下方 2026-10-25 紀錄 |
| A2 | accepted | 主 session | presentation port 改用 player ID；新增 `--host-simulations`／`--host-renderers` |
| A3 | doing | 主 session | B＋C＋C 單機跑通到雙方進場 |
| A4 | accepted | 主 session | headless batch 十局上限改成可設定 |
| A5 | todo | 主 session | |
| A6 | todo | 主 session | 需使用者確認的人工／硬體條件 |
| D1–D2 | accepted | session `source-index` | `work-D.md`、`source-index/`；97 主檔與 36／36／25 一致 |
| D3–D4 | doing | 待使用者決定 | 候選草案在 `source-index/candidates-draft.md`，尚未逐招判定 |
| F2 | accepted（L1） | session `omfx-decouple` | `work-F2.md`；UE Editor import／PIE 驗證未執行 |

## 2026-10-29（已移植英雄 conformance：被動數值、建築、預設 key；E227，已放回 live）

- 被動數值（54 個，被動照玩家流程逐級學）、建築目標（20 個）、沒寫 runtime_id 的狀態改用執行器的預設 key（B 的約定）。
- B 的 E226（施放破隱、IR 的 for_each 排除魔免）放回後，施法者隱形的判定改看原本的隱形 buff，A02T 影遁轉為通過。
- 16 個「一般情況可以施放、施法者隱形時不被接受」的情境記為 unsupported，已交給 B 確認。
- 結果（live）：1407 通過、25 (b)、4 (c1，B29E 破甲，等 B29 切 IR)、853 unsupported。E223 之前是 1243 通過、897 unsupported。
- 沒有新的 (c) 需要交給 M0。
- 編號：E225、E226 都已被使用，這批是 E227。
- 下一批：普攻事件（前搖、發出、命中）、法力增減、召喚物與產生單位、投射物命中、狀態疊層。

## 2026-10-30 凌晨（history-nobu 工作線 B：施放破隱、IR 選取排除魔免；E226）

- 已放回 live（5 檔；codegen、cargo、server、bridge 檢查通過）。known_differences.json 刪 105 筆 invisible_caster。
- 驗證：omoba-core 1335、replicas_identical＋v3_conformance 47、三組腳本施放與 5 局重播 IR 與 handler 每 tick 相同。
- 接著：踏火無痕與種子島（swept_dash 搬進 omoba-core、host.reads、projectile／dash／dodge／aura／狀態 triggers、execute_disclosed）。

## 2026-10-29 深夜（history-nobu M3 第二批：伊達政宗 B04 火焰強襲、炎之迅捷改由 IR 執行；E224）

- 已放回 live（10 檔；codegen、cargo、server、bridge 檢查通過）。
- 直譯器：selector 單位類型、stun／root／silence、法術增幅規則（require_source_alive=false 且無 no_amplify → SPELL）、cue 爆炸、destroy_trees、str／agi／int。
- 驗證：三組腳本施放各 5400 tick、5 局實機重播 IR 與 handler 每 tick 相同。
- 接著：IR 範圍選取排除魔免（E223 c1）、施放破隱（E223 c2）、踏火無痕與種子島（swept_dash 搬進 omoba-core、host.reads）。

## 2026-10-29（已移植英雄 conformance：unsupported 1155 → 897；E223，已放回 live）

- 依 unsupported 的原因排序，先支援轉換量最多的四類：看不見的目標、施法者隱形、施放後施法者死亡、效果範圍內的魔免敵人。
- quirk 的 288 個是文字政策，無法機械判定，維持 unsupported。
- 結果（live）：1243 通過、25 (b)、124 (c)、897 unsupported。
- 新差異：
  - (c2) 施放不破隱 120 個，交給 B。
  - (c1) B29E 破甲套到魔免敵人 4 個，B 切 B29 時驗證。
  - (b) A07 8 個。
  - (a) harness 少了 server 的 admission，已修。
- 已通知 B 與 M0；M0 沒有要修的 v3 文件。
- 10 位已移植英雄（含 ir-blockers 裡 M5 的 8 位）都已有完整情境，B 切每位英雄到 IR 時可直接比對。
- 下一批：其他狀態事件（普攻相關）、法力與 leap 效果、給友軍的效果、召喚與投射物。

## 2026-10-29 晚（E187 第四批：nobu 地圖接線與剩餘規則；地圖核心與規則已放回 live，野怪營地、樹木、UE 進行中）

- **已放回 live**（兩次放回後 `check_live_consistent` 都通過）：
  - 剩餘規則：建築受傷修正、主堡在大將 500 內無敵、home_aura 加速的取消。
  - nobu 地圖核心：
    - 每方各自的路線、成對的塔、每路 2 座兵營與 5 座大將塔。
    - 第一座兵營倒時全兵種 ×1.3。
    - 305 個地形矩形，加上網格索引。
    - fog 依地圖邊界計算；商店半徑 720。
- **量測**：
  - quick 中位 7.0 分；對調也是 7.0 分。
  - nobu（還沒有野怪營地）：6/6 打完，中位 34 分。team 1 打野去打魔王送頭，所以全是 team 2 勝。
- **進行中**：
  - 24 營、14 種野怪與樹木（影子 `target/e187/t11`）。
  - UE 前端：terrain 上限、minimap 與邊界（影子 `t12`）。
  - 之後做 nobu 量測與 UE 灰盒實機。
- 細節見 errors.md E187 第四批。

## 2026-10-29（已移植英雄 conformance：B 的 E221 之後，(c) 全部通過）

- B 放回 E221（public 狀態的圖示揭露給非擁有者隊伍；C15E 的 zone 燃燒改成每 interval 一跳）。之後 C15E 的 4 筆 periodic 在 live 上回報 listed but passes（每次 25／30／35／40，間隔正確）。B 那次沒看到，應該是在 E220 放回之前跑的。這 4 筆已用 land_to_live 從 known_differences 刪除。
- live conformance 11 項通過：1117 通過、17 (b)、0 (c)、1155 unsupported。known_differences 只剩 A07 的 17 個 (b)。

## 2026-10-29 晚（history-nobu 工作線 B：buff 圖示依 reveal 揭露、C15E 燃燒離散 tick；E221）

- 已放回 live（11 檔；land_to_live 的 codegen、cargo、server、bridge 檢查通過）。
- 對手隊伍現在看得到 `reveal = public` 狀態的圖示（stun、root 等）；擁有者隊伍照舊看到全部。known_differences.json 刪掉 104 筆可見性條目。
- C15E 區域燃燒每 0.25 秒一跳（原作 ThinkInterval）。
- 驗證：replicas_identical 與 v3_conformance（release）47 項通過；omoba-core 揭露與 zone 相關測試通過。

## 2026-10-29（已移植英雄 conformance：(c) 分類、圖示、治療／位移／periodic；E218、E220，已放回 live）

- **(c) 分類**（寫在 m5-conformance-baseline.md §4）：
  - c1（handler 與原作不同，切 IR 後自然修好）0 個；c3（v3 文件或產生器有錯）0 個，所以沒有交給 M0 的項目。
  - c2（host／規則層）108 個：
    - buff 圖示只揭露給擁有者那隊（104 個，E216）。
    - C15E 的 zone 燃燒連續結算（4 個，E220）。
    - 兩項都已交給 B，B 已收下。
- **E218 圖示**：
  - 新增 `templates/status_buffs.lua`，把每個 `buff`／`debuff` 的 v3 狀態都產生成 buff 條目。template-ids 依 runtime_id 樣式提供 `status_buff_by_key`，`buff_visual_state` 用它找圖示；樣式比對與 dispel.rs 共用。
  - 原本 68 個連擁有者都看不到的狀態現在都有圖示。
  - land_to_live 8 個檔，codegen、cargo、server、UBT 都通過；live 的 omoba-core lib 1327 項全過。
- **E220 效果與事件**：給施法者的治療 6 個、施法者 dash／teleport 8 個、periodic 傷害 4 個通過；另有 C15E 的 4 個列為 c2。產生器的 periodic 步驟帶上 interval。
- **結果（live）**：1009 通過、17 (b)、108 (c)、1155 unsupported，11 項測試全過。IR 與 handler 沒有差異（A02 已由 B 切到 IR）。
- **編號**：E219 已被 B 使用，這批改成 E220。
- **待辦**：
  - 其他狀態事件（普攻相關）、法力與 leap 效果、給友軍的效果。
  - B 修好揭露政策與 zone 燃燒後，刪掉對應的 known 條目。

## 2026-10-29 下午（history-nobu M3 第二批：稻姬 A02 改由 behavior IR 執行；E219）

- 已放回 live（14 檔，land_to_live 的 codegen、cargo、server、bridge 檢查通過；check_live_consistent 通過）。
- 直譯器新增：`heal`、`remove_status`、被動 `stats`、狀態 `states`（隱形、無單位碰撞）與 `tags`、`set_absolute`、toggle 進入點、`infinite` 時長、host `attack_orb` 法球、type none 與 rank 檢查。
- host：`base_attack_time_constant` 覆寫基礎攻擊間隔（取最小正值）；稻姬融影術的 handler 與 IR 都改用它（原作 BAT 0.9，先前以攻速 ×2 近似）。
- 稻姬 v3 文件四招改成 expressible；`authored_heroes.lua` 依目前 live 重新產生。
- 驗證：腳本施放（稻姬對阿市、孫市對阿市）各 5400 tick、4 局實機重播（共 200,749 tick）IR 與 handler 每 tick 相同；omoba-core lib 1317 通過。
- 接著：E216（buff 圖示依 reveal 揭露）、projectile／selector.types／狀態疊層／call 參數，B04 起逐一切換其餘 7 位。

## 2026-10-29（已移植英雄 conformance：傷害、驅散、可見性、套上狀態；E213、E214、E216、E217，已放回 live）

- **E213 傷害**：一般施放另外對一個沒有防禦的敵方 body 施放，比對第一次受到的傷害。harness 每 tick 照 authority 做 projection，並讓施法隊伍看得到目標。38 個情境通過，沒有不符。
- **E214 host 驅散**：
  - 狀態的 `dispel` 類別經 `v3_compile.lua` 與 content-model 進 `V3_CATALOG_JSON`；重新產生 authored_heroes.lua 與 item_texts_v3.lua，差異只有新增的欄位。
  - 新增 `omoba-core/src/runtime/native/dispel.rs`：基本／強驅散，debuff 由持有者那一方驅散，buff 由敵方淨化，hidden 看施加者。共用 key 取最嚴格的類別；沒寫 runtime_id 的狀態照 B 的約定，用預設 id 登錄。`magic_immunity::strong_dispel` 改走這裡。
  - 事前通知 B，B 確認沒有衝突。land_to_live 8 個檔，codegen、cargo、server、UBT 都通過；omoba-core lib 1320 項全過。
- **E216 可見性（雙 replica）**：讀每隊 projector 的 buff 圖示。新發現兩類 (c)，記錄未修，列入 known_differences：
  - 對方隊伍看不到 buff 圖示（35 個）：host 的 owner-only 揭露，和 v3 的 `reveal = public` 不同。
  - 許多 v3 可見狀態沒有圖示 id（69 個）。
- **E217 套上狀態**：施放後確認文件列出的狀態出現在該出現的單位上，只算施放前不在的狀態。64 個情境通過。
- **結果（live）**：
  - 1005 通過、17 已知 (b)、104 已知 (c)、1163 unsupported。
  - 先前是 805／14／1470（E211），E207 基準是 560／38／1691。
  - B 切到 IR 的英雄沒有出現 IR 與 handler 不同的情境，所以不需要通知 B。
  - 細節見 `docs/plans/2026-10-28-history-nobu-full-port/m5-conformance-baseline.md`。
- **編號**：可見性原本寫 E215，但已被工作線 A 使用，改成 E216。
- **待辦**：
  - 狀態事件（periodic、on_hit…）、治療／法力／位移效果。
  - 共用 key（stun）的類別分辨。
  - 兩類可見性 (c)：要先決定揭露政策，並由產生器補上圖示 id。

## 2026-10-29（工作線 A：分擔 ir.rs 阻擋項；已放回 live）

- 依 B 的分工，以 m3d（E224）後的 live 為底，經 land_to_live 放回 ir.rs 與 rules/vocab.rs（codegen、cargo 檢查通過）：
  - 運算式：`direction(a,b)`、`offset(p,dir,d)`、`position(x)`、名稱 `direction`（direction 型施放的方向）；新增值型別 `Vec`。
  - 施放目標：type `direction`；types hero+creep、BASIC，以及帶 building 的組合。不含建築的集合在 execute_inner 排除建築，與 handler 相同。
  - `events.on_learn`：只在第一次學會時執行。`passive.triggers.on_respawn`：由 IrHeroUnit 轉發，要已學會，rank 是 1。
  - STATS 新增四列：attack_damage、move_speed_pct、incoming_physical_pct、lifesteal_pct。
- **測試**：ir.rs 新增 4 個用 ParallelWorldAdapter 的測試，比對 handler 的 StatKey 與捨入；runtime::native 共 946 個通過。
- **重跑**：
  - 可載入的條目：m3d 後 55 個，這批後 71 個；與 probe 不一致 0 個；v3_ir_blockers 鏡像清單已同步。
  - M5 阻擋項 66→49 項；m7-items 的道具可載入 13→30 件。
  - b29_ground_slam、c15_precise_light 已能完整載入，是否切換由 B 決定。
- **沒做**：slow_pct 要走 generic_slow 的 ApplyStatus 路徑，屬 B 的範圍；vision_day／night、outgoing_damage_pct 規則層沒有讀取者。
- 影子測試樹缺檔的問題記為 E225。

## 2026-10-29（工作線 A：M7 前置，商店由 v3 values 產生；已放回 live）

- **單一來源**：上架 45 件道具的價錢、配方、屬性與主動數值改由 v3 道具文件產生。
  - gen_item_v3 寫出 `templates/item_shop_v3.lua`；moba_items.lua 用它覆蓋原作匯出的數字與配方。
  - 原作匯出（source_items.lua）只決定 row 的形狀，並作為比對的出處。v3 與原作的差異要寫 `source_differs`，否則建置失敗。
- **比對原作**：45 件的數字與配方差異 0 件。覆蓋前後的貨架 JSON 完全相同（64 列）。
- **網站**：重建 512 頁；45 件上架道具的網站價錢與遊戲一致，都以遊戲數值顯示。
- **解鎖順序**：[m7-items.md](../2026-10-28-history-nobu-full-port/m7-items.md)，由 `v3_item_order.lua` 產生。
  - 167 件中今天能由 IR 完整載入 13 件；只有屬性加成的有 26 件。
  - 前四步是 stat 的 attack_damage（我的 ir.rs 項目）以及 int／str／agi（B 的 m3d），可再解鎖 15 件。之後是 on_created／on_removed 觸發、status.triggers、repeat。
- **測試**：新增 gen_item_v3_test（51 項）；修好一直失敗的 item_sources_check_test（70 項，E222）。放回時 codegen 與 cargo 檢查通過。

## 2026-10-29（工作線 A：條目欄位 host；已放回 live）

- 應 B 的 B04 切換要求，新增 `host = { component, reads }`，也可以寫成這種表的陣列。它宣告行為是資料時，host 元件以固定名稱從 catalog 讀的值；驗證器把 reads 算成有人用。
  - component 的清單在 v3_vocab 的 `HOST_COMPONENTS`；rust_handler 與 deferred 的條目不能寫 host；catalog 與 IR 都不帶這個欄位。
  - 規則寫在 v3.md §12.12 與 proposal §4。package_v3 新增 `host_reads` 測試，共 87 個測試通過。
- inahime：`attack_orb_attack_speed_slow_constant` 從 v2_only 改成 a02_venom_arrow 的 host attack_orb reads。`move_slow_percent` 確認沒有任何程式讀，維持 v2_only；`invisibility_level` 也維持 v2_only。
- 放回時先放驗證器，再放 inahime（prep 用的是 live 的驗證器）。重新產生的 authored_heroes 沒有差異，check_v3_content 與 lint 都通過。
- 分擔 IR 阻擋項的工作（主 session 派工）還在等 B 回覆分工範圍，還沒有改 omoba-core。

## 2026-10-29（工作線 A：法術增幅普查；已放回 live）

- **規則**寫進 v3.md §5.1，B 的規則層以這條為準：
  - `require_source_alive = false` 等於原作的 KV Damage 或 ApplyDamage，帶 ability 才吃法術增幅；
  - 原作 ApplyDamage 沒帶 ability，或攻擊者是 dummy、召喚物，要加 `flags = { "no_amplify" }`；
  - 預設等於 AMHC:Damage：檢查存活、不吃增幅。
- **普查**：11 個子 agent 逐處對原作，結果在 [spell-amp-census.md](../2026-10-28-history-nobu-full-port/spell-amp-census.md)。
  - 第 1 輪是原本寫 false 的 174 處：A 63、B 37、C 23、E 10，另有 30 列不是 damage action。沒有一處是把 AMHC 誤寫成 false。C 與 E 加上 no_amplify，B08E 燒魔在其中。
  - 第 2 輪是原本寫預設的 307 處：A 53、B 34、C 31、D 179、E 3。
    - A、B 是漏掉的增幅，改成 false，存活檢查也跟原作一致。例如 B30 的 E、R、T，B28 的 W、E、R，A19，B23，B20。
    - C 改成 false 加上 no_amplify。
  - 另外補了兩處：C08W 段藏死後的傷害加上 no_amplify；B26W 的 quirk 文字原本說「死亡時那一點傷害由規則層歸零」，改成照原作不檢查存活。
- **放回**：60 份文件經 land_to_live 放回（codegen 檢查通過）。重新產生的 authored_heroes 與 item_texts 沒有差異。
- **驗證**：
  - package_v3 共 86 個測試通過；
  - check_v3_content：604 個 IR 條目全部帶 ir 欄位載入；
  - ir-blockers 重跑仍是 33／604 能載入，不一致 0 個。
- ir.rs 的 damage 目前不接受 `flags`，所以 `action:damage.flags` 成了新的阻擋項：19 招、6 件道具、14 位英雄，M5 的 B08 也受影響。B 接上 no_amplify 時要一起放行。

## 2026-10-29（工作線 A：ir-blockers 跟上 M3 第二批 A02；已放回 live）

- **v3_ir_blockers 的鏡像清單**照 B 的 E219 同步，經 land_to_live 放回（codegen 檢查通過）。新增可載入的寫法：
  - action：heal（只能由施法者治療）、remove_status；
  - 進入點：on_toggle_on 與 on_toggle_off（必須成對，on_toggle_on 要對施法者套狀態）；
  - passive.stats；
  - 狀態欄位 states、tags、triggers，以及 `duration = "infinite"`；
  - `set_absolute` 的 stat 列；
  - 法球狀態的 triggers：runtime_id 是 `attack_orb:` 開頭，觸發只有 on_attack_hit 與 on_attack_windup。
- 可用的 state 旗標與 set_absolute stat 直接從 rules/vocab.rs 的 `STATE_EFFECTS`、`ABSOLUTE_STATS` 讀，所以會跟著程式碼變動。
- **重跑結果**：604 個 IR 條目中有 33 個能載入（上一版是 13 個），掃描與實際載入的不一致 0 個。稻姬四招全部能載入。
  - M5 下一位是 B04，要做 projectile、selector.types、dash、destroy_trees、dodge_projectiles、agi、狀態旗標 stun／ranged_attack、status.aura、一般狀態的 triggers。
  - 細節見 [ir-blockers.md](../2026-10-28-history-nobu-full-port/ir-blockers.md)。
- **inahime 的 attack_speed_mult**：B 改成直接覆寫 BAT 以後，這個值只給說明用，source 與 source_differs 的文字照這個事實改寫。放回後，重新產生的 authored_heroes.lua 沒有差異。

## 2026-10-29（工作線 A：英文說明審查的三項決定；已放回 live）

依主 session 的決定處理，經 land_to_live 放回 19 個檔案（codegen 檢查通過）。

- **B33 改名**：Voodoo Power 改成 Miasma Illusion（霧毒幻化），技能與狀態都改，並寫進用語表 §4。
- **開發用語移出說明**：有 4 處，zh_TW 與 en 都改成只描述實際效果；原因原本就寫在 quirks 或 notes。
  - a12_mystic_field：原本寫「原作腳本錯誤」。
  - 新藤五國光、久棄邪物：原本寫「原作中不存在，定義被註解」。
  - 制魔腹卷的主動效果。
  - 鞍切的「原作以層數顯示」。
- **長音符號**：羅馬字一律不加長音符號，14 份文件共 40 處，例如 Nohime、Saito、Fuma Kotaro、Meito。
- **lint 新規則**：
  - 說明的 zh_TW 或 en 出現開發用語就報錯。中文是原作、腳本、死碼、執行期、錯誤、bug；英文是 original script、original game、script、bug、errors out、at runtime、commented out。
  - 英文出現長音符號就報錯。
  - 規則寫進用語表 §2、§3。`en_glossary_lint` 測試加了對應案例。
- **驗證**：
  - live 的 lint 是 0 error，package_v3 共 86 個測試通過；
  - 重新產生的 authored_heroes.lua 與 item_texts_v3.lua 沒有差異（改到的都不是上架英雄或貨架道具）；
  - 網站重建後中英文頁都沒有長音符號與開發用語。

## 2026-10-29（工作線 A：英文說明審查；已放回 live）

- **用語表**：新增 [en-glossary.md](../../character_pipeline/presentation/en-glossary.md)，內容包括：
  - 用語對照；
  - 寫法規則（英雄技能用第三人稱、道具用 you、美式拼字、前綴、不寫內部用語）；
  - 共用名稱的決定：風林火山、忍法、銘刀、〇之卷、兵法章、雪走、般若湯。
- **檢查工具**：新增 `docs/character_pipeline/tools/v3_en_lint.lua`，機械檢查用語表的規則。package_v3 新增 `en_glossary_lint` 測試，全部文件必須沒有 error。
- **審查**：6 個子 agent 分批逐條對照 zh_TW（英雄 4 批、道具 2 批），之後主 session 統一跨文件的名稱。
  - 100 份文件有改動，只改 `text.en`；`only_en.lua` 對 base 快照比對過，其他欄位完全沒變。
  - 主要修正：誤譯與漏譯，例如「僵直」被譯成 stunned、保底機制的「連續沒有觸發」被寫成 misses in a row。
  - 統一用詞：spell immunity、`Active:`、regeneration、maximum health、增益加 `+`。
  - 去掉內部用語：nobu slow pool、resistance pool、modifier、tick。
- **放回**：經 land_to_live 放回 104 個檔案（codegen、cargo、UBT 都通過），包括重新產生的 authored_heroes.lua 與 item_texts_v3.lua，差異只在文字。
- **驗證**：
  - live 的 lint 是 0 error，剩下 5 個 warning，都是刻意的同名異譯（技能與它的狀態或單位）。
  - package_v3 共 86 個測試，以及 hero_author_v3、v3_content、site_build、site_text_render 都通過。
- **留給使用者決定**：
  - mogami_yoshihime 的 Voodoo Power（霧毒幻化）名稱意思差很遠；
  - a12_mystic_field 的中文寫了「原作腳本錯誤」，是對開發者說的話；
  - 羅馬字長音符號沒有統一（Saitō 與 Saito 並存）。
- 子 agent 共用 scratchpad 的問題記在 E215。
- **網站**：介面文字 `scripts/site/i18n.lua` 照用語表改，經 land_to_live 放回：
  - Pierces spell immunity；傷害類型 Magic；
  - Movement speed、Health／Mana regeneration、Maximum health／mana。
- **網站重建**：512 頁。英文頁沒有 Kogatana、magic immunity、未展開的佔位符或內部用語。抽查過 B27（小島貞興）、銘刀與雪走的道具頁。

## 2026-10-29（工作線 A：unit_state 打開）

- B 已放回 `IrUnitState`（omoba-content-model/src/ir.rs），所以 `M.EMIT_UNIT_STATE` 改成 true，經 land_to_live 放回（codegen 檢查通過）。測試同步改成「預設打開、`opts.unit_state = false` 關閉」。
- 驗證：
  - package_v3：85 個測試通過。
  - gen_v3_content：103 個區塊帶 unit_state，content hash 是 f554333f9bd23fa7。
  - check_v3_content：604 個 IR 條目全部帶 ir 欄位載入。
  - 重跑 ir_probe 與 ir-blockers：仍是 13／604，不一致 0 個（這一批沒有新增允許的寫法）。

## 2026-10-29（工作線 A：IR 的 unit_state 宣告；已放回 live，預設關閉）

- 應工作線 B 的要求（m3-ir-design.md §7.2），`v3_ir.lua` 可以在 IrBlock 頂層輸出 `unit_state = [{ name, type, scope, reveal, init }]`；格式記在 v3-content-model-proposal.md §4。
- **目前關閉**：`M.EMIT_UNIT_STATE = false`。B 的 ir.rs 接受這個欄位後再打開，打開後重跑 gen_v3_content。
- **全量試算**：604 個區塊中有 103 個帶 unit_state，共 140 列。
  - 宣告問題 0 個，同名但宣告不同的 0 個。
  - 沒寫 init 的 26 列，用零值。
  - `tsum`（B27、B30）是 fixed／match。
- package_v3 新增 `ir_unit_state` 測試，85 個測試全過。
- 放回時順手修了 `ir[k]` 為 nil 時取長度的錯誤（旗標關閉時才會出現）。

## 2026-10-29（工作線 A：直譯器缺口與實作優先序；已放回 live）

- **新增並放回 live**（land_to_live，codegen 檢查通過）：
  - `tools/ir_probe/`：獨立的 Cargo 專案（自帶 workspace）。讀 `target/v3-content/v3-content.json`，對每個帶 IR 的條目呼叫 `IrAbility::from_block`，逐行輸出 JSON（id、成功與否、錯誤）。
  - `docs/character_pipeline/tools/v3_ir_blockers.lua`：鏡像 ir.rs 的允許清單，stat 從 `rules/vocab.rs` 的 `STATS` 讀取；依 Program::load 的可達範圍掃描，再與 probe 結果交叉比對。
  - 產出 [ir-blockers.md](../2026-10-28-history-nobu-full-port/ir-blockers.md)。
- **結果**：604 個 IR 條目中，目前只有 13 個能載入；掃描與實際載入的不一致是 0 個。
  - M5（8 位）總共要 67 項，前幾項是 heal、toggle 事件、passive stats、`status.states`／`tags`。
  - 之後是 M6 第 1～9 批的順序，總共 210 列。
- **重跑方法**：B 每放回一批就照 ir-blockers.md §4 重跑一次（gen_v3_content → ir_probe → v3_ir_blockers）。

## 2026-10-29（E187 第三批：原作遊戲模式補齊；已放回 live）

- **已放回 live**：經 land_to_live 分五次放回，每次之後 `check_live_consistent` 都通過。細節見 errors.md E187 第三批。
  - **投降**（伺服器端）：
    - 開打 900 秒後可投，再投一次收回；門檻是 1～5 人依序 1／2／2／3／3 票。
    - 票數到就走一般的結束流程；有 API 可呼叫。
    - **還沒有輸入入口**，UI 按鈕與 `PlayerInput.SurrenderVote` 的提案寫在 E187。
  - **經驗與金錢**（只在原作規則下生效）：
    - 兵線經驗：1400 內每位英雄各自乘人數倍率；小兵金錢依兵種給最後一擊。
    - 英雄擊殺：經驗依等級差；金錢 100＋300＋連殺項，加上連殺鏈與屯錢加成；助攻 300。
    - 建築賞金：150／300／50／1300，依擊殺者是否為英雄決定全拿或分享。
    - 野怪低等補償。
    - 訓練用的鍍層、首塔、首殺、落後補償金在原作規則下關閉。
  - **兵線與建築**：
    - 中路多出兵：足輕 4、弓兵 3、鐵炮 3。
    - 兵營倒下時，該路攻方小兵：足輕、弓兵 ×1.3，鐵炮、騎兵 ×2；並在 +60、+75、+120、+135 秒追加兵力。原作沒有「三路全破」規則，第二批的解讀已更正。
    - 大將塔 intensify：+300 攻擊、+30 護甲，直到自己的第一座兵營倒下。
    - 塔的護甲 15；大將塔與主堡每秒回 20 HP。
  - **魔王**：
    - 強盜之王 180 秒出現、300 秒重生；武士亡靈 1200 秒出現、600 秒重生；每次重生 HP 與攻擊成長。
    - 數值用原作 KV，兩隻都有 `tags={"boss"}` 與 `is_boss()`。
    - 獎勵：強盜之王全隊 300 金＋200 經驗＋輪替 buff；武士亡靈全隊 1000 金＋士氣 buff。
  - **home_aura**：600 秒起，自己主堡 1300 內的英雄回血、+30 護甲、+30 魔抗；1200 內移速 600。
  - **放回後發現的 replica 分歧（已修正）**：
    - intensify 原本會改到揭露的護甲，改成只作用在 Unit。
    - 建築回血改為發送 `CommittedVitals`。
    - lifecycle soak 與兩個 replica parity soak 在 release 都通過。
- **量測**（release headless，12 seed）：
  - quick 原陣容：12/12，中位 6.3 分（4.1–10.0）。
  - quick 對調：12/12，中位 5.4 分（4.0–9.8）。
  - 標準地圖：6/6，中位 16.7 分（13.0–21.6）。
  - 第三批之前，quick 中位 8.6 分。
- **測試**（放回後的 live 快照 `target/e187/live5`）：
  - template-ids 25、content-model 84、omoba-core 1313、moba-headless 7、base_content 地圖測試 343，全部通過。
  - release soak：5 個通過。lifecycle 的 replica 分歧修正後也通過。
- **UE quick 實機**（`target/e187/ue-quick3`）：
  - 開局前確認沒有別人的局，並持 live_match 鎖。
  - 第一次開局時，另一個 session 剛好放回了 19 個檔案，啟動器的新鮮度檢查正確拒絕了舊的二進位；重新開局。
  - 04:43（tick 17039）自然結束，隊伍 2 勝；04:28 時 HUD 顯示「藍隊摧毀防禦塔」，所以雙方都推倒了塔。
  - mismatch 0；截圖中小兵面向行進方向。
- **影子**：`target/e187/t3`、`t5`、`t6`、`t7`、`t8`，基底 `b3`／`b7`／`b8`。原作規則的調查在 side-session 筆記 `e187-original-game-mode-rules.md`。
- **還沒做**：
  - 投降的輸入入口與 UI。
  - 兵營被破的中間階段（一座兵營代表兩座）。
  - 建築受傷修正與主堡在大將附近無敵。
  - 可移動大將（需要 vmap 座標）。
  - 魔王技能。
  - speed_up 的「攻擊、施法、受傷即移除」。
  - 陣容強弱一面倒（E165）。

## 2026-10-29 早上（history-nobu M3 第一批：behavior IR 直譯器，孫市與阿市改由資料執行；E212）

- **已放回 live**（land_to_live 19 檔，`--checks codegen,cargo,server,bridge`；之後 `check_live_consistent` 通過）：
  - content-model `ir.rs`（V3Block 的 `ir`）、omoba-core `runtime::native::ir`（`IrAbility`、`IrHeroUnit`）、base_content `ir_switch.rs`（B01 四招、C17 五招、阿市 UnitScript 由 IR 執行）。
  - `v3_compile.lua` 的 `M.EMIT_IR = true`；saika_magoichi、oichi 重新產生 content（authored_heroes 有 9 個 `ir` 區塊）。content hash 改變，各 run 要照常重建。
  - 原作規則修正：B01R、C17R、C17T 的 handler 帶 `REQUIRE_SOURCE_ALIVE`（AMHC：施法者已死不造成傷害）。
- **M3 驗收**：
  - 實機局 accepted inputs 重播，IR 版與 handler 版 authority 狀態每 tick 相同：live-7 66653、live-8 45895、e190-tamako/live-1 49907、m1v3/live-3 38294、items-active-1 37402 tick。
  - 腳本化施放（孫市、阿市每招輪流施放）10800 tick 每 tick 相同。
  - omoba-core lib 1298、content-model 84、base_content：B01／C17／ir_switch／腳本化比對 22 個、`replicas_identical`＋`heroes::` 176 個全過。
  - 效能（headless，seed 7，18000 tick，六次交錯）：對局走向兩邊完全相同；最佳值新版 72.2 秒、舊版 73.0 秒，沒有退步。
- **影子**：`target/m3n/work`（基底 `target/m3n/base`，manifest `target/m3n/manifest.json`）。
- **接著**：排程器（`delay`、`repeat`、狀態 `periodic`）與一般 selector，讓 host 元件的形狀辨識改成直接執行；把其餘已移植英雄逐一切到 IR。

## 2026-10-29 上午（E210 走近再施法、E211 conformance 狀態情境與強驅散修正）

- **E210 host 的走近再施法**（已放回）：
  - 超出射程的單體技能先走到射程內再施放。authority 與 team replica 讀同一份 tick 開頭的位置快照；只有 authority 結束走近，並以合成的已接受輸入（施放或 Stop）通知 replica。
  - 點技能維持原作行為（位移、直線夾到最大射程）。
  - 新增雙 replica 逐 tick soak，0 mismatch。base_content release 全集 627 個通過。e199 實機局的 accepted inputs 以新程式重播，0 mismatch。
  - conformance 的 28 個 (c) 情境轉為通過。
- **E211 harness 擴充到狀態情境**（已放回）：
  - 支援強驅散、持有者死亡、到期。產生器在 status_on_unit 加上 runtime_id 與 visibility。
  - 順手修了 `strong_dispel` 漏掉以 id 標示施加者的敵方 debuff（B29E 破甲）。
- **conformance 數字**：
  - 2289 個情境：805 通過（E207 時 560）、14 已知 (b)、1470 unsupported（E207 時 1691）。
  - 詳見 `docs/plans/2026-10-28-history-nobu-full-port/m5-conformance-baseline.md`。
- **驗證**：check_live_consistent 通過（含 bridge）；conformance＋走近 soak 12 個測試在 live 上通過；omoba-core 1314 個全過。
- **下一步**：
  - harness：effects（傷害量、治療、位移）、可見性（雙 replica）、狀態事件。
  - host：基本驅散、對 buff 的敵方淨化。
  - v3 的 direction／point 進到 content 後，再決定點技能是否走近。
- **可清理**：`target/e208-approach`（cargo 1.6G）、`target/e211-status`（cargo 1.5G）、`target/e199-oichi/fix*`。

## 2026-10-29 深夜（M7 道具呈現：單一資料來源、尚未開放的道具、商店分類與網站一致）

- **單一來源確認**：
  - 上架道具的價錢、配方與屬性，全部由 `item_shop_v3.lua`（v3 values）經 `moba_items.lua` 產生。
  - UE 的商店、合成樹、背包與說明框都讀衍生資料：bridge 商店目錄、codegen 道具資料、FOmTooltipCatalog。
  - UE 手寫 C++ 沒有寫死的道具 id、價錢或屬性。
  - 45 件上架道具都有 v3 說明與資訊列；34 件有屬性行（由 passive.stats 產生的「+20 攻擊力」等），其餘是沒有屬性的消耗品。
- **尚未開放的道具**：
  - 新增 `scripts/gen_ue_item_roadmap.lua`，產生 `omfue/Content/Omoba/UI/item-roadmap-v1.json`。原因與解鎖步驟和 `m7-items.md` 用同一套掃描。為此在 `v3_item_order.lua` 加了 `M.load_rules`，輸出不變。
  - 122 件：2 件效果已能執行、等待上架；110 件被擋（列出缺的效果與解鎖步驟）；10 件暫不移植。
  - UE 商店的「全部道具」最後加一區「尚未開放（N）」：圖示變暗，不能購買。說明框走 TooltipRenderer（屬性行與 v3 說明），最後附上原因。搜尋也找得到這些道具。
  - 測試 `Om.Runtime.ItemRoadmap`。
- **商店分類與網站一致**：
  - 新增 `scripts/gen_ue_item_categories.lua`，產生 `item-categories-v1.json`。分類的 id、順序與名稱和網站相同：v3 的 shop.category，否則原作商店檔。
  - UE 商店依這套分類分組；沒有分類的道具（例如測試用的合成目錄）退回原本依價錢與配方推算的分層。
  - 測試 `Om.Runtime.ItemCategories`：每件販售中的道具都有分類。既有的商店版面與模型測試照常通過。
- **放回 live**：3 批（道具解鎖 5 檔、分類分組 4 檔），都通過 codegen＋UBT，`check_live_consistent --ubt` 通過。
- 沒有 commit。

## 2026-10-29 晚上（M6 第 2～9 批呈現準備、缺圖清單、狀態外觀）

- **各批報告**：`m6_presentation_coverage.lua --batch 2..9` 產生 `docs/plans/2026-10-28-history-nobu-full-port/m6-batch<N>-presentation.md`。
- **缺圖清單**：`--missing-art` 產生 `m6-missing-art.md`。
  - 頭像 26、技能圖示 130、沒有原作資產的 cue 129。
  - 逐項寫明目前的替代方案：頭像與技能圖示用共用佔位圖，cue 用依 kind 的通用佔位特效。
  - 留給使用者決定；沒有下載任何外部素材。
- **第 1 批沒有出現過的呈現類型**：
  - 新增通用機制「狀態外觀」：`OmStatusLook` 加上名牌的新一行。
    - 依狀態帶的 v3 states（catalog 從 IR 的 `ir.statuses[].states` 讀出）顯示無敵、變形、嘲諷、隱身、被揭露、魔免、物免、攻擊免疫、技能無法指定、無法選取、繳械、禁言、破壞、致盲、飛行。
    - 控場（暈眩、定身、沉默、緩速）維持原本的名牌行；內部 state（碰撞、攻擊優先、視野等）不顯示。
    - 敵我單位一律依身上的狀態顯示，不為任何英雄寫特例。測試 `Om.Runtime.StatusLook`。
  - 已有、或改由前面的工作涵蓋：
    - 連鎖：預覽幾何；
    - 多段充能：技能欄充能；
    - 增減技能：技能欄依每幀的 catalog id 換圖，加上隱藏旗標；
    - 繫繩：link cue 的連線佔位；
    - 全圖範圍：預覽沿用發布的射程。
  - 需要 runtime fact，等工作線 B：
    - 真正換模型（妖術變形、變身）；
    - 隱身的半透明呈現；
    - 地形與牆（新實體類型）；
    - 持續區域；
    - 分身標記；
    - 日夜（`set_time_of_day`）；
    - 持續繫繩。
- **放回 live**：狀態外觀 9 檔（codegen＋UBT），之後 `check_live_consistent --ubt` 通過。
- **團戰壓力重測**（`target/tooltip/teamfight-4`）：
  - 視覺池峰值 50／512，被拒 0 次，Tick 平均 0.21 ms；
  - 平均 16.95 ms 一幀，沒有因視覺掉幀。
  - 細節在 E211。
- 沒有 commit。

## 2026-10-29 傍晚（狀態視覺上限量測、M6 第 1 批呈現準備；E211）

- **視覺池量測**：團戰壓力局峰值 23／512，被拒 0 次，Tick 平均 0.21 ms；畫面尖峰都不是戰鬥視覺造成。不需要修正。細節在 E211。
  - 新增 `OM_VISUAL_POOL` 計數與 `scripts/lua_data/moba_teamfight_stress.lua`。
- **M6 第 1 批**（A01、B20、B26、C22、A12、A14、A26、B27、C03）：
  - 新增 `scripts/m6_presentation_coverage.lua --batch N`，產生 `docs/plans/2026-10-28-history-nobu-full-port/m6-batch1-presentation.md`。逐位英雄列出頭像、技能圖示、cue（含投射物）、狀態說明，以及仍缺的素材。
  - 補上兩個通用機制（不為單一英雄寫特例）：
    - 狀態沒有 `runtime_id` 時，依狀態 id 對應。這和資料執行器的預設一致（runtime id = 改名表，否則 runtime_id，否則 id），新英雄的狀態上架就有名稱與說明。
    - 投射物沒有編譯樣式、但 id 是 v3 cue 時，用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）。
  - 已有的通用機制：
    - 頭像與技能圖示：nobu 原作圖，否則共用佔位，由 HUD sync 自動處理；
    - cue：依 kind 播佔位特效；
    - 換技能：技能欄狀態欄位，等 bridge；
    - 引導：公開 channel 狀態的進度；
    - 強制位移：位置插值。
  - 第 1 批 9 位的狀態全部有說明（47 個）。仍缺的素材主要是 Dota 內建圖示造成的技能圖示佔位，以及 A01、B27、C03 的頭像；完整清單在報告裡，目前都用通用替代。
- **測試**：新增 `Om.Runtime.M6PresentationFallbacks`；加上既有的狀態、cue、戰鬥視覺測試，共 9 項通過。
- 沒有 commit。

## 2026-10-29 下午（選角完成後對局沒有開始：修正啟動器的 roster 檢查與 PID 重用；E210）

- **roster 檢查**：新增 `moba_role_launch.assert_final_roster`，單人 `selection_result` 與多人 `moba_shared_selection.result` 共用。
  - 改成驗證規則，不再比對哪幾席變動：
    - 主機規則、召喚師技能與各席的身分、隊伍、控制者、路線、角色都和基準相同，只有英雄可以變；
    - 每席都有英雄，同隊不重複，選角名單沒有提供的英雄一律拒絕；
    - 人類席的英雄等於自己鎖定的席位；
    - bot 只有在「原本的英雄被同隊人類選走」時才能被改派（E164）。
  - 錯誤訊息統一以「selection changed host-owned rules or roster:」開頭。
  - 新增測試：人類選到 bot 的預設英雄（含重複、名單外英雄、未鎖定、無故改 bot、改路線等拒絕情況），以及兩位人類各選走一位 bot 的英雄。單人 15/15、多人 28/28 通過。
- **PID 重用**：選角程序結束後，同一個 PID 被受保護的程序重用，lua-host 讀映像名稱被拒，交接失敗。
  - `inspect_owned`、`stop_owned` 改成先比對建立時間，不同就是 PID 被重用。
  - `process.lua` 帶入記錄的 token。相關的 3 組程序測試全過。
- **放回 live**：啟動器與測試 4 檔、lua-host 與 process.lua 2 檔，都經 land_to_live。
- **實機**（走 live_match 鎖）：`target/tooltip/sel-live5`。
  - 自動選角選雜賀孫市（bot 3 的預設英雄）；選角服務把 bot 3 改派成阿松，啟動器接受。
  - 對局正常開始：server、runtime、UE 都啟動，跑到 tick 7142，有戰鬥傷害事件。
  - 結束碼 1 是 `--finish-timeout-seconds 120` 到時沒有分出勝負，這是設計上的有界觀察（no forced winner）。
  - 中途另被擋一次：其他 session 更新了內容，影子的 bridge 過期；重新 stage 後通過。
- 沒有 commit。

## 2026-10-29 中午（history-nobu M6 前置，工作線 C：向量施法輸入與預覽、路徑與彈跳預覽、選角依 catalog 自動列出）

- **目標規格從 catalog 來**：
  - `FOmTooltipCatalog` 讀 IR 的 `target`：type，以及 range、min_range、aoe_radius、vector_length 對應的數值。只接受「單一 slot」的運算式，其餘當作未知。
  - `OmCastTargeting::Resolve` 依等級給出數值。
  - 不為個別英雄寫特判。目前上架的技能還沒有 vector；服部半藏上架後會自動生效。
- **向量施法（兩點）**：
  - `FOmCombatAimModel`：context 有向量長度的地點技能一律武裝（快速施法也一樣）。第一下定起點並繼續瞄準，第二下送出：起點，加上沿點擊方向延伸一個 `vector_length` 的終點。點在起點上不算確認；Esc、右鍵或狀態改變會連同起點一起取消。
  - 輸入元件畫出起點與整條向量帶，提示文字分兩段。
  - 送出時在本地 fail closed：協定還沒有第二點，絕不改成普通地點施法送出。
- **路徑與連鎖彈跳**：預覽幾何 `FOmPathPreview::Build`（每段寬帶）、`ChainRings`（每跳的搜尋範圍）完成；runtime 呈現等工作線 B 的 `path` 與 `ProjectileBounce`。
- **協定提案**：寫在 `m6-ue-presentation-contract.md` §2.1。
  - `CastAbility.target_pos_end`；
  - bridge `OmInputCommand.world_point_end` 與新旗標；
  - 錯誤碼 `vector_direction_missing`。
  - 待 B 同意後再升 CABI。
- **選角**：
  - 名單本來就由 compiled catalog 的 `HERO_PLAYABLE_IDS` 決定（Rust 選角服務），UE 端沒有白名單；手寫 C++ 裡也沒有任何寫死的英雄 id，只有一段註解提到。
  - 這次加上頭像：每列從 HUD registry 取頭像（人工美術、nobu 原作或共用佔位，HUD sync 會自動補），說明從 catalog 取。
  - M6 每開放一批英雄，只要內容編譯與 HUD sync 跑過，選角就會出現，UE 不必改程式。
- **測試**：`Om.Runtime.CastTargetingCatalog`、`VectorCastPreview`、`VectorCastAim`、`HeroSelectionRoster`，加上既有的瞄準模型、預覽與選角測試，共 36 項通過。
- **實機補驗（720p，走 live_match 鎖）**：截圖在 `target/tooltip/evidence/m6-check/`。
  - 選角（`run_moba_role_ue --interactive-selection --selection-smoke-hero`）：10 位英雄都有正確頭像，右側說明欄完整、沒有裁切。截圖背景的棋盤格是截圖的透明 alpha，畫面背後沒有場景，不是畫面問題。
    - 為了取證，自動選角在點選前會把 `hero-selection.png` 存在選角 log 同目錄。這段程式已放回 live。
  - 技能欄：雜賀眾（W）顯示充能「2」；技能提示框沒有裁切。
  - 商店：各分類道具都是原作圖，沒有文字退路。
  - 實機看不到、改由自動化測試涵蓋：
    - 開關、重施放等徽章：runtime 還沒有 fact。
    - 向量施法的 fail-closed 提示：沒有上架英雄使用向量施法。提示文字與不送出的行為由 `VectorCastAim` 與元件程式驗證。
  - 發現一個與本次改動無關的問題（後續項目）：選角確定後，`moba_role_launch.selection_result` 斷言「selection changed host-owned match rules or roster」，對局沒有開始。
    - 原因：人類選了 bot 原本的英雄（雜賀孫市），選角服務依 Rust 規則把 bot 改派成阿松；啟動器的預期只允許人類那一席變動。
    - 第一次嘗試另外遇到一次「cannot query exact process identity: 存取被拒」，重跑就沒有再出現。
- **流程失誤**：在影子有未放回的修改時執行 `stage_ue_shadow --sync`，live 原始碼覆蓋了影子的修改，第一次選角截圖因此沒有拍到。之後有未放回的修改時只用 `--build`。
- 沒有 commit。

## 2026-10-29 上午（history-nobu M6 前置，工作線 C：原作 nobu 圖示、商店道具圖示、bridge 商店測試；E209）

- **bridge 商店測試**：45 對 64 是測試過期，不是上架清單不一致。bridge 依 E155 只把 `in_shop()` 的道具送進商店，測試卻拿完整 catalog 比對；另一個假設「恰好一件配方有重複材料」也過期（現在有兩件）。兩個測試改成與產品程式同一個篩選；bridge 測試 149＋2 全過。
- **原作圖示匯入**：`scripts/import_nobu_icons.lua`（lua-host 新指令 `png_square_icon`：置中裁正方形、縮成 128px）。
  - id 依 v3 文件：英雄是 package id，技能是 v3 id，道具是遊戲 id。
  - 來源：
    - 技能與道具：KV 的 AbilityTextureName 指向 addon `custom/` 資料夾的檔案；
    - 頭像：nobugg 官網經審查確認為 nobu 自製的英雄圖（52 張），其次是 addon 的 panorama 覆寫圖。
  - 一律用佔位並列清單的情況：
    - 官網的 46 張 Dota 2 肖像（逐張審查，結果記在 `docs/character_pipeline/presentation/nobu-icon-overrides.lua`）；
    - Dota 內建圖示、`spellicons/dota2/` 的複製品；
    - （已解決）17 個 JPEG 檔：lua-host 加入 `jpeg-decoder` =0.3.2（crates.io，image-rs 維護，純 Rust，不帶預設功能、沒有其他相依；Cargo.lock checksum 00810f1d8b74be64b13dbf3db89ac67740615d6c891f0e7b6179326533011a07）。依檔頭判斷 JPEG，不看副檔名。
  - 結果：英雄 68／97、技能 303／446、道具 156／176 有原作圖。清單在 `docs/character_pipeline/presentation/nobu-icons.md`。
- **UE**：
  - HUD sync 缺圖時優先用 nobu 圖，其次才是共用佔位。人工指定的列一律優先，不會被取代。
  - registry 接受原始 PNG 的道具列，上限從 512 列提高到 2048 列。
  - 商店販售的 45 件原本全部沒有圖示（manifest 的 19 張都是測試道具），現在 45 件全部有原作圖。
  - 未上架英雄上架時，sync 會自動帶入他們的頭像與技能圖。
  - 測試 `Om.Runtime.NobuIconRows`（45 列都解碼成 128×128）、`HudAssetRegistry` 新增原始 PNG 道具列的斷言；Lua 測試 `scripts/tests/nobu_icons_test.lua`（41 項）。
- **網站**：`site/ue_icons.lua` 讓網站顯示與 HUD 相同的圖（人工美術 → nobu 原作 → 佔位），建置時縮成 128px，舊圖不殘留。525 張圖。
- **判定前提**：addon `custom/` 資料夾的檔案視為 nobu 自製；無法逐張追出處的，可在覆寫檔逐筆改成佔位。
- **實機**：
  - `target/tooltip/icons-1280x720`、`icons-1920x1080`（雜賀孫市對阿市，autopilot，走 live_match 鎖）：商店各分類的道具都顯示原作圖，只有 JPEG 那兩件仍是文字退路。截圖在 `target/tooltip/evidence/icons/`。
  - 1080p 第一次被啟動檢查擋下兩次：其他 session 先後更新了 core 與 buffs.lua，client runtime 與影子的 bridge 因此過期。重建、重新 stage 之後才通過。
- **放回 live**：
  - UE 兩批：registry、道具圖解析、測試 3 檔；registry 列舉介面與 `NobuIconRows` 2 檔。最後 `check_live_consistent --ubt` 通過。
  - bridge 測試 1 檔。
  - 直接寫入：lua-host `png_square_icon`（先在副本跑完 32 項測試）、工具與 sync 的 Lua，以及 live UI 目錄的 510 張 nobu 圖與 index。HUD manifest 由 sync 更新為 190 列。
- 沒有 commit。

## 2026-10-29 早上（history-nobu M6 前置，工作線 C：動態技能欄、cue 佔位特效；E208）

- **對齊文件**：`docs/plans/2026-10-28-history-nobu-full-port/m6-ue-presentation-contract.md`，列出 UE 需要的 runtime fact、建議欄位、誰做，已通知工作線 B（m2-rules-layer）。
- **充能（第 1 項）**：bridge C ABI 39 → 40，`OmAbilityHud` 加充能 4 個欄位（含所有寫死版本的 assert 與 checked-in `om_bridge.h`）。UE 技能格左下角顯示剩餘次數，tooltip 顯示「次數：1/2，下一次 X 秒」。測試 `Om.Runtime.AbilityChargeHud`。
- **其餘技能欄狀態（第 1 項，UE 端）**：`FOmAbilityHudStatePayload` 新增開關、自動施放、重施放窗口、限時換技能、`SlotFlags`（隱藏子技能）。技能格左上角徽章（再 X／換／自動／開），tooltip 各一行；不一致的值一律丟掉；隱藏旗標對五格都有效。測試 `Om.Runtime.AbilitySlotStateHud`。runtime fact 等工作線 B，之後接 bridge ABI 41。
- **cue 佔位特效（第 3 項）**：
  - `scripts/gen_ue_cue_table.lua` 把 cue 目錄編成 `OmCueCatalogData.inl`（1080 cue、852 原作資產）；`scripts/tests/ue_cue_table_test.lua` 檢查是否過期。
  - `OmCue::Resolve`／`CueForOriginalAsset`；Editor 在 Project Settings > Om Presentation Cues 覆寫（cue → 特效、音效、kind、顏色；原作路徑 → cue）。
  - 單位收到帶目錄 cue id 的 ability event 時播指定特效，沒有就播 kind 的通用佔位（環、圓盤、連線；sound 不畫）。測試 `Om.Runtime.CueCatalog`、`Om.Runtime.CueCatalogPresentation`。
- **向量施法、路徑投射物、彈跳（第 2 項）**：需要協定與執行器欄位，等工作線 B。
- **實機**：`target/tooltip/m6-1280x720`、`m6-1920x1080`（雜賀孫市對阿市，autopilot，每局 240 秒，走 live_match 鎖）。兩種解析度下，雜賀眾（W）的技能格左下角都顯示剩餘次數「2」。截圖在 `target/tooltip/evidence/m6/`。autopilot 懸停的是 Q，所以 tooltip 截圖沒有拍到「次數」那一行，這一行由 `AbilityChargeHud` 驗證。開關、重施放、cue 佔位在 runtime 送出 fact 之前沒有實機畫面。
- **放回 live**：3 批（充能 6 檔、cue 5 檔、技能欄狀態 5 檔），各自通過 codegen＋UBT；最後 `check_live_consistent --ubt` 通過。影子 77 項 HUD／呈現相關測試全過；bridge cargo 測試 147 過，剩 2 個既有的商店數量失敗（45／64，與本次無關）。
- 沒有 commit。

## 2026-10-29（E207：已移植 10 位英雄的 M5 conformance 基準）

- **已放回 live**：
  - `scripts/base_content/src/v3_conformance_tests.rs`：資料驅動，每位英雄一個 release-only 測試，10 位約 15 秒。
  - `scripts/test_fixtures/v3_conformance/known_differences.json`。
  - 產生器修正：有 charges 的技能不產生 cooldown 情境。
  - `hero_author/shadow.lua` 與它的涵蓋測試（影子樹加入 packages 與 `_bootstrap.lua`）。
- **結果**：2289 個情境中 560 通過、38 已知差異、1691 unsupported。
  - 已知差異：(b) 10 個，A07 沒有 `a07_rouse_free`；(c) 28 個，引擎沒有「超出射程先走近再施放」。
  - 報告在 `docs/plans/2026-10-28-history-nobu-full-port/m5-conformance-baseline.md`。
- **驗證**：live 上 conformance 11 個測試、hero_author 14 個測試通過；v3_scenarios 105 份文件無錯誤；check_live_consistent 通過。
- **下一步（M5）**：
  - 補 effects、status、dispel 的檢查（census 的 effect 類差異會在那時出現）。
  - 做 host 的走近施法指令。
  - 讓 known 清單清空。
- **可清理**：`target/e203-conformance/cargo`、`cargo-live`（各約 0.8G）、`shadow`、`land`。`target/v3-conformance` 的報告可以保留。

## 2026-10-29 清晨（history-nobu M1 工作線 C 收尾、M8 全量網站；E201 回歸測試、E204）

- **M1 工作線 C 收尾**：
  - **接線狀態**：
    - 技能欄、商店（購買卡、背包、詳細欄、道具欄）：有 v3 時改用 catalog 文字，不再讀舊的 description。
    - 選角：新增可捲動的說明欄，顯示英雄各招的 v3 說明。選角時由 bridge subsystem 的 `EnsureBridgeLoaded()` 載入 catalog。
    - 狀態圖示：已接（工作線 A 補上 `runtime_id` 與說明之後）。狀態列表依 BuffStore id 顯示 v3 名稱與說明；放回 4 個檔，`StatusTooltipFromCatalog` 等 80 項測試通過。
    - 召喚師技能、守衛：catalog 沒有它們的 v3 資料，暫不接。
  - **1080p**：提示框上限改為隨字級放大（`SOmHudToolTip::ScaledLimit`，1080p 為 611×764），長說明不再落到可視範圍外。
  - **實機截圖**（autopilot 截圖，阿市對雜賀孫市，都走 live_match 鎖）：
    - 720p：`target/tooltip/res-1280x720`；
    - 1080p：`target/tooltip/res-1920x1080b`；
    - 整理在 `target/tooltip/evidence/`（720p／1080p 各有技能兩張、商店一張，另有選角說明欄）。
    - 說明都是繁中、目前等級加粗、傷害類型有顏色、沒有截掉。
  - **E201 回歸測試**：
    - `scripts/tests/ue_bridge_ownership_test.lua`：遊戲模組裡只有 bridge subsystem 可以載入、釋放 DLL；
    - `Om.Runtime.TooltipBridgeCatalog`：讀完 catalog 後，runtime 的 bridge handle 不變，bridge 仍能回應。
  - **放回 live**：選角與 catalog 6 個檔、提示框上限 3 個檔、測試 1 個檔（每批都通過 codegen＋UBT）。
- **技能 tooltip 整併成一個區塊**（主 session 核對 720p 截圖後提出）：
  - 上方是 v3 說明與資訊列（catalog values；即時冷卻、耗魔和設計值不同時附「（目前 x）」）。
  - 下方只留 runtime 狀態與操作：目標、冷卻中的剩餘時間（只在冷卻時顯示）、施法取消規則、不含數字的形狀說明、升級提示、操作提示。
  - 拿掉的內容：重複的技能名稱、除錯用的「L1 CD 0s」、和資訊列矛盾的「目前魔力消耗未提供」、重複的「冷卻：剩餘／總長」與「基礎施法前搖」數值行。
  - `PointBlastMetadata` 測試改成接受 v3 資訊列的「範圍」。受影響的 75 項 UE 測試全部通過，放回 2 個檔。
  - 新截圖：`target/tooltip/evidence/v2/`（720p、1080p 各有阿市、雜賀孫市、商店）。
- **M8 全量網站**：
  - **預設建置**：`tools\lua\lua.exe scripts\build_site.lua` 會在同一個行程裡產生全量來源目錄（`gen_source_catalog`），並匯出遊戲的 V3_CATALOG_JSON（`--game-catalog FILE` 可以直接指定檔案，`--no-game-catalog` 只用來源目錄）。
  - **結果**：512 頁，97 位英雄、156 件道具，473 張圖示，zh-TW 與 en。
    - 已上架的 10 位英雄改用遊戲現行數值，頁面註明；
    - 未上架的英雄標「未上架」，首頁分成「已上架」「未上架」兩組；
    - 20 件暫不移植的道具在商店頁另外列出原因；
    - 頁尾顯示遊戲的 content hash 和來源目錄的 hash；
    - 改版說明在沒給 `--previous` 時，自動比對上一次建置留下的 catalog.json。
  - **缺值**：來源目錄原本有 4 個被說明引用、卻沒有匯出的值，頁面顯示「?」並列入建置警告。工作線 A 已在來源目錄新增 `owners`（文件層級的值改名為 `<文件>.shared`，另有單位模板與狀態），網站改讀它，現在 0 個缺值。
  - **配方與分類**：來源目錄已加入道具配方，網站有 89 件道具顯示合成樹；不在目錄裡的零件（例如 item_samurais_badge）以 id 顯示。分類仍從原作 nobu_shops.txt 補，v3 文件有寫 category 時優先採用。
  - **一度失敗的測試**：`site_build_test` 的 catalog.json 來回比對曾經失敗。原因是我新增的「改版說明自動比對上一次建置」讓輸出資料夾裡的舊 catalog.json 跑進了比對；測試已改成關掉這個功能。改版說明遇到沒有名稱的擁有者時，id 改放在 `<code>` 裡顯示。
  - **測試**：site_build_test 2,315 項（新增 fixture `scripts/tests/fixtures/source-catalog-sample.json`）、site_text_render_test 105 項，全部通過。網站檔案不是 live 輸入，直接寫入。
- 沒有 commit。

## 2026-10-29（E187 第二批：原作剩下的規則、bot 推進、quick 達標；已放回 live）

- **已放回 live**：分四次 land_to_live，最後一次之後 `check_live_consistent` 通過。
  - 原作規則（數值都在 `moba_maps.lua`）：
    - 外塔每 45 秒 +4 攻擊；
    - 騎兵：第 6 波起每 5 波，1800 秒後每 2 波；
    - 小兵每 300 秒 +1 護甲；
    - 兵營被破後該路小兵 ×1.3，全破後鐵炮與騎兵 ×2；超級兵換兵規則關閉。
  - 建築護甲：M2 之後傷害讀的是 `Unit.base_armor`，建築的護甲實際上是 0，已補上。
  - bot：
    - 自己的前搖中不再下新命令。原本 0.2 秒的前搖每次 think 都被重下打斷，honda、omatsu 整局卡在原地。
    - 判斷建築危險改用該建築的實際射程；兵營當作不會攻擊的目標。
  - headless 診斷取樣加入兵營與大將塔。
  - land_to_live 的寫入改為全有或全無。這次一批 15 檔寫到一半失敗，live 約 40 分鐘不能編譯，重跑後補齊。
  - 測試配合調整；lifecycle soak 改用 quick 地圖，因為標準地圖現在是原作的半小時局。
- **量測**（release headless，12 seed）：

  | 設定 | 20 分鐘內結束 | 局長中位數 | 雙方推塔 |
  |---|---|---|---|
  | quick 正式 recipe | 24/24（最長 13.8 分） | 7.5、8.1 分 | 23/24 局輸方也推倒了塔 |
  | 兩隊陣容相同（勝負 6：5、4：8） | 23/24 | 14.0、11.0 分 | 雙方都推得掉塔 |
  | 標準地圖（原作數值，6 seed） | 6/6 | 28.4 分（23.2–34.4） | — |

  - 勝方跟著陣容強弱走，屬於 E165。
- **測試**（放回後的 live 快照 `target/e187/live3`）：
  - template-ids 20、omoba-core 1290、moba-headless 7、base_content map 測試 342 通過；release soak 6 個通過。
  - omb lib 只剩既有的 E164 fixture 失敗。
- **UE quick 實機**（`target/e187/ue-quick1`）：
  - 開局前用 tasklist 確認沒有別人的局，並持 live_match 鎖。
  - 單人座位由 autopilot 操作，其餘 9 個 bot。
  - 07:32（tick 27123）自然結束，隊伍 2 勝；mismatch 0；截圖中小兵面向行進方向。
- **影子**：`target/e187/t2`（基底 `target/e187/b2`），量測結果在 `target/e187/runs/`。
- **還沒解決**：
  - 陣容相同、實力接近時局長中位 11–14 分。
  - 原作的中路加兵、兵營被破後追加四波、建築回血還沒做。

## 2026-10-29 清晨（阿市光鍊 mismatch 結案；lifecycle 測試現況；E202）

- **阿市對玉子重跑**：`target/e199-oichi/live-3`，quick 地圖，持 live_match 鎖、沒有其他實機局。自然結束，約 37400 tick。
  - observer mismatch 0，validation 中斷 0。光鍊有 16 次 handler 拒絕，但沒有分歧。
  - 結論寫進 E190 的未解決段，已結案：先前的分歧只在兩組實機局同時跑時出現，推測是 observer 在負載下的時序差。
  - verdict 是 FAIL，但原因只有玉子的大招 `c15_hundred_arrows` 整場沒被自動駕駛施放（覆蓋率問題），與 parity 無關。
- **重跑途中的兩件事（E202）**：
  - split 的 client 在開局途中被 E190 的檢查判過期：修正為子啟動器以開局時刻（`OMOBA_SOURCES_AS_OF`）為準，已放回。
  - live-1 的兩個 Unreal 同時消失，原因是 E201 的 bridge 卸載 bug：影子 UE 建於有問題的時段。live-3 用 01:41 以後重建的影子，不受影響。
- **`three_lane_60hz_full_filtered_match_lifecycle`（release，目前 live）**：
  - lumen_bolt 冷卻分歧已由 E196 修好，雙方雜湊一致到 tick 72001。
  - 測試仍會失敗，原因是 20 分鐘遊戲時間內打不完：塔剩 16／18，推塔 bot 死了 32 次。這是 E187 原作塔數值下 fixture 推不動，不是 replica 問題，需由 E187 調整 fixture 或測試上限。
- **清理清單**：見下方「2026-10-29（E196…）」段的表格，已補上 e194-reuse、e197-lock、e199-oichi 的項目，可刪合計約 184G。`e190-tamako/rel` 與 `ue-shadow` 現在已不再使用，可以刪。

## 2026-10-29（history-nobu 工作線 A：M3 放回後的全量 IR 載入確認）

- B 已放回 M3 第一批（E212）：ir.rs、直譯器，EMIT_IR=true。工作線 A 依約跑 `scripts/check_v3_content.lua`：615 個區塊全部通過，604 個 IR 都連同 `ir` 欄位被 content-model 載入（先前為 0）。
- package_v3（84 項）、hero_author_v3、v3_content、nobu_map、網站測試都通過；item_texts_v3、nobu_map 的 `--check` 都是最新。

## 2026-10-29 凌晨（history-nobu 工作線 A：nobu 地圖 template 與 9 項引擎能力規格）

- **template**：`tools\lua\lua.exe scripts\gen_nobu_map_content.lua`（`--check` 確認檔案是最新的）從 `docs/character_pipeline/maps/nobu.lua` 產生 `scripts/lua_data/templates/nobu_map.lua`。
  - 內容依兩隊整理：路線、塔、兵營、大將塔、主堡、泉水、出生點；另有 24 營、魔王、商店、1,564 棵樹，以及 gridnav 合併成的 305 個地形矩形。
  - 還沒被 templates.lua include，也沒有加進 moba_maps，遊戲內容不變。
- **驗證**（產生時執行，任何一項失敗就停）：
  - 每方三路的路徑點都在可通行格，並在 gridnav 上連通；
  - 每條路從己方主堡出發、到敵方主堡；
  - 每座塔、兵營都在自己隊伍的同名路線上；
  - 位置都在可玩範圍內；
  - 數量與 nobu.lua 一致；
  - 地形矩形面積剛好等於不可通行格。
- **發現**：原作的 world_bounds 沒有涵蓋聯軍方的泉水、出生點與 3 個營地。可玩範圍改用地形網格（約 17,920 見方，現行的 3.7 倍）；world_bounds 很可能是鏡頭範圍，原樣保留為資料。比較報告已更正。
- **交辦規格**：寫在 `nobu-map-comparison.md` §5，共 9 項，每項列出現況、要改的模組（lua_content.rs／MobaMapConst、moba_match.rs、combat_acquisition.rs、base_structures.rs、jungle.rs、moba_map_layout.rs、vision_occluder.rs、visibility.rs／fog_grid.rs、UE minimap）、資料欄位與驗收方式。
- **測試**：nobu_map_test 4 項通過，新增 template 驗證，以及「塔不在路線上」「缺一條路」兩個故意改壞的案例。放回時 codegen 與 cargo 檢查都通過。

## 2026-10-29 凌晨（history-nobu 工作線 A：cue kind、owner 規則）

- 依工作線 C 的請求，`v3_cues.lua` 為每個 cue 推導 `kind`，用途不只一種時加列 `kinds`，供 UE 依類型選通用佔位特效。每個 use 記錄掛在哪個動作、欄位、事件或狀態，where 改成可讀的描述。
  - 推導規則依序：狀態 → aura；動作的 cue 欄位 → projectile、area、link、summon；單位模板 → summon；名稱含 gesture → gesture；on_channel_* → channel；on_hit → impact；其餘依資產分成 sound 或 effect。
  - 分布：effect 377、aura 284、sound 271、projectile 78、impact 32、gesture 27、channel 5、area 3、summon 2、link 1。原作資產的對應仍是 910 個。
- 網站與 UE 回報：狀態說明 pieces 的 owner 用 v3 id，catalog 的道具用遊戲 id。決定維持「owner 一律是 v3 id」，規則寫進 v3-content-model-proposal.md §2.2；道具遊戲 id 等於 v3 id 去掉 `item_`，45 件全部成立。
- 另一個 session 依 E207 改了 v3_scenarios：有充能的技能不產生冷卻情境。package_v3_test 的情境測試跟著改條件，84 項通過。

## 2026-10-29 凌晨（history-nobu 工作線 A：原作地圖 nobu.vmap 解析）

- **解析器**：
  - `scripts/nobu_map/dmx.lua`：唯讀的 binary DMX 9 解析器，Lua 5.4 `string.unpack`，不需要 lua-host。nobu.vmap 的 4,449 個元素在 1.9 秒內讀完，剛好用完每個位元組。
  - `scripts/nobu_map/world.lua`：逐層累積節點變換，展開 156 個 prefab 擺放（38 份 prefab vmap，含巢狀）。2 個泉水 prefab 是 Dota 內建資產，只記錄位置。
- **資料**：`tools\lua\lua.exe scripts\gen_nobu_map.lua` → `docs/character_pipeline/maps/nobu.lua`（每筆附 `<vmap>#<nodeID>` 出處）與 `nobu.md`。換算係數集中在 `world_units_per_source_unit = 1`。
  - 地圖邊界、6 條有方向的出兵路徑（6～9 點）。
  - 塔 32 座：每方 16 座，邊路 3 層 × 2、中路 2 層 × 2。原作的塔名稱重複，層級由離己方主堡的距離推得。
  - 兵營 12 座（每路 2 座）、主堡 2、大將塔 10、泉水 2、商店 6、出生點 16。
  - 野怪 24 營（68 隻、14 種）、魔王 2、基地單位 24。
  - 地形網格：70×70 tile × 256；頂點 cliff 等級、水面 240 個頂點、樹木 1,564 棵、植物 1,168 個點、gridnav 不可通行 4,629 格。
- **無法確定的部分**（已寫明，沒有猜）：
  - cliff 一級的高度不在地圖裡，慣例 128 為推定。
  - 網格索引方向由資料推定（0／242 對 5／242）。
  - 原作沒有草叢：vmap 與腳本都沒有。
  - 每方 12 個 com_soldiercamp 基地單位的用途不明。
  - 3 處商店在 world_bounds 外。
- **UE 底稿**：`--heightmap target\maps` 產生 253×253 的 16-bit `.r16` 與說明檔，可以直接匯入 Landscape；這是產生檔，不提交。
- **比較與方案**：`docs/plans/2026-10-28-history-nobu-full-port/nobu-map-comparison.md`。
  - 原作範圍約 15,360 × 14,848，邊長是現行的 3.2 倍；邊路長約 21,000，是現行的 2.3 倍。
  - 原作每方塔 16 座（現行 9）、兵營 6 座（現行 3）、24 營（現行 16）。
  - 方案：新增 `nobu` 地圖，資料經產生器進 lua_data，由遊戲模式 session 接上 `moba_maps.lua`。引擎要補的能力有 9 項，例如每方各自的路線、明確的塔座標、樹木實體、gridnav 碰撞、地圖尺寸。沒有改 live 的地圖檔。
- **測試**：`scripts/tests/nobu_map_test.lua` 3 項通過：合成 DMX、尾端多餘位元組、原作數量。放回時 codegen 檢查通過，輸出可重現。

## 2026-10-29 凌晨（history-nobu 工作線 A：M6 批次規劃、bot 施放資料，E206）

- **M6 批次規劃**：
  - 新工具 `docs/character_pipeline/tools/v3_waves.lua` → `docs/plans/2026-10-28-history-nobu-full-port/m6-waves.md`。
  - 基準是上架 10 位英雄需要的 147 個元件（M3＋M5）。其餘 87 位依「每多一個元件解鎖最多英雄」分成 9 批：
    - 第 1 批：9 位，只需 4 個新元件。
    - 第 5 批：13 位，因為同一組元件會一次解鎖這些英雄。
    - 第 9 批：11 位，33 個新元件。
  - 每批列出新元件、英雄機制特色（召喚、分身、位移、光環、偷技能……），以及首次出現機制的驗收重點。
  - 已發給 B，機器可讀版放在共用筆記。
- **bot 施放資料**：
  - v3 新增技能與道具 active 的 `bot`，形狀同 omoba-core BotAbilityIntent。沒寫時由 target、cast、cost、行為推導（`v3_bot.lua`）。
  - catalog 的 V3Block 新增 `bot`（Rust 欄位已加）。
  - 553 個條目中，已支援種類 467、提案種類 79、不施放 87（被動與子技能）。
  - moba_role_match.lua 的 18 條手寫 policy 全部一致（6 條照實機門檻手寫在文件裡）。
  - Rust 部分只寫提案與介面：`m6-bot-intents.md` §3 交給 B（bot 預設讀 catalog、M6 英雄改讀 V3Block、cone／line 幾何、4 個新 intent、法力保留），§4 給 UE autopilot。
- **放回**：分兩段，先放回工具與 Rust 欄位，再放回 3 份文件並重建全部上架英雄。
  - codegen、cargo、server、bridge 檢查都通過。
  - package_v3 84 項、網站 2,764 項、v3_content 測試、content-model 載入與 gen_item_v3 --check 都通過。
  - 放回準備第一次只重建了改過的 package，這個問題記為 E206。

## 2026-10-29 凌晨（history-nobu 工作線 A：狀態說明與 runtime buff id，E204／E205）

- **schema v3.3**（v3.md §12.12）：
  - 狀態新增 `runtime_id`：現行 BuffStore 的 id，用 `{caster}` 代表 `id:gen`、`{rank}` 代表 rank；道具用去掉 `item_` 的遊戲 id。
  - catalog 的 `statuses[].runtime_id` 與 IR 都會帶上；Rust 的 V3Status 已加欄位。
  - 驗證器規定 buff／debuff 狀態必須有中英文 name 與 body。
- **資料**：6 個子 agent 分批補寫。
  - 上架 10 位英雄：52 個狀態，runtime_id 已寫（含 2 個帶 `{rank}` 的）。
  - 上架道具：27 個狀態中 20 個寫了 runtime_id。沒寫的是唯一移速這類不進 BuffStore 的狀態，以及多個狀態合併成同一個 buff 的情況。
  - 87 位未上架英雄共 395 個狀態、全部道具其餘的 137 個狀態都補了說明。105 份文件的 592 個可見狀態現在都有說明，文件中共 63 個 runtime_id。
  - 遊戲內容：77 個狀態中 63 個帶 runtime_id，59 個可見狀態都有說明。
  - 依實際效果改正 4 個 visibility（E205）。
- **放回**：分三段，避免 live 出現驗證失敗的空窗。
  1. 版號 3.3 與 `{rank}` 支援。
  2. 105 份文件，加上重新產生的 authored_heroes、item_texts_v3（107 個檔）。重新產生的內容中，說明、數值與 extras 都沒有變。
  3. 驗證規則與測試。
  - codegen、cargo、server、bridge 檢查都通過。package_v3 83 項、網站 2,762 項、v3_content 測試，以及 content-model 載入（615 個區塊）都通過。
- **其他確認**：網站讀 owners 之後缺值為 0，89 件道具有合成樹。site_build_test 第 138 行的失敗是網站 session 自己造成的，已由它修好。
- **待 B 確認**：87 位未上架英雄沒有 runtime_id，因為 runtime 目前沒有對應的 buff。M5／M6 改用資料執行時，BuffStore id 是否直接用 v3 狀態 id，已詢問 B。

## 2026-10-29 凌晨（history-nobu 工作線 A：6 筆 pending 依 D13 解開、全量內容管線）

- **pending 全部解開**：依原作腳本、KV 與 Dota 公開語意推定，confidence 都是 medium，理由寫在 quirk。
  - A13 換位之術：KV 沒寫施法距離，Dota GetCastRange 取 0，所以 cast_range 0（assumed）。另列 fix_candidate FC-132。
  - A15T：DAMAGE_TYPE_NONE 的 ApplyDamage 不扣血，飛標不造成傷害。
  - A19T：Dota 重套 modifier 只刷新、不重跑 OnCreated，刪掉 on_refreshed 的傷害，結果與說明的每秒 4 × dmg 相符。
  - A21R：光環子狀態依光環預設殘留 0.5 秒移除，Duration 5 不生效。
  - A32F：切換技死亡不關閉、modifier 死亡移除，資料照原作。另列 FC-133。
  - item_ninja1：用 unit_state 的兩個編號槽位、較舊槽位、下忍自己的編號，加上時限狀態與 on_death，照原作逐行表達。同編號清空的副作用另列 FC-134。
  - fix-candidates.md 增為 134 項。
  - coverage 現況：expressible 526、rust_handler 79、deferred 10，pending 與 needs_hook 都是 0。usage-stats 顯示 97 位英雄、156 件道具全部可解鎖。
- **全量內容管線（M6 前置）**：
  - `scripts/gen_v3_content.lua` 把全部 v3 文件編成 `target/v3-content/v3-content.json`：97 位英雄（10 listed、87 unlisted，不進選角與 roster）、448 招技能、167 件道具，604 個條目帶 IR。rust_handler 條目也附資料行為的 IR。
  - `scripts/check_v3_content.lua` 會跑新的 `omoba-content-model/tests/v3_full_content.rs`：615 個區塊都能以 V3Block 載入並通過 validate_block。
  - 目前 V3Block 沒有 `ir` 欄位，測試會先拆下 IR、只計數；ir.rs 放回後會自動連同 IR 一起載入，已告知 B。
  - Lua 單元測試 `scripts/tests/v3_content_test.lua` 3 項通過。
  - 遊戲內容不變，gen_item_v3 --check 通過。

## 2026-10-29 凌晨（history-nobu 工作線 A：來源目錄匯出共用值與模板值）

- 網站回報有 4 個說明引用的值不在來源目錄裡：文件層級的 `shared` 值（跨英雄同名）與單位模板的值。
- 修正：
  - `v3_compile` 多輸出 `owner_values`，涵蓋文件的 values、單位模板與文件層級狀態；遊戲內容不讀這個欄位，也沒有改變。
  - `gen_source_catalog` 新增頂層 `owners`，共 47 個。文件層級的 owner id 改成 `<文件 id>.shared`，區塊內的引用一起改名，owner id 全目錄唯一。
- 檢查結果：所有指定了 owner 的 piece 都找得到值（修正前 4 個找不到，修正後 0 個）。目錄 hash 939d22252f4f0856。
- live 的 site_build_test 第 138 行（catalog.json 重新渲染出相同頁面）目前失敗。換回改前的 v3_compile 重跑也一樣失敗，這個案例也不讀來源目錄；網站 session 在 02:12～02:15 正在改網站程式，已請它確認。

## 2026-10-29 凌晨（history-nobu 工作線 A：來源目錄補上道具配方）

- 依網站 session 的請求，`gen_source_catalog.lua` 的道具新增 `recipe = { components, recipe_cost }`（取自 v3 的 shop.recipe，`recipe_cost` 是 value 名稱），以及文件有寫時的 `category`。
- 166 件中 90 件有配方；7 個零件指向目錄外的道具（item_insight_gem、item_samurais_badge）。
- category 只有 13 件（secret_shop）。原作分頁有一件多頁的情況，要寫進 v3 得先定主分類，這次未補，網站繼續從 nobu_shops.txt 補。
- 目錄 hash 41eb581fd29513fc，網站測試 2289 項通過。

## 2026-10-29 凌晨（history-nobu 工作線 A：IR 加上施放驗證用的 target）

- 依工作線 B 的請求（B01 已由直譯器執行，live-8 重播 45,895 tick 與 handler 每 tick 相同），IR 頂層新增 `target = { type, team, types, flags, ability_flags, range, min_range, aoe_radius, vector_length }`。
  - `ability_flags` 取自技能或道具 active 的 flags，例如 no_target_self。
  - 距離欄位是運算式索引。
- 105 份文件中，500 個有 IR 的條目帶 target，編譯 0 問題。規格寫在 proposal §4.1，package_v3_test 共 81 項通過。
- EMIT_IR 仍是 false，live 內容不變。

## 2026-10-29 凌晨（history-nobu 工作線 A：呈現 cue 目錄）

- 新工具 `docs/character_pipeline/tools/v3_cues.lua`，輸出 `docs/character_pipeline/presentation/cue-catalog.lua`（機器可讀）與 `cue-catalog.md`（摘要），供 UE 線使用。
- 資料來源：
  - 原作 KV：EffectName（modifier、AttachEffect、FireEffect、投射物、FireSound）、OverheadEffect、StatusEffectName、precache。
  - 原作腳本：`.vpcf` 字串，以及 EmitSound*、StartSoundEvent*、EmitGlobalSound 的第一個字串參數。
  - 只記錄路徑，沒有複製任何資產。
- 對應方式：在各條目 provenance 的行範圍內找候選。
  - 狀態 cue：對到 notes 寫的原作 modifier 的 EffectName（high）。
  - 其他 cue：依技能編號前綴、名稱字詞與已佔用程度打分數（medium／low）。名稱是 vo、sound、voice、title、load 的 cue 只配音效，其他只配粒子。
- 結果：v3 用到 1,080 個 cue，910 個對到原作資產（high 221、medium 344、low 345）；170 個使用通用佔位特效（`generic`，音效為 `generic_sound`）。
- 出處範圍內有 852 個原作資產（粒子 560、音效 292），642 個已對到 cue；其餘依條目列在 `unmapped`。
- low 信心的對應要在導入美術時人工確認。
- package_v3_test 新增 KV 與腳本解析的測試，共 80 項通過。

## 2026-10-29 凌晨（history-nobu 工作線 A：使用頻率統計、M5 前待決事項依 D13 決定）

- **使用頻率統計**（M4 最後一項）：
  - 新工具 `docs/character_pipeline/tools/v3_usage.lua`，產生 `docs/plans/2026-10-28-history-nobu-full-port/usage-stats.md`：307 種元件，各列出使用次數與英雄、道具數，並附依解鎖英雄數排序的實作順序。
  - 里程碑：M3 第一批（B01、C17）55 個元件；104 個元件解鎖 10 位英雄，196 個 50 位，277 個 92 位（全部可解鎖的英雄）。
  - 清單已發給工作線 B（m2-rules-layer）。package_v3_test 加了一項測試。
- **M5 前待決事項（D13，照原作實際行為，資料只留一種寫法並記 quirk）**：
  - **鞋類被動屬性重複**（暗部長靴、仕手能靴與兩者的破魔版、褶裙）：
    - 移速（仕手能靴另含回魔、法術增幅）只由唯一狀態提供。原作 MOVESPEED_BONUS_UNIQUE 只取最高，Passive_item_mana_boots 非 MULTIPLE。
    - `passive.stats` 拿掉這些鍵，也拿掉會讓攻擊、攻速也變成不疊加的 `passive.unique_group`。
    - 說明文字補上 `{move_speed}`、`{mana_regen}`、`{spell_amp}`，資訊列少了這幾行。
  - **火之章主動**：照普查。同一施法者、同一道具、同名又非 MULTIPLE 的 modifier 只會刷新光環實例，沒有額外效果（原作 KV 1318-1331、1379-1385）。E173 的「直到死亡」改記為 port_adaptation；說明、chapter_bonus、chapter_duration 仍綁著現行 handler 與 v2 數值，M5 切換時刪除。
  - **般若湯**：依原作 OnToggleOn → Shock 補上行為（移除敵方來源狀態、魔免 `$immunity`、boots_family 共用冷卻）。
  - **B08R_SE**：原作腳本證實會把 AphoticShieldRemaining 設為零（讀不存在的 damage_absorb 得 0）。刪掉 kit 那條錯誤的「只有粒子」，資料照原作；是否修正仍是使用者的 FC-005。
  - **A07 奮起等級**：照原作結構（五級、等級＝碎擊槍等級＋1）。目前受 v2 max_level 1 限制，M5 切換時改 max_level 5、attack_speed 逐 rank，刪 attack_speed_spear_0～4，寫在 quirk。
  - **B04 火焰速度**：改成推導值 `$dash_speed * $fire_speed_ratio`（原作 speed × 0.5），數字不變。
  - **只需 M5 實測、不改資料的項目**：
    - B29 quake_axe、A34 frost_breath 的目標類型，kit 是 direction、普查是 point：要確認 host 給的點能否讓 300 截距成立。
    - A34 目標類型不含 neutral／summon，普查的範圍有含。
  - **schema v3.x 待辦（工作線 A）**：運算式不能在百分數與一般數字之間換算，所以 A07 `magic_resist_pool`、A02／A07／B06 的 `base_attack_time` 是同一數字的第二份。之後加單位轉換函式時一併消除。
  - 沒有需要使用者決定的新項目，fix-candidates.md 只補了出處的新位置。
- **放回**：8 個檔（6 份文件，加上重新產生的 item_texts_v3.lua、authored_heroes.lua），codegen、cargo、server、bridge 檢查都通過。重新產生的內容中，43 招只有 B04 fire_speed 多了推導標記；道具只有這 5 件的說明與資訊列不同。另外修了 v3_scenarios：`refs` 不再收 quirk 原文。package_v3（79 項）、網站測試與 gen_item_v3 --check 都通過。

## 2026-10-29 凌晨（history-nobu 工作線 A：v3 文件移到正式位置、測試情境產生器；E200）

- **正式位置**：
  - 英雄改放 `docs/character_pipeline/packages/<package>/v3.lua`，97 位各一份。87 位未上架英雄新建資料夾，以英文名 snake_case 命名，只放 v3.lua。
  - 道具改放 `docs/character_pipeline/items/batch_I0..I5.lua`、`pilot_a.lua`、`pilot_b.lua`，共 166 件。shelf_a、shelf_b 的 45 件上架道具已併入，每件只出現一次。
  - `census-v3/` 只剩 README，內容是對照表加上普查紀錄。
  - 找檔一律經 `docs/character_pipeline/tools/v3_sources.lua`。hero_author、gen_item_v3、gen_source_catalog、網站（census_adapter／build_site）與測試都已改讀這裡。
- **kit 與 v3 的關係**：合併成一份，kit.lua 刪除。
  - 數值、說明、`v2_key`、`rust_handler` 照 M1 kit；行為照普查，並改用 kit 的值名稱。
  - kit 沒有、只有行為用到的量，新增為 hidden 值：英雄 17 個、道具 9 個，數字衝突只有 B06 aura_radius 1 筆（kit 贏，加 quirk）。
  - 驗收：逐招比對編譯輸出（description、levels、extras、text、info），重新產生的 authored_heroes.lua（43 招）與 item_texts_v3.lua（45 件）和 live 零差異，只多出新值。
  - hero_author 遇到 kit.lua 會直接報錯。
- **測試情境產生器**：`tools\lua\lua.exe docs\character_pipeline\tools\v3_scenarios.lua [ID ...]`，輸出到 `target/v3-scenarios/<id>.json`。
  - 每個技能與道具主動的每個 rank 都有情境：正常、資源不足、冷卻、目標失效、魔免、驅散、死亡、可見性，狀態觸發也各有一條，每條 quirk 一條。
  - 期望只寫值名稱與文件裡的運算式，`refs` 列出要解析的值。
  - 105 份文件共 29,412 條；C17 247 條、B01 116 條，範例放在共用筆記。
- **放回**：land_to_live 共 310 個檔（其中刪除 123 個），codegen、cargo（base_content、omobab、bridge）與 server 檢查都通過。live 的 package_v3（78 項）、hero_author、網站測試，以及 gen_item_v3 --check 都通過。
- **出處**：已上架英雄第一次跑嚴格比對，修正 6 筆 kit 出處字串，數值與說明不動。現在 105 份全部 0 mismatch、0 not found。
- **M5 前要決定的事**：見共用筆記 `v3-formal-sources-and-scenarios.md` §3。例如道具被動屬性同時寫在 passive.stats 和 unique 狀態裡，切換前要擇一；火之章主動兩種解讀衝突；B08R_SE 清零。

## 2026-10-29（history-nobu M3 工作線 A：IR 編譯器、全量來源目錄；E199）

- **IR 格式**：v3-content-model-proposal.md §4.1 草案 v1，以工作線 B 的 m3-ir-design.md 為底，已送 B 對齊。
- **IR 編譯器** `docs/character_pipeline/tools/v3_ir.lua`：
  - census 105 份全部能編譯：603 個條目、5165 條指令、10565 個運算式，0 錯誤；阿市、雜賀孫市的 kit 也能編譯，範例 JSON 在共用筆記資料夾 `v3-ir-sample-c17-b01.json`。
  - v3_compile 已接上，但預設關閉（`M.EMIT_IR = false`），等 B 的 `ir.rs` 放回時同一批再打開。
  - 測試 `scripts/tests/package_v3_test.lua` 新增 3 項（共 75 項）：參考求值器驗證 and／or／select 的跳躍語意、105 份的 IR 都沒有混合 table 或空 table、C01 的降階結果。
- **全量來源目錄** `scripts/gen_source_catalog.lua`（已 land_to_live 放回）：
  - 輸出 `target/site/source-catalog.json`：97 位英雄（10 位 playable）、446 招技能、175 件道具（20 件 deferred）。
  - 不進遊戲 content；格式寫在 proposal §2.5，已通知網站 session。
- **check_live_consistent 目前失敗**，但不是這批造成的：`omoba-core` 的 `moba_match/base_structures.rs:206` 讀了不存在的欄位 `super_minion_replaces_siege`，看起來是別的工作線只放回了一半。已通知主 session。

## 2026-10-29（E197：機器層級實機局鎖；光鍊 mismatch 離線無法重現）

- **已放回 live**：
  - `scripts/machine_lock.lua`（新），以及 `moba_role_launch.lua`、`run_moba_role_ue.lua`、`run_moba_split_local.lua`、`verify_moba_local_baseline.lua`。
  - 測試 `scripts/tests/machine_lock_test.lua`，20 項通過。AGENTS.md 已補說明。
- **行為**：
  - 開實機局前取得 `target/locks/live_match`。被占用時拒絕，並列出持有者的 run 目錄與程序；`--wait-live-lock SECONDS` 可以排隊。
  - 持有者的程序都已結束時，自動接手過期鎖。split 的子程序以 token 加入父程序的鎖。
  - 效能量測另外取得 `target/locks/performance`；headless batch 不受限。
- **光鍊 mismatch（m0 回報的 `target/m1v3/live-4`）**：
  - 那一局在 00:05 出錯，E196 的修正 00:06 才放回，所以它跑的是沒有修正的 build。
  - 以目前 live 重播：7572 筆輸入全部重播，0 mismatch。
  - 但在影子停用 E196 再重播到 19500 tick，也是 0 mismatch，tick 18720 的分歧沒有重現。所以不能歸因於 E196，目前未解決。
  - 這和 e190 的 `live-10`（阿市＋阿松，team 1 小兵 HP）同一型：實機有分歧、離線重播沒有，兩局都在 CPU 爭用下。懷疑是實機 observer 路徑在負載下的時序差（例如輸入被投影到的 frame 和執行的 tick 不同），而不是 gameplay 判定。
  - 下一步：在有實機局鎖、沒有爭用的條件下重跑阿市的局，看是否還會出現；如果出現，比對 observer 收到的 frame 與 accepted-inputs 記錄的 tick。
- **影子**：`target/e197-lock/new`（內容與 live 相同）、`target/e194-reuse/shadow-m1`（重播用，含 `E196_OFF` 對照開關，不放回，可刪）。

## 2026-10-29 凌晨（history-nobu M2 完成：規則層第二批放回 live；E198）

- **M2 結束條件**：規則層單元測試（omoba-core 1286 全過）；10 位英雄的 parity soak 與 single_lane 的 replica parity 全部 0 mismatch（唯一失敗的三線生命週期測試在 live 基底同樣失敗，已由 E196 另外修好）；live-8 重播 45895 tick 0 mismatch；headless 效能沒有退步。
- **第二批內容**：瞬發攻擊走攻擊管線、多來源爆擊取最高、擲骰以揭露身分為鍵（英雄用玩家 id）、物理格擋、v3.1 規則詞彙（physical_immune、magic_resist_pool、base_move_speed、miss_chance_pct、hp／mana_regen_pct 與 amp、melee／ranged_attack、block_disabled、flying_for_pathing、temporary_night）。
- **影子**：`target/m2b/work`（基底 `target/m2b/base`，manifest `target/m2b/manifest.json`）。
- **接著**：M3 執行器核心，設計在 `docs/plans/2026-10-28-history-nobu-full-port/m3-ir-design.md`；content-model 的 `ir` 欄位已在 proposal §4 註記並通知 M0 session。

## 2026-10-29（E196：lifecycle 測試的 lumen_bolt 冷卻分歧已修、已放回；target 可清理清單）

- **E196**：
  - 根因不是 entity id 重用。施法 handler 讀到 authority 在同一 tick 稍早結算的生命（普攻擊殺），而 replica 依 M2 揭露政策不在本地結算傷害，看到的生命不同。
  - 修正：新增 `omoba-core/src/runtime/native/tick_start_hp.rs`；script dispatch 在兩邊都讀 tick 開頭的生命。同時改了 `native.rs`、`game_processor.rs`、`scripting/dispatch.rs`，以 3-way 經 land_to_live 放回；check_live_consistent 通過。
  - 驗證：omoba-core 1280 個測試全過；base_content 選定 199 個測試通過；e190 live-13 重播 0 mismatch。
  - 影子：`target/e194-reuse/shadow`（內容與 live 相同，沒有待放回的改動）。
- **還沒解決，交給 E187**：
  - `three_lane_60hz_full_filtered_match_lifecycle` 現在 replica 一致，但 seed 1 在 20 分鐘遊戲時間內打不完：tick 72001，18 座塔只倒 2 座，推塔 bot 死了 33 次。
  - 原因是 fixture 的推塔 bot 打不動原作數值的塔，需要調 fixture 或測試的上限。
- **target 可清理清單**（還沒刪；工作都已放回 live，證據目錄另列保留）：

| 路徑 | 大小 | 內容 | 建議 |
|---|---|---|---|
| `target/hero-port-8/ue-shadow` | 49G | hero-port-8 的影子 UE 專案（舊 build） | 可刪 |
| `target/hero-port-8/merge` | 17G | 3-way 合併工作樹（已放回） | 可刪 |
| `target/hero-port-8/rel` | 12G | 過期的 release build（E190 的成因） | 可刪 |
| `target/hero-port-8/w1`～`w4` | 11G／6.9G／9.5G／9.4G | worker 影子與 cargo 輸出（已合併放回） | 可刪 |
| `target/hero-port-8/cargo` | 3.3G | cargo 輸出 | 可刪 |
| `target/hero-port-8/shadow`、`base`、`fix*` | 約 90M | 快照與小型修正樹 | 可刪 |
| `target/hero-port-8/live-*`、`accept-*.txt`、`cards`、`packages*` | 約 1G | 實機證據、對照卡、package 快照 | 保留 |
| `target/e190-tamako/ue-shadow` | 49G | E190 的影子 UE 專案 | 可刪 |
| `target/e190-tamako/rel` | 12G | E190 的 release build | 可刪 |
| `target/e190-tamako/cargo2`、`cargo3` | 1.5G／0.8G | 重播用的測試建置 | 可刪 |
| `target/e190-tamako/shadow*`、`base2`、`diag-backup` | 約 70M | 影子與 base | 可刪 |
| `target/e190-tamako/live-*`、`accept-*.txt`、`replay*.log`、`fix-*.log` | 約 1G | E190 的實機與重播證據 | 保留 |
| `target/e194-reuse/cargo` | 1.9G | E196 的測試建置 | 可刪 |
| `target/e194-reuse/shadow`、`shadow-m1`、`base` | 約 47M | E196 影子（已放回）與重播用影子（含 `E196_OFF` 開關，不放回） | 可刪 |
| `target/e194-reuse/*.log` | <1M | E196 的診斷與重播紀錄 | 保留 |
| `target/e197-lock` | 0.2M | E197 影子與 base（已放回） | 可刪 |
| `target/e199-oichi/shadow` | 23M | lifecycle 測試用影子 | 可刪 |
| `target/e199-oichi/live-*`、`accept-*.txt`、`*.log` | 約 0.2G | 阿市重跑的實機證據 | 保留 |

  - 可刪合計約 184G：hero-port-8 約 118G、e190-tamako 約 64G（rel 現在 13G）、e194-reuse 約 2G、e197-lock 與 e199-oichi/shadow 不到 0.1G。
  - `e190-tamako/rel` 與 `ue-shadow` 在阿市重跑（e199）結束後已不再使用，可以刪。刪除前先用 tasklist 確認沒有程序正在使用這些目錄的檔案；launch 跑的是 `target/run-bin` 的副本。

## 2026-10-29 凌晨（history-nobu M1 工作線 A：數值單一來源放回 live；schema v3.2；E195）

- **放回 live**：116 個檔，land_to_live 的 codegen、cargo、server、bridge、UBT 全部通過。
  - content-model：`omoba-content-model/src/v3.rs`，`V3Block` 掛在 `AbilityDefinition.v3` 與 `MobaItemEntry.v3`，載入時驗證；raw 由十進位字串以整數運算換算，不經過 f32。
  - template-ids：`V3_CATALOG_JSON`（含 content hash），匯出：`cargo run -p omoba-template-ids --example export_content_catalog -- <out.json>`。
  - hero_author：`scripts/hero_author/v3.lua`。有 `kit.lua` 的 package，description、extras、levels 都由 kit 產生；v2 數字不一致、v2 數字沒有對應的 kit 值、handler 讀到被取代的鍵，三種情況都會讓建置失敗。
  - 10 位已移植英雄的 `packages/<hero>/kit.lua`（coverage `rust_handler`）。handler 共改了約 300 處鍵名，原本抄在註解裡的數字已刪掉。
  - 道具：`docs/character_pipeline/items/shelf_a.lua`、`shelf_b.lua`，涵蓋 45 件上架道具。`scripts/gen_item_v3.lua` 產生 `templates/item_texts_v3.lua`，並檢查道具列的每個數字都對得上 v3 值；`moba_items.lua` 在來源檢查之後掛上 v3 區塊、換成新的說明。
  - 出處比對：`docs/character_pipeline/tools/v3_provenance.lua`。
- **驗證**：
  - 候選樹 content-model 82、template-ids 全部、base_content heroes:: 146 項通過，含 10 位英雄的 parity soak；
  - `scripts/tests/package_v3_test.lua` 72 項、`hero_author_v3_test.lua` 6 項、`hero_author_test.lua` 14 項、`character_package_test.lua` 10 項。
- **schema v3.1、v3.2**（v3.md §12.10、§12.11）：處理了 M4 普查回報的全部缺口；新增跨文件登錄表 `schema/v3-shared-registry.lua`，驗證器加上跨文件檢查。
  - census 105 份全部通過（含登錄表），出處比對 4057 筆一致、0 mismatch。
  - coverage 614 筆：expressible 598、needs_hook 0、pending 6、deferred 10。
- **實機**（放回後重建 client runtime、bridge、UE 再開局；快速地圖、自動駕駛 lane_v0、10 分鐘）：
  - `target/m1v3/live-3`（阿市對村上義清）：
    - 0 mismatch、0 termination；
    - 村上 4 招全部 PASS，阿市 5 招中 4 招 PASS；
    - 光鍊 cast_ok=0：12 次都是自動駕駛把目標選在非英雄（handler「target is not a live hero」）。M1 之前的 e190 live-10 也有同樣的大量錯誤，與這次改動無關。
  - `target/m1v3/live-4`（阿市對玉子）：
    - 9 招全部 PASS，光鍊 cast_ok=3；
    - 1 次 observer mismatch：team 1、tick 18720，replica 有光鍊冷卻，authority 因為目標無效而拒絕施放。和 E175／E190 是同一類 replica 冷卻分歧，已轉給 hero-port-8-mismatch session；
    - 這一局和另一組實機局同時在跑。
  - **說明顯示**：
    - UE 產生碼（OmGenerated 的 OmContentClasses.cpp）已經是 v3 模板渲染後的說明；
    - 自動駕駛的 tooltip 截圖沒有拍到說明框（懸停沒生效），畫面上的確認由工作線 C 的截圖驗收負責。
- **尚未完成**：UE bridge 的 `om_v3_content_catalog_json` 由工作線 C 放回。

## 2026-10-29 凌晨（history-nobu M1 工作線 C：TooltipRenderer 接上正式 catalog，實機驗證完成；E201）

- **bridge**：M1 的 `V3_CATALOG_JSON` 放回後，bridge 新增 `om_v3_content_catalog_json`，C ABI 從 38 升到 39（含 assert 與 smoke 標頭，共 11 處），已用 land_to_live 放回。
- **UE 讀 catalog 的方式**：只從 runtime 已載入的 bridge 讀（`UOmRuntimeBridgeSubsystem::LoadedBridgeHandle()`），不重新載入、也不釋放 bridge。原本的寫法會讓渲染端死掉，見 E201。
- **語言**：預設 zh_TW，只有明確以英文執行（`-culture=en` 或遊戲設定檔）才用 en。
- **其他調整**：
  - 道具的資訊列不放價格；
  - v3 模式的技能 tooltip 保留目標與形狀說明，只拿掉和資訊列重複的耗魔、施法距離兩行。
- **autopilot 實機證據**：指標停住後，把元件自己的提示交給 HUD tooltip host（off-screen 的引擎不會自己開）；trace 記 `hint_shown`。
- **測試**：影子 79 項通過，包括 `Om.Runtime.TooltipBridgeCatalog`：catalog 88 個條目，兩種語言、每個等級，共 572 次渲染成功。所有斷言舊說明文字的既有測試，在 catalog 啟用後也都通過。
- **實機**：`target/tooltip/live8`（阿市對雜賀孫市，autopilot，約 8 分鐘後我自己停掉），截圖在 `target/tooltip/evidence/`：
  - 技能欄：繁中標題、資訊列，等級數列中目前等級加粗，魔法傷害為藍色；
  - 商店：「+5 攻擊力」。
- **放回 live**：bridge 4 個檔（codegen＋cargo＋bridge 檢查），UE 7＋4＋2 個檔（codegen＋UBT）。最後一次放回之後 `check_live_consistent --ubt` 通過。
- **還沒做**：召喚師技能、守衛、狀態的說明，要等 catalog 有它們的 v3 資料。網站與全量普查的擴充由工作線 E 接著做。
- 沒有 commit。

## 2026-10-28 深夜（history-nobu M1 工作線 C：UE 通用 TooltipRenderer；E194）

- **顯示位置清單**：`docs/plans/2026-10-28-history-nobu-full-port/m1-tooltip-sites.md`，共 15 處；接線狀態逐處列出。
- **渲染器**（OmRuntime）：
  - `FOmTooltipRenderer`：模板解析與代入，產生 runs `{ 文字, 加粗, 傷害類型 }` 與 rich markup。規則和網站相同。
  - `FOmTooltipCatalog`：讀 compiled catalog 的 v3 區塊。`Describe()` 產生標題、資訊列、屬性行與說明：
    - 等級數列以目前等級加粗；
    - 冷卻與耗魔可以附即時值，和設計值不同時顯示「（目前 x）」；
    - 語言跟隨 UE culture（en 或 zh_TW），缺少的語言退回 zh_TW。
  - `FOmTooltipRichStyle`：粗體、標題、標籤，以及物理、魔法、純粹三種傷害顏色。
  - `SOmHudToolTip` 新增 `RichText` 屬性：有 rich 內容就用 SRichTextBlock，沒有就照舊顯示純文字。
- **接線**：
  - 技能 tooltip（技能卡、技能圖示）：有 v3 時改用 rich 文件；升級預覽只留升級條件。
  - `BuildItemDescription`：有 v3 時改用 catalog 文字。商店購買卡、出售格、詳細欄、道具欄、搜尋都經過這裡。
  - catalog 裡沒有 v3 的條目照舊顯示 description。
  - catalog 由 bridge 匯出的 `om_v3_content_catalog_json` 提供，執行期才查找。
- **測試**（影子 `target/tooltip/ue`）：
  - 新增 3 項，全部通過：
    - `Om.Runtime.TooltipRendererVectors`：共用向量 32/32，和網站一致；
    - `Om.Runtime.TooltipCatalog`；
    - `Om.Runtime.RichHudToolTip`：720p 與 1080p 都在 480×600 的上限內，長內容改為捲動。
  - 回歸 14 項通過：AbilityUpgradePreview、ShopModel、AbilityTargetTooltip、CastDamageInterruptionTooltip、CastWindupTooltip、ReadableHudToolTip、HudHintFollowsRoutedPointer、ShopTooltipScope、ShopRecipePresentation、ShopPanelSearchAndRetirement、HudTextViewportFactor、ExtraAbilitySlotHud、HotkeyHudGuard、ShopIconGrid。
- **共用向量 version 2**：依正式 catalog 的定義，pct 存百分數，「%」只在最後加一次。網站同步修改；site_text_render_test 105 項、site_build_test 2,285 項通過。
- **放回 live**：land_to_live 11 個檔（codegen＋UBT），之後 `check_live_consistent --ubt` 通過。
- **實機證據步驟**：autopilot 新增 HoverTag 步驟。開截圖的對局在第一次開商店時：
  - 先停在技能卡上，截 `tooltip-ability`；
  - 再停在商店道具卡上，截 `tooltip-shop`；
  - trace 記 `tooltip_hover`。
  - 6 項 AutoPilot 測試通過；land_to_live 2 個檔（codegen＋UBT）。
- **網站改讀正式 catalog**：`build_site --catalog` 可以直接讀 V3_CATALOG_JSON 的匯出（`scripts/site/catalog_reader.lua`）；頁尾顯示 catalog 自己的 content hash。測試 fixture 是 `scripts/tests/fixtures/v3-content-catalog-sample.json`。
- **還沒做**：
  - bridge 匯出 `om_v3_content_catalog_json`：要等 M1 的 `V3_CATALOG_JSON` 放回 live。改好的版本在 `target/tooltip/bridge/lib.rs`，基底 `lib.base.rs`。
  - 匯出之後要做：
    - 重跑 tooltip 相關測試；
    - 實機截圖技能欄與商店的說明；
    - 召喚師技能、守衛、狀態的說明，等 catalog 有它們的 v3 資料再接。
- 沒有 commit。

## 2026-10-28 深夜（E190：hero-port-8 實機驗收 8 位英雄 PASS；過期 build 防呆、input_grace 與 replica 本地死亡）

- **live-7／8 的玉子 Q mismatch**：
  - 不是新的分歧路徑。驗收用 `--no-build`，跑的是 `target/hero-port-8/rel` 06:17 的 build，比 E175 第三路徑修正（07:03）早。
  - 以目前 live 重播 live-8、live-7，全場 0 mismatch。拿掉該修正再重播，tick 1679 重現同一分歧。
- **已放回／已改**：
  - `scripts/run_artifacts.lua`（Lua 工具）：啟動前讀 cargo dep-info（`<name>.d`）。server、runtime、renderer fixture 或 base_content 只要有任一來源比二進位新，就拒絕啟動。測試在 `scripts/tests/run_artifacts_test.lua`。
  - `omoba-core/src/runtime/filtered_specs.rs`（land_to_live）：input_grace 多揭露一個 frame 的輸入目標，如果 replica 自己的 step 已經結算了同一次死亡，就保留刪除狀態，等下個 frame 退休；其他本地已刪除的情況仍然報錯。回歸測試為 `graced_input_target_deleted_by_the_replica_step_stays_deleted_until_retired`。
  - check_live_consistent 通過。
- **實機（quick 地圖，--max-seconds 600，build 在 `target/e190-tamako/rel`，UE 影子在 `target/e190-tamako/ue-shadow`）**：

| run | 英雄 | verdict | mismatch |
|---|---|---|---|
| `target/e190-tamako/live-6` | 稻姬＋村上 | PASS | 0 |
| `target/e190-tamako/live-11` | 淺井＋伊達 | PASS（b08_endless_fang 16 級 NOT_REACHABLE_IN_10MIN） | 0 |
| `target/e190-tamako/live-12` | 雜賀＋玉子 | PASS | 0 |
| `target/e190-tamako/live-13` | 阿市＋阿松 | PASS | 0 |

- **未解決**：
  - `live-10`（阿市＋阿松）有 1 次 team 1 小兵 HP mismatch，經 rebase 恢復，沒有中止。離線重播無法重現（見 E190）。
  - 阿市 `c17_light_chain` 實機每局有 12–22 次 handler 拒絕（目標不是活著的英雄），verdict 不受影響，但 UE autopilot 的選目標要修。
  - bot 阿松同一 tick 對兩個目標重複下 `a34_frost_bolt`。
- **狀態**：
  - 所有修正都已放回 live。影子 `target/e190-tamako/shadow2` 與 live 的 filtered_specs.rs 相同，沒有待放回的改動。
  - 我啟動的程序都已結束。
  - 其他影子與證據：`shadow3`（live 快照，用來重播 live-10）、`rel`（release build）、`ue-shadow`（UE 影子），都在 `target/e190-tamako/` 下。
- **下一步**：
  1. 查 live-10 的分歧：比對實機 observer（encoded frame）與重播 harness，以及同 tick 多筆輸入的投影順序。
  2. 修 bot 同 tick 重複施放。
  3. 修 UE autopilot 光鍊的選目標。

## 2026-10-29（E187：塔／兵營／主堡／小兵／野怪改原作數值，quick 單一倍率；已放回 live）

- **已放回 live**：
  - 分三次用 land_to_live 放回，都通過 codegen、cargo、server、UBT；之後 `check_live_consistent` 也通過。
  - 地圖：`moba_maps.lua`，數值照原作（見 errors.md E187）。兵營和大將塔不重生。第 30 秒出第一波，之後每 35 秒；足輕、弓兵、鐵炮每波成長。野怪重生 120 秒。
  - 引擎：`omoba-template-ids`、`omoba-core` 的 moba_match、`server_config` 支援上面的地圖設定。新增 `SingleLaneConfig.first_wave_delay`。
  - headless 工具：失敗時印出原因；停滯監測納入兵營和大將塔。
  - 相關測試已修正。
- **quick 地圖**：只有一個倍率 `QUICK_TIME_SCALE = 3.5`。建築 HP 與塔攻擊 ÷3.5，英雄經驗 ×3.5，其他照標準地圖。
- **量測**（release headless，12 seed，原陣容＋兩隊陣容對調）：
  - quick k=3.5：
    - 17/22 局在 20 分鐘內自然結束；
    - 局長中位數 13.4 分（原陣容）、10.4 分（對調）；
    - 雙方都推得掉塔。
  - k=4 沒有比較快：14/22。
  - 塔攻擊不縮放：0/10。
  - 標準地圖（原作數值，上限 3600 秒）：6/6 自然結束，36.7–56.9 分，中位 41.4 分。
  - 勝方跟著 honda／sanada／tamako／inahime／omatsu 這組陣容走，屬於 E165。
- **測試**（放回後的 live 快照）：
  - omoba-core 1259、template-ids 20、moba-headless 7、base_content map 測試 342 通過。
  - release soak 5 個通過。
  - **`three_lane_60hz_full_filtered_match_lifecycle`（release-only）失敗**：新局勢觸發既有的 replica 施放冷卻分歧。原因是同一 tick 內被重新使用的 entity id，authority 拒絕施放、replica 卻記了冷卻。詳見 E187，已交 main 分派。
  - omb 有 1 個既有失敗（E164 fixture），與本項無關。
- **影子與量測資料**：`target/e187/`
  - `tree` 是放回的內容；`runs/*` 是各局報告；`summary.lua` 印結果。
  - `measure.sh` 的用法：`TREE=… ./measure.sh NAME RECIPE MAXSEC FROM TO PAR`。
- **還沒解決**：
  - quick 約 1/4 的局停在兵營前：外塔清空後 bot 不帶線推進，大將塔射程又涵蓋兵營。
  - 上面的 replica bug。
  - 原作的塔成長、騎兵、兵營破壞強化還沒做。
- **實機確認**：未開 UE 局。之後要確認時，用 quick 地圖跑 `run_ue.bat`，看 10 分鐘內雙方推塔、小兵面向。

## 2026-10-28 晚（history-nobu M2 第一批：規則層與傷害、攻擊管線；已放回 live）

- **負責**：工作線 B（M2 session）。詳細見 errors.md E193、`docs/plans/2026-10-28-history-nobu-full-port/m2-rules-inventory.md`、`m2-disclosure-policy.md`。
- **已放回 live**（land_to_live 27 檔，`--checks codegen,cargo,server,bridge`；之後 `check_live_consistent` 通過）：
  - 新模組 `omoba-core/src/runtime/native/rules/`：damage（四段管線）、attack（單次攻擊紀錄與事件、爆擊在前搖開始擲）、chance（偽隨機保底）、magic_resist（英雄魔抗池）、status（狀態抗性 ignore）、healing（減療取最強）、targeting（排除 tag）。
  - script-abi：`DamageProfile::REQUIRE_SOURCE_ALIVE`、`StatKey::MagicResistancePool`。
  - 英雄：A07T 改用魔抗池；B29T、C15T 的暈眩不受狀態抗性影響（base_content 與 package `rust/` 副本同步）。
- **驗證**：omoba-core 1277、script-abi 15、base_content `heroes::` 146 全過；`replicas_identical` 35 個中 34 過，唯一失敗的 gunner 視野邊緣 soak 在 live 基底同樣失敗（既有問題）；live-8 accepted inputs 重播 45895 tick 0 mismatch；headless 18000 tick 效能最佳值 98.45→98.41 秒。
- **影子**：`target/m2/work`（基底 `target/m2/base`，manifest `target/m2/manifest.json`，cargo target `target/m2/ct`）。
- **下一批**：兵線與野怪的瞬發攻擊走三段管線、多來源爆擊取最高、擲骰改以揭露身分為鍵、魔抗池類道具與 A34T。

## 2026-10-28 深夜（history-nobu M1 工作線 E：網站產生器 MVP）

- **入口**：`tools\lua\lua.exe scripts\build_site.lua [--catalog FILE] [--docs PATH ...] [--out target/site] [--previous FILE] [--source-root DIR] [--no-icons]`。
  - 預設讀 census-v3。由轉接層 `scripts/site/census_adapter.lua` 驗證每份 v3 文件（`package_v3.validate_doc`，不合格的文件不收），再編成 `v3-content-model-proposal.md` 的結構：
    - values：精確值與 Fixed64 raw；pct 存成比例；衍生值逐 rank 求值；
    - text：預先解析的模板；
    - header：資訊列。
  - 產物寫在 `target/site/`：`zh-TW/`、`en/`、`assets/`、`img/` 與 `catalog.json`。正式 compiled catalog 出來後，改用 `--catalog` 讀；轉接層到時刪除。
- **頁面**（繁中、英文各一份）：
  - 首頁英雄列表；
  - 英雄頁：三圍與成長，取自 `hero_sources.lua`；每招的資訊列（目標、冷卻、耗魔、耗血、次數、射程、半徑、前搖、引導、需要等級、傷害類型、穿透魔免、驅散）、說明、狀態列表；用等級分頁切換目前等級，目前等級的數值加粗，純 CSS；
  - 商店分類頁；
  - 道具頁：價格、商店分類、屬性加成、說明、主動資訊列、合成樹與可合成清單；
  - 改版說明：比對 `--previous` 的 catalog 與本版的 values；
  - 每頁頁尾顯示 content hash（canonical JSON 的 SHA-256）與原作 commit。
- **素材**：只用原作 nobu 自製圖示，依 KV 的 `AbilityTextureName` 與 `override_hero` 對應；Dota 內建圖示與缺圖一律用佔位圖。商店分類取自原作 `nobu_shops.txt`。
- **共用測試向量**：`docs/character_pipeline/schema/v3-text-render-vectors.lua`，29 筆，涵蓋格式 all／current／max／abs／int／signed、pct、四捨五入、等級加粗與夾限、hero_level、rules、`{@id}`、傷害類型、錯誤情況。UE TooltipRenderer 要用同一份驗證。
- **測試**：
  - `scripts\tests\site_text_render_test.lua`：29 筆向量、95 項檢查。
  - `scripts\tests\site_build_test.lua`：固定讀 M0 的 12 份範例（10 位英雄、pilot_a、pilot_b），78 頁、2,285 項檢查，涵蓋：
    - 每頁每個數字與 values 一致；
    - 說明裡沒有不是佔位符產生的數字；
    - 檢查器確實抓得到亂入的數字、改過的數字、全形數字；
    - 中英文同數字；
    - content hash 穩定，數值或文字一改就變；
    - 改版說明；
    - `catalog.json` 來回讀寫後頁面完全相同。
- **結果**：
  - 一度以整個 census-v3 建置成功：33 位英雄、51 個道具，共 174 頁、169 張圖示，10 位試點英雄與 25 個試點道具都在內。
  - 之後普查 session 寫到一半的文件（B14、batch_I0）沒通過驗證，預設建置就照規則拒絕、不寫入；要先建置可以用 `--docs` 指定已完成的文件。
  - 錯誤紀錄見 errors.md E192。
- **待做**：
  - 道具 package 缺 `shop.category`：目前從原作檔案補。
  - 未收錄的零件只顯示 id。
  - 「網站＝遊戲內說明＝結算」三處比對要等 M1 的 TooltipRenderer 與 catalog。
  - M8：單位模板頁、狀態圖示、作者手冊。
- 沒有 commit；只新增 `scripts/build_site.lua`、`scripts/site/`、兩個測試、共用向量，沒有改 live 輸入。

## 2026-10-28 深夜（history-nobu 完整還原 M1 工作線 A：v3 content catalog 格式定案，進行中）

- **catalog 格式定案**：`docs/character_pipeline/schema/v3-content-model-proposal.md` §2。UE TooltipRenderer 與網站產生器照這份實作。
  - Rust 型別是 `omoba_content_model::v3::V3Block`，掛在 `AbilityDefinition.v3` 與 `MobaItemEntry.v3`。
  - JSON 是 `omoba_template_ids::V3_CATALOG_JSON`，含 content hash；匯出：`cargo run -p omoba-template-ids --example export_content_catalog -- <out.json>`。
  - 範例 JSON 與摘要放在共用筆記資料夾的 `v3-catalog-sample.json`、`v3-catalog-format.md`。
- **尚未放回 live**：在影子 `target/m1v3/` 進行中；放回時在這裡補記。
- 舊欄位的處理（description、extras、levels 都改由 v3 產生，暫時保留）：
  - `description` 是 zh_TW 模板代入所有 rank 的結果；
  - Rust handler 改讀 v3 數值名稱；host 契約鍵（`shape_radius`、`charges_max` 等）保留舊名當別名；
  - `hero_author` 會檢查 v2 數字與 v3 一致，handler 讀的每個鍵都必須存在。

## 2026-10-28 深夜（history-nobu 完整還原 M0：package schema v3.0 凍結，工作線 A）

- **schema v3.0 凍結**。規格 `docs/character_pipeline/schema/v3.md`（§12 語意細則、§13 變更紀錄）；作者指南 `v3-authoring.md`；詞彙表 `v3-vocab.md`（由 `tools/v3_vocab.lua` 產生）；content-model 提案 `v3-content-model-proposal.md`（M1／M3 用，M0 未改 Rust）。
- **驗證器**：`docs/character_pipeline/tools/package_v3.lua`（`v3_vocab.lua` 詞彙、`v3_expr.lua` 運算式文法與名稱解析），已接進 `validate_package.lua`：給 `.lua` 檔或含 `kit.lua` 的 package 目錄就走 v3；v2 package 帶 `kit.lua` 時兩者都檢查。
- **測試**：`tools\lua\lua.exe scripts\tests\package_v3_test.lua` 66 項通過（含全部 census-v3 文件）；既有 `character_package_test.lua` 10 項照樣通過。
- **v3 範例 12 份全部通過**（`docs/plans/2026-10-28-history-nobu-full-port/census-v3/`）：
  - 8 位試點 A11、A13、A21、B11、B22、C01、C06、C07；道具 `items/pilot_a.lua`（13 件）、`pilot_b.lua`（12 件）；
  - 已移植範本 C17 阿市、B01 雜賀孫市（描述現行 Rust handler 行為，數值 `note` 對照 v2 extras 鍵，給 M1／M3）。
  - 合計 77 筆：expressible 73、needs_hook 0、pending 4；hook 0（試點 v0 的 4 個 hook 全部改用資料表達）；752 個數值、575 筆 quirk（fix_candidate 19 筆待使用者決定）。
  - 仍 pending：A13W2 施法距離（要實測）、A21R 光環殘留時間（要實測）、C06R 跨英雄共用護盾變數（fix_candidate）、下忍編號槽位副作用（fix_candidate）。
- 改寫由 7 個子 agent 平行進行兩輪；第一輪回報的缺口全部補進 schema（v3.md §13），第二輪用新詞彙把 pending 從 31 筆降到 4 筆。錯誤紀錄見 errors.md E191。
- **沒有 commit**；沒有改 live 輸入（`scripts/lua_data`、Rust crate、omfue）。

## 2026-10-27（E124：每個 run 自帶 base_content；影子專案統一 stage 流程）

通用流程（取代手動 stage／複製 DLL）：

```text
一般 launch（run_moba_role_ue / run_moba_split_local / record_hero_spotlight）：
  prepare → run_artifacts.stage_base_content：最新 scripts/target/{debug,release}/base_content.dll
           複製到 <run>/artifacts/scripts/，server、runtime、game.toml 都指向它
  split client → --base-content <authority 的 run 副本>（同一局的內容相同）
  UE renderer → ue_run_snapshot.prepare(root, nil, {base_content=run 副本})：快照內 plugin 的 base_content 換成 run 副本
  preflight → bridge 對照 om_bridge.stage.json（同一組 stage 的 dll＋header）；不檢查共用 scripts/base_content.dll
  exe（E125）→ omobab／client runtime 複製到 target/run-bin/<key>/ 後從副本啟動；target/<profile> 不再被鎖
  content（E125）→ server、runtime、renderer bridge 內嵌的 OM_CONTENT_CATALOG_DATA_HASH 必須等於 authority 的 content_hash，否則啟動前就拒絕
影子專案（E116）：
  tools\lua\lua.exe scripts\stage_ue_shadow.lua --project DIR [--sync] [--build]
  然後 OM_UE_RUN_BINARIES=DIR 啟動；不要手動 cp om_bridge.dll / base_content.dll
```

- 新檔：`scripts/run_artifacts.lua`、`scripts/ue_bridge_stage.lua`、`scripts/stage_ue_shadow.lua`，以及測試 `scripts/tests/run_artifacts_test.lua`、`scripts/tests/ue_shadow_stage_test.lua`。
- 修改：`moba_role_launch`、`ue_run_snapshot`、`moba_server_build`、`moba_stage_contract`、`build_ue_moba`、`build_ue_bridge`、`run_moba_split_local`，以及對應的測試。細節見 errors.md E124。
- live omfue 還沒有 stage record，所以仍沿用舊規則（要等於最新 bridge build）。目前 live plugin 的 bridge 比共用 build 舊，下次 `build_ue_moba` 之後就會改用 record 規則。

## 2026-10-26（J1 通過、H03～H06 著陸、盲測 I1／I2、Shipping 打包）

```text
P1 15c／15d ✓（session-ward，2026-10-27）：
  - 15c：fire_dash（dash_duration）與 quake_axe 跳躍（共用速度）改為 forced_motion Mode::Dash 連續位移；瞬移 commit 已移除。
  - 15d：omoba-core 的 examples 已修正，--all-targets 可以全部編譯。
  - 驗證：omoba-core --all-targets 通過；base_content 全量測試另行回報。
  - 實機：p15c-murakami-date PASS 8/8，mismatch 0。
  - package rust/ 副本已同步（murakami No2_quake_axe、b29_test_support）；registry 重新產生（dash_to_point、leap 兩列）。
  - 15b（重新產生 content，讓 dash_speed 3000 生效）需要 live build_ue_moba，等使用者關掉 editor 再做。
P1 #12～#15 ✓（session-ward，2026-10-27）：
  - #12 tether、#13 隱形＋無碰撞：程式中已實作。#13 補了一個 Rust 測試：攻擊後仍隱形，偵測等級足夠才看得到。
  - #14 新增 omoba-core blink_landing.rs（point_walkable，落點由 host 決定合法位置），asai b08_flash_cut 已改用。
  - #15 新增 forced_motion Mode::Dash（連續衝刺），date b04_flame_trail_dash 已改用；package speed 3000 要等下一次 hero_author build 才生效。
  - capability-registry：由 candidate-requirements.lua 重新產生，更新 dash_to_point、blink_to_target、ally_area_query 三列。
  - package rust/ 副本已同步：asai No2_flash_cut 與 b08_test_support、date No5_flame_trail_dash 與 h02_test_support。
  - 全量 Rust：omoba-core lib／tests 1185＋3、base_content 541、omb-script-abi 15、hero_author 13，全數通過。
    另外發現 omoba-core examples 編不過（WaveBReadView 缺 brush_regions），與本次修改無關，見 next-month。
  - 建置：手動建置 release omobab、client runtime、base_content、bridge（live UBT 未建置），stage-dll 後複製到影子專案。
  - 實機（quick map，lane_v0，影子專案加 OM_UE_RUN_BINARIES，離屏，≤10 分鐘）：
    - p1b-asai-date-2：PASS，flash_cut 6 次、flame_trail_dash 24 次，mismatch 0。
    - p1b-oichi-inahime-2：PASS 9/9，light_chain 8 次、shadow_meld 6 次，mismatch 0。
P1 #8～#10 ✓（session-ward，2026-10-27）：
  - 三項能力在程式中都已實作，只是 next-month 沒有更新，所以這次是驗證之後標成完成：
    - #10 ally_area_restore：生命＋法力。
    - #9 cast_cooldown_refund／cooldown_refund.rs。
    - #8 build.rs 產生的 hero_unit_registry。
  - Rust 測試：omoba-core ally_area 2、cooldown_refund 2、hero_unit 1；base_content A34 8、C17 10、B08 11，全數通過。
  - 實機（quick map，lane_v0，影子建置加 OM_UE_RUN_BINARIES，離屏，≤10 分鐘）：
    - p1-asai-murakami-2：PASS；b08_endless_fang 是 16 級大招，10 分鐘內到不了，判 NOT_REACHABLE。
    - p1-oichi-omatsu-2：oichi 五招與 omatsu rapid_cooling 都 PASS，mismatch 0。唯一失敗是 omatsu 的 a34_frost_bolt：autopilot 整局只嘗試施放一次，與能力本身無關。重跑 p1-oichi-omatsu-3：比賽自然結束，PASS 9/9（含 frost_bolt），mismatch 0。
  - capability-registry 的 ally_area_query 備註已更新（法力回復已支援）。
技能特效 doing（session-ward，使用者原話：「參考 D:\omoba\ref\vfx_mcp 來製作角色的技能特效」）：
  資產端 BLOCKED（E101）：
    - omfue 的 UECP 外掛沒有啟用授權，每次開 editor 只能寫入 6 次，需要使用者啟用。
    - ref/vfx_mcp/Tools/uecp.py 已補上 2.x 相容層：port 9877、扁平工具、create_asset／ParticleMaterial、唯讀 handshake。
    - 21:42 外掛被換成 1.7.3（不是本 session 換的），build_ue_moba 的 ensure_bpgu_animation_import 讀不到舊路徑，已詢問主 session。
  執行期已著陸，尚未編譯：
    - package presentation.lua 每招新增選填的 `vfx = { cast, area, impact, aura, aura_buff, area_reference_radius, none }`。
    - 流程：package_v2 驗證 → hero_author 寫入 ue.vfx → codegen 產出 OmAbilityVfxCatalog（每個 package 技能各一列）
      → OmGeneratedModule 安裝到 OmAbilityVfx registry。
    - AOmUnitActor 播放規則：
      - cast 掛在施放者身上；area 放在 cue 目標點，依 AoeRadius 縮放；impact 由非施放 cue 觸發；aura 隨 buff 生滅。
      - 每個 cue 只播一次，隱藏時不播。Blueprint 的 ability slot 優先；有圖層時取代 native 佔位效果。
    - 測試：Om.Runtime.AbilityVfxLayers、Om.Content.AbilityVfxCoverage（未綁定的技能會以 content gap 警告列出）。
    - om_codegen 89/89（含新測試），hero_author 13/13，8 個 package 驗證全過。
  規格與素材（不需要 UECP 的部分）：
    - omfue/VFXSpecs/<hero>/NS_AI_<ability>_<Layer>.json 共 63 份，涵蓋 34 招；b04_flame_swiftness、a02_wind_drift 兩招宣告為無視覺效果。
    - 招式與圖層的綁定表在 VFXSpecs/bindings.draft.json；資產建好之後才寫入各英雄的 presentation.lua。
    - Grok 原圖 9 張在 VFXSpecs/_shared/source（flash／ring／glint／smoke／slash／rune／flame／shard／wisp），由 113 份 per-spec 貼圖共用，都已檢查過。
    - 商店的兩項收尾已經交給商店改版的子 agent。
J5 perf ✓ split43-perf-after（build-37，quick map，training＋saika，--autopilot lane_v0，比賽自然結束）：tick 平均 13.21→6.18 ms、p99 35.37→12.41、tick rate 55.1→61.3 Hz、
  hitch 6/488→0/527、hero_command 16.8→4.50 ms，PASS。split42 漏了 autopilot，作廢（E078）。剩下的熱點：dispatcher 48%、team_projection 26%。
rejoin（J1 延伸）✓ split44-rejoin（build-38，含 E077，wan-80-20-loss1 seed 7，team 2 在 +120s 斷線 45s，--autopilot lane_v0）：check_netem_outage 8/8 PASS。link lost 1 次 → rejoin 1 次後串流持續，mismatch 0、terminated 0；proxy 的 nat_flows 中，舊 endpoint 在 194.1s／204.1s 被回收。split41 ✗（E077）為修正前紀錄。

## 2026-10-26 23:30 停電前暫停（使用者指示，所有活動已停止）
接續時的狀態：
- 商店改版（LoL 參考圖：建議／全部道具／道具組合分頁、分類圖示、初始／基礎／史詩／傳說分組、合成樹、佇列、出售與復原）：子 agent 在 23:30 中途被停止，OmCommandBarWidget.cpp 等檔案可能停在改到一半的狀態。接續前先跑 build_ue_moba --build-only；不能編譯就檢查 git diff 並收尾。log 在 target/mvp-a3/shop-v3-*。
- 載入畫面（10 張英雄卡、直式立繪、玩家名、召喚師技能、進度、版本）：子 agent 也在中途被停止，log 在 target/mvp-a3/loading-*。直式立繪的生成進度未知。
- 技能特效：VFX runtime 已編譯（nullrhi 全量 Om. 302/302）；8 位英雄的 VFXSpecs 草稿（63 份）與 9 張 Grok 原圖已完成。特效資產等使用者啟用 UECP 授權（BpGeneratorUltimate_1.7.3 的 BuildId 與 D:/UE5.8 不同，需要重新編譯或換回 2.0.6）。RHI 有 10 項失敗，因為使用者的編輯器蓋住了測試視窗，需要重跑。
- spotlight 錄影：OmSpotlight 已編譯，測試通過；saika 樣板還沒錄（等編輯器關閉後先 build 再 --no-build 錄）。ffmpeg 在 target/tools/ffmpeg。
- 建置規則：一律用 build_ue_moba（有 E106 的 repo 建置鎖）；外掛依 .uplugin 尋找（E104）。
- 停電前 crontab 的 5／10 分鐘例行檢查已取消；接續時再重建。

補師存活 ✓（source-index，E098）：oichi 在 regress-50 死 11 次，修正後最終版三局依序 7／5／3 次，support-survival-6 為 3／1 次，8.2 分鐘自然結束，PASS 11/11，0 mismatch。主要修正：受重擊後 3 秒內持續撤退，撤退中只允許 ally／self 卡；需要撤退時放棄 channel／tether；lane 判定改用 route polyline 投影；後退沿 route 退 450；點不到攻擊目標就放棄，不再 attack-move 走進敵方兵線。風險：quick map 比賽長度變成 5～10 分鐘，偶爾會撞到 600 秒 time box（會判 INCONCLUSIVE，不會誤判 PASS），已列入 next-month。
小地圖方位 ✓（使用者回報「點右下會移動到左上」，子 agent，E097）：小地圖原本把 +X 畫成右、+Y 畫成上，但主攝影機（yaw 0）的螢幕上方是 +X、右方是 +Y，兩者差一個轉置，所以左上和右下互換，繪圖與點擊都錯。修正後每 tick 從主攝影機的實際投影推導「螢幕右／上」在世界中的方向，繪圖、點擊、ping、地面施法與 autopilot minimap_move 共用同一個轉換。新增 Om.Runtime.MinimapCameraOrientation（yaw 0／90／37），全量 Om. 292/292。實機 minimap-orient-1 截圖確認：英雄在地圖右緣時，小地圖視野框也在右緣。
回歸 ✓ build-50（E090～E096 之後）：三組英雄都 PASS、0 mismatch，加上 quick-xp-1 的 inahime 對 asai，8 位 authored 英雄全部通過。saika 對 date（死亡 4／0，12／13 級）、oichi 對 omatsu（11／4，12／16 級）、murakami 對 tamako（4／1，11／14 級）。oichi 的死亡次數明顯偏高，交給 source-index 查補師型英雄的生存規則。
quick map 英雄經驗值倍率 ✓（source-index，E096）：地圖新增可選的 hero_xp_scale（整數 permille，三個 config 入口都已接通，replica 等級沿用 CommittedProgression），quick map 設為 2.5。quick-xp-1（build-50，inahime 對 asai）在 7.5 分鐘自然結束，兩人分別到 11／14 級，R 都有施放，check_hero_live_acceptance PASS（9 PASS＋1 NOT_REACHABLE），0 mismatch。
E095 ✓（omfx-decouple）：有 cast_time 的技能在 replica 被誤判為「windup 期間移動」而取消，因為 origin 取的是 tick 中途的預測位置。修正後改用「開始那個 tick 結束時的已提交位置」，只錨定一次。split59（build-49，inahime 對 asai）0 mismatch，死亡 2／1；但比賽在英雄 6 級前結束，兩個 R 為 INCONCLUSIVE，已交給 source-index 為 quick map 加上 hero_xp_scale。
autopilot 打法強化 ✓（source-index，E094，next-month #21）：同 quick map、同英雄，人類席死亡 19 → 5（-74%），比賽都自然結束、0 mismatch。修正內容：安全回城與重試、修掉地圖邊緣的 recentre 迴圈、跟著兵線走、依威脅調整撤退門檻、不進塔／不打吃虧的換血、撤退中不施法。consistent build-48 已納入。after-9 出現的 b08_fang_slash 冷卻 replica mismatch 交給 omfx-decouple 處理。
打包 ✓ package-dev-5（build-47）＋split-dev-5-smoke（打包版，oichi 對 murakami）：自然結束，0 mismatch；打包版截圖可見頭像、技能圖示與共用人形英雄（8:47、8 級）。
共用人形 fallback ✓（ward，E093）：沒有正式模型的英雄不再顯示灰色方塊，改由 templates/moba_shared_hero_fallback.lua 依 id 穩定分配 4 套訓練 rig 之一（含 idle／move／attack／cast／hit／death 動畫，auto forward 對齊）；在 Editor 指定 NativeMeshAsset 會完全覆蓋。roster 測試仍以 content gap 列出這 6 位，表示還沒有正式美術。全量 Om. 289/289（RHI／-nullrhi），實機 fallback-humanoid-1（oichi 對 tamako）截圖確認是人形，facing_probe 的 yaw 和移動方向一致。
英雄頭像 ✓（子 agent 用 Grok Build CLI 生成，E092）：原本 9 位英雄（3 位訓練英雄、6 位 authored、2 位盲測中的 8 位加 frost_mage／channeler／grappler）使用共用的 placeholder 頭像，現在都是原創 1254 PNG；真實 catalog 的 placeholder 頭像為 0（coverage 測試強制檢查）。實機 split58-portraits（p1 oichi）截圖確認頭像與 Q／W／E／R／X 圖示都有顯示；英雄模型仍是 fallback 方塊（next-month #20，要在 Editor 指定）。開局第 53 秒的截圖還在 Preparing Textures／Shaders，這是新匯入資產在 editor 第一次執行時的編譯，第 57 秒已完成。
技能圖示 ✓（子 agent 用 Grok Build CLI 生成，E091）：原本有 44 個英雄技能沒有 HUD 圖示（b01／b04 改名後沒對應到、新英雄沒有圖、部分 lumen 技能沒有圖），現在每個技能都有一張原創 1254 PNG，匯入為 /Game/Omoba/UI/Abilities；HUD 同步遇到缺圖的英雄技能會補上備用圖並印出 WARNING。實機 split57-icons（p6 date_masamune）截圖確認 Q／W／E／R 圖示都有顯示。
盲測實機 ✓ split56（build-46，含 E090：range 0 的跳躍技能公布 clamp_range 作為可達距離，autopilot 觸及距離為跳躍距離＋範圍半徑）：murakami 4/4、tamako 4/4，0 mismatch；quake_axe server cast ok 共 20 行（split54 為 12 行）。next-month #15a 完成。
G2 重開一局 ✓ rematch-1（ward，quick map，11:42:58–11:51:54）：check_managed_rematch.lua 26/26 PASS。round-1 自然結束 → 兩位 autopilot 各點一次結算面板的「準備下一局」（ready_next_match 被接受）→ round-2 重新選角，server／runtime／UE 都是新程序，move 接受 49／40、cast 接受 6／8 → 180 秒 time box 依預定結束。入口指令與 E089 見 errors.md。→ G2 完成。
G2 1 人類席＋9 bot ✓ split55-single（build-45，新 recipe scripts/lua_data/moba_single_player_quick.lua，p1 oichi，autopilot）：拓撲為 1 個 client＋dedicated authority，約 4.5 分鐘自然結束（隊伍 1 勝），有結算截圖，0 mismatch、0 terminated。oichi 4/5（R 在 6 級才學、比賽隨即結束，沒有施放；split49 已驗證 5/5）。「重開一局」交給 ward 處理（autopilot 的 ready_next_match 加上 managed session 兩輪判定）。
J3／J6 打包 ✓ package-dev-4（build-44，Development，50 個檔案）＋冒煙局 split-dev-4-smoke-2（打包版 OmGame.exe、quick map、autopilot）：約 4.5 分鐘自然結束、0 mismatch。E081 補充：match_finished 的 hero_level 改成記錄整局最高等級（陣亡時 HUD 顯示 0），build-45 已編入，AutoPilot UE 測試 3/3。
盲測實機 split54（build-44，含 E088：package 宣告的 targeting shape 公布到 HUD 卡片）：0 mismatch，比賽 7:01 自然結束；quake_axe 施放 6 次（split53 只有 2 次，E088 生效）。tamako 的 R 在 tick 21371 被接受，但她在前搖中（tick 21389）被超級兵擊殺，所以沒有 cast ok → 判定 FAIL；這是正常的戰鬥結果，不是缺陷。I1／I2 仍以 split53 的 8/8 為準。
盲測實機 ✓ split53（build-43，含 E087：filtered replica 套用 tick 結束的 CommittedEquipmentStats 必須在 tick 內 HP 之後）：murakami_yoshikiyo 4/4、tamako 4/4，0 mismatch、0 terminated，比賽在 7:03 自然結束（英雄 7 級）→ I1／I2 實機通過，G5 完成。E087 是用 E086 的重播紀錄離線重現 split52 後找到的。quake_axe 這局有施放（4 次），但它沒有公布 area radius，觸及距離仍是 100；公布 targeting shape 的通用修正交給 ward。
盲測實機 ✗ split52（build-42，含 E085 tamako R、E086 可重播輸入紀錄）：team 1 在 tick 12600／12720 出現 hero PROPERTY mismatch，rebase 後仍不一致，在 tick 12839 以 observer-mismatch-after-rebase 終止比賽（終止流程正常）。tamako R 已實際施放（E085 生效）。server/accepted-inputs.jsonl 已產生，交給 source-index 離線重播定位。
英雄實機 split51-heroes（build-41，含 E083 召喚物 CommittedAttack fact）：0 mismatch、0 terminated，E083 實機驗證通過。比賽 6:04 結束：saika 3/3 + R INCONCLUSIVE_RUN_SHORT（只到 5 級）；date 3/4，R 在比賽最後才學會，來不及施放 → 判 FAIL。H01／H02 合併證據：split50 兩位的 R 都已施放（各 4 次），split51 證明修正後 parity 為 0 → H01／H02 實機通過。
英雄實機 split50-heroes（build-40）：saika_magoichi 4/4、date_masamune 4/4（H02 在 E037 修正後首次實機），比賽自然結束；但 tick 9000 出現 1 次 summon ATTACK observer mismatch（雜賀眾的 asd_count 973 vs 188，team 1，rebase 修回）→ 判定 FAIL，交給 omfx-decouple 查 disclosure 時 asd_count 的 parity。
英雄實機 ✓ split49-heroes（build-40）：oichi 5/5（aurora_guard 在 split38 未施放，這局已施放）、omatsu 4/4，跑滿 600 秒，0 mismatch → H03／H04 實機通過。
盲測實機 split48-blind（build-40，含 E081 判定工具、E082 range-0 範圍技能觸及距離）：murakami_yoshikiyo 4/4、tamako 4/4，但 tick 36000／36120 出現 4 次 hero PROPERTY observer mismatch（rebase 修回，沒有 terminated）→ 判定 FAIL，交給 source-index 查兩位盲測英雄的 handler 在 replica 端的 parity。split47 太早結束（3:11），已改判為 INCONCLUSIVE（E081）。
英雄實機 ✓ split46-heroes（build-39，含 E080 autopilot 升級／unit 目標修正與 relearn 修正）：inahime 4/4、asai_nagamasa 4/4 + 1 NOT_REACHABLE（b08_endless_fang 要 16 級自動學會），mismatch 0、terminated 0，比賽自然結束 → H05／H06 實機通過。
英雄實機 ✗ split45-heroes（build-38，quick map，p1 inahime 對 p6 asai，autopilot，比賽自然結束，mismatch 0）：inahime 3/4（被動 E 沒學）；
  asai 1/4 + 1 NOT_REACHABLE（W flash_cut 學了沒施放、R 約 8 分鐘才學且沒施放、被動 E 沒學）。原因看起來是 autopilot 的升級與 unit 目標施放規則（交給 source-index 查），修完後重跑 H05/H06，再跑 I1/I2、H03 局。

J1 ✓ split35-outage（build-34）：check_netem_outage 11/11 PASS，mismatch 0、terminated 0；10s 斷線 stalled→recovered，40s 斷線 p6 link lost。
H03～H06（阿松、阿市、稻姬、淺井長政）：source-index 著陸了通用能力（cooldown refund、英雄隱形、no unit collision、tether、
  ally_area_restore、英雄 UnitScript 註冊）；第 5 技能欄（ward，CABI 38，X 鍵）與 auto_learn；6 個 kit 都已 hero_author build。
  base_content 英雄測試 47 passed，release parity soak 4/4。autopilot 補上 ally／toggle 的施放規則（source-index）。
  逐招判定工具 scripts/check_hero_live_acceptance.lua；split36（p1 oichi、p6 omatsu）進行中。
UE 測試全量 `Om.`：286/286 Success，`-d3d12 -NoSound`（ue-tests-39-rhi.log）與 `-nullrhi`（ue-tests-38.log）兩種模式都通過（E079）；需要 RHI 的段落在 headless 下以 warning 跳過，缺少 harness 的 7 個測試以 warning 跳過，content gap 以 warning 回報。
盲測（G5，由無上下文的 agent 只依 hero-authoring-guide 與 source-data 製作，不改共用引擎）：
  I1 C15 tamako：Q、E 有 Rust handler 並通過；W（ground_area_line）、R（retarget_on_death 等）標為 needs_framework。
  I2 B29 murakami_yoshikiyo：Q、W、R 有 Rust handler，17 tests 與 7200 tick soak 通過；E（ally_area_buff_query）標為 needs_framework。
  兩位都有自訂 Rust handler，缺口都如實回報 → G5 條件成立；guide 回饋共 18 條 → 交給 source-index 修正。
  2026-10-26：I1/I2 引擎批次著陸後，正式 `hero_author build --execute` 兩者都 OK（registry、ids、build_ue、verify_staged、compiled_contract）。途中修了 E075（孤兒佔位頭像）和 E076（stuck recover 的 UE 編譯錯誤）。
Shipping：package-shipping-4 ✓（28 個檔案、713MB、沒有 pdb；E065 測試開關在 Shipping 下關閉）；2 分鐘冒煙排在 split36 之後。
F3／F4 美術槽位與 unreal-art-guide.md（ward）；月末文件草稿：release-notes、next-month、player-guide、hero-authoring-guide。
使用者新需求：滑鼠預設鎖在視窗內、單按 Ctrl 切換（ward）；道具圖示用 grok 生成（子 agent 進行中）。
```

## 2026-10-25（J1 斷線實機 split33；第 5 技能欄；著陸覆蓋事故）

```text
split33（build-32，quick map，wan-80-20-loss1 seed 7，斷線 1:90:10、1:240:10、2:420:40）check_netem_outage：10／11 PASS。
  兩段 10s 斷線都 stalled→recovered（實際恢復花了 14.5s／11.25s，KCP 退避）；40s 斷線判定 p6 link lost 一次、server KCP link lost ✓、
  沒有 terminated。FAIL：1 次 mismatch（tick 6000，team 2，召喚物 gunner 的 ATTACK 揭露，發生在 team 1 恢復的那一刻，
  伴隨 retargeted late input）→ 交給 omfx-decouple。
UE STALLED 顯示「連線中斷，等待恢復」並暫停輸入：被 ward 的 23:07 著陸覆蓋（E062），已補回，等下次建置驗證 MatchTerminationRefusesInput。
第 5 技能欄 extra_action_slot（ward，CABI 38，按鍵 X）與 auto_learn 已著陸；UE 新測試等下次建置。
規則：禁止依程序映像名稱 taskkill（兩次事故）；著陸必須以 3-way merge 進行（E062）。
```

## 2026-10-25（使用者指示：測試局 ≤10 分鐘、塔要推得掉、小兵面向）

```text
測試局：run_moba_split_local 新增 --max-seconds；實機測試一律帶 --max-seconds 600。
quick 測試地圖 three_lane_quick_training（配方 moba_two_player_quick.lua）：小兵與 super 傷害 ×3；
  地圖可設定 tower_hp／base_hp（E055：三個 config 建立點都要接）。目前數值：塔 400、兵營 300、主堡塔 250、主堡 400，
  兵營與主堡塔的重生時間為 600 秒。
split26／27／28（10 分鐘）：塔分別倒 8／11／11 座，但都沒推到主堡。瓶頸是三層塔使得對等的雙方互相抵消，不是塔的血量
  → quick 地圖改為每路只保留最外層一座塔（tower_layers={2200}）。
split29（build-31）✓：7:00 時 phase=finished、finished_tick=25225、winner_side=0（team 1）；整場 mismatch 0、terminated 0；
  p1／p6 都記錄 match_finished，結算面板顯示「敗北 · 勝方：隊伍 1」與離開對局按鈕（p6 雜賀孫市 6 級）。
  實機測試的基準：quick 地圖＋--max-seconds 600，可以在 10 分鐘內驗到推塔、基地、結束與結算這一整段流程。
split30（p1 training_channeler）：約 10 分鐘內 team 2 的兵營、主堡塔、主堡依序倒下而自然結束；run_moba_split_local 在每個 renderer
  都記錄 match_finished 後 15 秒自動收局 ✓；引導 completed 5、recall 1、cancelled 0；mismatch 0。
J1（部分）✓ split31：--netem wan-80-20-loss1 --netem-seed 7（雙隊各自經 proxy），約 3.6 分鐘時 team 2 獲勝自然結束；
  mismatch 0、terminated 0、late input 0。待補：斷線恢復情境（schedule J1 的斷線部分）、proxy 收掉前輸出掉包統計。
連續 4 局（split27／28／30／31）0 desync。
J4（部分）：launcher 新增 --resolution WxH（預設 1280x720，範圍 640x480～7680x4320）。split32 要求 1920x1080，對局在約 5 分鐘
  時 team 2 獲勝自然結束，mismatch 0；HUD（command bar 置中、小地圖右下、隊友卡左上）沒有裁切或重疊。
  autopilot 截圖實際只有 1423x889：可能是 OS DPI 縮放，或視窗被限制在螢幕工作區內，不影響排版。
  OS 100%／150% DPI 屬於系統設定（不得自行修改）→ 留給使用者手動切換後驗收。連續 5 局 0 desync。
```

```text
H03～H06 能力缺口分派：冷卻調整 API、英雄隱形 → source-index；第 5 技能欄 extra_action_slot（G-3）→ ward-canonical。
split27：E056 修正後整場 mismatch 0、terminated 0、late input 0（第一場跑滿 10 分鐘都沒有 desync 的對局）。
小兵面向（E057，ward）：權威端 NPC 沒有 Facing → 改用 face_toward（位移或攻擊方向），出生時也帶 Facing。
  Om.Generated.EveryUnitClassFacesActorForward ✓；split28 的 creep_facing_probe 中，move、actor、visible 三個 yaw 完全一致 ✓。
```

## 2026-10-25（J6：打包版從獨立目錄實機對戰 ✓）

```text
launcher 新增 --packaged-game <archive|OmGame.exe>（moba_role_launch／run_moba_split_local），會拿掉 uproject 與 -game，並不做 shader 預熱。
package-dev-2：能連線並開打，但地面、英雄模型、道具圖示都沒被 cook（E054）→ DirectoriesToAlwaysCook 加入 /Game/Omoba、/Game/OmGenerated，
  並新增 scripts/tests/ue_cook_coverage_test.lua。
package-dev-3（ucas 155→261MB）：split23 用兩個打包版 OmGame.exe 對 dedicated server，草地與英雄模型都正常，autopilot 指令被接受；
  剩下的載入失敗只有 profiler DLL 和引擎的 UnlitText（名牌照常顯示）。
```

## 2026-10-25（split20／21：引導 parity ✓、終止面板 ✓、野怪觸發的 attack 揭露 desync 是目前主要阻擋）

```text
split20（build-24）：引導 start／pulse 在權威端與 replica 都有執行（E053 ✓）；tick 27120／27240 同一個 entity 連續兩次 DISCLOSED_ATTACK 不一致 → terminated（第 7 分鐘）。
split21（build-25）：9 個 Editor 測試全部 Success；終止面板實機正常顯示（「比賽已終止：伺服器偵測到狀態不同步」＋離開對局，單位仍在畫面上）。
  tick 30000 兩隊同時出現 DISCLOSED_ATTACK 不一致 → terminated（第 8.3 分鐘）。
  規律：attack／hero 揭露的 mismatch 前 1～2 秒都有野怪被擊殺（blue_wolves、blue_raptors、north_guardian、jungle_wolf_small），5／5 吻合 → 交給 omfx-decouple，為最高優先。
autopilot：引導中不送新指令（低血量只放行 bRetreat 撤退或回城）、施法間隔 0.8 秒、殘血遇到 3 隻以上敵方小兵會撤退（下一次建置生效）。
```

## 2026-10-25（split19：M1 對局自然結束 ✓）

```text
split19（build-23；p1 預設英雄、p6 saika_magoichi；B+C+C、2 個 UE、8 個 bot）：36:15 時 phase=finished，finished_tick=130508，
  winner_side=1（team 2）。
  過程：side 0 的 9 座塔全倒 → 兵營 ×2（一座被雜賀眾打倒）→ 主堡塔（19:10，雜賀孫市與 lane_caster）→ 主堡。
  p1 看到結算面板：「敗北 · 對戰結束 · 勝方：隊伍 2」、K/D/A 3/44/0，有「離開對局」按鈕
  （截圖 target/mvp-a3/split19/p1/autopilot-p1-shot-037-result.png）；autopilot 記錄 match_finished。
  E052 實機驗證：late InputSubmit 0 筆（split18 是 159），10054 次數為 0，查詢正常。
  mismatch：tick 10800 一次（DISCLOSED_FACING）、tick 128280 兩隊各一次（0x464f4702，剛好在主堡塔倒下後），都已 rebase 恢復，沒有終止。
  後續：autopilot p1 死了 44 次，撤退判斷太弱；match_status 新增 base_structures／base_unlocked_by_side（已編譯，下次建置後生效）。
```

## 2026-10-25（split14–16：W/E/R 送錯欄位、引導 desync、比賽終止 UI）

```text
split14/15（training_channeler）：W/E/R 一律送錯欄位（R→2、W→0），根因是舊 TD 藍圖的輪詢輸入圖（E051）
  → 新增 OmOwnedInput::FOwnerDispatchScope 防護；split16 實機已看到 OM_CAST slot=3，藍圖的施法被拒。
autopilot：技能先各學一級再加深；沒有英雄目標且法力至少 70% 時對小兵施法（E049）。
Editor 測試（build-21）：AutoPilot×2、NativeHeroPresentation、RejectReasonText、ActionFeedbackCorrelation、
  MatchTerminationRefusesInput、NativeMatchResult 都是 Success；MatchTerminationText 因大小寫規則不一致而失敗，已交給 ward。
split16（p1 training_channeler、p6 saika_magoichi）：lumen_tremor 的 pulse 造成 DISCLOSED_HERO（0x464f4705）mismatch，
  tick 3720 rebase、6480 terminated。AbilityChannel 只在一個 world 執行 → 交給 ward 修正 replica 端的 channel parity。
  終止流程實機生效（session_terminated），但截圖只拍到「連線失敗」沒拍到面板 → 已交給 ward。
  雜賀孫市：b01_sniper_mode 已學習並施放。
split17（p6 saika，沒有引導技）：前 9.3 分鐘沒有 mismatch，tick 33600 出現一次 DISCLOSED_ATTACK（0x464f4706），由 rebase 恢復 → 交給 omfx-decouple。
  雜賀孫市的 Q／W（召喚雜賀眾）／R 實機都有施放，E 是被動技能，依規則略過。
split18（build-22）面向結論 ✓：female_peasant 的 head_rise 11614cm、head_spread 1736cm、pitch／roll 都是 0 → 模型直立。
  俯視鏡頭（pitch -90）下，畫面上最長的是張開的雙臂，雙臂和面向垂直；修正前面向偏 90°，雙臂沿著畫面上下，看起來才像站著。
  visible_front 等於 move（90.3 對 94.8、44.9 對 43.9）。
  saika 的骨架沒有 toe／ball → 新增臉部骨頭（Jaw 等）作為第二順位的正面依據（Jaw 在 +Y、Cape 在 -Y），推算結果和現有的 yaw -90 一致。
```

## 2026-10-25（split12：塔開始受傷；角色面向通用修正；lane 對應修正）

```text
split12：預設英雄，帶 split11 之後的所有修正。11 波時塔數 9:9，lane 2 第 0 層塔兩邊都受傷（848／1094），對局有在推進；
  為了套用下列修正而主動停止。
  lane 事件：assigned=top、match=first → route id 是 route_<i>（照 authored lanes 順序），不是 lane 名稱。
  修正：launcher 用 M.authored_lane_index 從 templates/moba_maps.lua 算出索引，再傳 -om-autopilot-lane-index。
角色面向（E040）：female_peasant 腳尖朝 -Y → 新增 AOmAnimatedUnitActor::ComputeSkeletalForwardYaw（reference pose、toe／ball），
  bAutoAlignMeshForward 預設 true；只有 Unreal 端明確寫了 yaw 才關閉。codegen 測試 86/86 通過，Editor 測試已改成驗證對齊結果。
  驗證方式：split13 的 facing_probe 中，visible_front_world_yaw_deg 應等於 move_yaw_deg，並搭配 1200cm 特寫截圖。
launcher：shader warmup 改由 plan.shader_warmup 控制（E039），test_moba_role_launch 10/10 通過。
split13（build-18 stable；desync 修正、G-2 channel、ability_channel 進度條都已納入）：
  lane：match=authored-index，route 0（top）✓。
  面向 ✓（數值）：skeleton_forward=-90 → mesh_relative=+90；visible_front 90.1 對 move 94.0、-172.9 對 178.7。
  特寫截圖拍不到英雄：FocusOwnedHero 只定位一次，英雄在 1.2 秒內走出 1200cm 畫面。已改成每個 tick 追蹤 actor，
  並新增 facing_frame（camera_to_hero_cm）；下一次建置時生效。
  對局：77 波／38 分鐘時塔數 4:3（有推進，但主堡始終滿血）；42.5 分鐘（tick 152999）出現 observer-mismatch-after-rebase → 已停止。
  前 42 分鐘沒有 desync，可見 E037 修正有效，這次是另一條罕見路徑。已交給 omfx-decouple：補 mismatch 元件診斷、
  用 parity 框架重現（道具主動技、lumen_mend、死亡後傷害）、終止時明確通知 client（client 原本卡在 153003 且沒有任何提示）。
  autopilot（p1，42.5 分鐘）：submit 1193、no_effect 12、not_submitted 10、死亡 23；projection_failed 165，原因待查。
```

## 2026-10-25（split10／split11：autopilot 打法改善；對局在基地前停滯）

```text
split10（預設英雄）：沒有 desync；死亡判斷生效；facing_probe：actor／mesh yaw 和移動方向一致（90.1 對 94.0、-175.5 對 178.7），
  mesh 相對 yaw 為 0；模型 female_peasant 沒有 toe 骨頭，骨架判斷尚未得出結論（已改成列出骨頭名稱並以 1200cm 特寫）。
split11（撤退、被動略過、推塔需小兵陪同）：7.4 分鐘時死亡次數由 8/6 降到 4/1；42 分鐘塔數 4：2，主堡始終滿血 1600 → 停滯，停止。
  根因假設：配方把 player 1、6 都指派到上路，但 autopilot 一律走中路 → 上路無人推進、中路兩人。
  修正：launcher 傳 -om-autopilot-lane=<plan lane>，autopilot 依指派分路選 route（trace 的 lane 事件）。
其他 autopilot 修正（待 split12 驗證）：商店依當下狀態選擇要買的物品和要賣的格子；點擊時略過停用或被裁切的按鈕，並捲動清單；
  購買和賣出之間等收據；ping 改為本機溝通操作。
H02 desync 根因（source-index 找到）：dispatch.rs 只在有 MobaMatch 時執行施法前搖；filtered replica 沒有 MobaMatch → cast_time > 0 的技能
  在 replica 端立即施放（saika_reinforcements 也會中）。omfx-decouple 修正中；source-index 會補上所有前搖技能的通用回歸測試。
```

## 2026-10-25（H02 安裝；split8／split9 的發現）

```text
H02 伊達政宗：hero_author build --execute → exit 0（registry、ids、build_ue、verify_staged、compiled_contract）。
  實機（split9）：伊達政宗可以進場，Q b04_flame_trail_dash 被權威端接受並結算（AbilityCast ok）。
  **但施放後的下一個 tick，server 回報 observer-mismatch-after-rebase，secure match 被終止**（split9 server log 第 4185～4204 行）。
  → 已交給 omfx-decouple 當最高優先：deterministic 重現測試 + 通用修正（腳本產生的投射物和位移，在權威端與 replica 必須一致）。
  → 產品問題：match 被終止後，client 沒有明確的錯誤或結束畫面，畫面只是靜止（待補）。
split8：renderer 在 120 秒時自己退出 → E034（-om-...-seconds= 撞到引擎的 SECONDS=），已修並加上保護。
  截圖顯示整個世界是線框 → E035（F1～F5 撞到 UE 開發版的 debug 檢視鍵，真人玩家也會遇到），已在 DefaultInput.ini 清除。
  B3：商店購買成功（截圖可見「購買：成功」）；賣出按鈕在清單下方 → autopilot 改成會捲動清單。
match_status（MCP）實機可用：主堡血量、每位英雄的 KDA、血量、位置與重生。
角色面向：截圖距離太遠看不出來 → autopilot 新增 facing_probe（記下移動方向、權威 facing、actor／mesh yaw，並拍特寫），在 split10 驗證。
```

## 2026-10-25（A5 accepted：第一次 Windows 打包）

```text
命令：tools\lua\lua.exe scripts\package_ue_moba.lua --ue-root D:/UE5.8 --output target/ue-package/a5-first → exit 0
結果：BuildCookRun Development／Win64，cook 時 0 error、4 warning；archive 3.3 GB、50 個檔案。
  必要角色都在：Windows/OmGame.exe、om/Plugins/OmRuntime/Binaries/Win64/om_bridge.dll、base_content.dll、
  om/Content/Paks/om-Windows.pak。target/ue-package/a5-first/package-report.json 記錄了每個檔案的 SHA-256。
第一次打包要編譯整個 Game target（1446 個 action），和對局同時跑時明顯變慢。
尚未驗證：從乾淨目錄啟動並進入對局（J6）；Shipping 設定（autopilot 不能被編進去）。
```

## 2026-10-25（split7：58 分鐘沒有自然結束；MCP 狀態工具；5 分鐘例行檢查）

```text
split7（2 個 autopilot＋8 Bot）在 58 分鐘時由主 session 停止，沒有自然結束：
  team 1 已推進敵方基地，team 2 只剩基地前兩座塔、兵營與主堡；team 1 自己掉了 3 座外塔。
  對照：全 Bot 的 headless 100 局都會自然結束（finish_tick 27202～84070，約 7.5～23 分鐘）。
  判斷：G2 的風險在 autopilot 打得比 Bot 弱，不是規則有死局。已知成因：
    (a) 死亡判斷只看 hp，抓不到死亡 → 撤退、購物、重生後的流程都沒有觸發（已改看 respawn_remaining，待建置）；
    (b) 選目標時塔被排在小兵之後，不會主動推塔；
    (c) E 技能反覆 no_effect；還有 2 點技能點沒花掉。
  下一步：新 server 的 match_status 會直接列出主堡血量、擊殺與重生，用來確認。
新工具：
  - omb-mcp（.mcp.json 的 omoba-status）：match_status（需 server 加 --debug-query；split 拓撲預設開啟）、
    run_status（只讀 run 目錄）、list_players、inspect_player_view；CLI 為 `omb-mcp.exe --once ...`。
  - server 的所有 state query 改為預設關閉（OMB_DEBUG_QUERY=1 才回應）：修掉玩家連線可以讀取未過濾狀態的漏洞。
  - 每 5 分鐘的例行檢查（session cron）。
角色面向（使用者回報）：後端 facing 正確（移動與攻擊時都會轉向），UE yaw = atan2(y, x) 也一致；
  問題在模型前方軸向：codegen 沿用 Fyrox 的 render.yaw_offset_deg（例如 saika -90），訓練英雄的 explicit 設定則都是 0。
  下一輪會用 autopilot 截圖（俯視）比對模型朝向和移動方向，再在 codegen 修正換算規則。
```

## 2026-10-25（B1／B2 實機通過：真輸入 → ACK → 權威套用）

```text
目前工作 ID／狀態：B1 accepted、B2 accepted（L3a-loopback）；B3 doing；A3 accepted
證據：target/mvp-a3/split7/（run_moba_split_local.lua --no-build --autopilot lane_v0）
  - 兩個 client（p1 team 1、p6 team 2）都經合成 Slate 輸入送出，ACK accepted，server log 也確認套用：
    move_route、attack_unit、stop、hold、attack_move、attack_move_queued、move_only（經改鍵 API 綁到 Ctrl+右鍵）、
    cast_q_target、cast_e_target、cast_r_instant、upgrade_r。
  - server（split7/server/server.stdout.log）pid=1：UpgradeAbility 6 → AbilityUpgrade ok 6；CastAbility 18 → AbilityCast ok 18。
  - 兩個 client 都沒有 not_submitted；no_effect 只有 1 次 cast（待查）。
  - ward：split3～split7 的 runtime log 都沒有 NONCANONICAL（修正前一小時 6834 次）。
原 11 主項：3.1、3.3、3.4、4.3 都有 L3a-loopback 的 request → ACK → 套用證據（UI 觀察欄位仍偏少）；3.5 auto attack、hidden target 待補。
本批修正（errors.md）：E021 鍵盤非同步、E022 角落英雄、E023 ACK ≠ 套用、E024 modifier 合成；
  另外加了 HUD 區域判斷（正式的 pointer 分類器）、合成視窗焦點、改鍵 API 綁定 optional action。
工具：scripts/run_moba_split_local.lua（單一指令跑 B＋C＋C）。
```

## 2026-10-25（B1 autopilot v0、正式啟動路徑的修正）

```text
目前工作 ID／狀態：B1 doing（程式完成、UE 編譯通過，等實機 trace）；A3 doing
無人值守：使用者 2026-10-25 授權 72 小時，不再找使用者確認；錯誤一律記到 errors.md。
玩家或作者新增能做到的事：
  - run_moba_role_ue.lua --autopilot PROFILE：每個本機 renderer 由 FOmAutoPilot 合成真實 Slate 輸入，
    沿公開 lane route 前進、攻擊揭露的敵人；trace 寫到 autopilot-p<id>.jsonl，統計寫到 .coverage.json。
  - 啟動多個 renderer 前會自動暖 shader（ue_shader_warmup.lua），實測 191 秒，之後由 stamp 跳過。
  - 實機前用 scripts/consistent_build.lua 一次建好 server、runtime 與 UE，原始碼在建置中變動就整組重建。
  - 缺美術的英雄自動拿到共用的佔位頭像，不再擋住建置。
修正的正式路徑錯誤（詳見 errors.md）：
  E012 bridge 沒有 logger；E013 被拒後零延遲重連；E019 runtime 沒送 content hash（正式啟動路徑原本一律失敗）；
  E018 共用 checkout 的建置競爭；E020 新英雄缺頭像擋住建置；E003 冷 shader cache。
平行 session：
  - source-index：D3／D4／E1／E2 進行中（work-D.md、work-E.md）。
  - omfx-decouple：G-1 技能投射物進行中；已把協定升到 16，並新增 training_frost_mage 當 demo。
  - ward-canonical：E017 已修（server 用 store tick 封裝 owner ward），待實機驗證（work-ward.md）。
修改檔（本批，主 session）：
  omfue/Plugins/OmRuntime/Source/OmRuntime/{Public,Private}/OmAutoPilot.*、OmPlayerController.*、OmRuntimeBridgeSubsystem.h、
  OmGenerated/Private/OmWorldBridgeActor.cpp、OmEditor/Private/OmAutoPilotTests.cpp、
  omfue/bridge/src/{bridge_log.rs,driver.rs,lib.rs}、omoba-client-runtime/src/config.rs、
  scripts/{moba_role_launch.lua,run_moba_role_ue.lua,ue_shader_warmup.lua,consistent_build.lua,
  ue_hud_portrait_coverage.lua,sync_ue_hud_manifest_labels.lua}、omfue/Content/Omoba/UI/hero-placeholder-v1.png、
  以及對應的 scripts/tests/*。
測試：host split、shader warmup、consistent build、portrait coverage（33）、server build（5）、
  client runtime config（3）、bridge_log（2）全部通過；UE 編譯 Result: Succeeded。
```

## 2026-10-25（A3 進行中、平行 session 結果）

```text
目前工作 ID／狀態：A3 doing；D1、D2 accepted；F2 accepted（只到 L1）
玩家或作者新增能做到的事：
  - 單機 B＋C＋C 三次獨立呼叫可以一路走到「雙方 admit、兩個 UE 啟動」：
    server-only：--server-only --no-build --recipe scripts/lua_data/moba_two_player.lua --host-simulations 3 --host-renderers 2
    client：--connect 127.0.0.1 --no-build --recipe <server>/match-plan.json --local-player 1|6 --host-simulations 3 --host-renderers 2
  - presentation 分別綁 127.0.0.1:57062（p1）與 57067（p6）。
本次修改：
  - scripts/moba_server_build.lua：dedicated server 的 DLL 驗證改為接受「任一 profile 的最新建置」。
    原因：build_ue_moba 放上 debug 版 scripts/base_content.dll，而 --server-only release 要求 release 版，
    同機 B＋C＋C 兩邊不可能同時通過。stale copy 仍會拒絕。測試加在 scripts/test_moba_server_build.lua（5 groups passed）。
命令：
  - cargo build omobab／moba-config／client-runtime（release、compiled-content-only）→ 0
  - tools\lua\lua.exe scripts\build_ue_moba.lua --ue-root D:/UE5.8 --build-only → 0
    （base_content d4ce83d1…、om_bridge 2329e489…）
  - scripts/tests/role_config_command_integration_test.lua → 0（3 groups）
  - cargo test omoba-core --lib production_guards → 0（13 passed；驗證 remove-omfx-include-str session 的改動）
資源量測（24 邏輯核心；UE 仍在第 0 幀）：omobab 約 265→421 MB、每個 client runtime 約 33→109 MB、每個 UE 約 1.55 GB。
A3 發現：
  1. 冷 shader cache：兩個 UE 各自在第 0 幀編譯 shader 超過 45 分鐘。-corelimit=6 讓全機只有 6 個 ShaderCompileWorker，
     兩個實例重複編譯同一批 shader。效能量測與每日迭代前要先暖好 DDC（只開一個 UE、不加 corelimit）。
  2. 第一次 server 啟動失敗、重試也失敗的原因就是上面的 DLL profile 衝突（不是平行 session 造成）。
  3. 這段時間 server 照常推進（Bot 在打），client 端還沒有任何畫面證據。
L3：未取得（UE 尚未出第一幀）。
```

## 2026-10-25（A1／A2／A4）

```text
日期：2026-10-25
目前工作 ID／狀態：A1 accepted、A2 accepted、A4 accepted、A3 doing
玩家或作者新增能做到的事：
  - 同一台機器可以分三次獨立呼叫跑 B＋C＋C，兩個 --connect client 不再搶同一個 presentation port；
    三次呼叫傳相同的 --host-simulations 3 --host-renderers 2，就共用同一份 CPU 預算。
  - headless 批次可以跑超過 10 局（預設仍 10 局，上限 10000）。
  - 新增雙真人配方 scripts/lua_data/moba_two_player.lua（player 1 = team 1、player 6 = team 2，其餘 8 席 Bot）。
repo HEAD／本次修改：
  - 主 repo b1f00b9（工作開始時 tracked 檔案乾淨；docs/acceptance/ 有約 2010 個 untracked 證據檔，未動）
  - omb 7f26187（乾淨）、omfue d311ad4（乾淨）
  - 修改：scripts/moba_role_launch.lua、scripts/run_moba_role_ue.lua（help）、
    scripts/moba_headless_batch.lua、scripts/run_moba_headless_batch.lua（help）、
    scripts/tests/moba_headless_batch_test.lua
  - 新增：scripts/tests/moba_role_launch_host_split_test.lua、scripts/lua_data/moba_two_player.lua
來源英雄與 variant：—
已執行命令、exit code、build identity：
  - tools\lua\lua.exe scripts\tests\moba_role_launch_host_split_test.lua → 0
  - tools\lua\lua.exe scripts\tests\moba_host_budget_test.lua → 0（968 allocations）
  - tools\lua\lua.exe scripts\tests\moba_headless_batch_test.lua → 0（11 tests）
  - 工具：rustc 1.95.0、cargo 1.95.0、UE 在 D:\UE5.8
L1／L2／L3／L4 證據（fresh／沿用／未執行）：
  - L1 fresh：上列 Lua 測試。
  - L1 沿用：Rust／UE 建置基準沿用 2026-10-25-minion-cadence（2,163 passed／0 failed／9 ignored）。
    之後的 commit（召喚指令、技能整合，到 b1f00b9）沒有重跑全套。
  - L3／L4：未執行（A3 進行中）。
gate 與原 11 主項的變化：G0 進行中（A5、A6 未完成）；原 11 主項無變化。
失敗、原因、尚未修的風險：
  - scripts/tests/moba_role_finish_observer_test.lua 在第 80 行失敗（moba_role_launch.lua 的 "unknown launch mode"）。
    用 HEAD 版 moba_role_launch.lua 預載重跑，結果相同 → 既有問題，不是 A2 造成；測試 fixture 的 plan 缺 mode。尚未修。
  - scripts/tests/role_config_command_integration_test.lua 需要 release 版 moba-config.exe（環境前置）；
    scripts/tests/moba_managed_role_test.lua 需要輸出目錄參數。A3 建置後再跑。
  - omfue/.../OmActionOwnerTests.cpp 註解中的 "quota 10/10" 是輸入 ID 配額，不是對局數，不改。
下一個工作 ID：A3（B＋C＋C 進場）→ A5 → B1
人工／硬體需求與是否已取得：A6 未做。
owned process／資源清理結果：本批未啟動對局。
```

### A4 改動位置

- `scripts/moba_headless_batch.lua`：`MAX_MATCHES` 由 10 改為 10000（只是合理性上限），新增 `DEFAULT_MATCHES = 10`；錯誤訊息改用常數。
- `scripts/run_moba_headless_batch.lua`：help 改為 `--matches 1..10000 (default 10)`。
- 其他 launcher（`run_moba_role_ue.lua`、`run_moba_session_ue.lua`）沒有找到對局數上限。
