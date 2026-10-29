-- Build-time equipment source. Optional active = { kind = "shield", amount = 100,
-- duration = 3 }, cooldown = 10 compiles to native Rust; no runtime Lua.
-- Supported kinds: shield, sprint_buff, restore_mana, damage_reduce, headshot_next,
-- point_nova, unit_hex. Target defaults to self. Point and unit forms require
-- enemy (positive other team) or hostile (faction hostility including aggressive
-- neutrals), visible, and out_of_range = "reject". They do not queue or approach.
-- Mana passive bonuses remain unsupported. Active charm values are the initial
-- training balance; existing passive IDs/prices/recipes remain unchanged.
-- Ward Charm stays a self damage-reduction charm, not a ward.
--
-- E155: the shop sells history_nobu items only. Source items are built from
-- templates/source_items.lua (exported from the pinned source catalog) and checked by
-- templates/item_sources_check.lua at build time. The moba_* entries are engine fixtures
-- (test_only): never sold, never a bot goal, kept for the effect engine's tests until each
-- effect has a source item. A source item reaches the shop only when every capability it
-- needs exists (needs_framework rows stay out; see source-index/item-compare.md).
local SOURCE_ITEMS = {
  -- { catalog_id, source_ref }: items whose every effect the engine supports today.
  { 101, "item_power_ingot" },
  { 102, "item_spear" },
  { 103, "item_cursed_katana" },
  { 104, "item_lesser_shield" },
  { 105, "item_stable_shield" },
  { 106, "item_body_wrap" },
  { 107, "item_health_ingot" },
  -- Passive stat modifiers through BuffStore (E155 C).
  { 108, "item_katana" },
  { 109, "item_mana_ingot" },
  { 110, "item_regeneration_ingot" },
  { 111, "item_speed_clothes" },
  { 112, "item_spirit_ingot" },
  { 113, "item_tea_bowl" },
  { 114, "item_the_hooate_of_ghost" },
  -- Attribute bonuses (templates/attribute_rules.lua).
  { 115, "item_strength_ingot" },
  { 116, "item_agility_ingot" },
  { 117, "item_intelligence_ingot" },
  { 118, "item_naginata" },
  { 119, "item_tachi" },
  { 120, "item_staff" },
  { 121, "item_energy_ingot" },
  { 122, "item_moon_spear" },
  { 123, "item_gorgeous_tachi" },
  -- Datadriven actives translated from the source KV (E160, scripts/item_source/active.lua).
  { 127, "item_fire_scroll" },
  { 128, "item_red_spear" },
  -- Passive aura (E173): the aura and its permanent copy from the active follow history_nobu.
  { 129, "item_the_art_of_war_flame_chapter" },
  -- Ally unit-target heal and single-use consumables (E160 batch 2).
  { 130, "item_izuminokami_kanesada" },
  { 131, "item_rations" },
  { 132, "item_liquor" },
  { 133, "item_sake" },
  -- New in history_nobu (E169).
  { 134, "item_attack_speed_ingot" },
  -- Enemy unit-target damage and control, ported from the scripts (E176).
  { 135, "item_blowgun" },
  { 136, "item_shuriken" },
  -- Caster regeneration modifier (E176: the code decides, the older text is reported only).
  { 137, "item_the_poketto_of_meiso" },
  -- Point area damage from the script's FindUnitsInRadius loop (E178).
  { 138, "item_napalm_bomb" },
  -- Magic immunity with strong dispel (E179, item_perceive_wine.lua#Shock), single use.
  { 139, "item_perceive_wine" },
  -- Unique movespeed (highest wins), merged Modifiers blocks and the upgrade part (E183).
  { 140, "item_pleated_skirt" },
  { 141, "item_dark_boots" },
  { 142, "item_hama_orb" },
  { 143, "item_dark_boots_1" },
  -- Spell amplification (E188): skill and item damage with an inflictor +8% / +10%.
  { 144, "item_mana_boots" },
  { 145, "item_mana_boots_1" },
  -- Spell books (E188): +4.5% amplification; the actives read from the reviewed scripts.
  { 146, "item_spell_book" },
  { 147, "item_rockfall_book" },
  { 148, "item_ignite_book" },
}

return function(ctx)
  local source = ctx.include("templates/source_items.lua")
  local sources_check = ctx.include("templates/item_sources_check.lua")
  -- Package schema v3 texts and values (scripts/gen_item_v3.lua from docs/character_pipeline/items):
  -- the description is the rendered zh_TW template; an item without a body shows its stat lines.
  local v3_texts = ctx.include("templates/item_texts_v3.lua")
  local rows = {}
  for _, row in ipairs(source.items) do rows[row.key] = row end
  -- source_ref is `history_nobu@<commit>:item_<key>`; the game id is <key>.
  local function id_of(ref) return (ref:match("item_([%w_]+)$")) end
  local items = {
    { catalog_id = 1, id = "moba_sword", test_only = true, name = "鐵劍", cost = 350, atk = 10 },
    { catalog_id = 2, id = "moba_armor", test_only = true, name = "護身甲", cost = 300, hp = 100, armor = 5 },
    { catalog_id = 3, id = "moba_boots", test_only = true, name = "輕行靴", cost = 300,
      unique_passive = {group="movement",name="疾行",priority=1,ms=25},
      purchase_group = {id="footwear",name="鞋靴",limit=1} },
    { catalog_id = 4, id = "moba_greatsword", test_only = true, name = "精鋼重劍", cost = 950, atk = 25,
      recipe = { "moba_sword", "moba_sword" } },
    { catalog_id = 5, id = "moba_guard_charm", test_only = true, name = "守護符", cost = 500,
      active = { kind = "shield", amount = 100, duration = 3 }, cooldown = 12 },
    { catalog_id = 6, id = "moba_stride_charm", test_only = true, name = "疾行符", cost = 500,
      active = { kind = "sprint_buff", ms_bonus = 60, duration = 3 }, cooldown = 12 },
    { catalog_id = 7, id = "moba_mana_charm", test_only = true, name = "回魔符", cost = 400,
      active = { kind = "restore_mana", amount = 100 }, cooldown = 12 },
    { catalog_id = 8, id = "moba_ward_charm", test_only = true, name = "堅守符", cost = 600,
      active = { kind = "damage_reduce", percent = 0.25, duration = 2 }, cooldown = 15 },
    { catalog_id = 9, id = "moba_strike_charm", test_only = true, name = "強擊符", cost = 600,
      active = { kind = "headshot_next", bonus_damage = 60 }, cooldown = 10 },
    { catalog_id = 10, id = "moba_training_flare", test_only = true, name = "爆焰符", cost = 450,
      active = { kind = "point_nova", damage = 40, radius = 80 }, cooldown = 8,
      target = { shape = "point", range = 500, team = "enemy", requires_visible = true, out_of_range = "reject" } },
    { catalog_id = 11, id = "moba_training_hex", test_only = true, name = "咒擊符", cost = 450,
      active = { kind = "unit_hex", damage = 55, healing_reduction = {multiplier=0.5,duration=4} }, cooldown = 8,
      target = { shape = "unit", range = 450, team = "enemy", requires_visible = true, out_of_range = "reject" } },
    { catalog_id = 12, id = "moba_recovery_charm", test_only = true, name = "靜心符", cost = 500,
      active = { kind = "channel_heal", amount = 40, duration = 3, interval = 1 }, cooldown = 30 },
    { catalog_id = 13, id = "moba_binding_charm", test_only = true, name = "縛行符", cost = 650,
      active = { kind = "unit_hex", damage = 30, control = { kind = "root", duration = 1.5 } }, cooldown = 18,
      target = { shape = "unit", range = 450, team = "hostile", requires_visible = true, out_of_range = "reject" } },
    { catalog_id = 14, id = "moba_purification_charm", test_only = true, name = "淨化護符", cost = 1400,
      active = { kind = "self_control_cleanse" }, cooldown = 45 },
    { catalog_id = 15, id = "moba_leech_blade", test_only = true, name = "汲生短刃", cost = 900,
      atk = 10, lifesteal = 0.125, recipe = { "moba_sword" } },
    { catalog_id = 16, id = "moba_leech_saber", test_only = true, name = "汲生長刀", cost = 1800,
      atk = 25, lifesteal = 0.25, recipe = { "moba_leech_blade", "moba_sword" } },
    { catalog_id = 17, id = "moba_precision_glove", test_only = true, name = "精準手套", cost = 600,
      critical_chance = 0.125 },
    { catalog_id = 18, id = "moba_precision_saber", test_only = true, name = "精準長刀", cost = 1800,
      atk = 25, critical_chance = 0.25, recipe = { "moba_precision_glove", "moba_sword" } },
    { catalog_id = 19, id = "moba_swift_boots", test_only = true, name = "疾步長靴", cost = 900,
      unique_passive = {group="movement",name="疾行",priority=2,ms=45},recipe={"moba_boots"},
      purchase_group = {id="footwear",name="鞋靴",limit=1} },
  }
  for _, entry in ipairs(SOURCE_ITEMS) do
    local row = assert(rows[entry[2]], "unknown source item " .. entry[2])
    assert(row.source_ref == source.name .. "@" .. source.commit .. ":" .. entry[2], "source row " .. entry[2] .. " cites another source")
    local item = { catalog_id = entry[1], id = id_of(row.source_ref), source_ref = row.source_ref,
      name = row.name, cost = row.cost, description = row.description }
    for stat, value in pairs(row.stats) do item[stat] = value end
    if next(row.modifiers or {}) then
      item.modifiers = {}
      for key, value in pairs(row.modifiers) do item.modifiers[key] = value end
    end
    if row.active then
      item.active, item.cooldown, item.mana_cost = row.active, row.cooldown, row.mana_cost
      item.consumable, item.target = row.consumable, row.target
      item.shares_cooldown_with = row.shares_cooldown_with
    end
    if row.unique_passive then
      item.unique_passive = row.unique_passive
    end
    if row.aura then
      item.aura = row.aura
    end
    if row.recipe then
      item.recipe = {}
      for _, part in ipairs(row.recipe.components) do item.recipe[#item.recipe + 1] = id_of(part) end
    end
    items[#items + 1] = item
  end
  sources_check.check(items, source)
  -- Package schema v3 is the single source of the shelf's numbers and recipes (value-single-source
  -- §3.3): scripts/gen_item_v3.lua resolves every number of the row and the recipe from the v3 values
  -- (templates/item_shop_v3.lua) and compares them with the source catalog export, which only shapes
  -- the row here. `ctx.source_shelf` returns the shelf as the source catalog shapes it, before any v3
  -- number or text (gen_item_v3's comparison and scripts/tests/item_sources_check_test.lua).
  if ctx.source_shelf then return items end
  do
    local shop = ctx.include("templates/item_shop_v3.lua")
    local skip = { catalog_id = true, recipe = true, shares_cooldown_with = true, id = true, name = true,
      description = true, source_ref = true, v3 = true, test_only = true, purchase_group = true }
    for _, item in ipairs(items) do
      local s = shop[item.id]
      if s then
        local used = {}
        local function walk(t, prefix)
          for k, v in pairs(t) do
            if not (prefix == "" and skip[k]) then
              local path = prefix == "" and tostring(k) or (prefix .. "." .. tostring(k))
              if type(v) == "number" then
                t[k] = assert(s.numbers[path], item.id .. ": shelf number " .. path .. " has no v3 value; run scripts/gen_item_v3.lua")
                used[path] = true
              elseif type(v) == "table" then walk(v, path) end
            end
          end
        end
        walk(item, "")
        for path in pairs(s.numbers) do assert(used[path], item.id .. ": v3 number " .. path .. " is not on the shelf row; run scripts/gen_item_v3.lua") end
        item.recipe = s.recipe
      elseif not item.test_only then
        error(item.id .. " has no v3 shop row; run scripts/gen_item_v3.lua")
      end
    end
  end
  -- After the source check (which compares the exported source text): the v3 block replaces it.
  for _, item in ipairs(items) do
    local block = v3_texts[item.id]
    if block then
      item.v3 = block
      local zh = block.text and block.text.zh_TW
      item.description = zh and zh.rendered or nil
    end
  end
  return items
end
