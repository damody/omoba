# saito_yoshitatsu（C14）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c14_fire_curse` | style 1 | 0（預期 0 個） | c14w_curse：status_cue 1 | 有呈現 |
| `c14_fire_breath` | style 1 | 0（預期 0 個） | c14e_burn：0；c14e_lock：ring 1 | 有呈現 |
| `c14_fire_breath_dragon` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c14_dragon_vein` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `c14_dragon_form` | style 1 | 0（預期 0 個） | c14t_form：status_cue 1；c14t_burn：status_cue 1；c14t_slow：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/saito_yoshitatsu-run`（spotlight，2026-11-01），錄到 941 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `64803c87`（E583，一輪建一次）。
- E595：火龍吐息在施放後換進 slot，接著示範。
- 5 招中 4 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「化龍的美濃之主」。
