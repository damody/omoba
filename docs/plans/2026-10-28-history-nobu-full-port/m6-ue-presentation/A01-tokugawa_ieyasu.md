# tokugawa_ieyasu（A01）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a01_realm_momentum` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `a01_tosho_gongen` | style 1 | 0（預期 0 個） | a01w_haste：status_cue 2 | 有呈現 |
| `a01_heaven_net` | style 1 | 0（預期 0 個） | a01e_root：status_cue 1 | 有呈現 |
| `a01_thunder_blade` | style 1 | 0（預期 0 個） | a01r_thunder：ring 1 | 有呈現 |
| `a01_mikawa_valor` | style 2 | 0（預期 0 個） | a01t_armor：status_cue 2；a01t_rally_aura：status_cue 1；a01t_rally：status_cue 2 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/tokugawa_ieyasu-run`（spotlight，2026-10-31，pres14 佇列；v3 產生的 Rust，5facff2e 轉正式）。四個主動招式都有呈現：東照權現的加速（a01w_healing_ring）、天羅地網的定身（a01e_net）、雷霆刃的狀態圓環、三河之勇的護甲與號令（a01t_ring、a01t_banner），狀態 cue 都播 Niagara 特效（art=1）。施放由技能 cue style 繪製。
- 天下之勢是被動（受傷後回魔回血），沒有 cue 也沒有公開狀態。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上已補齊的資源；3D 身體目前是共用人形 fallback。選角稱號「三河的天下人」（17613b2f）。
