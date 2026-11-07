# akechi_hidemitsu（C02）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c02_mirage_strike` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c02_phantom_move` | style 1 | 0（預期 0 個） | c02e_buff：status_cue 1 | 有呈現 |
| `c02_kyoka_suigetsu` | 0 | 0（預期 0 個） | c02r_recharge：ring 1 | 有呈現 |
| `c02_culling_slash` | effect 2、style 1 | 2（預期 2 個） | c02t_stun：ring 1；c02t_vastness：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/akechi_hidemitsu-run`（spotlight，2026-11-01），錄到 907 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- E492 驗證：上一輪 W 幻影移動把秀滿傳到畫面外，汰選斬擊（R）和被動都被略過。這一輪改成先往目標走到畫面內最遠的可見點（`ClickWorldToward`），4 格全部下了指令，略過 0 格。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「湖水渡」。
