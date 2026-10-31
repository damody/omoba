-- Item spotlight (work line C): one hero uses each named item's active on the hero spotlight
-- stage, so the UE presentation of items the bots never buy can be checked in a live run. The
-- items start in the hero's inventory (showcase start_items, placed by the shop's purchase rules
-- without gold); the renderer autopilot's `spotlight` profile presses each item slot's use key
-- (FOmSpotlightScript item slots).
--
--   tools\lua\lua.exe scripts\item_spotlight.lua --items item_napalm_bomb,item_sake --out DIR [--hero HERO]
--
-- Writes DIR/item-spotlight-recipe.lua (scripts/lua_data/moba_hero_spotlight.lua plus start_items)
-- and DIR/item-spotlight-script.json; run them with run_moba_split_local.lua --recipe RECIPE
-- --autopilot spotlight --spotlight-script SCRIPT.
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local json, path = b.lib("json"), b.lib("path")

local M = {}
M.CATALOG = "target/site/source-catalog.json"
M.STAGE = "scripts/lua_data/moba_hero_spotlight.lua"
M.PLAYER = 1

-- How the bot uses an item or ability (catalog v3.bot.kind) -> who the demonstration aims at and
-- the target shape; every kind of docs/character_pipeline/tools/v3_bot.lua. A passive item (no bot
-- use) has nothing to press. channel_enemy_area channels around the caster (a point-target channel
-- is enemy_point).
M.AIM = {
  enemy_point = { team = "enemy", target = "point" },
  global_enemy_point = { team = "enemy", target = "point" },
  approach_enemy_point = { team = "enemy", target = "point" },
  leap_enemy_point = { team = "enemy", target = "point" },
  enemy_line = { team = "enemy", target = "point" },
  enemy_cone = { team = "enemy", target = "point" },
  channel_enemy_area = { team = "self", target = "none" },
  summon_near_enemy = { team = "enemy", target = "point" },
  enemy_unit = { team = "enemy", target = "unit" },
  enemy_hero = { team = "enemy", target = "unit" },
  ally_heal = { team = "ally", target = "unit" },
  ally_heal_or_cleanse = { team = "ally", target = "unit" },
  ally_engage = { team = "ally", target = "unit" },
  self_heal = { team = "self", target = "none" },
  self_channel_recovery = { team = "self", target = "none" },
  self_mana_regeneration = { team = "self", target = "none" },
  escape = { team = "self", target = "none" },
  toggle_engage = { team = "self", target = "none" },
  self_engage = { team = "self", target = "none" },
  self_area = { team = "self", target = "none" },
  self_recovery = { team = "self", target = "none" },
  -- A tree-target cast (B22 自然之力): shown on the tree stage (record_hero_spotlight.TREE_RECIPE).
  nearest_tree = { team = "tree", target = "point" },
}

local function first_line(text)
  return require("record_hero_spotlight").first_line(text)
end

-- catalog, item ids -> spotlight slots (item_slot i-1 for the i-th item)
-- A combined item is bought from its components (the shop refuses it without them): catalog,
-- items -> the start list with every recipe's components first (depth first), and the inventory
-- slot each shown item ends in, as the shop places it (components consumed, then the first free slot).
function M.placement(catalog, items)
  local by_id = {}
  for _, item in ipairs(catalog.items) do by_id[item.id] = item end
  local start, inventory = {}, {}
  local function free() for slot = 1, 6 do if not inventory[slot] then return slot end end error("an item spotlight inventory holds 6 items") end
  local function add(id)
    local item = assert(by_id[id], "unknown item " .. id .. " (not in " .. M.CATALOG .. ")")
    local components = item.recipe and item.recipe.components or {}
    for _, component in ipairs(components) do add(component) end
    for _, component in ipairs(components) do
      for slot = 1, 6 do if inventory[slot] == component then inventory[slot] = nil; break end end
    end
    inventory[free()] = id
    start[#start + 1] = id
  end
  for _, id in ipairs(items) do add(id) end
  local slot_of = {}
  for slot = 1, 6 do if inventory[slot] then slot_of[inventory[slot]] = slot - 1 end end
  return start, slot_of, by_id
end

function M.slots(catalog, items)
  local _, slot_of, by_id = M.placement(catalog, items)
  local slots = {}
  for _, id in ipairs(items) do
    local item = assert(by_id[id], "unknown item " .. id .. " (not in " .. M.CATALOG .. ")")
    local bot = item.v3 and item.v3.bot
    local aim = bot and M.AIM[bot.kind]
    assert(aim, id .. " has no active the spotlight can press (bot kind " .. tostring(bot and bot.kind) .. ")")
    local text = item.v3.text and (item.v3.text.zh_TW or item.v3.text.en) or {}
    local slot = assert(slot_of[id], id .. " is consumed by another shown item's recipe; show them in separate runs")
    slots[#slots + 1] = { item_slot = slot, id = id, name = text.name or item.name or id,
      line = first_line(text.rendered or text.body), kind = "item", team = aim.team, target = aim.target }
  end
  assert(#slots >= 1 and #slots <= 6, "an item spotlight shows 1..6 items")
  return slots
end

function M.script(hero, slots)
  return { schema = 1, hero = hero, hero_name = hero, title = "道具展示",
    timing = json.object({ caption_lead = 1.5, hold = 4.5, passive = 5 }), slots = slots }
end

local function lua_value(value, indent)
  indent = indent or ""
  if type(value) == "table" then
    local keys = {}
    for key in pairs(value) do keys[#keys + 1] = key end
    table.sort(keys, function(a, c) return tostring(a) < tostring(c) end)
    local out = {}
    for _, key in ipairs(keys) do
      local name = type(key) == "number" and ("[" .. key .. "]") or ("[" .. string.format("%q", key) .. "]")
      out[#out + 1] = indent .. "  " .. name .. "=" .. lua_value(value[key], indent .. "  ")
    end
    return "{\n" .. table.concat(out, ",\n") .. "\n" .. indent .. "}"
  elseif type(value) == "string" then return string.format("%q", value)
  else return tostring(value) end
end

-- The authority's item id of a catalog item (the catalog prefixes source items with `item_`).
function M.registry_id(id) return (id:gsub("^item_", "")) end

-- The hero spotlight stage with HERO on the demonstrator's seat and ITEMS (catalog ids) in its inventory.
function M.recipe(stage, hero, items)
  for _, seat in ipairs(stage.players) do if seat.player_id == M.PLAYER then seat.hero = hero end end
  local start = {}
  for index, id in ipairs(items) do start[index] = M.registry_id(id) end
  stage.showcase.start_items = { [tostring(M.PLAYER)] = start }
  return "-- Generated by scripts/item_spotlight.lua from " .. M.STAGE .. "; do not edit.\nreturn " .. lua_value(stage) .. "\n"
end

function M.main(args)
  local items, out, hero = nil, nil, "saika_magoichi"
  local i = 1
  while i <= #args do
    if args[i] == "--items" then
      items = {}
      for id in args[i + 1]:gmatch("[^,]+") do items[#items + 1] = id:find("^item_") and id or ("item_" .. id) end
      i = i + 1
    elseif args[i] == "--out" then out = args[i + 1]; i = i + 1
    elseif args[i] == "--hero" then hero = args[i + 1]; i = i + 1
    else error("unknown argument " .. args[i]) end
    i = i + 1
  end
  assert(items and out, "usage: item_spotlight.lua --items ID[,ID...] --out DIR [--hero HERO]")
  local function read(p) local f = assert(io.open(p, "rb"), p); local s = f:read("a"); f:close(); return s end
  local catalog = json.decode(read(path.join(b.root, M.CATALOG)))
  local slots = M.slots(catalog, items)
  path.mkdir_p(out)
  path.write(path.join(out, "item-spotlight-script.json"), json.encode(M.script(hero, slots)), true)
  local stage = assert(loadfile(path.join(b.root, M.STAGE)))()
  path.write(path.join(out, "item-spotlight-recipe.lua"), M.recipe(stage, hero, (M.placement(catalog, items))), true)
  print("item spotlight: " .. #slots .. " item(s) for " .. hero .. " in " .. out)
  return 0
end

if not ITEM_SPOTLIGHT_AS_MODULE then os.exit(M.main(arg)) end
return M
