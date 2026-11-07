# kato_kiyomasa（C23）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c23_tainted_frenzy` | style 1 | 0（預期 0 個） | c23w_frenzy：status_cue 1 | 有呈現 |
| `c23_malice_wall` | style 1 | 0（預期 0 個） | c23e_rage：ring 1 | 有呈現 |
| `c23_blood_for_blood` | 0 | 0（預期 0 個） | c23r_on_spell：ring 1 | 有呈現 |
| `c23_gift_of_death` | style 1 | 0（預期 0 個） | c23t_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/kato_kiyomasa-run`（spotlight，2026-11-01），錄到 732 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：每局各自建置（E583 之前）。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
