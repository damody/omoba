# sassa_narimasa（A17）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a17_shrapnel` | style 1 | 0（預期 0 個） | a17w_shock_slow：ring 1；a17w_slow：ring 1 | 有呈現 |
| `a17_rapid_fire` | style 1 | 0（預期 0 個） | a17e_rapid：ring 1 | 有呈現 |
| `a17_vital_shot` | 0 | 0（預期 0 個） | a17r_stun：ring 1 | 有呈現 |
| `a17_great_cannon` | style 2 | 0（預期 0 個） | a17t_cannon：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/sassa_narimasa-run`（spotlight，2026-11-01），錄到 812 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
