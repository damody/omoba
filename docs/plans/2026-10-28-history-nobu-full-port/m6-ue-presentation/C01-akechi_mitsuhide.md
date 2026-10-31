# akechi_mitsuhide（C01）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c01_blood_blade` | style 1 | 0（預期 0 個） | c01w_frenzy：status_cue 1 | 有呈現 |
| `c01_ground_flame` | style 1 | 0（預期 0 個） | c01e_stun：ring 1 | 有呈現 |
| `c01_blood_flame` | 0 | 0（預期 0 個） | c01r_charge：status_cue 1 | 有呈現 |
| `c01_honnoji` | style 1 | 0（預期 0 個） | c01t_stun：0 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/akechi_mitsuhide-run`（spotlight，2026-10-31，第二輪重跑；E442、E443 之後），錄到 803 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「敵在本能寺」。
