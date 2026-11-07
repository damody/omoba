# saika_magoichi（B01）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b01_sniper_mode` | style 1 | 0（預期 0 個） | b01w_sniper（b01_sniper_mode）：status_cue 1 | 有呈現 |
| `b01_saika_corps` | placeholder 1、style 1 | 1（預期 1 個） | — | 有呈現 |
| `b01_rain_iron_cannon` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b01_three_stage` | style 1 | 0（預期 0 個） | b01t_stage（b01_three_stage）：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/saika_magoichi-run`（spotlight，2026-11-01），錄到 779 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `63a7512d`（E583，一輪建一次）。
- 4 招中 3 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「千里狙擊手」。
