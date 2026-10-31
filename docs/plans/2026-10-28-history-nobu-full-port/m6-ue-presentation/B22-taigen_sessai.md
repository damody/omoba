# taigen_sessai（B22）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b22_natural_barrier` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b22_force_of_nature` | 0 | 0（預期 0 個） | b22e_fly：0 | 本局沒有呈現 |
| `b22_purify` | style 1 | 0（預期 0 個） | b22r_mark：0；b22r_slow：status_cue 1 | 有呈現 |
| `b22_wood_spirit` | 0 | 0（預期 1 個） | — | 本局沒有呈現 |
| `b22_wrath_of_nature` | effect 1、style 1 | 1（預期 1 個） | — | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/taigen_sessai-run`（spotlight，2026-10-31，第一輪），錄到 576 幀、掉 0 幀，下了 3 次指令，略過 0 格。
- 這一輪用的是舊腳本：自然之力（W）和 D 以樹為目標，當時 spotlight 不能瞄準樹，沒有示範（E459 列為未呈現）。之後已加入 nobu 地圖的樹木舞台與樹木瞄準（c2db3f34），已排入下一輪重跑。
- 5 招中 3 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「」。
