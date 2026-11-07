# miyoshi_nagayoshi（B10）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b10_wrath_of_the_dead` | style 1 | 0（預期 0 個） | b10w_exorcism：status_cue 1；b10w_slow：status_cue 1 | 有呈現 |
| `b10_corpse_assault` | style 1 | 0（預期 0 個） | b10e_stun：ring 1 | 有呈現 |
| `b10_demonic_war_spirit` | 0 | 0（預期 0 個） | b10r_buff：status_cue 1 | 有呈現 |
| `b10_power_of_the_abyss` | style 1 | 0（預期 0 個） | b10t_armor_cut：status_cue 1；b10t_self：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/miyoshi_nagayoshi-run`（spotlight，2026-10-31，第二輪重跑；E442、E443 之後），錄到 848 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「天下人前夜」。
