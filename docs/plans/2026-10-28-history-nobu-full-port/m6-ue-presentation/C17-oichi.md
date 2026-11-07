# oichi（C17）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c17_light_chain` | style 1 | 0（預期 0 個） | c17w_root（root）：0；c17w_guard：status_cue 1 | 有呈現 |
| `c17_victory_blessing` | placeholder 1、style 1 | 1（預期 1 個） | c17e_blessing：status_cue 1 | 有呈現 |
| `c17_divine_strike` | placeholder 1、style 1 | 1（預期 1 個） | — | 有呈現 |
| `c17_aurora_guard` | placeholder 1、style 1 | 1（預期 1 個） | — | 有呈現 |
| `c17_devotion` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/oichi-run`（spotlight，2026-11-01），錄到 1251 幀、掉 0 幀，下了 5 次指令，略過 2 格。
- observer：有檢查，mismatch 0。build：snapshot commit `63a7512d`（E583，一輪建一次）。
- 5 招中 4 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「戰國的光之祈願」。
