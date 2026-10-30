# shima_sakon（A35）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a35_flame_raid` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `a35_sealing_blaze` | style 1 | 0（預期 0 個） | a35e_stun：ring 1 | 有呈現 |
| `a35_flame_swiftness` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查 |
| `a35_scorching_fire` | style 1 | 0（預期 0 個） | a35t_burning：ring 1；a35t_stun：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/shima_sakon-run`（spotlight，2026-10-31，master 佇列；E378／E394／E399／E409 之後），錄到 864 幀、掉 110 幀。
- 焰襲、熾焰封魔（暈眩圓環）、焦灼之火（燃燒與暈眩圓環）都有呈現。炎之迅捷是被動，沒有 cue 也沒有公開狀態。
- 這局錄影掉了 110 幀，應該是錄影寫檔跟不上（同時有其他 session 的建置）；cue 與狀態的判定看 log，不受影響。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上；3D 身體目前是共用人形 fallback。選角稱號「治部的鬼左近」。
