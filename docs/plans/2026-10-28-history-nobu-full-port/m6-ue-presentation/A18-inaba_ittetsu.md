# inaba_ittetsu（A18）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a18_obsessive_pressure` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a18_suppress` | style 1 | 0（預期 0 個） | a18e_drain：ring 1；a18e_stun：ring 1；a18e_gain：ring 1 | 有呈現 |
| `a18_stubborn_tactics` | 0 | 0（預期 0 個） | a18r_shield：status_cue 1 | 有呈現 |
| `a18_by_force` | style 1 | 0（預期 0 個） | a18t_stun：ring 1；a18t_self_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/inaba_ittetsu-run`（spotlight，2026-10-31，master 佇列重跑；E378／E394／E399 之後），整段示範錄到 790 幀、沒有掉幀。執念壓迫、打壓（吸取、暈眩、增益三個狀態圓環）、無理矢理（目標暈眩與自身暈眩）的施放由技能 cue style 繪製，狀態 cue 播 Niagara 特效。
- 頑強戰法是被動（暴擊），本身沒有施放 cue；它的護盾狀態 a18r_shield 由狀態 cue 呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「頑固一徹」（17613b2f）。
