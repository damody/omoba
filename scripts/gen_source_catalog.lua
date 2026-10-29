-- Source catalog for the website (M3 work line A; v3-content-model-proposal.md §2.5): every
-- history_nobu hero and shop item, compiled from the v3 documents at their authoring locations
-- (docs/character_pipeline/tools/v3_sources.lua: packages/<pkg>/v3.lua, items/*.lua), whether or
-- not the game ships it. The game's own content (V3_CATALOG_JSON) stays limited to accepted
-- heroes; this file only feeds pages ("未上架" heroes are listed, never selectable).
--
--   tools\lua\lua.exe scripts\gen_source_catalog.lua [--root DIR] [--out FILE]
--
-- Output (JSON): { schema, kind = "source", source = { repository, commit }, catalog_hash,
--   heroes = [ { source_id, name = { zh_TW, en }, playable, game_id?, abilities = [ability ids] } ],
--   abilities = [ { id, hero, slot, display_name, max_level, v3 } ],
--   items = [ { id, source_id, name = { zh_TW, en }, cost?, recipe?, category?, deferred?, v3? } ],
--   owners = [ { id, kind = shared|template|status, doc, values } ] }
-- `owners` holds the values text pieces read outside any ability or item block: a document's own
-- `values` (owner id `<document id>.shared`; the blocks' pieces are renamed to match), its unit
-- templates and document-level statuses. Every owner id is unique in the catalog.
-- recipe = { components = [item ids], recipe_cost = value name } and category come from the document's
-- shop (category only where the document writes one).
-- `v3` is the same block as in V3_CATALOG_JSON (values, info, text pieces), without `ir`.
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local lfs = require("lfs")
local json = b.lib("json")
local sha256 = require("hero_source.sha256")

local M = {}
M.OUT = "target/site/source-catalog.json"

local function tools(root)
  PACKAGE_V3_AS_MODULE = true
  local v = dofile(root .. "/docs/character_pipeline/tools/package_v3.lua")
  PACKAGE_V3_AS_MODULE = nil
  return v, dofile(root .. "/docs/character_pipeline/tools/v3_compile.lua"), dofile(root .. "/docs/character_pipeline/tools/v3_sources.lua")
end

-- source hero id -> game hero id, for heroes the game ships (packages with an identity)
function M.playable(packages_dir)
  local map = {}
  if lfs.attributes(packages_dir, "mode") ~= "directory" then return map end
  local env = { pairs = pairs, ipairs = ipairs, string = string, table = table, math = math, utf8 = utf8 }
  for name in lfs.dir(packages_dir) do
    local p = packages_dir .. "/" .. name .. "/character.lua"
    local chunk = lfs.attributes(p) and loadfile(p, "t", env)
    if chunk then
      local ok, v = pcall(chunk)
      if ok and type(v) == "function" then ok, v = pcall(v, {}) end
      local id = ok and type(v) == "table" and v.identity and v.identity.source_hero_id
      if id and v.hero and v.hero.id then map[id] = v.hero.id end
    end
  end
  return map
end

-- root: the repo root (default: this checkout)
function M.build(root)
  root = root or b.root
  local validator, compiler, sources = tools(root)
  local problems = {}
  local playable = M.playable(root .. "/" .. sources.PACKAGES)
  local names = compiler.item_names(root .. "/" .. sources.ITEMS)
  local catalog = { schema = compiler.SCHEMA, kind = "source", heroes = {}, abilities = {}, items = {}, owners = {} }
  local owner_seen = {}
  -- the document-level owner is "shared" in every document: give it the document's id
  local function rename_shared(t, new_id)
    if type(t) ~= "table" then return end
    if t.owner == "shared" then t.owner = new_id end
    for _, v in pairs(t) do rename_shared(v, new_id) end
  end
  local function add_owners(doc, compiled, file)
    local shared_id = tostring(doc.id) .. ".shared"
    for _, block in pairs(compiled.abilities) do rename_shared(block, shared_id) end
    for _, block in pairs(compiled.items) do rename_shared(block, shared_id) end
    local ids = {}
    for id in pairs(compiled.owner_values or {}) do ids[#ids + 1] = id end
    table.sort(ids)
    for _, id in ipairs(ids) do
      local row = compiled.owner_values[id]
      local oid = id == "shared" and shared_id or id
      if owner_seen[oid] then problems[#problems + 1] = file .. ": owner " .. oid .. " also in " .. owner_seen[oid]
      else
        owner_seen[oid] = file
        rename_shared(row.values, shared_id)
        catalog.owners[#catalog.owners + 1] = { id = oid, kind = row.kind, doc = doc.id, values = row.values }
      end
    end
  end
  local hero_files = {}
  for _, h in ipairs(sources.heroes(root)) do hero_files[#hero_files + 1] = h.path end
  for _, kind in ipairs({ "heroes", "items" }) do
    for _, file in ipairs(kind == "heroes" and hero_files or sources.items(root)) do
      local doc, err = validator.load_file(file)
      if not doc then problems[#problems + 1] = file .. ": " .. tostring(err)
      else
        local errs = validator.validate_doc(doc)
        if #errs > 0 then problems[#problems + 1] = file .. ": " .. errs[1]
        else
          catalog.source = catalog.source or { repository = doc.source.repository, commit = doc.source.commit }
          local ok, compiled = pcall(compiler.compile, doc, { names = names, ir = false })
          if ok then add_owners(doc, compiled, file) end
          if not ok then problems[#problems + 1] = file .. ": " .. tostring(compiled)
          elseif kind == "heroes" then
            local h = doc.hero or {}
            local row = { source_id = h.source_id or doc.id, name = { zh_TW = h.text and h.text.zh_TW and h.text.zh_TW.name,
              en = h.text and h.text.en and h.text.en.name }, playable = playable[h.source_id or doc.id] ~= nil,
              game_id = playable[h.source_id or doc.id], abilities = {} }
            for _, a in ipairs(doc.abilities or {}) do
              row.abilities[#row.abilities + 1] = a.id
              local block = compiled.abilities[a.id]
              catalog.abilities[#catalog.abilities + 1] = { id = a.id, hero = row.source_id, slot = a.slot,
                display_name = a.text and a.text.zh_TW and a.text.zh_TW.name, max_level = a.max_level, v3 = block }
            end
            catalog.heroes[#catalog.heroes + 1] = row
          else
            for _, it in ipairs(doc.items or {}) do
              local block = compiled.items[it.id]
              local cost
              for _, v in ipairs(block and block.values or {}) do
                if block.info then
                  for _, r in ipairs(block.info) do
                    if r.field == "cost" and r.value == v.name and v.ranks then cost = tonumber((v.ranks[1]:gsub("/.*", ""))) end
                  end
                end
              end
              local shop, recipe = it.shop or {}, nil
              if type(shop.recipe) == "table" and type(shop.recipe.components) == "table" and #shop.recipe.components > 0 then
                recipe = { components = { table.unpack(shop.recipe.components) } }
                if type(shop.recipe.recipe_cost) == "string" then recipe.recipe_cost = shop.recipe.recipe_cost:match("^%$([%w_]+)$") end
              end
              catalog.items[#catalog.items + 1] = { id = it.id, source_id = it.source_id,
                name = { zh_TW = it.text and it.text.zh_TW and it.text.zh_TW.name, en = it.text and it.text.en and it.text.en.name },
                cost = cost, recipe = recipe, category = type(shop.category) == "string" and shop.category or nil,
                deferred = it.coverage == "deferred" and (it.deferred_reason or true) or nil, v3 = block }
            end
            for _, d in ipairs(doc.deferred or {}) do
              catalog.items[#catalog.items + 1] = { id = d.id or d.source_id:lower(), source_id = d.source_id,
                deferred = d.reason, note = d.note }
            end
          end
        end
      end
    end
  end
  table.sort(catalog.heroes, function(x, y) return x.source_id < y.source_id end)
  table.sort(catalog.items, function(x, y) return x.id < y.id end)
  table.sort(catalog.owners, function(x, y) return x.id < y.id end)
  -- the hash covers everything but itself
  catalog.catalog_hash = sha256(json.encode(catalog)):sub(1, 16)
  return catalog, problems
end

function M.main(argv)
  local opts = {}
  local i = 1
  while i <= #argv do
    local a = argv[i]
    if a == "--root" or a == "--out" then opts[a:sub(3)] = argv[i + 1]; i = i + 1
    else error("unknown argument " .. a) end
    i = i + 1
  end
  local catalog, problems = M.build(opts.root)
  if #problems > 0 then
    for _, p in ipairs(problems) do io.stderr:write(p, "\n") end
    io.stderr:write(#problems .. " problem(s)\n")
    return 1
  end
  local out = opts.out or (b.root .. "/" .. M.OUT)
  local dir = out:gsub("\\", "/"):match("^(.*)/")
  if dir then b.lib("path").mkdir_p(dir) end
  local f = assert(io.open(out, "wb")); f:write(json.encode(catalog)); f:close()
  local playable = 0
  for _, h in ipairs(catalog.heroes) do if h.playable then playable = playable + 1 end end
  print(("wrote %s: %d heroes (%d playable), %d abilities, %d items, hash %s"):format(out, #catalog.heroes, playable,
    #catalog.abilities, #catalog.items, catalog.catalog_hash))
  return 0
end

if rawget(_G, "GEN_SOURCE_CATALOG_AS_MODULE") then return M end
os.exit(M.main(arg))
