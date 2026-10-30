# inahime（A02）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a02_venom_arrow` | style 2 | 0（預期 0 個） | a02w_poison：status_cue 1 | 有呈現 |
| `a02_wind_prayer` | style 1 | 0（預期 0 個） | a02e_haste（a02_wind_prayer）：status_cue 1 | 有呈現 |
| `a02_wind_drift` | 0 | 0（預期 0 個） | — | 本局沒有呈現 |
| `a02_shadow_meld` | style 1 | 0（預期 0 個） | a02t_meld（a02_shadow_meld）：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres/inahime-run`（spotlight，2026-10-30）。毒矢、風之祈、影隱都有施放與狀態 cue。

- 遊風行身是被動數值（魔抗與攻擊加成），沒有狀態也沒有 cue，不需要呈現。
