# kirigakure_saizo（B17）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b17_death_ward` | style 1 | 0（預期 0 個） | b17w_dormant：status_cue 1；b17w_active：0；b17w_invisible：0；b17w_detector：0；b17w_stun：0；b17w_hit_stun：0 | 有呈現 |
| `b17_tooth_poison` | style 1 | 0（預期 0 個） | b17e_venom：0；b17e_poison：status_cue 1 | 有呈現 |
| `b17_shadow_leap` | effect 1、style 1 | 1（預期 1 個） | — | 有呈現 |
| `b17_heartseeker` | style 1 | 0（預期 0 個） | b17t_stalk：status_cue 1；b17t_invisible：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/kirigakure_saizo-run`（spotlight，2026-11-01），錄到 836 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `231ccafb`（E583，一輪建一次）。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
