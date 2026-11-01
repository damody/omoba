# ashikaga_yoshiaki（B07）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b07_starfall` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b07_battle_line` | placeholder 1、style 1 | 1（預期 1 個） | — | 有呈現 |
| `b07e_camp_regen` | 0 | 0（預期 0 個） | b07e_camp_regen_buff：0 | 本局沒有呈現 |
| `b07_glorious_momentum` | 0 | 0（預期 0 個） | b07r_regen：ring 3 | 有呈現 |
| `b07_reign_over_all` | style 1 | 0（預期 0 個） | — | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/ashikaga_yoshiaki-run`（spotlight，2026-11-01），錄到 783 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `231ccafb`（E583，一輪建一次）。
- 5 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
