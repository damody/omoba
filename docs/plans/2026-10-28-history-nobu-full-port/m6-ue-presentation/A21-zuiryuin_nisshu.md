# zuiryuin_nisshu（A21）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a21_mana_steal` | effect 1、style 1 | 1（預期 1 個） | — | 有呈現 |
| `a21_half_blade` | style 2 | 0（預期 0 個） | — | 有呈現 |
| `a21_energy_press` | effect 1、style 1 | 1（預期 1 個） | a21e_stun：ring 1 | 有呈現 |
| `a21_pearl_disc` | style 1 | 0（預期 0 個） | a21r_regen：status_cue 2；a21r_immune：ring 1；a21r_cast_stun：ring 1 | 有呈現 |
| `a21_false_intel` | style 1 | 0（預期 0 個） | a21t_mark：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/zuiryuin_nisshu-run`（spotlight，2026-11-01），錄到 1160 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `607b28ac`（E583，一輪建一次）。
- E592 smart targeting：偽報、盜法不再挑到疊在 stand-in 上的珠盤，5 招全部呈現。
- 5 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
