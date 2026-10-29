# M6 第 7 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 7` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時，上架後 HUD sync 自動用共用佔位圖。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| C13 龍造寺隆信 | 佔位 | 4／4 | 3／5 | 1 | 2／2 |
| A05 酒井忠次 | 原作 | 5／5 | 9／10 | 0 | 7／7 |
| A08 榊原康政 | 原作 | 4／4 | 4／4 | 0 | 4／4 |
| C21 宮本武藏 | 原作 | 3／4 | 7／10 | 0 | 4／4 |
| A06 井伊直政 | 原作 | 0／5 | 11／11 | 2 | 4／4 |
| A15 寧寧 | 原作 | 5／5 | 8／12 | 2 | 5／5 |
| B12 山本勘助 | 原作 | 4／4 | 7／8 | 0 | 2／2 |
| C19 松姬 | 原作 | 2／5 | 10／10 | 1 | 4／4 |
| C23 加藤清正 | 佔位 | 4／4 | 4／4 | 0 | 4／4 |

## 仍缺的素材（目前用通用替代）

### C13 龍造寺隆信

- 頭像（the site's portrait is Dota 2 art）
- 沒有原作資產的 cue（通用佔位特效）：`c13r_wind_fx`（effect）、`c13t_title`（sound）

### A05 酒井忠次

- 沒有原作資產的 cue（通用佔位特效）：`a05d_die_gesture`（aura）

### C21 宮本武藏

- 技能圖示 `c21_crescent_slash`（Dota built-in texture juggernaut_blade_fury）
- 沒有原作資產的 cue（通用佔位特效）：`c21r_crit_sound`（sound）、`c21r_slam_gesture`（gesture）、`c21t_flash`（effect）

### A06 井伊直政

- 技能圖示 `a06_red_power`（Dota built-in texture dragon_knight_dragon_blood）
- 技能圖示 `a06_red_meteor`（Dota built-in texture invoker_exort）
- 技能圖示 `a06_red_flame_flow`（Dota built-in texture bloodseeker_blood_bath）
- 技能圖示 `a06_flame_of_anger`（Dota built-in texture dragon_knight_breathe_fire）
- 技能圖示 `a06_red_fury`（Dota built-in texture huskar_life_break_alt）

### A15 寧寧

- 沒有原作資產的 cue（通用佔位特效）：`a15e_moonfall`（effect）、`a15r_glaive`（projectile）、`a15t_split_hit_sound`（impact）、`a15w_cast_sound`（sound）

### B12 山本勘助

- 沒有原作資產的 cue（通用佔位特效）：`b12e_heal_sound`（sound）

### C19 松姬

- 技能圖示 `c19_paralyzing_poison`（Dota built-in texture death_prophet_exorcism）
- 技能圖示 `c19_surprise_raid`（Dota built-in texture furion_teleportation）
- 技能圖示 `c19_inviolable`（Dota built-in texture enchantress_enchant）

### C23 加藤清正

- 頭像（the site's portrait is Dota 2 art）

