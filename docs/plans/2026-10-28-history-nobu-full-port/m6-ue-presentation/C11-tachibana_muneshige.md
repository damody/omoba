# tachibana_muneshige（C11）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c11_raikiri` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c11_thunder_speed` | style 1 | 0（預期 0 個） | c11e_haste：status_cue 1 | 有呈現 |
| `c11_thunder_eye` | style 1 | 0（預期 0 個） | c11r_stun：status_cue 1 | 有呈現 |
| `c11_unrivaled` | style 1 | 0（預期 0 個） | c11t_haste：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/tachibana_muneshige-run`（spotlight，2026-10-31，重跑佇列；E402／E409／E429 之後），錄到 839 幀、掉 0 幀，4 招都有下指令。
- 雷切、雷之疾（加速狀態）、雷之眼（暈眩狀態）、西國無雙（加速狀態）都有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「西國無雙」。
