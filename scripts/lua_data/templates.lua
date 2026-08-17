-- 範本 ID 是僅附加的：變更聲明順序會改變產生的 ID。
-- ID 0 保留為 UNSPECIFIED；活動 ID 是根據聲明順序分配的。
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
    td_layers = ctx.include("templates/td_layers.lua"),
  }
end
