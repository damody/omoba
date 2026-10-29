-- Where the package schema v3 documents live (the authoring source for every history_nobu hero
-- and shop item, shipped or not):
--   docs/character_pipeline/packages/<package>/v3.lua   one hero (kind = "hero")
--   docs/character_pipeline/items/*.lua                  shop items (kind = "items")
-- A shipped hero's package also holds the v2 files (character.lua, abilities.lua, ...); a hero
-- that is not shipped yet has only v3.lua. hero_author, gen_item_v3, the IR compiler, the source
-- catalog, the site and the tests find documents through this file.
local lfs = require("lfs")
local M = {}

M.PACKAGES = "docs/character_pipeline/packages"
M.ITEMS = "docs/character_pipeline/items"
M.HERO_FILE = "v3.lua"

local function sorted_dir(dir)
  local out = {}
  if lfs.attributes(dir, "mode") ~= "directory" then return out end
  for n in lfs.dir(dir) do if n:sub(1, 1) ~= "." then out[#out + 1] = n end end
  table.sort(out)
  return out
end

-- the hero's source id without running the document: `id = "B01"` on the top-level table
local function source_id_of(path)
  local f = io.open(path, "rb")
  if not f then return nil end
  local text = f:read("a")
  f:close()
  return text:match("kind%s*=%s*\"hero\"%s*,%s*id%s*=%s*\"([%w_]+)\"") or text:match("\n%s*id%s*=%s*\"([%w_]+)\"")
end

-- { { source_id, package, path } } sorted by source id; root: repo root (default ".")
function M.heroes(root)
  root = root or "."
  local out = {}
  for _, name in ipairs(sorted_dir(root .. "/" .. M.PACKAGES)) do
    local p = root .. "/" .. M.PACKAGES .. "/" .. name .. "/" .. M.HERO_FILE
    if lfs.attributes(p, "mode") == "file" then
      out[#out + 1] = { source_id = source_id_of(p) or name, package = name, path = p }
    end
  end
  table.sort(out, function(a, b) return a.source_id < b.source_id end)
  return out
end

-- path of one hero's document by source id (or package name); nil when there is none
function M.hero_path(root, id)
  for _, h in ipairs(M.heroes(root)) do
    if h.source_id == id or h.package == id then return h.path end
  end
end

-- item document paths, sorted
function M.items(root)
  root = root or "."
  local out = {}
  for _, n in ipairs(sorted_dir(root .. "/" .. M.ITEMS)) do
    if n:match("%.lua$") then out[#out + 1] = root .. "/" .. M.ITEMS .. "/" .. n end
  end
  return out
end

-- every document path: heroes first, then items
function M.all(root)
  local out = {}
  for _, h in ipairs(M.heroes(root)) do out[#out + 1] = h.path end
  for _, p in ipairs(M.items(root)) do out[#out + 1] = p end
  return out
end

return M
