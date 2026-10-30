# honda_tadakatsu（A07）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a07_unscathed` | style 1 | 0（預期 0 個） | a07w_convert：ring 1 | 有呈現 |
| `a07_red_thunder` | style 1 | 0（預期 0 個） | a07e_slow：ring 1 | 有呈現 |
| `a07_shattering_spear` | 0 | 0（預期 0 個） | — | 本局沒有呈現 |
| `a07_strongest` | style 1 | 0（預期 0 個） | a07t_strongest：status_cue 1 | 有呈現 |
| `a07_rouse` | style 1 | 0（預期 0 個） | a07d_rouse：status_cue 1 | 有呈現 |
| `a07_rouse_free` | 0 | 0（預期 0 個） | — | 本局沒有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres/honda-run`（spotlight，2026-10-30，修正 E321 後）。四招主動技都施放並呈現；施放用招式樣式（`cast` 事件），狀態走狀態 cue 或圓環。

- 碎擊槍（被動暴擊）與奮起（免費版子技能）不在 spotlight 的施放序列內，本局沒有觸發；不是呈現缺漏。

- 修正前（E321）：戰國最強、奮起、毫髮無傷、赤雷擊減速的狀態 key 為負，圓環與狀態 cue 都畫不出來。
