# 本多忠勝（A07 current）假設與阻塞

行為與數值以 damody/history_nobu@f966720 的原作 KV 與 vscript 為準（使用者 2026-10-27 指定），逐招出處見 acceptance.md 的「原作行為對照卡」。

## 阻塞

無。kit 需要的新通用能力在本批（errors.md E163）補上，不是英雄專用：

- `damage_kind_immunity`：`absolute_no_damage_{physical,magical,pure}` 在 MOBA 傷害結算（`handle_damage`）把該種類傷害歸零。原本這三個 StatKey 只有舊的 f32 路徑在讀，MOBA 對局沒有生效。
- `damage_to_heal_conversion`：`absorb_{physical,magical,pure}_as_heal`（0～1 比例）在同一結算點把該種類傷害的一部分改成回復。
- `status_resistance`／`slow_resistance`：BuffStore::add 縮短暈眩、定身、沉默、技能減速、技能 DoT 的持續時間，以及減弱技能減速的強度。
- `self_slow_cleanse`：只移除自己移動減速的 host 封包。
- 英雄 UnitScript 的 `on_attack_hit`：英雄普攻是投射物，原本只呼叫被動技能的 on_attack_hit，英雄 UnitScript 的這個掛點從來沒被呼叫。改成英雄普攻命中也會呼叫（塔與小兵不變）。
- 技能給的力量：最大生命改走 HealthBonus，不動 `Hero::attribute_hp`，否則另一隊的 replica 會不一致（已和 omfx-decouple 確認）。

## 能力缺口（回報，不影響可玩）

- 戰國最強期間攻擊前搖 0.2（原作 AttackAnimationPoint）：`AttackPointConstant` 在本遊戲沒有消費者，未實作。
- 戰國最強的模型放大 1.3 倍：呈現，未實作。

## 假設

- **距離**：1 個來源距離單位 = 1 個本遊戲世界單位。
- **版本**：只用 current（A07W／E／R／T、A07D／A07D2）。`_old`、`_20` 與 A07F_20 不混用。
- **基礎數值**：由名冊依 `A07_2.txt` 產生（E157），兩個來源相同。
- **毫髮無傷**：所有傷害無效、魔法與純粹傷害的 30% 化為生命（原作腳本），以結算後的傷害計算。
- **赤雷擊**：`stun_duration` 沒被使用，不暈眩；1 秒後解除自身減速照原作做。
- **碎擊槍**：在命中時決定下一次是否暴擊，機率與保底序列和原作相同；不做濺射。
- **戰國最強**：BAT 0.8 以攻速乘數 2.25 表達；魔抗 +50 以本遊戲乘法疊加；濺射傷害用最終攻擊力（不含暴擊）。
- **奮起**：1 級自動學會，攻速依碎擊槍等級；回復 3%；大招期間退還法力。
