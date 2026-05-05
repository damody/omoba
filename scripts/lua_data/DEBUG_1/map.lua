return function(ctx)
  return {
    Path = {
      {
        Name = "debug_bounce",
        Points = {
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
          "right",
          "left",
        },
      },
    },
    Creep = {
      {
        Name = "debug_dummy",
      },
    },
    CheckPoint = {
      {
        Name = "player_base",
        Class = "Base",
        X = 0,
        Y = 0,
      },
      {
        Name = "right",
        Class = "Path",
        X = 700,
        Y = 0,
      },
      {
        Name = "left",
        Class = "Path",
        X = 300,
        Y = 0,
      },
    },
    Tower = {},
    CreepWave = {
      {
        Name = "W1",
        StartTime = 2,
        Detail = {
          {
            Path = "debug_bounce",
            Creeps = {
              {
                Time = 0,
                Creep = "debug_dummy",
              },
            },
          },
        },
      },
    },
  }
end
