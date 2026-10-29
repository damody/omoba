# 本多忠勝（A07 current）驗收

status: implemented

覆蓋率：已對照原作（damody/history_nobu@f966720，2021-06-01；`D:\omoba\target\source\history_nobu`）。

| 欄 | 技能 | 來源 | 狀態 | 實作 |
|---|---|---|---|---|
| Q | 毫髮無傷 `a07_unscathed` | A07W | supported | `rust/No1_unscathed.rs`＋host damage_immunity |
| W | 赤雷擊 `a07_red_thunder` | A07E | supported | `rust/No2_red_thunder.rs`＋`hero_unit.rs` tick（1 秒後解除自身減速） |
| E | 碎擊槍 `a07_shattering_spear` | A07R | supported | `rust/No3_shattering_spear.rs`＋`hero_unit.rs` on_attack_hit（暴擊判定） |
| R | 戰國最強．啟動 `a07_strongest` | A07T | supported | `rust/No4_strongest.rs`＋`hero_unit.rs` on_attack_hit（濺射） |
| D（extra1） | 奮起 `a07_rouse` | A07D／A07D2 | supported | `rust/No5_rouse.rs` |

## 原作行為對照卡

路徑都在 `game/dota_addons/nobu/scripts/` 底下。

### Q 毫髮無傷（A07W）
- 原作：`npc/A_Oda/A07.txt` A07W（法力 100/110/120/130、冷卻 13、duration 1.25/2.25/3.25/4.25）；OnSpellStart → `A07W_SE` 掛 `modifier_A07W`（`vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:123`），它的 `OnTakeDamage`（:27）在每次受到敵方傷害後把生命設回受傷前，非物理傷害再加 `original_damage × 0.3`。
- 觸發點：每個傷害封包。本遊戲在 `handle_damage` 結算點（damage_immunity::settle）。
- 數值來源：持續時間與法力取 KV；0.3 取腳本常數。
- 目標篩選：只有本人。
- 例外：原作只處理敵方造成的傷害；本遊戲對所有傷害來源都生效（友方傷害在本遊戲不存在）。原作 original_damage 為減免前，本遊戲為減免後（英雄魔抗 0、純粹傷害不減免，所以相同）。

### W 赤雷擊（A07E）
- 原作：A07E（前搖 0.2、冷卻 10、法力 100/120/140/160、radius 525、damage 150/250/350/450、slow −75% 2/3/4/5 秒）；ActOnTargets HERO|BASIC 敵方；DelayedAction 1 秒 `A07E_Unslow` 清空施法者自己的 ms_slow／as_slow。
- 觸發點：前搖結束時結算；1 秒後由英雄 UnitScript tick 送 self_slow_cleanse。
- 例外：`stun_duration 1.53` 沒有被任何動作使用，不暈眩。

### E 碎擊槍（A07R）
- 原作：`A07R_old`（:81）每次攻擊開始擲 RandomInt(0,100)，>30 時計數 +1；計數 >4 或 ≤30 時暴擊並歸零，暴擊倍率 `crit_persent` 120/140/160/180%。`A07R_Learn_Skill`（:236）把 A07D 設為碎擊槍等級 +1。
- 觸發點：本遊戲在每次普攻命中時決定下一次（原作在攻擊開始時決定當次），序列相同。
- 例外：KV 的 CleaveAttack 已被註解，不做濺射。

### R 戰國最強．啟動（A07T）
- 原作：A07T（冷卻 100、法力 150/200/250、duration 8/14/20）；modifier_A07T：BAT 0.8、STATS_STRENGTH_BONUS 15/25/35、INCOMING_DAMAGE_PERCENTAGE −50、魔抗 +50、狀態抗性 50（`events.lua:409`）、減速抗性 50（`library/common/dummy.lua:816`）、OnAttackLanded `AON_Cleave_A07E`（`AON_Cleave.lua:67`：以施法者為中心 365，除目標外敵方英雄與一般單位受 %attack_damage × 100% 物理傷害）。`A07T_Transform`（:256）：免費施放 W、E（保留冷卻、退還法力），移除敵方暈眩，D 換成 0 法力的 A07D2。
- 例外：模型放大 1.3、攻擊前搖 0.2 只影響呈現與攻擊動畫，未實作（AttackPointConstant 在本遊戲沒有消費者，列為能力缺口）。

### D 奮起（A07D／A07D2）
- 原作：A07D（冷卻 5、法力 70、MODIFIER_ATTACKSPEED_BONUS_CONSTANT 60/70/80/90/100、3 秒）；`A07D_HealHP`（:214）回復最大生命 3%；開局 1 級（`library/events/eventforpichero.lua`，heromap_autoskill A07 = D）；等級＝碎擊槍等級 +1；A07D2 為 0 法力版本。
- 例外：大招期間以「退還 70 法力」表達 A07D2；host 仍要求施放當下有 70 法力。

## 已執行的檢查

- `tools\lua\lua.exe scripts\hero_author.lua validate docs\character_pipeline\packages\honda_tadakatsu`：OK（kit_status supported）。
- 私人樹 `D:\omoba\target\hero-a07-b06\shadow3`：`heroes::A07_honda_tadakatsu` 22 項通過（21 個單元測試加 soak）；parity soak `honda_tadakatsu_kit_keeps_team_replicas_identical`（release，7200 tick）0 mismatch，毫髮無傷、赤雷擊減速、大招、奮起、暴擊、自身解減速都有實際發生。
- `Om.Runtime.NativeHeroPoseSanity` Success（`.ai-collab/ue-tests/hero-a07-b06-pose-1`）。模型是暫用版，之後由 tripo-heroes 改成空手本體＋獨立武器。
- 實機（quick 地圖，≤ 10 分鐘）：`target/hero-a07-b06/match-2` 和 `match-5v5`（預設 10 位不重複陣容）的 check_hero_live_acceptance 中，5 個技能都 PASS。run gate 有 1～2 次 observer mismatch，都是淺井或阿市少了冷卻（E095 類），放回 A07 之前就有，見 E163。
- spotlight：`target/hero-a07-b06/spotlight/honda_tadakatsu/honda_tadakatsu_spotlight.mp4`（10 張 keyframe，skipped 0）。

## quick map 實機觀察點（≤ 10 分鐘）

### Q 毫髮無傷
1. 施放後 1.25 秒（rank 1）內，小兵、塔與英雄的攻擊不扣血；魔法技能打中時生命反而小幅上升（30%）。
2. 持續時間結束後恢復正常受傷。

### W 赤雷擊
1. 約 0.2 秒前搖後，周圍 525 內敵方英雄與小兵掉 150（rank 1）魔法傷害，移速大幅下降 2 秒。
2. 525 外的敵人、友軍、塔不受影響。

### E 碎擊槍
1. 學會後普攻約三成出現暴擊數字，倍率 1.2（rank 1），不會連續 5 次都沒暴擊。

### R 戰國最強．啟動
1. 6 級學會；施放時同時出現毫髮無傷與赤雷擊的效果，攻擊明顯變快。
2. 普攻命中時本多忠勝周圍 365 內的其他敵人一起掉血。
3. 期間受到的傷害減半，暈眩與減速時間減半。

### D 奮起
1. 開局即可使用；回復 3% 最大生命，3 秒內攻速提升；大招期間施放不耗法力。
