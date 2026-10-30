# saika_magoichi（B01）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b01_sniper_mode` | style 1 | 0（預期 0 個） | b01w_sniper（b01_sniper_mode）：status_cue 1 | 有呈現 |
| `b01_saika_corps` | placeholder 1、style 1 | 1（預期 1 個） | — | 有呈現 |
| `b01_rain_iron_cannon` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b01_three_stage` | style 1 | 0（預期 0 個） | b01t_stage（b01_three_stage）：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres/saika_magoichi-run`（spotlight，2026-10-30，queue4）。四招中三招都有呈現：狙擊模式與三段擊有狀態 cue，雜賀眾有目標 cue（placeholder 美術）。
- 雨鐵炮是被動技（普攻附加物理傷害），沒有 cue 也沒有公開狀態，畫面上只有傷害數字；這份報告不檢查傷害數字。
