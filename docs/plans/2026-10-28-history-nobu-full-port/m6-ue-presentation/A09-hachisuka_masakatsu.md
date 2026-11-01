# hachisuka_masakatsu（A09）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a09_treant_summon` | style 1 | 0（預期 0 個） | a09w_poison：ring 1 | 有呈現 |
| `a09_lignify` | effect 1、style 1 | 1（預期 1 個） | a09e_lignified：status_cue 1 | 有呈現 |
| `a09_forest_pulse` | effect 1、style 1 | 1（預期 1 個） | a09r_momentum：status_cue 1 | 有呈現 |
| `a09_deep_tentacles` | style 1 | 0（預期 0 個） | a09t_tentacle：ring 17；a09t_poison：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/hachisuka_masakatsu-run`（spotlight，2026-11-01），錄到 937 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `a88dee83`（E583，一輪建一次）。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
