# 真田幸村（B06 current）驗收

status: implemented

覆蓋率：已對照原作（damody/history_nobu@f966720，2021-06-01；`D:\omoba\target\source\history_nobu`）。

| 欄 | 技能 | 來源 | 狀態 | 實作 |
|---|---|---|---|---|
| Q | 真田勇士 `b06_brave_warriors` | B06W | supported | `rust/No1_brave_warriors.rs`（ally_area_buff） |
| W | 連牙炎突 `b06_flame_fangs` | B06E | supported | `rust/No2_flame_fangs.rs`（追蹤彈＋bounce chain） |
| E | 心眼 `b06_mind_eye` | B06R | supported | `rust/No3_mind_eye.rs`（閃避＋spell_guard） |
| R | 日本第一武士 `b06_first_warrior` | B06T | supported | `rust/No4_first_warrior.rs`（全軍光環：aura_emitter） |

## 原作行為對照卡

路徑都在 `game/dota_addons/nobu/scripts/` 底下。

### Q 真田勇士（B06W）
- 原作：`npc/B_Unified/B06.txt` B06W（冷卻 10、法力 100、AOERadius 500）；ActOnTargets 友方 HERO|BASIC、Center CASTER、Radius 500 → modifier_B06W（12 秒，MANA_REGEN_CONSTANT／HEALTH_REGEN_CONSTANT 10/15/20/25、PREATTACK_BONUS_DAMAGE 20/30/40/50）。
- 觸發點：施放時；本遊戲以 ally_area_buff 在 serial settlement 依施法位置選取。
- 例外：音效腳本 `item_soul_adder.lua sound` 不做。

### W 連牙炎突（B06E）
- 原作：B06E（前搖 0.25、冷卻 10、法力 120/140/160/180、施法距離 250、AbilityDamage 100/180/260/340、Stun_Time 0.5/0.8/1.1/1.4）；TrackingProjectile 900；`b06e_start`（`vscripts/heroes/B_Unified/B06_Sanada_Yukimura.lua:69`）設計數 5；`b06e_Shot`（:12）每次命中計數 −1，計數 >0 時在命中目標 500 內（FOW_VISIBLE）隨機挑另一個（排除當下目標）再發一顆。
- 數值來源：全部取 KV 與腳本常數（5、500、900）。
- 例外：原作的隨機改為決定性亂數。

### E 心眼（B06R）
- 原作：modifier_B06R_miss（EVASION_CONSTANT 9/14/19/24）；`library/events/eventfororder.lua:134` 敵方對他下達指定技能且在施法距離內、B06R_Buff 為真時，前搖＋0.05 秒後套 modifier_B06R（MAGICAL_RESISTANCE_BONUS 75，3 秒），結束後 modifier_B06R_Timer（Float_Timer 23/17/12/7）到期才恢復；`B06R_Respawn`（:92）重生重置。
- 例外：本遊戲在敵方指定技能施放成立時觸發，指令被取消時不觸發。

### R 日本第一武士（B06T）
- 原作：B06T（冷卻 155/145/135、法力 300/400/500）；modifier_B06T（During 40/50/60：BAT 0.9、HEALTH_BONUS／Heal 500/750/1000、BASEATTACK_BONUSDAMAGE 40/50/60、MOVESPEED_BONUS_PERCENTAGE 30）；modifier_B06T_Aura（aura_duration 15、Aura_Radius 99999、友方 HERO|BASIC 含自己 → modifier_B06T_2 移速 +8%）。
- 例外：B06T_SE、B06T_effect 只是特效。

## 已執行的檢查

- `tools\lua\lua.exe scripts\hero_author.lua validate docs\character_pipeline\packages\sanada_yukimura`：OK（kit_status supported）。
- 私人樹 `D:\omoba\target\hero-a07-b06\shadow3`：`heroes::B06_sanada_yukimura` 14 項通過，含 parity soak `sanada_yukimura_kit_keeps_team_replicas_identical`（release，7200 tick，兩隊 replica 逐 tick 比對）0 mismatch；真田勇士、炎突暈眩、心眼術（由敵方 lumen_bolt 觸發）、大招與全軍光環都有實際發生。soak 一開始抓到友軍小兵的回血 buff 讓 replica 不一致，修正見 errors.md E163（regen_tick）。
- `Om.Runtime.NativeHeroPoseSanity` Success。模型是暫用版，之後由 tripo-heroes 改成空手本體＋獨立武器。
- 實機：`target/hero-a07-b06/match-2`（玩家 6）的 check_hero_live_acceptance 中，4 個技能都 PASS。
- spotlight：`target/hero-a07-b06/spotlight2/sanada_yukimura/sanada_yukimura_spotlight.mp4`（8 張 keyframe，skipped 0）。替身距離從 280 改成 240 之後，W（施法距離 250）也錄到了。
- 全軍光環已由 source-index 改用通用光環發射器 aura_emitter（E173）；package 的 rust/ 已同步 live 版本。

## quick map 實機觀察點（≤ 10 分鐘）

### Q 真田勇士
1. 施放後周圍 500 內的友軍頭上出現增益，攻擊 +20（rank 1），生命與法力回復變快，持續 12 秒。

### W 連牙炎突
1. 對 250 內的敵人施放，約 0.25 秒前搖後炎彈飛出；命中時暈眩 0.5 秒（rank 1）並掉 100 魔法傷害。
2. 附近有其他敵人時炎彈在敵人之間彈跳，最多 5 次。

### E 心眼
1. 敵方普攻偶爾落空（約 9%，rank 1）。
2. 被敵方指定技能時出現護身效果，3 秒內魔法傷害明顯變少；之後 26 秒內不再觸發（rank 1）。

### R 日本第一武士
1. 6 級學會；施放後攻擊明顯變快、生命上限 +500 並回滿、移速變快；15 秒內全隊移速 +8%。
