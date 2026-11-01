# hattori_hanzo（A13）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a13_vanishing_step` | style 1 | 0（預期 0 個） | a13d_stealth：status_cue 1；a13d_invisible：ring 1 | 有呈現 |
| `a13_shadow_illusion` | 0 | 0（預期 0 個） | — | 本局沒有呈現（施放了但沒畫） |
| `a13_substitution` | 0 | 0（預期 0 個） | — | 本局沒有呈現（沒施放） |
| `a13_chain_sickle` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a13_iga_ninjutsu` | 0 | 0（預期 0 個） | a13r_debuff：ring 2 | 有呈現 |
| `a13_shadow_slash` | 0 | 0（預期 0 個） | a13t_shadow：0；a13t_blind：0 | 本局沒有呈現（沒施放） |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/hattori_hanzo-run`（spotlight，2026-11-01），錄到 1138 幀、掉 167 幀，下了 4 次指令，略過 2 格。
- observer：有檢查，mismatch 0。build：snapshot commit `a5c11697`（E583，一輪建一次）。
- E601 生效：影惑之後 owner HUD 不再跳動，observer 0 mismatch。替身排到時 Q 格還在冷卻，勾鎖、隱斬也沒有送出，原因待查。
- 6 招中 3 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
