# kato_danzo（C08）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c08_vanishing_step` | style 1 | 0（預期 0 個） | c08d_stealth：status_cue 1；c08d_invisible：ring 1 | 有呈現 |
| `c08_blood_hunt` | effect 1、style 1 | 1（預期 1 個） | c08w_bleed：ring 1；c08w_silence：ring 1 | 有呈現 |
| `c08_scatter_escape` | style 1 | 0（預期 0 個） | c08e_restrict：ring 1 | 有呈現 |
| `c08_kite_chain` | 0 | 0（預期 0 個） | c08r_stun：0 | 本局沒有呈現 |
| `c08_devour` | 0 | 0（預期 0 個） | c08t_devoured：0；c08t_steal_stack：0 | 本局沒有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/kato_danzo-run`（spotlight，2026-11-01），錄到 1558 幀、掉 0 幀，下了 3 次指令，略過 2 格。
- observer：有檢查，mismatch 0。build：snapshot commit `a88dee83`（E583，一輪建一次）。
- 5 招中 3 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
