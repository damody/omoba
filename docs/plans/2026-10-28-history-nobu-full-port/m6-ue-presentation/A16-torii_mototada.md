# torii_mototada（A16）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a16_bloody_grasp` | placeholder 1、style 1 | 1（預期 1 個） | a16w_empower：status_cue 1；a16w_weaken：status_cue 1 | 有呈現 |
| `a16_blood_link` | style 1 | 0（預期 0 個） | a16e_guard：status_cue 1；a16e_link：status_cue 1 | 有呈現 |
| `a16_blood_aura` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `a16_dark_blood_decree` | style 1 | 0（預期 0 個） | a16t_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/torii_mototada-run`（spotlight，2026-10-31，master 佇列；E378／E394／E399／E409 之後），錄到 707 幀、掉 0 幀。
- 血腥之攫、血魄連結、闇血敕令都有呈現；血殤靈光是被動光環，沒有公開狀態。
- 血腥之攫的施放 cue 有 1 次走共用 placeholder 效果（這個 cue 還沒有專屬特效），目標 cue 與狀態 cue 正常。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「伏見城的血守」。
