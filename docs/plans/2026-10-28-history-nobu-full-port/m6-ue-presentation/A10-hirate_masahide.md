# hirate_masahide（A10）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a10_blink_return` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a10_phase_shift` | effect 1、placeholder 1、style 1 | 2（預期 2 個） | a10w_phase：status_cue 1 | 有呈現 |
| `a10_slaughter_array` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a10_swap` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a10_shockwave` | effect 1、placeholder 1、style 1 | 2（預期 2 個） | a10t_quake：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/hirate_masahide-run`（spotlight，2026-10-31，第一輪），錄到 984 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- 震央光環：大地震動（R）的 a10t_quake 狀態套在替身身上（unit 12，route=status_cue），播出 a10t_move_slow 狀態 cue。
- 5 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「」。
