# H06 淺井長政：假設與阻塞

來源：`damody/history_nobu@f966720`（E169 前為 nobugg_official@ca6ee86；此英雄單位數值兩者相同），B08 current 版。KV `npc/B_Unified/B08.txt`、腳本 `vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua`。結構化版本在 `source.lua`，逐招對照在 `parity.md`。

## 阻塞

無。仍缺的通用能力列在 `parity.md` 各招的 needs_framework（可揭露的無敵狀態；魔法免疫見 E179，投射物視野與雙隊一般視野見 E185，已接上）。

## 假設

- 基礎數值：本地沒有淺井長政。生命、法力、攻擊沿用 H02 伊達政宗（近戰）的本地基準；護甲、移速、攻擊前搖取來源；近戰射程調為 150。
- 一刀．牙：施法距離 1100（B08.txt:161）與固定飛行 1200（278）是兩個欄位；StartRadius／EndRadius 175 是半徑（198、200）。
- 一刀．空牙：半徑 500（792、794），距離與速度 3000。6/12/18 級解鎖。
- 一刀．閃瞬：B08E_Action（腳本 110-171）以 SetAbsOrigin 站到目標所在點（118-119）；本遊戲用 host blink_landing 落在目標位置，不可走時留在原地。燒魔後另造成等於燒掉法力的魔法傷害（163-169）。
- 活心．魂：modifier_B08R_buff 是固定 +15／−4%（655-656），層數只顯示（260）；對技能與物品都觸發，排除切換技與 logging／tpscroll／samurai_cloth（252-255）。264、268 行 hCaster 未定義，依字面意圖每次觸發刷新。
- 一刀．連牙斷空：B08D_old（腳本 45-108）的 18 段瞬移與刀牙序列；無敵只做不受傷害。學會途徑不明（NOT_LEARNABLE，addon_game_mode.lua:390 為空），沿用 16 級 auto_learn。
- 魔免（原作設定，E170；引擎接上通用魔免時依此）：B08W、B08T 投射物 TargetFlags NONE、ENEMIES_NO（不打魔免）；B08E ENEMIES_NO，目標魔免時只瞬移、不暈不傷不燒魔（B08E_Action 121-124）；B08R 自身；B08D ENEMIES_NO，但目標暈眩不檢查魔免，投射物 TargetFlags NONE（不打魔免）。
