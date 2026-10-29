-- 村上義清 ability contract (source B29 current). Extras keys follow the
-- hero-authoring-guide §5.3 naming rule; the Rust handlers read only those.
-- 原作對照與差異：parity.md（每招的原作 KV／腳本行號）。
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
        description = "猛擊地面，對自身周圍 550 內的敵軍造成 150/250/350/450 魔法傷害並暈眩 1/1.3/1.6/1.9 秒。",
        capabilities = { "aoe_circle", "unit_damage", "stun" },
        targeting = { kind = "none", team = "enemy", shape = { kind = "circle", radius = { 550, 550, 550, 550 } },
          visibility = "self_centered_hits_fogged_enemies" },
        timing = { cast_point = 0.3, cooldown_starts = "on_cast" },
        resources = { cooldown = { 14, 14, 14, 14 }, mana_cost = { 100, 120, 140, 160 } },
        effects = {
          { kind = "damage", amount = { 150, 250, 350, 450 }, damage_kind = "magical", applies_to = "enemies_in_radius" },
          { kind = "control", control = "stun", duration = { 1, 1.3, 1.6, 1.9 }, applies_to = "enemies_in_radius" },
        },
        lifecycle = lifecycle(),
        implementation = { kind = "rust_handler", module = "No1_ground_slam",
          note = "以施法者為中心 550（B29.lua:15 寫死）query_enemies_in_range（排除建築）＋deal_damage＋BuffId::Stun" },
        bot_usage = { target = "self", when = "550 內有敵方英雄，或 3 個以上敵方單位", avoid = "周圍沒有敵人時" },
        verification = { scenarios = {
          { name = "rank_values", expect = "rank 1～4 對 550 內每個敵人造成 150/250/350/450 魔法傷害、暈眩 1/1.3/1.6/1.9 秒" },
          { name = "radius_edge", expect = "距離 550 的敵人被擊中，551 不受影響；友軍與建築不受影響" },
          { name = "cast_point", expect = "按下後約 0.3 秒才出傷害" },
        } },
        assumptions = { "distance_units", "ground_slam_targets", "spell_immunity_not_modeled" },
      },
      {
        id = "b29_quake_axe", source_ability_id = "B29E", action = "W", ability_type = "active",
        name = "震地斧", max_level = 4, status = "supported",
        description = "向指定方向跳躍最多 300 距離（0.15 秒落地），起跳後 0.2 秒猛擊地面，對自身當下位置周圍 500 內的敵軍造成 200 魔法傷害並減緩 18/22/26/30% 移動速度 8 秒；同時獲得 8 秒破軍靈光，使自身周圍 800 內的敵軍護甲降低 4/7/10/13（每 0.5 秒刷新，離開後保留 1 秒）。",
        capabilities = { "leap", "aoe_circle", "unit_damage", "slow_debuff", "aura", "delayed_effect" },
        targeting = { kind = "direction", team = "enemy", range = { 0, 0, 0, 0 },
          shape = { kind = "circle", radius = { 500, 500, 500, 500 } }, visibility = "direction_any_hits_fogged_enemies" },
        timing = { cast_point = 0.15, cooldown_starts = "on_cast" },
        resources = { cooldown = { 20, 18, 16, 14 }, mana_cost = { 105, 125, 145, 165 } },
        effects = {
          { kind = "dash", max_distance = { 300, 300, 300, 300 }, duration = { 0.15, 0.15, 0.15, 0.15 },
            note = "modifier_knockback：固定 0.15 秒落地（速度＝距離／0.15），受地形碰撞截斷" },
          { kind = "damage", amount = { 200, 200, 200, 200 }, damage_kind = "magical", delay = { 0.2, 0.2, 0.2, 0.2 },
            applies_to = "enemies_near_caster_after_delay", note = "B29.lua:63-69：0.2 秒後以施法者當下位置為中心 500" },
          { kind = "slow", move_speed_pct = { 18, 22, 26, 30 }, duration = { 8, 8, 8, 8 }, applies_to = "enemies_near_caster_after_delay" },
          { kind = "stat_buff", duration = { 8, 8, 8, 8 }, stats = { armor_bonus = { -4, -7, -10, -13 } },
            radius = { 800, 800, 800, 800 }, refresh_interval = { 0.5, 0.5, 0.5, 0.5 }, linger = { 1, 1, 1, 1 },
            applies_to = "enemies_around_caster_aura", note = "破軍靈光：施放當下（起跳點）先套一次，之後 hero UnitScript on_tick 每 refresh_interval 對 radius 內敵人套 linger 秒減甲" },
        },
        lifecycle = lifecycle({
          caster_death = "施法者死亡時破軍靈光立即結束，不再刷新；尚未結算的猛擊取消；已套上的減甲最多保留 1 秒",
          expire = "8 秒後靈光結束；減速 8 秒、減甲 1 秒自然到期",
        }),
        implementation = { kind = "rust_handler", module = "No2_quake_axe",
          note = "swept_dash 跳躍（motion_packet_for 0.15 秒）＋delayed_effect 0.2 秒後以施法者位置結算 500 傷害／通用減速；靈光標記 buff 以 rank 命名，hero_unit.rs on_tick 以 query_enemies_in_range 刷新減甲" },
        bot_usage = { target = "direction", when = "敵方英雄在 300＋500 內，或要追擊／團戰開場削甲", avoid = "需要位移逃生時保留" },
        verification = { scenarios = {
          { name = "leap", expect = "指定點超過 300 時只跳 300；指定點在 300 內則跳到該點；撞牆截斷；任何距離都 0.15 秒落地" },
          { name = "slam_values", expect = "施放 0.2 秒後施法者位置 500 內每個敵人受 200 魔法傷害、移速 −18/22/26/30%，持續 8 秒" },
          { name = "aura", expect = "施放瞬間起跳點 800 內敵人護甲 −4/7/10/13；之後 8 秒內跟著施法者移動；離開範圍 1 秒內恢復；8 秒後不再刷新" },
          { name = "caster_death", expect = "施法者死亡後靈光不再刷新" },
        } },
        assumptions = { "distance_units", "quake_leap", "quake_slow_duration", "quake_aura_kv", "quake_slam_delay", "slow_stacking", "spell_immunity_not_modeled" },
      },
      {
        id = "b29_morale_boost", source_ability_id = "B29R", action = "E", ability_type = "passive",
        name = "士氣提升", max_level = 4, status = "supported",
        description = "被動光環：自身周圍 600 內的友軍（含自身）攻擊傷害 +7/14/21/28%。並永久增加自身 4/8/12/16 點力量。",
        capabilities = { "aura", "ally_area_buff_query", "permanent_stat_bonus" },
        targeting = { kind = "self", team = "ally", shape = { kind = "circle", radius = { 600, 600, 600, 600 } },
          visibility = "aura_allies_any_visibility" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 0, 0, 0, 0 }, mana_cost = { 0, 0, 0, 0 } },
        effects = {
          { kind = "stat_buff", duration = { 0.5, 0.5, 0.5, 0.5 }, stats = { base_damage_outgoing_pct = { 7, 14, 21, 28 } },
            radius = { 600, 600, 600, 600 }, refresh_interval = { 0.25, 0.25, 0.25, 0.25 },
            applies_to = "allies_in_radius_including_self", note = "光環：每 0.25 秒刷新、每次 0.5 秒，離開後保留 0.5 秒（Dota 光環預設）" },
          { kind = "custom", strength_bonus = { 4, 8, 12, 16 },
            note = "modifier_B29R_str MODIFIER_PROPERTY_STATS_STRENGTH_BONUS：送 StatKey::StatsStrengthBonus，由三圍 settle 換算" },
        },
        lifecycle = lifecycle({
          caster_death = "施法者死亡時光環消失；永久力量保留",
          target_invalid = "光環只影響存活友軍",
          vision_lost = "光環不看視野",
          cancel = "被動，無施放",
          expire = "友軍離開 600 後 0.5 秒失效",
        }),
        implementation = { kind = "rust_handler", module = "No3_morale_boost",
          note = "on_learn 以同一 buff id 掛永久 StatsStrengthBonus 與 rank 標記；hero_unit.rs on_tick 每 0.25 秒送 host ally_area_buff 封包（600 內友軍，刷新 0.5 秒）" },
        verification = { scenarios = {
          { name = "aura_allies", expect = "600 內友方英雄、小兵與自身攻擊傷害 +7/14/21/28%；601 外不受影響；敵人不受影響" },
          { name = "strength", expect = "學會 rank 1～4 後力量 +4/8/12/16（生命、回血、攻擊由三圍規則換算）；升級取代不疊加" },
          { name = "not_castable", expect = "被動不能主動施放" },
        } },
        assumptions = { "morale_radius", "morale_passive_cast", "strength_bonus", "morale_aura_linger" },
      },
      {
        id = "b29_earth_tremor", source_ability_id = "B29T", action = "R", ability_type = "ultimate",
        name = "大地震撼", max_level = 3, status = "supported",
        -- B29.txt: SPELL_IMMUNITY_ENEMIES_YES, search flag MAGIC_IMMUNE_ENEMIES (the stun lands; magical damage stays 0).
        pierces_magic_immunity = true,
        required_hero_level = { 6, 12, 18 },
        description = "引導 4.5/5.2/5.9 秒：每 0.1 秒在施法點 10～350/400/450 距離內隨機落下一擊，每擊 0.45 秒後才著地，使 340 內的敵軍受到 45 魔法傷害並暈眩 0.2 秒（建築受 30% 傷害、不暈眩）；引導結束前已出手的打擊仍會落下。引導期間己方獲得施法點附近的視野。受控、移動、死亡或回城會中斷。",
        capabilities = { "channel", "random_target_det", "aoe_circle", "unit_damage", "stun", "delayed_effect", "fow_viewer" },
        targeting = { kind = "point", team = "enemy", range = { 1100, 1100, 1100 },
          shape = { kind = "circle", radius = { 700, 800, 900 } }, visibility = "point_any_hits_fogged_enemies" },
        timing = { cast_point = 0.3, cooldown_starts = "on_cast",
          channel = { anchor = "point", interval = 0.1, pulses = { 45, 52, 59 }, interrupts = "stun/silence/move/death/recall" } },
        resources = { cooldown = { 100, 100, 100 }, mana_cost = { 200, 225, 250 } },
        effects = {
          { kind = "damage", amount = { 45, 45, 45 }, damage_kind = "magical",
            wave_radius = { 340, 340, 340 }, wave_interval = { 0.1, 0.1, 0.1 }, wave_count = { 45, 52, 59 },
            spread_min = { 10, 10, 10 }, spread_radius = { 350, 400, 450 }, delay = { 0.45, 0.45, 0.45 },
            building_ratio = { 0.3, 0.3, 0.3 },
            applies_to = "enemies_near_each_wave", note = "B29.lua:161-167、241-280：打擊點距施法點 RandomInt(10, B29T_Radius/2)，0.45 秒後落下；傷害寫死 45（AbilityDamage 60 未使用）；建築 30% 不暈眩" },
          { kind = "control", control = "stun", duration = { 0.2, 0.2, 0.2 }, applies_to = "enemies_near_each_wave",
            note = "B29.lua:267 寫死 0.2（B29T_Duration 未使用）" },
          { kind = "custom", vision_margin = { 100, 100, 100 }, vision_linger = { 0.5, 0.5, 0.5 },
            caster_vision_radius = { 100, 100, 100 }, caster_vision_duration = { 6, 6, 6 },
            note = "B29.lua:166：每擊對施法者隊伍在施法點開半徑 spread_radius＋100、0.5 秒視野（不顯形）" },
        },
        lifecycle = lifecycle({
          caster_death = "施法者死亡立即結束引導，之後不再出擊；尚未落下的打擊取消",
          cancel = "前搖 0.3 秒內取消不扣費、不進冷卻；引導開始後中斷不退魔力與冷卻；已出手的打擊仍落下",
          expire = "最後一擊落下後結束；每擊暈眩 0.2 秒自然到期",
        }),
        implementation = { kind = "rust_handler", module = "No4_earth_tremor",
          note = "content channel（timing.channel）由 host 擁有引導；on_channel_pulse 以決定性雜湊選打擊點並排程 delayed_effect 0.45 秒，on_delayed_effect 結算；每擊送施法點己方視野（reveal_zone detect=false）" },
        bot_usage = { target = "point", when = "1100 內有 2 名以上敵方英雄聚集，且自身不會立刻被打斷", avoid = "敵方有可打斷的控制技能未交時" },
        verification = { scenarios = {
          { name = "wave_values", expect = "每擊 0.45 秒後落下，340 內的敵人受 45 魔法傷害並暈眩 0.2 秒；建築受 13.5 魔法傷害、不暈眩；340 在內、341 在外" },
          { name = "wave_count", expect = "完整引導共 45/52/59 擊，相隔 0.1 秒" },
          { name = "wave_spread", expect = "打擊點距施法點 10～350/400/450" },
          { name = "interrupt", expect = "被暈眩、移動或死亡後不再出擊；中斷前已出手的打擊仍落下（施法者死亡則取消）" },
        } },
        assumptions = { "distance_units", "tremor_waves", "tremor_wave_position", "tremor_levels", "tremor_delay", "tremor_vision", "spell_immunity_not_modeled" },
      },
    },
  }
end
