# asai_nagamasa（B08）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b08_fang_slash` | style 1 | 0（預期 0 個） | b08w_lockout（command_restricted）：ring 1 | 有呈現 |
| `b08_flash_cut` | placeholder 1、style 1 | 1（預期 1 個） | b08e_stun（stun）：ring 2；b08e_lockout（command_restricted）：0 | 有呈現 |
| `b08_living_soul` | 0 | 0（預期 0 個） | b08r_buff（b08_living_soul_buff）：status_cue 1 | 有呈現 |
| `b08_sky_fang` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b08_endless_fang` | style 1 | 0（預期 0 個） | b08d_invulnerable（b08_endless_fang_invulnerable）：ring 1；b08d_self_stun（stun）：ring 2；b08d_stun（stun）：ring 2 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/asai_nagamasa-run`（spotlight，2026-11-01），錄到 900 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `63a7512d`（E583，一輪建一次）。
- 5 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「一刀流的近江之主」。
