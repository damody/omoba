# M6 第 4 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 4` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時用 Grok 生成圖（`generated-art-v1.json`）；兩者都沒有時，上架後 HUD sync 自動用共用佔位圖。
- **cue 參考圖**：沒有原作資產的特效 cue 有 Grok 生成的參考圖（`omfue/SourceArt/CueReferences/`），在 Editor 做特效時參考；遊戲內仍播通用佔位特效。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作或生成／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| C02 明智秀滿 | 原作 | 4／4 | 14／15 | 1 | 4／4 |
| A33 本多正信 | 生成 | 4／4 | 3／4 | 0 | 5／5 |
| B03 齋藤朝信 | 原作 | 6／6 | 6／8 | 1 | 3／3 |
| B05 毛利元就 | 原作 | 4／4 | 11／11 | 0 | 8／8 |
| B11 菊姬 | 原作 | 6／6 | 5／8 | 0 | 3／3 |
| B09 真田昌幸 | 原作 | 4／4 | 6／10 | 0 | 4／4 |
| B18 湖衣姬 | 生成 | 4／4 | 4／7 | 0 | 4／4 |
| B31 武田信玄 | 生成 | 4／4 | 9／9 | 1 | 7／7 |
| A20 九鬼嘉隆 | 生成 | 4／4 | 4／6 | 1 | 6／6 |

## 仍缺的素材（目前用通用替代）

### C02 明智秀滿

- 沒有原作資產的 cue（通用佔位特效）：`c02t_riptide`（effect，已有生成參考圖）

### A33 本多正信

- 沒有原作資產的 cue（通用佔位特效）：`a33e_ghost_walk_sound`（sound）

### B03 齋藤朝信

- 沒有原作資產的 cue（通用佔位特效）：`b03t_hit_sound`（impact，已有生成參考圖）、`b03w_split_earth`（effect，已有生成參考圖）

### B11 菊姬

- 沒有原作資產的 cue（通用佔位特效）：`b11e_troll_priest_heal`（effect，已有生成參考圖）、`b11f_blade_glow`（effect，已有生成參考圖）、`b11t_echo_slam_gesture`（gesture）

### B09 真田昌幸

- 沒有原作資產的 cue（通用佔位特效）：`b09e_pulse`（effect，已有生成參考圖）、`b09r_toggle_sound`（sound）、`b09t_cast_sound`（sound）、`b09w_cast_sound`（sound）

### B18 湖衣姬

- 沒有原作資產的 cue（通用佔位特效）：`b18t_sound`（sound）、`b18t_title`（sound）、`b18t_vo`（sound）

### A20 九鬼嘉隆

- 沒有原作資產的 cue（通用佔位特效）：`a20r_wave`（projectile，已有生成參考圖）、`a20t_tidehunter_model`（aura，已有生成參考圖）

