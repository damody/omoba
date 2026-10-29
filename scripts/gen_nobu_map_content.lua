-- Original map as a lua_data template (history-nobu full port): docs/character_pipeline/maps/nobu.lua
-- (scripts/gen_nobu_map.lua, read from the original nobu.vmap) -> scripts/lua_data/templates/nobu_map.lua,
-- grouped per team and lane, with the gridnav collision merged into rectangles. Build-time only.
-- The template is not included by templates.lua yet: the game-mode line wires it into moba_maps.lua
-- (docs/plans/2026-10-28-history-nobu-full-port/nobu-map-comparison.md §5).
--
--   tools\lua\lua.exe scripts\gen_nobu_map_content.lua [--check] [--source FILE] [--out FILE]
--
-- Validation (any problem fails the run): every lane waypoint lies on a passable gridnav cell and the
-- three lanes of each team are connected on the gridnav; each lane starts at its team's base and ends at
-- the enemy base; every tower and barracks stands on its own team's lane of the same name; every
-- position is inside the playable bounds (the tile grid; world_bounds is smaller than what the map uses
-- and is kept only as data), except shops the source places outside it, which are flagged; and the
-- counts equal nobu.lua's.
local source_path = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source_path:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local serialize = require("hero_source.serialize")

local M = {}
M.SOURCE = "docs/character_pipeline/maps/nobu.lua"
M.OUT = "scripts/lua_data/templates/nobu_map.lua"
M.SIDES = { nobu = "goodguys", wl = "badguys" }        -- lane prefix -> team (織田方 nobu, 聯軍方 wl)
M.LANES = { top = "top", middle = "mid", button = "bot" }
M.BASE_REACH = 3000    -- a lane starts / ends within this of its own / the enemy fort
M.LANE_REACH = 1200    -- a tower or barracks stands within this of its own lane

local function dist(a, b) return math.sqrt((a[1] - b[1]) ^ 2 + (a[2] - b[2]) ^ 2) end
local function seg_dist(p, a, b)
  local dx, dy = b[1] - a[1], b[2] - a[2]
  local l2 = dx * dx + dy * dy
  local t = l2 > 0 and math.max(0, math.min(1, ((p[1] - a[1]) * dx + (p[2] - a[2]) * dy) / l2)) or 0
  return dist(p, { a[1] + t * dx, a[2] + t * dy })
end
local function lane_dist(p, pts)
  local best = math.huge
  for i = 2, #pts do best = math.min(best, seg_dist(p, pts[i - 1], pts[i])) end
  return best
end

-- gridnav blocked cells -> rectangles (row runs, then stacked while the run is identical)
local function rectangles(g)
  local W, H, cell = g.gridnav.width, g.gridnav.height, g.gridnav.cell
  local blocked = {}
  for _, idx in ipairs(g.gridnav.blocked) do blocked[idx] = true end
  local open = {} -- key "x0:x1" -> rect being extended
  local rects = {}
  for j = 0, H - 1 do
    local runs, i = {}, 0
    while i < W do
      if blocked[j * W + i] then
        local s = i
        while i < W and blocked[j * W + i] do i = i + 1 end
        runs[#runs + 1] = { s, i - 1 }
      else i = i + 1 end
    end
    local next_open = {}
    for _, run in ipairs(runs) do
      local key = run[1] .. ":" .. run[2]
      local rect = open[key]
      if rect then rect.j1 = j else rect = { i0 = run[1], i1 = run[2], j0 = j, j1 = j }; rects[#rects + 1] = rect end
      next_open[key] = rect
    end
    open = next_open
  end
  local out = {}
  for _, r in ipairs(rects) do
    out[#out + 1] = { min = { g.origin[1] + r.i0 * cell, g.origin[2] + r.j0 * cell },
      max = { g.origin[1] + (r.i1 + 1) * cell, g.origin[2] + (r.j1 + 1) * cell } }
  end
  table.sort(out, function(x, y) if x.min[2] ~= y.min[2] then return x.min[2] < y.min[2] end return x.min[1] < y.min[1] end)
  return out
end

-- passable-cell flood fill from a point; returns a cell -> true set
local function cell_of(g, p)
  local c = g.gridnav.cell
  local i, j = math.floor((p[1] - g.origin[1]) / c), math.floor((p[2] - g.origin[2]) / c)
  if i < 0 or j < 0 or i >= g.gridnav.width or j >= g.gridnav.height then return nil end
  return j * g.gridnav.width + i
end
local function flood(g, start, blocked)
  local W, H = g.gridnav.width, g.gridnav.height
  local seen, queue, head = { [start] = true }, { start }, 1
  while head <= #queue do
    local c = queue[head]; head = head + 1
    local i, j = c % W, c // W
    for _, d in ipairs({ { 1, 0 }, { -1, 0 }, { 0, 1 }, { 0, -1 } }) do
      local ni, nj = i + d[1], j + d[2]
      if ni >= 0 and nj >= 0 and ni < W and nj < H then
        local n = nj * W + ni
        if not seen[n] and not blocked[n] then seen[n] = true; queue[#queue + 1] = n end
      end
    end
  end
  return seen
end

function M.build(src)
  local g = src.tile_grid
  local map = {
    id = "nobu", schema = "nobu-map-content/1",
    source = { file = M.SOURCE, original = src.source.file, commit = src.source.commit },
    world_units_per_source_unit = src.world_units_per_source_unit,
    teams = {}, neutral_camps = {}, bosses = {}, shops = {}, trees = {},
  }
  -- playable bounds: the tile grid. world_bounds (kept as-is) is smaller than what the map uses: the
  -- badguys fountain, spawns and three camps stand above its max y.
  map.bounds = { min = { g.origin[1], g.origin[2] }, max = { g.origin[1] + g.width * g.tile, g.origin[2] + g.height * g.tile },
    source = g.source .. " tile grid extent" }
  map.world_bounds = src.bounds
  if src.minimap and #src.minimap == 2 then
    local a, c = src.minimap[1].position, src.minimap[2].position
    map.minimap_bounds = { min = { math.min(a[1], c[1]), math.min(a[2], c[2]) }, max = { math.max(a[1], c[1]), math.max(a[2], c[2]) },
      source = src.minimap[1].source .. " / " .. src.minimap[2].source }
  end
  for _, team in pairs(M.SIDES) do
    map.teams[team] = { lanes = {}, towers = {}, barracks = {}, general_towers = {}, base_units = {}, spawns = {} }
  end
  for _, l in ipairs(src.lanes) do
    local team, lane = M.SIDES[l.side], M.LANES[l.lane]
    local pts = {}
    for i, p in ipairs(l.points) do pts[i] = { p[1], p[2], source = p.source } end
    map.teams[team].lanes[lane] = { waypoints = pts, source_name = l.id }
  end
  local function put(list_name, rows, extra)
    for _, x in ipairs(rows) do
      local team = map.teams[x.team]
      if team then
        local row = { position = { x.position[1], x.position[2] }, source = x.source }
        for _, k in ipairs(extra) do row[k] = x[k] end
        table.insert(team[list_name], row)
      end
    end
  end
  put("towers", src.towers, { "lane", "tier", "unit", "name" })
  put("barracks", src.barracks, { "lane", "unit" })
  put("general_towers", src.general_towers, { "unit" })
  put("base_units", src.base_units, { "unit" })
  put("spawns", src.spawns, {})
  for _, f in ipairs(src.forts) do map.teams[f.team].fort = { position = { f.position[1], f.position[2] }, unit = f.unit, source = f.source } end
  for _, f in ipairs(src.fountains) do
    if map.teams[f.team] then map.teams[f.team].fountain = { position = { f.position[1], f.position[2] }, source = f.source } end
  end
  for _, c in ipairs(src.neutral_camps) do
    local units = {}
    for i, u in ipairs(c.units) do units[i] = { unit = u.unit, position = { u.position[1], u.position[2] }, source = u.source } end
    map.neutral_camps[#map.neutral_camps + 1] = { id = c.id, center = c.center, units = units }
  end
  for _, x in ipairs(src.bosses) do map.bosses[#map.bosses + 1] = { unit = x.unit, position = { x.position[1], x.position[2] }, source = x.source } end
  local SHOPTYPE = { [0] = "home", [1] = "side", [2] = "secret" }
  for _, s in ipairs(src.shops) do
    local inside = s.position[1] >= src.bounds.min[1] and s.position[1] <= src.bounds.max[1]
      and s.position[2] >= src.bounds.min[2] and s.position[2] <= src.bounds.max[2]
    map.shops[#map.shops + 1] = { kind = SHOPTYPE[s.shoptype] or tostring(s.shoptype), position = { s.position[1], s.position[2] },
      outside_bounds = (not inside) or nil, source = s.source }
  end
  for i, t in ipairs(g.trees) do map.trees[i] = { t[1], t[2] } end
  map.terrain = rectangles(g)
  map.terrain_source = g.source .. " gridnav (" .. g.gridnav.width .. "x" .. g.gridnav.height .. ", cell " .. g.gridnav.cell .. ")"
  return map
end

function M.validate(map, src)
  local problems = {}
  local function bad(s) problems[#problems + 1] = s end
  local g = src.tile_grid
  local blocked = {}
  for _, idx in ipairs(g.gridnav.blocked) do blocked[idx] = true end
  local function inside(p) return p[1] >= map.bounds.min[1] and p[1] <= map.bounds.max[1] and p[2] >= map.bounds.min[2] and p[2] <= map.bounds.max[2] end
  local enemy = { goodguys = "badguys", badguys = "goodguys" }
  for team, t in pairs(map.teams) do
    if not t.fort then bad(team .. ": no fort") end
    local reach
    for _, lane in ipairs({ "top", "mid", "bot" }) do
      local l = t.lanes[lane]
      if not l then bad(team .. ": no " .. lane .. " lane") else
        local pts = l.waypoints
        for _, p in ipairs(pts) do
          local c = cell_of(g, p)
          if not c then bad(team .. " " .. lane .. ": waypoint outside the tile grid " .. p.source)
          elseif blocked[c] then bad(team .. " " .. lane .. ": waypoint on a blocked cell " .. p.source) end
          if not inside(p) then bad(team .. " " .. lane .. ": waypoint outside the playable bounds " .. p.source) end
        end
        if t.fort and dist(pts[1], t.fort.position) > M.BASE_REACH then bad(team .. " " .. lane .. ": does not start at its base") end
        local foe = map.teams[enemy[team]]
        if foe and foe.fort and dist(pts[#pts], foe.fort.position) > M.BASE_REACH then bad(team .. " " .. lane .. ": does not end at the enemy base") end
        -- every waypoint reachable from the first on passable cells
        local start = cell_of(g, pts[1])
        if start and not blocked[start] then
          reach = reach or flood(g, start, blocked)
          for _, p in ipairs(pts) do
            local c = cell_of(g, p)
            if c and not reach[c] then bad(team .. " " .. lane .. ": waypoint not connected " .. p.source) end
          end
        end
      end
    end
    for _, kind in ipairs({ "towers", "barracks" }) do
      for _, x in ipairs(t[kind]) do
        local l = t.lanes[x.lane]
        if not l then bad(team .. " " .. kind .. " " .. x.source .. ": lane " .. tostring(x.lane) .. " is not one of the team's")
        elseif lane_dist(x.position, l.waypoints) > M.LANE_REACH then
          bad(team .. " " .. kind .. " " .. x.source .. ": not on its " .. x.lane .. " lane")
        end
        if not inside(x.position) then bad(team .. " " .. kind .. " " .. x.source .. ": outside the playable bounds") end
      end
    end
    for _, kind in ipairs({ "general_towers", "base_units", "spawns" }) do
      for _, x in ipairs(t[kind]) do if not inside(x.position) then bad(team .. " " .. kind .. " " .. x.source .. ": outside the playable bounds") end end
    end
    for _, k in ipairs({ "fort", "fountain" }) do if t[k] and not inside(t[k].position) then bad(team .. " " .. k .. ": outside the playable bounds") end end
  end
  for _, c in ipairs(map.neutral_camps) do for _, u in ipairs(c.units) do if not inside(u.position) then bad(c.id .. ": unit outside the playable bounds") end end end
  for _, x in ipairs(map.bosses) do if not inside(x.position) then bad("boss " .. x.unit .. ": outside the playable bounds") end end
  for _, s in ipairs(map.shops) do if (not inside(s.position)) ~= (s.outside_bounds == true) then bad("shop " .. s.source .. ": bounds flag") end end
  for _, tr in ipairs(map.trees) do if not inside(tr) then bad("tree outside the playable bounds at " .. tr[1] .. "," .. tr[2]) break end end
  -- counts against the source
  local function n(list) return #list end
  local towers, barracks, generals, base_units, lanes = 0, 0, 0, 0, 0
  for _, t in pairs(map.teams) do
    towers = towers + n(t.towers); barracks = barracks + n(t.barracks); generals = generals + n(t.general_towers)
    base_units = base_units + n(t.base_units)
    for _ in pairs(t.lanes) do lanes = lanes + 1 end
  end
  local checks = { { "lanes", lanes, #src.lanes }, { "towers", towers, #src.towers }, { "barracks", barracks, #src.barracks },
    { "general towers", generals, #src.general_towers }, { "base units", base_units, #src.base_units },
    { "neutral camps", #map.neutral_camps, #src.neutral_camps }, { "bosses", #map.bosses, #src.bosses },
    { "shops", #map.shops, #src.shops }, { "trees", #map.trees, #src.tile_grid.trees } }
  for _, c in ipairs(checks) do if c[2] ~= c[3] then bad(("%s: %d in the template, %d in nobu.lua"):format(c[1], c[2], c[3])) end end
  -- the terrain rectangles cover exactly the blocked cells
  local area = 0
  for _, r in ipairs(map.terrain) do area = area + (r.max[1] - r.min[1]) * (r.max[2] - r.min[2]) end
  if area ~= #g.gridnav.blocked * g.gridnav.cell * g.gridnav.cell then bad("terrain rectangles do not cover the blocked cells exactly") end
  return problems
end

function M.render(map)
  local header = "-- GENERATED by scripts/gen_nobu_map_content.lua from " .. M.SOURCE .. " (original history_nobu nobu.vmap); do not edit.\n" ..
    "-- Included by templates.lua (nobu_map) and read by templates/moba_maps.lua for the nobu map (E187): see\n" ..
    "-- docs/plans/2026-10-28-history-nobu-full-port/nobu-map-comparison.md §5.\n"
  local body = serialize.serialize(map, header)
  return (body:gsub("^(.-)return ", "%1local data = ", 1)) .. "return function(ctx) return data end\n"
end

function M.main(argv)
  local opts = {}
  local i = 1
  while i <= #argv do
    local a = argv[i]
    if a == "--check" then opts.check = true
    elseif a == "--source" or a == "--out" then opts[a:sub(3)] = argv[i + 1]; i = i + 1
    else error("unknown argument " .. a) end
    i = i + 1
  end
  local src = dofile(opts.source or (b.root .. "/" .. M.SOURCE))
  local map = M.build(src)
  local problems = M.validate(map, src)
  if #problems > 0 then
    for _, p in ipairs(problems) do io.stderr:write(p, "\n") end
    io.stderr:write(#problems .. " problem(s)\n")
    return 1
  end
  local out = opts.out or (b.root .. "/" .. M.OUT)
  local text = M.render(map)
  if opts.check then
    local f = io.open(out, "rb")
    local cur = f and f:read("a"); if f then f:close() end
    if cur ~= text then io.stderr:write(out .. " is stale; run scripts/gen_nobu_map_content.lua\n"); return 1 end
    print("OK: " .. out .. " is current")
    return 0
  end
  local f = assert(io.open(out, "wb")); f:write(text); f:close()
  print(("wrote %s: 2 teams, %d terrain rectangles, %d camps, %d trees"):format(out, #map.terrain, #map.neutral_camps, #map.trees))
  return 0
end

if rawget(_G, "GEN_NOBU_MAP_CONTENT_AS_MODULE") then return M end
os.exit(M.main(arg))
