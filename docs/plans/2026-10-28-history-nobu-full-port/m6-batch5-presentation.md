# M6 第 5 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 5` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時用 Grok 生成圖（`generated-art-v1.json`）；兩者都沒有時，上架後 HUD sync 自動用共用佔位圖。
- **cue 參考圖**：沒有原作資產的特效 cue 有 Grok 生成的參考圖（`omfue/SourceArt/CueReferences/`），在 Editor 做特效時參考；遊戲內仍播通用佔位特效。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作或生成／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| A03 石川數正 | 原作 | 4／4 | 10／11 | 2 | 3／3 |
| B35 上泉信綱 | 原作 | 4／4 | 11／12 | 1 | 3／3 |
| C20 長宗我部元親 | 原作 | 4／4 | 5／6 | 0 | 4／4 |
| B25 本願寺顯如 | 原作 | 4／4 | 9／9 | 2 | 1／1 |
| B36 直江兼續 | 原作 | 5／5 | 8／9 | 0 | 4／4 |
| C12 大友宗麟 | 原作 | 5／5 | 8／11 | 2 | 4／4 |
| A04 竹中重治 | 原作 | 5／5 | 18／18 | 0 | 8／8 |
| B21 馬場信房 | 原作 | 4／4 | 10／10 | 1 | 4／4 |
| A17 佐佐成政 | 原作 | 4／4 | 5／6 | 0 | 5／5 |
| B10 三好長慶 | 原作 | 4／4 | 11／13 | 1 | 6／6 |
| B19 宇佐美定滿 | 生成 | 4／4 | 6／7 | 0 | 2／2 |
| C11 立花宗茂 | 原作 | 4／4 | 9／12 | 1 | 3／3 |
| C16 果心居士 | 原作 | 5／5 | 7／10 | 1 | 3／3 |

## 仍缺的素材（目前用通用替代）

### A03 石川數正

- 沒有原作資產的 cue（通用佔位特效）：`a03w_tick_sound`（sound）

### B35 上泉信綱

- 沒有原作資產的 cue（通用佔位特效）：`b35r_counter_sound`（sound）

### C20 長宗我部元親

- 沒有原作資產的 cue（通用佔位特效）：`c20t_title`（sound）

### B36 直江兼續

- 沒有原作資產的 cue（通用佔位特效）：`b36e_cast`（effect，已有生成參考圖）

### C12 大友宗麟

- 沒有原作資產的 cue（通用佔位特效）：`c12r_mask`（aura，已有生成參考圖）、`c12t_beam`（projectile，已有生成參考圖）、`c12w_impact`（effect，已有生成參考圖）

### A17 佐佐成政

- 沒有原作資產的 cue（通用佔位特效）：`a17e_cast_sound`（sound）

### B10 三好長慶

- 沒有原作資產的 cue（通用佔位特效）：`b10t_ghost_ship_sound`（sound）、`b10w_spirit_hit_fx`（effect，已有生成參考圖）

### B19 宇佐美定滿

- 沒有原作資產的 cue（通用佔位特效）：`b19t_sound`（channel，已有生成參考圖）

### C11 立花宗茂

- 沒有原作資產的 cue（通用佔位特效）：`c11t_title`（sound）、`c11w_arc_lightning`（effect，已有生成參考圖）、`c11w_bolt`（projectile，已有生成參考圖）

### C16 果心居士

- 沒有原作資產的 cue（通用佔位特效）：`c16r_heal_sound`（sound）、`c16r_spirit_hit_fx`（summon，已有生成參考圖）、`c16t_split_earth`（effect，已有生成參考圖）

