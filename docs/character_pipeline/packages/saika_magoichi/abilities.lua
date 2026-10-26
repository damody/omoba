-- H01 ability contract (source B01 current). Per-rank arrays have exactly
-- max_level entries. New ability IDs are appended; the old local abilities
-- (sniper_mode, saika_reinforcements, rain_iron_cannon, three_stage_technique)
-- are not modified.
return function(ctx)
  local function standard_lifecycle(over)
    local l = {
      caster_death = "效果移除，冷卻照常計時",
      target_invalid = "不適用（無單位目標）",
      vision_lost = "不影響自身效果",
      cancel = "前搖內取消不扣費、不進冷卻",
      expire = "到期移除增益並通知呈現結束",
    }
    for k, v in pairs(over or {}) do l[k] = v end
    return l
  end
  return {
    schema_version = 2,
    abilities = {
      {
        id = "b01_sniper_mode", source_ability_id = "B01W", action = "Q", ability_type = "active",
        name = "狙擊模式", max_level = 4, status = "supported",
        description = "轉換為狙擊狀態 12 秒：攻擊力 +0/10/20/30，攻擊距離 +200/350/500/700（700 基礎射程下為 900/1050/1200/1400）。",
        capabilities = { "timed_stat_buff", "vision_radius_bonus" },
        targeting = { kind = "self", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 22, 22, 22, 22 }, mana_cost = { 100, 100, 100, 100 } },
        effects = {
          { kind = "stat_buff", duration = { 12, 12, 12, 12 },
            stats = { attack_damage_bonus = { 0, 10, 20, 30 }, attack_range_bonus = { 200, 350, 500, 700 } } },
        },
        lifecycle = standard_lifecycle(),
        implementation = { kind = "rust_handler", module = "No5_sniper_mode_timed",
          note = "沿用 No1_sniper_mode 的 buff 結構，改為限時、無懲罰" },
        bot_usage = { target = "self", when = "敵方英雄在 900～1400 內且自身未被近身", avoid = "被近身或血量低於 35%" },
        verification = { scenarios = {
          { name = "rank_values", expect = "每個 rank 的攻擊加成 0/10/20/30 與射程 +200/350/500/700 生效 12 秒後移除" },
          { name = "no_mana", expect = "法力 < 100 時拒絕並回報沒藍，不進冷卻" },
          { name = "cooldown", expect = "施放後 22 秒內再按被拒絕" },
          { name = "death_during_buff", expect = "死亡時移除增益，重生後沒有殘留射程" },
        } },
        assumptions = { "night_vision_not_applicable", "w_range_text_conflict" },
      },
      {
        id = "b01_saika_corps", source_ability_id = "B01E", action = "W", ability_type = "active",
        name = "雜賀眾", max_level = 4, status = "supported",
        description = "每 15 秒儲存 1 位雜賀眾，最多 2/3/4/5 位；施放消耗 1 位，在指定地點召喚不能移動的雜賀眾 60 秒，攻擊力隨孫市等級每級 +10。",
        capabilities = { "charges", "summon_timed", "summon_immobile" },
        targeting = { kind = "point", team = "ally", range = { 500, 500, 500, 500 }, shape = { kind = "none" },
          visibility = "point_must_be_pathable_and_visible" },
        timing = { cast_point = 0.15, cooldown_starts = "on_cast" },
        resources = { cooldown = { 0, 0, 0, 0 }, mana_cost = { 35, 35, 35, 35 },
          charges = { max = { 2, 3, 4, 5 }, recharge_seconds = { 15, 15, 15, 15 } } },
        effects = {
          { kind = "summon", unit = "b01_saika_corps_gunner", count = 1, lifetime = { 60, 60, 60, 60 } },
        },
        lifecycle = standard_lifecycle({
          caster_death = "已召喚的雜賀眾保留到存活時間結束；儲存次數保留",
          target_invalid = "落點不可走或不可見時拒絕，不扣次數",
          expire = "召喚物到期消失並退役歸因",
        }),
        implementation = { kind = "rust_handler", module = "No6_saika_corps",
          note = "host 依 charges 欄位扣次數（G-2）；召喚 package 自帶的 immobile 模板 b01_saika_corps_gunner（G-3，summons.lua）" },
        bot_usage = { target = "point", when = "有儲存次數且推線或守塔時，放在自身與敵方之間的安全位置", avoid = "敵方塔射程內" },
        verification = { scenarios = {
          { name = "charge_cap", expect = "rank 1～4 最多儲存 2/3/4/5 次；滿額時不再累積" },
          { name = "recharge", expect = "每 15 秒 +1 次，與施放時間無關" },
          { name = "no_charge", expect = "沒有儲存次數時拒絕且不扣法力" },
          { name = "immobile", expect = "召喚物不移動、不跟隨主人，仍會攻擊射程內敵人" },
          { name = "owner_retire", expect = "召喚物擊殺歸因給孫市；到期正確清除" },
        } },
        assumptions = { "gunner_stats_from_local", "gunner_reserve_start" },
      },
      {
        id = "b01_rain_iron_cannon", source_ability_id = "B01R", action = "E", ability_type = "passive",
        name = "雨鐵炮", max_level = 4, status = "supported",
        description = "被動：普攻命中時，依目標類別追加物理傷害：建築 +10/15/20/25%、英雄 +10/18/24/30%、部隊 +50/70/90/110%（以攻擊力計）。",
        capabilities = { "on_attack_hook", "target_class_multiplier" },
        targeting = { kind = "none", team = "enemy", shape = { kind = "none" }, visibility = "attack_target_only" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 0, 0, 0, 0 }, mana_cost = { 0, 0, 0, 0 } },
        effects = {
          { kind = "on_attack_bonus_by_target_class", damage_kind = "physical", unit = "percent_of_attack_damage",
            classes = { building = { 10, 15, 20, 25 }, hero = { 10, 18, 24, 30 }, creep = { 50, 70, 90, 110 } } },
        },
        lifecycle = standard_lifecycle({ target_invalid = "目標在命中前死亡則不加成", cancel = "被動，不可取消" }),
        implementation = { kind = "rust_handler", module = "No7_rain_iron_cannon_class",
          note = "on_attack_hit 內依 is_building／unit_id_of 判別類別，追加一筆物理傷害" },
        verification = { scenarios = {
          { name = "class_bonus", expect = "對建築／英雄／部隊分別多 10～25%／10～30%／50～110% 物理傷害" },
          { name = "skill_projectiles", expect = "只有真正普攻命中觸發，技能與召喚物攻擊不觸發" },
        } },
      },
      {
        id = "b01_three_stage", source_ability_id = "B01T", action = "R", ability_type = "ultimate",
        name = "三段", max_level = 3, status = "supported",
        description = "施展三段技法 7/11/13 秒：基礎傷害 +200%。每次普攻只判定一次傷害。",
        required_hero_level = { 6, 12, 18 },
        capabilities = { "timed_stat_buff" },
        targeting = { kind = "self", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0.1, cooldown_starts = "on_cast" },
        resources = { cooldown = { 135, 123, 111 }, mana_cost = { 300, 350, 400 } },
        effects = {
          { kind = "stat_buff", duration = { 7, 11, 13 }, stats = { base_damage_outgoing_pct = { 200, 200, 200 } } },
        },
        lifecycle = standard_lifecycle(),
        implementation = { kind = "rust_handler", module = "No8_three_stage",
          note = "沿用 No4_three_stage_technique 的 buff；去掉多段判定" },
        bot_usage = { target = "self", when = "團戰或敵方英雄在攻擊範圍內，自身血量高於 40%" },
        verification = { scenarios = {
          { name = "duration", expect = "rank 1～3 持續 7/11/13 秒，基礎傷害 +200%" },
          { name = "level_gate", expect = "英雄 6/12/18 級前不能學對應 rank" },
          { name = "single_hit", expect = "每次普攻只判定一次傷害" },
        } },
        assumptions = { "three_stage_no_multishot" },
      },
    },
  }
end
