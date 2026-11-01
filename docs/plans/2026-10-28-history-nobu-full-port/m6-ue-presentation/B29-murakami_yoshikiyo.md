# murakami_yoshikiyo（B29）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b29_ground_slam` | style 1 | 0（預期 0 個） | b29w_stun（stun）：ring 2 | 有呈現 |
| `b29_quake_axe` | style 1 | 0（預期 0 個） | b29e_slow：status_cue 1；b29e_aura：status_cue 1；b29e_armor_break：status_cue 1 | 有呈現 |
| `b29_morale_boost` | 0 | 0（預期 0 個） | b29r_morale（b29_morale）：status_cue 3 | 有呈現 |
| `b29_earth_tremor` | effect 1、style 1 | 1（預期 1 個） | b29t_stun（stun）：ring 2 | 有呈現 |

擊退拋物線（全局）：1 次，最高 20000 cm

- 實機：`target/lineC-pres14/murakami_yoshikiyo-run`（spotlight，2026-11-01），錄到 846 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `63a7512d`（E583，一輪建一次）。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「震地的信濃猛將」。
