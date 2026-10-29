-- What the behavior IR interpreter does not run yet, and in which order to add it (history-nobu full
-- port, work line A for work line B): every IR block of the full v3 content (scripts/gen_v3_content.lua)
-- is scanned for the forms omoba-core/src/runtime/native/ir.rs refuses at load; the blockers are ranked
-- by the heroes they hold back, in the M5 order (the shipped heroes still on Rust handlers) and then the
-- M6 waves (m6-waves.md).
--
--   tools\lua\lua.exe docs\character_pipeline\tools\v3_ir_blockers.lua [--content FILE] [--probe FILE] [--out FILE.md]
--
-- --probe: the JSON lines of tools/ir_probe (the interpreter's own verdict per block). With it the report
-- cross-checks this scan: a block the scan clears must load, a block that fails must have a blocker here;
-- any disagreement means ir.rs moved on and the lists below need the same change.
-- Mirrors ir.rs (2026-10-29, m3e E226: selector flags; m3d E224; work line A's geometry / target types / on_learn / on_respawn). Update the lists when a batch lands.
local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
local M = {}

local set = function(t) local s = {} for _, k in ipairs(t) do s[k] = true end return s end
M.ACTIONS = {          -- op -> allowed fields (`only` in Program::load)
  apply_status = set({ "op", "status", "to", "duration" }),
  damage = set({ "op", "to", "type", "amount", "require_source_alive", "flags" }),
  destroy_trees = false, -- any fields: the host has no trees yet (Nop)
  create_unit = set({ "op", "template", "at", "count", "lifetime", "bind", "control", "limit_per_owner", "over_limit" }),
  cue = false,         -- any fields
  ["if"] = set({ "op", "cond", "actions", "otherwise" }),
  let = set({ "op", "bind", "value" }),
  for_each = set({ "op", "select", "actions", "actions_bind" }),
  tether = set({ "op", "from", "to", "duration", "on_expire", "cue" }),
  reveal = set({ "op", "at", "follow", "radius", "duration", "teams" }),
  call = set({ "op", "routine", "args" }),
  heal = set({ "op", "to", "amount", "source" }),       -- the caster is the only healer
  remove_status = set({ "op", "status", "from" }),
}
M.DAMAGE_TYPES = set({ "physical", "magical", "pure" })
M.DAMAGE_FLAGS = set({ "no_amplify" })
-- selector flags (ir.rs for_each): magic immune and invulnerable units are out unless included
M.SELECTOR_FLAGS = set({ "include_magic_immune", "include_invulnerable", "include_untargetable" })
-- selector `types` the host query admits (ir.rs unit_types), as the sorted names joined by "+"
M.SELECTOR_TYPES = set({ "", "all", "hero", "creep+hero+neutral+summon" })
local function types_key(t)
  local names = {}
  for _, n in ipairs(type(t) == "table" and t or {}) do names[#names + 1] = n end
  table.sort(names)
  return table.concat(names, "+")
end
M.TOKENS = set({ "slot", "local", "raw", "bool", "sym", "name", "attr", "call", "op", "jump_if_false", "jump" })
M.NAMES = set({ "caster", "target", "point", "self", "event", "direction" })
M.ATTRS = set({ "attack", "is_building", "is_hero", "hp", "max_hp", "max_mana", "target", "attacker", "ability" })
M.FUNCS = { alive = 1, min = 2, max = 2, is_enemy = 2, edge_distance = 2, distance = 2, has_tag = 2, is_item = 2,
  direction = 2, offset = 3, position = 1 }
M.OPS = set({ "+", "-", "*", "/", "==", "~=", "<", "<=", ">", ">=", "and", "or", "neg", "not" })
M.SELECTOR_FIELDS = set({ "from", "radius", "shape", "team", "types", "flags", "where", "it" })
M.STATUS_FIELDS = set({ "id", "runtime_id", "duration", "stacking", "dispel", "visibility", "reveal", "cue", "remove_on", "stats", "per_source",
  "states", "tags", "triggers" })
M.ORB_TRIGGERS = set({ "on_attack_hit", "on_attack_windup" }) -- status triggers: only a host attack orb (`attack_orb:<ability>`)
M.EVENTS = set({ "on_cast", "on_toggle_on", "on_toggle_off", "on_learn" })
M.TARGET_FIELDS = set({ "type", "range", "aoe_radius", "team", "types", "flags", "ability_flags" })
M.TARGET_TYPES = set({ "unit", "point", "none", "direction" })
-- target `types` (ir.rs target_spec), sorted and joined by "+"
M.TARGET_UNIT_TYPES = set({ "", "all", "hero", "creep+hero", "creep+hero+neutral+summon", "building+creep+hero", "building+creep+hero+neutral+summon" })
M.TARGET_TEAMS = set({ "enemy", "ally", "both" })
M.TARGET_FLAGS = set({ "no_target_self", "self_cast_allowed", "can_target_buildings" })
M.PASSIVE_FIELDS = set({ "reveal", "triggers", "stats" })
M.PASSIVE_TRIGGERS = { on_attack_hit = set({ "program", "filter" }), on_ability_executed = set({ "program", "filter", "who" }),
  on_respawn = set({ "program", "filter" }) }
-- StatKeys the rules layer maps: read from omoba-core rules/vocab.rs (`STATS`), so it follows the code
function M.load_stats(root)
  local f = io.open(root .. "/omoba-core/src/runtime/native/rules/vocab.rs", "rb")
  if not f then return nil end
  local text = f:read("a"); f:close()
  local body = text:match("pub const STATS[^=]*=%s*&%[(.-)%];")
  if not body then return nil end
  local s = {}
  for name in body:gmatch('%(%s*"([%w_]+)"%s*,%s*StatKey::') do s[name] = true end
  return next(s) and s or nil
end

-- `set_absolute` stats and state flags the host runs (rules/vocab.rs ABSOLUTE_STATS / STATE_EFFECTS)
function M.load_vocab_list(root, const, value_type)
  local f = io.open(root .. "/omoba-core/src/runtime/native/rules/vocab.rs", "rb")
  if not f then return nil end
  local text = f:read("a"); f:close()
  local body = text:match("const " .. const .. "[^=]*=%s*&%[(.-)%];")
  if not body then return nil end
  local s = {}
  for name in body:gmatch('%(%s*"([%w_]+)"%s*,%s*' .. value_type) do s[name] = true end
  return s
end

local function keys(t) local o = {} for k in pairs(t or {}) do o[#o + 1] = k end table.sort(o, function(a, b) return tostring(a) < tostring(b) end) return o end

-- blockers of one IR block: { [feature] = true }; templates: registered summon ids
function M.scan(ir, templates)
  local b = {}
  local function add(f) b[f] = true end
  local exprs, programs, selectors = ir.exprs or {}, ir.programs or {}, ir.selectors or {}
  local used_exprs = {}
  local function expr(i) if type(i) == "number" then used_exprs[i] = true end end
  -- reachable programs
  local reach, pending = {}, {}
  for _, ev in ipairs(keys(ir.entries)) do
    local e = ir.entries[ev]
    local p = e.program
    if M.EVENTS[ev] then pending[#pending + 1] = p
    elseif ev == "on_phase_start" then
      local only_cues = true
      for _, instr in ipairs(programs[p] or {}) do if instr.op ~= "cue" and instr.op ~= "stop" then only_cues = false end end
      if not only_cues then add("event:on_phase_start") end
    else add("event:" .. ev) end
  end
  local entries = ir.entries or {}
  if (entries.on_toggle_on ~= nil) ~= (entries.on_toggle_off ~= nil) then add("toggle: needs both on_toggle_on and on_toggle_off") end
  if entries.on_cast and entries.on_toggle_on then add("toggle: cast and toggled") end
  local stat_rows -- defined below; passive.stats goes through the same rows as status stats
  if ir.passive then
    for _, k in ipairs(keys(ir.passive)) do if not M.PASSIVE_FIELDS[k] then add("passive:" .. k) end end
    for _, ev in ipairs(keys(ir.passive.triggers)) do
      local tr = ir.passive.triggers[ev]
      local allowed = M.PASSIVE_TRIGGERS[ev]
      if not allowed then add("passive_trigger:" .. ev)
      else
        for _, k in ipairs(keys(tr)) do if not allowed[k] then add("passive_trigger:" .. ev .. "." .. k) end end
        if tr.who and tr.who ~= "self" then add("passive_trigger:on_ability_executed.who") end
        expr(tr.filter)
        pending[#pending + 1] = tr.program
      end
    end
  end
  for _, name in ipairs(keys(ir.routines)) do pending[#pending + 1] = ir.routines[name].program end
  stat_rows = function(rows, what)
    for _, row in ipairs(rows or {}) do
      for _, k in ipairs(keys(row)) do if k ~= "stat" and k ~= "expr" and k ~= "op" then add(what .. ".stat." .. k) end end
      if row.op == nil or row.op == "add" then
        if M.STATS and not M.STATS[row.stat] then add("stat:" .. tostring(row.stat)) end
      elseif row.op == "set_absolute" then
        if M.ABSOLUTE_STATS and not M.ABSOLUTE_STATS[row.stat] then add("stat:" .. tostring(row.stat)) end
      else add("stat.op:" .. tostring(row.op)) end
      expr(row.expr)
    end
  end
  if ir.passive then stat_rows(ir.passive.stats, "passive") end
  local used_statuses = {}
  while #pending > 0 do
    local p = table.remove(pending)
    if p and not reach[p] then
      reach[p] = true
      for _, instr in ipairs(programs[p] or {}) do
        local op = instr.op
        local allowed = M.ACTIONS[op]
        if op == "stop" then
        elseif allowed == nil then add("action:" .. tostring(op))
        else
          if allowed then for _, k in ipairs(keys(instr)) do if not allowed[k] then add("action:" .. op .. "." .. k) end end end
          if op == "apply_status" then used_statuses[instr.status] = true; expr(instr.to); expr(instr.duration)
          elseif op == "damage" then
            if not M.DAMAGE_TYPES[instr.type] then add("damage.type:" .. tostring(instr.type)) end
            for _, fl in ipairs(instr.flags or {}) do if not M.DAMAGE_FLAGS[fl] then add("damage.flag:" .. tostring(fl)) end end
            expr(instr.to); expr(instr.amount)
          elseif op == "create_unit" then
            if templates and not templates[instr.template] then add("summon template not registered") end
            expr(instr.at); expr(instr.lifetime); expr(instr.count)
          elseif op == "if" then expr(instr.cond); pending[#pending + 1] = instr.actions; pending[#pending + 1] = instr.otherwise
          elseif op == "let" then expr(instr.value)
          elseif op == "heal" then
            local src = instr.source and exprs[instr.source]
            if src and not (#src == 1 and src[1][1] == "name" and src[1][2] == "caster") then add("action:heal.source") end
            expr(instr.to); expr(instr.amount); expr(instr.source)
          elseif op == "remove_status" then expr(instr.from)
          elseif op == "reveal" then
            if instr.teams and instr.teams ~= "all" then add("reveal.teams:" .. instr.teams) end
            expr(instr.at); expr(instr.follow); expr(instr.radius); expr(instr.duration)
          elseif op == "call" then for _, k in ipairs(keys(instr.args)) do expr(instr.args[k]) end
          elseif op == "tether" then
            if instr.from == nil then add("tether.from") end
            local seq = {}
            for _, x in ipairs(programs[instr.on_expire] or {}) do if x.op ~= "cue" then seq[#seq + 1] = x.op end end
            if table.concat(seq, ",") ~= "let,damage,apply_status" then add("tether.shape") end
            expr(instr.to); expr(instr.duration)
          elseif op == "for_each" then
            local s = selectors[instr.select] or {}
            for _, k in ipairs(keys(s)) do if not M.SELECTOR_FIELDS[k] then add("selector." .. k) end end
            if s.shape ~= "circle" then add("selector.shape:" .. tostring(s.shape)) end
            for _, fl in ipairs(type(s.flags) == "table" and s.flags or {}) do
              if not M.SELECTOR_FLAGS[fl] then add("selector.flag:" .. tostring(fl)) end
            end
            local tk = types_key(s.types)
            if not M.SELECTOR_TYPES[tk] then add("selector.types:" .. tk) end
            if s.team == "enemy" then pending[#pending + 1] = instr.actions
            elseif s.team == "ally" then
              if tk ~= "" and tk ~= "all" then add("ally_selector.types") end
              -- host ally restore: heal / restore_mana / set_health shares of max, onto `it`
              for _, x in ipairs(programs[instr.actions] or {}) do
                if x.op ~= "cue" and x.op ~= "heal" and x.op ~= "restore_mana" and x.op ~= "set_health" then add("ally_selector.effect:" .. tostring(x.op)) end
              end
            else add("selector.team:" .. tostring(s.team)) end
            expr(s.from); expr(s.radius); expr(s.where)
          end
        end
      end
    end
  end
  if entries.on_toggle_on then
    local ok = false
    for _, instr in ipairs(programs[entries.on_toggle_on.program] or {}) do
      local to = instr.op == "apply_status" and exprs[instr.to]
      if to and #to == 1 and to[1][1] == "name" and to[1][2] == "caster" then ok = true end
    end
    if not ok then add("toggle: on_toggle_on must put a status on the caster") end
  end
  -- statuses a program applies
  for _, s in ipairs(ir.statuses or {}) do
    if used_statuses[s.id] then
      for _, k in ipairs(keys(s)) do if not M.STATUS_FIELDS[k] then add("status." .. k) end end
      if s.stacking ~= "refresh" then add("status.stacking:" .. (type(s.stacking) == "table" and "stacks" or tostring(s.stacking))) end
      if s.per_source and not tostring(s.runtime_id or ""):find("{caster}", 1, true) then add("status.per_source without a {caster} runtime id") end
      if s.duration ~= "infinite" then expr(s.duration) end
      stat_rows(s.stats, "status")
      local payload = false
      for _, st in ipairs(s.states or {}) do
        if M.STATE_EFFECTS and not M.STATE_EFFECTS[st] then add("state:" .. tostring(st)) end
        if st == "invisible" then payload = true end
      end
      if s.triggers then
        local rid = tostring(s.runtime_id or "")
        if not rid:find("^attack_orb:") then add("status.triggers")
        else
          for _, k in ipairs(keys(s.triggers)) do if not M.ORB_TRIGGERS[k] then add("status.triggers:" .. k) end end
          if (s.stats and #s.stats > 0) or payload then add("status.triggers: an orb status with stats") end
        end
      end
    end
  end
  -- target and requires
  if ir.target then
    local t = ir.target
    for _, k in ipairs(keys(t)) do if not M.TARGET_FIELDS[k] then add("target." .. k) end end
    if t.type and not M.TARGET_TYPES[t.type] then add("target.type:" .. t.type) end
    if t.team and not M.TARGET_TEAMS[t.team] then add("target.team:" .. t.team) end
    if type(t.types) == "table" then
      local names = { table.unpack(t.types) }
      table.sort(names)
      if not M.TARGET_UNIT_TYPES[table.concat(names, "+")] then add("target.types:" .. table.concat(t.types, "+")) end
    end
    for _, list in ipairs({ t.flags or {}, t.ability_flags or {} }) do
      for _, f in ipairs(list) do if not M.TARGET_FLAGS[f] then add("target.flag:" .. f) end end
    end
    expr(t.range)
  end
  if ir.requires then add("requires") end
  if (ir.locals or 0) > 16 then add("more than 16 locals") end
  -- tokens of every reachable expression
  for e in pairs(used_exprs) do
    for _, tok in ipairs(exprs[e] or {}) do
      local kind = tok[1]
      if not M.TOKENS[kind] then add("token:" .. tostring(kind))
      elseif kind == "name" and not M.NAMES[tok[2]] then add("name:" .. tostring(tok[2]))
      elseif kind == "attr" and not M.ATTRS[tok[2]] then add("attr:" .. tostring(tok[2]))
      elseif kind == "call" and M.FUNCS[tok[2]] ~= tok[3] then add("fn:" .. tostring(tok[2]))
      elseif kind == "op" and not M.OPS[tok[2]] then add("op:" .. tostring(tok[2])) end
    end
  end
  return b
end

-- a short label for a probe error, to compare with the scan
function M.probe_feature(err)
  local e = tostring(err):gsub("^[%w_]+: ", "")
  e = e:gsub("^expression %d+: ", "")
  local m = e:match("^action `([%w_]+)` is not run yet") if m then return "action:" .. m end
  m = e:match("^program %d+ `([%w_]+)`: `([%w_]+)` is not run yet") if m then local f = select(2, e:match("^program %d+ `([%w_]+)`: `([%w_]+)`")); return "action:" .. m .. "." .. f end
  m = e:match("^event `([%w_]+)`") if m then return "event:" .. m end
  m = e:match("^passive `([%w_]+)` is not run yet") if m then return "passive_trigger:" .. m end
  m = e:match("^passive: `([%w_]+)`") if m then return "passive:" .. m end
  m = e:match("^status `[%w_]+`: stacking") if m then return "status.stacking" end
  m = e:match("^status `[%w_]+`: `([%w_]+)` is not run yet") if m then return "status." .. m end
  m = e:match("^function `([%w_]+)`") if m then return "fn:" .. m end
  m = e:match("^attribute `([%w_]+)`") if m then return "attr:" .. m end
  m = e:match("^name `([%w_]+)`") if m then return "name:" .. m end
  m = e:match("^token `([%w_]+)`") if m then return "token:" .. m end
  m = e:match("^operator `(%S+)`") if m then return "op:" .. m end
  if e:match("^selector: only circles") then return "selector.shape" end
  m = e:match("^status `[%w_]+`: state `([%w_]+)` is not run yet") if m then return "state:" .. m end
  if e:match("^status `[%w_]+`: triggers are not run yet") then return "status.triggers" end
  if e:match("^status `[%w_]+`: an attack orb settles") or e:match("has no host attack orb") then return "status.triggers" end
  if e:match("only the caster heals") then return "action:heal.source" end
  m = e:match("stat op `([%w_]+)` is not run yet") if m then return "stat.op:" .. m end
  m = e:match("^passive: unknown stat `([%w_]+)`") if m then return "stat:" .. m end
  if e:match("^a toggle needs both") then return "toggle: needs both on_toggle_on and on_toggle_off" end
  if e:match("^on_toggle_on must put") then return "toggle: on_toggle_on must put a status on the caster" end
  if e:match("^selector: `types`") then return "selector.types" end
  m = e:match("^selector types %[(.-)%] are not run yet")
  if m then
    local names = {}
    for n in m:gmatch('"([%w_]+)"') do names[#names + 1] = n end
    table.sort(names)
    return "selector.types:" .. table.concat(names, "+")
  end
  m = e:match("damage flag `([%w_]+)` is not run yet") if m then return "damage.flag:" .. m end
  m = e:match("selector flag `([%w_]+)` is not run yet") if m then return "selector.flag:" .. m end
  if e:match("^ally selector: `types`") then return "ally_selector.types" end
  if e:match("^cast `requires`") then return "requires" end
  m = e:match("^target type `([%w_]+)`") if m then return "target.type:" .. m end
  m = e:match("^status `[%w_]+`: unknown stat `([%w_]+)`") if m then return "stat:" .. m end
  m = e:match("^target flag `([%w_]+)`") if m then return "target.flag:" .. m end
  if e:match("^target types") then return "target.types" end
  if e:match("^unknown unit template") then return "summon template not registered" end
  return e
end

function M.build(content, probe, waves)
  local templates = {}
  do
    package.path = tools_dir .. "/../../../scripts/?.lua;scripts/?.lua;" .. package.path
    local ok, rows = pcall(function() return require("content_builder").new(tools_dir .. "/../../../scripts/lua_data").include("templates/summons.lua") end)
    if ok and type(rows) == "table" then for _, r in ipairs(rows) do if r.id then templates[r.id] = true end end end
  end
  local entries = {}
  for _, key in ipairs({ "abilities", "items" }) do
    for id, block in pairs(content[key] or {}) do
      if block.ir then entries[id] = { id = id, kind = key == "abilities" and "ability" or "item", blockers = M.scan(block.ir, next(templates) and templates or nil) } end
    end
  end
  local mismatch = {}
  if probe then
    for id, p in pairs(probe) do
      local e = entries[id]
      if e then
        e.probe = p
        local n = 0 for _ in pairs(e.blockers) do n = n + 1 end
        if p.ok and n > 0 then mismatch[#mismatch + 1] = { id = id, kind = "scan blocks a block that loads", scan = keys(e.blockers) } end
        if not p.ok and n == 0 then mismatch[#mismatch + 1] = { id = id, kind = "scan misses a load failure", error = p.error } end
      end
    end
    table.sort(mismatch, function(a, b) return a.id < b.id end)
  end
  -- heroes and stages
  local hero_abilities = {}
  for _, h in ipairs(content.heroes) do hero_abilities[h.source_id] = h end
  local stages = {}
  local m5 = { name = "M5（已上架、仍由 Rust handler 執行的英雄）", heroes = {} }
  for _, h in ipairs(content.heroes) do
    if h.listed and h.source_id ~= "B01" and h.source_id ~= "C17" then m5.heroes[#m5.heroes + 1] = h.source_id end
  end
  table.sort(m5.heroes)
  stages[1] = m5
  for _, w in ipairs(waves or {}) do
    local s = { name = "M6 第 " .. w.index .. " 批", heroes = {} }
    for _, u in ipairs(w.heroes) do s.heroes[#s.heroes + 1] = u.id end
    stages[#stages + 1] = s
  end
  local function hero_blockers(sid)
    local h = hero_abilities[sid]
    local out = {}
    for _, a in ipairs(h and h.abilities or {}) do
      local e = entries[a]
      if e then for f in pairs(e.blockers) do out[f] = (out[f] or 0) + 1 end end
    end
    return out
  end
  -- totals per feature
  local totals = {}
  for _, e in pairs(entries) do
    for f in pairs(e.blockers) do
      local t = totals[f] or { feature = f, abilities = 0, items = 0, heroes = {}, first_errors = 0 }
      totals[f] = t
      if e.kind == "ability" then t.abilities = t.abilities + 1 else t.items = t.items + 1 end
    end
    if e.probe and not e.probe.ok then
      local f = M.probe_feature(e.probe.error)
      local t = totals[f] or { feature = f, abilities = 0, items = 0, heroes = {}, first_errors = 0 }
      totals[f] = t
      t.first_errors = t.first_errors + 1
    end
  end
  for _, h in ipairs(content.heroes) do for f in pairs(hero_blockers(h.source_id)) do if totals[f] then totals[f].heroes[h.source_id] = true end end end
  -- order: stage by stage, the hero that needs the fewest new features per hero unlocked first
  local have, order, done = {}, {}, {}
  for _, stage in ipairs(stages) do
    local function missing(sid) local m = {} for f in pairs(hero_blockers(sid)) do if not have[f] then m[#m + 1] = f end end table.sort(m) return m end
    local function unlocked() local n = 0 for _, sid in ipairs(stage.heroes) do if #missing(sid) == 0 then n = n + 1 end end return n end
    stage.ready_at_start = unlocked()
    stage.steps = {}
    while true do
      local best, best_score, best_m
      for _, sid in ipairs(stage.heroes) do
        local m = missing(sid)
        if #m > 0 then
          local trial = {} for _, f in ipairs(m) do trial[f] = true end
          local gain = 0
          for _, o in ipairs(stage.heroes) do
            local ok = true
            for f in pairs(hero_blockers(o)) do if not have[f] and not trial[f] then ok = false end end
            if ok and #missing(o) > 0 then gain = gain + 1 end
          end
          local score = gain / #m
          if not best or score > best_score or (score == best_score and #m < #best_m) then best, best_score, best_m = sid, score, m end
        end
      end
      if not best then break end
      for _, f in ipairs(best_m) do
        have[f] = true
        local now = unlocked()
        stage.steps[#stage.steps + 1] = { feature = f, for_hero = best, unlocked = now }
      end
    end
    stage.unlocked_at_end = unlocked()
  end
  return { entries = entries, totals = totals, stages = stages, mismatch = mismatch, probe = probe ~= nil }
end

function M.render(r)
  local out = {}
  local function w(s) out[#out + 1] = s end
  local n_entries, blocked, probe_ok, probe_fail = 0, 0, 0, 0
  for _, e in pairs(r.entries) do
    n_entries = n_entries + 1
    if next(e.blockers) then blocked = blocked + 1 end
    if e.probe then if e.probe.ok then probe_ok = probe_ok + 1 else probe_fail = probe_fail + 1 end end
  end
  w("# 直譯器尚未支援的寫法與實作優先序")
  w("")
  w("由 `tools\\lua\\lua.exe docs\\character_pipeline\\tools\\v3_ir_blockers.lua` 產生，不要手改。每次工作線 B 放回一批，就照 §4 重跑一次。")
  w("")
  w(("- 全量內容中帶 IR 的條目：%d 個。依 ir.rs 的載入規則掃描，有 %d 個會被擋下。"):format(n_entries, blocked))
  if r.probe then
    w(("- 直譯器實際載入（`tools/ir_probe`）：%d 個成功、%d 個失敗。掃描與實際結果不一致：%d 個（§3）。"):format(probe_ok, probe_fail, #r.mismatch))
  else
    w("- 這次沒有附上直譯器實際載入的結果（`--probe`），只有靜態掃描。")
  end
  w("- 一位英雄要「全部技能都能載入」才算解鎖。順序依序是 M5（其餘 8 位已移植英雄），然後 M6 第 1～9 批（[m6-waves.md](m6-waves.md)）；每一批內，先做「每多支援一項、能解鎖最多英雄」的那位英雄缺的項目。")
  w("")
  w("## 1. 實作順序")
  w("")
  local idx = 0
  for _, stage in ipairs(r.stages) do
    w("### " .. stage.name .. "（" .. #stage.heroes .. " 位）")
    w("")
    w(("開始時已能全部載入：%d 位；做完本批的項目後：%d 位。"):format(stage.ready_at_start, stage.unlocked_at_end))
    w("")
    if #stage.steps == 0 then w("（本批不需要新的支援）") else
      w("| # | 要支援的寫法 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |")
      w("|---|---|---|---|---|")
      for _, s in ipairs(stage.steps) do
        idx = idx + 1
        local t = r.totals[s.feature] or { abilities = 0, items = 0 }
        w(("| %d | `%s` | %s | %d 招、%d 件道具 | %d / %d |"):format(idx, s.feature, s.for_hero, t.abilities, t.items, s.unlocked, #stage.heroes))
      end
    end
    w("")
  end
  w("## 2. 每項寫法擋住多少")
  w("")
  w("「第一個錯誤」是直譯器實際載入時，這一項是第一個報錯原因的條目數。一招常常有好幾項阻擋，所以兩欄不會相等。")
  w("")
  w("| 寫法 | 招式 | 道具 | 英雄 | 第一個錯誤 |")
  w("|---|---|---|---|---|")
  local rows = {}
  for _, t in pairs(r.totals) do local h = 0 for _ in pairs(t.heroes) do h = h + 1 end t.hero_count = h; rows[#rows + 1] = t end
  table.sort(rows, function(a, b) if a.hero_count ~= b.hero_count then return a.hero_count > b.hero_count end if a.abilities ~= b.abilities then return a.abilities > b.abilities end return a.feature < b.feature end)
  for _, t in ipairs(rows) do w(("| `%s` | %d | %d | %d | %d |"):format(t.feature, t.abilities, t.items, t.hero_count, t.first_errors)) end
  w("")
  w("## 3. 掃描與直譯器實際結果的差異")
  w("")
  if not r.probe then w("（沒有 `--probe`）")
  elseif #r.mismatch == 0 then w("沒有差異：掃描判定可載入的條目都能載入，載入失敗的條目都至少找到一項阻擋。")
  else
    w("有差異就表示 ir.rs 已經改了，`v3_ir_blockers.lua` 開頭的支援清單要跟著更新。")
    w("")
    for _, m in ipairs(r.mismatch) do
      w("- `" .. m.id .. "`：" .. m.kind .. (m.error and ("（直譯器：" .. m.error .. "）") or "") .. (m.scan and ("（掃描：" .. table.concat(m.scan, "、") .. "）") or ""))
    end
  end
  w("")
  w("## 4. 重跑")
  w("")
  w("```")
  w("tools\\lua\\lua.exe scripts\\gen_v3_content.lua")
  w("cargo run --release --manifest-path tools\\ir_probe\\Cargo.toml --target-dir target\\ir_probe -- target\\v3-content\\v3-content.json > target\\v3-content\\ir-probe.jsonl")
  w("tools\\lua\\lua.exe docs\\character_pipeline\\tools\\v3_ir_blockers.lua --probe target\\v3-content\\ir-probe.jsonl")
  w("```")
  w("")
  w("§3 有差異時，先更新 `v3_ir_blockers.lua` 開頭的支援清單（照 omoba-core/src/runtime/native/ir.rs），再重跑。")
  w("")
  return table.concat(out, "\n")
end

function M.main(argv)
  local root = tools_dir .. "/../../.."
  package.path = root .. "/scripts/?.lua;scripts/?.lua;" .. package.path
  local b = require("_bootstrap")
  local json = b.lib("json")
  local opts = { content = b.root .. "/target/v3-content/v3-content.json", out = b.root .. "/docs/plans/2026-10-28-history-nobu-full-port/ir-blockers.md" }
  local i = 1
  while i <= #argv do
    local a = argv[i]
    if a == "--content" or a == "--probe" or a == "--out" then opts[a:sub(3)] = argv[i + 1]; i = i + 1
    else error("unknown argument " .. a) end
    i = i + 1
  end
  local f = assert(io.open(opts.content, "rb"), "run scripts/gen_v3_content.lua first: " .. opts.content)
  local content = json.decode(f:read("a")); f:close()
  local probe
  if opts.probe then
    probe = {}
    for line in io.lines(opts.probe) do
      if line:match("^%s*{") then local row = json.decode(line); probe[row.id] = row end
    end
  end
  local prev = rawget(_G, "V3_WAVES_AS_MODULE")
  V3_WAVES_AS_MODULE = true
  local W = dofile(tools_dir .. "/v3_waves.lua")
  V3_WAVES_AS_MODULE = prev
  local waves = W.build(b.root).waves
  M.STATS = M.load_stats(b.root)
  M.ABSOLUTE_STATS = M.load_vocab_list(b.root, "ABSOLUTE_STATS", "StatKey::")
  M.STATE_EFFECTS = M.load_vocab_list(b.root, "STATE_EFFECTS", "StateEffect::")
  local r = M.build(content, probe, waves)
  local o = assert(io.open(opts.out, "wb")); o:write(M.render(r), "\n"); o:close()
  local blocked = 0
  for _, e in pairs(r.entries) do if next(e.blockers) then blocked = blocked + 1 end end
  print(("wrote %s: %d IR blocks, %d blocked, %d mismatches with the probe"):format(opts.out, (function() local n = 0 for _ in pairs(r.entries) do n = n + 1 end return n end)(), blocked, #r.mismatch))
  return #r.mismatch == 0 and 0 or 1
end

if rawget(_G, "V3_IR_BLOCKERS_AS_MODULE") then return M end
os.exit(M.main(arg))
