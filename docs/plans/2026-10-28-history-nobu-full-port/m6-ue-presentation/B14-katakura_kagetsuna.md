# katakura_kagetsuna（B14）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b14_temporary_night` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b14_iai_slash` | style 1 | 0（預期 0 個） | b14w_stun：0 | 有呈現 |
| `b14_shadow_tactics` | style 1 | 0（預期 0 個） | b14e_silence：status_cue 1；b14e_magic_immune：0 | 有呈現 |
| `b14_attendant_insight` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b14_shadow_cut` | style 1 | 0（預期 0 個） | b14t_power：status_cue 1；b14t2_shadow：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/katakura_kagetsuna-run`（spotlight，2026-10-31，第一輪），錄到 951 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- 5 招中 4 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
