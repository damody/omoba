# hattori_hanzo（A13）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a13_vanishing_step` | style 1 | 0（預期 0 個） | a13d_stealth：status_cue 1；a13d_invisible：ring 1 | 有呈現 |
| `a13_shadow_illusion` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a13_substitution` | 0 | 0（預期 0 個） | — | 本局沒有呈現（沒施放） |
| `a13_chain_sickle` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a13_iga_ninjutsu` | 0 | 0（預期 0 個） | a13r_debuff：ring 1 | 有呈現 |
| `a13_shadow_slash` | style 2 | 0（預期 0 個） | a13t_shadow：status_cue 1；a13t_blind：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/hattori_hanzo-run`（spotlight，2026-11-01），錄到 1366 幀、掉 0 幀，下了 5 次指令，略過 1 格。
- observer：有檢查，mismatch 0。build：snapshot commit `8ccb4310`（E583，一輪建一次）。
- E610、E615 生效：影惑、隱斬這次呈現了。替身還沒有施放，待查。
- 6 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
