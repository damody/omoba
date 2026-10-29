local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local path = b.lib('path')

-- E155: the build-time check keeps every shop item equal to its history_nobu row.
-- OM_ITEM_TEMPLATES points at a workspace copy of scripts/lua_data/templates before landing.
local root = os.getenv('OM_ITEM_TEMPLATES') or path.join(b.root, 'scripts', 'lua_data', 'templates')
local ctx
ctx = {include = function(rel) return dofile(path.join(root, rel:match('templates/(.+)')))(ctx) end}
local source = ctx.include('templates/source_items.lua')
local check = ctx.include('templates/item_sources_check.lua').check
-- the shelf as the source shapes it: the final shelf takes its numbers and texts from the v3 documents
ctx.source_shelf = true
local items = ctx.include('templates/moba_items.lua')
ctx.source_shelf = nil
local checks = 0
local function ok(value, message) assert(value, message); checks = checks + 1 end
local function copy(t) local c = {} for k, v in pairs(t) do c[k] = type(v) == 'table' and copy(v) or v end return c end
local function rejects(mutate, pattern)
  local list = copy(items)
  mutate(list)
  local passed, err = pcall(check, list, source)
  ok(not passed and tostring(err):find(pattern, 1, true), 'expected "' .. pattern .. '", got ' .. tostring(err))
end
local function find(list, id) for _, item in ipairs(list) do if item.id == id then return item end end end

ok(check(items, source), 'the real catalog follows the source')
local shop, fixtures = 0, 0
for _, item in ipairs(items) do
  if item.test_only then fixtures = fixtures + 1 else shop = shop + 1; ok(item.source_ref, item.id .. ' cites a source') end
end
ok(shop > 0 and fixtures > 0, 'both shop items and engine fixtures exist')

rejects(function(l) find(l, 'spear').name = '自創長槍' end, 'is not the source name')
rejects(function(l) find(l, 'spear').cost = 650 end, 'is not the source cost')
rejects(function(l) find(l, 'spear').atk = 15 end, 'atk 15 is not the source 14')
rejects(function(l) find(l, 'spear').hp = 50 end, 'hp 50 is not the source 0')
rejects(function(l) find(l, 'spear').description = '攻擊+14' end, 'description differs')
rejects(function(l) find(l, 'spear').recipe = {'power_ingot'} end, 'is not the source recipe')
-- Passive stat modifiers (E155 C) are the source's exactly.
if find(items, 'katana') then
  rejects(function(l) find(l, 'katana').modifiers.attackspeed_bonus_constant = 20 end, 'modifier attackspeed_bonus_constant 20')
  rejects(function(l) find(l, 'katana').modifiers = nil end, 'is not the source 15')
  rejects(function(l) find(l, 'spear').modifiers = {mana_bonus = 100} end, 'modifier mana_bonus 100 is not the source 0')
end
rejects(function(l) find(l, 'moba_sword').test_only = nil end, 'needs source_ref')
rejects(function(l) find(l, 'moba_sword').source_ref = 'item_spear' end, 'cannot cite a source')
rejects(function(l) find(l, 'spear').source_ref = 'item_not_in_source' end, 'is not a shop item of the source')
-- A source row with a capability gap never reaches the shop.
rejects(function(l)
  l[#l + 1] = {catalog_id = 900, id = 'lesser_armor', source_ref = source.name .. '@' .. source.commit .. ':item_lesser_armor', name = '短甲', cost = 400}
end, 'needs_framework: PHYSICAL_CONSTANT_BLOCK')
-- E160: a source active is copied exactly, with its cooldown and mana cost.
ok(find(items, 'fire_scroll').active.kind == 'apply_modifiers', 'fire_scroll carries its source active')
rejects(function(l) find(l, 'fire_scroll').active.radius = 900 end, 'active differs from the source')
rejects(function(l) find(l, 'fire_scroll').active.modifiers[1].stats.basedamageoutgoing_percentage = 0.3 end, 'active differs from the source')
rejects(function(l) find(l, 'fire_scroll').cooldown = 10 end, 'cooldown 10 is not the source 26')
rejects(function(l) find(l, 'fire_scroll').mana_cost = nil end, 'mana cost nil is not the source 50')
rejects(function(l) find(l, 'tachi').active = { kind = 'restore_mana', amount = 100 } end, 'active differs from the source')
-- Batch 2: the ally target contract and single use follow the source too.
ok(find(items, 'izuminokami_kanesada').target.team == 'ally', 'izuminokami targets an ally')
rejects(function(l) find(l, 'izuminokami_kanesada').target.range = 900 end, 'target differs from the source')
rejects(function(l) find(l, 'rations').consumable = nil end, 'consumable nil is not the source true')
rejects(function(l) find(l, 'fire_scroll').consumable = true end, 'consumable true is not the source nil')
print(('item sources check: %d checks passed'):format(checks))
