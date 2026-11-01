# mochizuki_chiyome（B16）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b16_flying_raid` | style 1 | 0（預期 0 個） | b16w_attack：0 | 有呈現 |
| `b16_falling_leaves` | style 1 | 0（預期 0 個） | b16e_stun：status_cue 1 | 有呈現 |
| `b16_swift_thunder` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b16_summon_hound` | style 2 | 0（預期 0 個） | b16d_body：status_cue 1 | 有呈現 |
| `b16_phase_shift` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b16_beast_union` | style 1 | 0（預期 0 個） | b16t_damage_bonus：ring 2；b16t_union：0 | 有呈現 |
| `b16_mm_phase_shift` | 0 | 0（預期 0 個） | — | 本局沒有呈現（沒施放） |
| `b16_mm_sweep` | 0 | 0（預期 0 個） | b16mmt_root：0 | 本局沒有呈現（沒施放） |
| `b16_mm_stealth` | 0 | 0（預期 0 個） | b16mmf_timer：0 | 本局沒有呈現（沒施放） |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/mochizuki_chiyome-run`（spotlight，2026-11-01），錄到 1396 幀、掉 0 幀，下了 6 次指令，略過 1 格。
- observer：有檢查，mismatch 0。build：snapshot commit `a5c11697`（E583，一輪建一次）。
- 忍犬「月」的技能：同上，E607 修好後在 rerun14 重驗。
- 9 招中 5 招有呈現，1 招被動沒有 cue 或狀態可檢查。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
