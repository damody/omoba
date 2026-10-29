-- Provenance check for package schema v3 values (value-single-source §5 "出處一致",
-- v3.md §10 M1 item): every value whose source is `kv:<Key>` is compared with the pinned
-- original KV of history_nobu.
--
--   lua v3_provenance.lua [--source <checkout>] [--strict] <v3 document|package dir> ...
--
-- source forms:  kv:<Key>                      looked up in the entry's own KV block
--                kv:<Key>@<path>[:<line>]      looked up in that file (relative to source.root)
--                kv:<Block> <Key>              <Key> inside the sub-block <Block> (modifiers)
-- The entry's KV block is the top-level node named like its source_id in the npc/*.txt
-- files its provenance cites. Inside it a key is searched in this order: the block's own
-- keys, AbilitySpecial / AbilityValues, then anywhere below. `%name` values follow the
-- AbilitySpecial entry they point at.
-- Compared numbers: the value's `kv` when it has one (the author recorded what the KV says
-- and why it is not used), otherwise its ranks. A value with source_differs but no `kv` is
-- not compared. Exit 1 on any mismatch; with --strict also on keys that cannot be found.
local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
local scripts_dir = tools_dir .. "/../../../scripts"
package.path = scripts_dir .. "/?.lua;" .. package.path
local kv = require("hero_source.kv")
PACKAGE_V3_AS_MODULE = true
local v3 = dofile(tools_dir .. "/package_v3.lua")
PACKAGE_V3_AS_MODULE = nil

local M = {}
M.DEFAULT_SOURCE = "C:/nobu/history_nobu"

local cache = {}
local function load_kv(root, rel)
  local path = root .. "/" .. rel
  if cache[path] == nil then
    local f = io.open(path, "rb")
    if not f then cache[path] = false
    else
      local text = f:read("a"); f:close()
      cache[path] = kv.parse(text)
    end
  end
  return cache[path] or nil
end

-- top-level entry blocks of a KV file (children of DOTAAbilities / DOTAItems / DOTAUnits)
local function entries(doc)
  local out = {}
  for _, n in ipairs(doc.nodes) do
    if n.children then
      for _, c in ipairs(n.children) do if c.key then out[c.key:lower()] = out[c.key:lower()] or c end end
    end
  end
  return out
end

local function find_in(nodes, key, depth)
  local lower = key:lower()
  for _, n in ipairs(nodes or {}) do
    if n.key and n.key:lower() == lower then return n end
  end
  if depth == 0 then return nil end
  for _, n in ipairs(nodes or {}) do
    local hit = n.children and find_in(n.children, key, depth - 1)
    if hit then return hit end
  end
end

-- Looks a key up in a block: own keys, AbilitySpecial/AbilityValues, then anywhere below.
local function lookup(block, key)
  local own = find_in(block.children, key, 0)
  if own then return own end
  for _, section in ipairs({ "AbilitySpecial", "AbilityValues" }) do
    local s = find_in(block.children, section, 0)
    if s then
      local hit = find_in(s.children, key, 1)
      if hit then return hit end
    end
  end
  return find_in(block.children, key, 8)
end

local function numbers_of(block, node)
  local value = node and node.value
  if value == nil and node and node.children then
    -- AbilityValues { key { value "1 2 3" } }
    value = kv.value(node.children, "value")
  end
  local ref = kv.placeholder(value)
  if ref and block then return numbers_of(nil, lookup(block, ref)) end
  return kv.rank_values(value)
end

local function same(a, b) return math.abs(a - b) <= 1e-6 * math.max(1, math.abs(a), math.abs(b)) end

-- returns status ("ok"|"mismatch"|"missing"|"skipped"), detail
function M.check_value(root, block, def, max_level)
  local spec = def.source:match("^kv:(.*)$")
  if def.expr and not def.kv then return "skipped", "derived value" end
  -- "kv:A + B" (a sum of several KV entries, e.g. components plus recipe) is not a single key
  if spec:find("+", 1, true) then return "skipped", "composite source" end
  local key_part, at = spec:match("^([^@]*)@(.*)$")
  key_part = key_part or spec
  local keys = {}
  for w in key_part:gmatch("[%w_]+") do keys[#keys + 1] = w end
  if #keys == 0 then return "missing", "empty kv key" end
  local line_block -- the block that starts at @path:line, when the entry has no KV block of its own
  if at then
    local rel, line = at:match("^([^:%s]+):?(%d*)")
    local doc = rel and load_kv(root, rel)
    if not doc then return "missing", "file " .. tostring(rel) .. " not found" end
    line = tonumber(line)
    local found
    kv.walk(doc.nodes, function(n)
      if found then return end
      if n.key and n.key:lower() == keys[#keys]:lower() and (not line or n.line == line) then found = n end
    end)
    if not found and line then
      -- line points at the block: search the key inside the block that starts there
      kv.walk(doc.nodes, function(n)
        if not line_block and n.line == line and n.children then line_block = n end
      end)
      found = line_block and lookup(line_block, keys[#keys])
    end
    if not found and block then found = lookup(block, keys[#keys]) end
    if not found then return "missing", keys[#keys] .. " not in " .. rel end
    local nums = numbers_of(block or line_block, found)
    if not nums then return "missing", keys[#keys] .. " is not numeric" end
    return M.compare(def, nums, max_level)
  end
  if not block then return "missing", "no KV block" end
  -- `kv:<Block> <Key>` descends; when the words after the first are a comment (v3.md §12.11),
  -- the first key alone is used
  local node = lookup(block, keys[1])
  if not node then return "missing", keys[1] .. " not in the block" end
  local nested = node
  for i = 2, #keys do
    nested = nested and nested.children and find_in(nested.children, keys[i], 8)
  end
  if #keys > 1 and nested and numbers_of(block, nested) then node = nested end
  local nums = numbers_of(block, node)
  if not nums then return "missing", table.concat(keys, " ") .. " is not numeric" end
  -- Dota expands RequiredLevel by LevelsBetweenUpgrades (6 + 6 per rank -> 6/12/18)
  if keys[1]:lower() == "requiredlevel" and #nums == 1 then
    local step = kv.rank_values(kv.value(block.children, "LevelsBetweenUpgrades"))
    if step then for i = 2, max_level do nums[i] = nums[1] + (i - 1) * step[1] end end
  end
  return M.compare(def, nums, max_level)
end

function M.compare(def, nums, max_level)
  local want
  if def.kv then want = def.kv
  elseif def.source_differs then return "skipped", "source_differs without kv"
  else want = def end
  local n = math.max(#want, #nums, 1)
  if def.per == "hero_level" then n = math.max(#want, #nums) end
  local function at(t, i) return t[i] or t[#t] end
  for i = 1, n do
    local w, k = at(want, i), at(nums, i)
    if type(w) == "string" then
      local a, b = w:match("^(%-?%d+)/(%d+)$")
      w = a and (tonumber(a) / tonumber(b)) or tonumber(w)
    end
    -- KV often stores a percent as a fraction (0.15 for a pct value of 15)
    local pct_ok = def.unit == "pct" and w ~= nil and k ~= nil and same(w / 100, k)
    if w == nil or k == nil or not (same(w, k) or pct_ok) then
      local shown = {}
      for j, x in ipairs(want) do shown[j] = tostring(x) end
      local kshown = {}
      for j, x in ipairs(nums) do kshown[j] = tostring(x) end
      return "mismatch", ("v3 %s ≠ KV %s"):format(table.concat(shown, "/"), table.concat(kshown, "/"))
    end
  end
  return "ok"
end

function M.check_doc(doc, root)
  local game = root .. "/" .. (doc.source and doc.source.root or "game/dota_addons/nobu/scripts/")
  game = game:gsub("/+$", "")
  local results = {}
  local function check_entry(e, max_level, path)
    local block
    for _, p in ipairs(e.provenance or {}) do
      local rel = p:match("^([^:]+%.txt)")
      if rel then
        local kdoc = load_kv(game, rel)
        if kdoc then block = block or entries(kdoc)[(e.source_id or ""):lower()] end
      end
    end
    for name, def in pairs(e.values or {}) do
      if type(def) == "table" and type(def.source) == "string" and def.source:match("^kv:") then
        local status, detail = M.check_value(game, block, def, max_level)
        results[#results + 1] = { path = path .. ".values." .. name, status = status, detail = detail, source = def.source }
      end
    end
    for _, s in ipairs(e.statuses or {}) do
      for name, def in pairs(s.values or {}) do
        if type(def) == "table" and type(def.source) == "string" and def.source:match("^kv:") then
          local status, detail = M.check_value(game, block, def, max_level)
          results[#results + 1] = { path = path .. ".statuses." .. s.id .. ".values." .. name, status = status, detail = detail, source = def.source }
        end
      end
    end
  end
  for _, a in ipairs(doc.abilities or {}) do check_entry(a, a.max_level or 1, a.id) end
  for _, it in ipairs(doc.items or {}) do check_entry(it, it.max_level or 1, it.id) end
  table.sort(results, function(a, b) return a.path < b.path end)
  return results
end

function M.main(argv)
  local root, strict, files = M.DEFAULT_SOURCE, false, {}
  local i = 1
  while i <= #argv do
    if argv[i] == "--source" then root = argv[i + 1]; i = i + 1
    elseif argv[i] == "--strict" then strict = true
    else files[#files + 1] = argv[i] end
    i = i + 1
  end
  local failed = 0
  for _, f in ipairs(files) do
    local target = f:match("%.lua$") and f or (f .. "/v3.lua")
    local doc = assert(v3.load_file(target))
    local counts = { ok = 0, mismatch = 0, missing = 0, skipped = 0 }
    for _, r in ipairs(M.check_doc(doc, root)) do
      counts[r.status] = counts[r.status] + 1
      if r.status == "mismatch" or (strict and r.status == "missing") then
        io.stderr:write(("%s: %s: %s (%s)\n"):format(f, r.path, r.detail, r.source))
      elseif r.status == "missing" then
        io.stderr:write(("%s: warning: %s: %s (%s)\n"):format(f, r.path, r.detail, r.source))
      end
    end
    print(("%s: kv %d ok, %d mismatch, %d not found, %d documented differences"):format(f, counts.ok, counts.mismatch, counts.missing, counts.skipped))
    if counts.mismatch > 0 or (strict and counts.missing > 0) then failed = failed + 1 end
  end
  return failed == 0 and 0 or 1
end

if rawget(_G, "V3_PROVENANCE_AS_MODULE") then return M end
os.exit(M.main(arg))
