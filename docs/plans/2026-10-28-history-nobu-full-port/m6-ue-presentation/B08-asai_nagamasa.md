# asai_nagamasa（B08）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b08_fang_slash` | style 1 | 0（預期 0 個） | b08w_lockout（command_restricted）：ring 1 | 有呈現 |
| `b08_flash_cut` | placeholder 1、style 1 | 1（預期 1 個） | b08e_stun（stun）：ring 2；b08e_lockout（command_restricted）：0 | 有呈現 |
| `b08_living_soul` | 0 | 0（預期 0 個） | b08r_buff（b08_living_soul_buff）：status_cue 1 | 有呈現 |
| `b08_sky_fang` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b08_endless_fang` | style 1 | 0（預期 0 個） | b08d_invulnerable（b08_endless_fang_invulnerable）：ring 1；b08d_self_stun（stun）：ring 2；b08d_stun（stun）：ring 2 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres/asai_nagamasa-run`（spotlight，2026-10-30）。五招都有呈現：牙斬的封鎖、閃斬的目標 cue 與暈眩、活魂的狀態 cue、無盡牙的無敵與暈眩圓環。

- 閃斬的 b08e_lockout（command_restricted）本局沒出現：目標同時被暈眩，共用狀態只顯示一個圓環。
