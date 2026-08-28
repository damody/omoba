return function(ctx)
  local visionTrees = {}
  for row = 0, 7 do
    for column = 0, 7 do
      local id = row * 8 + column + 1
      visionTrees[#visionTrees + 1] = {
        StableId = id,
        X = -875.0 + column * 250.0,
        Y = -875.0 + row * 250.0,
        Radius = (id % 3 == 0) and 82.0 or 62.0,
      }
    end
  end
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
    VisionTrees = visionTrees,
    VisionOccluderPolygons = {
      {
        StableId = 1001,
        Name = "central_rock",
        Points = {
          { X = -170.0, Y = -150.0 },
          { X = 190.0, Y = -150.0 },
          { X = 190.0, Y = 130.0 },
          { X = -170.0, Y = 130.0 },
        },
      },
      {
        StableId = 1002,
        Name = "west_crescent",
        Points = {
          { X = -980.0, Y = 180.0 },
          { X = -560.0, Y = 180.0 },
          { X = -560.0, Y = 520.0 },
          { X = -720.0, Y = 520.0 },
          { X = -720.0, Y = 340.0 },
          { X = -980.0, Y = 340.0 },
        },
      },
      {
        StableId = 1003,
        Name = "east_hook",
        Points = {
          { X = 520.0, Y = -560.0 },
          { X = 980.0, Y = -560.0 },
          { X = 980.0, Y = -360.0 },
          { X = 700.0, Y = -360.0 },
          { X = 700.0, Y = -160.0 },
          { X = 520.0, Y = -160.0 },
        },
      },
    },
    FogDemo = {
      Rows = 10,
      Columns = 10,
      Spacing = 220.0,
      OriginX = -990.0,
      OriginY = -990.0,
      VisionRadius = 700.0,
      -- 預設離開視野後Forget；指定索引保留sanitized LastKnown，
      -- 讓同一場景可同時驗收兩種transition。
      RememberPolicy = "Forget",
      LastKnownIndexes = { 14, 34, 54, 74 },
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
