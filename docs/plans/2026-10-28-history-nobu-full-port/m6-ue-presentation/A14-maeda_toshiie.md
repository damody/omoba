# maeda_toshiie（A14）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a14_impact_spear` | style 1 | 0（預期 0 個） | a14w_stun：ring 2；a14w_slow：status_cue 1 | 有呈現 |
| `a14_divine_strike` | style 1 | 0（預期 0 個） | a14e_stun：ring 1 | 有呈現 |
| `a14_stacked_spear` | 0 | 0（預期 0 個） | a14r_stack：ring 1 | 有呈現 |
| `a14_spear_fury` | style 1 | 0（預期 0 個） | a14t_aspd_stack：ring 1；a14t_rage：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/maeda_toshiie-run`（spotlight，2026-11-01），錄到 969 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
