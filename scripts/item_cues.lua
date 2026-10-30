-- The cues an item's active plays when it is used (work line C, M7): from each item document's
-- `active.events.on_cast`, every `op = "cue"` at the caster, the target or the point (a cue on a unit
-- found by a loop, `it`, has no place in the use event and is left out). Unreal plays them when the
-- host discloses the use (a cast fact `item:<shelf id>`, OmWorldBridgeActor -> AOmUnitActor::PlayItemCues);
-- a point cue plays at the target, else at the user. One JSON for every documented item, nothing per item.
--
--   tools\lua\lua.exe scripts\item_cues.lua [--out omfue/Content/Omoba/UI/item-cues-v1.json]
package.path = "scripts/?.lua;" .. package.path
local M = {}
M.ITEMS_DIR = "docs/character_pipeline/items"
M.OUT = "omfue/Content/Omoba/UI/item-cues-v1.json"
M.PLACES = { caster = "caster", self = "caster", target = "target", point = "target" }

local function load_doc(path)
  local env = { pairs = pairs, ipairs = ipairs, string = string, table = table, math = math, utf8 = utf8 }
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
function M.table_of(items)
  local out = {}
  for _, it in ipairs(items) do
    local on_cast = it.active and it.active.events and it.active.events.on_cast
    local rows = on_cast and collect(on_cast, {}) or {}
    if #rows > 0 then out[(it.id:gsub("^item_", ""))] = rows end
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
  local names, items = {}, {}
  for name in lfs.dir(M.ITEMS_DIR) do if name:match("%.lua$") then names[#names + 1] = name end end
  table.sort(names)
  for _, name in ipairs(names) do
    for _, it in ipairs(load_doc(M.ITEMS_DIR .. "/" .. name).items or {}) do items[#items + 1] = it end
  end
  local cues = M.table_of(items)
  local n = 0
  for _ in pairs(cues) do n = n + 1 end
  b.lib("json").write(out, { schema = "omoba.item-cues.v1", items = cues }, true)
  print(string.format("item cues: %d items -> %s", n, out))
end

if not ITEM_CUES_AS_MODULE and arg and arg[0] and arg[0]:match("item_cues%.lua$") then M.main(arg) end
return M
