# M6 第 8 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 8` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時，上架後 HUD sync 自動用共用佔位圖。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| B14 片倉景綱 | 原作 | 5／5 | 4／6 | 0 | 5／5 |
| B23 伊勢姬 | 原作 | 5／5 | 8／11 | 1 | 5／5 |
| C10 香宗我部親泰 | 原作 | 0／5 | 9／11 | 1 | 4／4 |
| C25 茶茶 | 佔位 | 4／4 | 5／5 | 0 | 3／3 |
| A28 羽柴秀吉 | 原作 | 4／6 | 11／13 | 1 | 4／4 |
| B33 最上義姬 | 原作 | 2／4 | 20／20 | 2 | 4／4 |
| A21 瑞龍院日秀 | 佔位 | 5／5 | 7／9 | 1 | 5／5 |
| B13 百地三太夫 | 原作 | 0／6 | 4／8 | 0 | 4／4 |
| A09 蜂須賀政勝 | 原作 | 4／4 | 8／9 | 0 | 5／5 |

## 仍缺的素材（目前用通用替代）

### B14 片倉景綱

- 沒有原作資產的 cue（通用佔位特效）：`b14d_night_sound`（sound）、`b14r_slam_gesture`（gesture）

### B23 伊勢姬

- 沒有原作資產的 cue（通用佔位特效）：`b23d_ghost_death`（effect）、`b23r_heal_sound`（sound）、`b23t_title`（sound）

### C10 香宗我部親泰

- 技能圖示 `c10_black_dragon_wave`（Dota built-in texture faceless_void_time_dilation）
- 技能圖示 `c10_energy_drain`（Dota built-in texture antimage_blink）
- 技能圖示 `c10_echo_sword`（Dota built-in texture antimage_mana_break）
- 技能圖示 `c10_day_night_shift`（Dota built-in texture faceless_void_time_lock）
- 技能圖示 `c10_formless_heaven`（Dota built-in texture luna_eclipse）
- 沒有原作資產的 cue（通用佔位特效）：`c10e_manavoid_cast`（effect）、`c10r_cleave`（effect）

### C25 茶茶

- 頭像（the site's portrait is Dota 2 art）

### A28 羽柴秀吉

- 技能圖示 `a28_creation`（Dota built-in texture zuus_arc_lightning）
- 技能圖示 `a28_spreading_fire`（Dota built-in texture medusa_split_shot）
- 沒有原作資產的 cue（通用佔位特效）：`a28tw_fire_projectile`（projectile）、`a28w_frog_model`（aura）

### B33 最上義姬

- 技能圖示 `b33_voodoo_power`（Dota built-in texture bristleback_viscous_nasal_goo）
- 技能圖示 `b33_spider_attack`（Dota built-in texture broodmother_insatiable_hunger）

### A21 瑞龍院日秀

- 頭像（the site's portrait is Dota 2 art）
- 沒有原作資產的 cue（通用佔位特效）：`a21d_mana_break_sound`（sound）、`a21w_attack_projectile`（aura）

### B13 百地三太夫

- 技能圖示 `b13_sense_break`（Dota built-in texture ogre_magi_ignite）
- 技能圖示 `b13_hypnosis`（Dota built-in texture winter_wyvern_winters_curse）
- 技能圖示 `b13_blood_disguise`（Dota built-in texture giant_wolf_critical_strike）
- 技能圖示 `b13_burrow`（Dota built-in texture rubick_telekinesis_land）
- 技能圖示 `b13_shukuchi`（Dota built-in texture kunkka_x_marks_the_spot）
- 技能圖示 `b13_storm`（Dota built-in texture brewmaster_storm_cyclone）
- 沒有原作資產的 cue（通用佔位特效）：`b13d_burrow_animation`（aura）、`b13d_crater_model`（aura）、`b13e_bell_toll`（effect）、`b13t_tornado_sound`（sound）

### A09 蜂須賀政勝

- 沒有原作資產的 cue（通用佔位特效）：`a09e_hex`（effect）

