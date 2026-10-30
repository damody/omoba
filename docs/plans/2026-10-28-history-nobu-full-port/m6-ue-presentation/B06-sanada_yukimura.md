# sanada_yukimura（B06）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b06_brave_warriors` | style 1 | 0（預期 0 個） | b06w_rally：status_cue 2 | 有呈現 |
| `b06_flame_fangs` | style 1 | 0（預期 0 個） | b06e_stun（stun）：ring 1 | 有呈現 |
| `b06_mind_eye` | 0 | 0（預期 0 個） | b06r_guard（__spell_guard_active）：0；b06r_cooldown（__spell_guard_blocked）：0 | 本局沒有呈現 |
| `b06_first_warrior` | style 1 | 0（預期 0 個） | b06t_warrior：status_cue 1；b06t_haste：status_cue 2 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres/sanada-run`（spotlight，2026-10-30）。三招主動技都施放並呈現；勇士（b06w_rally）與第一武士（b06t_warrior、b06t_haste）的狀態 cue 在自身與友軍上都出現，炎牙的暈眩走共用的 `stun` 圓環。

- 心眼（b06r_guard，被動：受到技能時擋下）本局沒有敵方對真田施放技能，沒有觸發；不是呈現缺漏。
