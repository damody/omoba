# bot 從未取得的 4 件道具：UE 呈現檢查（道具 spotlight）

gamemode 回報這 4 件道具從沒被 bot 取得，所以用道具 spotlight 單獨在實機驗證：關卡以 showcase start_items 直接放進背包（合成品會先放材料），autopilot 按道具欄的使用鍵施放。英雄是雜賀孫市，對手是兩名站樁替身。

## a 局：爆烈彈、清酒、仕手能靴（`target/lineC-items/a/run`，2026-10-30）

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `item_napalm_bomb` | placeholder 1 | 1（預期 0 個） | — | 有呈現 |
| `item_sake` | 0 | 0（預期 0 個） | sake_haste（Passive_sake）：status_cue 1 | 有呈現 |
| `item_mana_boots` | placeholder 1 | 0（預期 0 個） | mana_boots_immune（magic_immune）：ring 1 | 有呈現 |

擊退拋物線（全局）：0 次

## b 局：破魔．仕手能靴（`target/lineC-items/b/run`，2026-10-30）

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `item_mana_boots_1` | placeholder 1 | 0（預期 0 個） | mana_boots_1_immune（magic_immune）：ring 1 | 有呈現 |


## 結論

- 四件都在實機施放成功（伺服器 ItemUse ok，autopilot 有 authority accepted），而且畫面上都有呈現。
- 爆烈彈：投擲點播放爆炸 cue（snapjaw_rocket_explosion），以目標 cue 的形式出現。目前是 placeholder 美術，cast-cues 表裡也還沒有爆烈彈的專屬 cue（所以預期是 0 個）。
- 清酒：加速狀態 sake_haste 有狀態 cue。施放瞬間沒有 cue，這和施放表一致。
- 仕手能靴、破魔．仕手能靴：施放 cue 是 placeholder 美術（item_perceive_wine），魔法免疫狀態以圓環呈現。
- 美術：這兩個 cue 的 Grok 參考圖已補上（omfue 8fc5e24：gen-cue-snapjaw_rocket_explosion、gen-cue-item_perceive_wine），做法和其他道具一樣。它們是製作特效用的參考圖，遊戲裡的特效做出來之前，實機仍會播 placeholder。
