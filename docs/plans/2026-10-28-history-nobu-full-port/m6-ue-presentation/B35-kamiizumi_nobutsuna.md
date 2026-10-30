# kamiizumi_nobutsuna（B35）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b35_sword_essence` | style 1 | 0（預期 0 個） | b35w_focus：status_cue 1 | 有呈現 |
| `b35_isshun` | style 1 | 0（預期 0 個） | b35e_slow：status_cue 1 | 有呈現 |
| `b35_muto_dori` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `b35_hiten` | placeholder 1、style 1 | 1（預期 1 個） | b35t_crit：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/kamiizumi_nobutsuna-run`（spotlight，2026-10-31，pres14 佇列；v3 產生的 Rust，5facff2e 轉正式），沒有跳過任何示範。劍之極意的專注光環（b35w_aura_fx）、一瞬的緩速（b35e_move_slow）、秘轉的暴擊緞帶（b35t_ribbons）都播 Niagara 狀態 cue；秘轉的目標 cue 目前是 placeholder 美術。施放由技能 cue style 繪製。
- 無刀取是被動（被普攻時回血並回擊），沒有 cue 也沒有公開狀態。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上已補齊的資源；3D 身體目前是共用人形 fallback。選角稱號「新陰流的劍聖」（17613b2f）。
