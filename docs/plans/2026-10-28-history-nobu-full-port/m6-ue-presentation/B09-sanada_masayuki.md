# sanada_masayuki（B09）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b09_backlash_rift` | style 1 | 0（預期 0 個） | b09w_counter：status_cue 1 | 有呈現 |
| `b09_mind_burst` | style 1 | 0（預期 0 個） | b09e_burst：status_cue 1 | 有呈現 |
| `b09_will_shield` | style 2 | 0（預期 0 個） | b09r_shield：status_cue 1 | 有呈現 |
| `b09_mind_suppression` | style 2 | 0（預期 0 個） | b09t_silence：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/sanada_masayuki-run`（spotlight，2026-11-01），錄到 791 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
