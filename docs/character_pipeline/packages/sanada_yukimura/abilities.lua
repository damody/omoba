-- 真田幸村 ability contract (source B06 current; behaviour and numbers from history_nobu@f966720,
-- see source.lua assumption behavior_source). Q/W/E/R = B06W/E/R/T; no D slot.
return function(ctx)
  local function lifecycle(over)
    local l = {
      caster_death = "自身增益隨死亡移除；已結算的效果保留",
      target_invalid = "結算時目標已死亡或失效則施放失敗，不扣費",
      vision_lost = "只能對施放當下可見的目標施放",
      cancel = "前搖內取消不扣費、不進冷卻",
      expire = "效果到期移除",
    }
    for k, v in pairs(over or {}) do l[k] = v end
    return l
  end
  return {
    schema_version = 2,
    abilities = {
      {
        id = "b06_brave_warriors", source_ability_id = "B06W", action = "Q", ability_type = "active",
        name = "真田勇士", max_level = 4, status = "supported",
        description = "振奮周圍 500 內的友軍（含自己），12 秒內攻擊 +20/30/40/50，每秒回復 10/15/20/25 生命與法力。",
        capabilities = { "ally_area_buff_query", "timed_stat_buff" },
        targeting = { kind = "none", team = "ally", shape = { kind = "circle", radius = { 500, 500, 500, 500 } }, visibility = "area_from_caster" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 10, 10, 10, 10 }, mana_cost = { 100, 100, 100, 100 } },
        effects = {
          { kind = "custom", note = "ally_area_buff：PreattackBonusDamage、HealthRegenConstant、ManaRegenConstant",
            duration = { 12, 12, 12, 12 }, attack_bonus = { 20, 30, 40, 50 }, health_regen = { 10, 15, 20, 25 }, mana_regen = { 10, 15, 20, 25 } },
        },
        lifecycle = lifecycle({ target_invalid = "範圍增益，沒有單位目標", vision_lost = "範圍以施法者為中心，不需視野", caster_death = "已套上的增益照常持續到期" }),
        implementation = { kind = "rust_handler", module = "No1_brave_warriors", note = "ally_area_buff 封包（同隊、非建築、含施法者）" },
        bot_usage = { target = "self", when = "附近有友方英雄或即將交戰" },
        verification = { scenarios = {
          { name = "rank_values", expect = "500 內友軍攻擊 +20/30/40/50、每秒回復 10/15/20/25 生命與法力 12 秒" },
          { name = "edge", expect = "500 外的友軍、敵人與建築不受影響" },
        } },
        assumptions = { "warriors_targets" },
      },
      {
        id = "b06_flame_fangs", source_ability_id = "B06E", action = "W", ability_type = "active",
        name = "連牙炎突", max_level = 4, status = "supported",
        description = "從槍尖釋放炎彈（速度 900）攻擊指定敵人，命中後從目標 500 內隨機彈向另一個可見的敵人，最多命中 5 次；每次命中造成 100/180/260/340 魔法傷害並暈眩 0.5/0.8/1.1/1.4 秒。",
        capabilities = { "projectile_tracking", "bounce_chain", "unit_damage", "stun" },
        targeting = { kind = "unit", team = "enemy", range = { 250, 250, 250, 250 }, shape = { kind = "none" }, visibility = "target_must_be_visible" },
        timing = { cast_point = 0.25, cooldown_starts = "on_cast" },
        resources = { cooldown = { 10, 10, 10, 10 }, mana_cost = { 120, 140, 160, 180 } },
        effects = {
          { kind = "projectile", motion = "homing", speed = 900, radius = 0, lifetime = 10 },
          { kind = "damage", amount = { 100, 180, 260, 340 }, damage_kind = "magical", delivered_by = "projectile" },
          { kind = "control", control = "stun", duration = { 0.5, 0.8, 1.1, 1.4 } },
          { kind = "custom", note = "彈跳：命中後在目標 radius 內隨機挑另一個可見的敵方英雄或一般單位（只排除當下目標），總命中上限 hits",
            bounce_hits = { 5, 5, 5, 5 }, bounce_radius = { 500, 500, 500, 500 } },
        },
        lifecycle = lifecycle({ caster_death = "已發射的炎彈照常彈跳與結算", vision_lost = "只彈向可見的敵人；初始目標必須可見" }),
        implementation = { kind = "rust_handler", module = "No2_flame_fangs", note = "G-1 追蹤彈＋本批通用 bounce（ability_projectile Spec.bounce）" },
        bot_usage = { target = "enemy_unit", when = "250 內有敵方英雄，周圍有其他敵人時優先" },
        verification = { scenarios = {
          { name = "rank_values", expect = "每次命中 100/180/260/340 魔法傷害、暈眩 0.5/0.8/1.1/1.4 秒" },
          { name = "bounce", expect = "最多命中 5 次；只會跳向 500 內可見的敵人；不會連續打同一個目標" },
        } },
        assumptions = { "fangs_bounce" },
      },
      {
        id = "b06_mind_eye", source_ability_id = "B06R", action = "E", ability_type = "passive",
        name = "心眼", max_level = 4, status = "supported",
        description = "被動：9/14/19/24% 機率閃避攻擊。被敵方指定技能時獲得真田心眼術：3 秒內魔抗 +75；之後 23/17/12/7 秒才能再次發動，重生時重置。",
        capabilities = { "evasion", "permanent_stat_bonus", "spell_guard" },
        targeting = { kind = "self", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 0, 0, 0, 0 }, mana_cost = { 0, 0, 0, 0 } },
        effects = {
          { kind = "custom", note = "常駐閃避（EvasionConstant）", evasion_pct = { 9, 14, 19, 24 } },
          { kind = "custom", note = "spell_guard：被敵方指定技能時套魔抗，之後冷卻",
            magic_resist = { 75, 75, 75, 75 }, guard_duration = { 3, 3, 3, 3 }, guard_cooldown = { 23, 17, 12, 7 } },
        },
        lifecycle = lifecycle({ caster_death = "被動保留；心眼術冷卻在重生時重置", target_invalid = "被動，沒有目標", cancel = "被動，不能取消", expire = "心眼術 3 秒後移除，冷卻結束後再度可用" }),
        implementation = { kind = "rust_handler", module = "No3_mind_eye", note = "on_learn 永久閃避＋spell_guard 標記；host 在敵方指定技能施放時觸發" },
        bot_usage = { target = "self", when = "被動" },
        verification = { scenarios = {
          { name = "evasion", expect = "敵方普攻約 9/14/19/24% 落空" },
          { name = "guard", expect = "被敵方指定技能時 3 秒內魔抗 +75，冷卻內不再觸發" },
        } },
        assumptions = { "mind_eye_trigger" },
      },
      {
        id = "b06_first_warrior", source_ability_id = "B06T", action = "R", ability_type = "ultimate",
        name = "日本第一武士", max_level = 3, status = "supported",
        required_hero_level = { 6, 12, 18 },
        description = "持續 40/50/60 秒：基礎攻擊間隔 0.9（2 倍攻速）、生命上限 +500/750/1000 並立即回復同量、基礎攻擊 +40/50/60、移速 +30%；15 秒內全圖友軍（含自己）移速 +8%。",
        capabilities = { "timed_stat_buff", "heal", "aura", "ally_area_buff_query", "hero_unit_hooks" },
        targeting = { kind = "none", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 155, 145, 135 }, mana_cost = { 300, 400, 500 } },
        effects = {
          { kind = "stat_buff", duration = { 40, 50, 60 }, stats = { attack_speed_multiplier = { 2, 2, 2 }, max_hp_bonus = { 500, 750, 1000 }, move_speed_pct = { 30, 30, 30 } } },
          { kind = "custom", note = "基礎攻擊（BaseAttackBonusDamage）與立即回復生命上限加成的量", base_attack_bonus = { 40, 50, 60 }, heal = { 500, 750, 1000 } },
          { kind = "custom", note = "全圖友軍光環（ally_area_buff，英雄 UnitScript tick 每 0.5 秒刷新）",
            aura_move_speed_pct = { 8, 8, 8 }, aura_duration = { 15, 15, 15 }, aura_radius = { 100000, 100000, 100000 } },
        },
        lifecycle = lifecycle({ target_invalid = "自身增益，沒有單位目標", vision_lost = "不需視野", expire = "到期後攻速、生命上限、攻擊與移速移除；光環 15 秒後停止", caster_death = "所有增益與光環隨死亡移除" }),
        implementation = { kind = "rust_handler", module = "No4_first_warrior", note = "自身 stat buff＋heal；光環由 rust/hero_unit.rs on_tick" },
        bot_usage = { target = "self", when = "附近有敵方英雄且即將交戰，或推進時" },
        verification = { scenarios = {
          { name = "self", expect = "攻擊間隔減半、生命上限 +500 並回滿這 500、攻擊 +40、移速 +30%" },
          { name = "aura", expect = "15 秒內全圖友軍移速 +8%，之後消失" },
        } },
        assumptions = { "first_warrior_history", "first_warrior_attack_rate", "first_warrior_aura_radius", "first_warrior_levels" },
      },
    },
  }
end
