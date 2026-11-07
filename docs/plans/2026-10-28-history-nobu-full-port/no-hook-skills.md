# 不需要客製化 hook 的技能

產生自 `docs/character_pipeline/packages/*/v3.lua`（2026-11-07）。全部 867 招中，**864 招完全由 Lua package 描述、產生器產生 Rust**，只有 3 招用手寫 hook。

- 現行 445 招、舊版 368 招、2.0 51 招
- 用 hook 的 3 招（不在下表）：
  - 服部半藏（現行）`a13_chain_sickle` 忍法．勾鎖鐮 — hook:a13_chain_sickle_hook
  - 服部半藏（舊版）`a13_old_chain_sickle` 忍法．勾鎖鐮 — hook:a13_old_chain_sickle_hook
  - 石川五右衛門（舊版）`c06_old_murder_and_loot` 殺人越貨 — hook:c06e_old_murder_and_loot_hook

## 明智秀滿（現行）`akechi_hidemitsu`

- 蜃氣襲 `c02_mirage_strike`
- 幻移 `c02_phantom_move`
- 鏡花水月 `c02_kyoka_suigetsu`
- 汰選斬擊 `c02_culling_slash`

## 明智秀滿（舊版）`akechi_hidemitsu_old`

- 海波浩蕩 `c02_old_surging_waves`
- 吸魂 `c02_old_soul_drain`
- 鬼武者劍術 `c02_old_oni_swordsmanship`
- 鬼武者之力 `c02_old_oni_might`
- 炎龍劍 `c02_old_flame_dragon_sword`

## 明智光秀（現行）`akechi_mitsuhide`

- 血刃亂舞 `c01_blood_blade`
- 地焰召來 `c01_ground_flame`
- 血火共刃 `c01_blood_flame`
- 本能寺炎上 `c01_honnoji`

## 明智光秀（舊版）`akechi_mitsuhide_old`

- 血刃亂舞 `c01_old_blood_blade`
- 火雨術 `c01_old_fire_rain`
- 劍術 `c01_old_swordsmanship`
- 灌頂斬擊 `c01_old_anointing_slash`
- 本能寺燄上 `c01_old_honnoji`

## 秋山信友（現行）`akiyama_nobutomo`

- 地靈石碑 `b24_earth_stele`
- 嘲諷 `b24_taunt`
- 石之共鳴 `b24_stone_resonance`
- 大地禁錮 `b24_earth_prison`
- 擊碎石環 `b24_shatter`

## 淺井長政（現行）`asai_nagamasa`

- 一刀．牙 `b08_fang_slash`
- 一刀．閃瞬 `b08_flash_cut`
- 活心．魂 `b08_living_soul`
- 一刀．空牙 `b08_sky_fang`
- 一刀．連牙斷空 `b08_endless_fang`

## 淺井長政（舊版）`asai_nagamasa_old`

- 一刀．牙 `b08_old_fang_slash`
- 一刀．閃瞬 `b08_old_flash_cut`
- 活心．魂 `b08_old_living_soul`
- 一刀．空牙 `b08_old_sky_fang`

## 足利義昭（現行）`ashikaga_yoshiaki`

- 星瀑 `b07_starfall`
- 戰線國崩隊 `b07_battle_line`
- 再生光氣 `b07e_camp_regen`
- 輝煌之勢 `b07_glorious_momentum`
- 君臨天下 `b07_reign_over_all`

## 足利義昭（舊版）`ashikaga_yoshiaki_old`

- 加勢 `b07_old_soldier_camp`
- 再生光氣 `b07_old_camp_regen`
- 回復術 `b07_old_restoration`
- 流星雨 `b07_old_starfall`
- 君臨天下 `b07_old_reign_over_all`

## 馬場信房（現行）`baba_nobufusa`

- 破空波 `b21_sky_wave`
- 殺意 `b21_killing_intent`
- 反逆甲 `b21_rebel_armor`
- 不死鬼美濃 `b21_undying_demon`

## 馬場信房（舊版）`baba_nobufusa_old`

- 波動三連斬 `b21_old_triple_wave`
- 殺意 `b21_old_killing_intent`
- 領軍如神 `b21_old_divine_command`
- 不死鬼美濃 `b21_old_undying_demon`

## 茶茶（現行）`chacha`

- 突襲 `c25_raid`
- 血刃 `c25_blood_blade`
- 重創 `c25_grievous_wound`
- 落櫻散華 `c25_falling_sakura`

## 茶茶（舊版）`chacha_old`

- 迴旋裂斬 `c25_old_spin_slash`
- 短刃投擲 `c25_old_dagger_throw`
- 炫目 `c25_old_dazzle`
- 放血之術 `c25_old_bloodletting`
- 一刀．瞬 `c25_old_flash_slash`

## 長宗我部元親（現行）`chosokabe_motochika`

- 炎之加護 `c20_flame_aegis`
- 鬼血之流 `c20_demon_blood`
- 炎之舞 `c20_flame_dance`
- 鬼若子 `c20_demon_child`

## 長宗我部元親（舊版）`chosokabe_motochika_old`

- 炎之加護 `c20_old_flame_aegis`
- 鬼血之流 `c20_old_demon_blood`
- 炎之舞 `c20_old_flame_dance`
- 鬼若子 `c20_old_demon_child`

## 伊達政宗（現行）`date_masamune`

- 踏火無痕 `b04_flame_trail_dash`
- 火焰強襲 `b04_fire_blast`
- 炎之迅捷 `b04_flame_swiftness`
- 種子島 `b04_tanegashima`

## 伊達政宗（舊版）`date_masamune_old`

- 火焰刀 `b04_old_flame_blade`
- 踏火無痕 `b04_old_flame_trail`
- 火焰強襲 `b04_old_fire_blast`
- 靈活加成 `b04_old_agility_boost`
- 火繩銃 `b04_old_matchlock`

## 風魔小太郎（現行）`fuma_kotaro`

- 忍法．迷蹤步 `b02_vanishing_step`
- 忍法．朧 `b02_hazy_shadow`
- 忍法．熾烈之徑 `b02_blazing_path`
- 忍法．十八方手裏劍 `b02_eighteen_shuriken`
- 忍法密傳．四方導雷 `b02_four_thunders`

## 風魔小太郎（舊版）`fuma_kotaro_old`

- 忍法．迷蹤步 `b02_old_vanishing_step`
- 忍法．熾烈之徑 `b02_old_blazing_path`
- 忍法．十八方手裏劍 `b02_old_eighteen_shuriken`
- 忍法密傳．四方導雷 `b02_old_four_thunders`

## 蜂須賀政勝（現行）`hachisuka_masakatsu`

- 樹鬼召喚 `a09_treant_summon`
- 木化術 `a09_lignify`
- 林脈衝擊 `a09_forest_pulse`
- 深邃節觸 `a09_deep_tentacles`

## 蜂須賀政勝（舊版）`hachisuka_masakatsu_old`

- 食腐群 `a09_old_carrion_swarm`
- 木化術 `a09_old_lignify`
- 玄武之靈 `a09_old_black_tortoise`
- 生長藤蔓 `a09_old_growing_vines`

## 羽柴秀吉（現行）`hashiba_hideyoshi`

- 邪法 `a28_evil_art`
- 創念 `a28_creation`
- 縛足之沼 `a28_binding_swamp`
- 鳳凰召來 `a28_phoenix_summon`
- 天火 `a28_heaven_fire`
- 擴散之火 `a28_spreading_fire`

## 羽柴秀吉（舊版）`hashiba_hideyoshi_old`

- 邪法 `a28_old_evil_art`
- 治療之風 `a28_old_healing_wind`
- 火炎彈 `a28_old_fire_bomb`
- 火神‧迦具土 `a28_old_kagutsuchi`
- 靈魂燃燒 `a28_old_soul_burn`
- 熾烈焚身 `a28_old_blazing_body`

## 羽柴秀吉（2.0）`hashiba_hideyoshi_v20`

- 邪法 `a28_v20_evil_art`
- 創念 `a28_v20_creation`
- 縛足之沼 `a28_v20_binding_swamp`
- 鳳凰召來 `a28_v20_phoenix_summon`
- 鳳凰守護 `a28_v20_phoenix_bond`
- 天火 `a28_v20_heaven_fire`
- 熾烈焚身 `a28_v20_blazing_body`

## 服部半藏（現行）`hattori_hanzo`

- 忍法．迷蹤步 `a13_vanishing_step`
- 忍法．影惑之術 `a13_shadow_illusion`
- 忍法．換位之術 `a13_substitution`
- 伊賀忍術流 `a13_iga_ninjutsu`
- 忍法密傳．隱斬 `a13_shadow_slash`

## 服部半藏（舊版）`hattori_hanzo_old`

- 忍法．風行步 `a13_old_wind_step`
- 忍法．影分身之術 `a13_old_shadow_clone`
- 忍法密傳．隱斬 `a13_old_hidden_slash`

## 平手政秀（現行）`hirate_masahide`

- 瞬返之術 `a10_blink_return`
- 相移 `a10_phase_shift`
- 荒殺陣 `a10_slaughter_array`
- 移形換位 `a10_swap`
- 震盪 `a10_shockwave`

## 平手政秀（舊版）`hirate_masahide_old`

- 移型換位 `a10_old_swap`
- 飛石擊 `a10_old_flying_stones`
- 口臭 `a10_old_bad_breath`
- 聲東擊西 `a10_old_feint`

## 本多正信（現行）`honda_masanobu`

- 黑暗召喚 `a33_dark_summoning`
- 鬼軀之步 `a33_ghost_step`
- 腐化靈光 `a33_corrupting_aura`
- 傷害加深 `a33_deepen_wounds`

## 本多正信（舊版）`honda_masanobu_old`

- 妖火 `a33_old_demon_fire`
- 軀體腐爛 `a33_old_body_rot`
- 腐化靈光 `a33_old_corrupting_aura`
- 黑暗召喚 `a33_old_dark_summoning`

## 本多忠勝（現行）`honda_tadakatsu`

- 毫髮無傷 `a07_unscathed`
- 赤雷擊 `a07_red_thunder`
- 碎擊槍 `a07_shattering_spear`
- 戰國最強．啟動 `a07_strongest`
- 奮起 `a07_rouse`
- 奮起 `a07_rouse_free`

## 本多忠勝（舊版）`honda_tadakatsu_old`

- 毫髮無傷 `a07_old_unscathed`
- 赤雷擊 `a07_old_red_thunder`
- 碎擊槍 `a07_old_shattering_spear`
- 戰國最強．啟動 `a07_old_mightiest_awakening`

## 本多忠勝（2.0）`honda_tadakatsu_v20`

- 毫髮無傷 `a07_v20_unscathed`
- 赤雷擊 `a07_v20_red_thunder`
- 暴雷擊 `a07_v20_thunder_burst`
- 碎擊槍 `a07_v20_shattering_spear`
- 戰國最強．啟動 `a07_v20_mightiest_awakening`
- 雷擊切換 `a07_v20_switch_strike`

## 本願寺顯如（現行）`honganji_kennyo`

- 退魔擊 `b25_exorcism_strike`
- 除魔經 `b25_exorcism_sutra`
- 死之手指 `b25_finger_of_death`
- 阿修羅地獄 `b25_asura_hell`

## 本願寺顯如（舊版）`honganji_kennyo_old`

- 退魔擊 `b25_old_exorcism_strike`
- 除魔經 `b25_old_exorcism_sutra`
- 死之手指 `b25_old_finger_of_death`
- 阿修羅地獄 `b25_old_asura_hell`

## 井伊直政（現行）`ii_naomasa`

- 赤鬼之力 `a06_red_power`
- 赤流星 `a06_red_meteor`
- 赤焰流 `a06_red_flame_flow`
- 釋放怒焰 `a06_flame_of_anger`
- 赤炎之怒 `a06_red_fury`

## 井伊直政（舊版）`ii_naomasa_old`

- 赤鬼之力 `a06_old_red_demon_power`
- 赤炎斬 `a06_old_red_flame_slash`
- 戰魂 `a06_old_war_spirit`
- 赤鬼之怒 `a06_old_red_demon_fury`

## 今川義元（現行）`imagawa_yoshimoto`

- 東海巨人 `b15_tokai_giant`
- 烈焰箭 `b15_flame_arrow`
- 巨人之外皮 `b15_giant_hide`
- 多重箭 `b15_multishot`
- 虛張聲勢 `b15_bluff`

## 今川義元（舊版）`imagawa_yoshimoto_old`

- 巨人之臂力 `b15_old_giant_strength`
- 烈焰箭 `b15_old_flame_arrow`
- 巨人之外皮 `b15_old_giant_hide`
- 多重箭 `b15_old_multishot`

## 今川義元（2.0）`imagawa_yoshimoto_v20`

- 百步穿楊 `b15_v20_hundred_paces`
- 巨人之臂力 `b15_v20_giant_strength`
- 烈焰箭 `b15_v20_flame_arrow`
- 巨人之外皮 `b15_v20_giant_hide`
- 東海道一弓取 `b15_v20_tokaido_archer`

## 稻葉一鐵（現行）`inaba_ittetsu`

- 執念壓迫 `a18_obsessive_pressure`
- 打壓 `a18_suppress`
- 頑強戰法 `a18_stubborn_tactics`
- 無理矢理 `a18_by_force`

## 稻姬（現行）`inahime`

- 蠍毒箭 `a02_venom_arrow`
- 風之祈 `a02_wind_prayer`
- 遊風行身 `a02_wind_drift`
- 融影術 `a02_shadow_meld`

## 稻姬（舊版）`inahime_old`

- 蠍毒箭 `a02_old_venom_arrow`
- 風之力 `a02_old_wind_power`
- 自我恢復 `a02_old_self_recovery`
- 融影術 `a02_old_shadow_meld`

## 伊勢姬（現行）`ise_hime`

- 亡靈之契 `b23_undead_pact`
- 聚靈咒 `b23_spirit_gathering`
- 生靈衝擊 `b23_living_soul_shock`
- 魂饗 `b23_soul_feast`
- 英靈庇佑 `b23_heroic_spirit_blessing`

## 伊勢姬（舊版）`ise_hime_old`

- 亡靈之契 `b23_old_undead_pact`
- 聚靈咒 `b23_old_spirit_gathering`
- 生靈衝擊 `b23_old_living_soul_shock`
- 魂饗 `b23_old_soul_feast`
- 心靈控制 `b23_old_mind_control`

## 石田三成（現行）`ishida_mitsunari`

- 袖毒矢 `a29_sleeve_poison_arrow`
- 深淵之握 `a29_abyss_grip`
- 衰心靈光 `a29_withering_aura`
- 兀鷹群 `a29_vulture_swarm`

## 石田三成（舊版）`ishida_mitsunari_old`

- 袖毒矢 `a29_old_poison_arrow`
- 老樹盤根 `a29_old_entangling_roots`
- 機智消磨 `a29_old_wit_drain`
- 兀鷹群 `a29_old_vulture_swarm`

## 石川五右衛門（現行）`ishikawa_goemon`

- 大盜快手 `c06_quick_hands`
- 偷天換日 `c06_sleight_of_hand`
- 闊氣攻擊 `c06_lavish_strike`
- 殺人越貨 `c06_murder_and_loot`
- 豪奪 `c06_grand_heist`

## 石川五右衛門（舊版）`ishikawa_goemon_old`

- 狂氣 `c06_old_frenzy`
- 野蠻之力 `c06_old_savage_strength`
- 搶劫 `c06_old_robbery`
- 盜賊藏匿術 `c06_old_thief_hiding`

## 石川數正（現行）`ishikawa_kazumasa`

- 計步蠱 `a03_pacing_gu`
- 蜘蛛化型 `a03_spider_form`
- 蠱惑術 `a03_enchanting_venom`
- 蟲嗜群列 `a03_insect_swarm`

## 石川數正（舊版）`ishikawa_kazumasa_old`

- 蠱毒．寄生 `a03_old_parasite`
- 魔法護甲 `a03_old_magic_armor`
- 蠱惑術 `a03_old_enchanting_venom`
- 巨蠱‧小鹿 `a03_old_giant_gu`
- 小鹿動地跺 `a03_old_deer_stomp`

## 阿國（現行）`izumo_no_okuni`

- 神速之舞 `c04_swift_dance`
- 凋零舞步 `c04_withering_step`
- 聚曳之舞 `c04_gathering_dance`
- 巫魅之舞 `c04_witch_dance`

## 阿國（舊版）`izumo_no_okuni_old`

- 神速之舞 `c04_old_swift_dance`
- 凋零舞步 `c04_old_withering_step`
- 死亡之舞 `c04_old_dance_of_death`
- 巫魅之舞 `c04_old_witch_dance`

## 阿國（2.0）`izumo_no_okuni_v20`

- 神速之舞 `c04_v20_swift_dance`
- 凋零舞步 `c04_v20_withering_step`
- 聚曳之舞 `c04_v20_gathering_dance`
- 巫魅之舞 `c04_v20_witch_dance`
- 護身之舞 `c04_v20_guardian_dance`

## 柿崎景家（現行）`kakizaki_kageie`

- 逆壓之盾 `b30_reverse_shield`
- 碎魂劍 `b30_soul_breaker`
- 閃擊 `b30_flash_strike`
- 邪王煉獄斬 `b30_purgatory_slash`

## 柿崎景家（舊版）`kakizaki_kageie_old`

- 突擊 `b30_old_charge`
- 熾烈劍 `b30_old_blazing_sword`
- 煉獄之刃 `b30_old_purgatory_blade`
- 邪王煉獄斬 `b30_old_purgatory_slash`

## 上泉信綱（現行）`kamiizumi_nobutsuna`

- 劍之極意 `b35_sword_essence`
- 新陰流．一瞬 `b35_isshun`
- 新陰流‧無刀取 `b35_muto_dori`
- 新陰流‧秘轉 `b35_hiten`

## 上泉信綱（舊版）`kamiizumi_nobutsuna_old`

- 劍之極意 `b35_old_sword_essence`
- 新陰流．一閃 `b35_old_issen`
- 新陰流 `b35_old_shinkage_ryu`
- 絕‧陰流最終祕劍轉 `b35_old_final_secret_sword`

## 果心居士（現行）`kashin_koji`

- 咒術．斷魂 `c16_soul_sever`
- 咒術．咒影 `c16_curse_shadow`
- 咒術．行空魂 `c16_wandering_souls`
- 幻術．殘視 `c16_phantom_sight`
- 幻咒之術．奇門遁甲 `c16_eight_gates`

## 果心居士（舊版）`kashin_koji_old`

- 幻術．殘視 `c16_old_phantom_sight`
- 咒術．咒影 `c16_old_curse_shadow`
- 咒術．行空魂 `c16_old_wandering_souls`
- 亡魂不散 `c16_old_lingering_wraith`
- 幻咒之術．奇門遁甲 `c16_old_eight_gates`

## 片倉景綱（現行）`katakura_kagetsuna`

- 暫夜 `b14_temporary_night`
- 拔刀斬 `b14_iai_slash`
- 影武者戰法 `b14_shadow_tactics`
- 近侍心得 `b14_attendant_insight`
- 影切 `b14_shadow_cut`

## 片倉景綱（舊版）`katakura_kagetsuna_old`

- 拔刀斬 `b14_old_iai_slash`
- 近侍心得 `b14_old_attendant_insight`
- 刀術 `b14_old_swordsmanship`
- 影之斬 `b14_old_shadow_slash`
- 影武者 `b14_old_shadow_warrior`

## 加藤段藏（現行）`kato_danzo`

- 忍法．迷蹤步 `c08_vanishing_step`
- 忍法．血獵 `c08_blood_hunt`
- 忍法．亂擊脫離 `c08_scatter_escape`
- 忍法．飛鳶鎖 `c08_kite_chain`
- 忍法密傳．鬼吞之術 `c08_devour`

## 加藤段藏（舊版）`kato_danzo_old`

- 忍法．空禪之術 `c08_old_empty_zen`
- 忍法．雲隱之術 `c08_old_cloud_veil`
- 忍法．千鳥 `c08_old_chidori`
- 忍法．紅蓮之術 `c08_old_crimson_lotus`
- 飛鳶加藤 `c08_old_flying_kite`

## 加藤清正（現行）`kato_kiyomasa`

- 穢血狂暴 `c23_tainted_frenzy`
- 惡念障壁 `c23_malice_wall`
- 以血償血 `c23_blood_for_blood`
- 死冥之贈 `c23_gift_of_death`

## 菊姬（現行）`kikuhime`

- 忍法．櫻雨 `b11_cherry_rain`
- 忍法．調息 `b11_breath_control`
- 忍法．截殺術 `b11_interception`
- 鴉襲 `b11_crow_raid`
- 極限體術．鴉襲 `b11_crow_strike`
- 忍法．繼血 `b11_inherited_blood`

## 菊姬（舊版）`kikuhime_old`

- 忍法．櫻雨 `b11_old_cherry_rain`
- 忍法．土蜘蛛鎖 `b11_old_spider_bind`
- 忍法．調息 `b11_old_breath_control`
- 忍法．分身術 `b11_old_clone_art`
- 極限體術．鴉襲 `b11_old_crow_strike`
- 體術．真空斬 `b11_old_vacuum_slash`

## 霧隱才藏（現行）`kirigakure_saizo`

- 忍法．死亡守衛 `b17_death_ward`
- 忍法．牙毒 `b17_tooth_poison`
- 忍法．影躍 `b17_shadow_leap`
- 忍法密傳．覓心斬 `b17_heartseeker`

## 霧隱才藏（舊版）`kirigakure_saizo_old`

- 忍法．迷蹤步 `b17_old_phantom_step`
- 忍法．劇毒手裏劍 `b17_old_venom_shuriken`
- 忍法．迴旋手裏劍 `b17_old_spinning_shuriken`
- 忍法密傳．變化 `b17_old_transformation`

## 小早川隆景（現行）`kobayakawa_takakage`

- 謀計．單馬沓履 `c18_lone_rider`
- 謀計．雙車飛擊 `c18_twin_chariot`
- 謀計．參刀剜心 `c18_three_blades`
- 謀計．連環策動 `c18_chain_stratagem`
- 策謀之論．肆砲碾沙 `c18_four_cannons`

## 湖衣姬（現行）`koihime`

- 地襲 `b18_earth_raid`
- 迴響 `b18_echo`
- 大地庇護 `b18_earth_shelter`
- 大地魔爪 `b18_earth_claw`

## 湖衣姬（舊版）`koihime_old`

- 勝利的期許 `b18_old_victory_wish`
- 大地之刺 `b18_old_earth_spike`
- 大地束縛 `b18_old_earth_bind`
- 大地魔爪 `b18_old_earth_claw`

## 小島貞興（現行）`kojima_sadaoki`

- 騎術 `b27_horsemanship`
- 剛力擊 `b27_mighty_strike`
- 無畏衝鋒 `b27_fearless_charge`
- 裂地刃 `b27_rending_blade`

## 小島貞興（舊版）`kojima_sadaoki_old`

- 騎術 `b27_old_horsemanship`
- 剛力燄 `b27_old_mighty_strike`
- 法術護盾 `b27_old_spell_shield`
- 天崩地裂 `b27_old_earth_shatter`

## 香宗我部親泰（現行）`kosokabe_chikayasu`

- 黑龍波 `c10_black_dragon_wave`
- 能量奪取 `c10_energy_drain`
- 迴響劍 `c10_echo_sword`
- 晝夜轉換 `c10_day_night_shift`
- 無相天圓 `c10_formless_heaven`

## 香宗我部親泰（舊版）`kosokabe_chikayasu_old`

- 黑龍波 `c10_old_black_dragon_wave`
- 能量奪取 `c10_old_energy_drain`
- 無想轉生 `c10_old_mushin_rebirth`
- 昇龍亂舞 `c10_old_rising_dragon_dance`

## 九鬼嘉隆（現行）`kuki_yoshitaka`

- 海賊領域 `a20_pirate_domain`
- 水體爆破 `a20_water_burst`
- 海賊長靴 `a20_pirate_boots`
- 海賊大名．參上 `a20_pirate_daimyo`

## 九鬼嘉隆（舊版）`kuki_yoshitaka_old`

- 海噬 `a20_old_sea_devour`
- 海賊領域 `a20_old_pirate_domain`
- 海賊具足 `a20_old_pirate_armor`
- 船醉不知 `a20_old_seasick_frenzy`

## 黑田孝高（現行）`kuroda_yoshitaka`

- 鬼謀．火攻 `a19_fire_stratagem`
- 連環火 `a19_chain_fire`
- 火焰返響 `a19_fire_echo`
- 籠城之火 `a19_siege_fire`

## 黑田孝高（舊版）`kuroda_yoshitaka_old`

- 鬼謀．火攻 `a19_old_fire_stratagem`
- 連環火 `a19_old_chain_fire`
- 焰之附著 `a19_old_clinging_flame`
- 籠城之火 `a19_old_siege_fire`

## 黑田孝高（2.0）`kuroda_yoshitaka_v20`

- 鬼謀．火攻 `a19_v20_fire_stratagem`
- 連環火 `a19_v20_chain_fire`
- 火焰返響 `a19_v20_fire_echo`
- 籠城之火 `a19_v20_siege_fire`
- 焰之附著 `a19_v20_clinging_flame`

## 前田慶次（現行）`maeda_keiji`

- 松風 `c05_matsukaze`
- 雷舞 `c05_thunder_dance`
- 雷鳴豪槍 `c05_thunder_spear`
- 瞬雷閃 `c05_flash_thunder`

## 前田慶次（舊版）`maeda_keiji_old`

- 騎術 `c05_old_horsemanship`
- 雷舞 `c05_old_thunder_dance`
- 雷擊 `c05_old_thunder_strike`
- 雷鳴豪槍 `c05_old_thunder_spear`

## 前田利家（現行）`maeda_toshiie`

- 衝擊槍 `a14_impact_spear`
- 神現一擊 `a14_divine_strike`
- 疊擊槍術 `a14_stacked_spear`
- 狂怒．槍之又左 `a14_spear_fury`

## 前田利家（舊版）`maeda_toshiie_old`

- 本能之打擊 `a14_old_instinct_strike`
- 野生之斬擊 `a14_old_wild_slash`
- 戰魂 `a14_old_war_spirit`
- 火鳳凰 `a14_old_fire_phoenix`
- 狂怒．槍之又左 `a14_old_rage_spear`

## 前野長康（現行）`maeno_nagayasu`

- 潛影術 `a36_shadow_step`
- 地刺陷阱 `a36_spike_trap`
- 引爆 `a36_detonate`
- 苦楚秘釀 `a36_bitter_brew`
- 奪命攻擊 `a36_deadly_strike`

## 松姬（現行）`matsuhime`

- 媚誘目光 `c19_alluring_gaze`
- 婉約誓言 `c19_gentle_vow`
- 癱瘓毒 `c19_paralyzing_poison`
- 奇襲 `c19_surprise_raid`
- 不容侵犯 `c19_inviolable`

## 松姬（舊版）`matsuhime_old`

- 癱瘓毒 `c19_old_paralyzing_poison`
- 治療噴霧 `c19_old_healing_mist`
- 奇襲 `c19_old_surprise_raid`
- 不容侵犯 `c19_old_inviolable`

## 松永久秀（現行）`matsunaga_hisahide`

- 飛凰焰 `b28_phoenix_flame`
- 焰光桎梏 `b28_flame_shackle`
- 火流焰 `b28_fire_stream`
- 鬼謀．蛇火 `b28_serpent_fire`

## 松永久秀（舊版）`matsunaga_hisahide_old`

- 鬼謀．火攻 `b28_old_fire_attack`
- 鬼謀．綑綁 `b28_old_binding`
- 鬼謀．焰暴 `b28_old_flame_storm`
- 撕裂大地 `b28_old_earth_rend`

## 宮本武藏（現行）`miyamoto_musashi`

- 奧義．弧月斬 `c21_crescent_slash`
- 奧義‧連環鋼破 `c21_chain_breaker`
- 二天一流 `c21_two_heavens`
- 秘奧義‧飛翔烈震斬 `c21_soaring_quake`

## 宮本武藏（舊版）`miyamoto_musashi_old`

- 奧義．弧月斬 `c21_old_crescent_slash`
- 奧義．連環鋼破 `c21_old_chain_breaker`
- 二天一流 `c21_old_two_heavens`
- 秘奧義‧飛翔烈震斬 `c21_old_soaring_quake`

## 三好長慶（現行）`miyoshi_nagayoshi`

- 亡靈之怒 `b10_wrath_of_the_dead`
- 亡骸之襲 `b10_corpse_assault`
- 魔性戰魂 `b10_demonic_war_spirit`
- 深淵之力 `b10_power_of_the_abyss`

## 三好長慶（舊版）`miyoshi_nagayoshi_old`

- 疾行 `b10_old_swift_march`
- 飛斧 `b10_old_axe_throw`
- 戰魂 `b10_old_war_spirit`
- 魔性狂化 `b10_old_demonic_frenzy`

## 望月千代女（現行）`mochizuki_chiyome`

- 忍法．飛襲令 `b16_flying_raid`
- 忍法．落葉散華 `b16_falling_leaves`
- 忍法．迅雷 `b16_swift_thunder`
- 召喚忍法．召喚忍犬 `b16_summon_hound`
- 相移 `b16_phase_shift`
- 忍法密傳．獸靈同化 `b16_beast_union`
- 相移 `b16_mm_phase_shift`
- 掃擊 `b16_mm_sweep`
- 隱形 `b16_mm_stealth`

## 望月千代女（舊版）`mochizuki_chiyome_old`

- 忍法．召喚忍犬 `b16_old_summon_hound`
- 忍法．落葉散華 `b16_old_falling_leaves`
- 忍法．迅雷 `b16_old_swift_thunder`
- 忍法．煙霧彈 `b16_old_smoke_bomb`
- 一擊斬 `b16_old_mm_one_strike`
- 撕裂攻擊 `b16_old_mm_rending`
- 永久隱形 `b16_old_mm_stealth`

## 最上義姬（現行）`mogami_yoshihime`

- 腐蝕之箭 `b33_corrosive_arrow`
- 蛛網纏身 `b33_spider_web`
- 霧毒幻化 `b33_voodoo_power`
- 蜘蛛纏身 `b33_spider_attack`

## 最上義姬（舊版）`mogami_yoshihime_old`

- 腐蝕之箭 `b33_old_corrosive_arrow`
- 蛛網纏身 `b33_old_spider_web`
- 毒氣之箭 `b33_old_poison_arrow`
- 蜘蛛纏身 `b33_old_spider_attack`

## 百地三太夫（現行）`momochi_sandayu`

- 忍法．感官破壞 `b13_sense_break`
- 忍法．催眠 `b13_hypnosis`
- 忍法．血化裝 `b13_blood_disguise`
- 忍法．遁地術 `b13_burrow`
- 縮地 `b13_shukuchi`
- 忍法密傳．暴風 `b13_storm`

## 百地三太夫（舊版）`momochi_sandayu_old`

- 忍法．遁地 `b13_old_earth_hiding`
- 忍法．催眠 `b13_old_hypnosis`
- 忍具．隱雷 `b13_old_hidden_mine`
- 背刺 `b13_old_backstab`
- 忍法密傳．暴風 `b13_old_tempest`

## 毛利元就（現行）`mori_motonari`

- 鏈鎖閃電 `b05_chain_lightning`
- 天雷擊 `b05_heavenly_thunder`
- 三矢之訓 `b05_three_arrows`
- 氣霸西國 `b05_holy_light_burst`

## 毛利元就（舊版）`mori_motonari_old`

- 鏈鎖閃電 `b05_old_chain_lightning`
- 天雷擊 `b05_old_heavenly_thunder`
- 三矢之訓 `b05_old_three_arrows`
- 氣霸西國 `b05_old_spirit_dominion`

## 森蘭丸（現行）`mori_ranmaru`

- 響雨 `a31_echoing_rain`
- 瞬步 `a31_shunpo`
- 嵐矢 `a31_storm_arrow`
- 五月雨 `a31_samidare`

## 森蘭丸（舊版）`mori_ranmaru_old`

- 豪雷 `a31_old_thunder_rain`
- 瞬步 `a31_old_shunpo`
- 嵐矢 `a31_old_storm_arrow`
- 五月雨 `a31_old_samidare`

## 森蘭丸（2.0）`mori_ranmaru_v20`

- 響雨 `a31_v20_echoing_rain`
- 瞬步 `a31_v20_shunpo`
- 嵐矢 `a31_v20_storm_arrow`
- 五月雨 `a31_v20_samidare`
- 觀雲 `a31_v20_cloud_watch`

## 村上義清（現行）`murakami_yoshikiyo`

- 動地斧 `b29_ground_slam`
- 震地斧 `b29_quake_axe`
- 士氣提升 `b29_morale_boost`
- 大地震撼 `b29_earth_tremor`

## 村上義清（舊版）`murakami_yoshikiyo_old`

- 武王丸 `b29_old_takeoumaru`
- 動地斧 `b29_old_ground_slam`
- 震地斧 `b29_old_quake_axe`
- 士氣提升 `b29_old_morale_boost`
- 大地震撼 `b29_old_earth_tremor`

## 直江兼續（現行）`naoe_kanetsugu`

- 直江印 `b36_naoe_seal`
- 履及 `b36_footsteps`
- 劃心為籬 `b36_heart_fence`
- 技擊．七柱槍 `b36_seven_spears`
- 愛染明王 `b36_aizen_myoo`

## 直江兼續（舊版）`naoe_kanetsugu_old`

- 符咒‧金縛 `b36_old_binding_seal`
- 符咒‧護符障壁 `b36_old_charm_barrier`
- 符咒‧咒之符 `b36_old_curse_charm`
- 符咒攻擊 `b36_old_far_reach`
- 禁咒‧殺之陣 `b36_old_kill_formation`

## 寧寧（現行）`nene`

- 忍法．迷蹤步 `a15_shadow_step`
- 忍法．竭印 `a15_exhaust_seal`
- 忍法．月擊 `a15_moon_strike`
- 忍法．影風車 `a15_shadow_windmill`
- 忍法密傳．影月攻勢 `a15_shadow_moon_assault`

## 寧寧（舊版）`nene_old`

- 忍法．迷蹤步 `a15_old_vanishing_step`
- 忍法．觀月 `a15_old_moon_watch`
- 忍法．影風車 `a15_old_shadow_windmill`
- 忍法密傳．夢魘 `a15_old_nightmare`

## 丹羽長秀（現行）`niwa_nagahide`

- 狼狩之印 `a30_wolf_mark`
- 地之牙 `a30_earth_fang`
- 火海術 `a30_fire_sea`
- 紅富士 `a30_red_fuji`

## 丹羽長秀（舊版）`niwa_nagahide_old`

- 喚狼魂 `a30_old_wolf_call`
- 地之牙 `a30_old_earth_fang`
- 火海術 `a30_old_fire_sea`
- 紅富士 `a30_old_red_fuji`

## 濃姬（現行）`nohime`

- 連銃 `a26_volley`
- 火藥踢擊 `a26_powder_kick`
- 二丁銃 `a26_twin_guns`
- 灼熱爆裂投擲 `a26_scorching_bomb`

## 濃姬（舊版）`nohime_old`

- 四連彈 `a26_old_four_shot`
- 三味火 `a26_old_shamisen_fire`
- 二丁銃 `a26_old_twin_guns`
- 一擊必殺 `a26_old_one_shot`

## 濃姬（2.0）`nohime_v20`

- 連銃 `a26_v20_volley`
- 槍林彈雨 `a26_v20_bullet_storm`
- 二丁銃 `a26_v20_twin_guns`
- 灼熱爆裂投擲 `a26_v20_scorching_throw`
- 隱雷 `a26_v20_hidden_mine`

## 織田信長（現行）`oda_nobunaga`

- 導雷 `a25_lightning_lure`
- 厄災之棘 `a25_calamity_thorns`
- 壓切 `a25_heshikiri`
- 深紅障壁 `a25_crimson_barrier`
- 天下布武 `a25_tenka_fubu`

## 織田信長（舊版）`oda_nobunaga_old`

- 遠雷遙 `a25_old_distant_thunder`
- 厄災之棘 `a25_old_calamity_thorns`
- 壓切 `a25_old_heshikiri`
- 障壁深紅 `a25_old_crimson_barrier`

## 阿市（現行）`oichi`

- 光鍊罰則 `c17_light_chain`
- 祈勝光輝 `c17_victory_blessing`
- 神助一擊 `c17_divine_strike`
- 極光守護 `c17_aurora_guard`
- 自我奉獻 `c17_devotion`

## 阿市（舊版）`oichi_old`

- 光之精華 `c17_old_light_essence`
- 魂之精華 `c17_old_soul_essence`
- 靜滯陷阱 `c17_old_stasis_trap`
- 極光守護 `c17_old_aurora_guard`

## 大熊朝秀（現行）`okuma_tomohide`

- 蓄力波 `b26_charging_wave`
- 血性狂怒 `b26_blood_fury`
- 上野武勇 `b26_kozuke_valor`
- 當者披靡 `b26_unstoppable`
- 釋放蓄力波 `b26_release_wave`

## 阿松（現行）`omatsu`

- 寒霜彈 `a34_frost_bolt`
- 冰山之襲 `a34_iceberg_strike`
- 冰霜之息 `a34_frost_breath`
- 急速冷卻 `a34_rapid_cooling`

## 阿松（舊版）`omatsu_old`

- 寒霜彈 `a34_old_frost_bolt`
- 冰山襲擊 `a34_old_iceberg_strike`
- 冰霜之息 `a34_old_frost_breath`
- 雪女的祝福 `a34_old_snow_blessing`

## 阿松（2.0）`omatsu_v20`

- 寒霜彈 `a34_v20_frost_bolt`
- 冰山之襲 `a34_v20_iceberg_strike`
- 冰霜之息 `a34_v20_frost_breath`
- 雪女的祝福 `a34_v20_snow_blessing`
- 加賀百萬石 `a34_v20_kaga_million_koku`

## 大谷吉繼（現行）`otani_yoshitsugu`

- 智芒乍現 `a12_insight_flash`
- 奇計 `a12_scheme`
- 玄能通貫 `a12_mystic_pierce`
- 為智畏服 `a12_awe`
- 玄滅箭 `a12_mystic_arrow`
- 玄能附體 `a12_mystic_possession`
- 智芒乍現．遠 `a12_insight_flash_t`
- 玄能力場 `a12_mystic_field`
- 為智畏服．遠 `a12_awe_t`

## 大友宗麟（現行）`otomo_sorin`

- 定位導彈 `c12_homing_lock`
- 導彈發射！ `c12_missile_launch`
- 空間撕裂裝置 `c12_space_rift`
- 光學遮罩 `c12_optical_mask`
- 殲敵光線 `c12_annihilation_beam`

## 龍造寺隆信（現行）`ryuzoji_takanobu`

- 虹吸 `c13_siphon`
- 龍捲波 `c13_tornado_wave`
- 狂嵐之握 `c13_gale_grip`
- 龍捲之漩渦 `c13_tornado_vortex`

## 龍造寺隆信（舊版）`ryuzoji_takanobu_old`

- 風之鎖 `c13_old_wind_chain`
- 血腥狂風 `c13_old_bloody_gale`
- 龍捲波 `c13_old_tornado_wave`
- 力量強化 `c13_old_strength_up`
- 龍捲之漩渦 `c13_old_tornado_vortex`

## 雜賀孫市（現行）`saika_magoichi`

- 狙擊模式 `b01_sniper_mode`
- 雜賀眾 `b01_saika_corps`
- 雨鐵炮 `b01_rain_iron_cannon`
- 三段 `b01_three_stage`

## 雜賀孫市（舊版）`saika_magoichi_old`

- 狙擊模式 `b01_old_sniper_mode`
- 雜賀眾 `b01_old_saika_corps`
- 雨鐵炮 `b01_old_rain_iron_cannon`
- 三段 `b01_old_three_stage`

## 齋藤道三（現行）`saito_dosan`

- 妖術．漩渦 `a11_vortex`
- 妖術．陰邪凝視 `a11_evil_gaze`
- 老奸巨猾 `a11_old_fox`
- 禁術．邪妖萬化 `a11_myriad_forms`

## 齋藤道三（舊版）`saito_dosan_old`

- 妖術．漩渦 `a11_old_vortex`
- 妖術．冥鎖 `a11_old_viper_chain`
- 老奸巨猾 `a11_old_cunning`
- 禁術．妖風 `a11_old_evil_wind`

## 齋藤朝信（現行）`saito_tomonobu`

- 鍾馗一擊 `b03_zhongkui_strike`
- 鍾馗一擊（中） `b03_zhongkui_strike_mid`
- 鍾馗一擊（遠） `b03_zhongkui_strike_far`
- 越後金剛 `b03_echigo_vajra`
- 沉重打擊 `b03_heavy_blow`
- 鍾馗之審判 `b03_zhongkui_judgment`

## 齋藤朝信（舊版）`saito_tomonobu_old`

- 鍾馗之力 `b03_old_zhongkui_might`
- 越後金鋼 `b03_old_echigo_vajra`
- 怪力擊 `b03_old_mighty_blow`
- 鍾馗之審判 `b03_old_zhongkui_judgment`

## 齋藤義龍（現行）`saito_yoshitatsu`

- 火靈詛咒 `c14_fire_curse`
- 火龍吐息術 `c14_fire_breath`
- 火龍吐息術 `c14_fire_breath_dragon`
- 龍脈強韌 `c14_dragon_vein`
- 火龍化身 `c14_dragon_form`

## 齋藤義龍（舊版）`saito_yoshitatsu_old`

- 火靈詛咒 `c14_old_fire_curse`
- 火龍吐息術 `c14_old_fire_breath`
- 龍脈強韌 `c14_old_dragon_vein`
- 火龍化身 `c14_old_dragon_form`

## 酒井忠次（現行）`sakai_tadatsugu`

- 強悍之力 `a05_valiant_power`
- 三河之勇 `a05_mikawa_courage`
- 反擊態勢 `a05_counter_stance`
- 忠之極意 `a05_loyal_secret`
- 誓伐狀 `a05_crusade_affidavit`

## 酒井忠次（舊版）`sakai_tadatsugu_old`

- 三河之勇 `a05_old_mikawa_valor`
- 武者之道 `a05_old_warrior_way`
- 怪力擊 `a05_old_mighty_blow`
- 忠之極意 `a05_old_loyal_secret`

## 榊原康政（現行）`sakakibara_yasumasa`

- 碎顱打擊 `a08_skull_crusher`
- 檄文怒嗥 `a08_war_cry`
- 不屈武勇 `a08_indomitable`
- 剛毅大將 `a08_steadfast_general`

## 榊原康政（舊版）`sakakibara_yasumasa_old`

- 狂熱 `a08_old_frenzy`
- 狂嗥 `a08_old_roar`
- 力量強化 `a08_old_strength`
- 剛毅大將 `a08_old_steadfast_general`

## 佐久間盛政（現行）`sakuma_morimasa`

- 血盾集陣 `a24_blood_shield`
- 屠戮彈 `a24_slaughter_shot`
- 血之突進 `a24_blood_charge`
- 血之共鳴 `a24_blood_resonance`
- 浴血侵攻 `a24_bloodbath`

## 真田昌幸（現行）`sanada_masayuki`

- 反噬裂隙 `b09_backlash_rift`
- 意念爆發 `b09_mind_burst`
- 實念之盾 `b09_will_shield`
- 心靈壓制 `b09_mind_suppression`

## 真田昌幸（舊版）`sanada_masayuki_old`

- 落雷 `b09_old_thunder_strike`
- 地裂 `b09_old_earth_rupture`
- 衰弱 `b09_old_enfeeble`
- 旋風 `b09_old_whirlwind`
- 暴風 `b09_old_gale`

## 真田幸村（現行）`sanada_yukimura`

- 真田勇士 `b06_brave_warriors`
- 連牙炎突 `b06_flame_fangs`
- 心眼 `b06_mind_eye`
- 日本第一武士 `b06_first_warrior`

## 真田幸村（舊版）`sanada_yukimura_old`

- 真田勇士 `b06_old_brave_warriors`
- 炎突 `b06_old_flame_fang`
- 心眼 `b06_old_mind_eye`
- 日本第一武士 `b06_old_first_warrior`

## 佐佐木小次郎（現行）`sasaki_kojiro`

- 嚴流 `c22_ganryu`
- 祕劍．燕巢 `c22_swallow_nest`
- 祕劍．飛燕 `c22_flying_swallow`
- 祕劍．燕閃 `c22_swallow_flash`
- 奧義．滅剎夢霞 `c22_dream_mist`

## 佐佐木小次郎（舊版）`sasaki_kojiro_old`

- 秘劍．燕 `c22_old_swallow`
- 秘劍．返 `c22_old_return`
- 巖流 `c22_old_ganryu`
- 隱秘劍．地燕斬 `c22_old_earth_swallow`

## 佐佐成政（現行）`sassa_narimasa`

- 爆烈彈 `a17_shrapnel`
- 疾射 `a17_rapid_fire`
- 要害射擊 `a17_vital_shot`
- 大筒 `a17_great_cannon`

## 佐佐成政（舊版）`sassa_narimasa_old`

- 硫磺砲 `a17_old_sulfur_shot`
- 連擊砲 `a17_old_rapid_cannon`
- 雨鐵砲 `a17_old_rain_guns`
- 大筒 `a17_old_great_cannon`

## 仙石秀久（現行）`sengoku_hidehisa`

- 旋焰斬 `a22_flame_whirl`
- 焰心鼓動 `a22_flame_heart`
- 沸血靈光 `a22_boiling_blood`
- 折磨 `a22_torment`

## 仙石秀久（舊版）`sengoku_hidehisa_old`

- 嘲弄 `a22_old_taunt`
- 虎振 `a22_old_tiger_swing`
- 反擊 `a22_old_counterattack`
- 刀術 `a22_old_swordsmanship`
- 氣旋斬 `a22_old_cyclone_slash`

## 柴田勝家（現行）`shibata_katsuie`

- 破竹之勢 `a27_unstoppable_advance`
- 柴田家戰旗 `a27_war_banner`
- 武者之道 `a27_way_of_warrior`
- 暴虐之鎚 `a27_tyrant_hammer`
- 鬼柴田 `a27_oni_shibata`

## 柴田勝家（舊版）`shibata_katsuie_old`

- 鬼柴田 `a27_old_oni_shibata`
- 動地鎚 `a27_old_earth_hammer`
- 武者之道 `a27_old_way_of_warrior`
- 破竹之勢 `a27_old_split_bamboo`

## 島左近（現行）`shima_sakon`

- 焰襲 `a35_flame_raid`
- 熾焰封魔 `a35_sealing_blaze`
- 炎之迅捷 `a35_flame_swiftness`
- 焦灼之火 `a35_scorching_fire`

## 島左近（舊版）`shima_sakon_old`

- 炙炎突擊 `a35_old_blazing_charge`
- 炙焰護衛 `a35_old_flame_guardians`
- 靈活強化 `a35_old_agility_boost`
- 炙焰烈火 `a35_old_scorching_blaze`

## 島津義弘（現行）`shimazu_yoshihiro`

- 鐵布衫 `c03_iron_shirt`
- 震地斧 `c03_quake_axe`
- 暴雷怒擊 `c03_thunder_wrath`
- 不動戰錘 `c03_iron_hammer`

## 島津義弘（舊版）`shimazu_yoshihiro_old`

- 鐵布衫 `c03_old_iron_shirt`
- 震地斧 `c03_old_quake_axe`
- 暴雷怒擊 `c03_old_thunder_wrath`
- 不動戰錘 `c03_old_iron_hammer`

## 立花道雪（現行）`tachibana_dosetsu`

- 天雷降 `c07_heaven_thunder`
- 雷電桎梏 `c07_thunder_shackle`
- 雷霆閃電 `c07_thunderbolt`
- 雷神之怒 `c07_raijin_wrath`
- 移動雷雲 `c07_storm_move`
- 禦魔之盾 `c07_magic_shield`

## 立花道雪（舊版）`tachibana_dosetsu_old`

- 禦魔之盾 `c07_old_magic_shield`
- 雷電桎梏 `c07_old_thunder_shackle`
- 雷霆閃電 `c07_old_thunderbolt`
- 雷神之怒 `c07_old_raijin_wrath`

## 立花誾千代（現行）`tachibana_ginchiyo`

- 霜刺 `c09_frost_thorn`
- 白熱 `c09_white_heat`
- 冰靈舞魂 `c09_ice_soul_dance`
- 破甲術 `c09_armor_break`

## 立花誾千代（舊版）`tachibana_ginchiyo_old`

- 冰霜斬擊 `c09_old_frost_slash`
- 白熱 `c09_old_white_heat`
- 冰靈舞魂 `c09_old_ice_soul_dance`
- 破甲術 `c09_old_armor_break`

## 立花宗茂（現行）`tachibana_muneshige`

- 雷切 `c11_raikiri`
- 雷之疾 `c11_thunder_speed`
- 雷之眼 `c11_thunder_eye`
- 西國無雙 `c11_unrivaled`

## 立花宗茂（舊版）`tachibana_muneshige_old`

- 雷切 `c11_old_raikiri`
- 雷之疾 `c11_old_thunder_haste`
- 雷之眼 `c11_old_thunder_eye`
- 西國無雙 `c11_old_unrivaled`

## 立花宗茂（2.0）`tachibana_muneshige_v20`

- 雷光電閃 `c11_v20_lightning_flash`
- 疾雷伏牙 `c11_v20_thunder_fang`
- 雷之眼 `c11_v20_thunder_eye`

## 太原雪齋（現行）`taigen_sessai`

- 自然屏障 `b22_natural_barrier`
- 自然之力 `b22_force_of_nature`
- 淨化 `b22_purify`
- 木靈展勢 `b22_wood_spirit`
- 自然之怒 `b22_wrath_of_nature`

## 太原雪齋（舊版）`taigen_sessai_old`

- 自然屏障 `b22_old_natural_barrier`
- 自然之力 `b22_old_force_of_nature`
- 淨化 `b22_old_purify`
- 自然之怒 `b22_old_wrath_of_nature`

## 武田勝賴（現行）`takeda_katsuyori`

- 怒擊斧 `b34_wrath_axe`
- 刺針之牆 `b34_spike_wall`
- 狂斬 `b34_frenzy`
- 天目長嚎 `b34_tenmoku_howl`

## 武田勝賴（舊版）`takeda_katsuyori_old`

- 怒擊斧 `b34_old_wrath_axe`
- 狂戰 `b34_old_berserk`
- 撕裂傷口 `b34_old_rending_wound`
- 大地震撼 `b34_old_great_quake`

## 武田信玄（現行）`takeda_shingen`

- 疾如風 `b31_swift_wind`
- 徐如林 `b31_still_forest`
- 侵略如火 `b31_raging_fire`
- 不動如山 `b31_immovable_mountain`

## 武田信玄（舊版）`takeda_shingen_old`

- 疾如風 `b31_old_swift_as_wind`
- 徐如林 `b31_old_quiet_as_forest`
- 侵略如火 `b31_old_fierce_as_fire`
- 不動如山 `b31_old_immovable_as_mountain`

## 竹中重治（現行）`takenaka_shigeharu`

- 式神．水 `a04_water_shikigami`
- 凍結術 `a04_ice_path`
- 霜之庇護 `a04_frost_shelter`
- 霜雀 `a04_frost_sparrow`
- 吹雪 `a04_blizzard`

## 竹中重治（舊版）`takenaka_shigeharu_old`

- 式神．水 `a04_old_water_shikigami`
- 凍結術 `a04_old_freezing_path`
- 冰霜攻擊 `a04_old_frost_attack`
- 冰雪爆 `a04_old_ice_blast`
- 孤高的軍師 `a04_old_lone_strategist`
- 吹雪 `a04_old_blizzard`

## 瀧川一益（現行）`takigawa_kazumasu`

- 忍法．怨靈飛翔斬 `a32_flying_slash`
- 忍法．怨靈飛翔斬（二段） `a32_second_slash`
- 忍法．魂 `a32_soul_drain`
- 忍術．幽魄刀 `a32_phantom_blade`
- 忍術．斬魂刀 `a32_soul_blade`
- 忍法密傳．修羅吞世刀 `a32_asura_blade`

## 瀧川一益（舊版）`takigawa_kazumasu_old`

- 忍法．怨靈飛翔斬 `a32_old_flying_slash`
- 忍法．放逐 `a32_old_banish`
- 忍術．幽魄刀 `a32_old_phantom_blade`
- 忍術．斬魂刀 `a32_old_soul_blade`

## 玉子（現行）`tamako`

- 亂射 `c15_scatter_shot`
- 爆裂箭 `c15_burst_arrow`
- 精準之光 `c15_precise_light`
- 百箭齊發 `c15_hundred_arrows`

## 玉子（舊版）`tamako_old`

- 三連矢 `c15_old_triple_arrow`
- 烈焰箭 `c15_old_searing_arrow`
- 精準之光 `c15_old_precise_light`
- 百箭齊發 `c15_old_hundred_arrows`

## 德川家康（現行）`tokugawa_ieyasu`

- 天下之勢 `a01_realm_momentum`
- 東照權現 `a01_tosho_gongen`
- 天羅地網 `a01_heaven_net`
- 雷霆刃 `a01_thunder_blade`
- 三河之勇 `a01_mikawa_valor`

## 德川家康（舊版）`tokugawa_ieyasu_old`

- 東照權現 `a01_old_tosho_gongen`
- 天羅地網 `a01_old_heaven_net`
- 雷霆刃 `a01_old_thunder_blade`
- 三河之勇 `a01_old_mikawa_valor`

## 鳥居元忠（現行）`torii_mototada`

- 血腥之攫 `a16_bloody_grasp`
- 血魄連結 `a16_blood_link`
- 血殤靈光 `a16_blood_aura`
- 闇血敕令 `a16_dark_blood_decree`

## 上杉謙信（現行）`uesugi_kenshin`

- 神斬 `b32_divine_slash`
- 神閃 `b32_divine_flash`
- 神陣 `b32_divine_formation`
- 軍神．越後之龍 `b32_war_god`
- 毘沙門天 `b32_bishamonten`

## 上杉謙信（舊版）`uesugi_kenshin_old`

- 靈氣．軍神 `b32_old_war_god_aura`
- 神斬 `b32_old_divine_slash`
- 神閃 `b32_old_divine_flash`
- 神烈 `b32_old_divine_fury`
- 毘沙門天 `b32_old_bishamonten`

## 上杉謙信（2.0）`uesugi_kenshin_v20`

- 神斬 `b32_v20_divine_slash`
- 神閃 `b32_v20_divine_flash`
- 神陣 `b32_v20_divine_formation`
- 軍神．越後之龍 `b32_v20_war_god`
- 毘沙門天 `b32_v20_bishamonten`

## 宇喜多秀家（現行）`ukita_hideie`

- 鬼術．惡血咒 `a23_cursed_blood`
- 鬼術．靈魂鎖縛 `a23_soul_bind`
- 鬼術．墜火 `a23_falling_fire`
- 禁術．黃泉之焰 `a23_yomi_flame`

## 宇佐美定滿（現行）`usami_sadamitsu`

- 水龍攻 `b19_water_dragon`
- 冰茅圍籬 `b19_ice_fence`
- 越後流兵法 `b19_echigo_tactics`
- 車懸 `b19_kuruma_gakari`

## 宇佐美定滿（舊版）`usami_sadamitsu_old`

- 水龍攻 `b19_old_water_dragon`
- 冰茅圍籬 `b19_old_ice_fence`
- 越後流兵法 `b19_old_echigo_tactics`
- 駿河守 `b19_old_suruga_lord`
- 車懸 `b19_old_kuruma_gakari`

## 柳生宗嚴（現行）`yagyu_muneyoshi`

- 柳生新陰流．神取 `c24_kamitori`
- 柳生新陰流．浮舟 `c24_ukifune`
- 柳生新陰流．全劍 `c24_zenken`
- 柳生新陰流．最期一生 `c24_last_life`

## 柳生宗嚴（舊版）`yagyu_muneyoshi_old`

- 柳生新陰流．無刀取 `c24_old_muto_dori`
- 柳生新陰流．浮舟 `c24_old_ukifune`
- 柳生新陰流 `c24_old_shinkage`
- 柳生新陰流．激‧雙嵐陣 `c24_old_twin_storm`

## 山縣昌景（現行）`yamagata_masakage`

- 岩衝 `b20_rock_rush`
- 壞毀波 `b20_ruin_wave`
- 強韌 `b20_tenacity`
- 赤備猛攻 `b20_red_charge`

## 山縣昌景（舊版）`yamagata_masakage_old`

- 岩衝 `b20_old_rock_rush`
- 壞毀波 `b20_old_ruin_wave`
- 強韌 `b20_old_tenacity`
- 赤備猛攻 `b20_old_red_charge`

## 山本勘助（現行）`yamamoto_kansuke`

- 實界滑移 `b12_reality_slide`
- 浮世一生 `b12_floating_life`
- 訐謀攻勢 `b12_scheming_offense`
- 狂心 `b12_mad_heart`

## 瑞龍院日秀（現行）`zuiryuin_nisshu`

- 竊魔 `a21_mana_steal`
- 半刃擊 `a21_half_blade`
- 能量壓迫 `a21_energy_press`
- 珍珠圓盤 `a21_pearl_disc`
- 偽情報 `a21_false_intel`

## 瑞龍院日秀（舊版）`zuiryuin_nisshu_old`

- 意志腐化 `a21_old_will_corruption`
- 嗜血結界 `a21_old_blood_barrier`
- 珍珠圓盤 `a21_old_pearl_disc`
- 退化靈氣 `a21_old_decay_aura`
- 偽情報 `a21_old_false_intel`

