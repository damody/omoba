# ise_hime（B23）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b23_undead_pact` | 0 | 0（預期 0 個） | b23d_damage_reduction：0 | 本局沒有呈現 |
| `b23_spirit_gathering` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b23_living_soul_shock` | style 1 | 0（預期 0 個） | b23e_shock：ring 1 | 有呈現 |
| `b23_soul_feast` | style 1 | 0（預期 0 個） | b23r_magic_immune：ring 1 | 有呈現 |
| `b23_heroic_spirit_blessing` | style 1 | 0（預期 0 個） | b23t_invulnerable：ring 1；b23t_attack_speed：0 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/ise_hime-run`（spotlight，2026-11-01），錄到 958 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `a88dee83`（E583，一輪建一次）。
- 5 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
