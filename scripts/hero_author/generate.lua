-- Package (schema v2) -> template entries for scripts/lua_data/templates.
--
-- Generated content lives in one file, templates/authored_heroes.lua, merged
-- append-only: an existing package keeps its position and its ability IDs,
-- new packages and new abilities go at the end, and abilities a package drops
-- stay as tombstones so numeric content IDs never shift.
local serialize = require("hero_source.serialize")
local M = {}

M.AUTHORED_FILE = "templates/authored_heroes.lua"
-- extra1 is the fifth (D) slot: positional, so it needs all four basic slots.
M.SLOT_INDEX = { Q = 1, W = 2, E = 3, R = 4, extra1 = 5 }
M.MAX_SLOTS = 5
local TARGET_TYPE = { none = "none", self = "none", unit = "unit", point = "point", direction = "point" }

local function per_rank(value, max_level)
  if type(value) == "table" then
    local out = {}
    for i = 1, max_level do out[i] = value[i] end
    return out
  end
  local out = {}
  for i = 1, max_level do out[i] = value end
  return out
end

local function rank_at(value, i)
  if type(value) == "table" then return value[i] end
  return value
end

-- Flattens numeric effect parameters into extras keys the Rust handler reads
-- through the active runtime content lookup (`extras[key][rank]`).
local function extras_of(ab)
  local extras = {}
  local function put(key, value)
    assert(extras[key] == nil, ab.id .. ": duplicate generated extras key " .. key)
    extras[key] = per_rank(value, ab.max_level)
  end
  local counts = {}
  for _, e in ipairs(ab.effects or {}) do
    counts[e.kind] = (counts[e.kind] or 0) + 1
    local prefix = e.kind .. (counts[e.kind] > 1 and ("_" .. counts[e.kind]) or "")
    local keys = {}
    for k in pairs(e) do keys[#keys + 1] = k end
    table.sort(keys)
    for _, k in ipairs(keys) do
      local v = e[k]
      if type(v) == "number" or (type(v) == "table" and type(v[1]) == "number") then
        put(prefix .. "_" .. k, v)
      elseif (k == "stats" or k == "classes") and type(v) == "table" then
        local names = {}
        for name in pairs(v) do names[#names + 1] = name end
        table.sort(names)
        for _, name in ipairs(names) do put(prefix .. "_" .. name, v[name]) end
      end
    end
  end
  local shape = ab.targeting and ab.targeting.shape
  if shape and shape.radius ~= nil then put("shape_radius", shape.radius) end
  if shape and shape.width ~= nil then put("shape_width", shape.width) end
  if shape and shape.half_angle ~= nil then
    -- Package cones author the half angle in degrees; the content model reads radians.
    local radians = per_rank(shape.half_angle, ab.max_level)
    for i, v in ipairs(radians) do radians[i] = math.rad(v) end
    put("shape_half_angle", radians)
  end
  local channel = ab.timing and ab.timing.channel
  if channel then
    put("channel_interval", channel.interval)
    put("channel_pulses", channel.pulses)
    if channel.radius ~= nil then put("channel_radius", channel.radius) end
  end
  local charges = ab.resources and ab.resources.charges
  if charges then
    put("charges_max", charges.max)
    put("charges_recharge_seconds", charges.recharge_seconds)
  end
  return extras
end

-- `pres` is the package presentation.lua row of the ability (optional): its skill
-- VFX layers become ue.vfx (omfue codegen AbilityVfx, OmAbilityVfx.h).
function M.ability_entry(pkg_id, ab, pres)
  local tg, tm, res = ab.targeting or {}, ab.timing or {}, ab.resources or {}
  local levels = {}
  for i = 1, ab.max_level do
    levels[i] = {
      cooldown = rank_at(res.cooldown, i) or 0,
      mana_cost = rank_at(res.mana_cost, i) or 0,
      cast_time = rank_at(tm.cast_point, i) or 0,
      range = rank_at(tg.range, i) or 0,
      required_hero_level = ab.required_hero_level and ab.required_hero_level[i] or nil,
    }
  end
  local entry = {
    id = ab.id,
    -- Source NOT_LEARNABLE: each rank is learned at its required hero level, without points.
    auto_learn = ab.auto_learn or nil,
    -- Who a unit-target ability may be aimed at, for Rust handlers whose
    -- effects do not say it (omoba-content-model ability_unit_allegiance).
    unit_target = tg.kind == "unit" and (tg.team == "ally" or tg.team == "enemy") and tg.team or nil,
    display_name = ab.name,
    description = ab.description,
    ability_type = ab.ability_type,
    target_type = assert(TARGET_TYPE[tg.kind], ab.id .. ": unmapped targeting kind " .. tostring(tg.kind)),
    cast_type = tm.channel and "channeled" or "instant",
    max_level = ab.max_level,
    levels = levels,
    extras = extras_of(ab),
    authored_package = pkg_id,
  }
  -- Package targeting.shape as a presentation-only footprint (omoba-content-model
  -- AbilityTargetingShape). HUD cards publish it only where no gameplay contract
  -- (effects, aim_geometry, projectile) already publishes a footprint.
  local shape = tg.shape or {}
  if shape.kind == "circle" and shape.radius ~= nil then
    entry.targeting_shape = { kind = "circle", radius_key = "shape_radius" }
  elseif shape.kind == "line" and shape.width ~= nil then
    entry.targeting_shape = { kind = "line", width_key = "shape_width" }
  elseif shape.kind == "cone" and shape.half_angle ~= nil then
    entry.targeting_shape = { kind = "cone", half_angle_key = "shape_half_angle" }
  else
    assert(shape.kind == nil or shape.kind == "none" or shape.kind == "circle" or shape.kind == "line" or shape.kind == "cone",
      ab.id .. ": unmapped targeting shape kind " .. tostring(shape.kind))
  end
  -- A range-0 point/direction ability that moves the caster (a dash with
  -- max_distance) acts where it lands: publish that distance as the
  -- presentation-only reach (omoba-content-model targeting_reach_key), named
  -- like the generated extras key. The cast stays range-free.
  local range_free = true
  for i = 1, ab.max_level do if levels[i].range ~= 0 then range_free = false end end
  if range_free and (tg.kind == "point" or tg.kind == "direction") then
    local counts = {}
    for _, e in ipairs(ab.effects or {}) do
      counts[e.kind] = (counts[e.kind] or 0) + 1
      if e.kind == "dash" and e.max_distance ~= nil and entry.targeting_reach_key == nil then
        entry.targeting_reach_key = e.kind .. (counts[e.kind] > 1 and ("_" .. counts[e.kind]) or "") .. "_max_distance"
      end
    end
  end
  if tm.channel then
    -- Host-owned pulse channel (G-2): omoba-content-model ChannelDelivery. Rust
    -- handlers receive it as host execution and run per pulse in on_channel_pulse.
    local shape = tg.shape or {}
    entry.channel = { anchor = tm.channel.anchor, interval_key = "channel_interval", pulses_key = "channel_pulses",
      radius_key = tm.channel.radius ~= nil and "channel_radius" or (shape.radius ~= nil and "shape_radius" or nil) }
  end
  if res.charges then
    -- Host-owned stored uses (G-2): omoba-content-model AbilityCharges.
    entry.charges = { max_key = "charges_max", recharge_key = "charges_recharge_seconds" }
  end
  for _, e in ipairs(ab.effects or {}) do
    if e.kind == "cooldown_refund_on_cast" then
      -- omoba-content-model CastCooldownRefund (host-owned, replica-recomputed).
      entry.cast_cooldown_refund = { seconds_key = "cooldown_refund_on_cast_seconds" }
    end
    if e.kind == "attack_orb" then
      -- omoba-content-model AttackOrb reads slows as fractions of one.
      local function fraction(v)
        local out = per_rank(v, ab.max_level)
        for i, x in ipairs(out) do out[i] = x / 100 end
        return out
      end
      entry.attack_orb = { mana_per_attack_key = "attack_orb_mana_per_attack", amount_key = "attack_orb_amount", damage_kind = e.damage_kind }
      if e.move_slow_pct then entry.extras.attack_orb_move_slow = fraction(e.move_slow_pct); entry.attack_orb.move_slow_key = "attack_orb_move_slow" end
      if e.attack_slow_pct then entry.extras.attack_orb_attack_slow = fraction(e.attack_slow_pct); entry.attack_orb.attack_slow_key = "attack_orb_attack_slow" end
      if e.slow_duration then entry.attack_orb.slow_duration_key = "attack_orb_slow_duration" end
    end
  end
  local impl = ab.implementation or {}
  if impl.kind == "generic" then entry.effects = impl.effects end
  if pres and pres.vfx then
    local vfx = {}
    for key, value in pairs(pres.vfx) do vfx[key] = value end
    entry.ue = { vfx = vfx }
  end
  return entry
end

-- Birth rank of a slot: the package's explicit moba_loadout, otherwise rank 1
-- unless the first rank is gated above hero level 1 (e.g. ultimates at 6).
function M.birth_rank(character, ab)
  local explicit = character.moba_loadout and character.moba_loadout.ranks
    and character.moba_loadout.ranks[M.SLOT_INDEX[ab.action]]
  if explicit ~= nil then return explicit end
  local first = ab.required_hero_level and ab.required_hero_level[1] or 1
  return first <= 1 and 1 or 0
end

-- Lua replica of omoba_content_model::validate_moba_loadout, so generated
-- content fails here instead of inside cargo. Returns nil or an error string.
function M.check_moba_loadout(hero, abilities_by_id)
  if hero.tombstone then return nil end
  local list = hero.abilities or {}
  if #list > M.MAX_SLOTS then return "hero '" .. hero.id .. "': MOBA loadout exceeds " .. M.MAX_SLOTS .. " slots" end
  local ranks = (hero.moba_loadout and hero.moba_loadout.ranks) or { 1, 1, 1, 1 }
  for slot, id in ipairs(list) do
    local ability = abilities_by_id[id]
    if not ability or ability.tombstone then return "hero '" .. hero.id .. "': unknown MOBA ability '" .. id .. "'" end
    local rank = ranks[slot] or 1
    local max_level = ability.max_level or 4
    local level = rank > 0 and ability.levels and ability.levels[rank]
    if rank > max_level or (rank > 0 and ((level and level.required_hero_level) or 1) > 1) then
      return "hero '" .. hero.id .. "': invalid MOBA birth rank " .. rank .. " for '" .. id .. "'"
    end
  end
  for slot = #list + 1, M.MAX_SLOTS do
    if (ranks[slot] or 1) > 1 then return "hero '" .. hero.id .. "': MOBA loadout assigns rank to an empty slot" end
  end
  return nil
end

-- package: { character, abilities, ... } (loaded v2 tables); slice: set of ability ids or nil
function M.package_entries(package, slice)
  local character, abilities = package.character, package.abilities.abilities
  local hero_id = character.hero.id
  local built, skipped = {}, {}
  -- Extra actions (extra1..3) have no MOBA loadout slot until
  -- extra_action_slot exists; they are not emitted as orphan content.
  for _, ab in ipairs(abilities) do
    if (slice == nil or slice[ab.id]) and M.SLOT_INDEX[ab.action] then
      built[#built + 1] = M.ability_entry(hero_id, ab, ((package.presentation or {}).abilities or {})[ab.id])
    else
      skipped[#skipped + 1] = ab.id
    end
  end
  local slots, ranks = {}, {}
  for _, ab in ipairs(abilities) do
    local idx = M.SLOT_INDEX[ab.action]
    if (slice == nil or slice[ab.id]) and idx then slots[idx] = ab.id; ranks[idx] = M.birth_rank(character, ab) end
  end
  local skill_points = character.moba_loadout and character.moba_loadout.skill_points
  local result = { package = hero_id, abilities = built, skipped = skipped, summons = {} }
  for _, s in ipairs(package.summons and package.summons.summons or {}) do
    local row = {}
    for key, value in pairs(s) do row[key] = value end
    row.authored_package = hero_id
    result.summons[#result.summons + 1] = row
  end
  if character.identity.integration == "update_existing" then
    result.hero_update = { abilities_by_slot = slots, ranks_by_slot = ranks, skill_points = skill_points,
      rust_module = character.identity.rust_module }
  else
    -- A hero row is a dense list: a slice that leaves a slot empty moves later
    -- abilities up, and their birth ranks move with them.
    local list, dense = {}, {}
    for i = 1, 4 do if slots[i] then list[#list + 1] = slots[i]; dense[#list] = ranks[i] end end
    if slots[5] then
      -- The D slot is positional (fifth entry); it cannot move up.
      assert(#list == 4, hero_id .. ": the extra1 (D) slot needs all four basic slots in the build")
      list[5] = slots[5]; dense[5] = ranks[5]
    end
    ranks = dense
    local hs, atk = character.base_stats or {}, character.attack or {}
    result.hero = {
      id = hero_id,
      rust_module = character.identity.rust_module,
      display_name = character.hero.display_name,
      title = character.hero.title,
      background = character.hero.combat_read,
      abilities = list,
      attack_range = atk.range,
      base_damage = hs.base_damage,
      base_armor = hs.base_armor,
      base_hp = hs.base_hp,
      base_mana = hs.base_mana,
      move_speed = hs.move_speed,
      turn_speed = hs.turn_speed or 720,
      moba_loadout = { ranks = list[5] and { ranks[1] or 0, ranks[2] or 0, ranks[3] or 0, ranks[4] or 0, ranks[5] or 0 }
        or { ranks[1] or 0, ranks[2] or 0, ranks[3] or 0, ranks[4] or 0 }, skill_points = skill_points or 0 },
      unit_script = character.identity.unit_script or nil,
      authored_package = hero_id,
    }
  end
  return result
end

-- previous: the current authored data table (or nil); entries: package_entries result.
-- Returns the merged authored data. Never reorders existing rows.
function M.merge(previous, entries)
  local data = previous or {}
  data.packages = data.packages or {}
  data.heroes = data.heroes or {}
  data.hero_updates = data.hero_updates or {}
  data.abilities = data.abilities or {}
  data.summons = data.summons or {}
  local pkg = entries.package
  -- summons: same append-only rule as abilities (replace in place, append new,
  -- tombstone rows this package no longer declares).
  local new_summons, summon_position = {}, {}
  for _, s in ipairs(entries.summons or {}) do new_summons[s.id] = s end
  for i, s in ipairs(data.summons) do summon_position[s.id] = i end
  for i, s in ipairs(data.summons) do
    if new_summons[s.id] then
      assert(s.authored_package == pkg, "summon id " .. s.id .. " already belongs to package " .. tostring(s.authored_package))
      data.summons[i] = new_summons[s.id]
    elseif s.authored_package == pkg and not s.tombstone then
      data.summons[i] = { id = s.id, display_name = s.display_name, tombstone = true, authored_package = pkg }
    end
  end
  for _, s in ipairs(entries.summons or {}) do
    if not summon_position[s.id] then data.summons[#data.summons + 1] = s end
  end
  local found = false
  for _, p in ipairs(data.packages) do if p.id == pkg then found = true end end
  if not found then data.packages[#data.packages + 1] = { id = pkg } end
  -- abilities: replace in place, append new, tombstone dropped ones of this package
  local new_by_id, position = {}, {}
  for _, a in ipairs(entries.abilities) do new_by_id[a.id] = a end
  for i, a in ipairs(data.abilities) do position[a.id] = i end
  local skipped = {}
  for _, id in ipairs(entries.skipped or {}) do skipped[id] = true end
  for i, a in ipairs(data.abilities) do
    if new_by_id[a.id] then
      assert(a.authored_package == pkg, "ability id " .. a.id .. " already belongs to package " .. tostring(a.authored_package))
      data.abilities[i] = new_by_id[a.id]
    elseif a.authored_package == pkg and not a.tombstone and not skipped[a.id] then
      data.abilities[i] = { id = a.id, tombstone = true, authored_package = pkg }
    end
  end
  for _, a in ipairs(entries.abilities) do
    if not position[a.id] then data.abilities[#data.abilities + 1] = a end
  end
  if entries.hero then
    local replaced = false
    for i, h in ipairs(data.heroes) do
      if h.id == entries.hero.id then data.heroes[i] = entries.hero; replaced = true end
    end
    if not replaced then data.heroes[#data.heroes + 1] = entries.hero end
  end
  if entries.hero_update then
    local prev = data.hero_updates[pkg]
    local slots, ranks = {}, {}
    local u = entries.hero_update
    for i = 1, M.MAX_SLOTS do
      slots[i] = u.abilities_by_slot[i] or (prev and prev.abilities_by_slot[i])
      ranks[i] = (u.ranks_by_slot or {})[i] or (prev and prev.ranks_by_slot and prev.ranks_by_slot[i])
    end
    local skill_points = u.skill_points
    if skill_points == nil and prev then skill_points = prev.skill_points end
    data.hero_updates[pkg] = { abilities_by_slot = slots, ranks_by_slot = ranks, skill_points = skill_points,
      rust_module = u.rust_module }
  end
  return data
end

function M.render(data)
  local header = "-- GENERATED by scripts/hero_author.lua build from docs/character_pipeline/packages; do not edit.\n" ..
    "-- Append-only: rows keep their position; dropped abilities become tombstones.\n"
  local body = serialize.serialize(data, header)
  -- Template builders return a function(ctx).
  return (body:gsub("^(.-)return ", "%1local data = ", 1)) .. "return function(ctx) return data end\n"
end

function M.load(text)
  local chunk = assert(load(text, "=authored_heroes", "t", {}))
  return chunk()()
end

return M
