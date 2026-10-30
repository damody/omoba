-- The cue a status shows while it lasts (work line C, M7): from every hero package (packages/<hero>/v3.lua)
-- and item document, each status with a `cue`, keyed by the buff it lives as (`runtime_id`, else `id`,
-- as the generated Rust names it). Unreal shows the cue for as long as the unit carries that buff
-- (AOmUnitActor::OnBuffState). One table for heroes and items alike; nothing per status.
--
--   tools\lua\lua.exe scripts\status_cues.lua [--out omfue/Content/Omoba/UI/status-cues-v1.json]
package.path = "scripts/?.lua;" .. package.path
local M = {}
M.PACKAGES_DIR = "docs/character_pipeline/packages"
M.ITEMS_DIR = "docs/character_pipeline/items"
M.OUT = "omfue/Content/Omoba/UI/status-cues-v1.json"

local function load_doc(path)
  local env = { pairs = pairs, ipairs = ipairs, string = string, table = table, math = math, utf8 = utf8,
    tostring = tostring, tonumber = tonumber, type = type, select = select, next = next, error = error, assert = assert }
  local v = assert(loadfile(path, "t", env))()
  if type(v) == "function" then v = v({}) end
  return v
end

-- every `statuses` list anywhere in a document (an ability's, an item's or the document's own)
local function each_status(node, fn, seen)
  if type(node) ~= "table" or seen[node] then return end
  seen[node] = true
  if type(node.statuses) == "table" then
    for _, s in ipairs(node.statuses) do if type(s) == "table" then fn(s) end end
  end
  for _, v in pairs(node) do each_status(v, fn, seen) end
end

-- buff id -> cue for the given documents. A buff id filled in per source or rank (`x:{caster}` ->
-- `x:<entity>`) is kept as the prefix before its first `{`, ending in `*`. A buff id that statuses with
-- different cues (or one without a cue) share, such as the runtime's `stun`, cannot say which status it
-- is: it shows no cue and is listed as ambiguous.
function M.table_of(docs)
  local out, ambiguous = {}, {}
  for _, doc in ipairs(docs) do
    each_status(doc.value, function(s)
      if type(s.id) ~= "string" then return end
      local buff = s.runtime_id or s.id
      if type(buff) ~= "string" or buff:match("^attack_orb:") then return end
      local cue = type(s.cue) == "string" and s.cue or false
      local key = buff
      local at = buff:find("{", 1, true)
      if at then key = buff:sub(1, at - 1) .. "*" end
      if out[key] ~= nil and out[key] ~= cue then ambiguous[key] = true end
      if out[key] == nil then out[key] = cue end
    end, {})
  end
  local names = {}
  for key in pairs(ambiguous) do names[#names + 1] = key end
  for key, cue in pairs(out) do if not cue or ambiguous[key] then out[key] = nil end end
  table.sort(names)
  return out, names
end

function M.documents()
  local lfs = require("lfs")
  local docs = {}
  local packages = {}
  for name in lfs.dir(M.PACKAGES_DIR) do
    local file = M.PACKAGES_DIR .. "/" .. name .. "/v3.lua"
    if name ~= "." and name ~= ".." and lfs.attributes(file, "mode") == "file" then packages[#packages + 1] = name end
  end
  table.sort(packages)
  for _, name in ipairs(packages) do docs[#docs + 1] = { name = name, value = load_doc(M.PACKAGES_DIR .. "/" .. name .. "/v3.lua") } end
  local items = {}
  for name in lfs.dir(M.ITEMS_DIR) do if name:match("%.lua$") then items[#items + 1] = name end end
  table.sort(items)
  for _, name in ipairs(items) do docs[#docs + 1] = { name = "items/" .. name, value = load_doc(M.ITEMS_DIR .. "/" .. name) } end
  return docs
end

function M.main(args)
  local out = M.OUT
  local i = 1
  while i <= #args do
    if args[i] == "--out" then i = i + 1; out = args[i] else error("unknown argument: " .. tostring(args[i])) end
    i = i + 1
  end
  local b = require("_bootstrap")
  local cues, ambiguous = M.table_of(M.documents())
  local n = 0
  for _ in pairs(cues) do n = n + 1 end
  b.lib("json").write(out, { schema = "omoba.status-cues.v1", buffs = cues, ambiguous = ambiguous }, true)
  print(string.format("status cues: %d buffs -> %s (%d shared buff id(s) without a cue: %s)", n, out, #ambiguous, table.concat(ambiguous, ", ")))
end

if not STATUS_CUES_AS_MODULE and arg and arg[0] and arg[0]:match("status_cues%.lua$") then M.main(arg) end
return M
