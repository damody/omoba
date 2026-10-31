# takigawa_kazumasu（A32）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a32_flying_slash` | style 1 | 0（預期 0 個） | a32w_stun：ring 1 | 有呈現 |
| `a32_second_slash` | 0 | 0（預期 0 個） | a32d_stun：0 | 本局沒有呈現 |
| `a32_soul_drain` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a32_phantom_blade` | style 1 | 0（預期 0 個） | a32r_void：status_cue 1 | 有呈現 |
| `a32_soul_blade` | style 2 | 0（預期 0 個） | a32f_soul_blade：ring 1 | 有呈現 |
| `a32_asura_blade` | style 1 | 0（預期 0 個） | a32t_slow：ring 1 | 有呈現 |

擊退拋物線（全局）：1 次，最高 30000 cm

- 實機：`target/lineC-pres14/takigawa_kazumasu-run`（spotlight，2026-10-31，重跑佇列；E402／E409／E429 之後），錄到 887 幀、掉 0 幀，5 招都有下指令（含第 6 格 F 的斬魂刀開關）。
- 怨靈飛翔斬（暈眩圓環）、魂、幽魄刀（虛空狀態）、斬魂刀（開關圓環）、修羅吞世刀（緩速圓環）都有呈現。
- 怨靈飛翔斬（二段）是落地後換上的子技能（`slot = "sub"`，不能學），spotlight 只示範可學的格子，所以本局沒有示範。C14、A12 的換招也有同樣的缺口。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「進退皆瀧川」。
