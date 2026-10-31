# kuroda_yoshitaka（A19）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a19_fire_stratagem` | style 2 | 0（預期 0 個） | a19w_burn：status_cue 1；a19w_burn_building：0 | 有呈現 |
| `a19_chain_fire` | style 1 | 0（預期 0 個） | a19e_burn：status_cue 1 | 有呈現 |
| `a19_fire_echo` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `a19_siege_fire` | style 1 | 0（預期 0 個） | — | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/kuroda_yoshitaka-run`（spotlight，2026-10-31，第二輪重跑；E442、E443 之後），錄到 728 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- 4 招中 3 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「天下的軍師」。
