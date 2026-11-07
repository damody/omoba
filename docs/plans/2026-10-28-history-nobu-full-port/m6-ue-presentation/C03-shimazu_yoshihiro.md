# shimazu_yoshihiro（C03）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c03_iron_shirt` | style 1 | 0（預期 0 個） | c03w_iron：status_cue 1 | 有呈現 |
| `c03_quake_axe` | style 1 | 0（預期 0 個） | c03e_slow：status_cue 1 | 有呈現 |
| `c03_thunder_wrath` | 0 | 0（預期 0 個） | c03r_stun：ring 1 | 有呈現 |
| `c03_iron_hammer` | style 1 | 0（預期 0 個） | c03t_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/shimazu_yoshihiro-run`（spotlight，2026-11-01），錄到 753 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
