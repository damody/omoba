-- Build-time equipment source. Optional active = { kind = "shield", amount = 100,
-- duration = 3 }, cooldown = 10 compiles to native Rust; no runtime Lua.
-- Supported kinds: shield, sprint_buff, restore_mana, damage_reduce, headshot_next.
-- Mana passive bonuses remain unsupported. Current catalog retains its balance.
return function(ctx)
  return {
    { catalog_id = 1, id = "moba_sword", name = "Sword", cost = 350, atk = 10 },
    { catalog_id = 2, id = "moba_armor", name = "Armor", cost = 300, hp = 100, armor = 5 },
    { catalog_id = 3, id = "moba_boots", name = "Boots", cost = 300, ms = 25 },
    { catalog_id = 4, id = "moba_greatsword", name = "Greatsword", cost = 950, atk = 25,
      recipe = { "moba_sword", "moba_sword" } },
  }
end
