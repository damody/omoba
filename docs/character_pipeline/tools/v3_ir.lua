-- Package schema v3 behavior -> flat IR (M3, v3-content-model-proposal.md §4.1).
--
--   compile_doc(doc, opts) -> { [entry id] = ir }, problems
--
-- The document must already pass package_v3.lua; this module only lowers it. Every
-- expression becomes a postfix token list, every action list a program (flat instruction
-- array), every `$name` a value slot, every binding a local index. Status, template and
-- ability references stay as ids (resolved by the Rust loader). Indices start at 1.
local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
local V = dofile(tools_dir .. "/v3_vocab.lua")
local X = dofile(tools_dir .. "/v3_expr.lua")
local M = {}
M.VERSION = 1
M.SCALE = 1024
M.EMIT_UNIT_STATE = true

-- cross-document names (v3-shared-registry.lua): unit_state a block uses without declaring it
local registry
function M.registry()
  registry = registry or dofile(tools_dir .. "/../schema/v3-shared-registry.lua")
  return registry
end

local function sorted_keys(t)
  local ks = {}
  for k in pairs(t) do ks[#ks + 1] = k end
  table.sort(ks, function(a, b) return tostring(a) < tostring(b) end)
  return ks
end
local function is_array(t)
  if type(t) ~= "table" then return false end
  local n = 0
  for k in pairs(t) do
    if math.type(k) ~= "integer" or k < 1 then return false end
    n = n + 1
  end
  return n == #t
end

-- Function argument kinds that are written as names (v3_expr.lua) and become {"sym", name}.
local NAME_ARGS = { status = true, template = true, tag = true, name = true, item_name = true, state_flag = true,
  landmark = true, status_tag = true, slot = true, order_kind = true }

-- Actions whose `as` binds their result for the following actions (package_v3.lua BIND_KIND).
local RESULT_OPS = { create_unit = true, create_illusion = true, projectile = true, zone = true, tether = true,
  apply_status = true, damage = true, heal = true, restore_mana = true, burn_mana = true, take_gold = true, reveal = true,
  give_item = true, hook = true, create_pickup = true, create_tree = true, dash = true, attack = true, random_pick = true,
  pick_item = true, chain = true, impulse = true, add_ability = true }

-- Nested action lists that bind `it` (and their extras), as in package_v3.lua.
local IT_LISTS = { on_hit = true, on_enter = true, on_leave = true, on_pickup = true }

function M.compile_doc(doc, opts)
  opts = opts or {}
  local rules = opts.rules
  if not rules then
    PACKAGE_V3_AS_MODULE = true
    rules = dofile(tools_dir .. "/package_v3.lua").rules()
    PACKAGE_V3_AS_MODULE = nil
  end
  local problems = {}

  -- owners: entry / status / template / shared, for $name resolution
  local owners, statuses, abilities, templates = {}, {}, {}, {}
  local shared = { id = "shared", values = doc.values or {} }
  owners.shared = shared
  local function add(def, kind, parent)
    local o = { id = def.id, kind = kind, def = def, values = def.values or {}, parent = parent or shared }
    owners[def.id] = o
    return o
  end
  local entries = {}
  for _, a in ipairs(doc.abilities or {}) do
    local o = add(a, "ability"); abilities[a.id] = o; entries[#entries + 1] = o
    for _, s in ipairs(a.statuses or {}) do statuses[s.id] = add(s, "status", o) end
  end
  for _, it in ipairs(doc.items or {}) do
    local o = add(it, "item"); abilities[it.id] = o; entries[#entries + 1] = o
    for _, s in ipairs(it.statuses or {}) do statuses[s.id] = add(s, "status", o) end
  end
  for _, u in ipairs(doc.unit_templates or {}) do templates[u.id] = add(u, "template") end
  for _, s in ipairs(doc.statuses or {}) do statuses[s.id] = add(s, "status", shared) end
  local routines = doc.routines or {}

  local function find(owner, qual, name)
    if qual then
      local o = owners[qual]
      if o and o.values[name] ~= nil then return o.values[name], o end
      return nil
    end
    local o = owner
    while o do
      if o.values[name] ~= nil then return o.values[name], o end
      o = o.parent
    end
  end
  local function rule_value(parts)
    local t = rules
    for i = 2, #parts do t = type(t) == "table" and t[parts[i]] or nil end
    return type(t) == "number" and t or nil
  end

  local function compile_entry(entry)
    local B = { slots = {}, slot_index = {}, exprs = {}, selectors = {}, programs = {}, locals = 0,
      statuses = {}, routines = {}, templates = {}, hooks = {}, hook_seen = {} }
    local function problem(msg) problems[#problems + 1] = entry.id .. ": " .. msg end
    local function new_local() B.locals = B.locals + 1; return B.locals end
    local function new_scope(parent, names) return { parent = parent, names = names or {} } end
    local function lookup(scope, name)
      while scope do
        if scope.names[name] then return scope.names[name] end
        scope = scope.parent
      end
    end

    -- ctx: { owner, scope }
    local function slot_of(ctx, qual, name)
      local d, o = find(ctx.owner, qual, name)
      if not d then problem("value $" .. (qual and (qual .. ".") or "") .. name .. " not found"); return 0 end
      local key = o.id .. "." .. name
      if not B.slot_index[key] then
        local row = { name = name }
        if o.id ~= entry.id then row.owner = o.id end
        if d.unit == "pct" then row.scale = "pct" end
        -- the loader's pct division follows the value's rounding (B01R truncates, the rest round)
        if d.rounding == "truncate" then row.rounding = "truncate" end
        B.slots[#B.slots + 1] = row
        B.slot_index[key] = #B.slots
      end
      return B.slot_index[key]
    end

    local emit
    local function path_of(node)
      local parts = {}
      while node.k == "field" do table.insert(parts, 1, node.name); node = node.obj end
      if node.k ~= "id" then return nil end
      table.insert(parts, 1, node.name)
      return parts
    end
    emit = function(node, ctx, out)
      local k = node.k
      if k == "num" then out[#out + 1] = { "raw", math.floor(node.v * M.SCALE + 0.5) }
      elseif k == "bool" then out[#out + 1] = { "bool", node.v }
      elseif k == "val" then out[#out + 1] = { "slot", slot_of(ctx, node.owner, node.name) }
      elseif k == "id" then
        local l = lookup(ctx.scope, node.name)
        if l then out[#out + 1] = { "local", l }
        elseif node.name == "none" then out[#out + 1] = { "name", "none" }
        else out[#out + 1] = { "name", node.name } end
      elseif k == "field" then
        local parts = path_of(node)
        if parts and parts[1] == "rules" and not lookup(ctx.scope, "rules") then
          local v = rule_value(parts)
          if v == nil then problem("rules." .. table.concat(parts, ".", 2) .. " has no value") end
          out[#out + 1] = { "raw", math.floor((v or 0) * M.SCALE + 0.5) }
          return
        end
        if parts and (parts[1] == "state" or parts[1] == "vars") and #parts >= 2 and not lookup(ctx.scope, parts[1]) then
          out[#out + 1] = { parts[1] == "state" and "state" or "var", parts[2] }
          for i = 3, #parts do out[#out + 1] = { "attr", parts[i] } end
          return
        end
        emit(node.obj, ctx, out)
        out[#out + 1] = { "attr", node.name }
      elseif k == "un" then
        emit(node.e, ctx, out)
        out[#out + 1] = { "op", node.op == "-" and "neg" or "not" }
      elseif k == "bin" then
        if node.op == "and" or node.op == "or" then
          local l, r = {}, {}
          emit(node.l, ctx, l); emit(node.r, ctx, r)
          for _, t in ipairs(l) do out[#out + 1] = t end
          if node.op == "and" then
            -- l; jif -> false; r; jump over false; false
            out[#out + 1] = { "jump_if_false", #r + 1 }
            for _, t in ipairs(r) do out[#out + 1] = t end
            out[#out + 1] = { "jump", 1 }
            out[#out + 1] = { "bool", false }
          else
            -- l; jif -> r; true; jump over r; r
            out[#out + 1] = { "jump_if_false", 2 }
            out[#out + 1] = { "bool", true }
            out[#out + 1] = { "jump", #r }
            for _, t in ipairs(r) do out[#out + 1] = t end
          end
          return
        end
        emit(node.l, ctx, out); emit(node.r, ctx, out)
        out[#out + 1] = { "op", node.op }
      elseif k == "call" then
        if node.fn == "select" then
          local a, b = {}, {}
          emit(node.args[1], ctx, out)
          emit(node.args[2], ctx, a); emit(node.args[3], ctx, b)
          out[#out + 1] = { "jump_if_false", #a + 1 }
          for _, t in ipairs(a) do out[#out + 1] = t end
          out[#out + 1] = { "jump", #b }
          for _, t in ipairs(b) do out[#out + 1] = t end
          return
        end
        local sig = V.FUNCTIONS[node.fn] or { args = {} }
        local args = sig.args
        local variadic = args[#args] == "..."
        for i, a in ipairs(node.args) do
          local want = args[i] or (variadic and args[#args - 1]) or "any"
          local named = a.k == "id" and not lookup(ctx.scope, a.name) and (NAME_ARGS[want]
            or (want == "ability" and abilities[a.name]) or (want == "team" and V.TEAMS[a.name]))
          if named then
            out[#out + 1] = { "sym", a.name }
            if node.fn == "hook_value" and i == 1 and not B.hook_seen[a.name] then
              B.hook_seen[a.name] = true; B.hooks[#B.hooks + 1] = a.name
            end
          else emit(a, ctx, out) end
        end
        out[#out + 1] = { "call", node.fn, #node.args }
      else
        problem("cannot lower expression node " .. tostring(k))
      end
    end

    -- value (string expression, number or boolean) -> expression index
    local function expr(v, ctx)
      local out = {}
      if type(v) == "number" then out[1] = { "raw", math.floor(v * M.SCALE + 0.5) }
      elseif type(v) == "boolean" then out[1] = { "bool", v }
      elseif type(v) == "string" then
        local ast, err = X.parse(v)
        if not ast then problem("expression `" .. v .. "`: " .. tostring(err)); out[1] = { "bool", false }
        else emit(ast, ctx, out) end
      else
        problem("expression of type " .. type(v)); out[1] = { "bool", false }
      end
      B.exprs[#B.exprs + 1] = out
      return #B.exprs
    end
    local function child(ctx, names) return { owner = ctx.owner, scope = new_scope(ctx.scope, names) } end

    local compile_actions, compile_selector, compile_template

    compile_selector = function(s, ctx)
      local out = {}
      local cand_local = new_local()
      local cand = child(ctx, { it = cand_local })
      for _, k in ipairs(sorted_keys(s)) do
        local v = s[k]
        if k == "notes" then
        elseif k == "from" or k == "to" or k == "relative_to" or k == "visible_to" or k == "exclude"
          or k == "radius" or k == "inner" or k == "outer" or k == "angle" or k == "length" or k == "width" or k == "limit" then
          out[k] = expr(v, ctx)
        elseif k == "where" or k == "prefer" or k == "post_filter" or k == "take_while" then
          out[k] = expr(v, cand)
        else
          out[k] = v
        end
      end
      out.shape = out.shape or "circle"
      out.it = cand_local
      B.selectors[#B.selectors + 1] = out
      return #B.selectors
    end

    local function compile_routine(name)
      if B.routines[name] then return end
      local r = routines[name]
      if not r then problem("routine " .. name .. " not declared"); return end
      local names, params, locals = {}, {}, {}
      for _, p in ipairs(sorted_keys(r.params or {})) do
        local l = new_local(); names[p] = l; params[#params + 1] = p; locals[#locals + 1] = l
      end
      B.routines[name] = { params = params, locals = locals } -- reserve before compiling (recursion)
      local owner = r.owner and abilities[r.owner] or shared
      B.routines[name].program = compile_actions(r.actions or {}, { owner = owner, scope = new_scope(nil, names) })
    end

    local function compile_param(kind, v, ctx, instr, key, a)
      local base = kind:gsub("!$", "")
      if base:sub(1, 5) == "list:" then
        local sub = base:sub(6)
        local out = {}
        for i, x in ipairs(v) do
          if sub == "any" or sub == "expr" or sub == "unit" or sub == "point" then out[i] = expr(x, ctx) else out[i] = x end
        end
        return out
      end
      if base == "expr" or base == "cond" or base == "unit" or base == "point" or base == "vec" or base == "any" then
        return expr(v, ctx)
      elseif base == "selector" then return compile_selector(v, ctx)
      elseif base == "ability" or base == "item_ref" then
        if type(v) == "string" and (abilities[v] or v:match("^item_[a-z0-9_]+$")) then return v end
        instr[key .. "_expr"] = expr(v, ctx)
        return nil
      elseif base == "template" then
        compile_template(v)
        return v
      elseif base == "routine" then
        compile_routine(v)
        return v
      elseif base == "hook" then
        if not B.hook_seen[v] then B.hook_seen[v] = true; B.hooks[#B.hooks + 1] = v end
        return v
      elseif base == "table" or base == "unit_overrides" or base == "illusion_damage" then
        local out = {}
        for _, k in ipairs(sorted_keys(v)) do
          if type(v[k]) == "table" then
            out[k] = {}
            for _, k2 in ipairs(sorted_keys(v[k])) do out[k][k2] = expr(v[k][k2], ctx) end
          else out[k] = expr(v[k], ctx) end
        end
        return out
      elseif base == "illusion_copy" then
        local out = {}
        for _, k in ipairs(sorted_keys(v)) do
          out[k] = (k == "hp" or k == "mana") and expr(v[k], ctx) or v[k]
        end
        return out
      elseif base == "options" then
        local out = {}
        for i, opt in ipairs(v) do
          local o = {}
          if opt.weight ~= nil then o.weight = expr(opt.weight, ctx) end
          if opt.value ~= nil then o.value = expr(opt.value, ctx) end
          if opt.actions ~= nil then o.actions = compile_actions(opt.actions, child(ctx)) end
          out[i] = o
        end
        return out
      elseif base == "callbacks" then
        local out = {}
        for _, k in ipairs(sorted_keys(v)) do out[k] = compile_actions(v[k], child(ctx)) end
        return out
      end
      -- status, cue, text, item_name, name, bool, enum:*, flag, state_flag: as written
      return v
    end

    compile_actions = function(list, ctx)
      local prog = {}
      B.programs[#B.programs + 1] = prog
      local index = #B.programs
      local scope_ctx = child(ctx)
      for _, a in ipairs(list or {}) do
        local spec = V.ACTIONS[a.op]
        local instr = { op = a.op }
        if not spec then problem("unknown action " .. tostring(a.op))
        else
          -- result and callback bindings first, so nested lists can see them
          local self_bind, repeat_local
          if a.as and (a.op == "projectile" or a.op == "dash" or a.op == "zone" or a.op == "tether") then
            self_bind = new_local()
          end
          if a.op == "repeat" and a.as then repeat_local = new_local(); instr.index = repeat_local end
          for _, k in ipairs(sorted_keys(a)) do
            local v = a[k]
            local kind = spec[k]
            if k == "op" or k == "notes" then
            elseif not kind then problem(a.op .. " has no parameter " .. k)
            elseif k == "as" or (a.op == "let" and k == "name") then
              -- bound after the action (below)
            elseif (k == "id" and (a.op == "delay" or a.op == "repeat")) or (a.op == "cancel_delay" and k == "id") then
              instr[k] = v
            elseif kind:gsub("!$", "") == "actions" then
              local names, bind = {}, {}
              if a.op == "for_each" then local l = new_local(); names[a.as or "it"] = l; bind.it = l end
              if repeat_local then names[a.as] = repeat_local end
              if a.op == "chain" then
                local l, j = new_local(), new_local(); names.it = l; names.jump = j; bind.it = l; bind.jump = j
              end
              if IT_LISTS[k] then local l = new_local(); names.it = l; bind.it = l end
              if a.op == "projectile" then local l = new_local(); names.here = l; bind.here = l end
              if self_bind then names[a.as] = self_bind; bind.self = self_bind end
              instr[k] = compile_actions(v, child(scope_ctx, names))
              if next(bind) then instr[k .. "_bind"] = bind end
            elseif (a.op == "projectile" or a.op == "zone" or a.op == "chain") and (k == "where" or k == "take_while") then
              local l = new_local()
              instr[k] = expr(v, child(scope_ctx, { it = l }))
              instr[k .. "_it"] = l
            elseif a.op == "pick_item" and k == "where" then
              local l = new_local()
              instr.where = expr(v, child(scope_ctx, { it = l }))
              instr.where_it = l
            elseif a.op == "repeat" and (k == "while" or k == "stop_when") then
              local names = {}
              if repeat_local then names[a.as] = repeat_local end
              instr[k] = expr(v, child(scope_ctx, names))
            elseif a.op == "delay" and k == "capture" then
              local caps = {}
              for i, n in ipairs(v) do caps[i] = lookup(scope_ctx.scope, n) or 0 end
              instr.capture = caps
            else
              local value = compile_param(kind, v, scope_ctx, instr, k, a)
              if value ~= nil then instr[k] = value end
            end
          end
          if a.op == "let" then
            local l = new_local()
            instr.bind = l
            scope_ctx.scope.names[a.name] = l
          elseif a.as and RESULT_OPS[a.op] then
            local l = self_bind or new_local()
            instr.bind = l
            scope_ctx.scope.names[a.as] = l
          end
        end
        prog[#prog + 1] = instr
      end
      return index
    end

    local function trigger(def, ctx, event_names)
      if type(def) ~= "table" then return nil end
      if is_array(def) then return { program = compile_actions(def, child(ctx, event_names)) } end
      local t = { program = compile_actions(def.actions or {}, child(ctx, event_names)) }
      for _, k in ipairs({ "filter", "interval", "first", "priority" }) do
        if def[k] ~= nil then t[k] = expr(def[k], child(ctx, event_names)) end
      end
      for _, k in ipairs({ "who", "phase" }) do if def[k] ~= nil then t[k] = def[k] end end
      return t
    end
    local function triggers(map, ctx)
      if type(map) ~= "table" then return nil end
      local out = {}
      for _, name in ipairs(sorted_keys(map)) do out[name] = trigger(map[name], ctx, {}) end
      return next(out) and out or nil
    end
    local function stats(map, ctx)
      if type(map) ~= "table" then return nil end
      local out = {}
      for _, stat in ipairs(sorted_keys(map)) do
        local v = map[stat]
        local row = { stat = stat }
        if type(v) == "table" then
          row.op = v.op; row.expr = expr(v.value, ctx)
          if v.when ~= nil then row.when = expr(v.when, ctx) end
          row.per_stack = v.per_stack
        else row.expr = expr(v, ctx) end
        out[#out + 1] = row
      end
      return #out > 0 and out or nil
    end
    local function aura(a, ctx)
      if type(a) ~= "table" then return nil end
      local out = {}
      for _, k in ipairs(sorted_keys(a)) do
        if k == "radius" or k == "linger" then out[k] = expr(a[k], ctx)
        elseif k == "where" then local l = new_local(); out.where = expr(a.where, child(ctx, { it = l })); out.where_it = l
        else out[k] = a[k] end
      end
      return out
    end
    local function passive(pv, ctx)
      if type(pv) ~= "table" then return nil end
      local pctx = child(ctx, { self = nil })
      local out = { stats = stats(pv.stats, pctx), states = pv.states, unique_group = pv.unique_group,
        triggers = triggers(pv.triggers, pctx), aura = aura(pv.aura, pctx), reveal = pv.reveal, visibility = pv.visibility }
      if pv.orb_slot then
        out.orb_slot = { release = pv.orb_slot.release, claim = pv.orb_slot.claim, overridable = pv.orb_slot.overridable }
        if pv.orb_slot.priority ~= nil then out.orb_slot.priority = expr(pv.orb_slot.priority, pctx) end
      end
      return out
    end
    local function status(s, so)
      local ctx = { owner = so, scope = new_scope(nil, {}) }
      local out = { id = s.id }
      for _, k in ipairs({ "dispel", "visibility", "reveal", "status_resistance", "scope", "unique_group", "unique_rule",
        "block_reapply", "per_source", "reapply_if_removed", "cue", "tags", "states", "immune_to", "remove_on" }) do
        out[k] = s[k]
      end
      out.duration = s.duration == "infinite" and "infinite" or expr(s.duration, ctx)
      if type(s.stacking) == "table" then
        out.stacking = { mode = s.stacking.mode, value_stacks = s.stacking.value_stacks }
        for _, k in ipairs({ "max", "per_stack_duration", "decay", "initial" }) do
          if s.stacking[k] ~= nil then out.stacking[k] = expr(s.stacking[k], ctx) end
        end
      else out.stacking = s.stacking end
      out.stats = stats(s.stats, ctx)
      if s.vars then
        out.vars = {}
        for _, n in ipairs(sorted_keys(s.vars)) do
          local d = s.vars[n]
          out.vars[#out.vars + 1] = { name = n, type = d.type, reveal = d.reveal, init = d.init ~= nil and expr(d.init, ctx) or nil }
        end
      end
      if s.absorb then
        local ab = { types = s.absorb.types, lethal_only = s.absorb.lethal_only, phase = s.absorb.phase }
        ab.amount = expr(s.absorb.amount, ctx)
        if s.absorb.fraction ~= nil then ab.fraction = expr(s.absorb.fraction, ctx) end
        if s.absorb.on_absorb then ab.on_absorb = compile_actions(s.absorb.on_absorb, child(ctx)) end
        if s.absorb.on_broken then ab.on_broken = compile_actions(s.absorb.on_broken, child(ctx)) end
        out.absorb = ab
      end
      for _, k in ipairs({ "charges", "linger" }) do if s[k] ~= nil then out[k] = expr(s[k], ctx) end end
      out.aura = aura(s.aura, ctx)
      if s.forced_move then
        out.forced_move = { locks = s.forced_move.locks, toward = expr(s.forced_move.toward, ctx) }
        if s.forced_move.speed ~= nil then out.forced_move.speed = expr(s.forced_move.speed, ctx) end
      end
      out.triggers = triggers(s.triggers, ctx)
      -- the document's runtime_id (v3.3), unless the caller overrides it
      if opts.runtime_ids and opts.runtime_ids[s.id] then out.runtime_id = opts.runtime_ids[s.id]
      elseif s.runtime_id then out.runtime_id = s.runtime_id end
      return out
    end

    compile_template = function(id)
      if B.templates[id] then return end
      local o = templates[id]
      if not o then problem("unit template " .. id .. " not declared"); return end
      local u = o.def
      local ctx = { owner = o, scope = new_scope(nil, {}) }
      local out = { source_id = u.source_id }
      B.templates[id] = out
      for _, k in ipairs({ "statuses", "abilities", "ai", "over_limit", "tags", "blocks_pathing", "flying", "owner_death",
        "inventory_locked", "item_cast_blacklist" }) do out[k] = u[k] end
      for _, k in ipairs({ "collision_radius", "limit_per_owner", "acquisition_range", "turn_rate" }) do
        if u[k] ~= nil then out[k] = expr(u[k], ctx) end
      end
      for _, group in ipairs({ "stats", "bounty", "vision" }) do
        if type(u[group]) == "table" then
          out[group] = {}
          for _, k in ipairs(sorted_keys(u[group])) do out[group][k] = expr(u[group][k], ctx) end
        end
      end
      if type(u.attack) == "table" then
        out.attack = { kind = u.attack.kind, damage_type = u.attack.damage_type }
        for _, k in ipairs({ "range", "interval", "damage", "damage_max", "projectile_speed", "attack_point" }) do
          if u.attack[k] ~= nil then out.attack[k] = expr(u.attack[k], ctx) end
        end
      end
      if type(u.chase) == "table" then
        out.chase = { anchor = u.chase.anchor, retarget = u.chase.retarget }
        for _, k in ipairs({ "speed", "leash", "reach_radius", "give_up_after", "max_distance", "teleport_back_distance" }) do
          if u.chase[k] ~= nil then out.chase[k] = expr(u.chase[k], ctx) end
        end
        if u.chase.on_reach then
          local l = new_local()
          out.chase.on_reach = compile_actions(u.chase.on_reach, child(ctx, { target = l }))
          out.chase.target = l
        end
      end
      out.passive = passive(u.passive, ctx)
    end

    -- entry
    local def = entry.def
    local ctx = { owner = entry, scope = new_scope(nil, {}) }
    local castable = entry.kind == "ability" and def or def.active
    if castable and type(castable.events) == "table" then
      B.entries = {}
      for _, name in ipairs(sorted_keys(castable.events)) do B.entries[name] = trigger(castable.events[name], ctx, {}) end
    end
    B.passive = passive(def.passive, ctx)
    for _, s in ipairs(def.statuses or {}) do B.statuses[#B.statuses + 1] = status(s, statuses[s.id]) end
    if castable and castable.requires and castable.requires.condition ~= nil then
      B.requires = expr(castable.requires.condition, ctx)
    end
    -- cast validation (work line B): the v3 target block and the castable's flags, so a data entry
    -- rejects the same casts its handler did (self, non-hero, building, out of range, wrong team)
    if castable and type(castable.target) == "table" then
      local t = castable.target
      local tgt = { type = t.type, team = t.team }
      if type(t.types) == "table" then tgt.types = { table.unpack(t.types) } end
      if type(t.flags) == "table" then tgt.flags = { table.unpack(t.flags) } end
      if type(castable.flags) == "table" then tgt.ability_flags = { table.unpack(castable.flags) } end
      for _, k in ipairs({ "range", "min_range", "aoe_radius", "vector_length" }) do
        if t[k] ~= nil then tgt[k] = expr(t[k], ctx) end
      end
      B.target = tgt
    end

    -- unit_state declarations (work line B, m3-ir-design.md §7.2): every variable this block reads or
    -- writes plus the entry's own, with type, scope, reveal and init, so the interpreter knows where it
    -- lives and what it starts at. On by default (M.EMIT_UNIT_STATE; opts.unit_state = false turns it off);
    -- omoba-content-model IrUnitState checks it (sorted names, known scope, init in range).
    local unit_state
    if opts.unit_state or (opts.unit_state == nil and M.EMIT_UNIT_STATE) then
      local seen, names = {}, {}
      local function note(n)
        if type(n) == "string" and not seen[n] then seen[n] = true; names[#names + 1] = n end
      end
      local function scan(t)
        if type(t) ~= "table" then return end
        if t[1] == "state" and #t == 2 then note(t[2]) end
        if t.op == "set_state" or t.op == "clear_state" then note(t.var) end
        if t.op == "chance" then note(t.counter_var) end
        for _, v in pairs(t) do scan(v) end
      end
      for n in pairs(def.unit_state or {}) do note(n) end
      for _, part in ipairs({ B.exprs, B.selectors, B.programs, B.statuses, B.routines, B.templates, B.entries, B.passive }) do
        scan(part)
      end
      local function declaration(n)
        if def.unit_state and def.unit_state[n] then return def.unit_state[n], entry end
        if doc.unit_state and doc.unit_state[n] then return doc.unit_state[n], shared end
        -- a routine's owner reads its own entry's unit_state (v3.md §12.10)
        for _, e in ipairs(entries) do
          if e.def.unit_state and e.def.unit_state[n] then return e.def.unit_state[n], e end
        end
        local r = M.registry().unit_state[n]
        if r then return r, nil end
      end
      unit_state = {}
      local done = 0
      while done < #names do -- an init may read another variable: follow it too
        done = done + 1
        local n = names[done]
        local d, o = declaration(n)
        if not d then problem("unit_state " .. n .. " is not declared")
        else
          local first = #B.exprs + 1
          local row = { name = n, type = d.type, scope = d.scope, reveal = d.reveal }
          if d.init ~= nil then row.init = expr(d.init, { owner = o or shared, scope = new_scope(nil, {}) }) end
          for i = first, #B.exprs do scan(B.exprs[i]) end
          unit_state[#unit_state + 1] = row
        end
      end
      table.sort(unit_state, function(a, b) return a.name < b.name end)
    end

    local ir = { version = M.VERSION, slots = B.slots, exprs = B.exprs, selectors = B.selectors, programs = B.programs,
      locals = B.locals, entries = B.entries, passive = B.passive, statuses = B.statuses, routines = B.routines,
      templates = B.templates, hooks = B.hooks, requires = B.requires, target = B.target, unit_state = unit_state }
    -- empty tables have no JSON shape (v3_compile.lua): leave them out
    for _, k in ipairs({ "slots", "exprs", "selectors", "programs", "statuses", "hooks", "unit_state" }) do
      if ir[k] and #ir[k] == 0 then ir[k] = nil end
    end
    for _, k in ipairs({ "routines", "templates", "entries" }) do
      if ir[k] and next(ir[k]) == nil then ir[k] = nil end
    end
    for _, prog in ipairs(ir.programs or {}) do
      if #prog == 0 then prog[1] = { op = "stop" } end -- an empty program has no JSON shape either
    end
    -- and no empty table anywhere below (args = {}, an empty tag list, ...)
    local function prune(tab)
      for k, v in pairs(tab) do
        if type(v) == "table" then
          prune(v)
          if next(v) == nil then tab[k] = nil end
        end
      end
    end
    prune(ir)
    return ir
  end

  local out = {}
  for _, e in ipairs(entries) do
    local def = e.def
    local has_behavior = def.passive ~= nil or (def.statuses and #def.statuses > 0)
      or (e.kind == "ability" and def.events ~= nil) or (e.kind == "item" and def.active and def.active.events ~= nil)
    if has_behavior and def.coverage ~= "rust_handler" and def.coverage ~= "deferred" then
      out[def.id] = compile_entry(e)
    end
  end
  return out, problems
end

return M
