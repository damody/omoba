-- Directed autopilot plans (E280): steps a live check wants the UE autopilot to take besides
-- playing its lane (clear a camp, cast a slot at a tree, come back to see the camp respawned or
-- the tree regrown). A plan file is Lua returning { steps = { ... } }; map targets are named and
-- resolved here to backend points of the match map, so the renderer (OmAutoPilotPlan) only reads
-- numbers. moba_role_launch writes the resolved JSON per autopilot seat (--autopilot-plan FILE).
--
-- Step fields:
--   ['do']      walk | clear | watch | cast (FOmAutoPilotPlan::FStep::EKind)
--   target      exactly one of
--                 at = {x, y}
--                 camp = 'camp_id' | {side = 'own'|'enemy'|'any', rank = n}
--                      (rank by distance from the seat's home; side from the map's camp sides,
--                      else whichever home the camp is nearer to)
--                 tree = {near = <target>, rank = n}   the n-th nearest tree to that target
--                 same = 'label' | index               the point another step resolved
--   radius, delay, timeout, shot_start, shot, shot_delay, label   passed through
--   delay_from  'label' | index: `delay` counts from that earlier step's end (default the previous)
--   slot        cast only: q | w | e | r | extra (or 0..4)
--   from        cast only, a target like the step's own: where the hero stands to aim (a tree
--               inside a forest is aimed at from the open ground next to it)
local source_file = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source_file:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json = b.lib('path'), b.lib('json')

local M = {}

local SLOTS = { q = 0, w = 1, e = 2, r = 3, extra = 4, d = 4 }
local KINDS = { walk = true, clear = true, watch = true, cast = true }
-- Defaults per kind: a camp is cleared or watched within its leash, a walk ends near its point.
local RADIUS = { walk = 250, clear = 700, watch = 700, cast = 250 }

function M.load_map(lua_data, map_id)
  local ctx = {}
  ctx.include = function(rel)
    local value = assert(loadfile(lua_data .. '/' .. rel))()
    if type(value) == 'function' then value = value(ctx) end
    return value
  end
  for _, map in ipairs(ctx.include('templates/moba_maps.lua')) do
    if map.id == map_id then return map end
  end
  error('autopilot plan: no map ' .. tostring(map_id))
end

local function d2(a, b) return (a[1] - b[1]) ^ 2 + (a[2] - b[2]) ^ 2 end

-- The seat's home: its base, else the start of the first lane (lanes run from team 1's side).
function M.home(map, team_id)
  if map.bases and map.bases[team_id] then return map.bases[team_id].position, map.bases[team_id].team end
  local lane = assert(map.lanes and map.lanes[1], 'autopilot plan: map has no lane or base').waypoints
  return team_id == 1 and lane[1] or lane[#lane], nil
end

local function camp_point(map, team_id, spec)
  local camps = map.jungle_camps or {}
  if type(spec) == 'string' then
    for _, camp in ipairs(camps) do if camp.id == spec then return camp.position, camp.id end end
    error('autopilot plan: no camp ' .. spec .. ' on map ' .. map.id)
  end
  assert(type(spec) == 'table', 'autopilot plan: camp must be an id or {side, rank}')
  local side = spec.side or 'any'
  assert(side == 'own' or side == 'enemy' or side == 'any', 'autopilot plan: camp side must be own, enemy or any')
  local home, own_side = M.home(map, team_id)
  local away = M.home(map, team_id == 1 and 2 or 1)
  local list = {}
  for _, camp in ipairs(camps) do
    local mine
    if camp.side and own_side then mine = camp.side == own_side
    else mine = d2(camp.position, home) < d2(camp.position, away) end
    -- Bosses (tags boss) are objectives, not camps a check clears alone.
    local boss = false
    for _, tag in ipairs(camp.tags or {}) do if tag == 'boss' then boss = true end end
    if not boss and (side == 'any' or (side == 'own') == mine) then list[#list + 1] = camp end
  end
  table.sort(list, function(a, b)
    local da, db = d2(a.position, home), d2(b.position, home)
    if da ~= db then return da < db end
    return a.id < b.id
  end)
  local camp = assert(list[spec.rank or 1], 'autopilot plan: no camp of that side and rank')
  return camp.position, camp.id
end

function M.resolve(source, map, team_id)
  assert(type(source) == 'table' and type(source.steps) == 'table' and #source.steps >= 1, 'autopilot plan needs steps')
  local out, by_label, index_of = {}, {}, {}
  local function point(step, index)
    local forms = (step.at and 1 or 0) + (step.camp and 1 or 0) + (step.tree and 1 or 0) + (step.same and 1 or 0)
    assert(forms == 1, ('autopilot plan step %d: exactly one of at, camp, tree, same'):format(index))
    if step.at then return { step.at[1], step.at[2] }, nil end
    if step.camp then return camp_point(map, team_id, step.camp) end
    if step.same then
      local other = type(step.same) == 'number' and out[step.same] or by_label[step.same]
      return assert(other, ('autopilot plan step %d: same names no earlier step'):format(index)).at, nil
    end
    local near = point(step.tree.near or {}, index)
    local trees = {}
    for _, tree in ipairs(assert(map.trees and #map.trees > 0 and map.trees, 'autopilot plan: map ' .. map.id .. ' has no trees')) do
      trees[#trees + 1] = tree.position
    end
    table.sort(trees, function(a, b)
      local da, db = d2(a, near), d2(b, near)
      if da ~= db then return da < db end
      return a[1] < b[1] or a[1] == b[1] and a[2] < b[2]
    end)
    local tree = assert(trees[step.tree.rank or 1], 'autopilot plan: no tree of that rank')
    return { tree[1], tree[2] }, 'tree'
  end
  for index, step in ipairs(source.steps) do
    local kind = step['do']
    assert(KINDS[kind], ('autopilot plan step %d: do must be walk, clear, watch or cast'):format(index))
    local at, name = point(step, index)
    local row = { ['do'] = kind, at = at, radius = step.radius or RADIUS[kind],
      label = step.label or (kind .. '_' .. index), delay = step.delay, timeout = step.timeout,
      shot_start = step.shot_start, shot = step.shot, shot_delay = step.shot_delay, target = name }
    if step.from then
      assert(kind == 'cast', ('autopilot plan step %d: from belongs to a cast'):format(index))
      row.from = point(step.from, index)
    end
    if kind == 'cast' then
      local slot = type(step.slot) == 'number' and step.slot or SLOTS[step.slot]
      assert(math.tointeger(slot) and slot >= 0 and slot <= 4, ('autopilot plan step %d: cast needs slot q|w|e|r|extra'):format(index))
      row.slot = slot
    end
    if step.delay_from ~= nil then
      local from = type(step.delay_from) == 'number' and step.delay_from or index_of[step.delay_from]
      assert(math.tointeger(from) and from >= 1 and from < index, ('autopilot plan step %d: delay_from names no earlier step'):format(index))
      row.delay_from = from
    end
    out[index] = row
    by_label[row.label] = row
    index_of[row.label] = index
  end
  return { schema = 1, map_id = map.id, team_id = team_id, steps = out }
end

-- Reads FILE (Lua), resolves it on the match map for the seat's team and writes OUT (JSON).
function M.write(file, lua_data, map_id, team_id, out_path)
  local source = assert(loadfile(file))()
  local plan = M.resolve(source, M.load_map(lua_data, map_id), team_id)
  path.write(out_path, json.encode(plan))
  return plan
end

return M
