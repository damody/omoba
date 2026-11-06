# 產生器還沒產生的寫法與實作優先序

由 `tools\lua\lua.exe docs\character_pipeline\tools\v3_codegen_coverage.lua` 產生，不要手改。取代 [ir-blockers.md](ir-blockers.md)（M3' ③，[m3-rust-codegen-design.md](m3-rust-codegen-design.md) §2.3）。產生器（`v3_rust.lua`）或 `ability_kit` 每補一批寫法就重跑。

- 英雄技能：447 招中 447 招產生成 Rust。還不能產生的招式不輸出程式碼，列在這份報告；該英雄不進 `hero_dlls`。全部技能都產生的英雄：97 / 97 位。
- 道具（items.dll，M7）：168 件中 37 件產生成 Rust。
- 一位英雄要全部技能都產生才算解鎖。順序是 M5（已上架的 10 位），然後 M6 第 1～9 批（[m6-waves.md](m6-waves.md)）；每批內先做「每多支援一項、能解鎖最多英雄」的那位英雄缺的項目。
- 「主要負責」是起點：`產生器` 是運算式、選取、目標這類產生器自己能降低的寫法；`ability_kit` 是要先有 kit 呼叫或 host 元件的寫法（工作線 B）。
- 「分類」（D7，m3-rust-codegen-design.md §2.4）：英雄技能與道具合計 2 招以上用到的寫法是 `共用`，做成 v3 詞彙＋產生器＋`ability_kit`；只有 1 招用到的是 `hook 候選`，在 v3 標 `hook` 並手寫產生的 trait（§5 的清單）。

## 1. 實作順序

### M5（已上架的 10 位英雄：B01、C17 換掉 IR，其餘換掉 Rust handler）（10 位）

開始時已全部產生：10 位；做完本批的項目後：10 位。

（本批不需要新的支援）

### M6 第 1 批（9 位）

開始時已全部產生：9 位；做完本批的項目後：9 位。

（本批不需要新的支援）

### M6 第 2 批（10 位）

開始時已全部產生：10 位；做完本批的項目後：10 位。

（本批不需要新的支援）

### M6 第 3 批（9 位）

開始時已全部產生：9 位；做完本批的項目後：9 位。

（本批不需要新的支援）

### M6 第 4 批（9 位）

開始時已全部產生：9 位；做完本批的項目後：9 位。

（本批不需要新的支援）

### M6 第 5 批（9 位）

開始時已全部產生：9 位；做完本批的項目後：9 位。

（本批不需要新的支援）

### M6 第 6 批（9 位）

開始時已全部產生：9 位；做完本批的項目後：9 位。

（本批不需要新的支援）

### M6 第 7 批（9 位）

開始時已全部產生：9 位；做完本批的項目後：9 位。

（本批不需要新的支援）

### M6 第 8 批（9 位）

開始時已全部產生：9 位；做完本批的項目後：9 位。

（本批不需要新的支援）

### M6 第 9 批（13 位）

開始時已全部產生：13 位；做完本批的項目後：13 位。

（本批不需要新的支援）

## 2. 每位英雄、每招還缺什麼

### A02 `inahime`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `a02_venom_arrow` | ✓ 已產生 |
| `a02_wind_prayer` | ✓ 已產生 |
| `a02_wind_drift` | ✓ 已產生 |
| `a02_shadow_meld` | ✓ 已產生 |

### A07 `honda_tadakatsu`（6 / 6 招）　已上架

| 招式 | 還缺 |
|---|---|
| `a07_unscathed` | ✓ 已產生 |
| `a07_red_thunder` | ✓ 已產生 |
| `a07_shattering_spear` | ✓ 已產生 |
| `a07_strongest` | ✓ 已產生 |
| `a07_rouse` | ✓ 已產生 |
| `a07_rouse_free` | ✓ 已產生 |

### A34 `omatsu`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `a34_frost_bolt` | ✓ 已產生 |
| `a34_iceberg_strike` | ✓ 已產生 |
| `a34_frost_breath` | ✓ 已產生 |
| `a34_rapid_cooling` | ✓ 已產生 |

### B01 `saika_magoichi`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b01_sniper_mode` | ✓ 已產生 |
| `b01_saika_corps` | ✓ 已產生 |
| `b01_rain_iron_cannon` | ✓ 已產生 |
| `b01_three_stage` | ✓ 已產生 |

### B04 `date_masamune`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b04_flame_trail_dash` | ✓ 已產生 |
| `b04_fire_blast` | ✓ 已產生 |
| `b04_flame_swiftness` | ✓ 已產生 |
| `b04_tanegashima` | ✓ 已產生 |

### B06 `sanada_yukimura`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b06_brave_warriors` | ✓ 已產生 |
| `b06_flame_fangs` | ✓ 已產生 |
| `b06_mind_eye` | ✓ 已產生 |
| `b06_first_warrior` | ✓ 已產生 |

### B08 `asai_nagamasa`（5 / 5 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b08_fang_slash` | ✓ 已產生 |
| `b08_flash_cut` | ✓ 已產生 |
| `b08_living_soul` | ✓ 已產生 |
| `b08_sky_fang` | ✓ 已產生 |
| `b08_endless_fang` | ✓ 已產生 |

### B29 `murakami_yoshikiyo`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b29_ground_slam` | ✓ 已產生 |
| `b29_quake_axe` | ✓ 已產生 |
| `b29_morale_boost` | ✓ 已產生 |
| `b29_earth_tremor` | ✓ 已產生 |

### C15 `tamako`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `c15_scatter_shot` | ✓ 已產生 |
| `c15_burst_arrow` | ✓ 已產生 |
| `c15_precise_light` | ✓ 已產生 |
| `c15_hundred_arrows` | ✓ 已產生 |

### C17 `oichi`（5 / 5 招）　已上架

| 招式 | 還缺 |
|---|---|
| `c17_light_chain` | ✓ 已產生 |
| `c17_victory_blessing` | ✓ 已產生 |
| `c17_divine_strike` | ✓ 已產生 |
| `c17_aurora_guard` | ✓ 已產生 |
| `c17_devotion` | ✓ 已產生 |

### A01 `tokugawa_ieyasu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a01_realm_momentum` | ✓ 已產生 |
| `a01_tosho_gongen` | ✓ 已產生 |
| `a01_heaven_net` | ✓ 已產生 |
| `a01_thunder_blade` | ✓ 已產生 |
| `a01_mikawa_valor` | ✓ 已產生 |

### A03 `ishikawa_kazumasa`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a03_pacing_gu` | ✓ 已產生 |
| `a03_spider_form` | ✓ 已產生 |
| `a03_enchanting_venom` | ✓ 已產生 |
| `a03_insect_swarm` | ✓ 已產生 |

### A04 `takenaka_shigeharu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a04_water_shikigami` | ✓ 已產生 |
| `a04_ice_path` | ✓ 已產生 |
| `a04_frost_shelter` | ✓ 已產生 |
| `a04_frost_sparrow` | ✓ 已產生 |
| `a04_blizzard` | ✓ 已產生 |

### A05 `sakai_tadatsugu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a05_valiant_power` | ✓ 已產生 |
| `a05_mikawa_courage` | ✓ 已產生 |
| `a05_counter_stance` | ✓ 已產生 |
| `a05_loyal_secret` | ✓ 已產生 |
| `a05_crusade_affidavit` | ✓ 已產生 |

### A06 `ii_naomasa`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a06_red_power` | ✓ 已產生 |
| `a06_red_meteor` | ✓ 已產生 |
| `a06_red_flame_flow` | ✓ 已產生 |
| `a06_flame_of_anger` | ✓ 已產生 |
| `a06_red_fury` | ✓ 已產生 |

### A08 `sakakibara_yasumasa`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a08_skull_crusher` | ✓ 已產生 |
| `a08_war_cry` | ✓ 已產生 |
| `a08_indomitable` | ✓ 已產生 |
| `a08_steadfast_general` | ✓ 已產生 |

### A09 `hachisuka_masakatsu`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a09_treant_summon` | ✓ 已產生 |
| `a09_lignify` | ✓ 已產生 |
| `a09_forest_pulse` | ✓ 已產生 |
| `a09_deep_tentacles` | ✓ 已產生 |

### A10 `hirate_masahide`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a10_blink_return` | ✓ 已產生 |
| `a10_phase_shift` | ✓ 已產生 |
| `a10_slaughter_array` | ✓ 已產生 |
| `a10_swap` | ✓ 已產生 |
| `a10_shockwave` | ✓ 已產生 |

### A11 `saito_dosan`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a11_vortex` | ✓ 已產生 |
| `a11_evil_gaze` | ✓ 已產生 |
| `a11_old_fox` | ✓ 已產生 |
| `a11_myriad_forms` | ✓ 已產生 |

### A12 `otani_yoshitsugu`（9 / 9 招）

| 招式 | 還缺 |
|---|---|
| `a12_insight_flash` | ✓ 已產生 |
| `a12_scheme` | ✓ 已產生 |
| `a12_mystic_pierce` | ✓ 已產生 |
| `a12_awe` | ✓ 已產生 |
| `a12_mystic_arrow` | ✓ 已產生 |
| `a12_mystic_possession` | ✓ 已產生 |
| `a12_insight_flash_t` | ✓ 已產生 |
| `a12_mystic_field` | ✓ 已產生 |
| `a12_awe_t` | ✓ 已產生 |

### A13 `hattori_hanzo`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a13_vanishing_step` | ✓ 已產生 |
| `a13_shadow_illusion` | ✓ 已產生 |
| `a13_substitution` | ✓ 已產生 |
| `a13_chain_sickle` | ✓ 已產生 |
| `a13_iga_ninjutsu` | ✓ 已產生 |
| `a13_shadow_slash` | ✓ 已產生 |

### A14 `maeda_toshiie`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a14_impact_spear` | ✓ 已產生 |
| `a14_divine_strike` | ✓ 已產生 |
| `a14_stacked_spear` | ✓ 已產生 |
| `a14_spear_fury` | ✓ 已產生 |

### A15 `nene`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a15_shadow_step` | ✓ 已產生 |
| `a15_exhaust_seal` | ✓ 已產生 |
| `a15_moon_strike` | ✓ 已產生 |
| `a15_shadow_windmill` | ✓ 已產生 |
| `a15_shadow_moon_assault` | ✓ 已產生 |

### A16 `torii_mototada`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a16_bloody_grasp` | ✓ 已產生 |
| `a16_blood_link` | ✓ 已產生 |
| `a16_blood_aura` | ✓ 已產生 |
| `a16_dark_blood_decree` | ✓ 已產生 |

### A17 `sassa_narimasa`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a17_shrapnel` | ✓ 已產生 |
| `a17_rapid_fire` | ✓ 已產生 |
| `a17_vital_shot` | ✓ 已產生 |
| `a17_great_cannon` | ✓ 已產生 |

### A18 `inaba_ittetsu`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a18_obsessive_pressure` | ✓ 已產生 |
| `a18_suppress` | ✓ 已產生 |
| `a18_stubborn_tactics` | ✓ 已產生 |
| `a18_by_force` | ✓ 已產生 |

### A19 `kuroda_yoshitaka`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a19_fire_stratagem` | ✓ 已產生 |
| `a19_chain_fire` | ✓ 已產生 |
| `a19_fire_echo` | ✓ 已產生 |
| `a19_siege_fire` | ✓ 已產生 |

### A20 `kuki_yoshitaka`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a20_pirate_domain` | ✓ 已產生 |
| `a20_water_burst` | ✓ 已產生 |
| `a20_pirate_boots` | ✓ 已產生 |
| `a20_pirate_daimyo` | ✓ 已產生 |

### A21 `zuiryuin_nisshu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a21_mana_steal` | ✓ 已產生 |
| `a21_half_blade` | ✓ 已產生 |
| `a21_energy_press` | ✓ 已產生 |
| `a21_pearl_disc` | ✓ 已產生 |
| `a21_false_intel` | ✓ 已產生 |

### A22 `sengoku_hidehisa`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a22_flame_whirl` | ✓ 已產生 |
| `a22_flame_heart` | ✓ 已產生 |
| `a22_boiling_blood` | ✓ 已產生 |
| `a22_torment` | ✓ 已產生 |

### A23 `ukita_hideie`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a23_cursed_blood` | ✓ 已產生 |
| `a23_soul_bind` | ✓ 已產生 |
| `a23_falling_fire` | ✓ 已產生 |
| `a23_yomi_flame` | ✓ 已產生 |

### A24 `sakuma_morimasa`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a24_blood_shield` | ✓ 已產生 |
| `a24_slaughter_shot` | ✓ 已產生 |
| `a24_blood_charge` | ✓ 已產生 |
| `a24_blood_resonance` | ✓ 已產生 |
| `a24_bloodbath` | ✓ 已產生 |

### A25 `oda_nobunaga`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a25_lightning_lure` | ✓ 已產生 |
| `a25_calamity_thorns` | ✓ 已產生 |
| `a25_heshikiri` | ✓ 已產生 |
| `a25_crimson_barrier` | ✓ 已產生 |
| `a25_tenka_fubu` | ✓ 已產生 |

### A26 `nohime`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a26_volley` | ✓ 已產生 |
| `a26_powder_kick` | ✓ 已產生 |
| `a26_twin_guns` | ✓ 已產生 |
| `a26_scorching_bomb` | ✓ 已產生 |

### A27 `shibata_katsuie`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a27_unstoppable_advance` | ✓ 已產生 |
| `a27_war_banner` | ✓ 已產生 |
| `a27_way_of_warrior` | ✓ 已產生 |
| `a27_tyrant_hammer` | ✓ 已產生 |
| `a27_oni_shibata` | ✓ 已產生 |

### A28 `hashiba_hideyoshi`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a28_evil_art` | ✓ 已產生 |
| `a28_creation` | ✓ 已產生 |
| `a28_binding_swamp` | ✓ 已產生 |
| `a28_phoenix_summon` | ✓ 已產生 |
| `a28_heaven_fire` | ✓ 已產生 |
| `a28_spreading_fire` | ✓ 已產生 |

### A29 `ishida_mitsunari`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a29_sleeve_poison_arrow` | ✓ 已產生 |
| `a29_abyss_grip` | ✓ 已產生 |
| `a29_withering_aura` | ✓ 已產生 |
| `a29_vulture_swarm` | ✓ 已產生 |

### A30 `niwa_nagahide`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a30_wolf_mark` | ✓ 已產生 |
| `a30_earth_fang` | ✓ 已產生 |
| `a30_fire_sea` | ✓ 已產生 |
| `a30_red_fuji` | ✓ 已產生 |

### A31 `mori_ranmaru`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a31_echoing_rain` | ✓ 已產生 |
| `a31_shunpo` | ✓ 已產生 |
| `a31_storm_arrow` | ✓ 已產生 |
| `a31_samidare` | ✓ 已產生 |

### A32 `takigawa_kazumasu`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a32_flying_slash` | ✓ 已產生 |
| `a32_second_slash` | ✓ 已產生 |
| `a32_soul_drain` | ✓ 已產生 |
| `a32_phantom_blade` | ✓ 已產生 |
| `a32_soul_blade` | ✓ 已產生 |
| `a32_asura_blade` | ✓ 已產生 |

### A33 `honda_masanobu`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a33_dark_summoning` | ✓ 已產生 |
| `a33_ghost_step` | ✓ 已產生 |
| `a33_corrupting_aura` | ✓ 已產生 |
| `a33_deepen_wounds` | ✓ 已產生 |

### A35 `shima_sakon`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a35_flame_raid` | ✓ 已產生 |
| `a35_sealing_blaze` | ✓ 已產生 |
| `a35_flame_swiftness` | ✓ 已產生 |
| `a35_scorching_fire` | ✓ 已產生 |

### A36 `maeno_nagayasu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a36_shadow_step` | ✓ 已產生 |
| `a36_spike_trap` | ✓ 已產生 |
| `a36_detonate` | ✓ 已產生 |
| `a36_bitter_brew` | ✓ 已產生 |
| `a36_deadly_strike` | ✓ 已產生 |

### B02 `fuma_kotaro`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b02_vanishing_step` | ✓ 已產生 |
| `b02_hazy_shadow` | ✓ 已產生 |
| `b02_blazing_path` | ✓ 已產生 |
| `b02_eighteen_shuriken` | ✓ 已產生 |
| `b02_four_thunders` | ✓ 已產生 |

### B03 `saito_tomonobu`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b03_zhongkui_strike` | ✓ 已產生 |
| `b03_zhongkui_strike_mid` | ✓ 已產生 |
| `b03_zhongkui_strike_far` | ✓ 已產生 |
| `b03_echigo_vajra` | ✓ 已產生 |
| `b03_heavy_blow` | ✓ 已產生 |
| `b03_zhongkui_judgment` | ✓ 已產生 |

### B05 `mori_motonari`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b05_chain_lightning` | ✓ 已產生 |
| `b05_heavenly_thunder` | ✓ 已產生 |
| `b05_three_arrows` | ✓ 已產生 |
| `b05_holy_light_burst` | ✓ 已產生 |

### B07 `ashikaga_yoshiaki`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b07_starfall` | ✓ 已產生 |
| `b07_battle_line` | ✓ 已產生 |
| `b07e_camp_regen` | ✓ 已產生 |
| `b07_glorious_momentum` | ✓ 已產生 |
| `b07_reign_over_all` | ✓ 已產生 |

### B09 `sanada_masayuki`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b09_backlash_rift` | ✓ 已產生 |
| `b09_mind_burst` | ✓ 已產生 |
| `b09_will_shield` | ✓ 已產生 |
| `b09_mind_suppression` | ✓ 已產生 |

### B10 `miyoshi_nagayoshi`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b10_wrath_of_the_dead` | ✓ 已產生 |
| `b10_corpse_assault` | ✓ 已產生 |
| `b10_demonic_war_spirit` | ✓ 已產生 |
| `b10_power_of_the_abyss` | ✓ 已產生 |

### B11 `kikuhime`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b11_cherry_rain` | ✓ 已產生 |
| `b11_breath_control` | ✓ 已產生 |
| `b11_interception` | ✓ 已產生 |
| `b11_crow_raid` | ✓ 已產生 |
| `b11_crow_strike` | ✓ 已產生 |
| `b11_inherited_blood` | ✓ 已產生 |

### B12 `yamamoto_kansuke`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b12_reality_slide` | ✓ 已產生 |
| `b12_floating_life` | ✓ 已產生 |
| `b12_scheming_offense` | ✓ 已產生 |
| `b12_mad_heart` | ✓ 已產生 |

### B13 `momochi_sandayu`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b13_sense_break` | ✓ 已產生 |
| `b13_hypnosis` | ✓ 已產生 |
| `b13_blood_disguise` | ✓ 已產生 |
| `b13_burrow` | ✓ 已產生 |
| `b13_shukuchi` | ✓ 已產生 |
| `b13_storm` | ✓ 已產生 |

### B14 `katakura_kagetsuna`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b14_temporary_night` | ✓ 已產生 |
| `b14_iai_slash` | ✓ 已產生 |
| `b14_shadow_tactics` | ✓ 已產生 |
| `b14_attendant_insight` | ✓ 已產生 |
| `b14_shadow_cut` | ✓ 已產生 |

### B15 `imagawa_yoshimoto`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b15_tokai_giant` | ✓ 已產生 |
| `b15_flame_arrow` | ✓ 已產生 |
| `b15_giant_hide` | ✓ 已產生 |
| `b15_multishot` | ✓ 已產生 |
| `b15_bluff` | ✓ 已產生 |

### B16 `mochizuki_chiyome`（9 / 9 招）

| 招式 | 還缺 |
|---|---|
| `b16_flying_raid` | ✓ 已產生 |
| `b16_falling_leaves` | ✓ 已產生 |
| `b16_swift_thunder` | ✓ 已產生 |
| `b16_summon_hound` | ✓ 已產生 |
| `b16_phase_shift` | ✓ 已產生 |
| `b16_beast_union` | ✓ 已產生 |
| `b16_mm_phase_shift` | ✓ 已產生 |
| `b16_mm_sweep` | ✓ 已產生 |
| `b16_mm_stealth` | ✓ 已產生 |

### B17 `kirigakure_saizo`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b17_death_ward` | ✓ 已產生 |
| `b17_tooth_poison` | ✓ 已產生 |
| `b17_shadow_leap` | ✓ 已產生 |
| `b17_heartseeker` | ✓ 已產生 |

### B18 `koihime`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b18_earth_raid` | ✓ 已產生 |
| `b18_echo` | ✓ 已產生 |
| `b18_earth_shelter` | ✓ 已產生 |
| `b18_earth_claw` | ✓ 已產生 |

### B19 `usami_sadamitsu`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b19_water_dragon` | ✓ 已產生 |
| `b19_ice_fence` | ✓ 已產生 |
| `b19_echigo_tactics` | ✓ 已產生 |
| `b19_kuruma_gakari` | ✓ 已產生 |

### B20 `yamagata_masakage`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b20_rock_rush` | ✓ 已產生 |
| `b20_ruin_wave` | ✓ 已產生 |
| `b20_tenacity` | ✓ 已產生 |
| `b20_red_charge` | ✓ 已產生 |

### B21 `baba_nobufusa`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b21_sky_wave` | ✓ 已產生 |
| `b21_killing_intent` | ✓ 已產生 |
| `b21_rebel_armor` | ✓ 已產生 |
| `b21_undying_demon` | ✓ 已產生 |

### B22 `taigen_sessai`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b22_natural_barrier` | ✓ 已產生 |
| `b22_force_of_nature` | ✓ 已產生 |
| `b22_purify` | ✓ 已產生 |
| `b22_wood_spirit` | ✓ 已產生 |
| `b22_wrath_of_nature` | ✓ 已產生 |

### B23 `ise_hime`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b23_undead_pact` | ✓ 已產生 |
| `b23_spirit_gathering` | ✓ 已產生 |
| `b23_living_soul_shock` | ✓ 已產生 |
| `b23_soul_feast` | ✓ 已產生 |
| `b23_heroic_spirit_blessing` | ✓ 已產生 |

### B24 `akiyama_nobutomo`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b24_earth_stele` | ✓ 已產生 |
| `b24_taunt` | ✓ 已產生 |
| `b24_stone_resonance` | ✓ 已產生 |
| `b24_earth_prison` | ✓ 已產生 |
| `b24_shatter` | ✓ 已產生 |

### B25 `honganji_kennyo`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b25_exorcism_strike` | ✓ 已產生 |
| `b25_exorcism_sutra` | ✓ 已產生 |
| `b25_finger_of_death` | ✓ 已產生 |
| `b25_asura_hell` | ✓ 已產生 |

### B26 `okuma_tomohide`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b26_charging_wave` | ✓ 已產生 |
| `b26_blood_fury` | ✓ 已產生 |
| `b26_kozuke_valor` | ✓ 已產生 |
| `b26_unstoppable` | ✓ 已產生 |
| `b26_release_wave` | ✓ 已產生 |

### B27 `kojima_sadaoki`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b27_horsemanship` | ✓ 已產生 |
| `b27_mighty_strike` | ✓ 已產生 |
| `b27_fearless_charge` | ✓ 已產生 |
| `b27_rending_blade` | ✓ 已產生 |

### B28 `matsunaga_hisahide`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b28_phoenix_flame` | ✓ 已產生 |
| `b28_flame_shackle` | ✓ 已產生 |
| `b28_fire_stream` | ✓ 已產生 |
| `b28_serpent_fire` | ✓ 已產生 |

### B30 `kakizaki_kageie`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b30_reverse_shield` | ✓ 已產生 |
| `b30_soul_breaker` | ✓ 已產生 |
| `b30_flash_strike` | ✓ 已產生 |
| `b30_purgatory_slash` | ✓ 已產生 |

### B31 `takeda_shingen`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b31_swift_wind` | ✓ 已產生 |
| `b31_still_forest` | ✓ 已產生 |
| `b31_raging_fire` | ✓ 已產生 |
| `b31_immovable_mountain` | ✓ 已產生 |

### B32 `uesugi_kenshin`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b32_divine_slash` | ✓ 已產生 |
| `b32_divine_flash` | ✓ 已產生 |
| `b32_divine_formation` | ✓ 已產生 |
| `b32_war_god` | ✓ 已產生 |
| `b32_bishamonten` | ✓ 已產生 |

### B33 `mogami_yoshihime`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b33_corrosive_arrow` | ✓ 已產生 |
| `b33_spider_web` | ✓ 已產生 |
| `b33_voodoo_power` | ✓ 已產生 |
| `b33_spider_attack` | ✓ 已產生 |

### B34 `takeda_katsuyori`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b34_wrath_axe` | ✓ 已產生 |
| `b34_spike_wall` | ✓ 已產生 |
| `b34_frenzy` | ✓ 已產生 |
| `b34_tenmoku_howl` | ✓ 已產生 |

### B35 `kamiizumi_nobutsuna`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b35_sword_essence` | ✓ 已產生 |
| `b35_isshun` | ✓ 已產生 |
| `b35_muto_dori` | ✓ 已產生 |
| `b35_hiten` | ✓ 已產生 |

### B36 `naoe_kanetsugu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b36_naoe_seal` | ✓ 已產生 |
| `b36_footsteps` | ✓ 已產生 |
| `b36_heart_fence` | ✓ 已產生 |
| `b36_seven_spears` | ✓ 已產生 |
| `b36_aizen_myoo` | ✓ 已產生 |

### C01 `akechi_mitsuhide`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c01_blood_blade` | ✓ 已產生 |
| `c01_ground_flame` | ✓ 已產生 |
| `c01_blood_flame` | ✓ 已產生 |
| `c01_honnoji` | ✓ 已產生 |

### C02 `akechi_hidemitsu`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c02_mirage_strike` | ✓ 已產生 |
| `c02_phantom_move` | ✓ 已產生 |
| `c02_kyoka_suigetsu` | ✓ 已產生 |
| `c02_culling_slash` | ✓ 已產生 |

### C03 `shimazu_yoshihiro`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c03_iron_shirt` | ✓ 已產生 |
| `c03_quake_axe` | ✓ 已產生 |
| `c03_thunder_wrath` | ✓ 已產生 |
| `c03_iron_hammer` | ✓ 已產生 |

### C04 `izumo_no_okuni`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c04_swift_dance` | ✓ 已產生 |
| `c04_withering_step` | ✓ 已產生 |
| `c04_gathering_dance` | ✓ 已產生 |
| `c04_witch_dance` | ✓ 已產生 |

### C05 `maeda_keiji`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c05_matsukaze` | ✓ 已產生 |
| `c05_thunder_dance` | ✓ 已產生 |
| `c05_thunder_spear` | ✓ 已產生 |
| `c05_flash_thunder` | ✓ 已產生 |

### C06 `ishikawa_goemon`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `c06_quick_hands` | ✓ 已產生 |
| `c06_sleight_of_hand` | ✓ 已產生 |
| `c06_lavish_strike` | ✓ 已產生 |
| `c06_murder_and_loot` | ✓ 已產生 |
| `c06_grand_heist` | ✓ 已產生 |
| `item_c06e` | ✓ 已產生 |

### C07 `tachibana_dosetsu`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `c07_heaven_thunder` | ✓ 已產生 |
| `c07_thunder_shackle` | ✓ 已產生 |
| `c07_thunderbolt` | ✓ 已產生 |
| `c07_raijin_wrath` | ✓ 已產生 |
| `c07_storm_move` | ✓ 已產生 |
| `c07_magic_shield` | ✓ 已產生 |

### C08 `kato_danzo`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c08_vanishing_step` | ✓ 已產生 |
| `c08_blood_hunt` | ✓ 已產生 |
| `c08_scatter_escape` | ✓ 已產生 |
| `c08_kite_chain` | ✓ 已產生 |
| `c08_devour` | ✓ 已產生 |

### C09 `tachibana_ginchiyo`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c09_frost_thorn` | ✓ 已產生 |
| `c09_white_heat` | ✓ 已產生 |
| `c09_ice_soul_dance` | ✓ 已產生 |
| `c09_armor_break` | ✓ 已產生 |

### C10 `kosokabe_chikayasu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c10_black_dragon_wave` | ✓ 已產生 |
| `c10_energy_drain` | ✓ 已產生 |
| `c10_echo_sword` | ✓ 已產生 |
| `c10_day_night_shift` | ✓ 已產生 |
| `c10_formless_heaven` | ✓ 已產生 |

### C11 `tachibana_muneshige`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c11_raikiri` | ✓ 已產生 |
| `c11_thunder_speed` | ✓ 已產生 |
| `c11_thunder_eye` | ✓ 已產生 |
| `c11_unrivaled` | ✓ 已產生 |

### C12 `otomo_sorin`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c12_homing_lock` | ✓ 已產生 |
| `c12_missile_launch` | ✓ 已產生 |
| `c12_space_rift` | ✓ 已產生 |
| `c12_optical_mask` | ✓ 已產生 |
| `c12_annihilation_beam` | ✓ 已產生 |

### C13 `ryuzoji_takanobu`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c13_siphon` | ✓ 已產生 |
| `c13_tornado_wave` | ✓ 已產生 |
| `c13_gale_grip` | ✓ 已產生 |
| `c13_tornado_vortex` | ✓ 已產生 |

### C14 `saito_yoshitatsu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c14_fire_curse` | ✓ 已產生 |
| `c14_fire_breath` | ✓ 已產生 |
| `c14_fire_breath_dragon` | ✓ 已產生 |
| `c14_dragon_vein` | ✓ 已產生 |
| `c14_dragon_form` | ✓ 已產生 |

### C16 `kashin_koji`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c16_soul_sever` | ✓ 已產生 |
| `c16_curse_shadow` | ✓ 已產生 |
| `c16_wandering_souls` | ✓ 已產生 |
| `c16_phantom_sight` | ✓ 已產生 |
| `c16_eight_gates` | ✓ 已產生 |

### C18 `kobayakawa_takakage`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c18_lone_rider` | ✓ 已產生 |
| `c18_twin_chariot` | ✓ 已產生 |
| `c18_three_blades` | ✓ 已產生 |
| `c18_chain_stratagem` | ✓ 已產生 |
| `c18_four_cannons` | ✓ 已產生 |

### C19 `matsuhime`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c19_alluring_gaze` | ✓ 已產生 |
| `c19_gentle_vow` | ✓ 已產生 |
| `c19_paralyzing_poison` | ✓ 已產生 |
| `c19_surprise_raid` | ✓ 已產生 |
| `c19_inviolable` | ✓ 已產生 |

### C20 `chosokabe_motochika`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c20_flame_aegis` | ✓ 已產生 |
| `c20_demon_blood` | ✓ 已產生 |
| `c20_flame_dance` | ✓ 已產生 |
| `c20_demon_child` | ✓ 已產生 |

### C21 `miyamoto_musashi`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c21_crescent_slash` | ✓ 已產生 |
| `c21_chain_breaker` | ✓ 已產生 |
| `c21_two_heavens` | ✓ 已產生 |
| `c21_soaring_quake` | ✓ 已產生 |

### C22 `sasaki_kojiro`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c22_ganryu` | ✓ 已產生 |
| `c22_swallow_nest` | ✓ 已產生 |
| `c22_flying_swallow` | ✓ 已產生 |
| `c22_swallow_flash` | ✓ 已產生 |
| `c22_dream_mist` | ✓ 已產生 |

### C23 `kato_kiyomasa`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c23_tainted_frenzy` | ✓ 已產生 |
| `c23_malice_wall` | ✓ 已產生 |
| `c23_blood_for_blood` | ✓ 已產生 |
| `c23_gift_of_death` | ✓ 已產生 |

### C24 `yagyu_muneyoshi`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c24_kamitori` | ✓ 已產生 |
| `c24_ukifune` | ✓ 已產生 |
| `c24_zenken` | ✓ 已產生 |
| `c24_last_life` | ✓ 已產生 |

### C25 `chacha`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c25_raid` | ✓ 已產生 |
| `c25_blood_blade` | ✓ 已產生 |
| `c25_grievous_wound` | ✓ 已產生 |
| `c25_falling_sakura` | ✓ 已產生 |

## 3. 已產生的招式在執行期回 RErr 的情況

產生的程式碼遇到資料沒預期的世界狀態時，施放回 `RErr`（host 回滾成本）。下表是已產生的招式裡各種情況出現的招數；未產生的招式是 stub，一律回 `RErr` 並列出 §2 的缺項。

| 情況 | 招數 |
|---|---|
| 施放檢查：rank、施法者存活、目標存活／隊伍／類別、施法距離（同 IR） | 373 |
| 召喚：附近放不下、lifetime 不在 0～600 秒 | 10 |
| 召喚：count 不是 1 | 1 |
| 視野：半徑或位置不合法 | 24 |
| 光鍊：host tether 規格不合法 | 1 |
| 友方回復：比例不在 0～1、或不在施法者周圍時沒有施法距離 | 1 |
| 運算式除以零 | 28 |
| 讀位置的單位已不在世界中 | 240 |
| 需要地點，但施放沒有地點也沒有單位目標 | 102 |

### 已產生但 host 沒有做到的地方

產生器不默默略過欄位：不支援的一律擋下（上面各節）；下面這些已產生，但文件寫的某件事 host 目前做不到，照原樣列出。

| 招式 | 沒有做到 |
|---|---|
| `c06_grand_heist` | create_pickup.cue: the host pickup draws its own bag |

## 4. 每項寫法擋住多少

| 寫法 | 主要負責 | 分類 | 招式 | 道具 | 英雄 |
|---|---|---|---|---|---|
| `action:cancel_cast outside on_phase_start` | ability_kit | 共用 | 0 | 2 | 0 |
| `action:cue.at of a err` | ability_kit | 共用 | 0 | 2 | 0 |
| `action:damage.type:event` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:issue_order.ability` | ability_kit | 共用 | 0 | 2 | 0 |
| `action:issue_order.order:cast` | ability_kit | 共用 | 0 | 2 | 0 |
| `action:modify_cooldown.ability:event.order.ability` | ability_kit | 共用 | 0 | 2 | 0 |
| `action:projectile.lifetime` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:refund of mana or health` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:remove_status.exclude_source_tags` | ability_kit | 共用 | 0 | 3 | 0 |
| `action:remove_status.min_duration` | ability_kit | 共用 | 0 | 3 | 0 |
| `action:remove_status.source_is` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:spend_mana.ability:event.order.ability` | ability_kit | 共用 | 0 | 2 | 0 |
| `attr:game.creep_armor_bonus` | 產生器 | 共用 | 0 | 4 | 0 |
| `attr:game.creep_wave` | 產生器 | 共用 | 0 | 4 | 0 |
| `attr:illusion_damage` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `attr:is_ready` | 產生器 | 共用 | 0 | 3 | 0 |
| `attr:is_summon` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `aura.types:building+hero` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `aura.unique_group` | ability_kit | 共用 | 0 | 4 | 0 |
| `catalog: ability ninja1_flash is not a content ability (templates/abilities.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: ability ninja2_burrow is not a content ability (templates/abilities.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ancient_doll is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_big_tachi is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_brave_ring is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_flash_ring is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_flash_shoes is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_flood_book is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_frozen_ring is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_fubuki_shoulders is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ghost_gloves is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ghost_mask is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_gold_to_prestige is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_hachiman_bell is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_healing_totem is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ice_book is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_kat_steel is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_kisyosennfuyuzuki is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ksitigarbha is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_lesser_armor is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_lightning_scroll is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_logs_big_tachi is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_magic_ring is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_mountain_scroll is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_nagamaki is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_nannbann_armor is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ninja1 is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ninja2 is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ninja_sword is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_poisonous_ring is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_sakabatou is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_samurai_cloth is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_satsuma_gun is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_shintogo_kunimitsu is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_storm_book is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_sukezane is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_club_of_nebula is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_great_sword_of_anger is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_great_sword_of_disease is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_great_sword_of_hurricane is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_great_sword_of_spike is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_great_sword_of_tiger is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_overflame_art_of_war is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_red_lightning_chapter is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_red_lightning_chapter_hyper is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_storm_call_book is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_sword_of_kaminaobi is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_thunderstorms is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_toughness_ingot is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_true_katana is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_wayumi is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_wayumi2 is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_wind_scroll is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_yubashiri is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `event.order.target` | ability_kit | 共用 | 0 | 2 | 0 |
| `event:on_owner_death` | ability_kit | 共用 | 0 | 3 | 0 |
| `event:on_unequip` | ability_kit | 共用 | 0 | 2 | 0 |
| `fn:has_tag:courier of a unit` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `fn:has_tag:no_cast_trigger` | 產生器 | 共用 | 0 | 2 | 0 |
| `fn:has_tag:ultimate` | 產生器 | 共用 | 0 | 2 | 0 |
| `fn:is_hero_id` | 產生器 | 共用 | 0 | 7 | 0 |
| `fn:min of non-numbers` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `item.active.cast.interruptible` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `item.active.cost.charges` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `item.active.kind:passive` | ability_kit | 共用 | 0 | 3 | 0 |
| `item.active.kind:toggle` | ability_kit | 共用 | 0 | 2 | 0 |
| `passive:aura of an item` | ability_kit | 共用 | 0 | 22 | 0 |
| `passive:orb_slot (only a toggle that claims and releases at status_end)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `passive_trigger:on_ability_executed of an item (no learn marker)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `passive_trigger:on_death of an item (no learn marker)` | ability_kit | 共用 | 0 | 2 | 0 |
| `passive_trigger:on_respawn of an item (no learn marker)` | ability_kit | 共用 | 0 | 2 | 0 |
| `rust_handler without behavior` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `selector.exclude_types` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `selector.types:building+hero+tower` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `stat.when` | ability_kit | 共用 | 0 | 5 | 0 |
| `stat.when:not primary_is(self, agi)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `stat.when:not primary_is(self, int)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `stat.when:not primary_is(self, str)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `stat:all_stats` | ability_kit | 共用 | 0 | 9 | 0 |
| `stat:cooldown_reduction_pct` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `stat:physical_block_chance` | ability_kit | 共用 | 0 | 2 | 0 |
| `state type:bool (item_instance)` | ability_kit | 共用 | 0 | 2 | 0 |
| `state type:unit (item_instance)` | ability_kit | 共用 | 0 | 2 | 0 |
| `status.absorb on an independent status` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `status.absorb on this apply form` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `status.absorb.phase:pre_mitigation` | ability_kit | 共用 | 0 | 2 | 0 |
| `status.scope:item_instance` | ability_kit | 共用 | 0 | 26 | 0 |
| `status.unique_rule:latest` | ability_kit | 共用 | 0 | 9 | 0 |
| `status.vars type:ability` | ability_kit | 共用 | 0 | 2 | 0 |
| `status_trigger:on_order` | ability_kit | 共用 | 0 | 2 | 0 |
| `target.types:building+courier+creep+neutral+summon` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `token:vars (spell outside its status)` | 產生器 | 共用 | 0 | 2 | 0 |
| `unit_template blocks_pathing without collision_radius` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `unit_template passive.aura` | ability_kit | 共用 | 0 | 2 | 0 |

## 5. hook 候選

只有 1 招（英雄技能與道具合計）用到的寫法。這些招式在 v3 標 `hook = "<方法名>"`、寫明行為描述與預期結果，由產生的 trait 手寫實作（`src/hooks/<招式>.rs`）；第 2 招也用到時改成共用、刪掉 hook。

| 寫法 | 招式 | 所屬 |
|---|---|---|
| `catalog: item item_ancient_doll is not in the shop (templates/moba_items.lua)` | `item_ancient_doll` | 道具 |
| `catalog: item item_big_tachi is not in the shop (templates/moba_items.lua)` | `item_big_tachi` | 道具 |
| `passive_trigger:on_ability_executed of an item (no learn marker)` | `item_bizennosafunekanemitsu` | 道具 |
| `catalog: item item_brave_ring is not in the shop (templates/moba_items.lua)` | `item_brave_ring` | 道具 |
| `attr:is_summon` | `item_commander_of_fan` | 道具 |
| `selector.types:building+hero+tower` | `item_contemporary_armor` | 道具 |
| `status.absorb on this apply form` | `item_devil_supressor_armor` | 道具 |
| `item.active.cost.charges` | `item_earth_wall_book` | 道具 |
| `unit_template blocks_pathing without collision_radius` | `item_earth_wall_book` | 道具 |
| `catalog: item item_flash_ring is not in the shop (templates/moba_items.lua)` | `item_flash_ring` | 道具 |
| `catalog: item item_flash_shoes is not in the shop (templates/moba_items.lua)` | `item_flash_shoes` | 道具 |
| `catalog: item item_flood_book is not in the shop (templates/moba_items.lua)` | `item_flood_book` | 道具 |
| `catalog: item item_frozen_ring is not in the shop (templates/moba_items.lua)` | `item_frozen_ring` | 道具 |
| `catalog: item item_fubuki_shoulders is not in the shop (templates/moba_items.lua)` | `item_fubuki_shoulders` | 道具 |
| `catalog: item item_ghost_gloves is not in the shop (templates/moba_items.lua)` | `item_ghost_gloves` | 道具 |
| `catalog: item item_ghost_mask is not in the shop (templates/moba_items.lua)` | `item_ghost_mask` | 道具 |
| `catalog: item item_gold_to_prestige is not in the shop (templates/moba_items.lua)` | `item_gold_to_prestige` | 道具 |
| `catalog: item item_hachiman_bell is not in the shop (templates/moba_items.lua)` | `item_hachiman_bell` | 道具 |
| `rust_handler without behavior` | `item_hama_orb` | 道具 |
| `catalog: item item_healing_totem is not in the shop (templates/moba_items.lua)` | `item_healing_totem` | 道具 |
| `catalog: item item_ice_book is not in the shop (templates/moba_items.lua)` | `item_ice_book` | 道具 |
| `action:remove_status.source_is` | `item_ice_storm_book` | 道具 |
| `aura.types:building+hero` | `item_insight_gem_hero` | 道具 |
| `catalog: item item_kat_steel is not in the shop (templates/moba_items.lua)` | `item_kat_steel` | 道具 |
| `catalog: item item_kisyosennfuyuzuki is not in the shop (templates/moba_items.lua)` | `item_kisyosennfuyuzuki` | 道具 |
| `catalog: item item_ksitigarbha is not in the shop (templates/moba_items.lua)` | `item_ksitigarbha` | 道具 |
| `catalog: item item_lesser_armor is not in the shop (templates/moba_items.lua)` | `item_lesser_armor` | 道具 |
| `catalog: item item_lightning_scroll is not in the shop (templates/moba_items.lua)` | `item_lightning_scroll` | 道具 |
| `catalog: item item_logs_big_tachi is not in the shop (templates/moba_items.lua)` | `item_logs_big_tachi` | 道具 |
| `catalog: item item_magic_ring is not in the shop (templates/moba_items.lua)` | `item_magic_ring` | 道具 |
| `status.absorb on an independent status` | `item_magic_talisman` | 道具 |
| `attr:illusion_damage` | `item_mana_thief` | 道具 |
| `catalog: item item_mountain_scroll is not in the shop (templates/moba_items.lua)` | `item_mountain_scroll` | 道具 |
| `catalog: item item_nagamaki is not in the shop (templates/moba_items.lua)` | `item_nagamaki` | 道具 |
| `catalog: item item_nannbann_armor is not in the shop (templates/moba_items.lua)` | `item_nannbann_armor` | 道具 |
| `selector.exclude_types` | `item_napalm_bomb` | 道具 |
| `catalog: item item_ninja1 is not in the shop (templates/moba_items.lua)` | `item_ninja1` | 道具 |
| `catalog: item item_ninja2 is not in the shop (templates/moba_items.lua)` | `item_ninja2` | 道具 |
| `catalog: item item_ninja_sword is not in the shop (templates/moba_items.lua)` | `item_ninja_sword` | 道具 |
| `catalog: item item_poisonous_ring is not in the shop (templates/moba_items.lua)` | `item_poisonous_ring` | 道具 |
| `action:projectile.lifetime` | `item_ronin_jitte` | 道具 |
| `catalog: item item_sakabatou is not in the shop (templates/moba_items.lua)` | `item_sakabatou` | 道具 |
| `catalog: item item_samurai_cloth is not in the shop (templates/moba_items.lua)` | `item_samurai_cloth` | 道具 |
| `catalog: item item_satsuma_gun is not in the shop (templates/moba_items.lua)` | `item_satsuma_gun` | 道具 |
| `catalog: item item_shintogo_kunimitsu is not in the shop (templates/moba_items.lua)` | `item_shintogo_kunimitsu` | 道具 |
| `action:refund of mana or health` | `item_soul_adder` | 道具 |
| `catalog: item item_storm_book is not in the shop (templates/moba_items.lua)` | `item_storm_book` | 道具 |
| `catalog: item item_sukezane is not in the shop (templates/moba_items.lua)` | `item_sukezane` | 道具 |
| `catalog: item item_the_club_of_nebula is not in the shop (templates/moba_items.lua)` | `item_the_club_of_nebula` | 道具 |
| `action:damage.type:event` | `item_the_dark_yellow_teeth_and_zimbabwe` | 道具 |
| `catalog: item item_the_great_sword_of_anger is not in the shop (templates/moba_items.lua)` | `item_the_great_sword_of_anger` | 道具 |
| `catalog: item item_the_great_sword_of_disease is not in the shop (templates/moba_items.lua)` | `item_the_great_sword_of_disease` | 道具 |
| `catalog: item item_the_great_sword_of_hurricane is not in the shop (templates/moba_items.lua)` | `item_the_great_sword_of_hurricane` | 道具 |
| `stat.when:not primary_is(self, agi)` | `item_the_great_sword_of_invincible` | 道具 |
| `stat.when:not primary_is(self, int)` | `item_the_great_sword_of_invincible` | 道具 |
| `stat.when:not primary_is(self, str)` | `item_the_great_sword_of_invincible` | 道具 |
| `catalog: item item_the_great_sword_of_spike is not in the shop (templates/moba_items.lua)` | `item_the_great_sword_of_spike` | 道具 |
| `fn:has_tag:courier of a unit` | `item_the_great_sword_of_teleport` | 道具 |
| `item.active.cast.interruptible` | `item_the_great_sword_of_teleport` | 道具 |
| `target.types:building+courier+creep+neutral+summon` | `item_the_great_sword_of_teleport` | 道具 |
| `catalog: item item_the_great_sword_of_tiger is not in the shop (templates/moba_items.lua)` | `item_the_great_sword_of_tiger` | 道具 |
| `stat:cooldown_reduction_pct` | `item_the_over_lightning_art_of_war` | 道具 |
| `catalog: item item_the_overflame_art_of_war is not in the shop (templates/moba_items.lua)` | `item_the_overflame_art_of_war` | 道具 |
| `catalog: item item_the_red_lightning_chapter is not in the shop (templates/moba_items.lua)` | `item_the_red_lightning_chapter` | 道具 |
| `catalog: item item_the_red_lightning_chapter_hyper is not in the shop (templates/moba_items.lua)` | `item_the_red_lightning_chapter_hyper` | 道具 |
| `fn:min of non-numbers` | `item_the_spear_of_ghost` | 道具 |
| `passive:orb_slot (only a toggle that claims and releases at status_end)` | `item_the_spear_of_saddle` | 道具 |
| `catalog: item item_the_storm_call_book is not in the shop (templates/moba_items.lua)` | `item_the_storm_call_book` | 道具 |
| `catalog: item item_the_sword_of_kaminaobi is not in the shop (templates/moba_items.lua)` | `item_the_sword_of_kaminaobi` | 道具 |
| `catalog: item item_thunderstorms is not in the shop (templates/moba_items.lua)` | `item_thunderstorms` | 道具 |
| `catalog: item item_toughness_ingot is not in the shop (templates/moba_items.lua)` | `item_toughness_ingot` | 道具 |
| `catalog: item item_true_katana is not in the shop (templates/moba_items.lua)` | `item_true_katana` | 道具 |
| `catalog: item item_wayumi is not in the shop (templates/moba_items.lua)` | `item_wayumi` | 道具 |
| `catalog: item item_wayumi2 is not in the shop (templates/moba_items.lua)` | `item_wayumi2` | 道具 |
| `catalog: item item_wind_scroll is not in the shop (templates/moba_items.lua)` | `item_wind_scroll` | 道具 |
| `catalog: item item_yubashiri is not in the shop (templates/moba_items.lua)` | `item_yubashiri` | 道具 |
| `catalog: ability ninja1_flash is not a content ability (templates/abilities.lua)` | `ninja1_flash` | 道具 |
| `catalog: ability ninja2_burrow is not a content ability (templates/abilities.lua)` | `ninja2_burrow` | 道具 |

共 78 種寫法、73 招。

## 重跑

```
tools\lua\lua.exe docs\character_pipeline\tools\v3_codegen_coverage.lua
```

