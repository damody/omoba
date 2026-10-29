# H02 伊達政宗：假設與改編

結構化版本在 `source.lua` 的 `assumptions`；逐招原作對照與移植結果見 `parity.md`。來源：`damody/history_nobu@f966720`，B04 current 版（KV `npc/B_Unified/B04.txt`、腳本 `vscripts/heroes/B_Unified/B04.lua`）。

## 來源與編號

- 來源 B02 是風魔小太郎，與本 kit 無關。本 kit 依名稱對照到本地 `date_masamune`。
- 本地 Rust 模組資料夾叫 `B02_date_masamune`，那只是既有識別字，不代表來源編號。

## 基礎數值

- 英雄三圍、生命、攻擊、射程（近戰 128）、移速由原作 `B04_2.txt` 經 hero_source_stats 帶入；`character.lua` 的 base_stats 已不生效。

## 踏火無痕（W→Q）

- 衝刺期間 300 光環觸發（B04.lua:B04W_OnTrigger:52-70），本遊戲以「實際路徑兩側 300 內的敵人」在施法時決定；原作從衝刺途中當下位置發射，逐 tick 觸發是 needs_framework。
- 火焰速度 1500（B04.lua:65 speed × 0.5）。
- 原作沒有目標上限與投射物壽命：上限取 host 每來源上限 32、壽命取 10 秒，皆為引擎限制。
- 原作沒有隨機；以路徑接觸先後決定性排序。

## 火焰強襲（E→W）

- 500 半徑魔法傷害加暈眩，只打敵方英雄與一般單位（不含建築）。

## 炎之迅捷（R→E 被動）

- +10/20/30/40 敏捷（StatsAgilityBonus），由屬性規則換算護甲、攻速與主屬性攻擊。

## 種子島（T→R）

- 射程 +672（128＋672＝800）、基礎攻擊 +50/70/90。
- 普攻發射時 38% 暈眩 0.2 秒（不看命中、不含建築）。
- 每次普攻發射一發直線穿透子彈（35/50/65 魔法），另兩次約 26% 機率延遲 0.05／0.1 秒追加。
- 解鎖等級採 6/12/18（KV 未寫，依 history_nobu 慣例與現代 Dota 預設）。

## 與本地 kit 的差異

- 本地四招（`flame_blade`、`fire_dash`、`flame_assault`、`matchlock_gun`）保留不動。
- H02 以 `b04_*` 新 ID 附加；build 用 `update_existing` 換掉英雄的四個技能欄。
- 魔免（原作設定，E170；通用魔免已接上，E179／E184：B04T `pierces_magic_immunity`，發射時的暈眩穿透魔免，子彈魔法傷害歸零；其餘招式依全域規則）：B04W 光環預設旗標不含魔免、SPELL_IMMUNITY_ENEMIES_NO（不打魔免）；B04E 不打魔免（ENEMIES_NO，旗標 NONE）；B04R 被動自身；B04T 技能 ENEMIES_YES，暈眩不檢查魔免（可暈魔免），直線子彈 TargetFlags NONE 且命中時再檢查 IsMagicImmune（不傷魔免）。
