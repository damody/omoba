# kosokabe_chikayasu（C10）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c10_black_dragon_wave` | style 1 | 0（預期 0 個） | c10w_slow：0 | 有呈現 |
| `c10_energy_drain` | effect 1、style 1 | 1（預期 1 個） | c10e_stun：ring 1 | 有呈現 |
| `c10_echo_sword` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `c10_day_night_shift` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c10_formless_heaven` | style 1 | 0（預期 0 個） | c10t_slow：status_cue 1；c10t_haste：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/kosokabe_chikayasu-run`（spotlight，2026-11-01），錄到 1030 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 5 招中 4 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
