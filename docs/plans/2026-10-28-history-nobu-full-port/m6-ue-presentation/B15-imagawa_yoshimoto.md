# imagawa_yoshimoto（B15）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b15_tokai_giant` | style 1 | 0（預期 0 個） | b15w_giant：status_cue 1 | 有呈現 |
| `b15_flame_arrow` | style 2 | 0（預期 0 個） | b15e_orb：status_cue 1；b15e_illusion_orb：0 | 有呈現 |
| `b15_giant_hide` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b15_multishot` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b15_bluff` | style 1 | 0（預期 0 個） | b15t_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/imagawa_yoshimoto-run`（spotlight，2026-11-01），錄到 1008 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `a88dee83`（E583，一輪建一次）。
- 5 招中 3 招有呈現，2 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
