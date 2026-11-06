# 原作 bug 決定清單（fix_candidate）

> **2026-10-29 使用者決定：134 項全部修**（README D13）。修法照各條的建議欄。改資料的跟著該英雄的批次做，改規則層的由工作線 B 統一做；每一項都要有測試。做完就在表格加註完成。

日期：2026-10-28。來源：`census-v3/heroes/*.lua`、`census-v3/items/*.lua` 裡所有 `policy = "fix_candidate"` 的 quirk（130 處），加上 `census-v3/RESULTS.md` fix_candidate 表列出、但文件中寫成 deferred 的 1 項（C25 開場），共 **134 項**（FC-132～FC-134 為 2026-10-29 解開 pending 時新增）。

2026-10-29 起，普查文件已移到正式位置：英雄在 `docs/character_pipeline/packages/<package>/v3.lua`，道具在 `docs/character_pipeline/items/*.lua`。下表出處欄的 `census 檔名` 改看這兩處，對照表見 `census-v3/README.md`。同日依 D13 處理了 M5 前的待決事項（execution-status.md），都由文件照原作決定，沒有新增需要使用者決定的項目；FC-005（B08R_SE）的資料已統一照原作寫。

## 說明

**fix_candidate** 是普查時找到的原作 bug：原作程式的實際行為明顯違反設計意圖，例如說明寫「吸收傷害」，實際卻在受傷後才補血。依決策 D13，v3 文件預設照原作的實際行為寫，也就是保留 bug。這些項目另外標成 fix_candidate，交給使用者決定要不要修。

使用者要決定的是每一項「修」或「不修」：

- **不修**：照原作實際行為移植，v3 資料不用改。這是預設。
- **修**：照設計意圖移植。M5 遷移時改寫該條資料，並在 quirk 註明使用者已決定修正。

少數項目的資料已經照設計意圖寫了，因為原作行為依賴全域狀態，沒辦法用資料重現（FC-006、FC-007、FC-008、FC-022、FC-063、FC-121）。這幾項選「修」就是維持現狀；選「不修」要另外寫 hook，或接受無法完全重現。

回覆時可以整段指定，例如「修 FC-001～FC-037，FC-038 不修」。

影響的判準：

- **高**：技能主要效果失效，或嚴重失衡。
- **中**：效果明顯偏離說明。
- **低**：邊角情況，或只影響顯示。

出處格式：`census 檔名` · `條目 id（原作技能 id）` · `原作 path:line`。原作路徑相對於 `game/dota_addons/nobu/scripts/`。

## 各組數量

| 組別 | 項數 | 影響為高 | 編號 |
|---|---|---|---|
| 一、共用欄位互相覆寫 | 22 | 0 | FC-001～FC-022 |
| 二、護盾或保命擋不住致死傷害 | 15 | 2 | FC-023～FC-037 |
| 三、層數只顯示、不影響數值 | 8 | 1 | FC-038～FC-045 |
| 四、腳本報錯或寫錯名稱，技能失效或部分失效 | 24 | 7 | FC-046～FC-069 |
| 五、說明文字與實際數值不符 | 8 | 1 | FC-070～FC-077 |
| 六、可被濫用或數值異常 | 24 | 9 | FC-078～FC-101 |
| 七、死亡或驅散後狀態殘留、清理不完整 | 10 | 0 | FC-102～FC-111 |
| 八、持有多件同名道具時互相干擾 | 3 | 0 | FC-112～FC-114 |
| 九、其他（公式、判斷條件、目標錯誤、經濟） | 17 | 1 | FC-115～FC-131 |
| **合計** | **131** | **21** | |

---

## 一、共用欄位互相覆寫

原作把資料存在多個技能、多個實例或多個英雄共用的位置，結果互相覆寫。這類位置包括：`unit.AphoticShieldRemaining`、沒有 `local` 的全域變數、以固定名稱為鍵的 nobu 緩速池（`dummy.lua` Slow／ReturnSpeed），以及檔案層級變數。

**建議的整組處理方式：整組修。** 每個護盾、緩速、延遲傷害都改用自己實例的欄位：共用護盾拆成各技能私有的 unit_state（例如 `a24_shield`），全域變數改在施放當下捕捉，緩速池改成每個實例各自計時並依規則層合成。FC-006～FC-008、FC-022 的資料已經照這個方式寫，選「修」就不用動。這一組修正不會改變單一技能單獨使用時的數值，只會消除多個效果同時存在時的干擾，平衡風險低。

| 編號 | 英雄／道具 | 技能 | 原作實際行為 | 設計意圖（推定） | 影響 | 建議 | 出處 |
|---|---|---|---|---|---|---|---|
| FC-001 | A24 佐久間盛政 | 血盾集陣（A24D） | 剩餘量存在共用的 `unit.AphoticShieldRemaining`，和 A04、B08、B36、C06、C12、河童之殼同名，兩種護盾同時存在時互相覆寫、重複補血 | 每種護盾各自記錄剩餘量 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A24.lua · a24_blood_shield（A24D） · vscripts/heroes/A_Oda/A24.lua:12-16 |
| FC-002 | B36 直江兼續 | 劃心為籬（B36E） | 同 FC-001，剩餘量共用 `AphoticShieldRemaining` | 同 FC-001 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B36.lua · b36_heart_fence（B36E） · vscripts/heroes/B_Unified/B36.lua:85 |
| FC-003 | C06 石川五右衛門 | 闊氣攻擊（C06R） | 同 FC-001，剩餘量共用 `AphoticShieldRemaining` | 同 FC-001 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C06.lua · c06_lavish_strike（C06R） · vscripts/heroes/C_Neutral/C06.lua:168 |
| FC-004 | C12 大友宗麟 | 空間撕裂裝置（C12E） | 同 FC-001，剩餘量共用 `AphoticShieldRemaining` | 同 FC-001 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C12.lua · c12_space_rift（C12E） · vscripts/heroes/C_Neutral/C12.lua:148 |
| FC-005 | B08 淺井長政 | 活心．魂（B08R） | B08R_SE 讀不存在的 damage_absorb（得 0），把長政身上的共用護盾清零，隊友套在長政身上的 A04、A24、B36、C06、C12、河童之殼護盾跟著消失 | 只處理自己的增益，不碰其他護盾 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed`；測試 `v3_conformance::fix_candidates::fc005_b08_casts_keep_other_shields` ✓ 2026-10-30 | B08.lua · b08_living_soul（B08R） · vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua:194-198 |
| FC-006 | A08 榊原康政 | 檄文怒嗥（A08E） | 迴圈呼叫 ApplyDamage(damageTable)，讀到未定義的全域變數：可能報錯、只暈第一個；或用上一次別人的傷害資料，再打上一個受害者一次。結果不確定。**資料已照意圖寫**：範圍內全暈、不造成傷害 | 範圍內敵人全部暈眩 | 中 | 修（維持現狀） ✓ 2026-10-29 quirk 改 `fixed`（資料本來就照意圖寫） | A08.lua · a08_war_cry（A08E） · vscripts/heroes/A_Oda/A08.lua:20-24 |
| FC-007 | A11 齋藤道三 | 老奸巨猾（A11R） | 延遲傷害在觸發時才讀全域 damageTable；延遲期間其他腳本寫入後，傷害會打到別的目標、用別的數值。**資料已照意圖寫**：各自捕捉 | 每次延遲傷害打原本的目標 | 中 | 修（維持現狀） ✓ 2026-10-29 quirk 改 `fixed`（資料本來就照意圖寫） | A11.lua · a11_old_fox（A11R） · vscripts/heroes/A_Oda/A11.lua:189-200 |
| FC-008 | B04 伊達政宗 | 種子島（B04T） | 追加子彈讀全域 projectile_table，0.05～0.1 秒內若有另一次 B04T，或 C02、C05、C14 寫入，會用別人的位置與方向。**資料已照意圖寫**：用出手當下的位置與方向 | 追加子彈沿用本次出手 | 低 | 修（維持現狀） ✓ 2026-10-29 quirk 改 `fixed`（資料本來就照意圖寫） | B04.lua · b04_tanegashima（B04T） · vscripts/heroes/B_Unified/B04.lua:177-210 |
| FC-009 | B30 柿崎景家 | 邪王煉獄斬（B30T） | 全域 tsum 被所有目標共用，也和 B27T 共用；同時點燃 N 個目標時，補套窗口約縮成 1/N | 每個目標各自點燃 6/8/10 秒 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B30.lua · b30_purgatory_slash（B30T） · vscripts/heroes/B_Unified/B30.lua:176-212 |
| FC-010 | A29 石田三成 | 衰心靈光（A29R） | 緩速 modifier 是 MULTIPLE，但緩速池以名稱為鍵：重複觸發不疊加，最早的實例到期就清掉緩速 | 說明寫「緩速會疊加」 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A29.lua · a29_withering_aura（A29R） · npc/A_Oda/A29.txt:424-455 |
| FC-011 | B34 武田勝賴 | 天目長嚎（B34T） | 緩速池用固定名稱，重複命中不疊加，最早的實例一結束就清掉緩速 | 說明寫「可疊加」，至少應刷新時間 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B34.lua · b34_tenmoku_howl（B34T） · npc/B_Unified/B34.txt:528-552 |
| FC-012 | C14 齋藤義龍 | 火龍化身（C14T） | 火窪緩速寫進緩速池的「A14T」鍵，和前田利家的 A14T 互相覆寫，0.15 秒到期時清掉 A14T 的緩速 | 用自己的鍵 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C14.lua · c14_dragon_form（C14T） · npc/C_Neutral/C14.txt:485-503 |
| FC-013 | 吹雪護肩 | 冰盾反擊與冰霜新星 | 三個緩速共用同一個鍵：移速 −15% 被攻速緩速覆寫成 0，永遠不生效；攻速緩速與新星的移速緩速互相覆寫，任一個結束就兩個都清掉 | 三種緩速各自生效 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I1.lua · item_fubuki_shoulders · npc/npc_items_custom_load.txt:12163-12249 |
| FC-014 | A26 濃姬 | 灼熱爆裂投擲（A26T） | 大小炸彈共用 small_bomb 旗標與同一個命中回呼；大招飛行中射出小炸彈時，先落地的大招只造成小炸彈傷害、不暈眩，之後的小炸彈反而造成大招傷害與暈眩 | 大小炸彈各自結算 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A26.lua · a26_scorching_bomb（A26T） · vscripts/heroes/A_Oda/A26.lua:270,380-383 |
| FC-015 | B13 百地三太夫 | 忍法．催眠（B13E） | 醒來判定作用在 caster.B13E_old_target（最後一個被催眠的目標）；兩個目標同時沉睡時，打前一個會叫醒後一個 | 受傷的目標醒來 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B13.lua · b13_hypnosis（B13E） · vscripts/heroes/B_Unified/B13.lua:359-367 |
| FC-016 | C07 立花道雪 | 雷神之怒（C07T） | 一律讀 caster.C07T_dummy（最新一朵雲）；刷新後有兩朵雲時，舊雲以新雲為準，舊雲到期還會讓新雲無法移動 | 每朵雲各自運作 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C07.lua · c07_raijin_wrath（C07T） · vscripts/heroes/C_Neutral/C07.lua:184 |
| FC-017 | C08 加藤段藏 | 忍法密傳．鬼吞之術（C08T） | use_count 等是檔案層級變數，所有加藤段藏共用，包括敵我同名英雄；層數顯示會跳著減（3→2→0） | 每個實例各自計數 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C08.lua · c08_devour（C08T） · vscripts/heroes/C_Neutral/C08.lua:12-14,506,552-555 |
| FC-018 | 吹雪護肩 | 冰盾 | OnTakeDamage 是全域事件，沒有檢查受傷者是不是自己：場上每位持有者都對同一次傷害扣一次盾，兩人持有時盾消耗兩倍 | 只有受傷者自己的盾抵銷 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I1.lua · item_fubuki_shoulders · vscripts/nobu_modifiers/modifier_kisyosennfuyuzuki_shield.lua:23-54 |
| FC-019 | 大千鳥十文字槍 | 雷系保底 | 觸發 0.1 秒後把被攻擊者自己的 raikiri 設成 nil，打到持有雷系道具的敵方英雄會重設對方的保底計數 | 只重設攻擊者自己的計數 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | pilot_a.lua · item_oochidorijuumonnjiyari · vscripts/items/Addon_Items/item_lightning_scroll.lua:264-266 |
| FC-020 | 舞雷刀 | 雷系保底 | 同 FC-019 | 同 FC-019 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I2.lua · item_lightning_katana · vscripts/items/Addon_Items/item_lightning_scroll.lua:264-266 |
| FC-021 | 雷切 | 雷系保底 | 同 FC-019 | 同 FC-019 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I3.lua · item_raikiri · vscripts/items/Addon_Items/item_lightning_scroll.lua:264-266 |
| FC-022 | 菊一文字則宗．太刀 | 主動減狀態抗性 | 減抗直接加減 target.states_resistance；目標期間裝卸狀態抗性道具時會重算抹掉減抗，結束時又加回，抗性反而變高。**資料已照意圖寫**：用 status_resistance_pct 負值 | 期間減抗，結束後恢復原值 | 低 | 修（維持現狀） ✓ 2026-10-29 quirk 改 `fixed`（資料本來就照意圖寫） | batch_I4.lua · item_the_great_sword_of_soul_hyper · vscripts/items/Addon_Items/item_the_great_sword_of_soul.lua:40-89 |

---

## 二、護盾或保命擋不住致死傷害（受傷後才補血）

原作的護盾、減傷與保命，幾乎都寫成在 OnTakeDamage 時以 SetHealth 把血補回，這時傷害已經扣過血了。一次就致死的傷害會先殺死目標，護盾擋不住。補回的血也不經過治療事件與減療。

**建議的整組處理方式：整組修。** 改成傷害結算前處理：護盾用 absorb，保命用 `on_take_damage_pre` 加 `event.damage.lethal` 判定。說明文字本來就寫「吸收」「忽略」「不會低於」，修正後玩家看到的效果就和說明一致。FC-024（A05D）與 FC-031（B26R）的觸發條件也比錯了對象：原作比的是受傷後的生命，連非致死的重擊也受影響。修正時要一起改成比較受傷前的生命。

| 編號 | 英雄／道具 | 技能 | 原作實際行為 | 設計意圖（推定） | 影響 | 建議 | 出處 |
|---|---|---|---|---|---|---|---|
| FC-023 | A04 竹中重治 | 霜之庇護（A04R） | 受傷後以 SetHealth 補回，致死傷害擋不住，補血也不經減療 | 事前吸收傷害 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A04.lua · a04_frost_shelter（A04R） · vscripts/heroes/A_Oda/A04_Takenaka_Juji.lua:260-290 |
| FC-024 | A05 酒井忠次 | 忠之極意（A05D） | 說明是「死亡後兩秒原地重生」，但判定在扣血之後，以 damage > 剩餘生命比較：致死一擊已經殺死英雄，擋不住死亡；反而是超過受擊前生命約一半的非致死重擊會觸發 | 受到致死傷害時取消並觸發重生 | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A05.lua · a05_loyal_secret（A05D） · vscripts/heroes/A_Oda/A05.lua:10 |
| FC-025 | A07 本多忠勝 | 毫髮無傷（A07W） | 回寫生命在受傷之後，一次致死的魔法或純粹傷害照樣擊殺 | 擋下傷害 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed`；測試 `v3_conformance::fix_candidates::fc025_a07w_cancels_lethal_non_physical_damage` ✓ 2026-10-30 | A07.lua · a07_unscathed（A07W） · vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:27-46 |
| FC-026 | A18 稻葉一鐵 | 頑強戰法（A18R） | 扣血後才以 0.2 秒前的生命快照回寫：擋不住致死魔法傷害，還會抹掉這 0.2 秒內的其他傷害與治療 | 吸收魔法傷害 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A18.lua · a18_stubborn_tactics（A18R） · vscripts/heroes/A_Oda/A18.lua:27-59 |
| FC-027 | A24 佐久間盛政 | 血盾集陣（A24D） | 扣血後才 SetHealth 補回，致死傷害仍會殺死盛政 | 吸收傷害 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A24.lua · a24_blood_shield（A24D） · vscripts/heroes/A_Oda/A24.lua:59-85 |
| FC-028 | A33 本多正信 | 鬼軀之步（A33E） | 受傷後以 SetHealth(目前生命＋傷害) 補回，致命一擊不會被抵銷 | 說明寫「忽略」傷害 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A33.lua · a33_ghost_step（A33E） · vscripts/heroes/A_Oda/A33.lua:68-93 |
| FC-029 | A36 前野長康 | 苦楚秘釀（A36R） | 受傷後才補回，致命一擊在觸發前就已死亡；KV 註解寫「無敵弄不出來」 | 說明寫「生命值不會低於 6/12/18/24%」 | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A36.lua · a36_bitter_brew（A36R） · vscripts/heroes/A_Oda/A36.lua:113-120 |
| FC-030 | B16 望月千代女 | 忍法密傳．獸靈同化（B16T） | 受傷後才補回八成，致死傷害照樣打死千代女，死後忍犬也不分擔 | 說明寫「80% 傷害被忍犬吸收」 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B16.lua · b16_beast_union（B16T） · vscripts/heroes/B_Unified/B16_Mochidsuki_Chiyome.lua:275-286 |
| FC-031 | B26 大熊朝秀 | 上野武勇（B26R） | 以 dmg > 受傷後生命判斷：致死傷害不回復，超過目前生命一半的傷害也被排除 | 說明寫「無視該次傷害」 | 中 | 修（✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed`） | B26.lua · b26_kozuke_valor（B26R） · vscripts/heroes/B_Unified/B26.lua:69-73 |
| FC-032 | B36 直江兼續 | 劃心為籬（B36E） | 受傷後才補回，致死傷害擋不住 | 吸收傷害 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B36.lua · b36_heart_fence（B36E） · vscripts/heroes/B_Unified/B36.lua:89-110 |
| FC-033 | C06 石川五右衛門 | 闊氣攻擊（C06R） | 扣血後才 SetHealth 補回，致死傷害仍會殺死英雄 | 吸收物理、魔法、純粹傷害 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C06.lua · c06_lavish_strike（C06R） · vscripts/heroes/C_Neutral/C06.lua:186-192 |
| FC-034 | C12 大友宗麟 | 空間撕裂裝置（C12E） | 扣血後才補回，擋不住致死傷害 | 吸收傷害 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C12.lua · c12_space_rift（C12E） · vscripts/heroes/C_Neutral/C12.lua:271-289 |
| FC-035 | C23 加藤清正 | 以血償血（C23R） | 受傷後才補回，擋不住致死一擊，包括「必定吸收」的情況 | 說明寫「吸收該次傷害，自身不會受傷」 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C23.lua · c23_blood_for_blood（C23R） · vscripts/heroes/C_Neutral/C23.lua:95-96,113-114 |
| FC-036 | 吹雪護肩 | 冰盾 | 傷害結算後以 SetHealth 補回，致死傷害擋不住 | 吸收傷害 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I1.lua · item_fubuki_shoulders · vscripts/nobu_modifiers/modifier_kisyosennfuyuzuki_shield.lua:33-51 |
| FC-037 | 葵紋越前康繼．禦神刀 | 主動無敵 | 以 SetHealth(hp＋傷害) 事後補回，致死傷害擋不住 | 說明寫「免疫所有傷害」 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | pilot_b.lua · item_the_great_sword_of_invincible · vscripts/items/Addon_Items/item_the_great_sword_of_invincible.lua:82 |

---

## 三、層數只顯示、不影響數值

datadriven modifier 的 Properties 不會乘上 StackCount，所以腳本疊的層數只改變圖示上的數字，加成永遠是固定的一份。

**建議的整組處理方式：逐條決定，以照原作為主。** 原作玩家實際玩到的一直是固定值，平衡也是在固定值下形成的。只修上限小、說明明確的項目：FC-040、FC-042、FC-043、FC-045。疊層沒有上限或上限很高的項目照原作，因為修了數值會倍增，必須重新調整數值，已超出「還原」的範圍。例如 FC-038 A14R 最多 20 層 × 15，等於 +300 攻擊；FC-044 池田鬼神丸在兵線裡可以疊到 +18 以上護甲。

| 編號 | 英雄／道具 | 技能 | 原作實際行為 | 設計意圖（推定） | 影響 | 建議 | 出處 |
|---|---|---|---|---|---|---|---|
| FC-038 | A14 前田利家 | 疊擊槍術（A14R） | 攻擊力固定 +15；連擊層數（最多 5/10/15/20）只改顯示 | 每層 +15 攻擊力 | 中 | 修（使用者 2026-10-29 決定全修）：層數真的影響攻擊力，每層 15 ÷ 滿層數（5/10/15/20），滿層總加成等於原作 +15 ✓ 已改 v3 文件 | A14.lua · a14_stacked_spear（A14R） · npc/A_Oda/A14.txt:223-231 |
| FC-039 | A14 前田利家 | 狂怒．槍之又左（A14T） | 攻速固定 +10/15/20，層數只顯示；而且學會就永久生效、不必施放 | 每層加攻速 | 中 | 修（使用者 2026-10-29 決定全修）：改成要施放才生效，狂怒期間每層攻速 10/15/20 ÷ 10，滿 10 層總加成等於原作，狂怒結束歸零 ✓ 已改 v3 文件 | A14.lua · a14_spear_fury（A14T） · npc/A_Oda/A14.txt:414-422 |
| FC-040 | A31 森蘭丸 | 響雨（A31W） | 緩速固定 −5%，腳本疊的層數不影響數值 | 說明寫每秒疊加緩速（2/3/4/5%） | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A31.lua · a31_echoing_rain（A31W） · npc/A_Oda/A31.txt:51-61 |
| FC-041 | B21 馬場信房 | 不死鬼美濃（B21T） | 生命損失 12% 以上只得一份固定加成，層數只顯示 | 說明寫「生命每降低 12% 便提高攻擊力與護甲」 | 中 | 修（使用者 2026-10-29 決定全修）：生命每損失 12% 一層、最多 8 層，每層攻擊／護甲 = 原固定值 ÷ 8，滿層等於原作 ✓ 已改 v3 文件 | B21.lua · b21_undying_demon（B21T） · npc/B_Unified/B21.txt:388-395 |
| FC-042 | 般若鬼丸 | 被動追擊 | 裝備就常駐 +7.5% 移速，面向敵方英雄疊層只是顯示 | 最多 4 層 × 7.5% 的追擊加速，背對歸零 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I0.lua · item_hannya_onimaru · npc/npc_items_custom_load.txt:18296-18305 |
| FC-043 | 千鳥一文字 | 竊取智力 | 竊取固定 ±2，不隨層數累加 | 每次竊取 2 點智力並累加 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I0.lua · item_chidori · vscripts/items/Addon_Items/item_chidori.lua:21-29 |
| FC-044 | 池田鬼神丸國重．銘刀 | 被動 | 護甲 +3、回血 +5 是固定值，周圍單位數只記在層數 | 說明寫周圍每一位非英雄友軍加護甲 3、回血 5 | 中 | 修（使用者 2026-10-29 決定全修）：每位非英雄友軍一層、上限 4 位，每位護甲 3 ÷ 4、回血 5 ÷ 4，滿層等於原作 ✓ 已改 v3 文件 | batch_I4.lua · item_the_great_sword_of_disease · vscripts/items/Addon_Items/item_the_great_sword_of_disease.lua:19-22 |
| FC-045 | 鞍切．一階 | 減甲 | 「取其高」的三成分支實際只扣 1 護甲；目標護甲達 20 以上時，減甲反而從 6 降到 1 | 減少護甲 6 或 30%（取其高） | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I5.lua · item_the_spear_of_saddle · vscripts/items/Addon_Items/item_the_spear_of_saddle.lua:39-48 |

---

## 四、腳本報錯或寫錯名稱，技能失效或部分失效

原作讀到未定義的變數、寫錯鍵名或 modifier 名稱、把區塊寫錯位置，導致執行期報錯中斷，或數值讀成 0。

**建議的整組處理方式：整組修。** 這組的修法多半只要改一個名稱，例如把 `caster` 改成 `u`、`msBonus` 改成 `mrBonus`，或把 `item_recipe_the_art_of_war` 改成 `item_the_art_of_war`。設計意圖從腳本結構、KV 與說明都看得出來，照原作反而要刻意重現報錯。FC-058（C08T 無限迴圈會讓伺服器卡死）與 FC-061（C21T 宮本武藏大招）建議優先處理。

| 編號 | 英雄／道具 | 技能 | 原作實際行為 | 設計意圖（推定） | 影響 | 建議 | 出處 |
|---|---|---|---|---|---|---|---|
| FC-046 | A07 本多忠勝 | 碎擊槍（A07R） | 保底計數寫成 `0 or count`，永遠得 0，「超過 4 次沒暴擊必中」不成立，只剩單純機率 | 有保底 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed`；測試 `v3_conformance::fix_candidates::fc046_a07r_crits_after_misses_in_a_row` ✓ 2026-10-30 | A07.lua · a07_shattering_spear（A07R） · vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:87-93 |
| FC-047 | A12 大谷吉繼 | 智芒乍現（A12W） | 強化版把未定義的 v（nil）傳進 ApplyDataDrivenModifier 而報錯：沒有暈眩，也沒消耗強化 | 強化時暈眩範圍內敵人 等級×0.5 秒並消耗強化 | 中 | 修（✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed`） | A12.lua · a12_insight_flash（A12W） · vscripts/heroes/A_Oda/A12.lua:42-47 |
| FC-048 | A12 大谷吉繼 | 為智畏服（A12R） | 緩速的 OnCreated／OnDestroy 寫在 modifier 區塊外，一般版沒有緩速（A12R_HIDE 寫對了） | 緩速 15/20/25/30% | 中 | 修（✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed`） | A12.lua · a12_awe（A12R） · npc/A_Oda/A12.txt:520-551 |
| FC-049 | A12 大谷吉繼 | 玄能力場（A12E_HIDE） | 用 keys.attacker 取施法者，OnSpellStart 沒有這個值（nil），下一行報錯：沒有力場、沒有攻速，強化也沒清掉 | 400 範圍力場抵銷遠方傷害，強化時加攻速；把 attacker 改成 caster 即可 | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A12.lua · a12_mystic_field（A12E_HIDE） · vscripts/heroes/A_Oda/A12.lua:147-192 |
| FC-050 | A15 寧寧 | 忍法．影風車（A15R） | 變數寫成 damage_table（應為 damageTable），彈到侍大將時報錯：不受傷、不吸血，彈跳中斷 | 對侍大將造成一半傷害並繼續彈跳 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A15.lua · a15_shadow_windmill（A15R） · vscripts/heroes/A_Oda/A15.lua:217-219 |
| FC-051 | A15 寧寧 | 忍法密傳．影月攻勢（A15T） | 沒有 AbilityUnitDamageType，分裂飛標的傷害類型是 NONE，推定不造成傷害 | 說明寫「50% 傷害的飛標」 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A15.lua · a15_shadow_moon_assault（A15T） · vscripts/heroes/A_Oda/A15.lua:461-475 |
| FC-052 | B11 菊姬 | 忍法．截殺術（B11R） | 分身帶入鴉襲狀態時 duration 用了未定義的 rate（nil），分身拿到完整持續時間 | 分身只拿本體剩餘時間 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B11.lua · b11_interception（B11R） · vscripts/heroes/B_Unified/B11.lua:163-164 |
| FC-053 | B18 湖衣姬 | 大地魔爪（B18T） | Passive_insight_gem 寫在 modifier 區塊裡面，沒有生效，魔爪不能偵測隱形 | 說明寫「可以偵測隱形」 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B18.lua · b18_earth_claw（B18T） · npc/B_Unified/B18.txt:282-303 |
| FC-054 | B20 山縣昌景 | 強韌（B20R） | 魔抗引用 `%B20R_msBonus`，鍵名其實是 `B20R_mrBonus`，數值為 0，沒有魔抗加成 | 說明寫「降低受到的法術傷害 8/16/24/32%」 | 中 | 修（✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed`） | B20.lua · b20_tenacity（B20R） · npc/B_Unified/B20.txt:136,154 |
| FC-055 | B27 小島貞興 | 騎術（B27W） | 原本的加速屬性被註解掉，改成把正值傳進緩速池：加速永遠不生效，只會抵銷其他緩速；攻速 150 完全沒作用 | 加移速 10/13/16/19% 與攻速 150 | 高 | 修（✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed`） | B27.lua · b27_horsemanship（B27W） · npc/B_Unified/B27.txt:111-125 |
| FC-056 | B32 上杉謙信 | 神閃（B32E） | 施放距離小於 300 時除以零，冰道判定點變成 NaN，冰道完全打不到人 | 短距離也有冰道判定 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B32.lua · b32_divine_flash（B32E） · vscripts/heroes/B_Unified/B32_Uesugi_Kenshin.lua:73-78 |
| FC-057 | B33 最上義姬 | 蜘蛛纏身（B33T） | HealFilter 壓低治療後，緊接著因為沒有 modifier_decrease_HR 又設回 1，減療平常完全無效 | 治療降為 40/20/1% | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B33.lua · b33_spider_attack（B33T） · vscripts/events.lua:316-342 |
| FC-058 | C08 加藤段藏 | 忍法密傳．鬼吞之術（C08T） | 重抽迴圈沒有上限：目標三格都不可偷時無限迴圈，伺服器卡死 | 沒有可偷的技能時失敗並結束 | 高 | 修（優先） ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C08.lua · c08_devour（C08T） · vscripts/heroes/C_Neutral/C08.lua:485-488 |
| FC-059 | C11 立花宗茂 | 西國無雙（C11T） | 彈跳半徑讀 jump_range，C11T 沒有這個鍵（得 0），雷電不會彈跳，max_jump 沒有作用 | 說明寫會傳導；改用雷切的 700 | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C11.lua · c11_unrivaled（C11T） · vscripts/heroes/C_Neutral/C11.lua:73,113-146 |
| FC-060 | C11 立花宗茂 | 西國無雙（C11T） | 暈眩讀 stun_time（C11T 只有 stun_duration），得 0，推定沒有暈眩 | 說明寫「電暈 1 秒」 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C11.lua · c11_unrivaled（C11T） · vscripts/heroes/C_Neutral/C11.lua:316-325 |
| FC-061 | C21 宮本武藏 | 秘奧義‧飛翔烈震斬（C21T） | 連斬 timer 讀未定義的全域 caster 而報錯中止：只砍一刀，之後維持十秒無敵、沉默與 +1000 攻速（同文件 no_effect quirk :153-158） | 每 0.15 秒瞬移斬擊周圍隨機敵人，共 8/12/16 次，用完就結束無敵；把 caster 改成 u | 高 | 修（優先） ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C21.lua · c21_soaring_quake（C21T） · vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:98-168 |
| FC-062 | 制魔腹卷 | 主動護盾 | 推定 ApplyDataDrivenModifier 找不到 modifier 而報錯，主動完全沒有效果 | 800＋等級×20 法術護盾，持續 25 秒（腳本寫 20 秒） | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I0.lua · item_devil_supressor_armor · vscripts/items/Addon_Items/item_devil_supressor_armor.lua:119-129 |
| FC-063 | 吉光骨食．銘刀 | 主動驅散 | 遇到沒有技能來源的狀態（例如 modifier_kill）時報錯中斷，後面的狀態都不移除，結果看 modifier 順序。**資料已照意圖寫**：完整移除 | 完整驅散 | 中 | 修（維持現狀） ✓ 2026-10-29 quirk 改 `fixed`（資料本來就照意圖寫） | batch_I4.lua · item_the_great_sword_of_drain · vscripts/items/Addon_Items/item_the_great_sword_of_drain.lua:13-28 |
| FC-064 | 孫子兵法全本 | 共用冷卻 | 火、風、山、林四個系列清單都寫成 item_recipe_the_art_of_war，使用各卷不會讓全本進冷卻 | 全本和各卷共用冷卻 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | pilot_b.lua · item_the_art_of_war · vscripts/items/Addon_Items/item_fire_scroll.lua:32 |
| FC-065 | 山之卷 | 共用冷卻 | 同 FC-064 | 同 FC-064 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | pilot_b.lua · item_mountain_scroll · vscripts/items/Addon_Items/item_mountain_scroll.lua:10 |
| FC-066 | 孫子兵法．火之章 | 共用冷卻 | 同 FC-064 | 同 FC-064 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I4.lua · item_the_art_of_war_flame_chapter · vscripts/items/Addon_Items/item_fire_scroll.lua:60 |
| FC-067 | 孫子兵法．山之章 | 共用冷卻 | 同 FC-064 | 同 FC-064 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I4.lua · item_the_art_of_war_mountain_chapter · vscripts/items/Addon_Items/item_mountain_scroll.lua:10 |
| FC-068 | 孫子兵法．風之章 | 共用冷卻 | 同 FC-064 | 同 FC-064 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I4.lua · item_the_art_of_war_tornado_chapter · vscripts/items/Addon_Items/item_wind_scroll.lua:74 |
| FC-069 | C25 茶茶 | 開場自動學 D | 開場自動學 D，但 16 版沒有 C25D（只有 C25D_old），FindAbilityByName 得 nil，推定 PickHero 報錯，該 Timer 其餘的初始化被中斷。文件中寫成 deferred（not_in_source），RESULTS.md 列為 fix_candidate | 不自動學不存在的技能，其餘初始化照常 | 中 | 修（跳過 C25D） ✓ 2026-10-29 v3 不建立 C25D、其餘初始化照常（deferred 註記 FC-069） | C25.lua · deferred C25D · vscripts/library/events/eventforpichero.lua:57-61；vscripts/addon_game_mode.lua:341,444 |

---

## 五、說明文字與實際數值不符

腳本或 KV 的數值和說明不一致，而且看得出說明才是原意。

**建議的整組處理方式：整組修，FC-071 例外。** 修法是照說明與 KV 的數值。FC-071（B02R 二十枚對十八枚）只差約一成傷害，原作玩家實際玩到的一直是二十枚，建議照原作。FC-076（C19T）修之前要先確認「無法被指定」的秒數：modifier 時長 11/14/17 秒，直接全程不能被指定會過強。

| 編號 | 英雄／道具 | 技能 | 原作實際行為 | 設計意圖（推定） | 影響 | 建議 | 出處 |
|---|---|---|---|---|---|---|---|
| FC-070 | A23 宇喜多秀家 | 鬼術．墜火（A23R） | KV maxWave 10 沒被用到，只要還在引導就一直出波，4 秒共 14 波，傷害與燃燒多四成 | 說明寫「總共可降下 10 波」 | 中 | 修（✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed`） | A23.lua · a23_falling_fire（A23R） · vscripts/heroes/A_Oda/A23.lua:135-189 |
| FC-071 | B02 風魔小太郎 | 忍法．十八方手裏劍（B02R） | 實際發二十枚（總傷害 100/200/300/400） | 十八枚（90/180/270/360） | 低 | 修（使用者 2026-10-29 決定全修）：照說明 18 枚，總傷害 90/180/270/360 ✓ 已改 v3 文件 | B02.lua · b02_eighteen_shuriken（B02R） · npc/B_Unified/B02.txt:475-494 |
| FC-072 | B02 風魔小太郎 | 忍法密傳．四方導雷（B02T） | KV 有起手暈眩 modifier_B02T_2（1 秒），但腳本從未套用，施放當下沒有暈眩 | 說明寫「暈眩 1 秒，之後每秒一擊」 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B02.lua · b02_four_thunders（B02T） · npc/B_Unified/B02.txt:585-627 |
| FC-073 | B09 真田昌幸 | 反噬裂隙（B09W） | 固定傷害寫死 50，一到三級偏高 | KV basicDMG 20/30/40/50 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B09.lua · b09_backlash_rift（B09W） · vscripts/heroes/B_Unified/B09.lua:18 |
| FC-074 | B12 山本勘助 | 實界滑移（B12W） | 以 IsCreature() 排除友軍，友方小兵（creep_lane）推定仍會被拉走 | 說明寫「無法移動友軍 NPC」 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B12.lua · b12_reality_slide（B12W） · vscripts/heroes/B_Unified/B12.lua:39 |
| FC-075 | C12 大友宗麟 | 光學遮罩（C12R） | 隱形不因攻擊或施法解除；KV 註解寫「普攻 放技能 解除不了隱形…沒弄」 | 說明寫「攻擊或施放技能會使隱形提前結束」 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C12.lua · c12_optical_mask（C12R） · npc/C_Neutral/C12.txt:244 |
| FC-076 | C19 松姬 | 不容侵犯（C19T） | 主動的 modifier 只有特效，沒有任何狀態，11/14/17 秒完全沒作用，只剩擊退 | 說明寫「數秒內無法被指定」 | 高 | 修（先確認秒數） ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C19.lua · c19_inviolable（C19T） · npc/C_Neutral/C19.txt:636-652 |
| FC-077 | 長曾彌虎撤．銘刀 | 被動 | 判斷式寫死每 4 下，遠程也是每 4 下 | 說明寫近戰每 4 下、遠程每 5 下 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I4.lua · item_the_great_sword_of_tiger · vscripts/items/Addon_Items/item_the_great_sword_of_tiger.lua:18-22 |

---

## 六、可被濫用或數值異常

這組包括冷卻被重置、取消施法仍保留效果、對自己施放效果加倍、會永久存在的單位與狀態，以及數值超出合理範圍。

**建議的整組處理方式：整組修。** 這些行為在對戰中可以穩定地被利用，例如大招冷卻從 120 秒變約 6 秒（FC-088），反覆購買累積永久下忍、中忍（FC-093、FC-094），驅散標記就能全免傷（FC-080），會嚴重破壞平衡。冷卻類（FC-081、FC-085、FC-088）的修法是換技能時保留原本的冷卻，也就是 swap_abilities 的 cooldown_policy 改成保留。永久看穿（FC-097～FC-101）是邊角情況，可以一起修：光環結束時只移除光環自己掛的真視。

| 編號 | 英雄／道具 | 技能 | 原作實際行為 | 設計意圖（推定） | 影響 | 建議 | 出處 |
|---|---|---|---|---|---|---|---|
| FC-078 | A02 稻姬 | 蠍毒箭（A02W） | 目標是建築或魔免時改呼叫 PerformAttack，每次出手多打一下，拆塔速度加倍 | 註解寫「只對英雄或小兵作用」，對建築應單純不作用 | 中 | 修 ✓ 2026-10-30 拿掉 otherwise 的 attack，quirk 改 `fixed` | A02.lua · a02_venom_arrow（A02W） · vscripts/heroes/A_Oda/A02.lua:41-46 |
| FC-079 | A04 竹中重治 | 式神．水（A04W） | basic_speed 370～460 被當成移速 +370%～+460%，直接頂到引擎上限；攻擊間隔 1.4～0.8 被當成攻速常數，只加一點多 | 移速 370～460、攻擊間隔 1.4～0.8 | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A04.lua · a04_water_shikigami（A04W） · npc/A_Oda/A04.txt:146-152 |
| FC-080 | A05 酒井忠次 | 誓伐狀（A05T） | 目標標記被驅散或目標死亡後，施法者的免傷仍持續到時間結束，連原目標的傷害也免疫，等於完全免傷 | 只免疫標記目標以外的傷害，標記消失就結束 | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A05.lua · a05_crusade_affidavit（A05T） · vscripts/heroes/A_Oda/A05.lua:160-166 |
| FC-081 | A13 服部半藏 | 忍法．影惑之術（A13W） | 施放時移除技能、6 秒後加回新實例，新實例沒有冷卻，有效冷卻約 6 秒而不是 15 秒 | 冷卻 15 秒 | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A13.lua · a13_shadow_illusion（A13W） · vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:232-237 |
| FC-082 | A13 服部半藏 | 忍法．影惑之術（A13W） | 前搖開始就給 0.8 秒無敵，取消施法也保留，可以不花冷卻與法力取得無敵 | 施放成立才給無敵 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A13.lua · a13_shadow_illusion（A13W） · vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:196-201 |
| FC-083 | A24 佐久間盛政 | 血之突進（A24E） | 暈眩 1.8 −（距離−200）×1.5/600 沒有下限，距離超過 1000 時為負值，Dota 中可能變成永久暈眩（未驗證） | 暈眩時間不低於 0 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A24.lua · a24_blood_charge（A24E） · vscripts/heroes/A_Oda/A24.lua:349-361 |
| FC-084 | A28 羽柴秀吉 | 鳳凰召來（A28T） | 存在時間由可驅散（推定）的 modifier 控制，一次基本驅散就殺死鳳凰，沒有擊殺者、沒有賞金 | 鳳凰只能被擊殺或時間到才消失 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A28.lua · a28_phoenix_summon（A28T） · npc/A_Oda/A28.txt:384-395 |
| FC-085 | A32 瀧川一益 | 忍法．怨靈飛翔斬（A32W） | 換技能時重建 A32W，新實例沒有冷卻；落地命中後 12/11/10/9 秒冷卻實際約 5 秒 | 照 KV 冷卻 | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A32.lua · a32_flying_slash（A32W） · vscripts/heroes/A_Oda/A32.lua:93-101 |
| FC-086 | B17 霧隱才藏 | 忍法密傳．覓心斬（B17T） | 用 IsHero() 判斷，打死敵方幻象也能重置大招 | 說明寫「擊殺英雄」才重置 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B17.lua · b17_heartseeker（B17T） · vscripts/heroes/B_Unified/B17.lua:193-195 |
| FC-087 | B23 伊勢姬 | 魂饗（B23R） | 中斷時只移除亡魂的魔免，沒移除自己的：施放後立刻取消引導，仍有完整 4 秒魔免 | 魔免只在引導期間 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B23.lua · b23_soul_feast（B23R） · vscripts/heroes/B_Unified/B23.lua:321-327 |
| FC-088 | B24 秋山信友 | 大地禁錮（B24T） | 施放後移除技能、6 秒後加回新實例，冷卻歸零，大招約每 6 秒就能再放（KV 冷卻 120/100/80） | 照 KV 冷卻 | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B24.lua · b24_earth_prison（B24T） · vscripts/heroes/B_Unified/B24.lua:85-94 |
| FC-089 | B28 松永久秀 | 鬼謀．蛇火（B28T） | wave_num 4/8/12 沒被讀，每個可見敵人都放出一道蛇火；jump_max 100 等於無限跳，每跳傷害 +50%，聚在一起的敵人會被打很多次；兩道同時跳到同一單位時還會卡住 | 蛇火 4/8/12 道，跳數有限 | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B28.lua · b28_serpent_fire（B28T） · npc/B_Unified/B28.txt:491-495 |
| FC-090 | C09 立花誾千代 | 霜刺（C09W） | 對建築出手時多 PerformAttack 一次，開著霜刺拆塔攻擊次數加倍 | 對建築不扣魔、不套法球 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C09.lua · c09_frost_thorn（C09W） · vscripts/heroes/C_Neutral/C09.lua:43-47 |
| FC-091 | C24 柳生宗嚴 | 柳生新陰流．最期一生（C24T） | 可以對自己施放，這時回復兩次、拿到兩份 +80 基礎攻擊，比對隊友施放更強 | 說明寫「對友方英雄施展」，自己施放不應加倍 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C24.lua · c24_last_life（C24T） · vscripts/heroes/C_Neutral/C24.lua:108-114 |
| FC-092 | 菊一文字則宗．太刀 | 主動減狀態抗性 | 狀態抗性減 500（說明寫 50%），8 秒內英雄受到的敵方狀態時間約變成 6 倍 | 減 50% | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I4.lua · item_the_great_sword_of_soul_hyper · vscripts/items/Addon_Items/item_the_great_sword_of_soul.lua:74-82 |
| FC-093 | 下忍 | 召喚 | 超額時移除舊下忍的 modifier_kill，舊下忍不會死，永久存在且不再被追蹤；反覆購買可以累積任意多名 | 超額時舊下忍消失 | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | pilot_a.lua · item_ninja1 · vscripts/items/Addon_Items/item_ninja.lua:19,25 |
| FC-094 | 中忍 | 召喚 | 舊中忍還活著時再召喚，舊的變成永久存在；反覆購買可以累積多名永久中忍 | 說明寫「一人最多同時存在一隻」 | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I2.lua · item_ninja2 · vscripts/items/Addon_Items/item_ninja.lua:53-61 |
| FC-095 | 真視赤備挂甲 | 格擋 | 格擋 modifier 沒有時效，每次被命中擲 50% 增減一份，份數不會消失，幾乎一直有格擋 | 每次攻擊各自 50% 機率格擋 50 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I3.lua · item_prophet_akazonae_armor · npc/npc_items_custom_load.txt:11920-11958 |
| FC-096 | 真知短甲 | 格擋 | 同 FC-095 | 同 FC-095 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I3.lua · item_prophet_armor · npc/npc_items_custom_load.txt:11735-11773 |
| FC-097 | 洞察碎片 | 光環真視 | 光環結束時移除目標身上任一個 modifier_truesight，可能移除主動的十秒那份、留下光環的永久那份，目標從此一直被看穿（推定） | 只移除光環自己掛的真視 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I1.lua · item_insight_gem_shard · npc/npc_items_custom_load.txt:4304-4317 |
| FC-098 | 真視碎片 | 光環真視 | 同 FC-097 | 同 FC-097 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I1.lua · item_insight_gem_hero · npc/npc_items_custom_load.txt:16205-16218 |
| FC-099 | 真知碎片 | 光環真視 | 同 FC-097 | 同 FC-097 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I1.lua · item_insight_gem_ninja · npc/npc_items_custom_load.txt:16354-16367 |
| FC-100 | 真視赤備挂甲 | 光環真視 | 同 FC-097 | 同 FC-097 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I3.lua · item_prophet_akazonae_armor · npc/npc_items_custom_load.txt:11905-11918 |
| FC-101 | 真知短甲 | 光環真視 | 同 FC-097 | 同 FC-097 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I3.lua · item_prophet_armor · npc/npc_items_custom_load.txt:11720-11733 |

---

## 七、死亡或驅散後狀態殘留、清理不完整

狀態被死亡或驅散移除時，對應的開關、旗標、計數或強制效果沒有一起清掉。結果可能是復活後技能失效，或沒有標記也照樣觸發。

**建議的整組處理方式：整組修。** 修法是讓開關、旗標與計數跟著狀態一起移除，或改成可以恢復的被動。這組多半是邊角情況，修正後玩家看到的效果就和說明一致，平衡風險低。

| 編號 | 英雄／道具 | 技能 | 原作實際行為 | 設計意圖（推定） | 影響 | 建議 | 出處 |
|---|---|---|---|---|---|---|---|
| FC-102 | A03 石川數正 | 蜘蛛化型（A03E） | 開著死亡後，近戰攻擊方式與模型不還原、開關仍亮，復活後是沒有加成的近戰單位，要手動關再開（推測） | 死亡時關閉並還原 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A03.lua · a03_spider_form（A03E） · vscripts/heroes/A_Oda/A03.lua:58-73 |
| FC-103 | A06 井伊直政 | 赤焰流（A06R） | 層數狀態死亡時消失，但怒焰的啟用旗標不關：復活後不需層數仍能放一次，層數可能變成負值 | 旗標跟著層數清除 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A06.lua · a06_red_flame_flow（A06R） · vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:102-112 |
| FC-104 | B15 今川義元 | 烈焰箭（B15E） | 死亡時狀態被移除，但開關仍亮、法球槽仍被佔，復活後不扣魔也不加傷，要手動關再開（推定） | 復活後照常運作，或死亡時關閉 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B15.lua · b15_flame_arrow（B15E） · npc/B_Unified/B15.txt:217-238 |
| FC-105 | B21 馬場信房 | 不死鬼美濃（B21T） | 升級大招時 handle 被清空但狀態沒移除，加成可能殘留到滿血 | 生命回升時正確移除 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B21.lua · b21_undying_demon（B21T） · vscripts/heroes/B_Unified/B21.lua:158-208 |
| FC-106 | B21 馬場信房 | 不死鬼美濃（B21T） | 加成被驅散後 handle 指向已銷毀的 modifier，之後報錯，直到死亡或升級前都不會再出現 | 驅散後條件成立就重新套上 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B21.lua · b21_undying_demon（B21T） · vscripts/heroes/B_Unified/B21.lua:192-206 |
| FC-107 | B23 伊勢姬 | 亡靈之契（B23D） | 減傷可被驅散，驅散後亡魂死亡時報錯、不扣法力，伊勢姬死亡時也不帶走亡魂 | 驅散不破壞亡魂連動 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B23.lua · b23_undead_pact（B23D） · vscripts/heroes/B_Unified/B23.lua:106-118 |
| FC-108 | B24 秋山信友 | 嘲諷（B24E） | 強制攻擊只在 0.1 秒時設定一次，和可驅散的 modifier 無關，驅散不會解除嘲諷 | 驅散時解除嘲諷 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B24.lua · b24_taunt（B24E） · vscripts/heroes/B_Unified/B24.lua:340-373 |
| FC-109 | B24 秋山信友 | 嘲諷（B24E） | 秋山死後，被嘲諷者仍鎖定攻擊已死的他直到結束，只能站著；原作有未使用的移除函式 | 施法者死亡時移除嘲諷 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B24.lua · b24_taunt（B24E） · vscripts/heroes/B_Unified/B24.lua:341 |
| FC-110 | C07 立花道雪 | 禦魔之盾（C07D） | modifier 不是 Permanent，推定死亡時移除但開關仍開，復活後要關再開才有盾（未實測） | 復活後自動恢復 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C07.lua · c07_magic_shield（C07D） · npc/C_Neutral/C07.txt:512-541 |
| FC-111 | C12 大友宗麟 | 定位導彈（C12D） | 計數存在單位欄位，死亡或驅散後標記消失但計數留著，沒有標記也吃加傷與暈眩 | 計數跟標記一起清除 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C12.lua · c12_homing_lock（C12D） · vscripts/heroes/C_Neutral/C12.lua:7,31-35 |

---

## 八、持有多件同名道具時互相干擾

**建議的整組處理方式：逐條決定。** FC-112 修：卸下一件時，剩下的那件應繼續提供減傷。FC-113、FC-114 照原作：Dota 的暴擊道具本來就不疊加，同時持有兩件同名暴擊道具很少見，而且修成「各自擲骰」會提高暴擊率，偏離原作平衡。

| 編號 | 英雄／道具 | 技能 | 原作實際行為 | 設計意圖（推定） | 影響 | 建議 | 出處 |
|---|---|---|---|---|---|---|---|
| FC-112 | 當世具足 | 被動減傷 | 持有兩件時只有一份減傷，卸下任一件就整份移除，剩下那件不再提供，直到重新裝備 | 還有一件就保留減傷 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I0.lua · item_contemporary_armor · vscripts/items/Addon_Items/item_contemporary_armor.lua:16-29 |
| FC-113 | 紅雪左文字 | 被動暴擊 | 持有兩件時每次命中擲兩次，第二次蓋掉第一次，第二件不增加暴擊率，但兩次都推進保底計數 | 兩件各自判定（推定） | 低 | 修（使用者 2026-10-29 決定全修）：持有兩件時各自判定、各自累積保底計數（scope = item_instance） ✓ 已改 v3 文件；⚠ 2026-11-02 runtime 未完成：host 同種道具的被動事件只跑一次，第二件沒有作用（v3.md §12.13，工作線 B） | batch_I1.lua · item_kousetsusamonnji · npc/npc_items_custom_load.txt:9604-9616 |
| FC-114 | 長船．太刀 | 被動暴擊 | 同 FC-113 | 同 FC-113 | 低 | 修（使用者 2026-10-29 決定全修）：同 FC-113 ✓ 已改 v3 文件；⚠ runtime 未完成，同 FC-113 | batch_I4.lua · item_the_great_sword_of_precision · npc/npc_items_custom_load.txt:3218-3243 |

---

## 九、其他（公式、判斷條件、目標錯誤、經濟）

**建議的整組處理方式：大多修，逐條確認。** 這組是公式寫錯、條件恆真或差一、作用對象錯誤，修法都很明確。FC-121（A21T）的資料已經等同修正，因為 v3 表達不出原作的漏洞。FC-129～FC-131（GivePlayerGold_UnReliable 重複計入可靠金錢）屬於規則層經濟，建議在規則層統一修一次：若本作有可靠金錢，每次給錢或扣錢都會多出一份可靠金錢，影響可能比表中的「中」大。

| 編號 | 英雄／道具 | 技能 | 原作實際行為 | 設計意圖（推定） | 影響 | 建議 | 出處 |
|---|---|---|---|---|---|---|---|
| FC-115 | A04 竹中重治 | 霜之庇護（A04R） | 對已有霜盾的目標重施只刷新時間，剩餘吸收量不補滿，只得到 150 治療（推定） | 重施時重設護盾 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A04.lua · a04_frost_shelter（A04R） · npc/A_Oda/A04.txt:466-484 |
| FC-116 | A11 齋藤道三 | 妖術．陰邪凝視（A11E） | 目的地公式 (施法者位置＋目標位置)×pct/100 只有 4 級（50%）時等於中點；1～3 級的點往世界原點縮放，可能離戰場很遠 | 說明寫「雙方之間 p% 距離的點」 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A11.lua · a11_evil_gaze（A11E） · vscripts/heroes/A_Oda/A11.lua:68 |
| FC-117 | A15 寧寧 | 忍法．月擊（A15E） | 判斷寫成「層數 > maxHit」，同一單位可被打 5 次（暈眩可疊五次） | 說明與 KV 都是最多 4 次 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A15.lua · a15_moon_strike（A15E） · vscripts/heroes/A_Oda/A15.lua:94 |
| FC-118 | A16 鳥居元忠 | 血殤靈光（A16R） | 反射比例只在掛上時寫入，學會時是一級，升級後仍是 15%，要死亡後重掛才更新 | 說明寫 15/30/45/60% | 中 | 修（✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed`） | A16.lua · a16_blood_aura（A16R） · vscripts/heroes/A_Oda/A16.lua:63-78 |
| FC-119 | A17 佐佐成政 | 疾射（A17E） | 「閃避遠程攻擊」判斷的是被攻擊者自己（遠程英雄），條件恆真，近戰攻擊也會觸發 | 只對遠程攻擊觸發 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A17.lua · a17_rapid_fire（A17E） · vscripts/heroes/A_Oda/A17.lua:143-151 |
| FC-120 | A18 稻葉一鐵 | 頑強戰法（A18R） | 破盾溢出傷害用 (100 − 魔抗比例)×0.01 計算，幾乎不吃魔抗，而且以減免前的傷害扣盾 | 溢出傷害照魔抗減免 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A18.lua · a18_stubborn_tactics（A18R） · vscripts/heroes/A_Oda/A18.lua:46 |
| FC-121 | A21 瑞龍院日秀 | 偽情報（A21T） | order filter 只檢查第一個選取的單位，多選且英雄不是第一個時，指令可能漏過封鎖。**v3 表達不出這個漏洞，資料已等同修正** | 所有被控制單位都受封鎖 | 低 | 修（維持現狀） ✓ 2026-10-29 quirk 改 `fixed`（資料本來就照意圖寫） | A21.lua · a21_false_intel（A21T） · vscripts/library/events/eventfororder.lua:396-401 |
| FC-122 | A29 石田三成 | 衰心靈光（A29R） | 額外損失的法力取 min(三成「自己」的目前法力, mana_lost)，三成沒魔時敵人不會額外失去法力 | 以目標的法力為上限 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | A29.lua · a29_withering_aura（A29R） · vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:21-22 |
| FC-123 | A35 島左近 | 焰襲（A35W） | 瞬移用 SetAbsOrigin，沒有找空地，可能卡在樹、懸崖或不可通行處 | 落點找空地 | 低 | 修（✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed`） | A35.lua · a35_flame_raid（A35W） · vscripts/heroes/A_Oda/A35.lua:87-88 |
| FC-124 | B07 足利義昭 | 戰線國崩隊（B07E） | SetBaseMaxHealth 後 SetHealth(GetHealth())，生命維持原本的 1000，二到四級出生時不是滿血 | 出生滿血 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B07.lua · b07_battle_line（B07E） · vscripts/heroes/B_Unified/B07.lua:109-110 |
| FC-125 | B22 太原雪齋 | 淨化（B22R） | 友方分支從施法者自己身上移除狀態，目標只掛特效；對隊友施放等於只淨化自己 | 淨化目標隊友 | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | B22.lua · b22_purify（B22R） · vscripts/heroes/B_Unified/B22.lua:96-103 |
| FC-126 | C18 小早川隆景 | 謀計．單馬沓履（C18W） | 鎖的是隆景自己背包裡的閃現道具 3 秒，等於施放後自己不能閃現 | 鎖被拖住的敵人，不讓他閃走 | 中 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C18.lua · c18_lone_rider（C18W） · vscripts/heroes/C_Neutral/C18.lua:74-84 |
| FC-127 | 千鳥一文字 | 竊取智力 | 15 秒後扣層的 Timer 不論目標都會建立，打小兵、建築或分身也會扣掉自己的一層，竊得的智力提早消失 | 只有竊取成功才排扣層 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I0.lua · item_chidori · vscripts/items/Addon_Items/item_chidori.lua:31-52 |
| FC-128 | 月輪鏡 | 主動彈跳治療 | 宮燈冷卻掛在上一個目標而不是新目標，最後一位被治療者沒有宮燈冷卻 | 每個被治療者都掛冷卻 | 低 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | batch_I2.lua · item_moon_mirror · vscripts/items/Addon_Items/item_moon_mirror.lua:79-83 |
| FC-129 | C06 石川五右衛門 | 偷天換日（C06E） | GivePlayerGold_UnReliable 以總金錢（含可靠金錢）＋金額寫回不可靠金錢，可靠金錢被重複計入 | 只增加指定的不可靠金錢 | 中 | 修（規則層） ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C06.lua · c06_sleight_of_hand（C06E） · vscripts/amhc_library/amhc.lua:509-522 |
| FC-130 | C06 石川五右衛門 | 殺人越貨（C06D） | 同 FC-129 | 同 FC-129 | 中 | 修（規則層） ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C06.lua · c06_murder_and_loot（C06D） · vscripts/amhc_library/amhc.lua:509-522 |
| FC-131 | C06 石川五右衛門 | 豪奪（C06T） | 扣款 50 也走同一個函式，同樣重複計入可靠金錢；因為先檢查 > 50，不會變負 | 同 FC-129 | 中 | 修（規則層） ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | C06.lua · c06_grand_heist（C06T） · vscripts/amhc_library/amhc.lua:509-522 |
| FC-132 | A13 服部半藏 | 忍法．換位之術（A13W2） | KV 沒寫 AbilityCastRange，推定 Dota 取 0：要走到與自己的幻象相接才能換位（2026-10-29 依 D13 推定，無法實測） | 不限距離或與分身技能相同的距離 | 中 | 修（要先決定距離） ✓ 2026-10-29 已改 v3 文件（決定：不限距離），quirk 改 `fixed` | packages/hattori_hanzo/v3.lua · a13_substitution（A13W2） · npc/A_Oda/A13.txt:378-413 |
| FC-133 | A32 瀧川一益 | 魂刃（A32F） | 切換技死亡不關閉，但 modifier_A32F 死亡時移除；復活後物理普攻仍被取消，卻沒有魔法傷害（2026-10-29 依 D13 推定） | 死亡時關閉切換，或復活後重掛 modifier | 高 | 修 ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | packages/takigawa_kazumasu/v3.lua · a32_soul_blade（A32F） · npc/A_Oda/A32.txt |
| FC-134 | 下忍 | 召喚 | 被頂替的舊下忍死亡時，依自己的編號清空主人的槽位、不檢查槽位裡是不是自己，同編號的新下忍因此脫離名額（與 FC-093 同一段程式，資料已照原作寫） | 只清空自己所在的槽位 | 中 | 修（與 FC-093 一起） ✓ 2026-10-29 已改 v3 文件，quirk 改 `fixed` | items/pilot_a.lua · item_ninja1 · vscripts/items/Addon_Items/tofar_goback.lua:645-653 |
| FC-135 | B08 淺井長政 | 活心．魂（B08R） | 觸發時刷新增益持續時間的兩行（264、268）引用未定義的 hCaster，刷新不會發生 | 每次觸發都刷新持續時間 | 低 | 修 ✓ 2026-10-30 v3 文件本來就依意圖寫，quirk 改 `fixed`（工作線 A） | B08_Asai_Nagamasa.lua:264,268 |

---

## 建議的預設決定

**建議整組修**：
- FC-132～FC-134（2026-10-29 新增）：建議都修。FC-132 要先決定距離；FC-134 與 FC-093 一起修。

- 一、共用欄位互相覆寫（FC-001～FC-022）：只影響多個效果同時存在的情況，修正後單一技能的數值不變。其中 FC-006～FC-008、FC-022 選「修」即維持現狀。
- 二、護盾或保命擋不住致死傷害（FC-023～FC-037）：說明都寫「吸收」「忽略」「不會低於」，修正後玩家看到的效果就和說明一致。
- 四、腳本報錯或寫錯名稱（FC-046～FC-069）：修法多半是一個名稱。FC-058（伺服器卡死）、FC-061（C21T 宮本武藏大招）優先。
- 六、可被濫用或數值異常（FC-078～FC-101）：冷卻重置、永久單位、全免傷等可以穩定利用的漏洞。
- 七、死亡或驅散後狀態殘留（FC-102～FC-111）。
- 九、其他（FC-115～FC-131）：FC-129～FC-131 交給規則層統一修。

**建議大致修、個別照原作**：

- 五、說明與數值不符：FC-071（B02R 二十枚）照原作，其餘修。FC-076（C19T）修之前先確認「無法被指定」的秒數。
- 八、多件同名道具：FC-112 修；FC-113、FC-114 照原作。

**建議以照原作為主**：

- 三、層數只顯示：FC-038（A14R）、FC-039（A14T）、FC-041（B21T）、FC-044（池田鬼神丸）照原作，因為修了數值會倍增，需要重新調整數值；FC-040（A31W）、FC-042（般若鬼丸）、FC-043（千鳥一文字）、FC-045（鞍切）修。

照這個預設，**不修**的共 7 項：FC-038、FC-039、FC-041、FC-044、FC-071、FC-113、FC-114。其他 124 項修（FC-076 要先確認秒數），其中 6 項（FC-006、FC-007、FC-008、FC-022、FC-063、FC-121）資料已照意圖，不用改。

精簡的回覆可以寫：「照建議預設」。也可以寫：「修 FC-001～FC-037、FC-040、FC-042、FC-043、FC-045～FC-070、FC-072～FC-112、FC-115～FC-131；其餘不修」。

## 十、D22 舊版技能組（_old、_v20）

**2026-11-07 主 session 決定：比照 D13 全部修。** 工作線 D 普查 95 份變體文件（85 份 `_old`、10 份 `_v20`）時找到的原作 bug，資料都照設計意圖寫，quirk 標 `fixed`，text 開頭是下表的編號。沿用本體已修的同一段程式時，quirk 寫本體的 FC 編號（FC-015、016、024、025、046、050、056、062、078、084、090、104、110、116、123、129、131、133），不另編號。同一招有兩處修正時各給一個編號。

**測試**：fix_candidates 的測試要英雄上架後才能載入（`learned(<package>, …)`），變體英雄由 E 線上架後，D 依本表逐項補測試，並在 `check_fixed_quirk` 接上；目前全部是「已改 v3 文件，測試待上架後補」。

還不能修、保留 pending 或 fix_candidate 的項目：伊勢姬（舊版）心靈控制的「暫時換隊伍、交給施法者操作」（缺 host 支援，A 線已轉 B 線；瀕死判斷照 FC-024 已修），以及小島貞興（舊版）法術護盾淨化時連同剛套上不到 0.1 秒的受保護狀態一起移除（`remove_status` 依已存在時間篩選，同樣等 B 線）。風魔小太郎（舊版）四方導雷的回打已用 A 線的 `chain.on_end`（3064fce3）寫成。

| 編號 | 英雄 | 文件 · 招式 | 原作實際行為與修法 |
|---|---|---|---|
| FC-136 | C01_old | akechi_mitsuhide_old · `c01_old_honnoji`（C01T_old） | D22 舊版，比照 D13 修：C01T_old 呼叫本體的 C01T_Mitsuhide_Akechi，火球傷害讀 GetSpecialValueFor（…） |
| FC-137 | C25_old | chacha_old · `c25_old_spin_slash`（C25W_old） | D22 舊版，比照 D13 修：原作 RunScript 呼叫不存在的 C25W_OnSpellStart（C25.lua 只有 C25W_OnSpellStart_old，vscripts/heroes/C_Neutral/C25.lua:200-233），施放後只扣魔、進冷卻，沒有任何效果；照設計意圖改用 C25W_OnSpellStart_old 的行為。 |
| FC-138 | C25_old | chacha_old · `c25_old_dagger_throw`（C25E_old） | D22 舊版，比照 D13 修：原作 RunScript 呼叫不存在的 C25E（C25.lua 只有 C25E_old，vscripts/heroes/C_Neutral/C25.lua:246-277），短刃不會飛出、modifier_C25E 永遠不套，施放後沒有任何效果；照設計意圖改用 C25E_old 的行為。 |
| FC-139 | C25_old | chacha_old · `c25_old_dazzle`（C25R_old） | D22 舊版，比照 D13 修：原作 RunScript 呼叫不存在的 C25R_OnSpellStart（C25.lua 只有 C25R_OnSpellStart_old，vscripts/heroes/C_Neutral/C25.lua:235-244），只有致盲生效、從不消魔（KV 註解也寫「沒有弄」消魔）；照設計意圖與原作說明改用 C25R_OnSpellStart_old：25% 機率 ReduceMana。 |
| FC-140 | C25_old | chacha_old · `c25_old_bloodletting`（C25T_old） | D22 舊版，比照 D13 修：原作 OnSpellStart 呼叫不存在的 C25T_OnSpellStart、bleeding 的 OnCreated／OnIntervalThink 呼叫不存在的 modifier_C25T_bleeding_OnIntervalThink（C25.lua 只有加 _old 尾碼的版本，vscripts/heroes/C_Neutral/C25.lua:280-327），主動施放後沒有任何效果，只剩被動加攻；照設計意圖改用 _old 函式的行為。 |
| FC-141 | C25_old | chacha_old · `c25_old_flash_slash`（C25D_old） | D22 舊版，比照 D13 修：原作 RunScript 呼叫不存在的 C25D_Action1（C25.lua 只有 C25D_Action1_old，vscripts/heroes/C_Neutral/C25.lua:187-198），施放後不瞬移、不傷害；照設計意圖與原作說明改用 C25D_Action1_old 的行為。 |
| FC-142 | A28_old | hashiba_hideyoshi_old · `a28_old_healing_wind`（A28E_old） | D22 舊版，比照 D13 修：原作每個目標套上 modifier_arc_lightning_datadriven 時，OnCreated 先跑 A28E_Jump（友軍分支 target:Heal(AbilityDamage)），同一個 OnCreated 的 Heal 動作再治療一次 %AbilityDamage：每個目標實際治療兩次。原作說明不存在；KV 與腳本是兩條各自完整的治療路徑重疊，推定原意只治療一次。修法：每個目標治療一次 AbilityDamage。 |
| FC-143 | A07_old | honda_tadakatsu_old · `a07_old_mightiest_awakening`（A07T_old） | D22 舊版：A07T_Transform 照設計意圖找舊版技能（A07W_old／A07E_old），免費施放、冷卻不變 |
| FC-144 | A07_old | honda_tadakatsu_old · `a07_old_mightiest_awakening`（A07T_old） | D22 舊版，比照 D13 修：原作 A07T_Transform 找 FindAbilityByName（…） |
| FC-145 | A07_v20 | honda_tadakatsu_v20 · `a07_v20_shattering_spear`（A07R_20） | D22 舊版，比照 D13 修：暴擊共用舊版的 A07R_old，它讀 crit_persent，A07R_20 沒有這個鍵（KV 把 140/160/180/200 寫成第二個 CleavePercent），GetLevelSpecialValueFor 回 0，A07R_critical 回 0，原作實際沒有暴擊、只播音效與動作。修法：照設計意圖以 31/101 機率暴擊，倍率取 KV 的 140～200%。 |
| FC-146 | A07_v20 | honda_tadakatsu_v20 · `a07_v20_mightiest_awakening`（A07T_20） | D22：補上原作 A07T_Transform 的連發與解暈（2.0 版腳本找不到 A07W 而中斷） |
| FC-147 | A07_v20 | honda_tadakatsu_v20 · `a07_v20_mightiest_awakening`（A07T_20） | D22 舊版，比照 D13 修：照 A07T_Transform 的設計意圖（同本體 a07_strongest），施放時以目前等級免費施放毫髮無傷（A07W_20）與赤雷擊（A07E_20）：原作先 EndCooldown、CastAbilityImmediately、補回法力，再以施放前的剩餘冷卻 StartCooldown，所以兩招的冷卻不受影響；並移除敵方施加的暈眩類 modifier。未學的招不施放（同本體）。換 A07D2 不適用（2.0 版的 D 是雷擊切換，沒有 A07D2）；模型放大屬呈現。原作：A07T_Transform 找 FindAbilityByName（…） |
| FC-148 | B15_v20 | imagawa_yoshimoto_v20 · `b15_v20_flame_arrow`（B15E_20） | D22 舊版，比照 D13 修：2.0 版本體的 modifier_searing_arrow 命中時呼叫的是幻象用的 B15E_attack2（本體與舊版是 B15E_attack1／B15E_attack3）：它讀 caster.owner.nobuorb1，而英雄本身與 B15T_20 建立的幻象都沒有設 owner（只有本體 B15T_create_illusion 會設），nil 取欄位報錯中止，damage_bonus 15/30/45/60 與 damage_bonus2 從未造成傷害。修法照 B15E_attack2 的設計意圖：本體命中時檢查自己的法球槽（nobuorb1 == B15E_1，以 orb_slot_free 表達）加 damage_bonus 魔法傷害；幻象命中時檢查本體開關加 damage_bonus2。B15E_attack2 沒有建築折減，照它不折減。加傷走 AMHC:Damage（義元已死不傷害）；幻象的魔法傷害另由規則層乘 illusion_damage 0.3。 |
| FC-149 | A02_old | inahime_old · `a02_old_venom_arrow`（A02W_old） | D22 舊版：拿掉永遠成立的法球槽檢查（本體同一段已註解），照原作接下來的法力檢查與扣魔 |
| FC-150 | A02_old | inahime_old · `a02_old_venom_arrow`（A02W_old） | D22 舊版，比照 D13 修：原作 A02W_old_OnAttackStart 先檢查 caster.nobuorb1 ~= \ |
| FC-151 | C06_old | ishikawa_goemon_old · `c06_old_murder_and_loot`（C06E_old） | D22 舊版，比照 D13 修：C06E_old_OnSpellStart 在給錢與擊殺之前呼叫 ExecuteOrderFromTable(order)，order 是未宣告的全域變數：通常是 nil，執行期報錯，後面的給錢與擊殺都不會發生，只剩粒子與音效；若別的腳本（B18.lua:174、C13.lua:105 等）留下全域 order，則會重送那道舊指令。修法：照說明殺死目標並給四倍賞金，不送任何指令。 |
| FC-152 | A03_old | ishikawa_kazumasa_old · `a03_old_giant_gu`（A03T_old） | D22：原作 A03.lua:131-133 索引多一（實際 3000/4000/4000），照 KV 與原作說明修正 |
| FC-153 | A03_old | ishikawa_kazumasa_old · `a03_old_giant_gu`（A03T_old） | D22：同 deer_hp，原作實際 60/70/70，照 KV 與原作說明修正 |
| FC-154 | A03_old | ishikawa_kazumasa_old · `a03_old_giant_gu`（A03T_old） | D22 舊版，比照 D13 修：原作 level = ability:GetLevel()（1 起算）直接傳給 GetLevelSpecialValueFor（0 起算）：一級讀到 60 秒／3000 生命，二、三級都讀到最後一格 70 秒／4000 生命（Dota 超出範圍取最後一格，推定）。修法：照 KV 與原作說明，生命 deer_hp 與存在時間 deer_life 依大招等級逐級取 2000/3000/4000、50/60/70。攻擊力公式 90／110 + 等級 × 90／110 用的是 1 起算等級，與說明的 200/300/400（平均）一致，不改。 |
| FC-155 | C04_old | izumo_no_okuni_old · `c04_old_witch_dance`（C04T_old） | D22 舊版，比照 D13 修：比照 FC-062（同一個 ShockTarget，照意圖給魔法護盾）。原作：Shock_C04_old 呼叫 ShockTarget，ability:ApplyDataDrivenModifier 套的 modifier_devil_supressor_armor 是 LinkLuaModifier 的 Lua modifier，C04T_old 的 KV 沒有 Modifiers 區塊，推定套用失敗，接著 FindModifierByName(...).caster 對 nil 取值報錯，大招只剩稱號特效與音效（另外兩個 RunScript）。修法：照腳本意圖對友方英雄（不含幻象）套 90 秒魔法護盾。見 assumptions.md。 |
| FC-156 | C04_v20 | izumo_no_okuni_v20 · `c04_v20_withering_step`（C04E_20） | D22 舊版，比照 D13 修：C04E_OnIntervalThink 以 GetSpecialValueFor（…） |
| FC-157 | C04_v20 | izumo_no_okuni_v20 · `c04_v20_gathering_dance`（C04R_20） | D22 舊版，比照 D13 修：C04R_20 的 Modifiers 區塊是空的，ApplyDataDrivenModifier(modifier_flood_walk) 推定套不上（本體 C04R 有宣告），原作被拉到中心的單位只有 0.3 秒 phased、沒有穿越地形。修法：照本體的設計意圖給 0.3 秒 phased＋flying_for_pathing 合併狀態。 |
| FC-158 | C04_v20 | izumo_no_okuni_v20 · `c04_v20_guardian_dance`（C04D_20） | D22 舊版，比照 D13 修：比照 FC-062（同一個 ShockTarget，照意圖給魔法護盾；舊版 C04T_old 同修）。原作：Shock_C04D_20 呼叫 ShockTarget，ability:ApplyDataDrivenModifier 套的 modifier_devil_supressor_armor 是 LinkLuaModifier 的 Lua modifier，C04D_20 的 KV 沒有 Modifiers 區塊，推定套用失敗，接著 FindModifierByName(...).caster 對 nil 取值報錯，技能沒有效果。修法：照腳本意圖對友方英雄（不含幻象）套 6 秒、45 點的魔法護盾。 |
| FC-159 | C16_old | kashin_koji_old · `c16_old_eight_gates`（C16T_old） | D22 舊版，比照 D13 修：原作每顆殺生石出生時搜尋周圍 275 的敵方英雄／一般單位（含魔免），以 ApplyDataDrivenModifier 套 modifier_stunned，但這個名稱不在 C16T_old 的 Modifiers 裡，套不上，實際沒有效果（本體 C16T 同樣的寫法）。依設計意圖寫成每顆落石暈眩 275 內的敵人 stun 秒（與 ActOnTargets 同一個暈眩狀態，重疊時刷新）。 |
| FC-160 | C08_old | kato_danzo_old · `c08_old_empty_zen`（C08D_old） | D22 舊版，比照 D13 修（同 FC-017 的共用檔案層級變數）：原作 c08d_lock 是檔案層級的全域變數，所有加藤段藏（含敵我雙方、換英雄後的實例）共用同一個 10 秒鎖。修法：鎖改成每個段藏自己身上的隱藏狀態。鎖不因化樹被驅散而解除。 |
| FC-161 | C08_old | kato_danzo_old · `c08_old_cloud_veil`（C08W_old） | D22 舊版，比照 D13 修：原作隱形本體是腳本寫死 20 秒的 modifier_invisible；KV 另一個 ApplyModifier modifier_invisible（Duration %C08W_old_duration）指向技能 Modifiers 裡沒有定義的名稱，推定 ApplyDataDrivenModifier 找不到而沒有效果（見 assumptions.md），所以二到四級在 20 秒後隱形就結束，加速、無碰撞、低攻擊優先度與破隱加傷卻持續到 C08W_old_duration。修法：隱形與增益同為 C08W_old_duration（KV 的設計意圖）。 |
| FC-162 | C08_old | kato_danzo_old · `c08_old_chidori`（C08E_old） | D22 舊版，比照 D13 修：原作 counter 是全域變數，每次施放歸零；兩次千鳥的落雷時間重疊時（不同段藏，或冷卻被重置）會共用計數，後一次把前一次歸零或提早結束。修法：每次施放各自三跳。 |
| FC-163 | C08_old | kato_danzo_old · `c08_old_chidori`（C08E_old） | D22 舊版，比照 D13 修：原作目標實體失效（例如召喚物被移除）時改用 GetCursorPosition 作為落雷位置，但之後 AMHC:CreateParticle 與 target:Stop() 對失效實體報錯，Timer 中斷。修法照腳本意圖：目標不存在時在施放當下的目標位置繼續落雷（單位目標技能的 cursor 即施放時目標位置），只略過 Stop()。 |
| FC-164 | C08_old | kato_danzo_old · `c08_old_crimson_lotus`（C08R_old） | D22 舊版，比照 D13 修：原作 AbilitySpecial move_speed −20/−30/−40/−50 沒有任何 modifier 引用，火柱實際不緩速。修法：推定設計意圖是火柱範圍內的英雄與部隊被緩速，加在 c08r_old_burn（MOVESPEED_BONUS_PERCENTAGE）；建築不受影響。推定，請工作線 D 確認。 |
| FC-165 | C10_old | kosokabe_chikayasu_old · `c10_old_black_dragon_wave`（C10W_old） | D22 舊版，比照 D13 修：原作以 ability.target[i] == target 反查「目前是第幾串」：兩串同時打到同一個單位時（冷卻 9 秒、彈跳最長約 4 秒，冷卻被重置時才會發生），後面的迴圈會把兩串都認成同一串，計數與命中表混在一起、其中一串提早或延後結束。修法：每串各自計數與命中表（chain 的語意）。 |
| FC-166 | C05_old | maeda_keiji_old · `c05_old_thunder_spear`（C05T_old） | D22 舊版，比照 D13 修：C05T_old_OnIntervalThink 在宣告 local target（292 行）之前就用 target 建粒子與 target:GetAbsOrigin()（287-291 行），讀到的是未定義的全域 target（nil），執行期報錯，每秒的純粹傷害從未發生，只剩命中當下的傷害與攻速降低。修法：照意圖每秒對帶電目標造成 damage_per_think 純粹傷害（魔免減半），來源是慶次。 |
| FC-167 | C19_old | matsuhime_old · `c19_old_paralyzing_poison`（C19W_old） | D22 舊版，比照 D13 修（同今川義元烈焰箭 FC-104 的修法）：原作 modifier_incapacitating_bite_datadriven 不是 Passive，死亡時被移除，但開關仍亮（推定），復活後沒有加攻也不扣魔，要手動關再開；資料在死亡時關閉開關。 |
| FC-168 | C19_old | matsuhime_old · `c19_old_surprise_raid`（C19R_old） | D22 舊版，比照 D13 修：原作 Teleport2 第一行 ProjectileManager:ProjectileDodge(keys.caster) 引用未定義的 keys（參數名是 event），執行期報錯中斷：施法成功（扣魔、進冷卻）但不傳送、不閃避、不緩速。依設計意圖寫成閃避投射物 → 傳送到目標點（FindClearSpace）→ 停止 → 緩速周圍 500 內的敵人（含魔免）。 |
| FC-169 | C19_old | matsuhime_old · `c19_old_inviolable`（C19T_old） | D22 舊版，比照 D13 修：原作主動重新套上 modifier_ninja_cloth 並 SetStackCount(2)，10 秒後設回 1；但 datadriven 的 EVASION_CONSTANT 不乘層數，全專案也沒有讀這個層數的程式，主動只有特效、沒有任何效果（耗魔 200～300、冷卻 100）。推定原意是兩層＝兩份閃避，資料寫成 10 秒內再加一份 bonus_evasion 閃避（多個閃避來源怎麼合成由規則層決定）。 |
| FC-170 | B28_old | matsunaga_hisahide_old · `b28_old_binding`（B28E_old） | D22 舊版，比照 D13 修：照本體 B28E（RunScript 傳 duration = %B28E_duration）補上持續時間：分傷只在引導期間、目標仍被暈眩時生效；暈眩時間 = 引導時間 × max(0.5, (600 − (max(OBB 距離, 200) − 200)) / 600)，時間到以 EndChannel(true) 結束引導；0.2 秒檢查在 modifier_B28E2 消失（施放其他技能）時解除暈眩。原作：舊版 OnSpellStart 的 RunScript B28E 沒有傳 duration（本體 B28.txt:175 有）：keys.duration 是 nil，AddNewModifier 套上不限時的分傷 modifier，接著第 22 行 (600 − r) / 600 * nil 執行期報錯，腳本後半（依距離縮短暈眩、時間到 EndChannel、0.2 秒檢查迴圈、鎖鏈粒子）都沒有執行。結果：caster.B28E_target 永遠不清空，久秀之後受到的所有傷害（任何來源）都有 30% 以純粹傷害打到最後一次綁住的目標，直到久秀死亡。 |
| FC-171 | B16_old | mochizuki_chiyome_old · `b16_old_smoke_bomb`（B16T_old） | D22 舊版，比照 D13 修：改成暈眩周圍 500 內的敵方英雄與部隊 5 秒（範圍沿用落葉散華，屬推定）。原作：OnSpellStart 的 Stun 目標是 CASTER，施放後千代女自己暈眩 5 秒，十秒的加速只剩後五秒能用。 |
| FC-172 | A31_old | mori_ranmaru_old · `a31_old_thunder_rain`（A31W_old） | D22 舊版，比照 D13 修：原作 second 從 0 開始、先加一再判斷 second <= 10，第一次在施放後一秒，共十一次；KV duration 4、AbilityDamage 180/300/420/540（四次 × 傷害）與日文說明「持續 4 秒」都顯示設計是四次。修法：tick_count = duration ÷ tick_interval = 4，施放後第 1～4 秒各一次，與箭雨特效的 4 秒一致。 |
| FC-173 | B36_old | naoe_kanetsugu_old · `b36_old_far_reach`（B36D_old） | D22 舊版，比照 D13 修：原作天生被動的 modifier_B36D_old 寫了 IsPurgable 1 與 IsDebuff 1，敵方的一般驅散就能把它當減益移除，之後失去 +572 射程直到重新取得（SetAttackCapability 不會還原，變成射程很短的遠程攻擊）。這是天生特性，照設計意圖修為不可驅散（dispel = none）。 |
| FC-174 | A15_old | nene_old · `a15_old_shadow_windmill`（A15R_old） | D22 舊版，比照 D13 修：原作 OnAttackLanded 呼叫的 A15R_OnAttackLanded／A15R_bounce_init（A15.lua:114-136）以 FindAbilityByName（…） |
| FC-175 | A15_old | nene_old · `a15_old_shadow_windmill`（A15R_old） | D22 舊版，隨上一條一起修：本體與 A15T 連動的部分（彈跳半徑 + (A15T 等級 − 1) × 50、modifier_A15T2 的 jump_max 翻倍、modifier_A15T 的彈跳吸血）在舊版沒有對應技能（舊版大招是 A15T_old 夢魘，沒有這些 modifier），推定都不生效：半徑固定 700、不翻倍、不吸血。 |
| FC-176 | A34_old | omatsu_old · `a34_old_snow_blessing`（A34T_old） | D22 舊版，比照 D13 修：原作 MANA_REGEN_CONSTANT 引用 %add_def_magic（魔抗的鍵），註解寫「回魔」，回魔數值因此綁在魔抗上。修法：回魔照註解視為設計本意，獨立成 mana_regen 一個值、自己掛在 mana_regen 屬性上，不再依附魔抗；KV 沒有專屬的回魔數字，數值沿用 10/20/30，日後調魔抗不會連帶改回魔。 |
| FC-177 | C13_old | ryuzoji_takanobu_old · `c13_old_wind_chain`（C13D_old） | D22 舊版，比照 D13 修：原作 C13D_old_OnSpellStart 只套 modifier_C13D_old，KV 的 AbilityDamage 200 沒有任何地方使用，實際不造成傷害；原作說明寫「給予200點傷害」。修法：照說明在套上狀態後造成 AbilityDamage 魔法傷害（AbilityUnitDamageType MAGICAL；推定以 ApplyDamage 實作，隆信死後施放中的不會發生，require_source_alive 不影響）。 |
| FC-178 | C13_old | ryuzoji_takanobu_old · `c13_old_tornado_vortex`（C13T_old） | D22 舊版，比照 D13 修：原作的吹飛寫在 Timer 第 134 跳（tsum 累加 0.03 超過 4）之後，但 Timers 每幀（30Hz，約 0.033 秒）才跑一次，134 跳要約 4.4 秒，比 4 秒引導還長：引導結束時 IsChanneling 為假，Timer 直接結束，實際上永遠不會吹飛。原作說明寫「拉扯持續4秒，結束後吹飛所有敵人」。修法：完整引導結束時（on_channel_end 且沒被打斷）對 300 內敵人套 1.5 秒捲入狀態並從隆信位置擊退 800（0.3 秒、擊退中暈眩）；被打斷時照原作不吹飛。 |
| FC-179 | B01_old | saika_magoichi_old · `b01_old_saika_corps`（B01E_old） | D22 舊版，比照 D13 修：依腳本原意，雜賀眾攻擊力 = 70 + 15 × 孫市等級（最小最大相同），生成當下取樣一次。原作：OnSpawn 的 B01E_old_CHECK 第一行 table.insert(caster.B01E, target)，但 caster.B01E 從未初始化（唯一的初始化在 B01.lua:91-92 已註解），執行期報錯中斷：設定攻擊力與粒子都不會執行，雜賀眾保留 KV 攻擊力 67～73，不隨等級成長。 |
| FC-180 | C14_old | saito_yoshitatsu_old · `c14_old_dragon_vein`（C14R_old） | D22 舊版，比照 D13 修：原作 MODIFIER_PROPERTY_INCOMING_DAMAGE_PERCENTAGE 寫正值 %damage_bonus（6～12），實際讓義龍受到的傷害提高，和技能名「強韌」與同時給的護甲相反；推定原意是減傷，資料寫成 incoming_damage_pct = -damage_bonus（受到的傷害降低 6%～12%）。另一種可能是原意為增傷（變數名 damage_bonus、本體 C14R 是攻擊加成），列為不確定，見 assumptions.md。 |
| FC-181 | C22_old | sasaki_kojiro_old · `c22_old_earth_swallow`（C22T_old） | D22 舊版，比照 D13 修：原作 GetLevelSpecialValueFor（…） |
| FC-182 | A35_old | shima_sakon_old · `a35_old_flame_guardians`（A35E_old） | D22 舊版，比照 D13 修：原作護衛還在時再施放，粒子迴圈從 A35Ecount + 1 跑到 A35Ecount + count + 1，實際多加一個（count + 1），上限八個；第一次施放才是 count 個。說明寫「釋放出 1/2/3/4 個」，迴圈終點是差一錯誤。修法：再施放與第一次一樣加 count 個，上限仍是八個。 |
| FC-183 | B34_old | takeda_katsuyori_old · `b34_old_rending_wound`（B34R_old） | D22 舊版，比照 D13 修：原作說明寫「對魔免傷害減半」，但 B34R_old 以 AMHC:Damage 對任何非建築目標都造成完整 dmg 純粹傷害，eventfordamage 也只處理魔法傷害，魔免目標實際不減半。照說明的設計意圖修為魔免目標乘 0.5。 |
| FC-184 | A04_old | takenaka_shigeharu_old · `a04_old_freezing_path`（A04E_old） | D22：補上缺少的 OnAbilityPhaseStart（記下前搖開始時的面向），比照本體 A04E |
| FC-185 | A04_old | takenaka_shigeharu_old · `a04_old_freezing_path`（A04E_old） | D22 舊版，比照 D13 修：原作 OnSpellStart 呼叫共用的 FireEffect_IcePath，第 22 行讀 ability.forwardVector；這個欄位只由 A04E_OnAbilityPhaseStart（A04_Takenaka_Juji.lua:9-11）寫入，而 A04E_old 的 KV 沒有 OnAbilityPhaseStart，所以 nil × 數字在執行期報錯，只有音效、耗魔與冷卻，冰道特效、傷害與暈眩都不會發生。修法：補上前搖開始時記下面向（同本體 A04E），照 FireEffect_IcePath 沿面向放出長度 GetCastRange 的冰道，沿線 floor(730/300)+1 = 三個半徑 150√2 的圓（間距 365），非建築、非魔免的敵人套 A04E_old 自己的 modifier_A04E：暈眩 stun（KV duration 1/1.5/2/2.5）並在 OnCreated 造成 damage（150/200/250/300）魔法傷害。 |
| FC-186 | C15_old | tamako_old · `c15_old_searing_arrow`（C15E_old） | D22 舊版，比照 D13 修（同今川義元烈焰箭 FC-104 的修法）：原作 modifier_C15E_old 不是 Passive，死亡時被移除，但開關仍亮（推定），復活後不扣魔也不加傷，要手動關再開；資料在死亡時關閉開關。 |
