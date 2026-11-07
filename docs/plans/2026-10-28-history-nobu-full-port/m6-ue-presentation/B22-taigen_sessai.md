# taigen_sessai（B22）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b22_natural_barrier` | 0 | 0（預期 0 個） | — | 本局沒有呈現（施放了但沒畫） |
| `b22_force_of_nature` | style 1 | 0（預期 0 個） | b22e_fly：ring 1 | 有呈現 |
| `b22_purify` | style 1 | 0（預期 0 個） | b22r_mark：0；b22r_slow：status_cue 1 | 有呈現 |
| `b22_wood_spirit` | style 1 | 0（預期 1 個） | — | 有呈現（目標 cue 沒出現） |
| `b22_wrath_of_nature` | effect 1、style 1 | 1（預期 1 個） | — | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/taigen_sessai-run`（spotlight，2026-11-01），錄到 881 幀、掉 0 幀，下了 5 次指令，略過 0 格。
- observer：有檢查，mismatch 0。build：snapshot commit `d2536318`（E583，一輪建一次）。
- E620 生效：自然屏障這次權威端有執行（cast ok），但它沒有 cast cue，岩石是召喚單位，報告目前看不到，判成「施放了但沒畫」。
- 5 招中 4 招有呈現。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。
