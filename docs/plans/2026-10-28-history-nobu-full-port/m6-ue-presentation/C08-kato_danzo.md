# kato_danzo（C08）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c08_vanishing_step` | style 1 | 0（預期 0 個） | c08d_stealth：status_cue 1；c08d_invisible：ring 1 | 有呈現 |
| `c08_blood_hunt` | effect 1、style 1 | 1（預期 1 個） | c08w_bleed：ring 1；c08w_silence：ring 1 | 有呈現 |
| `c08_scatter_escape` | style 1 | 0（預期 0 個） | c08e_restrict：ring 1 | 有呈現 |
| `c08_kite_chain` | style 1 | 0（預期 0 個） | c08r_stun：ring 1 | 有呈現 |
| `c08_devour` | style 1 | 0（預期 0 個） | c08t_devoured：status_cue 2；c08t_steal_stack：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/kato_danzo-run`（spotlight，2026-11-01），錄到 968 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 1。build：snapshot commit `a5c11697`（E583，一輪建一次）。
- E598 生效：散逃之後不再卡在牆裡，吞噬也放出來了。偷技能時出現 1 筆 ability_levels mismatch（observer 多了 b29_quake_axe），已轉給 B。
- 5 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
