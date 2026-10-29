-- Reads the compiled content catalog's v3 export (omoba_template_ids::V3_CATALOG_JSON,
-- v3-content-model-proposal.md §2) into the site catalog that site/pages.lua renders, so
-- build_site --catalog can take the real catalog instead of the census adapter.
--   heroes[]    { id, display_name, title, abilities }   (unit stats from the source roster)
--   abilities[] { id, display_name, max_level, v3 = { text, values, info, statuses, source_id, ... } }
--   items[]     { id, catalog_id, name, cost, recipe, v3 = { text, values, info, stat_lines } }
local M = {}

local root = require("_bootstrap").root

-- info field -> site header key (site/pages.lua ROW_ORDER)
M.INFO = {
  cooldown = "cooldown", mana_cost = "mana", health_cost = "health", gold_cost = "gold",
  charges_max = "charges", charge_restore = "charge_restore", cast_range = "range", aoe_radius = "radius",
  vector_length = "vector_length", cast_point = "cast_point", channel_time = "channel_time",
  required_level = "required_level",
}

function M.is_v3_export(t)
  return type(t) == "table" and t.format == nil and t.schema ~= nil and type(t.abilities) == "table"
end

local function pieces_of(list, owner)
  local out = {}
  for _, p in ipairs(list or {}) do
    if p.value then out[#out + 1] = { value = p.value, owner = p.owner or owner, fmt = p.format or "all" }
    elseif p.name_of then out[#out + 1] = { lit = p.text or p.name_of }  -- resolved by the catalog
    elseif p.rules then
      if p.text then out[#out + 1] = { lit = p.text } else out[#out + 1] = { rules = p.rules, fmt = p.format } end
    else out[#out + 1] = { lit = p.text or "" } end
  end
  return out
end

local function text_of(t, owner, fallback_name)
  local out = {}
  for lang, lt in pairs(t or {}) do
    if type(lt) == "table" and lt.name then
      out[lang] = { name = { { lit = lt.name } } }
      if lt.pieces then out[lang].body = pieces_of(lt.pieces, owner) end
    end
  end
  if not next(out) and fallback_name then out.zh_TW = { name = { { lit = fallback_name } } } end
  return out
end

local function values_of(list)
  local out = {}
  for _, v in ipairs(list or {}) do
    local exact = {}
    for i, r in ipairs(v.ranks or {}) do exact[i] = tostring(r) end
    out[#out + 1] = { name = v.name, unit = v.unit, exact = exact, raw = v.raw, hidden = v.hidden or nil,
      damage_type = v.damage_type, per = v.per ~= "rank" and v.per or nil }
  end
  table.sort(out, function(a, b) return a.name < b.name end)
  return out
end

local function header_of(info, owner)
  local h = {}
  for _, f in ipairs(info or {}) do
    local key = M.INFO[f.field]
    if key and f.value then h[key] = { owner = owner, name = f.value } end
  end
  return h
end

local function load_roster()
  local chunk = assert(loadfile(root .. "/scripts/lua_data/templates/hero_sources.lua", "t", {}))
  local roster = chunk()({})
  local by_id = {}
  for _, h in ipairs(roster.heroes) do by_id[h.id] = h end
  return roster, by_id
end

-- export: decoded V3_CATALOG_JSON; supplement: site/source_supplement (icons, shop categories) or nil
function M.read(export, supplement)
  local roster_meta, roster = load_roster()
  local cat = {
    format = "omoba-site-catalog", format_version = 1, producer = "content_catalog",
    schema_version = 3, source_content_hash = export.content_data_hash,
    heroes = {}, abilities = {}, items = {}, statuses = {}, unit_templates = {}, values = {},
    roster = { repository = roster_meta.repository, commit = roster_meta.commit }, shop_order = {},
  }
  if supplement then for _, c in ipairs(supplement.shop_order or {}) do cat.shop_order[#cat.shop_order + 1] = c end end
  local ability_hero, ability_by_id = {}, {}
  for _, h in ipairs(export.heroes or {}) do
    for _, a in ipairs(h.abilities or {}) do ability_hero[a] = h.id end
  end
  for _, a in ipairs(export.abilities or {}) do
    local v3 = a.v3
    ability_by_id[a.id] = a
    if v3 then
      cat.values[a.id] = values_of(v3.values)
      local statuses = {}
      for _, s in ipairs(v3.statuses or {}) do
        statuses[#statuses + 1] = s.id
        if s.values then cat.values[s.id] = values_of(s.values) end
        cat.statuses[#cat.statuses + 1] = { id = s.id, owner = a.id, visibility = s.visibility, dispel = s.dispel,
          text = text_of(s.text, s.id) }
      end
      cat.abilities[#cat.abilities + 1] = {
        id = a.id, hero = ability_hero[a.id], source_id = v3.source_id, slot = v3.slot, kind = v3.ability_kind or "active",
        max_level = v3.max_level or a.max_level or 1, flags = v3.flags, text = text_of(v3.text, a.id, a.display_name),
        header = header_of(v3.info, a.id), statuses = statuses, coverage = v3.coverage,
        icon = supplement and v3.source_id and supplement.ability_icon(v3.source_id) or nil,
      }
    end
  end
  for _, h in ipairs(export.heroes or {}) do
    -- the source hero is the prefix of its abilities' source ids (B01W -> B01)
    local source
    for _, id in ipairs(h.abilities or {}) do
      local a = ability_by_id[id]
      local sid = a and a.v3 and a.v3.source_id
      if sid and sid:match("^%u%d%d") then source = sid:match("^(%u%d%d)"); break end
    end
    local r = source and roster[source]
    local list = {}
    for _, id in ipairs(h.abilities or {}) do if cat.values[id] then list[#list + 1] = id end end
    if #list > 0 then
      cat.heroes[#cat.heroes + 1] = {
        id = h.id, source_id = source, text = { zh_TW = { name = { { lit = h.display_name or h.id } } } },
        abilities = list, stats = r and r.stats or nil, roster_name = r and r.name or nil,
        icon = supplement and source and supplement.hero_icon(source) or nil,
      }
    end
  end
  for _, it in ipairs(export.items or {}) do
    local v3 = it.v3
    if v3 then
      local values = values_of(v3.values)
      local header = header_of(v3.info, it.id)
      local cost
      for _, f in ipairs(v3.info or {}) do if f.field == "cost" and f.value then cost = { owner = it.id, name = f.value } end end
      if not cost and it.cost then
        values[#values + 1] = { name = "__cost", unit = "gold", exact = { tostring(it.cost) }, raw = { it.cost * 1024 }, hidden = true }
        cost = { owner = it.id, name = "__cost" }
      end
      cat.values[it.id] = values
      local stats = {}
      for _, s in ipairs(v3.stat_lines or {}) do stats[#stats + 1] = { stat = s.stat, op = "add", ref = { owner = it.id, name = s.value } } end
      local components = {}
      for _, c in ipairs(type(it.recipe) == "table" and (it.recipe.components or it.recipe) or {}) do
        components[#components + 1] = type(c) == "table" and (c.id or c.item) or c
      end
      local source_id = v3.source_id or it.id
      cat.items[#cat.items + 1] = {
        id = it.id, source_id = source_id, max_level = 1, text = text_of(v3.text, it.id, it.name), statuses = {},
        shop = { cost = cost, recipe = #components > 0 and { components = components } or nil,
          category = supplement and supplement.shop_category(source_id) or nil },
        stats = stats, active = next(header) and { header = header } or nil,
        icon = supplement and supplement.item_icon(source_id) or nil,
      }
    end
  end
  return cat
end

-- The source catalog (scripts/gen_source_catalog.lua, kind = "source"): every original hero and item,
-- shipped or not. `game` is the game's V3_CATALOG_JSON export (or nil): shipped heroes and shelf items
-- show the game's values instead of the restored original ones (they can differ where the port changed
-- a number), and are marked as in the game.
function M.is_source(t) return type(t) == "table" and t.kind == "source" and type(t.heroes) == "table" end

function M.read_source(src, game, supplement)
  local roster_meta, roster = load_roster()
  local cat = {
    format = "omoba-site-catalog", format_version = 1, producer = "source_catalog",
    schema_version = 3, source_catalog_hash = src.catalog_hash, source_content_hash = game and game.content_data_hash or nil,
    heroes = {}, abilities = {}, items = {}, statuses = {}, unit_templates = {}, values = {}, deferred_items = {},
    roster = { repository = roster_meta.repository, commit = roster_meta.commit }, shop_order = {},
  }
  if supplement then for _, c in ipairs(supplement.shop_order or {}) do cat.shop_order[#cat.shop_order + 1] = c end end
  local game_ability, game_hero, game_item = {}, {}, {}
  for _, a in ipairs(game and game.abilities or {}) do if a.v3 then game_ability[a.id] = a end end
  for _, h in ipairs(game and game.heroes or {}) do game_hero[h.id] = h end
  -- the game lists items by game id ("fire_scroll"); the source catalog and text owners use the v3
  -- document id ("item_fire_scroll", the game item's v3.source_id)
  for _, it in ipairs(game and game.items or {}) do
    if it.v3 then
      game_item[it.id] = it
      if it.v3.source_id then game_item[it.v3.source_id] = it end
    end
  end
  local src_ability = {}
  for _, a in ipairs(src.abilities or {}) do src_ability[a.id] = a end
  -- values owned outside any ability or item block: document-level (<doc>.shared), unit templates,
  -- document-level statuses; pieces name them as owner
  for _, o in ipairs(src.owners or {}) do
    if o.id and o.values then cat.values[o.id] = values_of(o.values) end
  end

  local function add_ability(a, hero, from_game)
    local v3 = a.v3
    if not v3 or cat.values[a.id] then return false end
    cat.values[a.id] = values_of(v3.values)
    local statuses = {}
    for _, s in ipairs(v3.statuses or {}) do
      statuses[#statuses + 1] = s.id
      if s.values then cat.values[s.id] = values_of(s.values) end
      cat.statuses[#cat.statuses + 1] = { id = s.id, owner = a.id, visibility = s.visibility, dispel = s.dispel, text = text_of(s.text, s.id) }
    end
    cat.abilities[#cat.abilities + 1] = {
      id = a.id, hero = hero, source_id = v3.source_id, slot = a.slot or v3.slot, kind = v3.ability_kind,
      max_level = v3.max_level or a.max_level or 1, flags = v3.flags, text = text_of(v3.text, a.id, a.display_name),
      header = header_of(v3.info, a.id), statuses = statuses, coverage = v3.coverage, from_game = from_game or nil,
      icon = supplement and v3.source_id and supplement.ability_icon(v3.source_id) or nil,
    }
    return true
  end

  for _, h in ipairs(src.heroes or {}) do
    local list, gh = {}, h.game_id and game_hero[h.game_id]
    if gh then
      for _, id in ipairs(gh.abilities or {}) do
        local a = game_ability[id]
        if a and add_ability({ id = a.id, slot = src_ability[a.id] and src_ability[a.id].slot, display_name = a.display_name,
          max_level = a.max_level, v3 = a.v3 }, h.source_id, true) then list[#list + 1] = a.id end
      end
    end
    if #list == 0 then
      for _, id in ipairs(h.abilities or {}) do
        local a = src_ability[id]
        if a and add_ability(a, h.source_id, false) then list[#list + 1] = id end
      end
    end
    local r = roster[h.source_id]
    local text = {}
    for lang, name in pairs(h.name or {}) do text[lang] = { name = { { lit = name } } } end
    if not next(text) then text.zh_TW = { name = { { lit = h.source_id } } } end
    cat.heroes[#cat.heroes + 1] = {
      id = h.source_id, source_id = h.source_id, game_id = h.game_id, unlisted = not h.playable or nil,
      game_values = gh and true or nil, text = text, abilities = list, stats = r and r.stats or nil,
      roster_name = r and r.name or nil, icon = supplement and supplement.hero_icon(h.source_id) or nil,
    }
  end

  for _, it in ipairs(src.items or {}) do
    local gi = game_item[it.id]
    local v3 = gi and gi.v3 or it.v3
    local text_names = {}
    for lang, name in pairs(it.name or {}) do text_names[lang] = name end
    if it.deferred or not v3 then
      cat.deferred_items[#cat.deferred_items + 1] = { id = it.id, source_id = it.source_id, name = text_names,
        reason = type(it.deferred) == "string" and it.deferred or nil, note = it.note }
    else
      local values = values_of(v3.values)
      local cost
      for _, f in ipairs(v3.info or {}) do if f.field == "cost" and f.value then cost = { owner = it.id, name = f.value } end end
      if not cost and (it.cost or (gi and gi.cost)) then
        local c = it.cost or gi.cost
        values[#values + 1] = { name = "__cost", unit = "gold", exact = { tostring(c) }, raw = { c * 1024 }, hidden = true }
        cost = { owner = it.id, name = "__cost" }
      end
      cat.values[it.id] = values
      local stats = {}
      for _, s in ipairs(v3.stat_lines or {}) do stats[#stats + 1] = { stat = s.stat, op = "add", ref = { owner = it.id, name = s.value } } end
      local header = {}
      for k, v in pairs(header_of(v3.info, it.id)) do header[k] = v end
      local recipe_src = it.recipe or (gi and gi.recipe) or v3.recipe
      local recipe
      if type(recipe_src) == "table" then
        local components = {}
        for _, c in ipairs(recipe_src.components or recipe_src) do components[#components + 1] = type(c) == "table" and (c.id or c.item) or c end
        local rc
        if type(recipe_src.recipe_cost) == "string" and cat.values[it.id] then rc = { owner = it.id, name = recipe_src.recipe_cost } end
        if #components > 0 then recipe = { components = components, recipe_cost = rc } end
      end
      local source_id = it.source_id or v3.source_id or it.id
      local statuses = {}
      for _, s in ipairs(v3.statuses or {}) do
        statuses[#statuses + 1] = s.id
        if s.values then cat.values[s.id] = values_of(s.values) end
        cat.statuses[#cat.statuses + 1] = { id = s.id, owner = it.id, visibility = s.visibility, dispel = s.dispel, text = text_of(s.text, s.id) }
      end
      cat.items[#cat.items + 1] = {
        id = it.id, source_id = source_id, max_level = 1, text = text_of(v3.text, it.id, text_names.zh_TW), statuses = statuses,
        shop = { cost = cost, recipe = recipe, category = it.category or (it.shop and it.shop.category) or (supplement and supplement.shop_category(source_id)) or nil },
        stats = stats, active = next(header) and { header = header } or nil, unlisted = (game and not gi) or nil,
        game_values = gi and true or nil, icon = supplement and supplement.item_icon(source_id) or nil,
      }
    end
  end
  return cat
end

return M
