# niwa_nagahide（A30）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a30_wolf_mark` | style 1 | 0（預期 0 個） | a30w_view：ring 1 | 有呈現 |
| `a30_earth_fang` | style 1 | 0（預期 0 個） | a30e_stun：ring 1 | 有呈現 |
| `a30_fire_sea` | 0 | 0（預期 0 個） | a30r_burn：ring 1 | 有呈現 |
| `a30_red_fuji` | style 1 | 0（預期 0 個） | a30t_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/niwa_nagahide-run`（spotlight，2026-10-31，第一輪），錄到 792 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
