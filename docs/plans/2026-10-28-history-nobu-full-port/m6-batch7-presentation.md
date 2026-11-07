# M6 第 7 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 7` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時用 Grok 生成圖（`generated-art-v1.json`）；兩者都沒有時，上架後 HUD sync 自動用共用佔位圖。
- **cue 參考圖**：沒有原作資產的特效 cue 有 Grok 生成的參考圖（`omfue/SourceArt/CueReferences/`），在 Editor 做特效時參考；遊戲內仍播通用佔位特效。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作或生成／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| C13 龍造寺隆信 | 生成 | 4／4 | 3／5 | 1 | 2／2 |
| A05 酒井忠次 | 原作 | 5／5 | 9／10 | 0 | 7／7 |
| A08 榊原康政 | 原作 | 4／4 | 4／4 | 0 | 4／4 |
| C21 宮本武藏 | 原作 | 4／4 | 7／10 | 0 | 4／4 |
| A06 井伊直政 | 原作 | 5／5 | 11／11 | 2 | 4／4 |
| A15 寧寧 | 原作 | 5／5 | 8／12 | 2 | 5／5 |
| B12 山本勘助 | 原作 | 4／4 | 7／8 | 0 | 2／2 |
| C19 松姬 | 原作 | 5／5 | 10／10 | 1 | 4／4 |
| C23 加藤清正 | 生成 | 4／4 | 4／4 | 0 | 4／4 |

## 仍缺的素材（目前用通用替代）

### C13 龍造寺隆信

- 沒有原作資產的 cue（通用佔位特效）：`c13r_wind_fx`（effect，已有生成參考圖）、`c13t_title`（sound）

### A05 酒井忠次

- 沒有原作資產的 cue（通用佔位特效）：`a05d_die_gesture`（aura，已有生成參考圖）

### C21 宮本武藏

- 沒有原作資產的 cue（通用佔位特效）：`c21r_crit_sound`（sound）、`c21r_slam_gesture`（gesture）、`c21t_flash`（effect，已有生成參考圖）

### A15 寧寧

- 沒有原作資產的 cue（通用佔位特效）：`a15e_moonfall`（effect，已有生成參考圖）、`a15r_glaive`（projectile，已有生成參考圖）、`a15t_split_hit_sound`（impact，已有生成參考圖）、`a15w_cast_sound`（sound）

### B12 山本勘助

- 沒有原作資產的 cue（通用佔位特效）：`b12e_heal_sound`（sound）

