# mori_ranmaru（A31）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a31_echoing_rain` | style 2 | 0（預期 0 個） | a31w_slow：ring 1 | 有呈現 |
| `a31_shunpo` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a31_storm_arrow` | style 1 | 0（預期 0 個） | a31r_stun：status_cue 1 | 有呈現 |
| `a31_samidare` | style 1 | 0（預期 0 個） | — | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/mori_ranmaru-run`（spotlight，2026-11-01），錄到 784 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
