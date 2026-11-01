# kuki_yoshitaka（A20）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a20_pirate_domain` | style 1 | 0（預期 0 個） | a20w_buff：ring 2；a20w_invisible：ring 1 | 有呈現 |
| `a20_water_burst` | style 1 | 0（預期 0 個） | a20e_slow：status_cue 1；a20e_stun：ring 1 | 有呈現 |
| `a20_pirate_boots` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `a20_pirate_daimyo` | style 1 | 0（預期 0 個） | a20t_form：ring 1；a20t_model：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/kuki_yoshitaka-run`（spotlight，2026-11-01），錄到 828 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `63a7512d`（E583，一輪建一次）。
- 4 招中 3 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
