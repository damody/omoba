# fix-candidates 尚未掛 runtime 測試項目的驗證對照表

- **產生依據**：conformance 全量結果 `target/e229/v3c_full661/`（檔案時間 2026-11-02 12:43），harness `scripts/base_content/src/v3_conformance_tests.rs`（2026-11-02 12:08 版），情境由 `docs/character_pipeline/tools/v3_scenarios.lua --stdout <SID>` 重新輸出後逐筆讀 `expect`。日期 2026-11-02。
- **項目範圍**：`target/m3p/t258/fcleft.lua` 列出的 78 項（`check_fixed_quirk` 回 `FC-xxx is fixed but has no test here yet`）。逐項判斷 53 項，另 25 項只標狀態。
- **判斷標準**：D 情境要算數，status 必須是 pass，而且 harness 真的判定到這個修正涉及的數值或行為。只有施放成功不算，unsupported 不算。拿不準就不算，標為只有文件式。
- **文件式測試**：`scripts/tests/v3_fixed_quirks_test.lua`，本次實跑 45 tests passed（Lua，未跑 cargo）。多項合併的測試名簡寫如下：
  - `waves 1-4`：`waves 1-4: A26 bombs, A12 field, A11 gaze, C14 slow, B34 howl, A36 brew, C24 self cast, B02 stun, B09 damage, B18 sight, A33 step, B11 illusion`
  - `waves 5-8`：`waves 5-8: each fix holds in its document`
  - `wave 9`：`wave 9: C08 steal never loops, counts per Danzo; C07 clouds on their own; B16 hound shares before; B28 serpent cap`
  - `shared shields`：`shared AphoticShieldRemaining shields (FC-002/032, 003/033, 004/034, 023/115) absorb on their own`
- **含產生碼**：該文件式測試除了文件，也比對 `v3_rust.lua` 產生的 Rust。這類仍歸只有文件式。53 項裡沒有判定只落在產生碼的項目。
- 依據欄的 `pkg:行` 指 `docs/character_pipeline/packages/<pkg>/v3.lua` 的 quirk 行。

## 統計（53 項）

| 驗證方式 | 項數 | 項目 |
|---|---|---|
| runtime 測試 | 0 | |
| D 情境 | 5 | FC-012、048、060、072、080 |
| 只有文件式 | 48 | 其中可由 D 補判定 48、不適合 runtime 0 |
| 只有產生碼檢查 | 0 | |

## 逐項判斷

| FC | 英雄／招 | 修正類型 | 驗證方式 | 依據 | 備註 |
|---|---|---|---|---|---|
| FC-001 | A24 佐久間盛政／血盾集陣 | 共用護盾量改私有 | 只有文件式：`FC-001 / FC-027 / FC-083 A24 blood shield is private, absorbs, and the charge stun is never negative` | sakuma_morimasa:79；D 只有 passive_stats pass，trigger 全 unsupported | 可由 D 補判定：與第二份護盾同在時各扣各的 |
| FC-002 | B36 直江兼續／劃心為籬 | 共用護盾量改私有 | 只有文件式：`shared shields` | naoe_kanetsugu:191；b36e_shield 的 stats、expires、reapply pass，都只看單一護盾 | 可由 D 補判定：同上 |
| FC-003 | C06 石川五右衛門／闊氣攻擊 | 共用護盾量改私有 | 只有文件式：`shared shields` | ishikawa_goemon:199；c06r_shield 的 reapply、expires pass，只看單一護盾的時間 | 可由 D 補判定：同上 |
| FC-004 | C12 大友宗麟／空間撕裂裝置 | 共用護盾量改私有 | 只有文件式：`shared shields` | otomo_sorin:206；normal 判傷害；c12e_shield/expires unsupported（施放後沒觀察到） | 可由 D 補判定：先讓護盾放得上，再比兩份護盾 |
| FC-006 | A08 榊原康政／檄文怒嗥 | 全域變數；範圍全暈、無傷害 | 只有文件式：`FC-006 A08 war cry stuns everyone in range and deals no damage`（含產生碼） | sakakibara_yasumasa:115；a08e_stun/expires pass 只證明單一敵人被暈；normal 的 statuses 2 不是證據（見不一致 1） | 可由 D 補判定：兩個敵人都暈、沒有傷害 |
| FC-007 | A11 齋藤道三／老奸巨猾 | 全域變數改施放時捕捉 | 只有文件式：`FC-007 A11 old fox captures its amount when the delay is scheduled`（含產生碼） | saito_dosan:243；on_ability_executed unsupported，a11r_locker/expires unsupported | 可由 D 補判定：兩次延遲傷害各打各的數值 |
| FC-009 | B30 柿崎景家／邪王煉獄斬 | 全域計時改每目標各自計時 | 只有文件式：`FC-009 B30 purgatory slash ignites each target on its own clock` | kakizaki_kageie:274；b30t_burn 的 periodic、expires pass，只有一個目標 | 可由 D 補判定：兩個目標各自計時 |
| FC-010 | A29 石田三成／衰心靈光 | 緩速改獨立疊加 | 只有文件式：`waves 5-8` | ishida_mitsunari:203；a29r_slow 的 stats、expires 都 unsupported（沒觀察到） | 可由 D 補判定：兩份緩速並存、各自到期 |
| FC-011 | B34 武田勝賴／天目長嚎 | 緩速改獨立疊加 | 只有文件式：`waves 1-4` | takeda_katsuyori:271；b34t_slow_instance/stats pass 只判單份數值，沒有 reapply 情境 | 可由 D 補判定：兩份並存、早到期的不清掉另一份 |
| FC-012 | C14 齋藤義龍／火龍化身 | 緩速改用自己的鍵 | D 情境：`c14_dragon_form/r1~r3/normal/c14t_slow/stats`，判定持有者的 `c14t_slow:slow` 列帶文件的 slow_pct 與 attack_speed | saito_yoshitatsu:299；三級全 pass（2 stats）；文件式 `FC-012 C14 dragon form slows through its own status, not the A14T key` | 判的是鍵與數值。沒有實際放 A14T 同場，也不需要：兩者已不同鍵 |
| FC-014 | A26 濃姬／灼熱爆裂投擲 | 大小炸彈各自結算 | 只有文件式：`waves 1-4` | nohime:251；normal、on_attack_hit、a26t_stun/expires pass，沒有大小炸彈同時飛行 | 可由 D 補判定：同時飛行時比對傷害與暈眩 |
| FC-015 | B13 百地三太夫／忍法．催眠 | 醒來判定作用對象 | 只有文件式：`waves 5-8` | momochi_sandayu:126；on_take_damage unsupported | 可由 D 補判定：兩人沉睡，打誰誰醒 |
| FC-016 | C07 立花道雪／雷神之怒 | 每朵雲各自運作 | 只有文件式：`wave 9` | tachibana_dosetsu:321；periodic pass 只看一朵雲 | 可由 D 補判定：兩朵雲，舊雲到期新雲仍可動 |
| FC-017 | C08 加藤段藏／鬼吞之術 | 計數改每英雄、層數顯示 | 只有文件式：`wave 9` | kato_danzo:505；on_ability_executed 全 unsupported | 可由 D 補判定：層數逐次減 1；雙方同英雄各自計數 |
| FC-038 | A14 前田利家／疊擊槍術 | 層數真的影響攻擊力 | 只有文件式：`FC-038 A14 stacked spear: stacks grant attack, full stacks equal the original +15` | maeda_toshiie:210；a14r_stack/stats pass，但接受 1～滿層的任一倍數，原作固定 +15 也會過（見不一致 2） | 可由 D 補判定：k 層時數值等於每層值 × k |
| FC-039 | A14 前田利家／狂怒．槍之又左 | 要施放才生效、每層攻速 | 只有文件式：`FC-039 A14 spear fury: attack speed only while cast, per stack, full stacks equal the original` | maeda_toshiie:302；a14t_aspd_stack/stats 同上容許；on_attack_hit 與 on_removed unsupported | 可由 D 補判定：未施放無加成、k 層數值、結束歸零 |
| FC-040 | A31 森蘭丸／響雨 | 層數真的影響緩速 | 只有文件式：`waves 5-8` | mori_ranmaru:69；a31w_slow/stats pass 只判一層的值（一至三級可與原作 5% 區分），沒判疊層 | 可由 D 補判定：第二次命中後為 2 層、數值加倍 |
| FC-041 | B21 馬場信房／不死鬼美濃 | 層數真的影響數值 | 只有文件式：`FC-041 B21 undying demon: one stack per 12% health lost, up to 8, full stacks equal the original` | baba_nobufusa:246；b21t_bonus/stats 與 on_take_damage unsupported | 可由 D 補判定：降血後比對層數與數值 |
| FC-047 | A12 大谷吉繼／智芒乍現 | 補上失效的強化暈眩 | 只有文件式：`FC-047 A12 flash of insight, empowered, stuns and spends the empowerment` | otani_yoshitsugu:83；normal 只判未強化的傷害；a12w_stun/expires unsupported | 可由 D 補判定：強化施放後暈眩且消耗強化 |
| FC-048 | A12 大谷吉繼／為智畏服 | 補上失效的緩速 | D 情境：`a12_awe/r1~r4/normal/a12r_mark/stats`，判定標記的 slow 列等於 15/20/25/30% | otani_yoshitsugu:273；四級全 pass（1 stats）；文件式 `FC-048 A12 awe slows the enemies it marks` | |
| FC-050 | A15 寧寧／忍法．影風車 | 補上彈跳到侍大將的分支 | 只有文件式：`FC-050 A15 windmill hits a general for half, heals and keeps bouncing`（含產生碼） | nene:286；on_attack_hit 與 a15r_hit/expires unsupported | 可由 D 補判定：侍大將受一半傷害、吸血、繼續彈跳 |
| FC-051 | A15 寧寧／影月攻勢 | 補上傷害類型 | 只有文件式：`waves 5-8` | nene:383；on_attack_launch 與 on_attack_hit unsupported | 可由 D 補判定：分裂飛標的物理傷害量 |
| FC-052 | B11 菊姬／忍法．截殺術 | 分身拿本體剩餘時間 | 只有文件式：`waves 1-4` | kikuhime:194；b11t_enable/expires unsupported；with_b11t_enable pass 沒讀時間 | 可由 D 補判定：分身狀態時間等於本體剩餘 |
| FC-053 | B18 湖衣姬／大地魔爪 | 補上偵測隱形 | 只有文件式：`waves 1-4` | koihime:307；normal 只判做出魔爪；b18t_claw_ai/periodic unsupported | 可由 D 補判定：半徑內隱形敵人現形 |
| FC-056 | B32 上杉謙信／神閃 | 短距離除以零 | 只有文件式：`waves 5-8` | uesugi_kenshin:162；normal pass，但 harness 距離為 cast_range 1000 × 0.5 夾到 400，不在小於 300 的出錯範圍 | 可由 D 補判定：距離 200 施放時冰道命中 |
| FC-060 | C11 立花宗茂／西國無雙 | 補上暈眩 | D 情境：`c11_unrivaled/r1~r3/normal/c11t_stun/expires`，判定施放後目標持有 c11t_stun，不超過 stun_duration 且會結束 | tachibana_muneshige:384；三級全 pass；文件式 `waves 5-8` | 只判上限（見不一致 4）。另外 `fix_candidates::fc059_c11t_bolt_arcs_between_enemies` 斷言第二個敵人持有 c11t_stun，測試已 pass |
| FC-070 | A23 宇喜多秀家／鬼術．墜火 | 波數上限 10 | 只有文件式：`FC-070 A23 falling fire calls down at most max_wave (10) waves` | ukita_hideie:259；normal 沒有傷害註記；periodic 判的是燃燒，不是波數 | 可由 D 補判定：整段引導的波數 |
| FC-071 | B02 風魔小太郎／十八方手裏劍 | 枚數 18 | 只有文件式：`FC-071 B02 eighteen shuriken throws eighteen` | fuma_kotaro:304；normal 的 damage 5/10/15/20 只比第一枚（見不一致 5） | 可由 D 補判定：命中 18 次、總傷害 90/180/270/360 |
| FC-072 | B02 風魔小太郎／四方導雷 | 補上起手暈眩 | D 情境：`b02_four_thunders/r1~r3/normal/b02t_first_stun/expires`，判定施放後目標持有 b02t_first_stun，不超過 first_stun 且會結束 | fuma_kotaro:373；三級全 pass；文件式 `waves 1-4` | 用自己的 runtime id，與每擊的 b02t_stun 分得開。只判上限（見不一致 4） |
| FC-073 | B09 真田昌幸／反噬裂隙 | 固定傷害照 KV | 只有文件式：`waves 1-4` | sanada_masayuki:90；normal 無傷害註記（傷害在 repeat 與條件內）；on_ability_executed unsupported | 可由 D 補判定：比對固定傷害 20/30/40/50 |
| FC-074 | B12 山本勘助／實界滑移 | 不移動友軍 NPC | 只有文件式：`waves 5-8` | yamamoto_kansuke:83；normal 只有敵人目標 | 可由 D 補判定：友方小兵不動、友方英雄被拉 |
| FC-075 | C12 大友宗麟／光學遮罩 | 攻擊、施法解除隱形 | 只有文件式：`waves 5-8` | otomo_sorin:260；reapply pass 只判時間；c12r_invis/expires unsupported | 可由 D 補判定：攻擊、施法後狀態移除 |
| FC-080 | A05 酒井忠次／誓伐狀 | 標記消失時移除免傷 | D 情境：`a05_crusade_affidavit/r1~r3/trigger/a05t_mark/on_removed`，判定驅散敵人身上的標記後，施法者的 a05t_oath 被移除 | sakai_tadatsugu:338；三級全 pass（on_removed: a05t_oath taken off）；文件式 `waves 5-8` | 目標死亡路徑與「只免疫標記目標以外的傷害」沒有判到（a05t_oath/on_take_damage_pre unsupported） |
| FC-081 | A13 服部半藏／影惑之術 | 換回保留冷卻 | 只有文件式：`waves 5-8` | hattori_hanzo:241；swap/a13_substitution pass 判換入招的種子冷卻，沒讀換回後 A13W 的冷卻（見不一致 3） | 可由 D 補判定：換回後冷卻約 15 − swap_window |
| FC-083 | A24 佐久間盛政／血之突進 | 暈眩下限 0 | 只有文件式：`FC-001 / FC-027 / FC-083 A24 blood shield is private, absorbs, and the charge stun is never negative` | sakuma_morimasa:242；a24e_stun/expires unsupported（值解不開）；normal 距離沒超過門檻 | 可由 D 補判定：距離超過門檻時沒有長暈 |
| FC-085 | A32 瀧川一益／怨靈飛翔斬 | 換回保留冷卻 | 只有文件式：`waves 5-8` | takigawa_kazumasu:83；這招沒有 swap 情境，normal 只判位移 | 可由 D 補判定：補 swap 情境與換回後冷卻 |
| FC-086 | B17 霧隱才藏／覓心斬 | 幻象不重置大招 | 只有文件式：`waves 5-8` | kirigakure_saizo:374；on_ability_executed unsupported；on_attack_launch pass 判的是解除狀態 | 可由 D 補判定：打死幻象不重置、打死英雄重置 |
| FC-088 | B24 秋山信友／大地禁錮 | 換回保留冷卻 | 只有文件式：`waves 5-8` | akiyama_nobutomo:402；swap/b24_shatter pass，但帶 initial_cooldown，policy 不判（見不一致 3） | 可由 D 補判定：換回後大絕冷卻仍在跑 |
| FC-090 | C09 立花誾千代／霜刺 | 對建築不多打、不扣魔 | 只有文件式：`FC-090 C09 frost thorn adds no extra attack on buildings` | tachibana_ginchiyo:73；on_attack_windup 與 on_attack_hit pass 都打英雄 | 可由 D 補判定：打建築時不扣魔、攻擊次數不加倍 |
| FC-091 | C24 柳生宗嚴／最期一生 | 自施不加倍 | 只有文件式：`waves 1-4` | yagyu_muneyoshi:294；normal 目標是友方英雄 | 可由 D 補判定：自施時回血一次、增益一份 |
| FC-105 | B21 馬場信房／不死鬼美濃 | 升級後重算加成 | 只有文件式：`waves 5-8` | baba_nobufusa:248；passive_stats 以外全 unsupported | 可由 D 補判定：帶加成升級、回滿血後加成消失。文件式判定過寬（見不一致 7） |
| FC-115 | A04 竹中重治／霜之庇護 | 重施補滿吸收量 | 只有文件式：`shared shields` | takenaka_shigeharu:286；reapply pass 只判時間；on_refreshed unsupported | 可由 D 補判定：吸收一部分後重施，剩餘量回滿 |
| FC-116 | A11 齋藤道三／陰邪凝視 | 目的地公式 | 只有文件式：`waves 1-4` | saito_dosan:166；normal 的 statuses 3 不是證據；a11e_mesmerize/periodic unsupported | 可由 D 補判定：目標走向雙方之間的點 |
| FC-117 | A15 寧寧／忍法．月擊 | 上限差一 | 只有文件式：`waves 5-8` | nene:227；normal、expires pass，沒有計次 | 可由 D 補判定：同一單位最多被打 4 次 |
| FC-119 | A17 佐佐成政／疾射 | 只對遠程攻擊觸發 | 只有文件式：`waves 5-8` | sassa_narimasa:159；on_attacked_hit unsupported | 可由 D 補判定：近戰不觸發、遠程觸發 |
| FC-120 | A18 稻葉一鐵／頑強戰法 | 溢出照魔抗 | 只有文件式：`FC-026 / FC-120 A18 shield absorbs magic damage before it settles` | inaba_ittetsu:166；periodic 20 秒 unsupported；`absorb.rs` 的單元測試沒有魔抗 | 可由 D 補判定：減免後扣盾、溢出。FC-026 那批測試若判到可一併算 |
| FC-121 | A21 瑞龍院日秀／偽情報 | 封鎖所有指令 | 只有文件式：`FC-121 A21 false intel locks every order of the unit itself` | zuiryuin_nisshu:388；on_expired pass 只判移動 | 可由 D 補判定：六種指令都被拒。原作的多選漏洞本身不可重現，只能判封鎖 |
| FC-122 | A29 石田三成／衰心靈光 | 上限改看目標法力 | 只有文件式：`waves 5-8` | ishida_mitsunari:201；on_mana_spent unsupported | 可由 D 補判定：三成沒魔時目標仍多掉法力 |
| FC-123 | A35 島左近／焰襲 | 落點找空地 | 只有文件式：`FC-123 A35 flame raid lands on clear ground` | shima_sakon:95；normal 落在空地，沒有障礙 | 可由 D 補判定：落點有阻擋時位置被推開 |
| FC-124 | B07 足利義昭／戰線國崩隊 | 出生滿血 | 只有文件式：`waves 5-8` | ashikaga_yoshiaki:168；normal 判數量與時效，不讀生命 | 可由 D 補判定：二到四級出生時 hp 等於 mhp |
| FC-126 | C18 小早川隆景／單馬沓履 | 鎖定對象改敵人 | 只有文件式：`FC-126 C18 lone rider locks the dragged enemy's blink items, not Takakage's`（含產生碼） | kobayakawa_takakage:119；c18w_hold/on_created unsupported | 可由 D 補判定：敵人的閃現道具進冷卻、隆景的沒有。harness 內沒查到給道具的輔助函式 |
| FC-132 | A13 服部半藏／換位之術 | 不限距離 | 只有文件式：`waves 5-8` | hattori_hanzo:298；normal 只在預設距離判施放 | 可由 D 補判定：遠距離不走路即可換位 |
| FC-135 | B08 淺井長政／活心．魂 | 每次觸發刷新時間 | 只有文件式：`FC-135 B08 living soul refreshes its duration on every trigger`（含產生碼） | asai_nagamasa:228；on_ability_executed unsupported | 可由 D 補判定：再次觸發後時間回滿 |

## 其他 25 項（不判斷）

| FC | 英雄／招 | 驗證方式 |
|---|---|---|
| FC-023 | A04／霜之庇護 | runtime：批次進行中 |
| FC-026 | A18／頑強戰法 | runtime：批次進行中 |
| FC-027 | A24／血盾集陣 | runtime：批次進行中 |
| FC-028 | A33／鬼軀之步 | runtime：批次進行中 |
| FC-030 | B16／獸靈同化 | runtime：批次進行中 |
| FC-031 | B26／上野武勇 | runtime：批次進行中 |
| FC-032 | B36／劃心為籬 | runtime：批次進行中 |
| FC-033 | C06／闊氣攻擊 | runtime：批次進行中 |
| FC-034 | C12／空間撕裂裝置 | runtime：批次進行中 |
| FC-035 | C23／以血償血 | runtime：批次進行中 |
| FC-082 | A13／影惑之術 | runtime：批次進行中 |
| FC-084 | A28／鳳凰召來 | runtime：批次進行中 |
| FC-087 | B23／魂饗 | runtime：批次進行中 |
| FC-103 | A06／赤焰流 | runtime：批次進行中 |
| FC-106 | B21／不死鬼美濃 | runtime：批次進行中 |
| FC-107 | B23／亡靈之契 | runtime：批次進行中 |
| FC-108 | B24／嘲諷 | runtime：批次進行中 |
| FC-109 | B24／嘲諷 | runtime：批次進行中 |
| FC-111 | C12／定位導彈 | runtime：批次進行中 |
| FC-118 | A16／血殤靈光 | runtime：批次進行中 |
| FC-054 | B20／強韌 | B 自查 |
| FC-055 | B27／騎術 | B 自查 |
| FC-129 | C06／偷天換日 | B 自查 |
| FC-130 | C06／殺人越貨 | B 自查 |
| FC-131 | C06／豪奪 | B 自查 |

## 不一致與判定問題

行號都是 `scripts/base_content/src/v3_conformance_tests.rs`，另有註明者除外。

**情境 pass，但判定不到或判定過寬**

1. `statuses N` 不能當證據（3297-3301 行）。`check_statuses` 不把 `runtime_id_default = true` 的狀態算成缺少，而註記仍寫 `statuses N`。看過的情境都是 default，例如 `a08_war_cry/r1/normal` 的 statuses 2：狀態沒上也一樣 pass。
2. 疊層數值容許任一倍數（3407、3449 行）。`status_stats` 接受每層值 × 1～max_stacks 的任一倍數。a14r_stack 一級每層 3、滿 5 層 15，原作的固定 +15 會以 k=5 通過；a14t_aspd_stack 同理。
3. `ability_swapped` 沒讀換回後的冷卻（941-985 行）。`swapped()` 只讀換入招，不讀換回的原招。帶 `initial_cooldown` 時連 policy 都不判，所以 `b24_earth_prison/*/swap/b24_shatter` pass 與 FC-088 的 `continue` 無關。`a13_shadow_illusion/*/swap` 判到 policy，但也沒讀 A13W 換回後的冷卻。
4. 狀態時長只判上限（2510 行）。`expires` 只檢查不超過文件時間與會結束，時長偏短也會 pass。影響 FC-060、FC-072。
5. normal 的 `damage` 只比第一擊（3195-3203 行）。多段招的註記只代表單擊數值，例如 FC-071 的 5.0。

**註解、文件過期**

6. `fc059_c11t_bolt_arcs_between_enemies` 的註解（4216-4218 行）還寫 FAILS TODAY、沒有列入 `check_fixed_quirk`。實際上 1066 行已列入，v3c_full661 也是 pass。
7. FC-105 的文件式判定過寬。`waves 5-8` 只要求文件裡有 `call b21t_update`，而 `baba_nobufusa/v3.lua:203` 的 on_take_damage 就滿足了。真正的修正在 228-230 行的 on_upgrade，文件裡有，但測試沒有鎖住。
8. FC-053 的文件式只查 aura 的 status 名稱；FC-060 只查文件裡有 `apply_status c11t_stun`，不查時長。
9. `fix-candidates.md` 的統計與建議段落過期：開頭寫 134 項、各組合計 131，表內已編到 FC-135；第九組仍寫 FC-115～FC-131；文末的「不修 7 項」與開頭「全部修」不同。逐項的 ✓ 欄是對的。

**文件修正與 FC 描述**

53 項逐條對照 quirk 行與 FC 列，沒查到修法對不上的項目。FC-040 看似不同（FC 寫每秒、v3 寫每次命中），但 `tick_interval` 是 1 秒，每秒命中一次，兩者一致。
