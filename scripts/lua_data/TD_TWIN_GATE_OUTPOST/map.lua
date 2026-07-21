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
          "td_base",
        },
      },
      {
        Name = "td_main_b",
        Points = {
          "td_spawn_b",
          "td_b1",
          "td_merge",
          "td_base",
        },
      },
      {
        Name = "td_main_c",
        Points = {
          "td_spawn_c",
          "td_c1",
          "td_merge",
          "td_base",
        },
      },
    },
    SelectSpawnPath = function(_, balloon_index, _)
      return ((balloon_index - 1) % 3) + 1
    end,
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
        Y = 700.0,
      },
      {
        Name = "td_a1",
        Class = "Path",
        X = -350.0,
        Y = 520.0,
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
        Name = "td_spawn_c",
        Class = "Spawn",
        X = -1400.0,
        Y = -650.0,
      },
      {
        Name = "td_c1",
        Class = "Path",
        X = -500.0,
        Y = -650.0,
      },
      {
        Name = "td_merge",
        Class = "Path",
        X = 850.0,
        Y = -200.0,
      },
      {
        Name = "td_base",
        Class = "Base",
        X = 1400.0,
        Y = 100.0,
      },
    },
    Tower = {},
    CreepWave = {},
    Structures = {},
    BlockedRegions = {},
  }
end
