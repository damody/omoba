-- Package schema v3 compiler (M1): a validated v3 document -> the content blocks that
-- scripts/lua_data carries on each ability and item row (`v3 = { ... }`), plus the
-- v2-compatible pieces the current build still reads (`extras`, `description`).
-- Format: docs/character_pipeline/schema/v3-content-model-proposal.md §2.
--
--   compile(doc) -> { abilities = { [id] = block }, items = { [id] = block },
--                     extras = { [id] = { [name] = { per-rank numbers } } },
--                     description = { [id] = zh_TW body rendered with every rank },
--                     levels = { [id] = { cooldown, mana_cost, cast_time, range, required_hero_level } per rank } }
--
-- Numbers stay decimal strings from the author to here; raw Fixed64 (SCALE 1024) is
-- computed from the string with integer arithmetic, never through f32.
local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
local V = dofile(tools_dir .. "/v3_vocab.lua")
local X = dofile(tools_dir .. "/v3_expr.lua")
local BOT = dofile(tools_dir .. "/v3_bot.lua")
local M = {}

-- Item names of every item document in `dir` ({ [item id] = text }), for {@item_x} placeholders
-- that name an item of another document (v3.md §12.10).
function M.item_names(dir)
  local ok, lfs = pcall(require, "lfs")
  local names = {}
  if not ok or lfs.attributes(dir, "mode") ~= "directory" then return names end
  for file in lfs.dir(dir) do
    if file:match("%.lua$") then
      local chunk = loadfile(dir .. "/" .. file, "t", { pairs = pairs, ipairs = ipairs, string = string, table = table, math = math, utf8 = utf8 })
      local okc, v = pcall(chunk or error)
      if okc and type(v) == "function" then okc, v = pcall(v, {}) end
      if okc and type(v) == "table" then
        for _, item in ipairs(v.items or {}) do if item.id and item.text then names[item.id] = item.text end end
      end
    end
  end
  return names
end

M.SCALE = 1024
M.SCHEMA = V.SCHEMA_REVISION

local function sorted_keys(t)
  local ks = {}
  for k in pairs(t) do ks[#ks + 1] = k end
  table.sort(ks, function(a, b) return tostring(a) < tostring(b) end)
  return ks
end

-- Shortest decimal text that reads back as the same double.
function M.number_text(v)
  if math.type(v) == "integer" then return tostring(v) end
  if v == math.floor(v) and math.abs(v) < 2 ^ 53 then return string.format("%d", v) end
  for p = 1, 17 do
    local s = string.format("%." .. p .. "g", v)
    if tonumber(s) == v then
      if s:find("e") then s = string.format("%.17f", v):gsub("0+$", "") end
      return s
    end
  end
  return string.format("%.17g", v)
end

-- "12.5" / "-0.53" / "19/101" -> numerator, denominator (integers)
function M.rational(text)
  local sign, a, b = text:match("^(%-?)(%d+)/(%d+)$")
  if sign then
    local n, d = math.tointeger(tonumber(a)), math.tointeger(tonumber(b))
    assert(n and d and d > 0, "bad fraction " .. text)
    return sign == "-" and -n or n, d
  end
  local s, int, frac = text:match("^(%-?)(%d+)%.?(%d*)$")
  assert(s, "bad decimal " .. text)
  local n = math.tointeger(tonumber(int .. frac))
  local d = math.tointeger(10 ^ #frac)
  assert(n and d, "decimal out of range " .. text)
  return s == "-" and -n or n, d
end

-- Fixed64 raw of a decimal string: nearest (half away from zero) or truncate toward zero.
-- Long decimals (more digits than an int64 holds): multiply the digit string by SCALE exactly.
local function raw_of_long_decimal(text, rounding)
  local sign, int, frac = text:match("^(%-?)(%d+)%.(%d+)$")
  local digits = {}
  for c in (int .. frac):gmatch("%d") do digits[#digits + 1] = tonumber(c) end
  -- digits * SCALE, little-endian carry
  local carry = 0
  for i = #digits, 1, -1 do
    local v = digits[i] * M.SCALE + carry
    digits[i] = v % 10
    carry = v // 10
  end
  local head = carry -- digits above the original length
  local whole_len = #int
  local q = head
  for i = 1, whole_len do q = q * 10 + digits[i] end
  local first_frac = digits[whole_len + 1] or 0
  local rest = false
  for i = whole_len + 2, #digits do if digits[i] ~= 0 then rest = true end end
  if rounding ~= "truncate" and first_frac >= 5 then q = q + 1 end -- half away from zero, like v3.rs raw_of
  return sign == "-" and -q or q
end

function M.raw_of(text, rounding)
  if text:match("^%-?%d+%.%d+$") and #text:gsub("[%-%.]", "") > 15 then return raw_of_long_decimal(text, rounding) end
  local n, d = M.rational(text)
  local scaled = n * M.SCALE
  local q = scaled // d
  local r = scaled - q * d -- 0 <= r < d (floor division)
  if scaled < 0 and r ~= 0 then q, r = q + 1, r - d end -- truncate toward zero; r now in (-d, 0]
  if rounding == "truncate" then return q end
  if 2 * math.abs(r) >= d then q = q + (scaled < 0 and -1 or 1) end
  return q
end

function M.value_of(text)
  if text:match("^%-?%d+%.%d+$") and #text:gsub("[%-%.]", "") > 15 then return tonumber(text) end
  local n, d = M.rational(text)
  return n / d
end

local function rank_texts(def, count)
  local out = {}
  for i = 1, count do
    local x = def[i] or def[1]
    out[i] = type(x) == "string" and x or M.number_text(x)
  end
  return out
end

-- Display text of a number (at most two decimals, trailing zeros dropped).
local function show(v)
  if math.abs(v - math.floor(v + 0.5)) < 1e-9 then return string.format("%d", math.floor(v + 0.5)) end
  local s = string.format("%.2f", v):gsub("0+$", ""):gsub("%.$", "")
  return s
end

-- opts.names: { [id] = text } for {@id} names outside the document (items of other documents)
function M.compile(doc, opts)
  opts = opts or {}
  -- M3' ④: no behavior IR in the catalog (generated Rust runs the behavior, v3_rust.lua); the
  -- castable's flags go out as `ability_flags` for the host's rule layer.
  local out = { abilities = {}, items = {}, extras = {}, description = {}, levels = {} }
  local owners, names, statuses = {}, {}, {}
  local shared = { id = "shared", values = doc.values or {}, max_level = 1 }
  owners.shared = shared
  local function add(def, kind, parent)
    local o = { id = def.id, kind = kind, def = def, values = def.values or {}, max_level = def.max_level or (parent and parent.max_level) or 1, parent = parent or shared }
    owners[def.id] = o
    if def.text then names[def.id] = def.text end
    return o
  end
  local entries = {}
  for _, a in ipairs(doc.abilities or {}) do
    local o = add(a, "ability")
    entries[#entries + 1] = o
    for _, s in ipairs(a.statuses or {}) do statuses[s.id] = add(s, "status", o) end
  end
  for _, it in ipairs(doc.items or {}) do
    local o = add(it, "item")
    entries[#entries + 1] = o
    for _, s in ipairs(it.statuses or {}) do statuses[s.id] = add(s, "status", o) end
  end
  for _, u in ipairs(doc.unit_templates or {}) do add(u, "template") end
  for _, s in ipairs(doc.statuses or {}) do statuses[s.id] = add(s, "status", shared) end

  local function find(owner, qual, name)
    if qual then
      local o = owners[qual]
      if o and o.values[name] ~= nil then return o.values[name], o end
      return nil
    end
    local o = owner
    while o do
      if o.values[name] ~= nil then return o.values[name], o end
      o = o.parent
    end
  end

  -- per-rank numbers (display units) of every static value; nil for runtime expressions
  local cache = {}
  local evaluating = {}
  local rules -- loaded lazily from the validator's loader
  local function rules_value(path)
    if not rules then
      PACKAGE_V3_AS_MODULE = true
      rules = dofile(tools_dir .. "/package_v3.lua").rules()
      PACKAGE_V3_AS_MODULE = nil
    end
    local t = rules
    for part in path:gmatch("[^.]+") do t = t[part] end
    return t
  end
  local ranks_of
  local function eval(ast, owner, rank)
    local k = ast.k
    if k == "num" then return ast.v end
    if k == "val" then
      local d, o = find(owner, ast.owner, ast.name)
      local r = ranks_of(o, ast.name)
      if not r then return nil end
      local v = r.numbers[math.min(rank, #r.numbers)]
      if d.unit == "pct" then v = v / 100 end
      return v
    end
    if k == "field" then
      local parts, n = {}, ast
      while n.k == "field" do table.insert(parts, 1, n.name); n = n.obj end
      if n.k == "id" and n.name == "rules" then return rules_value(table.concat(parts, ".")) end
      return nil
    end
    if k == "un" and ast.op == "-" then local v = eval(ast.e, owner, rank); return v and -v end
    if k == "bin" then
      local l, r = eval(ast.l, owner, rank), eval(ast.r, owner, rank)
      if l == nil or r == nil then return nil end
      local op = ast.op
      if op == "+" then return l + r elseif op == "-" then return l - r elseif op == "*" then return l * r
      elseif op == "/" then return l / r elseif op == "%" then return l % r end
      return nil
    end
    if k == "call" then
      local args = {}
      for i, a in ipairs(ast.args) do args[i] = eval(a, owner, rank); if args[i] == nil then return nil end end
      local f = ({ min = math.min, max = math.max, abs = math.abs, floor = math.floor, ceil = math.ceil,
        sqrt = math.sqrt, clamp = function(x, a, b) return math.max(a, math.min(b, x)) end,
        round = function(x) return math.floor(x + 0.5) end })[ast.fn]
      return f and f(table.unpack(args)) or nil
    end
    return nil
  end
  ranks_of = function(o, name)
    local key = o.id .. "." .. name
    if cache[key] ~= nil then return cache[key] or nil end
    local d = o.values[name]
    local count = (d.per == "hero_level") and 25 or o.max_level
    local res
    if d.expr then
      assert(not evaluating[key], "derived value cycle at " .. key)
      evaluating[key] = true
      local ast = assert(X.parse(d.expr))
      local nums = {}
      for i = 1, count do
        local v = eval(ast, o, i)
        if v == nil then nums = nil; break end
        if d.unit == "pct" then v = v * 100 end
        nums[i] = v
      end
      evaluating[key] = nil
      if nums then
        local texts = {}
        for i, v in ipairs(nums) do
          -- derived values become decimal text (nine significant digits), then raw like any value
          local t = M.number_text(tonumber(string.format("%.9g", v)))
          texts[i] = t
          nums[i] = M.value_of(t)
        end
        res = { texts = texts, numbers = nums }
      end
    else
      local texts = rank_texts(d, count)
      local nums = {}
      for i, t in ipairs(texts) do nums[i] = M.value_of(t) end
      res = { texts = texts, numbers = nums }
    end
    cache[key] = res or false
    return res
  end

  local function value_rows(o)
    local rows = {}
    for _, n in ipairs(sorted_keys(o.values)) do
      local d = o.values[n]
      local row = { name = n, unit = d.unit, hidden = d.hidden or nil, damage_type = d.damage_type,
        per = d.per, v2_key = d.v2_key }
      local r = ranks_of(o, n)
      if r then
        row.ranks = r.texts
        row.raw = {}
        for i, t in ipairs(r.texts) do row.raw[i] = M.raw_of(t, d.rounding) end
      else
        row.expr = d.expr
      end
      if d.expr then row.derived = true end
      rows[#rows + 1] = row
    end
    return rows
  end

  -- `$name` (or `$owner.name`) -> value reference; anything else -> expression
  local function ref_of(expr)
    if type(expr) ~= "string" then return nil end
    local q, n = expr:match("^%s*%$([%w_]+)%.([%w_]+)%s*$")
    if not q then n = expr:match("^%s*%$([%w_]+)%s*$") end
    if n then return { value = n, owner = q } end
    return { expr = expr }
  end
  local INFO = {
    { "cast_range", { "target", "range" } }, { "aoe_radius", { "target", "aoe_radius" } },
    { "cast_point", { "cast", "cast_point" } }, { "channel_time", { "cast", "channel_time" } },
    { "mana_cost", { "cost", "mana" } }, { "health_cost", { "cost", "health" } }, { "gold_cost", { "cost", "gold" } },
    { "cooldown", { "cost", "cooldown" } }, { "charges_max", { "cost", "charges", "max" } },
    { "charge_restore", { "cost", "charges", "restore" } }, { "required_level", { "requires", "hero_level" } },
  }
  local function info_of(castable, shop)
    local rows = {}
    local function dig(t, path)
      for _, p in ipairs(path) do if type(t) ~= "table" then return nil end t = t[p] end
      return t
    end
    if shop then
      for _, f in ipairs({ { "cost", { "cost" } }, { "recipe_cost", { "recipe", "recipe_cost" } } }) do
        local r = ref_of(dig(shop, f[2]))
        if r then r.field = f[1]; rows[#rows + 1] = r end
      end
    end
    if castable then
      for _, f in ipairs(INFO) do
        local r = ref_of(dig(castable, f[2]))
        if r then r.field = f[1]; rows[#rows + 1] = r end
      end
    end
    return rows
  end

  local function value_display(o, name, fmt)
    local d, vo = find(o, nil, name)
    local r = ranks_of(vo, name)
    if not r then return "?" end
    local parts, seen_all_same = {}, true
    for i, v in ipairs(r.numbers) do
      if v ~= r.numbers[1] then seen_all_same = false end
      local x = v
      if fmt == "abs" then x = math.abs(x) end
      if fmt == "int" then x = math.floor(x + 0.5) end
      local s = show(x)
      if fmt == "signed" and x >= 0 then s = "+" .. s end
      parts[i] = s
    end
    local suffix = d.unit == "pct" and "%" or ""
    if fmt == "max" then return parts[#parts] .. suffix end
    if seen_all_same then return parts[1] .. suffix end
    return table.concat(parts, "/") .. suffix
  end

  local function pieces_of(template, o, lang)
    local pieces, rendered = {}, {}
    local pos = 1
    while true do
      local s, e, ph = template:find("{([^{}]*)}", pos)
      local lit = template:sub(pos, (s or #template + 1) - 1)
      if lit ~= "" then pieces[#pieces + 1] = { text = lit }; rendered[#rendered + 1] = lit end
      if not s then break end
      local name, fmt = ph:match("^([^:]+):?(.*)$")
      fmt = fmt ~= "" and fmt or nil
      if name:sub(1, 1) == "@" then
        local id = name:sub(2)
        local t = names[id] or (opts.names and opts.names[id])
        assert(t, "{@" .. id .. "} has no text (an item of another document needs the build's item names)")
        local shown = t[lang] and t[lang].name or t.zh_TW.name
        pieces[#pieces + 1] = { name_of = id, text = shown }
        rendered[#rendered + 1] = shown
      elseif name:match("^rules%.") then
        local v = rules_value(name:sub(7))
        local shown = show(v)
        pieces[#pieces + 1] = { rules = name:sub(7), text = shown }
        rendered[#rendered + 1] = shown
      else
        local q, vn = name:match("^([%w_]+)%.([%w_]+)$")
        if not q then vn = name end
        local _, vo = find(o, q, vn)
        assert(vo, "unknown placeholder {" .. name .. "}")
        pieces[#pieces + 1] = { value = vn, owner = vo ~= o and vo.id or nil, format = fmt }
        rendered[#rendered + 1] = value_display(vo, vn, fmt)
      end
      pos = e + 1
    end
    return pieces, table.concat(rendered)
  end

  local function text_of(o)
    local t = o.def.text
    if not t then return nil, nil end
    local res, description = {}, nil
    for _, lang in ipairs(V.LANGS) do
      local lt = t[lang]
      if lt then
        local row = { name = lt.name }
        if lt.body then
          row.body = lt.body
          local pieces, rendered = pieces_of(lt.body, o, lang)
          row.pieces = pieces
          row.rendered = rendered
          if lang == "zh_TW" then description = rendered end
        end
        res[lang] = row
      end
    end
    return res, description
  end

  for _, o in ipairs(entries) do
    local def = o.def
    local castable = o.kind == "ability" and def or def.active
    local block = { schema = M.SCHEMA, source_id = def.source_id, max_level = o.max_level, kind = o.kind,
      values = value_rows(o), info = info_of(castable, o.kind == "item" and def.shop or nil),
      coverage = def.coverage, handler = def.handler and def.handler.module or nil,
      charges_on_rank_up = castable and castable.cost and castable.cost.charges and castable.cost.charges.on_rank_up or nil }
    -- bot cast policy (v3.3, v3_bot.lua): authored or derived; a `*_key` must name a value with
    -- per-rank numbers (the bot reads it from the compiled extras)
    if castable or o.kind == "ability" then
      local policy = BOT.export(def, o.kind)
      local spec = BOT.KINDS[policy.kind]
      for k, v in pairs(policy) do
        if spec and spec.params[k] == "value" then
          local d, vo = find(o, nil, v)
          if not d or not ranks_of(vo, v) then policy = { kind = "none", reason = "bot " .. k .. " `" .. tostring(v) .. "` has no per-rank numbers", source = policy.source } break end
        end
      end
      block.bot = policy
    end
    if o.kind == "item" and def.passive and def.passive.stats then
      block.stat_lines = {}
      for _, stat in ipairs(sorted_keys(def.passive.stats)) do
        local v = def.passive.stats[stat]
        local r = ref_of(type(v) == "table" and v.value or v)
        if r then r.stat = stat; block.stat_lines[#block.stat_lines + 1] = r end
      end
    end
    -- statuses this entry owns: their names and own values, for tooltips and buff icons
    local st = {}
    for _, s in ipairs(def.statuses or {}) do
      local so = owners[s.id]
      -- dispel: the class the host's dispel reads (E214)
      local row = { id = s.id, visibility = s.visibility, dispel = s.dispel, reveal = s.reveal, runtime_id = s.runtime_id, values = next(so.values) and value_rows(so) or nil }
      local stext = text_of(so)
      if stext then row.text = stext end
      st[#st + 1] = row
    end
    if #st > 0 then block.statuses = st end
    local text, description = text_of(o)
    block.text = text
    -- An empty Lua table has no JSON shape (it reads back as a map): leave empty lists out.
    for _, k in ipairs({ "values", "info", "stat_lines", "statuses" }) do
      if block[k] and #block[k] == 0 then block[k] = nil end
    end
    for _, lt in pairs(block.text or {}) do if lt.pieces and #lt.pieces == 0 then lt.pieces = nil end end
    local castable = o.kind == "ability" and def or def.active
    if castable and type(castable.flags) == "table" and #castable.flags > 0 then
      block.ability_flags = { table.unpack(castable.flags) }
    end
    if o.kind == "ability" then out.abilities[def.id] = block else out.items[def.id] = block end
    out.description[def.id] = description
    -- v2-compatible extras: every static value, by its v3 name (and its v2 alias)
    local ex = {}
    for _, row in ipairs(block.values or {}) do
      if row.ranks and row.per ~= "hero_level" then
        local nums = {}
        for i, t in ipairs(row.ranks) do nums[i] = M.value_of(t) end
        ex[row.name] = nums
      end
    end
    out.extras[def.id] = ex
    -- per-rank level numbers from the info line
    if castable then
      local function nums(field)
        for _, r in ipairs(block.info or {}) do
          if r.field == field and r.value then
            local _, vo = find(o, r.owner, r.value)
            local rr = ranks_of(vo, r.value)
            return rr and rr.numbers
          end
        end
      end
      local lv = {}
      local cd, mana, cp, range, req = nums("cooldown"), nums("mana_cost"), nums("cast_point"), nums("cast_range"), nums("required_level")
      for i = 1, o.max_level do
        local function at(t) return t and (t[i] or t[#t]) or nil end
        lv[i] = { cooldown = at(cd), mana_cost = at(mana), cast_time = at(cp), range = at(range),
          required_hero_level = req and math.tointeger(at(req)) or nil }
      end
      out.levels[def.id] = lv
    end
  end
  -- values outside the entries (the document's `values`, unit templates, document-level statuses),
  -- for catalogs that resolve every text piece: { [owner id] = { kind, values = rows } }. The game
  -- content does not read it.
  out.owner_values = {}
  local function export(id, kind, o)
    if o and next(o.values) then
      local rows = value_rows(o)
      if #rows > 0 then out.owner_values[id] = { kind = kind, values = rows } end
    end
  end
  export("shared", "shared", shared)
  for _, u in ipairs(doc.unit_templates or {}) do export(u.id, "template", owners[u.id]) end
  for _, s in ipairs(doc.statuses or {}) do export(s.id, "status", owners[s.id]) end
  return out
end

return M
