# M6 第 6 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 6` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時，上架後 HUD sync 自動用共用佔位圖。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| A27 柴田勝家 | 原作 | 5／5 | 8／11 | 1 | 3／3 |
| A10 平手政秀 | 佔位 | 5／5 | 11／13 | 1 | 2／2 |
| A25 織田信長 | 原作 | 5／5 | 9／15 | 1 | 5／5 |
| A29 石田三成 | 佔位 | 0／4 | 13／15 | 1 | 6／6 |
| B32 上杉謙信 | 原作 | 5／5 | 14／14 | 1 | 6／6 |
| A31 森蘭丸 | 原作 | 0／4 | 13／15 | 2 | 2／2 |
| C04 阿國 | 原作 | 3／4 | 8／10 | 0 | 5／5 |
| A32 瀧川一益 | 原作 | 0／6 | 4／6 | 1 | 5／5 |
| B17 霧隱才藏 | 原作 | 4／4 | 8／8 | 1 | 10／10 |

## 仍缺的素材（目前用通用替代）

### A27 柴田勝家

- 沒有原作資產的 cue（通用佔位特效）：`a27r_crit_sound`（sound）、`a27r_echo_slam_gesture`（gesture）、`relocate_sound`（sound）

### A10 平手政秀

- 頭像（the site's portrait is Dota 2 art）
- 沒有原作資產的 cue（通用佔位特效）：`a10e_hit_burst`（impact）、`a10e_sword_wave`（projectile）

### A25 織田信長

- 沒有原作資產的 cue（通用佔位特效）：`a25d_message`（effect）、`a25d_press_cast`（effect）、`a25r_crit_sound`（sound）、`a25r_slam_gesture`（gesture）、`a25t_small_tornado`（effect）、`a25t_tornado`（effect）

### A29 石田三成

- 頭像（the site's portrait is Dota 2 art）
- 技能圖示 `a29_sleeve_poison_arrow`（Dota built-in texture viper_viper_strike）
- 技能圖示 `a29_abyss_grip`（Dota built-in texture treant_overgrowth）
- 技能圖示 `a29_withering_aura`（Dota built-in texture enchantress_natures_attendants）
- 技能圖示 `a29_vulture_swarm`（Dota built-in texture death_prophet_exorcism）
- 沒有原作資產的 cue（通用佔位特效）：`a29e_overgrowth_fx`（aura）、`a29w_poison_number`（effect）

### A31 森蘭丸

- 技能圖示 `a31_echoing_rain`（Dota built-in texture mirana_starfall）
- 技能圖示 `a31_shunpo`（Dota built-in texture riki_blink_strike）
- 技能圖示 `a31_storm_arrow`（Dota built-in texture drow_ranger_marksmanship）
- 技能圖示 `a31_samidare`（Dota built-in texture medusa_split_shot）
- 沒有原作資產的 cue（通用佔位特效）：`a31t_split_hit`（effect）、`a31w_impact_sound`（sound）

### C04 阿國

- 技能圖示 `c04_gathering_dance`（Dota built-in texture zuus_lightning_bolt_immortal）
- 沒有原作資產的 cue（通用佔位特效）：`c04r_sound`（sound）、`c04w_sound`（sound）

### A32 瀧川一益

- 技能圖示 `a32_flying_slash`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_w_old.png）
- 技能圖示 `a32_second_slash`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_w_old.png）
- 技能圖示 `a32_soul_drain`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_e.png）
- 技能圖示 `a32_phantom_blade`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_w.png）
- 技能圖示 `a32_soul_blade`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_w.png）
- 技能圖示 `a32_asura_blade`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a32_t.png）
- 沒有原作資產的 cue（通用佔位特效）：`a32e_time_lapse_sound`（sound）、`a32w_thunderclap`（effect）

