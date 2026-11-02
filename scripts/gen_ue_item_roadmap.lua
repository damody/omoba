-- Items the shop does not sell yet, for UE (M7, work line C): every history_nobu item outside the shelf
-- (templates/item_shop_v3.lua), with its v3 name, price, description and icon, and why it is not sold:
--   blocked   the generator cannot write its Rust yet: the forms it does not lower
--             (docs/character_pipeline/tools/v3_codegen_coverage.lua, the same scan as codegen-coverage.md);
--   ready     it generates today; it waits to be put on the shelf;
--   deferred  the original has no such item (no source row: the shop file names it, its KV is missing,
--             commented out or only a recipe), so it is never sold.
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

-- What players read for a blocker: its family, never the engine code (kept in blocker_codes).
M.BLOCKER_FAMILY = {
  { "^passive:aura", "光環" },
  { "^aura", "光環" },
  { "^passive_trigger:", "被動觸發" },
  { "^passive:", "被動效果" },
  { "^action:", "特殊動作" },
  { "^stat:", "屬性" },
  { "^state", "狀態" },
  { "^status", "狀態" },
  { "^event:", "觸發時機" },
  { "", "條件判斷" }, -- fn:, attr:, token: and anything new
}
function M.blocker_labels(codes)
  local out, seen = {}, {}
  for _, code in ipairs(codes) do
    for _, rule in ipairs(M.BLOCKER_FAMILY) do
      if code:find(rule[1]) then
        if not seen[rule[2]] then seen[rule[2]] = true; out[#out + 1] = rule[2] end
        break
      end
    end
  end
  return out
end

-- items: { id, name, blockers } per item; source: source catalog; icons: nobu index; shelf: shelf ids
function M.rows(items, source_catalog, icons, shelf, source_rows)
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
        state = source_rows and not source_rows[row.id:lower()] and "deferred" or (#row.blockers == 0 and "ready" or "blocked"),
        blockers = #row.blockers > 0 and M.blocker_labels(row.blockers) or nil,
        blocker_codes = #row.blockers > 0 and row.blockers or nil,
        v3 = { text = v3.text, values = v3.values, info = v3.info, stat_lines = v3.stat_lines },
      }
      rows[#rows + 1] = out
    end
  end
  table.sort(rows, function(x, y)
    if x.state ~= y.state then
      local rank = { ready = 1, blocked = 2, deferred = 3 }
      return rank[x.state] < rank[y.state]
    end
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
  V3_CODEGEN_COVERAGE_AS_MODULE = true
  local coverage = dofile(root .. "/docs/character_pipeline/tools/v3_codegen_coverage.lua")
  V3_CODEGEN_COVERAGE_AS_MODULE = nil
  local shelf = require("content_builder").new(root .. "/scripts/lua_data").include("templates/item_shop_v3.lua")
  local content = json.decode(read(M.CONTENT))
  -- "not in the shop" is why the generator skips an unsold item, not what it still lacks
  local items = {}
  for _, entry in ipairs(coverage.scan(root).items) do
    local blockers = {}
    for _, blocker in ipairs(entry.blockers) do
      local code = tostring(type(blocker) == "table" and (blocker.feature or blocker[1]) or blocker)
      if not code:find("is not in the shop", 1, true) then blockers[#blockers + 1] = code end
    end
    table.sort(blockers)
    items[#items + 1] = { id = entry.id, name = entry.id, blockers = blockers }
  end
  table.sort(items, function(x, y) return x.id < y.id end)
  local source_rows = {}
  for _, row in ipairs(require("content_builder").new(root .. "/scripts/lua_data").include("templates/source_items.lua").items) do
    source_rows[row.key:lower():gsub("'", "")] = true
  end
  local rows = M.rows(items, json.decode(read(M.CATALOG)), json.decode(read(M.ICONS)), shelf, source_rows)
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
