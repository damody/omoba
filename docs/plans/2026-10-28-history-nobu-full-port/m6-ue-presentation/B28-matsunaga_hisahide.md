# matsunaga_hisahide（B28）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b28_phoenix_flame` | style 1 | 0（預期 0 個） | b28w_burn：ring 2 | 有呈現 |
| `b28_flame_shackle` | placeholder 1、style 1 | 1（預期 1 個） | b28e_stun：status_cue 1 | 有呈現 |
| `b28_fire_stream` | style 1 | 0（預期 0 個） | b28r_flame：status_cue 1；b28r_burn：ring 1 | 有呈現 |
| `b28_serpent_fire` | style 1 | 0（預期 0 個） | — | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/matsunaga_hisahide-run`（spotlight，2026-11-01），錄到 772 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `a88dee83`（E583，一輪建一次）。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
