-- v3 conformance results (scripts/base_content/src/v3_conformance_tests.rs writes
-- target/v3-conformance/<id>.json, or target/v3-conformance-generated when the generated Rust runs).
--
--   tools\lua\lua.exe scripts\v3_conformance_report.lua summary DIR [--reasons N]
--       per document and total counts by status, then the N most common unsupported reasons
--       (identifiers folded to X) .
--   tools\lua\lua.exe scripts\v3_conformance_report.lua compare BEFORE AFTER [ID ...]
--       every scenario whose status or detail differs between two result directories (handler vs
--       generated Rust, before vs after a change). Exit code 1 when anything differs.
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local lfs = require("lfs")
local json = b.lib("json")

local function documents(dir)
  local ids = {}
  for file in lfs.dir(dir) do
    local id = file:match("^(.+)%.json$")
    if id then ids[#ids + 1] = id end
  end
  table.sort(ids)
  return ids
end

local function load(dir, id)
  local file = dir .. "/" .. id .. ".json"
  local handle = io.open(file, "rb")
  if not handle then return nil end
  local doc = json.decode(handle:read("a"))
  handle:close()
  return doc
end

local function sorted_keys(map)
  local keys = {}
  for key in pairs(map) do keys[#keys + 1] = key end
  table.sort(keys)
  return keys
end

local function counts_text(counts)
  local parts = {}
  for _, key in ipairs(sorted_keys(counts)) do parts[#parts + 1] = key .. "=" .. counts[key] end
  return table.concat(parts, " ")
end

local function summary(dir, reasons_wanted)
  local total, reasons = {}, {}
  for _, id in ipairs(documents(dir)) do
    local doc = load(dir, id)
    local counts = {}
    for _, s in ipairs(doc.scenarios) do
      counts[s.status] = (counts[s.status] or 0) + 1
      total[s.status] = (total[s.status] or 0) + 1
      if s.status == "unsupported" then
        local reason = (s.detail or ""):gsub("[%w_]+:[%w_:{}]+", "X"):gsub("%f[%w_][%a][%w]*_[%w_]+", "X"):sub(1, 70)
        reasons[reason] = (reasons[reason] or 0) + 1
      end
    end
    print(string.format("%s %s %d %s", id, doc.hero, #doc.scenarios, counts_text(counts)))
  end
  print("total " .. counts_text(total))
  local ranked = sorted_keys(reasons)
  table.sort(ranked, function(x, y) if reasons[x] ~= reasons[y] then return reasons[x] > reasons[y] end return x < y end)
  for i = 1, math.min(reasons_wanted, #ranked) do print(reasons[ranked[i]] .. " " .. ranked[i]) end
end

local function compare(before, after, ids)
  if #ids == 0 then ids = documents(before) end
  local differ = 0
  for _, id in ipairs(ids) do
    local a, z = load(before, id), load(after, id)
    if not a or not z then
      print(id .. ": missing in " .. (a and after or before))
      differ = differ + 1
    else
      local left, right, all = {}, {}, {}
      for _, s in ipairs(a.scenarios) do left[s.id] = s; all[s.id] = true end
      for _, s in ipairs(z.scenarios) do right[s.id] = s; all[s.id] = true end
      local rows = {}
      for _, sid in ipairs(sorted_keys(all)) do
        local x, y = left[sid] or {}, right[sid] or {}
        if x.status ~= y.status or x.detail ~= y.detail then rows[#rows + 1] = sid end
      end
      differ = differ + #rows
      print(string.format("%s: %d vs %d scenarios, %d differ", id, #a.scenarios, #z.scenarios, #rows))
      for _, sid in ipairs(rows) do
        local x, y = left[sid] or {}, right[sid] or {}
        print(string.format("  %s: %s [%s] -> %s [%s]", sid, tostring(x.status), (x.detail or ""):sub(1, 140),
          tostring(y.status), (y.detail or ""):sub(1, 140)))
      end
    end
  end
  print("total differ " .. differ)
  return differ
end

local mode = arg[1]
if mode == "summary" and arg[2] then
  local n = 25
  for i = 3, #arg do if arg[i] == "--reasons" then n = tonumber(arg[i + 1]) or n end end
  summary(arg[2], n)
elseif mode == "compare" and arg[2] and arg[3] then
  local ids = {}
  for i = 4, #arg do ids[#ids + 1] = arg[i] end
  os.exit(compare(arg[2], arg[3], ids) == 0 and 0 or 1)
else
  io.stderr:write("usage: v3_conformance_report.lua summary DIR [--reasons N] | compare BEFORE AFTER [ID ...]\n")
  os.exit(2)
end
