# mori_motonari（B05）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b05_chain_lightning` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b05_heavenly_thunder` | style 1 | 0（預期 0 個） | b05e_wrath：status_cue 6 | 有呈現 |
| `b05_three_arrows` | effect 1、style 1 | 1（預期 1 個） | b05r_armor：status_cue 2；b05r_stun：0；b05r_revealed：ring 1 | 有呈現 |
| `b05_holy_light_burst` | style 1 | 0（預期 0 個） | b05t_lock：ring 1；b05t_stun1：ring 1；b05t_stun2：ring 1；b05t_blind：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/mori_motonari-run`（spotlight，2026-11-01），錄到 716 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
