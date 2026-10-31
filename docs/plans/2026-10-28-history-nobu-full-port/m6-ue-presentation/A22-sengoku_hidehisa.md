# sengoku_hidehisa（A22）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a22_flame_whirl` | style 1 | 0（預期 0 個） | a22w_whirl：status_cue 1 | 有呈現 |
| `a22_flame_heart` | 0 | 0（預期 0 個） | a22e_slow：0 | 本局沒有呈現 |
| `a22_boiling_blood` | 0 | 0（預期 0 個） | a22r_weaken：status_cue 3；a22r_weaken_immune：0 | 有呈現 |
| `a22_torment` | style 1 | 0（預期 0 個） | a22t_torment：status_cue 1；a22t_torment_unit：0 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/sengoku_hidehisa-run`（spotlight，2026-10-31，重跑佇列；E402／E409／E429 之後），錄到 2222 幀、掉 333 幀，4 招都有下指令。
- 旋焰斬（旋轉狀態）、沸血靈光（減攻光環 3 次狀態 cue）、折磨（折磨狀態）都有呈現。
- 焰心鼓動沒有呈現：前一輪替身被沸血靈光燒死，E429 生效（`spotlight_wait_stand_in`），autopilot 等了約 49 秒，等替身復活走回舞台才施放；但這次按下 W 並點替身後，輸入層沒有送出任何指令（`not_submitted`：input layer submitted nothing），伺服器也沒有收到 slot 1 的施法。C24 神取也出現同樣的情況，原因待查，見 C24 報告。
- 這局掉了 333 幀，大多落在等待替身的 49 秒內（同時有其他 session 的建置與實機局）；cue 與狀態的判定看 log，不受影響。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「不屈的焰心」。
