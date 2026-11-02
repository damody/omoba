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
  { 140, "item_pleated_skirt", true },
  { 141, "item_dark_boots", true },
  { 142, "item_hama_orb" },
  { 143, "item_dark_boots_1", true },
  -- Spell amplification (E188): skill and item damage with an inflictor +8% / +10%.
  { 144, "item_mana_boots", true },
  { 145, "item_mana_boots_1", true },
  -- Spell books (E188): +4.5% amplification; the actives read from the reviewed scripts.
  { 146, "item_spell_book" },
  { 147, "item_rockfall_book" },
  { 148, "item_ignite_book" },
  -- Effects past the catalog's translation run in items.dll, generated from the item's v3 document
  -- (third field): the source row's capability gaps are that script's, not the shelf's.
  { 149, "item_brave_ring", true },
  { 150, "item_ghost_mask", true },
  { 151, "item_lesser_armor", true },
  { 152, "item_sakabatou", true },
  { 153, "item_satsuma_gun", true },
  { 154, "item_toughness_ingot", true },
  { 155, "item_big_tachi", true },
  -- Actives whose effects run in items.dll; the catalog's Scripted active and target come from the v3 document.
  { 156, "item_flash_ring", true },
  { 157, "item_flash_shoes", true },
  { 158, "item_flood_book", true },
  { 159, "item_frozen_ring", true },
  { 160, "item_fubuki_shoulders", true },
  { 161, "item_healing_totem", true },
  { 162, "item_kisyosennfuyuzuki", true },
  { 163, "item_lightning_scroll", true },
  { 164, "item_logs_big_tachi", true },
  { 165, "item_mountain_scroll", true },
  { 166, "item_nagamaki", true },
  { 167, "item_ninja_sword", true },
  { 168, "item_wind_scroll", true },
  { 169, "item_storm_book", true },
  { 170, "item_the_club_of_nebula", true },
  { 171, "item_the_great_sword_of_anger", true },
  { 172, "item_the_great_sword_of_disease", true },
  { 173, "item_the_great_sword_of_hurricane", true },
  { 174, "item_the_great_sword_of_spike", true },
  { 175, "item_the_overflame_art_of_war", true },
  { 176, "item_the_red_lightning_chapter", true },
  { 177, "item_the_red_lightning_chapter_hyper", true },
  { 178, "item_thunderstorms", true },
  { 179, "item_the_storm_call_book", true },
  { 180, "item_yubashiri", true },
  { 181, "item_the_great_sword_of_xnew_moon", true },
  { 182, "item_ninja1", true },
  { 183, "item_chidori", true },
  { 184, "item_Sukemune", true },
  { 185, "item_kokumo", true },
  { 186, "item_lightning_katana", true },
  { 187, "item_Oochidorijuumonnjiyari", true },
  { 188, "item_raikiri", true },
  { 189, "item_the_great_spear_of_dragonfly", true },
  { 190, "item_the_great_sword_of_toxic", true },
  { 191, "item_the_spear_of_ghost", true },
  { 192, "item_the_scream_of_spiders", true },
  { 193, "item_wayumi2", true },
  { 194, "item_the_lost_art_of_war_2", true },
  { 195, "item_the_dark_armor", true },
  { 196, "item_the_art_of_war_tornado_chapter", true },
  { 197, "item_the_art_of_war_mountain_chapter", true },
  { 198, "item_swiftness_boots_1", true },
  { 199, "item_swiftness_boots", true },
  { 200, "item_samurai_cloth", true },
  { 201, "item_ninja_cloth", true },
  { 202, "item_shun_armor", true },
  { 203, "item_samurai's_badge" },
  { 204, "item_flame_armor", true },
  { 205, "item_foxfire_armor", true },
  { 206, "item_wayumi", true },
  { 207, "item_glass_mitsutomo", true },
  { 208, "item_Hannya_Onimaru", true },
  { 209, "item_kousetsusamonnji", true },
  { 210, "item_magic_defence_boots", true },
  { 211, "item_magic_defence_boots_1", true },
  { 212, "item_magic_resistance_ingot", true },
  { 213, "item_ryukyu_ronin_clogs", true },
  { 214, "item_ryukyu_ronin_clogs_1", true },
  { 215, "item_the_great_silver_armor", true },
  { 216, "item_the_great_sword_of_precision", true },
  { 217, "item_the_great_sword_of_soul_hyper", true },
  { 218, "item_Sukezane", true },
  { 219, "item_the_spear_of_spiders", true },
  { 220, "item_the_blade_of_blood_devourer", true },
  { 221, "item_the_great_sword_of_tiger", true },
  { 222, "item_the_sword_of_kaminaobi", true },
  { 223, "item_ninja2", true },
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
  local function id_of(ref) local id = ref:match("item_([%w_']+)$"); return id and id:lower():gsub("'", "") end
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
  local scripted = {}
  ctx.scripted_items = scripted
  for _, entry in ipairs(SOURCE_ITEMS) do
    local row = assert(rows[entry[2]], "unknown source item " .. entry[2])
    if entry[3] then scripted[id_of(row.source_ref)] = true end
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
  sources_check.check(items, source, scripted)
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
        if s.scripted then
          item.active, item.target, item.cooldown, item.mana_cost = nil, nil, nil, nil
          item.consumable, item.shares_cooldown_with = nil, nil
        end
        if s.no_native_aura then item.aura = nil end
        for path, number in pairs(s.added or {}) do
          local key = path:match("^modifiers%.(.+)$")
          if key then
            item.modifiers = item.modifiers or {}
            item.modifiers[key] = number
          else
            item[path] = number
          end
        end
        for _, path in ipairs(s.not_passive or {}) do
          local key = path:match("^modifiers%.(.+)$")
          if key then
            item.modifiers[key] = nil
            if not next(item.modifiers) then item.modifiers = nil end
          else
            item[path] = nil
          end
        end
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
        -- its active runs in items.dll: the catalog fields the host checks come from the v3 active
        for field, value in pairs(s.scripted or {}) do item[field] = value end
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
  -- Items a hero document defines (C06 item_c06e, scripts/v3_test_heroes.lua hero_items): a catalog
  -- row each so the hero's generated item entry registers and give_item finds it (E571). Never
  -- sold, never a bot goal, no source row (test_only); listed under the v3 id `item_<key>` itself,
  -- which the kit's item_def and the host's item_script_id accept. Catalog ids from 900 in id order.
  local heroes = ctx.include("templates/v3_test_heroes.lua")
  for i, it in ipairs(heroes.hero_items or {}) do
    items[#items + 1] = { catalog_id = 900 + i, id = it.id, test_only = true, name = it.name, cost = it.cost }
  end
  return items
end
