-- 村上義清 ability contract (source B29 current). Extras keys follow the
-- hero-authoring-guide §5.3 naming rule; the Rust handlers read only those.
return function(ctx)
  local function lifecycle(over)
    local l = {
      caster_death = "施法者死亡時尚未結算的效果取消；已造成的傷害與控制保留",
      target_invalid = "範圍技能不需單位目標；結算時已死亡或失效的單位不受影響",
      vision_lost = "範圍結算不看視野，霧中的敵人一樣受影響",
      cancel = "前搖內被打斷或取消不扣費、不進冷卻",
      expire = "暈眩、減速與減甲到期自然移除",
    }
    for k, v in pairs(over or {}) do l[k] = v end
    return l
  end
  return {
    schema_version = 2,
    abilities = {
      {
        id = "b29_ground_slam", source_ability_id = "B29W", action = "Q", ability_type = "active",
        name = "動地斧", max_level = 4, status = "supported",
        description = "猛擊地面，對自身周圍 500 內的敵軍造成 150/250/350/450 魔法傷害並暈眩 1/1.3/1.6/1.9 秒。",
        capabilities = { "aoe_circle", "unit_damage", "stun" },
        targeting = { kind = "none", team = "enemy", shape = { kind = "circle", radius = { 500, 500, 500, 500 } },
          visibility = "self_centered_hits_fogged_enemies" },
        timing = { cast_point = 0.3, cooldown_starts = "on_cast" },
        resources = { cooldown = { 14, 14, 14, 14 }, mana_cost = { 100, 120, 140, 160 } },
        effects = {
          { kind = "damage", amount = { 150, 250, 350, 450 }, damage_kind = "magical", applies_to = "enemies_in_radius" },
          { kind = "control", control = "stun", duration = { 1, 1.3, 1.6, 1.9 }, applies_to = "enemies_in_radius" },
        },
        lifecycle = lifecycle(),
        implementation = { kind = "rust_handler", module = "No1_ground_slam",
          note = "以施法者為中心 query_enemies_in_range（排除建築）＋deal_damage＋BuffId::Stun" },
        bot_usage = { target = "self", when = "500 內有敵方英雄，或 3 個以上敵方單位", avoid = "周圍沒有敵人時" },
        verification = { scenarios = {
          { name = "rank_values", expect = "rank 1～4 對 500 內每個敵人造成 150/250/350/450 魔法傷害、暈眩 1/1.3/1.6/1.9 秒" },
          { name = "radius_edge", expect = "距離 500 的敵人被擊中，501 不受影響；友軍與建築不受影響" },
          { name = "cast_point", expect = "按下後約 0.3 秒才出傷害" },
        } },
        assumptions = { "distance_units", "vscripts_missing", "ground_slam_targets", "spell_immunity_not_modeled" },
      },
      {
        id = "b29_quake_axe", source_ability_id = "B29E", action = "W", ability_type = "active",
        name = "震地斧", max_level = 4, status = "supported",
        description = "向指定方向跳躍最多 300 距離並猛擊地面，對落點周圍 500 內的敵軍造成 200 魔法傷害並減緩 18/22/26/30% 移動速度 8 秒；接下來 8 秒獲得破軍靈光，使自身周圍 800 內的敵軍護甲降低 4/7/10/13（每 0.5 秒刷新，離開後保留 1 秒）。",
        capabilities = { "leap", "aoe_circle", "unit_damage", "slow_debuff", "aura" },
        targeting = { kind = "direction", team = "enemy", range = { 0, 0, 0, 0 },
          shape = { kind = "circle", radius = { 500, 500, 500, 500 } }, visibility = "direction_any_hits_fogged_enemies" },
        timing = { cast_point = 0.15, cooldown_starts = "on_cast" },
        resources = { cooldown = { 20, 18, 16, 14 }, mana_cost = { 105, 125, 145, 165 } },
        effects = {
          { kind = "dash", max_distance = { 300, 300, 300, 300 }, note = "瞬間跳躍，受地形碰撞截斷" },
          { kind = "damage", amount = { 200, 200, 200, 200 }, damage_kind = "magical", applies_to = "enemies_near_landing" },
          { kind = "slow", move_speed_pct = { 18, 22, 26, 30 }, duration = { 8, 8, 8, 8 }, applies_to = "enemies_near_landing" },
          { kind = "stat_buff", duration = { 8, 8, 8, 8 }, stats = { armor_bonus = { -4, -7, -10, -13 } },
            radius = { 800, 800, 800, 800 }, refresh_interval = { 0.5, 0.5, 0.5, 0.5 }, linger = { 1, 1, 1, 1 },
            applies_to = "enemies_around_caster_aura", note = "破軍靈光：hero UnitScript on_tick 每 refresh_interval 對 radius 內敵人套 linger 秒減甲" },
        },
        lifecycle = lifecycle({
          caster_death = "施法者死亡時破軍靈光立即結束，不再刷新；已套上的減甲最多保留 1 秒",
          expire = "8 秒後靈光結束；減速 8 秒、減甲 1 秒自然到期",
        }),
        implementation = { kind = "rust_handler", module = "No2_quake_axe",
          note = "swept_dash 跳躍＋落點 500 傷害／通用減速；靈光標記 buff 以 rank 命名，hero_unit.rs on_tick 以 query_enemies_in_range 刷新減甲" },
        bot_usage = { target = "direction", when = "敵方英雄在 300＋500 內，或要追擊／團戰開場削甲", avoid = "需要位移逃生時保留" },
        verification = { scenarios = {
          { name = "leap", expect = "指定點超過 300 時只跳 300；指定點在 300 內則跳到該點；撞牆截斷" },
          { name = "slam_values", expect = "落點 500 內每個敵人受 200 魔法傷害、移速 −18/22/26/30%，持續 8 秒" },
          { name = "aura", expect = "施放後 8 秒內，跟著施法者移動的 800 範圍內敵人護甲 −4/7/10/13；離開範圍 1 秒內恢復；8 秒後不再刷新" },
          { name = "caster_death", expect = "施法者死亡後靈光不再刷新" },
        } },
        assumptions = { "distance_units", "vscripts_missing", "quake_leap", "quake_slow_duration", "quake_aura_kv", "spell_immunity_not_modeled" },
      },
      {
        id = "b29_morale_boost", source_ability_id = "B29R", action = "E", ability_type = "passive",
        name = "士氣提升", max_level = 4, status = "supported",
        description = "被動光環：自身周圍 600 內的友軍（含自身）攻擊傷害 +7/14/21/28%。並永久增加自身 4/8/12/16 力量（+80/160/240/320 最大生命、+0.4/0.8/1.2/1.6 生命回復、+4/8/12/16 攻擊）。",
        capabilities = { "aura", "ally_area_buff_query", "permanent_stat_bonus" },
        targeting = { kind = "self", team = "ally", shape = { kind = "circle", radius = { 600, 600, 600, 600 } },
          visibility = "aura_allies_any_visibility" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 0, 0, 0, 0 }, mana_cost = { 0, 0, 0, 0 } },
        effects = {
          { kind = "stat_buff", duration = { 0.5, 0.5, 0.5, 0.5 }, stats = { base_damage_outgoing_pct = { 7, 14, 21, 28 } },
            radius = { 600, 600, 600, 600 }, applies_to = "allies_in_radius_including_self", note = "光環：離開後保留 0.5 秒" },
          { kind = "stat_bonus_permanent", stats = { attack_damage_bonus = { 4, 8, 12, 16 }, health_regen = { 0.4, 0.8, 1.2, 1.6 },
            max_hp_bonus = { 80, 160, 240, 320 } }, note = "力量換算，見 strength_conversion" },
        },
        lifecycle = lifecycle({
          caster_death = "施法者死亡時光環消失；永久力量保留",
          target_invalid = "光環只影響存活友軍",
          vision_lost = "光環不看視野",
          cancel = "被動，無施放",
          expire = "友軍離開 600 後 0.5 秒失效",
        }),
        implementation = { kind = "rust_handler", module = "No3_morale_boost",
          note = "on_learn 以同一 buff id 掛永久 HealthBonus／HealthRegenConstant／BaseAttackBonusDamage 與 rank 標記；hero_unit.rs on_tick 每 0.5 秒送 host ally_area_buff 封包（600 內友軍，刷新 0.75 秒）" },
        verification = { scenarios = {
          { name = "aura_allies", expect = "600 內友方英雄、小兵與自身攻擊傷害 +7/14/21/28%；601 外不受影響；敵人不受影響" },
          { name = "strength", expect = "學會 rank 1～4 後最大生命 +80/160/240/320、回血 +0.4/0.8/1.2/1.6、攻擊 +4/8/12/16；升級取代不疊加" },
          { name = "not_castable", expect = "被動不能主動施放" },
        } },
        assumptions = { "morale_radius", "morale_passive_cast", "strength_conversion", "vscripts_missing" },
      },
      {
        id = "b29_earth_tremor", source_ability_id = "B29T", action = "R", ability_type = "ultimate",
        name = "大地震撼", max_level = 3, status = "supported",
        required_hero_level = { 6, 12, 18 },
        description = "引導：在指定地點半徑 700/800/900 的範圍內每 0.1 秒隨機打出一個震波，共 50/60/70 波；每個震波使中心 340 內的敵軍受到 60 魔法傷害並暈眩 0.2/0.3/0.4 秒。受控、移動、死亡或回城會中斷。",
        capabilities = { "channel", "random_target_det", "aoe_circle", "unit_damage", "stun" },
        targeting = { kind = "point", team = "enemy", range = { 1100, 1100, 1100 },
          shape = { kind = "circle", radius = { 700, 800, 900 } }, visibility = "point_any_hits_fogged_enemies" },
        timing = { cast_point = 0.3, cooldown_starts = "on_cast",
          channel = { anchor = "point", interval = 0.1, pulses = { 50, 60, 70 }, interrupts = "stun/silence/move/death/recall" } },
        resources = { cooldown = { 100, 100, 100 }, mana_cost = { 200, 225, 250 } },
        effects = {
          { kind = "damage", amount = { 60, 60, 60 }, damage_kind = "magical",
            wave_radius = { 340, 340, 340 }, wave_interval = { 0.1, 0.1, 0.1 }, wave_count = { 50, 60, 70 },
            applies_to = "enemies_near_each_wave" },
          { kind = "control", control = "stun", duration = { 0.2, 0.3, 0.4 }, applies_to = "enemies_near_each_wave" },
        },
        lifecycle = lifecycle({
          caster_death = "施法者死亡立即結束引導，之後不再出波",
          cancel = "前搖 0.3 秒內取消不扣費、不進冷卻；引導開始後中斷不退魔力與冷卻",
          expire = "最後一波後結束；每波暈眩 0.2/0.3/0.4 秒自然到期",
        }),
        implementation = { kind = "rust_handler", module = "No4_earth_tremor",
          note = "content channel（timing.channel）由 host 擁有引導；handler 只實作 on_channel_pulse，每波以決定性雜湊選中心" },
        bot_usage = { target = "point", when = "1100 內有 2 名以上敵方英雄聚集，且自身不會立刻被打斷", avoid = "敵方有可打斷的控制技能未交時" },
        verification = { scenarios = {
          { name = "wave_values", expect = "每波中心 340 內的敵人受 60 魔法傷害並暈眩 0.2/0.3/0.4 秒；340 在內、341 在外" },
          { name = "wave_count", expect = "完整引導共 50/60/70 波，相隔 0.1 秒" },
          { name = "wave_spread", expect = "震波中心分佈在施法點 700/800/900 半徑內，不會落在範圍外" },
          { name = "interrupt", expect = "被暈眩、移動或死亡後不再出波" },
        } },
        assumptions = { "distance_units", "vscripts_missing", "tremor_waves", "tremor_wave_position", "tremor_levels", "tremor_cast_point", "spell_immunity_not_modeled" },
      },
    },
  }
end
