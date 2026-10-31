# sasaki_kojiro（C22）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c22_ganryu` | style 1 | 0（預期 0 個） | c22d_frenzy：status_cue 1 | 有呈現 |
| `c22_swallow_nest` | style 1 | 0（預期 0 個） | c22w_vanish：status_cue 1 | 有呈現 |
| `c22_flying_swallow` | style 1 | 0（預期 0 個） | c22e_stun：ring 1 | 有呈現 |
| `c22_swallow_flash` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `c22_dream_mist` | style 2 | 0（預期 0 個） | c22t_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/sasaki_kojiro-run`（spotlight，2026-10-31，第二輪重跑；E442、E443 之後），錄到 922 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- 5 招中 4 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「嚴流的劍聖」。
