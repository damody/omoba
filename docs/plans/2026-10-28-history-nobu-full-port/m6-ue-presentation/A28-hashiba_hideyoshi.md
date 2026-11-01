# hashiba_hideyoshi（A28）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a28_evil_art` | placeholder 1、style 1 | 1（預期 1 個） | a28w_hex：status_cue 1；a28w_slow：ring 1 | 有呈現 |
| `a28_creation` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a28_binding_swamp` | style 1 | 0（預期 0 個） | a28r_slow：ring 1 | 有呈現 |
| `a28_phoenix_summon` | style 1 | 0（預期 0 個） | a28t_lifetime：ring 1 | 有呈現 |
| `a28_heaven_fire` | 0 | 0（預期 0 個） | — | 本局沒有呈現（沒施放） |
| `a28_spreading_fire` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/hashiba_hideyoshi-run`（spotlight，2026-11-01），錄到 826 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `8ccb4310`（E583，一輪建一次）。
- E613 生效：天火的召喚物指令被 authority 接受，但伺服器沒有施放紀錄（已轉 B）。
- 6 招中 4 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
