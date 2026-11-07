# ishikawa_goemon（C06）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c06_quick_hands` | style 1 | 0（預期 0 個） | c06w_haste：status_cue 1 | 有呈現 |
| `c06_sleight_of_hand` | effect 2、style 1 | 2（預期 2 個） | — | 有呈現 |
| `c06_lavish_strike` | style 1 | 0（預期 0 個） | c06r_shield：status_cue 1 | 有呈現 |
| `c06_murder_and_loot` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `c06_grand_heist` | effect 2、style 1 | 2（預期 2 個） | c06t_mark：status_cue 1；c06t_stun：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/ishikawa_goemon-run`（spotlight，2026-11-01），錄到 1027 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `a88dee83`（E583，一輪建一次）。
- 5 招中 4 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
