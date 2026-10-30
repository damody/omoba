# omatsu（A34）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `a34_frost_bolt` | style 1 | 0（預期 0 個） | a34w_freeze（stun）：ring 1 | 有呈現 |
| `a34_iceberg_strike` | placeholder 1、style 1 | 1（預期 1 個） | a34e_slow：status_cue 1 | 有呈現 |
| `a34_frost_breath` | style 1 | 0（預期 0 個） | a34r_frostbite：status_cue 1 | 有呈現 |
| `a34_rapid_cooling` | style 1 | 0（預期 0 個） | a34t_active（a34_rapid_cooling）：status_cue 1；a34t_mr_cut（a34_rapid_cooling_resist_shred）：0 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres/omatsu-run`（spotlight，2026-10-30，queue4）。四招都有呈現：冰霜彈的冰凍圓環（stun）、冰山擊的目標 cue 與緩速、冰霜吐息的凍傷、急速冷卻的啟動狀態。
- 依技能文字，急速冷卻的 a34t_mr_cut（魔抗削減）要在啟動期間命中目標才會疊加。spotlight 最後才放急速冷卻，之後沒有再命中，所以是 0，不是缺陷。
