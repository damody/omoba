# M6 第 9 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 9` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時，上架後 HUD sync 自動用共用佔位圖。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| B15 今川義元 | 原作 | 0／5 | 8／9 | 1 | 4／4 |
| B28 松永久秀 | 原作 | 4／4 | 10／10 | 2 | 4／4 |
| C07 立花道雪 | 原作 | 0／6 | 9／12 | 0 | 3／3 |
| A13 服部半藏 | 原作 | 5／6 | 13／16 | 1 | 5／5 |
| B07 足利義昭 | 佔位 | 3／5 | 4／7 | 0 | 2／2 |
| B16 望月千代女 | 原作 | 9／9 | 14／14 | 0 | 7／7 |
| C08 加藤段藏 | 佔位 | 5／5 | 10／13 | 0 | 8／8 |
| C18 小早川隆景 | 佔位 | 5／5 | 8／9 | 0 | 4／4 |
| B24 秋山信友 | 原作 | 0／5 | 8／9 | 2 | 5／5 |
| B22 太原雪齋 | 原作 | 5／5 | 5／6 | 0 | 3／3 |
| C06 石川五右衛門 | 佔位 | 5／5 | 8／9 | 0 | 4／4 |

## 仍缺的素材（目前用通用替代）

### B15 今川義元

- 技能圖示 `b15_tokai_giant`（Dota built-in texture ogre_magi_frost_armor）
- 技能圖示 `b15_flame_arrow`（Dota built-in texture clinkz_searing_arrows）
- 技能圖示 `b15_giant_hide`（Dota built-in texture tiny_craggy_exterior）
- 技能圖示 `b15_multishot`（Dota built-in texture windrunner_focusfire）
- 技能圖示 `b15_bluff`（Dota built-in texture tiny_grow）
- 沒有原作資產的 cue（通用佔位特效）：`b15d_split_hit`（impact）

### C07 立花道雪

- 技能圖示 `c07_heaven_thunder`（Dota built-in texture disruptor_thunder_strike）
- 技能圖示 `c07_thunder_shackle`（Dota built-in texture razor_plasma_field）
- 技能圖示 `c07_thunderbolt`（Dota built-in texture zuus_lightning_bolt_immortal）
- 技能圖示 `c07_raijin_wrath`（Dota built-in texture razor_eye_of_the_storm）
- 技能圖示 `c07_storm_move`（Dota built-in texture razor_eye_of_the_storm）
- 技能圖示 `c07_magic_shield`（Dota built-in texture silencer_global_silence）
- 沒有原作資產的 cue（通用佔位特效）：`c07d_attack_pose`（effect）、`c07d_pose_freeze`（aura）、`c07t_bolt`（effect）

### A13 服部半藏

- 技能圖示 `a13_shadow_slash`（Dota built-in texture riki_tricks_of_the_trade）
- 沒有原作資產的 cue（通用佔位特效）：`a13e_hook_throw_sound`（sound）、`a13t_mirror_image_sound`（sound）、`a13t_title`（sound）

### B07 足利義昭

- 頭像（the site's portrait is Dota 2 art）
- 技能圖示 `b07_starfall`（Dota built-in texture mirana_starfall）
- 技能圖示 `b07e_camp_regen`（Dota built-in texture enchantress_natures_attendants）
- 沒有原作資產的 cue（通用佔位特效）：`b07e_teleport_sound`（sound）、`b07t_voice`（sound）、`b07w_impact`（effect）

### C08 加藤段藏

- 頭像（the site's portrait is Dota 2 art）
- 沒有原作資產的 cue（通用佔位特效）：`c08r_cast_gesture`（gesture）、`c08t_cheese`（effect）、`c08t_title`（sound）

### C18 小早川隆景

- 頭像（the site's portrait is Dota 2 art）
- 沒有原作資產的 cue（通用佔位特效）：`c18t_cast_sound`（sound）

### B24 秋山信友

- 技能圖示 `b24_earth_stele`（Dota built-in texture earthshaker_fissure_alt）
- 技能圖示 `b24_taunt`（Dota built-in texture earthshaker_enchant_totem）
- 技能圖示 `b24_stone_resonance`（Dota built-in texture earthshaker_echo_slam）
- 技能圖示 `b24_earth_prison`（Dota built-in texture earthshaker_fissure）
- 技能圖示 `b24_shatter`（Dota built-in texture earthshaker_fissure）
- 沒有原作資產的 cue（通用佔位特效）：`b24r_avalanche`（projectile）

### B22 太原雪齋

- 沒有原作資產的 cue（通用佔位特效）：`hero_lina_dragon_slave`（effect）

### C06 石川五右衛門

- 頭像（the site's portrait is Dota 2 art）
- 沒有原作資產的 cue（通用佔位特效）：`item_money_bag`（effect）

