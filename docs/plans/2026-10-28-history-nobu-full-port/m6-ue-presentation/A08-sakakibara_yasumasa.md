# sakakibara_yasumasa（A08）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a08_skull_crusher` | placeholder 1、style 1 | 1（預期 1 個） | a08w_haste：ring 1 | 有呈現 |
| `a08_war_cry` | style 1 | 0（預期 0 個） | a08e_regen：ring 1；a08e_stun：ring 1 | 有呈現 |
| `a08_indomitable` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `a08_steadfast_general` | style 1 | 0（預期 0 個） | a08t_guard：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/sakakibara_yasumasa-run`（spotlight，2026-11-01），錄到 863 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `a88dee83`（E583，一輪建一次）。
- 4 招中 3 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
