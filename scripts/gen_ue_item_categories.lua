-- Shop categories for UE (M7, work line C): the same categories, in the same order and with the same
-- names as the website's shop page (scripts/site: the v3 document's shop.category, else the original
-- history_nobu shop file, ordered as that file; names from site/i18n). The UE shop groups the items it
-- sells by them, so the game and the site show one shop. Items without a category keep the UE's own
-- price/recipe tiers.
--
--   tools\lua\lua.exe scripts\gen_ue_item_categories.lua [--check] [--source-root DIR]
--
-- Inputs: target/site/source-catalog.json (scripts/gen_source_catalog.lua) and the original sources.
-- Output: omfue/Content/Omoba/UI/item-categories-v1.json (content data; Lua never runs in the game).
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local json = b.lib("json")

local M = {}
M.SCHEMA = "omoba.item-categories.v1"
M.OUT = "omfue/Content/Omoba/UI/item-categories-v1.json"
M.CATALOG = "target/site/source-catalog.json"

-- catalog: source catalog; sup: site supplement (shop_order, shop_category); strings: lang -> i18n table
function M.build(catalog, sup, strings)
  local items, used = {}, {}
  for _, it in ipairs(catalog.items or {}) do
    local category = it.category or (it.shop and it.shop.category) or (sup and sup.shop_category(it.source_id)) or nil
    if category then
      items[(it.id:gsub("^item_", ""))] = category
      used[category] = true
    end
  end
  local order, seen = {}, {}
  local function add(id)
    if seen[id] or not used[id] then return end
    seen[id] = true
    order[#order + 1] = { id = id, zh_TW = strings.zh_TW.shop_category[id] or id, en = strings.en.shop_category[id] or id }
  end
  for _, c in ipairs(sup and sup.shop_order or {}) do add(c) end
  local rest = {}
  for c in pairs(used) do if not seen[c] then rest[#rest + 1] = c end end
  table.sort(rest)
  for _, c in ipairs(rest) do add(c) end
  return { schema = M.SCHEMA, order = order, items = items }
end

function M.main(args)
  local check, source_root = false, nil
  local i = 1
  while i <= #args do
    if args[i] == "--check" then check = true
    elseif args[i] == "--source-root" then source_root = args[i + 1]; i = i + 1
    else error("unknown argument " .. args[i]) end
    i = i + 1
  end
  local f = assert(io.open(b.root .. "/" .. M.CATALOG, "rb"), "missing " .. M.CATALOG .. "; run scripts/gen_source_catalog.lua")
  local catalog = json.decode(f:read("a")); f:close()
  local sup = require("site.source_supplement").open(source_root)
  local I = require("site.i18n")
  local doc = M.build(catalog, sup.available and sup or nil, { zh_TW = I.strings("zh_TW"), en = I.strings("en") })
  local text = json.encode(doc) .. "\n"
  local path = b.root .. "/" .. M.OUT
  if check then
    local cur = io.open(path, "rb")
    local current = cur and cur:read("a")
    if cur then cur:close() end
    if current ~= text then io.stderr:write(M.OUT .. " is stale; run tools\\lua\\lua.exe scripts\\gen_ue_item_categories.lua\n"); return 1 end
    print("item categories up to date")
    return 0
  end
  local out = assert(io.open(path, "wb")); out:write(text); out:close()
  local n = 0
  for _ in pairs(doc.items) do n = n + 1 end
  print(string.format("wrote %s: %d categories, %d items", M.OUT, #doc.order, n))
  return 0
end

if not ITEM_CATEGORIES_AS_MODULE then os.exit(M.main(arg)) end
return M
