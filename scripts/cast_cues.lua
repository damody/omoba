-- The cues a cast plays when it is disclosed (work line C, M7): every `op = "cue"` of an item's active
-- (`active.events.on_cast`, item documents) and of a hero ability's `events.on_cast` (packages/<hero>/v3.lua),
-- at the caster or at the target. A point cue plays at the target; a cue on a unit found by a loop
-- (`it`) has no place in the cast event and is left out. Unreal plays the caster rows when the host
-- discloses the use of an item (cast fact `item:<shelf id>`, AOmUnitActor::PlayItemCues) and the target
-- rows where any cast landed (the cast's target fact, AOmUnitActor::PlayCastTargetCues), for the teams
-- that see the target. One JSON for every item and hero; nothing per item or ability.
--
--   tools\lua\lua.exe scripts\cast_cues.lua [--out omfue/Content/Omoba/UI/cast-cues-v1.json]
package.path = "scripts/?.lua;" .. package.path
local M = {}
M.ITEMS_DIR = "docs/character_pipeline/items"
M.PACKAGES_DIR = "docs/character_pipeline/packages"
M.OUT = "omfue/Content/Omoba/UI/cast-cues-v1.json"
M.PLACES = { caster = "caster", self = "caster", target = "target", point = "target" }

local function load_doc(path)
  local env = { pairs = pairs, ipairs = ipairs, string = string, table = table, math = math, utf8 = utf8,
    tostring = tostring, tonumber = tonumber, type = type, select = select, next = next, error = error, assert = assert }
  local v = assert(loadfile(path, "t", env))()
  if type(v) == "function" then v = v({}) end
  return v
end

-- cues of an action list in order; `for_each` bodies are skipped (their cues are at `it`)
local function collect(actions, out)
  for _, a in ipairs(actions or {}) do
    if a.op == "cue" and type(a.id) == "string" and M.PLACES[a.at or "caster"] then
      out[#out + 1] = { cue = a.id, at = M.PLACES[a.at or "caster"] }
    elseif a.op ~= "for_each" then
      for _, key in ipairs({ "actions", "then_actions", "else_actions", "otherwise" }) do
        if type(a[key]) == "table" then collect(a[key], out) end
      end
    end
  end
  return out
end

-- shelf id -> ordered cue rows, for every item document with an active that plays a cue
function M.items_of(items)
  local out = {}
  for _, it in ipairs(items) do
    local on_cast = it.active and it.active.events and it.active.events.on_cast
    local rows = on_cast and collect(on_cast, {}) or {}
    if #rows > 0 then out[(it.id:gsub("^item_", ""))] = rows end
  end
  return out
end

-- ability id -> ordered cue rows, for every hero ability whose on_cast plays a cue
function M.abilities_of(heroes)
  local out = {}
  for _, hero in ipairs(heroes) do
    for _, a in ipairs(hero.abilities or {}) do
      local on_cast = type(a.id) == "string" and a.events and a.events.on_cast
      local rows = on_cast and collect(on_cast, {}) or {}
      if #rows > 0 then out[a.id] = rows end
    end
  end
  return out
end

function M.main(args)
  local out = M.OUT
  local i = 1
  while i <= #args do
    if args[i] == "--out" then i = i + 1; out = args[i] else error("unknown argument: " .. tostring(args[i])) end
    i = i + 1
  end
  local b = require("_bootstrap")
  local lfs = require("lfs")
  local function sorted(dir, keep)
    local names = {}
    for name in lfs.dir(dir) do if keep(name) then names[#names + 1] = name end end
    table.sort(names)
    return names
  end
  local items, heroes = {}, {}
  for _, name in ipairs(sorted(M.ITEMS_DIR, function(name) return name:match("%.lua$") end)) do
    for _, it in ipairs(load_doc(M.ITEMS_DIR .. "/" .. name).items or {}) do items[#items + 1] = it end
  end
  for _, name in ipairs(sorted(M.PACKAGES_DIR, function(name)
    return lfs.attributes(M.PACKAGES_DIR .. "/" .. name .. "/v3.lua", "mode") == "file" end)) do
    heroes[#heroes + 1] = load_doc(M.PACKAGES_DIR .. "/" .. name .. "/v3.lua")
  end
  local cues = { schema = "omoba.cast-cues.v1", items = M.items_of(items), abilities = M.abilities_of(heroes) }
  local function count(t) local n = 0; for _ in pairs(t) do n = n + 1 end; return n end
  b.lib("json").write(out, cues, true)
  print(string.format("cast cues: %d items, %d abilities -> %s", count(cues.items), count(cues.abilities), out))
end

if not CAST_CUES_AS_MODULE and arg and arg[0] and arg[0]:match("cast_cues%.lua$") then M.main(arg) end
return M
