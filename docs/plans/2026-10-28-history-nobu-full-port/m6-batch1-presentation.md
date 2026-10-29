# M6 第 1 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 1` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時，上架後 HUD sync 自動用共用佔位圖。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| A01 德川家康 | 佔位 | 3／5 | 11／12 | 0 | 6／6 |
| B20 山縣昌景 | 原作 | 4／4 | 5／6 | 0 | 4／4 |
| B26 大熊朝秀 | 原作 | 0／5 | 13／14 | 0 | 5／5 |
| C22 佐佐木小次郎 | 原作 | 5／5 | 9／11 | 1 | 4／4 |
| A12 大谷吉繼 | 原作 | 2／9 | 14／14 | 1 | 8／8 |
| A14 前田利家 | 原作 | 4／4 | 6／6 | 0 | 6／6 |
| A26 濃姬 | 原作 | 4／4 | 9／13 | 2 | 4／4 |
| B27 小島貞興 | 佔位 | 0／4 | 12／13 | 1 | 6／6 |
| C03 島津義弘 | 佔位 | 3／4 | 7／10 | 0 | 4／4 |

## 仍缺的素材（目前用通用替代）

### A01 德川家康

- 頭像（the site's portrait is Dota 2 art）
- 技能圖示 `a01_heaven_net`（Dota built-in texture meepo_earthbind）
- 技能圖示 `a01_thunder_blade`（Dota built-in texture zuus_lightning_bolt）
- 沒有原作資產的 cue（通用佔位特效）：`a01w_holy_bolt`（effect）

### B20 山縣昌景

- 沒有原作資產的 cue（通用佔位特效）：`b20t_epicenter`（effect）

### B26 大熊朝秀

- 技能圖示 `b26_charging_wave`（Dota built-in texture brewmaster_fire_permanent_immolation）
- 技能圖示 `b26_blood_fury`（Dota built-in texture ember_spirit_sleight_of_fist）
- 技能圖示 `b26_kozuke_valor`（Dota built-in texture brewmaster_drunken_brawler_miss）
- 技能圖示 `b26_unstoppable`（Dota built-in texture brewmaster_drunken_haze）
- 技能圖示 `b26_release_wave`（Dota built-in texture jakiro_liquid_fire）
- 沒有原作資產的 cue（通用佔位特效）：`b26e_model_scale_up`（effect）

### C22 佐佐木小次郎

- 沒有原作資產的 cue（通用佔位特效）：`c22d_cast_sound`（sound）、`c22r_crit_sound`（sound）

### A12 大谷吉繼

- 技能圖示 `a12_scheme`（Dota built-in texture invoker_quas）
- 技能圖示 `a12_mystic_pierce`（Dota built-in texture storm_spirit_static_remnant）
- 技能圖示 `a12_awe`（Dota built-in texture lone_druid_savage_roar_bear）
- 技能圖示 `a12_mystic_arrow`（Dota built-in texture storm_spirit_overload）
- 技能圖示 `a12_mystic_possession`（Dota built-in texture storm_spirit_overload）
- 技能圖示 `a12_mystic_field`（Dota built-in texture storm_spirit_static_remnant）
- 技能圖示 `a12_awe_t`（Dota built-in texture lone_druid_savage_roar_bear）

### A26 濃姬

- 沒有原作資產的 cue（通用佔位特效）：`a26r_crit_gesture`（gesture）、`a26t_big_explosion`（effect）、`a26t_small_explosion`（effect）、`a26w_flak_sound`（sound）

### B27 小島貞興

- 頭像（the site's portrait is Dota 2 art）
- 技能圖示 `b27_horsemanship`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/invoker_tornado.png）
- 技能圖示 `b27_mighty_strike`（Dota built-in texture beastmaster_wild_axes）
- 技能圖示 `b27_fearless_charge`（Dota built-in texture ogre_magi_frost_armor）
- 技能圖示 `b27_rending_blade`（Dota built-in texture beastmaster_primal_roar）
- 沒有原作資產的 cue（通用佔位特效）：`b27r_model_grow`（effect）

### C03 島津義弘

- 頭像（the site's portrait is Dota 2 art）
- 技能圖示 `c03_iron_shirt`（Dota built-in texture ogre_magi_frost_armor）
- 沒有原作資產的 cue（通用佔位特效）：`c03t_hammer`（effect）、`c03t_hammer_hit`（effect）、`c03w_grow`（effect）

