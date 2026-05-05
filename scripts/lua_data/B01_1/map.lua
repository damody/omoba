return function(ctx)
  return {
    Path = {
      {
        Name = "training_path",
        Points = {
          "square_nw",
          "square_ne",
          "square_se",
          "square_sw",
          "square_nw",
        },
      },
      {
        Name = "circle_path",
        Points = {
          "target_1000",
          "circle_1",
          "circle_2",
          "circle_3",
          "circle_4",
          "target_1000",
        },
      },
    },
    Creep = {
      {
        Name = "practice_dummy",
      },
      {
        Name = "moving_target",
      },
      {
        Name = "training_creep",
      },
      {
        Name = "armored_dummy",
      },
    },
    CheckPoint = {
      {
        Name = "player_start",
        Class = "Start",
        X = 0,
        Y = 0,
      },
      {
        Name = "creep_spawn",
        Class = "CreepSpawn",
        X = 200,
        Y = 100,
      },
      {
        Name = "target_800",
        Class = "Target",
        X = 800,
        Y = 0,
      },
      {
        Name = "target_1000",
        Class = "Target",
        X = 1000,
        Y = 0,
      },
      {
        Name = "target_1200",
        Class = "Target",
        X = 1200,
        Y = 0,
      },
      {
        Name = "target_1600",
        Class = "Target",
        X = 1600,
        Y = 0,
      },
      {
        Name = "circle_1",
        Class = "MovePath",
        X = 1100,
        Y = 100,
      },
      {
        Name = "circle_2",
        Class = "MovePath",
        X = 1100,
        Y = -100,
      },
      {
        Name = "circle_3",
        Class = "MovePath",
        X = 900,
        Y = -100,
      },
      {
        Name = "circle_4",
        Class = "MovePath",
        X = 900,
        Y = 100,
      },
      {
        Name = "square_nw",
        Class = "MovePath",
        X = 400,
        Y = 400,
      },
      {
        Name = "square_ne",
        Class = "MovePath",
        X = 1200,
        Y = 400,
      },
      {
        Name = "square_se",
        Class = "MovePath",
        X = 1200,
        Y = -400,
      },
      {
        Name = "square_sw",
        Class = "MovePath",
        X = 400,
        Y = -400,
      },
      {
        Name = "boss_area",
        Class = "BossArea",
        X = 1400,
        Y = 200,
      },
      {
        Name = "safe_zone",
        Class = "SafeZone",
        X = -100,
        Y = 0,
      },
      {
        Name = "item_shop",
        Class = "Shop",
        X = -50,
        Y = 50,
      },
    },
    Tower = {
      {
        Name = "target_tower",
        Property = {
          Hp = 500,
          Block = 0,
        },
        Attack = {
          Range = 0,
          AttackSpeed = 0,
          Physic = 0,
          Magic = 0,
          cost = 0,
        },
      },
      {
        Name = "practice_tower",
        Property = {
          Hp = 800,
          Block = 0,
        },
        Attack = {
          Range = 0,
          AttackSpeed = 0,
          Physic = 0,
          Magic = 0,
          cost = 0,
        },
      },
    },
    CreepWave = {
      {
        Name = "S0_LastHit_Wave",
        StartTime = 3,
        Detail = {
          {
            Path = "training_path",
            Creeps = {
              {
                Time = 0,
                Creep = "armored_dummy",
              },
              {
                Time = 10,
                Creep = "training_creep",
              },
              {
                Time = 20,
                Creep = "armored_dummy",
              },
              {
                Time = 30,
                Creep = "training_creep",
              },
            },
          },
        },
      },
      {
        Name = "S1b_MovingTarget_Wave",
        StartTime = 200,
        Detail = {
          {
            Path = "circle_path",
            Creeps = {
              {
                Time = 0,
                Creep = "moving_target",
              },
              {
                Time = 8,
                Creep = "moving_target",
              },
              {
                Time = 16,
                Creep = "moving_target",
              },
            },
          },
        },
      },
      {
        Name = "S6_ArmorTest_Wave",
        StartTime = 500,
        Detail = {
          {
            Path = "training_path",
            Creeps = {
              {
                Time = 0,
                Creep = "armored_dummy",
              },
              {
                Time = 10,
                Creep = "armored_dummy",
              },
            },
          },
        },
      },
    },
  }
end
