-- Build-time equipment source. Optional active = { kind = "shield", amount = 100,
-- duration = 3 }, cooldown = 10 compiles to native Rust; no runtime Lua.
-- Supported kinds: shield, sprint_buff, restore_mana, damage_reduce, headshot_next,
-- point_nova, unit_hex. Target defaults to self. Point and unit forms require
-- enemy, visible, and out_of_range = "reject". They do not queue or approach.
-- Mana passive bonuses remain unsupported. Active charm values are the initial
-- training balance; existing passive IDs/prices/recipes remain unchanged.
-- Ward Charm stays a self damage-reduction charm, not a ward.
return function(ctx)
  return {
    { catalog_id = 1, id = "moba_sword", name = "鐵劍", cost = 350, atk = 10 },
    { catalog_id = 2, id = "moba_armor", name = "護身甲", cost = 300, hp = 100, armor = 5 },
    { catalog_id = 3, id = "moba_boots", name = "輕行靴", cost = 300, ms = 25 },
    { catalog_id = 4, id = "moba_greatsword", name = "精鋼重劍", cost = 950, atk = 25,
      recipe = { "moba_sword", "moba_sword" } },
    { catalog_id = 5, id = "moba_guard_charm", name = "守護符", cost = 500,
      active = { kind = "shield", amount = 100, duration = 3 }, cooldown = 12 },
    { catalog_id = 6, id = "moba_stride_charm", name = "疾行符", cost = 500,
      active = { kind = "sprint_buff", ms_bonus = 60, duration = 3 }, cooldown = 12 },
    { catalog_id = 7, id = "moba_mana_charm", name = "回魔符", cost = 400,
      active = { kind = "restore_mana", amount = 100 }, cooldown = 12 },
    { catalog_id = 8, id = "moba_ward_charm", name = "堅守符", cost = 600,
      active = { kind = "damage_reduce", percent = 0.25, duration = 2 }, cooldown = 15 },
    { catalog_id = 9, id = "moba_strike_charm", name = "強擊符", cost = 600,
      active = { kind = "headshot_next", bonus_damage = 60 }, cooldown = 10 },
    { catalog_id = 10, id = "moba_training_flare", name = "爆焰符", cost = 450,
      active = { kind = "point_nova", damage = 40, radius = 80 }, cooldown = 8,
      target = { shape = "point", range = 500, team = "enemy", requires_visible = true, out_of_range = "reject" } },
    { catalog_id = 11, id = "moba_training_hex", name = "咒擊符", cost = 450,
      active = { kind = "unit_hex", damage = 55 }, cooldown = 8,
      target = { shape = "unit", range = 450, team = "enemy", requires_visible = true, out_of_range = "reject" } },
  }
end
