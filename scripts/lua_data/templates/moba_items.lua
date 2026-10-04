-- Passive equipment only. Active items and mana bonuses are not yet supported
-- by the MOBA slice; do not author placeholder effects that claim otherwise.
return function(ctx)
  return {
    { catalog_id = 1, id = "moba_sword", name = "Sword", cost = 350, atk = 10 },
    { catalog_id = 2, id = "moba_armor", name = "Armor", cost = 300, hp = 100, armor = 5 },
    { catalog_id = 3, id = "moba_boots", name = "Boots", cost = 300, ms = 25 },
    { catalog_id = 4, id = "moba_greatsword", name = "Greatsword", cost = 950, atk = 25,
      recipe = { "moba_sword", "moba_sword" } },
  }
end
