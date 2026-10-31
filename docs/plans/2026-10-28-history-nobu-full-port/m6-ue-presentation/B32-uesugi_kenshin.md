# uesugi_kenshin（B32）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b32_divine_slash` | style 1 | 0（預期 0 個） | b32w_stun：status_cue 1；b32w_haste：ring 1 | 有呈現 |
| `b32_divine_flash` | style 1 | 0（預期 0 個） | b32e_stun：status_cue 1 | 有呈現 |
| `b32_divine_formation` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b32_war_god` | style 1 | 0（預期 0 個） | b32d_mark：status_cue 1；b32d_rally：0 | 有呈現 |
| `b32_bishamonten` | style 1 | 0（預期 0 個） | b32t_avatar：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/uesugi_kenshin-run`（spotlight，2026-10-31，第二輪重跑；E442、E443 之後），錄到 1068 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- 5 招中 4 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「越後之龍」。
