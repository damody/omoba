# M6 第 3 批：UE 呈現準備

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --batch 3` 產生，不要手改。

- **頭像、技能圖示**：原作 nobu 圖（`nobu-icons-v1.json`）；沒有原作圖時用 Grok 生成圖（`generated-art-v1.json`）；兩者都沒有時，上架後 HUD sync 自動用共用佔位圖。
- **cue 參考圖**：沒有原作資產的特效 cue 有 Grok 生成的參考圖（`omfue/SourceArt/CueReferences/`），在 Editor 做特效時參考；遊戲內仍播通用佔位特效。
- **cue**：每個 cue 都有 kind，依 kind 播通用佔位特效。對到原作資產的 cue，可以在 Editor（Om Presentation Cues）指定美術。
- **投射物**：kind 為 projectile 的 cue 用該 cue 的顏色（Editor 覆寫，否則 kind 的佔位色）；沒有 cue 時用預設樣式。
- **狀態**：HUD 狀態列表顯示 v3 名稱與說明。有 `runtime_id` 時依它對應，否則依狀態 id 對應（資料執行器的預設）。

| 英雄 | 頭像 | 技能圖示（原作或生成／全部） | cue（有原作資產／全部） | 投射物 cue | 狀態（有說明／全部） |
|---|---|---|---|---|---|
| C14 齋藤義龍 | 生成 | 5／5 | 6／9 | 1 | 6／6 |
| C24 柳生宗嚴 | 原作 | 4／4 | 10／11 | 0 | 6／6 |
| A19 黑田孝高 | 原作 | 4／4 | 9／9 | 1 | 3／3 |
| A11 齋藤道三 | 原作 | 4／4 | 9／9 | 0 | 5／5 |
| A22 仙石秀久 | 生成 | 4／4 | 9／11 | 1 | 6／6 |
| A30 丹羽長秀 | 原作 | 4／4 | 6／8 | 1 | 4／4 |
| A36 前野長康 | 生成 | 5／5 | 4／6 | 0 | 5／5 |
| B34 武田勝賴 | 原作 | 4／4 | 10／10 | 1 | 4／4 |
| B02 風魔小太郎 | 原作 | 5／5 | 10／11 | 1 | 6／6 |

## 仍缺的素材（目前用通用替代）

### C14 齋藤義龍

- 沒有原作資產的 cue（通用佔位特效）：`c14e_breath`（projectile，已有生成參考圖）、`c14e_hit_sound`（impact，已有生成參考圖）、`c14t_cast_sound`（sound）

### C24 柳生宗嚴

- 沒有原作資產的 cue（通用佔位特效）：`c24w_cast_sound`（sound）

### A22 仙石秀久

- 沒有原作資產的 cue（通用佔位特效）：`a22e_bolt`（projectile，已有生成參考圖）、`a22w_spin_gesture`（gesture）

### A30 丹羽長秀

- 沒有原作資產的 cue（通用佔位特效）：`a30e_toss`（impact，已有生成參考圖）、`a30w_wolf_howl`（effect，已有生成參考圖）

### A36 前野長康

- 沒有原作資產的 cue（通用佔位特效）：`a36e_explosion`（effect，已有生成參考圖）、`a36r_vo`（sound）

### B02 風魔小太郎

- 沒有原作資產的 cue（通用佔位特效）：`b02w_spirit_lance`（effect，已有生成參考圖）

