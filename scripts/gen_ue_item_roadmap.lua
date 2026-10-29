-- Items the shop does not sell yet, for UE (M7, work line C): every history_nobu item outside the shelf
-- (templates/item_shop_v3.lua), with its v3 name, price, description and icon, and why it is not sold:
--   blocked   the interpreter cannot load its IR yet: the missing writings and the M7 unlock step
--             (docs/character_pipeline/tools/v3_item_order.lua, the same scan as m7-items.md);
--   ready     its IR loads today; it waits to be put on the shelf;
--   deferred  no IR (the v3 document defers it or the original has no behavior).
-- The shop lists them greyed out and never sells them; their tooltips use the same TooltipRenderer.
--
--   tools\lua\lua.exe scripts\gen_ue_item_roadmap.lua [--check]
--
-- Inputs: target/v3-content/v3-content.json (scripts/gen_v3_content.lua), target/site/source-catalog.json
-- (scripts/gen_source_catalog.lua) and omfue/Content/Omoba/UI/nobu-icons-v1.json.
-- Output: omfue/Content/Omoba/UI/item-roadmap-v1.json (content data; Lua never runs in the game).
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local json = b.lib("json")

local M = {}
M.SCHEMA = "omoba.item-roadmap.v1"
M.OUT = "omfue/Content/Omoba/UI/item-roadmap-v1.json"
M.CONTENT = "target/v3-content/v3-content.json"
M.CATALOG = "target/site/source-catalog.json"
M.ICONS = "omfue/Content/Omoba/UI/nobu-icons-v1.json"
M.SHELF = "scripts/lua_data/templates/item_shop_v3.lua"

-- items, order from v3_item_order.build; source: source catalog; icons: nobu index; shelf: shelf ids
function M.rows(items, source_catalog, icons, shelf)
  local by_source = {}
  for _, it in ipairs(source_catalog.items or {}) do by_source[it.source_id] = it; by_source[it.id] = it end
  local icon_of = {}
  for _, e in ipairs(icons.entries or {}) do if e.kind == "item" and e.status == "original" then icon_of[e.id] = e.file end end
  local rows = {}
  for _, row in ipairs(items) do
    local id = row.id:gsub("^item_", "")
    if not shelf[id] then
      local src = by_source[row.id] or {}
      local v3 = src.v3 or {}
      local out = {
        id = id, source_id = src.source_id or row.id,
        name = { zh_TW = src.name and src.name.zh_TW or row.name, en = src.name and src.name.en or nil },
        cost = src.cost, category = src.category, icon = icon_of[id] or "",
        state = not row.has_ir and "deferred" or (#row.blockers == 0 and "ready" or "blocked"),
        blockers = #row.blockers > 0 and row.blockers or nil,
        unlock_step = type(row.step) == "number" and row.step or nil,
        v3 = { text = v3.text, values = v3.values, info = v3.info, stat_lines = v3.stat_lines },
      }
      rows[#rows + 1] = out
    end
  end
  table.sort(rows, function(x, y)
    local sx, sy = x.unlock_step or 1e9, y.unlock_step or 1e9
    if x.state ~= y.state then
      local rank = { ready = 1, blocked = 2, deferred = 3 }
      return rank[x.state] < rank[y.state]
    end
    if sx ~= sy then return sx < sy end
    return x.id < y.id
  end)
  return rows
end

-- one entry per line so a regeneration diffs by item
function M.encode(doc)
  local items = doc.items
  doc.items = nil
  local head = json.encode(doc)
  doc.items = items
  local lines = {}
  for i, r in ipairs(items) do lines[i] = "  " .. json.encode(r) end
  return head:sub(1, -2) .. ',"items":[\n' .. table.concat(lines, ",\n") .. "\n]}\n"
end

function M.build(root)
  local function read(rel) local f = assert(io.open(root .. "/" .. rel, "rb"), "missing " .. rel); local s = f:read("a"); f:close(); return s end
  V3_ITEM_ORDER_AS_MODULE = true
  local order_tool = dofile(root .. "/docs/character_pipeline/tools/v3_item_order.lua")
  V3_ITEM_ORDER_AS_MODULE = nil
  order_tool.load_rules(root)
  local shelf = require("content_builder").new(root .. "/scripts/lua_data").include("templates/item_shop_v3.lua")
  local content = json.decode(read(M.CONTENT))
  local items, order = order_tool.build(content, shelf)
  -- the unlock step of each item (the order lists what each step completes)
  local step = {}
  for n, s in ipairs(order) do for _, row in ipairs(s.unlocked) do step[row.id] = n end end
  for _, row in ipairs(items) do row.step = step[row.id] end
  local rows = M.rows(items, json.decode(read(M.CATALOG)), json.decode(read(M.ICONS)), shelf)
  return { schema = M.SCHEMA, content_hash = content.content_hash, items = rows }
end

function M.main(args)
  local check = args[1] == "--check"
  local doc = M.build(b.root)
  local text = M.encode(doc)
  local path = b.root .. "/" .. M.OUT
  if check then
    local f = io.open(path, "rb")
    local current = f and f:read("a")
    if f then f:close() end
    if current ~= text then io.stderr:write(M.OUT .. " is stale; run tools\\lua\\lua.exe scripts\\gen_ue_item_roadmap.lua\n"); return 1 end
    print("item roadmap up to date")
    return 0
  end
  local f = assert(io.open(path, "wb")); f:write(text); f:close()
  local counts = {}
  for _, r in ipairs(doc.items) do counts[r.state] = (counts[r.state] or 0) + 1 end
  print(string.format("wrote %s: %d items (ready %d, blocked %d, deferred %d)", M.OUT, #doc.items,
    counts.ready or 0, counts.blocked or 0, counts.deferred or 0))
  return 0
end

if not ITEM_ROADMAP_AS_MODULE then os.exit(M.main(arg)) end
return M
