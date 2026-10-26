-- H06 ability contract (source B08 current). E (passive stacks on every cast)
-- needs a hero UnitScript registration path and D is a fifth action; the
-- package records them instead of approximating.
return function(ctx)
  local function lifecycle(over)
    local l = {
      caster_death = "已發射的刀牙照常結算；增益移除",
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
        id = "b08_fang_slash", source_ability_id = "B08W", action = "Q", ability_type = "active",
        name = "一刀．牙", max_level = 4, status = "supported",
        description = "朝指定方向斬出刀牙（寬 175、飛行 1200、速度 1500），路徑上每個敵人（英雄與小兵）受 200/300/400/500 魔法傷害一次。",
        capabilities = { "projectile_linear", "unit_damage" },
        targeting = { kind = "direction", team = "enemy", range = { 1200, 1200, 1200, 1200 },
          shape = { kind = "line", width = 175 }, visibility = "direction_any" },
        timing = { cast_point = 0.2, cooldown_starts = "on_cast" },
        resources = { cooldown = { 7, 6, 5, 4 }, mana_cost = { 100, 120, 140, 160 } },
        effects = {
          { kind = "projectile", motion = "linear", speed = 1500, radius = 87.5, max_hits = 64 },
          { kind = "damage", amount = { 200, 300, 400, 500 }, damage_kind = "magical", delivered_by = "projectile" },
        },
        lifecycle = lifecycle({ target_invalid = "直線彈不需單位目標" }),
        implementation = { kind = "rust_handler", module = "No1_fang_slash", note = "G-1 Linear 穿透" },
        bot_usage = { target = "direction", when = "朝 1200 內敵方英雄或兵群" },
        verification = { scenarios = {
          { name = "rank_values", expect = "刀牙沿途每個敵人受 200/300/400/500 魔法傷害一次" },
          { name = "pierce", expect = "穿透整條兵線；同一敵人不重複受傷" },
        } },
        assumptions = { "fang_distance" },
      },
      {
        id = "b08_flash_cut", source_ability_id = "B08E", action = "W", ability_type = "active",
        name = "一刀．閃瞬", max_level = 4, status = "supported",
        description = "瞬移到指定敵人身旁並斬擊：造成 100/200/300/400 魔法傷害、暈眩 0.3 秒，並使其損失 100/160/220/280 法力。",
        capabilities = { "blink_to_target", "unit_damage", "stun", "mana_burn" },
        targeting = { kind = "unit", team = "enemy", range = { 1100, 1100, 1100, 1100 }, shape = { kind = "none" }, visibility = "target_must_be_visible" },
        timing = { cast_point = 0.3, cooldown_starts = "on_cast" },
        resources = { cooldown = { 13, 13, 13, 13 }, mana_cost = { 100, 120, 140, 160 } },
        effects = {
          { kind = "dash", max_distance = { 1100, 1100, 1100, 1100 } },
          { kind = "damage", amount = { 100, 200, 300, 400 }, damage_kind = "magical" },
          { kind = "control", control = "stun", duration = { 0.3, 0.3, 0.3, 0.3 } },
          { kind = "mana_drain", amount = { 100, 160, 220, 280 } },
        },
        lifecycle = lifecycle(),
        implementation = { kind = "rust_handler", module = "No2_flash_cut", note = "set_pos 落在目標身旁＋deal_damage＋stun＋host mana_drain 封包" },
        bot_usage = { target = "enemy_unit", when = "可見敵方英雄在 1100 內且血量低於 60%", avoid = "目標在敵方塔射程內" },
        verification = { scenarios = {
          { name = "rank_values", expect = "瞬移到目標旁，目標受 100/200/300/400 魔法傷害、暈眩 0.3 秒、法力 −100/160/220/280（不低於 0）" },
          { name = "landing", expect = "瞬移不受中間單位或地形阻擋，落在目標身旁 100 並面向目標，可立即普攻" },
        } },
        assumptions = { "flash_landing" },
      },
      {
        id = "b08_living_soul", source_ability_id = "B08R", action = "E", ability_type = "passive",
        name = "活心．魂", max_level = 4, status = "supported",
        description = "被動：攻擊速度 +20/35/50/65。每次使用技能後 8 秒內攻擊 +15、受到傷害 −4%；持續時間內再次施放時攻擊加成疊加，最多 4/5/6/7 層。",
        capabilities = { "permanent_stat_bonus", "on_cast_trigger", "stacking_buff" },
        targeting = { kind = "self", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 0, 0, 0, 0 }, mana_cost = { 0, 0, 0, 0 } },
        effects = {
          { kind = "stat_bonus_permanent", stats = { attack_speed_bonus = { 20, 35, 50, 65 } } },
          { kind = "stat_buff", duration = { 8, 8, 8, 8 }, stats = { attack_damage_bonus = { 15, 15, 15, 15 }, incoming_damage_pct = { -4, -4, -4, -4 } },
            max_stacks = { 4, 5, 6, 7 }, trigger = "on_skill_cast" },
        },
        lifecycle = lifecycle({ caster_death = "疊層移除；永久攻速保留", target_invalid = "被動，沒有目標", cancel = "被動，不能取消",
          expire = "8 秒沒有再施放則疊層全部移除" }),
        implementation = { kind = "rust_handler", module = "No3_living_soul", note = "on_learn 永久攻速＋rank 標記；施法疊層由英雄 UnitScript（rust/hero_unit.rs）的 on_skill_cast 呼叫" },
        bot_usage = { target = "self", when = "被動" },
        verification = { scenarios = {
          { name = "attack_speed", expect = "學會 rank 1～4 後攻速 +20/35/50/65" },
          { name = "stacks", expect = "連續施放技能時攻擊 +15×層數（最多 4/5/6/7 層）、受到傷害 −4%；8 秒未施放全部消失" },
        } },
        assumptions = { "soul_stack_rule" },
      },
      {
        id = "b08_sky_fang", source_ability_id = "B08T", action = "R", ability_type = "ultimate",
        name = "一刀．空牙", max_level = 3, status = "supported",
        required_hero_level = { 6, 12, 18 },
        description = "朝指定方向斬出巨大刀牙（寬 500、飛行 3000、速度 3000），路徑上每個敵人受 800/1200/1600 魔法傷害一次。",
        capabilities = { "projectile_linear", "unit_damage" },
        targeting = { kind = "direction", team = "enemy", range = { 3000, 3000, 3000 },
          shape = { kind = "line", width = 500 }, visibility = "direction_any" },
        timing = { cast_point = 0.2, cooldown_starts = "on_cast" },
        resources = { cooldown = { 100, 85, 70 }, mana_cost = { 200, 200, 200 } },
        effects = {
          { kind = "projectile", motion = "linear", speed = 3000, radius = 250, max_hits = 64 },
          { kind = "damage", amount = { 800, 1200, 1600 }, damage_kind = "magical", delivered_by = "projectile" },
        },
        lifecycle = lifecycle({ target_invalid = "直線彈不需單位目標" }),
        implementation = { kind = "rust_handler", module = "No4_sky_fang", note = "G-1 Linear 穿透" },
        bot_usage = { target = "direction", when = "路徑上有低血量敵方英雄" },
        verification = { scenarios = {
          { name = "rank_values", expect = "路徑上每個敵人受 800/1200/1600 魔法傷害一次" },
          { name = "range", expect = "飛行 3000 後消失" },
        } },
        assumptions = { "sky_fang_width", "sky_fang_levels" },
      },
      {
        id = "b08_endless_fang", source_ability_id = "B08D", action = "extra1", ability_type = "ultimate",
        name = "一刀．連牙斷空", max_level = 1, status = "supported", auto_learn = true,
        required_hero_level = { 16 },
        description = "16 級大招：對指定敵人連斬 18 刀，每刀 100 魔法傷害，總共暈眩 2.2 秒。",
        capabilities = { "extra_action_slot", "unit_damage", "stun" },
        targeting = { kind = "unit", team = "enemy", range = 1100, shape = { kind = "none" }, visibility = "target_must_be_visible" },
        timing = { cast_point = 0.3, cooldown_starts = "on_cast" },
        resources = { cooldown = 150, mana_cost = 300 },
        effects = {
          { kind = "damage", amount = 1800, damage_kind = "magical", note = "18 刀 × 100" },
          { kind = "control", control = "stun", duration = 2.2 },
        },
        lifecycle = lifecycle(),
        implementation = { kind = "rust_handler", module = "No5_endless_fang", note = "D 欄（第 5 欄）auto_learn：16 級自動學會、不耗技能點" },
        bot_usage = { target = "enemy_unit", when = "可見敵方英雄在 1100 內" },
        verification = { scenarios = {
          { name = "damage", expect = "目標共受 1800 魔法傷害並暈眩 2.2 秒" },
        } },
        assumptions = { "distance_units" },
      },
    },
  }
end
