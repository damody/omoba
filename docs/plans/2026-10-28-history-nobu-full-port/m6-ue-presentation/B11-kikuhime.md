# kikuhime（B11）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b11_cherry_rain` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b11_breath_control` | style 1 | 0（預期 0 個） | b11e_heal：0；b11e_agi：ring 1 | 有呈現 |
| `b11_interception` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b11_crow_raid` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b11_crow_strike` | style 1 | 0（預期 0 個） | b11t_enable：ring 1 | 有呈現 |
| `b11_inherited_blood` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/kikuhime-run`（spotlight，2026-11-01），錄到 1249 幀、掉 0 幀，下了 6 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `a88dee83`（E583，一輪建一次）。
- 6 招中 4 招有呈現，2 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
