# M6 第 2 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 2` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時用 Grok 生成圖（`generated-art-v1.json`）；兩者都沒有時，上架後 HUD sync 自動用共用佔位圖。
- **cue 參考圖**：沒有原作資產的特效 cue 有 Grok 生成的參考圖（`omfue/SourceArt/CueReferences/`），在 Editor 做特效時參考；遊戲內仍播通用佔位特效。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作或生成／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| C05 前田慶次 | 原作 | 4／4 | 13／14 | 1 | 4／4 |
| C09 立花誾千代 | 生成 | 4／4 | 7／7 | 0 | 5／5 |
| A18 稻葉一鐵 | 原作 | 4／4 | 7／7 | 0 | 6／6 |
| B30 柿崎景家 | 生成 | 4／4 | 6／7 | 1 | 4／4 |
| A16 鳥居元忠 | 原作 | 4／4 | 9／9 | 0 | 5／5 |
| A24 佐久間盛政 | 生成 | 5／5 | 11／12 | 1 | 6／6 |
| C01 明智光秀 | 原作 | 4／4 | 7／11 | 0 | 4／4 |
| A23 宇喜多秀家 | 生成 | 4／4 | 8／9 | 0 | 4／4 |
| A35 島左近 | 生成 | 4／4 | 6／7 | 0 | 3／3 |

## 仍缺的素材（目前用通用替代）

### C05 前田慶次

- 沒有原作資產的 cue（通用佔位特效）：`c05e_impact`（impact，已有生成參考圖）

### B30 柿崎景家

- 沒有原作資產的 cue（通用佔位特效）：`b30r_windwalk_sound`（sound）

### A24 佐久間盛政

- 沒有原作資產的 cue（通用佔位特效）：`a24_blood_spray`（effect，已有生成參考圖）

### C01 明智光秀

- 沒有原作資產的 cue（通用佔位特效）：`c01e_light_strike`（effect，已有生成參考圖）、`c01r_crit_sound`（sound）、`c01r_slam_gesture`（gesture）、`c01w_slam_gesture`（gesture）

### A23 宇喜多秀家

- 沒有原作資產的 cue（通用佔位特效）：`a23t_cast`（effect，已有生成參考圖）

### A35 島左近

- 沒有原作資產的 cue（通用佔位特效）：`a35t_cast_sound`（sound）

