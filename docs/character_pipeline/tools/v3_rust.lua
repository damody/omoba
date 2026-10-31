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
  -- a summon template's passive.states (not a dummy: the shared dummy body has them): a hidden,
  -- permanent, undispellable status `<template>_passive` the creating code puts on the new unit
  -- with the template's statuses (line B: every state has a host vocab entry)
  for _, u in ipairs(doc.unit_templates or {}) do
    local p = u.passive or {}
    local dummy = false
    for _, tag in ipairs(u.tags or {}) do if tag == "dummy" then dummy = true end end
    if not dummy and type(p.states) == "table" and #p.states > 0 then
      local sd = { id = u.id .. "_passive", duration = "infinite", stacking = "keep", dispel = "none",
        visibility = "hidden", reveal = "public", states = p.states }
      statuses[sd.id] = add(sd, "status", shared)
      templates[u.id].passive_status = sd.id
    end
  end
  -- a document-level status a unit template carries is applied by the cast that creates the
  -- template (create_unit), so that ability is its host: the status_event marker names it and its
  -- on_status_event serves the status's triggers (line B, fifth batch item 9)
  local function creates(t, found)
    if type(t) ~= "table" then return end
    if t.op == "create_unit" and type(t.template) == "string" then found[t.template] = true end
    for _, v in pairs(t) do creates(v, found) end
  end
  for _, a in ipairs(doc.abilities or {}) do
    local found = {}
    creates(a.events, found); creates(a.passive, found); creates(a.statuses, found)
    for tid in pairs(found) do
      local u = templates[tid]
      for _, sid in ipairs(u and u.def.statuses or {}) do
        local so = type(sid) == "string" and statuses[sid]
        if so and so.parent == shared and not so.host then so.host = abilities[a.id] end
      end
    end
  end
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
    rows = rows, routines = doc.routines or {}, vars = vars, doc_id = doc.id, doc_statuses = doc.statuses or {} }
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
  -- the holder executed an ability or used an item (no event fields)
  on_ability_executed = { 128, { 9 } },
  -- the holder starts an attack (its windup); event.target is the attack's target
  on_attack_windup = { 256, { 10 } },
  -- the holder's hit killed event.victim (the packet's credit; omoba-core status_event KILL)
  on_kill = { 512, { 11 } },
  -- the holder's attack leaves (projectile out / melee strike; line B AttackEvent::Launch);
  -- event.target is its target
  on_attack_launch = { 1024, { 12 } },
  -- a status's absorb shield took part of a hit / ran out (status.absorb.on_absorb / on_broken,
  -- line B E393): event.other the damage's source, event.amount this hit's absorbed share
  on_absorb = { 2048, { 13 } }, on_broken = { 4096, { 14 } },
  -- the holder moved (walked, pushed or teleported) since the last status events: event.distance
  -- is the straight distance (line B E424)
  on_moved = { 8192, { 16 } },
  -- the holder spent mana (a cast's cost; line B E449): event.amount is how much
  on_mana_spent = { 16384, { 17 } },
  -- the holder gained health from a heal (line B E478): event.amount is what it gained (capped)
  on_heal_received = { 65536, { 19 } },
  -- applied again while it holds: the applying code runs it after the apply (no host event)
  on_refreshed = { 0 },
}
-- a status's triggers with its absorb shield's on_absorb / on_broken among them
local merged_triggers = setmetatable({}, { __mode = "k" })
local function team_call(n) return type(n) == "table" and n.k == "call" and (n.fn == "team_of" or n.fn == "enemy_team") and #(n.args or {}) == 1 end
local function triggers_of(def)
  local ab = type(def.absorb) == "table" and def.absorb or nil
  if not ab or (ab.on_absorb == nil and ab.on_broken == nil) then return def.triggers end
  if merged_triggers[def] then return merged_triggers[def] end
  local t = {}
  for k, v in pairs(def.triggers or {}) do t[k] = v end
  for _, k in ipairs({ "on_absorb", "on_broken" }) do
    if ab[k] ~= nil then t[k] = t[k] == nil and ab[k] or error(tostring(def.id) .. ": " .. k .. " in both absorb and triggers") end
  end
  merged_triggers[def] = t
  return t
end
M.STATUS_TRIGGERS = STATUS_TRIGGERS
-- `periodic`, `periodic_2`, ...: every periodic trigger of a status
local function trigger_kind(name) return name:match("^periodic_%d+$") and STATUS_TRIGGERS.periodic or STATUS_TRIGGERS[name] end
local function starts_on_apply(triggers, aura)
  -- a status aura's emitter is set up when its status starts
  if aura ~= nil then return true end
  for k in pairs(triggers or {}) do if k == "on_created" or k:match("^periodic") then return true end end
  return false
end
-- the event mask of a status' triggers (0: none the host calls back)
local function status_mask(def)
  if (def.runtime_id or ""):match("^attack_orb:") then return 0 end
  local mask = 0
  for name in pairs(triggers_of(def) or {}) do
    local t = STATUS_TRIGGERS[name]
    if t and t[1] ~= 0 and mask & t[1] == 0 then mask = mask + t[1] end
  end
  -- reapply_if_removed hears its own removal (status_event REMOVED, an end)
  if def.reapply_if_removed == true and mask & STATUS_TRIGGERS.on_removed[1] == 0 then mask = mask + STATUS_TRIGGERS.on_removed[1] end
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
-- A named delay's unit state key on its caster (instance and `:step`), per document.
function Gen:delay_key(id) return tostring(self.m.doc_id):lower() .. ":__delay_" .. id end
-- The entry whose program has the delay named `id` (its schedules run under that entry's ID; a
-- delay in a routine runs as part of the entry calling it); nil (blocked) when none or several.
function Gen:delay_entry(id)
  local holders = {}
  for name, r in pairs(self.m.routines) do holders[name] = r end
  local function has(x, seen)
    if type(x) ~= "table" or seen[x] then return false end
    seen[x] = true
    if x.op == "delay" and x.id == id then return true end
    if x.op == "call" and holders[x.routine] and has(holders[x.routine], seen) then return true end
    for _, v in pairs(x) do if has(v, seen) then return true end end
    return false
  end
  local found
  for _, e in ipairs(self.m.entries) do
    if has(e.def, {}) then
      if found then self:block("cancel_delay " .. id .. ": two entries delay it"); return nil end
      found = e.def.id
    end
  end
  if not found then self:block("cancel_delay " .. id .. ": no delay has that id") end
  return found
end
-- The rank a value of owner `o` is read at: the running entry's (cx.rank), or, for a value of another
-- ability of the document (its routine called from a different ability's hit, A34T's shred from A34W's
-- bolt, E388), the caster's rank of that ability; cx.rank when the caster has not learned it. That
-- rank is a `rk_<ability>` let at the top of each function using it (finish): a value sits inside
-- calls that already borrow cx.
function Gen:rank_of(o)
  local function ability(x) while x and x.kind == "status" do x = x.parent end return x end
  local a = ability(o)
  if not a or a.kind ~= "ability" or ability(self.entry) == a then return "cx.rank" end
  self.foreign_ranks = self.foreign_ranks or {}
  self.foreign_ranks[a.def.id] = true
  return "rk_" .. a.def.id
end

-- the `rk_<ability>` lets of rank_of, at the top of every function (or of a body fragment that
-- follows its `let mut cx`) naming them
local FN_HEAD = "fn [%w_]+%(cx: &mut Cx<'_, '_>[^\n]-%) %-> Res {\n"
function Gen:hoist_ranks(text)
  local heads, at = {}, 1
  while true do
    local _, b = text:find(FN_HEAD, at)
    if not b then break end
    heads[#heads + 1] = b; at = b + 1
  end
  if #heads == 0 then heads[1] = 0 end
  local out, prev = {}, 1
  for i, h in ipairs(heads) do
    local seg = text:sub(h + 1, (heads[i + 1] or #text + 1) - 1)
    local lets = {}
    for _, id in ipairs(sorted_keys(self.foreign_ranks)) do
      if seg:find("rk_" .. id .. "%f[^%w_]") then
        lets[#lets + 1] = "    let rk_" .. id .. " = cx.rank_of(cx.caster, " .. rust_str(id) .. ");\n"
      end
    end
    out[#out + 1] = text:sub(prev, h) .. table.concat(lets)
    prev = h + 1
  end
  return table.concat(out) .. text:sub(prev)
end
-- A status's start function lives in its parent entry's module: that module must be generated.
function Gen:need_module(so)
  local parent = so and so.parent
  if parent and (parent.kind == "ability" or parent.kind == "item") and not (self.entry and self.entry.def.id == parent.id) then
    self.modules_needed = self.modules_needed or {}
    self.modules_needed[parent.id] = so.def.id
  end
end
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
  -- a direction is a Vec2 as well: it travels as a point argument
  vec = { "ability_kit::arg_point(%s)", "ability_kit::arg_as_point(args, %d)" },
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
  if self.foreign_ranks then text = self:hoist_ranks(text) end
  for _, b in ipairs(self.lets) do
    local rust = (b.used and "" or "_") .. "l_" .. b.name:gsub("[^%w_]", "_")
    text = text:gsub(b.marker, rust)
  end
  return text
end

local function value_path(o, name, frac)
  return string.format("super::values::%s::%s%s", o.id, upper(name), frac and "_FRAC" or "")
end

-- a status var's unit var on the holder, and its expression type
local VAR_TYPES = { fixed = "num", int = "num", bool = "bool", unit = "unit", point = "point" }
-- Rust that reads / writes a status var of `ty` on `holder` (a point is two num vars `:x` / `:y`)
local function var_read(ty, holder, key, init)
  if ty == "unit" then return string.format("cx.state_unit(%s, %s)", holder, rust_str(key)) end
  if ty == "point" then
    return string.format("{ let i: Vec2 = %s; Vec2::new(cx.state_num(%s, %s, i.x), cx.state_num(%s, %s, i.y)) }",
      init, holder, rust_str(key .. ":x"), holder, rust_str(key .. ":y"))
  end
  return string.format("cx.state_%s(%s, %s, %s)", ty, holder, rust_str(key), init)
end
local function var_write(ty, holder, key, value)
  if ty == "unit" then return string.format("cx.set_state_unit(%s, %s, %s, false)?;", holder, rust_str(key), value) end
  if ty == "point" then
    return string.format("{ let v: Vec2 = %s; cx.set_state_num(%s, %s, v.x, false)?; cx.set_state_num(%s, %s, v.y, false)?; }",
      value, holder, rust_str(key .. ":x"), holder, rust_str(key .. ":y"))
  end
  return string.format("cx.set_state_%s(%s, %s, %s, false)?;", ty, holder, rust_str(key), value)
end
local function var_zero(ty) return ty == "bool" and "false" or ty == "point" and "Vec2::ZERO" or "Fixed64::ZERO" end
local function status_var_key(status_id, var) return "__sv:" .. status_id .. ":" .. var end
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
-- the attributes the host reads through GameWorld::ext UNIT_ATTR (line B; names in
-- omb_script_abi::ext::unit_attr): v3 name -> { type, kit read, unit_attr constant }
do
  local EXT = {
    level = { "num", "unit_num", "LEVEL" }, str = { "num", "unit_num", "STR" }, agi = { "num", "unit_num", "AGI" },
    int = { "num", "unit_num", "INT" }, max_mana = { "num", "unit_num", "MAX_MANA" },
    attack_speed_mult = { "num", "unit_num", "ATTACK_SPEED_MULT" }, projectile_speed = { "num", "unit_num", "PROJECTILE_SPEED" },
    base_magic_resist = { "num", "unit_num", "BASE_MAGIC_RESIST" }, base_attack_range = { "num", "unit_num", "BASE_ATTACK_RANGE" },
    is_illusion = { "bool", "unit_flag", "IS_ILLUSION" }, is_invulnerable = { "bool", "unit_flag", "IS_INVULNERABLE" },
    is_invisible = { "bool", "unit_flag", "IS_INVISIBLE" }, is_creep = { "bool", "unit_flag", "IS_CREEP" },
    is_ranged = { "bool", "unit_flag", "IS_RANGED" },
    -- neutral creeps and neutral units (line B E447)
    is_neutral = { "bool", "unit_flag", "IS_NEUTRAL" },
    owner = { "unit", "unit_ref", "OWNER" }, attack_target = { "unit", "unit_ref", "ATTACK_TARGET" },
    -- the player's single gold pool (a summon reads its owner's; line-gamemode E243)
    gold = { "num", "unit_num", "GOLD" },
    -- armor above the base (line B ee0292ee)
    bonus_armor = { "num", "unit_num", "BONUS_ARMOR" },
  }
  for name, e in pairs(EXT) do
    -- the operand first: a read takes &mut cx
    UNIT_ATTRS[name] = { e[1], "{ let u = %s; cx." .. e[2] .. "(u, ability_kit::unit_attr::" .. e[3] .. ") }" }
  end
  -- mana / max mana (0 without a pool), like hp_pct
  UNIT_ATTRS.mana_pct = { "num", "{ let u = %s; let max = cx.unit_num(u, ability_kit::unit_attr::MAX_MANA); if max > Fixed64::ZERO { cx.mana(u) / max } else { Fixed64::ZERO } }" }
  UNIT_ATTRS.missing_mana = { "num", "{ let u = %s; cx.unit_num(u, ability_kit::unit_attr::MAX_MANA) - cx.mana(u) }" }
  -- seconds between attacks after attack speed (GameWorld::get_asd_interval)
  UNIT_ATTRS.attack_interval = { "num", "{ let u = %s; cx.world.get_asd_interval(u) }" }
end

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
      return value_path(o, node.name, d.unit == "pct") .. ".at(" .. self:rank_of(o) .. ")", "num"
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
    -- the summon's owner hero; a hero is its own (v3.md §5.4)
    elseif n == "caster_hero" then
      return "{ let c = cx.caster; let o = cx.unit_ref(c, ability_kit::unit_attr::OWNER); if o.is_valid() { o } else { c } }", "unit"
    elseif n == "target" then
      -- a tree target is the standing tree's centre (the host snaps the cast point to it, E403)
      local t = self.entry and self.entry.def.target
      if t and t.type == "tree" then return "cx.target_point()?", "point" end
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
    -- `game.*` facts (GameWorld::ext GAME_ATTR)
    if base.k == "id" and base.name == "game" and #parts == 1 and not scope_find(ctx.scope, "game") then
      if parts[1] == "is_night" then return "cx.is_night()", "bool" end
      if parts[1] == "time" then return "cx.game_time()", "num" end
      if parts[1] == "dt" then return "cx.game_dt()", "num" end
      self:block("attr:game." .. parts[1]); return "Fixed64::ZERO", "err"
    end
    -- the running ability (v3.md §5.4: always this entry, in a trigger too): its rank is cx.rank in
    -- every generated context (a trigger's event level); readiness by its id on cx.caster (the
    -- learner of a passive, the applier of a status)
    if base.k == "id" and base.name == "ability" and #parts == 1 and not scope_find(ctx.scope, "ability") then
      if parts[1] == "level" then return "Fixed64::from_i32(i32::from(cx.rank))", "num" end
      -- the hero that owns this ability: the caster, or the hero owning the summon that runs it
      -- (like caster_hero)
      if parts[1] == "owner" then
        return "{ let c = cx.caster; let o = cx.unit_ref(c, ability_kit::unit_attr::OWNER); if o.is_valid() { o } else { c } }", "unit"
      end
      local id = self.entry and self.entry.kind == "ability" and self.entry.def.id
      if parts[1] == "is_ready" and id then return string.format("cx.ability_ready(cx.caster, %s)", rust_str(id)), "bool" end
      -- this rank's mana cost (`cost.mana`, GetManaCost)
      if parts[1] == "mana_cost" and id and self.entry.def.cost and self.entry.def.cost.mana ~= nil then
        return self:value(self.entry.def.cost.mana, { owner = self.entry, scope = scope_child(nil), event = false }, "num"), "num"
      end
      -- the cast range of this rank plus the bonuses the caster holds (原作 GetCastRange)
      if parts[1] == "cast_range" then
        return "(cx.cast_range.unwrap_or(Fixed64::ZERO) + { let u = cx.caster; cx.unit_num(u, ability_kit::unit_attr::CAST_RANGE_BONUS) })", "num"
      end
    end
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
      local ty = ({ int = "num", fixed = "num", bool = "bool", unit = "unit", point = "point", vec = "vec" })[v.def.type]
      if not ty then self:block("state type:" .. tostring(v.def.type)); return "Fixed64::ZERO", "err" end
      if ty == "unit" then return string.format("cx.state_unit(%s, %s)", holder, rust_str(v.key)), "unit" end
      -- a point (or vector) is two num vars `<key>:x` / `<key>:y` (like a status var)
      if ty == "point" or ty == "vec" then
        local init = v.def.init ~= nil and self:value(v.def.init, ctx, ty) or "Vec2::ZERO"
        return string.format("{ let h = %s; %s }", holder, var_read("point", "h", v.key, init)), ty
      end
      local init = v.def.init ~= nil and self:value(v.def.init, ctx, ty) or (ty == "bool" and "false" or "Fixed64::ZERO")
      return string.format("cx.state_%s(%s, %s, %s)", ty, holder, rust_str(v.key), init), ty
    end
    if base.k == "id" and base.name == "vars" and #parts == 1 and not scope_find(ctx.scope, "vars") then
      local so = ctx.owner
      local v = so and so.kind == "status" and so.def.vars and so.def.vars[parts[1]]
      local ty = v and VAR_TYPES[v.type]
      if not ty then self:block("token:vars (" .. parts[1] .. " outside its status)"); return "Fixed64::ZERO", "err" end
      if so.def.stacking == "independent" then self:block("status.vars of an independent status"); return "Fixed64::ZERO", "err" end
      local key = status_var_key(so.def.id, parts[1])
      local init = ty ~= "unit" and (v.init ~= nil and self:value(v.init, ctx, ty) or var_zero(ty)) or nil
      return var_read(ty, "cx.self_unit", key, init), ty
    end
    -- a status trigger: the stacks of its own status on its holder
    if base.k == "id" and base.name == "status" and #parts == 1 and parts[1] == "stacks" and not scope_find(ctx.scope, "status")
      and ctx.owner and ctx.owner.kind == "status" then
      if ctx.owner.def.stacking == "independent" then self:block("status.stacks of an independent status"); return "Fixed64::ZERO", "err" end
      return string.format("cx.stacks(cx.self_unit, &super::ids::status::%s)", upper(ctx.owner.def.id)), "num"
    end
    -- an end trigger of a status: the duration it had left (the host's end event carries it)
    if base.k == "id" and base.name == "status" and #parts == 1 and parts[1] == "remaining" and not scope_find(ctx.scope, "status") then
      if not ctx.ended then self:block("attr:status.remaining outside an end trigger"); return "Fixed64::ZERO", "err" end
      return "cx.event_amount", "num"
    end
    -- `state.x.attr` / `vars.x.attr`: the var (above) and then its attribute, like any unit
    if base.k == "id" and (base.name == "state" or base.name == "vars") and #parts == 1 and not scope_find(ctx.scope, base.name) then
      self:block("token:" .. base.name); return "Fixed64::ZERO", "err"
    end
    local obj, ot = self:expr(node.obj, ctx)
    if ot == "err" then return obj, "err" end
    if ot == "ability_ref" then
      local u, id = obj:match("^(.*)%z([%w_]+)$")
      local reads = { cooldown_remaining = { "cx.ability_cooldown(u, %s)", "num" }, is_ready = { "cx.ability_ready(u, %s)", "bool" },
        level = { "cx.ability_level(u, %s)", "num" }, cooldown = { "cx.ability_cooldown_total(u, %s)", "num" } }
      local r = reads[node.name]
      if not r then self:block("attr:" .. node.name .. " of ability_of"); return "Fixed64::ZERO", "err" end
      return string.format("{ let u = %s; " .. r[1] .. " }", u, rust_str(id)), r[2]
    end
    if ot == "event_ability" then
      if node.name == "is_toggle" then return "cx.event_is_toggle()", "bool" end
      if node.name == "is_item" then return "cx.event_is_any_item()", "bool" end
      -- the cast point at the caster's rank (0 for an item): the host tells it to a status trigger
      if node.name == "cast_point" and ctx.event == "ability_executed" then return "cx.event_ability_cast_point()", "num" end
      self:block("attr:" .. node.name .. " of an ability"); return "Fixed64::ZERO", "err"
    end
    if ot == "event" and (ctx.event == "channel_tick" or ctx.event == "channel_end") then
      if ctx.event == "channel_tick" and node.name == "tick" then return "cx.event_amount", "num" end
      if ctx.event == "channel_end" and node.name == "interrupted" then return "(cx.event_amount != Fixed64::ZERO)", "bool" end
      self:block("event:" .. node.name .. " of a channel"); return "Fixed64::ZERO", "err"
    end
    -- a status's on_moved tells only how far (line B E424)
    if ot == "event" and ctx.event == "moved" then
      if node.name == "distance" then return "cx.event_amount", "num" end
      self:block("event:" .. node.name .. " of a status on_moved"); return "Fixed64::ZERO", "err"
    end
    -- a status's on_mana_spent / on_heal_received tells only how much (line B E449, E478)
    if ot == "event" and (ctx.event == "mana_spent" or ctx.event == "heal_received") then
      if node.name == "amount" then return "cx.event_amount", "num" end
      self:block("event:" .. node.name .. " of a status on_" .. ctx.event); return "Fixed64::ZERO", "err"
    end
    -- a status's on_ability_executed tells only the skill (line B 94d609ce)
    if ot == "event" and ctx.event == "ability_executed" then
      if node.name == "ability" then return "", "event_ability" end
      if node.name == "point" then return "cx.event_cast_point()?", "point" end
      self:block("event:" .. node.name .. " of a status on_ability_executed"); return "EntityHandle::INVALID", "unit"
    end
    if ot == "event" then
      if node.name == "target" then return "cx.event_target", "unit"
      elseif node.name == "attacker" then return "cx.event_attacker", "unit"
      elseif node.name == "ability" then return "", "event_ability"
      elseif node.name == "damage" then return "", "event_damage"
      elseif node.name == "attack" then return "", "event_attack"
      elseif node.name == "victim" then return "cx.event_target", "unit"
      -- the cast's point (an owner's passive on_ability_executed: on_owner_cast's target)
      elseif node.name == "point" and ctx.event == "owner_cast" then return "cx.event_cast_point()?", "point" end
      self:block("event:" .. node.name); return "EntityHandle::INVALID", "unit"
    end
    if ot == "event_attack" then
      -- the attack of this trigger: the attacker's (on_attacked), else the holder's own (line B
      -- rules::attack AttackRecords through unit_attr)
      local u = "(if cx.event_attacker.is_valid() { cx.event_attacker } else { cx.self_unit })"
      local reads = { damage = { "unit_num", "ATTACK_DAMAGE", "num" }, effect_multiplier = { "unit_num", "ATTACK_EFFECT_MULTIPLIER", "num" },
        is_ranged = { "unit_flag", "IS_RANGED", "bool" } }
      local r = reads[node.name]
      if not r then self:block("event.attack." .. node.name); return "Fixed64::ZERO", "err" end
      return string.format("{ let u = %s; cx.%s(u, ability_kit::unit_attr::%s) }", u, r[1], r[2]), r[3]
    end
    if ot == "event_damage" then
      if node.name == "final" then return "cx.event_amount", "num"
      -- before it lands the host packet already has armor and resistance (rules::damage ③ runs at
      -- the raw packet's conversion): `raw` / `after_armor` read that amount (conformance checks it)
      elseif ctx.pre and (node.name == "raw" or node.name == "after_armor") then return "cx.event_amount", "num"
      -- after it landed: the hit's total before the target's mitigation (line B E478)
      elseif node.name == "raw" then return "cx.event_damage_raw()", "num"
      -- the kind is the low nibble; 0x10 marks a reflected hit (StatusEvent::REFLECTION)
      elseif node.name == "is_physical" then return "((cx.event_damage_kind & 0x0f) == 0)", "bool"
      elseif node.name == "is_magical" then return "((cx.event_damage_kind & 0x0f) == 1)", "bool"
      elseif node.name == "is_pure" then return "((cx.event_damage_kind & 0x0f) == 2)", "bool"
      elseif node.name == "is_reflection" then return "((cx.event_damage_kind & 0x10) != 0)", "bool"
      -- a basic attack's damage, melee or projectile (StatusEvent::ATTACK, line B E475)
      elseif node.name == "is_attack" then return "((cx.event_damage_kind & 0x40) != 0)", "bool"
      -- it killed (StatusEvent::LETHAL, line B E452); before it lands: it would (final >= hp)
      elseif node.name == "lethal" and ctx.pre then return "(cx.event_amount >= cx.hp(cx.self_unit))", "bool"
      elseif node.name == "lethal" then return "((cx.event_damage_kind & 0x20) != 0)", "bool" end
      self:block("event.damage." .. node.name); return "Fixed64::ZERO", "err"
    end
    if ot == "unit" then
      local a = UNIT_ATTRS[node.name]
      if a then
        if obj:match("^[%w_%.]+$") then return (a[2]:gsub("%%s", function() return obj end)), a[1] end
        return "{ let ua_ = " .. obj .. "; " .. (a[2]:gsub("%%s", function() return "ua_" end)) .. " }", a[1]
      end
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
  elseif k == "bin" and (node.op == "==" or node.op == "~=") and team_call(node.l) and team_call(node.r) then
    -- team_of(a) == team_of(b): allies; team_of(a) == enemy_team(b): a is on b's opposing player
    -- team (an enemy that is not neutral)
    local l, r = node.l, node.r
    if l.fn == "enemy_team" and r.fn == "team_of" then l, r = r, l end
    local a, at = self:expr(l.args[1], ctx)
    local b, bt = self:expr(r.args[1], ctx)
    if at == "err" or bt == "err" then return "false", "err" end
    if at ~= "unit" or bt ~= "unit" then self:block("fn:team_of of a non-unit"); return "false", "err" end
    local code
    if l.fn == "team_of" and r.fn == "team_of" then code = string.format("cx.is_ally(%s, %s)", a, b)
    elseif l.fn == "team_of" and r.fn == "enemy_team" then
      code = string.format("{ let a = %s; let b = %s; cx.is_enemy(a, b) && !cx.unit_flag(a, ability_kit::unit_attr::IS_NEUTRAL) }", a, b)
    else self:block("expr: enemy_team compared with enemy_team"); return "false", "err" end
    return node.op == "==" and ("(" .. code .. ")") or ("(!" .. code .. ")"), "bool"
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
      -- a right side that takes &mut cx (a block): the left value first, so no borrow of cx is held
      if r:find("{", 1, true) then return string.format("{ let l_ = %s; l_ %s %s }", l, op == "==" and "==" or "!=", r), "bool" end
      return string.format("(%s %s %s)", l, op == "==" and "==" or "!=", r), "bool"
    end
    -- vectors (Vec2): a point moved by an offset, the offset between points, an offset scaled
    local spatial = { point = true, vec = true }
    if spatial[lt] or spatial[rt] then
      local ty
      if op == "+" and spatial[lt] and spatial[rt] and not (lt == "point" and rt == "point") then
        ty = (lt == "point" or rt == "point") and "point" or "vec"
      elseif op == "-" and spatial[lt] and rt == "vec" then ty = lt
      elseif op == "-" and lt == "point" and rt == "point" then ty = "vec"
      elseif op == "*" and lt == "vec" and rt == "num" then ty = "vec"
      elseif op == "*" and lt == "num" and rt == "vec" then return string.format("(%s * %s)", r, l), "vec" end
      if not ty then self:block("expr: " .. op .. " of " .. lt .. " and " .. rt); return "Fixed64::ZERO", "err" end
      return string.format("(%s %s %s)", l, op, r), ty
    end
    if lt ~= "num" or rt ~= "num" then self:block("expr: " .. op .. " of " .. lt .. " and " .. rt); return "Fixed64::ZERO", "err" end
    if op == "<" or op == "<=" or op == ">" or op == ">=" then return string.format("(%s %s %s)", l, op, r), "bool" end
    if op == "+" or op == "-" or op == "*" then return string.format("(%s %s %s)", l, op, r), "num" end
    if op == "/" then return string.format("ability_kit::div(%s, %s)?", l, r), "num" end
    if op == "%" then return string.format("ability_kit::rem(%s, %s)?", l, r), "num" end
    self:block("op:" .. op); return "Fixed64::ZERO", "err"
  elseif k == "call" then
    return self:call(node, ctx)
  end
  self:block("expr:" .. tostring(k)); return "Fixed64::ZERO", "err"
end

-- `cx.<method>(..)` with unit arguments that may read through cx themselves (a borrow of cx
-- inside the call's own borrow, E285): every argument that is not a plain name is bound first
local function cx_call(fmt, ...)
  local names, lets = {}, {}
  for i, a in ipairs({ ... }) do
    if a:match("^[%w_%.]+$") then names[i] = a
    else names[i] = "a" .. i .. "_"; lets[#lets + 1] = "let " .. names[i] .. " = " .. a .. "; " end
  end
  local call = string.format(fmt, table.unpack(names))
  if #lets == 0 then return call end
  return "{ " .. table.concat(lets) .. call .. " }"
end

function Gen:point(code, ty)
  if ty == "point" or ty == "err" then return code end
  if ty == "unit" then return cx_call("cx.pos(%s)?", code) end
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
  if fn == "armor_multiplier" then
    -- the share of physical damage that lands through this armor (the rule layer's one formula)
    local x, t = arg(1)
    if t == "err" then return "Fixed64::ZERO", "err" end
    if t ~= "num" then self:block("fn:armor_multiplier of a " .. t); return "Fixed64::ZERO", "err" end
    return "ability_kit::armor_multiplier(" .. x .. ")", "num"
  end
  if fn == "min" or fn == "max" then
    if #args ~= 2 then self:block("fn:" .. fn .. "/" .. #args) end
    local a, at = arg(1); local b, bt = arg(2)
    if at ~= "num" or bt ~= "num" then self:block("fn:" .. fn .. " of non-numbers") end
    return string.format("%s.%s(%s)", a, fn, b), "num"
  elseif fn == "select" then
    local c, ct = arg(1); local a, at = arg(2); local b, bt = arg(3)
    if at == "err" or bt == "err" or ct == "err" then return "Fixed64::ZERO", "err" end
    -- a unit and a point: the unit stands for its position
    if (at == "unit" and bt == "point") or (at == "point" and bt == "unit") then
      a, b, at, bt = self:point(a, at), self:point(b, bt), "point", "point"
    end
    if ct ~= "bool" or at ~= bt then self:block("fn:select types") end
    return string.format("(if %s { %s } else { %s })", c, a, b), at
  elseif fn == "floor" or fn == "ceil" or fn == "abs" then
    local x, t = arg(1)
    if t ~= "num" then self:block("fn:" .. fn .. " of a " .. t) end
    return string.format("ability_kit::%s(%s)", fn, x), "num"
  elseif fn == "clamp" then
    local x = self:value_node(args[1], ctx, "num"); local lo = self:value_node(args[2], ctx, "num"); local hi = self:value_node(args[3], ctx, "num")
    return string.format("ability_kit::clamp(%s, %s, %s)", x, lo, hi), "num"
  elseif fn == "has_status_tag" then
    -- some status on the unit carries the catalog tag (host dispel registry)
    local u, t = arg(1)
    local tag = name_arg(2)
    if t == "err" or not tag then return "false", "bool" end
    if t ~= "unit" then self:block("fn:has_status_tag of a " .. t); return "false", "bool" end
    return string.format("{ let u = %s; cx.has_status_tagged(u, %s) }", u, rust_str(tag)), "bool"
  elseif fn == "owner_of" then
    -- a unit's owner (`u.owner`: a summon's summoner, the controlling hero)
    local u, t = arg(1)
    if t == "err" then return "EntityHandle::INVALID", "err" end
    if t ~= "unit" then self:block("fn:owner_of of a " .. t); return "EntityHandle::INVALID", "err" end
    return string.format("{ let u = %s; cx.unit_ref(u, ability_kit::unit_attr::OWNER) }", u), "unit"
  elseif fn == "is_template" then
    local u, t = arg(1)
    local name = name_arg(2)
    if t == "err" or not name then return "false", "bool" end
    if t ~= "unit" then self:block("fn:is_template of a " .. t); return "false", "bool" end
    if not self.m.templates[name] then self:block("template:" .. name .. " not declared"); return "false", "bool" end
    return cx_call("cx.is_template(%s, " .. rust_str(name) .. ")", u), "bool"
  elseif fn == "exists" then
    local u, t = arg(1)
    if t == "err" then return "false", "bool" end
    if t ~= "unit" then self:block("fn:exists of a " .. t); return "false", "bool" end
    return cx_call("cx.exists(%s)", u), "bool"
  elseif fn == "has_state" then
    local u, t = arg(1)
    local st = name_arg(2)
    local e = st and self.opts.vocab.states[st]
    if t ~= "unit" or not e or not e.buff then self:block("fn:has_state:" .. tostring(st)); return "false", "bool" end
    return cx_call("cx.has_buff(%s, " .. rust_str(e.buff) .. ")", u), "bool"
  elseif fn == "alive" then
    local u, t = arg(1)
    if t ~= "unit" then self:block("fn:alive of a " .. t) end
    return cx_call("cx.alive(%s)", u), "bool"
  elseif fn == "is_enemy" then
    local a = arg(1); local b = arg(2)
    return cx_call("cx.is_enemy(%s, %s)", a, b), "bool"
  elseif fn == "avg_attack_against" then
    -- the attacker's attack as it leaves, without a critical (unit attr AVG_ATTACK, line B
    -- ee0292ee); the target changes nothing there: armor and building modifiers apply at the hit
    local u, ut = arg(1); local _, tt = arg(2)
    if ut == "err" or tt == "err" then return "Fixed64::ZERO", "err" end
    if ut ~= "unit" or tt ~= "unit" then self:block("fn:avg_attack_against of a " .. ut .. " and a " .. tt); return "Fixed64::ZERO", "err" end
    return cx_call("cx.unit_num(%s, ability_kit::unit_attr::AVG_ATTACK)", u), "num"
  elseif fn == "is_visible_to" then
    -- whether u's team sees x now (host IS_VISIBLE_TO: its own units always; an enemy team by
    -- the EnemySight disclosure, the same on the authority and every replica)
    local x, xt = arg(1); local u, ut = arg(2)
    if xt == "err" or ut == "err" then return "false", "bool" end
    if xt ~= "unit" or ut ~= "unit" then self:block("fn:is_visible_to of a " .. xt .. " and a " .. ut); return "false", "bool" end
    return cx_call("cx.is_visible_to(%s, %s)", x, u), "bool"
  elseif fn == "is_ally" then
    local a, at = arg(1); local b, bt = arg(2)
    if at == "err" or bt == "err" then return "false", "bool" end
    if at ~= "unit" or bt ~= "unit" then self:block("fn:is_ally of a " .. at .. " and a " .. bt); return "false", "bool" end
    return string.format("{ let a = %s; let b = %s; cx.is_ally(a, b) }", a, b), "bool"
  elseif fn == "edge_distance" then
    local a, at = arg(1); local b, bt = arg(2)
    if at == "err" or bt == "err" then return "Fixed64::ZERO", "err" end
    if at ~= "unit" or bt ~= "unit" then self:block("fn:edge_distance of a " .. at .. " and a " .. bt); return "Fixed64::ZERO", "err" end
    return string.format("{ let a = %s; let b = %s; cx.edge_distance(a, b)? }", a, b), "num"
  elseif fn == "distance" then
    local a, at = arg(1); local b, bt = arg(2)
    return string.format("ability_kit::distance(%s, %s)", self:point(a, at), self:point(b, bt)), "num"
  elseif fn == "angle_between" then
    -- the unsigned angle between two directions, in degrees
    local a, at = arg(1); local b, bt = arg(2)
    if at == "err" or bt == "err" then return "Fixed64::ZERO", "err" end
    if at ~= "vec" or bt ~= "vec" then self:block("fn:angle_between of a " .. at .. " and a " .. bt); return "Fixed64::ZERO", "err" end
    return cx_call("ability_kit::angle_between(%s, %s)", a, b), "num"
  elseif fn == "direction" then
    local a, at = arg(1); local b, bt = arg(2)
    return string.format("ability_kit::direction(%s, %s)", self:point(a, at), self:point(b, bt)), "vec"
  elseif fn == "offset" then
    local p, pt = arg(1); local d, dt = arg(2); local n, nt = arg(3)
    if pt == "err" or dt == "err" or nt == "err" then return "Vec2::ZERO", "err" end
    -- a direction kept as a point (a state var or a let of a direction) is the same Vec2
    if (dt ~= "vec" and dt ~= "point") or nt ~= "num" then self:block("fn:offset types") end
    return string.format("ability_kit::offset(%s, %s, %s)", self:point(p, pt), d, n), "point"
  elseif fn == "ability_of" then
    -- read through its attributes (`ability_of(caster, a34_frost_bolt).cooldown_remaining`)
    local u, ut = arg(1)
    local ab = args[2]
    if ut == "err" then return "", "err" end
    if ut ~= "unit" or not (ab and ab.k == "id" and not scope_find(ctx.scope, ab.name)) then self:block("fn:ability_of"); return "", "err" end
    return u .. "\0" .. ab.name, "ability_ref"
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
    return cx_call("cx.status_remaining(%s, &super::ids::status::" .. upper(sname) .. ")", u), "num"
  elseif fn == "status_var" then
    -- a var of a status on a unit (apply_status `vars`); its init while the unit lacks the status
    local u, ut = arg(1)
    local sname, vname = name_arg(2), name_arg(3)
    if ut == "err" or not sname or not vname then return "Fixed64::ZERO", "err" end
    local so = self.m.statuses[sname]
    local v = so and so.def.vars and so.def.vars[vname]
    local ty = v and VAR_TYPES[v.type]
    if ut ~= "unit" or not ty then self:block("fn:status_var"); return "Fixed64::ZERO", "err" end
    if so.def.stacking == "independent" then self:block("status.vars of an independent status"); return "Fixed64::ZERO", "err" end
    local sctx = { owner = so, scope = scope_child(nil), event = false }
    local init = ty == "unit" and "EntityHandle::INVALID" or (v.init ~= nil and self:value(v.init, sctx, ty) or var_zero(ty))
    return string.format("{ let u = %s; if cx.has_status(u, &super::ids::status::%s) { %s } else { %s } }", u, upper(sname),
      var_read(ty, "u", status_var_key(so.def.id, vname), ty == "unit" and nil or init), init), ty
  elseif fn == "rotate" then
    local v, vt = arg(1); local d = self:value_node(args[2], ctx, "num")
    if vt == "err" then return "Vec2::ZERO", "err" end
    if vt ~= "vec" and vt ~= "point" then self:block("fn:rotate of a " .. vt); return "Vec2::ZERO", "err" end
    return string.format("{ let v = %s; let d = %s; ability_kit::rotate(v, d) }", v, d), vt
  elseif fn == "random_int" then
    local lo = self:value_node(args[1], ctx, "num"); local hi = self:value_node(args[2], ctx, "num")
    return string.format("{ let lo = %s; let hi = %s; cx.random_int(lo, hi) }", lo, hi), "num"
  elseif fn == "random" then
    local lo = self:value_node(args[1], ctx, "num"); local hi = self:value_node(args[2], ctx, "num")
    return string.format("{ let lo = %s; let hi = %s; cx.random(lo, hi) }", lo, hi), "num"
  elseif fn == "stacks" then
    local u, ut = arg(1)
    local sname = name_arg(2)
    if ut == "err" then return "Fixed64::ZERO", "err" end
    if ut ~= "unit" or not sname or not self.m.statuses[sname] then self:block("fn:stacks"); return "Fixed64::ZERO", "err" end
    return cx_call("cx.stacks(%s, &super::ids::status::" .. upper(sname) .. ")", u), "num"
  elseif fn == "polar" then
    local p, pt = arg(1); local ang = self:value_node(args[2], ctx, "num"); local d = self:value_node(args[3], ctx, "num")
    return string.format("ability_kit::polar(%s, %s, %s)", self:point(p, pt), ang, d), "point"
  elseif fn == "sin" or fn == "cos" then
    return string.format("ability_kit::%s_deg(%s)", fn, self:value_node(args[1], ctx, "num")), "num"
  elseif fn == "circle_point" then
    -- point i of n equal steps around the circle, from the +x axis (v3.md §5.6)
    local p, pt = arg(1)
    local r = self:value_node(args[2], ctx, "num"); local n = self:value_node(args[3], ctx, "num"); local i = self:value_node(args[4], ctx, "num")
    return string.format("{ let c = %s; let r = %s; let n = %s; let i = %s; ability_kit::polar(c, ability_kit::div(Fixed64::from_i32(360) * i, n)?, r) }",
      self:point(p, pt), r, n, i), "point"
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
  elseif fn == "tree_alive" then
    -- whether the tree at that point (a tree target's centre) still stands (line B E403)
    local p, pt = arg(1)
    return "{ let at = " .. self:point(p, pt) .. "; cx.tree_alive(at) }", "bool"
  elseif fn == "count_in_radius" then
    local c, ct = arg(1)
    local r, rt = arg(2)
    if rt ~= "num" and rt ~= "err" then self:block("fn:count_in_radius radius of a " .. rt) end
    local tm, class = args[3], name_arg(4)
    local CLASSES = { hero = true, creep = true, building = true, summon = true, neutral = true }
    if not (tm and tm.k == "call" and (tm.fn == "team_of" or tm.fn == "enemy_team") and #tm.args == 1) then
      self:block("fn:count_in_radius of a team other than team_of／enemy_team"); return "Fixed64::ZERO", "err"
    end
    if not class or not CLASSES[class] then self:block("fn:count_in_radius of class " .. tostring(class)); return "Fixed64::ZERO", "err" end
    local of, ot = self:expr(tm.args[1], ctx)
    if ot ~= "unit" then self:block("fn:" .. tm.fn .. " of a " .. ot); return "Fixed64::ZERO", "err" end
    return string.format("{ let center = %s; let radius = %s; let of = %s; cx.count_in_radius(center, radius, of, %s, %s) }",
      self:point(c, ct), r, of, rust_str(tm.fn == "team_of" and "ally" or "enemy"), rust_str(class)), "num"
  elseif fn == "has_status" then
    local u, ut = arg(1)
    local s = name_arg(2)
    if ut ~= "unit" or not s then return "false", "bool" end
    if not self.m.statuses[s] then self:block("status:" .. s .. " not declared"); return "false", "bool" end
    if self.m.statuses[s].def.stacking == "independent" then
      -- any live instance (line B: cx.status_instances)
      local u = arg(1)
      return string.format("({ let u = %s; cx.status_instances(u, &super::ids::status::%s) } > 0)", u, upper(s)), "bool"
    end
    return cx_call("cx.has_status(%s, &super::ids::status::" .. upper(s) .. ")", u), "bool"
  elseif fn == "has_tag" or fn == "is_item" or fn == "is_ability" then
    local u, at = arg(1)
    local n = name_arg(2)
    if fn == "has_tag" and at == "unit" and n then
      -- a unit class the host already tells; boss / dummy / fountain / general through the host's
      -- unit tags (GameWorld::ext TAG_PREFIX; this game has no fountain or general units)
      local reads = { building = "cx.is_building(%s)", hero = "cx.is_hero(%s)", creep = "cx.unit_flag(%s, ability_kit::unit_attr::IS_CREEP)" }
      if reads[n] then return "{ let u = " .. u .. "; " .. reads[n]:format("u") .. " }", "bool" end
      local HOST_TAGS = { boss = true, dummy = true, fountain = true, general = true, summon = true, neutral = true }
      if HOST_TAGS[n] then return "{ let u = " .. u .. "; cx.has_unit_tag(u, " .. rust_str(n) .. ") }", "bool" end
      self:block("fn:has_tag:" .. n .. " of a unit"); return "false", "bool"
    end
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
  "stats", "per_source", "states", "tags", "triggers", "text", "notes", "values", "vars", "absorb", "status_resistance", "aura",
  "unique_group", "unique_rule", "scope", "reapply_if_removed",
  -- immune_to: the host refuses the matching statuses from the catalog (line B E445, E448)
  "immune_to",
  -- forced_move: its apply_status also calls cx.force_move (line B E464)
  "forced_move" })

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
    -- a passive's learn marker is its ability id (passive_stats, `learned`): a status under that
    -- row would replace the marker and look applied whenever the passive is learned (E268)
    do
      local ab = m.abilities[runtime_id] and m.abilities[runtime_id].def
      if ab and ab.passive then p[#p + 1] = "status.runtime_id " .. runtime_id .. " is the learn marker of passive " .. ab.id end
    end
    local buffs, payload = {}, {}
    for _, st in ipairs(s.states or {}) do
      local e = voc.states[st]
      if st == "taunt" then -- cx.taunt next to every apply (Gen:taunting)
      elseif not e then p[#p + 1] = "state:" .. st
      elseif e.buff then buffs[#buffs + 1] = rust_str(e.buff)
      else payload[#payload + 1] = string.format("(StatKey::%s, %d)", e.payload, e.raw) end
    end
    -- triggers: the host calls back the owning ability (status_event); others stay blockers
    local owner, mask = "", status_mask(s)
    -- reapply_if_removed: one row put back with what it had left (a stack count or a second
    -- independent instance would not be the same status again)
    if s.reapply_if_removed ~= nil and s.reapply_if_removed ~= true and s.reapply_if_removed ~= false then
      p[#p + 1] = "status.reapply_if_removed of an expression"
    elseif s.reapply_if_removed == true and s.stacking ~= "refresh" and s.stacking ~= "keep" and s.stacking ~= "replace" then
      p[#p + 1] = "status.reapply_if_removed with stacking:" .. (type(s.stacking) == "table" and "stacks" or tostring(s.stacking))
    end
    if (triggers_of(s) or s.reapply_if_removed == true) and not runtime_id:match("^attack_orb:") then
      for _, name in ipairs(sorted_keys(triggers_of(s) or {})) do
        if not trigger_kind(name) then p[#p + 1] = "status_trigger:" .. name end
      end
      local o = m.statuses[id].parent
      -- a document-level status: its host (the ability creating the template that carries it);
      -- the applying code's on_created / periodic / aura / reapply live with the parent, so only
      -- the evented triggers move
      local evented_only = true
      for name in pairs(triggers_of(s) or {}) do if name == "on_created" or name == "periodic" then evented_only = false end end
      if s.aura or s.reapply_if_removed == true then evented_only = false end
      if o and o.kind ~= "ability" and o.kind ~= "item" and m.statuses[id].host and evented_only then o = m.statuses[id].host end
      if not o or (o.kind ~= "ability" and o.kind ~= "item") then p[#p + 1] = "status.triggers on a document-level status"
      else owner = o.id end
    end
    -- status resistance: "normal" (the default) or "ignore"; a number (its own resistance) waits
    local sr = s.status_resistance
    if sr ~= nil and sr ~= "normal" and sr ~= "ignore" then p[#p + 1] = "status.status_resistance:" .. (type(sr) == "table" and "value" or tostring(sr)) end
    -- `__name:...` is the form of a host packet id (`__aura:`, `__aura_emitter:`, `__ground_damage_zone:`…):
    -- cast settlement and the BuffStore would take the status for that packet (E299)
    if runtime_id:match("^__[%w_]+:") then p[#p + 1] = "status.runtime_id in a host packet namespace (" .. runtime_id:match("^(__[%w_]+:)") .. ")" end
    -- unique_group: per stat the strongest of the group (host aggregation family); `latest` waits
    local family = ""
    if s.unique_group ~= nil then
      if type(s.unique_group) ~= "string" then p[#p + 1] = "status.unique_group of an expression"
      else family = "unique_group:" .. s.unique_group end
    end
    -- heal_reduction_pct: the host's healing_reduction family (heal_received_multiplier, the
    -- strongest cut of every source, as the source's HealFilter takes the smallest factor)
    if s.stats and s.stats.heal_reduction_pct ~= nil then
      if family ~= "" then p[#p + 1] = "status.unique_group with heal_reduction_pct" else family = "healing_reduction" end
    end
    if s.unique_rule ~= nil and s.unique_rule ~= "max" then p[#p + 1] = "status.unique_rule:" .. tostring(s.unique_rule) end
    -- scope: "unit" (the default) ends with the unit; hero / permanent outlive the hero's death
    -- (StatusDef.scope, host status_scope); others (match, item_instance…) wait
    if s.scope ~= nil and s.scope ~= "unit" and s.scope ~= "hero" and s.scope ~= "permanent" then
      p[#p + 1] = "status.scope:" .. tostring(s.scope)
    end
    problems[id] = p
    out[#out + 1] = string.format("\n    pub const %s: StatusDef = StatusDef {\n        id: %s,\n        runtime_id: %s,\n        state_buffs: &[%s],\n        state_payload: &[%s],\n        owner: %s,\n        events: %d,\n        ignore_status_resistance: %s,\n        family: %s,\n        scope: %s,\n    };\n",
      upper(id), rust_str(id), rust_str(runtime_id), table.concat(buffs, ", "), table.concat(payload, ", "), rust_str(owner), mask, tostring(sr == "ignore"), rust_str(family),
      -- scope hero / permanent: kept across the hero's death (host status_scope)
      rust_str((s.scope == "hero" or s.scope == "permanent") and s.scope or ""))
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
    elseif name == "slow_pct" and type(v) == "table" and (v.op == nil or v.op == "add") and v.per_stack and self.per_rows
      and v.when == nil then
      -- a slow per stack of a counted status (B10W): the slow row is that times the stacks
      if self.last_slow_per then self:block("stat:slow_pct twice") end
      self.last_slow_per = self:value(expr, ctx, "num")
    elseif name == "heal_reduction_pct" then
      -- received healing times 1 - pct; its status joins the healing_reduction family (ids)
      if type(v) == "table" then self:block("stat:heal_reduction_pct of a table")
      elseif not self.slow_ok then self:block("stat:heal_reduction_pct outside a status")
      else rows[#rows + 1] = "(StatKey::HealReceivedMultiplier, (Fixed64::ONE - (" .. self:value(expr, ctx, "num") .. ")).raw())" end
    elseif not entry then self:block("stat:" .. name)
    elseif expr ~= nil then
      local raw
      -- percent points from a plain pct value keep the authored raw (exact, like the handlers)
      local ref = type(expr) == "string" and expr:match("^%s*%$([%w_]+)%s*$")
      if entry.unit == "PercentPoints" and ref then
        local d, o = find(self.m, ctx.owner, nil, ref)
        local row = o and self.m.rows[o.id] and self.m.rows[o.id][ref]
        if d and d.unit == "pct" and row and row.raw then raw = value_path(o, ref, false) .. ".at(" .. self:rank_of(o) .. ").raw()" end
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
  apply_status = set({ "op", "status", "to", "duration", "source", "stacks", "vars" }),
  -- exclude_source_tags, source_is, source_ability, min_duration, min_remaining: the host does not
  -- judge status sources or durations yet (line B), so they stay blocked
  remove_status = set({ "op", "status", "from", "dispel", "only", "team_of_source", "except", "states", "limit", "tags", "exclude_tags" }),
  damage = set({ "op", "to", "amount", "type", "require_source_alive", "flags", "source", "credit" }),
  teleport = set({ "op", "who", "to", "find_clear_space", "max_distance" }),
  add_stacks = set({ "op", "to", "status", "count", "max", "refresh" }),
  set_stacks = set({ "op", "to", "status", "count" }),
  modify_cooldown = set({ "op", "of", "ability", "mode", "amount" }),
  set_ability_level = set({ "op", "of", "ability", "level" }),
  end_channel = set({ "op", "interrupted" }),
  set_health = set({ "op", "to", "value" }),
  order_attack = set({ "op", "who", "target" }),
  random_pick = set({ "op", "options", "as" }),
  modify_attack = set({ "op", "crit_multiplier", "cannot_miss", "is_orb" }),
  knockback = set({ "op", "who", "from", "distance", "duration", "stun", "height", "stop_on_contact", "contact_radius", "on_end" }),
  cancel_cast = set({ "op", "reason" }),
  face = set({ "op", "who", "toward" }),
  leap = set({ "op", "who", "to", "duration", "height" }),
  swap_positions = set({ "op", "a", "b" }),
  clear_slow_pool = set({ "op", "of" }),
  kill = set({ "op", "to", "silent" }),
  remove_unit = set({ "op", "to" }),
  set_display_stacks = set({ "op", "status", "to", "count" }),
  toggle_ability = set({ "op", "of", "ability", "on" }),
  zone = set({ "op", "shape", "at", "radius", "duration", "interval", "first", "cue", "on_tick" }),
  spend_mana = set({ "op", "from", "amount", "ability" }),
  set_var = set({ "op", "var", "value", "on", "status" }),
  set_mana = set({ "op", "to", "value" }),
  -- line B's ability_slots (host): an ability named by its id
  set_ability_active = set({ "op", "of", "ability", "active" }),
  swap_abilities = set({ "op", "on", "from", "to", "duration", "cooldown_policy" }),
  -- force: a script's order already is one for a hero (hero_command: no owner restriction or
  -- taunt refuses it, it clears the queue; line B); queue (after the current one) is not lowered
  issue_order = set({ "op", "who", "order", "target", "at", "force" }),
  -- the attacker's own attack (line B perform_attack): an instant hit, or without `instant` an attack
  -- projectile (line B E486, its effects on arrival); `ignore_range` is a no-op there (a scripted
  -- attack never checks range), an effect multiplier other than 1 waits
  attack = set({ "op", "attacker", "target", "use_orb", "trigger_effects", "fake", "effect_multiplier", "ignore_range", "instant", "cannot_miss" }),
  create_pickup = set({ "op", "at", "who", "cue", "killable", "on_pickup", "lifetime" }),
  heal = set({ "op", "to", "amount", "source" }),
  restore_mana = set({ "op", "to", "amount", "source" }),
  cue = false, -- presentation: any fields
  ["if"] = set({ "op", "cond", "actions", "otherwise" }),
  let = set({ "op", "name", "value" }),
  for_each = set({ "op", "select", "as", "actions" }),
  create_unit = set({ "op", "template", "at", "count", "lifetime", "control", "limit_per_owner", "over_limit", "as", "owner", "team", "facing", "overrides" }),
  create_illusion = set({ "op", "of", "at", "duration", "control", "copy", "outgoing", "incoming", "as" }),
  reveal = set({ "op", "at", "radius", "duration", "teams", "follow", "ignore_terrain" }),
  call = set({ "op", "routine", "args" }),
  delay = set({ "op", "time", "actions", "capture", "id" }),
  cancel_delay = set({ "op", "id" }),
  ["repeat"] = set({ "op", "count", "interval", "first", "while", "stop_when", "as", "actions" }),
  modify_damage = set({ "op", "multiplier", "set", "cancel" }),
  chance = set({ "op", "p", "actions", "otherwise", "pity_after", "counter", "counter_var" }),
  give_gold = set({ "op", "to", "amount", "reliable", "show" }),
  take_gold = set({ "op", "from", "amount" }),
  burn_mana = set({ "op", "to", "amount", "source" }),
  dash = set({ "op", "who", "to", "direction", "speed", "distance" }),
  dodge_projectiles = set({ "op", "of", "from_team", "kind" }),
  set_state = set({ "op", "var", "value", "on" }),
  projectile = set({ "op", "kind", "from", "to", "direction", "speed", "radius", "distance", "pierce", "max_hits",
    "team", "types", "flags", "dodgeable", "vision", "end_radius", "cue", "on_hit", "as", "on_end" }),
  clear_state = set({ "op", "var", "on" }),
  stop = set({ "op" }),
  destroy_trees = set({ "op", "at", "radius" }),
  destroy_tree = set({ "op", "tree" }),
  regrow_trees = set({ "op" }),
  set_time_of_day = set({ "op", "mode", "duration" }),
  refund = set({ "op", "cooldown", "mana", "health" }),
  tether = false, -- shape-checked in Gen:tether
}
local SELECTOR_FIELDS = set({ "from", "radius", "shape", "team", "types", "flags", "where", "notes", "limit", "order", "exclude", "relative_to" })
local SELECT_TYPES = { [""] = "All", all = "All", hero = "Heroes", ["creep+hero+neutral+summon"] = "NonBuilding",
  -- every unit class, buildings included / every non-building class (couriers and wards are units too)
  ["building+creep+hero+neutral+summon"] = "All", ["courier+creep+hero+neutral+summon+ward"] = "NonBuilding",
  -- towers are buildings: every class again
  ["building+creep+hero+neutral+summon+tower"] = "All",
  -- buildings only / every class but heroes
  building = "Buildings", ["building+creep+neutral+summon"] = "NonHero" }
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
    root_damage = self:value(list[1].amount, { owner = so, scope = ctx.scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }, "num")
  end
  -- the v3 paths of the two damages the host settles (test damage trace)
  local expire_path, root_path = "?", ""
  for i, x in ipairs(a.on_expire) do
    if x == dmg then expire_path = ((self.paths and self.paths[a.on_expire]) or "?") .. "[" .. i .. "]" end
  end
  if trig then
    local list = trig[1] and trig or trig.actions
    root_path = ((self.paths and self.paths[list]) or "?") .. "[1]"
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
    .. line("        root: " .. self:expr(near, { owner = ctx.owner, scope = ctx.scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }) .. ",")
    .. line("        root_far: " .. self:expr(far, ctx) .. ",")
    .. line("        root_damage: " .. root_damage .. ",")
    .. line("        edge: " .. tostring(d.fn == "edge_distance") .. ",")
    .. line("        root_after_source_death: " .. tostring(dmg.require_source_alive ~= false) .. ",")
    .. line("        trace: (" .. rust_str(self.entry.def.id) .. ", " .. rust_str(expire_path) .. ", " .. rust_str(root_path) .. "),")
    .. line("    };")
    .. line("    cx.tether(to, " .. rust_str(self.entry.def.id .. ":{caster}") .. ", duration, link)?;")
    .. line("}")
end

-- list table -> its v3 effect path ("events.on_cast", "statuses[1].triggers.on_created", ...):
-- every array of actions under `node`, the path written the way line B's damage trace reports it
local function is_action_list(t) return type(t) == "table" and type(t[1]) == "table" and t[1].op ~= nil end
-- (the scenario generator`s spelling, v3_scenarios.lua: a trigger written { filter, actions } is
-- addressed as its actions, and a status by its id: `statuses.<id>.triggers.on_created[1]`)
local function effect_paths(node, path, out, key)
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
      local p
      if array and type(k) == "number" then
        p = (key == "statuses" and type(v.id) == "string") and (path .. "." .. v.id) or (path .. "[" .. k .. "]")
      else
        p = (path ~= "" and (path .. ".") or "") .. tostring(k)
      end
      if key == "triggers" and type(v.actions) == "table" then
        -- a trigger { filter, actions }: its actions are the trigger
        effect_paths(v.actions, p, out, k)
        for k2, v2 in pairs(v) do if k2 ~= "actions" and type(v2) == "table" then effect_paths(v2, p .. "." .. tostring(k2), out, k2) end end
      else
        effect_paths(v, p, out, k)
      end
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
    local before = list[i - 1]
    if a.op == "issue_order" and a.order == "attack" and a.target == "caster" and before and before.op == "apply_status"
      and before.to == a.who and self:taunts(before.status) then
      -- the taunt just applied (cx.taunt, host taunt.rs) already orders `who` to attack the caster
      out[#out + 1] = indent .. "// issue_order attack: given by the taunt above (host taunt.rs)\n"
    else
      out[#out + 1] = self:action(a, ctx, indent)
    end
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
  local starts = starts_on_apply(s.triggers, s.aura)
  local sctx = { owner = so, scope = ctx.scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }
  local to = self:value(a.to, ctx, "unit")
  if a.__aura then self:block("aura.status with counted stacks") end
  local dur
  if a.duration ~= nil then dur = self:value(a.duration, ctx, "num")
  elseif s.duration == nil or s.duration == "infinite" then dur = "Cx::FOREVER"
  else dur = self:value(s.duration, sctx, "num") end
  self.slow_ok, self.last_slow = true, nil
  self.per_rows = st.value_stacks and {} or nil
  local flat = s.stats and self:stat_rows(s.stats, sctx) or "&[]"
  local per = self.per_rows and ("&[" .. table.concat(self.per_rows, ", ") .. "]") or "&[]"
  local slow = self.last_slow
  if self.last_slow_per then
    if slow then self:block("status.stats: slow_pct both flat and per stack") end
    slow = "(" .. self.last_slow_per .. ") * ability_kit::floor(stacks)"
  end
  self.slow_ok, self.last_slow, self.per_rows, self.last_slow_per = nil, nil, nil, nil
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
    self:need_module(so)
    out = out .. line(string.format("    if !existed && cx.has_status(to, %s) { super::%s::st_%s_start(cx, to)?; }", S, so.parent.id, a.status))
  end
  return out .. line("}")
end

local AURA_FIELDS = set({ "radius", "team", "types", "flags", "include_self", "status", "linger", "where", "notes" })

-- `cx.emit_aura(..)` of aura `a` named `key` on `holder` (requires: the status it lives on, or None)
function Gen:aura_emit(a, ctx, key, holder, requires, passive)
  for _, k in ipairs(sorted_keys(a)) do if not AURA_FIELDS[k] then self:block("aura." .. k) end end
  if a.team ~= "ally" and a.team ~= "enemy" and a.team ~= "both" then self:block("aura.team:" .. tostring(a.team)) end
  if a.radius == nil then self:block("aura without radius"); return "Ok::<(), &'static str>(())" end
  local radius = self:value(a.radius, ctx, "num")
  return string.format("cx.emit_aura(%s, ID, &ability_kit::aura_key(%s, cx.caster), %s, %s, %s, Cx::FOREVER, %s, %s)",
    holder, rust_str(key), radius, rust_str(a.team or ""), a.include_self == true and "true" or "false", requires, passive)
end

-- the on_aura arm of aura `a` (owner: whose values it reads): every receiver the host listed that
-- passes `types`, `flags` and `where` gets the aura's status for `linger` (0 when absent) plus one
-- pulse period, renewed every pulse while it stays
function Gen:aura_arm(key, a, owner)
  local types = SELECT_TYPES[types_key(a.types)]
  if not types then self:block("aura.types:" .. types_key(a.types)); types = "All" end
  local mi, inv, hid, ill = false, false, false, false
  for _, f in ipairs(a.flags or {}) do
    if f == "include_magic_immune" then mi = true elseif f == "include_invulnerable" then inv = true
    elseif f == "include_hidden" then hid = true
    -- include_illusions: Select.include_illusions (a hero aura also reaches illusions)
    elseif f == "include_illusions" then ill = true
    else self:block("aura.flag:" .. f) end
  end
  if type(a.include_self) ~= "boolean" and a.include_self ~= nil then self:block("aura.include_self of an expression") end
  local so = self.m.statuses[a.status]
  if not so then self:block("aura.status:" .. tostring(a.status) .. " not declared") end
  local scope = scope_child(nil)
  local it = self:bind(scope, "it", "unit")
  local ctx = { owner = owner, scope = scope, event = false }
  local where = a.where ~= nil and self:value(a.where, ctx, "bool") or nil
  local linger = a.linger ~= nil and self:value(a.linger, ctx, "num") or "Fixed64::ZERO"
  local apply = so and self:action({ op = "apply_status", to = "it", status = a.status, __aura = true,
    __dur = "(" .. linger .. ") + ability_kit::AURA_PERIOD" }, ctx, "        ") or ""
  local fn = string.format("/// aura `%s`: the host's receivers this pulse.\nfn aura_%s(cx: &mut Cx<'_, '_>, receivers: &[EntityHandle]) -> Res {\n"
    .. "    let aura_select = Select { types: UnitTypes::%s, include_magic_immune: %s, include_invulnerable: %s, include_hidden: %s, include_illusions: %s };\n"
    .. "    for &%s in receivers {\n        if !cx.admits(%s, aura_select) { continue; }\n%s%s    }\n    Ok(())\n}\n",
    key, key, types, mi and "true" or "false", inv and "true" or "false", hid and "true" or "false", ill and "true" or "false", it, it,
    where and ("        if !(" .. where .. ") { continue; }\n") or "", apply)
  return { key = key, fn = fn }
end

-- whether a declared status holds the state `taunt` (host taunt.rs through cx.taunt)
function Gen:taunts(status)
  local so = status and self.m.statuses[status]
  if not so then return false end
  for _, st in ipairs(so.def.states or {}) do if st == "taunt" then return true end end
  return false
end

-- apply_status of a taunting status: every apply in the code also taunts `to`, the applier taunting
-- (`source` for apply_status_from, else the caster); removing it ends the taunt.
local function with_taunt(code, call, by)
  local found = false
  local out = code:gsub("([ \t]*)(cx%." .. call .. "%b()%?;)(\r?\n)", function(ws, stmt, nl)
    found = true
    return ws .. stmt .. nl .. ws .. "cx.taunt(to, " .. by .. ", duration)?;" .. nl
  end)
  return out, found
end

-- Does a routine (or one it calls) schedule steps of its own: delay, repeat, projectile, tether?
local STEP_OPS = { delay = true, ["repeat"] = true, projectile = true, tether = true }
local needs_cache = setmetatable({}, { __mode = "k" })
local function routine_needs_entry(m, name, seen)
  needs_cache[m] = needs_cache[m] or {}
  if needs_cache[m][name] ~= nil then return needs_cache[m][name] end
  seen = seen or {}
  if seen[name] then return false end
  seen[name] = true
  local found = false
  local function walk(node)
    if found or type(node) ~= "table" then return end
    if type(node.op) == "string" then
      if STEP_OPS[node.op] then found = true; return end
      if node.op == "call" and m.routines[node.routine] and routine_needs_entry(m, node.routine, seen) then found = true; return end
    end
    for _, v in pairs(node) do walk(v) end
  end
  walk((m.routines[name] or {}).actions)
  needs_cache[m][name] = found
  return found
end

-- Does routine `name` call itself again (directly or through other routines)?
local function routine_recursive(m, name)
  local seen = {}
  local function reaches(node)
    if type(node) ~= "table" then return false end
    if node.op == "call" and type(node.routine) == "string" then
      if node.routine == name then return true end
      if not seen[node.routine] and m.routines[node.routine] then
        seen[node.routine] = true
        if reaches(m.routines[node.routine].actions) then return true end
      end
    end
    for _, v in pairs(node) do if type(v) == "table" and reaches(v) then return true end end
    return false
  end
  return reaches((m.routines[name] or {}).actions)
end

function Gen:action(a, ctx, ind)
  local refso = a.op == "apply_status" and not a.__refr and a.status and self.m.statuses[a.status]
  if refso and (triggers_of(refso.def) or {}).on_refreshed ~= nil then
    -- on_refreshed: it was on `to` before this apply, so after it the status's on_refreshed runs
    -- with the holder as self (st_<id>_on_refreshed in its owner's module)
    if refso.def.stacking == "independent" then self:block("status.triggers.on_refreshed on an independent status"); return "" end
    local inner = {}
    for key, v in pairs(a) do inner[key] = v end
    inner.__refr = true
    local code = self:action(inner, ctx, ind)
    if code == "" then return code end
    local st = "&super::ids::status::" .. upper(a.status)
    local done = false
    code = code:gsub("([ \t]*)(let to = [^\n]-;)(\r?\n)", function(ws, stmt, eol)
      if done then return nil end
      done = true
      return ws .. stmt .. eol .. ws .. "let refreshed = cx.has_status(to, " .. st .. ");" .. eol
    end, 1)
    local closing = code:match("^(.*)\n[ \t]*}%s*$")
    if not done or not closing then self:block("status.triggers.on_refreshed on this apply form"); return code end
    self:need_module(refso)
    local owner_mod = (refso.host and refso.parent.kind ~= "ability" and refso.parent.kind ~= "item") and refso.host.id or refso.parent.id
    local pad = ind .. "    "
    return closing .. "\n" .. pad .. "if refreshed {\n"
      .. pad .. "    let prev = cx.self_unit;\n" .. pad .. "    cx.self_unit = to;\n"
      .. pad .. "    let r = super::" .. owner_mod .. "::st_" .. a.status .. "_on_refreshed(cx);\n"
      .. pad .. "    cx.self_unit = prev;\n" .. pad .. "    r?;\n" .. pad .. "}\n" .. ind .. "}\n"
  end
  local fso = a.op == "apply_status" and not a.__fmove and a.status and self.m.statuses[a.status]
  if fso and fso.def.forced_move ~= nil then
    -- forced_move (line B E464): once the status (and its vars) is on the holder, the holder is
    -- ordered toward the point for the status's time and its orders are refused (every lock)
    local fm = fso.def.forced_move
    for _, k in ipairs(sorted_keys(type(fm) == "table" and fm or {})) do
      if k ~= "toward" and k ~= "locks" then self:block("status.forced_move." .. k) end
    end
    if type(fm) ~= "table" or type(fm.toward) ~= "string" then self:block("status.forced_move without toward"); return "" end
    if fso.def.stacking == "independent" then self:block("status.forced_move on an independent status"); return "" end
    local inner = {}
    for key, v in pairs(a) do inner[key] = v end
    inner.__fmove = true
    local code = self:action(inner, ctx, ind)
    if code == "" then return code end
    -- `vars.x`: the holder's var, read after the apply set it; anything else in the status's scope
    local at
    local vname = fm.toward:match("^%s*vars%.([%a_][%w_]*)%s*$")
    if vname then
      local v = (fso.def.vars or {})[vname]
      if not v or v.type ~= "point" then self:block("status.forced_move.toward:" .. fm.toward); return "" end
      at = var_read("point", "to", status_var_key(fso.def.id, vname), "cx.pos(to)?")
    else
      local sctx = { owner = fso, scope = ctx.scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }
      at = self:value(fm.toward, sctx, "point")
    end
    local closing = code:match("^(.*)\n[ \t]*}%s*$")
    if not closing or not code:find("let duration = ", 1, true) then self:block("status.forced_move on this apply form"); return code end
    local st = "&super::ids::status::" .. upper(a.status)
    return closing .. "\n" .. ind .. "    cx.force_move(to, " .. st .. ", " .. at .. ", duration)?;\n" .. ind .. "}\n"
  end
  local rso = a.op == "apply_status" and not a.__replace and not a.__aura and a.status and self.m.statuses[a.status]
  if rso and rso.def.stacking == "replace" then
    -- `replace`: the old instance ends (its removal events) and a new one starts (on_created,
    -- vars at their init); the run's reads still see the old one, so it counts as absent
    local inner = {}
    for key, v in pairs(a) do inner[key] = v end
    inner.__replace = true
    local code = self:action(inner, ctx, ind)
    if code == "" then return code end
    local st = "&super::ids::status::" .. upper(a.status)
    local n
    code, n = code:gsub("cx%.has_status%(to, " .. st:gsub("%p", "%%%0") .. "%)", "false")
    local done = false
    code = code:gsub("([ \t]*)(let to = [^\n]-;)(\r?\n)", function(ws, stmt, eol)
      if done then return nil end
      done = true
      return ws .. stmt .. eol .. ws .. "cx.remove_status(to, " .. st .. ")?;" .. eol
    end, 1)
    if not done then self:block("status.stacking:replace on this apply form") end
    return code
  end
  local aso = a.op == "apply_status" and not a.__absorb and a.status and self.m.statuses[a.status]
  if aso and aso.def.absorb ~= nil then
    -- a shield (line B's cx.absorb): after the status lands, for the status's own time; the host
    -- removes the status when the shield is used up
    local ab = aso.def.absorb
    local ok = type(ab) == "table" and ab.amount ~= nil
    if not ok then self:block("status.absorb without amount"); return "" end
    for _, k in ipairs(sorted_keys(ab)) do
      if k ~= "amount" and k ~= "types" and k ~= "phase" and k ~= "on_absorb" and k ~= "on_broken" then self:block("status.absorb." .. k) end
    end
    if ab.phase ~= nil and ab.phase ~= "post_mitigation" then self:block("status.absorb.phase:" .. tostring(ab.phase)) end
    if aso.def.stacking == "independent" then self:block("status.absorb on an independent status"); return "" end
    local types = {}
    for _, t in ipairs(ab.types or {}) do
      if t == "physical" or t == "magical" or t == "pure" then types[#types + 1] = rust_str(t)
      else self:block("status.absorb.types:" .. tostring(t)) end
    end
    local inner = {}
    for key, v in pairs(a) do inner[key] = v end
    inner.__absorb = true
    local code = self:action(inner, ctx, ind)
    if code == "" then return code end
    local sctx = { owner = aso, scope = ctx.scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }
    local amount = self:value(ab.amount, sctx, "num")
    local st = "&super::ids::status::" .. upper(a.status)
    local found = false
    code = code:gsub("([ \t]*)(cx%.apply_status%(to, " .. st:gsub("%p", "%%%0") .. "[^\n]-%?;)(\r?\n)", function(ws, stmt, eol)
      if found then return nil end
      found = true
      return ws .. stmt .. eol .. ws .. "let absorb = " .. amount .. ";" .. eol
        .. ws .. "cx.absorb(to, " .. st .. ", absorb, &[" .. table.concat(types, ", ") .. "], duration)?;" .. eol
    end)
    if not found then self:block("status.absorb on this apply form") end
    return code
  end
  local vso = a.op == "apply_status" and not a.__vars and a.status and self.m.statuses[a.status]
  if vso and vso.def.vars and next(vso.def.vars) and vso.def.stacking ~= "independent" then
    -- a newly created status starts its vars at their init (apply_status `vars` overrides them)
    local inner = {}
    for key, v in pairs(a) do inner[key] = v end
    inner.__vars = true
    inner.vars = nil
    local code = self:action(inner, ctx, ind)
    if code == "" then return code end
    local sctx = { owner = vso, scope = ctx.scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }
    local inits = {}
    for _, name in ipairs(sorted_keys(vso.def.vars)) do
      local v = vso.def.vars[name]
      local ty = VAR_TYPES[v.type]
      if not ty then self:block("status.vars type:" .. tostring(v.type))
      else
        local given = a.vars and a.vars[name]
        local init = given ~= nil and self:value(given, ctx, ty) or (v.init ~= nil and self:value(v.init, sctx, ty) or (ty == "unit" and "EntityHandle::INVALID" or var_zero(ty)))
        inits[#inits + 1] = var_write(ty, "to", status_var_key(vso.def.id, name), init)
      end
    end
    for k in pairs(a.vars or {}) do if not vso.def.vars[k] then self:block("action:apply_status.vars:" .. k) end end
    -- around the first `let to`: was it there before; after the block's apply: set the vars when new
    local st = "&super::ids::status::" .. upper(a.status)
    local done = false
    code = code:gsub("([ \t]*)(let to = [^\n]-;)(\r?\n)", function(ws, stmt, eol)
      if done then return nil end
      done = true
      return ws .. stmt .. eol .. ws .. "let vars_new = !cx.has_status(to, " .. st .. ");" .. eol
    end, 1)
    local closing = code:match("^(.*)\n[ \t]*}%s*$")
    if not done or not closing then self:block("status.vars on this apply form"); return code end
    local pad = ind .. "    "
    local body = pad .. "if vars_new {" ..  "\n"
    for _, i in ipairs(inits) do body = body .. pad .. "    " .. i ..  "\n" end
    body = body .. pad .. "}" ..  "\n"
    return closing ..  "\n" .. body .. ind .. "}" ..  "\n"
  end
  if (a.op == "apply_status" or a.op == "remove_status") and not a.__taunt and self:taunts(a.status) then
    local inner = {}
    for key, v in pairs(a) do inner[key] = v end
    inner.__taunt = true
    local code = self:action(inner, ctx, ind)
    if code == "" then return code end
    if a.op == "remove_status" then
      local out, found = code:gsub("([ \t]*)(cx%.remove_status%(from, [^\n]-%?;)(\r?\n)", "%1%2%3%1cx.end_taunt(from)?;%3")
      if found == 0 then self:block("state:taunt removal") end
      return out
    end
    local n1, n2
    code, n1 = with_taunt(code, "apply_status_from", "source")
    code, n2 = with_taunt(code, "apply_status", "cx.caster")
    if not (n1 or n2) then self:block("state:taunt on this apply form") end
    -- remove_on ends the status row; the host does not end its taunt with it
    return code
  end
  local op = a.op
  local allowed = ACTION_FIELDS[op]
  if allowed == nil then self:block("action:" .. tostring(op)); return "" end
  if allowed then
    for _, k in ipairs(sorted_keys(a)) do if not allowed[k] and k ~= "notes" and k ~= "__taunt" and k ~= "__vars" and k ~= "__absorb" and k ~= "__replace" and k ~= "__dur" and k ~= "__aura" and k ~= "__fmove" and k ~= "__refr" then self:block("action:" .. op .. "." .. k) end end
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
    local independent = s.stacking == "independent"
    if s.stacking ~= "refresh" and s.stacking ~= "keep" and s.stacking ~= "replace" and not independent then
      self:block("status.stacking:" .. (type(s.stacking) == "table" and (s.stacking.mode or "stacks") or tostring(s.stacking)))
    end
    if independent then
      -- an instance row `<runtime id>#<n>` (cx.apply_status_instance); its triggers run as that
      -- instance (host events carry `instance`, its on_created and periodic clock get `n`); its
      -- shared states ride its own control-source row `__ctl:<id>#<n>` (line B 94aadadf)
      if s.aura ~= nil then self:block("status.stacking:independent with aura") end
      if a.__aura then self:block("aura.status:independent") end
    end
    for _, p in ipairs(self.status_problems[a.status] or {}) do self:block(p) end
    local starts = not (s.runtime_id or ""):match("^attack_orb:") and starts_on_apply(s.triggers, s.aura)
    local sctx = { owner = so, scope = ctx.scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }
    local to = self:value(a.to, ctx, "unit")
    local dur
    if a.__dur then dur = a.__dur
    elseif a.duration ~= nil then dur = self:value(a.duration, ctx, "num")
    elseif s.duration == nil or s.duration == "infinite" then dur = "Cx::FOREVER"
    else dur = self:value(s.duration, sctx, "num") end
    self.slow_ok, self.last_slow = true, nil
    local stats = s.stats and self:stat_rows(s.stats, sctx) or "&[]"
    local slow = self.last_slow
    self.slow_ok, self.last_slow = nil, nil
    if independent then
      -- slow_pct: the instance's own slow row `<id>#<n>:slow` (line B E463)
      local call = slow and ("cx.apply_status_instance_slowed(to, &super::ids::status::" .. upper(a.status) .. ", duration, stats, slow)")
        or ("cx.apply_status_instance(to, &super::ids::status::" .. upper(a.status) .. ", duration, stats)")
      local body = line("{") .. line("    let to = " .. to .. ";") .. line("    let duration = " .. dur .. ";")
        -- the stats first: their reads take &mut cx
        .. line("    let stats: &[(StatKey, i64)] = " .. stats .. ";")
        .. (slow and line("    let slow = Some(" .. slow .. ");") or "")
      -- the new instance's start (on_created, periodic) runs as that instance, its source the caster
      local start_fn = starts and string.format("super::%s::st_%s_start(cx, to)", so.parent.id, a.status)
      if starts then self:need_module(so) end
      local function started(indent)
        return line(indent .. "let r = match r {") .. line(indent .. "    Ok(n) if n != 0 => {")
          .. line(indent .. "        let prev = cx.event_instance;") .. line(indent .. "        cx.event_instance = n;")
          .. line(indent .. "        let r = " .. start_fn .. ";") .. line(indent .. "        cx.event_instance = prev;")
          .. line(indent .. "        r") .. line(indent .. "    }") .. line(indent .. "    Ok(_) => Ok(()),")
          .. line(indent .. "    Err(e) => Err(e),") .. line(indent .. "};")
      end
      if a.source ~= nil then
        -- `source` applies it (its `{caster}` id, its instance count), as apply_status_from
        return body .. line("    let source = " .. self:value(a.source, ctx, "unit") .. ";")
          .. line("    let caster = cx.caster;") .. line("    cx.caster = source;") .. line("    let r = " .. call .. ";")
          .. (starts and started("    ") or line("    let r = r.map(|_| ());"))
          .. line("    cx.caster = caster;") .. line("    r?;") .. line("}")
      end
      if starts then return body .. line("    let r = " .. call .. ";") .. started("    ") .. line("    r?;") .. line("}") end
      return body .. line("    " .. call .. "?;") .. line("}")
    end
    if slow or a.source ~= nil then
      local src = a.source ~= nil and self:value(a.source, ctx, "unit") or "cx.caster"
      local call = string.format("    cx.apply_status_from(source, to, &super::ids::status::%s, duration, %s, %s)?;", upper(a.status), stats,
        slow and ("Some(" .. slow .. ")") or "None")
      local s2 = line("{") .. line("    let to = " .. to .. ";") .. line("    let duration = " .. dur .. ";") .. line("    let source = " .. src .. ";")
      if starts then
        self:need_module(so)
        s2 = s2 .. line("    let existed = { let c = cx.caster; cx.caster = source; let e = cx.has_status(to, &super::ids::status::" .. upper(a.status) .. "); cx.caster = c; e };") .. line(call)
          .. line("    if !existed {") .. line("        let caster = cx.caster;") .. line("        cx.caster = source;")
          .. line(string.format("        let r = super::%s::st_%s_start(cx, to);", so.parent.id, a.status))
          .. line("        cx.caster = caster;") .. line("        r?;") .. line("    }")
        -- a refreshed aura status emits its aura again at this rank (an upgrade's level, D B31W)
        if s.aura then
          s2 = s2 .. line("    else {") .. line("        let caster = cx.caster;") .. line("        cx.caster = source;")
            .. line(string.format("        let r = super::%s::st_%s_reaura(cx, to);", so.parent.id, a.status))
            .. line("        cx.caster = caster;") .. line("        r?;") .. line("    }")
        end
      else
        s2 = s2 .. line(call)
      end
      return s2 .. line("}")
    end
    if starts then
      self:need_module(so)
      return line("{") .. line("    let to = " .. to .. ";") .. line("    let duration = " .. dur .. ";")
        .. line("    let existed = cx.has_status(to, &super::ids::status::" .. upper(a.status) .. ");")
        .. line("    cx.apply_status(to, &super::ids::status::" .. upper(a.status) .. ", duration, " .. stats .. ")?;")
        .. line(string.format("    if !existed { super::%s::st_%s_start(cx, to)?; }", so.parent.id, a.status)
          .. (s.aura and string.format(" else { super::%s::st_%s_reaura(cx, to)?; }", so.parent.id, a.status) or "")) .. line("}")
    end
    if s.stacking == "keep" and not a.__aura then
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
      -- the controls the host purge takes (omoba-core dispel::is_control)
      if buff == "stun" or buff == "root" or buff == "silence" or buff == "disarm" or buff == "mute" or buff == "hex" then states[#states + 1] = rust_str(buff)
      else self:block("action:remove_status.states:" .. tostring(name)) end
    end
    local limit = a.limit ~= nil and ("Some(" .. self:value(a.limit, ctx, "num") .. ")") or "None"
    local filter = "ability_kit::StatusFilter { dispel: " .. opt_str(a.dispel) .. ", only: " .. opt_str(a.only)
      .. ", team_of_source: " .. rust_str(a.team_of_source or "any") .. ", except: &[" .. table.concat(except, ", ")
      .. "], states: &[" .. table.concat(states, ", ") .. "], limit: " .. limit .. " }"
    if a.tags ~= nil or a.exclude_tags ~= nil then
      -- by the statuses' catalog tags (host reads them)
      local function list(t) local o = {} for _, x in ipairs(t or {}) do o[#o + 1] = rust_str(x) end return "&[" .. table.concat(o, ", ") .. "]" end
      return line("{") .. line("    let from = " .. from .. ";")
        .. line("    cx.remove_statuses_tagged(from, " .. filter .. ", " .. list(a.tags) .. ", " .. list(a.exclude_tags) .. ")?;") .. line("}")
    end
    return line("{") .. line("    let from = " .. from .. ";") .. line("    cx.remove_statuses(from, " .. filter .. ")?;") .. line("}")
  elseif op == "remove_status" then
    if not self.m.statuses[a.status] then self:block("action:remove_status without a declared status"); return "" end
    if self.m.statuses[a.status].def.stacking == "independent" then
      if ctx.owner == self.m.statuses[a.status] then self:block("action:remove_status of its own independent status (this instance waits)"); return "" end
      -- line B: the earliest instance first (`limit` of them, else every one); only `limit` may join
      for _, k in ipairs({ "dispel", "only", "team_of_source", "except", "states" }) do
        if a[k] ~= nil then self:block("action:remove_status.status with " .. k); return "" end
      end
      local from = self:value(a.from, ctx, "unit")
      local st = "&super::ids::status::" .. upper(a.status)
      local count = a.limit ~= nil and ("u32::try_from(ability_kit::floor(" .. self:value(a.limit, ctx, "num") .. ").raw() >> 10).unwrap_or(0)")
        or ("cx.status_instances(from, " .. st .. ")")
      return line("{") .. line("    let from = " .. from .. ";") .. line("    let count = " .. count .. ";")
        .. line("    for _ in 0..count { cx.remove_status_instance(from, " .. st .. ")?; }") .. line("}")
    end
    for _, k in ipairs({ "dispel", "only", "team_of_source", "except", "states", "limit" }) do
      if a[k] ~= nil then self:block("action:remove_status.status with " .. k) end
    end
    local from = self:value(a.from, ctx, "unit")
    return line("{") .. line("    let from = " .. from .. ";") .. line("    cx.remove_status(from, &super::ids::status::" .. upper(a.status) .. ")?;") .. line("}")
  elseif op == "damage" then
    local kinds = { physical = "Physical", magical = "Magical", pure = "Pure" }
    local kind = kinds[a.type]
    if not kind then self:block("action:damage.type:" .. tostring(a.type)); kind = "Pure" end
    local no_amp, hp_removal, ignore_armor, reflection, no_src = false, false, false, false, false
    for _, f in ipairs(a.flags or {}) do
      if f == "no_amplify" then no_amp = true
      -- line B's damage_reflect_at: the target's statuses see a reflected hit
      elseif f == "reflection" then reflection = true
      -- line B's damage_ex_at: life paid as a cost (no shield, no amplification) / armor ignored
      elseif f == "hp_removal" then hp_removal = true
      elseif f == "ignore_armor" then ignore_armor = true
      -- as a dummy would deal it: none of the source's bonuses (DamageProfile::NO_SOURCE_BONUSES)
      elseif f == "no_source_bonuses" then no_src = true
      else self:block("action:damage.flags:" .. f) end
    end
    local to = self:value(a.to, ctx, "unit")
    local amount = self:value(a.amount, ctx, "num")
    local who = self.entry and self.entry.def.id or self.trace_owner or "?"
    if hp_removal or ignore_armor or reflection or no_src or a.credit ~= nil then
      local credit = a.credit ~= nil and ("Some(" .. self:value(a.credit, ctx, "unit") .. ")") or "None"
      -- another source: line B's damage_ex_from_at
      local from = a.source ~= nil and a.source ~= "self"
      local src = from and line("    let source = " .. self:value(a.source, ctx, "unit") .. ";") or ""
      return line("{") .. line("    let to = " .. to .. ";") .. line("    let amount = " .. amount .. ";") .. src .. line("    let credit = " .. credit .. ";")
        .. line(string.format("    cx.%s(%s, %s, %sto, amount, DamageKind::%s, Dmg { require_source_alive: %s, no_amplify: %s }, ability_kit::DmgExtra { hp_removal: %s, ignore_armor: %s, no_source_bonuses: %s, credit })?;",
          reflection and (from and "damage_reflect_from_at" or "damage_reflect_at") or (from and "damage_ex_from_at" or "damage_ex_at"), rust_str(who), rust_str(self.cur_path or "?"), from and "source, " or "", kind, a.require_source_alive ~= false and "true" or "false", no_amp and "true" or "false",
          tostring(hp_removal), tostring(ignore_armor), tostring(no_src))) .. line("}")
    end
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
    -- the host heals with the caster as the healer: `self` is the caster in a passive or a cast;
    -- another `source` (a summon's owner, A29T) is the healer for this one heal
    local to = self:value(a.to, ctx, "unit")
    local amount = self:value(a.amount, ctx, "num")
    local head = line("{") .. line("    let to = " .. to .. ";") .. line("    let amount = " .. amount .. ";")
    if a.source == nil or a.source == "caster" or (a.source == "self" and self.entry) then
      return head .. line("    cx." .. op .. "(to, amount)?;") .. line("}")
    end
    local source = self:value(a.source, ctx, "unit")
    return head .. line("    let source = " .. source .. ";") .. line("    let caster = cx.caster;")
      .. line("    cx.caster = source;") .. line("    let r = cx." .. op .. "(to, amount);")
      .. line("    cx.caster = caster;") .. line("    r?;") .. line("}")
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
    local s = line("if " .. cond .. " {") .. self:actions(a.actions, { owner = ctx.owner, scope = scope_child(ctx.scope), event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }, I .. "    ")
    if a.otherwise then
      s = s .. line("} else {") .. self:actions(a.otherwise, { owner = ctx.owner, scope = scope_child(ctx.scope), event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }, I .. "    ")
    end
    return s .. line("}")
  elseif op == "knockback" then
    -- line B's knockback: `who` straight away from `from` over `duration` (stops at walls); a
    -- negative distance pulls toward `from`, `height` is the arc's peak (presentation only)
    -- distance 0 is a toss in place (host knockback_arc); without a height it would move nothing
    if (a.distance == "0" or a.distance == 0) and a.height == nil then self:block("action:knockback in place (distance 0) without height"); return "" end
    local who = self:value(a.who, ctx, "unit")
    local from, from_ty = self:value(a.from, ctx)
    if from_ty ~= "unit" and from_ty ~= "point" then self:block("action:knockback.from of a " .. tostring(from_ty)); return "" end
    local stun = a.stun == nil and "true" or (type(a.stun) == "boolean" and tostring(a.stun) or self:value(a.stun, ctx, "bool"))
    if a.stop_on_contact ~= nil or a.on_end ~= nil then
      -- line B E494: stop at the first living non-building unit ahead within contact_radius; on_end
      -- runs once on the caster for any end, `at` / `here` where the pushed unit stopped
      if a.stop_on_contact ~= nil and type(a.stop_on_contact) ~= "boolean" then self:block("action:knockback.stop_on_contact of an expression"); return "" end
      if a.stop_on_contact == true and a.contact_radius == nil then self:block("action:knockback.stop_on_contact without contact_radius"); return "" end
      local contact = a.stop_on_contact == true and ("Some(" .. self:value(a.contact_radius, ctx, "num") .. ")") or "None"
      local on_end, args_line = "None", ""
      if a.on_end and #a.on_end > 0 then
        local sc = step_scope(ctx, 0)
        sc.names.at = { name = "at", ty = "point", marker = "ability_kit::arg_as_point(args, \1END\1)", used = true }
        sc.names.here = sc.names.at
        local step = self:reserve_step()
        local body = self:actions(a.on_end, { owner = ctx.owner, scope = sc, event = false }, "    ")
        local eargs = self:capture_args(sc)
        if #eargs + 1 > 8 then self:block("knockback.on_end: more than 7 captured values") end
        body = body:gsub("\1END\1", tostring(#eargs))
        self:set_step(step, "`knockback` on_end", body, true)
        args_line = line("    let end_args = [" .. table.concat(eargs, ", ") .. "];")
        on_end = string.format("Some((ID, %d, &end_args[..]))", step)
      end
      return line("{") .. line("    let who = " .. who .. ";")
        .. line("    let from = " .. (from_ty == "unit" and ("{ let u = " .. from .. "; cx.pos(u)? }") or from) .. ";")
        .. line("    let distance = " .. self:value(a.distance, ctx, "num") .. ";")
        .. line("    let duration = " .. self:value(a.duration, ctx, "num") .. ";")
        .. line("    let height = " .. (a.height ~= nil and self:value(a.height, ctx, "num") or "Fixed64::ZERO") .. ";")
        .. line("    let contact = " .. contact .. ";") .. args_line
        .. line("    cx.knockback_ended(who, from, distance, duration, " .. stun .. ", Fixed64::from_i32(10_000), height, contact, " .. on_end .. ")?;")
        .. line("}")
    end
    return line("{") .. line("    let who = " .. who .. ";")
      .. line("    let from = " .. (from_ty == "unit" and ("{ let u = " .. from .. "; cx.pos(u)? }") or from) .. ";")
      .. line("    let distance = " .. self:value(a.distance, ctx, "num") .. ";")
      .. line("    let duration = " .. self:value(a.duration, ctx, "num") .. ";")
      -- the host's bound on who may be pushed (the caster, or a hostile unit this near it)
      .. (a.height == nil and line("    cx.knockback(who, from, distance, duration, " .. stun .. ", Fixed64::from_i32(10_000))?;")
        or line("    let height = " .. self:value(a.height, ctx, "num") .. ";")
          .. line("    cx.knockback_arc(who, from, distance, duration, " .. stun .. ", Fixed64::from_i32(10_000), height)?;"))
      .. line("}")
  elseif op == "zone" then
    -- a ground area that only ticks: its cue for the duration, and `on_tick` every `interval`
    -- from `first` (immediately without one) while the area lasts, the tick at `duration` included
    if (a.shape or "circle") ~= "circle" then self:block("action:zone.shape:" .. tostring(a.shape)); return "" end
    if a.interval == nil or a.on_tick == nil then self:block("action:zone without interval and on_tick"); return "" end
    local first = (a.first ~= nil and a.first ~= "0" and a.first ~= 0) and a.first or nil
    local count = string.format("floor(((%s) - (%s)) / (%s)) + 1", a.duration, first or "0", a.interval)
    local s2 = ""
    if a.cue ~= nil then s2 = self:action({ op = "cue", id = a.cue, at = a.at, radius = a.radius, duration = a.duration }, ctx, ind) end
    return s2 .. self:action({ op = "repeat", count = count, interval = a.interval, first = first, actions = a.on_tick }, ctx, ind)
  elseif op == "kill" then
    -- host script_kill: no damage packet, credited to the caster; `silent` removes a non-hero
    local silent = a.silent == true and "true" or "false"
    if a.silent ~= nil and type(a.silent) ~= "boolean" then self:block("action:kill.silent of an expression"); return "" end
    return line("{") .. line("    let to = " .. self:value(a.to, ctx, "unit") .. ";") .. line("    cx.kill(to, " .. silent .. ")?;") .. line("}")
  elseif op == "toggle_ability" then
    -- turning a toggle of this document off: `of` runs its on_toggle_off while it holds the
    -- toggle's status (the status is the toggle state; the host keeps no other)
    if a.on ~= false then self:block("action:toggle_ability.on:" .. tostring(a.on)); return "" end
    local ab = self.m.abilities[a.ability]
    local ev = ab and ab.def.events or {}
    if not (ab and ev.on_toggle_on) then self:block("action:toggle_ability of " .. tostring(a.ability) .. " (not a toggle of this document)"); return "" end
    local ts
    for _, x in ipairs(ev.on_toggle_on) do if x.op == "apply_status" and x.to == "caster" then ts = x.status break end end
    if not ts then self:block("toggle: on_toggle_on must put a status on the caster"); return "" end
    self:need_module({ parent = ab, def = { id = ts } })
    local here = self.entry and self.entry.def.id == a.ability
    local call = here and "on_toggle_off(cx)" or ("super::" .. a.ability .. "::on_toggle_off(cx)")
    local id = here and "ID" or ("super::" .. a.ability .. "::ID")
    return line("{") .. line("    let of = " .. self:value(a.of, ctx, "unit") .. ";")
      .. line("    if cx.has_status(of, &super::ids::status::" .. upper(ts) .. ") {")
      .. line("        let level = cx.ability_level(of, " .. id .. ");")
      .. line("        let (caster, rank) = (cx.caster, cx.rank);")
      .. line("        cx.caster = of;")
      .. line("        cx.rank = u8::try_from((ability_kit::floor(level).raw() >> 10).max(1)).unwrap_or(1);")
      .. line("        let r = " .. call .. ";")
      .. line("        cx.caster = caster;") .. line("        cx.rank = rank;") .. line("        r?;")
      .. line("    }") .. line("}")
  elseif op == "set_display_stacks" then
    -- the stack number a plain status shows (and `status.stacks` reads); its stats and time stay
    local so = self.m.statuses[a.status]
    if not so then self:block("status:" .. tostring(a.status) .. " not declared"); return "" end
    local s = so.def
    if type(s.stacking) == "table" then self:block("action:set_display_stacks of a counted status (set_stacks)") end
    if s.stacking == "independent" then self:block("action:set_display_stacks of an independent status") end
    for _, p in ipairs(self.status_problems[a.status] or {}) do self:block(p) end
    local sctx = { owner = so, scope = ctx.scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }
    local to = self:value(a.to, ctx, "unit")
    local count = self:value(a.count, ctx, "num")
    self.slow_ok, self.last_slow = true, nil
    local stats = s.stats and self:stat_rows(s.stats, sctx) or "&[]"
    if self.last_slow then self:block("action:set_display_stacks of a slowing status") end
    self.slow_ok, self.last_slow = nil, nil
    return line("{") .. line("    let to = " .. to .. ";") .. line("    let count = " .. count .. ";")
      .. line("    let stats: &[(StatKey, i64)] = " .. stats .. ";")
      .. line("    cx.set_display_stacks(to, &super::ids::status::" .. upper(a.status) .. ", count, stats)?;") .. line("}")
  elseif op == "remove_unit" then
    -- the unit leaves without dying (host script_kill silent): no bounty, no death events
    return line("{") .. line("    let to = " .. self:value(a.to, ctx, "unit") .. ";") .. line("    cx.remove_unit(to)?;") .. line("}")
  elseif op == "clear_slow_pool" then
    return line("{") .. line("    let of = " .. self:value(a.of, ctx, "unit") .. ";") .. line("    cx.clear_slow_pool(of)?;") .. line("}")
  elseif op == "swap_positions" then
    -- each to where the other stood (both positions read first)
    return line("{") .. line("    let a = " .. self:value(a.a, ctx, "unit") .. ";") .. line("    let b = " .. self:value(a.b, ctx, "unit") .. ";")
      .. line("    let (pa, pb) = (cx.pos(a)?, cx.pos(b)?);")
      .. line("    cx.teleport(a, pb, None)?;") .. line("    cx.teleport(b, pa, None)?;") .. line("}")
  elseif op == "leap" then
    -- the source's knockback with knockback_height toward a point: `who` is pulled onto `to`
    -- over `duration` (host forced_motion, no commands meanwhile), `height` the arc's peak
    local who = self:value(a.who, ctx, "unit")
    local to, to_ty = self:value(a.to, ctx)
    if to_ty == "err" then return "" end
    if to_ty ~= "unit" and to_ty ~= "point" then self:block("action:leap.to of a " .. tostring(to_ty)); return "" end
    return line("{") .. line("    let who = " .. who .. ";")
      .. line("    let to = " .. (to_ty == "unit" and ("{ let u = " .. to .. "; cx.pos(u)? }") or to) .. ";")
      .. line("    let distance = (to - cx.pos(who)?).length();")
      .. line("    let duration = " .. self:value(a.duration, ctx, "num") .. ";")
      .. line("    let height = " .. (a.height ~= nil and self:value(a.height, ctx, "num") or "Fixed64::ZERO") .. ";")
      .. line("    cx.knockback_arc(who, to, -distance, duration, true, Fixed64::from_i32(10_000), height)?;") .. line("}")
  elseif op == "face" then
    local who = self:value(a.who, ctx, "unit")
    local at, at_ty = self:value(a.toward, ctx)
    if at_ty == "err" then return "" end
    if at_ty ~= "unit" and at_ty ~= "point" then self:block("action:face.toward of a " .. tostring(at_ty)); return "" end
    return line("{") .. line("    let who = " .. who .. ";")
      .. line("    let at = " .. (at_ty == "unit" and ("{ let u = " .. at .. "; cx.pos(u)? }") or at) .. ";")
      .. line("    cx.face(who, at)?;") .. line("}")
  elseif op == "spend_mana" then
    -- the ability paid for: a literal id of this document, or `ability` (this one)
    local ab, abcode = a.ability, nil
    if ab == nil or ab == "ability" then abcode = "ID"
    elseif type(ab) == "string" and ab:match("^[%w_]+$") and self.m.abilities[ab] then abcode = rust_str(ab)
    else self:block("action:spend_mana.ability:" .. tostring(ab)); return "" end
    return line("{") .. line("    let from = " .. self:value(a.from, ctx, "unit") .. ";")
      .. line("    let amount = " .. self:value(a.amount, ctx, "num") .. ";")
      .. line("    cx.spend_mana(from, amount, " .. abcode .. ")?;") .. line("}")
  elseif op == "cancel_cast" then
    -- on_phase_start: the cast does not go on (nothing charged); the hook's effects stand and the
    -- rest of the list still runs (every use is the last action of its branch)
    if not self.phase then self:block("action:cancel_cast outside on_phase_start"); return "" end
    return line("cx.cancelled = true;")
  elseif op == "modify_attack" then
    -- line B's modify_attack_crit: the attack in its windup crits (the larger of the rolled and
    -- this multiplier); the holder is the attacker
    if not ctx.windup then self:block("action:modify_attack outside on_attack_windup"); return "" end
    for _, k in ipairs({ "cannot_miss", "is_orb" }) do
      if a[k] ~= nil and type(a[k]) ~= "boolean" then self:block("action:modify_attack." .. k .. " of an expression"); return "" end
    end
    if a.crit_multiplier == nil and a.cannot_miss ~= true and a.is_orb ~= true then self:block("action:modify_attack without a change"); return "" end
    if a.cannot_miss ~= true and a.is_orb ~= true then
      return line("{") .. line("    let multiplier = " .. self:value(a.crit_multiplier, ctx, "num") .. ";")
        .. line("    let attacker = cx.self_unit;") .. line("    cx.modify_attack_crit(attacker, multiplier)?;") .. line("}")
    end
    -- cannot_miss: no miss at launch, no evasion on arrival; is_orb: the attack is an orb's (line B E488)
    local crit = a.crit_multiplier ~= nil and ("Some(" .. self:value(a.crit_multiplier, ctx, "num") .. ")") or "None"
    return line("{") .. line("    let multiplier = " .. crit .. ";") .. line("    let attacker = cx.self_unit;")
      .. line(string.format("    cx.modify_attack(attacker, multiplier, %s, %s)?;", tostring(a.cannot_miss == true), tostring(a.is_orb == true))) .. line("}")
  elseif op == "random_pick" then
    local opts = a.options or {}
    if #opts == 0 then self:block("action:random_pick without options"); return "" end
    local weights, total = {}, {}
    for i, o in ipairs(opts) do
      for _, k in ipairs(sorted_keys(o)) do if k ~= "weight" and k ~= "value" and k ~= "actions" then self:block("action:random_pick option." .. k) end end
      weights[i] = self:value(o.weight or "1", ctx, "num")
      total[#total + 1] = "w" .. i
    end
    local head = line("{")
    for i, w in ipairs(weights) do head = head .. line("    let w" .. i .. " = (" .. w .. ").max(Fixed64::ZERO);") end
    head = head .. line("    let total = " .. table.concat(total, " + ") .. ";")
      .. line("    let roll = cx.random(Fixed64::ZERO, total);")
    -- option i when roll < w1 + ... + wi
    local function bound(i) local t = {} for j = 1, i do t[#t + 1] = "w" .. j end return table.concat(t, " + ") end
    if a.as ~= nil then
      -- the value of the picked option, bound for the rest of the list
      local codes, ty = {}, nil
      for i, o in ipairs(opts) do
        if o.value == nil or o.actions ~= nil then self:block("action:random_pick with as: options give a value"); return "" end
        local c, t = self:value(o.value, ctx)
        if ty and t ~= ty then self:block("action:random_pick values of different types"); return "" end
        codes[i], ty = c, t
      end
      if not ty or ty == "err" or ty:match("^sym") then return "" end
      local chain = ""
      for i = 1, #codes - 1 do chain = chain .. "if roll < " .. bound(i) .. " { " .. codes[i] .. " } else " end
      chain = chain .. "{ " .. codes[#codes] .. " }"
      local ws = {}
      for i, w in ipairs(weights) do ws[#ws + 1] = "let w" .. i .. " = (" .. w .. ").max(Fixed64::ZERO);" end
      local marker = self:bind(ctx.scope, a.as, ty)
      -- one expression: the binding outlives the weights
      return line("let " .. marker .. " = { " .. table.concat(ws, " ") .. " let total = " .. table.concat(total, " + ")
        .. "; let roll = cx.random(Fixed64::ZERO, total); " .. chain .. " };")
    end
    local body = head
    for i, o in ipairs(opts) do
      if o.value ~= nil then self:block("action:random_pick value without as"); return "" end
      local cond = i < #opts and ("if roll < " .. bound(i) .. " {") or "{"
      body = body .. line("    " .. (i > 1 and "else " or "") .. cond)
        .. self:actions(o.actions or {}, { owner = ctx.owner, scope = scope_child(ctx.scope), event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }, I .. "        ") .. line("    }")
    end
    return body .. line("}")
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
    if s.team == "ally" and (s.shape or "circle") == "circle" then
      -- a body that only restores shares of max HP / mana is the host's ally_area_restore
      local before = {}
      for k in pairs(self.blockers) do before[k] = true end
      local code = self:ally_restore(a, ctx, line)
      local fresh = false
      for k in pairs(self.blockers) do if not before[k] then fresh = true end end
      if not fresh then return code end
      for k in pairs(self.blockers) do if not before[k] then self.blockers[k] = nil end end
    end
    local team_kind = (s.team == "ally" or s.team == "ally_not_self") and "ally" or s.team == "both" and "both" or nil
    for _, k in ipairs(sorted_keys(s)) do if not SELECTOR_FIELDS[k] then self:block("selector." .. k) end end
    -- global: a circle past the map's diagonal (omoba_sim terrain coordinates stay within 2^19)
    local global = s.shape == "global"
    if global and s.radius ~= nil then self:block("selector.shape:global with a radius") end
    if (s.shape or "circle") ~= "circle" and not global then self:block("selector.shape:" .. s.shape) end
    if s.team ~= "enemy" and not team_kind then self:block("selector.team:" .. tostring(s.team)) end
    local types = SELECT_TYPES[types_key(s.types)]
    if not types then self:block("selector.types:" .. types_key(s.types)); types = "All" end
    local mi, inv, hid, ill = false, false, false, false
    local skip = {}
    for _, f in ipairs(s.flags or {}) do
      if f == "include_magic_immune" then mi = true elseif f == "include_invulnerable" then inv = true
      -- include_hidden: Select.include_hidden (ability_kit hidden_excluded: no hidden state in the host yet)
      elseif f == "include_hidden" then hid = true
      -- include_illusions: Select.include_illusions (a hero selection also takes illusions)
      elseif f == "include_illusions" then ill = true
      -- exclude_illusions / exclude_invisible: such a unit is skipped (before `where` and `limit`)
      elseif f == "exclude_illusions" then skip[#skip + 1] = "IS_ILLUSION"
      elseif f == "exclude_invisible" then skip[#skip + 1] = "IS_INVISIBLE"
      -- visible_only: enemies_in_circle already takes the caster team's disclosed units
      elseif f ~= "include_untargetable" and f ~= "visible_only" then self:block("selector.flag:" .. f) end
    end
    if ill and table.concat(skip, ","):find("IS_ILLUSION", 1, true) then self:block("selector.flag: include_illusions with exclude_illusions") end
    if s.order ~= nil and s.order ~= "id" and s.order ~= "nearest" and s.order ~= "farthest" and s.order ~= "random" then self:block("selector.order:" .. tostring(s.order)) end
    if s.exclude ~= nil and s.team == "ally_not_self" then self:block("selector.exclude with team ally_not_self") end
    local exclude = s.exclude ~= nil and self:value(s.exclude, ctx, "unit")
      or s.team == "ally_not_self" and self:value("self", ctx, "unit") or nil
    local from = s.from ~= nil and self:value(s.from, ctx, "point") or "cx.pos(cx.caster)?"
    local radius = global and "Fixed64::from_i32(1 << 21)" or self:value(s.radius, ctx, "num")
    local limit = s.limit ~= nil and self:value(s.limit, ctx, "num") or nil
    local body_scope = scope_child(ctx.scope)
    local it = self:bind(body_scope, a.as or "it", "unit")
    local nearest = s.order == "nearest" or s.order == "farthest"
    local farthest = s.order == "farthest"
    local random = s.order == "random"
    -- relative_to: the point nearest / farthest is measured from (default: `from`)
    if s.relative_to ~= nil and not nearest then self:block("selector.relative_to without order nearest/farthest") end
    local origin = s.relative_to ~= nil and nearest and self:value(s.relative_to, ctx, "point") or nil
    local str = line("{") .. line("    let center = " .. from .. ";") .. line("    let radius = " .. radius .. ";")
      .. (exclude and line("    let excluded = " .. exclude .. ";") or "")
      .. (origin and line("    let origin = " .. origin .. ";") or "")
      .. (team_kind
        and line(string.format("    let %sunits = cx.units_in_circle(center, radius, %s, Select { types: UnitTypes::%s, include_magic_immune: %s, include_invulnerable: %s, include_hidden: %s, include_illusions: %s });",
          (nearest or random) and "mut " or "", rust_str(team_kind), types, mi and "true" or "false", inv and "true" or "false", hid and "true" or "false", ill and "true" or "false"))
        or line(string.format("    let %sunits = cx.enemies_in_circle(center, radius, Select { types: UnitTypes::%s, include_magic_immune: %s, include_invulnerable: %s, include_hidden: %s, include_illusions: %s });",
          (nearest or random) and "mut " or "", types, mi and "true" or "false", inv and "true" or "false", hid and "true" or "false", ill and "true" or "false")))
      .. (nearest and line("    cx.nearest_first(&mut units, " .. (origin and "origin" or "center") .. ");") or "")
      .. (farthest and line("    units.reverse();") or "")
      .. (random and line("    cx.shuffle(&mut units);") or "")
      .. (limit and (line("    let limit = usize::try_from(ability_kit::floor(" .. limit .. ").raw() >> 10).unwrap_or(0);")
        .. line("    let mut taken = 0usize;")) or "")
      .. line("    for " .. it .. " in units {")
      .. (exclude and line("        if " .. it .. " == excluded { continue; }") or "")
    for _, flag in ipairs(skip) do
      str = str .. line("        if cx.unit_flag(" .. it .. ", ability_kit::unit_attr::" .. flag .. ") { continue; }")
    end
    if s.where then
      -- the selector's `where` names the candidate `it`
      local wscope = scope_child(ctx.scope)
      wscope.names.it = body_scope.names[a.as or "it"]
      local w = self:value(s.where, { owner = ctx.owner, scope = wscope, event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }, "bool")
      str = str .. line("        if !(" .. w .. ") { continue; }")
    end
    -- `limit` counts the units that pass `where`, in id order
    if limit then str = str .. line("        if taken >= limit { break; }") .. line("        taken += 1;") end
    str = str .. self:actions(a.actions, { owner = ctx.owner, scope = body_scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }, I .. "        ")
    return str .. line("    }") .. line("}")
  elseif op == "create_unit" then
    local CREATE_OVERRIDES = { hp = true, attack_damage = true, attack_damage_max = true, move_speed = true }
    local t = self.m.templates[a.template]
    -- the host spawns content summons only (summon_by_name): a v3 template is one through
    -- scripts/v3_summons.lua, a ranked one (`max_level` > 1) `<id>_r<rank>` at this cast's rank (E326)
    local ranked = t and (t.def.max_level or 1) > 1
    local row_id = ranked and (a.template .. "_r1") or a.template
    if not t then self:block("template:" .. tostring(a.template) .. " not declared")
    elseif not self.opts.summons[row_id] then
      local why = self.opts.summon_problems[a.template]
      if why then for _, p in ipairs(why) do self:block("unit_template " .. p) end
      else self:block("template:" .. tostring(a.template) .. " is not a content summon (templates/summons.lua)") end
    end
    -- a chase unit's on_reach runs in this entry's on_status_event (CHASE_REACH, line B E406)
    if t and t.def.chase and t.def.chase.on_reach then
      if not self.entry then self:block("create_unit of a chase_target template in a routine")
      else self.chase_reach = self.chase_reach or {}; self.chase_reach[a.template] = t end
    end
    local template = ranked and ("&format!(\"" .. a.template .. "_r{}\", cx.rank.clamp(1, " .. t.def.max_level .. "))") or rust_str(a.template)
    -- control: "ai" (default) and "owner" the host summon; "none" also refuses its owner's orders
    -- (line B's create_unit_controlled)
    if a.control ~= nil and a.control ~= "ai" and a.control ~= "owner" and a.control ~= "none" then
      self:block("action:create_unit.control:" .. tostring(a.control))
    end
    local make = (a.control == nil or a.control == "ai") and ("cx.create_unit(" .. template .. ", at, lifetime)?")
      or ("cx.create_unit_controlled(" .. template .. ", at, lifetime, " .. rust_str(a.control) .. ")?")
    if a.facing ~= nil then
      -- the new unit turns to `facing` (host: the cast's own summon may be turned, line B)
      local f, ft = self:value(a.facing, ctx)
      if ft ~= "vec" and ft ~= "point" and ft ~= "err" then self:block("action:create_unit.facing of a " .. tostring(ft)) end
      make = "{ let facing = " .. f .. "; let u = " .. make .. "; cx.face_dir(u, facing); u }"
    end
    if a.overrides ~= nil then
      -- the template's values replaced right after the spawn (host unit_overrides; the values
      -- are read before the unit exists, in key order)
      local lets, rows = {}, {}
      for _, k in ipairs(sorted_keys(a.overrides)) do
        if not CREATE_OVERRIDES[k] then self:block("action:create_unit.overrides." .. tostring(k))
        else
          local name = "ov_" .. k
          lets[#lets + 1] = "let " .. name .. " = " .. self:value(a.overrides[k], ctx, "num") .. "; "
          rows[#rows + 1] = "(" .. rust_str(k) .. ", " .. name .. ")"
        end
      end
      make = "{ " .. table.concat(lets) .. "let u = " .. make .. "; cx.unit_overrides(u, &[" .. table.concat(rows, ", ") .. "])?; u }"
    end
    -- the template's `statuses` go on the new unit as it appears, applied by this cast like an
    -- `apply_status` to it (v3_summons rows carry no statuses)
    local statuses = t and t.def.statuses or {}
    if t and t.passive_status then
      local all = {}
      for _, sid in ipairs(statuses) do all[#all + 1] = sid end
      all[#all + 1] = t.passive_status
      statuses = all
    end
    if #statuses > 0 then
      local scope = scope_child(ctx.scope)
      local sub = setmetatable({ scope = scope }, { __index = ctx })
      local var = self:bind(scope, "created", "unit")
      local list = {}
      for _, sid in ipairs(statuses) do
        if type(sid) ~= "string" then self:block("unit_template statuses entry of a " .. type(sid))
        else list[#list + 1] = { op = "apply_status", status = sid, to = "created" } end
      end
      make = "{ let " .. var .. " = " .. make .. ";\n" .. self:actions(list, sub, I .. "    ") .. I .. "    " .. var .. " }"
    end
    -- a dummy template's passive aura: emitted on the new unit by this entry (its on_aura arm
    -- `<template id>`; v3_summons leaves it to the creating code), for as long as the unit lives
    local ta = t and t.def.passive and t.def.passive.aura
    if ta then
      if not self.entry then self:block("action:create_unit of a template with an aura in a routine")
      else
        local emit = self:aura_emit(ta, { owner = t, scope = scope_child(nil), event = false }, a.template, "u", "None", "false")
        make = "{ let u = " .. make .. "; " .. emit .. "?; u }"
        self.template_auras = self.template_auras or {}; self.template_auras[a.template] = t
      end
    end
    -- limit_per_owner with over_limit remove_oldest: after the spawn, the caster's live summons
    -- of this template beyond the limit leave, earliest first, when the batch settles (kit
    -- limit_summons, line B E418; any path: a cast, a delay, a periodic)
    if a.limit_per_owner ~= nil or a.over_limit ~= nil then
      if a.limit_per_owner == nil then self:block("action:create_unit.over_limit without limit_per_owner")
      elseif a.over_limit ~= "remove_oldest" then self:block("action:create_unit.over_limit:" .. tostring(a.over_limit))
      else
        make = "{ let u = " .. make .. "; let limit = " .. self:value(a.limit_per_owner, ctx, "num") .. "; cx.limit_summons("
          .. template .. ", limit)?; u }"
      end
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
    s = s .. line("    let at = " .. at .. ";") .. line("    let lifetime = " .. life .. ";")
    if a.as then
      local marker = self:bind(ctx.scope, a.as, "unit")
      -- the binding lives after the block: hoist it
      return line("let " .. marker .. " = {") .. s:gsub("^" .. I:gsub("%p", "%%%0") .. "{\n", "")
        .. line("    " .. make) .. line("};")
    end
    return s .. line("    " .. make .. ";") .. line("}")
  elseif op == "reveal" then
    if a.teams ~= "own" and a.teams ~= "all" then self:block("action:reveal.teams:" .. tostring(a.teams)) end
    local at = self:value(a.at, ctx, "point")
    local radius = self:value(a.radius, ctx, "num")
    local dur = self:value(a.duration, ctx, "num")
    local follow = a.follow ~= nil and ("Some(" .. self:value(a.follow, ctx, "unit") .. ")") or "None"
    return line("{") .. line("    let at = " .. at .. ";") .. line("    let radius = " .. radius .. ";")
      .. line("    let duration = " .. dur .. ";") .. line("    let follow = " .. follow .. ";")
      .. (a.ignore_terrain == nil and line(string.format("    cx.reveal(at, radius, duration, %s, follow)?;", a.teams == "all" and "true" or "false"))
        -- line B's reveal_ex: trees and cliffs do not block this sight (brush still does)
        or line(string.format("    cx.reveal_ex(at, radius, duration, %s, follow, %s)?;", a.teams == "all" and "true" or "false",
          type(a.ignore_terrain) == "boolean" and tostring(a.ignore_terrain) or self:value(a.ignore_terrain, ctx, "bool")))) .. line("}")
  elseif op == "call" then
    local r = self.m.routines[a.routine]
    if not r then self:block("routine:" .. tostring(a.routine) .. " not declared"); return "" end
    if self.entry and routine_needs_entry(self.m, a.routine) then
      if routine_recursive(self.m, a.routine) then
        -- a routine that calls itself again (a bouncing bolt): one function of this entry's module,
        -- generated once; its steps are this entry's, and the call inside it calls it again
        local fname = "inl_" .. a.routine
        self.local_fn_state = self.local_fn_state or {}
        if not self.local_fn_state[a.routine] then
          self.local_fn_state[a.routine] = true
          local sc = scope_child(nil)
          local params = {}
          for _, pname in ipairs(sorted_keys(r.params or {})) do
            local kind = r.params[pname]
            local ty = ({ num = "Fixed64", bool = "bool", unit = "EntityHandle", point = "Vec2", vec = "Vec2" })[kind]
            if not ty then self:block("routine param kind " .. tostring(kind)); ty = "Fixed64" end
            params[#params + 1] = self:bind(sc, pname, (kind == "unit" or kind == "point" or kind == "bool" or kind == "vec") and kind or "num") .. ": " .. ty
          end
          self.paths = self.paths or {}
          for node, path in pairs(effect_paths(r, "routines." .. a.routine)) do if self.paths[node] == nil then self.paths[node] = path end end
          local owner = r.owner and self.m.abilities[r.owner] or ctx.owner
          local body = self:actions(r.actions, { owner = owner, scope = sc, event = false }, "    ")
          self.local_fns = self.local_fns or {}
          self.local_fns[#self.local_fns + 1] = string.format("/// routine `%s`, laid out in this entry (it calls itself).\nfn %s(cx: &mut Cx<'_, '_>%s) -> Res {\n%s    Ok(())\n}\n",
            a.routine, fname, #params > 0 and (", " .. table.concat(params, ", ")) or "", body)
        end
        local args = {}
        for _, pname in ipairs(sorted_keys(r.params or {})) do
          local v = (a.args or {})[pname]
          local kind = r.params[pname]
          local want = (kind == "unit" or kind == "point" or kind == "bool") and kind or (kind == "vec" and "vec" or "num")
          if v == nil then self:block("routine " .. a.routine .. ": missing arg " .. pname); return "" end
          args[#args + 1] = self:value(v, ctx, want)
        end
        -- the arguments first: their reads take &mut cx
        local lets, names = {}, {}
        for i, code in ipairs(args) do lets[#lets + 1] = line("    let a" .. i .. " = " .. code .. ";"); names[#names + 1] = "a" .. i end
        return line("{") .. table.concat(lets) .. line("    " .. fname .. "(cx" .. (#names > 0 and ", " or "") .. table.concat(names, ", ") .. ")?;") .. line("}")
      end
      self.inlining = self.inlining or {}
      if self.inlining[a.routine] then self:block("routine " .. a.routine .. ": recursive with timed steps"); return "" end
      -- a routine with timed steps (delay, repeat, projectile, tether) runs as part of the calling
      -- entry: its steps are the entry's (a routine function has no ability of its own to schedule)
      local sc = scope_child(ctx.scope)
      local lets = {}
      for _, pname in ipairs(sorted_keys(r.params or {})) do
        local v = (a.args or {})[pname]
        local kind = r.params[pname]
        local want = (kind == "unit" or kind == "point" or kind == "bool") and kind or (kind == "vec" and "vec" or "num")
        if v == nil then self:block("routine " .. a.routine .. ": missing arg " .. pname); return "" end
        local code = self:value(v, ctx, want)
        lets[#lets + 1] = line("    let " .. self:bind(sc, pname, (kind == "unit" or kind == "point" or kind == "bool" or kind == "vec") and kind or "num") .. " = " .. code .. ";")
      end
      self.paths = self.paths or {}
      for node, path in pairs(effect_paths(r, "routines." .. a.routine)) do if self.paths[node] == nil then self.paths[node] = path end end
      local owner = r.owner and self.m.abilities[r.owner] or ctx.owner
      self.inlining[a.routine] = true
      local body = self:actions(r.actions, { owner = owner, scope = sc, event = ctx.event, pre = ctx.pre, cast = ctx.cast,
        ended = ctx.ended, windup = ctx.windup }, I .. "    ")
      self.inlining[a.routine] = nil
      return line("{") .. table.concat(lets) .. body .. line("}")
    end
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
    if #args == 0 then return line("super::routines::" .. a.routine .. "(cx)?;") end
    -- the arguments first: their reads take &mut cx
    local lets, names = {}, {}
    for i, code in ipairs(args) do lets[#lets + 1] = line("    let a" .. i .. " = " .. code .. ";"); names[#names + 1] = "a" .. i end
    return line("{") .. table.concat(lets) .. line("    super::routines::" .. a.routine .. "(cx, " .. table.concat(names, ", ") .. ")?;") .. line("}")
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
    local head = line("{") .. line("    let time = " .. time .. ";")
      .. line("    let args = [" .. table.concat(args, ", ") .. "];")
      .. line("    let owner = cx.caster;")
    if a.id == nil then return head .. line(string.format("    cx.schedule(owner, ID, %d, time, &args)?;", step)) .. line("}") end
    -- a named delay: its schedule instance and step on the caster, for cancel_delay
    local key = self:delay_key(a.id)
    return head .. line(string.format("    let instance = cx.schedule(owner, ID, %d, time, &args)?;", step))
      .. line(string.format("    cx.set_state_num(owner, %s, Fixed64::from_i32(i32::try_from(instance).unwrap_or(-1)), false)?;", rust_str(key)))
      .. line(string.format("    cx.set_state_num(owner, %s, Fixed64::from_i32(%d), false)?;", rust_str(key .. ":step"), step)) .. line("}")
  elseif op == "cancel_delay" then
    -- the named delay this caster scheduled last, if it has not run yet (v3.md §5: delay.id)
    local entry = self:delay_entry(a.id)
    if not entry then return "" end
    local key = self:delay_key(a.id)
    return line("{") .. line("    let owner = cx.caster;")
      .. line(string.format("    let instance = cx.state_num(owner, %s, -Fixed64::ONE).raw() >> 10;", rust_str(key)))
      .. line(string.format("    let step = cx.state_num(owner, %s, -Fixed64::ONE).raw() >> 10;", rust_str(key .. ":step")))
      .. line("    if let (Ok(instance), Ok(step)) = (u32::try_from(instance), u16::try_from(step)) {")
      .. line(string.format("        cx.cancel_schedule(owner, %s, step, instance);", rust_str(entry)))
      .. line(string.format("        cx.set_state_num(owner, %s, -Fixed64::ONE, false)?;", rust_str(key)))
      .. line("    }") .. line("}")
  elseif op == "repeat" then
    if not self.entry then self:block("action:repeat in a routine"); return "" end
    if a.count == nil and a["while"] == nil and a.stop_when == nil then self:block("action:repeat without an end"); return "" end
    if a.interval == nil then
      -- every run within this step (v3 §12.11), bounded
      local body_scope = scope_child(ctx.scope)
      local i = self:bind(body_scope, a.as or "repeat_index", "num")
      local bctx = { owner = ctx.owner, scope = body_scope, event = ctx.event, pre = ctx.pre, cast = ctx.cast, ended = ctx.ended, windup = ctx.windup }
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
  elseif op == "create_pickup" then
    -- host pickup (line-gamemode): the picker gets `gold`; only that on_pickup is lowered
    local give = a.on_pickup and #a.on_pickup == 1 and a.on_pickup[1]
    local picker = give and (give.to == "it.hero" or give.to == "it")
    if not (give and give.op == "give_gold" and picker and give.reliable ~= true and give.show == nil) then
      self:block("action:create_pickup.on_pickup (only one give_gold to the picker)"); return ""
    end
    local who = ({ any = "any", ally = "allies", enemy = "enemies" })[a.who or "any"]
    if not who then self:block("action:create_pickup.who:" .. tostring(a.who)); return "" end
    if a.cue then self:gap("create_pickup.cue: the host pickup draws its own bag") end
    local at = self:value(a.at, ctx, "point")
    local gold = self:value(give.amount, ctx, "num")
    local lifetime = a.lifetime ~= nil and self:value(a.lifetime, ctx, "num") or "Fixed64::ZERO"
    return line("{") .. line("    let at = " .. at .. ";") .. line("    let gold = " .. gold .. ";")
      .. line(string.format("    cx.create_pickup(at, gold, %s, %s, %s)?;", rust_str(who), tostring(a.killable == true), lifetime)) .. line("}")
  elseif op == "create_illusion" then
    -- an `illusion` summon of the caster that copies `of` (host illusion.rs, line B t76): its body,
    -- level and items always; life (default: `of`'s), mana (default: `of`'s current), learned
    -- abilities (default) and the statuses `copy.statuses = { "all" }` names
    local c = a.copy or {}
    local COPY = set({ "level", "abilities", "except_abilities", "items", "statuses", "hp", "mana", "unit_state",
      "status_duration", "status_caster", "status_tags", "status_except", "team" })
    for _, k in ipairs(sorted_keys(c)) do if not COPY[k] then self:block("action:create_illusion.copy." .. k) end end
    if c.level == false then self:block("action:create_illusion.copy.level = false") end
    if c.items ~= nil and c.items ~= "fresh" then self:block("action:create_illusion.copy.items:" .. tostring(c.items)) end
    if c.unit_state == true then self:block("action:create_illusion.copy.unit_state = true") end
    if c.team ~= nil and c.team ~= "caster" then self:block("action:create_illusion.copy.team:" .. tostring(c.team)) end
    if a.control ~= nil and a.control ~= "ai" and a.control ~= "owner" and a.control ~= "none" then
      self:block("action:create_illusion.control:" .. tostring(a.control))
    end
    local function strs(list)
      local out = {}
      for _, x in ipairs(list or {}) do out[#out + 1] = rust_str(x) .. ".to_owned()" end
      return "vec![" .. table.concat(out, ", ") .. "]"
    end
    local statuses = "None"
    local st = c.statuses
    if type(st) == "table" and #st > 0 then
      if #st ~= 1 or st[1] ~= "all" then self:block("action:create_illusion.copy.statuses other than { \"all\" }") end
      if c.status_duration ~= nil and c.status_duration ~= "remaining" and c.status_duration ~= "original" then
        self:block("action:create_illusion.copy.status_duration:" .. tostring(c.status_duration))
      end
      if c.status_caster ~= nil and c.status_caster ~= "original" and c.status_caster ~= "illusion" then
        self:block("action:create_illusion.copy.status_caster:" .. tostring(c.status_caster))
      end
      statuses = string.format("Some(ability_kit::IllusionStatusCopy { tags: %s, except: %s, original_duration: %s, illusion_caster: %s })",
        strs(c.status_tags), strs(c.status_except), tostring(c.status_duration == "original"), tostring(c.status_caster == "illusion"))
    elseif st ~= nil and type(st) ~= "table" then self:block("action:create_illusion.copy.statuses of an expression") end
    -- outgoing / incoming: per type (physical, magical, pure) fractions, `all` for the rest
    local function scale(t, what)
      t = t or {}
      for _, k in ipairs(sorted_keys(t)) do
        if k ~= "all" and k ~= "physical" and k ~= "magical" and k ~= "pure" then self:block("action:create_illusion." .. what .. "." .. k) end
      end
      local parts, rest = {}, false
      for _, k in ipairs({ "physical", "magical", "pure" }) do
        if t[k] ~= nil then parts[#parts + 1] = self:value(t[k], ctx, "num") else parts[#parts + 1] = "all_" .. what; rest = true end
      end
      local all = rest and ("let all_" .. what .. " = " .. (t.all ~= nil and self:value(t.all, ctx, "num") or "Fixed64::ZERO") .. "; ") or ""
      return all .. "let " .. what .. " = ability_kit::IllusionScale([" .. table.concat(parts, ", ") .. "]);"
    end
    local of = self:value(a.of, ctx, "unit")
    local at = self:value(a.at, ctx, "point")
    local lifetime = self:value(a.duration, ctx, "num")
    local hp = c.hp ~= nil and ("Some(" .. self:value(c.hp, ctx, "num") .. ")") or "None"
    local mana = c.mana ~= nil and ("Some(" .. self:value(c.mana, ctx, "num") .. ")") or "Some(cx.mana(of))"
    local abilities = c.abilities == false and "None" or ("Some(" .. strs(c.except_abilities) .. ")")
    local body = line("{") .. line("    let of = " .. of .. ";") .. line("    let at = " .. at .. ";")
      .. line("    let lifetime = " .. lifetime .. ";")
      .. line("    let hp = " .. hp .. ";") .. line("    let mana = " .. mana .. ";")
      .. line("    " .. scale(a.outgoing, "outgoing")) .. line("    " .. scale(a.incoming, "incoming"))
      .. line("    let copy = ability_kit::IllusionCopy { hp, mana, abilities: " .. abilities .. ", statuses: " .. statuses .. ", outgoing, incoming };")
    local make = "cx.create_illusion(of, at, lifetime, " .. rust_str(a.control or "ai") .. ", &copy)?"
    if a.as then
      -- the binding outlives the block: declared before it
      local marker = self:bind(ctx.scope, a.as, "unit")
      return line("let " .. marker .. " = {") .. body:gsub("^" .. I:gsub("%p", "%%%0") .. "{\n", "") .. line("    " .. make) .. line("};")
    end
    return body .. line("    let _ = " .. make .. ";") .. line("}")
  elseif op == "end_channel" then
    if a.interrupted ~= true then self:block("action:end_channel without interrupted = true"); return "" end
    return line("cx.end_channel(cx.self_unit);")
  elseif op == "issue_order" then
    -- as the owner's order (hero_command; a summon, illusion or dummy takes it too, line B E412); nothing is locked
    if a.force ~= nil and type(a.force) ~= "boolean" then self:block("action:issue_order.force of an expression"); return "" end
    local who = self:value(a.who, ctx, "unit")
    if a.order == "attack" and a.target ~= nil and a.at == nil then
      local target = self:value(a.target, ctx, "unit")
      return line("{") .. line("    let who = " .. who .. ";") .. line("    let target = " .. target .. ";") .. line("    cx.order_attack(who, target)?;") .. line("}")
    end
    -- follow: a summon keeps walking to the target until a new order or its death (line B E489;
    -- a hero does not take it)
    if a.order == "follow" and a.target ~= nil and a.at == nil then
      local target = self:value(a.target, ctx, "unit")
      return line("{") .. line("    let who = " .. who .. ";") .. line("    let target = " .. target .. ";") .. line("    cx.order_follow(who, target)?;") .. line("}")
    end
    local kinds = { stop = "Stop", hold = "Hold", move = "Move", attack_move = "AttackMove" }
    local kind = kinds[a.order]
    if not kind or a.target ~= nil then self:block("action:issue_order.order:" .. tostring(a.order)); return "" end
    local at = "Vec2::ZERO"
    if kind == "Move" or kind == "AttackMove" then
      if a.at == nil then self:block("action:issue_order " .. a.order .. " without at"); return "" end
      at = self:value(a.at, ctx, "point")
    elseif a.at ~= nil then self:block("action:issue_order " .. a.order .. " with at"); return "" end
    return line("{") .. line("    let who = " .. who .. ";") .. line("    let at = " .. at .. ";")
      .. line("    cx.order(who, ability_kit::OrderKind::" .. kind .. ", at)?;") .. line("}")
  elseif op == "attack" then
    -- a projectile attack lands its effects on arrival, so it carries them (the kit refuses one without)
    if a.instant ~= true and a.trigger_effects ~= true then self:block("action:attack projectile without trigger_effects"); return "" end
    if a.effect_multiplier ~= nil and a.effect_multiplier ~= 1 and a.effect_multiplier ~= "1" then self:block("action:attack.effect_multiplier"); return "" end
    for _, k in ipairs({ "use_orb", "trigger_effects", "fake", "cannot_miss", "ignore_range" }) do
      if a[k] ~= nil and type(a[k]) ~= "boolean" then self:block("action:attack." .. k .. " of an expression"); return "" end
    end
    local function flag(k) return a[k] == true and "true" or "false" end
    local attacker = self:value(a.attacker, ctx, "unit")
    local target = self:value(a.target, ctx, "unit")
    return line("{") .. line("    let attacker = " .. attacker .. ";") .. line("    let target = " .. target .. ";")
      .. line(string.format("    cx.perform_attack(attacker, target, %s, %s, %s, %s, %s, Fixed64::ONE)?;", flag("instant"), flag("cannot_miss"), flag("fake"),
        flag("trigger_effects"), flag("use_orb"))) .. line("}")
  elseif op == "set_ability_active" or op == "swap_abilities" then
    local function id_of(v) return type(v) == "string" and v:match("^[%a_][%w_]*$") and v end
    if op == "set_ability_active" then
      local ab = id_of(a.ability)
      if not ab or type(a.active) ~= "boolean" then self:block("action:set_ability_active (an ability id and a fixed active)"); return "" end
      local of = self:value(a.of, ctx, "unit")
      return line("{") .. line("    let of = " .. of .. ";") .. line(string.format("    cx.set_ability_active(of, %s, %s)?;", rust_str(ab), tostring(a.active))) .. line("}")
    end
    local from, to = id_of(a.from), id_of(a.to)
    if not (from and to) then self:block("action:swap_abilities (ability ids)"); return "" end
    if a.cooldown_policy ~= "continue" and a.cooldown_policy ~= "reset" then self:block("action:swap_abilities.cooldown_policy:" .. tostring(a.cooldown_policy)); return "" end
    local on = self:value(a.on, ctx, "unit")
    local dur = a.duration ~= nil and ("Some(" .. self:value(a.duration, ctx, "num") .. ")") or "None"
    return line("{") .. line("    let on = " .. on .. ";") .. line("    let duration = " .. dur .. ";")
      .. line(string.format("    cx.swap_abilities(on, %s, %s, duration, %s)?;", rust_str(from), rust_str(to), rust_str(a.cooldown_policy))) .. line("}")
  elseif op == "set_mana" then
    -- line B: a closed request, clamped to the pool, in any context (status events, schedules)
    local to = self:value(a.to, ctx, "unit")
    local value = self:value(a.value, ctx, "num")
    return line("{") .. line("    let to = " .. to .. ";") .. line("    let value = " .. value .. ";") .. line("    cx.set_mana(to, value)?;") .. line("}")
  elseif op == "set_var" then
    -- the status: named, or the one whose trigger this is; the holder: `on`, or that status' holder
    local so = a.status and self.m.statuses[a.status] or (ctx.owner and ctx.owner.kind == "status" and ctx.owner)
    local v = so and so.def.vars and so.def.vars[a.var]
    local ty = v and VAR_TYPES[v.type]
    if not ty then self:block("action:set_var (" .. tostring(a.var) .. " not a var of its status)"); return "" end
    if so.def.stacking == "independent" then self:block("status.vars of an independent status"); return "" end
    local on = a.on ~= nil and self:value(a.on, ctx, "unit") or "cx.self_unit"
    local value = self:value(a.value, ctx, ty)
    local call = var_write(ty, "on", status_var_key(so.def.id, a.var), "value")
    return line("{") .. line("    let on = " .. on .. ";") .. line("    let value = " .. value .. ";") .. line("    " .. call) .. line("}")
  elseif op == "order_attack" then
    local who = self:value(a.who, ctx, "unit")
    local target = self:value(a.target, ctx, "unit")
    return line("{") .. line("    let who = " .. who .. ";") .. line("    let target = " .. target .. ";") .. line("    cx.order_attack(who, target)?;") .. line("}")
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
  elseif op == "burn_mana" and a.source ~= nil and a.source ~= "self" then
    -- line B: the burn credited to another unit (cx.burn_mana_from)
    local src = self:value(a.source, ctx, "unit")
    local to = self:value(a.to, ctx, "unit")
    local amount = self:value(a.amount, ctx, "num")
    return line("{") .. line("    let source = " .. src .. ";") .. line("    let to = " .. to .. ";") .. line("    let amount = " .. amount .. ";")
      .. line("    cx.burn_mana_from(source, to, amount)?;") .. line("}")
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
    -- the host dodges every team's tracking flights (line B), so `enemy` and `any` are the same
    if a.from_team ~= nil and a.from_team ~= "enemy" and a.from_team ~= "any" then self:block("action:dodge_projectiles.from_team:" .. tostring(a.from_team)) end
    if a.kind ~= nil and a.kind ~= "tracking" then self:block("action:dodge_projectiles.kind:" .. tostring(a.kind)) end
    return line("cx.dodge_projectiles()?;")
  elseif op == "chance" then
    -- independent rolls (prd is another field: blocked by the field check); pity counts misses in a
    -- unit var of the caster: `counter` names its range, `counter_var` a declared int state
    local p = self:value(a.p, ctx, "num")
    local pity = ""
    local roll = "cx.chance(p)"
    if a.pity_after ~= nil then
      local key, hero
      if a.counter_var ~= nil then
        local v = self.m.vars[a.counter_var]
        if not v or (v.def.type ~= "int" and v.def.type ~= "fixed") then self:block("action:chance.counter_var:" .. tostring(a.counter_var)); return "" end
        key, hero = v.key, v.def.scope == "hero"
      else
        local owner = self.entry and self.entry.def.id or self.trace_owner or "?"
        local ranges = { ability = owner, hero = "hero", item_name = owner, status = (ctx.owner and ctx.owner.kind == "status" and ctx.owner.def.id) or owner }
        local range = ranges[a.counter or "ability"]
        if not range then self:block("action:chance.counter:" .. tostring(a.counter)); return "" end
        key, hero = "__pity:" .. range, a.counter == "hero"
      end
      local after = self:value(a.pity_after, ctx, "num")
      pity = line("    let pity_key = " .. rust_str(key) .. ";") .. line("    let misses = cx.state_num(cx.caster, pity_key, Fixed64::ZERO);")
        .. line("    let hit = misses >= " .. after .. " || cx.chance(p);")
        .. line(string.format("    cx.set_state_num(cx.caster, pity_key, if hit { Fixed64::ZERO } else { misses + Fixed64::ONE }, %s)?;", tostring(hero)))
      roll = "hit"
    elseif a.counter ~= nil or a.counter_var ~= nil then self:block("action:chance.counter without pity_after"); return "" end
    local s2 = line("{") .. line("    let p = " .. p .. ";") .. pity .. line("    if " .. roll .. " {")
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
    -- `from` another point or unit: the flight starts there (host ability_projectile `start`)
    local start = "None"
    if a.from ~= nil and a.from ~= "caster" and not (a.from == "self" and self.entry) then
      local code, ty = self:value(a.from, ctx)
      if ty == "point" then start = "Some(" .. code .. ")"
      elseif ty == "unit" then start = "Some(cx.pos(" .. code .. ")?)"
      else self:block("action:projectile.from:" .. tostring(a.from)) end
    end
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
    local to_point
    if tracking and a.to ~= nil then
      local code, ty = self:value(a.to, ctx)
      if ty == "point" then to_point = code end
    end
    if to_point then
      -- tracking a point: it flies straight there and ends (nothing on the way is hit)
      if a.on_hit and #a.on_hit > 0 then self:block("action:projectile tracking a point with on_hit"); return "" end
      local origin = start ~= "None" and start:match("^Some%((.*)%)$") or "cx.pos(cx.caster)?"
      aim = to_point
      distance = "{ let o: Vec2 = " .. origin .. "; let p: Vec2 = " .. to_point .. "; ability_kit::distance(o, p) }"
    elseif tracking then
      homing = "Some(" .. self:value(a.to, ctx, "unit") .. ")"
    else
      distance = self:value(a.distance, ctx, "num")
      local dir = self:value(a.direction, ctx, "vec")
      -- from its start point when it has one (`from` a point or unit), else from the caster
      local origin = start ~= "None" and start:match("^Some%((.*)%)$") or "cx.pos(cx.caster)?"
      aim = "ability_kit::offset(" .. origin .. ", " .. dir .. ", " .. distance .. ")"
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
      args_line = line("    let hit_args = [" .. table.concat(args, ", ") .. "];")
      on_hit = string.format("Some((%d, &hit_args[..]))", step)
    end
    local on_end = "None"
    if a.on_end and #a.on_end > 0 then
      -- once when the flight retires; `at` / `here` is where it ended
      local sc = step_scope(ctx, 0)
      sc.names.at = { name = "at", ty = "point", marker = "ability_kit::arg_as_point(args, \1END\1)", used = true }
      sc.names.here = sc.names.at
      local step = self:reserve_step()
      local body = self:actions(a.on_end, { owner = ctx.owner, scope = sc, event = false }, "    ")
      local eargs = self:capture_args(sc)
      if #eargs + 1 > 8 then self:block("projectile.on_end: more than 7 captured values") end
      body = body:gsub("\1END\1", tostring(#eargs))
      self:set_step(step, "`projectile` on_end", body, true)
      args_line = args_line .. line("    let end_args = [" .. table.concat(eargs, ", ") .. "];")
      on_end = string.format("Some((%d, &end_args[..]))", step)
    end
    return line("{") .. args_line
      .. line(string.format("    let p = Proj { homing: %s, aim: %s, speed: %s, radius: %s, distance: %s, max_hits: %s, buildings: %s, dodgeable: %s, vision: %s, end_radius: %s, start: %s };",
        homing, aim, speed, radius, distance, max_hits, buildings and "true" or "false", a.dodgeable and "true" or "false", vision, end_radius, start))
      .. line("    cx.projectile_ended(ID, p, " .. on_hit .. ", " .. on_end .. ")?;") .. line("}")
  elseif op == "set_state" or op == "clear_state" then
    local v = self.m.vars[a.var]
    if not v then self:block("state:" .. tostring(a.var) .. " not declared"); return "" end
    local ty = ({ int = "num", fixed = "num", bool = "bool", unit = "unit", point = "point", vec = "vec" })[v.def.type]
    if not ty then self:block("state type:" .. tostring(v.def.type)); return "" end
    local on = a.on ~= nil and self:value(a.on, ctx, "unit") or self:expr({ k = "id", name = "self" }, ctx)
    local hero = v.def.scope == "hero" and "true" or "false"
    if ty == "point" or ty == "vec" then
      if op == "clear_state" or a.value == "none" then
        return line("{") .. line("    let on = " .. on .. ";") .. line(string.format("    cx.clear_state(on, %s)?;", rust_str(v.key .. ":x")))
          .. line(string.format("    cx.clear_state(on, %s)?;", rust_str(v.key .. ":y"))) .. line("}")
      end
      return line("{") .. line("    let on = " .. on .. ";") .. line("    let value: Vec2 = " .. self:value(a.value, ctx, ty) .. ";")
        .. line(string.format("    cx.set_state_num(on, %s, value.x, %s)?;", rust_str(v.key .. ":x"), hero))
        .. line(string.format("    cx.set_state_num(on, %s, value.y, %s)?;", rust_str(v.key .. ":y"), hero)) .. line("}")
    end
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
  elseif op == "destroy_tree" then
    -- that one tree (a tree target's centre) falls at the next tick (line B E403)
    return line("{") .. line("    let at = " .. self:value(a.tree, ctx, "point") .. ";") .. line("    cx.destroy_tree(at)?;") .. line("}")
  elseif op == "regrow_trees" then
    -- every fallen tree of the map stands again at the next tick (line B E403)
    return line("cx.regrow_trees()?;")
  elseif op == "refund" then
    -- `cooldown = true` in the cast itself: its own cooldown does not start; the mana stays
    -- spent (line B E455). A refund of mana or health has no host form.
    if a.mana or a.health then self:block("action:refund of mana or health"); return "" end
    if a.cooldown ~= true then self:block("action:refund without cooldown"); return "" end
    if not ctx.cast then self:block("action:refund outside the cast"); return "" end
    return line("cx.refund_cooldown();")
  elseif op == "set_time_of_day" then
    -- the map's temporary night for `duration`, or day at once (line B E453); a night without
    -- a duration (a clock set to a fixed time of day) has no host form
    if a.mode == "day" then
      if a.duration ~= nil then self:block("action:set_time_of_day day with a duration") end
      return line("cx.set_time_of_day(false, Fixed64::ZERO)?;")
    elseif a.mode == "night" and a.duration ~= nil then
      return line("{") .. line("    let duration = " .. self:value(a.duration, ctx, "num") .. ";")
        .. line("    cx.set_time_of_day(true, duration)?;") .. line("}")
    end
    self:block("action:set_time_of_day " .. tostring(a.mode) .. " without a duration"); return ""
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
  -- vector_length / min_range: the indicator preview only (Unreal OmCastTargeting)
  local allowed = set({ "type", "range", "aoe_radius", "team", "types", "flags", "ability_flags", "notes", "vector_length", "min_range" })
  for _, k in ipairs(sorted_keys(target)) do if not allowed[k] then g:block("target." .. k) end end
  local ty = target.type or "point"
  -- unit_or_point: a point, or a unit (its team and class checked, `point` is its position)
  -- tree: a point the host snaps to a standing tree (targets_tree, E403)
  if ty ~= "unit" and ty ~= "none" and ty ~= "point" and ty ~= "direction" and ty ~= "unit_or_point" and ty ~= "tree" then g:block("target.type:" .. ty) end
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

local ENTRY_EVENTS = set({ "on_cast", "on_toggle_on", "on_toggle_off", "on_learn", "on_upgrade", "on_phase_start",
  "on_channel_tick", "on_channel_end" })
local PASSIVE_TRIGGERS = { on_attack_hit = true, on_ability_executed = true, on_respawn = true, on_attack_windup = true, on_created = true,
  on_take_damage = true, on_deal_damage = true, on_attacked_hit = true, on_attacked = true, periodic = true,
  on_take_damage_pre = true, on_deal_damage_pre = true, on_kill = true, on_attack_launch = true, on_heal_received = true }
-- passive trigger -> status_event mask bit and host kind (the learner's marker listens)
local PASSIVE_EVENTS = { on_take_damage = { 8, 5 }, on_deal_damage = { 16, 6 }, on_attacked_hit = { 4, 4 }, on_attacked = { 4, 4 },
  on_attack_windup = { 256, 10 }, on_kill = { 512, 11 }, on_attack_launch = { 1024, 12 },
  on_take_damage_pre = { 32, 7 }, on_deal_damage_pre = { 64, 8 }, on_heal_received = { 65536, 19 } }

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
        cx.event_instance = event.instance;
        let _ = <hand::Impl as Hook>::on_status_event(&mut cx, status.as_str(), event);
    }
]]
  end
  if has.on_damage_pre then
    impl[#impl + 1] = [[
    fn on_damage_pre(&self, holder: EntityHandle, status: RStr<'_>, event: StatusEvent, world: &mut GameWorldDyn<'_>) -> DamageEdit {
        let mut cx = Cx::passive(world, holder, event.level.max(1));
        cx.caster = event.source;
        cx.event_instance = event.instance;
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
  -- an item's active is its cast program: its events, target and flags; the host settles its
  -- cost from the shop catalog (cooldown, mana, cooldown groups) and uses it without a windup
  if o.kind == "item" and def.active ~= nil then
    local act = def.active
    if act.kind ~= "active" then g:block("item.active.kind:" .. tostring(act.kind)) end
    for _, k in ipairs(sorted_keys(act)) do
      if k ~= "kind" and k ~= "target" and k ~= "cost" and k ~= "cast" and k ~= "events" and k ~= "flags" then g:block("item.active." .. k) end
    end
    for _, k in ipairs(sorted_keys(act.cost or {})) do
      if k ~= "cooldown" and k ~= "mana" and k ~= "cooldown_groups" and k ~= "cooldown_start" then g:block("item.active.cost." .. k) end
    end
    if (act.cost or {}).cooldown_start ~= nil and act.cost.cooldown_start ~= "on_cast" then g:block("item.active.cost.cooldown_start:" .. tostring(act.cost.cooldown_start)) end
    for _, k in ipairs(sorted_keys(act.cast or {})) do
      if k == "cast_point" then g:gap("item cast_point: the host uses items without a windup")
      elseif k ~= "immediate" and k ~= "ignore_backswing" then g:block("item.active.cast." .. k) end
    end
    def = setmetatable({ events = act.events, target = act.target, flags = act.flags }, { __index = o.def })
  end
  local struct = camel(def.id)
  local fns = {}
  -- a cast program reads `target`, `point` and `direction`; a passive trigger reads `event`
  local function program(name, list, event)
    local ctx = { owner = o, scope = scope_child(nil), event = event, cast = name == "on_cast" }
    g.phase = name == "on_phase_start"
    local body = g:actions(list, ctx, "    ")
    g.phase = false
    -- on_toggle_off: other entries of the document turn the toggle off too (toggle_ability)
    fns[#fns + 1] = string.format("%sfn %s(cx: &mut Cx<'_, '_>) -> Res {\n%s    Ok(())\n}\n", name == "on_toggle_off" and "pub(crate) " or "", name, body)
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
      elseif k == "condition" then
        -- a condition the cast must meet (checked before the program, like a target check)
        local cond = g:value(sname, { owner = o, scope = scope_child(nil), event = false, cast = true }, "bool")
        requires_check = requires_check .. string.format("    if !(%s) { return Err(%s); }\n", cond, rust_str("v3: requires condition"))
      elseif k ~= "hero_level" then g:block("requires." .. k) end
    end
  end
  local events = def.events or {}
  for _, ev in ipairs(sorted_keys(events)) do
    if not ENTRY_EVENTS[ev] then g:block("event:" .. ev) end
  end
  if events.on_toggle_on and not events.on_toggle_off or events.on_toggle_off and not events.on_toggle_on then g:block("toggle: needs both on_toggle_on and on_toggle_off") end
  if events.on_cast and events.on_toggle_on then g:block("toggle: cast and toggled") end
  local spec = target_spec(g, def.target, def.flags)
  if events.on_cast then
    program("on_cast", events.on_cast, false)
    if requires_check ~= "" then fns[#fns] = fns[#fns]:gsub("(fn on_cast%(cx: &mut Cx<'_, '_>%) %-> Res {\n)", "%1" .. requires_check:gsub("%%", "%%%%"), 1) end
  elseif requires_check ~= "" then g:block("requires without a cast") end
  -- the cast begins (admitted, before its windup); `cancel_cast` stops it
  if events.on_phase_start then
    if not events.on_cast then g:block("event:on_phase_start without on_cast") end
    program("on_phase_start", events.on_phase_start, false)
  end
  -- a channel (cast.channel_time): the host owns it (omoba-core ability_channel); its start runs
  -- on_cast (on_channel_start), each pulse on_channel_tick, its end on_channel_end
  local channel = def.cast and def.cast.channel_time ~= nil
  if (events.on_channel_tick or events.on_channel_end) and not channel then g:block("event:on_channel_* without cast.channel_time") end
  if channel and not events.on_cast then g:block("channel without on_cast") end
  if events.on_channel_tick then
    local tick = events.on_channel_tick
    if tick[1] ~= nil then tick = { actions = tick } end
    for _, k in ipairs(sorted_keys(tick)) do
      if k ~= "actions" and k ~= "filter" and k ~= "interval" and k ~= "notes" then g:block("event:on_channel_tick." .. k) end
    end
    local list = tick.actions or {}
    if tick.filter ~= nil then list = { { op = "if", cond = tick.filter, actions = list } } end
    program("on_channel_tick", list, "channel_tick")
  end
  if events.on_channel_end then program("on_channel_end", events.on_channel_end, "channel_end") end
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
  -- a permanent status on_learn puts on the learner, with stats: a rank up puts it again at the
  -- new rank (its stacks kept), as the source's modifier reads its level live (D a14t_aspd_stack)
  local rerank = {}
  for _, a in ipairs(events.on_learn or {}) do
    local so = a.op == "apply_status" and a.to == "caster" and m.statuses[a.status]
    local sd = so and so.def
    if sd and sd.stats and (sd.duration == nil or sd.duration == "infinite") and sd.stacking ~= "independent" then
      local again = type(sd.stacking) == "table"
        and { op = "set_stacks", status = a.status, to = "caster", count = "stacks(caster, " .. a.status .. ")" }
        or { op = "apply_status", status = a.status, to = "caster" }
      rerank[#rerank + 1] = { op = "if", cond = "has_status(caster, " .. a.status .. ")", actions = { again } }
    end
  end
  if #rerank > 0 then program("on_learn_rerank", rerank, false) end
  if events.on_upgrade then program("on_upgrade", events.on_upgrade, false) end
  local passive = def.passive or {}
  local owner_triggers = {}
  for _, k in ipairs(sorted_keys(passive)) do
    if k ~= "reveal" and k ~= "triggers" and k ~= "stats" and k ~= "notes" and k ~= "aura" and k ~= "visibility" then g:block("passive:" .. k) end
  end
  -- v3 aura: the host lists the receivers every pulse (on_aura); `auras` are the arms of on_aura
  local auras = {}
  local passive_aura
  if passive.aura then
    if o.kind ~= "ability" then g:block("passive:aura of an item") end
    local ctx = { owner = o, scope = scope_child(nil), event = false }
    passive_aura = "        let _ = " .. g:aura_emit(passive.aura, ctx, "passive", "caster", "None", "true") .. ";\n"
    auras[#auras + 1] = g:aura_arm("passive", passive.aura, o)
  end
  local filters = {}
  local passive_mask, passive_arms, passive_periodic, passive_created = 0, {}, nil, false
  local item_equip, item_mask, item_periodic = {}, 0, nil
  local function doc_id_of(_) return (m.doc_id or "doc"):lower() end
  for _, ev in ipairs(sorted_keys(passive.triggers)) do
    local tr = passive.triggers[ev]
    -- shorthand: a trigger written as its action list
    if tr[1] ~= nil then tr = { actions = tr } end
    -- an item's passive is made when a copy is gained and ends when it is lost (host item_equip:
    -- AbilityScript::on_equip / on_unequip); its status events and periodic clock listen through a
    -- marker made with the first copy and removed with the last. The host's own passive hooks
    -- (on_attack_hit, on_ability_executed, on_respawn…) walk learned abilities only, so those wait
    local item = o.kind == "item"
    if item and ev ~= "on_created" and ev ~= "on_removed" and ev ~= "periodic" and ev ~= "on_attack_hit" and not PASSIVE_EVENTS[ev] then
      g:block("passive_trigger:" .. ev .. " of an item (no learn marker)")
    elseif not PASSIVE_TRIGGERS[ev] and not (item and ev == "on_removed") then g:block("passive_trigger:" .. ev)
    else
      for _, k in ipairs(sorted_keys(tr)) do
        if k ~= "actions" and k ~= "filter" and k ~= "notes" and not (k == "who" and ev == "on_ability_executed")
          and not (ev == "periodic" and (k == "interval" or k == "first")) then g:block("passive_trigger:" .. ev .. "." .. k) end
      end
      if tr.who and tr.who ~= "self" then g:block("passive_trigger:on_ability_executed.who:" .. tr.who) end
      if ev == "periodic" then
        -- every `interval` while learned on this life: the first learn of a life starts it
        -- (a state variable of the life keeps rank-ups from starting a second chain)
        for _, k in ipairs(sorted_keys(tr)) do if k ~= "interval" and k ~= "first" and k ~= "actions" and k ~= "notes" and k ~= "filter" then g:block("passive_trigger:periodic." .. k) end end
        local ctx = { owner = o, scope = scope_child(nil), event = false }
        local step = g:reserve_step()
        local interval = g:value(tr.interval, ctx, "num")
        -- `filter`: this tick's actions run only when it holds (the clock goes on)
        local cond = tr.filter ~= nil and g:value(tr.filter, ctx, "bool") or nil
        local key = rust_str(tostring(doc_id_of(o)) .. ":" .. def.id .. ":periodic")
        -- an item's clock: the marker gone, the chain ends and the next first copy starts one again
        local stop = item and ("    if rank == 0 { let _ = cx.set_state_bool(cx.self_unit, " .. key .. ", false, false); return Ok(()); }\n")
          or "    if rank == 0 { return Ok(()); }\n"
        local body = "    let rank = cx.learned_rank(cx.self_unit, ID);\n" .. stop .. "    cx.rank = rank;\n"
          .. "    {\n        let interval = " .. interval .. ";\n        let owner = cx.self_unit;\n"
          .. string.format("        cx.schedule(owner, ID, %d, interval, args)?;\n    }\n", step)
          .. (cond and ("    if !(" .. cond .. ") { return Ok(()); }\n") or "")
          .. g:actions(tr.actions or {}, ctx, "    ")
        g:set_step(step, "passive periodic", body, true)
        local first = tr.first ~= nil and g:value(tr.first, ctx, "num") or interval
        local start = "        let key = " .. key .. ";\n"
          .. "        if !cx.state_bool(caster, key, false) {\n"
          .. "            let _ = cx.set_state_bool(caster, key, true, false);\n"
          .. "            let first = { " .. first .. " };\n"
          .. string.format("            let _ = cx.schedule(caster, ID, %d, first, &[]);\n", step)
          .. "        }\n"
        if item then item_periodic = start else passive_periodic = start end
      else
        local evt = PASSIVE_EVENTS[ev]
        local pre = ev == "on_take_damage_pre" or ev == "on_deal_damage_pre"
        local ctx = { owner = o, scope = scope_child(nil), event = ev == "on_ability_executed" and "owner_cast" or (ev == "on_heal_received" and "heal_received") or ev ~= "on_created", pre = pre, windup = ev == "on_attack_windup" }
        local filter = tr.filter and g:value(tr.filter, ctx, "bool") or "true"
        filters[ev] = filter
        local body = g:actions(tr.actions, ctx, "    ")
        fns[#fns + 1] = string.format("fn %s(cx: &mut Cx<'_, '_>) -> Res {\n    if !(%s) { return Ok(()); }\n%s    Ok(())\n}\n", ev, filter, body)
        if item and (ev == "on_created" or ev == "on_removed") then
          item_equip[ev] = true
        elseif item and evt then
          item_mask = item_mask + (item_mask & evt[1] == 0 and evt[1] or 0)
          passive_arms[#passive_arms + 1] = { kind = evt[2], fn = ev, pre = pre }
        elseif ev == "on_created" then
          -- a learned passive is a permanent implicit status made at the first learn (v3.md §3)
          passive_created = true
        elseif evt then
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
  local hosted = {}
  for _, sd in ipairs(def.statuses or {}) do hosted[#hosted + 1] = sd end
  for _, sd in ipairs(m.doc_statuses or {}) do
    if m.statuses[sd.id].host == o then hosted[#hosted + 1] = sd end
  end
  for _, sd in ipairs(hosted) do
    local so = m.statuses[sd.id]
    if (triggers_of(sd) or sd.aura or sd.reapply_if_removed == true) and not (sd.runtime_id or ""):match("^attack_orb:") then
      local begin = {}
      if sd.aura then
        -- cx.self_unit holds the status; cx.caster applied it
        local ctx = { owner = so, scope = scope_child(nil), event = false }
        local emit = g:aura_emit(sd.aura, ctx, sd.id, "cx.self_unit", "Some(&super::ids::status::" .. upper(sd.id) .. ")", "false")
        begin[#begin + 1] = "    " .. emit .. "?;\n"
        auras[#auras + 1] = g:aura_arm(sd.id, sd.aura, so)
        -- reapplied while it holds (refresh): its emitter again, at the applier's rank now
        fns[#fns + 1] = string.format("/// `%s` reapplied: its aura again at this rank.\n#[allow(dead_code)]\npub(crate) fn st_%s_reaura(cx: &mut Cx<'_, '_>, holder: EntityHandle) -> Res {\n"
          .. "    let prev = cx.self_unit;\n    cx.self_unit = holder;\n    let r = %s;\n    cx.self_unit = prev;\n    r\n}\n", sd.id, sd.id, emit)
      end
      if sd.reapply_if_removed == true then
        -- removed or dispelled before it ran out (status_event REMOVED; expiry and death are
        -- other kinds): the same status again from the same applier (cx.caster = event.source,
        -- at the rank it was applied) for the duration it had left, through apply_status, so
        -- immunities and immune_to still refuse it; the applier need not be alive
        local ctx = { owner = so, scope = scope_child(nil), event = false, ended = true }
        local fname = "st_" .. sd.id .. "_reapply"
        local again = g:actions({ { op = "apply_status", status = sd.id, to = "self", duration = "status.remaining" } }, ctx, "        ")
        fns[#fns + 1] = string.format("#[allow(dead_code)]\npub(crate) fn %s(cx: &mut Cx<'_, '_>) -> Res {\n"
          .. "    if cx.event_amount > Fixed64::ZERO && cx.alive(cx.self_unit) {\n%s    }\n    Ok(())\n}\n", fname, again)
        status_arms[#status_arms + 1] = { status = sd.id, kind = STATUS_TRIGGERS.on_removed[2][2], fn = fname }
      end
      local sd_triggers = triggers_of(sd) or {}
      -- every periodic of the row runs on one clock token: a second start_chain would make the
      -- first periodic's chain look stale (B28E's damage and guard, E335)
      local chained = false
      for _, name in ipairs(sorted_keys(sd_triggers)) do
        local t = trigger_kind(name)
        if not t then g:block("status_trigger:" .. name)
        else
          local tr = sd_triggers[name]
          if name:match("^periodic") then
            for _, k in ipairs(sorted_keys(tr)) do if k ~= "interval" and k ~= "first" and k ~= "actions" and k ~= "notes" and k ~= "filter" then g:block("status_trigger:periodic." .. k) end end
            local ctx = { owner = so, scope = scope_child(nil), event = false }
            local step = g:reserve_step()
            local interval = g:value(tr.interval, ctx, "num")
            -- `filter`: this tick's actions run only when it holds (the clock goes on)
            local cond = tr.filter ~= nil and g:value(tr.filter, ctx, "bool") or nil
            local inst = sd.stacking == "independent"
            -- the applier's current rank of this entry: a think reads the ability's level when it
            -- ticks (the source's %value), not the rank it was applied at (D: B31R's burn, E451)
            local body = "    cx.caster = ability_kit::arg_as_unit(args, 0);\n    cx.rank = cx.rank_of(cx.caster, ID);\n"
              .. (inst and ("    cx.event_instance = u32::try_from(ability_kit::arg_as_num(args, 1).raw() >> 10).unwrap_or(0);\n"
                .. "    if !cx.has_status_instance(cx.self_unit, &super::ids::status::" .. upper(sd.id) .. ", cx.event_instance) { return Ok(()); }\n")
                or ("    if !cx.has_status(cx.self_unit, &super::ids::status::" .. upper(sd.id) .. ") { return Ok(()); }\n"
                  -- one clock per holder and row (E335)
                  .. "    if !cx.chain_current(cx.self_unit, &super::ids::status::" .. upper(sd.id) .. ", ability_kit::arg_as_num(args, 1)) { return Ok(()); }\n"))
              .. "    {\n        let interval = " .. interval .. ";\n        let owner = cx.self_unit;\n"
              .. string.format("        cx.schedule(owner, ID, %d, interval, args)?;\n    }\n", step)
              .. (cond and ("    if !(" .. cond .. ") { return Ok(()); }\n") or "")
              .. g:actions(tr.actions or {}, ctx, "    ")
            g:set_step(step, "`" .. sd.id .. "` periodic", body, true)
            local first = tr.first ~= nil and g:value(tr.first, ctx, "num") or interval
            local args = inst and "[ability_kit::arg_unit(cx.caster), ability_kit::arg_num(Fixed64::from_i32(i32::try_from(cx.event_instance).unwrap_or(0)))]"
              or "[ability_kit::arg_unit(cx.caster), ability_kit::arg_num(chain_token)]"
            if not inst and not chained then
              chained = true
              begin[#begin + 1] = "    let chain_token = { let holder = cx.self_unit; cx.start_chain(holder, &super::ids::status::" .. upper(sd.id) .. ")? };\n"
            end
            begin[#begin + 1] = "    {\n        let first = " .. first .. ";\n"
              .. "        let args = " .. args .. ";\n"
              .. "        let owner = cx.self_unit;\n" .. string.format("        cx.schedule(owner, ID, %d, first, &args)?;\n    }\n", step)
          else
            local filter, actions = trigger_parts(tr)
            -- on_ability_executed of the holder only; its event is the skill (event.ability, line B
            -- 94d609ce), nothing else
            local executed = name == "on_ability_executed"
            if executed and tr.who ~= nil and tr.who ~= "self" then g:block("status_trigger:on_ability_executed.who:" .. tostring(tr.who)) end
            local evented = t[2] ~= nil and name ~= "on_removed" and name ~= "on_expired" and name ~= "on_death" and not executed
            local pre = name == "on_take_damage_pre" or name == "on_deal_damage_pre"
            local ended = name == "on_removed" or name == "on_expired" or name == "on_death"
            local ctx = { owner = so, scope = scope_child(nil), event = executed and "ability_executed" or (name == "on_moved" and "moved") or (name == "on_mana_spent" and "mana_spent") or (name == "on_heal_received" and "heal_received") or evented, pre = pre, ended = ended,
              windup = name == "on_attack_windup" }
            -- an absorb event names this hit's absorbed share `absorbed` (v3.md §4)
            if name == "on_absorb" or name == "on_broken" then
              ctx.scope.names.absorbed = { name = "absorbed", ty = "num", marker = "cx.event_amount", used = false }
            end
            local cond = filter and g:value(filter, ctx, "bool") or nil
            local fname = "st_" .. sd.id .. "_" .. name
            local guard = name == "on_death" and "    if cx.alive(cx.self_unit) { return Ok(()); }\n" or ""
            fns[#fns + 1] = string.format("#[allow(dead_code)]\npub(crate) fn %s(cx: &mut Cx<'_, '_>) -> Res {\n%s%s%s    Ok(())\n}\n", fname, guard,
              cond and ("    if !(" .. cond .. ") { return Ok(()); }\n") or "", g:actions(actions, ctx, "    "))
            if name == "on_created" then begin[#begin + 1] = "    " .. fname .. "(cx)?;\n"
            else
              for _, kind in ipairs(t[2] or {}) do
                status_arms[#status_arms + 1] = { status = sd.id, kind = kind, fn = fname }
              end
            end
          end
        end
      end
      if #begin > 0 then
        fns[#fns + 1] = string.format("/// `%s` starts on a unit: on_created, then the periodic schedule.\n#[allow(dead_code)]\npub(crate) fn st_%s_start(cx: &mut Cx<'_, '_>, holder: EntityHandle) -> Res {\n"
          -- the applier's current rank, as its periodic and events read it (E451)
          .. "    let prev = cx.self_unit;\n    cx.self_unit = holder;\n    let prev_rank = cx.rank;\n    cx.rank = cx.rank_of(cx.caster, ID);\n"
          .. "    let r = st_%s_begin(cx);\n    cx.rank = prev_rank;\n    cx.self_unit = prev;\n    r\n}\n\n#[allow(dead_code)]\nfn st_%s_begin(cx: &mut Cx<'_, '_>) -> Res {\n%s    Ok(())\n}\n",
          sd.id, sd.id, sd.id, sd.id, table.concat(begin))
      end
    end
  end

  -- a chase unit this entry creates reached its target (status_event CHASE_REACH 15, line B E406):
  -- `self` is the unit, `target` what it chased, the caster its owner at this entry's rank
  for _, tid in ipairs(sorted_keys(g.chase_reach or {})) do
    local to = g.chase_reach[tid]
    local fname = "ch_" .. tid .. "_reach"
    local body = g:actions(to.def.chase.on_reach, { owner = to, scope = scope_child(nil), event = false }, "    ")
    fns[#fns + 1] = string.format("pub(crate) fn %s(cx: &mut Cx<'_, '_>) -> Res {\n%s    Ok(())\n}\n", fname, body)
    status_arms[#status_arms + 1] = { status = tid, kind = 15, fn = fname }
  end
  -- the auras of the dummy templates this entry creates (create_unit): one arm each
  for _, tid in ipairs(sorted_keys(g.template_auras or {})) do
    local to = g.template_auras[tid]
    auras[#auras + 1] = g:aura_arm(tid, to.def.passive.aura, to)
  end
  -- the AbilityScript
  for _, a in ipairs(auras) do fns[#fns + 1] = a.fn end
  local blockers = sorted_keys(g.blockers)
  if #blockers > 0 then return stub_text(def, blockers), blockers, {}, g end
  for _, st in ipairs(g.steps or {}) do fns[#fns + 1] = st end
  for _, lf in ipairs(g.local_fns or {}) do fns[#fns + 1] = lf end
  local s = { entry_head(def) }
  local run = events.on_cast or events.on_toggle_on
  local learns = passive_stats or #owner_triggers > 0 or events.on_learn or events.on_upgrade or passive_mask ~= 0 or passive_periodic or passive_created or passive_aura
  if run or learns or #auras > 0 then s[#s + 1] = string.format("const MAX_LEVEL: u8 = %d;\n", def.max_level or 1) end
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
  if channel then
    impl[#impl + 1] = [=[
    fn on_channel_start(&self, caster: EntityHandle, target: Target, start: ChannelPulse, world: &mut GameWorldDyn<'_>) {
        if start.level == 0 || start.level > MAX_LEVEL { return; }
        let mut cx = Cx::new(world, caster, target, start.level);
        let _ = on_cast(&mut cx);
    }
]=]
    if events.on_channel_tick then
      impl[#impl + 1] = [=[
    fn on_channel_pulse(&self, caster: EntityHandle, target: Target, pulse: ChannelPulse, world: &mut GameWorldDyn<'_>) {
        if pulse.level == 0 || pulse.level > MAX_LEVEL { return; }
        let mut cx = Cx::new(world, caster, target, pulse.level);
        cx.event_amount = Fixed64::from_i32(i32::from(pulse.index));
        let _ = on_channel_tick(&mut cx);
    }
]=]
    end
    if events.on_channel_end then
      impl[#impl + 1] = [=[
    fn on_channel_end(&self, caster: EntityHandle, target: Target, end: ChannelPulse, interrupted: bool, world: &mut GameWorldDyn<'_>) {
        if end.level == 0 || end.level > MAX_LEVEL { return; }
        let mut cx = Cx::new(world, caster, target, end.level);
        cx.event_amount = if interrupted { Fixed64::ONE } else { Fixed64::ZERO };
        let _ = on_channel_end(&mut cx);
    }
]=]
    end
  end
  if events.on_phase_start then
    impl[#impl + 1] = [=[
    fn on_phase_start(&self, caster: EntityHandle, target: Target, level: u8, world: &mut GameWorldDyn<'_>) -> bool {
        if level == 0 || level > MAX_LEVEL { return true; }
        let mut cx = Cx::new(world, caster, target, level);
        let _ = on_phase_start(&mut cx);
        !cx.cancelled
    }
]=]
  end
  local learn = {}
  if passive_mask ~= 0 or passive_periodic then
    learn[#learn + 1] = string.format("        let _ = cx.passive_marker(ID, %s, %d);\n", passive_stats or "&[]", passive_mask)
  elseif passive_stats then learn[#learn + 1] = "        let _ = cx.passive_stats(ID, " .. passive_stats .. ");\n"
  elseif #owner_triggers > 0 then learn[#learn + 1] = "        let _ = cx.learn_marker(ID);\n" end
  if passive_periodic then learn[#learn + 1] = passive_periodic end
  if passive_aura then learn[#learn + 1] = passive_aura end
  if events.on_learn then learn[#learn + 1] = "        if new_level == 1 { let _ = on_learn_first(&mut cx); }\n" end
  if #rerank > 0 then learn[#learn + 1] = "        if new_level > 1 { let _ = on_learn_rerank(&mut cx); }\n" end
  -- the passive's implicit status is made: its on_created, once
  if passive_created then learn[#learn + 1] = "        if new_level == 1 { let _ = on_created(&mut cx); }\n" end
  -- on_upgrade: every rank, the first included (v3.md §12.4)
  if events.on_upgrade then learn[#learn + 1] = "        let _ = on_upgrade(&mut cx);\n" end
  if #learn > 0 then
    impl[#impl + 1] = "\n    fn on_learn(&self, caster: EntityHandle, new_level: u8, world: &mut GameWorldDyn<'_>) {\n"
      .. "        if new_level == 0 || new_level > MAX_LEVEL { return; }\n"
      .. "        let mut cx = Cx::passive(world, caster, new_level);\n" .. g:finish(table.concat(learn)) .. "    }\n"
  end
  -- an item's passive: on_created per copy gained, on_removed per copy lost (its stats are the
  -- shop catalog's, which the host applies)
  local marker = item_mask ~= 0 or item_periodic ~= nil
  local copies = rust_str("__item_copies:" .. def.id)
  if item_equip.on_created or marker then
    local first = marker and ("        let caster = owner;\n"
      .. "        let n = cx.state_num(caster, " .. copies .. ", Fixed64::ZERO) + Fixed64::ONE;\n"
      .. "        let _ = cx.set_state_num(caster, " .. copies .. ", n, false);\n"
      .. "        if n == Fixed64::ONE {\n"
      .. string.format("            let _ = cx.passive_marker(ID, &[], %d);\n", item_mask)
      .. (item_periodic and item_periodic:gsub("\n(.)", "\n    %1"):gsub("^", "    ") or "")
      .. "        }\n") or ""
    impl[#impl + 1] = "\n    fn on_equip(&self, owner: EntityHandle, world: &mut GameWorldDyn<'_>) {\n"
      .. "        let mut cx = Cx::passive(world, owner, 1);\n" .. g:finish(first)
      .. (item_equip.on_created and "        let _ = on_created(&mut cx);\n" or "") .. "    }\n"
  end
  if item_equip.on_removed or marker then
    local last = marker and ("        let caster = owner;\n"
      .. "        let n = cx.state_num(caster, " .. copies .. ", Fixed64::ZERO) - Fixed64::ONE;\n"
      .. "        let _ = cx.set_state_num(caster, " .. copies .. ", n.max(Fixed64::ZERO), false);\n"
      .. "        if n <= Fixed64::ZERO { cx.remove_passive_marker(ID); }\n") or ""
    impl[#impl + 1] = "\n    fn on_unequip(&self, owner: EntityHandle, world: &mut GameWorldDyn<'_>) {\n"
      .. "        let mut cx = Cx::passive(world, owner, 1);\n" .. last
      .. (item_equip.on_removed and "        let _ = on_removed(&mut cx);\n" or "") .. "    }\n"
  end
  if #auras > 0 then
    local arms = {}
    for _, a in ipairs(auras) do arms[#arms + 1] = string.format("            %s => aura_%s(&mut cx, &receivers),\n", rust_str(a.key), a.key) end
    impl[#impl + 1] = "\n    fn on_aura(&self, holder: EntityHandle, aura: RStr<'_>, level: u8, receivers: RVec<EntityHandle>, world: &mut GameWorldDyn<'_>) {\n"
      .. "        if level == 0 || level > MAX_LEVEL { return; }\n"
      .. "        let Some((name, source)) = ability_kit::aura_source(aura.as_str()) else { return };\n"
      .. "        let mut cx = Cx::passive(world, holder, level);\n"
      .. "        cx.caster = source;\n"
      .. "        let _ = match name {\n" .. table.concat(arms) .. "            _ => Ok(()),\n        };\n    }\n"
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
      -- an end: the duration the status had left
      [0] = "cx.event_amount = event.amount;", [1] = "cx.event_amount = event.amount;", [2] = "cx.event_amount = event.amount;",
      [3] = "cx.event_target = event.other; cx.target = Target::Entity(event.other);",
      [4] = "cx.event_attacker = event.other;",
      [5] = "cx.event_attacker = event.other; cx.event_amount = event.amount; cx.event_damage_kind = event.damage_kind;",
      [6] = "cx.event_target = event.other; cx.event_amount = event.amount; cx.event_damage_kind = event.damage_kind;",
      [7] = "cx.event_attacker = event.other; cx.event_amount = event.amount; cx.event_damage_kind = event.damage_kind;",
      [8] = "cx.event_target = event.other; cx.event_amount = event.amount; cx.event_damage_kind = event.damage_kind;",
      [10] = "cx.event_target = event.other; cx.target = Target::Entity(event.other);",
      [12] = "cx.event_target = event.other; cx.target = Target::Entity(event.other);",
      [11] = "cx.event_target = event.other; cx.event_amount = event.amount; cx.event_damage_kind = event.damage_kind;",
      [13] = "cx.event_attacker = event.other; cx.event_amount = event.amount;",
      [14] = "cx.event_attacker = event.other; cx.event_amount = event.amount;",
      [16] = "cx.event_amount = event.amount;",
      [17] = "cx.event_amount = event.amount;",
      [19] = "cx.event_amount = event.amount;",
      [15] = "cx.event_target = event.other; cx.target = Target::Entity(event.other);",
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
        -- the applier's current rank, as a periodic tick reads it (E451)
        .. "        cx.rank = cx.rank_of(cx.caster, ID);\n"
        .. "        cx.event_instance = event.instance;\n"
        .. "        match (status.as_str(), event.kind) {\n" .. table.concat(arms) .. "            _ => {}\n        }\n    }\n"
    end
    if #pre_arms > 0 then
      impl[#impl + 1] = "\n    fn on_damage_pre(&self, holder: EntityHandle, status: RStr<'_>, event: StatusEvent, world: &mut GameWorldDyn<'_>) -> DamageEdit {\n"
        .. "        let mut cx = Cx::passive(world, holder, event.level.max(1));\n"
        .. "        cx.caster = event.source;\n"
        -- the applier's current rank, as a periodic tick reads it (E451)
        .. "        cx.rank = cx.rank_of(cx.caster, ID);\n"
        .. "        cx.event_instance = event.instance;\n"
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

/// Learned: the learn marker (or the passive stats buff) is on the owner. Its triggers run at the
/// owner's learned rank of this ability (cx.rank_of).
fn learned(world: &GameWorldDyn<'_>, owner: EntityHandle) -> bool { world.has_buff(owner, RStr::from_str(ID)) }
]]
    for _, ev in ipairs(owner_triggers) do
      if ev == "on_ability_executed" then
        s[#s + 1] = [[

pub fn on_owner_cast(owner: EntityHandle, skill: &str, target: Target, world: &mut GameWorldDyn<'_>) {
    if !learned(world, owner) { return; }
    let mut cx = Cx::passive(world, owner, 1);
    cx.rank = cx.rank_of(owner, ID);
    cx.event_skill = Some(skill);
    // event.target / event.point: the cast's own target
    if let Target::Entity(e) = &target { cx.event_target = *e; }
    cx.target = target;
    let _ = on_ability_executed(&mut cx);
}
]]
      elseif ev == "on_respawn" then
        s[#s + 1] = [[

pub fn on_owner_respawn(owner: EntityHandle, world: &mut GameWorldDyn<'_>) {
    if !learned(world, owner) { return; }
    let mut cx = Cx::passive(world, owner, 1);
    cx.rank = cx.rank_of(owner, ID);
    let _ = on_respawn(&mut cx);
}
]]
      end
    end
  end
  if o.kind == "item" then
    -- a shop item: its AbilityDef is the catalog's (kind = item); passive without an active
    -- None for an item the shop catalog does not have (it does not exist in the source)
    s[#s + 1] = "\npub fn ffi() -> Option<AbilityDefFFI> { ability_kit::item_ffi(ID, " .. tostring(def.active == nil) .. ", " .. struct .. ") }\n"
  else
    s[#s + 1] = "\npub fn ffi() -> AbilityDefFFI { ability_kit::ability_ffi(ID, " .. struct .. ") }\n"
      .. "#[allow(dead_code)]\npub fn ffi_opt() -> Option<AbilityDefFFI> { ability_kit::ability_ffi_opt(ID, " .. struct .. ") }\n"
  end
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
      opts.routine_modules = opts.routine_modules or {}
      opts.routine_modules[name] = g
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
  local gopts = { rules = opts.rules, vocab = M.vocab(root), cues_used = {}, root = abs_root, package = opts.package,
    summons = M.content_summons(root), summon_problems = M.summon_problems(root) }
  local files = {}
  local crate = M.crate_name(doc)
  local gen = "src/generated/"
  files[gen .. "values.rs"] = emit_values(m)
  local status_text, status_problems = emit_statuses(m, root)
  local report = { crate = crate, id = doc.id, entries = {} }
  gopts.doc_hooks = doc.hooks or {}
  local mods, ffis, owner_calls = {}, {}, { on_ability_executed = {}, on_respawn = {} }
  local item_ffis = {}
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
  -- An entry that starts another entry's status needs that entry's module: when that entry is
  -- not generated, neither is this one (fixed point over the entries and the routines they call)
  do
    local by_id = {}
    for _, e in ipairs(entry_gens) do by_id[e.o.def.id] = e end
    local function needs(e)
      local out = {}
      for id, st in pairs(e.g.modules_needed or {}) do out[id] = st end
      for n in pairs(e.g.routines_needed) do
        local rg = (opts.routine_modules or {})[n]
        for id, st in pairs(rg and rg.modules_needed or {}) do out[id] = st end
      end
      return out
    end
    local function blocked(id) local x = by_id[id]; return not x or #x.blockers > 0 or #routine_closure(x.g) > 0 end
    local changed = true
    while changed do
      changed = false
      for _, e in ipairs(entry_gens) do
        if #e.blockers == 0 then
          for id, st in pairs(needs(e)) do
            if blocked(id) then
              e.blockers = { "status " .. st .. " of " .. id .. ", which is not generated" }
              e.text = stub_text(e.o.def, e.blockers); e.facts = {}
              changed = true
              break
            end
          end
        end
      end
    end
  end
  for _, e in ipairs(entry_gens) do
    -- items.dll registers only what the content has: anything else is a blocker, never a load-time skip
    if opts.catalog and #e.blockers == 0 then
      local id = e.o.def.id
      if e.o.kind == "item" and not opts.catalog.items[id] then
        e.blockers = { "catalog: item " .. id .. " is not in the shop (templates/moba_items.lua)" }
        e.text = stub_text(e.o.def, e.blockers); e.facts = {}
      elseif e.o.kind == "ability" and not opts.catalog.abilities[id] then
        e.blockers = { "catalog: ability " .. id .. " is not a content ability (templates/abilities.lua)" }
        e.text = stub_text(e.o.def, e.blockers); e.facts = {}
      end
    end
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
      if e.o.kind == "ability" or e.o.kind == "item" then ffis[#ffis + 1] = id; if e.o.kind == "item" then item_ffis[id] = true end end
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
  -- an item ffi is an Option (None off the shop catalog); a document with any item flattens them
  local any_item = next(item_ffis) ~= nil
  -- every entry under catch_unwind (ability_kit::guard_ffi): no panic crosses the DLL boundary
  for _, id in ipairs(ffis) do
    local call
    if item_ffis[id] then call = "super::" .. id .. "::ffi()"
    elseif any_item then call = "super::" .. id .. "::ffi_opt()"
    else call = "Some(super::" .. id .. "::ffi())" end
    pushes[#pushes + 1] = "ability_kit::guard_ffi(|| " .. call .. ")"
  end
  any_item = true -- the guarded entries are Options: always flattened
  -- E261: building the manifest registers every entry (guard_ffi would hide a panic at load)
  rg[#rg + 1] = string.format("\n#[cfg(test)]\nmod manifest_loads {\n    #[test]\n    fn every_entry_registers() {\n"
    .. "        assert_eq!(super::abilities().len(), %d, \"an entry failed to register (see stderr)\");\n        let _ = super::units();\n    }\n}\n", #pushes)
  if any_item then
    -- the items the shop catalog has (item_ffi is None for the others)
    rg[#rg + 1] = "/// Every item of the document the shop catalog has.\npub fn abilities() -> RVec<AbilityDefFFI> {\n"
      .. (#pushes == 0 and "    RVec::new()\n" or ("    [\n        " .. table.concat(pushes, ",\n        ") .. ",\n    ].into_iter().flatten().collect()\n")) .. "}\n"
  else
    rg[#rg + 1] = "/// Every ability of the document.\npub fn abilities() -> RVec<AbilityDefFFI> {\n"
      .. (#pushes == 0 and "    RVec::new()\n" or ("    RVec::from(vec![\n        " .. table.concat(pushes, ",\n        ") .. ",\n    ])\n")) .. "}\n"
  end
  local has_owner = #owner_calls.on_ability_executed + #owner_calls.on_respawn > 0
  if has_owner then
    local casts, respawns = {}, {}
    for _, id in ipairs(owner_calls.on_ability_executed) do casts[#casts + 1] = "        super::" .. id .. "::on_owner_cast(caster, skill_id.as_str(), target.clone(), world);\n" end
    for _, id in ipairs(owner_calls.on_respawn) do respawns[#respawns + 1] = "        super::" .. id .. "::on_owner_respawn(e, world);\n" end
    rg[#rg + 1] = string.format([[

/// The hero UnitScript: passives that trigger on the owner's casts and respawns.
pub struct HeroUnit;

impl UnitScript for HeroUnit {
    fn unit_id(&self) -> RStr<'_> { RStr::from_str(%s) }
    fn on_skill_cast(&self, caster: EntityHandle, skill_id: RStr<'_>, target: Target, world: &mut GameWorldDyn<'_>) {
        let _ = (&caster, &skill_id, &target, &world);
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
-- The content catalog items.dll registers against (scripts/lua_data): ability ids and the v3
-- item ids the shop cites (`source_ref` `…:<v3 id>`). Cached per root.
local catalog_cache = {}
-- The summon ids the host can spawn (templates/summons.lua with the authored packages' summons),
-- per root.
local summons_cache = {}
function M.content_summons(root)
  if summons_cache[root] then return summons_cache[root] end
  local saved = package.path
  package.path = root .. "/scripts/?.lua;" .. package.path
  local context = require("content_builder").new(root .. "/scripts/lua_data")
  package.path = saved
  local ids = {}
  for _, row in ipairs(context.include("templates/summons.lua")) do if not row.tombstone then ids[row.id] = true end end
  summons_cache[root] = ids
  return ids
end

-- template id -> why scripts/v3_summons.lua gives it no summon row yet, per root.
function M.summon_problems(root)
  local saved = package.path
  package.path = root .. "/scripts/?.lua;" .. root .. "/tools/lua/?.lua;" .. package.path
  -- its document loader needs scripts/ on the path while it runs
  local ok, problems = pcall(function() return require("v3_summons").problems(root) end)
  package.path = saved
  return assert(ok and problems, "scripts/v3_summons.lua: " .. tostring(problems))
end

function M.content_catalog(root)
  if catalog_cache[root] then return catalog_cache[root] end
  local saved = package.path
  package.path = root .. "/scripts/?.lua;" .. package.path
  local context = require("content_builder").new(root .. "/scripts/lua_data")
  package.path = saved
  local abilities, items = {}, {}
  for _, a in ipairs(context.include("templates/abilities.lua")) do if not a.tombstone then abilities[a.id] = true end end
  local shop = context.include("templates/moba_items.lua")
  for _, it in ipairs(shop.items or shop) do
    local source = type(it) == "table" and it.source_ref and it.source_ref:match(":([%w_]+)$")
    if source then items[source] = true end
  end
  catalog_cache[root] = { abilities = abilities, items = items }
  return catalog_cache[root]
end

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
      local files, report = M.generate(doc, { root = root, rules = rules, package = "items", catalog = M.content_catalog(root) })
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
