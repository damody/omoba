-- Test scenarios from package schema v3 documents (M5/M6 work line A): for every ability and
-- item active of a document, every rank, the cases a port must pass, derived from the document
-- alone. Expectations name values (`$damage`, `$mana_cost - 1`) and quote the document's own
-- expressions; they never copy numbers, so a value change moves the test with it.
--
--   tools\lua\lua.exe docs\character_pipeline\tools\v3_scenarios.lua [--out DIR] [--stdout] [ID ...]
--
-- ID: a hero source id or package name, or an item document file name; none = every document
-- (v3_sources.lua). Writes DIR/<id>.json (default target/v3-scenarios).
--
-- Output: { schema = "v3-scenarios/1", document, id, kind, scenarios = [ scenario ] }
--   scenario = { id, entry, rank, case, given = [fact], when = [step], expect = [outcome], refs, rule? , quirk? }
--   case: normal | trigger | toggle_on | toggle_off | insufficient_mana | insufficient_health |
--         no_charges | cooldown | target_invalid | magic_immune | dispel | death | visibility | quirk | hook
--   hook (§8): every hook an entry uses ({ op = "hook" }, hook_value(...), or the whole entry as
--   `hook = "<name>"` with kind = "ability", whose behavior scenarios are generated as usual) gives its declared `cases` (given/when/expect in this same
--   vocabulary) per rank, or, when it has none, one `hook_behavior` case carrying its text, so the
--   AI-written hook is compared with what the document says it does.
--   fact / step / outcome: tables with a `kind` and named fields (expressions are document strings);
--   `effects` outcomes list the actions the case runs: { op, path, phase, conds, args, status? }.
--   refs: every value the scenario reads (`name` or `owner.name`), so a test can resolve them per rank.
--   rule: the v3.md section the expectation comes from when the document does not say it itself.
local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
local M = {}
M.SCHEMA = "v3-scenarios/1"

local X -- v3_expr (from the validator)
local function expr_mod()
  if not X then
    local prev = rawget(_G, "PACKAGE_V3_AS_MODULE")
    PACKAGE_V3_AS_MODULE = true
    X = dofile(tools_dir .. "/package_v3.lua").expr
    PACKAGE_V3_AS_MODULE = prev
  end
  return X
end

local function sorted_keys(t)
  local ks = {}
  for k in pairs(t) do ks[#ks + 1] = k end
  table.sort(ks, function(a, b) return tostring(a) < tostring(b) end)
  return ks
end

local function is_program(v)
  return type(v) == "table" and #v > 0 and type(v[1]) == "table" and type(v[1].op) == "string"
end

-- value references of one expression string ("$x", "$owner.x")
local function refs_of(s, into)
  if type(s) ~= "string" then return into end
  for q, n in s:gmatch("%$([%a_][%w_]*)%.([%a_][%w_]*)") do into[q .. "." .. n] = true end
  for n, rest in s:gmatch("%$([%a_][%w_]*)(%.?)") do
    if rest == "" then into[n] = true end
  end
  return into
end

local function collect_refs(t, into)
  into = into or {}
  if type(t) == "string" then return refs_of(t, into) end
  if type(t) ~= "table" then return into end
  -- quirk text and refs are prose, not expressions
  for _, k in ipairs(sorted_keys(t)) do if k ~= "text" and k ~= "ref" then collect_refs(t[k], into) end end
  return into
end

local function ref_list(set)
  local out = sorted_keys(set)
  return out
end

-- every action of a program tree, in order, with its path and enclosing conditions
local function walk_program(prog, path, phase, conds, out)
  for i, a in ipairs(prog or {}) do
    local here = path .. "[" .. i .. "]"
    local args = {}
    local nested = {}
    for _, k in ipairs(sorted_keys(a)) do
      local v = a[k]
      if k ~= "op" then
        if is_program(v) then nested[#nested + 1] = k
        elseif type(v) ~= "table" or not is_program(v) then args[k] = v end
      end
    end
    out[#out + 1] = { op = a.op, path = here, phase = phase, conds = #conds > 0 and conds or nil, args = next(args) and args or nil }
    for _, k in ipairs(nested) do
      local c = {}
      for j, x in ipairs(conds) do c[j] = x end
      if a.op == "if" then
        c[#c + 1] = k == "otherwise" and ("not (" .. tostring(a.cond) .. ")") or tostring(a.cond)
      elseif a.op == "chance" then
        c[#c + 1] = (k == "otherwise" and "chance failed: " or "chance: ") .. tostring(a.p or a.chance)
      elseif k ~= "actions" then
        c[#c + 1] = a.op .. "." .. k
      end
      walk_program(a[k], here .. "." .. k, phase, c, out)
    end
  end
  return out
end

local function effects_of(def)
  local out = {}
  for _, ev in ipairs(sorted_keys(def.events or {})) do
    local e = def.events[ev]
    walk_program(e.actions or e, "events." .. ev, ev, {}, out)
  end
  return out
end

local function trigger_effects(trig, path, phase)
  local out = {}
  local conds = {}
  if trig.filter then conds[1] = trig.filter end
  walk_program(trig.actions or trig, path, phase, conds, out)
  return out
end

-- statuses the entry owns, by id; and the ones its actions apply (with the target expression)
local function status_index(doc, def)
  local by = {}
  for _, s in ipairs(doc.statuses or {}) do by[s.id] = s end
  for _, s in ipairs(def.statuses or {}) do by[s.id] = s end
  return by
end

local function applied_statuses(effects, by)
  local out, seen = {}, {}
  for _, e in ipairs(effects) do
    if e.op == "apply_status" and e.args and e.args.status and not seen[e.args.status] then
      seen[e.args.status] = true
      out[#out + 1] = { status = by[e.args.status], id = e.args.status, to = e.args.to, path = e.path, conds = e.conds }
    end
  end
  return out
end

-- The BuffStore key a status is held under: its runtime_id, else the executor's default (the
-- status id, `<id>:{caster}` per source; m3-ir-design §7, E227).
local function runtime_key(s)
  if s.runtime_id then return s.runtime_id end
  if s.id then return s.per_source and (s.id .. ":{caster}") or s.id end
end

local function status_summary(s)
  if not s then return nil end
  return { id = s.id, runtime_id = runtime_key(s), runtime_id_default = not s.runtime_id or nil, duration = s.duration, stacking = s.stacking, dispel = s.dispel, visibility = s.visibility,
    reveal = s.reveal, scope = s.scope, states = s.states, stats = s.stats, remove_on = s.remove_on, tags = s.tags }
end

local function has_flag(def, f)
  for _, x in ipairs(def.flags or {}) do if x == f then return true end end
  return false
end

local function has_damage(effects, type_)
  for _, e in ipairs(effects) do
    if e.op == "damage" and (not type_ or (e.args and e.args.type == type_)) then return true end
  end
  return false
end

-- one entry (ability or item) -> scenarios
local function entry_scenarios(doc, def, entry_kind, out)
  local castable = entry_kind == "ability" and def or def.active
  local passive = def.passive
  local max_level = def.max_level or 1
  local by_status = status_index(doc, def)
  local base_effects = castable and effects_of(castable) or {}
  local kind = entry_kind == "ability" and def.kind or (castable and "active" or "passive")
  local target = castable and castable.target or {}
  local cost = castable and castable.cost or {}
  local cast = castable and castable.cast or {}
  local function push(rank, case, s)
    s.id = def.id .. "/r" .. rank .. "/" .. case .. (s.suffix and ("/" .. s.suffix) or "")
    s.suffix = nil
    s.entry, s.rank, s.case = def.id, rank, case
    s.refs = ref_list(collect_refs({ s.given, s.when, s.expect }))
    out[#out + 1] = s
  end
  local cast_step = { kind = "cast", entry = def.id, target = target.type or "none", range = target.range }
  local function cost_outcomes(spent)
    local o = {}
    if cost.mana then o[#o + 1] = { kind = "mana_spent", amount = spent and cost.mana or "0" } end
    if cost.health then o[#o + 1] = { kind = "health_spent", amount = spent and cost.health or "0" } end
    if cost.charges then o[#o + 1] = { kind = "charge_spent", count = spent and "1" or "0" } end
    if cost.cooldown then
      o[#o + 1] = spent and { kind = "cooldown_started", length = cost.cooldown, starts = cost.cooldown_start or "on_cast" }
        or { kind = "cooldown_not_started" }
    end
    return o
  end
  local applied = applied_statuses(base_effects, by_status)

  for rank = 1, max_level do
    local learned = { kind = "learned", entry = def.id, rank = rank }
    -- facts after `learned`; nil entries (a fact that does not apply) are skipped
    local function given(extra)
      local g = { learned }
      local n = 0
      for k in pairs(extra or {}) do if k > n then n = k end end
      for k = 1, n do if extra[k] then g[#g + 1] = extra[k] end end
      return g
    end
    if castable and kind ~= "passive" then
      local valid_target = target.type == "unit" and { kind = "target_unit", team = target.team or "enemy", types = target.types or { "hero" } } or nil
      -- normal
      local exp = { { kind = "cast_accepted" } }
      for _, o in ipairs(cost_outcomes(true)) do exp[#exp + 1] = o end
      if #base_effects > 0 then exp[#exp + 1] = { kind = "effects", effects = base_effects } end
      for _, a in ipairs(applied) do
        exp[#exp + 1] = { kind = "status_applied", status = a.id, to = a.to, conds = a.conds, summary = status_summary(a.status) }
      end
      if kind == "toggle" then
        push(rank, "toggle_on", { given = given({ valid_target }), when = { { kind = "toggle", on = true } }, expect = exp })
        push(rank, "toggle_off", { given = given({ { kind = "toggled_on", entry = def.id } }), when = { { kind = "toggle", on = false } },
          expect = { { kind = "toggled_off" }, { kind = "effects", effects = effects_of({ events = { on_toggle_off = castable.events and castable.events.on_toggle_off } }) } } })
      elseif target.type == "unit" and target.team == "both" then
        -- both teams: one case per side; the effects' conds say which branch runs
        for _, side in ipairs({ "enemy", "ally" }) do
          push(rank, "normal", { suffix = side, given = given({ { kind = "target_unit", team = side, types = target.types or { "hero" } },
            cost.mana and { kind = "mana_at_least", amount = cost.mana } or nil }), when = { cast_step }, expect = exp })
        end
      else
        push(rank, "normal", { given = given({ valid_target, cost.mana and { kind = "mana_at_least", amount = cost.mana } or nil }),
          when = { cast_step }, expect = exp })
      end
      -- A summon over its per-owner limit removes the earliest one (host summon_owner_cap): removed
      -- after the cast commits, not a death, no bounty.
      for _, e in ipairs(base_effects) do
        local a = e.args or {}
        if e.op == "create_unit" and not e.conds and a.limit_per_owner and a.over_limit == "remove_oldest" and a.template then
          push(rank, "normal", { suffix = "summon_cap/" .. (a.as or a.template),
            given = given({ valid_target, cost.mana and { kind = "mana_at_least", amount = cost.mana } or nil }),
            when = { { kind = "cast_until_over_cap", entry = def.id, target = target.type or "none", template = a.template,
              count = a.count or "1", cap = a.limit_per_owner } },
            expect = { { kind = "summon_cap", template = a.template, cap = a.limit_per_owner, over_limit = a.over_limit,
              rule = "v3.md create_unit limit_per_owner／over_limit = remove_oldest：超過上限時最早的那位消失，不算死亡、沒有賞金" } } })
        end
      end
      -- resources
      if cost.mana then
        push(rank, "insufficient_mana", { given = given({ valid_target, { kind = "mana_exactly", amount = cost.mana .. " - 1" } }),
          when = { cast_step }, expect = { { kind = "cast_rejected", reason = "insufficient_mana" }, { kind = "no_effects" },
            table.unpack(cost_outcomes(false)) } })
      end
      if cost.health then
        push(rank, "insufficient_health", { given = given({ valid_target, { kind = "health_exactly", amount = cost.health } }),
          when = { cast_step }, expect = { { kind = "cast_rejected_or_rule", reason = "insufficient_health" }, { kind = "no_effects" } },
          rule = "v3.md §3.1 cost.health" })
      end
      if cost.charges then
        push(rank, "no_charges", { given = given({ valid_target, { kind = "charges", count = "0" } }), when = { cast_step },
          expect = { { kind = "cast_rejected", reason = "no_charges" }, { kind = "charge_restore", after = cost.charges.restore } } })
      end
      -- With stored charges a second cast right away spends the next charge; no_charges covers
      -- the empty case (E207).
      if cost.cooldown and not cost.charges then
        push(rank, "cooldown", { given = given({ valid_target }),
          when = { cast_step, { kind = "wait", seconds = "0" }, cast_step },
          expect = { { kind = "cast_rejected", nth = 2, reason = "cooldown", remaining = cost.cooldown },
            { kind = "cast_accepted", nth = 3, after_wait = cost.cooldown, note = "the third cast, after waiting the cooldown" } } })
      end
      -- targeting
      if target.type == "unit" then
        local cases = {}
        local team = target.team or "enemy"
        if team == "enemy" then cases[#cases + 1] = { suffix = "ally", fact = { kind = "target_unit", team = "ally" } } end
        if team == "ally" or team == "ally_not_self" then cases[#cases + 1] = { suffix = "enemy", fact = { kind = "target_unit", team = "enemy" } } end
        if has_flag(castable, "no_target_self") or team == "enemy" or team == "ally_not_self" then
          cases[#cases + 1] = { suffix = "self", fact = { kind = "target_unit", team = "self" } }
        end
        local types = {}
        for _, t in ipairs(target.types or { "hero" }) do types[t] = true end
        if not types.creep and not types.all then cases[#cases + 1] = { suffix = "creep", fact = { kind = "target_unit", team = team, types = { "creep" } } } end
        if not types.building and not has_flag(castable, "can_target_buildings") then
          cases[#cases + 1] = { suffix = "building", fact = { kind = "target_unit", team = team, types = { "building" } } }
        end
        cases[#cases + 1] = { suffix = "dead", fact = { kind = "target_unit", team = team, types = target.types, dead = true } }
        for _, c in ipairs(cases) do
          push(rank, "target_invalid", { suffix = c.suffix, given = given({ c.fact }), when = { cast_step },
            expect = { { kind = "cast_rejected", reason = "invalid_target" }, { kind = "no_effects" }, table.unpack(cost_outcomes(false)) } })
        end
        if cast.cast_point then
          push(rank, "target_invalid", { suffix = "dies_in_cast_point", given = given({ valid_target }),
            when = { cast_step, { kind = "target_dies", during = "cast_point", cast_point = cast.cast_point } },
            expect = { { kind = "cast_cancelled" }, { kind = "no_effects" }, table.unpack(cost_outcomes(false)) },
            rule = "Dota: cost and cooldown are paid at the end of the cast point" })
        end
        if target.range then
          push(rank, "target_invalid", { suffix = "out_of_range", given = given({ valid_target, { kind = "distance", more_than = target.range } }),
            when = { cast_step }, expect = { { kind = "approach_until", distance = target.range }, { kind = "then_cast_accepted" } } })
        end
      end
      -- magic immunity
      local enemy_side = target.team == "enemy" or target.team == "both" or (target.type ~= "unit" and has_damage(base_effects))
      if enemy_side then
        if target.type == "unit" and not has_flag(castable, "pierce_magic_immune") then
          push(rank, "magic_immune", { given = given({ { kind = "target_unit", team = "enemy", types = target.types, states = { "magic_immune" } } }),
            when = { cast_step }, expect = { { kind = "cast_rejected", reason = "magic_immune" }, { kind = "no_effects" } },
            rule = "預設（Dota 目標旗標沒有 MAGIC_IMMUNE_ENEMIES）：沒有 pierce_magic_immune 時不能以魔免敵人為目標" })
        else
          local o = { { kind = "cast_accepted" } }
          if has_damage(base_effects, "magical") then
            o[#o + 1] = { kind = "damage_to_magic_immune", type = "magical", amount = "0", rule = "v3.md §4 規則層：魔法傷害對魔免歸零" }
          end
          -- direct: statuses the cast puts on its own target (a piercing cast reaches it); the
          -- skip is about the units selectors pick (E223)
          local direct = {}
          for _, a in ipairs(applied) do
            if a.to == "target" and a.status and a.status.runtime_id then direct[#direct + 1] = a.status.runtime_id end
          end
          o[#o + 1] = { kind = "selectors_skip_magic_immune", unless_flag = "include_magic_immune", direct = direct,
            rule = "v3.md §4.1 選取預設排除魔免" }
          push(rank, "magic_immune", { given = given({ { kind = "enemy_unit_in_effect", states = { "magic_immune" } } }),
            when = { cast_step }, expect = o })
        end
      end
      -- visibility
      if target.type == "unit" then
        push(rank, "visibility", { suffix = "invisible_target", given = given({ { kind = "target_unit", team = "enemy", types = target.types, states = { "invisible" }, seen_by_caster_team = false } }),
          when = { cast_step }, expect = { { kind = "cast_rejected", reason = "not_visible" }, { kind = "no_effects" } },
          rule = "unit targets must be visible to the caster's team when the order is issued" })
        push(rank, "visibility", { suffix = "fogged_target", given = given({ { kind = "target_unit", team = "enemy", types = target.types, in_fog = true } }),
          when = { cast_step }, expect = { { kind = "cast_rejected", reason = "not_visible" } } })
      end
      do
        local breaks = not has_flag(castable, "does_not_break_invisibility")
        push(rank, "visibility", { suffix = "invisible_caster", given = given({ valid_target, { kind = "caster_state", states = { "invisible" } } }),
          when = { cast_step }, expect = { { kind = breaks and "caster_invisibility_broken" or "caster_invisibility_kept" } },
          rule = "v3.md ABILITY_FLAGS break_invisibility / does_not_break_invisibility" })
      end
      -- death
      if cast.cast_point then
        push(rank, "death", { suffix = "caster_dies_in_cast_point", given = given({ valid_target }),
          when = { cast_step, { kind = "caster_dies", during = "cast_point", cast_point = cast.cast_point } },
          expect = { { kind = "cast_cancelled" }, { kind = "no_effects" }, table.unpack(cost_outcomes(false)) } })
      end
      local late = {}
      for _, e in ipairs(base_effects) do
        if e.conds or e.phase ~= "on_cast" then late[#late + 1] = e end
      end
      local after = { { kind = "caster_dies", after = "cast" } }
      local exp_d = {}
      for _, a in ipairs(applied) do
        local s = a.status or {}
        local on_caster = a.to == "caster" or a.to == "self"
        local kept = s.scope == "hero" or s.scope == "unit" or s.scope == "permanent"
        exp_d[#exp_d + 1] = { kind = (on_caster and not kept) and "status_removed" or "status_kept", status = a.id, on = a.to, runtime_id = runtime_key(s),
          rule = on_caster and "v3.md §12：狀態隨持有者死亡移除，scope hero/unit/permanent 例外" or "施加者死亡不移除別人身上的狀態" }
      end
      for _, e in ipairs(late) do
        if e.op == "damage" and not (e.args and e.args.require_source_alive == false) then
          exp_d[#exp_d + 1] = { kind = "damage_skipped_if_source_dead", path = e.path, rule = "v3.md §4 規則層 require_source_alive 預設 true" }
        end
      end
      if #exp_d > 0 then push(rank, "death", { suffix = "caster_dies_after_cast", given = given({ valid_target }), when = { cast_step, after[1] }, expect = exp_d }) end
    end
    -- passive triggers (abilities' passive, items' passive)
    if passive and passive.triggers then
      for _, ev in ipairs(sorted_keys(passive.triggers)) do
        local eff = trigger_effects(passive.triggers[ev], "passive.triggers." .. ev, ev)
        push(rank, "trigger", { suffix = ev, given = given(), when = { { kind = "event", event = ev } },
          expect = { { kind = "effects", effects = eff } } })
        if has_damage(eff, "magical") or #applied_statuses(eff, by_status) > 0 then
          push(rank, "magic_immune", { suffix = ev, given = given({ { kind = "event_target", states = { "magic_immune" } } }),
            when = { { kind = "event", event = ev } }, expect = { { kind = "damage_to_magic_immune", type = "magical", amount = "0" },
              { kind = "statuses_follow_immune_to", rule = "v3.md §12：魔免單位不受一般狀態，除非施加方 pierce" } } })
        end
      end
    end
    if passive and (passive.stats or passive.states) then
      push(rank, "normal", { suffix = "passive_stats", given = given(), when = {},
        expect = { { kind = "passive_stats", stats = passive.stats, states = passive.states } } })
      push(rank, "death", { suffix = "passive_after_respawn", given = given(), when = { { kind = "owner_dies" }, { kind = "owner_respawns" } },
        expect = { { kind = "passive_stats", stats = passive.stats, states = passive.states } } })
    end
    -- statuses this entry can apply: dispel and expiry
    local statuses = {}
    for _, a in ipairs(applied) do statuses[#statuses + 1] = a end
    if passive and passive.triggers then
      for _, ev in ipairs(sorted_keys(passive.triggers)) do
        for _, a in ipairs(applied_statuses(trigger_effects(passive.triggers[ev], "passive.triggers." .. ev, ev), by_status)) do
          statuses[#statuses + 1] = a
        end
      end
    end
    -- statuses the entry owns but no action of it applies directly (auras, statuses of statuses)
    local listed = {}
    for _, a in ipairs(statuses) do listed[a.id] = true end
    for _, s in ipairs(def.statuses or {}) do
      if not listed[s.id] then statuses[#statuses + 1] = { status = s, id = s.id, to = "holder" } end
    end
    -- runtime_id: the BuffStore key the port uses for the status (E211), so a test can find it.
    local function on_unit(s, a) return { kind = "status_on_unit", status = s.id, on = a.to, runtime_id = runtime_key(s),
      visibility = s.visibility } end
    local done = {}
    for _, a in ipairs(statuses) do
      local s = a.status
      if s and not done[s.id] then
        done[s.id] = true
        local d = s.dispel or "basic"
        push(rank, "dispel", { suffix = s.id .. "/basic", given = given({ on_unit(s, a) }),
          when = { { kind = "dispel", strength = "basic" } },
          expect = { { kind = d == "basic" and "status_removed" or "status_kept", status = s.id, dispel = d } } })
        push(rank, "dispel", { suffix = s.id .. "/strong", given = given({ on_unit(s, a) }),
          when = { { kind = "dispel", strength = "strong" } },
          expect = { { kind = (d == "basic" or d == "strong") and "status_removed" or "status_kept", status = s.id, dispel = d } } })
        push(rank, "death", { suffix = s.id .. "/holder_dies", given = given({ on_unit(s, a) }),
          when = { { kind = "holder_dies" } },
          expect = { { kind = (s.scope == "hero" or s.scope == "unit" or s.scope == "permanent") and "status_kept" or "status_removed", status = s.id } } })
        for _, ev in ipairs(sorted_keys(s.triggers or {})) do
          push(rank, "trigger", { suffix = s.id .. "/" .. ev, given = given({ on_unit(s, a) }),
            -- interval: a periodic trigger's period (E220), so a test can time its ticks
            when = { { kind = "status_event", status = s.id, event = ev, interval = type(s.triggers[ev]) == "table" and s.triggers[ev].interval or nil } },
            expect = { { kind = "effects", effects = trigger_effects(s.triggers[ev], "statuses." .. s.id .. ".triggers." .. ev, ev) } } })
        end
        if s.duration and s.duration ~= "infinite" then
          push(rank, "normal", { suffix = s.id .. "/expires", given = given({ on_unit(s, a) }),
            when = { { kind = "wait", seconds = s.duration } }, expect = { { kind = "status_removed", status = s.id } } })
        end
        push(rank, "visibility", { suffix = s.id, given = given({ on_unit(s, a) }),
          when = { { kind = "observe", from = "both_teams" } },
          expect = { { kind = "status_display", status = s.id, visibility = s.visibility, reveal = s.reveal,
            rule = "v3.md §3.2 reveal：public 雙方 replica 都知道；team 只有己方；derived 只揭露結果" } } })
      end
    end
  end
  -- hooks the entry uses (§8): their declared cases, or their text
  local used, seen = {}, {}
  local function scan(t)
    if type(t) == "string" then
      for name in t:gmatch("hook_value%(%s*([%w_]+)") do if not seen[name] then seen[name] = true; used[#used + 1] = name end end
    elseif type(t) == "table" then
      if t.op == "hook" and type(t.name) == "string" and not seen[t.name] then seen[t.name] = true; used[#used + 1] = t.name end
      -- the whole entry (or an item's active) is a hook (D7): `hook = "<name>"`
      if type(t.hook) == "string" and not seen[t.hook] then seen[t.hook] = true; used[#used + 1] = t.hook end
      for _, k in ipairs(sorted_keys(t)) do scan(t[k]) end
    end
  end
  scan(def)
  for _, name in ipairs(used) do
    local h = (doc.hooks or {})[name] or {}
    if h.cases and #h.cases > 0 then
      for rank = 1, max_level do
        for i, c in ipairs(h.cases) do
          local g = { { kind = "learned", entry = def.id, rank = rank } }
          for _, fact in ipairs(c.given or {}) do g[#g + 1] = fact end
          push(rank, "hook", { suffix = name .. "/" .. (c.id or tostring(i)), given = g, when = c.when or {}, expect = c.expect or {},
            hook = name, rule = "v3.md §8 hook cases" })
        end
      end
    else
      push(1, "hook", { suffix = name, given = { { kind = "learned", entry = def.id, rank = 1 } }, when = {},
        expect = { { kind = "hook_behavior", hook = name, text = h.behavior or h.why, test = h.test } }, hook = name })
    end
  end
  -- one scenario per quirk (rank 1; the quirk text says when it applies)
  for i, q in ipairs(def.quirks or {}) do
    push(1, "quirk", { suffix = tostring(i), given = { { kind = "learned", entry = def.id, rank = 1 } }, when = {},
      expect = { { kind = "quirk_policy", policy = q.policy, text = q.text, ref = q.ref } }, quirk = { policy = q.policy, ref = q.ref } })
  end
end

-- doc -> { schema, id, kind, scenarios }
function M.build(doc, opts)
  opts = opts or {}
  local out = {}
  for _, a in ipairs(doc.abilities or {}) do
    if a.coverage ~= "deferred" then entry_scenarios(doc, a, "ability", out) end
  end
  for _, it in ipairs(doc.items or {}) do
    if it.coverage ~= "deferred" then entry_scenarios(doc, it, "item", out) end
  end
  return { schema = M.SCHEMA, id = doc.id, kind = doc.kind, document = opts.path, scenarios = out }
end

-- problems: every ref resolves to a value of the document; no number besides 0 and 1 in expectations
function M.check(doc, result)
  local problems = {}
  local values = {}
  local function add(owner, vals)
    for k, v in pairs(vals or {}) do
      local n = type(k) == "string" and k or (type(v) == "table" and v.name)
      if n then values[n] = true; if owner then values[owner .. "." .. n] = true end end
    end
  end
  add(nil, doc.values)
  for _, list in ipairs({ doc.abilities or {}, doc.items or {}, doc.unit_templates or {}, doc.statuses or {} }) do
    for _, e in ipairs(list) do
      add(e.id, e.values)
      for _, s in ipairs(e.statuses or {}) do add(s.id, s.values) end
    end
  end
  for _, s in ipairs(result.scenarios) do
    for _, r in ipairs(s.refs) do
      if not values[r] then problems[#problems + 1] = s.id .. ": $" .. r .. " is not a value of the document" end
    end
    local function scan(t, where)
      if type(t) == "number" and t ~= 0 and t ~= 1 and t ~= -1 and where ~= "rank" and where ~= "nth" then
        problems[#problems + 1] = s.id .. ": number " .. t .. " in " .. where
      elseif type(t) == "table" then
        for k, v in pairs(t) do if k ~= "text" and k ~= "ref" then scan(v, tostring(k)) end end
      end
    end
    scan(s.given, "given"); scan(s.when, "when"); scan(s.expect, "expect")
  end
  return problems
end

function M.main(argv)
  local root = tools_dir .. "/../../.."
  package.path = root .. "/scripts/?.lua;" .. package.path
  local b = require("_bootstrap")
  local json = b.lib("json")
  local sources = dofile(tools_dir .. "/v3_sources.lua")
  PACKAGE_V3_AS_MODULE = true
  local V = dofile(tools_dir .. "/package_v3.lua")
  PACKAGE_V3_AS_MODULE = nil
  local out_dir, ids, to_stdout = b.root .. "/target/v3-scenarios", {}, false
  local i = 1
  while i <= #argv do
    if argv[i] == "--out" then out_dir = argv[i + 1]; i = i + 1
    elseif argv[i] == "--stdout" then to_stdout = true
    else ids[#ids + 1] = argv[i] end
    i = i + 1
  end
  local files = {}
  if #ids == 0 then files = sources.all(b.root)
  else
    for _, id in ipairs(ids) do
      local p = sources.hero_path(b.root, id)
      if not p then
        for _, f in ipairs(sources.items(b.root)) do if f:match("[/\\]" .. id:gsub("%.lua$", "") .. "%.lua$") then p = f end end
      end
      files[#files + 1] = assert(p, "no v3 document for " .. id)
    end
  end
  if not to_stdout then b.lib("path").mkdir_p(out_dir) end
  local failed, total = 0, 0
  for _, f in ipairs(files) do
    local doc = assert(V.load_file(f))
    local rel = f:gsub("\\", "/"):gsub("^" .. b.root:gsub("\\", "/"):gsub("%p", "%%%0") .. "/", "")
    local result = M.build(doc, { path = rel })
    local problems = M.check(doc, result)
    for _, p in ipairs(problems) do io.stderr:write(rel, ": ", p, "\n") end
    if #problems > 0 then failed = failed + 1 end
    total = total + #result.scenarios
    if to_stdout then print(json.encode(result))
    else
      local fh = assert(io.open(out_dir .. "/" .. doc.id .. ".json", "wb")); fh:write(json.encode(result)); fh:close()
    end
  end
  if not to_stdout then print(("wrote %d document(s), %d scenarios to %s"):format(#files, total, out_dir)) end
  return failed == 0 and 0 or 1
end

if rawget(_G, "V3_SCENARIOS_AS_MODULE") then return M end
os.exit(M.main(arg))
