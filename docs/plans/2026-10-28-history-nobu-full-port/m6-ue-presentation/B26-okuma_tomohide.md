# okuma_tomohide（B26）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b26_charging_wave` | style 1 | 0（預期 0 個） | b26w_charge：status_cue 1；b26w_slow：status_cue 1 | 有呈現 |
| `b26_blood_fury` | style 1 | 0（預期 0 個） | b26e_fury：status_cue 1；b26e_immune：ring 1 | 有呈現 |
| `b26_kozuke_valor` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b26_unstoppable` | style 1 | 0（預期 0 個） | b26t_valor：status_cue 1 | 有呈現 |
| `b26_release_wave` | style 1 | 0（預期 0 個） | — | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/okuma_tomohide-run`（spotlight，2026-11-01），錄到 913 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `64803c87`（E583，一輪建一次）。
- E595：釋放波接在衝鋒波後面示範，這次呈現了。
- 5 招中 4 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
