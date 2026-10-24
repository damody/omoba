return function(ctx)
  local abilities = {
    {
      id = "sniper_mode",
      ue = { ability_cue = { shape = "toggle", color_rgb = {40,240,255}, radius_cm = 900, duration_seconds = 1.25, range_cm = 900, payload_distance_scale = 100 } },
      display_name = "狙擊模式",
      icon = "data/hero1_1.png",
      description = "切換到狙擊模式，增加射程和傷害，但降低攻擊速度和移動速度。",
      ability_type = "toggle",
      target_type = "none",
      cast_type = "instant",
      max_level = 4,
      levels = {
        {
          cooldown = 0.0,
          mana_cost = 0.0,
          cast_time = 0.0,
          range = 0.0,
        },
        {
          cooldown = 0.0,
          mana_cost = 0.0,
          cast_time = 0.0,
          range = 0.0,
        },
        {
          cooldown = 0.0,
          mana_cost = 0.0,
          cast_time = 0.0,
          range = 0.0,
        },
        {
          cooldown = 0.0,
          mana_cost = 0.0,
          cast_time = 0.0,
          range = 0.0,
        },
      },
      extras = {
        range_bonus = {
          100.0,
          200.0,
          300.0,
          400.0,
        },
        damage_bonus = {
          0.15,
          0.25,
          0.35,
          0.45,
        },
        attack_speed_penalty = {
          -0.3,
          -0.3,
          -0.3,
          -0.3,
        },
        move_speed_penalty = {
          -0.5,
          -0.5,
          -0.5,
          -0.5,
        },
        accuracy_bonus = {
          0.1,
          0.1,
          0.1,
          0.1,
        },
      },
    },
    {
      id = "saika_reinforcements",
      aim_geometry = { kind = "facing_formation", rows = 2 },
      ue = { ability_cue = { shape = "formation", color_rgb = {25,115,255}, radius_cm = 260, duration_seconds = 1.5, payload_distance_scale = 100 } },
      display_name = "雜賀援軍",
      icon = "data/hero1_2.png",
      description = "依自身朝向，在前方兩排召喚雜賀鐵炮兵協助作戰；地形與友軍佔位會調整合法出生位置。",
      ability_type = "active",
      target_type = "point",
      cast_type = "instant",
      max_level = 4,
      levels = {
        {
          cooldown = 27.0,
          mana_cost = 90.0,
          cast_time = 0.5,
          cast_interrupt_on_damage = true,
          range = 800.0,
        },
        {
          cooldown = 26.0,
          mana_cost = 90.0,
          cast_time = 0.5,
          cast_interrupt_on_damage = true,
          range = 800.0,
        },
        {
          cooldown = 25.0,
          mana_cost = 90.0,
          cast_time = 0.5,
          cast_interrupt_on_damage = true,
          range = 800.0,
        },
        {
          cooldown = 24.0,
          mana_cost = 90.0,
          cast_time = 0.5,
          cast_interrupt_on_damage = true,
          range = 800.0,
        },
      },
      extras = {
        summon_count = {
          2.0,
          4.0,
          6.0,
          8.0,
        },
        duration = {
          45.0,
          50.0,
          55.0,
          60.0,
        },
        row_spacing = {
          60.0,
          60.0,
          60.0,
          60.0,
        },
        col_spacing = {
          60.0,
          60.0,
          60.0,
          60.0,
        },
        front_row_distance = {
          120.0,
          120.0,
          120.0,
          120.0,
        },
        formation_radius = {
          100.0,
          100.0,
          100.0,
          100.0,
        },
        gunner_hp = {
          350.0,
          400.0,
          450.0,
          500.0,
        },
        gunner_damage = {
          50.0,
          60.0,
          70.0,
          80.0,
        },
      },
    },
    {
      id = "rain_iron_cannon",
      ue = { ability_cue = { shape = "cone", color_rgb = {255,55,20}, duration_seconds = 1.25, payload_distance_scale = 100 } },
      display_name = "雨鐵砲",
      icon = "data/hero1_3.png",
      description = "被動：真正普攻命中時，以受擊點為扇形起點，沿攻擊方向的 90°、150 半徑內，對每個存活敵人造成一次 15/25/35/45% 攻擊力的真實傷害。包含受擊點與邊界；技能子彈與純視覺連發不觸發。",
      ability_type = "passive",
      target_type = "none",
      cast_type = "instant",
      max_level = 4,
      levels = {
        {
          cooldown = 0.0,
          mana_cost = 0.0,
          cast_time = 0.0,
          range = 0.0,
        },
        {
          cooldown = 0.0,
          mana_cost = 0.0,
          cast_time = 0.0,
          range = 0.0,
        },
        {
          cooldown = 0.0,
          mana_cost = 0.0,
          cast_time = 0.0,
          range = 0.0,
        },
        {
          cooldown = 0.0,
          mana_cost = 0.0,
          cast_time = 0.0,
          range = 0.0,
        },
      },
      extras = {
        true_damage_pct = {
          0.15,
          0.25,
          0.35,
          0.45,
        },
        aoe_radius = {
          150.0,
          150.0,
          150.0,
          150.0,
        },
        arc_half_angle_rad = {
          0.7853982,
          0.7853982,
          0.7853982,
          0.7853982,
        },
      },
    },
    {
      id = "three_stage_technique",
      ue = { ability_cue = { shape = "fan", color_rgb = {255,25,190}, duration_seconds = 1.2, spacing_cm = 120, payload_distance_scale = 100 } },
      display_name = "三段擊",
      icon = "data/hero1_4.png",
      description = "變身 5 秒：攻擊力 +200%，普攻特效改為連發 3 發子彈（只判定 1 次傷害），身上有紅色變身特效。",
      ability_type = "active",
      target_type = "none",
      cast_type = "instant",
      max_level = 4,
      levels = {
        {
          cooldown = 15.0,
          mana_cost = 80.0,
          cast_time = 0.0,
          range = 0.0,
        },
        {
          cooldown = 13.0,
          mana_cost = 90.0,
          cast_time = 0.0,
          range = 0.0,
        },
        {
          cooldown = 11.0,
          mana_cost = 100.0,
          cast_time = 0.0,
          range = 0.0,
        },
        {
          cooldown = 9.0,
          mana_cost = 110.0,
          cast_time = 0.0,
          range = 0.0,
        },
      },
      extras = {
        duration = {
          5.0,
          5.0,
          5.0,
          5.0,
        },
        atk_bonus_pct = {
          2.0,
          2.0,
          2.0,
          2.0,
        },
        multi_shot_count = {
          3.0,
          3.0,
          3.0,
          3.0,
        },
      },
    },
    {
      id = "flame_blade",
      aim_geometry = { kind = "point_circle", radius_key = "swipe_radius" },
      display_name = "炎刃",
      icon = "data/ability_icons/ability_flame_blade.png",
      description = "在指定地點爆發火焰，對圓形範圍內敵人各造成一次火焰傷害。",
      ability_type = "active",
      target_type = "point",
      cast_type = "instant",
      max_level = 4,
      levels = {
        {
          cooldown = 10.0,
          mana_cost = 100.0,
          cast_time = 0.0,
          range = 1100.0,
        },
        {
          cooldown = 10.0,
          mana_cost = 120.0,
          cast_time = 0.0,
          range = 1100.0,
        },
        {
          cooldown = 10.0,
          mana_cost = 140.0,
          cast_time = 0.0,
          range = 1100.0,
        },
        {
          cooldown = 10.0,
          mana_cost = 160.0,
          cast_time = 0.0,
          range = 1100.0,
        },
      },
      extras = {
        damage = {
          200.0,
          250.0,
          300.0,
          350.0,
        },
        swipe_radius = {
          100.0,
          100.0,
          100.0,
          100.0,
        },
      },
    },
    {
      id = "fire_dash",
      aim_geometry = { kind = "swept_path", width_key = "dash_width" },
      display_name = "火焰衝擊",
      icon = "data/ability_icons/ability_fire_dash.png",
      description = "向指定方向瞬間衝刺，碰到地形會停止；對實際路徑內敵人各造成一次火焰傷害。",
      ability_type = "active",
      target_type = "point",
      cast_type = "instant",
      max_level = 4,
      levels = {
        {
          cooldown = 30.0,
          mana_cost = 80.0,
          cast_time = 0.0,
          range = 900.0,
        },
        {
          cooldown = 30.0,
          mana_cost = 100.0,
          cast_time = 0.0,
          range = 1000.0,
        },
        {
          cooldown = 30.0,
          mana_cost = 120.0,
          cast_time = 0.0,
          range = 1100.0,
        },
        {
          cooldown = 30.0,
          mana_cost = 140.0,
          cast_time = 0.0,
          range = 1200.0,
        },
      },
      extras = {
        damage_per_tick = {
          50.0,
          60.0,
          70.0,
          80.0,
        },
        dash_duration = {
          0.5,
          0.5,
          0.5,
          0.5,
        },
        speed_bonus = {
          2.0,
          2.0,
          2.0,
          2.0,
        },
        dash_width = {
          150.0,
          150.0,
          150.0,
          150.0,
        },
      },
    },
    {
      id = "flame_assault",
      aim_geometry = { kind = "point_circle", radius_key = "radius" },
      display_name = "炎襲",
      icon = "data/ability_icons/ability_flame_assault.png",
      description = "在指定地點爆發火焰，對圓形範圍內敵人各造成一次火焰傷害並暈眩；留下燃燒區域。施法者死亡時熄滅，重新施放會替換舊區域。",
      ability_type = "ultimate",
      target_type = "point",
      cast_type = "instant",
      max_level = 4,
      levels = {
        {
          cooldown = 14.0,
          mana_cost = 120.0,
          cast_time = 0.0,
          range = 700.0,
        },
        {
          cooldown = 14.0,
          mana_cost = 140.0,
          cast_time = 0.0,
          range = 700.0,
        },
        {
          cooldown = 14.0,
          mana_cost = 160.0,
          cast_time = 0.0,
          range = 700.0,
        },
        {
          cooldown = 14.0,
          mana_cost = 180.0,
          cast_time = 0.0,
          range = 700.0,
        },
      },
      extras = {
        zone_damage = {25.0,35.0,45.0,55.0},
        zone_interval = {0.5,0.5,0.5,0.5},
        zone_duration = {3.0,3.0,3.0,3.0},
        damage = {
          200.0,
          300.0,
          400.0,
          500.0,
        },
        stun_duration = {
          0.3,
          0.6,
          0.9,
          1.2,
        },
        radius = {
          500.0,
          500.0,
          500.0,
          500.0,
        },
      },
    },
    {
      id = "matchlock_gun",
      display_name = "火繩銃",
      icon = "data/ability_icons/ability_matchlock_gun.png",
      description = "變身火繩銃 45 秒，增加射程與傷害，普攻有機率暈眩。期間提供 12.5%／18.75%／25% 普通攻擊物理吸血，依實際物理扣血回復生命；護盾吸收、建築與非敵對目標不計，並受治療削減影響。",
      ability_type = "active",
      target_type = "none",
      cast_type = "instant",
      max_level = 3,
      levels = {
        {
          cooldown = 110.0,
          mana_cost = 260.0,
          cast_time = 0.0,
          range = 0.0,
        },
        {
          cooldown = 110.0,
          mana_cost = 310.0,
          cast_time = 0.0,
          range = 0.0,
        },
        {
          cooldown = 110.0,
          mana_cost = 360.0,
          cast_time = 0.0,
          range = 0.0,
        },
      },
      extras = {
        duration = {
          45.0,
          45.0,
          45.0,
        },
        range_bonus = {
          700.0,
          800.0,
          900.0,
        },
        damage_bonus = {
          90.0,
          130.0,
          170.0,
        },
        lifesteal = { 0.125, 0.1875, 0.25 },
        stun_chance = {
          0.87,
          0.87,
          0.87,
        },
        stun_duration = {
          0.1,
          0.2,
          0.3,
        },
      },
    },
    {
      id = "lumen_bolt",
      ue = { native_only = true },
      display_name = "光束",
      description = "對指定敵人造成法術傷害。",
      ability_type = "active",
      target_type = "unit",
      cast_type = "instant",
      max_level = 4,
      levels = {
        { cooldown = 7.0, mana_cost = 45.0, range = 600.0 },
        { cooldown = 6.5, mana_cost = 50.0, range = 600.0 },
        { cooldown = 6.0, mana_cost = 55.0, range = 600.0 },
        { cooldown = 5.5, mana_cost = 60.0, range = 600.0 },
      },
      extras = { damage = { 80.0, 125.0, 170.0, 215.0 } },
      effects = { { kind = "damage", amount_key = "damage", damage_kind = "magical" } },
    },
    {
      id = "lumen_touch",
      ue = { native_only = true },
      display_name = "回春",
      description = "治療自身並恢復少量魔力；必須先付得起施法成本。",
      ability_type = "active",
      target_type = "none",
      cast_type = "instant",
      max_level = 4,
      levels = {
        { cooldown = 12.0, mana_cost = 55.0 },
        { cooldown = 11.0, mana_cost = 60.0 },
        { cooldown = 10.0, mana_cost = 65.0 },
        { cooldown = 9.0, mana_cost = 70.0 },
      },
      extras = { heal = { 70.0, 110.0, 150.0, 190.0 }, mana_restore = {20.0,25.0,30.0,35.0} },
      effects = { { kind = "heal_self", amount_key = "heal" },
        {kind="restore_mana_self",amount_key="mana_restore"} },
    },
    {
      id = "lumen_lance",
      ue = { native_only = true },
      display_name = "光槍",
      description = "對指定敵人造成高額法術傷害。",
      ability_type = "ultimate",
      target_type = "unit",
      cast_type = "instant",
      max_level = 4,
      levels = {
        { required_hero_level = 1, cooldown = 80.0, mana_cost = 120.0, range = 700.0 },
        { required_hero_level = 6, cooldown = 70.0, mana_cost = 140.0, range = 700.0 },
        { required_hero_level = 11, cooldown = 60.0, mana_cost = 160.0, range = 700.0 },
        { required_hero_level = 16, cooldown = 50.0, mana_cost = 180.0, range = 700.0 },
      },
      extras = { damage = { 180.0, 280.0, 380.0, 480.0 } },
      effects = { { kind = "damage", amount_key = "damage", damage_kind = "magical" } },
    },
    {
      id = "lumen_mend",
      ue = { native_only = true },
      display_name = "晨光再生",
      description = "大量治療自身。",
      ability_type = "active",
      target_type = "none",
      cast_type = "instant",
      max_level = 4,
      levels = {
        { cooldown = 25.0, mana_cost = 90.0 },
        { cooldown = 23.0, mana_cost = 100.0 },
        { cooldown = 21.0, mana_cost = 110.0 },
        { cooldown = 19.0, mana_cost = 120.0 },
      },
      extras = { heal = { 140.0, 220.0, 300.0, 380.0 } },
      effects = { { kind = "heal_self", amount_key = "heal" } },
    },
  }
  local function copy(value)
    if type(value) ~= "table" then return value end
    local result = {}
    for key, child in pairs(value) do result[key] = copy(child) end
    return result
  end
  -- Separate append-only skills avoid duplicate FFI handler registrations.
  for index = 1, #abilities do
    local source = abilities[index]
    if source.id:match("^lumen_") then
      local ability = copy(source)
      ability.id = source.id:gsub("^lumen_", "apprentice_")
      if ability.id == "apprentice_lance" then ability.levels[1].required_hero_level = 6 end
      abilities[#abilities + 1] = ability
    end
  end
  -- Append after apprentice skills to preserve every previously exported ID.
  for _,ability in ipairs(ctx.include('templates/moba_archetypes.lua').abilities) do
    abilities[#abilities+1]=ability
  end
  local aid
  for _,ability in ipairs(abilities) do if ability.id=='lumen_mend' then aid=copy(ability) end end
  assert(aid)
  aid.id='lumen_aid';aid.display_name='晨光援護';aid.description='治療指定存活隊友，也可治療自身。'
  aid.target_type='unit';aid.effects={{kind='heal_ally',amount_key='heal'}}
  for _,level in ipairs(aid.levels) do level.range=600 end
  abilities[#abilities+1]=aid
  abilities[#abilities+1]=ctx.include('templates/moba_scatter.lua').ability
  for _,ability in ipairs(abilities) do
    local extras=ability.extras
    if extras and extras.zone_damage then
      local function ranks(values)
        local result={};local identical=true
        for _,value in ipairs(values) do result[#result+1]=string.format('%g',value);identical=identical and value==values[1] end
        if identical then return result[1] end
        return table.concat(result,'/')
      end
      ability.description=ability.description..string.format(' 每%s秒造成%s傷害，持續%s秒。',
        ranks(extras.zone_interval),ranks(extras.zone_damage),ranks(extras.zone_duration))
    end
  end
  return abilities
end
