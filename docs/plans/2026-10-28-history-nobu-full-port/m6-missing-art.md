# M6 仍缺的美術（第 1～9 批）

> **2026-10-29 使用者決定：缺的美術用 Grok 生成**（README D17）。沿用 Grok Build CLI 的 `image_gen`，經 `scripts/item_icon_import.lua`／`ui_texture_import.lua` 匯入，每次生成都記到 `docs/plans/2026-10-24-omfue-month-mvp/asset-ledger.md`。本檔由工具產生；這段註記在重新產生時要保留，或改寫進產生器。

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --missing-art` 產生，不要手改。逐批細節在 `m6-batch<N>-presentation.md`。

這份清單留給使用者決定要補哪些美術；目前不下載任何外部素材。替代方案都已經在 UE 生效：

- **頭像**：共用佔位圖 `hero-placeholder-v1.png`，由 HUD sync 在英雄上架時自動補上。
- **技能圖示**：共用佔位圖 `ability-placeholder-v1.png`，由 HUD sync 補上，建置時會印出警告。
- **cue**：依 cue 目錄的 kind 播通用佔位特效（環、圓盤、連線；sound 不畫）。之後可以在 Editor 的 Om Presentation Cues 指定美術。

## 頭像（替代：共用佔位圖）（26）

| 批 | 英雄 | 原因 |
|---|---|---|
| 1 | A01 德川家康 | 頭像（the site's portrait is Dota 2 art） |
| 1 | B27 小島貞興 | 頭像（the site's portrait is Dota 2 art） |
| 1 | C03 島津義弘 | 頭像（the site's portrait is Dota 2 art） |
| 2 | C09 立花誾千代 | 頭像（the site's portrait is Dota 2 art） |
| 2 | B30 柿崎景家 | 頭像（the site's portrait is Dota 2 art） |
| 2 | A24 佐久間盛政 | 頭像（the site's portrait is Dota 2 art） |
| 2 | A23 宇喜多秀家 | 頭像（the site's portrait is Dota 2 art） |
| 2 | A35 島左近 | 頭像（the site's portrait is Dota 2 art） |
| 3 | C14 齋藤義龍 | 頭像（the site's portrait is Dota 2 art） |
| 3 | A22 仙石秀久 | 頭像（the site's portrait is Dota 2 art） |
| 3 | A36 前野長康 | 頭像（the site's portrait is Dota 2 art） |
| 4 | A33 本多正信 | 頭像（the site's portrait is Dota 2 art） |
| 4 | B18 湖衣姬 | 頭像（the site's portrait is Dota 2 art） |
| 4 | B31 武田信玄 | 頭像（the site's portrait is Dota 2 art） |
| 4 | A20 九鬼嘉隆 | 頭像（the site's portrait is Dota 2 art） |
| 5 | B19 宇佐美定滿 | 頭像（the site's portrait is Dota 2 art） |
| 6 | A10 平手政秀 | 頭像（the site's portrait is Dota 2 art） |
| 6 | A29 石田三成 | 頭像（the site's portrait is Dota 2 art） |
| 7 | C13 龍造寺隆信 | 頭像（the site's portrait is Dota 2 art） |
| 7 | C23 加藤清正 | 頭像（the site's portrait is Dota 2 art） |
| 8 | C25 茶茶 | 頭像（the site's portrait is Dota 2 art） |
| 8 | A21 瑞龍院日秀 | 頭像（the site's portrait is Dota 2 art） |
| 9 | B07 足利義昭 | 頭像（the site's portrait is Dota 2 art） |
| 9 | C08 加藤段藏 | 頭像（the site's portrait is Dota 2 art） |
| 9 | C18 小早川隆景 | 頭像（the site's portrait is Dota 2 art） |
| 9 | C06 石川五右衛門 | 頭像（the site's portrait is Dota 2 art） |

## 技能圖示（替代：共用佔位圖）（130）

| 批 | 英雄 | 技能與原因 |
|---|---|---|
| 1 | A01 德川家康 | 技能圖示 `a01_heaven_net`（Dota built-in texture meepo_earthbind） |
| 1 | A01 德川家康 | 技能圖示 `a01_thunder_blade`（Dota built-in texture zuus_lightning_bolt） |
| 1 | B26 大熊朝秀 | 技能圖示 `b26_charging_wave`（Dota built-in texture brewmaster_fire_permanent_immolation） |
| 1 | B26 大熊朝秀 | 技能圖示 `b26_blood_fury`（Dota built-in texture ember_spirit_sleight_of_fist） |
| 1 | B26 大熊朝秀 | 技能圖示 `b26_kozuke_valor`（Dota built-in texture brewmaster_drunken_brawler_miss） |
| 1 | B26 大熊朝秀 | 技能圖示 `b26_unstoppable`（Dota built-in texture brewmaster_drunken_haze） |
| 1 | B26 大熊朝秀 | 技能圖示 `b26_release_wave`（Dota built-in texture jakiro_liquid_fire） |
| 1 | A12 大谷吉繼 | 技能圖示 `a12_scheme`（Dota built-in texture invoker_quas） |
| 1 | A12 大谷吉繼 | 技能圖示 `a12_mystic_pierce`（Dota built-in texture storm_spirit_static_remnant） |
| 1 | A12 大谷吉繼 | 技能圖示 `a12_awe`（Dota built-in texture lone_druid_savage_roar_bear） |
| 1 | A12 大谷吉繼 | 技能圖示 `a12_mystic_arrow`（Dota built-in texture storm_spirit_overload） |
| 1 | A12 大谷吉繼 | 技能圖示 `a12_mystic_possession`（Dota built-in texture storm_spirit_overload） |
| 1 | A12 大谷吉繼 | 技能圖示 `a12_mystic_field`（Dota built-in texture storm_spirit_static_remnant） |
| 1 | A12 大谷吉繼 | 技能圖示 `a12_awe_t`（Dota built-in texture lone_druid_savage_roar_bear） |
| 1 | B27 小島貞興 | 技能圖示 `b27_horsemanship`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/invoker_tornado.png） |
| 1 | B27 小島貞興 | 技能圖示 `b27_mighty_strike`（Dota built-in texture beastmaster_wild_axes） |
| 1 | B27 小島貞興 | 技能圖示 `b27_fearless_charge`（Dota built-in texture ogre_magi_frost_armor） |
| 1 | B27 小島貞興 | 技能圖示 `b27_rending_blade`（Dota built-in texture beastmaster_primal_roar） |
| 1 | C03 島津義弘 | 技能圖示 `c03_iron_shirt`（Dota built-in texture ogre_magi_frost_armor） |
| 2 | C05 前田慶次 | 技能圖示 `c05_matsukaze`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/invoker_tornado.png） |
| 2 | C05 前田慶次 | 技能圖示 `c05_thunder_dance`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/obsidian_destroyer_arcane_orb.png） |
| 2 | C05 前田慶次 | 技能圖示 `c05_thunder_spear`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/obsidian_destroyer_astral_imprisonment.png） |
| 2 | C05 前田慶次 | 技能圖示 `c05_flash_thunder`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/obsidian_destroyer_sanity_eclipse.png） |
| 2 | A16 鳥居元忠 | 技能圖示 `a16_bloody_grasp`（Dota built-in texture shadow_demon_shadow_poison） |
| 2 | A16 鳥居元忠 | 技能圖示 `a16_blood_link`（Dota built-in texture shadow_demon_demonic_purge） |
| 2 | A16 鳥居元忠 | 技能圖示 `a16_blood_aura`（Dota built-in texture ursa_enrage） |
| 2 | A16 鳥居元忠 | 技能圖示 `a16_dark_blood_decree`（Dota built-in texture earthshaker_fissure_egset） |
| 2 | A35 島左近 | 技能圖示 `a35_scorching_fire`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_r.png） |
| 3 | C24 柳生宗嚴 | 技能圖示 `c24_kamitori`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_w.png） |
| 3 | C24 柳生宗嚴 | 技能圖示 `c24_ukifune`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_e.png） |
| 3 | C24 柳生宗嚴 | 技能圖示 `c24_zenken`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_r.png） |
| 3 | C24 柳生宗嚴 | 技能圖示 `c24_last_life`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_t.png） |
| 3 | A19 黑田孝高 | 技能圖示 `a19_fire_stratagem`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a19_w.png） |
| 3 | A19 黑田孝高 | 技能圖示 `a19_chain_fire`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a19_e.png） |
| 3 | A19 黑田孝高 | 技能圖示 `a19_fire_echo`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a19_r.png） |
| 3 | A19 黑田孝高 | 技能圖示 `a19_siege_fire`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a19_t.png） |
| 3 | A22 仙石秀久 | 技能圖示 `a22_flame_whirl`（Dota built-in texture juggernaut_blade_fury） |
| 3 | A22 仙石秀久 | 技能圖示 `a22_flame_heart`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/Flame_Guard_icon.png） |
| 3 | A22 仙石秀久 | 技能圖示 `a22_boiling_blood`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/Launch_Fire_Spirit_icon.png） |
| 3 | A22 仙石秀久 | 技能圖示 `a22_torment`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/Soul_Catcher_icon.png） |
| 3 | A36 前野長康 | 技能圖示 `a36_deadly_strike`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_r.png） |
| 3 | B34 武田勝賴 | 技能圖示 `b34_wrath_axe`（Dota built-in texture beastmaster_wild_axes） |
| 3 | B34 武田勝賴 | 技能圖示 `b34_spike_wall`（Dota built-in texture sandking_burrowstrike_alt） |
| 3 | B34 武田勝賴 | 技能圖示 `b34_frenzy`（Dota built-in texture beastmaster_inner_beast） |
| 3 | B34 武田勝賴 | 技能圖示 `b34_tenmoku_howl`（Dota built-in texture beastmaster_primal_roar） |
| 3 | B02 風魔小太郎 | 技能圖示 `b02_vanishing_step`（Dota built-in texture riki_smoke_screen） |
| 3 | B02 風魔小太郎 | 技能圖示 `b02_hazy_shadow`（Dota built-in texture dark_seer_wall_of_replica） |
| 3 | B02 風魔小太郎 | 技能圖示 `b02_blazing_path`（Dota built-in texture ogre_magi_ignite） |
| 3 | B02 風魔小太郎 | 技能圖示 `b02_eighteen_shuriken`（Dota built-in texture meepo_earthbind） |
| 3 | B02 風魔小太郎 | 技能圖示 `b02_four_thunders`（Dota built-in texture shadow_shaman_ether_shock） |
| 4 | B05 毛利元就 | 技能圖示 `b05_chain_lightning`（Dota built-in texture zuus_arc_lightning） |
| 4 | B05 毛利元就 | 技能圖示 `b05_heavenly_thunder`（Dota built-in texture zuus_thundergods_wrath） |
| 4 | B05 毛利元就 | 技能圖示 `b05_three_arrows`（Dota built-in texture zuus_lightning_bolt） |
| 4 | B05 毛利元就 | 技能圖示 `b05_holy_light_burst`（Dota built-in texture greevil_purification） |
| 4 | B11 菊姬 | 技能圖示 `b11_cherry_rain`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_w.png） |
| 4 | B11 菊姬 | 技能圖示 `b11_breath_control`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_e.png） |
| 4 | B11 菊姬 | 技能圖示 `b11_interception`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_r.png） |
| 4 | B11 菊姬 | 技能圖示 `b11_crow_raid`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_f.png） |
| 4 | B11 菊姬 | 技能圖示 `b11_crow_strike`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_t.png） |
| 4 | B11 菊姬 | 技能圖示 `b11_inherited_blood`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_d.png） |
| 5 | B35 上泉信綱 | 技能圖示 `b35_muto_dori`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_r.png） |
| 5 | B25 本願寺顯如 | 技能圖示 `b25_exorcism_strike`（Dota built-in texture nyx_assassin_mana_burn） |
| 5 | B25 本願寺顯如 | 技能圖示 `b25_exorcism_sutra`（Dota built-in texture tinker_heat_seeking_missile） |
| 5 | B25 本願寺顯如 | 技能圖示 `b25_finger_of_death`（Dota built-in texture lion_finger_of_death） |
| 5 | A04 竹中重治 | 技能圖示 `a04_ice_path`（Dota built-in texture jakiro_ice_path） |
| 5 | A04 竹中重治 | 技能圖示 `a04_frost_shelter`（Dota built-in texture crystal_maiden_crystal_nova） |
| 5 | A04 竹中重治 | 技能圖示 `a04_blizzard`（Dota built-in texture crystal_maiden_freezing_field） |
| 5 | A17 佐佐成政 | 技能圖示 `a17_shrapnel`（Dota built-in texture sniper_shrapnel） |
| 5 | A17 佐佐成政 | 技能圖示 `a17_rapid_fire`（Dota built-in texture gyrocopter_flak_cannon） |
| 5 | A17 佐佐成政 | 技能圖示 `a17_vital_shot`（Dota built-in texture gyrocopter_skyhigh_call_down） |
| 5 | A17 佐佐成政 | 技能圖示 `a17_great_cannon`（Dota built-in texture gyrocopter_skyhigh_flak_cannon） |
| 5 | B19 宇佐美定滿 | 技能圖示 `b19_water_dragon`（Dota built-in texture zuus_arc_lightning） |
| 6 | A29 石田三成 | 技能圖示 `a29_sleeve_poison_arrow`（Dota built-in texture viper_viper_strike） |
| 6 | A29 石田三成 | 技能圖示 `a29_abyss_grip`（Dota built-in texture treant_overgrowth） |
| 6 | A29 石田三成 | 技能圖示 `a29_withering_aura`（Dota built-in texture enchantress_natures_attendants） |
| 6 | A29 石田三成 | 技能圖示 `a29_vulture_swarm`（Dota built-in texture death_prophet_exorcism） |
| 6 | A31 森蘭丸 | 技能圖示 `a31_echoing_rain`（Dota built-in texture mirana_starfall） |
| 6 | A31 森蘭丸 | 技能圖示 `a31_shunpo`（Dota built-in texture riki_blink_strike） |
| 6 | A31 森蘭丸 | 技能圖示 `a31_storm_arrow`（Dota built-in texture drow_ranger_marksmanship） |
| 6 | A31 森蘭丸 | 技能圖示 `a31_samidare`（Dota built-in texture medusa_split_shot） |
| 6 | C04 阿國 | 技能圖示 `c04_gathering_dance`（Dota built-in texture zuus_lightning_bolt_immortal） |
| 6 | A32 瀧川一益 | 技能圖示 `a32_flying_slash`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_w_old.png） |
| 6 | A32 瀧川一益 | 技能圖示 `a32_second_slash`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_w_old.png） |
| 6 | A32 瀧川一益 | 技能圖示 `a32_soul_drain`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_e.png） |
| 6 | A32 瀧川一益 | 技能圖示 `a32_phantom_blade`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_w.png） |
| 6 | A32 瀧川一益 | 技能圖示 `a32_soul_blade`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_w.png） |
| 6 | A32 瀧川一益 | 技能圖示 `a32_asura_blade`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_t.png） |
| 7 | C21 宮本武藏 | 技能圖示 `c21_crescent_slash`（Dota built-in texture juggernaut_blade_fury） |
| 7 | A06 井伊直政 | 技能圖示 `a06_red_power`（Dota built-in texture dragon_knight_dragon_blood） |
| 7 | A06 井伊直政 | 技能圖示 `a06_red_meteor`（Dota built-in texture invoker_exort） |
| 7 | A06 井伊直政 | 技能圖示 `a06_red_flame_flow`（Dota built-in texture bloodseeker_blood_bath） |
| 7 | A06 井伊直政 | 技能圖示 `a06_flame_of_anger`（Dota built-in texture dragon_knight_breathe_fire） |
| 7 | A06 井伊直政 | 技能圖示 `a06_red_fury`（Dota built-in texture huskar_life_break_alt） |
| 7 | C19 松姬 | 技能圖示 `c19_paralyzing_poison`（Dota built-in texture death_prophet_exorcism） |
| 7 | C19 松姬 | 技能圖示 `c19_surprise_raid`（Dota built-in texture furion_teleportation） |
| 7 | C19 松姬 | 技能圖示 `c19_inviolable`（Dota built-in texture enchantress_enchant） |
| 8 | C10 香宗我部親泰 | 技能圖示 `c10_black_dragon_wave`（Dota built-in texture faceless_void_time_dilation） |
| 8 | C10 香宗我部親泰 | 技能圖示 `c10_energy_drain`（Dota built-in texture antimage_blink） |
| 8 | C10 香宗我部親泰 | 技能圖示 `c10_echo_sword`（Dota built-in texture antimage_mana_break） |
| 8 | C10 香宗我部親泰 | 技能圖示 `c10_day_night_shift`（Dota built-in texture faceless_void_time_lock） |
| 8 | C10 香宗我部親泰 | 技能圖示 `c10_formless_heaven`（Dota built-in texture luna_eclipse） |
| 8 | A28 羽柴秀吉 | 技能圖示 `a28_creation`（Dota built-in texture zuus_arc_lightning） |
| 8 | A28 羽柴秀吉 | 技能圖示 `a28_spreading_fire`（Dota built-in texture medusa_split_shot） |
| 8 | B33 最上義姬 | 技能圖示 `b33_voodoo_power`（Dota built-in texture bristleback_viscous_nasal_goo） |
| 8 | B33 最上義姬 | 技能圖示 `b33_spider_attack`（Dota built-in texture broodmother_insatiable_hunger） |
| 8 | B13 百地三太夫 | 技能圖示 `b13_sense_break`（Dota built-in texture ogre_magi_ignite） |
| 8 | B13 百地三太夫 | 技能圖示 `b13_hypnosis`（Dota built-in texture winter_wyvern_winters_curse） |
| 8 | B13 百地三太夫 | 技能圖示 `b13_blood_disguise`（Dota built-in texture giant_wolf_critical_strike） |
| 8 | B13 百地三太夫 | 技能圖示 `b13_burrow`（Dota built-in texture rubick_telekinesis_land） |
| 8 | B13 百地三太夫 | 技能圖示 `b13_shukuchi`（Dota built-in texture kunkka_x_marks_the_spot） |
| 8 | B13 百地三太夫 | 技能圖示 `b13_storm`（Dota built-in texture brewmaster_storm_cyclone） |
| 9 | B15 今川義元 | 技能圖示 `b15_tokai_giant`（Dota built-in texture ogre_magi_frost_armor） |
| 9 | B15 今川義元 | 技能圖示 `b15_flame_arrow`（Dota built-in texture clinkz_searing_arrows） |
| 9 | B15 今川義元 | 技能圖示 `b15_giant_hide`（Dota built-in texture tiny_craggy_exterior） |
| 9 | B15 今川義元 | 技能圖示 `b15_multishot`（Dota built-in texture windrunner_focusfire） |
| 9 | B15 今川義元 | 技能圖示 `b15_bluff`（Dota built-in texture tiny_grow） |
| 9 | C07 立花道雪 | 技能圖示 `c07_heaven_thunder`（Dota built-in texture disruptor_thunder_strike） |
| 9 | C07 立花道雪 | 技能圖示 `c07_thunder_shackle`（Dota built-in texture razor_plasma_field） |
| 9 | C07 立花道雪 | 技能圖示 `c07_thunderbolt`（Dota built-in texture zuus_lightning_bolt_immortal） |
| 9 | C07 立花道雪 | 技能圖示 `c07_raijin_wrath`（Dota built-in texture razor_eye_of_the_storm） |
| 9 | C07 立花道雪 | 技能圖示 `c07_storm_move`（Dota built-in texture razor_eye_of_the_storm） |
| 9 | C07 立花道雪 | 技能圖示 `c07_magic_shield`（Dota built-in texture silencer_global_silence） |
| 9 | A13 服部半藏 | 技能圖示 `a13_shadow_slash`（Dota built-in texture riki_tricks_of_the_trade） |
| 9 | B07 足利義昭 | 技能圖示 `b07_starfall`（Dota built-in texture mirana_starfall） |
| 9 | B07 足利義昭 | 技能圖示 `b07e_camp_regen`（Dota built-in texture enchantress_natures_attendants） |
| 9 | B24 秋山信友 | 技能圖示 `b24_earth_stele`（Dota built-in texture earthshaker_fissure_alt） |
| 9 | B24 秋山信友 | 技能圖示 `b24_taunt`（Dota built-in texture earthshaker_enchant_totem） |
| 9 | B24 秋山信友 | 技能圖示 `b24_stone_resonance`（Dota built-in texture earthshaker_echo_slam） |
| 9 | B24 秋山信友 | 技能圖示 `b24_earth_prison`（Dota built-in texture earthshaker_fissure） |
| 9 | B24 秋山信友 | 技能圖示 `b24_shatter`（Dota built-in texture earthshaker_fissure） |

## 沒有原作資產的 cue（替代：依 kind 的通用佔位特效）（129）

| 批 | 英雄 | cue | kind |
|---|---|---|---|
| 1 | A01 德川家康 | `a01w_holy_bolt` | effect |
| 1 | B20 山縣昌景 | `b20t_epicenter` | effect |
| 1 | B26 大熊朝秀 | `b26e_model_scale_up` | effect |
| 1 | C22 佐佐木小次郎 | `c22d_cast_sound` | sound |
| 1 | C22 佐佐木小次郎 | `c22r_crit_sound` | sound |
| 1 | A26 濃姬 | `a26r_crit_gesture` | gesture |
| 1 | A26 濃姬 | `a26t_big_explosion` | effect |
| 1 | A26 濃姬 | `a26t_small_explosion` | effect |
| 1 | A26 濃姬 | `a26w_flak_sound` | sound |
| 1 | B27 小島貞興 | `b27r_model_grow` | effect |
| 1 | C03 島津義弘 | `c03t_hammer` | effect |
| 1 | C03 島津義弘 | `c03t_hammer_hit` | effect |
| 1 | C03 島津義弘 | `c03w_grow` | effect |
| 2 | C05 前田慶次 | `c05e_impact` | impact |
| 2 | B30 柿崎景家 | `b30r_windwalk_sound` | sound |
| 2 | A24 佐久間盛政 | `a24_blood_spray` | effect |
| 2 | C01 明智光秀 | `c01e_light_strike` | effect |
| 2 | C01 明智光秀 | `c01r_crit_sound` | sound |
| 2 | C01 明智光秀 | `c01r_slam_gesture` | gesture |
| 2 | C01 明智光秀 | `c01w_slam_gesture` | gesture |
| 2 | A23 宇喜多秀家 | `a23t_cast` | effect |
| 2 | A35 島左近 | `a35t_cast_sound` | sound |
| 3 | C14 齋藤義龍 | `c14e_breath` | projectile |
| 3 | C14 齋藤義龍 | `c14e_hit_sound` | impact |
| 3 | C14 齋藤義龍 | `c14t_cast_sound` | sound |
| 3 | C24 柳生宗嚴 | `c24w_cast_sound` | sound |
| 3 | A22 仙石秀久 | `a22e_bolt` | projectile |
| 3 | A22 仙石秀久 | `a22w_spin_gesture` | gesture |
| 3 | A30 丹羽長秀 | `a30e_toss` | impact |
| 3 | A30 丹羽長秀 | `a30w_wolf_howl` | effect |
| 3 | A36 前野長康 | `a36e_explosion` | effect |
| 3 | A36 前野長康 | `a36r_vo` | sound |
| 3 | B02 風魔小太郎 | `b02w_spirit_lance` | effect |
| 4 | C02 明智秀滿 | `c02t_riptide` | effect |
| 4 | A33 本多正信 | `a33e_ghost_walk_sound` | sound |
| 4 | B03 齋藤朝信 | `b03t_hit_sound` | impact |
| 4 | B03 齋藤朝信 | `b03w_split_earth` | effect |
| 4 | B11 菊姬 | `b11e_troll_priest_heal` | effect |
| 4 | B11 菊姬 | `b11f_blade_glow` | effect |
| 4 | B11 菊姬 | `b11t_echo_slam_gesture` | gesture |
| 4 | B09 真田昌幸 | `b09e_pulse` | effect |
| 4 | B09 真田昌幸 | `b09r_toggle_sound` | sound |
| 4 | B09 真田昌幸 | `b09t_cast_sound` | sound |
| 4 | B09 真田昌幸 | `b09w_cast_sound` | sound |
| 4 | B18 湖衣姬 | `b18t_sound` | sound |
| 4 | B18 湖衣姬 | `b18t_title` | sound |
| 4 | B18 湖衣姬 | `b18t_vo` | sound |
| 4 | A20 九鬼嘉隆 | `a20r_wave` | projectile |
| 4 | A20 九鬼嘉隆 | `a20t_tidehunter_model` | aura |
| 5 | A03 石川數正 | `a03w_tick_sound` | sound |
| 5 | B35 上泉信綱 | `b35r_counter_sound` | sound |
| 5 | C20 長宗我部元親 | `c20t_title` | sound |
| 5 | B36 直江兼續 | `b36e_cast` | effect |
| 5 | C12 大友宗麟 | `c12r_mask` | aura |
| 5 | C12 大友宗麟 | `c12t_beam` | projectile |
| 5 | C12 大友宗麟 | `c12w_impact` | effect |
| 5 | A17 佐佐成政 | `a17e_cast_sound` | sound |
| 5 | B10 三好長慶 | `b10t_ghost_ship_sound` | sound |
| 5 | B10 三好長慶 | `b10w_spirit_hit_fx` | effect |
| 5 | B19 宇佐美定滿 | `b19t_sound` | channel |
| 5 | C11 立花宗茂 | `c11t_title` | sound |
| 5 | C11 立花宗茂 | `c11w_arc_lightning` | effect |
| 5 | C11 立花宗茂 | `c11w_bolt` | projectile |
| 5 | C16 果心居士 | `c16r_heal_sound` | sound |
| 5 | C16 果心居士 | `c16r_spirit_hit_fx` | summon |
| 5 | C16 果心居士 | `c16t_split_earth` | effect |
| 6 | A27 柴田勝家 | `a27r_crit_sound` | sound |
| 6 | A27 柴田勝家 | `a27r_echo_slam_gesture` | gesture |
| 6 | A27 柴田勝家 | `relocate_sound` | sound |
| 6 | A10 平手政秀 | `a10e_hit_burst` | impact |
| 6 | A10 平手政秀 | `a10e_sword_wave` | projectile |
| 6 | A25 織田信長 | `a25d_message` | effect |
| 6 | A25 織田信長 | `a25d_press_cast` | effect |
| 6 | A25 織田信長 | `a25r_crit_sound` | sound |
| 6 | A25 織田信長 | `a25r_slam_gesture` | gesture |
| 6 | A25 織田信長 | `a25t_small_tornado` | effect |
| 6 | A25 織田信長 | `a25t_tornado` | effect |
| 6 | A29 石田三成 | `a29e_overgrowth_fx` | aura |
| 6 | A29 石田三成 | `a29w_poison_number` | effect |
| 6 | A31 森蘭丸 | `a31t_split_hit` | effect |
| 6 | A31 森蘭丸 | `a31w_impact_sound` | sound |
| 6 | C04 阿國 | `c04r_sound` | sound |
| 6 | C04 阿國 | `c04w_sound` | sound |
| 6 | A32 瀧川一益 | `a32e_time_lapse_sound` | sound |
| 6 | A32 瀧川一益 | `a32w_thunderclap` | effect |
| 7 | C13 龍造寺隆信 | `c13r_wind_fx` | effect |
| 7 | C13 龍造寺隆信 | `c13t_title` | sound |
| 7 | A05 酒井忠次 | `a05d_die_gesture` | aura |
| 7 | C21 宮本武藏 | `c21r_crit_sound` | sound |
| 7 | C21 宮本武藏 | `c21r_slam_gesture` | gesture |
| 7 | C21 宮本武藏 | `c21t_flash` | effect |
| 7 | A15 寧寧 | `a15e_moonfall` | effect |
| 7 | A15 寧寧 | `a15r_glaive` | projectile |
| 7 | A15 寧寧 | `a15t_split_hit_sound` | impact |
| 7 | A15 寧寧 | `a15w_cast_sound` | sound |
| 7 | B12 山本勘助 | `b12e_heal_sound` | sound |
| 8 | B14 片倉景綱 | `b14d_night_sound` | sound |
| 8 | B14 片倉景綱 | `b14r_slam_gesture` | gesture |
| 8 | B23 伊勢姬 | `b23d_ghost_death` | effect |
| 8 | B23 伊勢姬 | `b23r_heal_sound` | sound |
| 8 | B23 伊勢姬 | `b23t_title` | sound |
| 8 | C10 香宗我部親泰 | `c10e_manavoid_cast` | effect |
| 8 | C10 香宗我部親泰 | `c10r_cleave` | effect |
| 8 | A28 羽柴秀吉 | `a28tw_fire_projectile` | projectile |
| 8 | A28 羽柴秀吉 | `a28w_frog_model` | aura |
| 8 | A21 瑞龍院日秀 | `a21d_mana_break_sound` | sound |
| 8 | A21 瑞龍院日秀 | `a21w_attack_projectile` | aura |
| 8 | B13 百地三太夫 | `b13d_burrow_animation` | aura |
| 8 | B13 百地三太夫 | `b13d_crater_model` | aura |
| 8 | B13 百地三太夫 | `b13e_bell_toll` | effect |
| 8 | B13 百地三太夫 | `b13t_tornado_sound` | sound |
| 8 | A09 蜂須賀政勝 | `a09e_hex` | effect |
| 9 | B15 今川義元 | `b15d_split_hit` | impact |
| 9 | C07 立花道雪 | `c07d_attack_pose` | effect |
| 9 | C07 立花道雪 | `c07d_pose_freeze` | aura |
| 9 | C07 立花道雪 | `c07t_bolt` | effect |
| 9 | A13 服部半藏 | `a13e_hook_throw_sound` | sound |
| 9 | A13 服部半藏 | `a13t_mirror_image_sound` | sound |
| 9 | A13 服部半藏 | `a13t_title` | sound |
| 9 | B07 足利義昭 | `b07e_teleport_sound` | sound |
| 9 | B07 足利義昭 | `b07t_voice` | sound |
| 9 | B07 足利義昭 | `b07w_impact` | effect |
| 9 | C08 加藤段藏 | `c08r_cast_gesture` | gesture |
| 9 | C08 加藤段藏 | `c08t_cheese` | effect |
| 9 | C08 加藤段藏 | `c08t_title` | sound |
| 9 | C18 小早川隆景 | `c18t_cast_sound` | sound |
| 9 | B24 秋山信友 | `b24r_avalanche` | projectile |
| 9 | B22 太原雪齋 | `hero_lina_dragon_slave` | effect |
| 9 | C06 石川五右衛門 | `item_money_bag` | effect |

