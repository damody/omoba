# M6 第 4 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 4` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時，上架後 HUD sync 自動用共用佔位圖。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| C02 明智秀滿 | 原作 | 4／4 | 14／15 | 1 | 4／4 |
| A33 本多正信 | 佔位 | 4／4 | 3／4 | 0 | 5／5 |
| B03 齋藤朝信 | 原作 | 6／6 | 6／8 | 1 | 3／3 |
| B05 毛利元就 | 原作 | 0／4 | 11／11 | 0 | 8／8 |
| B11 菊姬 | 原作 | 0／6 | 5／8 | 0 | 3／3 |
| B09 真田昌幸 | 原作 | 4／4 | 6／10 | 0 | 4／4 |
| B18 湖衣姬 | 佔位 | 4／4 | 4／7 | 0 | 4／4 |
| B31 武田信玄 | 佔位 | 4／4 | 9／9 | 1 | 7／7 |
| A20 九鬼嘉隆 | 佔位 | 4／4 | 4／6 | 1 | 6／6 |

## 仍缺的素材（目前用通用替代）

### C02 明智秀滿

- 沒有原作資產的 cue（通用佔位特效）：`c02t_riptide`（effect）

### A33 本多正信

- 頭像（the site's portrait is Dota 2 art）
- 沒有原作資產的 cue（通用佔位特效）：`a33e_ghost_walk_sound`（sound）

### B03 齋藤朝信

- 沒有原作資產的 cue（通用佔位特效）：`b03t_hit_sound`（impact）、`b03w_split_earth`（effect）

### B05 毛利元就

- 技能圖示 `b05_chain_lightning`（Dota built-in texture zuus_arc_lightning）
- 技能圖示 `b05_heavenly_thunder`（Dota built-in texture zuus_thundergods_wrath）
- 技能圖示 `b05_three_arrows`（Dota built-in texture zuus_lightning_bolt）
- 技能圖示 `b05_holy_light_burst`（Dota built-in texture greevil_purification）

### B11 菊姬

- 技能圖示 `b11_cherry_rain`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_w.png）
- 技能圖示 `b11_breath_control`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_e.png）
- 技能圖示 `b11_interception`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_r.png）
- 技能圖示 `b11_crow_raid`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_f.png）
- 技能圖示 `b11_crow_strike`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_t.png）
- 技能圖示 `b11_inherited_blood`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_d.png）
- 沒有原作資產的 cue（通用佔位特效）：`b11e_troll_priest_heal`（effect）、`b11f_blade_glow`（effect）、`b11t_echo_slam_gesture`（gesture）

### B09 真田昌幸

- 沒有原作資產的 cue（通用佔位特效）：`b09e_pulse`（effect）、`b09r_toggle_sound`（sound）、`b09t_cast_sound`（sound）、`b09w_cast_sound`（sound）

### B18 湖衣姬

- 頭像（the site's portrait is Dota 2 art）
- 沒有原作資產的 cue（通用佔位特效）：`b18t_sound`（sound）、`b18t_title`（sound）、`b18t_vo`（sound）

### B31 武田信玄

- 頭像（the site's portrait is Dota 2 art）

### A20 九鬼嘉隆

- 頭像（the site's portrait is Dota 2 art）
- 沒有原作資產的 cue（通用佔位特效）：`a20r_wave`（projectile）、`a20t_tidehunter_model`（aura）

