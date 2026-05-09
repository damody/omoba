return function(ctx)
  local function barrel_frames(id)
    return {
      "assets/towers/" .. id .. "_barrel_frame_01.png",
      "assets/towers/" .. id .. "_barrel_frame_02.png",
      "assets/towers/" .. id .. "_barrel_frame_03.png",
    }
  end

  local function cake_splash_frames()
    return {
      "assets/towers/tower_cake_splash_frame_01.png",
      "assets/towers/tower_cake_splash_frame_02.png",
      "assets/towers/tower_cake_splash_frame_03.png",
      "assets/towers/tower_cake_splash_frame_04.png",
      "assets/towers/tower_cake_splash_frame_05.png",
      "assets/towers/tower_cake_splash_frame_06.png",
    }
  end

  local function fill_defaults(dst, defaults)
    for key, value in pairs(defaults) do
      if dst[key] == nil then
        dst[key] = value
      elseif type(dst[key]) == "table" and type(value) == "table" then
        fill_defaults(dst[key], value)
      end
    end
    return dst
  end

  local function default_base_barrel_render(id)
    return {
      render_mode = "base_barrel",
      base = "assets/towers/" .. id .. "_base.png",
      barrel = "assets/towers/" .. id .. "_barrel.png",
      barrel_frames = barrel_frames(id),
      barrel_animation = { fps = 10.0, loop = true, fire_fps = 18.0, fire_once = true },
      rotation_mode = "targeted",
      barrel_layout = "single",
      barrel_offset = { x = 0.0, y = -6.0 },
      barrel_pivot = { x = 0.5, y = 0.65 },
      muzzle_offset = { x = 0.0, y = -28.0 },
      default_angle_deg = 0.0,
      recoil = {
        mode = "directional",
        distance = 7.0,
        scale = 0.94,
        duration_ms = 70,
        return_ms = 110,
      },
    }
  end

  local function default_animated_area_render()
    return {
      render_mode = "animated_area",
      base = "assets/towers/tower_cake_splash_frame_01.png",
      animation = {
        frames = cake_splash_frames(),
        fps = 10.0,
        loop = true,
        fire_fps = 18.0,
        fire_once = true,
      },
      default_angle_deg = 0.0,
      recoil = {
        mode = "scale_pulse",
        distance = 0.0,
        scale = 0.9,
        duration_ms = 70,
        return_ms = 110,
      },
    }
  end

  local function apply_render_defaults(tower)
    local render = tower.render or {}
    if render.render_mode == "animated_area" then
      tower.render = fill_defaults(render, default_animated_area_render())
    else
      tower.render = fill_defaults(render, default_base_barrel_render(tower.id))
    end
  end

  local towers = {
    {
      id = "tower_dart",
      display_name = "飛鏢猴",
      atk = 10.0,
      asd_interval = 0.8,
      range = 350.0,
      bullet_speed = 1200.0,
      splash_radius = 0.0,
      hit_radius = 0.0,
      slow_factor = 0.0,
      slow_duration = 0.0,
      cost = 200,
      footprint = 10.0,
      hp = 1.0,
      turn_speed_deg = 360.0,
      render = {
        render_mode = "base_barrel",
        base = "assets/towers/tower_dart_base.png",
        barrel = "assets/towers/tower_dart_barrel.png",
        barrel_frames = barrel_frames("tower_dart"),
        barrel_animation = { fps = 12.0, loop = true, fire_fps = 22.0, fire_once = true },
        rotation_mode = "targeted",
        barrel_layout = "single",
        barrel_offset = { x = 0.0, y = -6.0 },
        barrel_pivot = { x = 0.5, y = 0.66 },
        muzzle_offset = { x = 0.0, y = -30.0 },
        recoil = {
          mode = "directional",
          distance = 6.0,
          scale = 0.95,
          duration_ms = 60,
          return_ms = 95,
        },
      },
      upgrades = {
        {
          {
            name = "長射程飛鏢",
            description = "射程 350→400",
            cost = 50,
            effects = {
              {
                type = "stat_mod",
                key = "AttackRangeBonus",
                value = 50.0,
                op = "add",
              },
            },
          },
          {
            name = "強化視力",
            description = "射程 →450, damage 10→15",
            cost = 100,
            effects = {
              {
                type = "stat_mod",
                key = "AttackRangeBonus",
                value = 50.0,
                op = "add",
              },
              {
                type = "stat_mod",
                key = "BaseDamageOutgoingPercentage",
                value = 0.5,
                op = "add",
              },
            },
          },
          {
            name = "剃刀銳利射擊",
            description = "穿透 +1, damage →20",
            cost = 200,
            effects = {
              {
                type = "behavior_flag",
                flag = "sharp_pierce",
              },
              {
                type = "stat_mod",
                key = "BaseDamageOutgoingPercentage",
                value = 0.5,
                op = "add",
              },
            },
          },
          {
            name = "巨釘投石機",
            description = "改投巨釘：splash 100, damage 40, 彈速減半",
            cost = 500,
            effects = {
              {
                type = "behavior_flag",
                flag = "spike_o_pult",
              },
            },
          },
        },
        {
          {
            name = "快速射擊",
            description = "攻速 +20%",
            cost = 50,
            effects = {
              {
                type = "stat_mod",
                key = "AttackSpeedMultiplier",
                value = 0.83,
                op = "mul",
              },
            },
          },
          {
            name = "極速射擊",
            description = "攻速再 +30%",
            cost = 100,
            effects = {
              {
                type = "stat_mod",
                key = "AttackSpeedMultiplier",
                value = 0.7,
                op = "mul",
              },
            },
          },
          {
            name = "三重射擊",
            description = "一發變 3 發扇形 ±15°",
            cost = 200,
            effects = {
              {
                type = "behavior_flag",
                flag = "triple_shot",
              },
            },
          },
          {
            name = "超級猴子粉絲俱樂部",
            description = "5 發扇形 + 彈速×2 + 攻速再 +30%",
            cost = 500,
            effects = {
              {
                type = "behavior_flag",
                flag = "fan_club",
              },
              {
                type = "stat_mod",
                key = "AttackSpeedMultiplier",
                value = 0.7,
                op = "mul",
              },
            },
          },
        },
        {
          {
            name = "銳利雙眼",
            description = "爆率 25→40%, 爆傷 30→40",
            cost = 50,
            effects = {
              {
                type = "stat_mod",
                key = "PreattackCriticalStrike",
                value = 0.4,
                op = "add",
              },
              {
                type = "stat_mod",
                key = "CritBonus",
                value = 40.0,
                op = "add",
              },
            },
          },
          {
            name = "弩弓",
            description = "爆率 →50%, 爆傷 →60, 射程 +30",
            cost = 100,
            effects = {
              {
                type = "stat_mod",
                key = "PreattackCriticalStrike",
                value = 0.1,
                op = "add",
              },
              {
                type = "stat_mod",
                key = "CritBonus",
                value = 20.0,
                op = "add",
              },
              {
                type = "stat_mod",
                key = "AttackRangeBonus",
                value = 30.0,
                op = "add",
              },
            },
          },
          {
            name = "神射手",
            description = "必爆 (100%), base dmg +30%",
            cost = 200,
            effects = {
              {
                type = "behavior_flag",
                flag = "always_crit",
              },
              {
                type = "stat_mod",
                key = "BaseDamageOutgoingPercentage",
                value = 0.3,
                op = "add",
              },
            },
          },
          {
            name = "究極重裝彈",
            description = "爆擊 100 dmg + splash 60",
            cost = 500,
            effects = {
              {
                type = "behavior_flag",
                flag = "mega_crit",
              },
            },
          },
        },
      },
    },
    {
      id = "tower_tack",
      display_name = "鐵釘射手",
      atk = 8.0,
      asd_interval = 1.2,
      range = 380.0,
      bullet_speed = 1400.0,
      splash_radius = 0.0,
      hit_radius = 80.0,
      slow_factor = 0.0,
      slow_duration = 0.0,
      cost = 400,
      footprint = 10.0,
      hp = 1.0,
      turn_speed_deg = 3600.0,
      render = {
        render_mode = "base_barrel",
        base = "assets/towers/tower_tack_base.png",
        barrel = "assets/towers/tower_tack_barrel_8.png",
        rotation_mode = "fixed",
        barrel_layout = "radial_count_variants",
        barrel_variants = {
          {
            min_path = 3,
            min_level = 0,
            count = 8,
            image = "assets/towers/tower_tack_barrel_8.png",
            frames = {
              "assets/towers/tower_tack_barrel_8_frame_01.png",
              "assets/towers/tower_tack_barrel_8_frame_02.png",
              "assets/towers/tower_tack_barrel_8_frame_03.png",
            },
          },
          {
            min_path = 3,
            min_level = 2,
            count = 12,
            image = "assets/towers/tower_tack_barrel_12.png",
            frames = {
              "assets/towers/tower_tack_barrel_12_frame_01.png",
              "assets/towers/tower_tack_barrel_12_frame_02.png",
              "assets/towers/tower_tack_barrel_12_frame_03.png",
            },
          },
          {
            min_path = 3,
            min_level = 3,
            count = 16,
            image = "assets/towers/tower_tack_barrel_16.png",
            frames = {
              "assets/towers/tower_tack_barrel_16_frame_01.png",
              "assets/towers/tower_tack_barrel_16_frame_02.png",
              "assets/towers/tower_tack_barrel_16_frame_03.png",
            },
          },
        },
        barrel_animation = { fps = 10.0, loop = true, fire_fps = 22.0, fire_once = true },
        barrel_offset = { x = 0.0, y = -4.0 },
        barrel_pivot = { x = 0.5, y = 0.5 },
        muzzle_offset = { x = 0.0, y = 0.0 },
        default_angle_deg = 0.0,
        recoil = {
          mode = "scale_pulse",
          distance = 0.0,
          scale = 0.9,
          duration_ms = 55,
          return_ms = 90,
        },
      },
      upgrades = {
        {
          {
            name = "更快射擊",
            description = "攻速 +20%",
            cost = 100,
            effects = {
              {
                type = "stat_mod",
                key = "AttackSpeedMultiplier",
                value = 0.83,
                op = "mul",
              },
            },
          },
          {
            name = "長射程鐵釘",
            description = "射程 380→460, damage 8→11",
            cost = 200,
            effects = {
              {
                type = "stat_mod",
                key = "AttackRangeBonus",
                value = 80.0,
                op = "add",
              },
              {
                type = "stat_mod",
                key = "BaseDamageOutgoingPercentage",
                value = 0.375,
                op = "add",
              },
            },
          },
          {
            name = "超遠程鐵釘",
            description = "射程 →530, damage →14",
            cost = 400,
            effects = {
              {
                type = "stat_mod",
                key = "AttackRangeBonus",
                value = 70.0,
                op = "add",
              },
              {
                type = "stat_mod",
                key = "BaseDamageOutgoingPercentage",
                value = 0.375,
                op = "add",
              },
            },
          },
          {
            name = "飛刀射手",
            description = "飛刀: dmg 20, hit_radius 110, 穿透 +2",
            cost = 1000,
            effects = {
              {
                type = "behavior_flag",
                flag = "blade_shooter",
              },
              {
                type = "stat_mod",
                key = "BaseDamageOutgoingPercentage",
                value = 1.5,
                op = "add",
              },
            },
          },
        },
        {
          {
            name = "灼熱射擊",
            description = "命中附 2s 灼燒 5dps",
            cost = 100,
            effects = {
              {
                type = "behavior_flag",
                flag = "burn_tier1",
              },
            },
          },
          {
            name = "易燃材料",
            description = "灼燒 3s × 10dps",
            cost = 200,
            effects = {
              {
                type = "behavior_flag",
                flag = "burn_tier2",
              },
            },
          },
          {
            name = "烈焰火環",
            description = "每次開火塔周 200 半徑 20 dmg",
            cost = 400,
            effects = {
              {
                type = "behavior_flag",
                flag = "ring_of_fire",
              },
            },
          },
          {
            name = "煉獄火環",
            description = "火圈 dmg →50, 針 dmg +10, 火圈附燃燒",
            cost = 1000,
            effects = {
              {
                type = "behavior_flag",
                flag = "inferno_ring",
              },
              {
                type = "stat_mod",
                key = "BaseDamageOutgoingPercentage",
                value = 1.25,
                op = "add",
              },
            },
          },
        },
        {
          {
            name = "更快射擊二型",
            description = "攻速 +30%",
            cost = 100,
            effects = {
              {
                type = "stat_mod",
                key = "AttackSpeedMultiplier",
                value = 0.77,
                op = "mul",
              },
            },
          },
          {
            name = "更多鐵釘",
            description = "針數 8→12",
            cost = 200,
            effects = {
              {
                type = "behavior_flag",
                flag = "needles_12",
              },
            },
          },
          {
            name = "鐵釘噴灑器",
            description = "針數 →16, 射程 +50",
            cost = 400,
            effects = {
              {
                type = "behavior_flag",
                flag = "needles_16",
              },
              {
                type = "stat_mod",
                key = "AttackRangeBonus",
                value = 50.0,
                op = "add",
              },
            },
          },
          {
            name = "鐵釘禁區",
            description = "針數 →32, 攻速再 +40%",
            cost = 1000,
            effects = {
              {
                type = "behavior_flag",
                flag = "needles_32",
              },
              {
                type = "stat_mod",
                key = "AttackSpeedMultiplier",
                value = 0.7,
                op = "mul",
              },
            },
          },
        },
      },
    },
    {
      id = "tower_bomb",
      display_name = "炸彈射手",
      atk = 30.0,
      asd_interval = 1.5,
      range = 400.0,
      bullet_speed = 900.0,
      splash_radius = 200.0,
      hit_radius = 0.0,
      slow_factor = 0.0,
      slow_duration = 0.0,
      cost = 650,
      footprint = 12.5,
      hp = 1.0,
      turn_speed_deg = 360.0,
      render = {
        render_mode = "base_barrel",
        base = "assets/towers/tower_bomb_base.png",
        barrel = "assets/towers/tower_bomb_barrel.png",
        barrel_frames = barrel_frames("tower_bomb"),
        barrel_animation = { fps = 9.0, loop = true, fire_fps = 18.0, fire_once = true },
        rotation_mode = "targeted",
        barrel_layout = "single",
        barrel_offset = { x = 0.0, y = -7.0 },
        barrel_pivot = { x = 0.5, y = 0.7 },
        muzzle_offset = { x = 0.0, y = -34.0 },
        recoil = {
          mode = "directional",
          distance = 12.0,
          scale = 0.92,
          duration_ms = 80,
          return_ms = 125,
        },
      },
      upgrades = {
        {
          {
            name = "額外射程",
            description = "射程 400→475",
            cost = 162,
            effects = {
              {
                type = "stat_mod",
                key = "AttackRangeBonus",
                value = 75.0,
                op = "add",
              },
            },
          },
          {
            name = "更大炸彈",
            description = "splash 200→250, damage 30→40",
            cost = 325,
            effects = {
              {
                type = "stat_mod",
                key = "SplashBonus",
                value = 50.0,
                op = "add",
              },
              {
                type = "stat_mod",
                key = "BaseDamageOutgoingPercentage",
                value = 0.33,
                op = "add",
              },
            },
          },
          {
            name = "超大炸彈",
            description = "splash →300, damage →60",
            cost = 650,
            effects = {
              {
                type = "stat_mod",
                key = "SplashBonus",
                value = 50.0,
                op = "add",
              },
              {
                type = "stat_mod",
                key = "BaseDamageOutgoingPercentage",
                value = 0.67,
                op = "add",
              },
            },
          },
          {
            name = "氣球衝擊",
            description = "splash →400, damage →100, 命中 0.5s 眩暈",
            cost = 1625,
            effects = {
              {
                type = "behavior_flag",
                flag = "bomb_stun",
              },
              {
                type = "stat_mod",
                key = "SplashBonus",
                value = 100.0,
                op = "add",
              },
              {
                type = "stat_mod",
                key = "BaseDamageOutgoingPercentage",
                value = 1.33,
                op = "add",
              },
            },
          },
        },
        {
          {
            name = "更快裝填",
            description = "攻速 +20%",
            cost = 162,
            effects = {
              {
                type = "stat_mod",
                key = "AttackSpeedMultiplier",
                value = 0.83,
                op = "mul",
              },
            },
          },
          {
            name = "飛彈發射器",
            description = "射程 +150, 彈速 900→1350",
            cost = 325,
            effects = {
              {
                type = "stat_mod",
                key = "AttackRangeBonus",
                value = 150.0,
                op = "add",
              },
              {
                type = "behavior_flag",
                flag = "missile",
              },
            },
          },
          {
            name = "飛艇粉碎者",
            description = "damage +30, 彈速再 +50%",
            cost = 650,
            effects = {
              {
                type = "stat_mod",
                key = "BaseDamageOutgoingPercentage",
                value = 1.0,
                op = "add",
              },
            },
          },
          {
            name = "飛艇刺客",
            description = "每 15s 超級彈 + 常攻再 +30% 攻速",
            cost = 1625,
            effects = {
              {
                type = "behavior_flag",
                flag = "moab_assassin",
              },
              {
                type = "stat_mod",
                key = "AttackSpeedMultiplier",
                value = 0.7,
                op = "mul",
              },
            },
          },
        },
        {
          {
            name = "破片炸彈",
            description = "爆炸後 8 方向碎片 15 dmg",
            cost = 162,
            effects = {
              {
                type = "behavior_flag",
                flag = "frag_8",
              },
            },
          },
          {
            name = "集束炸彈",
            description = "碎片 →12, dmg 25",
            cost = 325,
            effects = {
              {
                type = "behavior_flag",
                flag = "frag_12",
              },
            },
          },
          {
            name = "遞迴集束",
            description = "碎片 dmg →45, 再生 4 個小碎片",
            cost = 650,
            effects = {
              {
                type = "behavior_flag",
                flag = "frag_recursive",
              },
            },
          },
          {
            name = "炸彈閃擊",
            description = "碎片 →16 homing, 主彈 dmg +50",
            cost = 1625,
            effects = {
              {
                type = "behavior_flag",
                flag = "frag_homing",
              },
              {
                type = "stat_mod",
                key = "BaseDamageOutgoingPercentage",
                value = 1.67,
                op = "add",
              },
            },
          },
        },
      },
    },
    {
      id = "tower_ice",
      display_name = "冰凍猴",
      atk = 3.0,
      asd_interval = 1.5,
      range = 180.0,
      bullet_speed = 600.0,
      splash_radius = 90.0,
      hit_radius = 0.0,
      slow_factor = 0.5,
      slow_duration = 2.0,
      cost = 400,
      footprint = 10.0,
      hp = 1.0,
      turn_speed_deg = 360.0,
      render = {
        render_mode = "base_barrel",
        base = "assets/towers/tower_ice_base.png",
        barrel = "assets/towers/tower_ice_barrel.png",
        barrel_frames = barrel_frames("tower_ice"),
        barrel_animation = { fps = 10.0, loop = true, fire_fps = 18.0, fire_once = true },
        rotation_mode = "targeted",
        barrel_layout = "single",
        barrel_offset = { x = 0.0, y = -6.0 },
        barrel_pivot = { x = 0.5, y = 0.66 },
        muzzle_offset = { x = 0.0, y = -30.0 },
        recoil = {
          mode = "directional",
          distance = 5.0,
          scale = 0.96,
          duration_ms = 65,
          return_ms = 105,
        },
      },
      upgrades = {
        {
          {
            name = "永久凍霜",
            description = "slow 50%→65%",
            cost = 100,
            effects = {
              {
                type = "stat_mod",
                key = "SlowFactorOverride",
                value = 0.35,
                op = "add",
              },
            },
          },
          {
            name = "強化冰凍",
            description = "slow 持續 2.0→3.0s",
            cost = 200,
            effects = {
              {
                type = "stat_mod",
                key = "SlowDurationBonus",
                value = 1.0,
                op = "add",
              },
            },
          },
          {
            name = "深度凍結",
            description = "命中附 1.0s 完全凍結",
            cost = 400,
            effects = {
              {
                type = "behavior_flag",
                flag = "deep_freeze",
              },
            },
          },
          {
            name = "絕對零度",
            description = "每 15s 全屏凍結 2s, 常規 slow →80%",
            cost = 1000,
            effects = {
              {
                type = "behavior_flag",
                flag = "absolute_zero",
              },
              {
                type = "stat_mod",
                key = "SlowFactorOverride",
                value = -0.15,
                op = "add",
              },
            },
          },
        },
        {
          {
            name = "更大範圍",
            description = "range 180→250, splash 90→120",
            cost = 100,
            effects = {
              {
                type = "stat_mod",
                key = "AttackRangeBonus",
                value = 70.0,
                op = "add",
              },
              {
                type = "stat_mod",
                key = "SplashBonus",
                value = 30.0,
                op = "add",
              },
            },
          },
          {
            name = "極地寒風",
            description = "range →300, 塔周光環減速 20%",
            cost = 200,
            effects = {
              {
                type = "stat_mod",
                key = "AttackRangeBonus",
                value = 50.0,
                op = "add",
              },
              {
                type = "behavior_flag",
                flag = "arctic_aura_20",
              },
            },
          },
          {
            name = "暴風雪",
            description = "光環疊到 35%, 凍敵所有塔攻速 +10%",
            cost = 400,
            effects = {
              {
                type = "behavior_flag",
                flag = "snowstorm",
              },
            },
          },
          {
            name = "低溫冰砲",
            description = "range →400, 光環 40%, 每 10s 射巨冰彈",
            cost = 1000,
            effects = {
              {
                type = "behavior_flag",
                flag = "cryo_cannon",
              },
              {
                type = "stat_mod",
                key = "AttackRangeBonus",
                value = 100.0,
                op = "add",
              },
            },
          },
        },
        {
          {
            name = "強化凍傷",
            description = "本塔減速敵人受物理 +15%",
            cost = 100,
            effects = {
              {
                type = "behavior_flag",
                flag = "embrittle_15",
              },
            },
          },
          {
            name = "再次凍結",
            description = "攻擊刷新 slow 到滿 duration",
            cost = 200,
            effects = {
              {
                type = "behavior_flag",
                flag = "refreeze",
              },
            },
          },
          {
            name = "脆化",
            description = "減速中敵人受全來源 +25% 傷害",
            cost = 400,
            effects = {
              {
                type = "behavior_flag",
                flag = "embrittle_25",
              },
            },
          },
          {
            name = "冰錐穿刺",
            description = "冰錐穿透 3, base dmg 3→25, splash 150",
            cost = 1000,
            effects = {
              {
                type = "behavior_flag",
                flag = "icicle_impale",
              },
              {
                type = "stat_mod",
                key = "BaseDamageOutgoingPercentage",
                value = 7.33,
                op = "add",
              },
            },
          },
        },
      },
    },
    {
      id = "tower_cake_splash",
      display_name = "蛋糕濺射塔",
      atk = 18.0,
      asd_interval = 1.4,
      range = 240.0,
      bullet_speed = 0.0,
      splash_radius = 160.0,
      hit_radius = 0.0,
      slow_factor = 0.0,
      slow_duration = 0.0,
      cost = 500,
      footprint = 12.5,
      hp = 1.0,
      turn_speed_deg = 0.0,
      render = {
        render_mode = "animated_area",
        base = "assets/towers/tower_cake_splash_frame_01.png",
        animation = {
          frames = cake_splash_frames(),
          fps = 10.0,
          loop = true,
          fire_fps = 18.0,
          fire_once = true,
        },
        recoil = {
          mode = "scale_pulse",
          distance = 0.0,
          scale = 0.88,
          duration_ms = 70,
          return_ms = 110,
        },
      },
    },
  }

  for _, tower in ipairs(towers) do
    apply_render_defaults(tower)
  end

  return towers
end
