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

- 實機：`target/lineC-pres14/hashiba_hideyoshi-run`（spotlight，rerun18，2026-11-01），略過 0 格。錄影幀已在跑完後刪除，報告只讀 log。
- observer：有檢查，mismatch 0。build：snapshot commit `fb460511`（E583，一輪建一次）。
- 天火（B 的 E627 加 log 後）：伺服器最後一行是 `OwnedSummonOrder applied ... ability='a28_heaven_fire'`（tick 6583），之後沒有任何施放或拒絕紀錄。已轉 B。
- 6 招中 4 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
