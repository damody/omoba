# kashin_koji（C16）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c16_soul_sever` | effect 1、style 1 | 1（預期 1 個） | — | 有呈現 |
| `c16_curse_shadow` | style 1 | 0（預期 0 個） | c16e_curse：status_cue 1 | 有呈現 |
| `c16_wandering_souls` | style 1 | 0（預期 0 個） | c16r_exorcism：status_cue 1 | 有呈現 |
| `c16_phantom_sight` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c16_eight_gates` | style 1 | 0（預期 0 個） | c16t_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/kashin_koji-run`（spotlight，2026-11-01），錄到 957 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 5 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
