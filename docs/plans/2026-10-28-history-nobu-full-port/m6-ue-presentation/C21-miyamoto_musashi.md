# miyamoto_musashi（C21）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c21_crescent_slash` | style 1 | 0（預期 0 個） | c21w_blade_fury：status_cue 1 | 有呈現 |
| `c21_chain_breaker` | effect 2、style 1 | 2（預期 2 個） | c21e_combo：ring 1；c21e_stun：ring 1 | 有呈現 |
| `c21_two_heavens` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `c21_soaring_quake` | effect 1、style 1 | 1（預期 1 個） | c21t_omnislash：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/miyamoto_musashi-run`（spotlight，2026-11-01），錄到 843 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `a88dee83`（E583，一輪建一次）。
- 4 招中 3 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「二天一流」。
