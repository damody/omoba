# murakami_yoshikiyo（B29）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b29_ground_slam` | style 1 | 0（預期 0 個） | b29w_stun（stun）：ring 2 | 有呈現 |
| `b29_quake_axe` | style 1 | 0（預期 0 個） | b29e_slow：status_cue 1；b29e_aura：status_cue 1；b29e_armor_break：status_cue 1 | 有呈現 |
| `b29_morale_boost` | 0 | 0（預期 0 個） | b29r_morale（b29_morale）：status_cue 3 | 有呈現 |
| `b29_earth_tremor` | effect 1、style 1 | 1（預期 1 個） | b29t_stun（stun）：ring 2 | 有呈現 |

擊退拋物線（全局）：1 次，最高 20000 cm

- 實機：`target/lineC-pres/murakami_yoshikiyo-run`（spotlight，2026-10-30，E331 與 E337 修正後重跑）。四招都有呈現：震地的暈眩、震地斧的減速／破甲／光環、士氣的狀態 cue（自身與友軍）、地裂的特效與目標 cue。
- E331 已在實機確認：地裂的跳躍拋物線最高點是 20000 cm。UE 的比例是 1 遊戲單位 = 100 cm（`WorldUnitsToCm`，水平座標也用同一個比例），所以正好等於 v3 寫的跳躍高度 200。修正前同一招是 0 cm。
