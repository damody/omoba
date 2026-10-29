-- H03 ability contract (source A34 current, history_nobu A34.txt / A34_Matsu.lua).
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
  -- A34T_OnHit（A34_Matsu.lua:10-40）：急速冷卻主動期間，W/E/R 每次命中英雄疊一層魔抗減免。
  local shred_note = "急速冷卻主動期間命中英雄：疊魔抗減免（數值與層數由急速冷卻的主動 buff 決定）"
  return {
    schema_version = 2,
    abilities = {
      {
        id = "a34_frost_bolt", source_ability_id = "A34W", action = "Q", ability_type = "active",
        name = "寒霜彈", max_level = 4, status = "supported",
        description = "對敵人或建築物投出可閃避的追蹤寒霜彈（速度 2000），命中造成 100/200/300/400 魔法傷害並結凍 1/1.5/2/2.5 秒。",
        capabilities = { "projectile_tracking", "projectile_dodge", "unit_damage", "stun", "projectile_vs_building", "on_hit_stacking_debuff" },
        targeting = { kind = "unit", team = "enemy", range = { 600, 600, 600, 600 }, shape = { kind = "none" }, visibility = "target_must_be_visible" },
        timing = { cast_point = 0.25, cooldown_starts = "on_cast" },
        resources = { cooldown = { 14, 14, 14, 14 }, mana_cost = { 100, 125, 150, 175 } },
        effects = {
          { kind = "projectile", motion = "homing", speed = 2000, radius = 16, lifetime = 3 },
          { kind = "damage", amount = { 100, 200, 300, 400 }, damage_kind = "magical", delivered_by = "projectile" },
          { kind = "control", control = "stun", duration = { 1, 1.5, 2, 2.5 }, delivered_by = "projectile" },
          { kind = "custom", note = "TrackingProjectile Dodgeable 1：目標躲避（ProjectileDodge）時寒霜彈失效、不結算" },
          { kind = "custom", note = shred_note },
        },
        lifecycle = lifecycle(),
        implementation = { kind = "rust_handler", module = "No1_frost_bolt", note = "G-1 Homing＋dodgeable＋hit_stack＋control＋buildings=true" },
        bot_usage = { target = "enemy_unit", when = "可見的敵方英雄在 600 內，優先正在施法或追擊者" },
        verification = { scenarios = {
          { name = "rank_values", expect = "命中時 rank 1～4 造成 100/200/300/400 魔法傷害並暈眩 1/1.5/2/2.5 秒；施放當下不扣血" },
          { name = "travel_time", expect = "遠距目標比近距晚命中（速度 2000）" },
          { name = "building_target", expect = "可對敵方建築施放並命中" },
          { name = "dodge", expect = "目標施放躲避技能（例如伊達政宗踏火無痕）後，飛行中的寒霜彈消失，不造成傷害與結凍" },
        } },
        assumptions = { "distance_units" },
      },
      {
        id = "a34_iceberg_strike", source_ability_id = "A34E", action = "W", ability_type = "active",
        name = "冰山之襲", max_level = 4, status = "supported",
        description = "以冰山轟擊指定敵軍造成 100 魔法傷害，再對其周圍 335 內所有敵人（含指定目標）造成 100/200/300/400 冰爆傷害；被擊中的單位移動速度 −30%、攻擊速度 −20，持續 5 秒。不能對建築物施放。",
        capabilities = { "unit_damage", "aoe_circle", "slow_debuff", "on_hit_stacking_debuff" },
        targeting = { kind = "unit", team = "enemy", range = { 700, 700, 700, 700 },
          shape = { kind = "circle", radius = { 335, 335, 335, 335 } }, visibility = "target_must_be_visible" },
        timing = { cast_point = 0.1, cooldown_starts = "on_cast" },
        resources = { cooldown = { 7, 7, 7, 7 }, mana_cost = { 125, 145, 165, 185 } },
        effects = {
          -- KV AbilityDamage：半徑內每個敵人（含指定目標）各一次。
          { kind = "damage", amount = { 100, 200, 300, 400 }, damage_kind = "magical", applies_to = "all_in_radius" },
          -- KV aoe_damage：只打指定目標，先於冰爆。
          { kind = "damage", amount = { 100, 100, 100, 100 }, damage_kind = "magical", applies_to = "target" },
          -- attack_speed_reduction 是攻速常數點數（ATTACKSPEED_BONUS_CONSTANT），不是百分比。
          { kind = "slow", move_speed_pct = { 30, 30, 30, 30 }, attack_speed_reduction = { 20, 20, 20, 20 }, duration = { 5, 5, 5, 5 } },
          { kind = "custom", note = shred_note },
        },
        lifecycle = lifecycle({ target_invalid = "結算時主目標已死亡或不可見則施放失敗，不扣費" }),
        implementation = { kind = "rust_handler", module = "No2_iceberg_strike", note = "deal_damage＋query_enemies_in_range（排除建築）＋通用 slow buff＋hit_stack" },
        bot_usage = { target = "enemy_unit", when = "目標周圍 335 內有多名敵人，或清線" },
        verification = { scenarios = {
          { name = "rank_values", expect = "指定目標受 100＋100/200/300/400，周圍 335 內其他敵人各受 100/200/300/400，皆為魔法傷害" },
          { name = "slow", expect = "所有被擊中者移速 −30%、攻速 −20 點，5 秒" },
          { name = "radius_edge", expect = "335 外、友軍與建築物不受影響；不能對建築物施放" },
        } },
        assumptions = { "iceberg_damage_kv" },
      },
      {
        id = "a34_frost_breath", source_ability_id = "A34R", action = "E", ability_type = "active",
        name = "冰霜之息", max_level = 4, status = "supported",
        description = "朝前方灑出冰霜（直線投射物，速度 1250、飛行 1200，寬度由半徑 150 漸寬到 500），命中的每個敵人受 100/175/250/325 魔法傷害，之後 5 秒每 1 秒受 50/60/70/80 傷害。",
        capabilities = { "projectile_linear", "projectile_widening", "projectile_on_hit_dot", "discrete_dot", "on_hit_stacking_debuff" },
        targeting = { kind = "direction", team = "enemy", range = { 800, 800, 800, 800 },
          shape = { kind = "line", width = 650 }, visibility = "direction_any" },
        timing = { cast_point = 0.1, cooldown_starts = "on_cast" },
        resources = { cooldown = { 10, 9, 8, 7 }, mana_cost = { 100, 120, 140, 160 } },
        effects = {
          { kind = "projectile", motion = "linear", speed = 1250, radius = 150, end_radius = 500, distance = 1200, max_hits = 64 },
          { kind = "damage", amount = { 100, 175, 250, 325 }, damage_kind = "magical", delivered_by = "projectile" },
          { kind = "damage_over_time", amount_per_second = { 50, 60, 70, 80 }, duration = { 5, 5, 5, 5 }, interval = 1, damage_kind = "magical", delivered_by = "projectile" },
          { kind = "custom", note = shred_note },
        },
        lifecycle = lifecycle({ target_invalid = "直線彈不需單位目標" }),
        implementation = { kind = "rust_handler", module = "No3_frost_breath", note = "G-1 Linear 穿透＋end_radius 漸寬＋命中離散 DoT（dot_interval）＋hit_stack" },
        bot_usage = { target = "direction", when = "朝 800 內敵方英雄或兵群方向" },
        verification = { scenarios = {
          { name = "rank_values", expect = "每個被穿過的敵人受 100/175/250/325，之後第 1～5 秒各受一次 50/60/70/80" },
          { name = "pierce_once", expect = "同一敵人只受一次命中傷害與一筆 DoT" },
          { name = "widening", expect = "出手處只打到半徑 150 內的敵人，飛到 1200 時半徑 500" },
        } },
        assumptions = { "frost_breath_width", "frost_breath_distance", "frost_breath_dot_kv", "frost_breath_dot_ticks" },
      },
      {
        id = "a34_rapid_cooling", source_ability_id = "A34T", action = "R", ability_type = "ultimate",
        name = "急速冷卻", max_level = 3, status = "supported",
        required_hero_level = { 6, 12, 18 },
        description = "被動：施放基礎技能時，其他基礎技能冷卻減少 2 秒，最大法力 +300/400/500。主動 22 秒：寒霜彈、冰山之襲、冰霜之息的剩餘冷卻不超過全冷卻的 60/50/40%（含施放前已在冷卻的）；每施放一個技能回復 100/150/200 法力；這三招命中英雄時疊魔抗減免（每層 5/10/15、最多 3 層、3 秒，滿層不刷新）。",
        capabilities = { "cdr_on_cast_hook", "permanent_stat_bonus", "cooldown_cap_buff", "mana_on_cast", "on_hit_stacking_debuff" },
        targeting = { kind = "self", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 100, 90, 80 }, mana_cost = { 100, 200, 300 } },
        effects = {
          { kind = "stat_bonus_permanent", stats = { mana_bonus = { 300, 400, 500 } } },
          { kind = "custom", note = "主動 modifier_A34T：每 0.1 秒把 A34W/E/R 剩餘冷卻壓到 ≤ GetCooldown(-1)×(1−cd_recover%)（A34_Matsu.lua:100-113）；OnAbilityExecuted 回魔（120-123）",
            duration = { 22, 22, 22 }, cooldown_cap_pct = { 40, 50, 60 }, mana_on_cast = { 100, 150, 200 } },
          { kind = "custom", note = "A34T_OnHit：每層魔抗減免點數、層數上限 3（腳本寫死）、debuff 3 秒、滿層不刷新",
            magic_resistance_reduce = { 5, 10, 15 }, max_stacks = 3, stack_duration = 3 },
          { kind = "cooldown_refund_on_cast", seconds = { 2, 2, 2 } },
        },
        lifecycle = lifecycle({ caster_death = "主動效果移除；被動保留" }),
        implementation = { kind = "rust_handler", module = "No4_rapid_cooling", note = "on_learn 最大法力；execute 22 秒 cooldown_cap＋hit_stack 規則 buff；hero UnitScript on_skill_cast 回魔；被動減冷卻由 host content cast_cooldown_refund 結算" },
        bot_usage = { target = "self", when = "W／E／R 中至少兩招在冷卻且即將交戰" },
        verification = { scenarios = {
          { name = "passive_cdr", expect = "施放 Q／W／E 任一招後，另外兩招剩餘冷卻各 −2 秒（不低於 0）" },
          { name = "mana_bonus", expect = "學會 rank 1～3 後最大法力 +300/400/500，不重灌法力" },
          { name = "active_cap", expect = "22 秒內 Q／W／E 剩餘冷卻不超過全冷卻 ×(1−40/50/60%)，含施放前已在冷卻的；大招與道具不受影響" },
          { name = "mana_on_cast", expect = "主動期間每施放一個技能回復 100/150/200 法力" },
          { name = "resist_shred", expect = "主動期間 Q／W／E 命中英雄，每次疊一層 −5/10/15 魔抗、最多 3 層、3 秒；滿層不刷新；小兵與建築不受影響" },
        } },
        assumptions = { "cooling_levels", "magic_resist_points", "rapid_cooling_self_mana" },
      },
    },
  }
end
