-- Usage statistics of the package schema v3 vocabulary over every document (M4 deliverable,
-- history-nobu full port): how often each action, state, StatKey, selector feature, expression
-- function, event, status / passive / cost / template feature and rules-layer name is used, by how
-- many heroes and items; and the executor build order, sorted by heroes unlocked: after each
-- component, how many heroes have every ability expressible as data.
--
--   tools\lua\lua.exe docs\character_pipeline\tools\v3_usage.lua [--out FILE.md] [--json FILE.json]
--
-- Default output: docs/plans/2026-10-28-history-nobu-full-port/usage-stats.md.
-- Components are named `<group>:<name>` (action:damage, fn:distance, event:on_attack_hit,
-- state:stun, stat:armor, select:shape:cone, status:absorb, ...). An entry needs the components
-- of its own tables plus those of every doc-level routine, unit template, status or ability it
-- names (followed transitively). A hero needs the union over its document; heroes with a
-- `pending` or `needs_hook` entry are listed apart (they cannot be unlocked by components alone).
local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
local M = {}

local PROSE = { text = true, notes = true, note = true, quirks = true, provenance = true, source = true,
  source_differs = true, ref = true, why = true, test = true, pending = true, confidence = true, kv = true,
  handler = true, v2_key = true, coverage = true, deferred_reason = true, cue = true, id = true, source_id = true,
  name = true, slot = true, kind = true, tags = true, values = true }

local function sorted_keys(t)
  local ks = {}
  for k in pairs(t) do ks[#ks + 1] = k end
  table.sort(ks, function(a, b) return tostring(a) < tostring(b) end)
  return ks
end

local function modules()
  local prev = rawget(_G, "PACKAGE_V3_AS_MODULE")
  PACKAGE_V3_AS_MODULE = true
  local P = dofile(tools_dir .. "/package_v3.lua")
  PACKAGE_V3_AS_MODULE = prev
  return P, P.vocab, P.expr, dofile(tools_dir .. "/v3_sources.lua")
end

-- the components one table tree uses; refs collects id-like strings (for doc-level definitions)
local function scan(V, X, t, comps, refs, key, parent)
  local function add(c) comps[c] = (comps[c] or 0) + 1 end
  if type(t) == "string" then
    refs[t] = true
    if key and PROSE[key] then return end
    local ok, ast = pcall(X.parse, t)
    if ok and type(ast) == "table" and ast.k then
      X.walk(ast, function(n)
        if n.k == "call" and V.FUNCTIONS[n.fn] then add("fn:" .. n.fn) end
        if n.k == "field" then
          local parts, m = {}, n
          while m.k == "field" do table.insert(parts, 1, m.name); m = m.obj end
          if m.k == "id" and m.name == "rules" then add("rules:" .. table.concat(parts, ".")) end
        end
      end)
    end
    return
  end
  if type(t) ~= "table" then return end
  if type(t.op) == "string" and V.ACTIONS[t.op] then
    add("action:" .. t.op)
    if t.op == "damage" then
      if type(t.type) == "string" then add("damage:" .. t.type) end
      for _, f in ipairs(t.flags or {}) do add("damage_flag:" .. f) end
      if t.require_source_alive == false then add("rules:require_source_alive=false") end
    end
  end
  if type(t.shape) == "string" and V.SHAPES[t.shape] then
    add("select:shape:" .. t.shape)
    for _, f in ipairs(t.flags or {}) do add("select:flag:" .. f) end
    if t.order then add("select:order:" .. tostring(t.order)) end
    for _, f in ipairs({ "where", "prefer", "post_filter", "take_while", "limit", "exclude", "visible_to", "relative_to", "exclude_types" }) do
      if t[f] ~= nil then add("select:" .. f) end
    end
  end
  -- statuses: a table with dispel + stacking (v3 status)
  if t.dispel and t.stacking and t.visibility then
    add("status:stacking:" .. tostring(type(t.stacking) == "table" and (t.stacking.mode or "stacks") or t.stacking))
    add("status:dispel:" .. tostring(t.dispel))
    for _, f in ipairs({ "absorb", "aura", "charges", "immune_to", "vars", "forced_move", "linger", "remove_on",
      "unique_group", "per_source", "reapply_if_removed", "block_reapply", "status_resistance", "unique_rule" }) do
      if t[f] ~= nil then add("status:" .. f) end
    end
    if t.scope then add("status:scope:" .. tostring(t.scope)) end
    if type(t.stacking) == "table" then add("status:stacks") end
  end
  for _, k in ipairs(sorted_keys(t)) do
    local v = t[k]
    if k == "states" and type(v) == "table" then
      for _, s in ipairs(v) do if V.STATES[s] then add("state:" .. s) end end
      for s, on in pairs(v) do if type(s) == "string" and V.STATES[s] and on then add("state:" .. s) end end
    elseif k == "stats" and type(v) == "table" then
      for s in pairs(v) do if type(s) == "string" and V.STATS[s] then add("stat:" .. s) end end
    elseif (k == "events" or k == "triggers") and type(v) == "table" then
      for ev in pairs(v) do if type(ev) == "string" then add("event:" .. ev) end end
    elseif k == "flags" and type(v) == "table" and parent == "entry" then
      for _, f in ipairs(v) do add("flag:" .. f) end
    elseif k == "passive" and type(v) == "table" then
      for _, f in ipairs({ "aura", "orb_slot", "unique_group", "triggers" }) do if v[f] ~= nil then add("passive:" .. f) end end
    elseif k == "cost" and type(v) == "table" then
      for _, f in ipairs({ "charges", "cooldown_groups", "health", "gold", "refund" }) do if v[f] ~= nil then add("cost:" .. f) end end
      if v.cooldown_start and v.cooldown_start ~= "on_cast" then add("cost:cooldown_start:" .. v.cooldown_start) end
    elseif k == "cast" and type(v) == "table" then
      if v.channel_time then add("cast:channel") end
    elseif k == "unit_state" and type(v) == "table" then
      for _, d in pairs(v) do if type(d) == "table" and d.scope then add("unit_state:scope:" .. d.scope) end end
    end
    if not (PROSE[k] and k ~= "values") then scan(V, X, v, comps, refs, k, (k == "abilities" or k == "items") and "list" or (parent == "list" and "entry" or nil)) end
  end
end

-- doc -> { units = { { kind, id, name, comps = set, blocked = reason? } } }
local function units_of(P, V, X, doc)
  local defs = {}
  for _, k in ipairs({ "unit_templates", "statuses", "abilities" }) do
    for _, d in ipairs(doc[k] or {}) do if d.id then defs[d.id] = { kind = k, def = d } end end
  end
  for name, r in pairs(doc.routines or {}) do defs[name] = { kind = "routines", def = r } end
  if type(doc.unit_state) == "table" then defs["$unit_state"] = { kind = "unit_state", def = { unit_state = doc.unit_state } } end
  local cache = {}
  local function comps_of(def, seen)
    local comps, refs = {}, {}
    scan(V, X, { def }, comps, refs, nil, "list")
    for r in pairs(refs) do
      local d = defs[r]
      if d and not seen[r] then
        seen[r] = true
        if d.kind == "unit_templates" then comps["template:" .. "create"] = (comps["template:create"] or 0) + 1
          for _, f in ipairs({ "chase", "attack", "ai", "inventory_locked", "abilities", "illusion" }) do
            if d.def[f] ~= nil then comps["template:" .. f] = (comps["template:" .. f] or 0) + 1 end
          end
        end
        for c, n in pairs(comps_of(d.def, seen)) do comps[c] = (comps[c] or 0) + n end
      end
    end
    return comps
  end
  local function blocked(list)
    for _, e in ipairs(list) do
      if e.coverage == "pending" then return "pending" end
      if e.coverage == "needs_hook" then return "needs_hook" end
    end
  end
  local units = {}
  if doc.kind == "hero" then
    local comps = {}
    local entries = {}
    for _, a in ipairs(doc.abilities or {}) do
      if a.coverage ~= "deferred" then
        entries[#entries + 1] = a
        local seen = {}
        for c, n in pairs(comps_of(a, seen)) do comps[c] = (comps[c] or 0) + n end
      end
    end
    if doc.unit_state then for c, n in pairs(comps_of({ unit_state = doc.unit_state }, {})) do comps[c] = (comps[c] or 0) + n end end
    local name = doc.hero and doc.hero.text and doc.hero.text.zh_TW and doc.hero.text.zh_TW.name
    units[1] = { kind = "hero", id = doc.id, name = name, comps = comps, blocked = blocked(entries) }
  else
    for _, it in ipairs(doc.items or {}) do
      if it.coverage ~= "deferred" then
        local name = it.text and it.text.zh_TW and it.text.zh_TW.name
        units[#units + 1] = { kind = "item", id = it.id, name = name, comps = comps_of(it, {}), blocked = blocked({ it }) }
      end
    end
  end
  return units
end

-- the components B01 and C17 need: M3's first batch
M.FIRST_BATCH = { "B01", "C17" }

function M.build(root)
  local P, V, X, S = modules()
  local units = {}
  for _, f in ipairs(S.all(root)) do
    local doc = assert(P.load_file(f))
    for _, u in ipairs(units_of(P, V, X, doc)) do units[#units + 1] = u end
  end
  -- per component totals
  local comp = {}
  for _, u in ipairs(units) do
    for c, n in pairs(u.comps) do
      local row = comp[c] or { name = c, uses = 0, heroes = 0, items = 0 }
      comp[c] = row
      row.uses = row.uses + n
      if u.kind == "hero" then row.heroes = row.heroes + 1 else row.items = row.items + 1 end
    end
  end
  -- build order
  local have = {}
  local heroes, items = {}, {}
  for _, u in ipairs(units) do if u.kind == "hero" then heroes[#heroes + 1] = u else items[#items + 1] = u end end
  local function missing(u)
    local m = {}
    for c in pairs(u.comps) do if not have[c] then m[#m + 1] = c end end
    table.sort(m, function(a, b) if comp[a].heroes ~= comp[b].heroes then return comp[a].heroes > comp[b].heroes end return a < b end)
    return m
  end
  local function unlocked(list)
    local out = {}
    for _, u in ipairs(list) do if not u.blocked and #missing(u) == 0 then out[#out + 1] = u end end
    return out
  end
  local order, done = {}, {}
  local function take(c, why)
    if have[c] then return end
    have[c] = true
    local uh, ui = unlocked(heroes), unlocked(items)
    local new = {}
    for _, u in ipairs(uh) do if not done[u.id] then done[u.id] = true; new[#new + 1] = u.id end end
    for _, u in ipairs(ui) do if not done[u.id] then done[u.id] = true end end
    order[#order + 1] = { component = c, step = why, heroes = #uh, items = #ui, new_heroes = new,
      uses_heroes = comp[c].heroes, uses_items = comp[c].items }
  end
  local by_id = {}
  for _, u in ipairs(heroes) do by_id[u.id] = u end
  for _, id in ipairs(M.FIRST_BATCH) do
    for _, c in ipairs(missing(by_id[id])) do take(c, "M3 第一批（" .. table.concat(M.FIRST_BATCH, "、") .. "）") end
  end
  while true do
    local best, best_score, best_cost
    for _, u in ipairs(heroes) do
      if not u.blocked and not done[u.id] then
        local m = missing(u)
        if #m > 0 then
          local trial = {}
          for _, c in ipairs(m) do trial[c] = true end
          local gain = 0
          for _, v in ipairs(heroes) do
            if not v.blocked and not done[v.id] then
              local ok = true
              for c in pairs(v.comps) do if not have[c] and not trial[c] then ok = false; break end end
              if ok then gain = gain + 1 end
            end
          end
          local score = gain / #m
          if not best or score > best_score or (score == best_score and #m < best_cost) or (score == best_score and #m == best_cost and u.id < best.id) then
            best, best_score, best_cost = u, score, #m
          end
        end
      end
    end
    if not best then break end
    for _, c in ipairs(missing(best)) do take(c, best.id .. (best.name and ("（" .. best.name .. "）") or "")) end
  end
  -- what only items still need
  local rest = {}
  for c in pairs(comp) do if not have[c] then rest[#rest + 1] = c end end
  table.sort(rest, function(a, b) if comp[a].items ~= comp[b].items then return comp[a].items > comp[b].items end return a < b end)
  for _, c in ipairs(rest) do take(c, "道具") end
  local blocked = {}
  for _, u in ipairs(units) do if u.blocked then blocked[#blocked + 1] = u end end
  return { components = comp, order = order, units = units, blocked = blocked, heroes = #heroes, items = #items }
end

local GROUP_TITLES = {
  { "action", "Action" }, { "fn", "運算式函式" }, { "event", "事件（技能 events、狀態與被動 triggers）" },
  { "state", "狀態旗標（states）" }, { "stat", "StatKey（stats）" }, { "select", "Selector 形狀與過濾" },
  { "status", "狀態功能" }, { "passive", "被動功能" }, { "cost", "成本功能" }, { "cast", "施放功能" },
  { "template", "單位模板" }, { "unit_state", "unit_state 範圍" }, { "damage", "傷害類型" },
  { "damage_flag", "傷害旗標" }, { "flag", "技能旗標（規則層）" }, { "rules", "規則層數值與規則" },
}

function M.render(r)
  local out = {}
  local function w(s) out[#out + 1] = s end
  w("# v3 詞彙使用頻率與執行器實作順序")
  w("")
  w("由 `tools\\lua\\lua.exe docs\\character_pipeline\\tools\\v3_usage.lua` 從全部 v3 文件產生（`v3_sources.lua`：" ..
    r.heroes .. " 位英雄、" .. r.items .. " 件道具；deferred 條目不計），不要手改。")
  w("")
  w("- **使用次數**：文件中出現的次數。")
  w("- **英雄／道具數**：用到的英雄與道具數。條目引用的共用 routine、單位模板與狀態也算在條目上。")
  w("- 英雄以整份文件為單位，道具以單件為單位。")
  w("")
  w("## 1. 依解鎖英雄數排序的執行器元件清單")
  w("")
  w("從 M3 第一批（雜賀孫市、阿市需要的元件）開始。之後每一步選「每多實作一個元件、能解鎖最多英雄」的那位英雄，補齊他缺的元件，元件依使用英雄數排序。")
  w("「累計英雄」與「累計道具」是實作到這一列為止，所有技能（道具則是整件）都能用資料執行的數量。只剩道具使用的元件排在最後。")
  w("")
  local last = r.order[#r.order]
  local marks = { 2, 10, 25, 50, 75 }
  if last then marks[#marks + 1] = last.heroes end
  w("| 目標英雄數 | 需要實作的元件數 | 同時解鎖的道具 |")
  w("|---|---|---|")
  for _, target in ipairs(marks) do
    for i, o in ipairs(r.order) do
      if o.heroes >= target then w(("| %d | %d | %d |"):format(target, i, o.items)); break end
    end
  end
  if last then w(("| 全部道具（%d） | %d | %d |"):format(last.items, #r.order, last.items)) end
  w("")
  w("| # | 元件 | 為了 | 用到的英雄 | 用到的道具 | 累計英雄 | 累計道具 | 這一步解鎖 |")
  w("|---|---|---|---|---|---|---|---|")
  for i, o in ipairs(r.order) do
    w(("| %d | `%s` | %s | %d | %d | %d | %d | %s |"):format(i, o.component, o.step, o.uses_heroes, o.uses_items, o.heroes, o.items,
      table.concat(o.new_heroes, " ")))
  end
  w("")
  if #r.blocked > 0 then
    w("### 要先解決 pending／needs_hook 的條目")
    w("")
    w("這些條目還有待確認的項目，補齊元件也不會解鎖；上表不計入。")
    w("")
    for _, u in ipairs(r.blocked) do w(("- %s `%s`%s：%s"):format(u.kind == "hero" and "英雄" or "道具", u.id, u.name and ("（" .. u.name .. "）") or "", u.blocked)) end
    w("")
  end
  w("## 2. 各類使用次數")
  w("")
  local by_group = {}
  for name, row in pairs(r.components) do
    local g = name:match("^([^:]+):")
    by_group[g] = by_group[g] or {}
    table.insert(by_group[g], row)
  end
  for i, gt in ipairs(GROUP_TITLES) do
    local rows = by_group[gt[1]]
    if rows then
      table.sort(rows, function(a, b)
        if a.heroes + a.items ~= b.heroes + b.items then return a.heroes + a.items > b.heroes + b.items end
        if a.uses ~= b.uses then return a.uses > b.uses end
        return a.name < b.name end)
      w(("### 2.%d %s（%d 種）"):format(i, gt[2], #rows))
      w("")
      w("| 名稱 | 使用次數 | 英雄 | 道具 |")
      w("|---|---|---|---|")
      for _, row in ipairs(rows) do w(("| `%s` | %d | %d | %d |"):format(row.name:sub(#gt[1] + 2), row.uses, row.heroes, row.items)) end
      w("")
    end
  end
  return table.concat(out, "\n")
end

function M.main(argv)
  local root = tools_dir .. "/../../.."
  package.path = root .. "/scripts/?.lua;scripts/?.lua;" .. package.path
  local b = require("_bootstrap")
  local out = b.root .. "/docs/plans/2026-10-28-history-nobu-full-port/usage-stats.md"
  local json_out
  local i = 1
  while i <= #argv do
    if argv[i] == "--out" then out = argv[i + 1]; i = i + 1
    elseif argv[i] == "--json" then json_out = argv[i + 1]; i = i + 1
    else error("unknown argument " .. argv[i]) end
    i = i + 1
  end
  local r = M.build(b.root)
  local f = assert(io.open(out, "wb")); f:write(M.render(r), "\n"); f:close()
  if json_out then
    local rows = {}
    for _, row in pairs(r.components) do rows[#rows + 1] = row end
    table.sort(rows, function(a, c) return a.name < c.name end)
    local j = assert(io.open(json_out, "wb")); j:write(b.lib("json").encode({ components = rows, order = r.order })); j:close()
  end
  local last = r.order[#r.order]
  print(("wrote %s: %d components, %d steps, all %d heroes / %d items at the end (%d blocked)"):format(out,
    (function() local n = 0 for _ in pairs(r.components) do n = n + 1 end return n end)(), #r.order,
    last and last.heroes or 0, last and last.items or 0, #r.blocked))
  return 0
end

if rawget(_G, "V3_USAGE_AS_MODULE") then return M end
os.exit(M.main(arg))
