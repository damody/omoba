# takeda_katsuyori（B34）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b34_wrath_axe` | style 1 | 0（預期 0 個） | b34w_stun：ring 1 | 有呈現 |
| `b34_spike_wall` | style 1 | 0（預期 0 個） | b34e_spikes：ring 1 | 有呈現 |
| `b34_frenzy` | style 1 | 0（預期 0 個） | b34r_frenzy：status_cue 1 | 有呈現 |
| `b34_tenmoku_howl` | 0 | 0（預期 0 個） | b34t_slow_instance：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/takeda_katsuyori-run`（spotlight，2026-11-01），錄到 832 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `63a7512d`（E583，一輪建一次）。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
