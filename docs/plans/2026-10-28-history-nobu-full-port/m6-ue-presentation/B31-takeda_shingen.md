# takeda_shingen（B31）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b31_swift_wind` | style 1 | 0（預期 0 個） | b31w_haste：ring 2；b31w_aura_buff：ring 2 | 有呈現 |
| `b31_still_forest` | style 1 | 0（預期 0 個） | b31e_aura_buff：ring 2 | 有呈現 |
| `b31_raging_fire` | style 1 | 0（預期 0 個） | b31r_power：ring 1；b31r_aura：status_cue 1；b31r_burn：ring 1 | 有呈現 |
| `b31_immovable_mountain` | style 1 | 0（預期 0 個） | b31t_mountain：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/takeda_shingen-run`（spotlight，2026-11-01），錄到 802 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `a88dee83`（E583，一輪建一次）。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
