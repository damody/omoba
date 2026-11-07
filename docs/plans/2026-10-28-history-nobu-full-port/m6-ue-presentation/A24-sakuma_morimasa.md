# sakuma_morimasa（A24）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a24_blood_shield` | 0 | 0（預期 0 個） | a24d_count：ring 1 | 有呈現 |
| `a24_slaughter_shot` | style 1 | 0（預期 0 個） | a24w_slow：status_cue 1 | 有呈現 |
| `a24_blood_charge` | effect 1、style 1 | 1（預期 1 個） | a24e_stun：ring 1 | 有呈現 |
| `a24_blood_resonance` | style 1 | 0（預期 0 個） | a24r_haste：status_cue 1 | 有呈現 |
| `a24_bloodbath` | style 1 | 0（預期 0 個） | a24t_stun：ring 1；a24t_frenzy：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/sakuma_morimasa-run`（spotlight，2026-10-31，第二輪重跑；E442、E443 之後），錄到 981 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- 5 招中 5 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「鬼玄蕃」。
