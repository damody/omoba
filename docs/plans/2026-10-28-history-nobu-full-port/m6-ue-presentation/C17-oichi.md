# oichi（C17）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c17_light_chain` | style 1 | 0（預期 0 個） | c17w_root（root）：ring 1；c17w_guard：0 | 有呈現 |
| `c17_victory_blessing` | placeholder 1、style 1 | 1（預期 1 個） | c17e_blessing：status_cue 1 | 有呈現 |
| `c17_divine_strike` | placeholder 1、style 1 | 1（預期 1 個） | — | 有呈現 |
| `c17_aurora_guard` | placeholder 1、style 1 | 1（預期 1 個） | — | 有呈現 |
| `c17_devotion` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres/oichi-run`（spotlight，2026-10-30，queue4）。四個主動招式都有呈現：光之鎖的定身圓環、勝利祝福的狀態 cue、神聖打擊與極光守護的目標 cue（placeholder 美術）。
- 依技能文字，光之鎖的 c17w_guard（減傷）屬於連到友方英雄的分支；spotlight 是對敵方施放，本局沒有觸發，所以是 0，不是缺陷。它的 runtime id 依施法者區分（`c17_light_chain_guard:{caster}`），之後要在友方目標的局裡再確認一次。
- 自我奉獻是被動技（施放後治療／回魔），沒有 cue 也沒有公開狀態；治療數字不在這份報告的檢查範圍。
