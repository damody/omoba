-- Bot cast policies from package schema v3 (M6 prerequisite): when and at what a server bot or the
-- UE autopilot casts an ability, so no hero needs bot code of its own. An ability (or item active)
-- may write `bot = { kind = ..., <params> }` or `bot = false` (never cast); without it the policy is
-- derived from the document's target, cast, cost and behavior.
--
-- Kinds and parameters mirror omoba-core moba_match/bots/abilities.rs BotAbilityIntent (serde
-- snake_case, `kind` tag): a `*_key` parameter is written `$value` in the document and exported as
-- the value name (an extras key of the compiled ability). Kinds marked `proposed` are not in the
-- executor yet (docs/plans/2026-10-28-history-nobu-full-port/m6-bot-intents.md); a bot skips them.
local M = {}

-- kind -> { params = { name = "per_mille" | "radius" | "count" | "value" | "status" }, proposed? }
-- ("status": a status runtime id, the one the generated code reads to tell a toggle on)
M.KINDS = {
  enemy_unit = { params = {} },
  enemy_hero = { params = {} },
  enemy_point = { params = { radius_key = "value", min_targets = "count" } },
  enemy_cone = { params = { min_targets = "count" } },
  enemy_line = { params = { min_targets = "count" } },
  approach_enemy_point = { params = { min_distance = "radius", reach_key = "value" } },
  -- E473 (line B): the nearest standing tree in cast range; enemy_radius > 0 also wants a
  -- revealed enemy hero within it
  nearest_tree = { params = { enemy_radius = "radius0" } },
  self_heal = { params = { below_hp_per_mille = "per_mille" } },
  self_area = { params = { radius_key = "value", min_targets = "count" } },
  self_engage = { params = { threat_radius = "radius" } },
  self_channel_recovery = { params = { below_hp_per_mille = "per_mille", threat_radius = "radius" } },
  ally_heal = { params = { below_hp_per_mille = "per_mille" } },
  ally_heal_or_cleanse = { params = { below_hp_per_mille = "per_mille" } },
  self_recovery = { params = { below_hp_per_mille = "per_mille", below_mana_per_mille = "per_mille", restore_key = "value" } },
  self_mana_regeneration = { params = { below_hp_per_mille = "per_mille", below_mana_per_mille = "per_mille",
    rate_key = "value", duration_key = "value" } },
  -- in the executor since E260 (summon_near_enemy {threat_radius}) and E263 (toggle_engage)
  summon_near_enemy = { params = { threat_radius = "radius" } },
  toggle_engage = { params = { threat_radius = "radius", on_status = "status" } },
  -- E281: a leap aimed at a point with no cast range (it lands at most `reach_key` away)
  leap_enemy_point = { params = { radius_key = "value", reach_key = "value", min_targets = "count" } },
  -- E387 (line B): an area placed anywhere (no cast range) at the revealed enemy hero with the
  -- most enemy heroes around it; a cast on another allied hero near revealed enemies
  global_enemy_point = { params = { radius_key = "value", min_targets = "count" } },
  -- E605 (line B): what the cast's own guard asks for (on_phase_start cancel_cast, M.facts guard_*)
  -- enemies around the caster carrying a public status; bodies around the caster; a point at an
  -- allied non-hero unit nearest the revealed enemy heroes
  self_area_status = { params = { radius_key = "value", min_targets = "count", status = "status" } },
  self_area_corpses = { params = { radius_key = "value", min_targets = "count" } },
  ally_anchor_point = { params = { radius_key = "value", types = "types" } },
  ally_engage = { params = { threat_radius = "radius" } },
  -- proposed (m6-bot-intents.md): not cast until the executor has them
  escape = { params = { below_hp_per_mille = "per_mille", threat_radius = "radius" }, proposed = true },
  channel_enemy_area = { params = { radius_key = "value", min_targets = "count" }, proposed = true },
}
M.DEFAULTS = { heal_below = 600, threat = 900, engage = 900, min_targets = 1, mana_below = 400, escape_below = 300, approach = 300 }

local function sorted_keys(t)
  local ks = {}
  for k in pairs(t) do ks[#ks + 1] = k end
  table.sort(ks, function(a, b) return tostring(a) < tostring(b) end)
  return ks
end

local function value_name(expr)
  if type(expr) ~= "string" then return nil end
  return expr:match("^%s*%$([%a_][%w_]*)%s*$")
end

-- `$a`, or `$a` times or divided by a constant (a number, or a value equal at every rank:
-- c08_kite_chain's `$hook_width / $half`): the value's name and the factor, else nil
local function scaled_value(expr, def)
  local plain = value_name(expr)
  if plain then return plain, 1 end
  if type(expr) ~= "string" then return nil end
  local a, op, b = expr:match("^%s*%$([%a_][%w_]*)%s*([%*/])%s*(%S+)%s*$")
  if not a then return nil end
  local k = tonumber(b)
  local bn = not k and b:match("^%$([%a_][%w_]*)$")
  local v = bn and def.values and def.values[bn]
  if type(v) == "table" and #v > 0 then
    k = v[1]
    for _, n in ipairs(v) do if n ~= k then return nil end end
  end
  if type(k) ~= "number" or k == 0 then return nil end
  return a, op == "*" and k or 1 / k
end

-- no cast range: none written, or a value 0 at every rank (E366: range 0 is anywhere on the map)
local function unranged(target, def)
  if target.range == nil then return true end
  local name = value_name(target.range)
  local v = name and def.values and def.values[name]
  if type(v) ~= "table" or #v == 0 then return false end
  for _, n in ipairs(v) do if n ~= 0 then return false end end
  return true
end

-- a move toward the enemy; with no cast range its reach is the move's own cap (reach_key, E438)
local function approach(target, def, f)
  local reach = unranged(target, def) and (f.reach or f.clamp) or nil
  -- no cast range and no cap: a move anywhere on the map. It lands where the cast's own check
  -- counts units around the point (c19_surprise_raid: allies within $ally_radius), aimed at the
  -- revealed enemy heroes (the executor's global_enemy_point)
  -- the cast is cancelled unless allied units of `types` are near the point (c19_surprise_raid)
  if f.guard_anchor then return { kind = "ally_anchor_point", radius_key = f.guard_anchor.radius, types = f.guard_anchor.types } end
  if unranged(target, def) and not reach and f.point_radius then
    return { kind = "global_enemy_point", radius_key = f.point_radius, min_targets = M.DEFAULTS.min_targets }
  end
  return { kind = "approach_enemy_point", min_distance = M.DEFAULTS.approach, reach_key = reach }
end

-- every action and selector of a program tree, into the document's routines it calls (once each)
-- `args`: the routine's parameters bound to the calling action's arguments (a routine is walked
-- once, through the first call that reaches it)
local function walk(t, on_action, on_selector, doc, seen, owner, args)
  if type(t) ~= "table" then return end
  if type(t.op) == "string" then on_action(t, owner, args) end
  if type(t.shape) == "string" then on_selector(t) end
  if t.op == "call" and type(t.routine) == "string" and doc and doc.routines and doc.routines[t.routine] then
    seen = seen or {}
    if not seen[t.routine] then
      seen[t.routine] = true
      local r = doc.routines[t.routine]
      local bound = {}
      for k, v in pairs(t.args or {}) do
        bound[k] = (type(v) == "string" and args and args[v:match("^%s*([%a_][%w_]*)%s*$") or ""]) or v
      end
      walk(r.actions, on_action, on_selector, doc, seen, r.owner or owner, bound)
    end
  end
  for _, k in ipairs(sorted_keys(t)) do walk(t[k], on_action, on_selector, doc, seen, owner, args) end
end

-- facts about one castable's behavior
function M.facts(def, castable, doc)
  local f = { heal_self = false, heal_target = false, cleanse = false, mana = nil, damage = false,
    summon = false, move = false, selectors = {}, lines = {}, enemy_status = false }
  local function action(a, owner, args)
    -- a routine parameter (a bare name) stands for the value its caller passed
    local function arg_value(v)
      local name = type(v) == "string" and v:match("^%s*([%a_][%w_]*)%s*$")
      return (name and args and args[name]) or v
    end
    if a.op == "heal" then
      if a.to == "caster" or a.to == "self" then f.heal_self = true else f.heal_target = true end
    elseif a.op == "restore_mana" and (a.to == "caster" or a.to == "self") then f.mana = value_name(a.amount)
    elseif a.op == "remove_status" then f.cleanse = true
    elseif a.op == "damage" then f.damage = true
    elseif a.op == "create_unit" or a.op == "create_illusion" then
      f.summon = true
      if a.op == "create_illusion" then f.damage = f.damage or false end
      if type(a.template) == "string" then f.creates = f.creates or {}; f.creates[#f.creates + 1] = a.template end
    elseif a.op == "dash" or a.op == "leap" or a.op == "blink" or a.op == "teleport" then
      f.move = true
      -- how far the move goes at most (b13_shukuchi's teleport max_distance): the reach of a
      -- point skill with no cast range (line B E438)
      f.reach = f.reach or value_name(a.max_distance)
    elseif a.op == "apply_status" and a.to ~= "caster" and a.to ~= "self" then f.enemy_status = true
    elseif a.op == "let" and type(a.value) == "string" then
      -- the clamp idiom: a destination at most `$x` from the caster is the move's reach
      f.clamp = f.clamp or a.value:match("distance%(%s*caster%s*,%s*point%s*%)%s*>%s*%$([%a_][%w_]*)")
    -- a linear projectile sweeps a line as wide as its largest radius on both sides; a path one
    -- (a13_chain_sickle's two-point hook) sweeps its legs the same way
    elseif a.op == "projectile" and (a.kind == "linear" or a.kind == "path") then
      f.lines[#f.lines + 1] = { value = arg_value(a.end_radius or a.radius), owner = owner }
    end
  end
  -- the cast's own selectors decide the aim; status triggers (on-hit cleaves, auras) only add facts
  -- a routine names this entry's own value `$<id>.<name>` (b09_old_earth_rupture's bursts): the
  -- same value as `$<name>`, the extra the executor reads
  local function selector(s)
    local owner, name
    if type(s.radius) == "string" then owner, name = s.radius:match("^%s*%$([%a_][%w_]*)%.([%a_][%w_]*)%s*$") end
    if owner and owner == def.id then
      local own = {}
      for k, v in pairs(s) do own[k] = v end
      own.radius = "$" .. name
      s = own
    end
    f.selectors[#f.selectors + 1] = s
  end
  walk(castable.events, action, selector, doc)
  for _, s in ipairs(def.statuses or {}) do walk(s.triggers, action, function() end, doc) end
  -- a radius the cast checks around its target point (count_in_radius(point, $x, ...))
  local function scan(t)
    if f.point_radius then return end
    if type(t) == "string" then f.point_radius = t:match("count_in_radius%(%s*point%s*,%s*%$([%a_][%w_]*)")
    elseif type(t) == "table" then for _, k in ipairs(sorted_keys(t)) do scan(t[k]) end end
  end
  scan(castable.events)
  -- the cast's guard: on_phase_start cancels it unless its own scan finds something (a status on an
  -- enemy, a body, allied units near the point); the bot asks for the same (E605)
  local phase = (castable.events or {}).on_phase_start
  local cancels = false
  walk(phase, function(a) if a.op == "cancel_cast" then cancels = true end end, function() end, doc)
  if cancels then
    walk(phase, function(a)
      local cond = type(a.cond) == "string" and a.cond or ""
      for r, ty in cond:gmatch("count_in_radius%(%s*point%s*,%s*%$([%a_][%w_]*)%s*,%s*team_of%(%s*caster%s*%)%s*,%s*([%a_]+)%s*%)") do
        f.guard_anchor = f.guard_anchor or { radius = r, types = {} }
        if f.guard_anchor.radius == r then f.guard_anchor.types[#f.guard_anchor.types + 1] = ty end
      end
    end, function(s)
      local st = type(s.where) == "string" and s.where:match("^%s*has_status%(%s*it%s*,%s*([%a_][%w_]*)%s*%)%s*$")
      if st and (s.team == nil or s.team == "enemy") then f.guard_status = f.guard_status or { status = st, radius = value_name(s.radius) } end
      for _, fl in ipairs(s.flags or {}) do
        if fl == "include_dead" and (s.from == nil or s.from == "caster" or s.from == "self") then f.guard_dead = f.guard_dead or value_name(s.radius) end
      end
    end, doc)
  end
  return f
end

-- a status' runtime id (the host's buff id), searched in the whole document
local function status_runtime_id(doc, def, id)
  local function find(list)
    for _, st in ipairs(list or {}) do if st.id == id then return st.runtime_id or st.id end end
  end
  local rid = find(def.statuses)
  for _, ab in ipairs(doc and doc.abilities or {}) do rid = rid or find(ab.statuses) end
  if rid and rid:find("{", 1, true) then return nil end
  return rid
end

-- The summon a cast needs (line B E648 caster_summon): its requires.condition `exists(state.V) and alive(state.V)`,
-- optionally `and edge_distance(caster, state.V) <= R`, or an on_cast that starts with
-- `if exists(state.V) and alive(state.V)` and refunds otherwise. V is a state the document sets to
-- a unit it creates (`create_unit ... as = u`, `set_state V = u`): the summon's template, and R
-- (the smallest rank value of `$R`, or a number) as `within`.
local function created_template(doc, var)
  local found
  local function walk(list)
    local made = {}
    for _, a in ipairs(type(list) == "table" and list or {}) do
      if type(a) == "table" then
        if a.op == "create_unit" and type(a.as) == "string" and type(a.template) == "string"
          and (a.owner == nil or a.owner == "caster") then made[a.as] = a.template end
        if a.op == "set_state" and a.var == var and type(a.value) == "string" and made[a.value] then
          if found and found ~= made[a.value] then found = false elseif found == nil then found = made[a.value] end
        end
        for _, k in ipairs({ "actions", "otherwise" }) do walk(a[k]) end
      end
    end
  end
  for _, list in ipairs({ doc and doc.abilities or {}, doc and doc.items or {} }) do
    for _, ab in ipairs(list) do
      for _, ev in pairs(type(ab.events) == "table" and ab.events or {}) do walk(ev.actions or ev) end
    end
  end
  return found or nil
end
local function needed_summon(def, castable, doc)
  local function smallest(expr)
    if type(expr) == "number" then return expr end
    local name = value_name(expr)
    local v = name and def.values and def.values[name]
    if type(v) == "number" then return v end
    if type(v) ~= "table" or #v == 0 then return nil end
    local m
    for _, n in ipairs(v) do if type(n) ~= "number" then return nil end; m = m and math.min(m, n) or n end
    return m
  end
  local var, within, ok
  local cond = castable and castable.requires and castable.requires.condition
  if type(cond) == "string" then
    local v1, v2, rest = cond:match("^exists%(state%.([%w_]+)%) and alive%(state%.([%w_]+)%)(.*)$")
    if v1 and v1 == v2 then
      if rest == "" then var, ok = v1, true
      else
        local v3, r = rest:match("^ and edge_distance%(caster, state%.([%w_]+)%) <= (%S+)$")
        if v3 == v1 then
          within = smallest(r)
          var, ok = v1, within ~= nil
        end
      end
    end
  end
  local first = not ok and castable and type(castable.events) == "table" and castable.events.on_cast
  first = type(first) == "table" and (first.actions or first)
  for _, a in ipairs(type(first) == "table" and first or {}) do
    if a.op == "if" and type(a.cond) == "string" then
      local v1, v2 = a.cond:match("^exists%(state%.([%w_]+)%) and alive%(state%.([%w_]+)%)$")
      local refunds = false
      for _, o in ipairs(type(a.otherwise) == "table" and a.otherwise or {}) do if o.op == "refund" then refunds = true end end
      if v1 and v1 == v2 and refunds then var, ok = v1, true end
      break
    elseif a.op ~= "cue" then break end
  end
  if not ok then return nil end
  local template = created_template(doc, var)
  if not template or not template:match("^[%w_]+$") then return nil end
  -- a ranked template spawns `<id>_r<rank>` (v3_rust create_unit): no single id to name
  for _, t in ipairs(doc and doc.unit_templates or {}) do
    if t.id == template and (t.max_level or 1) > 1 then return nil end
  end
  local within_int = within and math.tointeger(within)
  if within and (not within_int or within_int <= 0 or within_int > 10000) then return nil end
  return { template = template, within = within_int }
end

-- Whether a `swap_abilities` of the document puts ability `id` into a slot (`to`): it is cast from
-- that slot while it sits there, so the bot needs its own policy (derived from its own document).
function M.swapped_in(doc, id)
  if type(doc) ~= "table" then return false end
  local found = false
  local function scan(t)
    if found or type(t) ~= "table" then return end
    if t.op == "swap_abilities" and t.to == id then found = true; return end
    for _, k in ipairs(sorted_keys(t)) do scan(t[k]) end
  end
  scan(doc.abilities)
  scan(doc.routines)
  scan(doc.statuses)
  return found
end

-- the derived policy (a table with `kind`, or { kind = "none", reason })
function M.derive(def, kind_of_entry, doc)
  local castable = kind_of_entry == "item" and def.active or def
  if not castable or (kind_of_entry == "ability" and def.kind == "passive") then return { kind = "none", reason = "passive" } end
  if kind_of_entry == "ability" and (def.slot == "hidden" or def.slot == "sub") and not M.swapped_in(doc, def.id) then
    return { kind = "none", reason = "sub-ability (cast through its parent)" }
  end
  local D = M.DEFAULTS
  local target = castable.target or {}
  local cast = castable.cast or {}
  local f = M.facts(def, castable, doc)
  local toggle = kind_of_entry == "ability" and def.kind == "toggle"
  if toggle then
    -- on: the first status on_toggle_on puts on the caster under a fixed runtime id
    local on_status
    local by_id = {}
    for _, st in ipairs(def.statuses or {}) do by_id[st.id] = st end
    for _, a in ipairs((castable.events or {}).on_toggle_on or {}) do
      local st = a.op == "apply_status" and (a.to == "caster" or a.to == "self") and by_id[a.status]
      local rid = st and (st.runtime_id or st.id)
      if rid and not rid:find("{caster}", 1, true) then on_status = rid; break end
    end
    return { kind = "toggle_engage", threat_radius = D.threat, on_status = on_status }
  end
  if cast.channel_time and target.type ~= "unit" then
    -- E281: a channel aimed at a point hits the area there (the bot waits for the channel)
    local channel_area = (target.type == "point") and value_name(target.aoe_radius)
    if channel_area and not f.heal_self then
      return { kind = unranged(target, def) and "global_enemy_point" or "enemy_point", radius_key = channel_area, min_targets = D.min_targets }
    end
    if f.heal_self then return { kind = "self_channel_recovery", below_hp_per_mille = D.heal_below, threat_radius = D.threat } end
    -- any other channel aimed at a point (b31_old_quiet_as_forest's arrow barrage) is aimed like
    -- a point skill below: self_area and self_engage take no target
    if target.type ~= "point" then
      for _, s in ipairs(f.selectors) do
        local r = value_name(s.radius)
        -- a channel around the caster: the executor's self_area takes a channeled cast (line B)
        if s.shape == "circle" and r then return { kind = "self_area", radius_key = r, min_targets = D.min_targets } end
      end
      return { kind = "self_engage", threat_radius = D.engage, note = "channel without a recovery or area rule" }
    end
  end
  local t = target.type or "none"
  local team = target.team or "enemy"
  local hero_only = type(target.types) == "table" and #target.types == 1 and target.types[1] == "hero"
  -- a tree target: the nearest standing tree in cast range (line B E473); a summon or a hit
  -- waits for a revealed enemy hero near it, anything else (vision) casts on any tree
  if t == "tree" then
    -- a summon that fights (a template with an attack) also waits for an enemy; a dummy (vision) does not
    local fights = f.damage
    for _, a in ipairs(f.creates or {}) do
      for _, tpl in ipairs(doc and doc.unit_templates or {}) do
        if tpl.id == a and type(tpl.attack) == "table" and tpl.attack.kind ~= nil and tpl.attack.kind ~= "none" then fights = true end
      end
    end
    return { kind = "nearest_tree", enemy_radius = fights and D.engage or 0 }
  end
  if t == "unit" then
    if team == "enemy" or team == "both" then
      if team == "both" and (f.heal_target or f.cleanse) and not f.damage then
        return { kind = f.cleanse and "ally_heal_or_cleanse" or "ally_heal", below_hp_per_mille = D.heal_below }
      end
      return { kind = hero_only and "enemy_hero" or "enemy_unit" }
    end
    if f.heal_target or f.heal_self then
      return { kind = f.cleanse and "ally_heal_or_cleanse" or "ally_heal", below_hp_per_mille = D.heal_below }
    end
    return { kind = "ally_engage", threat_radius = D.engage }
  end
  -- unit_or_point: the row's target is a point (hero_author/v3.lua), so the bot aims at one
  if t == "point" or t == "direction" or t == "vector" or t == "unit_or_point" then
    if f.move and not f.damage then return approach(target, def, f) end
    local area = value_name(target.aoe_radius)
    -- no cast range (range 0, E366): anywhere on the map
    if area then return { kind = unranged(target, def) and "global_enemy_point" or "enemy_point", radius_key = area, min_targets = D.min_targets } end
    for _, s in ipairs(f.selectors) do
      if s.shape == "cone" then return { kind = "enemy_cone", min_targets = D.min_targets } end
      -- line_width / line_width_scale: the value the line's width is (scaled), for the row's
      -- targeting shape (hero_author/v3.lua shape_width); not an executor parameter
      if s.shape == "line" or s.shape == "segment" or s.shape == "rect" then
        local w, k = scaled_value(s.width, def)
        return { kind = "enemy_line", min_targets = D.min_targets, line_width = w, line_width_scale = w and k or nil }
      end
    end
    for _, s in ipairs(f.selectors) do
      local r = value_name(s.radius)
      -- a splash around what a linear projectile hits (from `it`) is not where the cast aims:
      -- the projectile's line is (b31_old_quiet_as_forest)
      local splash = s.from == "it" and #f.lines > 0
      if s.shape == "circle" and r and not splash then return { kind = unranged(target, def) and "global_enemy_point" or "enemy_point", radius_key = r, min_targets = D.min_targets } end
    end
    if f.summon then return { kind = "summon_near_enemy", threat_radius = D.threat } end
    if f.move then return approach(target, def, f) end
    -- a point skill whose only reach is a linear projectile: the line it sweeps
    -- line_width_owner: the entry whose values the width is (a routine another entry owns)
    for _, l in ipairs(f.lines) do
      if value_name(l.value) then
        return { kind = "enemy_line", min_targets = D.min_targets, line_width = value_name(l.value), line_width_scale = 2,
          line_width_owner = l.owner ~= def.id and l.owner or nil }
      end
    end
    return { kind = "enemy_line", min_targets = D.min_targets, note = "point skill without an area: aimed at the nearest enemy" }
  end
  -- no target
  if f.mana and not f.damage then
    return { kind = "self_recovery", below_hp_per_mille = 0, below_mana_per_mille = D.mana_below, restore_key = f.mana }
  end
  if f.heal_self and not f.damage then return { kind = "self_heal", below_hp_per_mille = D.heal_below } end
  -- a cast that needs a body near the caster (b23_undead_pact): bodies, not enemies
  if f.guard_dead then return { kind = "self_area_corpses", radius_key = f.guard_dead, min_targets = D.min_targets } end
  -- a summon without a target point (C02 phantom_move, B18 earth_claw): summon_near_enemy takes
  -- target none too (line B E419)
  if f.summon then return { kind = "summon_near_enemy", threat_radius = D.threat } end
  for _, s in ipairs(f.selectors) do
    local r = value_name(s.radius)
    local around_caster = s.from == nil or s.from == "caster" or s.from == "self"
    if s.shape == "circle" and r and around_caster and (s.team == nil or s.team == "enemy") and (f.damage or f.enemy_status) then
      -- a cast that needs an enemy carrying a status (c25_falling_sakura: a bleeding one)
      local rid = f.guard_status and status_runtime_id(doc, def, f.guard_status.status)
      if rid then return { kind = "self_area_status", radius_key = f.guard_status.radius or r, min_targets = D.min_targets, status = rid } end
      return { kind = "self_area", radius_key = r, min_targets = D.min_targets }
    end
  end
  return { kind = "self_engage", threat_radius = D.engage }
end

-- validation of an authored `bot` field: problems as strings; find_value(name) -> def|nil
function M.check(bot, find_value)
  local problems = {}
  if bot == false then return problems end
  if type(bot) ~= "table" then return { "bot 必須是 table 或 false" } end
  local spec = M.KINDS[bot.kind]
  if not spec then return { "bot.kind `" .. tostring(bot.kind) .. "` 不在詞彙表（v3_bot.lua KINDS）" } end
  for k, v in pairs(bot) do
    if k ~= "kind" and k ~= "notes" then
      local p = spec.params[k]
      if not p then problems[#problems + 1] = "bot." .. tostring(k) .. " 不是 " .. bot.kind .. " 的參數"
      elseif p == "value" then
        local n = value_name(v)
        if not n then problems[#problems + 1] = "bot." .. k .. " 必須寫成 `$值名稱`"
        elseif not find_value(n) then problems[#problems + 1] = "bot." .. k .. "：值 `" .. n .. "` 不存在" end
      elseif p == "status" then
        if type(v) ~= "string" or v == "" then problems[#problems + 1] = "bot." .. k .. " 必須是狀態 id" end
      elseif p == "types" then
        local ok = type(v) == "table" and #v > 0
        for _, ty in ipairs(type(v) == "table" and v or {}) do
          if ty ~= "creep" and ty ~= "summon" and ty ~= "building" then ok = false end
        end
        if not ok then problems[#problems + 1] = "bot." .. k .. " 必須是 creep／summon／building 的清單" end
      elseif math.type(v) ~= "integer" then problems[#problems + 1] = "bot." .. k .. " 必須是整數"
      elseif p == "per_mille" and (v < 0 or v > 1000) then problems[#problems + 1] = "bot." .. k .. " 必須在 0～1000"
      elseif p == "radius" and (v < 1 or v > 10000) then problems[#problems + 1] = "bot." .. k .. " 必須在 1～10000"
      elseif p == "radius0" and (v < 0 or v > 10000) then problems[#problems + 1] = "bot." .. k .. " 必須在 0～10000"
      elseif p == "count" and (v < 1 or v > 128) then problems[#problems + 1] = "bot." .. k .. " 必須在 1～128" end
    end
  end
  for k in pairs(spec.params) do
    if bot[k] == nil and not (bot.kind == "self_recovery" and k == "below_hp_per_mille")
      and not (bot.kind == "approach_enemy_point" and k == "reach_key") then
      problems[#problems + 1] = "bot 缺少 " .. bot.kind .. " 的參數 " .. k
    end
  end
  return problems
end

-- the catalog row: Rust BotAbilityIntent shape plus `source` (authored | derived) and `proposed`
function M.export(def, kind_of_entry, doc)
  local castable = kind_of_entry == "item" and def.active or def
  local authored = castable and castable.bot
  if kind_of_entry == "item" and authored == nil then authored = def.bot end
  if authored == false then return { kind = "none", reason = "authored: never cast", source = "authored" } end
  local policy, source
  if type(authored) == "table" then
    policy, source = {}, "authored"
    for k, v in pairs(authored) do policy[k] = v end
  else
    policy, source = M.derive(def, kind_of_entry, doc), "derived"
  end
  local spec = M.KINDS[policy.kind]
  for k, v in pairs(policy) do
    if spec and spec.params[k] == "value" then policy[k] = value_name(v) or v end
  end
  policy.notes = nil
  policy.source = source
  -- the cast's own requires on the caster (E619): cast only while the caster has (or lacks) it
  local req = castable and castable.requires or {}
  for key, field in pairs({ has_status = "caster_status", not_status = "caster_no_status" }) do
    local rid = type(req[key]) == "string" and status_runtime_id(doc, def, req[key])
    if rid and rid:match("^[%w_:]+$") and #rid <= 96 and policy.kind ~= "none" then policy[field] = rid end
  end
  -- the summon it needs (line B's caster_summon)
  if policy.kind ~= "none" then policy.caster_summon = needed_summon(def, castable, doc) end
  if spec and spec.proposed then policy.proposed = true end
  return policy
end

return M
