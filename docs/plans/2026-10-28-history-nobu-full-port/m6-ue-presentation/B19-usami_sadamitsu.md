# usami_sadamitsu（B19）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b19_water_dragon` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b19_ice_fence` | effect 1、style 2 | 1（預期 1 個） | b19e_stun：status_cue 1 | 有呈現 |
| `b19_echigo_tactics` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b19_kuruma_gakari` | effect 1、style 1 | 1（預期 1 個） | b19t_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/usami_sadamitsu-run`（spotlight，2026-10-31，第二輪重跑；E442、E443 之後），錄到 692 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「越後的軍師」。
