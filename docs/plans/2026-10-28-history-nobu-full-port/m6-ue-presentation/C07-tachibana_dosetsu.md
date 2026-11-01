# tachibana_dosetsu（C07）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c07_heaven_thunder` | style 2 | 0（預期 0 個） | — | 有呈現 |
| `c07_thunder_shackle` | 0 | 0（預期 0 個） | c07e_bind：0 | 本局沒有呈現 |
| `c07_thunderbolt` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c07_raijin_wrath` | style 2 | 0（預期 0 個） | c07t_static：status_cue 2 | 有呈現 |
| `c07_storm_move` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c07_magic_shield` | style 2 | 0（預期 0 個） | c07d_shield：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/tachibana_dosetsu-run`（spotlight，2026-11-01），錄到 1243 幀、掉 0 幀，下了 5 次指令，略過 1 格。
- observer：有檢查，mismatch 2。build：snapshot commit `a88dee83`（E583，一輪建一次）。
- observer：2 筆 mismatch（DISCLOSED_HERO ability_levels，狀態期間授予的 level-0 `c07_storm_move`），已轉給工作線 B。
- 6 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
