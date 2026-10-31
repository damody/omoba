# yagyu_muneyoshi（C24）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c24_kamitori` | 0 | 0（預期 1 個） | c24w_guard：0 | 本局沒有呈現 |
| `c24_ukifune` | style 1 | 0（預期 0 個） | c24e_haste：status_cue 1；c24e_silence：ring 1 | 有呈現 |
| `c24_zenken` | style 1 | 0（預期 0 個） | c24r_active：ring 1；c24r_stun：ring 1 | 有呈現 |
| `c24_last_life` | effect 1、style 1 | 1（預期 1 個） | c24t_resolve：ring 2 | 有呈現 |

擊退拋物線（全局）：1 次，最高 10000 cm

- 實機：`target/lineC-pres14/yagyu_muneyoshi-run`（spotlight，2026-10-31，重跑佇列；E402／E409／E429 之後），錄到 822 幀、掉 0 幀。
- 浮舟（加速狀態與沉默圓環）、全劍（啟動與暈眩圓環）、最期一生（目標 cue 1 個、決意圓環）都有呈現。
- 神取沒有呈現：E409 讓射程外（射程 200，替身在 240 外）的單位目標照樣下指令，autopilot 也按下 Q 並點了替身，但輸入層沒有送出任何指令（`not_submitted`：input layer submitted nothing），伺服器沒有收到施法。所以 E409 的假設「交給伺服器先走進射程」在輸入層這一關就沒有成立，A22 焰心鼓動也是同樣的情況；要查明單位目標的點擊為什麼被輸入層吞掉。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「柳生新陰流之祖」。
