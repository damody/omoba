# otomo_sorin（C12）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c12_homing_lock` | style 1 | 0（預期 0 個） | c12d_mark：status_cue 1 | 有呈現 |
| `c12_missile_launch` | style 1 | 0（預期 0 個） | c12w_stun：0 | 有呈現 |
| `c12_space_rift` | style 1 | 0（預期 0 個） | c12e_shield：0 | 有呈現 |
| `c12_optical_mask` | style 1 | 0（預期 0 個） | c12r_invis：status_cue 1 | 有呈現 |
| `c12_annihilation_beam` | style 1 | 0（預期 0 個） | — | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/otomo_sorin-run`（spotlight，2026-11-01），錄到 807 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 5 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
