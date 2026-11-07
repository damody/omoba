# ryuzoji_takanobu（C13）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c13_siphon` | style 1 | 0（預期 0 個） | c13w_stun：ring 1 | 有呈現 |
| `c13_tornado_wave` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c13_gale_grip` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `c13_tornado_vortex` | style 1 | 0（預期 0 個） | c13t_hold：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/ryuzoji_takanobu-run`（spotlight，2026-11-01），錄到 807 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 4 招中 3 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
