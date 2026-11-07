# chacha（C25）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c25_raid` | style 1 | 0（預期 0 個） | c25w_haste：ring 1 | 有呈現 |
| `c25_blood_blade` | 0 | 0（預期 0 個） | c25e_dot：status_cue 1 | 有呈現 |
| `c25_grievous_wound` | effect 1、style 1 | 1（預期 1 個） | c25r_slow：ring 1 | 有呈現 |
| `c25_falling_sakura` | style 1 | 0（預期 0 個） | — | 有呈現 |

擊退拋物線（全局）：1 次，最高 0 cm

- 實機：`target/lineC-pres14/chacha-run`（spotlight，2026-11-01），錄到 850 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
