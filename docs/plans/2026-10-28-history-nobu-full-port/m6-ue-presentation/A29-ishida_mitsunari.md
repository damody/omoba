# ishida_mitsunari（A29）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a29_sleeve_poison_arrow` | style 1 | 0（預期 0 個） | a29w_poison：status_cue 1 | 有呈現 |
| `a29_abyss_grip` | style 1 | 0（預期 0 個） | a29e_overgrowth：status_cue 1；a29e_grip：ring 1 | 有呈現 |
| `a29_withering_aura` | 0 | 0（預期 0 個） | a29r_drain：status_cue 1；a29r_slow：0 | 有呈現 |
| `a29_vulture_swarm` | style 1 | 0（預期 0 個） | a29t_exorcism：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/ishida_mitsunari-run`（spotlight，2026-10-31，第一輪），錄到 779 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
