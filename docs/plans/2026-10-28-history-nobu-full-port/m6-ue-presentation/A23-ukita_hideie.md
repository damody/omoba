# ukita_hideie（A23）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a23_cursed_blood` | effect 2、style 1 | 2（預期 2 個） | a23w_stun：ring 1 | 有呈現 |
| `a23_soul_bind` | style 1 | 0（預期 0 個） | a23e_bind：status_cue 1 | 有呈現 |
| `a23_falling_fire` | style 1 | 0（預期 0 個） | a23r_burn：status_cue 1 | 有呈現 |
| `a23_yomi_flame` | effect 1、style 1 | 1（預期 1 個） | a23t_curse：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/ukita_hideie-run`（spotlight，2026-10-31，master 佇列；E378／E394／E399／E409 之後），錄到 772 幀、掉 0 幀。
- 惡血咒、靈魂鎖縛、墜火（引導）、黃泉之焰都有呈現；惡血咒與黃泉之焰的目標 cue 數量和預期一致，狀態 cue 播 Niagara 特效。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「備前的鬼術師」。
