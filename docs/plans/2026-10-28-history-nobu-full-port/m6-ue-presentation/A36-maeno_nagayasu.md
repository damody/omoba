# maeno_nagayasu（A36）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a36_shadow_step` | style 1 | 0（預期 0 個） | a36w_invisible：ring 1 | 有呈現 |
| `a36_spike_trap` | style 1 | 0（預期 0 個） | a36e_detector：ring 1；a36e_slow：ring 1 | 有呈現 |
| `a36_detonate` | 0 | 0（預期 0 個） | — | 本局沒有呈現（沒施放） |
| `a36_bitter_brew` | style 1 | 0（預期 0 個） | a36r_brew：status_cue 1 | 有呈現 |
| `a36_deadly_strike` | 0 | 0（預期 0 個） | a36t_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/maeno_nagayasu-run`（spotlight，rerun20，2026-11-01）。observer 有檢查。build：snapshot commit `a0d43837`。
- 地雷：spike_trap 在 tick 3162 施放成功。之後 spotlight_summon_scan 只看到一座塔（`5:2:alive`），直到 tick 3715 跳過。kind 4 的地雷從來沒有進 bridge frame。B 確認 server 和 replica 都有地雷，所以問題在 client runtime 到 bridge frame 之間，已轉 B。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
