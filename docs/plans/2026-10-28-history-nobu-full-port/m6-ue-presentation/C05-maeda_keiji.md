# maeda_keiji（C05）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `c05_matsukaze` | style 1 | 0（預期 0 個） | c05w_haste：status_cue 1 | 有呈現 |
| `c05_thunder_dance` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `c05_thunder_spear` | 0 | 0（預期 0 個） | c05r_slow：status_cue 2 | 有呈現 |
| `c05_flash_thunder` | style 1 | 0（預期 0 個） | c05t_restrict：ring 1；c05t_stun：0 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/maeda_keiji-run`（spotlight，2026-10-30，pres14 佇列；v3 產生的 Rust，5facff2e 轉正式）。
- 美術：四招的施放都由技能 cue style 繪製（目前沒有原作 cast 美術）；松風的加速（c05w_armlet）與雷鳴豪槍的緩速（c05r_tempest）都播 Niagara 狀態 cue（art=1）。頭像、技能圖示、載入圖由 HUD sync 依 id 接上已補齊的 nobu 原圖與生成圖；3D 身體目前是共用人形 fallback（專屬 Tripo 身體需要使用者同意花費 credits）。
- 選角稱號與介紹：「傾奇者的雷槍」（17613b2f）。
- 瞬雷閃：引導狀態（ability_channel）與施法限制圓環（c05t_restrict）有呈現；落地暈眩 c05t_stun 本局沒有出現在前端。spotlight 在施放後約 4.5 秒停止錄影，引導 1.5 秒後才落地，敵方替身的暈眩狀態沒有送到前端，原因還沒確認（可能是替身離落點超過 600，或錄影停止時尚未結算）。之後在 1+9 實戰局再確認一次。
