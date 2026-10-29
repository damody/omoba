# M6 第 9 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 9` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時用 Grok 生成圖（`generated-art-v1.json`）；兩者都沒有時，上架後 HUD sync 自動用共用佔位圖。
- **cue 參考圖**：沒有原作資產的特效 cue 有 Grok 生成的參考圖（`omfue/SourceArt/CueReferences/`），在 Editor 做特效時參考；遊戲內仍播通用佔位特效。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作或生成／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| B15 今川義元 | 原作 | 5／5 | 8／9 | 1 | 4／4 |
| B28 松永久秀 | 原作 | 4／4 | 10／10 | 2 | 4／4 |
| C07 立花道雪 | 原作 | 6／6 | 9／12 | 0 | 3／3 |
| A13 服部半藏 | 原作 | 6／6 | 13／16 | 1 | 5／5 |
| B07 足利義昭 | 生成 | 5／5 | 4／7 | 0 | 2／2 |
| B16 望月千代女 | 原作 | 9／9 | 14／14 | 0 | 7／7 |
| C08 加藤段藏 | 生成 | 5／5 | 10／13 | 0 | 8／8 |
| C18 小早川隆景 | 生成 | 5／5 | 8／9 | 0 | 4／4 |
| B24 秋山信友 | 原作 | 5／5 | 8／9 | 2 | 5／5 |
| B22 太原雪齋 | 原作 | 5／5 | 5／6 | 0 | 3／3 |
| C06 石川五右衛門 | 生成 | 5／5 | 8／9 | 0 | 4／4 |

## 仍缺的素材（目前用通用替代）

### B15 今川義元

- 沒有原作資產的 cue（通用佔位特效）：`b15d_split_hit`（impact，已有生成參考圖）

### C07 立花道雪

- 沒有原作資產的 cue（通用佔位特效）：`c07d_attack_pose`（effect，已有生成參考圖）、`c07d_pose_freeze`（aura，已有生成參考圖）、`c07t_bolt`（effect，已有生成參考圖）

### A13 服部半藏

- 沒有原作資產的 cue（通用佔位特效）：`a13e_hook_throw_sound`（sound）、`a13t_mirror_image_sound`（sound）、`a13t_title`（sound）

### B07 足利義昭

- 沒有原作資產的 cue（通用佔位特效）：`b07e_teleport_sound`（sound）、`b07t_voice`（sound）、`b07w_impact`（effect，已有生成參考圖）

### C08 加藤段藏

- 沒有原作資產的 cue（通用佔位特效）：`c08r_cast_gesture`（gesture）、`c08t_cheese`（effect，已有生成參考圖）、`c08t_title`（sound）

### C18 小早川隆景

- 沒有原作資產的 cue（通用佔位特效）：`c18t_cast_sound`（sound）

### B24 秋山信友

- 沒有原作資產的 cue（通用佔位特效）：`b24r_avalanche`（projectile，已有生成參考圖）

### B22 太原雪齋

- 沒有原作資產的 cue（通用佔位特效）：`hero_lina_dragon_slave`（effect，已有生成參考圖）

### C06 石川五右衛門

- 沒有原作資產的 cue（通用佔位特效）：`item_money_bag`（effect，已有生成參考圖）

