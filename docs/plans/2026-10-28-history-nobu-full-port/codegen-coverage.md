# 產生器還沒產生的寫法與實作優先序

由 `tools\lua\lua.exe docs\character_pipeline\tools\v3_codegen_coverage.lua` 產生，不要手改。取代 [ir-blockers.md](ir-blockers.md)（M3' ③，[m3-rust-codegen-design.md](m3-rust-codegen-design.md) §2.3）。產生器（`v3_rust.lua`）或 `ability_kit` 每補一批寫法就重跑。

- 英雄技能：868 招中 868 招產生成 Rust。還不能產生的招式不輸出程式碼，列在這份報告；該英雄不進 `hero_dlls`。全部技能都產生的英雄：192 / 192 位。
- 道具（items.dll，M7）：170 件中 121 件產生成 Rust。
- 一位英雄要全部技能都產生才算解鎖。順序是 M5（已上架的 10 位），然後 M6 第 1～9 批（[m6-waves.md](m6-waves.md)）；每批內先做「每多支援一項、能解鎖最多英雄」的那位英雄缺的項目。
- 「主要負責」是起點：`產生器` 是運算式、選取、目標這類產生器自己能降低的寫法；`ability_kit` 是要先有 kit 呼叫或 host 元件的寫法（工作線 B）。
- 「分類」（D7，m3-rust-codegen-design.md §2.4）：英雄技能與道具合計 2 招以上用到的寫法是 `共用`，做成 v3 詞彙＋產生器＋`ability_kit`；只有 1 招用到的是 `hook 候選`，在 v3 標 `hook` 並手寫產生的 trait（§5 的清單）。

## 1. 實作順序

### M5（已上架的 10 位英雄：B01、C17 換掉 IR，其餘換掉 Rust handler）（10 位）

開始時已全部產生：10 位；做完本批的項目後：10 位。

（本批不需要新的支援）

### M6 第 1 批（12 位）

開始時已全部產生：12 位；做完本批的項目後：12 位。

（本批不需要新的支援）

### M6 第 2 批（9 位）

開始時已全部產生：9 位；做完本批的項目後：9 位。

（本批不需要新的支援）

### M6 第 3 批（10 位）

開始時已全部產生：10 位；做完本批的項目後：10 位。

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

### M6 第 8 批（12 位）

開始時已全部產生：12 位；做完本批的項目後：12 位。

（本批不需要新的支援）

### M6 第 9 批（10 位）

開始時已全部產生：10 位；做完本批的項目後：10 位。

（本批不需要新的支援）

### M6 第 10 批（9 位）

開始時已全部產生：9 位；做完本批的項目後：9 位。

（本批不需要新的支援）

### M6 第 11 批（14 位）

開始時已全部產生：14 位；做完本批的項目後：14 位。

（本批不需要新的支援）

### M6 第 12 批（12 位）

開始時已全部產生：12 位；做完本批的項目後：12 位。

（本批不需要新的支援）

### M6 第 13 批（9 位）

開始時已全部產生：9 位；做完本批的項目後：9 位。

（本批不需要新的支援）

### M6 第 14 批（9 位）

開始時已全部產生：9 位；做完本批的項目後：9 位。

（本批不需要新的支援）

### M6 第 15 批（9 位）

開始時已全部產生：9 位；做完本批的項目後：9 位。

（本批不需要新的支援）

### M6 第 16 批（10 位）

開始時已全部產生：10 位；做完本批的項目後：10 位。

（本批不需要新的支援）

### M6 第 17 批（9 位）

開始時已全部產生：9 位；做完本批的項目後：9 位。

（本批不需要新的支援）

### M6 第 18 批（7 位）

開始時已全部產生：7 位；做完本批的項目後：7 位。

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

### A01_old `tokugawa_ieyasu_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a01_old_tosho_gongen` | ✓ 已產生 |
| `a01_old_heaven_net` | ✓ 已產生 |
| `a01_old_thunder_blade` | ✓ 已產生 |
| `a01_old_mikawa_valor` | ✓ 已產生 |

### A02_old `inahime_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a02_old_venom_arrow` | ✓ 已產生 |
| `a02_old_wind_power` | ✓ 已產生 |
| `a02_old_self_recovery` | ✓ 已產生 |
| `a02_old_shadow_meld` | ✓ 已產生 |

### A03 `ishikawa_kazumasa`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a03_pacing_gu` | ✓ 已產生 |
| `a03_spider_form` | ✓ 已產生 |
| `a03_enchanting_venom` | ✓ 已產生 |
| `a03_insect_swarm` | ✓ 已產生 |

### A03_old `ishikawa_kazumasa_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a03_old_parasite` | ✓ 已產生 |
| `a03_old_magic_armor` | ✓ 已產生 |
| `a03_old_enchanting_venom` | ✓ 已產生 |
| `a03_old_giant_gu` | ✓ 已產生 |
| `a03_old_deer_stomp` | ✓ 已產生 |

### A04 `takenaka_shigeharu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a04_water_shikigami` | ✓ 已產生 |
| `a04_ice_path` | ✓ 已產生 |
| `a04_frost_shelter` | ✓ 已產生 |
| `a04_frost_sparrow` | ✓ 已產生 |
| `a04_blizzard` | ✓ 已產生 |

### A04_old `takenaka_shigeharu_old`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a04_old_water_shikigami` | ✓ 已產生 |
| `a04_old_freezing_path` | ✓ 已產生 |
| `a04_old_frost_attack` | ✓ 已產生 |
| `a04_old_ice_blast` | ✓ 已產生 |
| `a04_old_lone_strategist` | ✓ 已產生 |
| `a04_old_blizzard` | ✓ 已產生 |

### A05 `sakai_tadatsugu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a05_valiant_power` | ✓ 已產生 |
| `a05_mikawa_courage` | ✓ 已產生 |
| `a05_counter_stance` | ✓ 已產生 |
| `a05_loyal_secret` | ✓ 已產生 |
| `a05_crusade_affidavit` | ✓ 已產生 |

### A05_old `sakai_tadatsugu_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a05_old_mikawa_valor` | ✓ 已產生 |
| `a05_old_warrior_way` | ✓ 已產生 |
| `a05_old_mighty_blow` | ✓ 已產生 |
| `a05_old_loyal_secret` | ✓ 已產生 |

### A06 `ii_naomasa`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a06_red_power` | ✓ 已產生 |
| `a06_red_meteor` | ✓ 已產生 |
| `a06_red_flame_flow` | ✓ 已產生 |
| `a06_flame_of_anger` | ✓ 已產生 |
| `a06_red_fury` | ✓ 已產生 |

### A06_old `ii_naomasa_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a06_old_red_demon_power` | ✓ 已產生 |
| `a06_old_red_flame_slash` | ✓ 已產生 |
| `a06_old_war_spirit` | ✓ 已產生 |
| `a06_old_red_demon_fury` | ✓ 已產生 |

### A07_old `honda_tadakatsu_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a07_old_unscathed` | ✓ 已產生 |
| `a07_old_red_thunder` | ✓ 已產生 |
| `a07_old_shattering_spear` | ✓ 已產生 |
| `a07_old_mightiest_awakening` | ✓ 已產生 |

### A07_v20 `honda_tadakatsu_v20`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a07_v20_unscathed` | ✓ 已產生 |
| `a07_v20_red_thunder` | ✓ 已產生 |
| `a07_v20_thunder_burst` | ✓ 已產生 |
| `a07_v20_shattering_spear` | ✓ 已產生 |
| `a07_v20_mightiest_awakening` | ✓ 已產生 |
| `a07_v20_switch_strike` | ✓ 已產生 |

### A08 `sakakibara_yasumasa`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a08_skull_crusher` | ✓ 已產生 |
| `a08_war_cry` | ✓ 已產生 |
| `a08_indomitable` | ✓ 已產生 |
| `a08_steadfast_general` | ✓ 已產生 |

### A08_old `sakakibara_yasumasa_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a08_old_frenzy` | ✓ 已產生 |
| `a08_old_roar` | ✓ 已產生 |
| `a08_old_strength` | ✓ 已產生 |
| `a08_old_steadfast_general` | ✓ 已產生 |

### A09 `hachisuka_masakatsu`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a09_treant_summon` | ✓ 已產生 |
| `a09_lignify` | ✓ 已產生 |
| `a09_forest_pulse` | ✓ 已產生 |
| `a09_deep_tentacles` | ✓ 已產生 |

### A09_old `hachisuka_masakatsu_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a09_old_carrion_swarm` | ✓ 已產生 |
| `a09_old_lignify` | ✓ 已產生 |
| `a09_old_black_tortoise` | ✓ 已產生 |
| `a09_old_growing_vines` | ✓ 已產生 |

### A10 `hirate_masahide`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a10_blink_return` | ✓ 已產生 |
| `a10_phase_shift` | ✓ 已產生 |
| `a10_slaughter_array` | ✓ 已產生 |
| `a10_swap` | ✓ 已產生 |
| `a10_shockwave` | ✓ 已產生 |

### A10_old `hirate_masahide_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a10_old_swap` | ✓ 已產生 |
| `a10_old_flying_stones` | ✓ 已產生 |
| `a10_old_bad_breath` | ✓ 已產生 |
| `a10_old_feint` | ✓ 已產生 |

### A11 `saito_dosan`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a11_vortex` | ✓ 已產生 |
| `a11_evil_gaze` | ✓ 已產生 |
| `a11_old_fox` | ✓ 已產生 |
| `a11_myriad_forms` | ✓ 已產生 |

### A11_old `saito_dosan_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a11_old_vortex` | ✓ 已產生 |
| `a11_old_viper_chain` | ✓ 已產生 |
| `a11_old_cunning` | ✓ 已產生 |
| `a11_old_evil_wind` | ✓ 已產生 |

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

### A13_old `hattori_hanzo_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a13_old_wind_step` | ✓ 已產生 |
| `a13_old_chain_sickle` | ✓ 已產生 |
| `a13_old_shadow_clone` | ✓ 已產生 |
| `a13_old_hidden_slash` | ✓ 已產生 |

### A14 `maeda_toshiie`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a14_impact_spear` | ✓ 已產生 |
| `a14_divine_strike` | ✓ 已產生 |
| `a14_stacked_spear` | ✓ 已產生 |
| `a14_spear_fury` | ✓ 已產生 |

### A14_old `maeda_toshiie_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a14_old_instinct_strike` | ✓ 已產生 |
| `a14_old_wild_slash` | ✓ 已產生 |
| `a14_old_war_spirit` | ✓ 已產生 |
| `a14_old_fire_phoenix` | ✓ 已產生 |
| `a14_old_rage_spear` | ✓ 已產生 |

### A15 `nene`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a15_shadow_step` | ✓ 已產生 |
| `a15_exhaust_seal` | ✓ 已產生 |
| `a15_moon_strike` | ✓ 已產生 |
| `a15_shadow_windmill` | ✓ 已產生 |
| `a15_shadow_moon_assault` | ✓ 已產生 |

### A15_old `nene_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a15_old_vanishing_step` | ✓ 已產生 |
| `a15_old_moon_watch` | ✓ 已產生 |
| `a15_old_shadow_windmill` | ✓ 已產生 |
| `a15_old_nightmare` | ✓ 已產生 |

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

### A17_old `sassa_narimasa_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a17_old_sulfur_shot` | ✓ 已產生 |
| `a17_old_rapid_cannon` | ✓ 已產生 |
| `a17_old_rain_guns` | ✓ 已產生 |
| `a17_old_great_cannon` | ✓ 已產生 |

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

### A19_old `kuroda_yoshitaka_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a19_old_fire_stratagem` | ✓ 已產生 |
| `a19_old_chain_fire` | ✓ 已產生 |
| `a19_old_clinging_flame` | ✓ 已產生 |
| `a19_old_siege_fire` | ✓ 已產生 |

### A19_v20 `kuroda_yoshitaka_v20`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a19_v20_fire_stratagem` | ✓ 已產生 |
| `a19_v20_chain_fire` | ✓ 已產生 |
| `a19_v20_fire_echo` | ✓ 已產生 |
| `a19_v20_siege_fire` | ✓ 已產生 |
| `a19_v20_clinging_flame` | ✓ 已產生 |

### A20 `kuki_yoshitaka`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a20_pirate_domain` | ✓ 已產生 |
| `a20_water_burst` | ✓ 已產生 |
| `a20_pirate_boots` | ✓ 已產生 |
| `a20_pirate_daimyo` | ✓ 已產生 |

### A20_old `kuki_yoshitaka_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a20_old_sea_devour` | ✓ 已產生 |
| `a20_old_pirate_domain` | ✓ 已產生 |
| `a20_old_pirate_armor` | ✓ 已產生 |
| `a20_old_seasick_frenzy` | ✓ 已產生 |

### A21 `zuiryuin_nisshu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a21_mana_steal` | ✓ 已產生 |
| `a21_half_blade` | ✓ 已產生 |
| `a21_energy_press` | ✓ 已產生 |
| `a21_pearl_disc` | ✓ 已產生 |
| `a21_false_intel` | ✓ 已產生 |

### A21_old `zuiryuin_nisshu_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a21_old_will_corruption` | ✓ 已產生 |
| `a21_old_blood_barrier` | ✓ 已產生 |
| `a21_old_pearl_disc` | ✓ 已產生 |
| `a21_old_decay_aura` | ✓ 已產生 |
| `a21_old_false_intel` | ✓ 已產生 |

### A22 `sengoku_hidehisa`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a22_flame_whirl` | ✓ 已產生 |
| `a22_flame_heart` | ✓ 已產生 |
| `a22_boiling_blood` | ✓ 已產生 |
| `a22_torment` | ✓ 已產生 |

### A22_old `sengoku_hidehisa_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a22_old_taunt` | ✓ 已產生 |
| `a22_old_tiger_swing` | ✓ 已產生 |
| `a22_old_counterattack` | ✓ 已產生 |
| `a22_old_swordsmanship` | ✓ 已產生 |
| `a22_old_cyclone_slash` | ✓ 已產生 |

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

### A25_old `oda_nobunaga_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a25_old_distant_thunder` | ✓ 已產生 |
| `a25_old_calamity_thorns` | ✓ 已產生 |
| `a25_old_heshikiri` | ✓ 已產生 |
| `a25_old_crimson_barrier` | ✓ 已產生 |

### A26 `nohime`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a26_volley` | ✓ 已產生 |
| `a26_powder_kick` | ✓ 已產生 |
| `a26_twin_guns` | ✓ 已產生 |
| `a26_scorching_bomb` | ✓ 已產生 |

### A26_old `nohime_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a26_old_four_shot` | ✓ 已產生 |
| `a26_old_shamisen_fire` | ✓ 已產生 |
| `a26_old_twin_guns` | ✓ 已產生 |
| `a26_old_one_shot` | ✓ 已產生 |

### A26_v20 `nohime_v20`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a26_v20_volley` | ✓ 已產生 |
| `a26_v20_bullet_storm` | ✓ 已產生 |
| `a26_v20_twin_guns` | ✓ 已產生 |
| `a26_v20_scorching_throw` | ✓ 已產生 |
| `a26_v20_hidden_mine` | ✓ 已產生 |

### A27 `shibata_katsuie`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a27_unstoppable_advance` | ✓ 已產生 |
| `a27_war_banner` | ✓ 已產生 |
| `a27_way_of_warrior` | ✓ 已產生 |
| `a27_tyrant_hammer` | ✓ 已產生 |
| `a27_oni_shibata` | ✓ 已產生 |

### A27_old `shibata_katsuie_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a27_old_oni_shibata` | ✓ 已產生 |
| `a27_old_earth_hammer` | ✓ 已產生 |
| `a27_old_way_of_warrior` | ✓ 已產生 |
| `a27_old_split_bamboo` | ✓ 已產生 |

### A28 `hashiba_hideyoshi`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a28_evil_art` | ✓ 已產生 |
| `a28_creation` | ✓ 已產生 |
| `a28_binding_swamp` | ✓ 已產生 |
| `a28_phoenix_summon` | ✓ 已產生 |
| `a28_heaven_fire` | ✓ 已產生 |
| `a28_spreading_fire` | ✓ 已產生 |

### A28_old `hashiba_hideyoshi_old`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a28_old_evil_art` | ✓ 已產生 |
| `a28_old_healing_wind` | ✓ 已產生 |
| `a28_old_fire_bomb` | ✓ 已產生 |
| `a28_old_kagutsuchi` | ✓ 已產生 |
| `a28_old_soul_burn` | ✓ 已產生 |
| `a28_old_blazing_body` | ✓ 已產生 |

### A28_v20 `hashiba_hideyoshi_v20`（7 / 7 招）

| 招式 | 還缺 |
|---|---|
| `a28_v20_evil_art` | ✓ 已產生 |
| `a28_v20_creation` | ✓ 已產生 |
| `a28_v20_binding_swamp` | ✓ 已產生 |
| `a28_v20_phoenix_summon` | ✓ 已產生 |
| `a28_v20_phoenix_bond` | ✓ 已產生 |
| `a28_v20_heaven_fire` | ✓ 已產生 |
| `a28_v20_blazing_body` | ✓ 已產生 |

### A29 `ishida_mitsunari`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a29_sleeve_poison_arrow` | ✓ 已產生 |
| `a29_abyss_grip` | ✓ 已產生 |
| `a29_withering_aura` | ✓ 已產生 |
| `a29_vulture_swarm` | ✓ 已產生 |

### A29_old `ishida_mitsunari_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a29_old_poison_arrow` | ✓ 已產生 |
| `a29_old_entangling_roots` | ✓ 已產生 |
| `a29_old_wit_drain` | ✓ 已產生 |
| `a29_old_vulture_swarm` | ✓ 已產生 |

### A30 `niwa_nagahide`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a30_wolf_mark` | ✓ 已產生 |
| `a30_earth_fang` | ✓ 已產生 |
| `a30_fire_sea` | ✓ 已產生 |
| `a30_red_fuji` | ✓ 已產生 |

### A30_old `niwa_nagahide_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a30_old_wolf_call` | ✓ 已產生 |
| `a30_old_earth_fang` | ✓ 已產生 |
| `a30_old_fire_sea` | ✓ 已產生 |
| `a30_old_red_fuji` | ✓ 已產生 |

### A31 `mori_ranmaru`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a31_echoing_rain` | ✓ 已產生 |
| `a31_shunpo` | ✓ 已產生 |
| `a31_storm_arrow` | ✓ 已產生 |
| `a31_samidare` | ✓ 已產生 |

### A31_old `mori_ranmaru_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a31_old_thunder_rain` | ✓ 已產生 |
| `a31_old_shunpo` | ✓ 已產生 |
| `a31_old_storm_arrow` | ✓ 已產生 |
| `a31_old_samidare` | ✓ 已產生 |

### A31_v20 `mori_ranmaru_v20`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a31_v20_echoing_rain` | ✓ 已產生 |
| `a31_v20_shunpo` | ✓ 已產生 |
| `a31_v20_storm_arrow` | ✓ 已產生 |
| `a31_v20_samidare` | ✓ 已產生 |
| `a31_v20_cloud_watch` | ✓ 已產生 |

### A32 `takigawa_kazumasu`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a32_flying_slash` | ✓ 已產生 |
| `a32_second_slash` | ✓ 已產生 |
| `a32_soul_drain` | ✓ 已產生 |
| `a32_phantom_blade` | ✓ 已產生 |
| `a32_soul_blade` | ✓ 已產生 |
| `a32_asura_blade` | ✓ 已產生 |

### A32_old `takigawa_kazumasu_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a32_old_flying_slash` | ✓ 已產生 |
| `a32_old_banish` | ✓ 已產生 |
| `a32_old_phantom_blade` | ✓ 已產生 |
| `a32_old_soul_blade` | ✓ 已產生 |

### A33 `honda_masanobu`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a33_dark_summoning` | ✓ 已產生 |
| `a33_ghost_step` | ✓ 已產生 |
| `a33_corrupting_aura` | ✓ 已產生 |
| `a33_deepen_wounds` | ✓ 已產生 |

### A33_old `honda_masanobu_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a33_old_demon_fire` | ✓ 已產生 |
| `a33_old_body_rot` | ✓ 已產生 |
| `a33_old_corrupting_aura` | ✓ 已產生 |
| `a33_old_dark_summoning` | ✓ 已產生 |

### A34_old `omatsu_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a34_old_frost_bolt` | ✓ 已產生 |
| `a34_old_iceberg_strike` | ✓ 已產生 |
| `a34_old_frost_breath` | ✓ 已產生 |
| `a34_old_snow_blessing` | ✓ 已產生 |

### A34_v20 `omatsu_v20`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a34_v20_frost_bolt` | ✓ 已產生 |
| `a34_v20_iceberg_strike` | ✓ 已產生 |
| `a34_v20_frost_breath` | ✓ 已產生 |
| `a34_v20_snow_blessing` | ✓ 已產生 |
| `a34_v20_kaga_million_koku` | ✓ 已產生 |

### A35 `shima_sakon`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a35_flame_raid` | ✓ 已產生 |
| `a35_sealing_blaze` | ✓ 已產生 |
| `a35_flame_swiftness` | ✓ 已產生 |
| `a35_scorching_fire` | ✓ 已產生 |

### A35_old `shima_sakon_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a35_old_blazing_charge` | ✓ 已產生 |
| `a35_old_flame_guardians` | ✓ 已產生 |
| `a35_old_agility_boost` | ✓ 已產生 |
| `a35_old_scorching_blaze` | ✓ 已產生 |

### A36 `maeno_nagayasu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a36_shadow_step` | ✓ 已產生 |
| `a36_spike_trap` | ✓ 已產生 |
| `a36_detonate` | ✓ 已產生 |
| `a36_bitter_brew` | ✓ 已產生 |
| `a36_deadly_strike` | ✓ 已產生 |

### B01_old `saika_magoichi_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b01_old_sniper_mode` | ✓ 已產生 |
| `b01_old_saika_corps` | ✓ 已產生 |
| `b01_old_rain_iron_cannon` | ✓ 已產生 |
| `b01_old_three_stage` | ✓ 已產生 |

### B02 `fuma_kotaro`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b02_vanishing_step` | ✓ 已產生 |
| `b02_hazy_shadow` | ✓ 已產生 |
| `b02_blazing_path` | ✓ 已產生 |
| `b02_eighteen_shuriken` | ✓ 已產生 |
| `b02_four_thunders` | ✓ 已產生 |

### B02_old `fuma_kotaro_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b02_old_vanishing_step` | ✓ 已產生 |
| `b02_old_blazing_path` | ✓ 已產生 |
| `b02_old_eighteen_shuriken` | ✓ 已產生 |
| `b02_old_four_thunders` | ✓ 已產生 |

### B03 `saito_tomonobu`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b03_zhongkui_strike` | ✓ 已產生 |
| `b03_zhongkui_strike_mid` | ✓ 已產生 |
| `b03_zhongkui_strike_far` | ✓ 已產生 |
| `b03_echigo_vajra` | ✓ 已產生 |
| `b03_heavy_blow` | ✓ 已產生 |
| `b03_zhongkui_judgment` | ✓ 已產生 |

### B03_old `saito_tomonobu_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b03_old_zhongkui_might` | ✓ 已產生 |
| `b03_old_echigo_vajra` | ✓ 已產生 |
| `b03_old_mighty_blow` | ✓ 已產生 |
| `b03_old_zhongkui_judgment` | ✓ 已產生 |

### B04_old `date_masamune_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b04_old_flame_blade` | ✓ 已產生 |
| `b04_old_flame_trail` | ✓ 已產生 |
| `b04_old_fire_blast` | ✓ 已產生 |
| `b04_old_agility_boost` | ✓ 已產生 |
| `b04_old_matchlock` | ✓ 已產生 |

### B05 `mori_motonari`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b05_chain_lightning` | ✓ 已產生 |
| `b05_heavenly_thunder` | ✓ 已產生 |
| `b05_three_arrows` | ✓ 已產生 |
| `b05_holy_light_burst` | ✓ 已產生 |

### B05_old `mori_motonari_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b05_old_chain_lightning` | ✓ 已產生 |
| `b05_old_heavenly_thunder` | ✓ 已產生 |
| `b05_old_three_arrows` | ✓ 已產生 |
| `b05_old_spirit_dominion` | ✓ 已產生 |

### B06_old `sanada_yukimura_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b06_old_brave_warriors` | ✓ 已產生 |
| `b06_old_flame_fang` | ✓ 已產生 |
| `b06_old_mind_eye` | ✓ 已產生 |
| `b06_old_first_warrior` | ✓ 已產生 |

### B07 `ashikaga_yoshiaki`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b07_starfall` | ✓ 已產生 |
| `b07_battle_line` | ✓ 已產生 |
| `b07e_camp_regen` | ✓ 已產生 |
| `b07_glorious_momentum` | ✓ 已產生 |
| `b07_reign_over_all` | ✓ 已產生 |

### B07_old `ashikaga_yoshiaki_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b07_old_soldier_camp` | ✓ 已產生 |
| `b07_old_camp_regen` | ✓ 已產生 |
| `b07_old_restoration` | ✓ 已產生 |
| `b07_old_starfall` | ✓ 已產生 |
| `b07_old_reign_over_all` | ✓ 已產生 |

### B08_old `asai_nagamasa_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b08_old_fang_slash` | ✓ 已產生 |
| `b08_old_flash_cut` | ✓ 已產生 |
| `b08_old_living_soul` | ✓ 已產生 |
| `b08_old_sky_fang` | ✓ 已產生 |

### B09 `sanada_masayuki`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b09_backlash_rift` | ✓ 已產生 |
| `b09_mind_burst` | ✓ 已產生 |
| `b09_will_shield` | ✓ 已產生 |
| `b09_mind_suppression` | ✓ 已產生 |

### B09_old `sanada_masayuki_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b09_old_thunder_strike` | ✓ 已產生 |
| `b09_old_earth_rupture` | ✓ 已產生 |
| `b09_old_enfeeble` | ✓ 已產生 |
| `b09_old_whirlwind` | ✓ 已產生 |
| `b09_old_gale` | ✓ 已產生 |

### B10 `miyoshi_nagayoshi`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b10_wrath_of_the_dead` | ✓ 已產生 |
| `b10_corpse_assault` | ✓ 已產生 |
| `b10_demonic_war_spirit` | ✓ 已產生 |
| `b10_power_of_the_abyss` | ✓ 已產生 |

### B10_old `miyoshi_nagayoshi_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b10_old_swift_march` | ✓ 已產生 |
| `b10_old_axe_throw` | ✓ 已產生 |
| `b10_old_war_spirit` | ✓ 已產生 |
| `b10_old_demonic_frenzy` | ✓ 已產生 |

### B11 `kikuhime`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b11_cherry_rain` | ✓ 已產生 |
| `b11_breath_control` | ✓ 已產生 |
| `b11_interception` | ✓ 已產生 |
| `b11_crow_raid` | ✓ 已產生 |
| `b11_crow_strike` | ✓ 已產生 |
| `b11_inherited_blood` | ✓ 已產生 |

### B11_old `kikuhime_old`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b11_old_cherry_rain` | ✓ 已產生 |
| `b11_old_spider_bind` | ✓ 已產生 |
| `b11_old_breath_control` | ✓ 已產生 |
| `b11_old_clone_art` | ✓ 已產生 |
| `b11_old_crow_strike` | ✓ 已產生 |
| `b11_old_vacuum_slash` | ✓ 已產生 |

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

### B13_old `momochi_sandayu_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b13_old_earth_hiding` | ✓ 已產生 |
| `b13_old_hypnosis` | ✓ 已產生 |
| `b13_old_hidden_mine` | ✓ 已產生 |
| `b13_old_backstab` | ✓ 已產生 |
| `b13_old_tempest` | ✓ 已產生 |

### B14 `katakura_kagetsuna`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b14_temporary_night` | ✓ 已產生 |
| `b14_iai_slash` | ✓ 已產生 |
| `b14_shadow_tactics` | ✓ 已產生 |
| `b14_attendant_insight` | ✓ 已產生 |
| `b14_shadow_cut` | ✓ 已產生 |

### B14_old `katakura_kagetsuna_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b14_old_iai_slash` | ✓ 已產生 |
| `b14_old_attendant_insight` | ✓ 已產生 |
| `b14_old_swordsmanship` | ✓ 已產生 |
| `b14_old_shadow_slash` | ✓ 已產生 |
| `b14_old_shadow_warrior` | ✓ 已產生 |

### B15 `imagawa_yoshimoto`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b15_tokai_giant` | ✓ 已產生 |
| `b15_flame_arrow` | ✓ 已產生 |
| `b15_giant_hide` | ✓ 已產生 |
| `b15_multishot` | ✓ 已產生 |
| `b15_bluff` | ✓ 已產生 |

### B15_old `imagawa_yoshimoto_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b15_old_giant_strength` | ✓ 已產生 |
| `b15_old_flame_arrow` | ✓ 已產生 |
| `b15_old_giant_hide` | ✓ 已產生 |
| `b15_old_multishot` | ✓ 已產生 |

### B15_v20 `imagawa_yoshimoto_v20`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b15_v20_hundred_paces` | ✓ 已產生 |
| `b15_v20_giant_strength` | ✓ 已產生 |
| `b15_v20_flame_arrow` | ✓ 已產生 |
| `b15_v20_giant_hide` | ✓ 已產生 |
| `b15_v20_tokaido_archer` | ✓ 已產生 |

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

### B16_old `mochizuki_chiyome_old`（7 / 7 招）

| 招式 | 還缺 |
|---|---|
| `b16_old_summon_hound` | ✓ 已產生 |
| `b16_old_falling_leaves` | ✓ 已產生 |
| `b16_old_swift_thunder` | ✓ 已產生 |
| `b16_old_smoke_bomb` | ✓ 已產生 |
| `b16_old_mm_one_strike` | ✓ 已產生 |
| `b16_old_mm_rending` | ✓ 已產生 |
| `b16_old_mm_stealth` | ✓ 已產生 |

### B17 `kirigakure_saizo`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b17_death_ward` | ✓ 已產生 |
| `b17_tooth_poison` | ✓ 已產生 |
| `b17_shadow_leap` | ✓ 已產生 |
| `b17_heartseeker` | ✓ 已產生 |

### B17_old `kirigakure_saizo_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b17_old_phantom_step` | ✓ 已產生 |
| `b17_old_venom_shuriken` | ✓ 已產生 |
| `b17_old_spinning_shuriken` | ✓ 已產生 |
| `b17_old_transformation` | ✓ 已產生 |

### B18 `koihime`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b18_earth_raid` | ✓ 已產生 |
| `b18_echo` | ✓ 已產生 |
| `b18_earth_shelter` | ✓ 已產生 |
| `b18_earth_claw` | ✓ 已產生 |

### B18_old `koihime_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b18_old_victory_wish` | ✓ 已產生 |
| `b18_old_earth_spike` | ✓ 已產生 |
| `b18_old_earth_bind` | ✓ 已產生 |
| `b18_old_earth_claw` | ✓ 已產生 |

### B19 `usami_sadamitsu`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b19_water_dragon` | ✓ 已產生 |
| `b19_ice_fence` | ✓ 已產生 |
| `b19_echigo_tactics` | ✓ 已產生 |
| `b19_kuruma_gakari` | ✓ 已產生 |

### B19_old `usami_sadamitsu_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b19_old_water_dragon` | ✓ 已產生 |
| `b19_old_ice_fence` | ✓ 已產生 |
| `b19_old_echigo_tactics` | ✓ 已產生 |
| `b19_old_suruga_lord` | ✓ 已產生 |
| `b19_old_kuruma_gakari` | ✓ 已產生 |

### B20 `yamagata_masakage`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b20_rock_rush` | ✓ 已產生 |
| `b20_ruin_wave` | ✓ 已產生 |
| `b20_tenacity` | ✓ 已產生 |
| `b20_red_charge` | ✓ 已產生 |

### B20_old `yamagata_masakage_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b20_old_rock_rush` | ✓ 已產生 |
| `b20_old_ruin_wave` | ✓ 已產生 |
| `b20_old_tenacity` | ✓ 已產生 |
| `b20_old_red_charge` | ✓ 已產生 |

### B21 `baba_nobufusa`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b21_sky_wave` | ✓ 已產生 |
| `b21_killing_intent` | ✓ 已產生 |
| `b21_rebel_armor` | ✓ 已產生 |
| `b21_undying_demon` | ✓ 已產生 |

### B21_old `baba_nobufusa_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b21_old_triple_wave` | ✓ 已產生 |
| `b21_old_killing_intent` | ✓ 已產生 |
| `b21_old_divine_command` | ✓ 已產生 |
| `b21_old_undying_demon` | ✓ 已產生 |

### B22 `taigen_sessai`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b22_natural_barrier` | ✓ 已產生 |
| `b22_force_of_nature` | ✓ 已產生 |
| `b22_purify` | ✓ 已產生 |
| `b22_wood_spirit` | ✓ 已產生 |
| `b22_wrath_of_nature` | ✓ 已產生 |

### B22_old `taigen_sessai_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b22_old_natural_barrier` | ✓ 已產生 |
| `b22_old_force_of_nature` | ✓ 已產生 |
| `b22_old_purify` | ✓ 已產生 |
| `b22_old_wrath_of_nature` | ✓ 已產生 |

### B23 `ise_hime`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b23_undead_pact` | ✓ 已產生 |
| `b23_spirit_gathering` | ✓ 已產生 |
| `b23_living_soul_shock` | ✓ 已產生 |
| `b23_soul_feast` | ✓ 已產生 |
| `b23_heroic_spirit_blessing` | ✓ 已產生 |

### B23_old `ise_hime_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b23_old_undead_pact` | ✓ 已產生 |
| `b23_old_spirit_gathering` | ✓ 已產生 |
| `b23_old_living_soul_shock` | ✓ 已產生 |
| `b23_old_soul_feast` | ✓ 已產生 |
| `b23_old_mind_control` | ✓ 已產生 |

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

### B25_old `honganji_kennyo_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b25_old_exorcism_strike` | ✓ 已產生 |
| `b25_old_exorcism_sutra` | ✓ 已產生 |
| `b25_old_finger_of_death` | ✓ 已產生 |
| `b25_old_asura_hell` | ✓ 已產生 |

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

### B27_old `kojima_sadaoki_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b27_old_horsemanship` | ✓ 已產生 |
| `b27_old_mighty_strike` | ✓ 已產生 |
| `b27_old_spell_shield` | ✓ 已產生 |
| `b27_old_earth_shatter` | ✓ 已產生 |

### B28 `matsunaga_hisahide`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b28_phoenix_flame` | ✓ 已產生 |
| `b28_flame_shackle` | ✓ 已產生 |
| `b28_fire_stream` | ✓ 已產生 |
| `b28_serpent_fire` | ✓ 已產生 |

### B28_old `matsunaga_hisahide_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b28_old_fire_attack` | ✓ 已產生 |
| `b28_old_binding` | ✓ 已產生 |
| `b28_old_flame_storm` | ✓ 已產生 |
| `b28_old_earth_rend` | ✓ 已產生 |

### B29_old `murakami_yoshikiyo_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b29_old_takeoumaru` | ✓ 已產生 |
| `b29_old_ground_slam` | ✓ 已產生 |
| `b29_old_quake_axe` | ✓ 已產生 |
| `b29_old_morale_boost` | ✓ 已產生 |
| `b29_old_earth_tremor` | ✓ 已產生 |

### B30 `kakizaki_kageie`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b30_reverse_shield` | ✓ 已產生 |
| `b30_soul_breaker` | ✓ 已產生 |
| `b30_flash_strike` | ✓ 已產生 |
| `b30_purgatory_slash` | ✓ 已產生 |

### B30_old `kakizaki_kageie_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b30_old_charge` | ✓ 已產生 |
| `b30_old_blazing_sword` | ✓ 已產生 |
| `b30_old_purgatory_blade` | ✓ 已產生 |
| `b30_old_purgatory_slash` | ✓ 已產生 |

### B31 `takeda_shingen`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b31_swift_wind` | ✓ 已產生 |
| `b31_still_forest` | ✓ 已產生 |
| `b31_raging_fire` | ✓ 已產生 |
| `b31_immovable_mountain` | ✓ 已產生 |

### B31_old `takeda_shingen_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b31_old_swift_as_wind` | ✓ 已產生 |
| `b31_old_quiet_as_forest` | ✓ 已產生 |
| `b31_old_fierce_as_fire` | ✓ 已產生 |
| `b31_old_immovable_as_mountain` | ✓ 已產生 |

### B32 `uesugi_kenshin`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b32_divine_slash` | ✓ 已產生 |
| `b32_divine_flash` | ✓ 已產生 |
| `b32_divine_formation` | ✓ 已產生 |
| `b32_war_god` | ✓ 已產生 |
| `b32_bishamonten` | ✓ 已產生 |

### B32_old `uesugi_kenshin_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b32_old_war_god_aura` | ✓ 已產生 |
| `b32_old_divine_slash` | ✓ 已產生 |
| `b32_old_divine_flash` | ✓ 已產生 |
| `b32_old_divine_fury` | ✓ 已產生 |
| `b32_old_bishamonten` | ✓ 已產生 |

### B32_v20 `uesugi_kenshin_v20`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b32_v20_divine_slash` | ✓ 已產生 |
| `b32_v20_divine_flash` | ✓ 已產生 |
| `b32_v20_divine_formation` | ✓ 已產生 |
| `b32_v20_war_god` | ✓ 已產生 |
| `b32_v20_bishamonten` | ✓ 已產生 |

### B33 `mogami_yoshihime`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b33_corrosive_arrow` | ✓ 已產生 |
| `b33_spider_web` | ✓ 已產生 |
| `b33_voodoo_power` | ✓ 已產生 |
| `b33_spider_attack` | ✓ 已產生 |

### B33_old `mogami_yoshihime_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b33_old_corrosive_arrow` | ✓ 已產生 |
| `b33_old_spider_web` | ✓ 已產生 |
| `b33_old_poison_arrow` | ✓ 已產生 |
| `b33_old_spider_attack` | ✓ 已產生 |

### B34 `takeda_katsuyori`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b34_wrath_axe` | ✓ 已產生 |
| `b34_spike_wall` | ✓ 已產生 |
| `b34_frenzy` | ✓ 已產生 |
| `b34_tenmoku_howl` | ✓ 已產生 |

### B34_old `takeda_katsuyori_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b34_old_wrath_axe` | ✓ 已產生 |
| `b34_old_berserk` | ✓ 已產生 |
| `b34_old_rending_wound` | ✓ 已產生 |
| `b34_old_great_quake` | ✓ 已產生 |

### B35 `kamiizumi_nobutsuna`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b35_sword_essence` | ✓ 已產生 |
| `b35_isshun` | ✓ 已產生 |
| `b35_muto_dori` | ✓ 已產生 |
| `b35_hiten` | ✓ 已產生 |

### B35_old `kamiizumi_nobutsuna_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b35_old_sword_essence` | ✓ 已產生 |
| `b35_old_issen` | ✓ 已產生 |
| `b35_old_shinkage_ryu` | ✓ 已產生 |
| `b35_old_final_secret_sword` | ✓ 已產生 |

### B36 `naoe_kanetsugu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b36_naoe_seal` | ✓ 已產生 |
| `b36_footsteps` | ✓ 已產生 |
| `b36_heart_fence` | ✓ 已產生 |
| `b36_seven_spears` | ✓ 已產生 |
| `b36_aizen_myoo` | ✓ 已產生 |

### B36_old `naoe_kanetsugu_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b36_old_binding_seal` | ✓ 已產生 |
| `b36_old_charm_barrier` | ✓ 已產生 |
| `b36_old_curse_charm` | ✓ 已產生 |
| `b36_old_far_reach` | ✓ 已產生 |
| `b36_old_kill_formation` | ✓ 已產生 |

### C01 `akechi_mitsuhide`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c01_blood_blade` | ✓ 已產生 |
| `c01_ground_flame` | ✓ 已產生 |
| `c01_blood_flame` | ✓ 已產生 |
| `c01_honnoji` | ✓ 已產生 |

### C01_old `akechi_mitsuhide_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c01_old_blood_blade` | ✓ 已產生 |
| `c01_old_fire_rain` | ✓ 已產生 |
| `c01_old_swordsmanship` | ✓ 已產生 |
| `c01_old_anointing_slash` | ✓ 已產生 |
| `c01_old_honnoji` | ✓ 已產生 |

### C02 `akechi_hidemitsu`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c02_mirage_strike` | ✓ 已產生 |
| `c02_phantom_move` | ✓ 已產生 |
| `c02_kyoka_suigetsu` | ✓ 已產生 |
| `c02_culling_slash` | ✓ 已產生 |

### C02_old `akechi_hidemitsu_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c02_old_surging_waves` | ✓ 已產生 |
| `c02_old_soul_drain` | ✓ 已產生 |
| `c02_old_oni_swordsmanship` | ✓ 已產生 |
| `c02_old_oni_might` | ✓ 已產生 |
| `c02_old_flame_dragon_sword` | ✓ 已產生 |

### C03 `shimazu_yoshihiro`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c03_iron_shirt` | ✓ 已產生 |
| `c03_quake_axe` | ✓ 已產生 |
| `c03_thunder_wrath` | ✓ 已產生 |
| `c03_iron_hammer` | ✓ 已產生 |

### C03_old `shimazu_yoshihiro_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c03_old_iron_shirt` | ✓ 已產生 |
| `c03_old_quake_axe` | ✓ 已產生 |
| `c03_old_thunder_wrath` | ✓ 已產生 |
| `c03_old_iron_hammer` | ✓ 已產生 |

### C04 `izumo_no_okuni`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c04_swift_dance` | ✓ 已產生 |
| `c04_withering_step` | ✓ 已產生 |
| `c04_gathering_dance` | ✓ 已產生 |
| `c04_witch_dance` | ✓ 已產生 |

### C04_old `izumo_no_okuni_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c04_old_swift_dance` | ✓ 已產生 |
| `c04_old_withering_step` | ✓ 已產生 |
| `c04_old_dance_of_death` | ✓ 已產生 |
| `c04_old_witch_dance` | ✓ 已產生 |

### C04_v20 `izumo_no_okuni_v20`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c04_v20_swift_dance` | ✓ 已產生 |
| `c04_v20_withering_step` | ✓ 已產生 |
| `c04_v20_gathering_dance` | ✓ 已產生 |
| `c04_v20_witch_dance` | ✓ 已產生 |
| `c04_v20_guardian_dance` | ✓ 已產生 |

### C05 `maeda_keiji`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c05_matsukaze` | ✓ 已產生 |
| `c05_thunder_dance` | ✓ 已產生 |
| `c05_thunder_spear` | ✓ 已產生 |
| `c05_flash_thunder` | ✓ 已產生 |

### C05_old `maeda_keiji_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c05_old_horsemanship` | ✓ 已產生 |
| `c05_old_thunder_dance` | ✓ 已產生 |
| `c05_old_thunder_strike` | ✓ 已產生 |
| `c05_old_thunder_spear` | ✓ 已產生 |

### C06 `ishikawa_goemon`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `c06_quick_hands` | ✓ 已產生 |
| `c06_sleight_of_hand` | ✓ 已產生 |
| `c06_lavish_strike` | ✓ 已產生 |
| `c06_murder_and_loot` | ✓ 已產生 |
| `c06_grand_heist` | ✓ 已產生 |
| `item_c06e` | ✓ 已產生 |

### C06_old `ishikawa_goemon_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c06_old_frenzy` | ✓ 已產生 |
| `c06_old_murder_and_loot` | ✓ 已產生 |
| `c06_old_savage_strength` | ✓ 已產生 |
| `c06_old_robbery` | ✓ 已產生 |
| `c06_old_thief_hiding` | ✓ 已產生 |

### C07 `tachibana_dosetsu`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `c07_heaven_thunder` | ✓ 已產生 |
| `c07_thunder_shackle` | ✓ 已產生 |
| `c07_thunderbolt` | ✓ 已產生 |
| `c07_raijin_wrath` | ✓ 已產生 |
| `c07_storm_move` | ✓ 已產生 |
| `c07_magic_shield` | ✓ 已產生 |

### C07_old `tachibana_dosetsu_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c07_old_magic_shield` | ✓ 已產生 |
| `c07_old_thunder_shackle` | ✓ 已產生 |
| `c07_old_thunderbolt` | ✓ 已產生 |
| `c07_old_raijin_wrath` | ✓ 已產生 |

### C08 `kato_danzo`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c08_vanishing_step` | ✓ 已產生 |
| `c08_blood_hunt` | ✓ 已產生 |
| `c08_scatter_escape` | ✓ 已產生 |
| `c08_kite_chain` | ✓ 已產生 |
| `c08_devour` | ✓ 已產生 |

### C08_old `kato_danzo_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c08_old_empty_zen` | ✓ 已產生 |
| `c08_old_cloud_veil` | ✓ 已產生 |
| `c08_old_chidori` | ✓ 已產生 |
| `c08_old_crimson_lotus` | ✓ 已產生 |
| `c08_old_flying_kite` | ✓ 已產生 |

### C09 `tachibana_ginchiyo`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c09_frost_thorn` | ✓ 已產生 |
| `c09_white_heat` | ✓ 已產生 |
| `c09_ice_soul_dance` | ✓ 已產生 |
| `c09_armor_break` | ✓ 已產生 |

### C09_old `tachibana_ginchiyo_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c09_old_frost_slash` | ✓ 已產生 |
| `c09_old_white_heat` | ✓ 已產生 |
| `c09_old_ice_soul_dance` | ✓ 已產生 |
| `c09_old_armor_break` | ✓ 已產生 |

### C10 `kosokabe_chikayasu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c10_black_dragon_wave` | ✓ 已產生 |
| `c10_energy_drain` | ✓ 已產生 |
| `c10_echo_sword` | ✓ 已產生 |
| `c10_day_night_shift` | ✓ 已產生 |
| `c10_formless_heaven` | ✓ 已產生 |

### C10_old `kosokabe_chikayasu_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c10_old_black_dragon_wave` | ✓ 已產生 |
| `c10_old_energy_drain` | ✓ 已產生 |
| `c10_old_mushin_rebirth` | ✓ 已產生 |
| `c10_old_rising_dragon_dance` | ✓ 已產生 |

### C11 `tachibana_muneshige`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c11_raikiri` | ✓ 已產生 |
| `c11_thunder_speed` | ✓ 已產生 |
| `c11_thunder_eye` | ✓ 已產生 |
| `c11_unrivaled` | ✓ 已產生 |

### C11_old `tachibana_muneshige_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c11_old_raikiri` | ✓ 已產生 |
| `c11_old_thunder_haste` | ✓ 已產生 |
| `c11_old_thunder_eye` | ✓ 已產生 |
| `c11_old_unrivaled` | ✓ 已產生 |

### C11_v20 `tachibana_muneshige_v20`（3 / 3 招）

| 招式 | 還缺 |
|---|---|
| `c11_v20_lightning_flash` | ✓ 已產生 |
| `c11_v20_thunder_fang` | ✓ 已產生 |
| `c11_v20_thunder_eye` | ✓ 已產生 |

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

### C13_old `ryuzoji_takanobu_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c13_old_wind_chain` | ✓ 已產生 |
| `c13_old_bloody_gale` | ✓ 已產生 |
| `c13_old_tornado_wave` | ✓ 已產生 |
| `c13_old_strength_up` | ✓ 已產生 |
| `c13_old_tornado_vortex` | ✓ 已產生 |

### C14 `saito_yoshitatsu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c14_fire_curse` | ✓ 已產生 |
| `c14_fire_breath` | ✓ 已產生 |
| `c14_fire_breath_dragon` | ✓ 已產生 |
| `c14_dragon_vein` | ✓ 已產生 |
| `c14_dragon_form` | ✓ 已產生 |

### C14_old `saito_yoshitatsu_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c14_old_fire_curse` | ✓ 已產生 |
| `c14_old_fire_breath` | ✓ 已產生 |
| `c14_old_dragon_vein` | ✓ 已產生 |
| `c14_old_dragon_form` | ✓ 已產生 |

### C15_old `tamako_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c15_old_triple_arrow` | ✓ 已產生 |
| `c15_old_searing_arrow` | ✓ 已產生 |
| `c15_old_precise_light` | ✓ 已產生 |
| `c15_old_hundred_arrows` | ✓ 已產生 |

### C16 `kashin_koji`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c16_soul_sever` | ✓ 已產生 |
| `c16_curse_shadow` | ✓ 已產生 |
| `c16_wandering_souls` | ✓ 已產生 |
| `c16_phantom_sight` | ✓ 已產生 |
| `c16_eight_gates` | ✓ 已產生 |

### C16_old `kashin_koji_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c16_old_phantom_sight` | ✓ 已產生 |
| `c16_old_curse_shadow` | ✓ 已產生 |
| `c16_old_wandering_souls` | ✓ 已產生 |
| `c16_old_lingering_wraith` | ✓ 已產生 |
| `c16_old_eight_gates` | ✓ 已產生 |

### C17_old `oichi_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c17_old_light_essence` | ✓ 已產生 |
| `c17_old_soul_essence` | ✓ 已產生 |
| `c17_old_stasis_trap` | ✓ 已產生 |
| `c17_old_aurora_guard` | ✓ 已產生 |

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

### C19_old `matsuhime_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c19_old_paralyzing_poison` | ✓ 已產生 |
| `c19_old_healing_mist` | ✓ 已產生 |
| `c19_old_surprise_raid` | ✓ 已產生 |
| `c19_old_inviolable` | ✓ 已產生 |

### C20 `chosokabe_motochika`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c20_flame_aegis` | ✓ 已產生 |
| `c20_demon_blood` | ✓ 已產生 |
| `c20_flame_dance` | ✓ 已產生 |
| `c20_demon_child` | ✓ 已產生 |

### C20_old `chosokabe_motochika_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c20_old_flame_aegis` | ✓ 已產生 |
| `c20_old_demon_blood` | ✓ 已產生 |
| `c20_old_flame_dance` | ✓ 已產生 |
| `c20_old_demon_child` | ✓ 已產生 |

### C21 `miyamoto_musashi`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c21_crescent_slash` | ✓ 已產生 |
| `c21_chain_breaker` | ✓ 已產生 |
| `c21_two_heavens` | ✓ 已產生 |
| `c21_soaring_quake` | ✓ 已產生 |

### C21_old `miyamoto_musashi_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c21_old_crescent_slash` | ✓ 已產生 |
| `c21_old_chain_breaker` | ✓ 已產生 |
| `c21_old_two_heavens` | ✓ 已產生 |
| `c21_old_soaring_quake` | ✓ 已產生 |

### C22 `sasaki_kojiro`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c22_ganryu` | ✓ 已產生 |
| `c22_swallow_nest` | ✓ 已產生 |
| `c22_flying_swallow` | ✓ 已產生 |
| `c22_swallow_flash` | ✓ 已產生 |
| `c22_dream_mist` | ✓ 已產生 |

### C22_old `sasaki_kojiro_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c22_old_swallow` | ✓ 已產生 |
| `c22_old_return` | ✓ 已產生 |
| `c22_old_ganryu` | ✓ 已產生 |
| `c22_old_earth_swallow` | ✓ 已產生 |

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

### C24_old `yagyu_muneyoshi_old`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c24_old_muto_dori` | ✓ 已產生 |
| `c24_old_ukifune` | ✓ 已產生 |
| `c24_old_shinkage` | ✓ 已產生 |
| `c24_old_twin_storm` | ✓ 已產生 |

### C25 `chacha`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c25_raid` | ✓ 已產生 |
| `c25_blood_blade` | ✓ 已產生 |
| `c25_grievous_wound` | ✓ 已產生 |
| `c25_falling_sakura` | ✓ 已產生 |

### C25_old `chacha_old`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c25_old_spin_slash` | ✓ 已產生 |
| `c25_old_dagger_throw` | ✓ 已產生 |
| `c25_old_dazzle` | ✓ 已產生 |
| `c25_old_bloodletting` | ✓ 已產生 |
| `c25_old_flash_slash` | ✓ 已產生 |

## 3. 已產生的招式在執行期回 RErr 的情況

產生的程式碼遇到資料沒預期的世界狀態時，施放回 `RErr`（host 回滾成本）。下表是已產生的招式裡各種情況出現的招數；未產生的招式是 stub，一律回 `RErr` 並列出 §2 的缺項。

| 情況 | 招數 |
|---|---|
| 施放檢查：rank、施法者存活、目標存活／隊伍／類別、施法距離（同 IR） | 703 |
| 召喚：附近放不下、lifetime 不在 0～600 秒 | 19 |
| 召喚：count 不是 1 | 2 |
| 視野：半徑或位置不合法 | 53 |
| 光鍊：host tether 規格不合法 | 1 |
| 友方回復：比例不在 0～1、或不在施法者周圍時沒有施法距離 | 1 |
| 運算式除以零 | 52 |
| 讀位置的單位已不在世界中 | 445 |
| 需要地點，但施放沒有地點也沒有單位目標 | 195 |

### 已產生但 host 沒有做到的地方

產生器不默默略過欄位：不支援的一律擋下（上面各節）；下面這些已產生，但文件寫的某件事 host 目前做不到，照原樣列出。

| 招式 | 沒有做到 |
|---|---|
| `c06_grand_heist` | create_pickup.cue: the host pickup draws its own bag |
| `item_flash_ring` | item cast_point: the host uses items without a windup |
| `item_flood_book` | item cast_point: the host uses items without a windup |
| `item_glass_mitsutomo` | item cast_point: the host uses items without a windup |
| `item_kisyosennfuyuzuki` | item cast_point: the host uses items without a windup |
| `item_kokumo` | item cast_point: the host uses items without a windup |
| `item_lightning_scroll` | item cast_point: the host uses items without a windup |
| `item_logs_big_tachi` | item cast_point: the host uses items without a windup |
| `item_raikiri` | item cast_point: the host uses items without a windup |
| `item_storm_book` | item cast_point: the host uses items without a windup |
| `item_the_club_of_nebula` | item cast_point: the host uses items without a windup |
| `item_the_great_sword_of_anger` | item cast_point: the host uses items without a windup |
| `item_the_great_sword_of_disease` | item cast_point: the host uses items without a windup |
| `item_the_great_sword_of_soul_hyper` | item cast_point: the host uses items without a windup |
| `item_the_great_sword_of_tiger` | item cast_point: the host uses items without a windup |
| `item_the_great_sword_of_toxic` | item cast_point: the host uses items without a windup |
| `item_the_great_sword_of_xnew_moon` | item cast_point: the host uses items without a windup |
| `item_the_overflame_art_of_war` | item cast_point: the host uses items without a windup |
| `item_the_red_lightning_chapter` | item cast_point: the host uses items without a windup |
| `item_the_red_lightning_chapter_hyper` | item cast_point: the host uses items without a windup |
| `item_yubashiri` | item cast_point: the host uses items without a windup |

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
| `catalog: item item_ancient_doll is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_cross_spear is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ghost_gloves is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_gold_to_prestige is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_great_dragon is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_hachiman_bell is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ice_book is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_insight_gem is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_kat_steel is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ksitigarbha is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_magic_ring is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_nannbann_armor is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_poisonous_ring is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_prophet_akazonae_armor is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_prophet_armor is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_samurais_badge is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_shintogo_kunimitsu is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `event.order.target` | ability_kit | 共用 | 0 | 2 | 0 |
| `event:on_owner_death` | ability_kit | 共用 | 0 | 3 | 0 |
| `event:on_unequip` | ability_kit | 共用 | 0 | 2 | 0 |
| `fn:has_tag:courier of a unit` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `fn:has_tag:no_cast_trigger` | 產生器 | 共用 | 0 | 2 | 0 |
| `fn:has_tag:ultimate` | 產生器 | 共用 | 0 | 2 | 0 |
| `item.active.cast.interruptible` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `item.active.cost.charges` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `item.active.kind:passive` | ability_kit | 共用 | 0 | 3 | 0 |
| `item.active.kind:toggle` | ability_kit | 共用 | 0 | 2 | 0 |
| `passive:orb_slot (only a toggle that claims and releases at status_end)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `passive_trigger:on_ability_executed of an item (no learn marker)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `passive_trigger:on_death of an item (no learn marker)` | ability_kit | 共用 | 0 | 2 | 0 |
| `passive_trigger:on_respawn of an item (no learn marker)` | ability_kit | 共用 | 0 | 2 | 0 |
| `rust_handler without behavior` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `selector.exclude_types` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `selector.types:building+hero+tower` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `stat.when:not primary_is(self, agi)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `stat.when:not primary_is(self, int)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `stat.when:not primary_is(self, str)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `stat:cooldown_reduction_pct` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `state type:bool (item_instance)` | ability_kit | 共用 | 0 | 2 | 0 |
| `state type:unit (item_instance)` | ability_kit | 共用 | 0 | 2 | 0 |
| `status.absorb on an independent status` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `status.absorb on this apply form` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `status.absorb.phase:pre_mitigation` | ability_kit | 共用 | 0 | 2 | 0 |
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
| `passive_trigger:on_ability_executed of an item (no learn marker)` | `item_bizennosafunekanemitsu` | 道具 |
| `attr:is_summon` | `item_commander_of_fan` | 道具 |
| `selector.types:building+hero+tower` | `item_contemporary_armor` | 道具 |
| `catalog: item item_cross_spear is not in the shop (templates/moba_items.lua)` | `item_cross_spear` | 道具 |
| `status.absorb on this apply form` | `item_devil_supressor_armor` | 道具 |
| `item.active.cost.charges` | `item_earth_wall_book` | 道具 |
| `unit_template blocks_pathing without collision_radius` | `item_earth_wall_book` | 道具 |
| `catalog: item item_ghost_gloves is not in the shop (templates/moba_items.lua)` | `item_ghost_gloves` | 道具 |
| `catalog: item item_gold_to_prestige is not in the shop (templates/moba_items.lua)` | `item_gold_to_prestige` | 道具 |
| `catalog: item item_great_dragon is not in the shop (templates/moba_items.lua)` | `item_great_dragon` | 道具 |
| `catalog: item item_hachiman_bell is not in the shop (templates/moba_items.lua)` | `item_hachiman_bell` | 道具 |
| `rust_handler without behavior` | `item_hama_orb` | 道具 |
| `catalog: item item_ice_book is not in the shop (templates/moba_items.lua)` | `item_ice_book` | 道具 |
| `action:remove_status.source_is` | `item_ice_storm_book` | 道具 |
| `catalog: item item_insight_gem is not in the shop (templates/moba_items.lua)` | `item_insight_gem` | 道具 |
| `aura.types:building+hero` | `item_insight_gem_hero` | 道具 |
| `catalog: item item_kat_steel is not in the shop (templates/moba_items.lua)` | `item_kat_steel` | 道具 |
| `catalog: item item_ksitigarbha is not in the shop (templates/moba_items.lua)` | `item_ksitigarbha` | 道具 |
| `catalog: item item_magic_ring is not in the shop (templates/moba_items.lua)` | `item_magic_ring` | 道具 |
| `status.absorb on an independent status` | `item_magic_talisman` | 道具 |
| `attr:illusion_damage` | `item_mana_thief` | 道具 |
| `catalog: item item_nannbann_armor is not in the shop (templates/moba_items.lua)` | `item_nannbann_armor` | 道具 |
| `selector.exclude_types` | `item_napalm_bomb` | 道具 |
| `catalog: item item_poisonous_ring is not in the shop (templates/moba_items.lua)` | `item_poisonous_ring` | 道具 |
| `catalog: item item_prophet_akazonae_armor is not in the shop (templates/moba_items.lua)` | `item_prophet_akazonae_armor` | 道具 |
| `catalog: item item_prophet_armor is not in the shop (templates/moba_items.lua)` | `item_prophet_armor` | 道具 |
| `action:projectile.lifetime` | `item_ronin_jitte` | 道具 |
| `catalog: item item_samurais_badge is not in the shop (templates/moba_items.lua)` | `item_samurais_badge` | 道具 |
| `catalog: item item_shintogo_kunimitsu is not in the shop (templates/moba_items.lua)` | `item_shintogo_kunimitsu` | 道具 |
| `action:refund of mana or health` | `item_soul_adder` | 道具 |
| `action:damage.type:event` | `item_the_dark_yellow_teeth_and_zimbabwe` | 道具 |
| `stat.when:not primary_is(self, agi)` | `item_the_great_sword_of_invincible` | 道具 |
| `stat.when:not primary_is(self, int)` | `item_the_great_sword_of_invincible` | 道具 |
| `stat.when:not primary_is(self, str)` | `item_the_great_sword_of_invincible` | 道具 |
| `fn:has_tag:courier of a unit` | `item_the_great_sword_of_teleport` | 道具 |
| `item.active.cast.interruptible` | `item_the_great_sword_of_teleport` | 道具 |
| `target.types:building+courier+creep+neutral+summon` | `item_the_great_sword_of_teleport` | 道具 |
| `stat:cooldown_reduction_pct` | `item_the_over_lightning_art_of_war` | 道具 |
| `passive:orb_slot (only a toggle that claims and releases at status_end)` | `item_the_spear_of_saddle` | 道具 |

共 40 種寫法、35 招。

## 重跑

```
tools\lua\lua.exe docs\character_pipeline\tools\v3_codegen_coverage.lua
```

