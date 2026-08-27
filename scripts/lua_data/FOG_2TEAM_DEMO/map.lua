return function(ctx)
  return {
    GameMode = "Moba",
    Path = ctx.array({}),
    Creep = ctx.array({}),
    CheckPoint = ctx.array({}),
    Tower = ctx.array({}),
    CreepWave = ctx.array({}),
    InitialCreeps = ctx.array({}),
    Structures = ctx.array({}),
    BlockedRegions = ctx.array({}),
    FogDemo = {
      Rows = 10,
      Columns = 10,
      Spacing = 220.0,
      OriginX = -990.0,
      OriginY = -990.0,
      VisionRadius = 700.0,
      -- 此 demo 用來直接驗收兩隊的即時圓形視野；離開視野後完全移除，
      -- 不保留 LastKnown 灰色殘影，避免被誤認為仍同步中的單位。
      RememberPolicy = "Forget",
      GridUnitTemplate = "practice_dummy",
      HeroTemplate = "saika_magoichi",
      HeroSpawns = {
        { PlayerId = 1, TeamId = 1, X = -1320.0, Y = -1100.0 },
        { PlayerId = 2, TeamId = 2, X = 1320.0, Y = 1100.0 },
      },
      PatrolIndexes = { 4, 9, 14, 19, 24, 29, 34, 39, 44, 54, 59, 64, 69, 74, 84, 94 },
      PatrolOffset = 330.0,
      PatrolSpeed = 180.0,
    },
  }
end
