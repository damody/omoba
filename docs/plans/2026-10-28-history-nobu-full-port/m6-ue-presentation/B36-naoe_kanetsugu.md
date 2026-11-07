# naoe_kanetsugu（B36）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b36_naoe_seal` | style 1 | 0（預期 0 個） | b36w_slow：status_cue 1 | 有呈現 |
| `b36_footsteps` | 0 | 0（預期 0 個） | — | 本局沒有呈現（被取消） |
| `b36_heart_fence` | style 1 | 0（預期 0 個） | b36e_shield：status_cue 2；b36e_armor：0 | 有呈現 |
| `b36_seven_spears` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b36_aizen_myoo` | style 1 | 0（預期 0 個） | b36t_aizen：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/naoe_kanetsugu-run`（spotlight，2026-11-01），錄到 1012 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `63a7512d`（E583，一輪建一次）。
- 5 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
