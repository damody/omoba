# sakai_tadatsugu（A05）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a05_valiant_power` | style 1 | 0（預期 0 個） | a05w_bash：status_cue 1；a05w_stun：ring 1 | 有呈現 |
| `a05_mikawa_courage` | style 1 | 0（預期 0 個） | a05e_courage：status_cue 1 | 有呈現 |
| `a05_counter_stance` | style 1 | 0（預期 0 個） | a05r_stun：ring 1 | 有呈現 |
| `a05_loyal_secret` | 0 | 0（預期 0 個） | a05d_feign：0 | 本局沒有呈現（沒施放） |
| `a05_crusade_affidavit` | style 1 | 0（預期 0 個） | a05t_mark：status_cue 1；a05t_oath：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/sakai_tadatsugu-run`（spotlight，2026-11-01），錄到 998 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `63a7512d`（E583，一輪建一次）。
- 5 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「德川四天王之首」。
