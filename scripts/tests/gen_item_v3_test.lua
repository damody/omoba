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

local _, problems, shops = G.build(shelf, { { path = 'batch_I1.lua', doc = doc } }, true)
ok(#problems == 0, table.concat(problems, '\n'))
ok(shops.katana.numbers.cost == 1120 and math.type(shops.katana.numbers.cost) == 'integer', 'the price is the v3 cost, an integer')
ok(shops.katana.numbers.atk == 14, 'the attack bonus is the v3 value')

-- a v3 number that differs from the source catalog is an error unless it says so
katana(doc).values.cost[1] = 1200
_, problems = G.build(shelf, { { path = 'batch_I1.lua', doc = doc } }, true)
local found = false
for _, p in ipairs(problems) do if p:find('item_katana', 1, true) and p:find('source_differs', 1, true) then found = true end end
ok(found, 'an undeclared difference from the source catalog fails')
katana(doc).values.cost.source_differs = 'balance test'
_, problems, shops = G.build(shelf, { { path = 'batch_I1.lua', doc = doc } }, true)
ok(#problems == 0, table.concat(problems, '\n'))
ok(shops.katana.numbers.cost == 1200, 'the declared v3 price reaches the shelf')

-- the final shelf carries the generated numbers
local final = cb.new(b.root .. '/scripts/lua_data').include('templates/moba_items.lua')
local generated = cb.new(b.root .. '/scripts/lua_data').include('templates/item_shop_v3.lua')
for _, item in ipairs(final) do
  local s = generated[item.id]
  if s then ok(item.cost == s.numbers.cost, item.id .. ' costs what v3 says') end
end
print('gen_item_v3 test: ' .. checks .. ' checks passed')
