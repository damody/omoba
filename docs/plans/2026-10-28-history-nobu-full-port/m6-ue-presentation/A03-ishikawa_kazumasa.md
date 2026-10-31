# ishikawa_kazumasa（A03）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a03_pacing_gu` | style 1 | 0（預期 0 個） | a03w_debuff：status_cue 1 | 有呈現 |
| `a03_spider_form` | style 1 | 0（預期 0 個） | a03e_spider：status_cue 1 | 有呈現 |
| `a03_enchanting_venom` | 0 | 0（預期 0 個） | a03r_poison：status_cue 1 | 有呈現 |
| `a03_insect_swarm` | style 1 | 0（預期 0 個） | — | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/ishikawa_kazumasa-run`（spotlight，2026-10-31，第一輪），錄到 854 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「」。
