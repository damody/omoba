-- v3 summons: every hero document's `unit_templates` (docs/character_pipeline/packages/*/v3.lua)
-- become content summon rows, so the host's create_unit (omoba_template_ids::summon_by_name)
-- spawns what the document says (E326). Writes scripts/lua_data/templates/v3_summons.lua, which
-- templates/summons.lua appends.
--
--   tools\lua\lua.exe scripts\v3_summons.lua [--check]
--
-- Every number comes from the document (value single source). A template whose fields the
-- summon model cannot hold yet gets no row; its problems are listed (M.problems), and the
-- generator (v3_rust.lua) blocks create_unit of it with them, never a silent drop. A template
-- with `max_level` > 1 has one row per rank, `<id>_r<rank>` (v3.md §7: `$x` at the rank of the
-- ability that creates it). An id the content already has (a package summons.lua) is left to it.
local source = debug.getinfo(1, "S").source:sub(2)
local here = source:match("^(.*)[/\\]") or "."
package.path = here .. "/?.lua;" .. here .. "/../tools/lua/?.lua;" .. package.path
local M = {}

M.OUT = "scripts/lua_data/templates/v3_summons.lua"

-- The host fires a summon's ranged attack at this speed (omoba-core summons; the v3 B01 document
-- records it as `projectile_speed = 900`).
local HOST_PROJECTILE_SPEED = 900

local function safe_env() return require("hero_author.cli").safe_env() end

-- The value `ref` (`$name`, `$owner.name` or a number) of template `t` at `rank`; nil and the
-- reason when it is not a plain value.
local function resolver(doc, t)
  local owners = {}
  for _, a in ipairs(doc.abilities or {}) do owners[a.id] = a end
  for _, u in ipairs(doc.unit_templates or {}) do owners[u.id] = u end
  local function at(row, rank)
    if type(row) == "number" then return row end
    if type(row) ~= "table" then return nil end
    return row[math.min(rank, #row)] or row[1]
  end
  return function(ref, rank)
    if type(ref) == "number" then return ref end
    if type(ref) ~= "string" then return nil, "a " .. type(ref) end
    if tonumber(ref) then return tonumber(ref) end
    local owner, name = ref:match("^%s*%$([%w_]+)%.([%w_]+)%s*$")
    if not owner then name = ref:match("^%s*%$([%w_]+)%s*$") end
    if not name then return nil, "the expression `" .. ref .. "`" end
    local row
    if owner then row = owners[owner] and owners[owner].values and owners[owner].values[name]
    else row = (t.values or {})[name]; if row == nil then row = (doc.values or {})[name] end end
    local v = at(row, rank)
    if v == nil then return nil, "`" .. ref .. "` has no value" end
    return v
  end
end

local function sorted(t) local k = {} for x in pairs(t) do k[#k + 1] = x end table.sort(k) return k end

-- The shared dummy body (omoba-core dummy_unit.rs STATES: invulnerable, unselectable,
-- attack_immune, command_restricted, no_healthbar, no_unit_collision) covers the original majia
-- states; magic_immune adds nothing on a unit no selector takes and no damage reaches, and a
-- dummy shows no model, so not_on_minimap holds too.
local DUMMY_STATES = { invulnerable = true, unselectable = true, attack_immune = true, command_restricted = true,
  no_healthbar = true, no_unit_collision = true, magic_immune = true, not_on_minimap = true,
  -- unselectable already keeps every spell off it
  untargetable_by_spells = true }

local function has(list, v) for _, x in ipairs(list or {}) do if x == v then return true end end return false end

-- A template with no attack that is a dummy (`ai = "none"` or tag `dummy`): the host's dummy body
-- (line B ee50b7e7), with only its vision and owner rule from the document.
local function dummy_rows(doc, t, value)
  local problems = {}
  local function no(what) problems[#problems + 1] = what end
  local known = { id = true, source_id = true, values = true, text = true, notes = true, provenance = true, quirks = true,
    max_level = true, attack = true, vision = true, owner_death = true, ai = true, tags = true, passive = true }
  for _, k in ipairs(sorted(t)) do if not known[k] then no(k .. " on a dummy") end end
  if t.ai ~= nil and t.ai ~= "none" then no("ai:" .. tostring(t.ai) .. " on a dummy") end
  for _, tag in ipairs(t.tags or {}) do if tag ~= "dummy" and tag ~= "summon" then no("tag:" .. tag) end end
  if t.owner_death ~= nil and t.owner_death ~= "persist" and t.owner_death ~= "expire" then no("owner_death:" .. tostring(t.owner_death)) end
  local p = t.passive or {}
  for _, k in ipairs(sorted(p)) do if k ~= "reveal" and k ~= "states" then no("passive." .. k .. " on a dummy") end end
  if p.reveal ~= nil and p.reveal ~= "public" then no("passive.reveal:" .. tostring(p.reveal)) end
  for _, st in ipairs(p.states or {}) do if not DUMMY_STATES[st] then no("passive.states:" .. st .. " on a dummy") end end
  local rows = {}
  for rank = 1, t.max_level or 1 do
    local vision = t.vision or {}
    local day = vision.day ~= nil and value(vision.day, rank) or nil
    local night = vision.night ~= nil and value(vision.night, rank) or nil
    if (vision.day ~= nil and day == nil) or (vision.night ~= nil and night == nil) then no("vision is not a plain value") end
    if day and night and day ~= night then no("vision.day differs from vision.night") end
    rows[#rows + 1] = { id = (t.max_level or 1) > 1 and (t.id .. "_r" .. rank) or t.id, authored_package = doc.__package,
      display_name = t.text and t.text.zh_TW and t.text.zh_TW.name or t.id,
      dummy = true, immobile = true, vision_radius = day or night or 0, owner_death = t.owner_death or "persist" }
  end
  local seen, list = {}, {}
  for _, x in ipairs(problems) do if not seen[x] then seen[x] = true; list[#list + 1] = x end end
  return #list == 0 and rows or {}, list
end

-- The summon rows of template `t` of document `doc` and the problems that keep it from having any.
function M.template_rows(doc, t)
  local problems = {}
  local function no(what) problems[#problems + 1] = what end
  local value = resolver(doc, t)
  local ranks = t.max_level or 1
  if (not t.attack or t.attack.kind == "none") and (t.ai == "none" or has(t.tags, "dummy")) then return dummy_rows(doc, t, value) end
  local known = { id = true, source_id = true, values = true, text = true, notes = true, provenance = true, quirks = true,
    max_level = true, stats = true, attack = true, bounty = true, vision = true, collision_radius = true,
    acquisition_range = true, owner_death = true, ai = true, tags = true, passive = true }
  for _, k in ipairs(sorted(t)) do if not known[k] then no(k) end end
  local stats = t.stats or {}
  for _, k in ipairs(sorted(stats)) do
    if k ~= "hp" and k ~= "armor" and k ~= "magic_resist" and k ~= "move_speed" then no("stats." .. k) end
  end
  local attack = t.attack
  if not attack or attack.kind == "none" then no("no attack (the dummy unit type, line B)") attack = {} end
  if attack.kind and attack.kind ~= "melee" and attack.kind ~= "ranged" and attack.kind ~= "none" then no("attack.kind:" .. tostring(attack.kind)) end
  if attack.damage_type and attack.damage_type ~= "physical" then no("attack.damage_type:" .. tostring(attack.damage_type)) end
  for _, k in ipairs(sorted(attack)) do
    if k ~= "kind" and k ~= "range" and k ~= "interval" and k ~= "damage" and k ~= "damage_max" and k ~= "damage_type"
      and k ~= "attack_point" and k ~= "projectile_speed" then no("attack." .. k) end
  end
  -- owner_controlled is guard without following: the player's summons move on its orders only
  -- (host summon_orders; line B: follow 0, the original does not follow on its own)
  if t.ai ~= "guard" and t.ai ~= "owner_controlled" then no("ai:" .. tostring(t.ai)) end
  if t.owner_death ~= nil and t.owner_death ~= "persist" and t.owner_death ~= "expire" then no("owner_death:" .. tostring(t.owner_death)) end
  local p = t.passive or {}
  for _, k in ipairs(sorted(p)) do if k ~= "reveal" then no("passive." .. k) end end
  if p.reveal ~= nil and p.reveal ~= "public" then no("passive.reveal:" .. tostring(p.reveal)) end
  for _, tag in ipairs(t.tags or {}) do if tag ~= "summon" then no("tag:" .. tag) end end
  local rows = {}
  for rank = 1, ranks do
    local function num(ref, what)
      if ref == nil then return nil end
      local v, why = value(ref, rank)
      if v == nil then no(what .. " is " .. why) end
      return v
    end
    local row = { id = ranks > 1 and (t.id .. "_r" .. rank) or t.id, authored_package = doc.__package,
      display_name = t.text and t.text.zh_TW and t.text.zh_TW.name or t.id }
    row.hp = num(stats.hp, "stats.hp")
    if row.hp == nil then no("stats.hp missing") end
    row.armor = num(stats.armor, "stats.armor") or 0
    row.magic_resistance = num(stats.magic_resist, "stats.magic_resist") or 0
    row.move_speed = num(stats.move_speed, "stats.move_speed")
    if row.move_speed == nil then no("stats.move_speed missing") end
    -- one damage per attack: the host's summons hit for `damage` (the mean of a min/max range,
    -- as unit_overrides does); `$base + $per_level * (caster_hero.level - 1)` is the per owner
    -- level growth the host samples at the spawn
    local dmg = attack.damage
    local base, per = type(dmg) == "string" and dmg:match("^%s*(%$[%w_%.]+)%s*%+%s*(%$[%w_%.]+)%s*%*%s*%(%s*caster_hero%.level%s*%-%s*1%s*%)%s*$")
    if base then dmg = base; row.damage_per_owner_level = num(per, "attack.damage per owner level") end
    local lo = num(dmg, "attack.damage")
    local hi = attack.damage_max ~= nil and num(attack.damage_max, "attack.damage_max") or lo
    if lo and hi then row.damage = (lo + hi) / 2 end
    row.attack_range = num(attack.range, "attack.range")
    row.attack_interval = num(attack.interval, "attack.interval")
    if attack.kind and attack.kind ~= "none" and (not row.attack_range or not row.attack_interval) then no("attack range or interval missing") end
    -- the animation point as the share of the attack interval before the hit (attack_timing)
    if attack.attack_point ~= nil and row.attack_interval and row.attack_interval > 0 then
      local ap = num(attack.attack_point, "attack.attack_point")
      if ap then
        local windup = math.floor(ap / row.attack_interval * 1000 + 0.5)
        if windup <= 0 or windup >= 1000 then no("attack.attack_point outside the interval")
        else row.attack_timing = { windup = windup, backswing = 1000 - windup } end
      end
    end
    if attack.projectile_speed ~= nil then
      local ps = num(attack.projectile_speed, "attack.projectile_speed")
      if ps and ps ~= HOST_PROJECTILE_SPEED then no("attack.projectile_speed " .. ps .. " (the host fires at " .. HOST_PROJECTILE_SPEED .. ")") end
    end
    -- the host acquires at the attack range
    if t.acquisition_range ~= nil then
      local acq = num(t.acquisition_range, "acquisition_range")
      if acq and row.attack_range and acq ~= row.attack_range then no("acquisition_range " .. acq .. " differs from attack.range") end
    end
    -- gold and experience to the enemy player who kills it (line B 3b1c87db), whole numbers
    if t.bounty then
      for _, k in ipairs(sorted(t.bounty)) do
        if k ~= "gold" and k ~= "xp" then no("bounty." .. k)
        else
          local b = num(t.bounty[k], "bounty." .. k)
          if b and (b < 0 or b ~= math.floor(b)) then no("bounty." .. k .. " is not a whole number")
          elseif b and b > 0 then row["bounty_" .. k] = math.tointeger(b) end
        end
      end
    end
    local vision = t.vision or {}
    local day, night = num(vision.day, "vision.day"), num(vision.night, "vision.night")
    if day and night and day ~= night then no("vision.day differs from vision.night") end
    row.vision_radius = day or night or 0
    row.collision_radius = num(t.collision_radius, "collision_radius") or 0
    row.immobile = row.move_speed == 0 or nil
    row.follow_start_distance, row.follow_stop_distance = 0, 0
    row.target_priority = "nearest"
    row.owner_death = t.owner_death or "persist"
    rows[#rows + 1] = row
  end
  -- one problem once
  local seen, list = {}, {}
  for _, x in ipairs(problems) do if not seen[x] then seen[x] = true; list[#list + 1] = x end end
  return #list == 0 and rows or {}, list
end

-- Every hero document, in package order, then every item document (docs/character_pipeline/items,
-- in file order): { package, doc }.
function M.documents(root)
  local lfs = require("lfs")
  local dir = root .. "/docs/character_pipeline/packages"
  local names = {}
  for name in lfs.dir(dir) do
    if name ~= "." and name ~= ".." and lfs.attributes(dir .. "/" .. name .. "/v3.lua", "mode") == "file" then names[#names + 1] = name end
  end
  table.sort(names)
  local docs = {}
  for _, name in ipairs(names) do
    local chunk = assert(loadfile(dir .. "/" .. name .. "/v3.lua", "t", safe_env()))
    local doc = chunk()
    if type(doc) == "function" then doc = doc({}) end
    doc.__package = name
    docs[#docs + 1] = { package = name, doc = doc }
  end
  local items = root .. "/docs/character_pipeline/items"
  local files = {}
  for name in lfs.dir(items) do if name:match("%.lua$") then files[#files + 1] = name end end
  table.sort(files)
  for _, name in ipairs(files) do
    local chunk = assert(loadfile(items .. "/" .. name, "t", safe_env()))
    local doc = chunk()
    if type(doc) == "function" then doc = doc({}) end
    doc.__package = "items"
    docs[#docs + 1] = { package = "items/" .. name, doc = doc }
  end
  return docs
end

-- `existing`: the summon ids the content has without this file (package summons.lua).
-- -> rows, { [template id] = problems }
function M.build(root, existing)
  local rows, problems = {}, {}
  for _, d in ipairs(M.documents(root)) do
    for _, t in ipairs(d.doc.unit_templates or {}) do
      if not existing[t.id] then
        local r, p = M.template_rows(d.doc, t)
        if #p > 0 then problems[t.id] = p end
        for _, row in ipairs(r) do rows[#rows + 1] = row end
      end
    end
  end
  return rows, problems
end

-- The summon ids of the content apart from this file's rows.
function M.existing(root)
  local saved = package.path
  package.path = root .. "/scripts/?.lua;" .. package.path
  local context = require("content_builder").new(root .. "/scripts/lua_data")
  package.path = saved
  local ours = {}
  local f = io.open(root .. "/" .. M.OUT, "rb")
  if f then
    f:close()
    for _, row in ipairs(context.include("templates/v3_summons.lua")) do ours[row.id] = true end
  end
  local ids = {}
  for _, row in ipairs(context.include("templates/summons.lua")) do
    if not row.tombstone and not ours[row.id] then ids[row.id] = true end
  end
  return ids
end

-- template id -> problems (the generator's blockers), cached per root.
local problems_cache = {}
function M.problems(root)
  if problems_cache[root] then return problems_cache[root] end
  local _, problems = M.build(root, M.existing(root))
  problems_cache[root] = problems
  return problems
end

function M.render(rows)
  local serialize = require("hero_source.serialize")
  local header = "-- GENERATED by scripts/v3_summons.lua from docs/character_pipeline/packages/*/v3.lua unit_templates; do not edit.\n"
    .. "-- templates/summons.lua appends these rows (E326).\n"
  local body = serialize.serialize(rows, header)
  return (body:gsub("^(.-)return ", "%1local rows = ", 1)) .. "return function(ctx) return rows end\n"
end

if ... == nil then
  local root = "."
  local check = arg[1] == "--check"
  local rows = M.build(root, M.existing(root))
  local text = M.render(rows)
  local f = io.open(M.OUT, "rb")
  local old = f and f:read("a"); if f then f:close() end
  if old and old:find("\r\n", 1, true) then text = text:gsub("\r?\n", "\r\n") end
  if check then
    if old ~= text then io.stderr:write(M.OUT .. " is out of date: run tools\\lua\\lua.exe scripts\\v3_summons.lua\n"); os.exit(1) end
    print("ok " .. M.OUT)
  elseif old ~= text then
    f = assert(io.open(M.OUT, "wb")); f:write(text); f:close(); print("wrote " .. M.OUT .. " (" .. #rows .. " rows)")
  else
    print("unchanged " .. M.OUT)
  end
end

return M
