return function(ctx)
  return {
    GameMode = "TowerDefense",
    Path = {
      {
        Name = "td_main_a",
        Points = {
          "td_spawn_a",
          "td_a1",
          "td_merge",
          "td_cp1",
          "td_cp2",
          "td_exit",
        },
      },
      {
        Name = "td_main_b",
        Points = {
          "td_spawn_b",
          "td_b1",
          "td_merge",
          "td_cp1",
          "td_cp2",
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
        Name = "td_spawn_a",
        Class = "Spawn",
        X = -1400.0,
        Y = -650.0,
      },
      {
        Name = "td_a1",
        Class = "Path",
        X = -500.0,
        Y = -650.0,
      },
      {
        Name = "td_spawn_b",
        Class = "Spawn",
        X = -1400.0,
        Y = 100.0,
      },
      {
        Name = "td_b1",
        Class = "Path",
        X = -500.0,
        Y = 100.0,
      },
      {
        Name = "td_merge",
        Class = "Path",
        X = 0.0,
        Y = -200.0,
      },
      {
        Name = "td_cp1",
        Class = "Path",
        X = 900.0,
        Y = -200.0,
      },
      {
        Name = "td_cp2",
        Class = "Path",
        X = 1400.0,
        Y = 250.0,
      },
      {
        Name = "td_exit",
        Class = "Base",
        X = -1400.0,
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
            Path = "td_main_a",
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
            Path = "td_main_b",
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
            Path = "td_main_a",
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
            Path = "td_main_b",
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
            Path = "td_main_a",
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
