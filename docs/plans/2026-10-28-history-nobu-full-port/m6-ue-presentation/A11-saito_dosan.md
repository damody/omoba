# saito_dosan（A11）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a11_vortex` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a11_evil_gaze` | placeholder 1、style 1 | 1（預期 1 個） | a11e_mesmerize：status_cue 1；a11e_int_down：ring 1；a11e_int_up：ring 1 | 有呈現 |
| `a11_old_fox` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `a11_myriad_forms` | style 2 | 0（預期 0 個） | a11t_aura：status_cue 1；a11t_debuff：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/saito_dosan-run`（spotlight，2026-11-01），錄到 773 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- 4 招中 3 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
