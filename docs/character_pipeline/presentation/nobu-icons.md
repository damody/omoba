# 原作 nobu 圖示

由 `tools\lua\lua.exe scripts\import_nobu_icons.lua` 產生，不要手改。人工指定寫在 `nobu-icon-overrides.lua`。

- 來源：history_nobu@f966720（頭像優先用 nobu 官網的英雄圖）。
- 只用 nobu 自製的圖：技能與道具取 addon `resource/flash3/images` 底下 custom 資料夾的檔案；KV 指向 Dota 內建圖示的一律用佔位圖。
- 每張圖裁成置中正方形、縮放成 128×128，存成 `omfue/Content/Omoba/UI/nobu-<kind>-<id>.png`。

| 種類 | 原作圖 | 佔位圖 |
|---|---|---|
| hero | 68 | 29 |
| ability | 303 | 143 |
| item | 156 | 20 |

## 使用佔位圖的條目

| 種類 | id | 原作 id | 原因 |
|---|---|---|---|
| ability | `a01_heaven_net` | A01E | Dota built-in texture meepo_earthbind |
| ability | `a01_thunder_blade` | A01R | Dota built-in texture zuus_lightning_bolt |
| ability | `a02_shadow_meld` | A02T | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/windrunner_windrun_sylvan.png |
| ability | `a02_venom_arrow` | A02W | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/gnoll_assassin_envenomed_weapon.png |
| ability | `a02_wind_drift` | A02R | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/windrunner_windrun.png |
| ability | `a02_wind_prayer` | A02E | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/skywrath_mage_concussive_shot.png |
| ability | `a04_blizzard` | A04T | Dota built-in texture crystal_maiden_freezing_field |
| ability | `a04_frost_shelter` | A04R | Dota built-in texture crystal_maiden_crystal_nova |
| ability | `a04_ice_path` | A04E | Dota built-in texture jakiro_ice_path |
| ability | `a06_flame_of_anger` | A06D | Dota built-in texture dragon_knight_breathe_fire |
| ability | `a06_red_flame_flow` | A06R | Dota built-in texture bloodseeker_blood_bath |
| ability | `a06_red_fury` | A06T | Dota built-in texture huskar_life_break_alt |
| ability | `a06_red_meteor` | A06E | Dota built-in texture invoker_exort |
| ability | `a06_red_power` | A06W | Dota built-in texture dragon_knight_dragon_blood |
| ability | `a07_unscathed` | A07W | Dota built-in texture phantom_lancer_sunwarrior_phantom_edge |
| ability | `a12_awe` | A12R | Dota built-in texture lone_druid_savage_roar_bear |
| ability | `a12_awe_t` | A12R_HIDE | Dota built-in texture lone_druid_savage_roar_bear |
| ability | `a12_mystic_arrow` | A12F | Dota built-in texture storm_spirit_overload |
| ability | `a12_mystic_field` | A12E_HIDE | Dota built-in texture storm_spirit_static_remnant |
| ability | `a12_mystic_pierce` | A12E | Dota built-in texture storm_spirit_static_remnant |
| ability | `a12_mystic_possession` | A12T | Dota built-in texture storm_spirit_overload |
| ability | `a12_scheme` | A12D | Dota built-in texture invoker_quas |
| ability | `a13_shadow_slash` | A13T | Dota built-in texture riki_tricks_of_the_trade |
| ability | `a16_blood_aura` | A16R | Dota built-in texture ursa_enrage |
| ability | `a16_blood_link` | A16E | Dota built-in texture shadow_demon_demonic_purge |
| ability | `a16_bloody_grasp` | A16W | Dota built-in texture shadow_demon_shadow_poison |
| ability | `a16_dark_blood_decree` | A16T | Dota built-in texture earthshaker_fissure_egset |
| ability | `a17_great_cannon` | A17T | Dota built-in texture gyrocopter_skyhigh_flak_cannon |
| ability | `a17_rapid_fire` | A17E | Dota built-in texture gyrocopter_flak_cannon |
| ability | `a17_shrapnel` | A17W | Dota built-in texture sniper_shrapnel |
| ability | `a17_vital_shot` | A17R | Dota built-in texture gyrocopter_skyhigh_call_down |
| ability | `a19_chain_fire` | A19E | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a19_e.png |
| ability | `a19_fire_echo` | A19R | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a19_r.png |
| ability | `a19_fire_stratagem` | A19W | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a19_w.png |
| ability | `a19_siege_fire` | A19T | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a19_t.png |
| ability | `a22_boiling_blood` | A22R | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/Launch_Fire_Spirit_icon.png |
| ability | `a22_flame_heart` | A22E | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/Flame_Guard_icon.png |
| ability | `a22_flame_whirl` | A22W | Dota built-in texture juggernaut_blade_fury |
| ability | `a22_torment` | A22T | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/Soul_Catcher_icon.png |
| ability | `a28_creation` | A28E | Dota built-in texture zuus_arc_lightning |
| ability | `a28_spreading_fire` | A28TW | Dota built-in texture medusa_split_shot |
| ability | `a29_abyss_grip` | A29E | Dota built-in texture treant_overgrowth |
| ability | `a29_sleeve_poison_arrow` | A29W | Dota built-in texture viper_viper_strike |
| ability | `a29_vulture_swarm` | A29T | Dota built-in texture death_prophet_exorcism |
| ability | `a29_withering_aura` | A29R | Dota built-in texture enchantress_natures_attendants |
| ability | `a31_echoing_rain` | A31W | Dota built-in texture mirana_starfall |
| ability | `a31_samidare` | A31T | Dota built-in texture medusa_split_shot |
| ability | `a31_shunpo` | A31E | Dota built-in texture riki_blink_strike |
| ability | `a31_storm_arrow` | A31R | Dota built-in texture drow_ranger_marksmanship |
| ability | `a32_asura_blade` | A32T | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_t.png |
| ability | `a32_flying_slash` | A32W | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_w_old.png |
| ability | `a32_phantom_blade` | A32R | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_w.png |
| ability | `a32_second_slash` | A32D | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_w_old.png |
| ability | `a32_soul_blade` | A32F | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_w.png |
| ability | `a32_soul_drain` | A32E | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_e.png |
| ability | `a34_frost_bolt` | A34W | Dota built-in texture crystal_maiden_frostbite |
| ability | `a34_frost_breath` | A34R | Dota built-in texture ancient_apparition_chilling_touch |
| ability | `a34_iceberg_strike` | A34E | Dota built-in texture invoker_ice_wall |
| ability | `a34_rapid_cooling` | A34T | Dota built-in texture invoker_quas |
| ability | `a35_scorching_fire` | A35T | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_r.png |
| ability | `a36_deadly_strike` | A36T | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_r.png |
| ability | `b02_blazing_path` | B02E | Dota built-in texture ogre_magi_ignite |
| ability | `b02_eighteen_shuriken` | B02R | Dota built-in texture meepo_earthbind |
| ability | `b02_four_thunders` | B02T | Dota built-in texture shadow_shaman_ether_shock |
| ability | `b02_hazy_shadow` | B02W | Dota built-in texture dark_seer_wall_of_replica |
| ability | `b02_vanishing_step` | B02D | Dota built-in texture riki_smoke_screen |
| ability | `b05_chain_lightning` | B05W | Dota built-in texture zuus_arc_lightning |
| ability | `b05_heavenly_thunder` | B05E | Dota built-in texture zuus_thundergods_wrath |
| ability | `b05_holy_light_burst` | B05T | Dota built-in texture greevil_purification |
| ability | `b05_three_arrows` | B05R | Dota built-in texture zuus_lightning_bolt |
| ability | `b07_starfall` | B07W | Dota built-in texture mirana_starfall |
| ability | `b07e_camp_regen` | B07W_old_soldiercamp | Dota built-in texture enchantress_natures_attendants |
| ability | `b11_breath_control` | B11E | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_e.png |
| ability | `b11_cherry_rain` | B11W | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_w.png |
| ability | `b11_crow_raid` | B11F | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_f.png |
| ability | `b11_crow_strike` | B11T | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_t.png |
| ability | `b11_inherited_blood` | B11D | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_d.png |
| ability | `b11_interception` | B11R | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_r.png |
| ability | `b13_blood_disguise` | B13R | Dota built-in texture giant_wolf_critical_strike |
| ability | `b13_burrow` | B13D | Dota built-in texture rubick_telekinesis_land |
| ability | `b13_hypnosis` | B13E | Dota built-in texture winter_wyvern_winters_curse |
| ability | `b13_sense_break` | B13W | Dota built-in texture ogre_magi_ignite |
| ability | `b13_shukuchi` | B13B | Dota built-in texture kunkka_x_marks_the_spot |
| ability | `b13_storm` | B13T | Dota built-in texture brewmaster_storm_cyclone |
| ability | `b15_bluff` | B15T | Dota built-in texture tiny_grow |
| ability | `b15_flame_arrow` | B15E | Dota built-in texture clinkz_searing_arrows |
| ability | `b15_giant_hide` | B15R | Dota built-in texture tiny_craggy_exterior |
| ability | `b15_multishot` | B15D | Dota built-in texture windrunner_focusfire |
| ability | `b15_tokai_giant` | B15W | Dota built-in texture ogre_magi_frost_armor |
| ability | `b19_water_dragon` | B19W | Dota built-in texture zuus_arc_lightning |
| ability | `b24_earth_prison` | B24T | Dota built-in texture earthshaker_fissure |
| ability | `b24_earth_stele` | B24W | Dota built-in texture earthshaker_fissure_alt |
| ability | `b24_shatter` | B24T2 | Dota built-in texture earthshaker_fissure |
| ability | `b24_stone_resonance` | B24R | Dota built-in texture earthshaker_echo_slam |
| ability | `b24_taunt` | B24E | Dota built-in texture earthshaker_enchant_totem |
| ability | `b25_exorcism_strike` | B25W | Dota built-in texture nyx_assassin_mana_burn |
| ability | `b25_exorcism_sutra` | B25E | Dota built-in texture tinker_heat_seeking_missile |
| ability | `b25_finger_of_death` | B25R | Dota built-in texture lion_finger_of_death |
| ability | `b26_blood_fury` | B26E | Dota built-in texture ember_spirit_sleight_of_fist |
| ability | `b26_charging_wave` | B26W | Dota built-in texture brewmaster_fire_permanent_immolation |
| ability | `b26_kozuke_valor` | B26R | Dota built-in texture brewmaster_drunken_brawler_miss |
| ability | `b26_release_wave` | B26D | Dota built-in texture jakiro_liquid_fire |
| ability | `b26_unstoppable` | B26T | Dota built-in texture brewmaster_drunken_haze |
| ability | `b27_fearless_charge` | B27R | Dota built-in texture ogre_magi_frost_armor |
| ability | `b27_horsemanship` | B27W | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/invoker_tornado.png |
| ability | `b27_mighty_strike` | B27E | Dota built-in texture beastmaster_wild_axes |
| ability | `b27_rending_blade` | B27T | Dota built-in texture beastmaster_primal_roar |
| ability | `b33_spider_attack` | B33T | Dota built-in texture broodmother_insatiable_hunger |
| ability | `b33_voodoo_power` | B33R | Dota built-in texture bristleback_viscous_nasal_goo |
| ability | `b34_frenzy` | B34R | Dota built-in texture beastmaster_inner_beast |
| ability | `b34_spike_wall` | B34E | Dota built-in texture sandking_burrowstrike_alt |
| ability | `b34_tenmoku_howl` | B34T | Dota built-in texture beastmaster_primal_roar |
| ability | `b34_wrath_axe` | B34W | Dota built-in texture beastmaster_wild_axes |
| ability | `b35_muto_dori` | B35R | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_r.png |
| ability | `c03_iron_shirt` | C03W | Dota built-in texture ogre_magi_frost_armor |
| ability | `c04_gathering_dance` | C04R | Dota built-in texture zuus_lightning_bolt_immortal |
| ability | `c05_flash_thunder` | C05T | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/obsidian_destroyer_sanity_eclipse.png |
| ability | `c05_matsukaze` | C05W | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/invoker_tornado.png |
| ability | `c05_thunder_dance` | C05E | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/obsidian_destroyer_arcane_orb.png |
| ability | `c05_thunder_spear` | C05R | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/obsidian_destroyer_astral_imprisonment.png |
| ability | `c07_heaven_thunder` | C07W | Dota built-in texture disruptor_thunder_strike |
| ability | `c07_magic_shield` | C07D | Dota built-in texture silencer_global_silence |
| ability | `c07_raijin_wrath` | C07T | Dota built-in texture razor_eye_of_the_storm |
| ability | `c07_storm_move` | C07T2 | Dota built-in texture razor_eye_of_the_storm |
| ability | `c07_thunder_shackle` | C07E | Dota built-in texture razor_plasma_field |
| ability | `c07_thunderbolt` | C07R | Dota built-in texture zuus_lightning_bolt_immortal |
| ability | `c10_black_dragon_wave` | C10W | Dota built-in texture faceless_void_time_dilation |
| ability | `c10_day_night_shift` | C10D | Dota built-in texture faceless_void_time_lock |
| ability | `c10_echo_sword` | C10R | Dota built-in texture antimage_mana_break |
| ability | `c10_energy_drain` | C10E | Dota built-in texture antimage_blink |
| ability | `c10_formless_heaven` | C10T | Dota built-in texture luna_eclipse |
| ability | `c15_burst_arrow` | C15E | Dota built-in texture clinkz_searing_arrows |
| ability | `c15_hundred_arrows` | C15T | Dota built-in texture drow_ranger_marksmanship |
| ability | `c15_precise_light` | C15R | Dota built-in texture mirana_invis |
| ability | `c15_scatter_shot` | C15W | Dota built-in texture mirana_starfall |
| ability | `c19_inviolable` | C19T | Dota built-in texture enchantress_enchant |
| ability | `c19_paralyzing_poison` | C19R | Dota built-in texture death_prophet_exorcism |
| ability | `c19_surprise_raid` | C19D | Dota built-in texture furion_teleportation |
| ability | `c21_crescent_slash` | C21W | Dota built-in texture juggernaut_blade_fury |
| ability | `c24_kamitori` | C24W | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_w.png |
| ability | `c24_last_life` | C24T | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_t.png |
| ability | `c24_ukifune` | C24E | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_e.png |
| ability | `c24_zenken` | C24R | file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_r.png |
| hero | `ashikaga_yoshiaki` | B07 | the site's portrait is Dota 2 art |
| hero | `chacha` | C25 | the site's portrait is Dota 2 art |
| hero | `hirate_masahide` | A10 | the site's portrait is Dota 2 art |
| hero | `honda_masanobu` | A33 | the site's portrait is Dota 2 art |
| hero | `inahime` | A02 | the site's portrait is Dota 2 art |
| hero | `ishida_mitsunari` | A29 | the site's portrait is Dota 2 art |
| hero | `ishikawa_goemon` | C06 | the site's portrait is Dota 2 art |
| hero | `kakizaki_kageie` | B30 | the site's portrait is Dota 2 art |
| hero | `kato_danzo` | C08 | the site's portrait is Dota 2 art |
| hero | `kato_kiyomasa` | C23 | the site's portrait is Dota 2 art |
| hero | `kobayakawa_takakage` | C18 | the site's portrait is Dota 2 art |
| hero | `koihime` | B18 | the site's portrait is Dota 2 art |
| hero | `kojima_sadaoki` | B27 | the site's portrait is Dota 2 art |
| hero | `kuki_yoshitaka` | A20 | the site's portrait is Dota 2 art |
| hero | `maeno_nagayasu` | A36 | the site's portrait is Dota 2 art |
| hero | `murakami_yoshikiyo` | B29 | the site's portrait is Dota 2 art |
| hero | `omatsu` | A34 | the site's portrait is Dota 2 art |
| hero | `ryuzoji_takanobu` | C13 | the site's portrait is Dota 2 art |
| hero | `saito_yoshitatsu` | C14 | the site's portrait is Dota 2 art |
| hero | `sakuma_morimasa` | A24 | the site's portrait is Dota 2 art |
| hero | `sengoku_hidehisa` | A22 | the site's portrait is Dota 2 art |
| hero | `shima_sakon` | A35 | the site's portrait is Dota 2 art |
| hero | `shimazu_yoshihiro` | C03 | the site's portrait is Dota 2 art |
| hero | `tachibana_ginchiyo` | C09 | the site's portrait is Dota 2 art |
| hero | `takeda_shingen` | B31 | the site's portrait is Dota 2 art |
| hero | `tokugawa_ieyasu` | A01 | the site's portrait is Dota 2 art |
| hero | `ukita_hideie` | A23 | the site's portrait is Dota 2 art |
| hero | `usami_sadamitsu` | B19 | the site's portrait is Dota 2 art |
| hero | `zuiryuin_nisshu` | A21 | the site's portrait is Dota 2 art |
| item | `ancient_doll` | item_ancient_doll | no AbilityTextureName in the original KV |
| item | `ghost_gloves` | item_ghost_gloves | no AbilityTextureName in the original KV |
| item | `gold_to_prestige` | item_gold_to_prestige | no AbilityTextureName in the original KV |
| item | `hachiman_bell` | item_hachiman_bell | no AbilityTextureName in the original KV |
| item | `ice_book` | item_ice_book | no AbilityTextureName in the original KV |
| item | `kat_steel` | item_kat_steel | no AbilityTextureName in the original KV |
| item | `ksitigarbha` | item_ksitigarbha | no AbilityTextureName in the original KV |
| item | `magic_ring` | item_magic_ring | no AbilityTextureName in the original KV |
| item | `poisonous_ring` | item_poisonous_ring | no AbilityTextureName in the original KV |
| item | `ronin_jitte_2` | item_ronin_jitte_2 | no AbilityTextureName in the original KV |
| item | `shintogo_kunimitsu` | item_Shintogo_Kunimitsu | no AbilityTextureName in the original KV |
| item | `the_great_bow_of_scorpion` | item_the_great_bow_of_scorpion | no AbilityTextureName in the original KV |
| item | `the_great_spirited_away` | item_the_great_spirited_away | no AbilityTextureName in the original KV |
| item | `the_great_sword_of_banish` | item_the_great_sword_of_banish | no AbilityTextureName in the original KV |
| item | `the_great_sword_of_berserker` | item_the_great_sword_of_berserker | no AbilityTextureName in the original KV |
| item | `the_great_sword_of_disarmament` | item_the_great_sword_of_disarmament | no AbilityTextureName in the original KV |
| item | `the_great_sword_of_weakness` | item_the_great_sword_of_weakness | no AbilityTextureName in the original KV |
| item | `the_shell_of_spirituality_kappa` | item_the_shell_of_spirituality_kappa | no AbilityTextureName in the original KV |
| item | `wakizashi` | item_wakizashi | no AbilityTextureName in the original KV |
| item | `waterfall_book` | item_waterfall_book | no AbilityTextureName in the original KV |
