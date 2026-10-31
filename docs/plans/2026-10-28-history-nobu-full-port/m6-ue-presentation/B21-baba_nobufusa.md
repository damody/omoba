# baba_nobufusa（B21）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b21_sky_wave` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b21_killing_intent` | style 1 | 0（預期 0 個） | b21e_buff：status_cue 1；b21e_stun：0 | 有呈現 |
| `b21_rebel_armor` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b21_undying_demon` | 0 | 0（預期 0 個） | b21t_bonus：0；b21t_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/baba_nobufusa-run`（spotlight，2026-11-01），錄到 834 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- 4 招中 3 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
