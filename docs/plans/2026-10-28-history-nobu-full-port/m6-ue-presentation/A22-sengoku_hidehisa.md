# sengoku_hidehisa（A22）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a22_flame_whirl` | style 1 | 0（預期 0 個） | a22w_whirl：status_cue 1 | 有呈現 |
| `a22_flame_heart` | style 1 | 0（預期 0 個） | a22e_slow：status_cue 1 | 有呈現 |
| `a22_boiling_blood` | 0 | 0（預期 0 個） | a22r_weaken：status_cue 2；a22r_weaken_immune：0 | 有呈現 |
| `a22_torment` | style 1 | 0（預期 0 個） | a22t_torment：status_cue 1；a22t_torment_unit：0 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/sengoku_hidehisa-run`（spotlight，2026-10-31，第二輪重跑；E442、E443 之後），錄到 1538 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- E442 驗證：焰心鼓動（W，友方單位目標）這次有下指令並呈現；確認點擊改為點友方目前的位置，上一輪輸入層沒有送出的情況沒有再出現。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「不屈的焰心」。
