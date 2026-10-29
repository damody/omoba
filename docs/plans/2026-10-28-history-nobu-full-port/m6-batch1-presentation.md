# M6 第 1 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 1` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時用 Grok 生成圖（`generated-art-v1.json`）；兩者都沒有時，上架後 HUD sync 自動用共用佔位圖。
- **cue 參考圖**：沒有原作資產的特效 cue 有 Grok 生成的參考圖（`omfue/SourceArt/CueReferences/`），在 Editor 做特效時參考；遊戲內仍播通用佔位特效。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作或生成／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| A01 德川家康 | 生成 | 5／5 | 11／12 | 0 | 6／6 |
| B20 山縣昌景 | 原作 | 4／4 | 5／6 | 0 | 4／4 |
| B26 大熊朝秀 | 原作 | 5／5 | 13／14 | 0 | 5／5 |
| C22 佐佐木小次郎 | 原作 | 5／5 | 9／11 | 1 | 4／4 |
| A12 大谷吉繼 | 原作 | 9／9 | 14／14 | 1 | 8／8 |
| A14 前田利家 | 原作 | 4／4 | 6／6 | 0 | 6／6 |
| A26 濃姬 | 原作 | 4／4 | 9／13 | 2 | 4／4 |
| B27 小島貞興 | 生成 | 4／4 | 12／13 | 1 | 6／6 |
| C03 島津義弘 | 生成 | 4／4 | 7／10 | 0 | 4／4 |

## 仍缺的素材（目前用通用替代）

### A01 德川家康

- 沒有原作資產的 cue（通用佔位特效）：`a01w_holy_bolt`（effect，已有生成參考圖）

### B20 山縣昌景

- 沒有原作資產的 cue（通用佔位特效）：`b20t_epicenter`（effect，已有生成參考圖）

### B26 大熊朝秀

- 沒有原作資產的 cue（通用佔位特效）：`b26e_model_scale_up`（effect，已有生成參考圖）

### C22 佐佐木小次郎

- 沒有原作資產的 cue（通用佔位特效）：`c22d_cast_sound`（sound）、`c22r_crit_sound`（sound）

### A26 濃姬

- 沒有原作資產的 cue（通用佔位特效）：`a26r_crit_gesture`（gesture）、`a26t_big_explosion`（effect，已有生成參考圖）、`a26t_small_explosion`（effect，已有生成參考圖）、`a26w_flak_sound`（sound）

### B27 小島貞興

- 沒有原作資產的 cue（通用佔位特效）：`b27r_model_grow`（effect，已有生成參考圖）

### C03 島津義弘

- 沒有原作資產的 cue（通用佔位特效）：`c03t_hammer`（effect，已有生成參考圖）、`c03t_hammer_hit`（effect，已有生成參考圖）、`c03w_grow`（effect，已有生成參考圖）

