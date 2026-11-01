# ii_naomasa（A06）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a06_red_power` | style 1 | 0（預期 0 個） | a06w_power：status_cue 1 | 有呈現 |
| `a06_red_meteor` | style 1 | 0（預期 0 個） | a06e_stun：status_cue 1 | 有呈現 |
| `a06_red_flame_flow` | 0 | 0（預期 0 個） | a06r_charge：ring 1 | 有呈現 |
| `a06_flame_of_anger` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a06_red_fury` | style 1 | 0（預期 0 個） | a06t_fury：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/ii_naomasa-run`（spotlight，2026-11-01），錄到 908 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 5 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
