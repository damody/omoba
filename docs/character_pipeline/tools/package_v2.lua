-- Character package schema v2: the gameplay contract (plan README §4.2) on top of
-- the v1 art package. v1 packages keep validating unchanged; a package opts in
-- with `schema_version = 2` in character.lua.
--
-- validate(dir, loaded, opts) -> errors, warnings
--   loaded: name -> table for the Lua files already loaded by the caller
--   opts.catalog: hero source catalog table (optional, cross-checks provenance)
--   opts.requirements: D3 requirements table (optional, cross-checks statuses)
local M = {}

M.LUA_FILES = { "character.lua", "source.lua", "abilities.lua", "capabilities.lua", "presentation.lua" }
M.TEXT_FILES = { "assumptions.md", "acceptance.md" }
M.STATUSES = { supported = 1, ambiguous = 2, needs_framework = 3, deferred = 4 }
-- Loadout slots. Passives occupy a slot too (the MOBA loadout holds four
-- abilities); extra slots need the extra_action_slot capability.
M.ACTIONS = { Q = true, W = true, E = true, R = true, extra1 = true, extra2 = true, extra3 = true }
M.ABILITY_TYPES = { active = true, toggle = true, passive = true, ultimate = true }
M.TARGET_KINDS = { none = true, self = true, unit = true, point = true, direction = true }
M.TEAMS = { enemy = true, ally = true, both = true, self = true }
M.SHAPES = { none = true, circle = true, cone = true, line = true, ring = true }
M.DAMAGE_KINDS = { physical = true, magical = true, pure = true }
M.LIFECYCLE_KEYS = { "caster_death", "target_invalid", "vision_lost", "cancel", "expire" }
-- Closed effect vocabulary. Each kind lists its required parameter keys;
-- values may be a number or a per-rank array of max_level numbers.
M.EFFECTS = {
  damage = { "amount", "damage_kind" },
  heal = { "amount" },
  shield = { "amount", "duration" },
  stat_buff = { "stats", "duration" },
  stat_bonus_permanent = { "stats" },
  control = { "control", "duration" },
  slow = { "move_speed_pct", "duration" },
  damage_over_time = { "amount_per_second", "duration", "damage_kind" },
  damage_zone = { "radius", "amount_per_tick", "interval", "duration", "damage_kind" },
  summon = { "unit", "count", "lifetime" },
  dash = { "max_distance" },
  -- motion = linear (distance = cast range) or homing (needs lifetime).
  projectile = { "motion", "speed", "radius" },
  on_attack_bonus_by_target_class = { "classes", "damage_kind" },
  mana_drain = { "amount" },
  -- G-2 host-owned toggle orb: each attack spends mana_per_attack (auto-off
  -- when short); a landed hit adds amount and the optional slows (percent).
  attack_orb = { "mana_per_attack", "amount", "damage_kind" },
  -- G-4 host owner-cast hook: while learned, each cast of another non-ultimate
  -- ability shortens the hero's other non-ultimate cooldowns by seconds.
  cooldown_refund_on_cast = { "seconds" },
  -- A mechanic the vocabulary cannot express (line zone, forced-target summon,
  -- every N attacks, ...): stated in `note`, implemented by a Rust handler.
  custom = { "note" },
  -- G-4 host tether: settles damage/root by caster distance when it runs out.
  tether = { "duration", "max_distance" },
}
M.CONTROLS = { stun = true, root = true, silence = true }
M.TARGET_CLASSES = { hero = true, creep = true, building = true, summon = true }
-- StatKey names accepted in stat effects (subset of scripts/script-abi stat_keys).
M.STATS = {
  attack_damage_bonus = true, base_damage_outgoing_pct = true, attack_range_bonus = true,
  attack_speed_bonus = true, move_speed_bonus = true, move_speed_pct = true, armor_bonus = true,
  magic_resist_bonus = true, incoming_damage_pct = true, health_regen = true, mana_bonus = true,
  attack_stun_chance = true, attack_stun_duration = true, cooldown_reduction_pct = true,
  physical_constant_block = true, attack_speed_multiplier = true, invisibility_level = true, max_hp_bonus = true,
}

local function is_array(t)
  if type(t) ~= "table" then return false end
  local n = 0
  for k in pairs(t) do
    if math.type(k) ~= "integer" or k < 1 then return false end
    n = n + 1
  end
  return n == #t
end

-- Same authored envelope as scripts/gen_hero_registry.lua and omoba-content-model:
-- zero, or a finite f32 with magnitude in [1/1024, 1000000].
function M.authored_number(v)
  if type(v) ~= "number" or v ~= v or v == math.huge or v == -math.huge then return false end
  local f = string.unpack("f", string.pack("f", v))
  local a = math.abs(f)
  return f == 0 or (a >= 1 / 1024 and a <= 1000000)
end

local function snake(v) return type(v) == "string" and v:match("^[a-z][a-z0-9_]*$") ~= nil and not v:find("__", 1, true) end

function M.validate(dir, loaded, opts)
  opts = opts or {}
  local errors, warnings = {}, {}
  local function err(m) errors[#errors + 1] = "SchemaError: " .. m end
  local function warn(m) warnings[#warnings + 1] = "GameContractWarning: " .. m end

  for _, name in ipairs(M.TEXT_FILES) do
    local f = io.open(dir .. "/" .. name, "rb")
    if not f then err(name .. " is required for schema v2")
    else
      local t = f:read("a"); f:close()
      if not t:find("%S") then err(name .. " must not be empty") end
    end
  end
  for _, name in ipairs(M.LUA_FILES) do
    local t = loaded[name]
    if not t then err(name .. " is required for schema v2")
    elseif t.schema_version ~= 2 then err(name .. " must set schema_version = 2") end
  end
  local character, source, abilities_file = loaded["character.lua"], loaded["source.lua"], loaded["abilities.lua"]
  local caps_file, presentation = loaded["capabilities.lua"], loaded["presentation.lua"]
  if not (character and source and abilities_file and caps_file and presentation) then return errors, warnings end

  -- identity
  local hero = character.hero or {}
  if not snake(hero.id) then err("invalid hero id: " .. tostring(hero.id)) end
  for _, field in ipairs({ "display_name", "title", "role", "combat_read" }) do
    if type(hero[field]) ~= "string" or hero[field] == "" then err("character.lua hero." .. field .. " must be a non-empty string") end
  end
  local identity = character.identity or {}
  if identity.game_hero_id ~= hero.id then err("identity.game_hero_id must equal hero.id") end
  if type(identity.rust_module) ~= "string" or not identity.rust_module:match("^[A-Za-z_][A-Za-z0-9_]*$") then
    err("identity.rust_module must be a Rust identifier")
  end
  local integration = identity.integration
  if integration ~= "append" and integration ~= "update_existing" then
    err("identity.integration must be append or update_existing")
  end
  -- A hero UnitScript (owner hooks) is rust/hero_unit.rs exporting hero_unit_script().
  if identity.unit_script ~= nil then
    if identity.unit_script ~= true then err("identity.unit_script must be true when present") end
    if integration ~= "append" then err("identity.unit_script is only supported for appended heroes") end
    local f = io.open(dir .. "/rust/hero_unit.rs", "rb")
    if not f then err("identity.unit_script needs rust/hero_unit.rs") else f:close() end
  end
  local loadout = character.moba_loadout
  if loadout ~= nil then
    if type(loadout) ~= "table" or type(loadout.ranks) ~= "table" or (#loadout.ranks ~= 4 and #loadout.ranks ~= 5) then err("character.moba_loadout.ranks must list 4 or 5 slot ranks")
    else
      for _, r in ipairs(loadout.ranks) do
        if math.type(r) ~= "integer" or r < 0 or r > 255 then err("character.moba_loadout ranks must be integers in 0..255") end
      end
    end
    if loadout.skill_points ~= nil and (math.type(loadout.skill_points) ~= "integer" or loadout.skill_points < 0 or loadout.skill_points > 255) then
      err("character.moba_loadout.skill_points must be an integer in 0..255")
    end
  end
  local attack = character.attack or {}
  if attack.kind ~= "melee" and attack.kind ~= "ranged" then err("character.attack.kind must be melee or ranged") end
  if not M.authored_number(attack.range) or (attack.range or 0) <= 0 then err("character.attack.range must be a positive authored number") end

  -- provenance
  local prov = source.provenance or {}
  for _, field in ipairs({ "repository", "commit", "source_hero_id", "variant", "language" }) do
    if type(prov[field]) ~= "string" or prov[field] == "" then err("source.lua provenance." .. field .. " is required") end
  end
  if identity.source_hero_id ~= prov.source_hero_id then err("identity.source_hero_id must match source provenance") end
  local assumption_ids = {}
  for i, a in ipairs(source.assumptions or {}) do
    if not snake(a.id) then err("assumption #" .. i .. " needs a snake_case id")
    elseif assumption_ids[a.id] then err("duplicate assumption id " .. a.id) end
    if a.kind ~= "assumed" and a.kind ~= "adaptation" and a.kind ~= "kv_text_conflict" and a.kind ~= "not_applicable" then
      err("assumption " .. tostring(a.id) .. " kind must be assumed/adaptation/kv_text_conflict/not_applicable")
    end
    if type(a.text) ~= "string" or a.text == "" then err("assumption " .. tostring(a.id) .. " needs text") end
    assumption_ids[a.id or i] = true
  end
  local source_abilities = {}
  for _, s in ipairs(source.abilities or {}) do
    if type(s.source_ability_id) ~= "string" or type(s.raw_sha256) ~= "string" or #s.raw_sha256 ~= 64 then
      err("source.lua abilities entries need source_ability_id and 64-hex raw_sha256")
    else
      source_abilities[s.source_ability_id] = s
    end
  end
  -- Unit-slot entries that are not part of the kit (engine-generic passives such
  -- as magical_resistance, *_precache helpers): listed with a reason, never in
  -- `abilities`, so the "every source skill is mapped" rule stays meaningful.
  for i, n in ipairs(source.not_in_kit or {}) do
    if type(n.source_ability_id) ~= "string" or type(n.reason) ~= "string" or n.reason == "" then
      err("source.lua not_in_kit #" .. i .. " needs source_ability_id and reason")
    elseif source_abilities[n.source_ability_id] then
      err("source.lua lists " .. n.source_ability_id .. " both in abilities and not_in_kit")
    end
  end

  -- abilities
  local list = abilities_file.abilities
  if not is_array(list) or #list == 0 then err("abilities.lua abilities must be a non-empty array"); return errors, warnings end
  local seen_id, seen_action, mapped_source = {}, {}, {}
  local caps_by_id = {}
  for _, c in ipairs(caps_file.capabilities or {}) do caps_by_id[c.id] = c end
  local kit_status
  local function worse(a, b)
    if not a then return b end
    if not b then return a end
    return M.STATUSES[a] >= M.STATUSES[b] and a or b
  end
  for i, ab in ipairs(list) do
    local label = "ability " .. tostring(ab.id or ("#" .. i))
    if not snake(ab.id) then err(label .. " id must be snake_case")
    elseif seen_id[ab.id] then err("duplicate ability id " .. ab.id) end
    seen_id[ab.id or i] = true
    if not M.ACTIONS[ab.action] then err(label .. " action invalid: " .. tostring(ab.action))
    else
      if seen_action[ab.action] then err(label .. " action " .. ab.action .. " used twice") end
      seen_action[ab.action] = true
      if ab.action:match("^extra") then
        local listed = false
        for _, c in ipairs(ab.capabilities or {}) do if c == "extra_action_slot" then listed = true end end
        if not listed then err(label .. " uses " .. ab.action .. " but does not list capability extra_action_slot") end
      end
    end
    if not M.ABILITY_TYPES[ab.ability_type] then err(label .. " ability_type invalid") end
    if type(ab.name) ~= "string" or ab.name == "" then err(label .. " needs name") end
    if type(ab.description) ~= "string" or ab.description == "" then err(label .. " needs description (tooltip)") end
    if not source_abilities[ab.source_ability_id] then err(label .. " source_ability_id " .. tostring(ab.source_ability_id) .. " not in source.lua") end
    mapped_source[ab.source_ability_id or ""] = true
    local max_level = ab.max_level
    if math.type(max_level) ~= "integer" or max_level < 1 or max_level > 255 then err(label .. " max_level must be an integer in 1..255"); max_level = 1 end
    local function ranked(value, what, opts2)
      opts2 = opts2 or {}
      local values = type(value) == "table" and value or { value }
      if type(value) == "table" and #value ~= max_level then
        err(label .. " " .. what .. " needs " .. max_level .. " rank values, got " .. #value)
      end
      for _, v in ipairs(values) do
        if not M.authored_number(v) then err(label .. " " .. what .. " has a non-authored number: " .. tostring(v))
        elseif opts2.non_negative and v < 0 then err(label .. " " .. what .. " must not be negative") end
      end
    end
    -- resources / ranks
    local res = ab.resources or {}
    ranked(res.cooldown, "resources.cooldown", { non_negative = true })
    ranked(res.mana_cost, "resources.mana_cost", { non_negative = true })
    if res.charges then
      ranked(res.charges.max, "resources.charges.max", { non_negative = true })
      ranked(res.charges.recharge_seconds, "resources.charges.recharge_seconds", { non_negative = true })
    end
    local req_levels = ab.required_hero_level
    if req_levels ~= nil then
      if type(req_levels) ~= "table" or #req_levels ~= max_level then err(label .. " required_hero_level needs one value per rank")
      else
        local prev = 1
        for _, lv in ipairs(req_levels) do
          if math.type(lv) ~= "integer" or lv < prev or lv > 25 then err(label .. " required_hero_level must be monotonic integers in 1..25") end
          prev = math.type(lv) == "integer" and lv or prev
        end
      end
    end
    -- targeting
    local tg = ab.targeting or {}
    if not M.TARGET_KINDS[tg.kind] then err(label .. " targeting.kind invalid: " .. tostring(tg.kind)) end
    if not M.TEAMS[tg.team] then err(label .. " targeting.team invalid: " .. tostring(tg.team)) end
    if tg.range ~= nil then ranked(tg.range, "targeting.range", { non_negative = true }) end
    if (tg.kind == "unit" or tg.kind == "point" or tg.kind == "direction") and tg.range == nil then err(label .. " targeting.range required for " .. tg.kind) end
    local shape = tg.shape or { kind = "none" }
    if not M.SHAPES[shape.kind] then err(label .. " targeting.shape.kind invalid") end
    if shape.kind == "circle" then ranked(shape.radius, "targeting.shape.radius", { non_negative = true }) end
    if type(tg.visibility) ~= "string" then err(label .. " targeting.visibility policy required") end
    -- timing
    local tm = ab.timing or {}
    ranked(tm.cast_point or 0, "timing.cast_point", { non_negative = true })
    if tm.cooldown_starts ~= "on_cast" and tm.cooldown_starts ~= "on_end" then err(label .. " timing.cooldown_starts must be on_cast or on_end") end
    if tm.channel ~= nil then
      local ch = tm.channel
      local anchors = { self = "self", point = "point", unit = "unit" }
      if type(ch) ~= "table" or not anchors[ch.anchor] then err(label .. " timing.channel.anchor must be self, point or unit")
      else
        local want = ({ self = { none = true, self = true }, point = { point = true }, unit = { unit = true } })[ch.anchor]
        if not want[(ab.targeting or {}).kind] then err(label .. " timing.channel.anchor " .. ch.anchor .. " does not match targeting.kind") end
        if ch.interval == nil or ch.pulses == nil then err(label .. " timing.channel needs interval and pulses") end
        ranked(ch.interval, "timing.channel.interval", { non_negative = true })
        ranked(ch.pulses, "timing.channel.pulses", { non_negative = true })
        if ch.radius ~= nil then ranked(ch.radius, "timing.channel.radius", { non_negative = true }) end
        if ch.anchor ~= "unit" and ch.radius == nil and ((ab.targeting or {}).shape or {}).radius == nil then
          err(label .. " timing.channel with a self or point anchor needs a radius (channel.radius or targeting.shape.radius)")
        end
      end
    end
    -- effects
    if not is_array(ab.effects) then err(label .. " effects must be an array")
    else
      for j, e in ipairs(ab.effects) do
        local spec = M.EFFECTS[e.kind]
        local elabel = label .. " effect #" .. j
        if not spec then err(elabel .. " unknown kind " .. tostring(e.kind))
        else
          for _, key in ipairs(spec) do if e[key] == nil then err(elabel .. " (" .. e.kind .. ") missing " .. key) end end
          for key, v in pairs(e) do
            if type(v) == "number" or (is_array(v) and type(v[1]) == "number") then ranked(v, "effect " .. e.kind .. "." .. key) end
          end
          if e.damage_kind and not M.DAMAGE_KINDS[e.damage_kind] then err(elabel .. " damage_kind invalid") end
          if e.control and not M.CONTROLS[e.control] then err(elabel .. " control invalid") end
          if e.kind == "custom" and (ab.implementation or {}).kind ~= "rust_handler" then
            err(elabel .. " custom effects need a rust_handler implementation")
          end
          if e.kind == "attack_orb" then
            if ab.ability_type ~= "toggle" or (ab.targeting or {}).kind ~= "none" then
              err(elabel .. " attack_orb needs a toggle ability with targeting.kind none")
            end
            if #ab.effects ~= 1 then err(elabel .. " attack_orb is exclusive") end
            local slows = e.move_slow_pct ~= nil or e.attack_slow_pct ~= nil
            if slows ~= (e.slow_duration ~= nil) then err(elabel .. " attack_orb slows and slow_duration go together") end
          end
          if e.kind == "projectile" then
            if e.motion ~= "linear" and e.motion ~= "homing" then err(elabel .. " projectile motion must be linear or homing") end
            if e.motion == "homing" and e.lifetime == nil then err(elabel .. " homing projectile needs lifetime") end
          end
          if e.stats then
            for stat, v in pairs(e.stats) do
              if not M.STATS[stat] then err(elabel .. " unknown stat " .. tostring(stat)) end
              ranked(v, "effect stat " .. tostring(stat))
            end
          end
          if e.classes then
            for class, v in pairs(e.classes) do
              if not M.TARGET_CLASSES[class] then err(elabel .. " unknown target class " .. tostring(class)) end
              ranked(v, "effect class " .. tostring(class))
            end
          end
        end
      end
    end
    -- lifecycle
    local lc = ab.lifecycle or {}
    for _, key in ipairs(M.LIFECYCLE_KEYS) do
      if type(lc[key]) ~= "string" or lc[key] == "" then err(label .. " lifecycle." .. key .. " must be stated") end
    end
    -- implementation
    local impl = ab.implementation or {}
    if impl.kind ~= "generic" and impl.kind ~= "rust_handler" then err(label .. " implementation.kind must be generic or rust_handler") end
    if impl.kind == "rust_handler" and (type(impl.module) ~= "string" or not impl.module:match("^[A-Za-z_][A-Za-z0-9_]*$")) then
      err(label .. " rust_handler needs a Rust module name")
    end
    -- bot usage and verification
    if ab.ability_type ~= "passive" then
      local bu = ab.bot_usage
      if type(bu) ~= "table" or type(bu.target) ~= "string" or type(bu.when) ~= "string" then err(label .. " bot_usage.target/when required") end
    end
    local ver = ab.verification or {}
    if not is_array(ver.scenarios) or #ver.scenarios == 0 then err(label .. " verification.scenarios required")
    else
      for _, s in ipairs(ver.scenarios) do
        if type(s.name) ~= "string" or type(s.expect) ~= "string" then err(label .. " verification scenario needs name and expect") end
      end
    end
    for _, ref in ipairs(ab.assumptions or {}) do
      if not assumption_ids[ref] then err(label .. " references unknown assumption " .. tostring(ref)) end
    end
    -- capability status
    if not M.STATUSES[ab.status] then err(label .. " status invalid: " .. tostring(ab.status)) end
    local derived
    for _, cap_id in ipairs(ab.capabilities or {}) do
      local c = caps_by_id[cap_id]
      if not c then err(label .. " capability " .. tostring(cap_id) .. " not declared in capabilities.lua")
      else derived = worse(derived, c.status) end
    end
    if not ab.capabilities or #ab.capabilities == 0 then err(label .. " must list capabilities") end
    if derived and ab.status ~= derived then err(label .. " status " .. tostring(ab.status) .. " but capabilities derive " .. derived) end
    kit_status = worse(kit_status, ab.status)
    -- MOBA birth rank (omoba-content-model validate_moba_loadout): a slot may
    -- start learned only if its first rank needs hero level 1.
    if ab.auto_learn ~= nil and ab.auto_learn ~= true then err(label .. " auto_learn must be true when present") end
    local slot = ({ Q = 1, W = 2, E = 3, R = 4, extra1 = 5 })[ab.action]
    if slot then
      local rank = loadout and loadout.ranks and loadout.ranks[slot]
      if rank == nil then rank = ((ab.required_hero_level or {})[1] or 1) <= 1 and 1 or 0 end
      if math.type(rank) == "integer" and (rank > max_level or (rank > 0 and ((ab.required_hero_level or {})[rank] or 1) > 1)) then
        err(label .. " birth rank " .. rank .. " is not learnable at hero level 1")
      end
    end
  end
  for id in pairs(source_abilities) do
    if not mapped_source[id] then err("source ability " .. id .. " is not mapped to any package ability") end
  end
  for _, c in ipairs(caps_file.capabilities or {}) do
    if not M.STATUSES[c.status] then err("capability " .. tostring(c.id) .. " status invalid") end
  end
  if caps_file.kit_status ~= kit_status then err("capabilities.lua kit_status must be " .. tostring(kit_status) .. " (derived), got " .. tostring(caps_file.kit_status)) end
  if caps_file.catalog_ready and kit_status ~= "supported" then err("catalog_ready requires every ability supported (kit is " .. tostring(kit_status) .. ")") end
  if caps_file.catalog_ready then
    local acc = io.open(dir .. "/acceptance.md", "rb")
    local text = acc and acc:read("a") or ""
    if acc then acc:close() end
    -- The status is the first `status:` line after the title, not any mention in prose.
    local status = text:gsub("\r\n", "\n"):match("\nstatus:%s*([%w_]+)%s*\n")
    if status ~= "accepted" then err("catalog_ready requires acceptance.md to contain 'status: accepted'") end
  end

  -- optional summons.lua: summon templates the kit spawns (appended to templates/summons.lua)
  local summon_ids = {}
  local summons_file = io.open(dir .. "/summons.lua", "rb")
  if summons_file then
    summons_file:close()
    local chunk, load_err = loadfile(dir .. "/summons.lua", "t", { pairs = pairs, ipairs = ipairs, table = table, math = math, string = string })
    local ok, fn = pcall(chunk or error, load_err)
    local data = ok and type(fn) == "function" and fn({}) or nil
    if type(data) ~= "table" or data.schema_version ~= 2 or not is_array(data.summons) then
      err("summons.lua must return function(ctx) with schema_version = 2 and a summons array")
    else
      for i, s in ipairs(data.summons) do
        local label = "summon " .. tostring(s.id or ("#" .. i))
        if not snake(s.id) then err(label .. " id must be snake_case") end
        if summon_ids[s.id or ""] then err("duplicate summon id " .. tostring(s.id)) end
        summon_ids[s.id or ""] = true
        if type(s.display_name) ~= "string" or s.display_name == "" then err(label .. " needs display_name") end
        for _, key in ipairs({ "hp", "damage", "duration", "move_speed", "attack_range", "attack_interval", "armor",
          "magic_resistance", "collision_radius", "vision_radius", "follow_start_distance", "follow_stop_distance", "damage_per_owner_level" }) do
          if not M.authored_number(s[key]) or s[key] < 0 then err(label .. " " .. key .. " must be a non-negative authored number") end
        end
        if (s.hp or 0) <= 0 or (s.duration or 0) <= 0 or (s.attack_interval or 0) <= 0 then err(label .. " hp, duration and attack_interval must be positive") end
        if s.immobile ~= nil and type(s.immobile) ~= "boolean" then err(label .. " immobile must be boolean") end
        if s.invulnerable ~= nil and type(s.invulnerable) ~= "boolean" then err(label .. " invulnerable must be boolean") end
        if s.immobile and (s.move_speed ~= 0 or s.follow_start_distance ~= 0 or s.follow_stop_distance ~= 0) then
          err(label .. " is immobile but has move speed or follow distances")
        end
        if s.owner_death ~= "persist" and s.owner_death ~= "expire" then err(label .. " owner_death must be persist or expire") end
        if s.target_priority ~= "nearest" and s.target_priority ~= "owner_focus" then err(label .. " target_priority must be nearest or owner_focus") end
      end
    end
  end
  for _, ab in ipairs(list) do
    for _, e in ipairs(ab.effects or {}) do
      if e.kind == "summon" and type(e.unit) == "string" and next(summon_ids) and not summon_ids[e.unit] and e.unit_source ~= "existing" then
        err("ability " .. tostring(ab.id) .. " summons " .. e.unit .. ", which summons.lua does not declare (set unit_source = \"existing\" for a pre-existing template)")
      end
    end
  end

  -- presentation covers every ability
  local pres = presentation.abilities or {}
  for _, ab in ipairs(list) do
    local p = pres[ab.id]
    if type(p) ~= "table" then err("presentation.lua missing ability " .. tostring(ab.id))
    else
      if type(p.preview) ~= "string" then err("presentation " .. ab.id .. " needs preview shape") end
      if type(p.animation) ~= "string" then err("presentation " .. ab.id .. " needs animation state") end
      if type(p.cues) ~= "table" then err("presentation " .. ab.id .. " needs cues") end
      -- Optional skill VFX layers (Niagara content paths), carried to ue.vfx by hero_author.
      if p.vfx ~= nil then
        if type(p.vfx) ~= "table" then err("presentation " .. ab.id .. ".vfx must be a table")
        else
          local layers = { cast = true, area = true, impact = true, aura = true }
          for key, value in pairs(p.vfx) do
            if layers[key] then
              if type(value) ~= "string" or value:sub(1, 1) ~= "/" then err("presentation " .. ab.id .. ".vfx." .. tostring(key) .. " must be a content path like /Game/AIVFX/<hero>/NS_Name") end
            elseif key == "aura_buff" or key == "none" then
              if type(value) ~= "string" or value == "" then err("presentation " .. ab.id .. ".vfx." .. key .. " must be a non-empty string") end
            elseif key == "area_reference_radius" then
              if type(value) ~= "number" or value <= 0 then err("presentation " .. ab.id .. ".vfx.area_reference_radius must be positive") end
            else
              err("presentation " .. ab.id .. ".vfx has unknown key " .. tostring(key))
            end
          end
        end
      end
    end
  end
  for _, clip in ipairs({ "idle", "move", "attack", "cast", "death" }) do
    if type((presentation.animation_states or {})[clip]) ~= "string" then err("presentation.animation_states." .. clip .. " required") end
  end

  -- optional cross-checks
  if opts.catalog then
    local cat = opts.catalog
    if cat.source and cat.source.commit ~= prov.commit then err("source commit " .. tostring(prov.commit) .. " differs from catalog " .. tostring(cat.source.commit)) end
    local by = {}
    for _, a in ipairs(cat.abilities or {}) do by[a.id] = a end
    for id, s in pairs(source_abilities) do
      local a = by[id]
      if not a then err("source ability " .. id .. " not in catalog")
      else
        if a.raw_sha256 ~= s.raw_sha256 then err("source ability " .. id .. " raw_sha256 differs from catalog (stale package)") end
        if a.variant ~= prov.variant then err("source ability " .. id .. " is variant " .. a.variant .. ", package is " .. tostring(prov.variant)) end
        if a.source_hero_id ~= prov.source_hero_id then err("source ability " .. id .. " belongs to " .. tostring(a.source_hero_id)) end
      end
    end
  end
  if opts.requirements then
    local reg = {}
    for _, c in ipairs(opts.requirements.capabilities or {}) do reg[c.id] = c end
    for _, c in ipairs(caps_file.capabilities or {}) do
      local r = reg[c.id]
      if not r then warn("capability " .. c.id .. " is not in the D3 registry")
      elseif r.status ~= c.status and not c.override_reason then
        err("capability " .. c.id .. " is " .. tostring(c.status) .. " here but " .. r.status .. " in the D3 registry (add override_reason)")
      end
    end
  end
  return errors, warnings
end

return M
