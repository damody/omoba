# kakizaki_kageie（B30）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b30_reverse_shield` | style 1 | 0（預期 0 個） | b30w_shield：status_cue 1 | 有呈現 |
| `b30_soul_breaker` | style 1 | 0（預期 0 個） | b30e_debuff：status_cue 1 | 有呈現 |
| `b30_flash_strike` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b30_purgatory_slash` | style 1 | 0（預期 0 個） | b30t_burn：status_cue 1；b30t_lock：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/kakizaki_kageie-run`（spotlight，2026-10-30，pres14 佇列；v3 產生的 Rust，5facff2e 轉正式）。四招都有呈現：逆壓之盾的盾牌狀態 cue、碎魂劍的減益狀態 cue、閃擊的施放 cue、邪王煉獄斬的點燃狀態 cue 與鎖定圓環。
- 美術：施放由技能 cue style 繪製；狀態 cue 有 Niagara 特效。頭像、技能圖示、載入圖由 HUD sync 依 id 接上已補齊的資源；3D 身體目前是共用人形 fallback。
- 選角稱號與介紹：「越後的先鋒猛將」（17613b2f）。
