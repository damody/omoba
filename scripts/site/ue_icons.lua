-- The site shows the same icons as the game's HUD (M6): for each hero, ability and item the art
-- the HUD manifest assigns by hand (a portrait file, an imported ability or item source PNG),
-- else the original nobu art (scripts/import_nobu_icons.lua), else the placeholder. Ids are
-- matched through the nobu index, which records every source id with its game id.
-- wrap(sup, root) returns a supplement whose hero_icon/ability_icon/item_icon give site paths
-- "ue/<kind>-<id>.png"; files[rel] is the absolute source the build resamples into img/.
local M = {}

M.UI_DIR = "omfue/Content/Omoba/UI"

local function read_json(p)
  local f = io.open(p, "rb")
  if not f then return nil end
  local s = f:read("a"); f:close()
  return require("tools.lua.lib.json").decode(s)
end

local function exists(p) local f = io.open(p, "rb"); if f then f:close(); return true end; return false end

-- -> { [kind][source_id] = absolute file } with kind hero|ability|item, or nil without an index
function M.resolve(root)
  local ui = root .. "/" .. M.UI_DIR
  local index = read_json(ui .. "/nobu-icons-v1.json")
  if not index then return nil end
  local manifest = read_json(ui .. "/hud-assets-v1.json") or { entries = {} }
  local authored = { portrait = {}, ability = {}, item = {} }
  for _, row in ipairs(manifest.entries or {}) do
    if authored[row.kind] and row.placeholder == nil and row.source == nil then
      local file
      if row.kind == "portrait" and row.columns == 1 and row.rows == 1 then file = ui .. "/" .. row.file
      elseif row.kind == "ability" and row.resource then file = ui .. "/Abilities/Source/" .. row.file
      elseif row.kind == "item" and row.resource then file = ui .. "/Items/Source/" .. row.file end
      if file and exists(file) then authored[row.kind][row.id] = file end
    end
  end
  local kind_of = { hero = "portrait", ability = "ability", item = "item" }
  local out = { hero = {}, ability = {}, item = {} }
  for _, e in ipairs(index.entries or {}) do
    if e.source_id and out[e.kind] then
      out[e.kind][e.source_id] = { id = e.id,
        file = authored[kind_of[e.kind]][e.id] or (e.status == "original" and e.file and ui .. "/" .. e.file) or nil }
    end
  end
  return out
end

function M.wrap(sup, root)
  local resolved = M.resolve(root)
  if not resolved then return sup, false end
  local w = setmetatable({ files = {} }, { __index = sup })
  local function lookup(kind, source_id)
    local r = source_id and resolved[kind][source_id]
    if not (r and r.file) then return nil end
    local rel = "ue/" .. kind .. "-" .. r.id .. ".png"
    w.files[rel] = r.file
    return rel
  end
  function w.hero_icon(source_id) return lookup("hero", source_id) end
  function w.ability_icon(source_id) return lookup("ability", source_id) end
  function w.item_icon(source_id) return lookup("item", source_id) end
  w.available = true
  return w, true
end

return M
