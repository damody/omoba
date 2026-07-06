return function(ctx)
  return {
    GameMode = "TowerDefense",
    Path = {
      {
        Name = "td_ice_top",
        Points = {
          "td_spawn_top",
          "td_top1",
          "td_top2",
          "td_top3",
          "td_exit_top",
        },
      },
      {
        Name = "td_ice_mid",
        Points = {
          "td_spawn_mid",
          "td_mid1",
          "td_mid2",
          "td_mid3",
          "td_exit_mid",
        },
      },
      {
        Name = "td_ice_bot",
        Points = {
          "td_spawn_bot",
          "td_bot1",
          "td_bot2",
          "td_bot3",
          "td_exit_bot",
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
        X = -700.0,
        Y = -700.0,
      },
      {
        Name = "td_top2",
        Class = "Path",
        X = -200.0,
        Y = -250.0,
      },
      {
        Name = "td_top3",
        Class = "Path",
        X = 400.0,
        Y = -250.0,
      },
      {
        Name = "td_exit_top",
        Class = "Base",
        X = 1400.0,
        Y = -650.0,
      },
      {
        Name = "td_spawn_mid",
        Class = "Spawn",
        X = -1400.0,
        Y = 0.0,
      },
      {
        Name = "td_mid1",
        Class = "Path",
        X = -500.0,
        Y = 0.0,
      },
      {
        Name = "td_mid2",
        Class = "Path",
        X = -100.0,
        Y = 350.0,
      },
      {
        Name = "td_mid3",
        Class = "Path",
        X = 700.0,
        Y = 350.0,
      },
      {
        Name = "td_exit_mid",
        Class = "Base",
        X = 1400.0,
        Y = 0.0,
      },
      {
        Name = "td_spawn_bot",
        Class = "Spawn",
        X = -1400.0,
        Y = 700.0,
      },
      {
        Name = "td_bot1",
        Class = "Path",
        X = -750.0,
        Y = 700.0,
      },
      {
        Name = "td_bot2",
        Class = "Path",
        X = -300.0,
        Y = 450.0,
      },
      {
        Name = "td_bot3",
        Class = "Path",
        X = 450.0,
        Y = 650.0,
      },
      {
        Name = "td_exit_bot",
        Class = "Base",
        X = 1400.0,
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
            Path = "td_ice_top",
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
            Path = "td_ice_mid",
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
            Path = "td_ice_bot",
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
            Path = "td_ice_top",
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
            Path = "td_ice_mid",
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
