# nohime（A26）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a26_volley` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a26_powder_kick` | style 1 | 0（預期 0 個） | a26e_stun：ring 1 | 有呈現 |
| `a26_twin_guns` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `a26_scorching_bomb` | style 2 | 0（預期 0 個） | a26t_cast_lock：ring 1；a26t_stun：ring 1；a26t_big_bomb：0 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/nohime-run`（spotlight，2026-10-31，master 佇列；E378／E394／E399／E409 之後），錄到 802 幀、掉 0 幀。
- 連銃、火藥踢擊（暈眩圓環）、灼熱爆裂投擲（施法鎖定與暈眩圓環）都有呈現；二丁銃是被動。
- 大爆彈預告 a26t_big_bomb 要在普攻計數差一次時才出現；示範只主動施放灼熱爆裂投擲，普攻次數不夠，本局沒有出現。其餘狀態都有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「美濃的蝮之女」。
