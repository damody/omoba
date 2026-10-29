-- Item texts and values from package schema v3 (M1, docs/character_pipeline/schema/
-- v3-content-model-proposal.md §2.4). Build-time only; the game never runs Lua.
--
--   tools\lua\lua.exe scripts\gen_item_v3.lua [--check] [--content-root DIR] [--items-dir DIR] [--out FILE]
--
-- Reads every v3 document in docs/character_pipeline/items/*.lua (kind = "items"; every
-- history_nobu shop item, the website's source catalog reads the same files), validates and
-- compiles it, and checks it against the shelf (templates/moba_items.lua):
--   * every shelf item has exactly one v3 entry (matched by source_id = the source key);
--     items not on the shelf are catalog-only and skipped here;
--   * every number of the shelf row (cost, stats, modifiers, active, target, cooldown, ...)
--     is mapped by a v3 value's `v2_key` (its dotted path in the row) with the same number,
--     so the shelf numbers and the texts cannot drift apart.
-- Writes templates/item_texts_v3.lua: { [shelf id] = v3 block }. moba_items.lua attaches the
-- block as `v3` and replaces the description with the rendered zh_TW template.
-- Writes templates/item_shop_v3.lua: { [shelf id] = { numbers = { [path] = n }, recipe = { id, ... } } },
-- every number of the shelf row and the recipe resolved from the v3 values and `shop.recipe`.
-- moba_items.lua takes the row's numbers and recipe from it, so the v3 documents are the single
-- source of prices, recipes and stats (value-single-source §3.3); the source catalog export only
-- shapes the row, and this script compares the v3 numbers with it (a difference needs
-- `source_differs` on the value).
-- --check verifies both files are current instead of writing them.
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local lfs = require("lfs")
local serialize = require("hero_source.serialize")

local M = {}
M.OUT = "templates/item_texts_v3.lua"
M.SHOP_OUT = "templates/item_shop_v3.lua"
-- Row fields that are identity, wiring or presentation, not gameplay numbers.
M.SKIP = { catalog_id = true, recipe = true, shares_cooldown_with = true, id = true, name = true,
  description = true, source_ref = true, v3 = true, test_only = true, purchase_group = true }

local function tools()
  PACKAGE_V3_AS_MODULE = true
  local v = dofile(b.root .. "/docs/character_pipeline/tools/package_v3.lua")
  PACKAGE_V3_AS_MODULE = nil
  return v, dofile(b.root .. "/docs/character_pipeline/tools/v3_compile.lua")
end

-- number leaves of a row: { ["active.damage"] = 300, cost = 150, ... }
function M.numbers(row)
  local out = {}
  local function walk(t, prefix)
    for k, v in pairs(t) do
      if not (prefix == "" and M.SKIP[k]) then
        local path = prefix == "" and tostring(k) or (prefix .. "." .. tostring(k))
        if type(v) == "number" then out[path] = v
        elseif type(v) == "table" then walk(v, path) end
      end
    end
  end
  walk(row, "")
  return out
end

local function same(a, b) return math.abs(a - b) <= 1e-6 * math.max(1, math.abs(a), math.abs(b)) end

-- shelf rows by source key (item_xxx); docs: list of v3 documents
function M.build(shelf, docs, partial)
  local validator, compiler = tools()
  local problems, blocks, seen, shops = {}, {}, {}, {}
  local by_key = {}
  for _, row in ipairs(shelf) do
    if not row.test_only and row.source_ref then by_key[row.source_ref:match(":(item_[%w_]+)$")] = row end
  end
  for _, d in ipairs(docs) do
    local errs = validator.validate_doc(d.doc)
    for _, e in ipairs(errs) do problems[#problems + 1] = d.path .. ": " .. e end
    if #errs == 0 then
      assert(d.doc.kind == "items", d.path .. ": item documents must be kind = \"items\"")
      local names = {}
      for _, other in ipairs(docs) do
        for _, it in ipairs(other.doc.items or {}) do if it.id and it.text then names[it.id] = it.text end end
      end
      local compiled = compiler.compile(d.doc, { names = names })
      for _, item in ipairs(d.doc.items or {}) do
        local row = by_key[item.source_id]
        if not row then
          -- catalog-only (not on the shelf yet)
        elseif seen[row.id] then
          problems[#problems + 1] = d.path .. ": " .. item.id .. ": shelf item " .. row.id .. " already has a v3 entry in " .. seen[row.id]
        else
          seen[row.id] = d.path
          local block = compiled.items[item.id]
          local mapped = {}
          for _, value in ipairs(block.values or {}) do
            if value.v2_key then
              mapped[value.v2_key] = value
            end
          end
          local nums = M.numbers(row)
          local shop = { numbers = {} }
          for path, n in pairs(nums) do
            local value = mapped[path]
            if not value then
              problems[#problems + 1] = d.path .. ": " .. item.id .. ": shelf number " .. path .. " = " .. n .. " has no v3 value (v2_key = \"" .. path .. "\")"
            elseif not value.ranks then
              problems[#problems + 1] = d.path .. ": " .. item.id .. ": value " .. value.name .. " maps " .. path .. " but has no static ranks"
            else
              local v = compiler.value_of(value.ranks[1])
              -- the shelf stores some percents as fractions (0.075); a pct value is the percent
              local fraction = value.unit == "pct" and (same(v / 100, n) or (not same(v, n) and math.abs(n) < 1))
              local out = fraction and v / 100 or v
              if math.type(n) == "integer" and out == math.floor(out) then out = math.tointeger(out) end
              if not same(out, n) and not item.values[value.name].source_differs then
                problems[#problems + 1] = d.path .. ": " .. item.id .. ": v3 " .. value.name .. " = " .. value.ranks[1] ..
                  " differs from the source catalog " .. path .. " = " .. n .. " (write source_differs when it is meant)"
              end
              shop.numbers[path] = out
            end
          end
          -- the recipe: v3 shop.recipe.components (item ids) against the source row (game ids)
          local recipe = item.shop and item.shop.recipe and item.shop.recipe.components
          if recipe then
            shop.recipe = {}
            for i, c in ipairs(recipe) do shop.recipe[i] = (tostring(c):gsub("^item_", "")) end
          end
          if table.concat(shop.recipe or {}, ",") ~= table.concat(row.recipe or {}, ",") then
            problems[#problems + 1] = d.path .. ": " .. item.id .. ": v3 recipe {" .. table.concat(shop.recipe or {}, ", ") ..
              "} differs from the source catalog {" .. table.concat(row.recipe or {}, ", ") .. "}"
          end
          shops[row.id] = shop
          for path, value in pairs(mapped) do
            if nums[path] == nil then
              problems[#problems + 1] = d.path .. ": " .. item.id .. ": value " .. value.name .. " v2_key " .. path .. " is not a number of the shelf row"
            end
          end
          blocks[row.id] = block
        end
      end
    end
  end
  for key, row in pairs(by_key) do
    if not partial and not seen[row.id] then problems[#problems + 1] = "shelf item " .. row.id .. " (" .. key .. ") has no v3 entry" end
  end
  table.sort(problems)
  return blocks, problems, shops
end

function M.render(blocks, header)
  header = header or ("-- GENERATED by scripts/gen_item_v3.lua from docs/character_pipeline/items/*.lua; do not edit.\n" ..
    "-- Shelf item id -> package schema v3 block (values, info line, text); see templates/moba_items.lua.\n")
  local body = serialize.serialize(blocks, header)
  return (body:gsub("^(.-)return ", "%1local data = ", 1)) .. "return function(ctx) return data end\n"
end

local function load_doc(path)
  local env = { pairs = pairs, ipairs = ipairs, string = string, table = table, math = math, utf8 = utf8 }
  local chunk = assert(loadfile(path, "t", env))
  local v = chunk()
  if type(v) == "function" then v = v({}) end
  return v
end

function M.main(argv)
  local opts = {}
  local i = 1
  while i <= #argv do
    local a = argv[i]
    if a == "--check" then opts.check = true
    elseif a == "--content-root" then opts.content_root = argv[i + 1]; i = i + 1
    elseif a == "--items-dir" then opts.items_dir = argv[i + 1]; i = i + 1
    elseif a == "--out" then opts.out = argv[i + 1]; i = i + 1
    elseif a == "--partial" then opts.partial = true
    else error("unknown argument " .. a) end
    i = i + 1
  end
  local content_root = opts.content_root or (b.root .. "/scripts/lua_data")
  local items_dir = opts.items_dir or (b.root .. "/docs/character_pipeline/items")
  -- the shelf as the source catalog shapes it: the comparison runs before the v3 numbers replace its own
  local ctx = require("content_builder").new(content_root)
  ctx.source_shelf = true
  local shelf = ctx.include("templates/moba_items.lua")
  local docs = {}
  if lfs.attributes(items_dir, "mode") == "directory" then
    local names = {}
    for name in lfs.dir(items_dir) do if name:match("%.lua$") then names[#names + 1] = name end end
    table.sort(names)
    for _, name in ipairs(names) do docs[#docs + 1] = { path = name, doc = load_doc(items_dir .. "/" .. name) } end
  end
  local blocks, problems, shops = M.build(shelf, docs, opts.partial)
  if #problems > 0 then
    for _, p in ipairs(problems) do io.stderr:write(p, "\n") end
    io.stderr:write(#problems .. " problem(s)\n")
    return 1
  end
  local out = opts.out or (content_root .. "/" .. M.OUT)
  local shop_out = opts.out and (opts.out:gsub("[^/\\]+$", "item_shop_v3.lua")) or (content_root .. "/" .. M.SHOP_OUT)
  local files = {
    { out, M.render(blocks) },
    { shop_out, M.render(shops, "-- GENERATED by scripts/gen_item_v3.lua from docs/character_pipeline/items/*.lua; do not edit.\n" ..
      "-- Shelf item id -> the row's numbers (path = value) and recipe from the v3 values; see templates/moba_items.lua.\n") },
  }
  if opts.check then
    for _, file in ipairs(files) do
      local f = io.open(file[1], "rb")
      local current = f and f:read("a"); if f then f:close() end
      if current ~= file[2] then io.stderr:write(file[1] .. " is stale; run scripts/gen_item_v3.lua\n"); return 1 end
      print("OK: " .. file[1] .. " is current")
    end
    return 0
  end
  for _, file in ipairs(files) do
    local f = assert(io.open(file[1], "wb")); f:write(file[2]); f:close()
  end
  local n = 0 for _ in pairs(blocks) do n = n + 1 end
  print("wrote " .. out .. " and " .. shop_out .. " (" .. n .. " items)")
  return 0
end

if rawget(_G, "GEN_ITEM_V3_AS_MODULE") then return M end
os.exit(M.main(arg))
