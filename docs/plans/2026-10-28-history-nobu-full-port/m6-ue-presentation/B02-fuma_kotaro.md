# fuma_kotaro（B02）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b02_vanishing_step` | style 1 | 0（預期 0 個） | b02d_stealth：status_cue 1；b02d_invisible：ring 1 | 有呈現 |
| `b02_hazy_shadow` | effect 1、style 1 | 1（預期 1 個） | — | 有呈現 |
| `b02_blazing_path` | style 1 | 0（預期 0 個） | b02e_path：status_cue 1 | 有呈現 |
| `b02_eighteen_shuriken` | style 1 | 0（預期 0 個） | b02r_root：status_cue 1；b02r_ministun：ring 1 | 有呈現 |
| `b02_four_thunders` | style 1 | 0（預期 0 個） | b02t_stun：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/fuma_kotaro-run`（spotlight，2026-11-01），錄到 963 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- 5 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
