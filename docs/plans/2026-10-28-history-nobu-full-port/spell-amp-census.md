# 法術增幅普查：v3 damage 逐處對原作（2026-10-29）

工作線 A 為工作線 B 的規則層做的普查。規則在 v3.md §5.1：
- 原作的 KV `Damage` 與帶 ability 的 `ApplyDamage` 吃法術增幅，資料寫 `require_source_alive = false`；
- 沒帶 ability 的 `ApplyDamage`，以及攻擊者是 dummy 或召喚物的傷害，再加 `flags = { "no_amplify" }`；
- `AMHC:Damage`／`unit:Damage` 寫預設：檢查存活、不吃增幅。

規則層的判斷：`require_source_alive = false` 而且沒有 `no_amplify` 的傷害才帶 SPELL。

原作位置：`C:/nobu/history_nobu/game/dota_addons/nobu/scripts/`（commit f966720）。11 個子 agent 分批對過，主 session 彙整。

## 1. 數量

| 分類 | 原作 | 第 1 輪（原本寫 false 的 174 處） | 第 2 輪（原本寫預設的 307 處） |
|---|---|---|---|
| A | KV Damage | 63 | 53 |
| B | ApplyDamage 帶 ability | 37 | 34 |
| C | ApplyDamage 沒帶 ability | 23 | 31 |
| D | AMHC／unit:Damage | 0 | 179 |
| E | 其他（dummy、召喚物等） | 10 | 3 |
| - | 不是 damage action（註解、quirk 文字、重複列） | 30 | 3 |

- 第 1 輪：原本寫 false 的傷害裡沒有 D，也就是沒有把 AMHC 誤寫成 false 的。C 與 E 加上 `no_amplify`。
- 第 2 輪：原本寫預設的傷害裡，A、B 是漏掉的增幅，改成 false；C 改成 `false` 加上 `no_amplify`，存活檢查也跟原作一致。D 不動。
- 普查之後另外補了一處：C08W 段藏死後由信使造成的傷害，除了原有的 `no_source_bonuses`，也加上 `no_amplify`，規則層只需要看這一個旗標。

## 2. 改動清單（A、B、C、E）

每列的位置是普查當時影子樹的行號。

### 第 1 輪（只有 C、E 有改動；A、B 原本就正確）

| 文件 | 行 | 條目 | 分類 | 原作 | 處理 |
|---|---|---|---|---|---|
| items/batch_I1.lua | 624 | item_ice_storm_book | E | vscripts/items/Addon_Items/item_frozen_ring.lua:350-356 (storm_break) | add no_amplify（施法者存活走 AMHC:Damage，死亡改由 caster.dummy AMHC:Damage；都不吃增幅；保留 require_source_alive=false） |
| items/batch_I3.lua | 1255 | item_storm_book | E | vscripts/items/Addon_Items/item_storm_book.lua:100-111（存活時 AMHC:Damage(caster)，死亡後 AMHC:Damage(caster.dummy)） | add no_amplify（兩條路徑都是 AMHC、不帶 ability；保留 require_source_alive=false 表達 dummy 接手） |
| items/batch_I3.lua | 1267 | item_storm_book | E | vscripts/items/Addon_Items/item_storm_book.lua:103-112 | keep（quirk 文字；未提不吃增幅，建議日後補述） |
| items/batch_I5.lua | 522 | item_the_overflame_art_of_war | E | vscripts/items/Addon_Items/item_the_overflame_art_of_war.lua:107-111 | add no_amplify（施法者存活走 AMHC:Damage，死亡改由 dummy AMHC:Damage；兩者都不吃增幅；保留 require_source_alive=false 因 dummy 照樣打出傷害） |
| items/batch_I5.lua | 527 | item_the_overflame_art_of_war | E | vscripts/items/Addon_Items/item_the_overflame_art_of_war.lua:119-123 | add no_amplify（同上，建築一成分支） |
| items/batch_I5.lua | 540 | item_the_overflame_art_of_war | E | npc/npc_items_custom_load.txt:605-612 (modifier_the_overflame_art_of_war KV Damage，施加者是 dummy；lua:115) | 併入 flags no_amplify（KV Damage 帶 inflictor，但攻擊者是 dummy、無增幅；已有 no_source_bonuses） |
| packages/akechi_hidemitsu/v3.lua | 286 | c02_culling_slash | C | vscripts/heroes/C_Neutral/C02.lua:465 | add no_amplify |
| packages/akechi_hidemitsu/v3.lua | 305 | c02_culling_slash | C | vscripts/heroes/C_Neutral/C02.lua:397 | add no_amplify |
| packages/akechi_hidemitsu/v3.lua | 311 | c02_culling_slash | C | vscripts/heroes/C_Neutral/C02.lua:414 | add no_amplify |
| packages/asai_nagamasa/v3.lua | 153 | b08_flash_cut | C | vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua:162-169 | add no_amplify（燒魔 damageTable 沒帶 ability） |
| packages/hachisuka_masakatsu/v3.lua | 69 | a09_treant_summon | E | npc/A_Oda/A09.txt:135 | add no_amplify（KV Damage，但 modifier 由小樹鬼施加（A09.lua:55 ApplyDataDrivenModifier(attacker,...)），攻擊者是召喚物、無增幅） |
| packages/hashiba_hideyoshi/v3.lua | 409 | a28_spreading_fire | C | vscripts/heroes/A_Oda/A28_Hashiba_Hideyoshi.lua:495（SplitShotDamage 的 damage_table 沒有 ability） | add no_amplify |
| packages/honda_masanobu/v3.lua | 70 | a33_dark_summoning | C | vscripts/heroes/A_Oda/A33.lua:26-32 (damageTable 無 ability) | add no_amplify |
| packages/imagawa_yoshimoto/v3.lua | 202 | b15_multishot | C | vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:190-210 (damage_table 無 ability) | add no_amplify |
| packages/ishida_mitsunari/v3.lua | 355 | a29t_spirit | C | vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:330-373（damage_table 無 ability） | add no_amplify |
| packages/kashin_koji/v3.lua | 428 | c16r_spirit | C | vscripts/heroes/C_Neutral/C16.lua:414 | add no_amplify（damage_table 沒有 ability，attacker 是果心） |
| packages/kato_danzo/v3.lua | 136 | c08_blood_hunt | E | vscripts/heroes/C_Neutral/C08.lua:132-136（段藏死後 AMHC:Damage(caster.donkey,…)） | keep（攻擊者改為信使；已有 no_source_bonuses 涵蓋不吃增幅） |
| packages/maeno_nagayasu/v3.lua | 250 | a36_deadly_strike | C | vscripts/heroes/A_Oda/A36.lua:145 | add no_amplify |
| packages/matsunaga_hisahide/v3.lua | 335 | b28_serpent_fire | C | vscripts/heroes/B_Unified/B28.lua:280 | add no_amplify（彈跳傷害 ApplyDamage 沒帶 ability） |
| packages/miyoshi_nagayoshi/v3.lua | 335 | b10_power_of_the_abyss | C | vscripts/heroes/B_Unified/B10.lua:343-404 (ExorcismPhysics damage_table 無 ability) | add no_amplify（實際是 routine b10w_spirit_hit，屬 B10W 亡靈命中，sites.tsv 條目標成 B10T） |
| packages/mori_ranmaru/v3.lua | 299 | a31_samidare | C | vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:263 | add no_amplify |
| packages/naoe_kanetsugu/v3.lua | 75 | b36_naoe_seal | C | vscripts/heroes/B_Unified/B36.lua:34-40 (damageTable 無 ability) | add no_amplify |
| packages/naoe_kanetsugu/v3.lua | 248 | b36_seven_spears | C | vscripts/heroes/B_Unified/B36.lua:154-175 (damageTable 無 ability) | add no_amplify |
| packages/saito_dosan/v3.lua | 56 | a11_vortex | E | vscripts/heroes/A_Oda/A11.lua:39-53 (ApplyDamage 帶 ability，但道三死亡時 attacker 換成 dummy) | 併入 flags no_amplify（此分支只在道三死亡時，攻擊者是 dummy、無增幅；已有 no_source_bonuses） |
| packages/tachibana_muneshige/v3.lua | 29 | ? | C | vscripts/heroes/C_Neutral/C11.lua:153 | add no_amplify（共用 land 產生器） |
| packages/tachibana_muneshige/v3.lua | 290 | c11_thunder_eye | C | vscripts/heroes/C_Neutral/C11.lua:214 | add no_amplify |
| packages/tachibana_muneshige/v3.lua | 349 | c11_unrivaled | C | vscripts/heroes/C_Neutral/C11.lua:297 | add no_amplify |
| packages/takigawa_kazumasu/v3.lua | 67 | a32_flying_slash | C | vscripts/heroes/A_Oda/A32.lua:84 | add no_amplify |
| packages/takigawa_kazumasu/v3.lua | 135 | a32_second_slash | C | vscripts/heroes/A_Oda/A32.lua:165 | add no_amplify |
| packages/takigawa_kazumasu/v3.lua | 185 | a32_soul_drain | C | vscripts/heroes/A_Oda/A32.lua:227 | add no_amplify（計時器每跳另查 caster:IsAlive()，A32.lua:209） |
| packages/takigawa_kazumasu/v3.lua | 383 | a32_asura_blade | C | vscripts/heroes/A_Oda/A32.lua:329 | add no_amplify |
| packages/tamako/v3.lua | 153 | c15_burst_arrow | E | npc/C_Neutral/C15.txt:220-232（modifier_C15E_3 Damage，施加者是 npc_dummy_unit；C15.lua:30-32） | add no_amplify（KV Damage 但攻擊者是 dummy，無增幅） |
| packages/ukita_hideie/v3.lua | 246 | a23_falling_fire | C | vscripts/heroes/A_Oda/A23.lua:168-182 (damage_table 無 ability) | add no_amplify |

### 第 2 輪

| 文件 | 行 | 條目 | 分類 | 原作 | 處理 |
|---|---|---|---|---|---|
| items/batch_I0.lua | 458 | item_blowgun | A | npc/npc_items_custom_load.txt:15230-15235 (OnProjectileHitUnit Damage 50) | add require_source_alive=false |
| items/batch_I1.lua | 427 | item_great_dragon | A | npc/npc_items_custom_load.txt:937（OnProjectileHitUnit Damage） | 改 require_source_alive = false |
| items/batch_I1.lua | 433 | item_great_dragon | A | npc/npc_items_custom_load.txt:937（OnProjectileHitUnit Damage） | 改 require_source_alive = false |
| items/batch_I1.lua | 715 | item_ignite_book | A | npc/npc_items_custom_load.txt:14519（modifier_ignite_book OnIntervalThink Damage） | 改 require_source_alive = false |
| items/batch_I1.lua | 1013 | item_kisyosennfuyuzuki | A | npc/npc_items_custom_load.txt:9716（OnProjectileHitUnit Damage） | 改 require_source_alive = false |
| items/batch_I1.lua | 1020 | item_kisyosennfuyuzuki | A | npc/npc_items_custom_load.txt:9741（modifier_frost_bite_root_datadriven OnIntervalThink Damage） | 改 require_source_alive = false |
| items/batch_I2.lua | 307 | item_lightning_scroll | C | vscripts/items/Addon_Items/item_lightning_scroll.lua:63-65 | add require_source_alive=false + no_amplify（ApplyDamage 無 ability） |
| items/batch_I2.lua | 310 | item_lightning_scroll | C | vscripts/items/Addon_Items/item_lightning_scroll.lua:83-85 | add require_source_alive=false + no_amplify（ApplyDamage 無 ability） |
| items/batch_I2.lua | 415 | item_logs_big_tachi | A | npc/npc_items_custom_load.txt:4641 | add require_source_alive=false（modifier_spell_book_enemy OnCreated KV Damage，施加者是持有者） |
| items/batch_I2.lua | 781 | item_mana_thief | C | vscripts/items/Addon_Items/item_mana_thief.lua:70 | add require_source_alive=false + no_amplify（mana_burn_function ApplyDamage 無 ability） |
| items/batch_I2.lua | 792 | item_mana_thief | C | vscripts/items/Addon_Items/item_mana_thief.lua:218 | add require_source_alive=false + no_amplify（Shock ApplyDamage 無 ability） |
| items/batch_I3.lua | 385 | item_raikiri | C | vscripts/items/Addon_Items/item_lightning_scroll.lua:63,65 | add require_source_alive=false + no_amplify |
| items/batch_I3.lua | 389 | item_raikiri | C | vscripts/items/Addon_Items/item_lightning_scroll.lua:83,85 | add require_source_alive=false + no_amplify |
| items/batch_I3.lua | 1136 | item_spell_book | A | npc/npc_items_custom_load.txt:1807 | add require_source_alive=false |
| items/batch_I4.lua | 614 | item_the_club_of_nebula | A | npc/npc_items_custom_load.txt:7406（OnProjectileHitUnit KV Damage） | 改成 require_source_alive = false |
| items/batch_I4.lua | 1320 | item_the_great_sword_of_soul_hyper | A | npc/npc_items_custom_load.txt:15326（OnSpellStart KV Damage 1） | 改成 require_source_alive = false |
| items/batch_I4.lua | 1486 | item_the_great_sword_of_tiger | A | npc/npc_items_custom_load.txt:5363（OnSpellStart KV Damage 1） | 改成 require_source_alive = false |
| items/batch_I5.lua | 128 | item_the_great_sword_of_toxic | A | npc/npc_items_custom_load.txt:6342-6347（modifier_the_great_sword_of_toxic2 OnIntervalThink Damage） | add require_source_alive=false |
| items/batch_I5.lua | 138 | item_the_great_sword_of_toxic | A | npc/npc_items_custom_load.txt:6361-6366（modifier_the_great_sword_of_toxic4 OnIntervalThink Damage） | add require_source_alive=false |
| items/batch_I5.lua | 650 | item_the_red_lightning_chapter | C | vscripts/items/Addon_Items/item_the_red_lightning_chapter.lua:77 | add require_source_alive=false + no_amplify |
| items/batch_I5.lua | 652 | item_the_red_lightning_chapter | C | vscripts/items/Addon_Items/item_the_red_lightning_chapter.lua:93 | add require_source_alive=false + no_amplify |
| items/batch_I5.lua | 723 | item_the_red_lightning_chapter_hyper | C | vscripts/items/Addon_Items/item_the_red_lightning_chapter.lua:77 | add require_source_alive=false + no_amplify |
| items/batch_I5.lua | 725 | item_the_red_lightning_chapter_hyper | C | vscripts/items/Addon_Items/item_the_red_lightning_chapter.lua:93 | add require_source_alive=false + no_amplify |
| items/batch_I5.lua | 1043 | item_the_storm_call_book | E | vscripts/items/Addon_Items/item_the_over_lightning_art_of_war.lua:84-88 | add require_source_alive=false；flags 併為 { no_source_bonuses, no_amplify }（AMHC 攻擊者是 hide_unit dummy，活到雷暴結束，不看施放者存活；dummy 無增幅） |
| items/batch_I5.lua | 1147 | item_thunderstorms | C | vscripts/items/Addon_Items/item_thunderstorms.lua:56-61 | add require_source_alive=false + no_amplify |
| items/batch_I5.lua | 1155 | item_thunderstorms | C | vscripts/items/Addon_Items/item_thunderstorms.lua:96-101 | add require_source_alive=false + no_amplify |
| items/pilot_a.lua | 132 | item_the_dark_yellow_teeth_and_zimbabwe | C | vscripts/items/Addon_Items/item_ancient_doll.lua:49（damageTable 無 ability、REFLECTION） | 改成 flags = { "reflection", "no_amplify" }, require_source_alive = false（續行 133；原作另判 caster:IsAlive，v3 cond 已有 alive(self)） |
| items/pilot_a.lua | 628 | item_oochidorijuumonnjiyari | A | npc/npc_items_custom_load.txt:17687（OnProjectileHitUnit KV Damage 1） | 改成 require_source_alive = false |
| items/pilot_a.lua | 640 | item_oochidorijuumonnjiyari | A | npc/npc_items_custom_load.txt:17658（modifier_frost_bite_root_datadriven OnIntervalThink，施加者為持有英雄） | 改成 require_source_alive = false |
| packages/akiyama_nobutomo/v3.lua | 116 | b24_earth_stele | A | npc/B_Unified/B24.txt:68 | add require_source_alive=false（OnProjectileHitUnit KV Damage；投射物由石碑位置發出但 attacker 是英雄） |
| packages/akiyama_nobutomo/v3.lua | 201 | b24_taunt | A | npc/B_Unified/B24.txt:214 | add require_source_alive=false |
| packages/akiyama_nobutomo/v3.lua | 393 | b24_earth_prison | A | npc/B_Unified/B24.txt:404 | add require_source_alive=false |
| packages/baba_nobufusa/v3.lua | 51 | b21_sky_wave | B | vscripts/heroes/B_Unified/B21.lua:52-58 (ApplyDamage 帶 ability) | add require_source_alive=false |
| packages/baba_nobufusa/v3.lua | 157 | b21_rebel_armor | B | vscripts/heroes/B_Unified/B21.lua:134-141 (ApplyDamage 帶 ability、REFLECTION) | add require_source_alive=false（保留 flags reflection） |
| packages/fuma_kotaro/v3.lua | 208 | b02_blazing_path | A | npc/B_Unified/B02.txt:393 | add require_source_alive=false（modifier_B02E_3 KV Damage，施加者是英雄） |
| packages/hachisuka_masakatsu/v3.lua | 137 | a09_lignify | B | vscripts/heroes/A_Oda/A09.lua:140 | add require_source_alive=false（ApplyDamage 帶 ability） |
| packages/hachisuka_masakatsu/v3.lua | 196 | a09_forest_pulse | B | vscripts/heroes/A_Oda/A09.lua:193 | add require_source_alive=false（ApplyDamage 帶 ability） |
| packages/hattori_hanzo/v3.lua | 341 | a13_chain_sickle | B | vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:634-636,654-656（ApplyDamage ability=self:GetAbility()） | 改 require_source_alive = false |
| packages/hirate_masahide/v3.lua | 119 | a10_phase_shift | B | vscripts/heroes/A_Oda/A10.lua:104 | add require_source_alive=false |
| packages/hirate_masahide/v3.lua | 124 | a10_phase_shift | B | vscripts/heroes/A_Oda/A10.lua:84 | add require_source_alive=false |
| packages/hirate_masahide/v3.lua | 248 | a10_swap | B | vscripts/heroes/A_Oda/A10.lua:312 | add require_source_alive=false |
| packages/hirate_masahide/v3.lua | 308 | a10_shockwave | B | vscripts/heroes/A_Oda/A10.lua:404 | add require_source_alive=false（光環 modifier 的 caster 是英雄） |
| packages/honganji_kennyo/v3.lua | 36 | b25_exorcism_strike | C | vscripts/heroes/B_Unified/B25_Honganji_Kennyo.lua:47-53（mana_burn_function，damageTable 無 ability） | 改 require_source_alive = false, flags = { "no_amplify" } |
| packages/honganji_kennyo/v3.lua | 83 | b25_exorcism_sutra | A | npc/B_Unified/B25.txt:194（OnProjectileHitUnit Damage） | 改 require_source_alive = false |
| packages/honganji_kennyo/v3.lua | 140 | b25_finger_of_death | A | npc/B_Unified/B25.txt:403（OnProjectileHitUnit Damage） | 改 require_source_alive = false |
| packages/honganji_kennyo/v3.lua | 214 | b25_asura_hell | C | vscripts/heroes/B_Unified/B25_Honganji_Kennyo.lua:383-388,410（damageTable 無 ability） | 改 require_source_alive = false, flags = { "no_amplify" } |
| packages/ii_naomasa/v3.lua | 100 | a06_red_meteor | A | npc/A_Oda/A06.txt:370（OnProjectileHitUnit ActOnTargets Damage） | 改 require_source_alive = false |
| packages/ii_naomasa/v3.lua | 220 | a06_flame_of_anger | A | npc/A_Oda/A06.txt:85（OnProjectileHitUnit Damage） | 改 require_source_alive = false |
| packages/inaba_ittetsu/v3.lua | 44 | a18_obsessive_pressure | B | vscripts/heroes/A_Oda/A18.lua:86 | add require_source_alive=false（ApplyDamage 帶 ability） |
| packages/inaba_ittetsu/v3.lua | 258 | a18_by_force | B | vscripts/heroes/A_Oda/A18.lua:178 | add require_source_alive=false（ApplyDamage 帶 ability） |
| packages/ise_hime/v3.lua | 211 | b23_spirit_gathering | B | vscripts/heroes/B_Unified/B23.lua:204-211 | add require_source_alive=false |
| packages/ise_hime/v3.lua | 218 | b23_spirit_gathering | B | vscripts/heroes/B_Unified/B23.lua:221-228 | add require_source_alive=false |
| packages/ise_hime/v3.lua | 273 | b23_living_soul_shock | B | vscripts/heroes/B_Unified/B23.lua:282-289 | add require_source_alive=false |
| packages/ise_hime/v3.lua | 280 | b23_living_soul_shock | B | vscripts/heroes/B_Unified/B23.lua:282-289（亡魂投射物 Ability=ability，同一 OnProjectileHitUnit，attacker=caster） | add require_source_alive=false |
| packages/ishikawa_goemon/v3.lua | 177 | c06_lavish_strike | C | vscripts/heroes/C_Neutral/C06.lua:229,231（damageTable 無 ability） | 改成 require_source_alive = false, flags = { "no_amplify" }（續行 178） |
| packages/ishikawa_goemon/v3.lua | 408 | item_c06e | A | npc/npc_items_custom_load.txt:16782（modifier_item_c06e OnCreated KV Damage；施加者＝陷阱道具持有者） | 改成 require_source_alive = false |
| packages/kakizaki_kageie/v3.lua | 127 | b30_soul_breaker | A | npc/B_Unified/B30.txt:163-170 (modifier_B30E_debuff OnIntervalThink Damage) | add require_source_alive=false |
| packages/kakizaki_kageie/v3.lua | 187 | b30_flash_strike | B | vscripts/heroes/B_Unified/B30.lua:102-113 (damageTable 帶 ability) | add require_source_alive=false |
| packages/kakizaki_kageie/v3.lua | 243 | b30_purgatory_slash | A | npc/B_Unified/B30.txt:330-337 (modifier_B30T_debuff OnIntervalThink Damage) | add require_source_alive=false |
| packages/kakizaki_kageie/v3.lua | 262 | b30_purgatory_slash | A | npc/B_Unified/B30.txt:305-313 (OnProjectileHitUnit Damage) | add require_source_alive=false |
| packages/katakura_kagetsuna/v3.lua | 101 | b14_iai_slash | B | vscripts/heroes/B_Unified/B14.lua:35-46（ApplyDamage ability=ability） | 改 require_source_alive = false |
| packages/katakura_kagetsuna/v3.lua | 148 | b14_shadow_tactics | A | npc/B_Unified/B14.txt:172（modifier_B14E OnCreated Damage） | 改 require_source_alive = false |
| packages/kikuhime/v3.lua | 48 | b11_cherry_rain | B | vscripts/heroes/B_Unified/B11.lua:41-48 | add require_source_alive=false |
| packages/kikuhime/v3.lua | 50 | b11_cherry_rain | B | vscripts/heroes/B_Unified/B11.lua:51-58 | add require_source_alive=false |
| packages/kirigakure_saizo/v3.lua | 284 | b17_shadow_leap | B | vscripts/heroes/B_Unified/B17.lua:173 | 改成 require_source_alive = false（續行 285） |
| packages/kojima_sadaoki/v3.lua | 130 | b27_mighty_strike | A | npc/B_Unified/B27.txt:210 | add require_source_alive=false（modifier_B27E OnCreated KV Damage） |
| packages/kosokabe_chikayasu/v3.lua | 154 | c10_echo_sword | E | npc/C_Neutral/C10.txt:300-307 (引擎 datadriven CleaveAttack) | keep（引擎濺射不吃法術增幅；近戰、攻擊命中當下同步濺射，攻擊者必存活，預設語意等同原作） |
| packages/kuroda_yoshitaka/v3.lua | 55 | a19_fire_stratagem | A | npc/A_Oda/A19.txt:92 | add require_source_alive=false（modifier_A19W KV Damage） |
| packages/kuroda_yoshitaka/v3.lua | 67 | a19_fire_stratagem | A | npc/A_Oda/A19.txt:109 | add require_source_alive=false（modifier_A19W2 KV Damage） |
| packages/kuroda_yoshitaka/v3.lua | 81 | a19_fire_stratagem | B | vscripts/heroes/A_Oda/A19.lua:44 | add require_source_alive=false（ApplyDamage 帶 ability） |
| packages/kuroda_yoshitaka/v3.lua | 145 | a19_chain_fire | A | npc/A_Oda/A19.txt:237 | add require_source_alive=false（modifier_A19E KV Damage） |
| packages/kuroda_yoshitaka/v3.lua | 269 | a19_siege_fire | A | npc/A_Oda/A19.txt:391 | add require_source_alive=false（modifier_A19T_3 OnCreated KV Damage） |
| packages/kuroda_yoshitaka/v3.lua | 272 | a19_siege_fire | A | npc/A_Oda/A19.txt:400 | add require_source_alive=false（modifier_A19T_3 OnIntervalThink KV Damage） |
| packages/kuroda_yoshitaka/v3.lua | 280 | a19_siege_fire | A | npc/A_Oda/A19.txt:448 | add require_source_alive=false（modifier_A19T_5 KV Damage） |
| packages/maeda_toshiie/v3.lua | 85 | a14_impact_spear | C | vscripts/heroes/A_Oda/A14.lua:56,97 | add require_source_alive=false + no_amplify |
| packages/maeda_toshiie/v3.lua | 143 | a14_divine_strike | C | vscripts/heroes/A_Oda/A14.lua:127 | add require_source_alive=false + no_amplify |
| packages/matsunaga_hisahide/v3.lua | 53 | b28_phoenix_flame | A | npc/B_Unified/B28.txt:98-106 (modifier_B28W_dot OnIntervalThink Damage) | add require_source_alive=false |
| packages/matsunaga_hisahide/v3.lua | 65 | b28_phoenix_flame | A | npc/B_Unified/B28.txt:74-82 (OnProjectileHitUnit Damage) | add require_source_alive=false |
| packages/matsunaga_hisahide/v3.lua | 119 | b28_flame_shackle | A | npc/B_Unified/B28.txt:250-257 (modifier_B28E OnIntervalThink Damage) | add require_source_alive=false |
| packages/matsunaga_hisahide/v3.lua | 157 | b28_flame_shackle | A | npc/B_Unified/B28.txt:182-187 (OnSpellStart Damage) | add require_source_alive=false |
| packages/matsunaga_hisahide/v3.lua | 255 | b28_fire_stream | A | npc/B_Unified/B28.txt:352-359 (modifier_B28R_dot OnIntervalThink Damage) | add require_source_alive=false |
| packages/matsunaga_hisahide/v3.lua | 328 | b28_serpent_fire | C | vscripts/heroes/B_Unified/B28.lua:169 (ApplyDamage 無 ability) | add require_source_alive=false, flags no_amplify |
| packages/mori_motonari/v3.lua | 70 | b05_chain_lightning | C | vscripts/heroes/B_Unified/B05_Moari.lua:93,95（LightningJump，damageTable 無 ability） | 改成 require_source_alive = false, flags = { "no_amplify" }（改在續行 71） |
| packages/mori_motonari/v3.lua | 88 | b05_chain_lightning | C | vscripts/heroes/B_Unified/B05_Moari.lua:93,95（首目標同樣經 LightningJump） | 改成 require_source_alive = false, flags = { "no_amplify" }（改在續行 89） |
| packages/mori_motonari/v3.lua | 149 | b05_heavenly_thunder | C | vscripts/heroes/B_Unified/B05_Moari.lua:157 | 改成 require_source_alive = false, flags = { "no_amplify" } |
| packages/mori_motonari/v3.lua | 216 | b05_three_arrows | C | vscripts/heroes/B_Unified/B05_Moari.lua:190 | 改成 require_source_alive = false, flags = { "no_amplify" } |
| packages/mori_motonari/v3.lua | 293 | b05_holy_light_burst | C | vscripts/heroes/B_Unified/B05_Moari.lua:385 | 改成 require_source_alive = false, flags = { "no_amplify" } |
| packages/mori_motonari/v3.lua | 297 | b05_holy_light_burst | C | vscripts/heroes/B_Unified/B05_Moari.lua:390 | 改成 require_source_alive = false, flags = { "no_amplify" } |
| packages/okuma_tomohide/v3.lua | 55 | b26_charging_wave | A | npc/B_Unified/B26.txt:105（modifier_B26W OnIntervalThink ActOnTargets Damage） | 改 require_source_alive = false |
| packages/okuma_tomohide/v3.lua | 82 | b26_charging_wave | A | npc/B_Unified/B26.txt:149（modifier_B26W_2 OnCreated Damage） | 改 require_source_alive = false |
| packages/otani_yoshitsugu/v3.lua | 52 | a12_insight_flash | A | npc/A_Oda/A12.txt:57 | add require_source_alive=false |
| packages/otani_yoshitsugu/v3.lua | 247 | a12_awe | A | npc/A_Oda/A12.txt:505 | add require_source_alive=false |
| packages/saito_dosan/v3.lua | 51 | a11_vortex | B | vscripts/heroes/A_Oda/A11.lua:54（道三存活分支，ability = ability） | 改成 require_source_alive = false（續行 52） |
| packages/saito_dosan/v3.lua | 220 | a11_old_fox | B | vscripts/heroes/A_Oda/A11.lua:199 | 改成 require_source_alive = false |
| packages/saito_dosan/v3.lua | 298 | a11_myriad_forms | B | vscripts/heroes/A_Oda/A11.lua:243 | 改成 require_source_alive = false |
| packages/saito_tomonobu/v3.lua | 201 | b03_heavy_blow | C | vscripts/heroes/B_Unified/B03.lua:90-96 (damageTable 無 ability) | add require_source_alive=false, flags no_amplify |
| packages/saito_tomonobu/v3.lua | 291 | b03w_strike(routine) | B | vscripts/heroes/B_Unified/B03.lua:42-52 (damageTable 帶 ability；實際是 B03W 系列 routine，條目標成 B03T) | add require_source_alive=false |
| packages/sakai_tadatsugu/v3.lua | 71 | a05_valiant_power | B | vscripts/heroes/A_Oda/A05.lua:46 | add require_source_alive=false |
| packages/sakai_tadatsugu/v3.lua | 196 | a05_counter_stance | B | vscripts/heroes/A_Oda/A05.lua:131 | add require_source_alive=false |
| packages/sakakibara_yasumasa/v3.lua | 46 | a08_skull_crusher | A | npc/A_Oda/A08.txt:48-53（OnSpellStart Damage） | add require_source_alive=false |
| packages/sanada_masayuki/v3.lua | 292 | b09_mind_burst（routine b09e_pulse，sites 標 B09T 有誤） | B | vscripts/heroes/B_Unified/B09.lua:81-90 | add require_source_alive=false（寫在 293 行 amount 後） |
| packages/sassa_narimasa/v3.lua | 87 | a17_shrapnel | A | npc/A_Oda/A17.txt:122 | add require_source_alive=false（modifier 由英雄 ApplyDataDrivenModifier，attacker 是英雄；inflictor 是 dummy 身上的 A17W 複本） |
| packages/sengoku_hidehisa/v3.lua | 47 | a22_flame_whirl | A | npc/A_Oda/A22.txt:100 | add require_source_alive=false |
| packages/tachibana_dosetsu/v3.lua | 107 | c07_thunder_shackle | A | npc/C_Neutral/C07.txt:194 | add require_source_alive=false（modifier_C07E KV Damage） |
| packages/tachibana_dosetsu/v3.lua | 136 | c07_thunder_shackle | A | npc/C_Neutral/C07.txt:136 | add require_source_alive=false（OnSpellStart KV Damage） |
| packages/tachibana_dosetsu/v3.lua | 200 | c07_thunderbolt | A | npc/C_Neutral/C07.txt:300 | add require_source_alive=false（modifier_C07R OnCreated KV Damage） |
| packages/taigen_sessai/v3.lua | 394 | b22_wrath_of_nature | B | vscripts/heroes/B_Unified/B22.lua:147-154 (ApplyDamage 帶 ability，attacker = caster) | add require_source_alive=false |
| packages/takeda_shingen/v3.lua | 252 | b31_raging_fire | C | vscripts/heroes/B_Unified/B31.lua:200-206（damageTable 無 ability） | 改 require_source_alive = false, flags = { "no_amplify" } |
| packages/tokugawa_ieyasu/v3.lua | 200 | a01_thunder_blade | B | vscripts/heroes/A_Oda/A01.lua:214-223（A01R_OnAttackLanded，ability=ability） | 改 require_source_alive = false |
| packages/tokugawa_ieyasu/v3.lua | 215 | a01_thunder_blade | B | vscripts/heroes/A_Oda/A01.lua:175-180（A01R_OnSpellStart，ability=ability） | 改 require_source_alive = false |
| packages/torii_mototada/v3.lua | 179 | a16_blood_aura | C | vscripts/heroes/A_Oda/A16.lua:53 | add require_source_alive=false，flags 併為 { "reflection", "no_amplify" }（ApplyDamage 無 ability，DAMAGE_FLAG_REFLECTION） |
| packages/torii_mototada/v3.lua | 263 | a16_dark_blood_decree | A | npc/A_Oda/A16.txt:373 | add require_source_alive=false（modifier_A16T OnCreated KV Damage） |
| packages/uesugi_kenshin/v3.lua | 69 | b32_divine_slash | A | npc/B_Unified/B32.txt:292（OnProjectileHitUnit Damage） | 改 require_source_alive = false |
| packages/uesugi_kenshin/v3.lua | 205 | b32_divine_formation | C | vscripts/heroes/B_Unified/B32_Uesugi_Kenshin.lua:225-233（damageTable 無 ability，帶 REFLECTION） | 改 require_source_alive = false, flags = { "reflection", "no_amplify" } |
| packages/uesugi_kenshin/v3.lua | 279 | b32_war_god | A | npc/B_Unified/B32.txt:103（modifier_B32D OnIntervalThink Damage） | 改 require_source_alive = false |
| packages/usami_sadamitsu/v3.lua | 55 | b19_water_dragon | C | vscripts/heroes/B_Unified/B19.lua:113（LightningJump，由 modifier_arc_lightning_datadriven OnCreated 呼叫） | add require_source_alive=false + no_amplify |
| packages/usami_sadamitsu/v3.lua | 71 | b19_water_dragon | C | vscripts/heroes/B_Unified/B19.lua:113 | add require_source_alive=false + no_amplify |
| packages/usami_sadamitsu/v3.lua | 232 | b19_kuruma_gakari | E | vscripts/heroes/B_Unified/B19.lua:461（Passive_B19T 套在召喚的 B19T_old 狼身上、caster=狼，AMHC:Damage(狼, …)） | keep（攻擊者是召喚物本身＝source self；AMHC 檢查攻擊者存活、不吃增幅，預設正確） |
| packages/yamagata_masakage/v3.lua | 61 | b20_rock_rush | B | vscripts/heroes/B_Unified/B20.lua:56-65 | add require_source_alive=false |
| packages/yamagata_masakage/v3.lua | 120 | b20_ruin_wave | B | vscripts/heroes/B_Unified/B20.lua:92-101 | add require_source_alive=false |
| packages/yamagata_masakage/v3.lua | 247 | b20_red_charge（routine b20t_pulse） | B | vscripts/heroes/B_Unified/B20.lua:159-167 | add require_source_alive=false |

## 3. 原始表

逐處的分類（含 D 與 `-`）在 `target/m1v3/amp/result*.tsv`，不進版控。重做時照 v3.md §5.1 的規則逐處對原作。
