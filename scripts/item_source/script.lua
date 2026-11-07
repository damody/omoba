-- Read-only analysis of a source item script function (history_nobu items/Addon_Items/*.lua, E169).
-- The script is text: it is never loaded or executed. `analyze` finds the function's body and
-- lists the engine calls it makes, so the item translator can prove what a RunScript does
-- instead of assuming it:
--   * presentation_only: every call is particles, sounds, timers or plain reads;
--   * cooldown_items: the "every listed item in the caster's inventory starts this item's
--     cooldown" loop (`item:GetName() == "item_x"` ... `item:StartCooldown(ability:GetCooldown(-1))`);
--   * calls: every other call, by name (the translator refuses a function that makes any).
local M = {}

-- Calls with no game effect (visual/audio, reading state, scheduling more of the same).
M.PRESENTATION = {
  CreateParticle = true, SetParticleControl = true, SetParticleControlEnt = true, SetParticleControlForward = true,
  ReleaseParticleIndex = true, DestroyParticle = true, EmitSound = true, EmitSoundOn = true, EmitSoundOnLocationWithCaster = true,
  EmitSoundOnLocationForAllies = true, StopSound = true, StopSoundOn = true, CreateTimer = true,
  GetAbsOrigin = true, GetLevel = true, GetCaster = true, GetPlayerID = true, GetPlayerOwnerID = true, GetForwardVector = true,
  Normalized = true, GetCursorPosition = true, Vector = true, IsNull = true, IsAlive = true,
}
-- Functions read in full and judged presentation although they touch the engine (path#function -> why).
M.REVIEWED_PRESENTATION = {
  ["items/Addon_Items/item_soul_adder.lua#sound"] = "plays keys.sound on the caster/unit/target; a point cast "
    .. "spawns an invisible npc_dummy_unit (modifier_kill 3 s) only to emit the sound there",
}
-- Functions read in full whose effect is a whole engine capability (path#function -> effect).
M.REVIEWED_EFFECTS = {
  ["items/Addon_Items/item_perceive_wine.lua#Shock"] = { kind = "magic_immunity", seconds = "Time",
    reason = "removes every modifier an enemy applied (keeping truesight), applies modifier_perceive_wine (MAGIC_IMMUNE) "
      .. "for keys.Time, strong-dispels, re-applies it if removed before Time, then toggles the item off: "
      .. "magic_immunity.rs grant (E179)" },
  -- E188: `requires` are verbatim fragments of the function (spaces ignored); the catalog drops
  -- the review when one is missing, so a changed source script is never translated from it.
  ["items/Addon_Items/item_spell_book.lua#Shock"] = { kind = "target_modifier", modifier = "modifier_spell_book_enemy",
    requires = { 'keys.ability:ApplyDataDrivenModifier(caster, target,"modifier_spell_book_enemy", nil)' },
    reason = "applies modifier_spell_book_enemy to an enemy target and plays a particle; the ally branch "
      .. "(modifier_spell_book_friendly, never declared) is unreachable: AbilityUnitTargetTeam is ENEMY" },
  ["items/Addon_Items/item_rockfall_book.lua#Shock"] = { kind = "point_area",
    area = { radius = 300, damage = { special = "damage" }, damage_type = "magical", stun = 0.1, skip_buildings = true,
      vision = { radius = 400, duration = 3 } },
    requires = { "local SEARCH_RADIUS = 300", "DOTA_UNIT_TARGET_FLAG_MAGIC_IMMUNE_ENEMIES", "AddFOWViewer(caster:GetTeamNumber(), point, 400, 3.0, false)",
      'if (not(it:IsBuilding())) and not it:HasModifier("modifier_C08T_bleeding") then',
      'AMHC:Damage(caster,it,ability:GetLevelSpecialValueFor("damage", 0 ),AMHC:DamageType( "DAMAGE_TYPE_MAGICAL" ) )',
      'ability:ApplyDataDrivenModifier(caster, it,"modifier_stunned", {duration = 0.1})' },
    reason = "every enemy (magic immune included) within 300 of the point but buildings and C08T bleeders takes "
      .. "the `damage` special as inflictor-less magical damage (AMHC:Damage) and a 0.1 s stun applied directly "
      .. "(through magic immunity); the caster's team sees 400 around the point for 3 s (AddFOWViewer, terrain "
      .. "blocks it here). modifier_C08T_bleeding belongs to hero C08, which is not in the game, so the guard never matches" },
  ["items/Addon_Items/item_ignite_book.lua#Shock"] = { kind = "point_area",
    area = { radius = 800, damage = 550, damage_type = "magical", building_damage = 200, excludes_magic_immune = true,
      modifier = { name = "modifier_ignite_book", duration = 5 } },
    requires = { "local SEARCH_RADIUS = 800", "DOTA_UNIT_TARGET_ALL,\n          0,", "if not target:IsBuilding() then",
      'ability:ApplyDataDrivenModifier(caster, target, "modifier_ignite_book", {duration = 5})',
      'AMHC:Damage(caster,target,550,AMHC:DamageType( "DAMAGE_TYPE_MAGICAL" ) )',
      'AMHC:Damage(caster,target,200,AMHC:DamageType( "DAMAGE_TYPE_MAGICAL" ) )', "if IsValidEntity(it) then" },
    reason = "every enemy within 800 of the point found without MAGIC_IMMUNE_ENEMIES (so magic-immune ones are skipped): "
      .. "a non-building gets modifier_ignite_book for 5 s and 550 inflictor-less magical damage (AMHC:Damage), a "
      .. "building only 200; the flame particle is guarded by IsValidEntity(it) on the undefined `it`, so it never plays" },
}

-- Whether every `requires` fragment of a reviewed effect is in `body` (whitespace ignored).
function M.review_matches(reviewed, body)
  local squeeze = function(s) return (s:gsub("%s+", "")) end
  local code = squeeze(body)
  for _, fragment in ipairs(reviewed.requires or {}) do
    if not code:find(squeeze(fragment), 1, true) then return false, fragment end
  end
  return true
end
-- The shared-cooldown loop's own calls.
local COOLDOWN = { GetItemInSlot = true, GetName = true, StartCooldown = true, GetCooldown = true }

-- Body of `function <name>(` up to its matching top-level `end` (block keywords counted).
function M.function_body(text, name)
  local s, e = text:find("\nfunction%s+" .. name:gsub("%p", "%%%0") .. "%s*%(")
  if not s and text:find("^function%s+" .. name:gsub("%p", "%%%0") .. "%s*%(") then s, e = 1, text:find("%(") end
  if not s then return nil end
  local depth, pos = 1, text:find("\n", e) or #text
  local line = select(2, text:sub(1, s):gsub("\n", "")) + 1
  while pos <= #text do
    local ws, we, word = text:find("%f[%w_]([%a_]+)%f[^%w_]", pos)
    if not ws then return nil end
    -- Skip comments and strings on the way (a keyword inside them is not code).
    local cs = text:find("%-%-", pos, true)
    local qs = text:find('["\']', pos)
    if cs and cs < ws and (not qs or cs < qs) then
      pos = (text:find("\n", cs) or #text) + 1
    elseif qs and qs < ws then
      local quote = text:sub(qs, qs)
      local close = qs + 1
      while true do
        local q = text:find(quote, close, true)
        if not q then return nil end
        if text:sub(q - 1, q - 1) ~= "\\" then close = q; break end
        close = q + 1
      end
      pos = close + 1
    else
      if word == "function" or word == "if" or word == "do" then depth = depth + 1
      elseif word == "end" then
        depth = depth - 1
        if depth == 0 then return text:sub(s, we), line end
      end
      pos = we + 1
    end
  end
  return nil
end

function M.analyze(text, name)
  local body, line = M.function_body(text, name)
  if not body then return { found = false } end
  local code = body:gsub("%-%-[^\n]*", "")
  local calls, other = {}, {}
  for method in code:gmatch("[%w_%]%)]+[:.]([%a_][%w_]*)%s*%(") do calls[method] = true end
  for fn in code:gmatch("%f[%w_.:]([%a_][%w_]*)%s*%(") do
    if fn ~= "function" and fn ~= "if" and fn ~= "for" and fn ~= "while" and fn ~= "and" and fn ~= "or" and fn ~= "not"
      and fn ~= "ipairs" and fn ~= "pairs" and fn ~= "tostring" and fn ~= "tonumber" and fn ~= "print" then calls[fn] = true end
  end
  local cooldown_items = {}
  if code:find("StartCooldown%s*%(%s*ability:GetCooldown%s*%(%s*%-1%s*%)%s*%)") then
    local seen = {}
    for item in code:gmatch('GetName%(%)%s*==%s*"item_([%w_]+)"') do
      if not item:match("^recipe_") and not seen[item] then seen[item] = true; cooldown_items[#cooldown_items + 1] = item end
    end
  end
  -- Target effects (E176): `ability:ApplyDataDrivenModifier(caster, target, "<modifier>", nil|{duration=x})`
  -- and `AMHC:Damage(caster, target, <n>, AMHC:DamageType("DAMAGE_TYPE_X"))`, optionally inside
  -- `if not target:IsBuilding() then`. They count as explained only when every such call matches.
  local effects, guarded = {}, code:find("if%s+not%s+target:IsBuilding%(%)%s+then") ~= nil
  for modifier, opts in code:gmatch('ApplyDataDrivenModifier%(%s*caster%s*,%s*target%s*,%s*"([%w_]+)"%s*,%s*([^%)]*)%)') do
    effects[#effects + 1] = { kind = "apply_modifier", modifier = modifier,
      duration = tonumber(opts:match("duration%s*=%s*([%d%.]+)")), non_building = guarded or nil }
  end
  local applied = #effects
  for amount, kind in code:gmatch('AMHC:Damage%(%s*caster%s*,%s*target%s*,%s*([%d%.]+)%s*,%s*AMHC:DamageType%(%s*"DAMAGE_TYPE_(%u+)"%s*%)%s*%)') do
    effects[#effects + 1] = { kind = "damage", amount = tonumber(amount), damage_type = kind:lower(), non_building = guarded or nil }
  end
  local _, apply_calls = code:gsub("ApplyDataDrivenModifier%s*%(", "")
  local _, damage_calls = code:gsub("AMHC:Damage%s*%(", "")
  local explained = apply_calls == applied and damage_calls == #effects - applied
  -- Point area (E178): `local p = keys.target_points[1]`, `FindUnitsInRadius(caster:GetTeamNumber(), p, nil,
  -- R, DOTA_UNIT_TARGET_TEAM_ENEMY, ...)`, then `for _,u in pairs(units) do if not u:IsBuilding() then
  -- AMHC:Damage(caster,u,D,AMHC:DamageType("DAMAGE_TYPE_X")) [ability:ApplyDataDrivenModifier(caster,u,"m",{duration=d})]`.
  -- R and D are literals, locals assigned a literal, or `ability:GetLevelSpecialValueFor("name", 0)`.
  local area
  local point = code:match("local%s+([%w_]+)%s*=%s*keys%.target_points%[1%]")
  if point then
    local args = code:match("FindUnitsInRadius%s*%(%s*caster:GetTeamNumber%(%)%s*,%s*" .. point .. "%s*,%s*nil%s*,([^%)]*)%)")
    local units = code:match("local%s+([%w_]+)%s*=%s*FindUnitsInRadius")
    local function value_of(expr)
      expr = (expr or ""):match("^%s*(.-)%s*$")
      if tonumber(expr) then return tonumber(expr) end
      local special = expr:match('^ability:GetLevelSpecialValueFor%(%s*"([%w_]+)"%s*,%s*0%s*%)$')
      if special then return { special = special } end
      local literal = code:match("local%s+" .. expr:gsub("%p", "%%%0") .. "%s*=%s*([%d%.]+)")
      return tonumber(literal)
    end
    local radius_expr, team
    if args then radius_expr, team = args:match("^%s*([^,]+),%s*([%w_]+)") end
    local loop = units and code:match("for%s+_%s*,%s*([%w_]+)%s+in%s+pairs%(%s*" .. units .. "%s*%)%s+do")
    if loop and team == "DOTA_UNIT_TARGET_TEAM_ENEMY" then
      local u = loop:gsub("%p", "%%%0")
      local amount, kind = code:match('AMHC:Damage%(%s*caster%s*,%s*' .. u .. '%s*,%s*([^,]+),%s*AMHC:DamageType%(%s*"DAMAGE_TYPE_(%u+)"%s*%)%s*%)')
      local stun = code:match('ApplyDataDrivenModifier%(%s*caster%s*,%s*' .. u .. '%s*,%s*"modifier_stunned"%s*,%s*{%s*duration%s*=%s*([%d%.]+)%s*}%s*%)')
      area = { radius = value_of(radius_expr), damage = value_of(amount), damage_type = kind and kind:lower(),
        stun = tonumber(stun), skip_buildings = code:find("not%s*%(?%s*" .. u .. ":IsBuilding%(%)") ~= nil or nil }
      if not area.radius or not area.damage or not area.damage_type then area = nil end
    end
  end
  if area then
    -- The area loop's own calls: one damage call and at most the one stun (no target effects).
    explained = damage_calls == 1 and apply_calls == (area.stun and 1 or 0)
    effects = {}
  end
  local EFFECT = { ApplyDataDrivenModifier = true, Damage = true, DamageType = true, IsBuilding = true }
  calls[name] = nil
  for c in pairs(calls) do
    local area_call = area and explained and (c == "FindUnitsInRadius" or c == "GetTeamNumber" or c == "GetLevelSpecialValueFor"
      or c == "DestroyTreesAroundPoint" or c == "pairs")
    if not M.PRESENTATION[c] and not (COOLDOWN[c] and #cooldown_items > 0) and not (explained and EFFECT[c]) and not area_call then other[#other + 1] = c end
  end
  table.sort(other)
  if not explained then effects = {} end
  if area and not explained then area = nil end
  return { found = true, line = line, presentation_only = #other == 0 and #effects == 0 and not area, cooldown_items = cooldown_items,
    effects = effects, area = area, calls = other }
end

return M
