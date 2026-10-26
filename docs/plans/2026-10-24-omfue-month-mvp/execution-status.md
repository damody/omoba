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

## 2026-10-26（J1 通過、H03～H06 著陸、盲測 I1／I2、Shipping 打包）

```text
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
