-- Source item datadriven active -> the game's item active (E160).
--
-- Reads a catalog item's `active_kv` (OnSpellStart actions and Modifiers blocks, as written in
-- history_nobu) and returns { active, cooldown, mana_cost, applied } when every part of it
-- maps to an engine capability, else nil and the reason. `applied` names the modifiers the
-- active applies: their properties belong to the active, not to the item's passive stats.
--
-- Supported today (kind = "apply_modifiers"):
--   * ApplyModifier with Target CASTER                     -> a modifier on the caster
--   * ActOnTargets {Teams FRIENDLY, Center CASTER, Types HERO[|BASIC], Radius}
--       Action ApplyModifier Target TARGET                 -> an area modifier on allies
--   * Heal {Target CASTER, HealAmount}                      -> caster heal
--   * RunScript with a `Mana` parameter                     -> caster mana restore
--   * RunScript without parameters, FireSound, FireEffect   -> presentation (no game effect)
--   * ApplyModifier naming an undeclared modifier           -> nothing (the engine applies none)
--   * SpendCharge (also inside a DelayedAction) on a 1-charge item -> consumable (used up)
-- Supported (kind = "unit_heal"): a UNIT_TARGET item whose AbilityUnitTargetTeam is FRIENDLY and
-- whose only effect is Heal {Target TARGET} -> heal one allied non-building unit in cast range.
-- An ApplyModifier `Duration` overrides the modifier's own.
-- A modifier may carry Properties (see STATS), Duration, and presentation keys only. The
-- source has no item vscripts, so a parameterless RunScript is taken as presentation only when
-- every number the 【使用】 text names is explained by the KV values above.
local M = {}

-- Source property -> { BuffStore StatKey, scale }; percentages become fractions.
M.STATS = {
  MODIFIER_PROPERTY_BASEDAMAGEOUTGOING_PERCENTAGE = { "basedamageoutgoing_percentage", 0.01 },
  MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT = { "attackspeed_bonus_constant", 1 },
  MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE = { "movespeed_bonus_percentage", 0.01 },
  MODIFIER_PROPERTY_MOVESPEED_BONUS_CONSTANT = { "movespeed_bonus_constant", 1 },
  MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS = { "physical_armor_bonus", 1 },
  MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT = { "health_regen_constant", 1 },
  MODIFIER_PROPERTY_MANA_REGEN_CONSTANT = { "mana_regen_constant", 1 },
}

-- RunScript parameters that only pick presentation (the source's `sound` helper).
-- `Target` only names which keys the engine hands the script (POINT: keys.target_points).
local PRESENTATION_PARAMS = { ScriptFile = true, Function = true, sound = true, time = true, Target = true }

-- What one RunScript does, from the catalog's analysis of the script function (E169: the
-- history_nobu scripts are read, never assumed). Returns { mana = n, cooldown_items = {...} } or
-- nil and the reason.
local function run_script(item, body)
  local file, fn, mana = nil, nil, nil
  for _, p in ipairs(body or {}) do
    if p.key == "ScriptFile" then file = p.value elseif p.key == "Function" then fn = p.value end
  end
  local analysis
  for _, ref in ipairs(item.scripts or {}) do
    if ref.file == file and ref["function"] == fn then analysis = ref.analysis end
  end
  local reviewed = analysis and analysis.reviewed_effect
  local reviewed_value
  for _, p in ipairs(body or {}) do
    if p.key == "Mana" then mana = tonumber(p.value); if not mana or mana <= 0 then return nil, "RunScript Mana" end
    elseif reviewed and p.key == reviewed.seconds then reviewed_value = tonumber(p.value)
    elseif not PRESENTATION_PARAMS[p.key] then return nil, "RunScript parameter " .. p.key end
  end
  if not analysis or not analysis.found then return nil, ("script %s#%s is not in the source"):format(tostring(file), tostring(fn)) end
  if analysis and analysis.reviewed_mismatch then
    return nil, ("script %s#%s no longer has its reviewed fragment %s"):format(file, fn, analysis.reviewed_mismatch)
  end
  if reviewed then
    -- A reviewed function whose whole effect is an engine capability (script.REVIEWED_EFFECTS).
    if reviewed.kind == "target_modifier" then
      return { cooldown_items = {}, effects = { { kind = "apply_modifier", modifier = reviewed.modifier } } }
    elseif reviewed.kind == "point_area" then
      return { cooldown_items = {}, effects = {}, area = reviewed.area }
    end
    if reviewed.kind ~= "magic_immunity" or not reviewed_value or reviewed_value <= 0 then
      return nil, ("script %s#%s needs %s"):format(file, fn, tostring(reviewed.seconds))
    end
    return { magic_immunity = reviewed_value, cooldown_items = {}, effects = {} }
  end
  if mana then
    -- `caster:SetMana(caster:GetMana()+keys.Mana)`: a plain restore of the Mana parameter.
    for _, call in ipairs(analysis.calls) do
      if call ~= "GetMana" and call ~= "SetMana" then return nil, ("script %s#%s does more than restore Mana (%s)"):format(file, fn, call) end
    end
    return { mana = mana, cooldown_items = {}, effects = {} }
  end
  if #(analysis.calls or {}) > 0 and not analysis.reviewed_presentation then
    return nil, ("script %s#%s calls %s"):format(file, fn, table.concat(analysis.calls, ","))
  end
  return { cooldown_items = analysis.cooldown_items or {}, effects = analysis.effects or {}, area = analysis.area }
end

local PRESENTATION = {
  IsHidden = true, IsBuff = true, IsDebuff = true, IsPurgable = true, EffectName = true,
  EffectAttachType = true, TextureName = true, StatusEffectName = true, StatusEffectPriority = true,
  OverrideAnimation = true,
}

local function find(nodes, key)
  for _, n in ipairs(nodes or {}) do if n.key == key then return n end end
end
local function value(nodes, key) local n = find(nodes, key); return n and n.value end
local function flags(text)
  local set = {}
  for word in tostring(text or ""):gmatch("[%w_]+") do set[word] = true end
  return set
end
local function only(set, allowed)
  for word in pairs(set) do if not allowed[word] then return false end end
  return true
end

-- One Modifiers block -> { name, duration, stats } or nil, reason; false for an undeclared name.
local function modifier(blocks, name, numbers, duration)
  local block = find(blocks, name)
  if not block or not block.children then return false end
  local out = { name = name, stats = {} }
  for _, n in ipairs(block.children) do
    if n.key == "Properties" then
      for _, p in ipairs(n.children or {}) do
        local map = M.STATS[p.key]
        local v = tonumber(p.value)
        if not v then return nil, name .. " property " .. p.key .. " is not a number" end
        if v ~= 0 then
          if not map then return nil, name .. " property " .. p.key:gsub("^MODIFIER_PROPERTY_", "") .. " is not supported" end
          out.stats[map[1]] = (out.stats[map[1]] or 0) + v * map[2]
          numbers[#numbers + 1] = math.abs(v)
        end
      end
    elseif n.key == "Duration" then
      out.duration = tonumber(n.value)
      numbers[#numbers + 1] = out.duration
    elseif n.key == "Passive" then
      if n.value ~= "0" then return nil, name .. " is passive" end
    elseif not PRESENTATION[n.key] then
      return nil, name .. " uses " .. n.key
    end
  end
  if duration then out.duration = duration; numbers[#numbers + 1] = duration end
  -- No Duration: the source modifier stays until removed (E173); the game stores it as 0.
  if not out.duration then out.duration = 0 end
  if out.duration < 0 then return nil, name .. " has a negative duration" end
  if not next(out.stats) then return nil, name .. " has no stat" end
  return out
end

-- The numbers the 【使用】 clause of a description names (the whole text when it has no clause).
function M.described_numbers(description)
  local text = tostring(description or "")
  local clause = text:match("【使用】(.*)") or (not text:find("【", 1, true) and text) or nil
  local out = {}
  if not clause then return out end
  clause = clause:gsub("【.*", "")
  for n in clause:gmatch("%d+%.?%d*") do out[#out + 1] = tonumber(n) end
  return out
end

-- A charge spend, directly or as the only action of a DelayedAction.
local function spends_charge(action)
  if action.key == "SpendCharge" then return true end
  if action.key ~= "DelayedAction" then return false end
  local act = find(action.children, "Action")
  local inner = act and act.children or {}
  return #inner == 1 and inner[1].key == "SpendCharge"
end

-- Shared tail: cooldown, mana cost, consumable and the description check.
local function finish(item, out, numbers, groups)
  -- Shared cooldowns: the script loops' item lists plus the KV AbilitySharedCooldown group.
  local share, seen = {}, { [item.key] = true }
  local function add(key) if not seen[key] then seen[key] = true; share[#share + 1] = key end end
  for _, key in ipairs(out.cooldown_items or {}) do add(key) end
  if item.shared_cooldown and groups then
    for _, key in ipairs(groups[item.shared_cooldown] or {}) do add(key) end
  end
  table.sort(share)
  out.cooldown_items = nil
  out.shares_cooldown_with = #share > 0 and share or nil
  local cooldown, mana_cost = tonumber(item.cooldown) or 0, tonumber(item.mana_cost) or 0
  if cooldown <= 0 then return nil, "no cooldown" end
  numbers[#numbers + 1] = cooldown
  numbers[#numbers + 1] = mana_cost
  for _, p in ipairs(item.properties or {}) do
    local v = tonumber(p.value)
    if v and v ~= 0 then numbers[#numbers + 1] = math.abs(v) end
  end
  if out.consumable and tostring(item.initial_charges) ~= "1" then return nil, "charges other than one" end
  -- The source code (KV and its scripts, E176) decides; a described number it does not explain is
  -- reported, not a refusal (the source's text is often older than its code).
  for _, n in ipairs(M.described_numbers(item.description)) do
    local explained = false
    for _, k in ipairs(numbers) do if math.abs(k - n) < 1e-9 then explained = true end end
    if not explained then
      out.description_unexplained = out.description_unexplained or {}
      table.insert(out.description_unexplained, n)
    end
  end
  out.cooldown, out.mana_cost = cooldown, mana_cost
  return out
end

-- A friendly unit-target item whose only effect is a Heal on the target.
local property_number -- defined with the aura translation below

-- Controls the engine applies with a duration (UnitHex control).
local STATES = { MODIFIER_STATE_STUNNED = "stun", MODIFIER_STATE_ROOTED = "root", MODIFIER_STATE_SILENCED = "silence" }

-- What applying `name` to the enemy target does: { kind, duration } control or nil, reason.
local function target_control(item, name, duration)
  if name == "modifier_stunned" then
    -- Dota's own stun; the script gives its duration.
    if not duration or duration <= 0 then return nil, "modifier_stunned without a duration" end
    return { kind = "stun", duration = duration }
  end
  local block = find((item.active_kv or {}).modifiers or {}, name)
  if not block or not block.children then return nil, "target modifier " .. name .. " is not declared" end
  local control, own = nil, nil
  for _, n in ipairs(block.children) do
    if n.key == "States" then
      for _, s in ipairs(n.children or {}) do
        local kind = STATES[s.key]
        if not kind or s.value ~= "MODIFIER_STATE_VALUE_ENABLED" then return nil, name .. " state " .. tostring(s.key) end
        if control then return nil, name .. " has more than one control" end
        control = kind
      end
    elseif n.key == "Duration" then own = tonumber(n.value)
    elseif n.key == "Properties" then
      for _, p in ipairs(n.children or {}) do
        if property_number(item, p.value) ~= 0 then return nil, name .. " changes " .. p.key:gsub("^MODIFIER_PROPERTY_", "") end
      end
    elseif not PRESENTATION[n.key] and n.key ~= "Passive" and n.key ~= "IsStunDebuff" then
      return nil, name .. " uses " .. n.key
    end
  end
  if not control then return nil, name .. " has no control state" end
  local seconds = duration or own
  if not seconds or seconds <= 0 then return nil, name .. " has no duration" end
  return { kind = control, duration = seconds }
end

-- A target modifier that is only the source's named slow (E188): OnCreated RunScript
-- library/common/dummy.lua#Slow {name, ms_slow} and OnDestroy #ReturnSpeed {name}, with a
-- Duration. Returns { move_speed = fraction, duration } or false when it is not such a slow.
local SLOW_SCRIPT = "scripts/vscripts/library/common/dummy.lua"
local function target_slow(item, name, duration)
  local block = find((item.active_kv or {}).modifiers or {}, name)
  if not block or not block.children then return false end
  local slow, restored, own = nil, false, nil
  for _, n in ipairs(block.children) do
    if n.key == "OnCreated" or n.key == "OnDestroy" then
      local run = n.children and #n.children == 1 and n.children[1]
      if not run or run.key ~= "RunScript" or value(run.children, "ScriptFile") ~= SLOW_SCRIPT then return false end
      local fn = value(run.children, "Function")
      if n.key == "OnCreated" and fn == "Slow" and #run.children == 4 then slow = tonumber(value(run.children, "ms_slow"))
      elseif n.key == "OnDestroy" and fn == "ReturnSpeed" and #run.children == 3 then restored = true
      else return false end
    elseif n.key == "Duration" then own = tonumber(n.value)
    elseif not PRESENTATION[n.key] and n.key ~= "Passive" then return false end
  end
  local seconds = duration or own
  if not slow or slow >= 0 or slow <= -100 or not restored or not seconds or seconds <= 0 then return false end
  return { move_speed = -slow / 100, duration = seconds }
end

-- A target modifier whose only effect is a datadriven OnIntervalThink Damage on its TARGET
-- (E188): { damage, interval, duration, damage_type } or nil and the reason.
local function target_dot(item, name, duration)
  local block = find((item.active_kv or {}).modifiers or {}, name)
  if not block or not block.children then return nil, "target modifier " .. name .. " is not declared" end
  local dot, interval, own = nil, nil, nil
  for _, n in ipairs(block.children) do
    if n.key == "OnIntervalThink" then
      local d = n.children and #n.children == 1 and n.children[1]
      if not d or d.key ~= "Damage" or value(d.children, "Target") ~= "TARGET" then return nil, name .. " thinks something but Damage" end
      local kind = (value(d.children, "Type") or ""):match("^DAMAGE_TYPE_(%u+)$")
      dot = { damage = tonumber(value(d.children, "Damage")), damage_type = kind and kind:lower() }
    elseif n.key == "ThinkInterval" then interval = tonumber(n.value)
    elseif n.key == "Duration" then own = tonumber(n.value)
    elseif not PRESENTATION[n.key] and n.key ~= "Passive" then return nil, name .. " uses " .. n.key end
  end
  local seconds = duration or own
  if not dot or not dot.damage or dot.damage <= 0 or not dot.damage_type or not interval or interval <= 0
    or not seconds or seconds < interval then return nil, name .. " is not one periodic Damage" end
  dot.interval, dot.duration = interval, seconds
  return dot
end

-- An enemy unit-target item (E176): damage of one type plus at most one control on the target.
local function unit_hex(item, groups)
  local types = flags(item.target_types)
  if not types.DOTA_UNIT_TARGET_HERO or not only(types, { DOTA_UNIT_TARGET_HERO = true, DOTA_UNIT_TARGET_BASIC = true }) then
    return nil, "unit target types " .. tostring(item.target_types)
  end
  local range = tonumber(item.cast_range)
  if not range or range <= 0 then return nil, "unit target without cast range" end
  local damage, damage_type, control, consumable, projectile, spell, slow = 0, nil, nil, false, nil, nil, nil
  local numbers, cooldown_items, notes = { range }, {}, {}
  -- `inflictor`: KV Damage carries the ability, script AMHC:Damage does not (E188).
  local function add_damage(amount, kind, inflictor)
    if damage_type and damage_type ~= kind then return false end
    if spell ~= nil and spell ~= inflictor then return false end
    damage_type, spell = kind, inflictor
    damage = damage + amount
    numbers[#numbers + 1] = amount
    return true
  end
  local function add_control(name, duration)
    local s = target_slow(item, name, duration)
    if s then
      if slow then return nil, "more than one slow" end
      slow = s
      numbers[#numbers + 1] = s.duration
      numbers[#numbers + 1] = s.move_speed * 100
      return true
    end
    local c, why = target_control(item, name, duration)
    if not c then return nil, why end
    if control then return nil, "more than one control" end
    control = c
    numbers[#numbers + 1] = c.duration
    return true
  end
  local actions = {}
  for _, a in ipairs(item.active_kv.on_spell_start or {}) do actions[#actions + 1] = a end
  for _, a in ipairs(item.active_kv.on_projectile_hit or {}) do actions[#actions + 1] = a end
  for _, action in ipairs(actions) do
    local body = action.children
    if action.key == "Damage" then
      local amount, kind = tonumber(value(body, "Damage")), (value(body, "Type") or ""):match("^DAMAGE_TYPE_(%u+)$")
      if value(body, "Target") ~= "TARGET" or not amount or amount <= 0 or not kind then return nil, "Damage other than the target" end
      if not add_damage(amount, kind:lower(), true) then return nil, "damage of two types or inflictors" end
    elseif action.key == "ApplyModifier" then
      if value(body, "Target") ~= "TARGET" then return nil, "ApplyModifier other than the target" end
      local ok, why = add_control(value(body, "ModifierName"), tonumber(value(body, "Duration")))
      if not ok then return nil, why end
    elseif action.key == "RunScript" then
      local effect, why = run_script(item, body)
      if not effect then return nil, why end
      if effect.mana then return nil, "unit target RunScript restores mana" end
      for _, key in ipairs(effect.cooldown_items) do cooldown_items[#cooldown_items + 1] = key end
      for _, e in ipairs(effect.effects) do
        if e.kind == "damage" then
          if not add_damage(e.amount, e.damage_type, false) then return nil, "damage of two types or inflictors" end
        else
          local ok, why2 = add_control(e.modifier, e.duration)
          if not ok then return nil, why2 end
        end
      end
    elseif action.key == "TrackingProjectile" then
      -- E184: the OnProjectileHitUnit effects ride this flight (speed, Dodgeable) and land on arrival.
      if value(body, "Target") ~= "TARGET" then return nil, "TrackingProjectile other than the target" end
      local speed = tonumber(value(body, "MoveSpeed"))
      if not speed or speed <= 0 or projectile then return nil, "TrackingProjectile without one positive MoveSpeed" end
      projectile = { speed = speed, dodgeable = value(body, "Dodgeable") == "1" or nil }
      numbers[#numbers + 1] = speed
      -- E185: ProvidesVision gives the caster's team sight of VisionRadius around the flight.
      local sight = tonumber(value(body, "VisionRadius"))
      if value(body, "ProvidesVision") == "1" and sight and sight > 0 then
        projectile.vision = sight
        numbers[#numbers + 1] = sight
      end
    elseif spends_charge(action) then
      consumable = true
    elseif action.key ~= "FireSound" and action.key ~= "FireEffect" and action.key ~= "DeleteOnHit" then
      return nil, "unit target action " .. action.key
    end
  end
  if damage <= 0 then return nil, "enemy unit target without damage" end
  if damage_type ~= "magical" and damage_type ~= "physical" and damage_type ~= "pure" then return nil, "damage type " .. tostring(damage_type) end
  local out = { active = { kind = "unit_hex", damage = damage, damage_type = damage_type ~= "pure" and damage_type or nil,
      control = control, projectile = projectile, spell = spell or nil, slow = slow }, consumable = consumable or nil, applied = {}, cooldown_items = cooldown_items,
    notes = #notes > 0 and notes or nil,
    target = { shape = "unit", range = range, team = "enemy", requires_visible = true, out_of_range = "reject" } }
  return finish(item, out, numbers, groups)
end

-- A point item (E178): one script area around the target point (enemies, optional building guard
-- and stun), with the item's cast range. The source's FindUnitsInRadius TEAM_ENEMY includes neutrals.
local function point_nova(item, groups)
  local team = flags(item.target_team)
  if not team.DOTA_UNIT_TARGET_TEAM_ENEMY or not only(team, { DOTA_UNIT_TARGET_TEAM_ENEMY = true }) then
    return nil, "point target on " .. tostring(item.target_team)
  end
  local range = tonumber(item.cast_range)
  if not range or range <= 0 then return nil, "point target without cast range" end
  local area, consumable, cooldown_items = nil, false, {}
  local numbers = { range }
  for _, action in ipairs(item.active_kv.on_spell_start or {}) do
    if action.key == "RunScript" then
      local effect, why = run_script(item, action.children)
      if not effect then return nil, why end
      if effect.mana or #effect.effects > 0 then return nil, "a point script with other effects" end
      for _, key in ipairs(effect.cooldown_items) do cooldown_items[#cooldown_items + 1] = key end
      if effect.area then
        if area then return nil, "more than one area" end
        area = effect.area
      end
    elseif spends_charge(action) then
      consumable = true
    elseif action.key ~= "FireSound" and action.key ~= "FireEffect" then
      return nil, "point action " .. action.key
    end
  end
  if not area then return nil, "point effect is not an area the engine reads" end
  local damage = area.damage
  if type(damage) == "table" then
    damage = property_number(item, "%" .. damage.special)
    if not damage then return nil, "area damage special " .. tostring(area.damage.special) end
  end
  if damage <= 0 or area.radius <= 0 then return nil, "area without damage or radius" end
  numbers[#numbers + 1] = damage
  numbers[#numbers + 1] = area.radius
  if area.stun then numbers[#numbers + 1] = area.stun end
  local dot
  if area.modifier then
    -- The modifier each hit non-building gets: a periodic Damage whose attacker is the caster,
    -- so the caster's spell amplification scales its ticks (E188).
    local why
    dot, why = target_dot(item, area.modifier.name, area.modifier.duration)
    if not dot then return nil, why end
    if dot.damage_type ~= area.damage_type then return nil, area.modifier.name .. " damage type differs from the area's" end
    numbers[#numbers + 1] = dot.damage
    numbers[#numbers + 1] = dot.duration
  end
  if area.building_damage then numbers[#numbers + 1] = area.building_damage end
  if area.vision then numbers[#numbers + 1] = area.vision.radius; numbers[#numbers + 1] = area.vision.duration end
  local active = { kind = "point_nova", damage = damage, radius = area.radius,
    damage_type = area.damage_type ~= "pure" and area.damage_type or nil, skip_buildings = area.skip_buildings,
    control = area.stun and { kind = "stun", duration = area.stun } or nil,
    building_damage = area.building_damage, excludes_magic_immune = area.excludes_magic_immune or nil,
    dot = dot and { damage = dot.damage, interval = dot.interval, duration = dot.duration, spell = true } or nil,
    vision = area.vision }
  return finish(item, { active = active, consumable = consumable or nil, applied = {}, cooldown_items = cooldown_items,
    target = { shape = "point", range = range, team = "hostile", requires_visible = true, out_of_range = "reject" } }, numbers, groups)
end

local function unit_heal(item, groups)
  local team, types = flags(item.target_team), flags(item.target_types)
  if team.DOTA_UNIT_TARGET_TEAM_ENEMY and only(team, { DOTA_UNIT_TARGET_TEAM_ENEMY = true }) then return unit_hex(item, groups) end
  if not only(team, { DOTA_UNIT_TARGET_TEAM_FRIENDLY = true }) or not team.DOTA_UNIT_TARGET_TEAM_FRIENDLY then
    return nil, "targeted active (" .. tostring(item.target_team) .. ")"
  end
  if not types.DOTA_UNIT_TARGET_HERO or not only(types, { DOTA_UNIT_TARGET_HERO = true, DOTA_UNIT_TARGET_BASIC = true }) then
    return nil, "unit target types " .. tostring(item.target_types)
  end
  local range = tonumber(item.cast_range)
  if not range or range <= 0 then return nil, "unit target without cast range" end
  local amount, consumable = 0, false
  local numbers = { range }
  local cooldown_items = {}
  for _, action in ipairs(item.active_kv.on_spell_start) do
    local body = action.children
    if action.key == "Heal" then
      local v = tonumber(value(body, "HealAmount"))
      if value(body, "Target") ~= "TARGET" or not v or v <= 0 then return nil, "Heal other than the target" end
      amount = amount + v
      numbers[#numbers + 1] = v
    elseif spends_charge(action) then
      consumable = true
    elseif action.key == "RunScript" then
      local effect, why = run_script(item, body)
      if not effect then return nil, why end
      if effect.mana then return nil, "unit target RunScript restores mana" end
      if #effect.effects > 0 then return nil, "an ally target script applies effects" end
      for _, key in ipairs(effect.cooldown_items) do cooldown_items[#cooldown_items + 1] = key end
    elseif action.key ~= "FireSound" and action.key ~= "FireEffect" then
      return nil, "unit target action " .. action.key
    end
  end
  if amount <= 0 then return nil, "unit target effect is only in a script absent from the source" end
  return finish(item, { active = { kind = "unit_heal", amount = amount }, consumable = consumable or nil, applied = {},
    cooldown_items = cooldown_items,
    target = { shape = "unit", range = range, team = "ally", requires_visible = true, out_of_range = "reject" } }, numbers, groups)
end

-- `groups`: AbilitySharedCooldown group -> item keys of the catalog (item_source_compare.index).
-- A property value: a number, or `%name` naming one of the item's AbilitySpecial values.
property_number = function(item, raw)
  local ref = tostring(raw or ""):match("^%%([%w_]+)$")
  if ref then
    for _, s in ipairs(item.specials or {}) do if s.name == ref then return tonumber(s.value) end end
    return nil
  end
  return tonumber(raw)
end

-- The item's passive aura (E173): a Passive modifier with Aura keys naming a declared modifier.
-- Supported: friendly heroes (and basic units), the holder included, with stat Properties only.
-- Returns { aura = { name, radius, stats }, modifiers = { holder, aura } } or nil, reason; nil
-- and no reason when the item has no aura.
function M.aura(item)
  local blocks = (item.active_kv or {}).modifiers or {}
  local found
  for _, block in ipairs(blocks) do
    if block.children and find(block.children, "Aura") then
      if found then return nil, "more than one aura" end
      found = block
    end
  end
  if not found then return nil end
  local c = found.children
  local name = value(c, "Aura")
  local teams, types = flags(value(c, "Aura_Teams")), flags(value(c, "Aura_Types"))
  if not teams.DOTA_UNIT_TARGET_TEAM_FRIENDLY or not only(teams, { DOTA_UNIT_TARGET_TEAM_FRIENDLY = true }) then
    return nil, "aura on " .. tostring(value(c, "Aura_Teams"))
  end
  if not types.DOTA_UNIT_TARGET_HERO or not only(types, { DOTA_UNIT_TARGET_HERO = true, DOTA_UNIT_TARGET_BASIC = true }) then
    return nil, "aura types " .. tostring(value(c, "Aura_Types"))
  end
  if value(c, "Aura_ApplyToCaster") ~= "1" then return nil, "aura excludes its holder" end
  local radius = tonumber(value(c, "Aura_Radius"))
  if not radius or radius <= 0 then return nil, "aura radius" end
  local target = find(blocks, name)
  if not target or not target.children then return nil, "aura modifier " .. tostring(name) .. " is not declared" end
  local stats = {}
  for _, n in ipairs(target.children) do
    if n.key == "Properties" then
      for _, p in ipairs(n.children or {}) do
        local v = property_number(item, p.value)
        if not v then return nil, name .. " property " .. p.key .. " is not a number" end
        if v ~= 0 then
          local map = M.STATS[p.key]
          if not map then return nil, name .. " property " .. p.key:gsub("^MODIFIER_PROPERTY_", "") .. " is not supported" end
          stats[map[1]] = (stats[map[1]] or 0) + v * map[2]
        end
      end
    elseif not PRESENTATION[n.key] and n.key ~= "Passive" then
      return nil, name .. " uses " .. n.key
    end
  end
  if not next(stats) then return nil, name .. " has no stat" end
  return { aura = { name = name, radius = radius, stats = stats }, modifiers = { [found.key] = true, [name] = true } }
end

function M.translate(item, groups)
  local kv = item.active_kv
  if not item.has_active then return nil, "no active" end
  local behavior = flags(item.behavior)
  -- A toggle item acts on OnToggleOn (the source's scripts toggle it straight back off).
  if kv and not kv.on_spell_start and behavior.DOTA_ABILITY_BEHAVIOR_TOGGLE and kv.on_toggle_on then
    kv = { on_spell_start = kv.on_toggle_on, modifiers = kv.modifiers, on_projectile_hit = kv.on_projectile_hit }
    item = setmetatable({ active_kv = kv }, { __index = item })
  end
  if not kv or not kv.on_spell_start then return nil, "no OnSpellStart" end
  if behavior.DOTA_ABILITY_BEHAVIOR_UNIT_TARGET and not behavior.DOTA_ABILITY_BEHAVIOR_NO_TARGET then return unit_heal(item, groups) end
  if behavior.DOTA_ABILITY_BEHAVIOR_POINT and not behavior.DOTA_ABILITY_BEHAVIOR_NO_TARGET then return point_nova(item, groups) end
  if not behavior.DOTA_ABILITY_BEHAVIOR_NO_TARGET and not behavior.DOTA_ABILITY_BEHAVIOR_TOGGLE then return nil, "targeted active" end
  local consumable = false
  local cooldown_items = {}
  local blocks = kv.modifiers or {}
  local numbers, applied, mods, seen = {}, {}, {}, {}
  local active = { kind = "apply_modifiers" }
  local radius
  local function add(name, area, duration)
    if not name then return nil, "ApplyModifier without a name" end
    if seen[name] then return nil, "modifier " .. name .. " applied twice" end
    local m, why = modifier(blocks, name, numbers, tonumber(duration))
    if m == false then return true end
    if not m then return nil, why end
    m.area = area or nil
    seen[name] = true
    applied[name] = true
    mods[#mods + 1] = m
    return true
  end
  for _, action in ipairs(kv.on_spell_start) do
    local body = action.children
    if action.key == "ApplyModifier" then
      if value(body, "Target") ~= "CASTER" then return nil, "ApplyModifier target " .. tostring(value(body, "Target")) end
      local ok, why = add(value(body, "ModifierName"), false, value(body, "Duration"))
      if not ok then return nil, why end
    elseif action.key == "ActOnTargets" then
      local target = find(body, "Target")
      local t = target and target.children
      local teams, types = flags(value(t, "Teams")), flags(value(t, "Types"))
      if value(t, "Center") ~= "CASTER" or not only(teams, { DOTA_UNIT_TARGET_TEAM_FRIENDLY = true })
        or not teams.DOTA_UNIT_TARGET_TEAM_FRIENDLY or not types.DOTA_UNIT_TARGET_HERO
        or not only(types, { DOTA_UNIT_TARGET_HERO = true, DOTA_UNIT_TARGET_BASIC = true }) then
        return nil, "ActOnTargets other than friendly heroes around the caster"
      end
      local r = tonumber(value(t, "Radius"))
      if not r or r <= 0 or (radius and radius ~= r) then return nil, "ActOnTargets radius" end
      radius = r
      numbers[#numbers + 1] = r
      active.include_basic = types.DOTA_UNIT_TARGET_BASIC or nil
      local act = find(body, "Action")
      for _, a in ipairs(act and act.children or {}) do
        if a.key ~= "ApplyModifier" or value(a.children, "Target") ~= "TARGET" then
          return nil, "ActOnTargets action " .. a.key
        end
        local ok, why = add(value(a.children, "ModifierName"), true, value(a.children, "Duration"))
        if not ok then return nil, why end
      end
    elseif action.key == "Heal" then
      local amount = tonumber(value(body, "HealAmount"))
      if value(body, "Target") ~= "CASTER" or not amount or amount <= 0 then return nil, "Heal other than the caster" end
      active.heal = (active.heal or 0) + amount
      numbers[#numbers + 1] = amount
    elseif action.key == "RunScript" then
      local effect, why = run_script(item, body)
      if not effect then return nil, why end
      if #effect.effects > 0 then return nil, "a no-target item's script applies effects to a target" end
      if effect.magic_immunity then
        active.magic_immunity = effect.magic_immunity
        numbers[#numbers + 1] = effect.magic_immunity
      end
      if effect.mana then
        active.mana = (active.mana or 0) + effect.mana
        numbers[#numbers + 1] = effect.mana
      end
      for _, key in ipairs(effect.cooldown_items) do cooldown_items[#cooldown_items + 1] = key end
    elseif spends_charge(action) then
      consumable = true
    elseif action.key ~= "FireSound" and action.key ~= "FireEffect" then
      return nil, "action " .. action.key
    end
  end
  if #mods == 0 and not active.heal and not active.mana and not active.magic_immunity then return nil, "effect is only in a script absent from the source" end
  if radius then active.radius = radius end
  if #mods > 0 then active.modifiers = mods end
  return finish(item, { active = active, applied = applied, consumable = consumable or nil, cooldown_items = cooldown_items }, numbers, groups)
end

return M
