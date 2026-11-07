-- M1 migration helper (v3-content-model-proposal.md §2.3): point a v2 package's Rust
-- handlers at the v3 value names of its v3.lua.
--
--   lua v3_migrate_m1.lua <package_dir> [--notes] [--dry-run]
--
--   --notes    first turn `note = "v2 extras: KEY ..."` into `v2_key = "KEY", note = ...`
--              in v3.lua (the census templates recorded the mapping in notes).
--   (always)   in rust/<implementation.module>.rs of each ability, rewrite the
--              extras keys read through extra*(...) from each value's v2_key to its v3 name.
-- Prints every rewrite; keys it cannot place (test support files, other abilities' keys)
-- are left for `hero_author.lua validate`, which fails on any key the kit does not produce.
local package_dir = assert(arg[1], "usage: lua v3_migrate_m1.lua <package_dir> [--notes] [--dry-run]")
local flags = {}
for i = 2, #arg do flags[arg[i]] = true end

local function read(p) local f = assert(io.open(p, "rb")); local t = f:read("a"); f:close(); return t end
local function write(p, t)
  if flags["--dry-run"] then return end
  local f = assert(io.open(p, "wb")); f:write(t); f:close()
end
local function load(p)
  local chunk = assert(loadfile(p, "t", { pairs = pairs, ipairs = ipairs, string = string, table = table, math = math }))
  local v = chunk()
  if type(v) == "function" then v = v({}) end
  return v
end

-- --list-v2: print, per ability, the numbers the v2 generator emits today (each needs a
-- kit value with that v2_key) and the v2 levels, to start a kit from.
if flags["--list-v2"] then
  local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
  local scripts = flags["--scripts"] or (tools_dir .. "/../../../scripts")
  package.path = scripts .. "/?.lua;" .. package.path
  local generate = require("hero_author.generate")
  local v2 = load(package_dir .. "/abilities.lua")
  for _, ab in ipairs(v2.abilities) do
    local ok, entry = pcall(generate.ability_entry, "x", ab, nil)
    print(("== %s (%s, action %s, max_level %d, module %s)"):format(ab.id, ab.source_ability_id or "?", tostring(ab.action),
      ab.max_level, tostring((ab.implementation or {}).module)))
    if not ok then print("  (v2 generator failed: " .. tostring(entry) .. ")")
    else
      local keys = {}
      for k in pairs(entry.extras) do keys[#keys + 1] = k end
      table.sort(keys)
      for _, k in ipairs(keys) do
        local t = {}
        for i, v in ipairs(entry.extras[k]) do t[i] = tostring(v) end
        print(("  v2_key %-40s %s"):format(k, table.concat(t, " / ")))
      end
      for _, f in ipairs({ "cooldown", "mana_cost", "cast_time", "range", "required_hero_level" }) do
        local t = {}
        for i, l in ipairs(entry.levels) do t[i] = tostring(l[f] or "-") end
        print(("  levels.%-38s %s"):format(f, table.concat(t, " / ")))
      end
      for _, k in ipairs({ "targeting_shape", "channel", "charges", "cast_cooldown_refund", "attack_orb" }) do
        if entry[k] then
          local t = {}
          for kk, vv in pairs(entry[k]) do if tostring(kk):match("_key$") then t[#t + 1] = kk .. "=" .. tostring(vv) end end
          table.sort(t)
          print("  host " .. k .. ": " .. table.concat(t, ", ") .. "  (these keys need a kit value with that v2_key)")
        end
      end
      if entry.targeting_reach_key then print("  host targeting_reach_key = " .. entry.targeting_reach_key) end
    end
  end
  os.exit(0)
end

local kit_path = package_dir .. "/v3.lua"
if flags["--notes"] then
  local text = read(kit_path)
  local n
  text, n = text:gsub('([%s{,])note = "v2 extras: ([%w_]+)', '%1v2_key = "%2", note = "v2 extras: %2')
  print(("v3.lua: %d v2_key added from notes"):format(n))
  write(kit_path, text)
end

local kit = load(kit_path)
local v2 = load(package_dir .. "/abilities.lua")
local module_of = {}
for _, ab in ipairs(v2.abilities) do
  if ab.implementation and ab.implementation.module then module_of[ab.id] = ab.implementation.module end
end

local function escape(s) return (s:gsub("%p", "%%%0")) end
-- Host contract keys keep their v2 name in extras (hero_author/v3.lua); everything else a handler
-- names as a string literal (extra("k"), const K:&str="k", match arms, payload keys) is renamed.
local HOST = { shape_radius = true, shape_width = true, shape_half_angle = true, charges_max = true,
  charges_recharge_seconds = true, channel_interval = true, channel_pulses = true, channel_radius = true,
  cooldown_refund_on_cast_seconds = true, summon_owner_cap = true, attack_orb_attack_speed_slow_constant = true,
  custom_charges_rank_up_keep_count = true, attack_orb_mana_per_attack = true, attack_orb_amount = true,
  attack_orb_move_slow = true, attack_orb_attack_slow = true, attack_orb_slow_duration = true }
local function is_host(k) return HOST[k] or k:match("_max_distance$") ~= nil end
local total = 0
for _, ab in ipairs(kit.abilities or {}) do
  local module = module_of[ab.id]
  local renames = {}
  for name, def in pairs(ab.values or {}) do
    if type(def) == "table" and def.v2_key and def.v2_key ~= name then renames[def.v2_key] = name end
  end
  if module and next(renames) then
    local file = package_dir .. "/rust/" .. module .. ".rs"
    local f = io.open(file, "rb")
    if f then
      f:close()
      local text = read(file)
      for old, new in pairs(renames) do
        local count
        if is_host(old) then
          text, count = text:gsub('(extra[%w_]*%s*%(%s*[%w_]*%s*,?%s*)"' .. escape(old) .. '"', '%1"' .. new .. '"')
        else
          text, count = text:gsub('"' .. escape(old) .. '"', '"' .. new .. '"')
        end
        if count > 0 then print(("%s.rs: %s -> %s (%d)"):format(module, old, new, count)); total = total + count end
      end
      write(file, text)
    end
  end
end
print(("%d key reads rewritten%s"):format(total, flags["--dry-run"] and " (dry run)" or ""))
