-- Build-time equipment source. Optional active = { kind = "shield", amount = 100,
-- duration = 3 }, cooldown = 10 compiles to native Rust; no runtime Lua.
-- Supported kinds: shield, sprint_buff, restore_mana, damage_reduce, headshot_next,
-- point_nova, unit_hex. Target defaults to self. Point and unit forms require
-- enemy (positive other team) or hostile (faction hostility including aggressive
-- neutrals), visible, and out_of_range = "reject". They do not queue or approach.
-- Mana passive bonuses remain unsupported. Active charm values are the initial
-- training balance; existing passive IDs/prices/recipes remain unchanged.
-- Ward Charm stays a self damage-reduction charm, not a ward.
return function(ctx)
  return {
    { catalog_id = 1, id = "moba_sword", name = "鐵劍", cost = 350, atk = 10 },
    { catalog_id = 2, id = "moba_armor", name = "護身甲", cost = 300, hp = 100, armor = 5 },
    { catalog_id = 3, id = "moba_boots", name = "輕行靴", cost = 300,
      unique_passive = {group="movement",name="疾行",priority=1,ms=25},
      purchase_group = {id="footwear",name="鞋靴",limit=1} },
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
      active = { kind = "unit_hex", damage = 55, healing_reduction = {multiplier=0.5,duration=4} }, cooldown = 8,
      target = { shape = "unit", range = 450, team = "enemy", requires_visible = true, out_of_range = "reject" } },
    { catalog_id = 12, id = "moba_recovery_charm", name = "靜心符", cost = 500,
      active = { kind = "channel_heal", amount = 40, duration = 3, interval = 1 }, cooldown = 30 },
    { catalog_id = 13, id = "moba_binding_charm", name = "縛行符", cost = 650,
      active = { kind = "unit_hex", damage = 30, control = { kind = "root", duration = 1.5 } }, cooldown = 18,
      target = { shape = "unit", range = 450, team = "hostile", requires_visible = true, out_of_range = "reject" } },
    { catalog_id = 14, id = "moba_purification_charm", name = "淨化護符", cost = 1400,
      active = { kind = "self_control_cleanse" }, cooldown = 45 },
    { catalog_id = 15, id = "moba_leech_blade", name = "汲生短刃", cost = 900,
      atk = 10, lifesteal = 0.125, recipe = { "moba_sword" } },
    { catalog_id = 16, id = "moba_leech_saber", name = "汲生長刀", cost = 1800,
      atk = 25, lifesteal = 0.25, recipe = { "moba_leech_blade", "moba_sword" } },
    { catalog_id = 17, id = "moba_precision_glove", name = "精準手套", cost = 600,
      critical_chance = 0.125 },
    { catalog_id = 18, id = "moba_precision_saber", name = "精準長刀", cost = 1800,
      atk = 25, critical_chance = 0.25, recipe = { "moba_precision_glove", "moba_sword" } },
    { catalog_id = 19, id = "moba_swift_boots", name = "疾步長靴", cost = 900,
      unique_passive = {group="movement",name="疾行",priority=2,ms=45},recipe={"moba_boots"},
      purchase_group = {id="footwear",name="鞋靴",limit=1} },
  }
end
