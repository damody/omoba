return function(ctx)
  return {
    heroes = {
      { id = "saika_magoichi" },
    },
    enemies = ctx.array({}),
    creeps = {
      { id = "practice_dummy" },
    },
    neutrals = ctx.array({}),
    summons = ctx.array({}),
  }
end
