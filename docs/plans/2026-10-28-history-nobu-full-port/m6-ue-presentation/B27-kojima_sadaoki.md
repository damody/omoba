# kojima_sadaoki（B27）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b27_horsemanship` | style 1 | 0（預期 0 個） | b27w_buff：status_cue 1 | 有呈現 |
| `b27_mighty_strike` | style 1 | 0（預期 0 個） | b27e_mark：ring 1；b27e_stun：ring 1 | 有呈現 |
| `b27_fearless_charge` | style 1 | 0（預期 0 個） | b27r_courage：status_cue 1 | 有呈現 |
| `b27_rending_blade` | style 1 | 0（預期 0 個） | b27t_slow：status_cue 1；b27t_guard：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/kojima_sadaoki-run`（spotlight，2026-10-31，pres14 佇列；v3 產生的 Rust，5facff2e 轉正式）。四招都有呈現：騎術的強化（b27w_armlet_glow）、剛力擊的標記與暈眩圓環、無畏衝鋒的勇氣光環（b27r_b15w3_aura）、裂地刃的緩速（b27t_move_slow）與減傷圓環。狀態 cue 都播 Niagara 特效（art=1），施放由技能 cue style 繪製，沒有跳過任何示範。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上已補齊的資源；3D 身體目前是共用人形 fallback。選角稱號「鬼小島」（17613b2f）。
