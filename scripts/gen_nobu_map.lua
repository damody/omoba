-- Original map data (history_nobu nobu.vmap) for the game-mode and UE lines: read-only, from the
-- original checkout's Hammer map; no asset is copied and nothing of the original is executed.
--
--   tools\lua\lua.exe scripts\gen_nobu_map.lua [--source C:\nobu\history_nobu] [--out DIR] [--heightmap DIR]
--
-- Writes DIR (default docs/character_pipeline/maps): nobu.lua (machine-readable, every entry with its
-- source `<vmap>#<nodeID>`) and nobu.md (summary). --heightmap also writes UE landscape drafts
-- (target/maps by default is a good place; they are generated, not committed).
-- Coordinates: source (Dota/Hammer) units times WORLD_UNITS_PER_SOURCE_UNIT, the one conversion
-- factor (value-single-source: a `dist` is the original distance; ranges are 1:1 today, so 1).
local source_path = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source_path:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local world = require("nobu_map.world")

local M = {}
M.WORLD_UNITS_PER_SOURCE_UNIT = 1
M.MAP = "maps/nobu.vmap"
M.CONTENT = "content/dota_addons/nobu"
M.TILE = 256        -- Dota tile grid: tile edge in source units (70 tiles cover the playable area)
M.SUB = 4           -- object and gridnav points per tile edge
M.CAMP_LINK = 450   -- neutral units this close (source units) belong to one camp

local function r(x) return math.floor(x * M.WORLD_UNITS_PER_SOURCE_UNIT + 0.5) end
local function pos(o) return { r(o[1]), r(o[2]), r(o[3]) } end

local TEAMS = { [2] = "goodguys", [3] = "badguys", [4] = "neutral" }

function M.build(source_root)
  local content_root = source_root .. "/" .. M.CONTENT
  local ents, info = world.entities(content_root, M.MAP)
  local out = {
    schema = "nobu-map/1",
    source = { repository = "history_nobu", commit = "f966720", file = M.CONTENT .. "/" .. M.MAP,
      format = info.root.encoding .. " " .. info.root.version .. ", " .. info.root.format .. " " .. info.root.format_version,
      prefab_files = (function() local n = 0 for _ in pairs(info.files) do n = n + 1 end return n end)(),
      unresolved_prefabs = info.missing },
    world_units_per_source_unit = M.WORLD_UNITS_PER_SOURCE_UNIT,
    lanes = {}, towers = {}, barracks = {}, forts = {}, shops = {}, fountains = {}, spawns = {}, couriers = {},
    neutral_camps = {}, other = {},
  }
  local function base(e)
    local via = {}
    for i, v in ipairs(e.via) do via[i] = v.name or v.path end
    return { position = pos(e.origin), yaw = math.floor((e.yaw or 0) * 10 + 0.5) / 10, source = e.source,
      prefab = #via > 0 and table.concat(via, " > ") or nil }
  end
  local corners = {}
  local camps = {}
  for _, e in ipairs(ents) do
    local c, p = e.classname or "", e.props
    if c == "world_bounds" then
      local mn = {} for v in tostring(p.min):gmatch("%S+") do mn[#mn + 1] = r(tonumber(v)) end
      local mx = {} for v in tostring(p.max):gmatch("%S+") do mx[#mx + 1] = r(tonumber(v)) end
      out.bounds = { min = { mn[1], mn[2] }, max = { mx[1], mx[2] }, source = e.source }
    elseif c == "dota_minimap_boundary" then
      out.minimap = out.minimap or {}
      out.minimap[#out.minimap + 1] = { position = pos(e.origin), source = e.source }
    elseif c == "path_corner" then
      corners[p.targetname] = { name = p.targetname, next = p.target ~= "" and p.target or nil, e = e }
    elseif c == "npc_dota_tower" or c == "npc_dota_barracks" or c == "npc_dota_fort" then
      local row = base(e)
      row.name, row.unit, row.team = p.targetname, p.MapUnitName, TEAMS[tonumber(p.teamnumber)] or p.teamnumber
      -- the original reuses names (every tower is towerN_*, every barracks good_rax_*): the lane comes
      -- from the name suffix, or the unit name (com_soldiercamp_*_top/mid/down); the tier is set below
      row.lane = tostring(p.targetname):match("_(%a+)$")
      local ul = tostring(p.MapUnitName):match("_(%a+)$")
      if c == "npc_dota_barracks" and ul then row.lane = ({ down = "bot", top = "top", mid = "mid" })[ul] or ul end
      local list = c == "npc_dota_tower" and out.towers or (c == "npc_dota_barracks" and out.barracks or out.forts)
      list[#list + 1] = row
    elseif c == "ent_dota_shop" then
      local row = base(e); row.shoptype = tonumber(p.shoptype); row.model = p.model
      out.shops[#out.shops + 1] = row
    elseif c:match("^prefab:") and c:match("fountain") then
      local row = base(e); row.team = c:match("radiant") and "goodguys" or (c:match("dire") and "badguys" or nil)
      row.unresolved = e.unresolved_prefab
      out.fountains[#out.fountains + 1] = row
    elseif c == "info_player_start_goodguys" or c == "info_player_start_badguys" or c == "info_player_start_dota" then
      local row = base(e); row.team = c:match("goodguys") and "goodguys" or (c:match("badguys") and "badguys" or "any")
      out.spawns[#out.spawns + 1] = row
    elseif c == "info_courier_spawn_radiant" or c == "info_courier_spawn_dire" then
      local row = base(e); row.team = c:match("radiant") and "goodguys" or "badguys"
      out.couriers[#out.couriers + 1] = row
    elseif c == "npc_dota_base" then
      -- neutral and camp units: grouped by their prefab placement
      local v = e.via[#e.via]
      local key = v and v.node or e.source
      local camp = camps[key]
      if not camp then
        camp = { kind = v and v.path:match("([^/]+)%.vmap$") or "?", position = v and pos(v.origin) or pos(e.origin),
          source = v and v.node or e.source, units = {} }
        camps[key] = camp
      end
      camp.units[#camp.units + 1] = { unit = p.MapUnitName, team = TEAMS[tonumber(p.m_iTeamNum)] or p.m_iTeamNum,
        position = pos(e.origin), source = e.source }
    elseif c ~= "prop_static" and c ~= "" then
      out.other[c] = (out.other[c] or 0) + 1
    end
  end
  -- lanes: chains of path_corner, the head being the name without a trailing number
  for name, cn in pairs(corners) do
    local side, lane = name:match("^chubinluxian_location_of_(%a+)_(%a+)$")
    if side then
      local pts, guard, cur = {}, 0, cn
      while cur and guard < 200 do
        pts[#pts + 1] = { pos(cur.e.origin)[1], pos(cur.e.origin)[2], name = cur.name, source = cur.e.source }
        cur = cur.next and corners[cur.next] or nil
        guard = guard + 1
      end
      out.lanes[#out.lanes + 1] = { id = side .. "_" .. lane, side = side, lane = lane, points = pts }
    end
  end
  table.sort(out.lanes, function(x, y) return x.id < y.id end)
  local in_lane = {}
  for _, l in ipairs(out.lanes) do for _, p in ipairs(l.points) do in_lane[p.name] = true end end
  out.unlinked_path_corners = {}
  for name in pairs(corners) do if not in_lane[name] then out.unlinked_path_corners[#out.unlinked_path_corners + 1] = name end end
  table.sort(out.unlinked_path_corners)
  -- npc_dota_base units by role: general towers, base camp units, decoration, bosses, neutral camps
  out.general_towers, out.base_units, out.decoration, out.bosses = {}, {}, {}, {}
  local neutrals = {}
  local keys = {}
  for k in pairs(camps) do keys[#keys + 1] = k end
  table.sort(keys)
  for _, k in ipairs(keys) do
    local placement = camps[k]
    for _, u in ipairs(placement.units) do
      local row = { unit = u.unit, team = u.team, position = u.position, source = u.source, prefab = placement.kind }
      local name = tostring(u.unit)
      if name:match("^com_general") then out.general_towers[#out.general_towers + 1] = row
      elseif name:match("_fake$") then out.decoration[#out.decoration + 1] = row
      elseif name == "com_soldiercamp" then out.base_units[#out.base_units + 1] = row
      elseif name == "npc_dota_cursed_warrior_souls" or name == "npc_dota_the_king_of_robbers" then out.bosses[#out.bosses + 1] = row
      elseif u.team == "neutral" then neutrals[#neutrals + 1] = row
      else out.base_units[#out.base_units + 1] = row end
    end
  end
  -- neutral camps: units within CAMP_LINK of each other (single link) form one camp
  local parent = {}
  local function find(i) while parent[i] ~= i do i = parent[i] end return i end
  for i = 1, #neutrals do parent[i] = i end
  for i = 1, #neutrals do
    for j = i + 1, #neutrals do
      local a, c = neutrals[i].position, neutrals[j].position
      if (a[1] - c[1]) ^ 2 + (a[2] - c[2]) ^ 2 <= M.CAMP_LINK ^ 2 then parent[find(i)] = find(j) end
    end
  end
  local groups, order = {}, {}
  for i, u in ipairs(neutrals) do
    local g = find(i)
    if not groups[g] then groups[g] = { units = {} }; order[#order + 1] = g end
    table.insert(groups[g].units, u)
  end
  for _, g in ipairs(order) do
    local camp = groups[g]
    local sx, sy, names = 0, 0, {}
    for _, u in ipairs(camp.units) do sx = sx + u.position[1]; sy = sy + u.position[2]; names[u.unit] = (names[u.unit] or 0) + 1 end
    camp.center = { math.floor(sx / #camp.units + 0.5), math.floor(sy / #camp.units + 0.5) }
    local ns = {}
    for n, c in pairs(names) do ns[#ns + 1] = n .. (c > 1 and (" x" .. c) or "") end
    table.sort(ns)
    camp.composition = table.concat(ns, ", ")
    out.neutral_camps[#out.neutral_camps + 1] = camp
  end
  table.sort(out.neutral_camps, function(x, y) if x.center[2] ~= y.center[2] then return x.center[2] > y.center[2] end return x.center[1] < y.center[1] end)
  for i, camp in ipairs(out.neutral_camps) do camp.id = ("camp_%02d"):format(i) end
  for _, list in ipairs({ out.general_towers, out.base_units, out.decoration, out.bosses }) do
    table.sort(list, function(x, y) return x.source < y.source end)
  end
  -- tower tier: 1 = the outermost pair on the lane (farthest from the team's own fort); the original
  -- names do not say
  local fort_of = {}
  for _, f in ipairs(out.forts) do fort_of[f.team] = f.position end
  local groups = {}
  for _, t in ipairs(out.towers) do local k = tostring(t.team) .. "/" .. tostring(t.lane); groups[k] = groups[k] or {}; table.insert(groups[k], t) end
  for _, list in pairs(groups) do
    local f = fort_of[list[1].team]
    if f then
      table.sort(list, function(x, y)
        local dx = (x.position[1] - f[1]) ^ 2 + (x.position[2] - f[2]) ^ 2
        local dy = (y.position[1] - f[1]) ^ 2 + (y.position[2] - f[2]) ^ 2
        if dx ~= dy then return dx > dy end
        return x.source < y.source
      end)
      -- towers stand in pairs: a new tier starts when the distance to the fort drops by more than 500
      local tier, last = 0, nil
      for _, t in ipairs(list) do
        local dist = math.sqrt((t.position[1] - f[1]) ^ 2 + (t.position[2] - f[2]) ^ 2)
        if not last or last - dist > 500 then tier = tier + 1 end
        t.tier, last = tier, dist
      end
    end
  end
  for _, list in ipairs({ out.towers, out.barracks, out.forts, out.shops, out.fountains, out.spawns, out.couriers }) do
    table.sort(list, function(x, y) return x.source < y.source end)
  end
  -- Dota tile grid
  local grid
  for _, el in ipairs(info.root.elements) do if el.type == "CMapDotaTileGrid" then grid = el end end
  if grid then
    local L = grid.attrs.layerData[1].attrs
    local W, H = L.gridWidth, L.gridHeight
    local o = grid.attrs.origin
    local N = W * M.SUB + 1
    local step = M.TILE / M.SUB
    local tg = { origin = pos(o), tile = r(M.TILE), width = W, height = H, source = M.MAP .. "#" .. tostring(grid.attrs.nodeID),
      index_order = "row-major, x fastest: index = j * width + i (i along +x, j along +y), from the grid origin",
      index_order_evidence = "gameplay entities on blocked gridnav cells: 0 of 242 in this order, 5 of 242 with x and y swapped",
      vertex_levels = {}, water = {}, trees = {}, plants = {}, gridnav = { cell = r(step), width = W * M.SUB, height = H * M.SUB, blocked = {} },
      level_height_note = "verticesHeight is an integer cliff level per vertex (0..5, one vertex -1); every structure stands on level 0 (z about 128-150). The height of one level is not stored in the map; the Dota tile editor convention is 128 units per level (assumed, not verified).",
      tree_types = {}, plant_types = {} }
    for i, v in ipairs(L.verticesHeight) do tg.vertex_levels[i] = v end
    for i, v in ipairs(L.verticesWater) do if v then tg.water[#tg.water + 1] = i - 1 end end
    local infos = L.tileSetMapInfo or {}
    for idx, t in ipairs(L.objectsTreeType) do
      local set = (L.objectsTileSet[idx] or 0) + 1
      local i, j = (idx - 1) % N, (idx - 1) // N
      local x, y = o[1] + i * step, o[2] + j * step
      if t ~= 0 then
        local names = infos[set] and infos[set].attrs.treeTypes or {}
        tg.trees[#tg.trees + 1] = { r(x), r(y), names[t + 1] or tostring(t), L.objectsTreeSize[idx] }
      end
      local pt = L.objectsPlantType[idx]
      if pt ~= 0 then
        local names = infos[set] and infos[set].attrs.plantTypes or {}
        tg.plants[#tg.plants + 1] = { r(x), r(y), names[pt + 1] or tostring(pt) }
      end
    end
    for idx, f in ipairs(L.gridnavFlags) do if f ~= 0 then tg.gridnav.blocked[#tg.gridnav.blocked + 1] = idx - 1 end end
    out.tile_grid = tg
  end
  return out
end

-- Lua literal with sorted keys (deterministic output)
local function literal(v, ind, compact)
  ind = ind or ""
  if type(v) == "string" then return string.format("%q", v) end
  if type(v) == "number" then return math.type(v) == "integer" and tostring(v) or string.format("%.4g", v) end
  if type(v) ~= "table" then return tostring(v) end
  local n = #v
  local keys = {}
  for k in pairs(v) do if not (math.type(k) == "integer" and k >= 1 and k <= n) then keys[#keys + 1] = k end end
  table.sort(keys, function(a, b) return tostring(a) < tostring(b) end)
  local flat = true
  for i = 1, n do if type(v[i]) == "table" then flat = false end end
  for _, k in ipairs(keys) do if type(v[k]) == "table" then flat = false end end
  local parts = {}
  for i = 1, n do parts[#parts + 1] = literal(v[i], ind .. "  ", true) end
  for _, k in ipairs(keys) do
    local key = type(k) == "string" and k:match("^[%a_][%w_]*$") and k or ("[" .. literal(k) .. "]")
    parts[#parts + 1] = key .. " = " .. literal(v[k], ind .. "  ", true)
  end
  if #parts == 0 then return "{}" end
  if flat then
    -- long flat arrays wrap every 24 entries
    local lines, line = {}, {}
    for i, p in ipairs(parts) do line[#line + 1] = p; if #line == 24 then lines[#lines + 1] = table.concat(line, ", "); line = {} end end
    if #line > 0 then lines[#lines + 1] = table.concat(line, ", ") end
    if #lines == 1 then return "{ " .. lines[1] .. " }" end
    return "{\n" .. ind .. "  " .. table.concat(lines, ",\n" .. ind .. "  ") .. ",\n" .. ind .. "}"
  end
  return "{\n" .. ind .. "  " .. table.concat(parts, ",\n" .. ind .. "  ") .. ",\n" .. ind .. "}"
end

function M.render_lua(data)
  return "-- GENERATED by scripts/gen_nobu_map.lua from the original history_nobu nobu.vmap (read-only;\n" ..
    "-- nothing copied); do not edit. Summary and caveats: nobu.md next to this file.\n" ..
    "return " .. literal(data) .. "\n"
end

function M.render_md(d)
  local out = {}
  local function w(s) out[#out + 1] = s end
  local function count_by(list, field)
    local t = {}
    for _, x in ipairs(list) do local k = tostring(x[field]); t[k] = (t[k] or 0) + 1 end
    local o = {} for k, v in pairs(t) do o[#o + 1] = k .. " " .. v end table.sort(o)
    return table.concat(o, "、")
  end
  w("# 原作地圖 nobu.vmap 摘要")
  w("")
  w("由 `tools\\lua\\lua.exe scripts\\gen_nobu_map.lua` 從原作 history_nobu（commit " .. d.source.commit .. "）`" .. d.source.file ..
    "` 唯讀解析產生，不要手改。資料在 [nobu.lua](nobu.lua)，每筆都附出處 `<vmap>#<nodeID>`。")
  w("")
  w("- 格式：" .. d.source.format .. "。解析器 `scripts/nobu_map/dmx.lua` 讀到檔尾剛好用完每個位元組。")
  w("- prefab：共讀了 " .. d.source.prefab_files .. " 份 vmap（主地圖與 " .. (d.source.prefab_files - 1) .. " 份 prefab，含巢狀），逐層累積位置、角度、縮放後展開。")
  w("- 有 " .. #d.source.unresolved_prefabs .. " 個 prefab 不在 addon 裡，是 Dota 內建資產，只記錄它的擺放位置：" ..
    (#d.source.unresolved_prefabs > 0 and ("`" .. table.concat(d.source.unresolved_prefabs, "`、`") .. "`") or "無") .. "。")
  w("- 座標：原作單位 × `world_units_per_source_unit` = " .. d.world_units_per_source_unit ..
    "。這是唯一的換算係數，射程等 `dist` 數值目前也是 1:1。")
  w("")
  w("| 項目 | 數量 | 備註 |")
  w("|---|---|---|")
  w(("| 地圖邊界 world_bounds | 1 | min (%d, %d)、max (%d, %d) |"):format(d.bounds.min[1], d.bounds.min[2], d.bounds.max[1], d.bounds.max[2]))
  local lanes = {}
  for _, l in ipairs(d.lanes) do lanes[#lanes + 1] = l.id .. " " .. #l.points .. " 點" end
  w("| 出兵路徑 | " .. #d.lanes .. " | " .. table.concat(lanes, "、") .. (#d.unlinked_path_corners > 0 and ("；未串入路徑的 path_corner：" .. table.concat(d.unlinked_path_corners, "、")) or "") .. " |")
  w("| 塔 | " .. #d.towers .. " | 隊伍：" .. count_by(d.towers, "team") .. "；路：" .. count_by(d.towers, "lane") .. " |")
  w("| 兵營 | " .. #d.barracks .. " | " .. count_by(d.barracks, "team") .. " |")
  w("| 主堡 | " .. #d.forts .. " | " .. count_by(d.forts, "team") .. " |")
  w("| 泉水 | " .. #d.fountains .. " | Dota 內建 prefab，只有位置 |")
  w("| 商店 | " .. #d.shops .. " | shoptype：" .. count_by(d.shops, "shoptype") .. " |")
  w("| 出生點 | " .. #d.spawns .. " | " .. count_by(d.spawns, "team") .. " |")
  local units = 0
  for _, c in ipairs(d.neutral_camps) do units = units + #c.units end
  w("| 大將塔（com_general*） | " .. #d.general_towers .. " | " .. count_by(d.general_towers, "team") .. "，是基地內的 npc_dota_base 單位 |")
  w("| 野怪營地 | " .. #d.neutral_camps .. " | 中立單位 " .. units .. " 個，相距 " .. 450 .. " 以內的單位歸為同一營 |")
  w("| 魔王 | " .. #d.bosses .. " | " .. (function() local o = {} for _, x in ipairs(d.bosses) do o[#o + 1] = x.unit end return table.concat(o, "、") end)() .. " |")
  w("| 基地單位 com_soldiercamp | " .. #d.base_units .. " | " .. count_by(d.base_units, "team") .. "，用途地圖看不出來（見下） |")
  w("| 裝飾用的假主堡 | " .. #d.decoration .. " | *_fake |")
  if d.tile_grid then
    local g = d.tile_grid
    w(("| 地形網格 | %d×%d tile | 原點 (%d, %d)，tile 邊長 %d；頂點高度等級 %d 個；水面頂點 %d 個 |"):format(g.width, g.height, g.origin[1], g.origin[2], g.tile, #g.vertex_levels, #g.water))
    w(("| 樹木 | %d | 物件格點（每 %d 單位）上的樹種 |"):format(#g.trees, g.gridnav.cell))
    w(("| 植物 | %d | 稀疏／中等／茂密，屬於呈現，不是遊戲的草叢 |"):format(#g.plants))
    w(("| gridnav 不可通行格 | %d | %d×%d 格，每格 %d 單位 |"):format(#g.gridnav.blocked, g.gridnav.width, g.gridnav.height, g.gridnav.cell))
  end
  w("")
  w("## 無法確定或沒有的資料")
  w("")
  w("- **草叢**：地圖裡沒有草叢實體或觸發區。原作的草叢視野若由腳本或其他資料定義，不在 vmap 裡；植物格點是裝飾。")
  w("- **高度**：只有每個頂點的整數等級（0～5）。每一級代表多少世界單位，檔案裡沒有記錄；Dota 地形編輯器的慣例是每級 128（推定，未驗證）。")
  w("- **網格方向**：以「x 先、逐列」排列，這是從資料推定的。依據：這個順序下，242 個遊戲實體沒有一個落在不可通行格上；x、y 對調時有 5 個。")
  w("- **泉水**：用的是 Dota 內建 prefab，addon 沒有它的內容，只知道擺放位置。")
  w("- **野怪的生成規則**：vmap 只有擺放的單位；重生與成長在原作腳本裡，不在地圖。")
  w("- **網格數值**：有一個頂點的高度是 -1，原樣保留。")
  w("- **基地單位 com_soldiercamp**：每方 12 個 npc_dota_base，放在基地裡，單位名稱與兵營建築（npc_dota_barracks 的 com_soldiercamp_*_top 等）不同。作用要看原作的單位 KV 與腳本，這裡只列位置。")
  w("")
  w("## 野怪營地")
  w("")
  w("| 營地 | 中心 | 組成 |")
  w("|---|---|---|")
  for _, c in ipairs(d.neutral_camps) do w(("| %s | (%d, %d) | %s |"):format(c.id, c.center[1], c.center[2], c.composition)) end
  w("")
  return table.concat(out, "\n")
end

-- UE landscape draft: 16-bit little-endian heightmap resampled to a valid landscape size (253)
function M.write_heightmap(d, dir)
  local g = d.tile_grid
  local V = g.width + 1
  local size = 253
  local lmin, lmax = math.huge, -math.huge
  for _, v in ipairs(g.vertex_levels) do lmin = math.min(lmin, v); lmax = math.max(lmax, v) end
  local function at(i, j) return g.vertex_levels[j * V + i + 1] end
  local parts = {}
  for y = 0, size - 1 do
    for x = 0, size - 1 do
      local fx, fy = x / (size - 1) * (V - 1), y / (size - 1) * (V - 1)
      local i, j = math.floor(fx), math.floor(fy)
      local i1, j1 = math.min(i + 1, V - 1), math.min(j + 1, V - 1)
      local tx, ty = fx - i, fy - j
      local v = (at(i, j) * (1 - tx) + at(i1, j) * tx) * (1 - ty) + (at(i, j1) * (1 - tx) + at(i1, j1) * tx) * ty
      local n = math.floor((v - lmin) / (lmax - lmin) * 65535 + 0.5)
      parts[#parts + 1] = string.pack("<I2", n)
    end
  end
  b.lib("path").mkdir_p(dir)
  local f = assert(io.open(dir .. "/nobu_height_253.r16", "wb")); f:write(table.concat(parts)); f:close()
  local note = string.format("nobu_height_253.r16: %dx%d, 16-bit little-endian, row-major (+x fastest, +y rows) from the grid origin (%d, %d);\n" ..
    "covers %d x %d source units (tile grid %dx%d of %d); level %d maps to 0 and level %d to 65535 (levels are cliff steps;\n" ..
    "their height in units is not in the map, see nobu.md). UE: Landscape > Import from file, resolution 253x253.\n",
    size, size, g.origin[1], g.origin[2], g.width * g.tile, g.height * g.tile, g.width, g.height, g.tile, lmin, lmax)
  f = assert(io.open(dir .. "/nobu_height_253.txt", "wb")); f:write(note); f:close()
  return dir .. "/nobu_height_253.r16"
end

function M.main(argv)
  local opts = { source = "C:/nobu/history_nobu", out = b.root .. "/docs/character_pipeline/maps" }
  local i = 1
  while i <= #argv do
    local a = argv[i]
    if a == "--source" or a == "--out" or a == "--heightmap" then opts[a:sub(3)] = argv[i + 1]; i = i + 1
    else error("unknown argument " .. a) end
    i = i + 1
  end
  local data = M.build((opts.source:gsub("\\", "/")))
  b.lib("path").mkdir_p(opts.out)
  local f = assert(io.open(opts.out .. "/nobu.lua", "wb")); f:write(M.render_lua(data)); f:close()
  f = assert(io.open(opts.out .. "/nobu.md", "wb")); f:write(M.render_md(data), "\n"); f:close()
  if opts.heightmap then print("heightmap " .. M.write_heightmap(data, opts.heightmap)) end
  print(("wrote %s: %d lanes, %d towers, %d barracks, %d forts, %d camps, %d trees"):format(opts.out, #data.lanes, #data.towers,
    #data.barracks, #data.forts, #data.neutral_camps, data.tile_grid and #data.tile_grid.trees or 0))
  return 0
end

if rawget(_G, "GEN_NOBU_MAP_AS_MODULE") then return M end
os.exit(M.main(arg))
