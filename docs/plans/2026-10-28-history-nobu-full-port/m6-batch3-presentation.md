# M6 第 3 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 3` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時，上架後 HUD sync 自動用共用佔位圖。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| C14 齋藤義龍 | 佔位 | 5／5 | 6／9 | 1 | 6／6 |
| C24 柳生宗嚴 | 原作 | 0／4 | 10／11 | 0 | 6／6 |
| A19 黑田孝高 | 原作 | 0／4 | 9／9 | 1 | 3／3 |
| A11 齋藤道三 | 原作 | 4／4 | 9／9 | 0 | 5／5 |
| A22 仙石秀久 | 佔位 | 0／4 | 9／11 | 1 | 6／6 |
| A30 丹羽長秀 | 原作 | 4／4 | 6／8 | 1 | 4／4 |
| A36 前野長康 | 佔位 | 4／5 | 4／6 | 0 | 5／5 |
| B34 武田勝賴 | 原作 | 0／4 | 10／10 | 1 | 4／4 |
| B02 風魔小太郎 | 原作 | 0／5 | 10／11 | 1 | 6／6 |

## 仍缺的素材（目前用通用替代）

### C14 齋藤義龍

- 頭像（the site's portrait is Dota 2 art）
- 沒有原作資產的 cue（通用佔位特效）：`c14e_breath`（projectile）、`c14e_hit_sound`（impact）、`c14t_cast_sound`（sound）

### C24 柳生宗嚴

- 技能圖示 `c24_kamitori`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_w.png）
- 技能圖示 `c24_ukifune`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_e.png）
- 技能圖示 `c24_zenken`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_r.png）
- 技能圖示 `c24_last_life`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/c24_t.png）
- 沒有原作資產的 cue（通用佔位特效）：`c24w_cast_sound`（sound）

### A19 黑田孝高

- 技能圖示 `a19_fire_stratagem`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a19_w.png）
- 技能圖示 `a19_chain_fire`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a19_e.png）
- 技能圖示 `a19_fire_echo`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a19_r.png）
- 技能圖示 `a19_siege_fire`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/a19_t.png）

### A22 仙石秀久

- 頭像（the site's portrait is Dota 2 art）
- 技能圖示 `a22_flame_whirl`（Dota built-in texture juggernaut_blade_fury）
- 技能圖示 `a22_flame_heart`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/Flame_Guard_icon.png）
- 技能圖示 `a22_boiling_blood`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/Launch_Fire_Spirit_icon.png）
- 技能圖示 `a22_torment`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/Soul_Catcher_icon.png）
- 沒有原作資產的 cue（通用佔位特效）：`a22e_bolt`（projectile）、`a22w_spin_gesture`（gesture）

### A30 丹羽長秀

- 沒有原作資產的 cue（通用佔位特效）：`a30e_toss`（impact）、`a30w_wolf_howl`（effect）

### A36 前野長康

- 頭像（the site's portrait is Dota 2 art）
- 技能圖示 `a36_deadly_strike`（file outside nobu's custom folders: game/dota_addons/nobu/resource/flash3/images/spellicons/dota2/b11_r.png）
- 沒有原作資產的 cue（通用佔位特效）：`a36e_explosion`（effect）、`a36r_vo`（sound）

### B34 武田勝賴

- 技能圖示 `b34_wrath_axe`（Dota built-in texture beastmaster_wild_axes）
- 技能圖示 `b34_spike_wall`（Dota built-in texture sandking_burrowstrike_alt）
- 技能圖示 `b34_frenzy`（Dota built-in texture beastmaster_inner_beast）
- 技能圖示 `b34_tenmoku_howl`（Dota built-in texture beastmaster_primal_roar）

### B02 風魔小太郎

- 技能圖示 `b02_vanishing_step`（Dota built-in texture riki_smoke_screen）
- 技能圖示 `b02_hazy_shadow`（Dota built-in texture dark_seer_wall_of_replica）
- 技能圖示 `b02_blazing_path`（Dota built-in texture ogre_magi_ignite）
- 技能圖示 `b02_eighteen_shuriken`（Dota built-in texture meepo_earthbind）
- 技能圖示 `b02_four_thunders`（Dota built-in texture shadow_shaman_ether_shock）
- 沒有原作資產的 cue（通用佔位特效）：`b02w_spirit_lance`（effect）

