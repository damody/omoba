# akechi_hidemitsu（C02）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c02_mirage_strike` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c02_phantom_move` | style 1 | 0（預期 0 個） | c02e_buff：status_cue 1 | 有呈現 |
| `c02_kyoka_suigetsu` | 0 | 0（預期 0 個） | c02r_recharge：ring 1 | 有呈現 |
| `c02_culling_slash` | 0 | 0（預期 2 個） | c02t_stun：0；c02t_vastness：0 | 本局沒有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/akechi_hidemitsu-run`（spotlight，2026-10-31，第二輪重跑；E442、E443 之後），錄到 1268 幀、掉 0 幀，下了 2 次指令，略過 2 格。
- 汰選斬擊（R）沒有示範：W 幻影移動把秀滿傳送到離舞台很遠的地方，鏡頭跟著英雄，替身和原點都落在畫面安全邊界（7%）之外，之後回位（`spotlight_return`）、走近（`spotlight_approach`）和被動攻擊的點擊都被拒（`target is outside the visible world area`，共 16 次走近），最後兩格被略過。已改為先往目標方向走到畫面內最遠的可見點（E492，`ClickWorldToward`），待下一輪重跑確認。
- 4 招中 3 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「湖水渡」。
