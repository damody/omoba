# yagyu_muneyoshi（C24）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c24_kamitori` | effect 1、style 1 | 1（預期 1 個） | c24w_guard：ring 1 | 有呈現 |
| `c24_ukifune` | style 1 | 0（預期 0 個） | c24e_haste：status_cue 1；c24e_silence：ring 1 | 有呈現 |
| `c24_zenken` | style 1 | 0（預期 0 個） | c24r_active：ring 1；c24r_stun：ring 1 | 有呈現 |
| `c24_last_life` | effect 1、style 1 | 1（預期 1 個） | c24t_resolve：ring 2 | 有呈現 |

擊退拋物線（全局）：1 次，最高 10000 cm

- 實機：`target/lineC-pres14/yagyu_muneyoshi-run`（spotlight，2026-10-31，第二輪重跑；E442、E443 之後），錄到 855 幀、掉 0 幀，下了 4 次指令，略過 0 格。
- E442 驗證：神取（射程 200，替身在 240 外）這次先走近（`spotlight_approach` 2 次），進入射程後才施放，伺服器收到施法並播出呈現；上一輪「輸入層沒有送出」的情況沒有再出現。擊退拋物線 1 次來自全劍。
- 4 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「」。
