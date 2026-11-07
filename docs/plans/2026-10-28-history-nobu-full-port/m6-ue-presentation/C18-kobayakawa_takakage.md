# kobayakawa_takakage（C18）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c18_lone_rider` | style 1 | 0（預期 0 個） | c18w_ride：status_cue 1；c18w_hold：status_cue 1 | 有呈現 |
| `c18_twin_chariot` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c18_three_blades` | style 1 | 0（預期 0 個） | c18r_focus：ring 1 | 有呈現 |
| `c18_chain_stratagem` | 0 | 0（預期 0 個） | c18d_combo：status_cue 1 | 有呈現 |
| `c18_four_cannons` | style 1 | 0（預期 0 個） | — | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/kobayakawa_takakage-run`（spotlight，2026-11-01），錄到 935 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `a88dee83`（E583，一輪建一次）。
- 5 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
