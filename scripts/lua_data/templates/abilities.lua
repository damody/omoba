return function(ctx)
  local summon_guidance = ctx.include("templates/summon_guidance.lua")
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
      description = "依自身朝向，在前方兩排召喚雜賀鐵炮兵協助作戰；地形與友軍佔位會調整合法出生位置。\n"
        .. assert(summon_guidance.saika_gunner, "missing authored summon guidance"),
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
      zone_lifetime = "independent",
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
        zone_damage = {10.0,15.0,20.0,25.0},
        zone_interval = {0.5,0.5,0.5,0.5},
        zone_duration = {2.0,2.0,2.0,2.0},
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
      if ability.id == "apprentice_mend" then
        ability.display_name="定心療癒"
        ability.cast_type="channeled"
        ability.extras={pulse_heal={35,55,75,95},channel_duration={2,2,2,2},pulse_interval={0.5,0.5,0.5,0.5}}
        ability.effects={{kind="recovery_channel_self",amount_key="pulse_heal",duration_key="channel_duration",interval_key="pulse_interval"}}
      end
      abilities[#abilities + 1] = ability
    end
  end
  -- Append after apprentice skills to preserve every previously exported ID.
  -- Change the original bolt after cloning: apprentice balance remains independent.
  for _,ability in ipairs(abilities)do if ability.id=='lumen_bolt' then
    ability.description='對指定存活敵人造成80/125/170/215法術傷害，另削減20/30/40/50魔力，最多扣至0；沒有魔力池則不削減。魔力削減不是生命傷害，不受護盾或禁療影響，也不觸發目標自行耗魔效果。'
    ability.extras.mana_drain={20,30,40,50}
    ability.effects[#ability.effects+1]={kind='drain_mana_enemy',amount_key='mana_drain'}
  end end
  for _,ability in ipairs(ctx.include('templates/moba_archetypes.lua').abilities) do
    abilities[#abilities+1]=ability
  end
  local aid
  for _,ability in ipairs(abilities) do if ability.id=='lumen_mend' then aid=copy(ability) end end
  assert(aid)
  aid.id='lumen_aid';aid.display_name='晨光援護';aid.description='治療指定存活友方單位，也可治療自身，並給予70/100/130/160護盾，持續2秒；與既有護盾取較大剩餘量、較長時間，不累加。施放成功後另回復施法者自身10/15/20/25魔力，最多回到上限，不替目標回魔。仍須先支付原有魔力消耗；護盾與回魔不受禁療影響。'
  aid.extras.self_mana_restore={10,15,20,25}
  aid.extras.shield={70,100,130,160};aid.extras.shield_duration={2,2,2,2}
  aid.target_type='unit';aid.effects={{kind='heal_ally',amount_key='heal'},{kind='restore_mana_self',amount_key='self_mana_restore'},{kind='shield_ally',amount_key='shield',duration_key='shield_duration'},{kind='cleanse_ally'}}
  for _,level in ipairs(aid.levels) do level.range=600 end
  abilities[#abilities+1]=aid
  abilities[#abilities+1]=ctx.include('templates/moba_scatter.lua').ability
  -- Append-only shared linear skill. Existing skills and numeric IDs stay stable.
  abilities[#abilities+1]={id='lumen_ray',ue={native_only=true},display_name='貫光射線',
    description='朝指定地點瞬間施放寬120的直線光束，對線上存活敵方非建築單位各造成一次100/150/200/250魔法傷害。超距離截短至900；包含兩端圓帽與邊界，不受地形阻擋、不移動自身，定身時仍可施放。原地無法指定方向。',
    ability_type='active',target_type='point',cast_type='instant',max_level=4,
    levels={{range=900,cooldown=9,mana_cost=45},{range=900,cooldown=8,mana_cost=50},
      {range=900,cooldown=7,mana_cost=55},{range=900,cooldown=6,mana_cost=60}},
    extras={damage={100,150,200,250},line_width={120,120,120,120}},
    effects={{kind='line_damage',amount_key='damage',width_key='line_width',damage_kind='magical'}}}
  -- Append-only G-1 projectile skills: host-owned flight, damage on contact, never instant.
  abilities[#abilities+1]={id='lumen_frost_bolt',ue={native_only=true},display_name='寒霜彈',
    description='向指定存活敵人發射追蹤寒霜彈，飛行速度900，命中時造成90/135/180/225魔法傷害並減速25%，持續1.5秒。距離越遠越晚命中；目標死亡或失效時彈體消失且不造成傷害；施法者死亡後已發射的彈體仍會命中。',
    ability_type='active',target_type='unit',cast_type='instant',max_level=4,
    levels={{range=700,cooldown=8,mana_cost=50},{range=700,cooldown=7.5,mana_cost=55},
      {range=700,cooldown=7,mana_cost=60},{range=700,cooldown=6.5,mana_cost=65}},
    extras={damage={90,135,180,225},speed={900,900,900,900},radius={16,16,16,16},lifetime={3,3,3,3},
      slow={0.25,0.25,0.25,0.25},slow_duration={1.5,1.5,1.5,1.5}},
    effects={{kind='damage',amount_key='damage',damage_kind='magical'},
      {kind='slow_enemy',reduction_key='slow',duration_key='slow_duration'}},
    projectile={motion='homing',speed_key='speed',radius_key='radius',lifetime_key='lifetime'}}
  abilities[#abilities+1]={id='lumen_frost_lance',ue={native_only=true},display_name='冰晶飛槍',
    description='朝指定方向擲出直線冰晶飛槍，飛行速度1200、寬120、飛行1000距離，沿途依先接觸先命中的順序貫穿最多3個存活敵方非建築單位，各造成一次70/105/140/175魔法傷害。同一單位只受一次傷害；原地無法指定方向。',
    ability_type='active',target_type='point',cast_type='instant',max_level=4,
    levels={{range=1000,cooldown=10,mana_cost=55},{range=1000,cooldown=9.5,mana_cost=60},
      {range=1000,cooldown=9,mana_cost=65},{range=1000,cooldown=8.5,mana_cost=70}},
    extras={damage={70,105,140,175},speed={1200,1200,1200,1200},radius={60,60,60,60}},
    effects={{kind='damage',amount_key='damage',damage_kind='magical'}},
    projectile={motion='linear',speed_key='speed',radius_key='radius',max_hits=3}}
  -- Append-only G-2/G-3 demo: stored uses + fixed emplacement summon at a point.
  abilities[#abilities+1]={id='lumen_sentry',ue={native_only=true},display_name='晨光哨塔',
    description='在指定地點設置一座無法移動的晨光哨塔，持續30秒，攻擊力為30加上英雄每級10點。每15秒儲存1次，最多2/3/4/5次，每次施放消耗1次；施法者死亡時哨塔消失。',
    ability_type='active',target_type='point',cast_type='instant',max_level=4,
    levels={{range=600,cooldown=1,mana_cost=40},{range=600,cooldown=1,mana_cost=45},
      {range=600,cooldown=1,mana_cost=50},{range=600,cooldown=1,mana_cost=55}},
    extras={charges={2,3,4,5},charge_time={15,15,15,15}},
    charges={max_key='charges',recharge_key='charge_time'},
    summon={unit='lumen_sentry_turret'}}
  -- Append-only G-2 toggle orb demo (A02 W shape): mana per attack, auto-off when short.
  abilities[#abilities+1]={id='lumen_venom_orb',ue={native_only=true},display_name='蝕光法球',
    description='切換：開啟期間每次普攻消耗10法力，命中時額外造成30/50/70/90魔法傷害，並使目標移速與攻速降低9/13/17/21%，持續4秒。法力不足時自動關閉，該次攻擊不附加效果。',
    ability_type='toggle',target_type='none',cast_type='instant',max_level=4,
    levels={{cooldown=0,mana_cost=0},{cooldown=0,mana_cost=0},{cooldown=0,mana_cost=0},{cooldown=0,mana_cost=0}},
    extras={orb_mana={10,10,10,10},orb_damage={30,50,70,90},orb_slow={0.09,0.13,0.17,0.21},orb_slow_time={4,4,4,4}},
    attack_orb={mana_per_attack_key='orb_mana',amount_key='orb_damage',damage_kind='magical',
      move_slow_key='orb_slow',attack_slow_key='orb_slow',slow_duration_key='orb_slow_time'}}
  -- Append-only G-2 pulse channel demos: host-owned lifecycle, payload per pulse.
  abilities[#abilities+1]={id='lumen_tremor',ue={native_only=true},display_name='晨光震波',
    description='引導：在指定地點每0.25秒釋放一波震波，共12/14/16/18波，每波對半徑340內的敵人造成20/25/30/35魔法傷害並暈眩0.2秒。受到暈眩或沉默、移動、死亡或回城都會立即中斷；中斷後不再出波，也不退還魔力與冷卻。',
    ability_type='active',target_type='point',cast_type='channeled',max_level=4,
    levels={{range=700,cooldown=30,mana_cost=100},{range=700,cooldown=27,mana_cost=110},
      {range=700,cooldown=24,mana_cost=120},{range=700,cooldown=21,mana_cost=130}},
    extras={wave_interval={0.25,0.25,0.25,0.25},wave_count={12,14,16,18},wave_radius={340,340,340,340},
      wave_damage={20,25,30,35},wave_stun={0.2,0.2,0.2,0.2}},
    effects={{kind='damage',amount_key='wave_damage',damage_kind='magical'},{kind='stun_enemy',duration_key='wave_stun'}},
    channel={anchor='point',interval_key='wave_interval',pulses_key='wave_count',radius_key='wave_radius'}}
  abilities[#abilities+1]={id='lumen_tether',ue={native_only=true},display_name='晨光牽引',
    description='引導：鎖定一名敵人，每0.5秒對其造成30/40/50/60魔法傷害並減速30%（持續0.6秒），共8波。目標死亡、失效，或施法者受控、移動、死亡時立即中斷。',
    ability_type='active',target_type='unit',cast_type='channeled',max_level=4,
    levels={{range=600,cooldown=16,mana_cost=70},{range=600,cooldown=15,mana_cost=80},
      {range=600,cooldown=14,mana_cost=90},{range=600,cooldown=13,mana_cost=100}},
    extras={tether_interval={0.5,0.5,0.5,0.5},tether_count={8,8,8,8},tether_damage={30,40,50,60},
      tether_slow={0.3,0.3,0.3,0.3},tether_slow_time={0.6,0.6,0.6,0.6}},
    effects={{kind='damage',amount_key='tether_damage',damage_kind='magical'},
      {kind='slow_enemy',reduction_key='tether_slow',duration_key='tether_slow_time'}},
    channel={anchor='unit',interval_key='tether_interval',pulses_key='tether_count'}}
  -- Append-only G-4 forced displacement demos: host-owned motion, stunned while moved.
  abilities[#abilities+1]={id='lumen_repulse',ue={native_only=true},display_name='晨光震退',
    description='對一名敵人造成40/60/80/100魔法傷害，並在0.35秒內把它從施法者身邊直線擊退350距離，期間暈眩。撞到地形時停在牆前；其他單位不會阻擋。',
    ability_type='active',target_type='unit',cast_type='instant',max_level=4,
    levels={{range=500,cooldown=12,mana_cost=60},{range=500,cooldown=11,mana_cost=65},
      {range=500,cooldown=10,mana_cost=70},{range=500,cooldown=9,mana_cost=75}},
    extras={damage={40,60,80,100},push={350,350,350,350},push_time={0.35,0.35,0.35,0.35}},
    effects={{kind='damage',amount_key='damage',damage_kind='magical'},
      {kind='forced_move_enemy',motion='knockback',distance_key='push',duration_key='push_time'}}}
  abilities[#abilities+1]={id='lumen_hook',ue={native_only=true},display_name='晨光鉤鎖',
    description='朝指定方向擲出直線鉤鎖，飛行速度1600、寬120、飛行1100距離，撞牆即停。命中第一個敵方非建築單位時造成60/90/120/150魔法傷害，並在0.5秒內把它拉回施法者身旁，期間暈眩。',
    ability_type='active',target_type='point',cast_type='instant',max_level=4,
    levels={{range=1100,cooldown=16,mana_cost=90},{range=1100,cooldown=15,mana_cost=95},
      {range=1100,cooldown=14,mana_cost=100},{range=1100,cooldown=13,mana_cost=105}},
    extras={damage={60,90,120,150},speed={1600,1600,1600,1600},radius={60,60,60,60},
      pull={1100,1100,1100,1100},pull_time={0.5,0.5,0.5,0.5}},
    effects={{kind='damage',amount_key='damage',damage_kind='magical'},
      {kind='forced_move_enemy',motion='pull',distance_key='pull',duration_key='pull_time'}},
    projectile={motion='linear',speed_key='speed',radius_key='radius',max_hits=1,wall='stop'}}
  -- Append-only G-4 detection demo: a timed team true-sight area at a point.
  abilities[#abilities+1]={id='lumen_flare',ue={native_only=true},display_name='晨光顯影',
    description='在指定地點點亮半徑450的晨光，持續4/5/6/7秒：我方可看見範圍內的一切，包括隱形的敵人。仍會被地形與樹木遮擋視線，施法者死亡後依然持續。',
    ability_type='active',target_type='point',cast_type='instant',max_level=4,
    levels={{range=1000,cooldown=20,mana_cost=50},{range=1000,cooldown=18,mana_cost=50},
      {range=1000,cooldown=16,mana_cost=50},{range=1000,cooldown=14,mana_cost=50}},
    extras={flare_radius={450,450,450,450},flare_time={4,5,6,7}},
    effects={{kind='reveal_area',radius_key='flare_radius',duration_key='flare_time'}}}
  for _,ability in ipairs(abilities) do
    local extras=ability.extras
    local channel=ability.effects and ability.effects[1]
    if channel and channel.kind=='recovery_channel_self' then
      assert(#ability.effects==1 and extras,ability.id..' requires exclusive channel extras')
      local amounts=assert(extras[channel.amount_key],ability.id..' missing pulse amounts')
      local durations=assert(extras[channel.duration_key],ability.id..' missing channel durations')
      local intervals=assert(extras[channel.interval_key],ability.id..' missing pulse intervals')
      local function ranks(values)
        local result,identical={},true
        for _,value in ipairs(values)do result[#result+1]=string.format('%g',value);identical=identical and value==values[1] end
        return identical and result[1] or table.concat(result,'/')
      end
      local counts,totals={},{}
      for rank,amount in ipairs(amounts)do counts[rank]=durations[rank]/intervals[rank];totals[rank]=amount*counts[rank] end
      ability.description=string.format('站定引導%s秒，每%s秒恢復%s生命，共%s次（總計%s）。移動、停止、普攻、施放其他技能、生命受傷、暈眩或沉默會中斷；定身不會中斷，暫停時不計時。治療受最大生命與治療削減影響。',
        ranks(durations),ranks(intervals),ranks(amounts),ranks(counts),ranks(totals))
    end
    if extras and extras.zone_damage then
      local function ranks(values)
        local result={};local identical=true
        for _,value in ipairs(values) do result[#result+1]=string.format('%g',value);identical=identical and value==values[1] end
        if identical then return result[1] end
        return table.concat(result,'/')
      end
      ability.description=ability.description..string.format(' 每%s秒造成%s傷害，持續%s秒。',
        ranks(extras.zone_interval),ranks(extras.zone_damage),ranks(extras.zone_duration))
      if ability.zone_lifetime=='independent' then
        ability.description=ability.description..' 此區域在施法者死亡後仍持續至到期；重新施放會替換同一生命建立的舊區域。'
      end
    end
  end
  -- @hero_author:authored-include (scripts/hero_author.lua build): appended last.
  for _, ability in ipairs(ctx.include('templates/authored_heroes.lua').abilities or {}) do
    abilities[#abilities + 1] = ability
  end
  return abilities
end
