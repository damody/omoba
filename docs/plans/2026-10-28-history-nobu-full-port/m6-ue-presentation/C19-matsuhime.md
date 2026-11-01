# matsuhime（C19）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c19_alluring_gaze` | effect 1、placeholder 1、style 1 | 2（預期 2 個） | c19w_silence：ring 1 | 有呈現 |
| `c19_gentle_vow` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c19_paralyzing_poison` | style 1 | 0（預期 0 個） | c19r_poison：0；c19r_lock：ring 1 | 有呈現 |
| `c19_surprise_raid` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c19_inviolable` | style 1 | 0（預期 0 個） | c19t_aura：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/matsuhime-run`（spotlight，rerun20，2026-11-01，props 舞台）。observer 有檢查。build：snapshot commit `a0d43837`（E583，一輪建一次）。
- 奇襲（E630）：props 舞台有一隻不參戰的友方小兵，autopilot 用 ally_anchor 把施法點落在它身上。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上。
