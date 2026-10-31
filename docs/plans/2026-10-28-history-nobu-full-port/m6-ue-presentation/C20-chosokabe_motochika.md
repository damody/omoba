# chosokabe_motochika（C20）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c20_flame_aegis` | style 1 | 0（預期 0 個） | c20w_aegis：status_cue 1 | 有呈現 |
| `c20_demon_blood` | style 1 | 0（預期 0 個） | c20e_regen：ring 1 | 有呈現 |
| `c20_flame_dance` | style 1 | 0（預期 0 個） | c20r_stun：ring 1 | 有呈現 |
| `c20_demon_child` | style 1 | 0（預期 0 個） | c20t_demon：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/chosokabe_motochika-run`（spotlight，2026-10-31，重跑佇列；E402／E409／E429 之後），錄到 825 幀、掉 0 幀，4 招都有下指令（E402 之後 UE 建置不再被其他 session 的 MSBuild 中斷）。
- 炎之加護（加護狀態）、鬼血之流（回復圓環）、炎之舞（暈眩圓環）、鬼若子（變身狀態）都有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「四國的霸主」。
