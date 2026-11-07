-- Text template renderer (schema v3 §9, value-single-source §3.1).
--
-- parse(template) turns a description template into pieces (the proposal's
-- TextPiece): { lit = "..." } | { value = name, owner = qualifier|nil, fmt = fmt }
-- | { rules = "a.b" } | { name_of = id }. render(pieces, ctx) substitutes values
-- and returns runs: { t = text, b = true|nil (current rank), dt = damage type|nil },
-- adjacent runs with the same style merged. The website turns runs into HTML; UE's
-- TooltipRenderer must produce the same runs for the same input. Every rule here
-- is pinned by docs/character_pipeline/schema/v3-text-render-vectors.lua.
--
-- Display rules
--   * number: the exact value, rounded half away from zero to two decimals,
--     trailing zeros dropped, never "-0". pct values are percent numbers (25 is
--     25%); one "%" follows the whole placeholder ("5/6/8/11%").
--   * all (default): every rank, "/" between; when all ranks show the same text,
--     one value. With ctx.rank, that rank (clamped to the list) is bold.
--   * current: only ctx.rank's value (clamped); without a rank, same as all.
--   * max: the last rank.  abs: absolute values.  int: rounded to an integer
--     first.  signed: "+" before values above zero.
--   * per = "hero_level" values follow ctx.hero_level instead of ctx.rank.
--   * a damage-typed value marks the whole placeholder (separators too) with dt.
--   * {rules.a.b} is a plain number; {@id} is that id's name in the same language.
local Q = require("site.rational")
local M = {}

M.FORMATS = { all = true, current = true, max = true, abs = true, int = true, signed = true }
M.DECIMALS = 2

function M.parse(template)
  assert(type(template) == "string", "template must be a string")
  local pieces, i = {}, 1
  local function lit(s) if s ~= "" then pieces[#pieces + 1] = { lit = s } end end
  while true do
    local s, e = template:find("%b{}", i)
    if not s then lit(template:sub(i)); break end
    lit(template:sub(i, s - 1))
    local ph = template:sub(s + 1, e - 1)
    if ph:find("[{}]") then error("nested braces in placeholder {" .. ph .. "}") end
    local name, fmt = ph:match("^([^:]+):?(.*)$")
    if not name then error("empty placeholder {}") end
    if name:sub(1, 1) == "@" then
      if fmt ~= "" then error("{" .. ph .. "}: a name reference takes no format") end
      pieces[#pieces + 1] = { name_of = name:sub(2) }
    elseif name:match("^rules%.") then
      pieces[#pieces + 1] = { rules = name:sub(7), fmt = fmt ~= "" and fmt or nil }
    else
      local owner, vn = name:match("^([%w_]+)%.([%w_]+)$")
      if not owner then vn = name end
      if not vn:match("^[%w_]+$") then error("bad placeholder {" .. ph .. "}") end
      if fmt ~= "" and not M.FORMATS[fmt] then error("unknown format :" .. fmt .. " in {" .. ph .. "}") end
      pieces[#pieces + 1] = { value = vn, owner = owner, fmt = fmt ~= "" and fmt or "all" }
    end
    i = e + 1
  end
  if template:sub(i):find("[{}]") then error("unbalanced brace in template") end
  for _, p in ipairs(pieces) do
    if p.lit and p.lit:find("[{}]") then error("unbalanced brace in template") end
  end
  return pieces
end

-- one rank's text (the unit suffix is added once per placeholder by value_runs)
function M.number(q, unit, fmt)
  if fmt == "abs" then q = Q.abs(q) end
  if fmt == "int" then q = Q.new(Q.round_int(q)) end
  local s = Q.decimal(q, M.DECIMALS)
  if fmt == "signed" and s:sub(1, 1) ~= "-" and s ~= "0" then s = "+" .. s end
  return s
end

M.SUFFIX = { pct = "%" }

-- value = { exact = { rational|string|number, ... }, unit, damage_type?, per? }
-- returns runs for one placeholder
function M.value_runs(value, fmt, ctx)
  fmt = fmt or "all"
  local list = value.exact
  assert(type(list) == "table" and #list > 0, "value has no ranks")
  local n = #list
  local texts = {}
  for i = 1, n do texts[i] = M.number(Q.parse(list[i]), value.unit, fmt) end
  local rank = ctx.rank
  if value.per == "hero_level" then rank = ctx.hero_level end
  local dt = value.damage_type
  local suffix = M.SUFFIX[value.unit]
  local function done(runs)
    if suffix then runs[#runs + 1] = { t = suffix, dt = dt } end
    return runs
  end
  if fmt == "max" then return done({ { t = texts[n], dt = dt } }) end
  local same = true
  for i = 2, n do if texts[i] ~= texts[1] then same = false end end
  if same then return done({ { t = texts[1], dt = dt } }) end
  local cur = rank and math.max(1, math.min(n, rank)) or nil
  if fmt == "current" and cur then return done({ { t = texts[cur], dt = dt } }) end
  local runs = {}
  for i = 1, n do
    if i > 1 then runs[#runs + 1] = { t = "/", dt = dt } end
    runs[#runs + 1] = { t = texts[i], b = (i == cur) or nil, dt = dt }
  end
  return done(runs)
end

local function same_style(a, b) return a.b == b.b and a.dt == b.dt end

function M.merge(runs)
  local out = {}
  for _, r in ipairs(runs) do
    if r.t ~= "" then
      local last = out[#out]
      if last and same_style(last, r) then last.t = last.t .. r.t
      else out[#out + 1] = { t = r.t, b = r.b, dt = r.dt } end
    end
  end
  return out
end

-- ctx: rank, hero_level, value(piece) -> value table, rule(path) -> number, name(id) -> string
function M.render(pieces, ctx)
  local runs = {}
  for _, p in ipairs(pieces) do
    if p.lit then runs[#runs + 1] = { t = p.lit }
    elseif p.name_of then
      local s = ctx.name(p.name_of)
      if not s then error("{@" .. p.name_of .. "} has no name") end
      runs[#runs + 1] = { t = s }
    elseif p.rules then
      local x = ctx.rule(p.rules)
      if x == nil then error("{rules." .. p.rules .. "} is not a rules value") end
      runs[#runs + 1] = { t = M.number(Q.parse(x), nil, p.fmt) }
    else
      local v = ctx.value(p)
      if not v then error("{" .. (p.owner and (p.owner .. ".") or "") .. p.value .. "} has no value") end
      for _, r in ipairs(M.value_runs(v, p.fmt, ctx)) do runs[#runs + 1] = r end
    end
  end
  return M.merge(runs)
end

function M.plain(runs)
  local out = {}
  for i, r in ipairs(runs) do out[i] = r.t end
  return table.concat(out)
end

local ESC = { ["&"] = "&amp;", ["<"] = "&lt;", [">"] = "&gt;", ['"'] = "&quot;" }
function M.escape(s) return (s:gsub('[&<>"]', ESC)) end

function M.html(runs)
  local out = {}
  for _, r in ipairs(runs) do
    local s = M.escape(r.t):gsub("\n", "<br>")
    if r.b then s = "<b>" .. s .. "</b>" end
    if r.dt then s = '<span class="dt dt-' .. r.dt .. '">' .. s .. "</span>" end
    out[#out + 1] = s
  end
  return table.concat(out)
end

-- Convenience for flat inputs (the shared vectors): values keyed by name, other
-- owners' values keyed owners[owner][name], rules as a nested table, names by id.
function M.render_template(template, input)
  local pieces = M.parse(template)
  local function rule(path)
    local cur = input.rules
    for part in path:gmatch("[^.]+") do cur = type(cur) == "table" and cur[part] or nil end
    if type(cur) == "table" then return nil end
    return cur
  end
  local function value(p)
    local src = p.owner and (input.owners or {})[p.owner] or input.values
    local v = src and src[p.value]
    if not v then return nil end
    local exact = {}
    for i, x in ipairs(v) do exact[i] = x end
    return { exact = exact, unit = v.unit, damage_type = v.damage_type, per = v.per }
  end
  return M.render(pieces, {
    rank = input.rank, hero_level = input.hero_level, value = value, rule = rule,
    name = function(id) return (input.names or {})[id] end,
  })
end

return M
