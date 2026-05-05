-- Template ids are append-only: changing declaration order shifts generated ids.
-- Id 0 is reserved as UNSPECIFIED; active ids are assigned from declaration order.
return function(ctx)
  local note = ctx.read_text("templates/catalog_note.txt")
  local meta = ctx.read_toml("templates/catalog_meta.toml")

  if pcall(function() ctx.read_text("../secret.txt") end) then
    error("ctx.read_text accepted parent-directory escape")
  end
  if pcall(function() ctx.read_toml("C:/secret.toml") end) then
    error("ctx.read_toml accepted absolute path")
  end
  if pcall(function() ctx.include("templates/include_cycle_a.lua") end) then
    error("ctx.include accepted an include cycle")
  end

  return {
    _meta = {
      note = note,
      version = meta.catalog.version,
    },
    towers = ctx.include("templates/towers.lua"),
    heroes = ctx.include("templates/heroes.lua"),
    abilities = ctx.include("templates/abilities.lua"),
    buffs = ctx.include("templates/buffs.lua"),
    summons = ctx.include("templates/summons.lua"),
    creeps = ctx.include("templates/creeps.lua"),
    projectile_kinds = ctx.include("templates/projectile_kinds.lua"),
  }
end
