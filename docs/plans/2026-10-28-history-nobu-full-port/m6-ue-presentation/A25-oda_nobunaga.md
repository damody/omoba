# oda_nobunaga（A25）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a25_lightning_lure` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a25_calamity_thorns` | effect 1、style 2 | 1（預期 1 個） | a25e_stun：ring 1；a25e_attack：ring 1 | 有呈現 |
| `a25_heshikiri` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `a25_crimson_barrier` | style 1 | 0（預期 0 個） | a25t_barrier：status_cue 1 | 有呈現 |
| `a25_tenka_fubu` | 0 | 0（預期 0 個） | a25d_hero_buff：ring 2；a25d_soldier_buff：0 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/oda_nobunaga-run`（spotlight，2026-10-31，master 佇列；E378／E394／E399／E409 之後），錄到 932 幀、掉 0 幀。
- 導雷、厄災之棘（暈眩與強化圓環，目標 cue 1 個）、深紅障壁、天下布武（英雄加速圓環）都有呈現；壓切是被動。
- 天下布武的部隊加速 a25d_soldier_buff 沒有出現：示範舞台沒有兵線。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「第六天魔王」。
