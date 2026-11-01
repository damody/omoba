# akiyama_nobutomo（B24）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b24_earth_stele` | effect 1、style 1 | 1（預期 1 個） | b24w_stele_life：status_cue 1；b24w_ministun：ring 1 | 有呈現 |
| `b24_taunt` | effect 1、style 1 | 1（預期 1 個） | b24e_mark：status_cue 1；b24e_strength：status_cue 1 | 有呈現 |
| `b24_stone_resonance` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b24_earth_prison` | 0 | 0（預期 0 個） | b24t_silence：ring 2 | 有呈現 |
| `b24_shatter` | 0 | 0（預期 0 個） | — | 本局沒有呈現（沒施放） |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/akiyama_nobutomo-run`（spotlight，2026-11-01），錄到 1119 幀、掉 120 幀，下了 4 次指令，略過 1 格。
- observer：有檢查，mismatch 0。build：snapshot commit `6c1251ac`（E583，一輪建一次）。
- 新轉正式（a5c11697）。粉碎（施放後換進 slot）沒有施放，原因待查。
- 5 招中 3 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
