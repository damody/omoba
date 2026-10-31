# momochi_sandayu（B13）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b13_sense_break` | style 1 | 0（預期 0 個） | b13w_blind：status_cue 1 | 有呈現 |
| `b13_hypnosis` | effect 1、style 1 | 1（預期 1 個） | b13e_sleep：status_cue 1；b13e_wake_stun：0 | 有呈現 |
| `b13_blood_disguise` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b13_burrow` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b13_shukuchi` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b13_storm` | style 1 | 0（預期 0 個） | b13t_slow：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/momochi_sandayu-run`（spotlight，2026-10-31，第二輪重跑；E442、E443 之後），錄到 1246 幀、掉 0 幀，下了 6 次指令，略過 0 格。
- 6 招中 5 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「伊賀的上忍」。
