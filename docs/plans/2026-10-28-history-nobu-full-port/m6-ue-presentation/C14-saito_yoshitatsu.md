# saito_yoshitatsu（C14）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c14_fire_curse` | style 1 | 0（預期 0 個） | c14w_curse：status_cue 1 | 有呈現 |
| `c14_fire_breath` | style 1 | 0（預期 0 個） | c14e_burn：0；c14e_lock：ring 1 | 有呈現 |
| `c14_fire_breath_dragon` | 0 | 0（預期 0 個） | — | 本局沒有呈現 |
| `c14_dragon_vein` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `c14_dragon_form` | style 1 | 0（預期 0 個） | c14t_form：status_cue 1；c14t_burn：status_cue 1；c14t_slow：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/saito_yoshitatsu-run`（spotlight，2026-10-31，master 佇列；E378／E394／E399／E409 之後），錄到 848 幀、掉 0 幀。
- 火靈詛咒、火龍吐息術、火龍化身（化身、燃燒、緩速）都有呈現；龍脈強韌是被動。
- 火龍化身期間換入的 c14_fire_breath_dragon 本局沒有示範：spotlight 只依固定槽位逐一示範，不會在化身期間再放換入的技能（後續可讓 spotlight 腳本示範換入技能）。火龍吐息術的燃燒狀態 c14e_burn 本局沒有觸發。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「化龍的美濃之主」。
