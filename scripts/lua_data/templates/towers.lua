return function(ctx)
  return {
    {
      id = "tower_dart",
      display_name = "Dart Monkey",
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
      upgrades = {
        {
          {
            name = "Long Range Darts",
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
            name = "Enhanced Eyesight",
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
            name = "Razor Sharp Shots",
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
            name = "Spike-o-pult",
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
            name = "Quick Shots",
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
            name = "Very Quick Shots",
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
            name = "Triple Shot",
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
            name = "Super Monkey Fan Club",
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
            name = "Keen Eyes",
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
            name = "Crossbow",
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
            name = "Sharp Shooter",
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
            name = "Ultra-Juggernaut",
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
      display_name = "Tack Shooter",
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
      upgrades = {
        {
          {
            name = "Faster Shooting",
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
            name = "Long Range Tacks",
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
            name = "Super Range Tacks",
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
            name = "Blade Shooter",
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
            name = "Hot Shots",
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
            name = "Burny Stuff",
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
            name = "Ring of Fire",
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
            name = "Inferno Ring",
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
            name = "Faster Shooting II",
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
            name = "Even More Tacks",
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
            name = "Tack Sprayer",
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
            name = "The Tack Zone",
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
      display_name = "Bomb Shooter",
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
      upgrades = {
        {
          {
            name = "Extra Range",
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
            name = "Bigger Bombs",
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
            name = "Really Big Bombs",
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
            name = "Bloon Impact",
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
            name = "Faster Reload",
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
            name = "Missile Launcher",
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
            name = "MOAB Mauler",
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
            name = "MOAB Assassin",
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
            name = "Frag Bombs",
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
            name = "Cluster Bombs",
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
            name = "Recursive Cluster",
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
            name = "Bomb Blitz",
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
      display_name = "Ice Monkey",
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
      upgrades = {
        {
          {
            name = "Permafrost",
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
            name = "Enhanced Freeze",
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
            name = "Deep Freeze",
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
            name = "Absolute Zero",
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
            name = "Larger Range",
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
            name = "Arctic Wind",
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
            name = "Snowstorm",
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
            name = "Cryo Cannon",
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
            name = "Enhanced Freeze",
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
            name = "Re-Freeze",
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
            name = "Embrittlement",
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
            name = "Icicle Impale",
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
  }
end
