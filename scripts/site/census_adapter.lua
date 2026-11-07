-- Adapter: schema v3 documents (packages/<pkg>/v3.lua, items/*.lua; see
-- docs/character_pipeline/tools/v3_sources.lua) -> site catalog.
--
-- Until the compiled content catalog exists (M1 data line), the site reads v3
-- documents directly and compiles them into the shape of
-- docs/character_pipeline/schema/v3-content-model-proposal.md §2:
--   values  { name, unit, exact = { "n"|"n/d", ... }, raw = { Fixed64 raw, ... },
--             hidden, damage_type, derived, per }  (the displayed number: pct 25 = 25%)
--   text    lang -> { field -> pieces }  (parsed templates, value pieces resolved to
--             { owner = <id>, value = <name>, fmt })
--   header  info-row fields -> { owner, name } references into values
-- Hero unit stats come from the generated roster (scripts/lua_data/templates/hero_sources.lua).
-- Icons and shop categories come from the pinned original (site/source_supplement.lua).
-- When the real catalog lands, build_site reads it with --catalog and this file goes away.
local Q = require("site.rational")
local R = require("site.text_render")
local M = {}

local root = require("_bootstrap").root
local tools = root .. "/docs/character_pipeline/tools"
local P = dofile(tools .. "/package_v3.lua")
local X = P.expr

-- every v3 document at its authoring location (absolute paths)
function M.default_docs() return dofile(tools .. "/v3_sources.lua").all(root) end
M.SLOT_ORDER = { Q = 1, W = 2, E = 3, R = 4, D = 5, F = 6, sub = 7, hidden = 8 }

local function sorted_keys(t)
  local ks = {}
  for k in pairs(t) do ks[#ks + 1] = k end
  table.sort(ks, function(a, b) return tostring(a) < tostring(b) end)
  return ks
end

-------------------------------------------------------------------------------
-- owners and values
-------------------------------------------------------------------------------
local function new_state()
  return { owners = {}, order = {}, errors = {}, warnings = {} }
end

local function err(st, msg) st.errors[#st.errors + 1] = msg end
local function warn(st, msg) st.warnings[#st.warnings + 1] = msg end

local function add_owner(st, id, kind, def, parent, extra)
  if st.owners[id] then err(st, "duplicate id across documents: " .. id); return st.owners[id] end
  local o = { id = id, kind = kind, def = def, parent = parent, max_level = def.max_level or (parent and parent.max_level) or 1 }
  for k, v in pairs(extra or {}) do o[k] = v end
  st.owners[id] = o
  st.order[#st.order + 1] = o
  return o
end

-- the owner that defines `name` looking outward from `o` (schema v3 §2.1)
local function resolve(st, o, qual, name)
  if qual then
    local q = st.owners[qual]
    if q and q.def.values and q.def.values[name] ~= nil then return q end
    return nil
  end
  while o do
    if o.def.values and o.def.values[name] ~= nil then return o end
    o = o.parent
  end
  return nil
end

local compile_value -- forward

local FUNCS = {
  floor = function(a) return Q.floor(a[1]) end,
  ceil = function(a) return Q.ceil(a[1]) end,
  abs = function(a) return Q.abs(a[1]) end,
  round = function(a) return Q.new(Q.round_int(a[1])) end,
  -- IEEE sqrt is correctly rounded, so this is deterministic; kept to 9 decimals
  sqrt = function(a) return Q.round(Q.parse(string.format("%.12f", math.sqrt(a[1].n / a[1].d))), 9) end,
  min = function(a) local m = a[1]; for i = 2, #a do if Q.cmp(a[i], m) < 0 then m = a[i] end end; return m end,
  max = function(a) local m = a[1]; for i = 2, #a do if Q.cmp(a[i], m) > 0 then m = a[i] end end; return m end,
}

-- constant evaluation of a value expression at one rank; nil + reason when it reads runtime state
local function eval(st, o, ast, rank)
  local k = ast.k
  if k == "num" then return Q.parse(ast.text) end
  if k == "val" then
    local holder = resolve(st, o, ast.owner, ast.name)
    if not holder then return nil, "unknown value $" .. (ast.owner and (ast.owner .. ".") or "") .. ast.name end
    local v, why = compile_value(st, holder, ast.name)
    if not v then return nil, why end
    if not v.exact then return nil, "$" .. ast.name .. " is not constant" end
    local list = v.exact
    return Q.parse(list[math.max(1, math.min(#list, rank))])
  end
  if k == "un" and ast.op == "-" then
    local a, why = eval(st, o, ast.e, rank); if not a then return nil, why end
    return Q.neg(a)
  end
  if k == "bin" then
    local a, why = eval(st, o, ast.l, rank); if not a then return nil, why end
    local b, why2 = eval(st, o, ast.r, rank); if not b then return nil, why2 end
    if ast.op == "+" then return Q.add(a, b) elseif ast.op == "-" then return Q.sub(a, b)
    elseif ast.op == "*" then return Q.mul(a, b) elseif ast.op == "/" then return Q.div(a, b)
    elseif ast.op == "%" then return Q.mod(a, b) end
    return nil, "operator " .. ast.op .. " is not numeric"
  end
  if k == "call" and FUNCS[ast.fn] then
    local args = {}
    for i, a in ipairs(ast.args) do
      local x, why = eval(st, o, a, rank); if not x then return nil, why end
      args[i] = x
    end
    return FUNCS[ast.fn](args)
  end
  return nil, "reads runtime state"
end

compile_value = function(st, o, name)
  o.values = o.values or {}
  local cached = o.values[name]
  if cached == false then return nil, "circular value " .. o.id .. "." .. name end
  if cached then return cached end
  local d = o.def.values[name]
  o.values[name] = false
  local v = { name = name, unit = d.unit, hidden = d.hidden or nil, damage_type = d.damage_type,
    per = d.per ~= "rank" and d.per or nil }
  local list = {}
  if d.expr then
    v.derived = true
    v.expr = d.expr
    local ast, perr = X.parse(d.expr)
    if not ast then o.values[name] = nil; return nil, perr end
    local n = o.max_level
    for r = 1, n do
      local x, why = eval(st, o, ast, r)
      if not x then list = nil; v.runtime = why; break end
      list[r] = x
    end
  else
    -- catalog values are the displayed numbers (pct 25 = 25%); the IR compiler turns pct
    -- into a ratio for behavior (v3-content-model-proposal.md §2.2)
    for i, x in ipairs(d) do list[i] = Q.parse(x) end
  end
  if list then
    v.exact, v.raw = {}, {}
    for i, q in ipairs(list) do
      v.exact[i] = Q.exact(q)
      if d.rounding == "truncate" then
        local s = Q.mul(q, Q.new(Q.SCALE))
        v.raw[i] = s.n < 0 and -((-s.n) // s.d) or s.n // s.d
      else
        v.raw[i] = Q.to_raw(q)
      end
    end
  end
  o.values[name] = v
  return v
end

-------------------------------------------------------------------------------
-- text and header
-------------------------------------------------------------------------------
local TEXT_FIELDS = { "name", "body", "short", "lore", "status_name", "status_body" }

local function compile_text(st, o)
  local t = o.def.text
  if type(t) ~= "table" then return nil end
  local out = {}
  for _, lang in ipairs(P.vocab.LANGS) do
    local lt = t[lang]
    if lt then
      out[lang] = {}
      for _, f in ipairs(TEXT_FIELDS) do
        if lt[f] then
          local okp, pieces = pcall(R.parse, lt[f])
          if not okp then err(st, o.id .. ".text." .. lang .. "." .. f .. ": " .. tostring(pieces))
          else
            for _, p in ipairs(pieces) do
              if p.value then
                local holder = resolve(st, o, p.owner, p.value)
                if not holder then err(st, o.id .. ": {" .. p.value .. "} has no value")
                else
                  p.owner = holder.id
                  local v, why = compile_value(st, holder, p.value)
                  if not v then err(st, o.id .. ": " .. why)
                  elseif not v.exact then err(st, o.id .. ": {" .. p.value .. "} is not constant (" .. tostring(v.runtime) .. ")") end
                end
              end
            end
            out[lang][f] = pieces
          end
        end
      end
    end
  end
  return out
end

-- "$x" / "$owner.x" -> { owner, name }; other expressions become an inline derived value
local function header_ref(st, o, src, key)
  if src == nil then return nil end
  if type(src) ~= "string" then src = tostring(src) end
  local ast = X.parse(src)
  if ast and ast.k == "val" then
    local holder = resolve(st, o, ast.owner, ast.name)
    if holder then
      local v = compile_value(st, holder, ast.name)
      if v and v.exact then return { owner = holder.id, name = ast.name } end
    end
  end
  -- not a plain value reference: evaluate it as an anonymous derived value on this owner
  local name = "__header_" .. key
  o.def.values = o.def.values or {}
  o.def.values[name] = { expr = src, unit = "number", hidden = true }
  local v = compile_value(st, o, name)
  if v and v.exact then return { owner = o.id, name = name } end
  warn(st, o.id .. ": header " .. key .. " = " .. src .. " is not constant; not shown")
  return nil
end

local function compile_header(st, o, a)
  local h = {}
  local target, cast, cost, req = a.target or {}, a.cast or {}, a.cost or {}, a.requires or {}
  h.cooldown = header_ref(st, o, cost.cooldown, "cooldown")
  h.mana = header_ref(st, o, cost.mana, "mana")
  h.health = header_ref(st, o, cost.health, "health")
  h.gold = header_ref(st, o, cost.gold, "gold")
  if type(cost.charges) == "table" then
    h.charges = header_ref(st, o, cost.charges.max, "charges")
    h.charge_restore = header_ref(st, o, cost.charges.restore, "charge_restore")
  end
  h.range = header_ref(st, o, target.range, "range")
  h.radius = header_ref(st, o, target.aoe_radius, "radius")
  h.vector_length = header_ref(st, o, target.vector_length, "vector_length")
  h.cast_point = header_ref(st, o, cast.cast_point, "cast_point")
  h.channel_time = header_ref(st, o, cast.channel_time, "channel_time")
  h.required_level = header_ref(st, o, req.hero_level, "required_level")
  h.target_type = target.type
  h.target_team = target.team
  return h
end

-------------------------------------------------------------------------------
-- documents
-------------------------------------------------------------------------------
local function list_docs(dir)
  local lfs = require("lfs")
  local files = {}
  local function visit(d)
    local names = {}
    for n in lfs.dir(d) do if n ~= "." and n ~= ".." then names[#names + 1] = n end end
    table.sort(names)
    for _, n in ipairs(names) do
      local p = d .. "/" .. n
      local mode = lfs.attributes(p, "mode")
      if mode == "directory" then visit(p)
      elseif n:match("%.lua$") then files[#files + 1] = p end
    end
  end
  if lfs.attributes(dir, "mode") == "file" then files[1] = dir else visit(dir) end
  return files
end

local function load_roster()
  local chunk = assert(loadfile(root .. "/scripts/lua_data/templates/hero_sources.lua", "t", {}))
  local roster = chunk()({})
  local by_id = {}
  for _, h in ipairs(roster.heroes) do by_id[h.id] = h end
  return roster, by_id
end

local function stat_entries(st, o, stats)
  local out = {}
  if type(stats) ~= "table" then return out end
  for _, key in ipairs(sorted_keys(stats)) do
    local s = stats[key]
    local src, op = s, "add"
    if type(s) == "table" then src, op = s.value, s.op or "add" end
    local ref = header_ref(st, o, src, "stat_" .. key)
    if ref then out[#out + 1] = { stat = key, op = op, ref = ref, when = type(s) == "table" and s.when or nil } end
  end
  return out
end

-- docs: list of files/dirs. opts.validate (default true), opts.supplement (site/source_supplement)
function M.build(paths, opts)
  opts = opts or {}
  local st = new_state()
  local docs = {}
  for _, p in ipairs(paths) do
    for _, f in ipairs(list_docs(p)) do
      local doc, lerr = P.load_file(f)
      if not doc then err(st, f .. ": " .. lerr)
      elseif doc.schema_version == 3 then
        if opts.validate ~= false then
          local errors = P.validate_doc(doc)
          for _, e in ipairs(errors) do err(st, f .. ": " .. e) end
          -- an invalid document is reported and left out (its fields cannot be trusted)
          if #errors > 0 then doc = nil end
        end
        if doc then docs[#docs + 1] = { path = f:gsub("^" .. root:gsub("%p", "%%%0") .. "[/\\]", ""), doc = doc } end
      end
    end
  end
  local roster_meta, roster = load_roster()

  local heroes, abilities, items, statuses, templates = {}, {}, {}, {}, {}
  for _, entry in ipairs(docs) do
    local doc = entry.doc
    local docowner = add_owner(st, "doc:" .. doc.id, "doc", doc, nil, { source = entry.path })
    local function add_statuses(list, parent)
      for _, s in ipairs(list or {}) do
        local so = add_owner(st, s.id, "status", s, parent)
        statuses[#statuses + 1] = so
      end
    end
    add_statuses(doc.statuses, docowner)
    for _, u in ipairs(doc.unit_templates or {}) do
      local uo = add_owner(st, u.id, "template", u, docowner)
      templates[#templates + 1] = uo
    end
    local hero
    if doc.kind == "hero" then
      hero = { id = doc.hero.source_id or doc.id, doc = doc.id, source = entry.path, def = doc.hero, abilities = {} }
      heroes[#heroes + 1] = hero
    end
    for _, a in ipairs(doc.abilities or {}) do
      local ao = add_owner(st, a.id, "ability", a, docowner, { hero = hero and hero.id })
      abilities[#abilities + 1] = ao
      if hero then hero.abilities[#hero.abilities + 1] = a.id end
      add_statuses(a.statuses, ao)
    end
    for _, it in ipairs(doc.items or {}) do
      local io_ = add_owner(st, it.id, "item", it, docowner)
      items[#items + 1] = io_
      add_statuses(it.statuses, io_)
    end
  end

  -- compile every value (so unused or hidden ones also get exact/raw)
  for _, o in ipairs(st.order) do
    if o.def.values then
      for _, n in ipairs(sorted_keys(o.def.values)) do
        local v, why = compile_value(st, o, n)
        if not v then err(st, o.id .. "." .. n .. ": " .. tostring(why)) end
      end
    end
  end

  local sup = opts.supplement
  local catalog = {
    format = "omoba-site-catalog", format_version = 1,
    producer = "census_adapter",
    schema_version = 3,
    documents = {},
    heroes = {}, abilities = {}, items = {}, statuses = {}, unit_templates = {}, values = {},
    rules = nil,
  }
  for _, e in ipairs(docs) do catalog.documents[#catalog.documents + 1] = { id = e.doc.id, kind = e.doc.kind, path = e.path } end

  local function text_of(o) return compile_text(st, o) end
  local function owner_values(o)
    local out = {}
    for _, n in ipairs(sorted_keys(o.values or {})) do
      local v = o.values[n]
      if v then out[#out + 1] = v end
    end
    return out
  end

  for _, h in ipairs(heroes) do
    local r = roster[h.id]
    if not r then warn(st, "hero " .. h.id .. " is not in the source roster; no unit stats") end
    table.sort(h.abilities, function(a, b)
      local sa, sb = st.owners[a].def.slot, st.owners[b].def.slot
      local oa, ob = M.SLOT_ORDER[sa] or 9, M.SLOT_ORDER[sb] or 9
      if oa ~= ob then return oa < ob end
      return a < b
    end)
    local text = {}
    for _, lang in ipairs(P.vocab.LANGS) do
      local lt = h.def.text and h.def.text[lang]
      if lt then text[lang] = { name = { { lit = lt.name } } } end
    end
    catalog.heroes[#catalog.heroes + 1] = {
      id = h.id, document = h.doc, text = text, abilities = h.abilities,
      stats = r and r.stats or nil, roster_name = r and r.name or nil,
      icon = sup and sup.hero_icon(h.id) or nil,
    }
  end

  for _, o in ipairs(abilities) do
    local a = o.def
    local st_ids = {}
    for _, s in ipairs(a.statuses or {}) do st_ids[#st_ids + 1] = s.id end
    catalog.abilities[#catalog.abilities + 1] = {
      id = a.id, hero = o.hero, source_id = a.source_id, slot = a.slot, kind = a.kind, tags = a.tags,
      max_level = a.max_level or 1, flags = a.flags, learnable = a.learnable, level_link = a.level_link,
      text = text_of(o), header = compile_header(st, o, a), statuses = st_ids,
      passive_stats = a.passive and stat_entries(st, o, a.passive.stats) or nil,
      coverage = a.coverage, confidence = a.confidence,
      icon = sup and sup.ability_icon(a.source_id) or nil,
    }
  end

  for _, o in ipairs(items) do
    local it = o.def
    local shop = it.shop or {}
    local st_ids = {}
    for _, s in ipairs(it.statuses or {}) do st_ids[#st_ids + 1] = s.id end
    local recipe
    if type(shop.recipe) == "table" then
      recipe = { components = shop.recipe.components or {}, recipe_cost = header_ref(st, o, shop.recipe.recipe_cost, "recipe_cost") }
    end
    local active
    if type(it.active) == "table" then
      active = { kind = it.active.kind, header = compile_header(st, o, it.active), flags = it.active.flags }
    end
    local category = shop.category or (sup and sup.shop_category(it.source_id)) or nil
    catalog.items[#catalog.items + 1] = {
      id = it.id, source_id = it.source_id, max_level = it.max_level or 1,
      text = text_of(o), statuses = st_ids,
      shop = {
        cost = header_ref(st, o, shop.cost, "cost"), recipe = recipe, category = category,
        categories = sup and sup.shop_categories(it.source_id) or nil,
        sellable = shop.sellable, droppable = shop.droppable, shareable = shop.shareable, unique = shop.unique,
        purchasable = shop.purchasable,
        charges = type(shop.consumable) == "table" and header_ref(st, o, shop.consumable.charges, "charges") or nil,
      },
      stats = it.passive and stat_entries(st, o, it.passive.stats) or {},
      has_passive = it.passive ~= nil and (it.passive.triggers ~= nil or it.passive.aura ~= nil or it.passive.states ~= nil) or nil,
      active = active,
      coverage = it.coverage, confidence = it.confidence,
      icon = sup and sup.item_icon(it.source_id) or nil,
    }
  end

  for _, o in ipairs(statuses) do
    local s = o.def
    catalog.statuses[#catalog.statuses + 1] = {
      id = s.id, owner = o.parent and o.parent.id or nil, duration = s.duration ~= "infinite" and header_ref(st, o, s.duration, "duration") or nil,
      infinite = s.duration == "infinite" or nil, dispel = s.dispel, visibility = s.visibility, states = s.states,
      text = text_of(o),
    }
  end
  for _, o in ipairs(templates) do
    catalog.unit_templates[#catalog.unit_templates + 1] = { id = o.def.id, text = text_of(o) }
  end

  -- values table: owner id -> list (sorted by name, the proposal's slot order)
  for _, o in ipairs(st.order) do
    if o.values and next(o.values) then catalog.values[o.id] = owner_values(o) end
  end

  -- rules referenced by {rules.*}
  local rules_chunk = loadfile(root .. "/scripts/lua_data/templates/game_rules.lua", "t", {})
  if rules_chunk then
    local okr, rules = pcall(function() return rules_chunk()({}) end)
    if okr then catalog.rules = rules end
  end

  catalog.roster = { repository = roster_meta.repository, commit = roster_meta.commit }
  catalog.shop_order = {}
  for _, c in ipairs(sup and sup.shop_order or {}) do catalog.shop_order[#catalog.shop_order + 1] = c end
  return catalog, st.errors, st.warnings
end

return M
