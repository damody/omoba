-- H03 ability contract (source A34 current).
return function(ctx)
  local function lifecycle(over)
    local l = {
      caster_death = "已發射的投射物照常結算；增益移除",
      target_invalid = "目標死亡或失效時投射物消失，不造成傷害",
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
        id = "a34_frost_bolt", source_ability_id = "A34W", action = "Q", ability_type = "active",
        name = "寒霜彈", max_level = 4, status = "supported",
        description = "對敵人或建築物投出追蹤寒霜彈（速度 2000），命中造成 100/200/300/400 魔法傷害並結凍 1/1.5/2/2.5 秒。",
        capabilities = { "projectile_tracking", "unit_damage", "stun", "projectile_vs_building" },
        targeting = { kind = "unit", team = "enemy", range = { 600, 600, 600, 600 }, shape = { kind = "none" }, visibility = "target_must_be_visible" },
        timing = { cast_point = 0.25, cooldown_starts = "on_cast" },
        resources = { cooldown = { 14, 14, 14, 14 }, mana_cost = { 100, 125, 150, 175 } },
        effects = {
          { kind = "projectile", motion = "homing", speed = 2000, radius = 16, lifetime = 3 },
          { kind = "damage", amount = { 100, 200, 300, 400 }, damage_kind = "magical", delivered_by = "projectile" },
          { kind = "control", control = "stun", duration = { 1, 1.5, 2, 2.5 }, delivered_by = "projectile" },
        },
        lifecycle = lifecycle(),
        implementation = { kind = "rust_handler", module = "No1_frost_bolt", note = "G-1 Homing＋control＋buildings=true" },
        bot_usage = { target = "enemy_unit", when = "可見的敵方英雄在 600 內，優先正在施法或追擊者" },
        verification = { scenarios = {
          { name = "rank_values", expect = "命中時 rank 1～4 造成 100/200/300/400 魔法傷害並暈眩 1/1.5/2/2.5 秒；施放當下不扣血" },
          { name = "travel_time", expect = "遠距目標比近距晚命中（速度 2000）" },
          { name = "building_target", expect = "可對敵方建築施放並命中" },
        } },
        assumptions = { "distance_units" },
      },
      {
        id = "a34_iceberg_strike", source_ability_id = "A34E", action = "W", ability_type = "active",
        name = "冰山之襲", max_level = 4, status = "supported",
        description = "以冰山轟擊指定敵軍造成 100/200/300/400 魔法傷害，並對其周圍 335 內的其他敵人造成 100 冰爆傷害；被擊中的單位移動速度 −30%、攻擊速度 −20%，持續 5 秒。",
        capabilities = { "unit_damage", "aoe_circle", "slow_debuff" },
        targeting = { kind = "unit", team = "enemy", range = { 700, 700, 700, 700 },
          shape = { kind = "circle", radius = { 335, 335, 335, 335 } }, visibility = "target_must_be_visible" },
        timing = { cast_point = 0.25, cooldown_starts = "on_cast" },
        resources = { cooldown = { 7, 7, 7, 7 }, mana_cost = { 125, 145, 165, 185 } },
        effects = {
          { kind = "damage", amount = { 100, 200, 300, 400 }, damage_kind = "magical" },
          { kind = "damage", amount = { 100, 100, 100, 100 }, damage_kind = "magical", applies_to = "others_in_radius" },
          { kind = "slow", move_speed_pct = { 30, 30, 30, 30 }, attack_speed_reduction = { 20, 20, 20, 20 }, duration = { 5, 5, 5, 5 } },
        },
        lifecycle = lifecycle({ target_invalid = "結算時主目標已死亡或不可見則施放失敗，不扣費" }),
        implementation = { kind = "rust_handler", module = "No2_iceberg_strike", note = "deal_damage＋query_enemies_in_range＋通用 slow buff" },
        bot_usage = { target = "enemy_unit", when = "目標周圍 335 內有多名敵人，或清線" },
        verification = { scenarios = {
          { name = "rank_values", expect = "主目標受 100/200/300/400，周圍 335 內其他敵人各受 100，皆為魔法傷害" },
          { name = "slow", expect = "所有被擊中者移速 −30%、攻速 −20%，5 秒" },
          { name = "radius_edge", expect = "335 外與友軍不受影響；主目標不重複受到冰爆傷害" },
        } },
        assumptions = { "iceberg_damage_kv" },
      },
      {
        id = "a34_frost_breath", source_ability_id = "A34R", action = "E", ability_type = "active",
        name = "冰霜之息", max_level = 4, status = "supported",
        description = "朝前方灑出冰霜（直線投射物，速度 1250、飛行 1200），命中的每個敵人受 100/175/250/325 魔法傷害，之後 5 秒每秒 50/60/70/80 傷害。",
        capabilities = { "projectile_linear", "projectile_on_hit_dot" },
        targeting = { kind = "direction", team = "enemy", range = { 1200, 1200, 1200, 1200 },
          shape = { kind = "line", width = 325 }, visibility = "direction_any" },
        timing = { cast_point = 0.25, cooldown_starts = "on_cast" },
        resources = { cooldown = { 10, 9, 8, 7 }, mana_cost = { 100, 120, 140, 160 } },
        effects = {
          { kind = "projectile", motion = "linear", speed = 1250, radius = 162.5, max_hits = 64 },
          { kind = "damage", amount = { 100, 175, 250, 325 }, damage_kind = "magical", delivered_by = "projectile" },
          { kind = "damage_over_time", amount_per_second = { 50, 60, 70, 80 }, duration = { 5, 5, 5, 5 }, damage_kind = "magical", delivered_by = "projectile" },
        },
        lifecycle = lifecycle({ target_invalid = "直線彈不需單位目標" }),
        implementation = { kind = "rust_handler", module = "No3_frost_breath", note = "G-1 Linear 穿透＋命中 DoT（Spec.dot）" },
        bot_usage = { target = "direction", when = "朝 1200 內敵方英雄或兵群方向" },
        verification = { scenarios = {
          { name = "rank_values", expect = "每個被穿過的敵人受 100/175/250/325，之後 5 秒每秒 50/60/70/80" },
          { name = "pierce_once", expect = "同一敵人只受一次命中傷害與一筆 DoT" },
        } },
        assumptions = { "frost_breath_width", "frost_breath_distance", "frost_breath_dot_kv" },
      },
      {
        id = "a34_rapid_cooling", source_ability_id = "A34T", action = "R", ability_type = "ultimate",
        name = "急速冷卻", max_level = 3, status = "supported",
        required_hero_level = { 6, 12, 18 },
        description = "被動：施放基礎技能時，其他基礎技能冷卻減少 2 秒，最大法力 +300/400/500。主動：22 秒內基礎技能冷卻減免 40/50/60%。",
        capabilities = { "cdr_on_cast_hook", "permanent_stat_bonus", "cooldown_reduction" },
        targeting = { kind = "self", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 100, 90, 80 }, mana_cost = { 100, 200, 300 } },
        effects = {
          { kind = "stat_bonus_permanent", stats = { mana_bonus = { 300, 400, 500 } } },
          { kind = "stat_buff", duration = { 22, 22, 22 }, stats = { cooldown_reduction_pct = { 40, 50, 60 } } },
          { kind = "cooldown_refund_on_cast", seconds = { 2, 2, 2 } },
        },
        lifecycle = lifecycle({ caster_death = "主動減免移除；被動保留" }),
        implementation = { kind = "rust_handler", module = "No4_rapid_cooling", note = "on_learn 最大法力；execute 22 秒 cooldown_percentage；被動減冷卻由 host content cast_cooldown_refund 結算" },
        bot_usage = { target = "self", when = "W／E／R 中至少兩招在冷卻且即將交戰" },
        verification = { scenarios = {
          { name = "passive_cdr", expect = "施放 Q／W／E 任一招後，另外兩招剩餘冷卻各 −2 秒（不低於 0）" },
          { name = "mana_bonus", expect = "學會 rank 1～3 後最大法力 +300/400/500，不重灌法力" },
          { name = "active_cdr", expect = "22 秒內基礎技能冷卻 ×(1−40/50/60%)" },
        } },
        assumptions = { "cooling_levels" },
      },
    },
  }
end
