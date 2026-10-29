# M6 第 2 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 2` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時，上架後 HUD sync 自動用共用佔位圖。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| C05 前田慶次 | 原作 | 0／4 | 13／14 | 1 | 4／4 |
| C09 立花誾千代 | 佔位 | 4／4 | 7／7 | 0 | 5／5 |
| A18 稻葉一鐵 | 原作 | 4／4 | 7／7 | 0 | 6／6 |
| B30 柿崎景家 | 佔位 | 4／4 | 6／7 | 1 | 4／4 |
| A16 鳥居元忠 | 原作 | 0／4 | 9／9 | 0 | 5／5 |
| A24 佐久間盛政 | 佔位 | 5／5 | 11／12 | 1 | 6／6 |
| C01 明智光秀 | 原作 | 4／4 | 7／11 | 0 | 4／4 |
| A23 宇喜多秀家 | 佔位 | 4／4 | 8／9 | 0 | 4／4 |
| A35 島左近 | 佔位 | 3／4 | 6／7 | 0 | 3／3 |

## 仍缺的素材（目前用通用替代）

### C05 前田慶次

- 技能圖示 `c05_matsukaze`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/invoker_tornado.png）
- 技能圖示 `c05_thunder_dance`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/obsidian_destroyer_arcane_orb.png）
- 技能圖示 `c05_thunder_spear`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/obsidian_destroyer_astral_imprisonment.png）
- 技能圖示 `c05_flash_thunder`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/obsidian_destroyer_sanity_eclipse.png）
- 沒有原作資產的 cue（通用佔位特效）：`c05e_impact`（impact）

### C09 立花誾千代

- 頭像（the site's portrait is Dota 2 art）

### B30 柿崎景家

- 頭像（the site's portrait is Dota 2 art）
- 沒有原作資產的 cue（通用佔位特效）：`b30r_windwalk_sound`（sound）

### A16 鳥居元忠

- 技能圖示 `a16_bloody_grasp`（Dota built-in texture shadow_demon_shadow_poison）
- 技能圖示 `a16_blood_link`（Dota built-in texture shadow_demon_demonic_purge）
- 技能圖示 `a16_blood_aura`（Dota built-in texture ursa_enrage）
- 技能圖示 `a16_dark_blood_decree`（Dota built-in texture earthshaker_fissure_egset）

### A24 佐久間盛政

- 頭像（the site's portrait is Dota 2 art）
- 沒有原作資產的 cue（通用佔位特效）：`a24_blood_spray`（effect）

### C01 明智光秀

- 沒有原作資產的 cue（通用佔位特效）：`c01e_light_strike`（effect）、`c01r_crit_sound`（sound）、`c01r_slam_gesture`（gesture）、`c01w_slam_gesture`（gesture）

### A23 宇喜多秀家

- 頭像（the site's portrait is Dota 2 art）
- 沒有原作資產的 cue（通用佔位特效）：`a23t_cast`（effect）

### A35 島左近

- 頭像（the site's portrait is Dota 2 art）
- 技能圖示 `a35_scorching_fire`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_r.png）
- 沒有原作資產的 cue（通用佔位特效）：`a35t_cast_sound`（sound）

