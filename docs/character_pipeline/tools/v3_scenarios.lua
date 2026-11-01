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
-- The document being built: in a trigger's effects, `call` expands into its routine's actions
-- (E267). An entry's own events keep the call as one effect: the statuses and scenarios an entry
-- gets come from what it applies itself, not from the routines it shares with other entries.
local current_doc
local expanding = {}
local expand_calls = false

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
    -- A call runs its routine's actions here (E267): they carry the call's conditions and the
    -- routine's own path (`routines.<name>.actions[i]`, as the generated Rust tags them).
    local routine = expand_calls and a.op == "call" and current_doc and current_doc.routines and current_doc.routines[a.routine]
    if routine and not expanding[a.routine] then
      expanding[a.routine] = true
      walk_program(routine.actions or routine, "routines." .. a.routine .. ".actions", phase, conds, out)
      expanding[a.routine] = nil
    end
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
  local was = expand_calls
  expand_calls = true
  walk_program(trig.actions or trig, path, phase, conds, out)
  expand_calls = was
  return out
end

-- statuses the entry owns, by id; and the ones its actions apply (with the target expression)
local function status_index(doc, def)
  local by = {}
  for _, s in ipairs(doc.statuses or {}) do by[s.id] = s end
  -- A routine an entry calls may apply another entry's status (A34W's on-hit routine puts on
  -- A34T's resist shred; E267): every entry's statuses are known, the entry's own win.
  for _, list in ipairs({ doc.abilities or {}, doc.items or {} }) do
    for _, e in ipairs(list) do for _, s in ipairs(e.statuses or {}) do by[s.id] = by[s.id] or s end end
  end
  for _, s in ipairs(def.statuses or {}) do by[s.id] = s end
  return by
end

local function applied_statuses(effects, by)
  local out, seen = {}, {}
  for _, e in ipairs(effects) do
    if e.op == "apply_status" and e.args and e.args.status and not seen[e.args.status] then
      seen[e.args.status] = true
      out[#out + 1] = { status = by[e.args.status], id = e.args.status, to = e.args.to, path = e.path, conds = e.conds,
        duration = e.args.duration }
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
-- Quirk topics routed to the entry's own scenarios of a case (`covered_by`, line D judges it): the
-- quirk names the topic and says which way it goes; the definition has exactly one place deciding
-- it (`points`: path, flag, value); the two agree. Anything less adds nothing (D19: no pass by
-- association). D13: no number is read from the prose.
local function contains_any(text, words)
  for _, w in ipairs(words) do if text:find(w, 1, true) then return true end end
  return false
end
-- Whether a quirk names the status `id`: the id itself, or the source modifier it ports
-- (a08w_haste is modifier_A08W, not modifier_A08W_counter)
local function names_status(text, id)
  if text:find(id, 1, true) then return true end
  local modifier = "modifier_" .. (id:match("^([^_]+)_") or id):upper()
  local at = 1
  while true do
    local s, e = text:find(modifier, at, true)
    if not s then return false end
    if not text:sub(e + 1, e + 1):match("[%w_]") then return true end
    at = e + 1
  end
end
M.QUIRK_TOPICS = {
  { topic = "magic_immune", word = "魔免", case = "magic_immune",
    -- the claim: magic-immune enemies are affected (true), are not (false), or unclear (nil)
    claim = function(text)
      local no = contains_any(text, { "魔免敵人不", "魔免單位不", "不能對魔免", "不能指定魔免", "排除魔免", "不含魔免", "不打魔免", "魔免英雄不" })
      local yes = contains_any(text, { "魔免敵人會", "魔免單位也", "魔免敵人也", "無視魔免", "穿透魔免", "MAGIC_IMMUNE_ENEMIES", "魔免照樣", "魔免也" })
        or (text:find("含魔免", 1, true) and not text:find("不含魔免", 1, true))
      if no and not yes then return false elseif yes and not no then return true end
      return nil
    end,
    -- where the definition decides it: an enemy unit target (pierce_magic_immune), and every
    -- for_each / projectile selector reaching enemies (include_magic_immune)
    points = function(castable, target, effects)
      local points = {}
      if castable and target.type == "unit" and (target.team == "enemy" or target.team == "both") then
        local pierce = false
        for _, f in ipairs(castable.flags or {}) do if f == "pierce_magic_immune" then pierce = true end end
        points[#points + 1] = { path = "target", flag = "pierce_magic_immune", value = pierce }
      end
      for _, e in ipairs(effects) do
        local sel = e.op == "for_each" and e.args and e.args.select or (e.op == "projectile" and e.args) or nil
        if sel and sel.team ~= "ally" and sel.team ~= "ally_not_self" then
          local keep = false
          for _, f in ipairs(type(sel.flags) == "table" and sel.flags or {}) do if f == "include_magic_immune" then keep = true end end
          points[#points + 1] = { path = e.path, flag = "include_magic_immune", value = keep }
        end
      end
      return points
    end },
  -- whether a status can be dispelled: the entry's dispel scenarios check every status' `dispel`.
  -- Only the statuses the quirk names (its id, or the source modifier: a08w_haste is
  -- modifier_A08W), exactly one of them, the quirk saying the same: not dispellable
  -- (dispel none) or dispellable by a basic dispel (basic)
  { topic = "dispel", word = "驅散", case = "dispel",
    claim = function(text)
      local none = contains_any(text, { "不可驅散", "不能驅散", "無法驅散", "不能被驅散", "不可被驅散" })
      local basic = contains_any(text, { "可被一般驅散", "可一般驅散", "推定可驅散", "暫定可驅散", "可以被驅散", "可驅散" })
        and not none
      if none and not contains_any(text, { "可被一般驅散", "可一般驅散", "暫定可驅散" }) then return "none" end
      if basic then return "basic" end
      return nil
    end,
    points = function(_, _, _, scenarios, text)
      local by_status, order = {}, {}
      for _, s in ipairs(scenarios or {}) do
        local e = s.expect and s.expect[1]
        if e and e.status and e.dispel and names_status(text, e.status) then
          if not by_status[e.status] then by_status[e.status] = { dispel = e.dispel, list = {} }; order[#order + 1] = e.status end
          local l = by_status[e.status].list
          l[#l + 1] = s
        end
      end
      local points = {}
      for _, id in ipairs(order) do
        points[#points + 1] = { path = "statuses." .. id, flag = "dispel", value = by_status[id].dispel, scenarios = by_status[id].list }
      end
      return points
    end },
  -- how a status takes being put on again: the entry's reapply scenarios check `stacking`. Only
  -- the statuses the quirk names, exactly one, the quirk saying the same (independent: several
  -- copies at once, each timed; refresh: the duration starts over; keep: it does not)
  { topic = "stacking", word = "", case = "reapply",
    claim = function(text)
      local independent = contains_any(text, { "stacking = independent", "各自計時", "同時存在" })
        or (text:find("MULTIPLE", 1, true) and not contains_any(text, { "不是 MULTIPLE", "非 MULTIPLE", "沒有 MULTIPLE" }))
      local refresh = contains_any(text, { "stacking = refresh", "只刷新", "刷新持續", "重施刷新" })
      local keep = contains_any(text, { "stacking = keep", "不刷新" })
      local n = (independent and 1 or 0) + (refresh and 1 or 0) + (keep and 1 or 0)
      if n ~= 1 then return nil end
      return independent and "independent" or refresh and "refresh" or "keep"
    end,
    points = function(_, _, _, scenarios, text)
      local points = {}
      for _, s in ipairs(scenarios or {}) do
        local e = s.expect and s.expect[1]
        if e and e.kind == "reapplied" and names_status(text, e.status) then
          points[#points + 1] = { path = "statuses." .. e.status, flag = "stacking", value = e.stacking, scenarios = { s } }
        end
      end
      return points
    end },
  -- the cooldown an ability swap keeps: the entry's swap scenarios (its cast's or its status's
  -- swap_abilities) check the policy (line D E642; a swap with an initial_cooldown is checked
  -- against that instead, so it decides nothing here). Only an entry with exactly one such swap,
  -- the quirk naming the policy (continue: kept or carried over; reset: cleared), not both
  { topic = "swap_cooldown", word = "冷卻", case = "swap",
    claim = function(text)
      local continue = contains_any(text, { "cooldown_policy = continue", "保留冷卻", "冷卻延續" })
      local reset = contains_any(text, { "cooldown_policy = reset", "冷卻清空" })
      if continue and not reset then return "continue" elseif reset and not continue then return "reset" end
      return nil
    end,
    points = function(_, _, effects, scenarios)
      local by_to, order = {}, {}
      for _, s in ipairs(scenarios or {}) do
        local e = s.expect and s.expect[2]
        if e and e.kind == "ability_swapped" and e.policy and e.initial_cooldown == nil then
          if not by_to[e.to] then
            local path = e.by_status and ("statuses." .. e.by_status .. ".triggers.on_created") or nil
            for _, x in ipairs(effects) do
              if not path and x.op == "swap_abilities" and x.args and x.args.to == e.to then path = x.path end
            end
            by_to[e.to] = { path = path or e.to, policy = e.policy, list = {} }; order[#order + 1] = e.to
          end
          local l = by_to[e.to].list
          l[#l + 1] = s
        end
      end
      local points = {}
      for _, to in ipairs(order) do
        points[#points + 1] = { path = by_to[to].path, flag = "cooldown_policy", value = by_to[to].policy, scenarios = by_to[to].list }
      end
      return points
    end },
  -- a cast on itself cancelled before the cost and cooldown (no_target_self): the entry's
  -- target_invalid/self scenario checks the refusal and that no cooldown starts. Only a quirk
  -- naming the flag and saying the cooldown is not paid
  { topic = "self_cast_cooldown", word = "冷卻", case = "target_invalid",
    claim = function(text)
      if text:find("no_target_self", 1, true) and contains_any(text, { "不進冷卻", "不扣冷卻" }) then return true end
      return nil
    end,
    points = function(castable, target, _, scenarios)
      if not (castable and target.type == "unit" and has_flag(castable, "no_target_self")) then return {} end
      local list = {}
      for _, s in ipairs(scenarios or {}) do
        if s.id:sub(-#"/self") == "/self" then list[#list + 1] = s end
      end
      if #list == 0 then return {} end
      return { { path = "target", flag = "no_target_self", value = true, scenarios = list } }
    end },
  -- units the caster's team cannot see in a selector's area: the entry's hidden_in_effect
  -- scenarios (an invisible and a fogged enemy per selector). Only a selector where both agree
  -- (no exclude_invisible) and exactly one of them, the quirk saying the same as its `affected`
  { topic = "hidden_selection", word = "", case = "hidden_in_effect",
    claim = function(text)
      local no = contains_any(text, { "看不見的敵人不", "迷霧中的敵人不", "隱形的敵人不", "隱形敵人不", "只選看得見", "FOW_VISIBLE", "不含隱形", "排除隱形" })
      local yes = contains_any(text, { "隱形與迷霧中的敵人照樣", "隱形與迷霧中的敵人也", "隱形、迷霧中的敵人", "迷霧中的敵人也", "迷霧中的敵人照樣",
        "包含隱形與迷霧", "FOW 不篩選", "隱形單位也會", "不看視野", "沒有 FOW 旗標" })
      if no and not yes then return false elseif yes and not no then return true end
      return nil
    end,
    points = function(_, _, _, scenarios)
      local by_path, order = {}, {}
      for _, s in ipairs(scenarios or {}) do
        local e = s.expect and s.expect[1]
        if e and e.kind == "hidden_affected" then
          local p = by_path[e.path]
          if not p then p = { affected = e.affected, list = {} }; by_path[e.path] = p; order[#order + 1] = e.path end
          if p.affected ~= e.affected then p.mixed = true end
          p.list[#p.list + 1] = s
        end
      end
      local points = {}
      for _, path in ipairs(order) do
        local p = by_path[path]
        if not p.mixed then points[#points + 1] = { path = path, flag = "visible_only", value = p.affected, scenarios = p.list } end
      end
      return points
    end },
  -- an enemy hero's illusion in a selector's area: the entry's one illusion_in_effect scenario
  -- (one hero selector it could be about), the quirk saying the same as its `affected`
  { topic = "illusion_selection", word = "幻象", case = "illusion_in_effect",
    claim = function(text)
      local no = contains_any(text, { "不含幻象", "排除幻象", "不包括幻象", "幻象不受", "幻象不會" })
      local yes = text:find("含幻象", 1, true) ~= nil and not contains_any(text, { "不含幻象" })
        or contains_any(text, { "幻象也會被", "包括幻象", "幻象一樣" })
      if no and not yes then return false elseif yes and not no then return true end
      return nil
    end,
    points = function(_, _, _, scenarios)
      local points = {}
      for _, s in ipairs(scenarios or {}) do
        local e = s.expect and s.expect[1]
        if e and e.kind == "illusion_affected" then
          points[#points + 1] = { path = e.path, flag = "include_illusions", value = e.affected, scenarios = { s } }
        end
      end
      return points
    end },
  -- buildings in a selector's area: the entry's one building_in_effect scenario (one selector it
  -- could be about), the quirk saying the same as its `affected`
  { topic = "building_selection", word = "建築", case = "building_in_effect",
    claim = function(text)
      local no = contains_any(text, { "不含建築", "不打建築", "搜不到建築", "不包括建築", "不對建築", "建築不受", "不能對建築" })
      local yes = contains_any(text, { "含建築", "包括建築", "建築也", "TARGET_ALL", "BUILDING" })
        and not contains_any(text, { "不含建築", "不包括建築" })
      if no and not yes then return false elseif yes and not no then return true end
      return nil
    end,
    points = function(_, _, _, scenarios)
      local points = {}
      for _, s in ipairs(scenarios or {}) do
        local e = s.expect and s.expect[1]
        if e and e.kind == "building_affected" then
          points[#points + 1] = { path = e.path, flag = "types_building", value = e.affected, scenarios = { s } }
        end
      end
      return points
    end },
  -- the cast's damage after its caster died: it lands only with require_source_alive = false. The
  -- death scenarios check the default (damage_skipped_if_source_dead per path); a quirk saying the
  -- damage stops is covered when the entry has exactly one such path. One saying it goes on has
  -- no scenario that checks it, so it adds nothing.
  { topic = "source_dead_damage", word = "死亡", case = "death",
    claim = function(text)
      if not text:find("傷害", 1, true) then return nil end
      local stops = contains_any(text, { "死亡後不造成", "死亡時不造成", "死亡時不再", "死亡後不再", "不會再有傷害", "攻擊者已死不造成",
        "攻擊者需存活", "要求政秀存活", "後續尖刺不造成", "之後的波次不造成", "不再造成傷害", "死亡時傷害由規則層歸零",
        "死亡後由規則層歸", "死亡時由規則層歸零", "require_source_alive 預設", "require_source_alive 歸零", "規則層 require_source_alive）" })
      local goes = contains_any(text, { "仍造成", "仍會造成", "傷害照常", "照常攻擊", "照常落下", "不檢查施法者存活", "require_source_alive = false" })
      if stops and not goes then return true end
      return nil
    end,
    points = function(_, _, _, scenarios)
      local by_path = {}
      local order = {}
      for _, s in ipairs(scenarios or {}) do
        for _, e in ipairs(s.expect or {}) do
          if e.kind == "damage_skipped_if_source_dead" and e.path then
            if not by_path[e.path] then by_path[e.path] = {}; order[#order + 1] = e.path end
            local list = by_path[e.path]
            if list[#list] ~= s then list[#list + 1] = s end
          end
        end
      end
      local points = {}
      for _, p in ipairs(order) do
        points[#points + 1] = { path = p, flag = "require_source_alive", value = true, scenarios = by_path[p] }
      end
      return points
    end },
}

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
  -- An entry whose rank shows only when it is learned rank by rank (E425): a passive block, or an
  -- on_upgrade / on_learn that puts its aura on again at each level (B05R, B31W).
  local by_upgrade = (type(def.passive) == "table" and next(def.passive) ~= nil)
    or (type(def.events) == "table" and (def.events.on_upgrade ~= nil or def.events.on_learn ~= nil))
  local function push(rank, case, s)
    s.id = def.id .. "/r" .. rank .. "/" .. case .. (s.suffix and ("/" .. s.suffix) or "")
    s.suffix = nil
    s.entry, s.rank, s.case = def.id, rank, case
    if by_upgrade then s.learn_by_upgrade = true end
    s.refs = ref_list(collect_refs({ s.given, s.when, s.expect }))
    out[#out + 1] = s
  end
  local cast_step = { kind = "cast", entry = def.id, target = target.type or "none", range = target.range }
  -- Mana the cast itself gives back to its caster (restore_mana / set_mana "x.mana + n" on the
  -- caster, or on `it` under a for_each that picks allies; B19's tactics; E381): the caster's
  -- mana nets the cost and this regain in the same tick.
  local function caster_regain()
    local allies = {}
    for _, e in ipairs(base_effects) do
      local team = e.op == "for_each" and e.args and e.args.select and e.args.select.team
      if team == "ally" or team == "both" then allies[#allies + 1] = e.path end
    end
    local function under_allies(path)
      for _, outer in ipairs(allies) do if path and path:sub(1, #outer + 1) == outer .. "." then return true end end
      return false
    end
    local regain
    for _, e in ipairs(base_effects) do
      local to = e.args and e.args.to
      local reaches = to == "caster" or to == "self" or (to == "it" and under_allies(e.path))
      if reaches and e.phase == "on_cast" and (e.op == "restore_mana" or e.op == "set_mana") then
        local amount = e.op == "restore_mana" and e.args.amount
          or (type(e.args.value) == "string" and e.args.value:match("^min%(%s*[%w_]+%.mana%s*%+%s*(.-),%s*[%w_]+%.max_mana%s*%)$"))
          or (type(e.args.value) == "string" and e.args.value:match("^[%w_]+%.mana%s*%+%s*(.+)$"))
        regain = amount or "any"
      end
    end
    return regain
  end
  local function cost_outcomes(spent)
    local o = {}
    if cost.mana then o[#o + 1] = { kind = "mana_spent", amount = spent and cost.mana or "0", regain = spent and caster_regain() or nil } end
    if cost.health then o[#o + 1] = { kind = "health_spent", amount = spent and cost.health or "0" } end
    if cost.charges then o[#o + 1] = { kind = "charge_spent", count = spent and "1" or "0" } end
    if cost.cooldown then
      o[#o + 1] = spent and { kind = "cooldown_started", length = cost.cooldown, starts = cost.cooldown_start or "on_cast" }
        or { kind = "cooldown_not_started" }
    end
    return o
  end
  local applied = applied_statuses(base_effects, by_status)

  -- A sub-ability that is not in the hero's list until its parent's status swaps it in (A07D2
  -- replaces A07D while A07T's status is up; E313): every case starts with that swap.
  local swapped
  -- Any sub-ability a status swaps in (A12T's form swaps three; E340), the required one first,
  -- or one its parent swaps in from any other action (A32's second slash after the first; E351).
  local function swaps_in(node)
    if type(node) ~= "table" then return nil end
    if node.op == "swap_abilities" and node.to == def.id then return node end
    for _, child in pairs(node) do
      local found = swaps_in(child)
      if found then return found end
    end
    return nil
  end
  if def.slot == "sub" and current_doc then
    local required = def.requires and def.requires.has_status
    for _, list in ipairs({ current_doc.abilities or {}, current_doc.items or {} }) do
      for _, owner in ipairs(list) do
        for _, st in ipairs(owner.statuses or {}) do
          if (required == nil or st.id == required) and not swapped then
            for _, a in ipairs((st.triggers or {}).on_created or {}) do
              if a.op == "swap_abilities" and a.to == def.id then
                swapped = { kind = "swapped_in", status = st.id, by = owner.id, replaces = a.from }
              end
            end
          end
        end
      end
    end
    for _, list in ipairs({ current_doc.abilities or {}, current_doc.items or {} }) do
      for _, owner in ipairs(list) do
        local a = not swapped and owner.id ~= def.id and swaps_in(owner)
        if a then swapped = { kind = "swapped_in", by = owner.id, replaces = a.from } end
      end
    end
  end
  -- An entry in its own slot that requires a status another ability gives (A12F needs A12T's
  -- form; E362): every case starts by casting that ability.
  if not swapped and def.slot ~= "sub" and def.requires and def.requires.has_status and current_doc then
    for _, list in ipairs({ current_doc.abilities or {}, current_doc.items or {} }) do
      for _, owner in ipairs(list) do
        for _, st in ipairs(owner.statuses or {}) do
          if st.id == def.requires.has_status and owner.id ~= def.id and not swapped then
            swapped = { kind = "required_status", by = owner.id, status = runtime_key(st) }
          end
        end
      end
    end
  end
  -- An entry another ability gives a level only for a while (it sets the level and sets it back
  -- to 0: C07T's cloud makes C07T2 castable while it lasts; E440): every case starts by casting
  -- that ability. A level kept in step with the owner's rank (on_upgrade `ability.level + 1`)
  -- is the scenario's rank already.
  if not swapped and def.learnable == false and current_doc then
    local function levels(node, zero)
      if type(node) ~= "table" then return false end
      if node.op == "set_ability_level" and node.ability == def.id and (tostring(node.level) == "0") == zero then return true end
      for _, child in pairs(node) do if levels(child, zero) then return true end end
      return false
    end
    for _, owner in ipairs(current_doc.abilities or {}) do
      if owner.id ~= def.id and not swapped and levels(owner.events, false) and levels(owner.events, true) then
        swapped = { kind = "leveled_by", by = owner.id }
      end
    end
  end
  -- What on_phase_start cancels the cast without (E457): the world each case starts with.
  --   a for_each that flags an enemy with a status (`where = has_status(it, X)`), then cancels
  --     when no flag: the target holds X (C25T's bleeding enemy);
  --   `target.hp_pct >= caster.hp_pct` cancels: an ally target stands lower than the caster (C23T);
  --   `count_in_radius(point, r, team_of(caster), …) == 0` cancels: an allied unit stands at the
  --     point (C19T).
  local phase_needs = {}
  do
    local actions = type(def.events) == "table" and def.events.on_phase_start or nil
    local function status_runtime(id)
      for _, list in ipairs({ current_doc and current_doc.abilities or {}, current_doc and current_doc.items or {} }) do
        for _, owner in ipairs(list) do
          for _, st in ipairs(owner.statuses or {}) do if st.id == id then return runtime_key(st) end end
        end
      end
      return id
    end
    local function cancels(node)
      if type(node) ~= "table" then return false end
      if node.op == "cancel_cast" then return true end
      for _, child in ipairs(node.actions or {}) do if cancels(child) then return true end end
      return false
    end
    local flagged, corpse = {}, {}
    local function has_flag_in(list, flag)
      for _, f in ipairs(type(list) == "table" and list or {}) do if f == flag then return true end end
      return false
    end
    for _, a in ipairs(type(actions) == "table" and actions or {}) do
      -- a corpse within the radius (B23D; E547)
      if a.op == "for_each" and type(a.select) == "table" and a.select.where == "not alive(it)" and has_flag_in(a.select.flags, "include_dead") then
        for _, act in ipairs(a.actions or {}) do
          if act.op == "set_state" and act.value == "true" then corpse[act.var] = a.select.radius end
        end
      elseif a.op == "for_each" and type(a.select) == "table" and a.select.team == "enemy" and type(a.select.where) == "string" then
        local status = a.select.where:match("^has_status%(it, ([%w_]+)%)$")
        for _, act in ipairs(a.actions or {}) do
          if status and act.op == "set_state" and act.value == "true" then flagged[act.var] = status end
        end
      elseif a.op == "if" and type(a.cond) == "string" and cancels(a) then
        local var = a.cond:match("^not state%.([%w_]+)$")
        if var and flagged[var] then phase_needs[#phase_needs + 1] = { kind = "target_status", runtime_id = status_runtime(flagged[var]) } end
        if var and corpse[var] then phase_needs[#phase_needs + 1] = { kind = "corpse_near_caster", radius = corpse[var] } end
        if a.cond:find("target.hp_pct >= caster.hp_pct", 1, true) then phase_needs[#phase_needs + 1] = { kind = "ally_target_lower" } end
        local radius = a.cond:match("^count_in_radius%(point, ([^,]+), team_of%(caster%)")
        if radius and a.cond:find("== 0$") then phase_needs[#phase_needs + 1] = { kind = "ally_unit_at_point", radius = radius } end
      end
    end
    -- What another entry of the hero makes (B16's moon, `exists(state.b16_moon)`; E568): the
    -- entry whose on_cast sets that state is cast first.
    do
      local texts = {}
      if type(def.requires) == "table" and type(def.requires.condition) == "string" then texts[#texts + 1] = def.requires.condition end
      local cast_actions = type(def.events) == "table" and def.events.on_cast or nil
      for _, a in ipairs(type(cast_actions) == "table" and cast_actions or {}) do
        if a.op == "if" and type(a.cond) == "string" then texts[#texts + 1] = a.cond end
      end
      local done = {}
      for _, text in ipairs(texts) do
        for var in text:gmatch("exists%(state%.([%w_]+)%)") do
          for _, other in ipairs(current_doc and current_doc.abilities or {}) do
            local acts = other ~= def and type(other.events) == "table" and other.events.on_cast or nil
            for _, a in ipairs(type(acts) == "table" and acts or {}) do
              if a.op == "set_state" and a.var == var and not done[other.id] then
                done[other.id] = true
                phase_needs[#phase_needs + 1] = { kind = "cast_first", entry = other.id }
              end
            end
          end
        end
      end
    end
    -- A cast that blinks its caster ahead first (C12E; E502): its target stands where it lands.
    local on_cast = type(def.events) == "table" and def.events.on_cast or nil
    for _, a in ipairs(type(on_cast) == "table" and on_cast or {}) do
      local ahead = a.op == "teleport" and a.who == "caster" and type(a.to) == "string" and a.to:match("^offset%(caster, caster%.forward, (.+)%)$")
      if ahead and ahead:sub(1, 1) ~= "-" then phase_needs[#phase_needs + 1] = { kind = "target_ahead", distance = ahead }; break end
    end
  end
  -- An ability of a unit the hero creates (a unit template's list, add_ability on another unit):
  -- not the hero's, so its cases need that unit (E351).
  local unit_owned
  if current_doc then
    for _, t in ipairs(current_doc.unit_templates or {}) do
      for _, id in ipairs(t.abilities or {}) do
        if id == def.id and not unit_owned then unit_owned = { kind = "unit_ability", unit = t.id } end
      end
    end
    local function adds(node)
      if type(node) ~= "table" then return false end
      if node.op == "add_ability" and node.ability == def.id and node.to ~= "caster" and node.to ~= "self" then return true end
      -- and an item given to another unit (C06T's trap item; E481)
      if node.op == "give_item" and node.item == def.id and node.to ~= "caster" and node.to ~= "self" then return true end
      for _, child in pairs(node) do if adds(child) then return true end end
      return false
    end
    if not unit_owned and adds(current_doc) then unit_owned = { kind = "unit_ability" } end
  end
  for rank = 1, max_level do
    local learned = { kind = "learned", entry = def.id, rank = rank }
    -- facts after `learned`; nil entries (a fact that does not apply) are skipped
    local function given(extra)
      local g = { learned }
      if swapped then g[#g + 1] = swapped end
      if unit_owned then g[#g + 1] = unit_owned end
      for _, need in ipairs(phase_needs) do g[#g + 1] = need end
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
        -- An entry its own cast deactivates (A05R until it has taken enough damage; E367) stays
        -- refused after the cooldown.
        local deactivates = false
        for _, e in ipairs(base_effects) do
          if e.op == "set_ability_active" and e.args and e.args.ability == def.id and e.args.active == false
            and e.phase == "on_cast" and not (e.conds and #e.conds > 0) then deactivates = true end
        end
        push(rank, "cooldown", { given = given({ valid_target }),
          when = { cast_step, { kind = "wait", seconds = "0" }, cast_step },
          expect = { { kind = "cast_rejected", nth = 2, reason = "cooldown", remaining = cost.cooldown },
            deactivates and { kind = "cast_rejected", nth = 3, after_wait = cost.cooldown, reason = "inactive",
              note = "the third cast, after waiting the cooldown: the cast deactivated the entry" }
            or { kind = "cast_accepted", nth = 3, after_wait = cost.cooldown, note = "the third cast, after waiting the cooldown" } } })
      end
      -- An ability swap its cast makes on its caster (B24T's shatter, A32's second slash; E608):
      -- the slot holds the ability swapped in, with the cooldown a plain modify_cooldown of the same
      -- cast sets, in the authority and the team replicas, and the original after the duration.
      for _, e in ipairs(base_effects) do
        local a = e.args
        if e.op == "swap_abilities" and e.phase == "on_cast" and not (e.conds and #e.conds > 0) and a
          and (a.on == "caster" or a.on == "self") and type(a.from) == "string" and type(a.to) == "string" then
          local initial
          for _, m in ipairs(base_effects) do
            local b = m.args
            if m.op == "modify_cooldown" and m.phase == "on_cast" and not (m.conds and #m.conds > 0) and b
              and b.ability == a.to and b.mode == "set" then initial = b.amount end
          end
          push(rank, "swap", { suffix = a.to, given = given({ valid_target }), when = { cast_step },
            expect = { { kind = "cast_accepted" }, { kind = "ability_swapped", from = a.from, to = a.to, duration = a.duration,
              policy = a.cooldown_policy, initial_cooldown = initial } },
            rule = "v3.md swap_abilities: the slot, its cooldown and the swap back" })
        end
      end
      -- A swap a status of this cast makes when it is put on the caster (A12T's form, A07T's
      -- strongest; E629), and the reverse swap of its on_removed when it runs out. Its only
      -- condition may be that the caster lacks it (A12T's first cast).
      local own_status = {}
      for _, st in ipairs(def.statuses or {}) do own_status[st.id] = st end
      local function money(v) return type(v) == "number" or (type(v) == "string" and v:match("^%$[%w_.]+$") ~= nil) end
      for _, e in ipairs(base_effects) do
        local a = e.args
        local st = e.op == "apply_status" and e.phase == "on_cast" and a and (a.to == "caster" or a.to == "self") and own_status[a.status]
        local lacks = true
        for _, c in ipairs(st and e.conds or {}) do
          if c ~= "not (has_status(caster, " .. a.status .. "))" and c ~= "not has_status(caster, " .. a.status .. ")" then lacks = false end
        end
        local trig = st and lacks and st.triggers or {}
        for _, w in ipairs(trig.on_created or {}) do
          if w.op == "swap_abilities" and (w.on == "self" or w.on == "caster") and type(w.from) == "string" and type(w.to) == "string" then
            local back, initial
            for _, r in ipairs(trig.on_removed or {}) do
              if r.op == "swap_abilities" and r.from == w.to and r.to == w.from and money(st.duration) then back = st.duration end
            end
            for _, m in ipairs(trig.on_created) do
              if m.op == "modify_cooldown" and m.ability == w.to and m.mode == "set" and money(m.amount) then initial = m.amount end
            end
            push(rank, "swap", { suffix = w.to, given = given({ valid_target }), when = { cast_step },
              expect = { { kind = "cast_accepted" }, { kind = "ability_swapped", from = w.from, to = w.to, duration = back,
                policy = w.cooldown_policy, initial_cooldown = initial, by_status = st.id } },
              rule = "v3.md swap_abilities in a status's on_created / on_removed" })
          end
        end
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
          -- direct: statuses the cast puts on its own target (a piercing cast reaches it) and ones
          -- applied under a selector that keeps magic-immune units (`include_magic_immune`); the
          -- skip is about the units the other selectors pick (E223, E324)
          local keeps = {}
          for _, e in ipairs(base_effects) do
            -- a for_each selector's flags, or a projectile's own (its hits; A12F; E369)
            local flags = (e.op == "for_each" and e.args and e.args.select and e.args.select.flags)
              or (e.op == "projectile" and e.args and e.args.flags)
            for _, flag in ipairs(type(flags) == "table" and flags or {}) do
              if flag == "include_magic_immune" then keeps[#keeps + 1] = e.path end
            end
          end
          local function kept(path)
            for _, outer in ipairs(keeps) do
              if path and path:sub(1, #outer + 1) == outer .. "." then return true end
            end
            return false
          end
          local direct = {}
          for _, a in ipairs(applied) do
            local key = a.status and runtime_key(a.status)
            -- a cast that pierces magic immunity reaches them with every status (B02; E485)
            if key and (a.to == "target" or kept(a.path) or has_flag(castable, "pierce_magic_immune")) then direct[#direct + 1] = key end
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
      local after = { { kind = "caster_dies", after = "cast" } }
      local exp_d = {}
      -- Only statuses on the scenario's target side that land before the death (E246): not the
      -- other team's branch (`is_enemy(caster, target)` against an ally, and the reverse), and not
      -- ones a later block applies (a tether's on_expire, a delay); a projectile's on_hit lands
      -- soon after the cast and stays.
      local side = valid_target and valid_target.team == "ally" and "ally" or "enemy"
      -- A delay's or repeat's actions carry no condition; their paths sit under its path (E251).
      local later = {}
      for _, e in ipairs(base_effects) do
        -- A repeat with neither `first` nor `interval` runs all its rounds at once (E324).
        local spaced = e.args and (e.args.first or e.args.interval)
        if e.op == "delay" or (e.op == "repeat" and spaced) then later[#later + 1] = e.path end
      end
      local function deferred(path)
        for _, outer in ipairs(later) do
          if path and path:sub(1, #outer + 1) == outer .. "." then return true end
        end
        return false
      end
      -- Late = settles after the cast (E288): another event, under a delay or repeat, or under a
      -- program key that runs later (a projectile's on_hit, a tether's on_break...). An `if` or
      -- `chance` directly in on_cast settles with the cast; its conditions do not make it late.
      local function structural(path)
        for k in (path or ""):gmatch("%]%.([%w_]+)") do
          if k ~= "actions" and k ~= "otherwise" then return true end
        end
        return false
      end
      local late = {}
      for _, e in ipairs(base_effects) do
        if e.phase ~= "on_cast" or deferred(e.path) or structural(e.path) then late[#late + 1] = e end
      end
      local function this_branch(conds)
        for _, c in ipairs(conds or {}) do
          if side == "enemy" and c == "not (is_enemy(caster, target))" then return false end
          if side == "ally" and c == "is_enemy(caster, target)" then return false end
          if c:match("^[%w_]+%.on_expire$") or c:match("^delay%.") or c:match("^repeat%.") then return false end
          -- Applied only when some other status is up (A34W's routine adds A34T's shred only under
          -- a34t_active; E267): this scenario does not set that up.
          if c:find("has_status(", 1, true) or c:find("stacks(", 1, true) then return false end
        end
        return true
      end
      local death_removes = {}
      for _, a in ipairs(applied) do
        local on_death = a.status and a.status.triggers and a.status.triggers.on_death
        local actions = type(on_death) == "table" and (on_death.actions or on_death) or {}
        for _, act in ipairs(actions) do
          if act.op == "remove_status" and act.from == "self" and act.status then death_removes[act.status] = true end
        end
      end
      -- The caster's death ends a channel: what its on_channel_end / on_interrupted removes goes (B09E; E391).
      for _, ev in ipairs({ "on_channel_end", "on_interrupted" }) do
        local list = castable and castable.events and castable.events[ev]
        local function walk(node)
          if type(node) ~= "table" then return end
          if node.op == "remove_status" and node.status then death_removes[node.status] = true end
          for _, child in pairs(node) do walk(child) end
        end
        walk(list)
      end
      for _, a in ipairs(applied) do
        if not this_branch(a.conds) or deferred(a.path) then goto continue end
        -- Only the scenario's own units: a status on a unit the cast creates (C15T's arrows; E324)
        -- is not one the death case can look at.
        if not (a.to == "caster" or a.to == "self" or a.to == "target" or a.to == "it") then goto continue end
        local s = a.status or {}
        local on_caster = a.to == "caster" or a.to == "self"
        local kept = (s.scope == "hero" or s.scope == "unit" or s.scope == "permanent") and not death_removes[a.id]
        exp_d[#exp_d + 1] = { kind = ((on_caster and not kept) or death_removes[a.id]) and "status_removed" or "status_kept", status = a.id, on = a.to, runtime_id = runtime_key(s),
          rule = on_caster and "v3.md §12：狀態隨持有者死亡移除，scope hero/unit/permanent 例外" or "施加者死亡不移除別人身上的狀態" }
        ::continue::
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
        -- put on again by the same source halfway through (rank 1; line D judges it, casting the
        -- entry again, so only a status its cast applies unconditionally): refresh, keep,
        -- independent or a counted stack (`{ max }` alone); replace and extend look like refresh
        -- from outside, and a status without a duration has no clock to tell them by
        local st = s.stacking
        local how = (st == "refresh" or st == "keep" or st == "independent") and st
          or (type(st) == "table" and st.max ~= nil and st.mode == nil and st.per_stack_duration == nil and st.decay == nil
            and st.initial == nil) and "stack" or nil
        local by_cast = castable and kind ~= "passive" and type(a.path) == "string" and a.path:find("^events%.on_cast") ~= nil
          and not (a.conds and #a.conds > 0)
        -- line D cannot tell these apart (E644): a cast that also sets the stack count (it sets
        -- the same count again: C06W), a status on a unit the cast creates (the second cast makes
        -- another unit), and a duration within twice the recast (cast point + 0.5 s)
        local function smallest(v)
          if type(v) == "number" then return v end
          local name = type(v) == "string" and v:match("^%$([%w_]+)$")
          local x = name and def.values and def.values[name]
          if type(x) == "number" then return x end
          if type(x) ~= "table" or #x == 0 then return nil end
          local m
          for _, n in ipairs(x) do if type(n) ~= "number" then return nil end; m = m and math.min(m, n) or n end
          return m
        end
        local made = {}
        for _, x in ipairs(base_effects) do
          if x.op == "create_unit" and x.args and type(x.args.as) == "string" then made[x.args.as] = true end
          if x.op == "set_stacks" and x.phase == "on_cast" and x.args and x.args.status == s.id then by_cast = false end
        end
        if made[a.to] then by_cast = false end
        local lasts, point = smallest(s.duration), cast.cast_point == nil and 0 or smallest(cast.cast_point)
        if lasts and point and lasts <= 2 * (point + 0.5) then by_cast = false end
        if rank == 1 and how and by_cast and s.duration ~= nil and s.duration ~= "infinite" and not s.block_reapply then
          push(rank, "reapply", { suffix = s.id, given = given({ on_unit(s, a) }),
            when = { { kind = "reapply", status = s.id, after = "half" } },
            expect = { { kind = "reapplied", status = s.id, runtime_id = runtime_key(s), stacking = how,
              max = type(st) == "table" and st.max or nil } } })
        end
        for _, ev in ipairs(sorted_keys(s.triggers or {})) do
          push(rank, "trigger", { suffix = s.id .. "/" .. ev, given = given({ on_unit(s, a) }),
            -- interval: a periodic trigger's period (E220), so a test can time its ticks; duration:
            -- how long the status lasts where it is applied, which bounds its hits (E415)
            when = { { kind = "status_event", status = s.id, event = ev, interval = type(s.triggers[ev]) == "table" and s.triggers[ev].interval or nil,
              duration = a.duration or s.duration } },
            expect = { { kind = "effects", effects = trigger_effects(s.triggers[ev], "statuses." .. s.id .. ".triggers." .. ev, ev) } } })
        end
        -- The status's own stats on its holder (E388).
        if type(s.stats) == "table" and next(s.stats) then
          push(rank, "normal", { suffix = s.id .. "/stats", given = given({ on_unit(s, a) }),
            when = { { kind = "observe", from = "stats" } },
            expect = { { kind = "status_stats", status = s.id, stats = s.stats,
              value_stacks = type(s.stacking) == "table" and s.stacking.value_stacks or nil,
              max_stacks = type(s.stacking) == "table" and s.stacking.max or nil } } })
        end
        -- One instance per source (E359): a per-source status that another status's holders apply
        -- from themselves (for_each from self) stacks once per holder that reaches the unit.
        if s.per_source then
          local function applies_from_self(node, radius)
            if type(node) ~= "table" then return nil end
            if node.op == "for_each" and type(node.select) == "table" and node.select.from == "self" and node.select.radius then
              radius = node.select.radius
            end
            if node.op == "apply_status" and node.status == s.id and node.source == "self" and radius then return radius end
            for _, child in pairs(node) do
              local found = applies_from_self(child, radius)
              if found then return found end
            end
            return nil
          end
          for _, h in ipairs(def.statuses or {}) do
            local radius = h.id ~= s.id and applies_from_self(h)
            if radius then
              push(rank, "normal", { suffix = s.id .. "/per_source", given = given({ on_unit(s, a) }),
                when = { { kind = "observe_sources", holder = runtime_key(h), radius = radius } },
                expect = { { kind = "instances_per_source", status = s.id } } })
              break
            end
          end
        end
        -- The applying action's own duration wins over the status's default (C21R; E391).
        local lasts = a.duration or s.duration
        if lasts and lasts ~= "infinite" then
          -- from_passive (E483): the entry's passive applies it again and again.
          push(rank, "normal", { suffix = s.id .. "/expires", given = given({ on_unit(s, a) }),
            when = { { kind = "wait", seconds = lasts, from_passive = type(a.path) == "string" and a.path:find("^passive") ~= nil or nil } },
            expect = { { kind = "status_removed", status = s.id } } })
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
  -- rank 1 with the facts every case of the entry starts from (an ability swapped in or one
  -- needing another's status: A12T's a12_awe_t; the unit owning it; the cast-point guards)
  local function rank1_given(fact)
    local g = { { kind = "learned", entry = def.id, rank = 1 } }
    if swapped then g[#g + 1] = swapped end
    if unit_owned then g[#g + 1] = unit_owned end
    for _, need in ipairs(phase_needs) do g[#g + 1] = need end
    g[#g + 1] = fact
    return g
  end
  -- buildings in a selector's area (line D judges it): one scenario per on_cast for_each selector
  -- reaching enemies whose spot line D can place a building at (a circle from the caster, the
  -- point or the target, no `where`, unconditional). Whether the building is affected is the
  -- selector's types (building or all); `ops` are that selector's own unconditional damage and
  -- apply_status, the trace line D looks for (rank 1: the types do not change with the rank)
  if castable then
    for _, e in ipairs(base_effects) do
      local sel = e.op == "for_each" and e.phase == "on_cast" and not e.conds and e.args and e.args.select
      local from = sel and (sel.from or "caster")
      if sel and (sel.shape or "circle") == "circle" and sel.where == nil and sel.team ~= "ally" and sel.team ~= "ally_not_self"
        and (from == "caster" or from == "self" or from == "point" or from == "target") then
        local ops, seen = {}, {}
        for _, c in ipairs(base_effects) do
          if not c.conds and (c.op == "damage" or c.op == "apply_status") and c.path:match("^" .. e.path:gsub("%p", "%%%0") .. "%.actions%[%d+%]$")
            and not seen[c.op] then
            seen[c.op] = true; ops[#ops + 1] = c.op
          end
        end
        if #ops > 0 then
          local affected = false
          for _, ty in ipairs(type(sel.types) == "table" and sel.types or {}) do if ty == "building" or ty == "all" then affected = true end end
          push(1, "building_in_effect", { suffix = e.path, given = rank1_given({ kind = "building_in_effect", path = e.path, team = "enemy" }), when = { cast_step },
            expect = { { kind = "building_affected", path = e.path, affected = affected, ops = ops } },
            rule = "selector types decide whether a building in the area is picked" })
          -- an enemy the caster's team cannot see in the same area (invisible, or in the fog; line
          -- D places it and a visible enemy that shows the cast landed): picked unless the
          -- selector has visible_only (and, for the invisible one, exclude_invisible)
          local flags = {}
          for _, f in ipairs(type(sel.flags) == "table" and sel.flags or {}) do flags[f] = true end
          for _, state in ipairs({ "invisible", "fogged" }) do
            local picked = not flags.visible_only and not (state == "invisible" and flags.exclude_invisible)
            push(1, "hidden_in_effect", { suffix = state .. "/" .. e.path, given = rank1_given({ kind = "hidden_in_effect", path = e.path, team = "enemy", state = state }), when = { cast_step },
              expect = { { kind = "hidden_affected", path = e.path, affected = picked, ops = ops, state = state } },
              rule = "selector flags visible_only / exclude_invisible decide whether an enemy the caster's team cannot see is picked" })
          end
          -- an enemy hero's illusion in the same area, for a selector reaching heroes (line D
          -- places it and a real enemy hero that shows the cast landed): picked with
          -- include_illusions, never with exclude_illusions; without either an illusion is no
          -- hero but a summon (ability_kit Select), so then the types decide
          local types = {}
          for _, ty in ipairs(type(sel.types) == "table" and sel.types or { "all" }) do types[ty] = true end
          if types.hero or types.all then
            local picked = not flags.exclude_illusions and (flags.include_illusions or types.summon or types.all or false)
            push(1, "illusion_in_effect", { suffix = e.path,
              given = rank1_given({ kind = "illusion_in_effect", path = e.path, team = "enemy" }), when = { cast_step },
              expect = { { kind = "illusion_affected", path = e.path, affected = picked, ops = ops } },
              rule = "selector flags include_illusions / exclude_illusions (else its summon type) decide whether an enemy hero's illusion is picked" })
          end
        end
      end
    end
  end
  -- an enemy the caster's team cannot see on a linear flight's path (line D judges it): one pair
  -- per on_cast linear projectile from the caster with a distance, flying at enemies, and
  -- unconditional damage / apply_status right under its on_hit. Line D puts the unseen enemy at a
  -- third of `distance` and a visible one at two thirds; `pierce` says whether the flight goes on
  -- after a hit. Hit unless the projectile has visible_only, which no package writes today: these
  -- check that a flight hits unseen units by default
  if castable then
    for _, e in ipairs(base_effects) do
      local a = e.op == "projectile" and e.phase == "on_cast" and not e.conds and e.args
      if a and a.kind == "linear" and (a.from == nil or a.from == "caster" or a.from == "self") and a.distance ~= nil
        and a.where == nil and (a.team == nil or a.team == "enemy") and a.max_hits == nil then
        local ops, seen = {}, {}
        for _, c in ipairs(base_effects) do
          -- the on_hit body carries one condition, `projectile.on_hit` (walk_program)
          local only_hit = c.conds and #c.conds == 1 and c.conds[1] == "projectile.on_hit"
          if only_hit and (c.op == "damage" or c.op == "apply_status") and c.path:match("^" .. e.path:gsub("%p", "%%%0") .. "%.on_hit%[%d+%]$")
            and not seen[c.op] then
            seen[c.op] = true; ops[#ops + 1] = c.op
          end
        end
        if #ops > 0 then
          local visible = false
          for _, f in ipairs(type(a.flags) == "table" and a.flags or {}) do if f == "visible_only" then visible = true end end
          for _, state in ipairs({ "invisible", "fogged" }) do
            push(1, "hidden_in_flight", { suffix = state .. "/" .. e.path,
              given = rank1_given({ kind = "hidden_in_flight", path = e.path, team = "enemy", state = state,
                distance = a.distance, pierce = a.pierce == true, types = a.types }),
              when = { cast_step },
              expect = { { kind = "hidden_hit", path = e.path, affected = not visible, ops = ops, state = state } },
              rule = "a linear flight hits an enemy its caster's team cannot see unless it has visible_only (none does"
                .. " today: this checks the default)" })
          end
        end
      end
    end
  end
  -- one scenario per quirk (rank 1; the quirk text says when it applies). A quirk about a topic the
  -- entry's own checkable scenarios test (its selectors' magic-immune flags: the magic_immune case)
  -- also carries `covered_by` those scenarios: it holds when they all pass (line D judges it)
  local own = {}
  for _, s in ipairs(out) do
    if s.entry == def.id then own[s.case] = own[s.case] or {}; own[s.case][#own[s.case] + 1] = s end
  end
  for i, q in ipairs(def.quirks or {}) do
    local expect = { { kind = "quirk_policy", policy = q.policy, text = q.text, ref = q.ref } }
    for _, t in ipairs(M.QUIRK_TOPICS) do
      local mine = own[t.case]
      local claim
      if q.policy ~= "fixed" and mine and type(q.text) == "string" and (t.word == "" or q.text:find(t.word, 1, true)) then claim = t.claim(q.text) end
      local points = claim ~= nil and t.points(castable, target, base_effects, mine, q.text) or {}
      if #points == 1 and points[1].value == claim then
        local ids = {}
        for _, s in ipairs(points[1].scenarios or mine) do ids[#ids + 1] = s.id end
        if #ids > 0 then
          expect[#expect + 1] = { kind = "covered_by", topic = t.topic, entry = def.id, cases = { t.case }, scenarios = ids,
            path = points[1].path, flag = points[1].flag, value = points[1].value }
        end
      end
    end
    push(1, "quirk", { suffix = tostring(i), given = { { kind = "learned", entry = def.id, rank = 1 } }, when = {},
      expect = expect, quirk = { policy = q.policy, ref = q.ref } })
  end
end

-- doc -> { schema, id, kind, scenarios }
function M.build(doc, opts)
  opts = opts or {}
  current_doc = doc
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
