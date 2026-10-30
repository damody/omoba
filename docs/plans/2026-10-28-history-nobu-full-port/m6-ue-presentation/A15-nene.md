# nene（A15）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a15_shadow_step` | style 1 | 0（預期 0 個） | a15d_stealth：status_cue 1；a15d_invisible：ring 1 | 有呈現 |
| `a15_exhaust_seal` | style 1 | 0（預期 0 個） | a15w_seal：status_cue 1 | 有呈現 |
| `a15_moon_strike` | style 1 | 0（預期 0 個） | a15e_stun：ring 1 | 有呈現 |
| `a15_shadow_windmill` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `a15_shadow_moon_assault` | style 1 | 0（預期 0 個） | a15t_volley：status_cue 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/nene-run`（spotlight，2026-10-31，pres14 佇列；v3 產生的 Rust，5facff2e 轉正式），沒有跳過任何示範。迷蹤步的隱形（a15d_vendetta_start 與隱身外觀圓環）、竭印（a15w_seal）、月擊的暈眩圓環、影月攻勢（a15t_glow）都有呈現，狀態 cue 播 Niagara 特效；施放由技能 cue style 繪製。
- 影風車是被動（普攻彈跳），沒有 cue 也沒有公開狀態。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上已補齊的資源；3D 身體目前是共用人形 fallback。選角稱號「影月的女忍」（17613b2f）。
