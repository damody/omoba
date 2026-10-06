-- Build-time equipment source. Optional active = { kind = "shield", amount = 100,
-- duration = 3 }, cooldown = 10 compiles to native Rust; no runtime Lua.
-- Supported kinds: shield, sprint_buff, restore_mana, damage_reduce, headshot_next.
-- Mana passive bonuses remain unsupported. Active charm values are the initial
-- training balance; existing passive IDs/prices/recipes remain unchanged.
return function(ctx)
  return {
    { catalog_id = 1, id = "moba_sword", name = "Sword", cost = 350, atk = 10 },
    { catalog_id = 2, id = "moba_armor", name = "Armor", cost = 300, hp = 100, armor = 5 },
    { catalog_id = 3, id = "moba_boots", name = "Boots", cost = 300, ms = 25 },
    { catalog_id = 4, id = "moba_greatsword", name = "Greatsword", cost = 950, atk = 25,
      recipe = { "moba_sword", "moba_sword" } },
    { catalog_id = 5, id = "moba_guard_charm", name = "Guard Charm", cost = 500,
      active = { kind = "shield", amount = 100, duration = 3 }, cooldown = 12 },
    { catalog_id = 6, id = "moba_stride_charm", name = "Stride Charm", cost = 500,
      active = { kind = "sprint_buff", ms_bonus = 60, duration = 3 }, cooldown = 12 },
    { catalog_id = 7, id = "moba_mana_charm", name = "Mana Charm", cost = 400,
      active = { kind = "restore_mana", amount = 100 }, cooldown = 12 },
    { catalog_id = 8, id = "moba_ward_charm", name = "Ward Charm", cost = 600,
      active = { kind = "damage_reduce", percent = 0.25, duration = 2 }, cooldown = 15 },
    { catalog_id = 9, id = "moba_strike_charm", name = "Strike Charm", cost = 600,
      active = { kind = "headshot_next", bonus_damage = 60 }, cooldown = 10 },
  }
end
