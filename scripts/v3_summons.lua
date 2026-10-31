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

-- A summon row's acquisition_range and projectile_speed (line B 53efe6a5): 0..=20000.
local HOST_RANGE_MAX = 20000

local function safe_env() return require("hero_author.cli").safe_env() end

-- The value `ref` (`$name`, `$owner.name` or a number) of template `t` at `rank`; nil and the
-- reason when it is not a plain value.
-- A derived value (`expr`) as the catalog evaluates it (v3_compile.lua, the single evaluator):
-- { [owner id] = { [name] = per-rank numbers } } of the document, cached per document.
local compiled_cache = setmetatable({}, { __mode = "k" })
local function compiled_values(doc)
  if compiled_cache[doc] then return compiled_cache[doc] end
  local tools = here .. "/../docs/character_pipeline/tools"
  local compiler = dofile(tools .. "/v3_compile.lua")
  local out = compiler.compile(doc, { names = compiler.item_names(here .. "/../docs/character_pipeline/items") })
  local by_owner = {}
  local function take(id, rows)
    local t = by_owner[id] or {}
    for _, row in ipairs(rows or {}) do
      if row.ranks then
        local nums = {}
        for i, text in ipairs(row.ranks) do nums[i] = compiler.value_of(text) end
        t[row.name] = { nums = nums, unit = row.unit }
      end
    end
    by_owner[id] = t
  end
  for id, o in pairs(out.owner_values or {}) do take(id, o.values) end
  for id, b in pairs(out.abilities or {}) do take(id, b.values) end
  for id, b in pairs(out.items or {}) do take(id, b.values) end
  compiled_cache[doc] = by_owner
  return by_owner
end

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
    local row, from
    if owner then row = owners[owner] and owners[owner].values and owners[owner].values[name]; from = owner
    else
      row = (t.values or {})[name]; from = t.id
      if row == nil then row = (doc.values or {})[name]; from = "shared" end
    end
    -- derived: the compiled number (a pct value is shown x100 there, so it has none here)
    if type(row) == "table" and row.expr ~= nil then
      if row.unit == "pct" then return nil, "`" .. ref .. "` is a derived pct value" end
      local c = (compiled_values(doc)[from] or {})[name]
      if not c then return nil, "`" .. ref .. "` (derived) has no compiled value" end
      row = c.nums
    end
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

-- Day and night sight into `row` (line B b8a572ea): `vision_radius` by day, `night_vision_radius`
-- when the night differs. The host reads a night radius 0 as the day one, so a template blind at
-- night has no row value.
local function vision_into(row, vision, num, no)
  local day, night = num(vision.day, "vision.day"), num(vision.night, "vision.night")
  row.vision_radius = day or night or 0
  if day and night and night ~= day then
    if night > 0 then row.night_vision_radius = night else no("vision.night 0 (the host reads 0 as the day radius)") end
  end
end

-- `statuses`: the generated create_unit applies them to the new unit (v3_rust.lua); the row
-- carries none. -> the problem, or nil
local function statuses_problem(t)
  if t.statuses == nil then return nil end
  if type(t.statuses) ~= "table" then return "statuses is not a list" end
  for _, sid in ipairs(t.statuses) do if type(sid) ~= "string" then return "statuses entry of a " .. type(sid) end end
  return nil
end

-- The unit's mana (full at birth) and mana per second (host unit mana, line B 35a0eaee: 0..100000
-- and 0..1000; regeneration needs mana).
local function mana_into(row, stats, num, no)
  local mana, regen = num(stats.mana, "stats.mana"), num(stats.mana_regen, "stats.mana_regen")
  if mana and (mana < 0 or mana > 100000) then no("stats.mana " .. mana .. " outside 0..100000") elseif mana and mana > 0 then row.mana = mana end
  if regen and (regen < 0 or regen > 1000) then no("stats.mana_regen " .. regen .. " outside 0..1000")
  elseif regen and regen > 0 then
    if not row.mana then no("stats.mana_regen without mana") else row.mana_regen = regen end
  end
end

-- The ability of `doc` whose program creates template `id`: the host runs a chase unit's
-- on_reach in that ability's on_status_event (line B E406). -> id, or nil and why
function M.reach_ability(doc, id)
  local function creates(x, seen)
    if type(x) ~= "table" or seen[x] then return false end
    seen[x] = true
    if x.op == "create_unit" and x.template == id then return true end
    for _, v in pairs(x) do if creates(v, seen) then return true end end
    return false
  end
  local found = {}
  for _, a in ipairs(doc.abilities or {}) do if creates(a, {}) then found[#found + 1] = a.id end end
  for name, r in pairs(doc.routines or {}) do if creates(r, {}) then return nil, "created in routine " .. name end end
  if #found == 0 then return nil, "no ability creates it" end
  if #found > 1 then return nil, "created by " .. table.concat(found, " and ") end
  return found[1]
end

-- The host chase table of a chase_target template (line B E406): every number from the document.
local CHASE_NUMS = { "speed", "max_distance", "teleport_back_distance", "reach_radius", "turn_degrees", "turn_every_frames",
  "snap_degrees", "random_retarget_chance", "wander_return_chance" }
local function chase_row(doc, t, num, no)
  local c = t.chase
  local known = { on_reach = true, anchor = true, retarget = true, leash = true }
  for _, k in ipairs(CHASE_NUMS) do known[k] = true end
  for _, k in ipairs(sorted(c)) do if not known[k] then no("chase." .. k) end end
  local row = { anchor = c.anchor or "owner", retarget = c.retarget or "none" }
  if row.anchor ~= "owner" and row.anchor ~= "spawn" then no("chase.anchor:" .. tostring(row.anchor)) end
  if row.retarget ~= "owner_target" and row.retarget ~= "none" then no("chase.retarget:" .. tostring(row.retarget)) end
  for _, k in ipairs(CHASE_NUMS) do
    -- leash is max_distance's old name (v3.md §7)
    local ref = c[k]; if k == "max_distance" and ref == nil then ref = c.leash end
    if ref == nil then no("chase." .. k .. " missing") else row[k] = num(ref, "chase." .. k) end
  end
  if row.turn_every_frames and math.type(row.turn_every_frames) ~= "integer" then
    local n = math.tointeger(row.turn_every_frames)
    if n then row.turn_every_frames = n else no("chase.turn_every_frames is not a whole number") end
  end
  local ability, why = M.reach_ability(doc, t.id)
  if ability then row.reach_ability = ability else no("chase reach ability: " .. why) end
  return row
end

-- Gold and experience to the enemy player who kills it (line B 3b1c87db; a dummy too), whole numbers.
local function bounty_into(row, bounty, num, no)
  for _, k in ipairs(sorted(bounty or {})) do
    if k ~= "gold" and k ~= "xp" then no("bounty." .. k)
    else
      local b = num(bounty[k], "bounty." .. k)
      if b and (b < 0 or b ~= math.floor(b)) then no("bounty." .. k .. " is not a whole number")
      elseif b and b > 0 then row["bounty_" .. k] = math.tointeger(b) end
    end
  end
end

-- A template with no attack that is a dummy (`ai = "none"` or tag `dummy`): the host's dummy body
-- (line B ee50b7e7). Its life and defenses (b8a572ea: a dummy with life can be destroyed), vision
-- and owner rule come from the document.
local function dummy_rows(doc, t, value)
  local problems = {}
  local function no(what) problems[#problems + 1] = what end
  local known = { id = true, source_id = true, values = true, text = true, notes = true, provenance = true, quirks = true,
    max_level = true, attack = true, vision = true, owner_death = true, ai = true, tags = true, passive = true, stats = true,
    statuses = true, flying = true, bounty = true }
  -- a chase_target dummy moves by its chase table and acquires within acquisition_range (E406)
  if t.ai == "chase_target" then known.chase = true; known.acquisition_range = true end
  -- a pathing blocker (line B E426): immobile, its square's half side its collision radius
  if t.blocks_pathing ~= nil then known.blocks_pathing = true; known.collision_radius = true end
  for _, k in ipairs(sorted(t)) do if not known[k] then no(k .. " on a dummy") end end
  if t.ai == "chase_target" and type(t.chase) ~= "table" then no("ai chase_target without a chase table") end
  local sp = statuses_problem(t); if sp then no(sp) end
  local stats = t.stats or {}
  for _, k in ipairs(sorted(stats)) do
    -- move_speed: a mobile dummy the scripts order around (line B E417)
    if k ~= "hp" and k ~= "hp_regen" and k ~= "armor" and k ~= "magic_resist" and k ~= "mana" and k ~= "mana_regen"
      and k ~= "move_speed" then no("stats." .. k .. " on a dummy") end
  end
  if t.ai ~= nil and t.ai ~= "none" and t.ai ~= "chase_target" then no("ai:" .. tostring(t.ai) .. " on a dummy") end
  -- tag building: a building unit like a tower (line B E428): immobile, no chase
  for _, tag in ipairs(t.tags or {}) do if tag ~= "dummy" and tag ~= "summon" and tag ~= "building" then no("tag:" .. tag) end end
  local building = has(t.tags, "building")
  if t.owner_death ~= nil and t.owner_death ~= "persist" and t.owner_death ~= "expire" then no("owner_death:" .. tostring(t.owner_death)) end
  local p = t.passive or {}
  for _, k in ipairs(sorted(p)) do if k ~= "reveal" and k ~= "states" then no("passive." .. k .. " on a dummy") end end
  if p.reveal ~= nil and p.reveal ~= "public" then no("passive.reveal:" .. tostring(p.reveal)) end
  for _, st in ipairs(p.states or {}) do if not DUMMY_STATES[st] then no("passive.states:" .. st .. " on a dummy") end end
  local rows = {}
  for rank = 1, t.max_level or 1 do
    local function num(ref, what)
      if ref == nil then return nil end
      local v, why = value(ref, rank)
      if v == nil then no(what .. " is " .. why) end
      return v
    end
    local row = { id = (t.max_level or 1) > 1 and (t.id .. "_r" .. rank) or t.id, authored_package = doc.__package,
      display_name = t.text and t.text.zh_TW and t.text.zh_TW.name or t.id,
      dummy = true, immobile = true, owner_death = t.owner_death or "persist" }
    vision_into(row, t.vision or {}, num, no)
    row.hp = num(stats.hp, "stats.hp")
    row.hp_regen = num(stats.hp_regen, "stats.hp_regen")
    row.armor = num(stats.armor, "stats.armor")
    row.magic_resistance = num(stats.magic_resist, "stats.magic_resist")
    mana_into(row, stats, num, no)
    bounty_into(row, t.bounty, num, no)
    local ms = num(stats.move_speed, "stats.move_speed")
    if ms and (ms < 0 or ms > HOST_RANGE_MAX) then no("stats.move_speed " .. ms .. " outside 0.." .. HOST_RANGE_MAX)
    elseif ms and ms > 0 then row.move_speed, row.immobile = ms, nil end
    if t.flying ~= nil and type(t.flying) ~= "boolean" then no("flying of a " .. type(t.flying)) elseif t.flying then row.flying = true end
    if t.blocks_pathing ~= nil then
      if t.blocks_pathing ~= true then no("blocks_pathing of " .. tostring(t.blocks_pathing))
      elseif not row.immobile or t.ai == "chase_target" then no("blocks_pathing on a dummy that moves")
      else
        local cr = num(t.collision_radius, "collision_radius")
        if cr == nil then no("blocks_pathing without collision_radius")
        elseif cr < 1 or cr > 512 then no("collision_radius " .. cr .. " outside 1..512 of a pathing blocker")
        else row.blocks_pathing, row.collision_radius = true, cr end
      end
    end
    if building then
      if not row.immobile or t.ai == "chase_target" then no("tag:building on a dummy that moves") else row.building = true end
    end
    if t.ai == "chase_target" and type(t.chase) == "table" then
      row.immobile, row.ai = nil, "chase_target"
      row.chase = chase_row(doc, t, num, no)
      local acq = num(t.acquisition_range, "acquisition_range")
      if acq and (acq < 0 or acq > HOST_RANGE_MAX) then no("acquisition_range " .. acq .. " outside 0.." .. HOST_RANGE_MAX)
      elseif acq then row.acquisition_range = acq end
    end
    rows[#rows + 1] = row
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
  if (not t.attack or t.attack.kind == "none") and (t.ai == "none" or t.ai == "chase_target" or has(t.tags, "dummy")) then return dummy_rows(doc, t, value) end
  local known = { id = true, source_id = true, values = true, text = true, notes = true, provenance = true, quirks = true,
    max_level = true, stats = true, attack = true, bounty = true, vision = true, collision_radius = true,
    acquisition_range = true, owner_death = true, ai = true, tags = true, passive = true, statuses = true, turn_rate = true, flying = true }
  for _, k in ipairs(sorted(t)) do if not known[k] then no(k) end end
  local sp = statuses_problem(t); if sp then no(sp) end
  local stats = t.stats or {}
  for _, k in ipairs(sorted(stats)) do
    if k ~= "hp" and k ~= "hp_regen" and k ~= "armor" and k ~= "magic_resist" and k ~= "move_speed" and k ~= "mana" and k ~= "mana_regen" then no("stats." .. k) end
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
  -- idle: never acquires a target by itself (host summons::generic, line B 53efe6a5)
  if t.ai ~= "guard" and t.ai ~= "owner_controlled" and t.ai ~= "idle" then no("ai:" .. tostring(t.ai)) end
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
    row.hp_regen = num(stats.hp_regen, "stats.hp_regen")
    row.move_speed = num(stats.move_speed, "stats.move_speed")
    if row.move_speed == nil then no("stats.move_speed missing") end
    mana_into(row, stats, num, no)
    if t.flying ~= nil and type(t.flying) ~= "boolean" then no("flying of a " .. type(t.flying)) elseif t.flying then row.flying = true end
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
    -- the ranged attack's projectile (the host's default when absent)
    if attack.projectile_speed ~= nil then
      local ps = num(attack.projectile_speed, "attack.projectile_speed")
      if ps and (ps <= 0 or ps > HOST_RANGE_MAX) then no("attack.projectile_speed " .. ps .. " outside 0.." .. HOST_RANGE_MAX)
      elseif ps then row.projectile_speed = ps end
    end
    -- the host acquires within max(acquisition_range, attack range): a smaller one has no row value
    if t.acquisition_range ~= nil then
      local acq = num(t.acquisition_range, "acquisition_range")
      if acq and (acq < 0 or acq > HOST_RANGE_MAX) then no("acquisition_range " .. acq .. " outside 0.." .. HOST_RANGE_MAX)
      elseif acq and row.attack_range and acq < row.attack_range then no("acquisition_range " .. acq .. " below attack.range")
      elseif acq then row.acquisition_range = acq end
    end
    bounty_into(row, t.bounty, num, no)
    vision_into(row, t.vision or {}, num, no)
    row.collision_radius = num(t.collision_radius, "collision_radius") or 0
    -- the source's MovementTurnRate (0..10; the host turns at turn_rate / 0.03 rad/s, line B 530f1f9f)
    if t.turn_rate ~= nil then
      local tr = num(t.turn_rate, "turn_rate")
      if tr and (tr < 0 or tr > 10) then no("turn_rate " .. tr .. " outside 0..10") elseif tr then row.turn_rate = tr end
    end
    row.immobile = row.move_speed == 0 or nil
    row.follow_start_distance, row.follow_stop_distance = 0, 0
    row.target_priority = "nearest"
    if t.ai == "idle" then row.ai = "idle" end
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

-- Every authored summon (templates/authored_heroes.lua) and every row of ours must load from
-- templates/summons.lua: a lost row makes its summon fail only at run time (E341). Returns the
-- missing ids.
function M.missing_from_content(root)
  local saved = package.path
  package.path = root .. "/scripts/?.lua;" .. package.path
  local context = require("content_builder").new(root .. "/scripts/lua_data")
  package.path = saved
  local loaded = {}
  for _, row in ipairs(context.include("templates/summons.lua")) do loaded[row.id] = true end
  local missing = {}
  for _, row in ipairs(context.include("templates/authored_heroes.lua").summons or {}) do
    if not loaded[row.id] then missing[#missing + 1] = row.id end
  end
  local f = io.open(root .. "/" .. M.OUT, "rb")
  if f then
    f:close()
    for _, row in ipairs(context.include("templates/v3_summons.lua")) do
      if not loaded[row.id] then missing[#missing + 1] = row.id end
    end
  end
  return missing
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

-- require("v3_summons") passes the module name; a script run passes its arguments (with
-- `--check` the old `... == nil` test skipped the CLI, so the check never ran, E341)
if ... ~= "v3_summons" then
  local root = "."
  local check = arg[1] == "--check"
  local missing = M.missing_from_content(root)
  if #missing > 0 then
    io.stderr:write("templates/summons.lua does not load " .. #missing .. " summon row(s): " .. table.concat(missing, ", ") .. "\n")
    os.exit(1)
  end
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
