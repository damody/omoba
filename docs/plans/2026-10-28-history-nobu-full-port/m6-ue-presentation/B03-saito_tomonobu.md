# saito_tomonobu（B03）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b03_zhongkui_strike` | style 1 | 0（預期 0 個） | b03w_stun：ring 1 | 有呈現 |
| `b03_zhongkui_strike_mid` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b03_zhongkui_strike_far` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b03_echigo_vajra` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b03_heavy_blow` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b03_zhongkui_judgment` | style 1 | 0（預期 0 個） | b03t_judgment：status_cue 1；b03t_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/saito_tomonobu-run`（spotlight，2026-10-31，第一輪），錄到 1165 幀、掉 0 幀，下了 6 次指令，略過 0 格。
- 6 招中 4 招有呈現，2 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「」。
