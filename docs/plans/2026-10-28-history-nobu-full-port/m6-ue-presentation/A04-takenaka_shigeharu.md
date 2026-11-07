# takenaka_shigeharu（A04）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a04_water_shikigami` | style 1 | 0（預期 0 個） | a04w_tech：ring 2；a04w_leash：ring 2；a04w_building_freeze：0 | 有呈現 |
| `a04_ice_path` | style 1 | 0（預期 0 個） | a04e_stun：status_cue 1 | 有呈現 |
| `a04_frost_shelter` | placeholder 1、style 1 | 1（預期 1 個） | a04r_shield：status_cue 1；a04r_freeze：0 | 有呈現 |
| `a04_frost_sparrow` | 0 | 0（預期 0 個） | a04d_frost：status_cue 1 | 有呈現 |
| `a04_blizzard` | style 1 | 0（預期 0 個） | a04t_frost：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/takenaka_shigeharu-run`（spotlight，2026-11-01），錄到 947 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 5 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
