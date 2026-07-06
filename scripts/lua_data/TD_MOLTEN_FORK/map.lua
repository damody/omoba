return function(ctx)
  return {
    GameMode = "TowerDefense",
    Path = {
      {
        Name = "td_lava_top",
        Points = {
          "td_spawn_top",
          "td_top1",
          "td_top2",
          "td_merge",
          "td_exit",
        },
      },
      {
        Name = "td_lava_mid",
        Points = {
          "td_spawn_mid",
          "td_mid1",
          "td_mid2",
          "td_merge",
          "td_exit",
        },
      },
      {
        Name = "td_lava_bot",
        Points = {
          "td_spawn_bot",
          "td_bot1",
          "td_bot2",
          "td_merge",
          "td_exit",
        },
      },
    },
    Creep = {
      {
        Name = "td_basic",
      },
      {
        Name = "td_tough",
      },
    },
    CheckPoint = {
      {
        Name = "td_spawn_top",
        Class = "Spawn",
        X = -1400.0,
        Y = -700.0,
      },
      {
        Name = "td_top1",
        Class = "Path",
        X = -600.0,
        Y = -700.0,
      },
      {
        Name = "td_top2",
        Class = "Path",
        X = -100.0,
        Y = -250.0,
      },
      {
        Name = "td_spawn_mid",
        Class = "Spawn",
        X = -1400.0,
        Y = -100.0,
      },
      {
        Name = "td_mid1",
        Class = "Path",
        X = -500.0,
        Y = -100.0,
      },
      {
        Name = "td_mid2",
        Class = "Path",
        X = 0.0,
        Y = 250.0,
      },
      {
        Name = "td_spawn_bot",
        Class = "Spawn",
        X = -1400.0,
        Y = 400.0,
      },
      {
        Name = "td_bot1",
        Class = "Path",
        X = -300.0,
        Y = 400.0,
      },
      {
        Name = "td_bot2",
        Class = "Path",
        X = 300.0,
        Y = 0.0,
      },
      {
        Name = "td_merge",
        Class = "Path",
        X = 1400.0,
        Y = 100.0,
      },
      {
        Name = "td_exit",
        Class = "Base",
        X = 700.0,
        Y = 700.0,
      },
    },
    Tower = {},
    CreepWave = {
      {
        Name = "W01",
        StartTime = 0.0,
        Detail = {
          {
            Path = "td_lava_top",
            Creeps = {
              {
                Time = 0.0,
                Creep = "td_basic",
              },
              {
                Time = 1.2,
                Creep = "td_basic",
              },
              {
                Time = 2.4,
                Creep = "td_basic",
              },
              {
                Time = 3.6,
                Creep = "td_tough",
              },
              {
                Time = 4.8,
                Creep = "td_basic",
              },
              {
                Time = 6.0,
                Creep = "td_tough",
              },
            },
          },
        },
      },
      {
        Name = "W02",
        StartTime = 0.0,
        Detail = {
          {
            Path = "td_lava_mid",
            Creeps = {
              {
                Time = 0.0,
                Creep = "td_basic",
              },
              {
                Time = 1.2,
                Creep = "td_basic",
              },
              {
                Time = 2.4,
                Creep = "td_basic",
              },
              {
                Time = 3.6,
                Creep = "td_tough",
              },
              {
                Time = 4.8,
                Creep = "td_basic",
              },
              {
                Time = 6.0,
                Creep = "td_tough",
              },
            },
          },
        },
      },
      {
        Name = "W03",
        StartTime = 0.0,
        Detail = {
          {
            Path = "td_lava_bot",
            Creeps = {
              {
                Time = 0.0,
                Creep = "td_basic",
              },
              {
                Time = 1.2,
                Creep = "td_basic",
              },
              {
                Time = 2.4,
                Creep = "td_basic",
              },
              {
                Time = 3.6,
                Creep = "td_tough",
              },
              {
                Time = 4.8,
                Creep = "td_basic",
              },
              {
                Time = 6.0,
                Creep = "td_tough",
              },
            },
          },
        },
      },
      {
        Name = "W04",
        StartTime = 0.0,
        Detail = {
          {
            Path = "td_lava_top",
            Creeps = {
              {
                Time = 0.0,
                Creep = "td_basic",
              },
              {
                Time = 1.2,
                Creep = "td_basic",
              },
              {
                Time = 2.4,
                Creep = "td_basic",
              },
              {
                Time = 3.6,
                Creep = "td_tough",
              },
              {
                Time = 4.8,
                Creep = "td_basic",
              },
              {
                Time = 6.0,
                Creep = "td_tough",
              },
            },
          },
        },
      },
      {
        Name = "W05",
        StartTime = 0.0,
        Detail = {
          {
            Path = "td_lava_mid",
            Creeps = {
              {
                Time = 0.0,
                Creep = "td_basic",
              },
              {
                Time = 1.2,
                Creep = "td_basic",
              },
              {
                Time = 2.4,
                Creep = "td_basic",
              },
              {
                Time = 3.6,
                Creep = "td_tough",
              },
              {
                Time = 4.8,
                Creep = "td_basic",
              },
              {
                Time = 6.0,
                Creep = "td_tough",
              },
            },
          },
        },
      },
    },
    Structures = {},
    BlockedRegions = {},
  }
end
