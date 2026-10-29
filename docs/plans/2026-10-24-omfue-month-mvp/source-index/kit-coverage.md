# 本月交付 kit 與盲測能力覆蓋

由 `scripts/hero_source_index.lua coverage` 從各 package 產生，不手寫。來源目錄、版本與能力分類見 `source-catalog.lua`；能力狀態見 `../capability-registry.md`。

「原作對照」：`已對照原作` 表示該招已依 history_nobu 原作 KV 與腳本逐項對照並移植（package `source.lua` 的 `parity`，對照卡在 package 的 `parity.md`）；`待重審` 表示來源改變後尚未重新對照（`stale_review`）。

| 計畫 | 來源 | 名稱 | package | kit 狀態 | 驗收 | 原作對照 | 技能數 |
|---|---|---|---|---|---|---|---|
| H01 | B01 | 雜賀孫市 | `saika_magoichi` | supported | pending | 已對照原作 | 4 |
| H02 | B04 | 伊達政宗 | `date_masamune` | supported | pending | 已對照原作 | 4 |
| H03 | A34 | 阿松 | `omatsu` | supported | implemented | 已對照原作 | 4 |
| H04 | C17 | 阿市 | `oichi` | supported | implemented | 已對照原作 | 5 |
| H05 | A02 | 稻姬 | `inahime` | supported | implemented | 已對照原作 | 4 |
| H06 | B08 | 淺井長政 | `asai_nagamasa` | supported | implemented | 已對照原作 | 5 |
| I1（盲測） | C15 | 玉子 | `tamako` | supported | implemented | 已對照原作 | 4 |
| I2（盲測） | B29 | 村上義清 | `murakami_yoshikiyo` | supported | implemented | 已對照原作 | 4 |

## H01 B01 雜賀孫市（`saika_magoichi`）

| 欄位 | 技能 | 來源 | 狀態 | 原作對照 | 能力 |
|---|---|---|---|---|---|
| Q | `b01_sniper_mode` | B01W | supported | 已對照原作 | timed_stat_buff、vision_radius_bonus |
| W | `b01_saika_corps` | B01E | supported | 已對照原作 | charges、summon_timed、summon_immobile |
| E | `b01_rain_iron_cannon` | B01R | supported | 已對照原作 | on_attack_hook、target_class_multiplier |
| R | `b01_three_stage` | B01T | supported | 已對照原作 | timed_stat_buff |

## H02 B04 伊達政宗（`date_masamune`）

| 欄位 | 技能 | 來源 | 狀態 | 原作對照 | 能力 |
|---|---|---|---|---|---|
| Q | `b04_flame_trail_dash` | B04W | supported | 已對照原作 | dash_to_point、random_target_det、projectile_tracking |
| W | `b04_fire_blast` | B04E | supported | 已對照原作 | aoe_circle、stun |
| E | `b04_flame_swiftness` | B04R | supported | 已對照原作 | permanent_stat_bonus |
| R | `b04_tanegashima` | B04T | supported | 已對照原作 | timed_stat_buff、proc_chance、stun、attack_range_switch |

## H03 A34 阿松（`omatsu`）

| 欄位 | 技能 | 來源 | 狀態 | 原作對照 | 能力 |
|---|---|---|---|---|---|
| Q | `a34_frost_bolt` | A34W | supported | 已對照原作 | projectile_tracking、projectile_dodge、unit_damage、stun、projectile_vs_building、on_hit_stacking_debuff |
| W | `a34_iceberg_strike` | A34E | supported | 已對照原作 | unit_damage、aoe_circle、slow_debuff、on_hit_stacking_debuff |
| E | `a34_frost_breath` | A34R | supported | 已對照原作 | projectile_linear、projectile_widening、projectile_on_hit_dot、discrete_dot、on_hit_stacking_debuff |
| R | `a34_rapid_cooling` | A34T | supported | 已對照原作 | cdr_on_cast_hook、permanent_stat_bonus、cooldown_cap_buff、mana_on_cast、on_hit_stacking_debuff |

## H04 C17 阿市（`oichi`）

| 欄位 | 技能 | 來源 | 狀態 | 原作對照 | 能力 |
|---|---|---|---|---|---|
| Q | `c17_light_chain` | C17W | supported | 已對照原作 | tether_distance_scaled、unit_damage、root、timed_stat_buff |
| W | `c17_victory_blessing` | C17E | supported | 已對照原作 | global_ally_target、timed_stat_buff |
| E | `c17_divine_strike` | C17R | supported | 已對照原作 | aoe_circle、unit_damage |
| R | `c17_aurora_guard` | C17T | supported | 已對照原作 | delayed_effect、aoe_circle、ally_area_query、heal |
| extra1 | `c17_devotion`（auto_learn） | C17D | supported | 已對照原作 | extra_action_slot、on_cast_trigger、ally_area_query、heal |

## H05 A02 稻姬（`inahime`）

| 欄位 | 技能 | 來源 | 狀態 | 原作對照 | 能力 |
|---|---|---|---|---|---|
| Q | `a02_venom_arrow` | A02W | supported | 已對照原作 | toggle_mana_per_attack、on_attack_hook、slow_debuff |
| W | `a02_wind_prayer` | A02E | supported | 已對照原作 | heal、timed_stat_buff |
| E | `a02_wind_drift` | A02R | supported | 已對照原作 | permanent_stat_bonus |
| R | `a02_shadow_meld` | A02T | supported | 已對照原作 | stealth_attack_no_reveal、timed_stat_buff、no_unit_collision |

## H06 B08 淺井長政（`asai_nagamasa`）

| 欄位 | 技能 | 來源 | 狀態 | 原作對照 | 能力 |
|---|---|---|---|---|---|
| Q | `b08_fang_slash` | B08W | supported | 已對照原作 | projectile_linear、unit_damage、command_restricted |
| W | `b08_flash_cut` | B08E | supported | 已對照原作 | blink_to_target、unit_damage、stun、mana_burn、command_restricted |
| E | `b08_living_soul` | B08R | supported | 已對照原作 | permanent_stat_bonus、on_cast_trigger、stacking_buff |
| R | `b08_sky_fang` | B08T | supported | 已對照原作 | projectile_linear、unit_damage |
| extra1 | `b08_endless_fang`（auto_learn） | B08D | supported | 已對照原作 | extra_action_slot、unit_damage、stun、projectile_linear |

## I1 C15 玉子（`tamako`）

| 欄位 | 技能 | 來源 | 狀態 | 原作對照 | 能力 |
|---|---|---|---|---|---|
| Q | `c15_scatter_shot` | C15W | supported | 已對照原作 | projectile_tracking、aoe_circle、unit_damage、stun |
| W | `c15_burst_arrow` | C15E | supported | 已對照原作 | projectile_linear、ground_area_dot、ground_area_line、fow_viewer |
| E | `c15_precise_light` | C15R | supported | 已對照原作 | permanent_stat_bonus |
| R | `c15_hundred_arrows` | C15T | supported | 已對照原作 | unit_damage、stun、summon_timed、every_nth_attack、retarget_on_death、summon_forced_target、summon_invulnerable、summon_flying_chase |

## I2 B29 村上義清（`murakami_yoshikiyo`）

| 欄位 | 技能 | 來源 | 狀態 | 原作對照 | 能力 |
|---|---|---|---|---|---|
| Q | `b29_ground_slam` | B29W | supported | 已對照原作 | aoe_circle、unit_damage、stun |
| W | `b29_quake_axe` | B29E | supported | 已對照原作 | leap、aoe_circle、unit_damage、slow_debuff、aura、delayed_effect |
| E | `b29_morale_boost` | B29R | supported | 已對照原作 | aura、ally_area_buff_query、permanent_stat_bonus |
| R | `b29_earth_tremor` | B29T | supported | 已對照原作 | channel、random_target_det、aoe_circle、unit_damage、stun、delayed_effect、fow_viewer |
