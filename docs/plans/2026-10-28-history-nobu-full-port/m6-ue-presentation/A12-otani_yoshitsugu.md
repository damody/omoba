# otani_yoshitsugu（A12）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a12_insight_flash` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a12_scheme` | 0 | 0（預期 0 個） | a12d_mark：ring 1 | 有呈現 |
| `a12_mystic_pierce` | style 1 | 0（預期 0 個） | a12e_vital：ring 1；a12e_mana：0 | 有呈現 |
| `a12_awe` | effect 1、style 1 | 1（預期 1 個） | a12r_mark：status_cue 1 | 有呈現 |
| `a12_mystic_arrow` | style 1 | 0（預期 0 個） | a12f_shred：ring 1 | 有呈現 |
| `a12_mystic_possession` | style 2 | 0（預期 0 個） | a12t_form：status_cue 1 | 有呈現 |
| `a12_insight_flash_t` | 0 | 0（預期 1 個） | a12w_t_stun：0 | 本局沒有呈現（沒施放） |
| `a12_mystic_field` | 0 | 0（預期 0 個） | — | 本局沒有呈現（沒施放） |
| `a12_awe_t` | 0 | 0（預期 1 個） | a12r_t_slow：0 | 本局沒有呈現（沒施放） |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/otani_yoshitsugu-run`（spotlight，2026-11-01），錄到 1157 幀、掉 0 幀，下了 6 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `8ccb4310`（E583，一輪建一次）。
- a12 的三招 _t／mystic_field 沒有施放，待查。
- 9 招中 6 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「玄能的智將」。
