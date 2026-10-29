-- Original map tools (scripts/nobu_map, scripts/gen_nobu_map.lua): the binary DMX reader on a
-- synthetic file, and, when the original checkout is present, the counts of nobu.vmap.
-- tools\lua\lua.exe scripts\tests\nobu_map_test.lua
package.path = 'scripts/?.lua;' .. package.path
local b = require('_bootstrap')
local dmx = require('nobu_map.dmx')

local count = 0
local function test(name, fn) fn(); count = count + 1; print('PASS ' .. name) end

-- a binary 9 DMX with one prefix attribute, two elements and every scalar kind the maps use
local function synthetic()
  local p = string.pack
  local strings = { 'CMapRootElement', 'root', 'CMapEntity', 'ent', 'world', 'origin', 'nodeID', 'classname', 'tower', 'flags', 'names' }
  local idx = {}
  for i, s in ipairs(strings) do idx[s] = i - 1 end
  local parts = { '<!-- dmx encoding binary 9 format vmap 29 -->\n\0' }
  -- prefix: one element with one inline string attribute
  parts[#parts + 1] = p('<i4', 1) .. p('<i4', 1) .. p('z', 'thumb_format') .. p('B', 5) .. p('z', 'jpg')
  parts[#parts + 1] = p('<i4', #strings)
  for _, s in ipairs(strings) do parts[#parts + 1] = p('z', s) end
  parts[#parts + 1] = p('<i4', 2)
  parts[#parts + 1] = p('<i4', idx.CMapRootElement) .. p('<i4', idx.root) .. string.rep('\1', 16)
  parts[#parts + 1] = p('<i4', idx.CMapEntity) .. p('<i4', idx.ent) .. string.rep('\2', 16)
  -- root: world -> element 2
  parts[#parts + 1] = p('<i4', 1) .. p('<i4', idx.world) .. p('B', 1) .. p('<i4', 1)
  -- entity: origin vector3, nodeID int, classname string (table index), flags bool array, names string array
  parts[#parts + 1] = p('<i4', 5)
    .. p('<i4', idx.origin) .. p('B', 10) .. p('<fff', 1.5, -2, 128)
    .. p('<i4', idx.nodeID) .. p('B', 2) .. p('<i4', 42)
    .. p('<i4', idx.classname) .. p('B', 5) .. p('<i4', idx.tower)
    .. p('<i4', idx.flags) .. p('B', 4 + 32) .. p('<i4', 3) .. p('BBB', 1, 0, 1)
    .. p('<i4', idx.names) .. p('B', 5 + 32) .. p('<i4', 2) .. p('z', 'a') .. p('z', 'bc')
  return table.concat(parts)
end

test('dmx_reads_binary_9', function()
  local doc = dmx.parse(synthetic())
  assert(doc.version == 9 and doc.format == 'vmap' and doc.format_version == 29, 'header')
  assert(doc.prefix[1].thumb_format == 'jpg', 'prefix attributes use inline strings')
  local ent = doc.root.attrs.world
  assert(ent and ent.type == 'CMapEntity' and ent.name == 'ent', 'element references resolve to elements')
  local o = ent.attrs.origin
  assert(o[1] == 1.5 and o[2] == -2 and o[3] == 128, 'vector3')
  assert(ent.attrs.nodeID == 42 and ent.attrs.classname == 'tower', 'int and string-table string')
  assert(ent.attrs.flags[1] == true and ent.attrs.flags[2] == false and #ent.attrs.flags == 3, 'bool array')
  assert(ent.attrs.names[2] == 'bc', 'string arrays are inline')
end)

test('dmx_rejects_trailing_bytes', function()
  local ok, err = pcall(dmx.parse, synthetic() .. '\0')
  assert(not ok and tostring(err):find('ended at byte', 1, true), 'every byte must be consumed')
end)

-- lua_data template from the committed map data (no original checkout needed)
test('nobu_map_template_validates', function()
  GEN_NOBU_MAP_CONTENT_AS_MODULE = true
  local C = dofile(b.root .. '/scripts/gen_nobu_map_content.lua')
  local src = dofile(b.root .. '/docs/character_pipeline/maps/nobu.lua')
  local map = C.build(src)
  local p = C.validate(map, src)
  assert(#p == 0, table.concat(p, '\n'))
  assert(map.teams.goodguys.lanes.top and map.teams.badguys.lanes.bot, 'lanes per team')
  assert(#map.terrain > 0 and #map.trees == #src.tile_grid.trees, 'terrain rectangles and trees')
  -- a tower moved off its lane and a lane removed are both caught
  local moved = C.build(src)
  moved.teams.goodguys.towers[1].position = { moved.teams.goodguys.fort.position[1], moved.teams.goodguys.fort.position[2] }
  local mp = C.validate(moved, src)
  assert(table.concat(mp, '\n'):find('not on its', 1, true), 'a tower off its lane is reported')
  local missing = C.build(src)
  missing.teams.badguys.lanes.mid = nil
  assert(table.concat(C.validate(missing, src), '\n'):find('no mid lane', 1, true), 'a missing lane is reported')
  -- the committed template is current
  local f = assert(io.open(b.root .. '/' .. C.OUT, 'rb')); local cur = f:read('a'); f:close()
  assert(cur == C.render(map), C.OUT .. ' is stale; run scripts/gen_nobu_map_content.lua')
end)

-- E187 §5.4: every unit the camps place has a neutral template, and every template is placed.
local neutrals = dofile(b.root .. '/scripts/lua_data/templates/nobu_neutrals.lua')({})
test('nobu_neutral_templates_cover_every_camp_unit', function()
  local map = dofile(b.root .. '/scripts/lua_data/templates/nobu_map.lua')({})
  local used, count = {}, 0
  for _, c in ipairs(map.neutral_camps) do
    for _, u in ipairs(c.units) do assert(neutrals[u.unit], 'no template for ' .. u.unit); used[u.unit] = true; count = count + 1 end
  end
  for name, row in pairs(neutrals) do
    assert(used[name], name .. ' is placed by no camp')
    assert(row.id == name:gsub('^npc_dota_', ''), name .. ' template id')
  end
  assert(#map.neutral_camps == 24 and count == 68, '24 camps, 68 units')
end)

-- the original map, when the checkout is there
local source = 'C:/nobu/history_nobu'
local f = io.open(source .. '/content/dota_addons/nobu/maps/nobu.vmap', 'rb')
if f then
  f:close()
  test('nobu_vmap_counts', function()
    GEN_NOBU_MAP_AS_MODULE = true
    local G = dofile(b.root .. '/scripts/gen_nobu_map.lua')
    local d = G.build(source)
    assert(#d.lanes == 6 and #d.towers == 32 and #d.barracks == 12 and #d.forts == 2, 'lanes, towers, barracks, forts')
    assert(#d.neutral_camps == 24 and #d.bosses == 2 and #d.general_towers == 10, 'camps, bosses, general towers')
    for _, l in ipairs(d.lanes) do assert(#l.points >= 6, l.id .. ' is a chain') end
    assert(d.tile_grid and #d.tile_grid.vertex_levels == 71 * 71, 'tile grid vertices')
  end)
  -- E187 §5.4: each neutral template equals its KV block in npc_units_custom_load.txt.
  test('nobu_neutral_templates_match_the_original_kv', function()
    local kv = assert(io.open(source .. '/game/dota_addons/nobu/scripts/npc/npc_units_custom_load.txt', 'rb'))
    local lines = {}
    for l in kv:lines() do lines[#lines + 1] = l end
    kv:close()
    for name, row in pairs(neutrals) do
      assert(lines[row.line]:match('^%s*"([%w_]+)"%s*$') == name, name .. ' block starts at line ' .. row.line)
      local values, depth, i = {}, 0, row.line + 1
      repeat
        local code = lines[i]:gsub('//.*$', '')
        if code:find('{', 1, true) then depth = depth + 1 end
        if code:find('}', 1, true) then depth = depth - 1 end
        local k, v = code:match('^%s*"([%w_]+)"%s*"([^"]*)"')
        if k and depth == 1 then values[k] = v end
        i = i + 1
      until depth == 0
      local function n(k) return tonumber(values[k]) end
      local want = { level = n('Level'), hp = n('StatusHealth'), damage_min = n('AttackDamageMin'), damage_max = n('AttackDamageMax'),
        attack_rate_ms = math.floor(n('AttackRate') * 1000 + 0.5), attack_range = n('AttackRange'), armor = n('ArmorPhysical'),
        magic_resistance = n('MagicalResistance') or 0, move_speed = n('MovementSpeed'), gold = n('BountyGoldMin'), xp = n('BountyXP') }
      assert(n('BountyGoldMax') == want.gold, name .. ' fixed bounty')
      for k, v in pairs(want) do assert(row[k] == v, ('%s %s: template %s, KV %s'):format(name, k, tostring(row[k]), tostring(v))) end
    end
  end)
else
  print('SKIP nobu_vmap_counts (no original checkout at ' .. source .. ')')
end

print(count .. ' nobu map tests passed')
