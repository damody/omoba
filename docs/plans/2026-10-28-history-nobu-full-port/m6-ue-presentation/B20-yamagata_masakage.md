# yamagata_masakage（B20）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b20_rock_rush` | style 1 | 0（預期 0 個） | b20w_stun：ring 1 | 有呈現 |
| `b20_ruin_wave` | style 1 | 0（預期 0 個） | b20e_stun：ring 1 | 有呈現 |
| `b20_tenacity` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b20_red_charge` | style 2 | 0（預期 0 個） | b20t_onslaught：ring 1；b20t_slow：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/yamagata_masakage-run`（spotlight，2026-11-01），錄到 755 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 4 招中 3 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
