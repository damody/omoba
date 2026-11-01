# hattori_hanzo（A13）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a13_vanishing_step` | style 1 | 0（預期 0 個） | a13d_stealth：status_cue 1；a13d_invisible：ring 1 | 有呈現 |
| `a13_shadow_illusion` | 0 | 0（預期 0 個） | — | 本局沒有呈現 |
| `a13_substitution` | 0 | 0（預期 0 個） | — | 本局沒有呈現 |
| `a13_chain_sickle` | 0 | 0（預期 0 個） | — | 本局沒有呈現 |
| `a13_iga_ninjutsu` | 0 | 0（預期 0 個） | a13r_debuff：ring 2 | 有呈現 |
| `a13_shadow_slash` | 0 | 0（預期 0 個） | a13t_shadow：0；a13t_blind：0 | 本局沒有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/hattori_hanzo-run`（spotlight，2026-11-01），錄到 1174 幀、掉 0 幀，下了 3 次指令，略過 2 格。
- observer：有檢查，mismatch 0。build：snapshot commit `607b28ac`（E583，一輪建一次）。
- 影惑出現後本機 HUD 快照交替，之後的指令在本地被拒（已轉 B）。替身是幻象的技能（E595 處理）。
- 6 招中 2 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
