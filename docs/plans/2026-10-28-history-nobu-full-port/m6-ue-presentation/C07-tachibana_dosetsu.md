# tachibana_dosetsu（C07）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c07_heaven_thunder` | style 2 | 0（預期 0 個） | — | 有呈現 |
| `c07_thunder_shackle` | style 1 | 0（預期 0 個） | c07e_bind：status_cue 1 | 有呈現 |
| `c07_thunderbolt` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c07_raijin_wrath` | style 2 | 0（預期 0 個） | c07t_static：status_cue 1 | 有呈現 |
| `c07_storm_move` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c07_magic_shield` | style 2 | 0（預期 0 個） | c07d_shield：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/tachibana_dosetsu-run`（spotlight，2026-11-01），錄到 1157 幀、掉 0 幀，下了 6 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `64803c87`（E583，一輪建一次）。
- E595：雷雲移動接在雷神之怒後面示範。
- 6 招中 6 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
