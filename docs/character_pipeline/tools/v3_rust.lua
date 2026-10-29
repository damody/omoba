-- Package schema v3 -> one Rust crate per hero (M3' ③, m3-rust-codegen-design.md §2, D15/D16).
--
--   tools\lua\lua.exe docs\character_pipeline\tools\v3_rust.lua --out DIR [--hero PACKAGE ...] [--listed] [--all] [--strict]
--
-- Writes DIR/heroes/<id>_<package>/src/generated/*.rs (mod.rs, registry.rs, values.rs, ids.rs,
-- routines.rs, one file per entry). Only files whose text changed are rewritten (cargo keys on
-- mtimes). src/generated/ is not version-controlled (.gitignore). Cargo.toml and src/lib.rs (the
-- export_root_module shell over generated::registry) are written only when missing: line B owns
-- them; src/hooks/ is hand-written and only reads the generated constants.
--
-- What the generated code may do (m3-rust-codegen-design §2.2-2.4):
-- - every number comes from the document's `values`, converted from its decimal text to a raw
--   Fixed64 with integer arithmetic (v3_compile.raw_of); `pct` values also get a `_FRAC` table
--   (÷100, truncated or rounded like the IR loader's pct_raw);
-- - statuses, cues and stat keys are typed constants: naming one that does not exist fails rustc;
-- - the behavior lowers to straight-line Rust over ability_kit::Cx (the IR interpreter's host
--   mappings). Selections are sorted by entity id.
--
-- A form the generator does not lower yet makes that entry a stub that refuses to run (the cast
-- fails with the list of forms), and the form is reported (v3_codegen_coverage.lua). With --strict
-- the generator fails instead, naming the entry and the form.
local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
local V = dofile(tools_dir .. "/v3_vocab.lua")
local X = dofile(tools_dir .. "/v3_expr.lua")
local C = dofile(tools_dir .. "/v3_compile.lua")
local M = {}

M.HEADER = "// 由 v3_rust.lua 產生，不要手改（docs/character_pipeline/tools/v3_rust.lua）。\n"

local function sorted_keys(t)
  local ks = {}
  for k in pairs(t or {}) do ks[#ks + 1] = k end
  table.sort(ks, function(a, b) return tostring(a) < tostring(b) end)
  return ks
end
local function set(list) local s = {} for _, k in ipairs(list) do s[k] = true end return s end
local function upper(id) return (id:gsub("[^%w]", "_"):upper()) end
local function camel(id)
  local s = id:gsub("[^%w]", "_"):gsub("_(%w)", function(c) return c:upper() end):gsub("^%l", string.upper)
  return s
end
local function rust_str(s) return '"' .. s:gsub("\\", "\\\\"):gsub('"', '\\"') .. '"' end
function M.crate_name(doc) return "hero_" .. doc.id:lower() end
-- scripts/heroes/<package> (line B, e.g. heroes/inahime); the DLL package is heroes/<package>/dll
function M.crate_dir(_, package) return package end

---------------------------------------------------------------------------
-- rules-layer vocabulary (omoba-core rules/vocab.rs): v3 stat / state names -> host keys
---------------------------------------------------------------------------
local vocab_cache
function M.vocab(root)
  if vocab_cache then return vocab_cache end
  local f = assert(io.open(root .. "/omoba-core/src/runtime/native/rules/vocab.rs", "rb"))
  local text = f:read("a"); f:close()
  local function table_of(const)
    local body = assert(text:match("const " .. const .. "[^=]*=%s*&%[(.-)%];"), "vocab.rs: no " .. const)
    local out = {}
    for name, key, unit in body:gmatch('%(%s*"([%w_]+)"%s*,%s*StatKey::([%w_]+)%s*,%s*Unit::([%w_]+)%s*%)') do
      out[name] = { key = key, unit = unit }
    end
    return out
  end
  local states = {}
  local body = assert(text:match("const STATE_EFFECTS[^=]*=%s*&%[(.-)%];"), "vocab.rs: no STATE_EFFECTS")
  for name, key, raw in body:gmatch('%(%s*"([%w_]+)"%s*,%s*StateEffect::Payload%(StatKey::([%w_]+)%s*,%s*(%-?%d+)%)%)') do
    states[name] = { payload = key, raw = math.tointeger(tonumber(raw)) }
  end
  for name, arg in body:gmatch('%(%s*"([%w_]+)"%s*,%s*StateEffect::Buff%(([^%)]+)%)%)') do
    local lit = arg:match('^"([%w_]+)"$')
    -- a path to a host constant (ability_runtime::NO_UNIT_COLLISION): its value is the lowercase name
    states[name] = { buff = lit or arg:match("([%w_]+)$"):lower() }
  end
  -- host states held as their own buff (rules::vocab::state; line B, ability-kit-api.md §6)
  for _, name in ipairs({ "invulnerable", "command_restricted" }) do
    if not states[name] then states[name] = { buff = name } end
  end
  vocab_cache = { stats = table_of("STATS"), absolute = table_of("ABSOLUTE_STATS"), states = states }
  return vocab_cache
end

---------------------------------------------------------------------------
-- the document's owners and values
---------------------------------------------------------------------------
local function model(doc, compiled)
  local owners, statuses, abilities, templates = {}, {}, {}, {}
  local shared = { id = "shared", kind = "shared", values = doc.values or {}, max_level = 1 }
  owners.shared = shared
  local function add(def, kind, parent)
    local o = { id = def.id, kind = kind, def = def, values = def.values or {},
      max_level = def.max_level or (parent and parent.max_level) or 1, parent = parent or shared }
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
  -- state variables: the host key is `<document id>:<var>`, a shared registry name stays itself
  local vars = {}
  local shared_vars = (M.registry().unit_state or {})
  local function declare(t)
    for name, d in pairs(t or {}) do
      vars[name] = { def = d, key = shared_vars[name] and name or (tostring(doc.id):lower() .. ":" .. name) }
    end
  end
  declare(doc.unit_state)
  for _, a in ipairs(doc.abilities or {}) do declare(a.unit_state) end
  for _, it in ipairs(doc.items or {}) do declare(it.unit_state) end
  for _, s in ipairs(doc.statuses or {}) do statuses[s.id] = add(s, "status", shared) end
  -- compiled value rows (raw per rank) by owner id
  local rows = {}
  local function take(id, list) if list then rows[id] = rows[id] or {}; for _, r in ipairs(list) do rows[id][r.name] = r end end end
  for id, block in pairs(compiled.abilities) do
    take(id, block.values)
    for _, s in ipairs(block.statuses or {}) do take(s.id, s.values) end
  end
  for id, block in pairs(compiled.items) do
    take(id, block.values)
    for _, s in ipairs(block.statuses or {}) do take(s.id, s.values) end
  end
  for id, ov in pairs(compiled.owner_values or {}) do take(id, ov.values) end
  return { owners = owners, statuses = statuses, abilities = abilities, templates = templates, entries = entries,
    rows = rows, routines = doc.routines or {}, vars = vars, doc_id = doc.id }
end

-- docs/character_pipeline/schema/v3-shared-registry.lua (names shared between documents)
local registry_cache
function M.registry()
  if registry_cache == nil then
    local here = debug.getinfo(1, "S").source:match("^@(.*)[/\\][^/\\]*$") or "."
    local ok, t = pcall(dofile, here .. "/../schema/v3-shared-registry.lua")
    registry_cache = ok and type(t) == "table" and t or {}
  end
  return registry_cache
end

-- status.triggers the generated code serves: name -> { mask bit (status_event), host kinds }
local STATUS_TRIGGERS = {
  on_created = { 0 }, periodic = { 0 },
  on_expired = { 1, { 0 } }, on_removed = { 1, { 0, 1, 2 } }, on_death = { 1, { 2 } },
  on_attack_hit = { 2, { 3 } }, on_attacked_hit = { 4, { 4 } }, on_attacked = { 4, { 4 } },
  on_take_damage = { 8, { 5 } }, on_deal_damage = { 16, { 6 } },
  -- before the damage lands: AbilityScript::on_damage_pre (omoba-core scripting::damage_pre)
  on_take_damage_pre = { 32, { 7 } }, on_deal_damage_pre = { 64, { 8 } },
}
M.STATUS_TRIGGERS = STATUS_TRIGGERS
-- `periodic`, `periodic_2`, ...: every periodic trigger of a status
local function trigger_kind(name) return name:match("^periodic_%d+$") and STATUS_TRIGGERS.periodic or STATUS_TRIGGERS[name] end
local function starts_on_apply(triggers)
  for k in pairs(triggers or {}) do if k == "on_created" or k:match("^periodic") then return true end end
  return false
end
-- the event mask of a status' triggers (0: none the host calls back)
local function status_mask(def)
  if (def.runtime_id or ""):match("^attack_orb:") then return 0 end
  local mask = 0
  for name in pairs(def.triggers or {}) do
    local t = STATUS_TRIGGERS[name]
    if t and t[1] ~= 0 and mask & t[1] == 0 then mask = mask + t[1] end
  end
  return mask
end
-- a trigger written as its action list, or { filter, actions }
local function trigger_parts(t)
  if t[1] ~= nil or next(t) == nil then return nil, t end
  return t.filter, t.actions or {}
end

local function find(m, owner, qual, name)
  if qual then
    local o = m.owners[qual]
    if o and o.values[name] ~= nil then return o.values[name], o end
    return nil
  end
  local o = owner
  while o do
    if o.values[name] ~= nil then return o.values[name], o end
    o = o.parent
  end
end

-- Fixed64 raw of a pct display raw as a fraction (omoba-core ir.rs pct_raw).
local function pct_raw(raw, truncate)
  if truncate then
    local q = math.abs(raw) // 100
    return raw < 0 and -q or q
  end
  local q = (math.abs(raw) + 50) // 100
  return raw < 0 and -q or q
end

local function emit_values(m)
  local out = { M.HEADER, "//! Every number of the document, per rank (raw Fixed64; rank 1 first).\n" }
  local ids = sorted_keys(m.rows)
  for _, id in ipairs(ids) do
    local o = m.owners[id]
    local names = sorted_keys(m.rows[id])
    local lines = {}
    for _, n in ipairs(names) do
      local row = m.rows[id][n]
      if row.raw and row.per ~= "hero_level" then
        local d = o and o.values[n] or {}
        local raws = {}
        for i, r in ipairs(row.raw) do raws[i] = tostring(math.tointeger(r)) end
        lines[#lines + 1] = string.format("    /// %s%s\n    pub const %s: Ranked<%d> = Ranked([%s]);\n",
          table.concat(row.ranks, " / "), row.unit and (" " .. row.unit) or "", upper(n), #raws, table.concat(raws, ", "))
        if row.unit == "pct" then
          local fr = {}
          for i, r in ipairs(row.raw) do fr[i] = tostring(pct_raw(math.tointeger(r), d.rounding == "truncate")) end
          lines[#lines + 1] = string.format("    /// %s as a fraction\n    pub const %s_FRAC: Ranked<%d> = Ranked([%s]);\n",
            n, upper(n), #fr, table.concat(fr, ", "))
        end
      end
    end
    if #lines > 0 then
      out[#out + 1] = string.format("\n#[allow(dead_code)]\npub mod %s {\n    use ability_kit::Ranked;\n\n%s}\n", id, table.concat(lines, "\n"))
    end
  end
  return table.concat(out)
end

---------------------------------------------------------------------------
-- expressions
---------------------------------------------------------------------------
-- An emitter context: { m, owner, scope, rank, blockers, entry, event (set of event roots), uses (cues, statuses) }
local Gen = {}
Gen.__index = Gen

local function new_gen(m, entry, opts)
  return setmetatable({ m = m, entry = entry, blockers = {}, gaps = {}, opts = opts, routines_needed = {}, cues = {},
    local_n = 0, lets = {} }, Gen)
end
function Gen:block(feature) self.blockers[feature] = true end
-- generated, but the host does not do what the document says (reported per entry, never silent)
function Gen:gap(feature) self.gaps[feature] = true end

local function scope_child(parent) return { parent = parent, names = {} } end
local function scope_find(scope, name)
  while scope do
    if scope.names[name] then return scope.names[name] end
    scope = scope.parent
  end
end
-- a flag set on a scope or one of its parents (`step`: the body of a delay)
local function scope_flag(scope, key)
  while scope do
    if scope[key] ~= nil then return scope[key] end
    scope = scope.parent
  end
end

local CAPTURE = {
  unit = { "ability_kit::arg_unit(%s)", "ability_kit::arg_as_unit(args, %d)" },
  point = { "ability_kit::arg_point(%s)", "ability_kit::arg_as_point(args, %d)" },
  num = { "ability_kit::arg_num(%s)", "ability_kit::arg_as_num(args, %d)" },
  bool = { "ability_kit::arg_num(if %s { Fixed64::ONE } else { Fixed64::ZERO })", "(ability_kit::arg_as_num(args, %d) != Fixed64::ZERO)" },
}
-- A name seen from `ctx`: a binding of the scope chain, or, across the boundary of a delay or
-- repeat step, the value of the name where the step was scheduled (captured into the schedule's
-- arguments, as v3 §12 says: bindings are the values at scheduling time).
function Gen:lookup(ctx, name)
  local sc = ctx.scope
  while sc do
    if sc.names[name] then return sc.names[name] end
    if sc.capture then
      if name == "caster" or name == "event" or name == "none" then return nil end
      local cap = sc.capture
      local code, ty = self:expr({ k = "id", name = name }, { owner = ctx.owner, scope = cap.from, event = cap.event })
      if not CAPTURE[ty] then return nil end
      cap.list[#cap.list + 1] = { code = code, ty = ty }
      local nb = { name = name, ty = ty, marker = string.format(CAPTURE[ty][2], cap.base + #cap.list - 1), used = true }
      sc.names[name] = nb
      return nb
    end
    sc = sc.parent
  end
end
-- A step scope: names from `ctx` are captured; `base` arguments come first (repeat's index).
local function step_scope(ctx, base)
  return { names = {}, capture = { from = ctx.scope, event = ctx.event, list = {}, base = base or 0 } }
end
function Gen:capture_args(sc)
  local out = {}
  for _, c in ipairs(sc.capture.list) do out[#out + 1] = string.format(CAPTURE[c.ty][1], c.code) end
  if #out + sc.capture.base > 8 then self:block("schedule: more than 8 captured values") end
  return out
end
function Gen:add_step(name, body, uses_args)
  self.steps = self.steps or {}
  local step = #self.steps + 1
  self.steps[step] = string.format("/// %s (step %d, host scheduled_effect).\nfn step_%d(cx: &mut Cx<'_, '_>, %s: &[ScheduleArg]) -> Res {\n%s    Ok(())\n}\n",
    name, step, step, uses_args and "args" or "_args", body)
  return step
end
function Gen:reserve_step()
  self.steps = self.steps or {}
  self.steps[#self.steps + 1] = false
  return #self.steps
end
function Gen:set_step(step, name, body, uses_args)
  self.steps[step] = string.format("/// %s (step %d, host scheduled_effect).\nfn step_%d(cx: &mut Cx<'_, '_>, %s: &[ScheduleArg]) -> Res {\n%s    Ok(())\n}\n",
    name, step, step, uses_args and "args" or "_args", body)
end

-- A local binding: its Rust name is decided after the body (`_` prefix when unused).
function Gen:bind(scope, name, ty)
  self.local_n = self.local_n + 1
  local b = { name = name, ty = ty, marker = "\0L" .. self.local_n .. "\0", used = false }
  self.lets[#self.lets + 1] = b
  scope.names[name] = b
  return b.marker
end
function Gen:finish(text)
  for _, b in ipairs(self.lets) do
    local rust = (b.used and "" or "_") .. "l_" .. b.name:gsub("[^%w_]", "_")
    text = text:gsub(b.marker, rust)
  end
  return text
end

local function value_path(o, name, frac)
  return string.format("super::values::%s::%s%s", o.id, upper(name), frac and "_FRAC" or "")
end

local UNIT_ATTRS = {
  attack = { "num", "cx.attack(%s)" }, hp = { "num", "cx.hp(%s)" }, max_hp = { "num", "cx.max_hp(%s)" },
  mana = { "num", "cx.mana(%s)" }, is_building = { "bool", "cx.is_building(%s)" }, is_hero = { "bool", "cx.is_hero(%s)" },
  is_magic_immune = { "bool", "cx.is_magic_immune(%s)" }, position = { "point", "cx.pos(%s)?" },
  alive = { "bool", "cx.alive(%s)" },
  armor = { "num", "cx.armor(%s)" },
  missing_hp = { "num", "(cx.max_hp(%s) - cx.hp(%s))" },
  -- a host channel (ability_channel) holds its public status on the channeler
  is_channeling = { "bool", "cx.has_buff(%s, \"ability_channel\")" },
  hp_pct = { "num", "cx.hp_pct(%s)" }, forward = { "vec", "cx.forward(%s)" },
}

-- -> code, type; on a form it does not lower: records the blocker, returns a typed placeholder
function Gen:expr(node, ctx)
  local k = node.k
  if k == "num" then
    return string.format("Fixed64::from_raw(%d)", C.raw_of(node.text or C.number_text(node.v))), "num"
  elseif k == "bool" then
    return node.v and "true" or "false", "bool"
  elseif k == "val" then
    local d, o = find(self.m, ctx.owner, node.owner, node.name)
    if not d then self:block("value:$" .. node.name .. " not found"); return "Fixed64::ZERO", "err" end
    local row = self.m.rows[o.id] and self.m.rows[o.id][node.name]
    if row and row.raw and row.per ~= "hero_level" then
      return value_path(o, node.name, d.unit == "pct") .. ".at(cx.rank)", "num"
    end
    if d.per == "hero_level" then self:block("value.per:hero_level"); return "Fixed64::ZERO", "err" end
    if d.expr then
      ctx.derived = ctx.derived or {}
      local key = o.id .. "." .. node.name
      if ctx.derived[key] then self:block("value: derived cycle " .. key); return "Fixed64::ZERO", "err" end
      ctx.derived[key] = true
      local ast = X.parse(d.expr)
      local code, ty = self:expr(ast, { owner = o, scope = ctx.scope, event = ctx.event, derived = ctx.derived })
      ctx.derived[key] = nil
      return "(" .. code .. ")", ty
    end
    self:block("value:$" .. node.name .. " has no number"); return "Fixed64::ZERO", "err"
  elseif k == "id" then
    local b = self:lookup(ctx, node.name)
    if b then b.used = true; return b.marker, b.ty end
    local n = node.name
    if n == "caster" then return "cx.caster", "unit"
    elseif n == "self" then return "cx.self_unit", "unit"
    elseif n == "source" then return "cx.caster", "unit"
    elseif n == "target" then
      return "cx.target_unit()", "unit"
    elseif n == "point" then
      return "cx.target_point()?", "point"
    elseif n == "direction" then return "cx.target_direction()?", "vec"
    elseif n == "event" then
      if not ctx.event then self:block("name:event outside an event") end
      return "", "event"
    elseif n == "none" then return "EntityHandle::INVALID", "unit"
    end
    return "", "sym:" .. n
  elseif k == "field" then
    -- rules.<a>.<b>: a rules-layer constant
    local parts, base = {}, node
    while base.k == "field" do table.insert(parts, 1, base.name); base = base.obj end
    if base.k == "id" and base.name == "rules" and not scope_find(ctx.scope, "rules") then
      local t = self.opts.rules
      for _, p in ipairs(parts) do t = type(t) == "table" and t[p] or nil end
      if type(t) ~= "number" then self:block("rules." .. table.concat(parts, ".") .. " has no value"); return "Fixed64::ZERO", "err" end
      return string.format("Fixed64::from_raw(%d) /* rules.%s */", C.raw_of(C.number_text(t)), table.concat(parts, ".")), "num"
    end
    if base.k == "id" and base.name == "state" and #parts == 1 and not scope_find(ctx.scope, "state") then
      local v = self.m.vars[parts[1]]
      if not v then self:block("state:" .. parts[1] .. " not declared"); return "Fixed64::ZERO", "err" end
      local holder = self:expr({ k = "id", name = "self" }, ctx)
      local ty = ({ int = "num", fixed = "num", bool = "bool", unit = "unit" })[v.def.type]
      if not ty then self:block("state type:" .. tostring(v.def.type)); return "Fixed64::ZERO", "err" end
      if ty == "unit" then return string.format("cx.state_unit(%s, %s)", holder, rust_str(v.key)), "unit" end
      local init = v.def.init ~= nil and self:value(v.def.init, ctx, ty) or (ty == "bool" and "false" or "Fixed64::ZERO")
      return string.format("cx.state_%s(%s, %s, %s)", ty, holder, rust_str(v.key), init), ty
    end
    if base.k == "id" and (base.name == "state" or base.name == "vars") and not scope_find(ctx.scope, base.name) then
      self:block("token:" .. base.name); return "Fixed64::ZERO", "err"
    end
    local obj, ot = self:expr(node.obj, ctx)
    if ot == "err" then return obj, "err" end
    if ot == "event_ability" then
      if node.name == "is_toggle" then return "cx.event_is_toggle()", "bool" end
      if node.name == "is_item" then return "cx.event_is_any_item()", "bool" end
      self:block("attr:" .. node.name .. " of an ability"); return "Fixed64::ZERO", "err"
    end
    if ot == "event" then
      if node.name == "target" then return "cx.event_target", "unit"
      elseif node.name == "attacker" then return "cx.event_attacker", "unit"
      elseif node.name == "ability" then return "", "event_ability"
      elseif node.name == "damage" then return "", "event_damage"
      elseif node.name == "victim" then return "cx.event_target", "unit" end
      self:block("event:" .. node.name); return "EntityHandle::INVALID", "unit"
    end
    if ot == "event_damage" then
      if node.name == "final" then return "cx.event_amount", "num"
      -- before it lands the host packet already has armor and resistance (rules::damage ③ runs at
      -- the raw packet's conversion): `raw` / `after_armor` read that amount (conformance checks it)
      elseif ctx.pre and (node.name == "raw" or node.name == "after_armor") then return "cx.event_amount", "num"
      elseif node.name == "is_physical" then return "(cx.event_damage_kind == 0)", "bool"
      elseif node.name == "is_magical" then return "(cx.event_damage_kind == 1)", "bool"
      elseif node.name == "is_pure" then return "(cx.event_damage_kind == 2)", "bool" end
      self:block("event.damage." .. node.name); return "Fixed64::ZERO", "err"
    end
    if ot == "unit" then
      local a = UNIT_ATTRS[node.name]
      if a then return (a[2]:gsub("%%s", function() return obj end)), a[1] end
    end
    self:block("attr:" .. node.name); return "Fixed64::ZERO", "err"
  elseif k == "un" then
    local e, t = self:expr(node.e, ctx)
    if t == "err" then return e, "err" end
    if node.op == "-" then
      if t ~= "num" then self:block("expr: negating a " .. t) end
      return "(-" .. e .. ")", "num"
    end
    if t ~= "bool" then self:block("expr: not of a " .. t) end
    return "(!" .. e .. ")", "bool"
  elseif k == "bin" then
    local l, lt = self:expr(node.l, ctx)
    local r, rt = self:expr(node.r, ctx)
    local op = node.op
    -- a blocked operand was reported already
    if lt == "err" or rt == "err" then return "Fixed64::ZERO", "err" end
    if op == "and" or op == "or" then
      if lt ~= "bool" or rt ~= "bool" then self:block("expr: " .. op .. " of non-booleans") end
      return string.format("(%s %s %s)", l, op == "and" and "&&" or "||", r), "bool"
    end
    if op == "==" or op == "~=" then
      if lt ~= rt or lt:match("^sym") or lt == "event" or lt == "event_ability" then self:block("expr: " .. op .. " of " .. lt .. " and " .. rt) end
      return string.format("(%s %s %s)", l, op == "==" and "==" or "!=", r), "bool"
    end
    if lt ~= "num" or rt ~= "num" then self:block("expr: " .. op .. " of " .. lt .. " and " .. rt); return "Fixed64::ZERO", "err" end
    if op == "<" or op == "<=" or op == ">" or op == ">=" then return string.format("(%s %s %s)", l, op, r), "bool" end
    if op == "+" or op == "-" or op == "*" then return string.format("(%s %s %s)", l, op, r), "num" end
    if op == "/" then return string.format("ability_kit::div(%s, %s)?", l, r), "num" end
    self:block("op:" .. op); return "Fixed64::ZERO", "err"
  elseif k == "call" then
    return self:call(node, ctx)
  end
  self:block("expr:" .. tostring(k)); return "Fixed64::ZERO", "err"
end

function Gen:point(code, ty)
  if ty == "point" or ty == "err" then return code end
  if ty == "unit" then return "cx.pos(" .. code .. ")?" end
  self:block("expr: a position from a " .. ty); return "Vec2::ZERO"
end

function Gen:call(node, ctx)
  local fn, args = node.fn, node.args
  local function arg(i) return self:expr(args[i], ctx) end
  local function name_arg(i)
    local a = args[i]
    if a and a.k == "id" and not scope_find(ctx.scope, a.name) then return a.name end
    self:block("fn:" .. fn .. " with a computed name"); return nil
  end
  if fn == "min" or fn == "max" then
    if #args ~= 2 then self:block("fn:" .. fn .. "/" .. #args) end
    local a, at = arg(1); local b, bt = arg(2)
    if at ~= "num" or bt ~= "num" then self:block("fn:" .. fn .. " of non-numbers") end
    return string.format("%s.%s(%s)", a, fn, b), "num"
  elseif fn == "select" then
    local c, ct = arg(1); local a, at = arg(2); local b, bt = arg(3)
    if at == "err" or bt == "err" or ct == "err" then return "Fixed64::ZERO", "err" end
    if ct ~= "bool" or at ~= bt then self:block("fn:select types") end
    return string.format("(if %s { %s } else { %s })", c, a, b), at
  elseif fn == "floor" or fn == "ceil" or fn == "abs" then
    local x, t = arg(1)
    if t ~= "num" then self:block("fn:" .. fn .. " of a " .. t) end
    return string.format("ability_kit::%s(%s)", fn, x), "num"
  elseif fn == "clamp" then
    local x = self:value_node(args[1], ctx, "num"); local lo = self:value_node(args[2], ctx, "num"); local hi = self:value_node(args[3], ctx, "num")
    return string.format("ability_kit::clamp(%s, %s, %s)", x, lo, hi), "num"
  elseif fn == "exists" then
    local u, t = arg(1)
    if t == "err" then return "false", "bool" end
    if t ~= "unit" then self:block("fn:exists of a " .. t); return "false", "bool" end
    return "cx.exists(" .. u .. ")", "bool"
  elseif fn == "has_state" then
    local u, t = arg(1)
    local st = name_arg(2)
    local e = st and self.opts.vocab.states[st]
    if t ~= "unit" or not e or not e.buff then self:block("fn:has_state:" .. tostring(st)); return "false", "bool" end
    return string.format("cx.has_buff(%s, %s)", u, rust_str(e.buff)), "bool"
  elseif fn == "alive" then
    local u, t = arg(1)
    if t ~= "unit" then self:block("fn:alive of a " .. t) end
    return "cx.alive(" .. u .. ")", "bool"
  elseif fn == "is_enemy" then
    local a = arg(1); local b = arg(2)
    return string.format("cx.is_enemy(%s, %s)", a, b), "bool"
  elseif fn == "distance" then
    local a, at = arg(1); local b, bt = arg(2)
    return string.format("ability_kit::distance(%s, %s)", self:point(a, at), self:point(b, bt)), "num"
  elseif fn == "direction" then
    local a, at = arg(1); local b, bt = arg(2)
    return string.format("ability_kit::direction(%s, %s)", self:point(a, at), self:point(b, bt)), "vec"
  elseif fn == "offset" then
    local p, pt = arg(1); local d, dt = arg(2); local n, nt = arg(3)
    if pt == "err" or dt == "err" or nt == "err" then return "Vec2::ZERO", "err" end
    if dt ~= "vec" or nt ~= "num" then self:block("fn:offset types") end
    return string.format("ability_kit::offset(%s, %s, %s)", self:point(p, pt), d, n), "point"
  elseif fn == "ability_level" then
    local u, ut = arg(1)
    local ab = args[2]
    if ut == "err" then return "Fixed64::ZERO", "err" end
    if ut ~= "unit" or not (ab and ab.k == "id" and not scope_find(ctx.scope, ab.name)) then self:block("fn:ability_level"); return "Fixed64::ZERO", "err" end
    return string.format("{ let u = %s; cx.ability_level(u, %s) }", u, rust_str(ab.name)), "num"
  elseif fn == "status_remaining" then
    local u, ut = arg(1)
    local sname = name_arg(2)
    if ut == "err" then return "Fixed64::ZERO", "err" end
    if ut ~= "unit" or not sname or not self.m.statuses[sname] then self:block("fn:status_remaining"); return "Fixed64::ZERO", "err" end
    return string.format("cx.status_remaining(%s, &super::ids::status::%s)", u, upper(sname)), "num"
  elseif fn == "random" then
    local lo = self:value_node(args[1], ctx, "num"); local hi = self:value_node(args[2], ctx, "num")
    return string.format("{ let lo = %s; let hi = %s; cx.random(lo, hi) }", lo, hi), "num"
  elseif fn == "stacks" then
    local u, ut = arg(1)
    local sname = name_arg(2)
    if ut == "err" then return "Fixed64::ZERO", "err" end
    if ut ~= "unit" or not sname or not self.m.statuses[sname] then self:block("fn:stacks"); return "Fixed64::ZERO", "err" end
    return string.format("cx.stacks(%s, &super::ids::status::%s)", u, upper(sname)), "num"
  elseif fn == "polar" then
    local p, pt = arg(1); local ang = self:value_node(args[2], ctx, "num"); local d = self:value_node(args[3], ctx, "num")
    return string.format("ability_kit::polar(%s, %s, %s)", self:point(p, pt), ang, d), "point"
  elseif fn == "sin" or fn == "cos" then
    return string.format("ability_kit::%s_deg(%s)", fn, self:value_node(args[1], ctx, "num")), "num"
  elseif fn == "random_point" then
    local p, pt = arg(1); local lo = self:value_node(args[2], ctx, "num"); local hi = self:value_node(args[3], ctx, "num")
    return string.format("{ let c = %s; let lo = %s; let hi = %s; cx.random_point(c, lo, hi) }", self:point(p, pt), lo, hi), "point"
  elseif fn == "vec" then
    local x = self:value_node(args[1], ctx, "num"); local y = self:value_node(args[2], ctx, "num")
    return string.format("Vec2::new(%s, %s)", x, y), "vec"
  elseif fn == "lerp_point" then
    local a, at = arg(1); local b2, bt = arg(2); local t = self:value_node(args[3], ctx, "num")
    return string.format("ability_kit::lerp_point(%s, %s, %s)", self:point(a, at), self:point(b2, bt), t), "point"
  elseif fn == "position" then
    local p, pt = arg(1)
    return self:point(p, pt), "point"
  elseif fn == "has_status" then
    local u, ut = arg(1)
    local s = name_arg(2)
    if ut ~= "unit" or not s then return "false", "bool" end
    if not self.m.statuses[s] then self:block("status:" .. s .. " not declared"); return "false", "bool" end
    return string.format("cx.has_status(%s, &super::ids::status::%s)", u, upper(s)), "bool"
  elseif fn == "has_tag" or fn == "is_item" or fn == "is_ability" then
    local _, at = arg(1)
    local n = name_arg(2)
    if at ~= "event_ability" or not n then self:block("fn:" .. fn .. " of a non-event ability"); return "false", "bool" end
    if fn == "is_item" then return "cx.event_is_item(" .. rust_str(n) .. ")", "bool" end
    if fn == "is_ability" then return "cx.event_is_ability(" .. rust_str(n) .. ")", "bool" end
    if n == "item" then return "cx.event_is_any_item()", "bool" end
    if n == "toggle" then return "cx.event_is_toggle()", "bool" end
    self:block("fn:has_tag:" .. n); return "false", "bool"
  end
  self:block("fn:" .. fn); return "Fixed64::ZERO", "err"
end

-- an AST node with a wanted type -> code
function Gen:value_node(n, ctx, want)
  local code, ty = self:expr(n, ctx)
  if ty ~= "err" and ty ~= want then self:block("expr: wanted a " .. want .. ", got a " .. ty) end
  return code
end

-- a value written in the document (string expression, number or boolean) -> code, type
function Gen:value(v, ctx, want)
  local code, ty
  if type(v) == "number" then code, ty = string.format("Fixed64::from_raw(%d)", C.raw_of(C.number_text(v))), "num"
  elseif type(v) == "boolean" then code, ty = v and "true" or "false", "bool"
  elseif type(v) == "string" then
    local ast, err = X.parse(v)
    if not ast then self:block("expr: " .. tostring(err)); return "Fixed64::ZERO", "err" end
    code, ty = self:expr(ast, ctx)
  else
    self:block("expr of type " .. type(v)); return "Fixed64::ZERO", "err"
  end
  if ty == "err" then return code, want or ty end
  if want == "point" then return self:point(code, ty), "point" end
  if want and ty ~= want then self:block("expr: wanted a " .. want .. ", got a " .. ty) end
  return code, ty
end

---------------------------------------------------------------------------
-- statuses
---------------------------------------------------------------------------
local STATUS_FIELDS = set({ "id", "runtime_id", "duration", "stacking", "dispel", "visibility", "reveal", "cue", "remove_on",
  "stats", "per_source", "states", "tags", "triggers", "text", "notes", "values" })

-- The StatusDef constant of every status; problems per status (states the host does not hold).
local function emit_statuses(m, root)
  local voc = M.vocab(root)
  local out = { M.HEADER, "//! The document's statuses as the host BuffStore runs them.\n\n#[allow(dead_code)]\npub mod status {\n    #[allow(unused_imports)]\n    use ability_kit::{StatKey, StatusDef};\n" }
  local problems = {}
  for _, id in ipairs(sorted_keys(m.statuses)) do
    local s = m.statuses[id].def
    local p = {}
    local runtime_id = s.runtime_id or id
    if s.per_source and not runtime_id:find("{caster}", 1, true) then p[#p + 1] = "status.per_source without {caster}" end
    local buffs, payload = {}, {}
    for _, st in ipairs(s.states or {}) do
      local e = voc.states[st]
      if not e then p[#p + 1] = "state:" .. st
      elseif e.buff then buffs[#buffs + 1] = rust_str(e.buff)
      else payload[#payload + 1] = string.format("(StatKey::%s, %d)", e.payload, e.raw) end
    end
    -- triggers: the host calls back the owning ability (status_event); others stay blockers
    local owner, mask = "", status_mask(s)
    if s.triggers and not runtime_id:match("^attack_orb:") then
      for _, name in ipairs(sorted_keys(s.triggers)) do
        if not trigger_kind(name) then p[#p + 1] = "status_trigger:" .. name end
      end
      local o = m.statuses[id].parent
      if not o or (o.kind ~= "ability" and o.kind ~= "item") then p[#p + 1] = "status.triggers on a document-level status"
      else owner = o.id end
    end
    problems[id] = p
    out[#out + 1] = string.format("\n    pub const %s: StatusDef = StatusDef {\n        id: %s,\n        runtime_id: %s,\n        state_buffs: &[%s],\n        state_payload: &[%s],\n        owner: %s,\n        events: %d,\n    };\n",
      upper(id), rust_str(id), rust_str(runtime_id), table.concat(buffs, ", "), table.concat(payload, ", "), rust_str(owner), mask)
  end
  out[#out + 1] = "}\n"
  return table.concat(out), problems
end

-- (StatKey, raw) rows of a stats table -> Rust slice literal
function Gen:stat_rows(stats, ctx)
  local voc = self.opts.vocab
  local rows = {}
  for _, name in ipairs(sorted_keys(stats)) do
    local v = stats[name]
    local entry, expr
    if type(v) == "table" then
      if v.op == "set_absolute" then entry = voc.absolute[name]; expr = v.value
      elseif v.op == nil or v.op == "add" then entry = voc.stats[name]; expr = v.value
      else self:block("stat.op:" .. tostring(v.op)) end
      if v.when ~= nil then self:block("stat.when") end
      if v.per_stack and not self.per_rows then self:block("stat.per_stack") end
    else
      entry = voc.stats[name]; expr = v
    end
    if name == "slow_pct" and type(v) ~= "table" then
      if not self.slow_ok then self:block("stat:slow_pct outside a status") else self.last_slow = self:value(expr, ctx, "num") end
    elseif not entry then self:block("stat:" .. name)
    elseif expr ~= nil then
      local raw
      -- percent points from a plain pct value keep the authored raw (exact, like the handlers)
      local ref = type(expr) == "string" and expr:match("^%s*%$([%w_]+)%s*$")
      if entry.unit == "PercentPoints" and ref then
        local d, o = find(self.m, ctx.owner, nil, ref)
        local row = o and self.m.rows[o.id] and self.m.rows[o.id][ref]
        if d and d.unit == "pct" and row and row.raw then raw = value_path(o, ref, false) .. ".at(cx.rank).raw()" end
      end
      if not raw then
        local code = self:value(expr, ctx, "num")
        raw = entry.unit == "PercentPoints" and ("ability_kit::percent_points(" .. code .. ")") or ("(" .. code .. ").raw()")
      end
      if type(v) == "table" and v.per_stack and self.per_rows then
        self.per_rows[#self.per_rows + 1] = string.format("(StatKey::%s, %s)", entry.key, raw)
      else
        rows[#rows + 1] = string.format("(StatKey::%s, %s)", entry.key, raw)
      end
    end
  end
  return "&[" .. table.concat(rows, ", ") .. "]"
end

---------------------------------------------------------------------------
-- actions
---------------------------------------------------------------------------
local ACTION_FIELDS = {
  apply_status = set({ "op", "status", "to", "duration", "source", "stacks" }),
  -- tags, exclude_tags, exclude_source_tags, source_is, source_ability, min_duration, min_remaining:
  -- the host does not know status tags or sources yet (line B), so they stay blocked
  remove_status = set({ "op", "status", "from", "dispel", "only", "team_of_source", "except", "states", "limit" }),
  damage = set({ "op", "to", "amount", "type", "require_source_alive", "flags", "source" }),
  teleport = set({ "op", "who", "to", "find_clear_space", "max_distance" }),
  add_stacks = set({ "op", "to", "status", "count", "max", "refresh" }),
  set_stacks = set({ "op", "to", "status", "count" }),
  modify_cooldown = set({ "op", "of", "ability", "mode", "amount" }),
  set_ability_level = set({ "op", "of", "ability", "level" }),
  set_health = set({ "op", "to", "value" }),
  heal = set({ "op", "to", "amount", "source" }),
  restore_mana = set({ "op", "to", "amount", "source" }),
  cue = false, -- presentation: any fields
  ["if"] = set({ "op", "cond", "actions", "otherwise" }),
  let = set({ "op", "name", "value" }),
  for_each = set({ "op", "select", "as", "actions" }),
  create_unit = set({ "op", "template", "at", "count", "lifetime", "control", "limit_per_owner", "over_limit", "as", "owner", "team" }),
  reveal = set({ "op", "at", "radius", "duration", "teams", "follow" }),
  call = set({ "op", "routine", "args" }),
  delay = set({ "op", "time", "actions", "capture" }),
  ["repeat"] = set({ "op", "count", "interval", "first", "while", "stop_when", "as", "actions" }),
  modify_damage = set({ "op", "multiplier", "set", "cancel" }),
  chance = set({ "op", "p", "actions", "otherwise" }),
  give_gold = set({ "op", "to", "amount", "reliable", "show" }),
  take_gold = set({ "op", "from", "amount" }),
  burn_mana = set({ "op", "to", "amount" }),
  dash = set({ "op", "who", "to", "direction", "speed", "distance" }),
  dodge_projectiles = set({ "op", "of", "from_team", "kind" }),
  set_state = set({ "op", "var", "value", "on" }),
  projectile = set({ "op", "kind", "from", "to", "direction", "speed", "radius", "distance", "pierce", "max_hits",
    "team", "types", "flags", "dodgeable", "vision", "end_radius", "cue", "on_hit", "as" }),
  clear_state = set({ "op", "var", "on" }),
  stop = set({ "op" }),
  destroy_trees = set({ "op", "at", "radius" }),
  tether = false, -- shape-checked in Gen:tether
}
local SELECTOR_FIELDS = set({ "from", "radius", "shape", "team", "types", "flags", "where", "notes", "limit", "order", "exclude" })
local SELECT_TYPES = { [""] = "All", all = "All", hero = "Heroes", ["creep+hero+neutral+summon"] = "NonBuilding" }
local function types_key(t)
  local names = {}
  for _, n in ipairs(type(t) == "table" and t or {}) do names[#names + 1] = n end
  table.sort(names)
  return table.concat(names, "+")
end

---------------------------------------------------------------------------
-- host component shapes (the IR interpreter's mappings, ir.rs Loader)
---------------------------------------------------------------------------
local function parse(v) return type(v) == "string" and X.parse(v) or nil end
local function is_field(n, obj, name) return n and n.k == "field" and n.name == name and n.obj.k == "id" and n.obj.name == obj end
local function is_id(n, name) return n and n.k == "id" and n.name == name end
local function is_val(n) return n and n.k == "val" end
local function same_val(a, b) return is_val(a) and is_val(b) and a.name == b.name and a.owner == b.owner end
local function bin(n, op) if n and n.k == "bin" and n.op == op then return n.l, n.r end end
local function no_cues(list)
  local out = {}
  for _, a in ipairs(list or {}) do if a.op ~= "cue" then out[#out + 1] = a end end
  return out
end
-- `it.<prop> * $share` (either order) -> the share's value node
local function share_of(v, it, prop)
  local l, r = bin(parse(v), "*")
  if is_field(l, it, prop) and is_val(r) then return r end
  if is_field(r, it, prop) and is_val(l) then return l end
end

-- for_each over allies whose body only restores shares of max HP / mana (ally_area_restore)
function Gen:ally_restore(a, ctx, line)
  local s, it = a.select, a.as or "it"
  local types = types_key(s.types)
  if types ~= "" and types ~= "all" then self:block("selector.team:ally (types " .. types .. ")"); return "" end
  local no_buildings = false
  if s.where ~= nil then
    local w = parse(s.where)
    if w and w.k == "un" and w.op == "not" and is_field(w.e, "it", "is_building") then no_buildings = true
    else self:block("selector.team:ally (where)"); return "" end
  end
  local hp, mana, direct
  for _, e in ipairs(no_cues(a.actions)) do
    if not is_id(parse(e.to), it) then self:block("selector.team:ally (an effect not on " .. it .. ")"); return "" end
    if e.op == "heal" and not hp then hp = share_of(e.amount, it, "max_hp")
      if not hp then self:block("selector.team:ally (heal is not a share of max_hp)"); return "" end
    elseif e.op == "restore_mana" and not mana then mana = share_of(e.amount, it, "max_mana")
      if not mana then self:block("selector.team:ally (restore_mana is not a share of max_mana)"); return "" end
    elseif e.op == "set_health" and not hp then
      -- `min(it.hp + it.max_hp * $x, it.max_hp)`: a direct heal that bypasses healing modifiers
      local v = parse(e.value)
      local sum = v and v.k == "call" and v.fn == "min" and #v.args == 2 and is_field(v.args[2], it, "max_hp") and v.args[1]
      local l, r = bin(sum or nil, "+")
      if is_field(l, it, "hp") and r then
        local ml, mr = bin(r, "*")
        if is_field(ml, it, "max_hp") and is_val(mr) then hp = mr elseif is_field(mr, it, "max_hp") and is_val(ml) then hp = ml end
      end
      if not hp then self:block("selector.team:ally (set_health is not a capped share of max_hp)"); return "" end
      direct = true
    else
      self:block("selector.team:ally (" .. tostring(e.op) .. ")"); return ""
    end
  end
  if not hp and not mana then self:block("selector.team:ally (nothing restored)"); return "" end
  local center, cty = "cx.caster", "unit"
  if s.from ~= nil then center, cty = self:value(s.from, ctx) end
  local radius = self:value(s.radius, ctx, "num")
  local share = function(n) return n and (self:expr(n, ctx)) or "Fixed64::ZERO" end
  local head
  if cty == "unit" then
    head = line("    let center_unit = Some(" .. center .. ");") .. line("    let center = cx.pos(" .. center .. ")?;")
  else
    head = line("    let center_unit = None;") .. line("    let center = " .. self:point(center, cty) .. ";")
  end
  return line("{") .. head .. line("    let radius = " .. radius .. ";")
    .. line("    let hp = " .. share(hp) .. ";") .. line("    let mana = " .. share(mana) .. ";")
    .. line(string.format("    cx.ally_restore(center, center_unit, radius, hp, mana, %s, %s)?;", tostring(direct == true), tostring(no_buildings)))
    .. line("}")
end

-- tether whose on_expire is distance-scaled damage and an interpolated root (host tether)
function Gen:tether(a, ctx, line)
  local function shape(why) self:block("action:tether (" .. why .. ")"); return "" end
  if a.from ~= "caster" then return shape("from is not the caster") end
  for _, k in ipairs(sorted_keys(a)) do
    if k ~= "op" and k ~= "from" and k ~= "to" and k ~= "duration" and k ~= "on_expire" and k ~= "cue" then return shape(k) end
  end
  local fx = no_cues(a.on_expire)
  if #fx ~= 3 or fx[1].op ~= "let" or fx[2].op ~= "damage" or fx[3].op ~= "apply_status" then return shape("on_expire") end
  local r = fx[1].name
  local v = parse(fx[1].value)
  if not (v and v.k == "call" and v.fn == "min" and #v.args == 2 and is_val(v.args[2])) then return shape("distance") end
  local d = v.args[1]
  if not (d.k == "call" and (d.fn == "edge_distance" or d.fn == "distance") and is_id(d.args[1], "caster") and is_id(d.args[2], "target")) then
    return shape("distance")
  end
  local max = v.args[2]
  local dmg = fx[2]
  for _, k in ipairs(sorted_keys(dmg)) do
    if k ~= "op" and k ~= "to" and k ~= "type" and k ~= "amount" and k ~= "require_source_alive" then return shape("damage." .. k) end
  end
  local al, ar = bin(parse(dmg.amount), "/")
  local amount
  do
    local ml, mr = bin(al, "*")
    if is_val(ml) and is_id(mr, r) and same_val(ar, max) then amount = ml end
  end
  if not amount then return shape("damage amount") end
  local kinds = { physical = "Physical", magical = "Magical", pure = "Pure" }
  local kind = kinds[dmg.type]
  if not kind then return shape("damage type") end
  local root = fx[3]
  -- $far + ($near - $far) * ($max - r) / $max
  local far, near
  do
    local pl, pr = bin(parse(root.duration), "+")
    local ql, qr = bin(pr, "/")
    local ml, mr = bin(ql, "*")
    local nl, nr = bin(ml, "-")
    local xl, xr = bin(mr, "-")
    if is_val(pl) and is_val(nl) and same_val(nr, pl) and same_val(xl, max) and is_id(xr, r) and same_val(qr, max) then far, near = pl, nl end
  end
  if not far then return shape("root duration") end
  local so = self.m.statuses[root.status]
  if not so then return shape("root status") end
  local rooted = false
  for _, st in ipairs(so.def.states or {}) do if st == "root" then rooted = true end end
  if not rooted then return shape("the status must root") end
  local root_damage = "Fixed64::ZERO"
  local trig = so.def.triggers and so.def.triggers.on_created
  if trig then
    local list = trig[1] and trig or trig.actions or {}
    if #list ~= 1 or list[1].op ~= "damage" or list[1].type ~= dmg.type or not is_val(parse(list[1].amount)) then return shape("root damage") end
    root_damage = self:value(list[1].amount, { owner = so, scope = ctx.scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast }, "num")
  end
  -- the expire program reads `target` as the tethered unit: only the cast target is admitted
  if a.to ~= "target" then return shape("to is not the cast target") end
  local to = self:value(a.to, ctx, "unit")
  local dur = self:value(a.duration, ctx, "num")
  if a.cue then self.cues[a.cue] = true end
  return line("{") .. line("    let to = " .. to .. ";") .. line("    let duration = " .. dur .. ";")
    .. line("    let link = Tether {")
    .. line("        amount: " .. self:expr(amount, ctx) .. ",")
    .. line("        kind: DamageKind::" .. kind .. ",")
    .. line("        range: " .. self:expr(max, ctx) .. ",")
    .. line("        root: " .. self:expr(near, { owner = ctx.owner, scope = ctx.scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast }) .. ",")
    .. line("        root_far: " .. self:expr(far, ctx) .. ",")
    .. line("        root_damage: " .. root_damage .. ",")
    .. line("        edge: " .. tostring(d.fn == "edge_distance") .. ",")
    .. line("        root_after_source_death: " .. tostring(dmg.require_source_alive ~= false) .. ",")
    .. line("    };")
    .. line("    cx.tether(to, " .. rust_str(self.entry.def.id .. ":{caster}") .. ", duration, link)?;")
    .. line("}")
end

-- list table -> its v3 effect path ("events.on_cast", "statuses[1].triggers.on_created", ...):
-- every array of actions under `node`, the path written the way line B's damage trace reports it
local function is_action_list(t) return type(t) == "table" and type(t[1]) == "table" and t[1].op ~= nil end
local function effect_paths(node, path, out)
  out = out or {}
  if type(node) ~= "table" then return out end
  if is_action_list(node) then
    out[node] = path
    for i, a in ipairs(node) do effect_paths(a, path .. "[" .. i .. "]", out) end
    return out
  end
  local array = node[1] ~= nil
  for k, v in pairs(node) do
    if type(v) == "table" then
      local p = array and type(k) == "number" and (path .. "[" .. k .. "]") or ((path ~= "" and (path .. ".") or "") .. tostring(k))
      effect_paths(v, p, out)
    end
  end
  return out
end
M.effect_paths = effect_paths

function Gen:actions(list, ctx, indent)
  local out = {}
  local base = self.paths and list and self.paths[list]
  for i, a in ipairs(list or {}) do
    local prev = self.cur_path
    self.cur_path = base and (base .. "[" .. i .. "]") or "?"
    out[#out + 1] = self:action(a, ctx, indent)
    self.cur_path = prev
  end
  return table.concat(out)
end

-- apply_status / add_stacks / set_stacks of a status with `stacking = { max, mode, value_stacks }`
function Gen:counted_status(a, ctx, line)
  local so = self.m.statuses[a.status]
  local s = so.def
  for _, k in ipairs(sorted_keys(s)) do if not STATUS_FIELDS[k] then self:block("status." .. k) end end
  local st = s.stacking
  for _, k in ipairs(sorted_keys(st)) do
    if k ~= "max" and k ~= "mode" and k ~= "value_stacks" then self:block("status.stacking." .. k) end
  end
  if st.mode ~= nil and st.mode ~= "refresh_on_add" and st.mode ~= "no_refresh" then self:block("status.stacking.mode:" .. tostring(st.mode)) end
  for _, p in ipairs(self.status_problems[a.status] or {}) do self:block(p) end
  local starts = starts_on_apply(s.triggers)
  local sctx = { owner = so, scope = ctx.scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast }
  local to = self:value(a.to, ctx, "unit")
  local dur
  if a.duration ~= nil then dur = self:value(a.duration, ctx, "num")
  elseif s.duration == nil or s.duration == "infinite" then dur = "Cx::FOREVER"
  else dur = self:value(s.duration, sctx, "num") end
  self.slow_ok, self.last_slow = true, nil
  self.per_rows = st.value_stacks and {} or nil
  local flat = s.stats and self:stat_rows(s.stats, sctx) or "&[]"
  local per = self.per_rows and ("&[" .. table.concat(self.per_rows, ", ") .. "]") or "&[]"
  local slow = self.last_slow
  self.slow_ok, self.last_slow, self.per_rows = nil, nil, nil
  local S = "&super::ids::status::" .. upper(a.status)
  local max = (a.max ~= nil and self:value(a.max, ctx, "num")) or (st.max ~= nil and self:value(st.max, sctx, "num")) or nil
  local count, keep
  if a.op == "set_stacks" then count = "(" .. self:value(a.count, ctx, "num") .. ")"; keep = true
  elseif a.op == "add_stacks" then count = "(cx.stacks(to, " .. S .. ") + " .. self:value(a.count, ctx, "num") .. ")"; keep = a.refresh ~= true
  else
    count = "(cx.stacks(to, " .. S .. ") + " .. (a.stacks ~= nil and self:value(a.stacks, ctx, "num") or "Fixed64::ONE") .. ")"
    keep = st.mode == "no_refresh"
  end
  if max then count = count .. ".min(" .. max .. ")" end
  local out = line("{") .. line("    let to = " .. to .. ";") .. line("    let duration = " .. dur .. ";")
    .. ((keep or starts) and line("    let existed = cx.has_status(to, " .. S .. ");") or "")
    .. line("    let stacks = " .. count .. ";")
    .. line(string.format("    cx.put_stacks(to, %s, %s, duration, stacks, %s, %s, %s)?;", S,
      keep and "if existed { None } else { Some(duration) }" or "Some(duration)", flat, per, slow and ("Some(" .. slow .. ")") or "None"))
  if starts then
    out = out .. line(string.format("    if !existed && cx.has_status(to, %s) { super::%s::st_%s_start(cx, to)?; }", S, so.parent.id, a.status))
  end
  return out .. line("}")
end

function Gen:action(a, ctx, ind)
  local op = a.op
  local allowed = ACTION_FIELDS[op]
  if allowed == nil then self:block("action:" .. tostring(op)); return "" end
  if allowed then
    for _, k in ipairs(sorted_keys(a)) do if not allowed[k] and k ~= "notes" then self:block("action:" .. op .. "." .. k) end end
  end
  local I = ind
  local function line(s) return I .. s .. "\n" end
  local counted = (op == "apply_status" or op == "add_stacks" or op == "set_stacks") and self.m.statuses[a.status]
    and type(self.m.statuses[a.status].def.stacking) == "table"
  if counted then return self:counted_status(a, ctx, line) end
  if op == "add_stacks" or op == "set_stacks" then self:block("action:" .. op .. " on a status without counted stacks"); return "" end
  if op == "apply_status" and a.stacks ~= nil then self:block("action:apply_status.stacks on a status without counted stacks"); return "" end
  if op == "apply_status" then
    local so = self.m.statuses[a.status]
    if not so then self:block("status:" .. tostring(a.status) .. " not declared"); return "" end
    local s = so.def
    for _, k in ipairs(sorted_keys(s)) do if not STATUS_FIELDS[k] then self:block("status." .. k) end end
    if s.stacking ~= "refresh" and s.stacking ~= "keep" then
      self:block("status.stacking:" .. (type(s.stacking) == "table" and (s.stacking.mode or "stacks") or tostring(s.stacking)))
    end
    for _, p in ipairs(self.status_problems[a.status] or {}) do self:block(p) end
    local starts = not (s.runtime_id or ""):match("^attack_orb:") and starts_on_apply(s.triggers)
    local sctx = { owner = so, scope = ctx.scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast }
    local to = self:value(a.to, ctx, "unit")
    local dur
    if a.duration ~= nil then dur = self:value(a.duration, ctx, "num")
    elseif s.duration == nil or s.duration == "infinite" then dur = "Cx::FOREVER"
    else dur = self:value(s.duration, sctx, "num") end
    self.slow_ok, self.last_slow = true, nil
    local stats = s.stats and self:stat_rows(s.stats, sctx) or "&[]"
    local slow = self.last_slow
    self.slow_ok, self.last_slow = nil, nil
    if slow or a.source ~= nil then
      local src = a.source ~= nil and self:value(a.source, ctx, "unit") or "cx.caster"
      local call = string.format("    cx.apply_status_from(source, to, &super::ids::status::%s, duration, %s, %s)?;", upper(a.status), stats,
        slow and ("Some(" .. slow .. ")") or "None")
      local s2 = line("{") .. line("    let to = " .. to .. ";") .. line("    let duration = " .. dur .. ";") .. line("    let source = " .. src .. ";")
      if starts then
        s2 = s2 .. line("    let existed = cx.has_status(to, &super::ids::status::" .. upper(a.status) .. ");") .. line(call)
          .. line("    if !existed {") .. line("        let caster = cx.caster;") .. line("        cx.caster = source;")
          .. line(string.format("        let r = super::%s::st_%s_start(cx, to);", so.parent.id, a.status))
          .. line("        cx.caster = caster;") .. line("        r?;") .. line("    }")
      else
        s2 = s2 .. line(call)
      end
      return s2 .. line("}")
    end
    if starts then
      return line("{") .. line("    let to = " .. to .. ";") .. line("    let duration = " .. dur .. ";")
        .. line("    let existed = cx.has_status(to, &super::ids::status::" .. upper(a.status) .. ");")
        .. line("    cx.apply_status(to, &super::ids::status::" .. upper(a.status) .. ", duration, " .. stats .. ")?;")
        .. line(string.format("    if !existed { super::%s::st_%s_start(cx, to)?; }", so.parent.id, a.status)) .. line("}")
    end
    if s.stacking == "keep" then
      return line("{") .. line("    let to = " .. to .. ";") .. line("    let duration = " .. dur .. ";")
        .. line("    if !cx.has_status(to, &super::ids::status::" .. upper(a.status) .. ") {")
        .. line("        cx.apply_status(to, &super::ids::status::" .. upper(a.status) .. ", duration, " .. stats .. ")?;") .. line("    }") .. line("}")
    end
    return line("{") .. line("    let to = " .. to .. ";") .. line("    let duration = " .. dur .. ";")
      .. line("    cx.apply_status(to, &super::ids::status::" .. upper(a.status) .. ", duration, " .. stats .. ")?;") .. line("}")
  elseif op == "remove_status" and a.status == nil then
    -- by filters: the host purges what they name (dispel::purge; controls follow their statuses)
    local from = self:value(a.from, ctx, "unit")
    local function opt_str(v) return v ~= nil and ("Some(" .. rust_str(v) .. ")") or "None" end
    if a.dispel == nil and a.states == nil and a.tags == nil then self:block("action:remove_status without status, dispel or states") end
    local except = {}
    for _, id in ipairs(a.except or {}) do
      local so = self.m.statuses[id]
      if not so then self:block("action:remove_status.except:" .. tostring(id) .. " not declared")
      else
        -- the status's row key: its runtime id up to the per-source part (`<id>:{caster}`)
        local rid = so.def.runtime_id or so.def.id
        except[#except + 1] = rust_str(rid:match("^(.-):{") or rid)
      end
    end
    local states = {}
    for _, name in ipairs(a.states or {}) do
      local e = self.opts.vocab.states[name]
      local buff = e and e.buff
      if buff == "stun" or buff == "root" or buff == "silence" then states[#states + 1] = rust_str(buff)
      else self:block("action:remove_status.states:" .. tostring(name)) end
    end
    local limit = a.limit ~= nil and ("Some(" .. self:value(a.limit, ctx, "num") .. ")") or "None"
    return line("{") .. line("    let from = " .. from .. ";")
      .. line("    cx.remove_statuses(from, ability_kit::StatusFilter { dispel: " .. opt_str(a.dispel) .. ", only: " .. opt_str(a.only)
        .. ", team_of_source: " .. rust_str(a.team_of_source or "any") .. ", except: &[" .. table.concat(except, ", ")
        .. "], states: &[" .. table.concat(states, ", ") .. "], limit: " .. limit .. " })?;") .. line("}")
  elseif op == "remove_status" then
    if not self.m.statuses[a.status] then self:block("action:remove_status without a declared status"); return "" end
    for _, k in ipairs({ "dispel", "only", "team_of_source", "except", "states", "limit" }) do
      if a[k] ~= nil then self:block("action:remove_status.status with " .. k) end
    end
    local from = self:value(a.from, ctx, "unit")
    return line("{") .. line("    let from = " .. from .. ";") .. line("    cx.remove_status(from, &super::ids::status::" .. upper(a.status) .. ")?;") .. line("}")
  elseif op == "damage" then
    local kinds = { physical = "Physical", magical = "Magical", pure = "Pure" }
    local kind = kinds[a.type]
    if not kind then self:block("action:damage.type:" .. tostring(a.type)); kind = "Pure" end
    local no_amp = false
    for _, f in ipairs(a.flags or {}) do
      if f == "no_amplify" then no_amp = true else self:block("action:damage.flags:" .. f) end
    end
    local to = self:value(a.to, ctx, "unit")
    local amount = self:value(a.amount, ctx, "num")
    local who = self.entry and self.entry.def.id or self.trace_owner or "?"
    if a.source ~= nil and a.source ~= "self" then
      local src = self:value(a.source, ctx, "unit")
      return line("{") .. line("    let to = " .. to .. ";") .. line("    let amount = " .. amount .. ";") .. line("    let source = " .. src .. ";")
        .. line(string.format("    cx.damage_from_at(%s, %s, source, to, amount, DamageKind::%s, Dmg { require_source_alive: %s, no_amplify: %s })?;",
          rust_str(who), rust_str(self.cur_path or "?"), kind, a.require_source_alive ~= false and "true" or "false", no_amp and "true" or "false")) .. line("}")
    end
    return line("{") .. line("    let to = " .. to .. ";") .. line("    let amount = " .. amount .. ";")
      .. line(string.format("    cx.damage_at(%s, %s, to, amount, DamageKind::%%s, Dmg { require_source_alive: %%s, no_amplify: %%s })?;",
        rust_str(who), rust_str(self.cur_path or "?")):format(
        kind, a.require_source_alive ~= false and "true" or "false", no_amp and "true" or "false")) .. line("}")
  elseif op == "heal" or op == "restore_mana" then
    -- the host heals with the caster as the healer: `self` is the caster in a passive or a cast
    if a.source ~= nil and a.source ~= "caster" and not (a.source == "self" and self.entry) then
      self:block("action:" .. op .. ".source:" .. tostring(a.source))
    end
    local to = self:value(a.to, ctx, "unit")
    local amount = self:value(a.amount, ctx, "num")
    return line("{") .. line("    let to = " .. to .. ";") .. line("    let amount = " .. amount .. ";")
      .. line("    cx." .. op .. "(to, amount)?;") .. line("}")
  elseif op == "cue" then
    self.cues[a.id] = true
    -- presentation never fails a program: a unit without a position shows nothing
    local at, at_ty = "cx.caster", "unit"
    if a.at ~= nil then at, at_ty = self:value(a.at, ctx) end
    if at_ty ~= "unit" and at_ty ~= "point" then self:block("action:cue.at of a " .. at_ty) end
    local radius = a.radius ~= nil and ("Some(" .. self:value(a.radius, ctx, "num") .. ")") or "None"
    return line("{") .. line("    let at = " .. at .. ";") .. line("    let radius = " .. radius .. ";")
      .. line("    cx." .. (at_ty == "unit" and "cue_on" or "cue") .. "(super::ids::Cue::" .. camel(a.id) .. ".as_str(), at, radius);") .. line("}")
  elseif op == "if" then
    local cond = self:value(a.cond, ctx, "bool")
    local s = line("if " .. cond .. " {") .. self:actions(a.actions, { owner = ctx.owner, scope = scope_child(ctx.scope), event = ctx.event, pre = ctx.pre, cast = ctx.cast }, I .. "    ")
    if a.otherwise then
      s = s .. line("} else {") .. self:actions(a.otherwise, { owner = ctx.owner, scope = scope_child(ctx.scope), event = ctx.event, pre = ctx.pre, cast = ctx.cast }, I .. "    ")
    end
    return s .. line("}")
  elseif op == "let" then
    local code, ty = self:value(a.value, ctx)
    if ty:match("^sym") or ty == "event" or ty == "event_ability" then self:block("action:let of a " .. ty); return "" end
    local marker = self:bind(ctx.scope, a.name, ty)
    return line("let " .. marker .. " = " .. code .. ";")
  elseif op == "tether" then
    if not self.entry then self:block("action:tether in a routine"); return "" end
    return self:tether(a, ctx, line)
  elseif op == "for_each" then
    local s = a.select
    if s.team == "ally" and (s.shape or "circle") == "circle" then return self:ally_restore(a, ctx, line) end
    for _, k in ipairs(sorted_keys(s)) do if not SELECTOR_FIELDS[k] then self:block("selector." .. k) end end
    if (s.shape or "circle") ~= "circle" then self:block("selector.shape:" .. s.shape) end
    if s.team ~= "enemy" then self:block("selector.team:" .. tostring(s.team)) end
    local types = SELECT_TYPES[types_key(s.types)]
    if not types then self:block("selector.types:" .. types_key(s.types)); types = "All" end
    local mi, inv = false, false
    for _, f in ipairs(s.flags or {}) do
      if f == "include_magic_immune" then mi = true elseif f == "include_invulnerable" then inv = true
      -- visible_only: enemies_in_circle already takes the caster team's disclosed units
      elseif f ~= "include_untargetable" and f ~= "visible_only" then self:block("selector.flag:" .. f) end
    end
    if s.order ~= nil and s.order ~= "id" and s.order ~= "nearest" then self:block("selector.order:" .. tostring(s.order)) end
    local exclude = s.exclude ~= nil and self:value(s.exclude, ctx, "unit") or nil
    local from = s.from ~= nil and self:value(s.from, ctx, "point") or "cx.pos(cx.caster)?"
    local radius = self:value(s.radius, ctx, "num")
    local limit = s.limit ~= nil and self:value(s.limit, ctx, "num") or nil
    local body_scope = scope_child(ctx.scope)
    local it = self:bind(body_scope, a.as or "it", "unit")
    local nearest = s.order == "nearest"
    local str = line("{") .. line("    let center = " .. from .. ";") .. line("    let radius = " .. radius .. ";")
      .. (exclude and line("    let excluded = " .. exclude .. ";") or "")
      .. line(string.format("    let %sunits = cx.enemies_in_circle(center, radius, Select { types: UnitTypes::%s, include_magic_immune: %s, include_invulnerable: %s });",
        nearest and "mut " or "", types, mi and "true" or "false", inv and "true" or "false"))
      .. (nearest and line("    cx.nearest_first(&mut units, center);") or "")
      .. (limit and (line("    let limit = usize::try_from(ability_kit::floor(" .. limit .. ").raw() >> 10).unwrap_or(0);")
        .. line("    let mut taken = 0usize;")) or "")
      .. line("    for " .. it .. " in units {")
      .. (exclude and line("        if " .. it .. " == excluded { continue; }") or "")
    if s.where then
      -- the selector's `where` names the candidate `it`
      local wscope = scope_child(ctx.scope)
      wscope.names.it = body_scope.names[a.as or "it"]
      local w = self:value(s.where, { owner = ctx.owner, scope = wscope, event = ctx.event, pre = ctx.pre, cast = ctx.cast }, "bool")
      str = str .. line("        if !(" .. w .. ") { continue; }")
    end
    -- `limit` counts the units that pass `where`, in id order
    if limit then str = str .. line("        if taken >= limit { break; }") .. line("        taken += 1;") end
    str = str .. self:actions(a.actions, { owner = ctx.owner, scope = body_scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast }, I .. "        ")
    return str .. line("    }") .. line("}")
  elseif op == "create_unit" then
    local t = self.m.templates[a.template]
    if not t then self:block("template:" .. tostring(a.template) .. " not declared") end
    if a.control ~= nil and a.control ~= "ai" then self:block("action:create_unit.control:" .. tostring(a.control)) end
    -- limit_per_owner: only the host's summon_owner_cap (omoba-core summon_owner_cap: after the
    -- cast commits, the earliest live summons of the cast's templates beyond the cap leave). That
    -- needs the cap to be the content extra `summon_owner_cap` of this very ability, `remove_oldest`,
    -- and the summon inside the cast itself.
    local host_cap = false
    if a.limit_per_owner ~= nil or a.over_limit ~= nil then
      local ref = type(a.limit_per_owner) == "string" and a.limit_per_owner:match("^%s*%$([%w_]+)%s*$")
      local d, owner
      if ref then d, owner = find(self.m, ctx.owner, nil, ref) end
      host_cap = a.over_limit == "remove_oldest" and d and d.v2_key == "summon_owner_cap" and owner == self.entry and ctx.cast
      if not host_cap then self:block("action:create_unit.limit_per_owner (no host cap for it)") end
    end
    -- the host summons for the caster, on its team
    local mine = function(v) return v == nil or v == "caster" or (v == "self" and self.entry) end
    if not mine(a.owner) then self:block("action:create_unit.owner:" .. tostring(a.owner)) end
    if not mine(a.team) then self:block("action:create_unit.team:" .. tostring(a.team)) end
    local at = self:value(a.at, ctx, "point")
    local life = a.lifetime ~= nil and self:value(a.lifetime, ctx, "num") or "Fixed64::ZERO"
    local s = line("{")
    if a.count ~= nil then
      s = s .. line("    let count = " .. self:value(a.count, ctx, "num") .. ";")
        .. line('    if count != Fixed64::ONE { return Err("v3: create_unit makes one unit at a time"); }')
    end
    if host_cap then s = s .. line("    // limit_per_owner: the host's summon_owner_cap of this ability retires the earliest beyond it after the cast commits") end
    s = s .. line("    let at = " .. at .. ";") .. line("    let lifetime = " .. life .. ";")
    if a.as then
      local marker = self:bind(ctx.scope, a.as, "unit")
      -- the binding lives after the block: hoist it
      return line("let " .. marker .. " = {") .. s:gsub("^" .. I:gsub("%p", "%%%0") .. "{\n", "")
        .. line("    cx.create_unit(" .. rust_str(a.template) .. ", at, lifetime)?") .. line("};")
    end
    return s .. line("    cx.create_unit(" .. rust_str(a.template) .. ", at, lifetime)?;") .. line("}")
  elseif op == "reveal" then
    if a.teams ~= "own" and a.teams ~= "all" then self:block("action:reveal.teams:" .. tostring(a.teams)) end
    local at = self:value(a.at, ctx, "point")
    local radius = self:value(a.radius, ctx, "num")
    local dur = self:value(a.duration, ctx, "num")
    local follow = a.follow ~= nil and ("Some(" .. self:value(a.follow, ctx, "unit") .. ")") or "None"
    return line("{") .. line("    let at = " .. at .. ";") .. line("    let radius = " .. radius .. ";")
      .. line("    let duration = " .. dur .. ";") .. line("    let follow = " .. follow .. ";")
      .. line(string.format("    cx.reveal(at, radius, duration, %s, follow)?;", a.teams == "all" and "true" or "false")) .. line("}")
  elseif op == "call" then
    local r = self.m.routines[a.routine]
    if not r then self:block("routine:" .. tostring(a.routine) .. " not declared"); return "" end
    self.routines_needed[a.routine] = true
    local args = {}
    for _, p in ipairs(sorted_keys(r.params or {})) do
      local v = (a.args or {})[p]
      if v == nil then self:block("routine " .. a.routine .. ": missing arg " .. p); args[#args + 1] = "Default::default()"
      else
        local kind = r.params[p]
        local want = (kind == "unit" or kind == "point" or kind == "bool") and kind or (kind == "vec" and "vec" or "num")
        args[#args + 1] = (self:value(v, ctx, want))
      end
    end
    return line("super::routines::" .. a.routine .. "(cx" .. (#args > 0 and ", " or "") .. table.concat(args, ", ") .. ")?;")
  elseif op == "delay" then
    if not self.entry then self:block("action:delay in a routine"); return "" end
    -- `capture` only names what is captured anyway: every name the body reads is
    local sc = step_scope(ctx, 0)
    for _, c in ipairs(a.capture or {}) do self:lookup({ owner = ctx.owner, scope = sc }, c) end
    local step = self:reserve_step()
    local body = self:actions(a.actions, { owner = ctx.owner, scope = sc, event = false }, "    ")
    local args = self:capture_args(sc)
    self:set_step(step, "`delay`", body, #args > 0)
    local time = self:value(a.time, ctx, "num")
    return line("{") .. line("    let time = " .. time .. ";")
      .. line("    let args = [" .. table.concat(args, ", ") .. "];")
      .. line("    let owner = cx.caster;")
      .. line(string.format("    cx.schedule(owner, ID, %d, time, &args)?;", step)) .. line("}")
  elseif op == "repeat" then
    if not self.entry then self:block("action:repeat in a routine"); return "" end
    if a.count == nil and a["while"] == nil and a.stop_when == nil then self:block("action:repeat without an end"); return "" end
    if a.interval == nil then
      -- every run within this step (v3 §12.11), bounded
      local body_scope = scope_child(ctx.scope)
      local i = self:bind(body_scope, a.as or "repeat_index", "num")
      local bctx = { owner = ctx.owner, scope = body_scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast }
      local s2 = line("{")
      s2 = s2 .. line("    let count = " .. (a.count ~= nil and ("usize::try_from(ability_kit::floor(" .. self:value(a.count, ctx, "num") .. ").raw() >> 10).unwrap_or(0).min(ability_kit::MAX_REPEAT)") or "ability_kit::MAX_REPEAT") .. ";")
      s2 = s2 .. line("    for index in 0..count {")
      s2 = s2 .. line("        let " .. i .. " = Fixed64::from_i32(index as i32);")
      if a["while"] ~= nil then s2 = s2 .. line("        if !(" .. self:value(a["while"], bctx, "bool") .. ") { break; }") end
      if a.stop_when ~= nil then s2 = s2 .. line("        if " .. self:value(a.stop_when, bctx, "bool") .. " { break; }") end
      s2 = s2 .. self:actions(a.actions, bctx, I .. "        ")
      return s2 .. line("    }") .. line("}")
    end
    -- one scheduled step per run: argument 0 is the run index, then the captures
    local sc = step_scope(ctx, 1)
    local i = self:bind(sc, a.as or "repeat_index", "num")
    sc.names[a.as or "repeat_index"].used = true
    local sctx = { owner = ctx.owner, scope = sc, event = false }
    local step = self:reserve_step()
    local function sline(x) return "    " .. x .. "\n" end
    local b = sline("let " .. i .. " = ability_kit::arg_as_num(args, 0);")
    if a.count ~= nil then b = b .. sline("if " .. i .. " >= " .. self:value(a.count, sctx, "num") .. " { return Ok(()); }") end
    if a["while"] ~= nil then b = b .. sline("if !(" .. self:value(a["while"], sctx, "bool") .. ") { return Ok(()); }") end
    if a.stop_when ~= nil then b = b .. sline("if " .. self:value(a.stop_when, sctx, "bool") .. " { return Ok(()); }") end
    -- the next run first (the body may `stop`), with this run's captures
    b = b .. sline("{") .. sline("    let mut next = args.to_vec();")
      .. sline("    next[0] = ability_kit::arg_num(" .. i .. " + Fixed64::ONE);")
      .. sline("    let interval = " .. self:value(a.interval, sctx, "num") .. ";")
      .. sline("    let owner = cx.caster;")
      .. sline(string.format("    cx.schedule(owner, ID, %d, interval, &next)?;", step)) .. sline("}")
    local body = b .. self:actions(a.actions, sctx, "    ")
    local args = self:capture_args(sc)
    self:set_step(step, "`repeat`", body, true)
    local first = a.first ~= nil and self:value(a.first, ctx, "num") or nil
    local out = line("{") .. line("    let args = [ability_kit::arg_num(Fixed64::ZERO)" .. (#args > 0 and ", " or "") .. table.concat(args, ", ") .. "];")
    if first then
      out = out .. line("    let first = " .. first .. ";") .. line("    let owner = cx.caster;")
        .. line(string.format("    cx.schedule(owner, ID, %d, first, &args)?;", step))
    else
      out = out .. line(string.format("    step_%d(cx, &args)?;", step))
    end
    return out .. line("}")
  elseif op == "modify_cooldown" or op == "set_ability_level" then
    -- the ability: a literal id of this document, or `ability` (this one)
    local ab = a.ability
    local abcode
    if ab == nil or ab == "ability" then abcode = "ID"
    elseif type(ab) == "string" and ab:match("^[%w_]+$") and self.m.abilities[ab] then abcode = rust_str(ab)
    else self:block("action:" .. op .. ".ability:" .. tostring(ab)); return "" end
    local of = self:value(a.of or "self", ctx, "unit")
    if op == "set_ability_level" then
      local lv = self:value(a.level, ctx, "num")
      return line("{") .. line("    let of = " .. of .. ";") .. line("    let level = " .. lv .. ";")
        .. line("    cx.set_ability_level(of, " .. abcode .. ", level)?;") .. line("}")
    end
    local mode = a.mode or "start"
    if mode == "end" then return line("{") .. line("    let of = " .. of .. ";") .. line("    cx.cooldown_end(of, " .. abcode .. ")?;") .. line("}") end
    if mode ~= "start" and mode ~= "set" and mode ~= "reduce" then self:block("action:modify_cooldown.mode:" .. tostring(mode)); return "" end
    local amount_expr = a.amount
    if amount_expr == nil then
      -- `start` / `set` with no amount: the ability's own `cooldown` value (its full cooldown)
      local target_id = abcode == "ID" and (self.entry and self.entry.def.id) or ab
      local tgt = target_id and self.m.owners[target_id]
      if mode ~= "reduce" and tgt and tgt.values.cooldown ~= nil then amount_expr = "$" .. target_id .. ".cooldown"
      else self:block("action:modify_cooldown without amount"); return "" end
    end
    local amount = self:value(amount_expr, ctx, "num")
    return line("{") .. line("    let of = " .. of .. ";") .. line("    let amount = " .. amount .. ";")
      .. line(string.format("    cx.cooldown_%s(of, %s, amount)?;", mode == "reduce" and "reduce" or "start", abcode)) .. line("}")
  elseif op == "set_health" then
    local to = self:value(a.to, ctx, "unit")
    local value = self:value(a.value, ctx, "num")
    return line("{") .. line("    let to = " .. to .. ";") .. line("    let value = " .. value .. ";") .. line("    cx.set_health(to, value)?;") .. line("}")
  elseif op == "teleport" and a.find_clear_space == true then
    if a.who ~= "caster" and not (a.who == "self" and self.entry) then self:block("action:teleport.find_clear_space of " .. tostring(a.who)); return "" end
    local to = self:value(a.to, ctx, "point")
    local range = a.max_distance ~= nil and self:value(a.max_distance, ctx, "num") or nil
    return line("{") .. line("    let to = " .. to .. ";")
      .. line("    let range = " .. (range or "ability_kit::distance(cx.pos(cx.caster)?, to) + Fixed64::ONE") .. ";")
      .. line("    cx.teleport_clear(to, to, range)?;") .. line("}")
  elseif op == "teleport" then
    local who = self:value(a.who, ctx, "unit")
    local to = self:value(a.to, ctx, "point")
    local max = a.max_distance ~= nil and ("Some(" .. self:value(a.max_distance, ctx, "num") .. ")") or "None"
    return line("{") .. line("    let who = " .. who .. ";") .. line("    let to = " .. to .. ";") .. line("    let max = " .. max .. ";")
      .. line("    cx.teleport(who, to, max)?;") .. line("}")
  elseif op == "give_gold" or op == "take_gold" or op == "burn_mana" then
    -- host_ext (line B): gold changes only the one gold pool; burn_mana only takes mana
    local who = self:value(op == "take_gold" and a.from or a.to, ctx, "unit")
    local amount = self:value(a.amount, ctx, "num")
    return line("{") .. line("    let who = " .. who .. ";") .. line("    let amount = " .. amount .. ";")
      .. line(string.format("    cx.%s(who, amount)?;", op)) .. line("}")
  elseif op == "dash" then
    -- host forced_motion Dash: the caster to a point at `speed`
    if a.who ~= "caster" and not (a.who == "self" and self.entry) then self:block("action:dash.who:" .. tostring(a.who)); return "" end
    local to
    if a.to ~= nil then to = self:value(a.to, ctx, "point")
    elseif a.direction ~= nil and a.distance ~= nil then
      to = "ability_kit::offset(cx.pos(cx.caster)?, " .. self:value(a.direction, ctx, "vec") .. ", " .. self:value(a.distance, ctx, "num") .. ")"
    else self:block("action:dash without a destination"); return "" end
    local speed = self:value(a.speed, ctx, "num")
    return line("{") .. line("    let to = " .. to .. ";") .. line("    let speed = " .. speed .. ";")
      .. line("    let duration = ability_kit::div(ability_kit::distance(cx.pos(cx.caster)?, to), speed)?;")
      .. line("    cx.dash_to(to, duration)?;") .. line("}")
  elseif op == "dodge_projectiles" then
    if a.of ~= "caster" and not (a.of == "self" and self.entry) then self:block("action:dodge_projectiles.of:" .. tostring(a.of)); return "" end
    if a.from_team ~= nil and a.from_team ~= "enemy" then self:block("action:dodge_projectiles.from_team:" .. tostring(a.from_team)) end
    if a.kind ~= nil and a.kind ~= "tracking" then self:block("action:dodge_projectiles.kind:" .. tostring(a.kind)) end
    return line("cx.dodge_projectiles()?;")
  elseif op == "chance" then
    -- independent rolls (pity_after / counter / prd are other fields: blocked by the field check)
    local p = self:value(a.p, ctx, "num")
    local s2 = line("{") .. line("    let p = " .. p .. ";") .. line("    if cx.chance(p) {")
      .. self:actions(a.actions, ctx, I .. "        ")
    if a.otherwise then s2 = s2 .. line("    } else {") .. self:actions(a.otherwise, ctx, I .. "        ") end
    return s2 .. line("    }") .. line("}")
  elseif op == "modify_damage" then
    if not ctx.pre then self:block("action:modify_damage outside a damage-pre trigger"); return "" end
    if a.cancel then return line("cx.damage_cancel();") end
    local out = ""
    if a.set ~= nil then out = out .. line("{ let value = " .. self:value(a.set, ctx, "num") .. "; cx.damage_set(value); }") end
    if a.multiplier ~= nil then out = out .. line("{ let factor = " .. self:value(a.multiplier, ctx, "num") .. "; cx.damage_multiply(factor); }") end
    return out
  elseif op == "projectile" then
    if not self.entry then self:block("action:projectile in a routine"); return "" end
    if a.from ~= nil and a.from ~= "caster" and not (a.from == "self" and self.entry) then self:block("action:projectile.from:" .. tostring(a.from)) end
    if a.team ~= nil and a.team ~= "enemy" then self:block("action:projectile.team:" .. tostring(a.team)) end
    local tk = types_key(a.types)
    local buildings = tk:find("building", 1, true) ~= nil
    if tk ~= "" and tk ~= "all" and tk ~= "creep+hero+neutral+summon" and tk ~= "building+creep+hero+neutral+summon" then
      self:block("action:projectile.types:" .. tk)
    end
    for _, fl in ipairs(a.flags or {}) do
      if fl ~= "include_magic_immune" and fl ~= "visible_only" then self:block("action:projectile.flag:" .. fl) end
    end
    local tracking = a.kind == "tracking"
    if a.kind ~= "tracking" and a.kind ~= "linear" then self:block("action:projectile.kind:" .. tostring(a.kind)); return "" end
    if not tracking and buildings then self:block("action:projectile linear hitting buildings") end
    local speed = self:value(a.speed, ctx, "num")
    local radius = a.radius ~= nil and self:value(a.radius, ctx, "num") or "Fixed64::ZERO"
    local homing, aim, distance = "None", "Vec2::ZERO", "Fixed64::ZERO"
    if tracking then
      homing = "Some(" .. self:value(a.to, ctx, "unit") .. ")"
    else
      distance = self:value(a.distance, ctx, "num")
      local dir = self:value(a.direction, ctx, "vec")
      aim = "ability_kit::offset(cx.pos(cx.caster)?, " .. dir .. ", " .. distance .. ")"
    end
    local max_hits = "1"
    if a.max_hits ~= nil then max_hits = "u16::try_from(ability_kit::floor(" .. self:value(a.max_hits, ctx, "num") .. ").raw() >> 10).unwrap_or(1)"
    elseif a.pierce == true then max_hits = "64" end
    local vision = a.vision ~= nil and ("Some(" .. self:value(a.vision, ctx, "num") .. ")") or "None"
    local end_radius = a.end_radius ~= nil and ("Some(" .. self:value(a.end_radius, ctx, "num") .. ")") or "None"
    local on_hit = "None"
    local args_line = ""
    if a.on_hit and #a.on_hit > 0 then
      local sc = step_scope(ctx, 0)
      local hit_name = a.as or "it"
      sc.names[hit_name] = { name = hit_name, ty = "unit", marker = "ability_kit::arg_as_unit(args, \1HIT\1)", used = true }
      if hit_name ~= "it" then sc.names.it = sc.names[hit_name] end
      sc.names.here = { name = "here", ty = "point", marker = "ability_kit::arg_as_point(args, \1HERE\1)", used = true }
      local step = self:reserve_step()
      local body = self:actions(a.on_hit, { owner = ctx.owner, scope = sc, event = false }, "    ")
      local args = self:capture_args(sc)
      if #args + 2 > 8 then self:block("projectile.on_hit: more than 6 captured values") end
      body = body:gsub("\1HIT\1", tostring(#args)):gsub("\1HERE\1", tostring(#args + 1))
      self:set_step(step, "`projectile` on_hit", body, true)
      args_line = line("    let args = [" .. table.concat(args, ", ") .. "];")
      on_hit = string.format("Some((%d, &args[..]))", step)
    end
    return line("{") .. args_line
      .. line(string.format("    let p = Proj { homing: %s, aim: %s, speed: %s, radius: %s, distance: %s, max_hits: %s, buildings: %s, dodgeable: %s, vision: %s, end_radius: %s };",
        homing, aim, speed, radius, distance, max_hits, buildings and "true" or "false", a.dodgeable and "true" or "false", vision, end_radius))
      .. line("    cx.projectile(ID, p, " .. on_hit .. ")?;") .. line("}")
  elseif op == "set_state" or op == "clear_state" then
    local v = self.m.vars[a.var]
    if not v then self:block("state:" .. tostring(a.var) .. " not declared"); return "" end
    local ty = ({ int = "num", fixed = "num", bool = "bool", unit = "unit" })[v.def.type]
    if not ty then self:block("state type:" .. tostring(v.def.type)); return "" end
    local on = a.on ~= nil and self:value(a.on, ctx, "unit") or self:expr({ k = "id", name = "self" }, ctx)
    local hero = v.def.scope == "hero" and "true" or "false"
    if op == "clear_state" or a.value == "none" then
      return line("{") .. line("    let on = " .. on .. ";") .. line(string.format("    cx.clear_state(on, %s)?;", rust_str(v.key))) .. line("}")
    end
    local value = self:value(a.value, ctx, ty)
    return line("{") .. line("    let on = " .. on .. ";") .. line("    let value = " .. value .. ";")
      .. line(string.format("    cx.set_state_%s(on, %s, value, %s)?;", ty, rust_str(v.key), hero)) .. line("}")
  elseif op == "stop" then
    return line("return Ok(());")
  elseif op == "destroy_trees" then
    -- omoba-core tree_cut (E242): trees in the radius fall at the next tick, regrow after 300 s
    local at = self:value(a.at, ctx, "point")
    local radius = self:value(a.radius, ctx, "num")
    return line("{") .. line("    let at = " .. at .. ";") .. line("    let radius = " .. radius .. ";")
      .. line("    cx.destroy_trees(at, radius)?;") .. line("}")
  end
  return ""
end

---------------------------------------------------------------------------
-- entries
---------------------------------------------------------------------------
local RUST_TYPE = { num = "Fixed64", bool = "bool", unit = "EntityHandle", point = "Vec2", vec = "Vec2" }

-- `target` and the ability's own `flags` (no_target_self, self_cast_allowed …) -> the cast check
local function target_spec(g, target, ability_flags)
  target = target or {}
  local allowed = set({ "type", "range", "aoe_radius", "team", "types", "flags", "ability_flags", "notes" })
  for _, k in ipairs(sorted_keys(target)) do if not allowed[k] then g:block("target." .. k) end end
  local ty = target.type or "point"
  if ty ~= "unit" and ty ~= "none" and ty ~= "point" and ty ~= "direction" then g:block("target.type:" .. ty) end
  local team = ({ enemy = "Some(TeamRule::Enemy)", ally = "Some(TeamRule::Ally)", both = "Some(TeamRule::Both)" })[target.team or ""]
  if target.team and not team then g:block("target.team:" .. target.team) end
  local types = ({ [""] = { false, false }, all = { false, false }, hero = { true, false }, ["creep+hero"] = { false, true },
    ["creep+hero+neutral+summon"] = { false, true }, ["building+creep+hero"] = { false, false },
    ["building+creep+hero+neutral+summon"] = { false, false } })[types_key(target.types)]
  if not types then g:block("target.types:" .. types_key(target.types)); types = { false, false } end
  local flags = {}
  for _, list in ipairs({ target.flags or {}, target.ability_flags or {}, ability_flags or {} }) do
    for _, f in ipairs(list) do
      -- include_magic_immune: the cast check never refuses a magic-immune target; visible_only:
      -- the host only offers disclosed units as cast targets
      if f ~= "no_target_self" and f ~= "self_cast_allowed" and f ~= "can_target_buildings"
        and f ~= "include_magic_immune" and f ~= "pierce_magic_immune" and f ~= "visible_only" then g:block("target.flag:" .. f) end
      flags[f] = true
    end
  end
  return string.format("TargetSpec { unit: %s, none: %s, team: %s, heroes_only: %s, no_self: %s, self_cast: %s, no_buildings: %s }",
    tostring(ty == "unit"), tostring(ty == "none"), team or "None", tostring(types[1]), tostring(flags.no_target_self == true),
    tostring(flags.self_cast_allowed == true), tostring(types[2]))
end

local ENTRY_EVENTS = set({ "on_cast", "on_toggle_on", "on_toggle_off", "on_learn", "on_upgrade", "on_phase_start" })
local PASSIVE_TRIGGERS = { on_attack_hit = true, on_ability_executed = true, on_respawn = true,
  on_take_damage = true, on_deal_damage = true, on_attacked_hit = true, on_attacked = true, periodic = true,
  on_take_damage_pre = true, on_deal_damage_pre = true }
-- passive trigger -> status_event mask bit and host kind (the learner's marker listens)
local PASSIVE_EVENTS = { on_take_damage = { 8, 5 }, on_deal_damage = { 16, 6 }, on_attacked_hit = { 4, 4 }, on_attacked = { 4, 4 },
  on_take_damage_pre = { 32, 7 }, on_deal_damage_pre = { 64, 8 } }

-- the module of an entry the generator does not lower yet: casting it fails with the forms
local STUB = [[/// Not generated yet: %s
impl AbilityScript for %s {
    fn ability_id(&self) -> RStr<'_> { RStr::from_str(ID) }
    fn execute(&self, _: EntityHandle, _: Target, _: u8, _: RStr<'_>, _: &mut GameWorldDyn<'_>) -> RResult<(), RString> {
        RErr(%s.into())
    }
}

pub fn ffi() -> AbilityDefFFI { ability_kit::ability_ffi(ID, %s) }
]]
local function entry_head(def)
  return table.concat({ M.HEADER,
    string.format("//! %s %s（%s）\n", def.id, def.source_id or "", (def.text and def.text.zh_TW and def.text.zh_TW.name) or ""),
    "#![allow(unused_imports)]\nuse ability_kit::*;\n\n",
    string.format("pub const ID: &str = %s;\n\npub struct %s;\n\n", rust_str(def.id), camel(def.id)) })
end
local function stub_text(def, blockers)
  local why = def.id .. " is not generated yet: " .. table.concat(blockers, ", ")
  return entry_head(def) .. string.format(STUB, table.concat(blockers, ", "), camel(def.id), rust_str(why), camel(def.id))
end

-- An entry written by hand (`hook = "<name>"`, D7 / m3 §2.4): the trait its events need and the
-- AbilityScript that calls `hooks/<id>.rs`'s `Impl`. A missing file or method does not compile.
local function hook_methods(def)
  local events, passive = def.events or {}, def.passive or {}
  local list = {}
  if events.on_cast or events.on_toggle_on then list[#list + 1] = "on_cast" end
  if passive.stats or events.on_learn or events.on_upgrade or (passive.triggers and next(passive.triggers)) then list[#list + 1] = "on_learn" end
  if passive.triggers and passive.triggers.on_attack_hit then list[#list + 1] = "on_attack_hit" end
  local status_events, pre = false, false
  for _, sd in ipairs(def.statuses or {}) do
    for name in pairs(sd.triggers or {}) do
      if name == "on_take_damage_pre" or name == "on_deal_damage_pre" then pre = true
      elseif name ~= "on_created" and name ~= "periodic" then status_events = true end
    end
  end
  if status_events then list[#list + 1] = "on_status_event" end
  if pre then list[#list + 1] = "on_damage_pre" end
  return list
end

local function emit_hook_entry(m, o, opts)
  local def = o.def
  local root = (opts.root or "."):gsub("\\", "/")
  local package = opts.package or "?"
  local hook_file = string.format("%s/scripts/heroes/%s/src/hooks/%s.rs", root, package, def.id)
  local h = (opts.doc_hooks or {})[def.hook] or {}
  local methods = hook_methods(def)
  local sigs = {
    on_cast = "    /// The cast (`events.on_cast`): target checks already passed.\n    fn on_cast(cx: &mut Cx<'_, '_>) -> Res;\n",
    on_learn = "    /// Learned or ranked up (`on_learn`, passive stats and markers).\n    fn on_learn(cx: &mut Cx<'_, '_>) -> Res;\n",
    on_attack_hit = "    /// The holder's attack hit `cx.event_target` (`passive.triggers.on_attack_hit`).\n    fn on_attack_hit(cx: &mut Cx<'_, '_>) -> Res;\n",
    on_status_event = "    /// An event of one of this entry's statuses (omoba-core status_event).\n    fn on_status_event(cx: &mut Cx<'_, '_>, status: &str, event: StatusEvent) -> Res;\n",
    on_damage_pre = "    /// Before damage lands on / from a holder of this entry's status (scripting::damage_pre).\n    fn on_damage_pre(cx: &mut Cx<'_, '_>, status: &str, event: StatusEvent) -> DamageEdit;\n",
  }
  local t = { entry_head(def) }
  t[#t + 1] = string.format("/// Written by hand: %s\n#[path = %s]\nmod hand;\n\n", (h.why or ""):gsub("\n", " "), rust_str(hook_file))
  if #methods > 0 or true then t[#t + 1] = string.format("const MAX_LEVEL: u8 = %d;\n", def.max_level or 1) end
  t[#t + 1] = "\n/// `" .. def.id .. "` (hook `" .. def.hook .. "`): implemented by `hand::Impl` in src/hooks/" .. def.id .. ".rs.\npub trait Hook {\n"
  for _, mth in ipairs(methods) do t[#t + 1] = sigs[mth] end
  t[#t + 1] = "    /// A schedule it made came due (Cx::schedule).\n    fn on_scheduled(_cx: &mut Cx<'_, '_>, _at: Scheduled, _args: &[ScheduleArg]) -> Res { Ok(()) }\n}\n\n"
  local struct = camel(def.id)
  local impl = { string.format("impl AbilityScript for %s {\n    fn ability_id(&self) -> RStr<'_> { RStr::from_str(ID) }\n", struct) }
  local has = {}
  for _, mth in ipairs(methods) do has[mth] = true end
  if has.on_cast then
    local spec = target_spec(new_gen(m, o, opts), def.target, def.flags)
    t[#t + 1] = string.format("const TARGET: TargetSpec = %s;\n\n", spec)
    impl[#impl + 1] = [[
    fn execute(&self, caster: EntityHandle, target: Target, level: u8, level_data_json: RStr<'_>, world: &mut GameWorldDyn<'_>) -> RResult<(), RString> {
        let result = Cx::cast(world, caster, target, level, MAX_LEVEL, TARGET, level_data_json.as_str())
            .and_then(|mut cx| <hand::Impl as Hook>::on_cast(&mut cx));
        match result { Ok(()) => ROk(()), Err(e) => RErr(e.into()) }
    }
]]
  else
    impl[#impl + 1] = [[
    fn execute(&self, _: EntityHandle, _: Target, _: u8, _: RStr<'_>, _: &mut GameWorldDyn<'_>) -> RResult<(), RString> {
        RErr("not castable".into())
    }
]]
  end
  if has.on_learn then
    impl[#impl + 1] = [[
    fn on_learn(&self, caster: EntityHandle, new_level: u8, world: &mut GameWorldDyn<'_>) {
        if new_level == 0 || new_level > MAX_LEVEL { return; }
        let mut cx = Cx::passive(world, caster, new_level);
        let _ = <hand::Impl as Hook>::on_learn(&mut cx);
    }
]]
  end
  if has.on_attack_hit then
    impl[#impl + 1] = [[
    fn on_attack_hit(&self, owner: EntityHandle, attacker: EntityHandle, victim: EntityHandle, rank: u8, world: &mut GameWorldDyn<'_>) {
        if owner != attacker || attacker == victim { return; }
        let mut cx = Cx::passive(world, owner, rank);
        cx.target = Target::Entity(victim);
        cx.event_target = victim;
        cx.event_attacker = attacker;
        let _ = <hand::Impl as Hook>::on_attack_hit(&mut cx);
    }
]]
  end
  if has.on_status_event then
    impl[#impl + 1] = [[
    fn on_status_event(&self, holder: EntityHandle, status: RStr<'_>, event: StatusEvent, world: &mut GameWorldDyn<'_>) {
        let mut cx = Cx::passive(world, holder, event.level.max(1));
        cx.caster = event.source;
        let _ = <hand::Impl as Hook>::on_status_event(&mut cx, status.as_str(), event);
    }
]]
  end
  if has.on_damage_pre then
    impl[#impl + 1] = [[
    fn on_damage_pre(&self, holder: EntityHandle, status: RStr<'_>, event: StatusEvent, world: &mut GameWorldDyn<'_>) -> DamageEdit {
        let mut cx = Cx::passive(world, holder, event.level.max(1));
        cx.caster = event.source;
        <hand::Impl as Hook>::on_damage_pre(&mut cx, status.as_str(), event)
    }
]]
  end
  impl[#impl + 1] = [[
    fn on_scheduled(&self, owner: EntityHandle, at: Scheduled, args: RVec<ScheduleArg>, world: &mut GameWorldDyn<'_>) {
        let mut cx = Cx::passive(world, owner, at.level);
        let _ = <hand::Impl as Hook>::on_scheduled(&mut cx, at, args.as_slice());
    }
}

]]
  t[#t + 1] = table.concat(impl)
  t[#t + 1] = string.format("pub fn ffi() -> AbilityDefFFI { ability_kit::ability_ffi(ID, %s) }\n", struct)
  return table.concat(t), {}, {}, new_gen(m, o, opts)
end

-- -> rust text of the entry module, its blockers (sorted list), facts { owner_triggers }
local function emit_entry(m, o, opts, status_problems)
  local def = o.def
  if def.hook ~= nil then return emit_hook_entry(m, o, opts) end
  local g = new_gen(m, o, opts)
  g.status_problems = status_problems
  g.paths = effect_paths(def, "")
  local struct = camel(def.id)
  local fns = {}
  -- a cast program reads `target`, `point` and `direction`; a passive trigger reads `event`
  local function program(name, list, event)
    local ctx = { owner = o, scope = scope_child(nil), event = event, cast = name == "on_cast" }
    local body = g:actions(list, ctx, "    ")
    fns[#fns + 1] = string.format("fn %s(cx: &mut Cx<'_, '_>) -> Res {\n%s    Ok(())\n}\n", name, body)
  end
  if def.coverage == "rust_handler" and not def.events and not def.passive then g:block("rust_handler without behavior") end
  if def.hooks then g:block("hooks") end
  local requires_check = ""
  if def.requires then
    for _, k in ipairs(sorted_keys(def.requires)) do
      local sname = def.requires[k]
      if k == "has_status" or k == "not_status" then
        if type(sname) ~= "string" or not m.statuses[sname] then g:block("requires." .. k .. ":" .. tostring(sname))
        else
          requires_check = requires_check .. string.format("    if %scx.has_status(cx.caster, &super::ids::status::%s) { return Err(%s); }\n",
            k == "has_status" and "!" or "", upper(sname), rust_str("v3: requires " .. k .. " " .. sname))
        end
      elseif k ~= "hero_level" then g:block("requires." .. k) end
    end
  end
  local events = def.events or {}
  for _, ev in ipairs(sorted_keys(events)) do
    if not ENTRY_EVENTS[ev] then g:block("event:" .. ev) end
  end
  -- a windup hint is presentation only
  for _, a in ipairs(events.on_phase_start or {}) do if a.op ~= "cue" then g:block("event:on_phase_start") break end end
  if events.on_toggle_on and not events.on_toggle_off or events.on_toggle_off and not events.on_toggle_on then g:block("toggle: needs both on_toggle_on and on_toggle_off") end
  if events.on_cast and events.on_toggle_on then g:block("toggle: cast and toggled") end
  local spec = target_spec(g, def.target, def.flags)
  if events.on_cast then
    program("on_cast", events.on_cast, false)
    if requires_check ~= "" then fns[#fns] = fns[#fns]:gsub("(fn on_cast%(cx: &mut Cx<'_, '_>%) %-> Res {\n)", "%1" .. requires_check:gsub("%%", "%%%%"), 1) end
  elseif requires_check ~= "" then g:block("requires without a cast") end
  local toggle_status
  if events.on_toggle_on then
    program("on_toggle_on", events.on_toggle_on, false)
    program("on_toggle_off", events.on_toggle_off or {}, false)
    for _, a in ipairs(events.on_toggle_on) do
      if a.op == "apply_status" and a.to == "caster" then toggle_status = a.status break end
    end
    if not toggle_status then g:block("toggle: on_toggle_on must put a status on the caster") end
  end
  if events.on_learn then program("on_learn_first", events.on_learn, false) end
  if events.on_upgrade then program("on_upgrade", events.on_upgrade, false) end
  local passive = def.passive or {}
  local owner_triggers = {}
  for _, k in ipairs(sorted_keys(passive)) do
    if k ~= "reveal" and k ~= "triggers" and k ~= "stats" and k ~= "notes" then g:block("passive:" .. k) end
  end
  local filters = {}
  local passive_mask, passive_arms, passive_periodic = 0, {}, nil
  local function doc_id_of(_) return (m.doc_id or "doc"):lower() end
  for _, ev in ipairs(sorted_keys(passive.triggers)) do
    local tr = passive.triggers[ev]
    -- shorthand: a trigger written as its action list
    if tr[1] ~= nil then tr = { actions = tr } end
    if not PASSIVE_TRIGGERS[ev] then g:block("passive_trigger:" .. ev)
    else
      for _, k in ipairs(sorted_keys(tr)) do
        if k ~= "actions" and k ~= "filter" and k ~= "notes" and not (k == "who" and ev == "on_ability_executed") then g:block("passive_trigger:" .. ev .. "." .. k) end
      end
      if tr.who and tr.who ~= "self" then g:block("passive_trigger:on_ability_executed.who:" .. tr.who) end
      if ev == "periodic" then
        -- every `interval` while learned on this life: the first learn of a life starts it
        -- (a state variable of the life keeps rank-ups from starting a second chain)
        for _, k in ipairs(sorted_keys(tr)) do if k ~= "interval" and k ~= "first" and k ~= "actions" and k ~= "notes" then g:block("passive_trigger:periodic." .. k) end end
        local ctx = { owner = o, scope = scope_child(nil), event = false }
        local step = g:reserve_step()
        local interval = g:value(tr.interval, ctx, "num")
        local body = "    let rank = cx.learned_rank(cx.self_unit, ID);\n    if rank == 0 { return Ok(()); }\n    cx.rank = rank;\n"
          .. "    {\n        let interval = " .. interval .. ";\n        let owner = cx.self_unit;\n"
          .. string.format("        cx.schedule(owner, ID, %d, interval, args)?;\n    }\n", step)
          .. g:actions(tr.actions or {}, ctx, "    ")
        g:set_step(step, "passive periodic", body, true)
        local first = tr.first ~= nil and g:value(tr.first, ctx, "num") or interval
        passive_periodic = "        let key = " .. rust_str(tostring(doc_id_of(o)) .. ":" .. def.id .. ":periodic") .. ";\n"
          .. "        if !cx.state_bool(caster, key, false) {\n"
          .. "            let _ = cx.set_state_bool(caster, key, true, false);\n"
          .. "            let first = { " .. first .. " };\n"
          .. string.format("            let _ = cx.schedule(caster, ID, %d, first, &[]);\n", step)
          .. "        }\n"
      else
        local evt = PASSIVE_EVENTS[ev]
        local pre = ev == "on_take_damage_pre" or ev == "on_deal_damage_pre"
        local ctx = { owner = o, scope = scope_child(nil), event = true, pre = pre }
        local filter = tr.filter and g:value(tr.filter, ctx, "bool") or "true"
        filters[ev] = filter
        local body = g:actions(tr.actions, ctx, "    ")
        fns[#fns + 1] = string.format("fn %s(cx: &mut Cx<'_, '_>) -> Res {\n    if !(%s) { return Ok(()); }\n%s    Ok(())\n}\n", ev, filter, body)
        if evt then
          passive_mask = passive_mask + (passive_mask & evt[1] == 0 and evt[1] or 0)
          passive_arms[#passive_arms + 1] = { kind = evt[2], fn = ev, pre = pre }
        elseif ev ~= "on_attack_hit" then owner_triggers[#owner_triggers + 1] = ev end
      end
    end
  end
  local passive_stats
  if passive.stats then passive_stats = g:stat_rows(passive.stats, { owner = o, scope = scope_child(nil) }) end

  -- status triggers: on_created / periodic run from the applying code (st_<id>_start); the rest
  -- come back through on_status_event (kind -> functions)
  local status_arms = {}
  for _, sd in ipairs(def.statuses or {}) do
    local so = m.statuses[sd.id]
    if sd.triggers and not (sd.runtime_id or ""):match("^attack_orb:") then
      local begin = {}
      for _, name in ipairs(sorted_keys(sd.triggers)) do
        local t = trigger_kind(name)
        if not t then g:block("status_trigger:" .. name)
        else
          local tr = sd.triggers[name]
          if name:match("^periodic") then
            for _, k in ipairs(sorted_keys(tr)) do if k ~= "interval" and k ~= "first" and k ~= "actions" and k ~= "notes" then g:block("status_trigger:periodic." .. k) end end
            local ctx = { owner = so, scope = scope_child(nil), event = false }
            local step = g:reserve_step()
            local interval = g:value(tr.interval, ctx, "num")
            local body = "    cx.caster = ability_kit::arg_as_unit(args, 0);\n"
              .. "    if !cx.has_status(cx.self_unit, &super::ids::status::" .. upper(sd.id) .. ") { return Ok(()); }\n"
              .. "    {\n        let interval = " .. interval .. ";\n        let owner = cx.self_unit;\n"
              .. string.format("        cx.schedule(owner, ID, %d, interval, args)?;\n    }\n", step)
              .. g:actions(tr.actions or {}, ctx, "    ")
            g:set_step(step, "`" .. sd.id .. "` periodic", body, true)
            local first = tr.first ~= nil and g:value(tr.first, ctx, "num") or interval
            begin[#begin + 1] = "    {\n        let first = " .. first .. ";\n        let args = [ability_kit::arg_unit(cx.caster)];\n"
              .. "        let owner = cx.self_unit;\n" .. string.format("        cx.schedule(owner, ID, %d, first, &args)?;\n    }\n", step)
          else
            local filter, actions = trigger_parts(tr)
            local evented = t[2] ~= nil and name ~= "on_removed" and name ~= "on_expired" and name ~= "on_death"
            local pre = name == "on_take_damage_pre" or name == "on_deal_damage_pre"
            local ctx = { owner = so, scope = scope_child(nil), event = evented, pre = pre }
            local cond = filter and g:value(filter, ctx, "bool") or nil
            local fname = "st_" .. sd.id .. "_" .. name
            local guard = name == "on_death" and "    if cx.alive(cx.self_unit) { return Ok(()); }\n" or ""
            fns[#fns + 1] = string.format("#[allow(dead_code)]\npub(crate) fn %s(cx: &mut Cx<'_, '_>) -> Res {\n%s%s%s    Ok(())\n}\n", fname, guard,
              cond and ("    if !(" .. cond .. ") { return Ok(()); }\n") or "", g:actions(actions, ctx, "    "))
            if name == "on_created" then begin[#begin + 1] = "    " .. fname .. "(cx)?;\n"
            else
              for _, kind in ipairs(t[2]) do
                status_arms[#status_arms + 1] = { status = sd.id, kind = kind, fn = fname }
              end
            end
          end
        end
      end
      if #begin > 0 then
        fns[#fns + 1] = string.format("/// `%s` starts on a unit: on_created, then the periodic schedule.\n#[allow(dead_code)]\npub(crate) fn st_%s_start(cx: &mut Cx<'_, '_>, holder: EntityHandle) -> Res {\n"
          .. "    let prev = cx.self_unit;\n    cx.self_unit = holder;\n    let r = st_%s_begin(cx);\n    cx.self_unit = prev;\n    r\n}\n\n#[allow(dead_code)]\nfn st_%s_begin(cx: &mut Cx<'_, '_>) -> Res {\n%s    Ok(())\n}\n",
          sd.id, sd.id, sd.id, sd.id, table.concat(begin))
      end
    end
  end

  -- the AbilityScript
  local blockers = sorted_keys(g.blockers)
  if #blockers > 0 then return stub_text(def, blockers), blockers, {}, g end
  for _, st in ipairs(g.steps or {}) do fns[#fns + 1] = st end
  local s = { entry_head(def) }
  local run = events.on_cast or events.on_toggle_on
  local learns = passive_stats or #owner_triggers > 0 or events.on_learn or events.on_upgrade or passive_mask ~= 0 or passive_periodic
  if run or learns then s[#s + 1] = string.format("const MAX_LEVEL: u8 = %d;\n", def.max_level or 1) end
  if run then s[#s + 1] = string.format("const TARGET: TargetSpec = %s;\n", spec) end
  s[#s + 1] = "\n"
  for _, f in ipairs(fns) do s[#s + 1] = g:finish(f) .. "\n" end
  local impl = {}
  impl[#impl + 1] = string.format("impl AbilityScript for %s {\n    fn ability_id(&self) -> RStr<'_> { RStr::from_str(ID) }\n\n", struct)
  if events.on_cast then run = "on_cast(&mut cx)"
  elseif events.on_toggle_on then
    run = string.format("if cx.has_status(caster, &super::ids::status::%s) { on_toggle_off(&mut cx) } else { on_toggle_on(&mut cx) }", upper(toggle_status))
  end
  if run then
    impl[#impl + 1] = string.format([[    fn execute(&self, caster: EntityHandle, target: Target, level: u8, level_data_json: RStr<'_>, world: &mut GameWorldDyn<'_>)
        -> RResult<(), RString> {
        let result = Cx::cast(world, caster, target, level, MAX_LEVEL, TARGET, level_data_json.as_str())
            .and_then(|mut cx| %s);
        match result { Ok(()) => ROk(()), Err(e) => RErr(e.into()) }
    }
]], run)
  else
    impl[#impl + 1] = [[    fn execute(&self, _: EntityHandle, _: Target, _: u8, _: RStr<'_>, _: &mut GameWorldDyn<'_>) -> RResult<(), RString> {
        RErr("passive ability cannot be cast actively".into())
    }
]]
  end
  local learn = {}
  if passive_mask ~= 0 or passive_periodic then
    learn[#learn + 1] = string.format("        let _ = cx.passive_marker(ID, %s, %d);\n", passive_stats or "&[]", passive_mask)
  elseif passive_stats then learn[#learn + 1] = "        let _ = cx.passive_stats(ID, " .. passive_stats .. ");\n"
  elseif #owner_triggers > 0 then learn[#learn + 1] = "        let _ = cx.learn_marker(ID);\n" end
  if passive_periodic then learn[#learn + 1] = passive_periodic end
  if events.on_learn then learn[#learn + 1] = "        if new_level == 1 { let _ = on_learn_first(&mut cx); }\n" end
  -- on_upgrade: every rank, the first included (v3.md §12.4)
  if events.on_upgrade then learn[#learn + 1] = "        let _ = on_upgrade(&mut cx);\n" end
  if #learn > 0 then
    impl[#impl + 1] = "\n    fn on_learn(&self, caster: EntityHandle, new_level: u8, world: &mut GameWorldDyn<'_>) {\n"
      .. "        if new_level == 0 || new_level > MAX_LEVEL { return; }\n"
      .. "        let mut cx = Cx::passive(world, caster, new_level);\n" .. g:finish(table.concat(learn)) .. "    }\n"
  end
  for _, pa in ipairs(passive_arms) do status_arms[#status_arms + 1] = { status = def.id, kind = pa.kind, fn = pa.fn } end
  if #status_arms > 0 then
    local by = {}
    local order = {}
    for _, a in ipairs(status_arms) do
      local key = a.status .. "\0" .. a.kind
      if not by[key] then by[key] = { status = a.status, kind = a.kind, fns = {} }; order[#order + 1] = key end
      table.insert(by[key].fns, a.fn)
    end
    table.sort(order)
    local SETUP = {
      [3] = "cx.event_target = event.other; cx.target = Target::Entity(event.other);",
      [4] = "cx.event_attacker = event.other;",
      [5] = "cx.event_attacker = event.other; cx.event_amount = event.amount; cx.event_damage_kind = event.damage_kind;",
      [6] = "cx.event_target = event.other; cx.event_amount = event.amount; cx.event_damage_kind = event.damage_kind;",
      [7] = "cx.event_attacker = event.other; cx.event_amount = event.amount; cx.event_damage_kind = event.damage_kind;",
      [8] = "cx.event_target = event.other; cx.event_amount = event.amount; cx.event_damage_kind = event.damage_kind;",
    }
    local arms, pre_arms = {}, {}
    for _, key in ipairs(order) do
      local b = by[key]
      local calls = {}
      for _, fn in ipairs(b.fns) do calls[#calls + 1] = "let _ = " .. fn .. "(&mut cx);" end
      local list = (b.kind == 7 or b.kind == 8) and pre_arms or arms
      list[#list + 1] = string.format("            (%s, %d) => { %s%s }\n", rust_str(b.status), b.kind,
        SETUP[b.kind] and (SETUP[b.kind] .. " ") or "", table.concat(calls, " "))
    end
    if #arms > 0 then
      impl[#impl + 1] = "\n    fn on_status_event(&self, holder: EntityHandle, status: RStr<'_>, event: StatusEvent, world: &mut GameWorldDyn<'_>) {\n"
        .. "        let mut cx = Cx::passive(world, holder, event.level.max(1));\n"
        .. "        cx.caster = event.source;\n"
        .. "        match (status.as_str(), event.kind) {\n" .. table.concat(arms) .. "            _ => {}\n        }\n    }\n"
    end
    if #pre_arms > 0 then
      impl[#impl + 1] = "\n    fn on_damage_pre(&self, holder: EntityHandle, status: RStr<'_>, event: StatusEvent, world: &mut GameWorldDyn<'_>) -> DamageEdit {\n"
        .. "        let mut cx = Cx::passive(world, holder, event.level.max(1));\n"
        .. "        cx.caster = event.source;\n"
        .. "        match (status.as_str(), event.kind) {\n" .. table.concat(pre_arms) .. "            _ => {}\n        }\n        cx.damage_edit\n    }\n"
    end
  end
  if g.steps and #g.steps > 0 then
    local arms = {}
    for i = 1, #g.steps do arms[#arms + 1] = string.format("            %d => step_%d(&mut cx, args.as_slice()),\n", i, i) end
    impl[#impl + 1] = "\n    fn on_scheduled(&self, owner: EntityHandle, at: Scheduled, args: RVec<ScheduleArg>, world: &mut GameWorldDyn<'_>) {\n"
      .. "        let mut cx = Cx::passive(world, owner, at.level);\n"
      .. "        let _ = match at.step {\n" .. table.concat(arms) .. "            _ => Ok(()),\n        };\n    }\n"
  end
  if filters.on_attack_hit then
    impl[#impl + 1] = [[

    fn on_attack_hit(&self, owner: EntityHandle, attacker: EntityHandle, victim: EntityHandle, rank: u8, world: &mut GameWorldDyn<'_>) {
        // The holder's own attacks only, never on itself.
        if owner != attacker || attacker == victim { return; }
        let mut cx = Cx::passive(world, owner, rank);
        cx.target = Target::Entity(victim);
        cx.event_target = victim;
        cx.event_attacker = attacker;
        let _ = on_attack_hit(&mut cx);
    }
]]
  end
  impl[#impl + 1] = "}\n"
  s[#s + 1] = table.concat(impl)
  -- owner triggers (the hero UnitScript calls these)
  if #owner_triggers > 0 then
    s[#s + 1] = [[

/// Learned: the learn marker (or the passive stats buff) is on the owner. Rank 1: the world API
/// has no rank read (the IR interpreter's on_owner_cast).
fn learned(world: &GameWorldDyn<'_>, owner: EntityHandle) -> bool { world.has_buff(owner, RStr::from_str(ID)) }
]]
    for _, ev in ipairs(owner_triggers) do
      if ev == "on_ability_executed" then
        s[#s + 1] = [[

pub fn on_owner_cast(owner: EntityHandle, skill: &str, world: &mut GameWorldDyn<'_>) {
    if !learned(world, owner) { return; }
    let mut cx = Cx::passive(world, owner, 1);
    cx.event_skill = Some(skill);
    let _ = on_ability_executed(&mut cx);
}
]]
      elseif ev == "on_respawn" then
        s[#s + 1] = [[

pub fn on_owner_respawn(owner: EntityHandle, world: &mut GameWorldDyn<'_>) {
    if !learned(world, owner) { return; }
    let mut cx = Cx::passive(world, owner, 1);
    let _ = on_respawn(&mut cx);
}
]]
      end
    end
  end
  s[#s + 1] = "\npub fn ffi() -> AbilityDefFFI { ability_kit::ability_ffi(ID, " .. struct .. ") }\n"
  return table.concat(s), blockers, { owner_triggers = owner_triggers }, g
end

-- routines a supported entry calls: one Rust fn each (owner values, typed params)
local function emit_routines(m, opts, status_problems, wanted)
  local done, out, blockers_of = {}, {}, {}
  local pending = sorted_keys(wanted)
  while #pending > 0 do
    local name = table.remove(pending, 1)
    if not done[name] then
      done[name] = true
      local r = m.routines[name]
      local g = new_gen(m, nil, opts)
      g.status_problems = status_problems
      local owner = r.owner and m.abilities[r.owner] or m.owners.shared
      local scope = scope_child(nil)
      local params = {}
      for _, p in ipairs(sorted_keys(r.params or {})) do
        local kind = r.params[p]
        local ty = RUST_TYPE[kind]
        if not ty then g:block("routine param kind " .. tostring(kind)); ty = "Fixed64" end
        local marker = g:bind(scope, p, RUST_TYPE[kind] and kind or "num")
        params[#params + 1] = marker .. ": " .. ty
      end
      g.paths = effect_paths(r, "routines." .. name)
      g.trace_owner = r.owner or name
      local body = g:actions(r.actions, { owner = owner, scope = scope, event = false }, "    ")
      local blockers = sorted_keys(g.blockers)
      blockers_of[name] = { own = blockers, calls = g.routines_needed }
      opts.routine_gaps = opts.routine_gaps or {}
      opts.routine_gaps[name] = g.gaps
      local sig = string.format("pub fn %s(cx: &mut Cx<'_, '_>%s) -> Res {\n", name, #params > 0 and (", " .. table.concat(params, ", ")) or "")
      if #blockers > 0 then
        -- not generated yet: the caller's cast fails
        for _, b in ipairs(g.lets) do b.used = false end
        out[#out + 1] = g:finish("#[allow(unused_variables)]\n" .. sig .. "    let _ = cx;\n    Err(" .. rust_str("routine " .. name .. " is not generated yet: " .. table.concat(blockers, ", ")) .. ")\n}\n")
      else
        out[#out + 1] = g:finish(sig .. body .. "    Ok(())\n}\n")
      end
      for n in pairs(g.routines_needed) do if not done[n] then pending[#pending + 1] = n end end
    end
  end
  -- blockers of a routine and of every routine it calls
  local closed = {}
  local function close(name, seen)
    if closed[name] then return closed[name] end
    seen = seen or {}
    if seen[name] then return {} end
    seen[name] = true
    local all = {}
    local b = blockers_of[name]
    for _, x in ipairs(b and b.own or {}) do all[#all + 1] = "routine " .. name .. ": " .. x end
    for n in pairs(b and b.calls or {}) do for _, x in ipairs(close(n, seen)) do all[#all + 1] = x end end
    table.sort(all)
    closed[name] = all
    return all
  end
  local result = {}
  for name in pairs(blockers_of) do result[name] = close(name) end
  return out, result
end

---------------------------------------------------------------------------
-- the crate
---------------------------------------------------------------------------
local function collect_cues(doc)
  local cues = {}
  local function walk(x)
    if type(x) ~= "table" then return end
    if x.op == "cue" and type(x.id) == "string" then cues[x.id] = true end
    for k, v in pairs(x) do
      if k == "cue" and type(v) == "string" then cues[v] = true end
      walk(v)
    end
  end
  walk(doc.abilities); walk(doc.items); walk(doc.statuses); walk(doc.routines); walk(doc.unit_templates)
  return cues
end

-- doc -> { [relative path] = text }, report { crate, entries = { {id, blockers} } }
function M.generate(doc, opts)
  opts = opts or {}
  local root = assert(opts.root, "opts.root")
  local compiled = C.compile(doc, { ir = false, names = opts.names })
  local m = model(doc, compiled)
  -- an absolute root: a hook entry's `#[path]` names its hand-written file from OUT_DIR
  local abs_root = root
  if not abs_root:match("^%a:") and not abs_root:match("^/") then
    local ok, lfs = pcall(require, "lfs")
    abs_root = (ok and lfs.currentdir() or ".") .. "/" .. abs_root
  end
  abs_root = abs_root:gsub("\\", "/"):gsub("/%./", "/"):gsub("/%.$", "")
  local gopts = { rules = opts.rules, vocab = M.vocab(root), cues_used = {}, root = abs_root, package = opts.package }
  local files = {}
  local crate = M.crate_name(doc)
  local gen = "src/generated/"
  files[gen .. "values.rs"] = emit_values(m)
  local status_text, status_problems = emit_statuses(m, root)
  local report = { crate = crate, id = doc.id, entries = {} }
  gopts.doc_hooks = doc.hooks or {}
  local mods, ffis, owner_calls = {}, {}, { on_ability_executed = {}, on_respawn = {} }
  local wanted = {}
  local entry_gens = {}
  for _, o in ipairs(m.entries) do
    local text, blockers, facts, g = emit_entry(m, o, gopts, status_problems)
    entry_gens[#entry_gens + 1] = { o = o, text = text, blockers = blockers, facts = facts, g = g }
    if #blockers == 0 then for n in pairs(g.routines_needed) do wanted[n] = true end end
  end
  local routines, routine_blockers = emit_routines(m, gopts, status_problems, wanted)
  -- an entry that calls a routine with blockers is blocked by them (re-emitted as a stub)
  local function routine_closure(g)
    local all = {}
    for n in pairs(g.routines_needed) do for _, b in ipairs(routine_blockers[n] or {}) do all[b] = true end end
    return sorted_keys(all)
  end
  for _, e in ipairs(entry_gens) do
    local inherited = #e.blockers == 0 and routine_closure(e.g) or {}
    if #inherited > 0 then
      -- a routine it calls has blockers: the entry is a stub too
      e.blockers = inherited
      e.text = stub_text(e.o.def, inherited)
      e.facts = {}
    else
      for c in pairs(e.g.cues) do gopts.cues_used[c] = true end
    end
    local id = e.o.def.id
    -- not generated yet (D7): no code; the coverage report lists it and the hero stays out of hero_dlls
    if #e.blockers == 0 then
      files[gen .. id .. ".rs"] = e.text
      mods[#mods + 1] = id
      if e.o.kind == "ability" then ffis[#ffis + 1] = id end
    end
    for _, ev in ipairs(e.facts.owner_triggers or {}) do owner_calls[ev][#owner_calls[ev] + 1] = id end
    local gaps = {}
    if #e.blockers == 0 then
      for k in pairs(e.g.gaps) do gaps[k] = true end
      for n in pairs(e.g.routines_needed) do for k in pairs((gopts.routine_gaps or {})[n] or {}) do gaps["routine " .. n .. ": " .. k] = true end end
    end
    report.entries[#report.entries + 1] = { id = id, kind = e.o.kind, source_id = e.o.def.source_id, blockers = e.blockers, gaps = sorted_keys(gaps) }
  end
  -- routines that only blocked entries call are not emitted
  files[gen .. "routines.rs"] = M.HEADER .. "//! The document's routines a generated entry calls.\n#![allow(unused_imports)]\nuse ability_kit::*;\n\n"
    .. table.concat(routines, "\n")
  -- ids: statuses and cues
  local cues = collect_cues(doc)
  local cue_ids = sorted_keys(cues)
  local cue = { "\n/// Presentation cue ids (m6-ue-presentation-contract.md); never read by settlement.\n#[allow(dead_code)]\n#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub enum Cue {\n" }
  for _, c in ipairs(cue_ids) do cue[#cue + 1] = "    " .. camel(c) .. ",\n" end
  cue[#cue + 1] = "}\n\nimpl Cue {\n    pub const fn as_str(self) -> &'static str {\n        match self {\n"
  for _, c in ipairs(cue_ids) do cue[#cue + 1] = string.format("            Cue::%s => %s,\n", camel(c), rust_str(c)) end
  cue[#cue + 1] = "        }\n    }\n}\n"
  if #cue_ids == 0 then cue = { "\n#[allow(dead_code)]\n#[derive(Clone, Copy, Debug)]\npub enum Cue {}\n\nimpl Cue {\n    pub const fn as_str(self) -> &'static str { match self {} }\n}\n" } end
  files[gen .. "ids.rs"] = status_text .. table.concat(cue)
  -- mod.rs: the modules
  local hero_name = (doc.hero and doc.hero.text and doc.hero.text.zh_TW and doc.hero.text.zh_TW.name) or ""
  local mr = { M.HEADER, string.format("//! %s %s：由 v3 文件 %s 產生的技能。\n\n", doc.id, hero_name, opts.package or ""),
    "#![allow(unused_parens)]\n\npub mod ids;\npub mod registry;\npub mod routines;\npub mod values;\n" }
  for _, id in ipairs(mods) do mr[#mr + 1] = "pub mod " .. id .. ";\n" end
  files[gen .. "mod.rs"] = table.concat(mr)
  -- registry.rs: what src/lib.rs exports through the manifest (line B's lib.rs shell calls these)
  local rg = { M.HEADER, "//! The abilities and hero units of this DLL; src/lib.rs exports them (`export_root_module`).\n",
    "#![allow(unused_imports)]\nuse ability_kit::*;\n\n" }
  rg[#rg + 1] = string.format("/// The hero id of this DLL (m3-rust-codegen-design §3.1).\npub const HERO_ID: &str = %s;\n\n", rust_str(doc.id))
  local pushes = {}
  for _, id in ipairs(ffis) do pushes[#pushes + 1] = "super::" .. id .. "::ffi()" end
  rg[#rg + 1] = "/// Every ability of the document.\npub fn abilities() -> RVec<AbilityDefFFI> {\n"
    .. (#pushes == 0 and "    RVec::new()\n" or ("    RVec::from(vec![\n        " .. table.concat(pushes, ",\n        ") .. ",\n    ])\n")) .. "}\n"
  local has_owner = #owner_calls.on_ability_executed + #owner_calls.on_respawn > 0
  if has_owner then
    local casts, respawns = {}, {}
    for _, id in ipairs(owner_calls.on_ability_executed) do casts[#casts + 1] = "        super::" .. id .. "::on_owner_cast(caster, skill_id.as_str(), world);\n" end
    for _, id in ipairs(owner_calls.on_respawn) do respawns[#respawns + 1] = "        super::" .. id .. "::on_owner_respawn(e, world);\n" end
    rg[#rg + 1] = string.format([[

/// The hero UnitScript: passives that trigger on the owner's casts and respawns.
pub struct HeroUnit;

impl UnitScript for HeroUnit {
    fn unit_id(&self) -> RStr<'_> { RStr::from_str(%s) }
    fn on_skill_cast(&self, caster: EntityHandle, skill_id: RStr<'_>, _: Target, world: &mut GameWorldDyn<'_>) {
        let _ = (&caster, &skill_id, &world);
%s    }
    fn on_respawn(&self, e: EntityHandle, world: &mut GameWorldDyn<'_>) {
        let _ = (&e, &world);
%s    }
}

/// The hero units of this DLL.
pub fn units() -> RVec<UnitDef> {
    RVec::from(vec![UnitDef { unit_id: HeroUnit.unit_id().into(), script: UnitScript_TO::from_value(HeroUnit, TD_Opaque) }])
}
]], rust_str(opts.unit_id or opts.package or doc.id), table.concat(casts), table.concat(respawns))
  else
    rg[#rg + 1] = "\n/// The hero units of this DLL (none: no owner-triggered passives).\npub fn units() -> RVec<UnitDef> { RVec::new() }\n"
  end
  files[gen .. "registry.rs"] = table.concat(rg)
  -- scaffold: written only when missing, then hand-owned (line B). Two packages per hero (line B,
  -- 2026-10-29): heroes/<package>/ is `hero_<id>_impl` (rlib: the abilities and the manifest, no
  -- export, so base_content tests can link it), heroes/<package>/dll/ is `hero_<id>` (cdylib: only
  -- export_root_module over the impl's get_manifest).
  local scaffold = {}
  local module_id = crate
  scaffold["src/lib.rs"] = string.format([[//! %s: %s（%s）的技能腳本，自己一個 DLL（M3'，m3-rust-codegen-design.md §3）。
//! 技能程式由 build.rs 在每次建置前以 v3_rust.lua 產生到 $OUT_DIR/generated.rs；本檔手寫。
//! 匯出與 base_content 同一種 `Manifest`；host 以 `lib_header_from_path` 逐檔載入。
#![deny(warnings)]

/// This hero's abilities, generated by build.rs (v3_rust.lua --single) from its v3 document.
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

use ability_kit::*;

pub use ability_kit::Manifest_Ref;

/// Module id in the manifest and in scripts-manifest.json.
pub const MODULE_ID: &str = %s;

/// The root module `dll/` exports as %s.dll.
pub fn get_manifest() -> Manifest_Ref {
    Manifest { units, abilities, dev_reload_runtime_lua_content, module_id, content_marker, content_identity }.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn units() -> RVec<UnitDef> { generated::registry::units() }

#[sabi_extern_fn]
pub fn abilities() -> RVec<AbilityDefFFI> { generated::registry::abilities() }

#[sabi_extern_fn]
fn dev_reload_runtime_lua_content(expected_hash: RStr<'_>) -> RuntimeLuaReloadResultFFI {
    ability_kit::module::dev_reload_runtime_lua_content(MODULE_ID, expected_hash)
}

#[sabi_extern_fn]
fn module_id() -> RStr<'static> { RStr::from_str(MODULE_ID) }

#[sabi_extern_fn]
fn content_marker() -> RStr<'static> { ability_kit::module::content_marker() }

#[sabi_extern_fn]
fn content_identity() -> RStr<'static> { ability_kit::module::content_identity() }
]], module_id, doc.id, hero_name, rust_str(module_id), module_id)
  scaffold["Cargo.toml"] = string.format([[[package]
name = "%s_impl"
version = "0.1.0"
edition = "2021"
description = "%s (%s) ability scripts generated from docs/character_pipeline/packages/%s/v3.lua; dll/ builds them into %s.dll (M3')"

[lib]
name = "%s_impl"
# rlib only: base_content's tests link it in-process. The DLL is the separate package in dll/.
crate-type = ["rlib"]

[dependencies]
abi_stable = "0.11"
ability_kit = { path = "../../ability_kit" }

[features]
default = []
runtime-lua-content = ["ability_kit/runtime-lua-content"]
compiled-content-only = ["ability_kit/compiled-content-only"]
]], crate, doc.id, opts.package or "", opts.package or "", crate, crate)
  scaffold["build.rs"] = string.format([[//! Generates this hero's abilities from its v3 document before every build
//! (docs/character_pipeline/tools/v3_rust.lua --single $OUT_DIR/generated.rs), like base_content's
//! build.rs runs gen_hero_registry.lua: checkouts, land_to_live and hot reloads (cargo only) never
//! compile stale code, and a value change rebuilds only this hero.
use std::env;
use std::path::PathBuf;
use std::process::Command;

const PACKAGE: &str = %s;

fn main() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // The repository root: the first ancestor with the pinned Lua and the generator (also from a shadow tree).
    let root = crate_dir.ancestors()
        .find(|p| p.join("tools/lua/lua.exe").is_file() && p.join("docs/character_pipeline/tools/v3_rust.lua").is_file())
        .expect("repository root with tools/lua/lua.exe")
        .to_path_buf();
    for path in [
        "docs/character_pipeline/tools",
        "docs/character_pipeline/schema",
        "docs/character_pipeline/items",
        "omoba-core/src/runtime/native/rules/vocab.rs",
        "scripts/lua_data/templates/game_rules.lua",
    ] {
        println!("cargo:rerun-if-changed={}", root.join(path).display());
    }
    println!("cargo:rerun-if-changed={}", root.join("docs/character_pipeline/packages").join(PACKAGE).display());
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("generated.rs");
    let output = Command::new(root.join("tools/lua/lua.exe"))
        .current_dir(&root)
        .arg("docs/character_pipeline/tools/v3_rust.lua")
        .arg("--hero").arg(PACKAGE)
        .arg("--single").arg(&out)
        .output()
        .unwrap_or_else(|error| panic!("run v3_rust.lua: {error}"));
    if !output.status.success() {
        panic!("v3_rust.lua failed: {}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    }
}
]], rust_str(opts.package or doc.id))
  scaffold["dll/Cargo.toml"] = string.format([[[package]
name = "%s"
version = "0.1.0"
edition = "2021"
description = "%s.dll: exports %s_impl's manifest (M3')"

[lib]
name = "%s"
crate-type = ["cdylib"]

[dependencies]
abi_stable = "0.11"
%s_impl = { path = ".." }

[features]
default = []
runtime-lua-content = ["%s_impl/runtime-lua-content"]
compiled-content-only = ["%s_impl/compiled-content-only"]
]], crate, crate, crate, crate, crate, crate, crate)
  scaffold["dll/src/lib.rs"] = string.format([[//! %s.dll：只匯出 %s_impl 的 manifest。技能本體在上一層的 rlib，base_content 的測試連結那個
//! rlib；這個 cdylib 沒有人依賴，所以測試建置不會覆寫 DLL。
#![deny(warnings)]

#[abi_stable::export_root_module]
fn get_manifest() -> %s_impl::Manifest_Ref {
    %s_impl::get_manifest()
}
]], crate, crate, crate, crate)
  report.scaffold = scaffold
  return files, report
end

---------------------------------------------------------------------------
-- files on disk
---------------------------------------------------------------------------
local function read(path)
  local f = io.open(path, "rb")
  if not f then return nil end
  local t = f:read("a"); f:close()
  return t
end

-- writes only files whose text changed; removes generated files no longer produced; scaffold
-- files (Cargo.toml, src/lib.rs) only when missing
function M.write_crate(dir, files, lfs, mkdir_p, scaffold)
  local changed = 0
  for _, rel in ipairs(sorted_keys(scaffold)) do
    local path = dir .. "/" .. rel
    if not read(path) then
      mkdir_p((path:match("^(.*)/") or "."))
      local f = assert(io.open(path, "wb")); f:write(scaffold[rel]); f:close()
      changed = changed + 1
    end
  end
  for _, rel in ipairs(sorted_keys(files)) do
    local path = dir .. "/" .. rel
    local text = files[rel]
    if read(path) ~= text then
      mkdir_p((path:match("^(.*)/") or "."))
      local f = assert(io.open(path, "wb")); f:write(text); f:close()
      changed = changed + 1
    end
  end
  local gdir = dir .. "/src/generated"
  if lfs.attributes(gdir, "mode") == "directory" then
    for name in lfs.dir(gdir) do
      if name:match("%.rs$") and not files["src/generated/" .. name] then os.remove(gdir .. "/" .. name); changed = changed + 1 end
    end
  end
  return changed
end

-- the listed heroes (packages with character.lua) and every hero document
function M.heroes(root, which)
  local lfs = require("lfs")
  local list = {}
  local dir = root .. "/docs/character_pipeline/packages"
  for name in lfs.dir(dir) do
    local v3 = dir .. "/" .. name .. "/v3.lua"
    if name:sub(1, 1) ~= "." and lfs.attributes(v3, "mode") == "file" then
      local listed = lfs.attributes(dir .. "/" .. name .. "/character.lua", "mode") == "file"
      if which == "all" or listed then list[#list + 1] = { package = name, path = v3, listed = listed } end
    end
  end
  table.sort(list, function(a, b) return a.package < b.package end)
  return list
end

function M.load(root)
  PACKAGE_V3_AS_MODULE = true
  local P = dofile(tools_dir .. "/package_v3.lua")
  PACKAGE_V3_AS_MODULE = nil
  return P
end

-- generate the crates of `heroes` into out_dir/heroes/<package>; returns reports, problems
-- The generated modules as one file for `include!` (line B: build.rs writes $OUT_DIR/generated.rs):
-- every src/generated/<name>.rs becomes `pub mod <name> { ... }` (an include!'d file cannot carry
-- inner attributes; each module block can).
function M.single(files)
  local names = {}
  for rel in pairs(files) do
    local n = rel:match("^src/generated/([%w_]+)%.rs$")
    if n and n ~= "mod" then names[#names + 1] = n end
  end
  table.sort(names)
  local out = { M.HEADER, "// 每個模組原本是 src/generated/<name>.rs；build.rs 以 include! 放進 `generated` 模組。\n" }
  for _, n in ipairs(names) do
    out[#out + 1] = string.format("\n#[allow(unused_parens)]\npub mod %s {\n%s}\n", n, files["src/generated/" .. n .. ".rs"])
  end
  return table.concat(out)
end

-- items.dll (M7): every item document generated on its own, wrapped as `pub mod <document>`;
-- `registry::abilities()` lists the items every document generates (an item the generator
-- cannot lower yet is left out: the shop shows it as not open yet). -> text, report rows
function M.items_single(root)
  local lfs = require("lfs")
  local P = M.load(root)
  local rules = P.rules()
  local dir = root .. "/docs/character_pipeline/items"
  local docs = {}
  for name in lfs.dir(dir) do if name:match("%.lua$") then docs[#docs + 1] = name:gsub("%.lua$", "") end end
  table.sort(docs)
  local out = { M.HEADER, "// items.dll：每份道具文件一個模組，模組裡是那份文件的 v3_rust.lua 產生碼。\n" }
  local regs, rows = {}, {}
  for _, name in ipairs(docs) do
    local doc = P.load_file(dir .. "/" .. name .. ".lua")
    local errs = doc and P.validate_doc(doc) or { "load failed" }
    if doc and #errs == 0 then
      local files, report = M.generate(doc, { root = root, rules = rules, package = "items" })
      local text = M.single(files):gsub("^" .. M.HEADER:gsub("%p", "%%%0"), "")
      out[#out + 1] = string.format("\n#[allow(unused_imports, unused_parens, dead_code)]\npub mod %s {\n%s}\n", name:lower(), text)
      regs[#regs + 1] = name:lower()
      for _, e in ipairs(report.entries) do rows[#rows + 1] = { doc = name, id = e.id, blockers = e.blockers } end
    else
      error("items: " .. name .. ": " .. tostring(errs[1]))
    end
  end
  out[#out + 1] = "\n/// The items of every document, in document order.\npub mod registry {\n    use ability_kit::*;\n\n"
    .. "    pub fn abilities() -> RVec<AbilityDefFFI> {\n        let mut all = RVec::new();\n"
  for _, n in ipairs(regs) do out[#out + 1] = string.format("        for a in super::%s::registry::abilities() { all.push(a); }\n", n) end
  out[#out + 1] = "        all\n    }\n\n    /// Items own no units.\n    pub fn units() -> RVec<UnitDef> { RVec::new() }\n}\n"
  return table.concat(out), rows
end

function M.run(root, out_dir, heroes, opts)
  opts = opts or {}
  local lfs = require("lfs")
  local function mkdir_p(p)
    local acc = ""
    for part in p:gsub("\\", "/"):gmatch("[^/]+") do
      acc = acc == "" and part or (acc .. "/" .. part)
      if not acc:match("^%a:$") and lfs.attributes(acc, "mode") ~= "directory" then lfs.mkdir(acc) end
    end
  end
  local P = M.load(root)
  local rules = P.rules()
  local names = C.item_names(root .. "/docs/character_pipeline/items")
  local reports, problems = {}, {}
  for _, h in ipairs(heroes) do
    local doc, err = P.load_file(h.path)
    if not doc then problems[#problems + 1] = h.package .. ": " .. tostring(err)
    else
      local errs = P.validate_doc(doc)
      if #errs > 0 then problems[#problems + 1] = h.package .. ": " .. errs[1]
      else
        local files, report = M.generate(doc, { root = root, rules = rules, names = names, package = h.package, unit_id = h.package })
        report.package = h.package
        report.listed = h.listed
        if opts.keep_files then report.files = files end
        report.dir = "heroes/" .. M.crate_dir(doc, h.package)
        if opts.single then
          local text = M.single(files)
          local fh = io.open(opts.single, "rb")
          local now = fh and fh:read("a")
          if fh then fh:close() end
          report.changed = 0
          if now ~= text then
            mkdir_p((opts.single:gsub("\\", "/"):match("^(.*)/") or "."))
            local w = assert(io.open(opts.single, "wb")); w:write(text); w:close()
            report.changed = 1
          end
        elseif opts.write ~= false then
          -- the crate's scaffold (only when missing); src/generated/ only with --tree (inspection)
          report.changed = M.write_crate(out_dir .. "/" .. report.dir, opts.tree and files or {}, lfs, mkdir_p, report.scaffold)
        end
        reports[#reports + 1] = report
        if opts.strict then
          for _, e in ipairs(report.entries) do
            if #e.blockers > 0 then problems[#problems + 1] = h.package .. " " .. e.id .. ": " .. table.concat(e.blockers, ", ") end
          end
        end
      end
    end
  end
  return reports, problems
end

function M.main(argv)
  local root = tools_dir:gsub("\\", "/"):match("^(.-)/?docs/character_pipeline/tools$")
  if not root or root == "" then root = "." end
  local out, which, picked, strict, single, tree, items = nil, "listed", {}, false, nil, false, false
  local i = 1
  while i <= #argv do
    local a = argv[i]
    if a == "--out" then out = argv[i + 1]; i = i + 1
    elseif a == "--hero" then picked[#picked + 1] = argv[i + 1]; i = i + 1
    elseif a == "--all" then which = "all"
    elseif a == "--listed" then which = "listed"
    elseif a == "--strict" then strict = true
    elseif a == "--single" then single = argv[i + 1]; i = i + 1
    elseif a == "--tree" then tree = true
    elseif a == "--items" then items = true
    else io.stderr:write("unknown argument " .. a .. "\n"); return 2 end
    i = i + 1
  end
  if items then
    -- items.dll (M7): every item document in one file
    if not single then io.stderr:write("--items needs --single FILE\n"); return 2 end
    local text, rows = M.items_single(root)
    local old = io.open(single, "rb")
    local same = old and old:read("a") == text
    if old then old:close() end
    if not same then local w = assert(io.open(single, "wb")); w:write(text); w:close() end
    local generated = 0
    for _, r in ipairs(rows) do if #r.blockers == 0 then generated = generated + 1 end end
    print(string.format("v3_rust: items, %d/%d entries generated -> %s", generated, #rows, single))
    return 0
  end
  out = out or (root .. "/scripts")
  local heroes = M.heroes(root, #picked > 0 and "all" or which)
  if #picked > 0 then
    local want = set(picked)
    local keep = {}
    for _, h in ipairs(heroes) do if want[h.package] then keep[#keep + 1] = h; want[h.package] = nil end end
    if single and #keep ~= 1 then io.stderr:write("--single takes exactly one --hero\n"); return 2 end
    for name in pairs(want) do io.stderr:write("no v3 document for " .. name .. "\n"); return 2 end
    heroes = keep
  end
  if single and #picked ~= 1 then io.stderr:write("--single takes exactly one --hero\n"); return 2 end
  local reports, problems = M.run(root, out, heroes, { strict = strict, single = single, tree = tree })
  local entries, generated, changed = 0, 0, 0
  for _, r in ipairs(reports) do
    changed = changed + (r.changed or 0)
    for _, e in ipairs(r.entries) do entries = entries + 1; if #e.blockers == 0 then generated = generated + 1 end end
  end
  print(string.format("v3_rust: %d crates, %d/%d entries generated, %d files written -> %s", #reports, generated, entries, changed,
    single or (out .. "/heroes")))
  for _, p in ipairs(problems) do io.stderr:write(p, "\n") end
  return #problems > 0 and 1 or 0
end

if not rawget(_G, "V3_RUST_AS_MODULE") and arg and arg[0] and arg[0]:match("v3_rust%.lua$") then
  os.exit(M.main(arg))
end
return M
