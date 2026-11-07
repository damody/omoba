# shibata_katsuie（A27）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a27_unstoppable_advance` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a27_war_banner` | style 1 | 0（預期 0 個） | a27ee_attack_speed：status_cue 2 | 有呈現 |
| `a27_way_of_warrior` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `a27_tyrant_hammer` | style 1 | 0（預期 0 個） | a27d_stun：ring 1 | 有呈現 |
| `a27_oni_shibata` | style 1 | 0（預期 0 個） | a27t_oni：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/shibata_katsuie-run`（spotlight，2026-11-01），錄到 1039 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- 5 招中 4 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
