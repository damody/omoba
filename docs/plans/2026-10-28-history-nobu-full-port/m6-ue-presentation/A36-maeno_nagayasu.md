# maeno_nagayasu（A36）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a36_shadow_step` | style 1 | 0（預期 0 個） | a36w_invisible：ring 1 | 有呈現 |
| `a36_spike_trap` | style 1 | 0（預期 0 個） | a36e_detector：ring 1；a36e_slow：ring 1 | 有呈現 |
| `a36_detonate` | 0 | 0（預期 0 個） | — | 本局沒有呈現（沒施放） |
| `a36_bitter_brew` | style 1 | 0（預期 0 個） | a36r_brew：status_cue 1 | 有呈現 |
| `a36_deadly_strike` | 0 | 0（預期 0 個） | a36t_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/maeno_nagayasu-run`（spotlight，rerun18，2026-11-01），略過 1 格（detonate）。錄影幀已在跑完後刪除，報告只讀 log。
- observer：有檢查，mismatch 0。build：snapshot commit `fb460511`（E583，一輪建一次）。
- 地雷：spike_trap 在 tick 3571 施放，但 detonate 一直等不到我方的召喚物（`no_own_summon`），tick 4125 跳過。UE 的實體數在施放後 8 秒內沒有增加。已轉 B，UE 下一輪會加 spawn log。
- 5 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
