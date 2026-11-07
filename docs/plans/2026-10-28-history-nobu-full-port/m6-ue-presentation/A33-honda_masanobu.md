# honda_masanobu（A33）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a33_dark_summoning` | style 1 | 0（預期 0 個） | a33w_stun：ring 1；a33w_skeleton_decay：ring 5 | 有呈現 |
| `a33_ghost_step` | style 1 | 0（預期 0 個） | a33e_ghost_walk：status_cue 1 | 有呈現 |
| `a33_corrupting_aura` | 0 | 0（預期 0 個） | a33r_armor_cut：ring 11 | 有呈現 |
| `a33_deepen_wounds` | style 1 | 0（預期 0 個） | a33t_mark：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/honda_masanobu-run`（spotlight，2026-10-31，第一輪），錄到 680 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
