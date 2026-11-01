# omatsu（A34）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a34_frost_bolt` | 0 | 0（預期 0 個） | a34w_freeze（stun）：0 | 本局沒有呈現（沒施放） |
| `a34_iceberg_strike` | placeholder 1、style 1 | 1（預期 1 個） | a34e_slow：status_cue 1 | 有呈現 |
| `a34_frost_breath` | style 1 | 0（預期 0 個） | a34r_frostbite：status_cue 2 | 有呈現 |
| `a34_rapid_cooling` | style 1 | 0（預期 0 個） | a34t_active（a34_rapid_cooling）：status_cue 1；a34t_mr_cut（a34_rapid_cooling_resist_shred）：0 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/omatsu-run`（spotlight，2026-11-01），錄到 859 幀、掉 0 幀，下了 3 次指令，略過 1 格。
- observer：有檢查，mismatch 0。build：snapshot commit `63a7512d`（E583，一輪建一次）。
- 4 招中 3 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「加賀的冰華」。
