# takigawa_kazumasu（A32）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a32_flying_slash` | style 1 | 0（預期 0 個） | a32w_stun：ring 1 | 有呈現 |
| `a32_second_slash` | 0 | 0（預期 0 個） | a32d_stun：0 | 本局沒有呈現（沒施放） |
| `a32_soul_drain` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a32_phantom_blade` | style 1 | 0（預期 0 個） | a32r_void：status_cue 1 | 有呈現 |
| `a32_soul_blade` | style 2 | 0（預期 0 個） | a32f_soul_blade：ring 1 | 有呈現 |
| `a32_asura_blade` | style 1 | 0（預期 0 個） | a32t_slow：ring 1 | 有呈現 |

擊退拋物線（全局）：1 次，最高 30000 cm

- 實機：`target/lineC-pres14/takigawa_kazumasu-run`（spotlight，2026-11-01），錄到 1197 幀、掉 0 幀，下了 5 次指令，略過 1 格。
- observer：有檢查，mismatch 0。build：snapshot commit `64803c87`（E583，一輪建一次）。
- 二段斬只在第一段命中後才換入；這局第一段沒命中，所以沒施放。
- 6 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「進退皆瀧川」。
