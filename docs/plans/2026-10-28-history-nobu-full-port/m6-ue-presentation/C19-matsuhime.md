# matsuhime（C19）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c19_alluring_gaze` | effect 1、placeholder 1、style 1 | 2（預期 2 個） | c19w_silence：ring 1 | 有呈現 |
| `c19_gentle_vow` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c19_paralyzing_poison` | style 1 | 0（預期 0 個） | c19r_poison：0；c19r_lock：ring 1 | 有呈現 |
| `c19_surprise_raid` | 0 | 0（預期 0 個） | — | 本局沒有呈現 |
| `c19_inviolable` | style 1 | 0（預期 0 個） | c19t_aura：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/matsuhime-run`（spotlight，2026-11-01），錄到 900 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 5 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
