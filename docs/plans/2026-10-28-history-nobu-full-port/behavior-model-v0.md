# 技能行為模型 v0（普查用詞彙草案）

這是 omoba 自己的技能／道具 API 草案，不追求與 Dota 相容。用途是先當作普查的共同詞彙：把原作每一招、每一個道具，都改寫成下面這些概念。寫不出來的部分記為缺口（gap），v1 再依缺口的出現頻率補上概念。

原則：

- **以資料為主**。行為以 Lua 撰寫，建置期編譯成 content，由 Rust 的通用執行器執行。
- **少數例外用 Rust hook**。資料表達不了的邏輯才寫 Rust hook，而且要寫明理由。
- **呈現與玩法分開**。cue 只負責畫面與音效，不影響結算。
- **所有狀態都是決定性的**，而且可以 hash。

## 1. Ability（技能／道具主動）

- `slot`：Q／W／E／R／D／F／item／hidden。`kind`：`active`、`passive`、`toggle`、`autocast`、`channel`。
- `target`：`none`、`unit`、`point`、`direction`、`unit_or_point`；搭配 `filter`（見 §4）與各 rank 的 `range`。
- `cast`：`cast_point`、`backswing`、`channel_time`、可否被打斷、施法中可否移動或轉向。
- `cost`：`mana`、`health`、`charges {max, recharge}`、`cooldown`、`cooldown_start`（`on_cast`、`on_end`、`on_toggle_off`）、`shared_cooldown_group`。
- `requires`：英雄等級、其他技能的狀態、自身是否有某個狀態。
- `ranks`：每個 rank 的數值表。
- `flags`：`pierce_magic_immune`、`ignore_invulnerable`、`can_target_buildings` 等。
- `events`：`on_phase_start`、`on_cast`、`on_channel_tick`、`on_channel_end {interrupted}`、`on_toggle_on`、`on_toggle_off`、`on_learn`、`on_upgrade`、`on_projectile_hit`、`on_projectile_end`、`on_equip`、`on_unequip`。每個事件接一串 action（§3）。

## 2. Status（狀態，對應原作 modifier）

- `id`、`duration`。
- `stacking`：`refresh`、`extend`、`independent`，或 `stack {max, per_stack, refresh_on_add, decay}`。
- `dispel`：`none`、`basic`、`strong`、`death_only`。`visibility`：`hidden`、`buff`、`debuff`。
- `stats`：`StatKey → 數值表達式`，可以隨層數或時間變化。
- `states`：
  - 控制類：`stun`、`root`、`silence`、`disarm`、`mute`、`hex`、`frozen`、`command_restricted`；
  - 防護類：`invulnerable`、`magic_immune`、`attack_immune`、`out_of_game`；
  - 隱匿與選取：`invisible {level, break_on_attack, break_on_cast, fade}`、`unselectable`、`no_healthbar`、`not_on_minimap`；
  - 其他：`no_unit_collision`、`flying`、`cannot_miss`、`evade_disabled`、`passives_disabled`、`blind`、`low_attack_priority`、`provides_vision`。
- `triggers`：每個觸發接一串 action，可以讀事件參數。
  - 生命週期：`on_created`、`on_expired`、`on_removed {reason}`、`periodic(interval)`；
  - 攻擊：`on_attack_start`、`on_attack_landed`、`on_attacked`；
  - 傷害：`on_deal_damage`、`on_take_damage`，兩者都可以修改傷害；
  - 死亡與重生：`on_kill`、`on_death`、`on_owner_death`、`on_respawn`；
  - 其他：`on_ability_executed {who: self|any_enemy|any_ally}`、`on_order`、`on_moved`、`on_heal_received`。
- `aura`：`radius`、`team`、`filter`、`child_status`、`linger`、`include_self`。
- `cue`：附著的呈現 id。

## 3. Action（動作）

| 類別 | action |
|---|---|
| 數值 | `damage {type: physical/magical/pure, amount, flags: no_amplify/no_reflect/hp_removal/non_lethal}`、`heal`、`restore_mana`、`burn_mana`、`set_health`、`set_mana` |
| 狀態 | `apply_status`、`remove_status {by: id/dispel_class/all_debuffs}`、`add_stacks`、`set_stacks` |
| 投遞 | `projectile {linear/tracking, speed, radius, end_radius, distance, pierce, dodgeable, hit_buildings, vision, on_hit, on_end}`、`zone {shape, duration, periodic, on_enter, on_leave, follow}`、`tether {max_distance, on_break, on_expire}` |
| 單位 | `create_unit {template, owner, team, lifetime, control: none/owner/ai, limit_per_owner}`、`create_illusion {incoming_dmg, outgoing_dmg, duration}`、`kill`、`remove_unit` |
| 位移 | `teleport`、`dash`、`leap {arc}`、`knockback`、`pull`、`swap_positions`、`face` |
| 視野 | `reveal {radius, duration, teams, follow, true_sight, ignore_terrain}` |
| 技能 | `modify_cooldown {set/reduce/reset/end}`、`modify_charges`、`add_ability`、`remove_ability`、`swap_abilities`、`set_ability_level`、`set_ability_active` |
| 控制流 | `delay`、`repeat {count, interval}`、`for_each(selector)`、`if(condition)`、`chance {p, pseudo_random}`、`random_pick` |
| 經濟／其他 | `give_gold`、`give_xp`、`issue_order`（對召喚物或受控單位）、`destroy_trees`、`set_state(var)` |
| 呈現 | `cue {id, at, attach, duration}`，只影響畫面，不影響玩法 |

## 4. Selector（選取）

- 起點：`caster`、`target`、`point`、`event_unit`、`projectile`、`zone`。
- 形狀：`circle`、`ring`、`cone`、`line`（膠囊）、`rect`、`global`。
- filter：
  - `team`（enemy、ally、both、self）；
  - `types`（hero、creep、building、summon、ward、neutral、boss、illusion）；
  - `flags`（visible_only、include_magic_immune、include_invulnerable、exclude_self、exclude_list）。
- `order`：nearest、farthest、random、lowest_hp、id。`limit`：上限數量。

## 5. Value（數值表達式）

- 常數與 rank 表。
- 施法者／目標屬性：str、agi、int、primary、attack、max_hp、hp、missing_hp、mana、armor、level、move_speed。
- 情境值：距離、層數、經過時間、事件參數（例如本次傷害量、扣掉的法力）、被選到的單位數、自身狀態變數。
- 運算：四則運算、`min`、`max`、`clamp`。

## 6. Unit state（單位腳本狀態）

取代原作的 `caster.xxx`。每位英雄、每個技能可以宣告有型別的變數（counter、tick、unit ref、vec、bool），進 hash，由 action `set_state` 改寫。

## 7. Hook（Rust 例外）

`hook = "name"`：資料表達不了時，呼叫一個 Rust 函式，參數是有型別的 context。普查時要寫明「為什麼資料表達不了」。

## 7.1 攻擊管線（使用者 2026-10-28 確認，v1 採用）

- 攻擊分三個階段：**前搖** → **發出** → **後搖**。近戰在發出時直接判定；遠程在發出時產生投射物，抵達時判定。
- **命中**與**未命中**不是階段，而是攻擊判定時觸發的事件。判定依閃避、致盲、必中、目標失效等規則。
- 可掛的事件：`on_attack_windup_start`、`on_attack_launch`（出手時才計入的加成、法球扣魔都在這裡結算）、`on_attack_hit`、`on_attack_miss`、`on_attack_cancel`（前搖被打斷）。後搖可以被移動或其他指令取消。

## 8. 道具額外欄位

- 持有時的被動：stats、`unique_group`（同組只取最高的一個）。
- 主動共用冷卻組、消耗品與次數、合成配方、售價。
- 能否丟棄、能否分享、只限持有者使用。
