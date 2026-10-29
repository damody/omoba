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
    world_units = ctx.include("templates/world_units.lua"),
    -- Playable heroes take their unit stats from the source roster (templates/hero_source_stats.lua).
    heroes = ctx.include("templates/hero_source_stats.lua").apply(
      ctx.include("templates/heroes.lua"), ctx.include("templates/hero_sources.lua")),
    abilities = ctx.include("templates/abilities.lua"),
    -- Hand-written buffs, then one per visible v3 status (E218).
    buffs = ctx.include("templates/status_buffs.lua").apply(ctx.include("templates/buffs.lua"),
      ctx.include("templates/authored_heroes.lua").abilities, ctx.include("templates/item_texts_v3.lua")),
    summons = ctx.include("templates/summons.lua"),
    creeps = ctx.include("templates/creeps.lua"),
    projectile_kinds = ctx.include("templates/projectile_kinds.lua"),
    td_layers = ctx.include("templates/td_layers.lua"),
    moba_items = ctx.include("templates/moba_items.lua"),
    moba_summoners = ctx.include("templates/moba_summoners.lua"),
    moba_trinkets = ctx.include("templates/moba_trinkets.lua"),
    moba_communication = ctx.include("templates/moba_communication.lua"),
    moba_economy = ctx.include("templates/moba_economy.lua"),
    moba_maps = ctx.include("templates/moba_maps.lua"),
    -- The original nobu.vmap layout (generated, E187): the nobu map in moba_maps.lua reads it; its
    -- camps and trees wait for nobu-map-comparison.md §5.4 / §5.5.
    nobu_map = ctx.include("templates/nobu_map.lua"),
    -- What one point of strength, agility and intelligence is worth (Dota 2 rules).
    attribute_rules = ctx.include("templates/attribute_rules.lua"),
    -- Game mode rules of the original game: gold, experience, respawn, acquisition (history_nobu).
    game_rules = ctx.include("templates/game_rules.lua"),
    -- Source roster every playable hero's source_ref is checked against (generated).
    hero_sources = ctx.include("templates/hero_sources.lua"),
  }
end
