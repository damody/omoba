return function(ctx)
  return {
    GameMode = "TowerDefense",
    Path = {
      {
        Name = "td_main",
        Points = {
          "td_spawn",
          "td_cp1",
          "td_cp2",
          "td_cp3",
          "td_cp4",
          "td_cp5",
          "td_cp6",
          "td_cp7",
          "td_cp8",
          "td_cp9",
          "td_cp10",
          "td_cp11",
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
        Name = "td_spawn",
        Class = "Spawn",
        X = -1400.0,
        Y = -750.0,
      },
      {
        Name = "td_cp1",
        Class = "Path",
        X = -250.0,
        Y = -750.0,
      },
      {
        Name = "td_cp2",
        Class = "Path",
        X = -250.0,
        Y = -350.0,
      },
      {
        Name = "td_cp3",
        Class = "Path",
        X = -1000.0,
        Y = -350.0,
      },
      {
        Name = "td_cp4",
        Class = "Path",
        X = -1000.0,
        Y = 150.0,
      },
      {
        Name = "td_cp5",
        Class = "Path",
        X = -100.0,
        Y = 150.0,
      },
      {
        Name = "td_cp6",
        Class = "Path",
        X = -100.0,
        Y = 650.0,
      },
      {
        Name = "td_cp7",
        Class = "Path",
        X = 900.0,
        Y = 650.0,
      },
      {
        Name = "td_cp8",
        Class = "Path",
        X = 900.0,
        Y = 150.0,
      },
      {
        Name = "td_cp9",
        Class = "Path",
        X = 250.0,
        Y = 150.0,
      },
      {
        Name = "td_cp10",
        Class = "Path",
        X = 250.0,
        Y = -350.0,
      },
      {
        Name = "td_cp11",
        Class = "Path",
        X = 1400.0,
        Y = -350.0,
      },
      {
        Name = "td_exit",
        Class = "Base",
        X = 1400.0,
        Y = 750.0,
      },
    },
    Tower = {},
    CreepWave = {
      {
        Name = "W01",
        StartTime = 0.0,
        Detail = {
          {
            Path = "td_main",
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
            Path = "td_main",
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
            Path = "td_main",
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
            Path = "td_main",
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
            Path = "td_main",
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
