# census-v3（已移到正式位置）

2026-10-29 起，普查寫的 v3 文件就是正式的作者來源，這個資料夾不再放任何文件：

- 英雄：`docs/character_pipeline/packages/<package>/v3.lua`（下表）。10 位已上架英雄的 `kit.lua` 已和普查文件合併成同一份 `v3.lua`：數值與說明照現行移植（M1），行為照原作，`coverage` 在 M5 切換前維持 `rust_handler`。
- 道具：`docs/character_pipeline/items/*.lua`（`batch_I0`～`I5`、`pilot_a`、`pilot_b`）。原本的 `shelf_a.lua`、`shelf_b.lua` 已併入這幾份，每件道具只出現一次。
- 工具一律經 `docs/character_pipeline/tools/v3_sources.lua` 找文件；跨文件登錄表在 `docs/character_pipeline/schema/v3-shared-registry.lua`（`registry-additions-*.lua` 已在 M4 合併進去）。

下面保留普查期間的批次與結果紀錄；文中的 `census-v3/heroes/<ID>.lua` 對應下表的位置。

| source id | 文件 | 狀態 |
|---|---|---|
| A01 | [tokugawa_ieyasu](../../../character_pipeline/packages/tokugawa_ieyasu/v3.lua) | 未上架 |
| A02 | [inahime](../../../character_pipeline/packages/inahime/v3.lua) | 已上架 |
| A03 | [ishikawa_kazumasa](../../../character_pipeline/packages/ishikawa_kazumasa/v3.lua) | 未上架 |
| A04 | [takenaka_shigeharu](../../../character_pipeline/packages/takenaka_shigeharu/v3.lua) | 未上架 |
| A05 | [sakai_tadatsugu](../../../character_pipeline/packages/sakai_tadatsugu/v3.lua) | 未上架 |
| A06 | [ii_naomasa](../../../character_pipeline/packages/ii_naomasa/v3.lua) | 未上架 |
| A07 | [honda_tadakatsu](../../../character_pipeline/packages/honda_tadakatsu/v3.lua) | 已上架 |
| A08 | [sakakibara_yasumasa](../../../character_pipeline/packages/sakakibara_yasumasa/v3.lua) | 未上架 |
| A09 | [hachisuka_masakatsu](../../../character_pipeline/packages/hachisuka_masakatsu/v3.lua) | 未上架 |
| A10 | [hirate_masahide](../../../character_pipeline/packages/hirate_masahide/v3.lua) | 未上架 |
| A11 | [saito_dosan](../../../character_pipeline/packages/saito_dosan/v3.lua) | 未上架 |
| A12 | [otani_yoshitsugu](../../../character_pipeline/packages/otani_yoshitsugu/v3.lua) | 未上架 |
| A13 | [hattori_hanzo](../../../character_pipeline/packages/hattori_hanzo/v3.lua) | 未上架 |
| A14 | [maeda_toshiie](../../../character_pipeline/packages/maeda_toshiie/v3.lua) | 未上架 |
| A15 | [nene](../../../character_pipeline/packages/nene/v3.lua) | 未上架 |
| A16 | [torii_mototada](../../../character_pipeline/packages/torii_mototada/v3.lua) | 未上架 |
| A17 | [sassa_narimasa](../../../character_pipeline/packages/sassa_narimasa/v3.lua) | 未上架 |
| A18 | [inaba_ittetsu](../../../character_pipeline/packages/inaba_ittetsu/v3.lua) | 未上架 |
| A19 | [kuroda_yoshitaka](../../../character_pipeline/packages/kuroda_yoshitaka/v3.lua) | 未上架 |
| A20 | [kuki_yoshitaka](../../../character_pipeline/packages/kuki_yoshitaka/v3.lua) | 未上架 |
| A21 | [zuiryuin_nisshu](../../../character_pipeline/packages/zuiryuin_nisshu/v3.lua) | 未上架 |
| A22 | [sengoku_hidehisa](../../../character_pipeline/packages/sengoku_hidehisa/v3.lua) | 未上架 |
| A23 | [ukita_hideie](../../../character_pipeline/packages/ukita_hideie/v3.lua) | 未上架 |
| A24 | [sakuma_morimasa](../../../character_pipeline/packages/sakuma_morimasa/v3.lua) | 未上架 |
| A25 | [oda_nobunaga](../../../character_pipeline/packages/oda_nobunaga/v3.lua) | 未上架 |
| A26 | [nohime](../../../character_pipeline/packages/nohime/v3.lua) | 未上架 |
| A27 | [shibata_katsuie](../../../character_pipeline/packages/shibata_katsuie/v3.lua) | 未上架 |
| A28 | [hashiba_hideyoshi](../../../character_pipeline/packages/hashiba_hideyoshi/v3.lua) | 未上架 |
| A29 | [ishida_mitsunari](../../../character_pipeline/packages/ishida_mitsunari/v3.lua) | 未上架 |
| A30 | [niwa_nagahide](../../../character_pipeline/packages/niwa_nagahide/v3.lua) | 未上架 |
| A31 | [mori_ranmaru](../../../character_pipeline/packages/mori_ranmaru/v3.lua) | 未上架 |
| A32 | [takigawa_kazumasu](../../../character_pipeline/packages/takigawa_kazumasu/v3.lua) | 未上架 |
| A33 | [honda_masanobu](../../../character_pipeline/packages/honda_masanobu/v3.lua) | 未上架 |
| A34 | [omatsu](../../../character_pipeline/packages/omatsu/v3.lua) | 已上架 |
| A35 | [shima_sakon](../../../character_pipeline/packages/shima_sakon/v3.lua) | 未上架 |
| A36 | [maeno_nagayasu](../../../character_pipeline/packages/maeno_nagayasu/v3.lua) | 未上架 |
| B01 | [saika_magoichi](../../../character_pipeline/packages/saika_magoichi/v3.lua) | 已上架 |
| B02 | [fuma_kotaro](../../../character_pipeline/packages/fuma_kotaro/v3.lua) | 未上架 |
| B03 | [saito_tomonobu](../../../character_pipeline/packages/saito_tomonobu/v3.lua) | 未上架 |
| B04 | [date_masamune](../../../character_pipeline/packages/date_masamune/v3.lua) | 已上架 |
| B05 | [mori_motonari](../../../character_pipeline/packages/mori_motonari/v3.lua) | 未上架 |
| B06 | [sanada_yukimura](../../../character_pipeline/packages/sanada_yukimura/v3.lua) | 已上架 |
| B07 | [ashikaga_yoshiaki](../../../character_pipeline/packages/ashikaga_yoshiaki/v3.lua) | 未上架 |
| B08 | [asai_nagamasa](../../../character_pipeline/packages/asai_nagamasa/v3.lua) | 已上架 |
| B09 | [sanada_masayuki](../../../character_pipeline/packages/sanada_masayuki/v3.lua) | 未上架 |
| B10 | [miyoshi_nagayoshi](../../../character_pipeline/packages/miyoshi_nagayoshi/v3.lua) | 未上架 |
| B11 | [kikuhime](../../../character_pipeline/packages/kikuhime/v3.lua) | 未上架 |
| B12 | [yamamoto_kansuke](../../../character_pipeline/packages/yamamoto_kansuke/v3.lua) | 未上架 |
| B13 | [momochi_sandayu](../../../character_pipeline/packages/momochi_sandayu/v3.lua) | 未上架 |
| B14 | [katakura_kagetsuna](../../../character_pipeline/packages/katakura_kagetsuna/v3.lua) | 未上架 |
| B15 | [imagawa_yoshimoto](../../../character_pipeline/packages/imagawa_yoshimoto/v3.lua) | 未上架 |
| B16 | [mochizuki_chiyome](../../../character_pipeline/packages/mochizuki_chiyome/v3.lua) | 未上架 |
| B17 | [kirigakure_saizo](../../../character_pipeline/packages/kirigakure_saizo/v3.lua) | 未上架 |
| B18 | [koihime](../../../character_pipeline/packages/koihime/v3.lua) | 未上架 |
| B19 | [usami_sadamitsu](../../../character_pipeline/packages/usami_sadamitsu/v3.lua) | 未上架 |
| B20 | [yamagata_masakage](../../../character_pipeline/packages/yamagata_masakage/v3.lua) | 未上架 |
| B21 | [baba_nobufusa](../../../character_pipeline/packages/baba_nobufusa/v3.lua) | 未上架 |
| B22 | [taigen_sessai](../../../character_pipeline/packages/taigen_sessai/v3.lua) | 未上架 |
| B23 | [ise_hime](../../../character_pipeline/packages/ise_hime/v3.lua) | 未上架 |
| B24 | [akiyama_nobutomo](../../../character_pipeline/packages/akiyama_nobutomo/v3.lua) | 未上架 |
| B25 | [honganji_kennyo](../../../character_pipeline/packages/honganji_kennyo/v3.lua) | 未上架 |
| B26 | [okuma_tomohide](../../../character_pipeline/packages/okuma_tomohide/v3.lua) | 未上架 |
| B27 | [kojima_sadaoki](../../../character_pipeline/packages/kojima_sadaoki/v3.lua) | 未上架 |
| B28 | [matsunaga_hisahide](../../../character_pipeline/packages/matsunaga_hisahide/v3.lua) | 未上架 |
| B29 | [murakami_yoshikiyo](../../../character_pipeline/packages/murakami_yoshikiyo/v3.lua) | 已上架 |
| B30 | [kakizaki_kageie](../../../character_pipeline/packages/kakizaki_kageie/v3.lua) | 未上架 |
| B31 | [takeda_shingen](../../../character_pipeline/packages/takeda_shingen/v3.lua) | 未上架 |
| B32 | [uesugi_kenshin](../../../character_pipeline/packages/uesugi_kenshin/v3.lua) | 未上架 |
| B33 | [mogami_yoshihime](../../../character_pipeline/packages/mogami_yoshihime/v3.lua) | 未上架 |
| B34 | [takeda_katsuyori](../../../character_pipeline/packages/takeda_katsuyori/v3.lua) | 未上架 |
| B35 | [kamiizumi_nobutsuna](../../../character_pipeline/packages/kamiizumi_nobutsuna/v3.lua) | 未上架 |
| B36 | [naoe_kanetsugu](../../../character_pipeline/packages/naoe_kanetsugu/v3.lua) | 未上架 |
| C01 | [akechi_mitsuhide](../../../character_pipeline/packages/akechi_mitsuhide/v3.lua) | 未上架 |
| C02 | [akechi_hidemitsu](../../../character_pipeline/packages/akechi_hidemitsu/v3.lua) | 未上架 |
| C03 | [shimazu_yoshihiro](../../../character_pipeline/packages/shimazu_yoshihiro/v3.lua) | 未上架 |
| C04 | [izumo_no_okuni](../../../character_pipeline/packages/izumo_no_okuni/v3.lua) | 未上架 |
| C05 | [maeda_keiji](../../../character_pipeline/packages/maeda_keiji/v3.lua) | 未上架 |
| C06 | [ishikawa_goemon](../../../character_pipeline/packages/ishikawa_goemon/v3.lua) | 未上架 |
| C07 | [tachibana_dosetsu](../../../character_pipeline/packages/tachibana_dosetsu/v3.lua) | 未上架 |
| C08 | [kato_danzo](../../../character_pipeline/packages/kato_danzo/v3.lua) | 未上架 |
| C09 | [tachibana_ginchiyo](../../../character_pipeline/packages/tachibana_ginchiyo/v3.lua) | 未上架 |
| C10 | [kosokabe_chikayasu](../../../character_pipeline/packages/kosokabe_chikayasu/v3.lua) | 未上架 |
| C11 | [tachibana_muneshige](../../../character_pipeline/packages/tachibana_muneshige/v3.lua) | 未上架 |
| C12 | [otomo_sorin](../../../character_pipeline/packages/otomo_sorin/v3.lua) | 未上架 |
| C13 | [ryuzoji_takanobu](../../../character_pipeline/packages/ryuzoji_takanobu/v3.lua) | 未上架 |
| C14 | [saito_yoshitatsu](../../../character_pipeline/packages/saito_yoshitatsu/v3.lua) | 未上架 |
| C15 | [tamako](../../../character_pipeline/packages/tamako/v3.lua) | 已上架 |
| C16 | [kashin_koji](../../../character_pipeline/packages/kashin_koji/v3.lua) | 未上架 |
| C17 | [oichi](../../../character_pipeline/packages/oichi/v3.lua) | 已上架 |
| C18 | [kobayakawa_takakage](../../../character_pipeline/packages/kobayakawa_takakage/v3.lua) | 未上架 |
| C19 | [matsuhime](../../../character_pipeline/packages/matsuhime/v3.lua) | 未上架 |
| C20 | [chosokabe_motochika](../../../character_pipeline/packages/chosokabe_motochika/v3.lua) | 未上架 |
| C21 | [miyamoto_musashi](../../../character_pipeline/packages/miyamoto_musashi/v3.lua) | 未上架 |
| C22 | [sasaki_kojiro](../../../character_pipeline/packages/sasaki_kojiro/v3.lua) | 未上架 |
| C23 | [kato_kiyomasa](../../../character_pipeline/packages/kato_kiyomasa/v3.lua) | 未上架 |
| C24 | [yagyu_muneyoshi](../../../character_pipeline/packages/yagyu_muneyoshi/v3.lua) | 未上架 |
| C25 | [chacha](../../../character_pipeline/packages/chacha/v3.lua) | 未上架 |

---

## M4 普查結果彙整

每批完成後由主 session 追加。schema 缺口累積一批之後，一起交給 schema 負責人（M0／M1 session）做 v3.x。fix_candidate 整理給使用者逐條決定（D13）。

### 總計（2026-10-29，M4 清理完成、v3.2 登錄表合併後；schema 負責人重新驗證）

- **105 份 v3 文件**（97 位英雄、8 份道具）全部通過 `package_v3.lua`，含 v3.2 的跨文件登錄表檢查（`docs/character_pipeline/schema/v3-shared-registry.lua`，已合併 registry-additions-A／B／C／items，去重 great_sword_of_disease、from_ability、c08t_bleeding；tsum 登錄為 scope = match）。
- **coverage 共 614 筆**：expressible 598（97.4%）、needs_hook 0、pending 6、deferred 10。另有文件層 deferred 13 筆（英雄 3、道具 10；原作不存在等）。
  - 英雄 447 筆：expressible 442、pending 5（A13W2 a13_substitution、A15 a15_shadow_moon_assault、A19 a19_siege_fire、A21R a21_pearl_disc、A32 a32_soul_blade，都要在原作實測）。
  - 道具 167 筆：expressible 156、pending 1（item_ninja1 編號槽位 bug，fix_candidate）、deferred 10。
- **出處比對**（`v3_provenance.lua --strict`，105 份）：kv 4057 筆一致、0 mismatch、0 not found；84 筆是寫明 `source_differs` 的已知差異。
- 召喚物追擊（A29T、B10W、C16R）改用 `chase.max_distance` 加 `anchor = owner`，原作「離目標點太遠就瞬移回主人」用新的 `chase.teleport_back_distance`。I1 八幡鈴的出處繞路已拿掉；冰風暴之書的驅散改用 `source_is = { "has_ability", "alive" }`。
- `scripts/tests/package_v3_test.lua` 的 census 全檔檢查已打開登錄表檢查。

### 總計（2026-10-28 23:30，M4 普查完成）

- **97 位英雄全部有 v3 文件**，另有 8 份道具文件（商店 176 項扣掉原作不存在的道具）。105 份全部通過 `package_v3.lua`。
- **coverage 共 614 筆**：expressible 545（89%）、needs_hook 4、pending 65。原作不存在的道具另記在各文件的 deferred。
- **出處比對**（`v3_provenance.lua --strict`）：mismatch 2 筆（A11、C06）；not found 19 筆，集中在 13 份檔案，多半是出處字串的寫法問題。
- **fix_candidate**：文件中共 131 處標記，本表彙整約 90 項，等使用者決定（D13）。
- **needs_hook 4 筆**：A29T 兀鷹 AI、B10W 亡靈追擊、C16R 魂體 AI（原作同一套召喚物追擊 AI），以及 B28T 蛇火連鎖。v3.2 的 chain 加上一個共用的「召喚物追擊 AI」行為，預計可以消除全部 4 筆。

**下一步（清理階段，等 v3.2）**：
1. 依跨文件登錄表統一名稱：共用 unit_state、status／unit tag、共用冷卻組名。
2. 65 筆 pending 改用 v3.1／v3.2 詞彙重寫。
3. 修正 2 筆 mismatch 與 19 筆 not found。

### 清理階段（v3.2）

| 範圍 | pending | needs_hook | 出處比對 | additions |
|---|---|---|---|---|
| C01～C25 | 15→0 | 1→0（C16R 改 chase_target） | 0 mismatch、0 not found | 1 項（from_ability） |
| 道具 8 份 | 23→1（下忍編號槽位） | 0 | 約 330 筆出處改成正確寫法；0／0 | 17 項 |
| A01～A36 | 17→5（剩下 5 筆都要在原作實測，不是詞彙缺口） | 1→0（A29T 改 chase_target） | 11 mismatch／112 not found → 0／0 | 2 項（great_sword_of_disease、from_ability） |
| B01～B36 | 11→0 | 2→0（B10W 改 chase_target、B28T 改 chain＋jump） | 2 mismatch／38 not found → 0／0 | 2 項（b13e_sleep、tsum） |

尚未解決（A）：A13 預設施法距離、A15T 的 DAMAGE_TYPE_NONE、A19T 重套時是否重跑 OnCreated、A21R 光環殘留時間、A32F 死亡後切換是否保留，這 5 筆要在原作實測或由 M2 推定。chase_target 沒有 max_distance（A29T 暫用與主人的距離近似）→ v3.2.x。

尚未解決（B）：
- B02W 只複製技能來源的狀態：等 `from_ability` tag 登錄後就能改。
- B15E 開啟時強制佔用、可以被覆寫的法球槽：需要比 orb_slot.claim 更細的語意。
- B30T 全場共用變數：需要 `scope = "match"`。
- B10W 原作的轉向與閒晃細節：交給 chase_target 執行器。
- B28T 第幾跳用施放時間推算：chain 沒有跳數綁定。

尚未解決（C）：C10D 夜晚與白天的時鐘落點交給規則層；C25R 用目前等級的冷卻（原作推定是下一級，未實測）；C14T 的旗標不會把既有攻速緩速從池裡刪掉；C18W 逐件用 item，保留「只鎖可用的閃現道具」。

### 筆數

| 批次 | 英雄／道具 | 筆數 | expressible | needs_hook | pending | 驗證 |
|---|---|---|---|---|---|---|
| H1 | A01 A03 A04 A05 A06 | 24 | 23 | 0 | 1 | 5 檔 OK |
| H4 | A20 A22 A23 A24 A25 | 22 | 21 | 0 | 1 | 5 檔 OK |
| I1 | 25 個道具 | 25 | 17 | 0 | 8 | OK |
| H3 | A15 A16 A17 A18 A19 | 21 | 16 | 0 | 5 | 5 檔 OK |
| H6 | A31 A32 A33 A35 A36 | 23 | 21 | 0 | 2 | 5 檔 OK |
| H5 | A26 A27 A28 A29 A30 | 23 | 19 | 1 | 3 | 5 檔 OK（主 session 重新驗證過） |
| H2 | A08 A09 A10 A12 A14 | 26 | 24 | 0 | 2 | 5 檔 OK |
| H8 | B10 B12 B13 B14 B15 | 24 | 18 | 1 | 5 | 5 檔 OK |
| H7 | B02 B03 B05 B07 B09 | 24 | 21 | 0 | 3 | 5 檔 OK |
| H10 | B21 B23 B24 B25 B26 | 23 | 21 | 0 | 2 | 5 檔 OK；出處比對 0 mismatch |
| I3 | 25 個道具 | 25 | 24 | 0 | 0（1 筆 deferred：ronin_jitte_2） | OK；出處比對 0 mismatch |
| H13 | C03 C04 C05 C08 C09 | 21 | 19 | 0 | 2 | 5 檔 OK；出處比對 0 mismatch |
| H11 | B27 B28 B30 B31 B32 | 21 | 19 | 1 | 1 | 5 檔 OK；出處比對 0 mismatch |
| H9 | B16 B17 B18 B19 B20 | 25 | 25 | 0 | 0 | 5 檔 OK；出處比對 0 mismatch、0 not found |
| H12 | B33 B34 B35 B36 C02 | 21 | 18 | 0 | 3 | 5 檔 OK；出處比對 0 mismatch |
| H14 | C10 C11 C12 C13 C14 | 23 | 19 | 0 | 4 | 5 檔 OK；出處比對 --strict 0 mismatch、0 not found |
| H15 | C16 C18 C19 C20 C21 | 23 | 19 | 1 | 3 | 5 檔 OK；出處比對 0 mismatch |
| I5 | 25 個道具 | 25 | 21 | 0 | 0（4 筆 deferred：天之叢雲、妖化河童殼、水瀑之書、脇差） | OK；出處比對 --strict 202 筆一致 |
| H18 | 已移植：B06 B08 B29 C15 | 17 | 17 | 0 | 0 | 4 檔 OK；出處比對 0 mismatch、0 not found |
| H17 | 已移植：A02 A07 A34 B04 | 18 | 17 | 0 | 1 | 4 檔 OK；出處比對 --strict 0 mismatch |
| I4 | 25 個道具 | 25 | 20 | 0 | 0（5 筆 deferred：蠍尾妖弓、神隱、大典太、數珠丸、童子切） | OK；出處比對 187 筆一致 |
| H16 | C22 C23 C24 C25 | 17 | 16 | 0 | 1 | 4 檔 OK；出處比對 0 mismatch、0 not found（C25R 待 ability_in_slot，與 C08T 同一個缺口） |

**跨文件一致性（清理階段要處理）**：同一個原作欄位或 tag 在不同文件用了不同名稱，就不會真的共用。例如共用護盾 B36 用 `aphotic_shield_remaining`，A24 用 `a24_shield`；C02T 用 tag `zimbabwe` 移除闇牙黃泉津的免死，B35R 用 tag `a32r_void` 判斷 A32R 的標記，這兩個 tag 要由對應的文件宣告。清理時建立一份「共用 unit_state 與 status tag 登錄表」，並請 schema 負責人在驗證器加上跨文件引用檢查。

I4 回報的對齊項目：
- 火系列共用冷卻組名三份各寫各的：pilot_b 用 `fire_family`、I0 用 `share_fire_scroll`、I1 用 `fire_scroll_family`，統一為 `fire_family`。
- pilot_b 的 `mountain_scroll_lockout` 狀態要補 tag `mountain_scroll_lockout`。
- pilot_b 的 `swiftness_boots_calm` 狀態要補 tag `swiftness_calm`。

I5 回報的對齊項目（與 I4 有衝突，清理時統一）：
- I5 沿用 `share_fire_scroll`、`wind_family`、`mountain_family`，與 I4 的 `fire_family` 衝突。清理時全部統一成 `<系列>_family`。
- 山之卷鎖定：I5 用 tag `mountain_lockout`，I4 用 `mountain_scroll_lockout`，統一成一個。
- C08 大招的狀態要給 tag `c08t_bleeding`，大炎上之書才判斷得到。
- selector 缺「遇到第一個不符就停止」的語意（赤雷卷、紫雷卷的連鎖遇到隱形單位就中斷），目前用 visible_only 近似 → v3.2。
| I2 | 25 個道具 | 25 | 18 | 0 | 7（含 2 筆 not_in_source：magic_ring、poisonous_ring） | OK |
| I0 | 25 個道具 | 25 | 18 | 0 | 7 | OK（新藤五國光、久棄邪物在原作 KV 整段被註解 → not_in_source） |

**主 session 決定（D13 範圍內）**：商店清單裡有 6 個道具在原作 KV 沒有定義，遊戲裡根本買不到（ghost_gloves、gold_to_prestige、hachiman_bell 只有合成表、ice_book、kat_steel、ksitigarbha）。這些不移植，標為 `deferred: not_in_source`；因為「還原原作」不包括原作不存在的東西。

### schema 缺口（尚未交給 schema 負責人）

| 批次 | 缺口 | 建議 |
|---|---|---|
| H1 | 攻擊方式切換（遠程↔近戰，原作 SetAttackCapability） | states 加 `melee_attack`，或 stats 加 `attack_kind`；`is_ranged` 跟著反映 |
| H1 | 說明的 `{@id}` 不能指向別份文件的道具 | 允許跨文件引用道具 id |
| H1 | `set_stacks` 設成負值的語意 | 交給規則層定義 |
| H4 | 原作多個英雄與道具共用 `unit.AphoticShieldRemaining`（A04、A24、B08、B36、C06、C12、kappa 道具），和 C06 是同一個缺口 | 讓 `scope="unit"` 的 unit_state 可以跨文件同名共用，或改成規則層的共用護盾池 |

| I1 | 運算式不能比較傷害類型（吹雪護肩的反擊只對物理傷害生效） | 加 `event.damage.is_physical` 這類布林欄位，或加列舉字面值 |
| I1 | `remove_status` 不能依施加來源技能篩選（冰風暴驅散時要跳過大招） | 加 `exclude_source_tags = {"ultimate"}` 與 `require_source_ability` |

| I0 | 缺 StatKey `magic_resist_pool`：增減 nobu 魔抗池、上限 70、同名道具只算一次（般若鬼丸、謙信助宗、千鳥一文字、制魔腹卷） | 規則層 M2 的魔抗池要開放給 stats 使用 |
| I0 | 缺狀態旗標 `flying_for_pathing`：只改尋路，不給飛行視野（水計之書） | states 加這個旗標 |

| H3 | 傷害類型比較（A18R、A32；與 I1 同一個缺口） | 同 I1 |
| H3 | chance 缺 PRD 偽隨機分布模式（A17R；pity_after 只能表達保底） | chance 加 `mode = "prd"` |
| H3 | 跨文件辨識侍大將（com_general_*）（A15R） | 單位 tag，例如 `general` |
| H3 | `repeat` 省略 interval 時的語意（A19T） | 明定同一步內跑完，或讓 for_each 走訪位置串列 |

（以上 H1～H3、H4、I0、I1 的缺口已於 2026-10-28 22:40 交給 M0／M1 session 做 v3.1）

| H6 | 傷害類型字面值（A32F 要取消施法者所有物理傷害；與 I1、H3 同一個缺口） | 同上 |
| H5 | create_illusion.copy 要能指定複製狀態的時長（剩餘或原始）與施加者（A27T） | copy 選項加 `duration = remaining/original`、`caster = original/illusion` |
| H5 | 攻擊型態近戰↔遠程切換（A27T；與 H1 同一個缺口） | 同 H1 |
| H5 | 引擎內建 modifier 對照（A29E 套用 modifier_treant_overgrowth） | 規則層提供內建狀態表，或寫成一般 status |
| H5 | **驗證器 bug**：寫 `hooks.reads` 會當掉（package_v3.lua:1179 只取到一個回傳值） | 修驗證器 |
| H5 | routine 讀不到技能層的 unit_state（A26 改放在文件層） | 允許 routine 讀所屬技能的 unit_state |
| H2 | 基礎移速覆寫（加成照算） | StatKey `base_move_speed` |
| H2 | 狀態旗標 `block_disabled` | states 加這個旗標 |
| H2 | knockback 碰到單位就提前停止並執行 on_end（A14W） | knockback 加 `stop_on_contact` |
| H8 | hook 不能回呼資料 routine | hook 的 params 支援 routine／actions |
| H8 | 沒有清空 unit_state 的寫法 | `set_state` 接受 none |
| H8 | 攻擊落空沒有詞彙 | StatKey `miss_chance_pct` |
| H8 | `mana_regen_pct` 是增幅還是比例沒有定義 | 明定，或加 `mana_regen_amp_pct` |
| H8 | 全圖暫時夜晚（B14D） | `temporary_night`，`game.is_night` 跟著反映 |
| H8 | 自我強驅散不能只移除負面狀態 | `remove_status` 加只移除 debuff 的選項 |
| H8 | 法球槽不能強制佔用 | `orb_slot.claim` |
| H8 | 不能到指定等級才自動學會（B15D） | `auto_learn` 支援 `at_hero_level` |
| H7 | create_illusion.copy 要有 status_duration、status_source，以及「只複製技能來源狀態」的篩選（B02W；與 H5 A27T 同一個缺口） | 同 H5 |
| H7 | StatKey `miss_chance_pct`（B05T；與 H8 同一個缺口） | 同 H8 |
| H7 | 單位模板缺「禁止物品管理指令」與「禁止施放特定道具」（B07E 國崩隊；原作在全域 order filter 用單位名稱擋） | 單位模板加 `inventory_locked`、`item_cast_blacklist` |
| I2 | coverage 沒有 `deferred` 值（not_in_source 暫寫成 pending） | coverage 加 `deferred` 與 reason |
| I2 | 分身燒魔倍率（原作 caster.illusion_damage）沒有屬性可讀 | 規則層的分身倍率開放給運算式 |
| I2 | 商店缺「第一次補貨前的等待秒數」（爆烈彈開局等 210 秒） | `shop.stock.first_restock` |
| I2 | unit 型別變數沒有空值 | unit 型別允許 none |
| H10（v3.1 後） | 擊退途中撞到岩石要改朝石碑反彈，速度 1000−150×檢查次數（B24W） | 擊退支援中途改向與重設速度，或確認 stop_on_contact＋on_end 再擊退足以表達 |
| H10（v3.1 後） | 可疊加、帶衰減的速度衝量（B25T，原作 Physics 每幀乘 0.95） | 新 action `impulse { velocity, decay }`，多個衝量相加 |
| I3（v3.1 後） | `cooldown_groups.length` 的可用值（own／group_max／group_min／caster）沒寫進文件；`caster` 的語意是推定 | 補文件並定義語意 |
| I3（v3.1 後） | `v3_provenance.lua` 解析不到 source 括號裡的註解，產生大量 not found warning | 工具允許 `kv:Name（註解）` 寫法，或作者指南規定註解另寫 note |
| H13（v3.1 後） | 偷技能（C08T 鬼吞）：`ability_in_slot(unit, slot)`、add_ability／remove_ability 用運算式指定技能、`cast_point_override`、`event.ability` 的技能格位置 | 補這四項，偷技能就能用資料表達 |
| H13（v3.1 後） | 依單位名稱判斷（C08R 要納入 EXCLUDE_TARGET_NAME 的例外：盜賊王、怨靈武者） | 改用單位 tag（v3.1 已有 tag 機制），給這兩種魔王加 tag |
| H11（v3.1 後） | 連鎖彈跳，每一串有自己的命中表（B28T 蛇火；目前用 hook b28t_chain） | 新 action `chain { jumps, radius, exclude_hit }`，或 selector 可以排除這一串已打過的單位 |
| H11（v3.1 後） | 負的 slow_pct（B27W，第一次出現） | 規則層合成時接受負值；合成結果不為正時不套用 |
| H9（v3.1 後） | 改既有單位的基礎生命（SetBaseMaxHealth）；B16 用隱藏狀態的 `vars.hp_bonus` 加 `stats.max_hp` 繞道，前提是狀態屬性會即時讀 vars | 新 action `set_base_max_hp`，或在規格裡保證 stats 會即時讀 vars |
| H12（v3.1 後） | create_illusion.copy 的狀態來源篩選：只重套「非道具技能施加的 datadriven 狀態」，排除指定狀態（C02W／E／R、C16） | copy 加 `status_filter` |
| H12（v3.1 後） | create_illusion 的 `of` 指向敵方單位時，分身屬於哪一隊沒有定義（推定為施放者的隊伍） | 規格明定 |
| H14（v3.1 後） | 夜轉日、改寫全域時鐘（C10D） | `temporary_day` 或 `set_time_of_day` |
| H14（v3.1 後） | 單位 tag `mine`（C10T 收回時要跳過地雷） | 加入 tag 登錄表 |
| H14（v3.1 後） | `modify_cooldown` 不能作用在道具實例（C13T 每跳鎖閃現道具 0.3 秒） | modify_cooldown 支援道具實例或道具 tag |
| H14（v3.1 後） | 只免疫攻速緩速（C14T） | 狀態旗標 `attack_slow_immune` |
| H15（v3.1 後） | modify_cooldown 不接受道具，pick_item 只能挑一件；要讓背包裡每件閃現道具都進冷卻（C18W） | 同 H14：modify_cooldown 支援道具 tag，作用在所有符合的道具 |
| H15（v3.1 後） | attack 沒有假攻擊選項（只觸發命中效果、不造成傷害，C19R） | attack 加 `fake = true` |
| H15（v3.1 後） | provenance 路徑不能含空白（C21 的腳本檔名有空白） | 出處格式允許引號 |
| H6 | 缺「物理傷害免疫」狀態旗標（A32R，目前以 incoming_physical_pct −100% 近似） | states 加 `physical_immune`，並定義它和增傷 % 怎麼合成 |

| H17（v3.1 後） | 只清空緩速池、不移除狀態本身（A07E） | 新 action `clear_slow_pool` |

### 已移植英雄：現有 package 與原作的差異（M5 遷移時以 v3 為準）

| 英雄 | 差異 |
|---|---|
| A07 碎擊槍 | 原作 `0 or count` 讓計數永遠歸零，沒有保底；package 有保底 |
| A07 毫髮無傷 | 原作只處理敵方傷害、按減免前數值、擋不住致死；package 對所有來源、按減免後、擋得住致死 |
| A07 解緩速 | 原作也清攻速緩速；package 只清移速 |
| A07 奮起 | 原作大招期間換成 0 法力的 A07D2；package 用退還法力代替 |
| A02 | 原作打建築或魔免目標時用 PerformAttack 多打一擊；package 沒做 |
| B04T、B04W | package 沒切成遠程攻擊；B04W 在施法時就決定火焰目標 |
| B06R、B06E | 原作在敵方「下達指令」時就觸發（取消也算），現行等施放成立；原作彈跳計數由施法者共用、重施會重設，現行每串各自計數 |
| B08 | B08D 原作在 16 級時 SetLevel（eventforpichero.lua:152-160）；現行沒做 B08R_SE 清空共用護盾；B08D 的無敵只做到不受傷害，終點的 phased 沒做 |
| B29 | 原作施法者死亡後仍會套緩速與暈眩，現行整筆取消；B29T 原作第一擊必出、中斷後還多一擊，現行固定 45/52/59 擊 |
| C15E、C15T | 原作每團火焰各自灼燒、重疊時疊加（推定），現行只算一個；C15T 現行沒有施法者自動攻擊，被動計數死亡就清除 |

### fix_candidate（原作 bug，待使用者決定）

| 批次 | 技能／道具 | 內容 |
|---|---|---|
| H1 | a03_spider_form | 開著死亡後，近戰攻擊方式與模型不還原，開關仍亮（推定） |
| H1 | a04_water_shikigami | basic_speed 370～460 被當成移速 +%，直接到上限；攻擊間隔值被當成攻速常數 |
| H1 | a04_frost_shelter | 霜盾在受傷後才補血，擋不住一次致死傷害；重施只刷新時間，剩餘吸收量不補滿 |
| H1 | a05_loyal_secret | 判定在扣血之後，擋不住致死一擊 |
| H1 | a05_crusade_affidavit | 目標標記消失後，施法者仍完全免傷到時間結束 |
| H1 | a06_red_flame_flow／a06_flame_of_anger | 死亡後啟用旗標不關，復活後可以不靠層數再放一次，層數可能變負 |
| H4 | A23R | KV 的 maxWave 10 沒有用到，實際只要還在引導就一直出波，4 秒共 14 波 |
| H4 | A24D | 血盾受傷後才補血，擋不住致死傷害；護盾欄位跟其他英雄、道具同名共用，會互相覆寫、重複補血 |
| I1 | 吹雪護肩 | 冰盾是全域 OnTakeDamage，場上每位持有者各扣一次盾；擋不住致死傷害；三個緩速共用同一個鍵，移速 −15% 永遠不生效，攻速與移速緩速互相覆寫 |
| I1 | 三種洞察碎片 | 光環結束時移除任意一個 modifier_truesight，可能留下永久看穿（推定） |
| I1 | 紅雪左文字 | 持有多件時，後一次擲骰覆蓋前一次，第二件不增加暴擊率 |
| I0 | 般若鬼丸 | 7.5% 移速實際上常駐，疊層只是顯示 |
| I0 | 千鳥一文字 | 竊取智力固定 ±2，不隨層數累加；打任何目標都會在 15 秒後扣自己一層 |
| I0 | 當世具足 | 持有兩件時卸下一件，減傷整個消失 |
| I0 | 制魔腹卷 | 主動護盾推定腳本報錯、沒有效果 |
| H3 | A15E | 判斷寫成「層數 > maxHit」，同一單位最多被打 5 次，不是 4 次 |
| H3 | A15R | 變數名稱寫錯（damage_table），彈到侍大將時報錯，彈跳中斷 |
| H3 | A15T | 分裂飛標的傷害類型是 NONE，推定不造成傷害 |
| H3 | A16R | 反射比例停在掛上時的等級，升級後不更新 |
| H3 | A17E | 「閃避遠程攻擊」條件恆真，近戰攻擊也會觸發 |
| H3 | A18R | 護盾在扣血後才回寫生命，擋不住致死魔法傷害，還會抹掉 0.2 秒內的其他傷害與治療；破盾溢出幾乎不吃魔抗 |
| H2 | A08E | 讀未定義的全域 damageTable，結果不確定（可能報錯只暈第一個，或重打前一次的受害者）；資料照意圖寫成「範圍內全暈、不造成傷害」。原作行為本身不確定，主 session 同意照意圖 |
| H2 | A12W | 強化版把 nil 傳進 ApplyDataDrivenModifier 而報錯，沒有暈眩，也沒消耗強化 |
| H2 | A12E_HIDE | 用 keys.attacker 取施法者（nil），整招沒有效果 |
| H2 | A12R | 緩速的 OnCreated／OnDestroy 寫在 modifier 區塊外，一般版沒有緩速 |
| H2 | A14R、A14T | 攻擊力、攻速都不乘層數，層數只是顯示 |
| I2 | 舞雷刀 | 觸發後把被攻擊者自己的保底計數清零 |
| I2 | 月輪鏡 | 宮燈冷卻掛在上一個目標，最後一位被治療者沒有 |
| I2 | 中忍 | 舊中忍失去時限，反覆購買可以累積多名永久中忍 |
| H18 | B08R_SE | 讀到不存在的 damage_absorb（得 0），把長政身上的共用護盾清零，連帶清掉隊友套在長政身上的同類護盾（A04、A24、B36、C06、C12、河童之殼） |
| I5 | 鞍切．一階 | 「取其高」的 30% 分支實際只扣 1 護甲；目標護甲 ≥20 時，減甲反而從 6 降到 1 |
| H17 | A07W | 擋不住致死傷害 |
| H17 | A07R | 保底失效 |
| H17 | A02W | 對建築與魔免目標每次出手打兩下 |
| H17 | B04T | 追加子彈讀全域 `projectile_table`，會被其他英雄覆寫 |
| I4 | 火之章、風之章、山之章 | 系列清單寫成 item_recipe_the_art_of_war，孫子兵法全本不會一起進冷卻 |
| I4 | 池田鬼神丸 | 每多一個周圍單位應加的護甲與回血不會累加 |
| I4 | 吉光骨食 | 主動驅散遇到沒有技能來源的狀態時報錯中斷，後面的狀態不移除 |
| I4 | 長船 | 持有兩件時第二件會蓋掉第一件的暴擊判定 |
| I4 | 菊一文字 | 主動的狀態抗性減 500（說明寫 50%），英雄受到的狀態時間約變成 6 倍；三種減抗在目標裝卸道具時被重算抹掉，結束時又加回，抗性反而變高 |
| I4 | 長曾彌虎撤 | 判斷寫死每 4 下，遠程也是 4 下（說明寫遠程每 5 下） |
| H16 | C23R | 說明寫「吸收傷害」，實際是受傷後才補回，擋不住致死傷害 |
| H16 | C24T | 可以對自己施放，這時回復兩次、拿到兩份 +80 基礎攻擊，比對隊友施放更強 |
| H16 | C25（開場） | 開場自動學 D 拿到不存在的 C25D（nil），推定 PickHero 報錯，之後的初始化被中斷 |
| H15 | **C21T（宮本武藏大招）** | 連斬 timer 讀未定義的全域 caster 而報錯中止：只砍一刀，然後維持十秒無敵、沉默與 +1000 攻速。資料照中斷後的結果寫；把 caster 改成 u 就是設計意圖。影響大，建議優先決定 |
| H15 | C18W | 鎖的是隆景自己背包裡的閃現道具三秒，推測原意是鎖被拖住的敵人 |
| H15 | C19T | 說明寫「數秒內無法被指定」，但主動的 modifier 只有特效，沒有效果 |
| H14 | C11T | jump_range 讀不到（0），實際不會彈跳；stun_time 鍵名寫錯（0），不會暈眩 |
| H14 | C12D | 計數存在單位欄位，死亡或驅散後仍留著，沒有標記也會觸發加傷與暈眩 |
| H14 | C12E | 扣血後才補血，擋不住致死傷害；共用護盾欄位互相覆寫 |
| H14 | C12R | 隱形不會因攻擊或施法解除，和說明不符 |
| H14 | C14T | 火窪緩速寫進「A14T」鍵，和前田利家的 A14T 互相覆寫 |
| H12 | B33T | HealFilter 隨後把 decrease_health 設回 1，原本的減療（40/20/1%）平常完全無效 |
| H12 | B34T | 緩速池用固定名稱，重複命中不疊加，最早的實例一結束就清掉緩速 |
| H12 | B36E | 護盾在受傷後才補回，擋不住致死傷害；剩餘量欄位和多位英雄、道具共用，會互相覆寫 |
| H9 | B16T | 說明寫傷害被忍犬吸收，實際是受傷後才補回八成，致死傷害照樣打死千代女 |
| H9 | B17T | 重置冷卻用 IsHero 判斷，打死幻象也能重置大招 |
| H9 | B18T | Passive_insight_gem 寫在 modifier 區塊裡面沒有生效，魔爪其實不能偵測隱形（說明寫可以） |
| H9 | B20R | 魔抗屬性引用 `%B20R_msBonus`，但鍵名是 `B20R_mrBonus`，數值為 0，實際沒有魔抗加成（說明寫 8/16/24/32%） |
| H11 | B27W | 正值傳進緩速池，加速永遠不生效，攻速 150 完全沒作用 |
| H11 | B28T | wave_num 沒被讀，蛇火數量沒有上限；jump_max 100 等於無限跳，傷害一路疊高；兩道蛇火同時跳到同一單位時，一道卡住、另一道每步執行兩次 |
| H11 | B30T | 全域 tsum 被多個目標共用，點燃時間被縮短，也會和 B27T 互相干擾 |
| H11 | B32E | 施放距離小於 300 時除以零，冰道完全打不到人 |
| H13 | C09W | 開著霜刺攻擊建築時多打一次 PerformAttack，拆塔攻擊次數加倍 |
| H13 | C08T | 重抽迴圈沒有上限，目標三格都不可偷時伺服器卡死；use_count 是檔案層級變數，所有實例共用，層數顯示也算錯 |
| I3 | 真知短甲、赤備挂甲 | 格擋 modifier 沒有時效，份數只增減不消失，幾乎常駐格擋（說明是每次攻擊 50%）；光環結束可能移除錯的 truesight，留下永久看穿 |
| I3 | 雷切 | 打到持有雷系道具的敵方英雄時，會重設對方的保底計數 |
| H10 | B21T | 加成不乘層數（與說明「每降 12%」不符）；升級後可能殘留到滿血；被驅散後要到死亡或升級才會再出現 |
| H10 | B23D | 減傷可被驅散，驅散後亡魂死亡時報錯、不扣法力 |
| H10 | B23R | 取消引導後仍保有完整 4 秒魔免 |
| H10 | B24E | 驅散不會解除嘲諷；秋山死後被嘲諷者仍鎖定他 |
| H10 | B24T | 換技能讓冷卻歸零，實際約 6 秒就能再放 |
| H10 | B26R | 比的是受傷後的生命，致死傷害時不會回復 |
| H7 | B02R | 名稱與說明是十八枚手裏劍，實際發二十枚 |
| H7 | B02T | 說明的起手 1 秒暈眩從未套用 |
| H7 | B07E | 國崩隊出生時生命固定 1000，二到四級不是滿血 |
| H7 | B09W | 固定傷害寫死 50，沒有用 KV 的 20/30/40/50 |
| H8 | b12_reality_slide | 友方小兵推定仍會被拉走，與說明「不移動友軍 NPC」不符 |
| H8 | b13_hypnosis | 醒來判定作用在最後一個被催眠的目標，不是受傷的那個 |
| H8 | b15_flame_arrow | 死亡後開關仍亮、法球槽仍被佔，復活後失效（推定） |
| H5 | a26_scorching_bomb | 大招與小炸彈共用旗標和命中回呼，先落地時兩者效果互換 |
| H5 | a28_phoenix_summon | 一次基本驅散移除 modifier_A28T 就殺死鳳凰，沒有擊殺者、沒有賞金（可否驅散是推定） |
| H5 | a29_withering_aura | 額外失去的法力上限是三成「自己」的法力；緩速池以名稱為鍵，最早的實例到期就清掉緩速，不會疊加 |
| H6 | A31W | 說明寫緩速逐秒疊加，實際固定 5%（層數只顯示） |
| H6 | A32W | 換技能時重建 W，換回來後冷卻歸零，命中時實際冷卻約 5 秒 |
| H6 | A33E | 致命的一擊不會被抵銷 |
| H6 | A35W | 瞬移沒有找空地，可能卡在樹或懸崖上 |
| H6 | A36R | 致命的一擊不會被保命 |
| H4 | A24E | 暈眩公式沒有下限，距離超過 1000 時為負值，原作可能變成永久暈眩（未驗證） |

### 待 M2 實測的推定

- H1：A01D 的傷害量、A04 的 datadriven modifier 重套、A05R 的治療基準、A06R 的擴散傷害基準。


---

## M4 全量普查批次（主 session 以子 agent 執行）

輸出：英雄 `census-v3/heroes/<ID>.lua`、道具 `census-v3/items/batch_NN.lua`。每個檔案都要通過 `tools\lua\lua.exe docs\character_pipeline\tools\package_v3.lua <檔案>`。

| 批次 | 內容 | 狀態 |
|---|---|---|
| H1 | A01 A03 A04 A05 A06 | wave 1 |
| H2 | A08 A09 A10 A12 A14 | wave 1 |
| H3 | A15 A16 A17 A18 A19 | wave 1 |
| H4 | A20 A22 A23 A24 A25 | wave 1 |
| H5 | A26 A27 A28 A29 A30 | wave 1 |
| H6 | A31 A32 A33 A35 A36 | wave 1 |
| H7 | B02 B03 B05 B07 B09 | wave 2 |
| H8 | B10 B12 B13 B14 B15 | wave 2 |
| H9 | B16 B17 B18 B19 B20 | wave 2 |
| H10 | B21 B23 B24 B25 B26 | wave 2 |
| H11 | B27 B28 B30 B31 B32 | wave 2 |
| H12 | B33 B34 B35 B36 C02 | wave 2 |
| H13 | C03 C04 C05 C08 C09 | wave 3 |
| H14 | C10 C11 C12 C13 C14 | wave 3 |
| H15 | C16 C18 C19 C20 C21 | wave 3 |
| H16 | C22 C23 C24 C25 | wave 3 |
| H17 | 已移植：A02 A07 A34 B04（以 package 的 parity.md 為主） | wave 3 |
| H18 | 已移植：B06 B08 B29 C15（以 package 的 parity.md 為主） | wave 3 |
| I0～I5 | 其餘 150 個商店道具，每批 25 個（清單見下） | I0、I1 wave 1；I2、I3 wave 2；I4、I5 wave 3 |

道具清單（`shops/nobu_shops.txt` 去掉已完成的 30 個，依名稱排序後每 25 個一批）：

- **I0**：item_Hannya_Onimaru item_Shintogo_Kunimitsu item_Sukemune item_Sukezane item_agility_ingot item_ancient_doll item_attack_speed_ingot item_big_tachi item_bizennosafunekanemitsu item_blowgun item_body_wrap item_chidori item_commander_of_fan item_contemporary_armor item_cross_spear item_cursed_katana item_dark_boots item_dark_boots_1 item_devil_supressor_armor item_energy_ingot item_fire_scroll item_flame_armor item_flash_shoes item_flood_book item_foxfire_armor
- **I1**：item_frozen_ring item_fubuki_shoulders item_ghost_gloves item_glass_mitsutomo item_gold_to_prestige item_gorgeous_tachi item_great_dragon item_hachiman_bell item_hama_orb item_healing_totem item_health_ingot item_ice_book item_ice_storm_book item_ignite_book item_insight_gem_Shard item_insight_gem_hero item_insight_gem_ninja item_intelligence_ingot item_izuminokami_kanesada item_kat_steel item_katana item_kisyosennfuyuzuki item_kokumo item_kousetsusamonnji item_ksitigarbha
- **I2**：item_lesser_armor item_lesser_shield item_lightning_katana item_lightning_scroll item_liquor item_logs_big_tachi item_magic_defence_boots item_magic_defence_boots_1 item_magic_resistance_ingot item_magic_ring item_mana_boots item_mana_boots_1 item_mana_ingot item_mana_thief item_moon_mirror item_moon_spear item_nagamaki item_naginata item_nannbann_armor item_napalm_bomb item_ninja2 item_ninja_sword item_pleated_skirt item_poisonous_ring item_power_ingot
- **I3**：item_prophet_Akazonae_armor item_prophet_armor item_protection_amulet item_raikiri item_rations item_red_spear item_regeneration_ingot item_rockfall_book item_ronin_jitte_2 item_ryukyu_ronin_clogs item_ryukyu_ronin_clogs_1 item_sakabatou item_sake item_satsuma_gun item_shun_armor item_shuriken item_soul_adder item_spear item_speed_clothes item_spell_book item_spirit_ingot item_stable_shield item_staff item_storm_book item_strength_ingot
- **I4**：item_swiftness_boots_1 item_tachi item_tea_bowl item_the_art_of_war_flame_chapter item_the_art_of_war_forest_chapter item_the_art_of_war_mountain_chapter item_the_art_of_war_tornado_chapter item_the_blade_of_blood_devourer item_the_club_of_nebula item_the_dark_armor item_the_great_bow_of_scorpion item_the_great_silver_armor item_the_great_spear_of_dragonfly item_the_great_spirited_away item_the_great_sword_of_anger item_the_great_sword_of_banish item_the_great_sword_of_berserker item_the_great_sword_of_disarmament item_the_great_sword_of_disease item_the_great_sword_of_drain item_the_great_sword_of_hurricane item_the_great_sword_of_precision item_the_great_sword_of_soul_hyper item_the_great_sword_of_spike item_the_great_sword_of_tiger
- **I5**：item_the_great_sword_of_toxic item_the_great_sword_of_weakness item_the_great_sword_of_xnew_moon item_the_hooate_of_ghost item_the_lost_art_of_war_1 item_the_lost_art_of_war_2 item_the_overflame_art_of_war item_the_poketto_of_meiso item_the_red_lightning_chapter item_the_red_lightning_chapter_hyper item_the_scream_of_spiders item_the_shell_of_spirituality_kappa item_the_spear_of_ghost item_the_spear_of_saddle item_the_spear_of_spiders item_the_storm_call_book item_the_sword_of_kaminaobi item_thunderstorms item_toughness_ingot item_true_katana item_wakizashi item_waterfall_book item_wayumi item_wind_scroll item_yubashiri
