# sakai_tadatsugu（A05）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a05_valiant_power` | style 1 | 0（預期 0 個） | a05w_bash：status_cue 1；a05w_stun：ring 1 | 有呈現 |
| `a05_mikawa_courage` | style 1 | 0（預期 0 個） | a05e_courage：status_cue 1 | 有呈現 |
| `a05_counter_stance` | style 1 | 0（預期 0 個） | a05r_stun：ring 1 | 有呈現 |
| `a05_loyal_secret` | 0 | 0（預期 0 個） | a05d_feign：0 | 本局沒有呈現 |
| `a05_crusade_affidavit` | style 1 | 0（預期 0 個） | a05t_mark：status_cue 1；a05t_oath：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/sakai_tadatsugu-run`（spotlight，2026-10-31，重跑佇列；E402／E409／E429 之後），錄到 1047 幀、掉 0 幀，5 招都有下指令。
- 強悍之力（普攻暈眩狀態與暈眩圓環）、三河之勇、反擊態勢（暈眩圓環）、誓伐狀（標記與誓約兩個狀態）都有呈現。
- 忠之極意是被動，要在受到致死傷害時才觸發假死；spotlight 讓英雄維持高血量（`hold_hp_permille`），所以不會觸發，本局沒有呈現屬於預期。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「德川四天王之首」。
