local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')

-- The shelf's prices, recipes and stats come from the v3 item documents (value-single-source §3.3):
-- gen_item_v3 resolves them into templates/item_shop_v3.lua, and a v3 number that differs from the
-- source catalog needs `source_differs`.
GEN_ITEM_V3_AS_MODULE = true
local G = dofile(b.root .. '/scripts/gen_item_v3.lua')
GEN_ITEM_V3_AS_MODULE = nil
local cb = require('content_builder')
local ctx = cb.new(b.root .. '/scripts/lua_data')
ctx.source_shelf = true
local shelf = ctx.include('templates/moba_items.lua')
local function load_doc(path)
  local env = { pairs = pairs, ipairs = ipairs, string = string, table = table, math = math, utf8 = utf8 }
  local v = assert(loadfile(path, 't', env))()
  if type(v) == 'function' then v = v({}) end
  return v
end
local doc = load_doc(b.root .. '/docs/character_pipeline/items/batch_I1.lua')
local function katana(d) for _, it in ipairs(d.items) do if it.id == 'item_katana' then return it end end end
local checks = 0
local function ok(v, m) assert(v, m); checks = checks + 1 end

local _, problems, shops = G.build(shelf, { { path = 'batch_I1.lua', doc = doc } }, true, ctx.scripted_items)
ok(#problems == 0, table.concat(problems, '\n'))
ok(shops.katana.numbers.cost == 1120 and math.type(shops.katana.numbers.cost) == 'integer', 'the price is the v3 cost, an integer')
ok(shops.katana.numbers.atk == 14, 'the attack bonus is the v3 value')

-- a v3 number that differs from the source catalog is an error unless it says so
katana(doc).values.cost[1] = 1200
_, problems = G.build(shelf, { { path = 'batch_I1.lua', doc = doc } }, true, ctx.scripted_items)
local found = false
for _, p in ipairs(problems) do if p:find('item_katana', 1, true) and p:find('source_differs', 1, true) then found = true end end
ok(found, 'an undeclared difference from the source catalog fails')
katana(doc).values.cost.source_differs = 'balance test'
_, problems, shops = G.build(shelf, { { path = 'batch_I1.lua', doc = doc } }, true, ctx.scripted_items)
ok(#problems == 0, table.concat(problems, '\n'))
ok(shops.katana.numbers.cost == 1200, 'the declared v3 price reaches the shelf')

-- the final shelf carries the generated numbers
local final = cb.new(b.root .. '/scripts/lua_data').include('templates/moba_items.lua')
local generated = cb.new(b.root .. '/scripts/lua_data').include('templates/item_shop_v3.lua')
for _, item in ipairs(final) do
  local s = generated[item.id]
  if s then ok(item.cost == s.numbers.cost, item.id .. ' costs what v3 says') end
end
-- unit_or_point (巨龍): a named unit is checked as a unit target, else a point; it needs a range
do
  local num = function(v) return v end
  local fields = G.scripted_active({ id = 'item_x', active = { target = { type = 'unit_or_point', range = 600, team = 'both' }, cost = { cooldown = 30 } } }, num, {})
  ok(fields and fields.target.shape == 'unit_or_point' and fields.target.range == 600 and fields.target.team == 'any'
    and fields.target.out_of_range == 'reject', 'a unit_or_point active keeps its range, both teams read as any')
  local none, why = G.scripted_active({ id = 'item_x', active = { target = { type = 'unit_or_point', team = 'both' }, cost = {} } }, num, {})
  ok(none == nil and why:find('range'), 'a unit_or_point active without a range has no catalog target')
end
-- every item marked scripted has its generated script: without one its effects are simply missing
do
  V3_CODEGEN_COVERAGE_AS_MODULE = true
  local coverage = dofile(b.root .. '/docs/character_pipeline/tools/v3_codegen_coverage.lua')
  V3_CODEGEN_COVERAGE_AS_MODULE = nil
  local entries = {}
  for _, e in ipairs(coverage.scan(b.root).items) do entries[e.id] = e end
  for id in pairs(ctx.scripted_items) do
    local e = entries['item_' .. id]
    ok(e and #e.blockers == 0, id .. ' is marked scripted and has no script: ' .. table.concat(e and e.blockers or {}, '; '))
  end
end
-- recharging uses go to the catalog row (line B 1f8fecee); scope hero only, max 1..10
do
  local by = { charges_max = 2, charge_restore = 40 }
  local function number(e) if type(e) == 'number' then return e end; local n = type(e) == 'string' and e:match('^%$([%w_]+)$'); return n and by[n] end
  local item = { id = 'item_x', active = { kind = 'active', target = { type = 'none' },
    cost = { mana = 100, charges = { max = '$charges_max', restore = '$charge_restore', initial = '$charges_max', scope = 'hero' } } } }
  local out = assert(G.scripted_active(item, number, {}))
  ok(out.charges.max == 2 and math.type(out.charges.max) == 'integer' and out.charges.initial == 2 and out.charges.restore == 40, 'charges reach the row')
  item.active.cost.charges.scope = 'ability'
  ok(G.scripted_active(item, number, {}) == nil, 'only scope hero')
  item.active.cost.charges.scope, item.active.cost.charges.max = 'hero', 11
  ok(G.scripted_active(item, number, {}) == nil, 'at most 10')
end

-- the windup goes to the catalog row (line B 2be4ab76): none at 0, interruptible only with one
do
  local function number(e) if type(e) == 'number' then return e end; return e == '$cast_point' and 0.3 or nil end
  local item = { id = 'item_x', active = { kind = 'active', target = { type = 'none' }, cast = { cast_point = '$cast_point', interruptible = true }, cost = {} } }
  local out = assert(G.scripted_active(item, number, {}))
  ok(out.cast and out.cast.point == 0.3 and out.cast.interruptible == true, 'cast_point and interruptible reach the row')
  item.active.cast = { immediate = true }
  ok(assert(G.scripted_active(item, number, {})).cast == nil, 'no windup, no cast')
  item.active.cast = { interruptible = true }
  ok(G.scripted_active(item, number, {}) == nil, 'interruptible needs a cast_point')
end

print('gen_item_v3 test: ' .. checks .. ' checks passed')
