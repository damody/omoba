-- Package schema v3 validator (docs/character_pipeline/schema/v3.md).
--
-- A v3 document is one Lua file returning function(ctx) -> table with
-- schema_version = 3 (a hero kit or an item set). The validator checks structure
-- and types, parses every expression and resolves every name, and runs the
-- single-source checks of value-single-source §5.
--
--   lua package_v3.lua <file.lua|package_dir> ...   validate (package_dir/v3.lua)
--   lua package_v3.lua --vocab-md                   print the vocabulary as Markdown
--
-- As a module: validate_doc(doc, opts) / validate_file(path, opts) -> errors, warnings.
local tools_dir = (debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".")
local V = dofile(tools_dir .. "/v3_vocab.lua")
local X = dofile(tools_dir .. "/v3_expr.lua")
local BOT = dofile(tools_dir .. "/v3_bot.lua")
local M = { vocab = V, expr = X, bot = BOT }

local function is_array(t)
  if type(t) ~= "table" then return false end
  local n = 0
  for k in pairs(t) do
    if math.type(k) ~= "integer" or k < 1 then return false end
    n = n + 1
  end
  return n == #t
end
local function snake(v) return type(v) == "string" and v:match("^[a-z][a-z0-9_]*$") ~= nil and not v:find("__", 1, true) end
local function sorted_keys(t)
  local ks = {}
  for k in pairs(t) do ks[#ks + 1] = k end
  table.sort(ks, function(a, b) return tostring(a) < tostring(b) end)
  return ks
end

local rules_cache
local function load_rules(opts)
  if opts.rules then return opts.rules end
  if rules_cache then return rules_cache end
  local path = tools_dir .. "/../../../scripts/lua_data/templates/game_rules.lua"
  local chunk = loadfile(path, "t", {})
  local rules = {}
  if chunk then
    local ok, fn = pcall(chunk)
    if ok and type(fn) == "function" then
      local ok2, t = pcall(fn, {})
      if ok2 and type(t) == "table" then rules = t end
    end
  end
  local function merge(dst, src)
    for k, v in pairs(src) do
      if type(v) == "table" then dst[k] = type(dst[k]) == "table" and dst[k] or {}; merge(dst[k], v)
      elseif dst[k] == nil then dst[k] = v end
    end
  end
  merge(rules, V.RULES_PENDING)
  rules_cache = rules
  return rules
end

function M.rules(opts) return load_rules(opts or {}) end

-- docs/character_pipeline/schema/v3-shared-registry.lua (v3.2): cross-document names
local registry_cache
local function load_registry(opts)
  if opts.registry ~= nil then return opts.registry or nil end
  if registry_cache == nil then
    local chunk = loadfile(tools_dir .. "/../schema/v3-shared-registry.lua", "t", {})
    local ok, t = false, nil
    if chunk then ok, t = pcall(chunk) end
    registry_cache = ok and type(t) == "table" and t or false
  end
  return registry_cache or nil
end
M.registry = load_registry

function M.validate_doc(doc, opts)
  opts = opts or {}
  local errors, warnings = {}, {}
  local function err(path, msg) errors[#errors + 1] = "SchemaError: " .. path .. ": " .. msg end
  local function warn(path, msg) warnings[#warnings + 1] = "GameContractWarning: " .. path .. ": " .. msg end
  local rules = load_rules(opts)

  local function fields(t, allowed, path)
    if type(t) ~= "table" then err(path, "必須是 table"); return false end
    for _, k in ipairs(sorted_keys(t)) do
      if not allowed[k] then err(path, "不認得的欄位 `" .. tostring(k) .. "`") end
    end
    return true
  end
  local function need_string(v, path)
    if type(v) ~= "string" or not v:find("%S") then err(path, "必須是非空字串"); return false end
    return true
  end
  local function enum(v, allowed, path, required)
    if v == nil then if required then err(path, "必填") end return end
    if not allowed[v] then
      local opts_list = sorted_keys(allowed)
      err(path, "`" .. tostring(v) .. "` 不是允許的值（" .. table.concat(opts_list, "、") .. "）")
    end
  end

  if type(doc) ~= "table" then err("document", "必須回傳 table"); return errors, warnings end
  fields(doc, V.DOC_FIELDS, "document")
  if doc.schema_version ~= V.SCHEMA_VERSION then err("schema_version", "必須是 3") end
  enum(doc.kind, V.DOC_KINDS, "kind", true)
  need_string(doc.id, "id")
  if fields(doc.source or {}, V.SOURCE_FIELDS, "source") then
    for _, k in ipairs({ "repository", "commit", "root" }) do need_string((doc.source or {})[k], "source." .. k) end
  end

  -------------------------------------------------------------------------
  -- declarations
  -------------------------------------------------------------------------
  local owners = {}        -- id -> owner { id, kind, values, max_level, path, parent }
  local statuses = {}      -- id -> { def, owner, path }
  local abilities, templates, routines, hooks = {}, {}, {}, {}
  local declared = {}      -- every id -> path (uniqueness)
  local function declare(id, path)
    if not snake(id) then err(path .. ".id", "必須是 snake_case ASCII：" .. tostring(id)); return false end
    if declared[id] then err(path .. ".id", "`" .. id .. "` 重複（先出現在 " .. declared[id] .. "）"); return false end
    declared[id] = path
    return true
  end
  local shared = { id = "shared", kind = "shared", values = doc.values or {}, max_level = 1, path = "values" }
  owners.shared = shared
  local function add_owner(o) owners[o.id] = o; return o end

  local function collect_statuses(list, owner, path)
    if list == nil then return end
    if not is_array(list) then err(path, "必須是陣列"); return end
    for i, s in ipairs(list) do
      local p = path .. "[" .. i .. "]" .. (type(s) == "table" and s.id and ("(" .. tostring(s.id) .. ")") or "")
      if type(s) ~= "table" then err(p, "必須是 table")
      elseif declare(s.id, p) then
        local so = add_owner({ id = s.id, kind = "status", values = s.values or {}, max_level = owner.max_level, path = p, parent = owner })
        local vs = {}
        if type(s.vars) == "table" then for k in pairs(s.vars) do vs[k] = "any" end end
        statuses[s.id] = { def = s, owner = so, path = p, vars = vs }
      end
    end
  end

  local entry_list = {}    -- { kind = "ability"|"item", def, owner, path }
  if doc.abilities ~= nil and not is_array(doc.abilities) then err("abilities", "必須是陣列") end
  for i, a in ipairs(type(doc.abilities) == "table" and doc.abilities or {}) do
    local p = "abilities[" .. i .. "]" .. (type(a) == "table" and a.id and ("(" .. tostring(a.id) .. ")") or "")
    if type(a) ~= "table" then err(p, "必須是 table")
    elseif declare(a.id, p) then
      local ml = a.max_level
      if math.type(ml) ~= "integer" or ml < 1 or ml > 25 then err(p .. ".max_level", "必須是 1..25 的整數"); ml = 1 end
      local o = add_owner({ id = a.id, kind = "ability", values = a.values or {}, max_level = ml, path = p, parent = shared, def = a })
      abilities[a.id] = o
      entry_list[#entry_list + 1] = { kind = "ability", def = a, owner = o, path = p }
      collect_statuses(a.statuses, o, p .. ".statuses")
    end
  end
  if doc.items ~= nil and not is_array(doc.items) then err("items", "必須是陣列") end
  for i, it in ipairs(type(doc.items) == "table" and doc.items or {}) do
    local p = "items[" .. i .. "]" .. (type(it) == "table" and it.id and ("(" .. tostring(it.id) .. ")") or "")
    if type(it) ~= "table" then err(p, "必須是 table")
    elseif declare(it.id, p) then
      local ml = it.max_level or 1
      if math.type(ml) ~= "integer" or ml < 1 then err(p .. ".max_level", "必須是正整數"); ml = 1 end
      local o = add_owner({ id = it.id, kind = "item", values = it.values or {}, max_level = ml, path = p, parent = shared, def = it })
      abilities[it.id] = o -- an item's active is cast like an ability
      entry_list[#entry_list + 1] = { kind = "item", def = it, owner = o, path = p }
      collect_statuses(it.statuses, o, p .. ".statuses")
    end
  end
  if doc.unit_templates ~= nil and not is_array(doc.unit_templates) then err("unit_templates", "必須是陣列") end
  for i, u in ipairs(type(doc.unit_templates) == "table" and doc.unit_templates or {}) do
    local p = "unit_templates[" .. i .. "]" .. (type(u) == "table" and u.id and ("(" .. tostring(u.id) .. ")") or "")
    if type(u) ~= "table" then err(p, "必須是 table")
    elseif declare(u.id, p) then
      local ml = u.max_level or 1
      if math.type(ml) ~= "integer" or ml < 1 then err(p .. ".max_level", "必須是正整數"); ml = 1 end
      templates[u.id] = add_owner({ id = u.id, kind = "template", values = u.values or {}, max_level = ml, path = p, parent = shared, def = u })
    end
  end
  collect_statuses(doc.statuses, shared, "statuses")
  for _, name in ipairs(sorted_keys(type(doc.routines) == "table" and doc.routines or {})) do
    local p = "routines." .. tostring(name)
    if declare(name, p) then routines[name] = doc.routines[name] end
  end
  for _, name in ipairs(sorted_keys(type(doc.hooks) == "table" and doc.hooks or {})) do
    local p = "hooks." .. tostring(name)
    if declare(name, p) then hooks[name] = { def = doc.hooks[name], used = false, path = p } end
  end

  -------------------------------------------------------------------------
  -- values
  -------------------------------------------------------------------------
  local uses, visible, in_text = {}, {}, {}  -- "owner.name" -> true
  local tag_reads = {}  -- status tags this document reads: { tag, path }
  local item_tag_uses = {}  -- item tags declared or read (v3.2): { tag, path }
  local function vkey(o, n) return o.id .. "." .. n end
  local function find_value(owner, qual, name)
    if qual then
      local o = owners[qual]
      if o and type(o.values) == "table" and o.values[name] ~= nil then return o.values[name], o end
      return nil
    end
    local o = owner
    while o do
      if type(o.values) == "table" and o.values[name] ~= nil then return o.values[name], o end
      o = o.parent
    end
    return nil
  end

  local function value_error_check_numbers(list, o, n, p)
    local is_count = (list.unit == "count") or list.integer
    for i, x in ipairs(list) do
      local ok = false
      if type(x) == "number" then
        ok = x == x and x ~= math.huge and x ~= -math.huge
        if ok and is_count and math.type(x) ~= "integer" and x % 1 ~= 0 then err(p .. "[" .. i .. "]", "count／integer 必須是整數") end
      elseif type(x) == "string" then
        ok = x:match("^%-?%d+%.?%d*$") ~= nil or x:match("^%-?%d+/%d+$") ~= nil
      end
      if not ok then err(p .. "[" .. i .. "]", "必須是有限數字或十進位／分數字串（例如 \"1900/101\"）") end
      -- pct values are percent numbers (25 = 25%): a fraction string below one is almost
      -- always a probability written as a ratio (19/101 instead of 1900/101).
      if ok and type(x) == "string" and list.unit == "pct" then
        local n, dd = x:match("^%-?(%d+)/(%d+)$")
        if n and tonumber(n) < tonumber(dd) then
          err(p .. "[" .. i .. "]", "pct 是百分比數字：" .. x .. " 代表 " .. string.format("%.3g", tonumber(n) / tonumber(dd)) .. "%；機率 " .. x .. " 要寫成 \"" .. tonumber(n) * 100 .. "/" .. dd .. "\"")
        end
      end
    end
  end

  local value_exprs = {} -- key -> { ast, owner }
  local function check_values(o)
    local vals = o.values
    if vals == nil then return end
    local p0 = o.path .. ".values"
    if type(vals) ~= "table" or (next(vals) ~= nil and is_array(vals)) then err(p0, "必須是 名稱 → 定義 的 table"); return end
    for _, n in ipairs(sorted_keys(vals)) do
      local d, p = vals[n], p0 .. "." .. tostring(n)
      if not snake(n) then err(p, "數值名稱必須是 snake_case") end
      if type(d) ~= "table" then err(p, "必須是 table，例如 { 1, 2, unit = \"s\", source = \"kv:...\" }")
      else
        for k in pairs(d) do
          if math.type(k) ~= "integer" and not V.VALUE_FIELDS[k] then err(p, "不認得的欄位 `" .. tostring(k) .. "`") end
        end
        enum(d.unit, V.UNITS, p .. ".unit", true)
        if d.hidden ~= nil and type(d.hidden) ~= "boolean" then err(p .. ".hidden", "必須是 boolean") end
        if d.damage_type ~= nil then enum(d.damage_type, V.DAMAGE_TYPES, p .. ".damage_type") end
        if d.per ~= nil then enum(d.per, V.set({ "rank", "hero_level" }), p .. ".per") end
        if d.rounding ~= nil then enum(d.rounding, V.ROUNDING, p .. ".rounding") end
        if d.expr ~= nil then
          if #d > 0 then err(p, "expr 與逐 rank 數值只能擇一") end
          local ast, perr = X.parse(d.expr)
          if not ast then err(p .. ".expr", "運算式錯誤：" .. perr)
          else value_exprs[vkey(o, n)] = { ast = ast, owner = o, path = p .. ".expr" } end
          if d.source ~= nil and not tostring(d.source):match("^%a+:") and d.source ~= "derived" then
            err(p .. ".source", "格式是 <前綴>:<出處>")
          end
        else
          if #d == 0 then err(p, "至少要有一個數值，或寫 expr")
          else
            local want = (d.per == "hero_level") and 25 or o.max_level
            if #d ~= 1 and #d ~= want then
              err(p, "數值個數必須是 1 或 " .. want .. "（" .. (d.per == "hero_level" and "英雄等級" or "max_level") .. "），現在是 " .. #d)
            end
            value_error_check_numbers(d, o, n, p)
          end
          if d.source == nil then err(p .. ".source", "必須寫出處（" .. table.concat(sorted_keys(V.SOURCE_PREFIXES), "、") .. "）")
          elseif type(d.source) ~= "string" then err(p .. ".source", "必須是字串")
          else
            local prefix, rest = d.source:match("^(%a+):(.*)$")
            if not prefix or not V.SOURCE_PREFIXES[prefix] then
              err(p .. ".source", "前綴必須是 " .. table.concat(sorted_keys(V.SOURCE_PREFIXES), "、") .. "，例如 kv:AbilityCooldown")
            elseif not rest:find("%S") then err(p .. ".source", "前綴後要寫出處")
            end
          end
        end
        if d.kv ~= nil and (type(d.source_differs) ~= "string" or not d.source_differs:find("%S")) then
          err(p .. ".source_differs", "寫了 kv（原作 KV 的不同數值）就必須說明為什麼不用它")
        end
        if d.source_differs ~= nil and type(d.source_differs) ~= "string" then err(p .. ".source_differs", "必須是字串") end
      end
    end
  end
  for _, id in ipairs(sorted_keys(owners)) do check_values(owners[id]) end

  -------------------------------------------------------------------------
  -- expression environment
  -------------------------------------------------------------------------
  local doc_state = {}
  local state_inits, init_owner, init_names = {}, shared, {}
  local function collect_state(t, path, into)
    if t == nil then return end
    if type(t) ~= "table" or is_array(t) and next(t) ~= nil then err(path, "必須是 名稱 → { type, reveal } 的 table"); return end
    for _, n in ipairs(sorted_keys(t)) do
      local d, p = t[n], path .. "." .. tostring(n)
      if not snake(n) then err(p, "名稱必須是 snake_case") end
      if fields(d, V.STATE_FIELDS, p) then
        enum(d.type, V.STATE_TYPES, p .. ".type", true)
        enum(d.reveal, V.REVEAL, p .. ".reveal", true)
        if d.scope ~= nil then enum(d.scope, V.STATE_SCOPES, p .. ".scope") end
        into[n] = ({ int = "num", fixed = "num", tick = "num", bool = "bool", unit = "unit", point = "point",
          vec = "vec", unit_list = "any", ability = "ability", item = "item", item_name = "any" })[d.type] or "any"
        if d.init ~= nil then state_inits[#state_inits + 1] = { init = d.init, path = p .. ".init", owner = init_owner, names = init_names } end
      end
    end
  end
  collect_state(doc.unit_state, "unit_state", doc_state)

  -- scope: { parent, names = {name -> kind} }
  local function new_scope(parent, names) return { parent = parent, names = names or {} } end
  local function lookup(scope, name)
    while scope do
      if scope.names[name] ~= nil then return scope.names[name] end
      scope = scope.parent
    end
    return nil
  end

  -- ctx: { owner, scope, event (param table)|nil, state (name->kind), vars (name->kind)|nil, path }
  local function expr_env(ctx, path)
    return {
      err = function(m) err(path, m) end,
      value = function(qual, name)
        local d, o = find_value(ctx.owner, qual, name)
        if d then uses[vkey(o, name)] = true; if ctx.info then visible[vkey(o, name)] = true end end
        return d
      end,
      root = function(name)
        local k = lookup(ctx.scope, name)
        if k then return k end
        if V.ROOTS_ALWAYS[name] then return V.ROOTS_ALWAYS[name] end
        if name == "event" and ctx.event then return "event" end
        if name == "vars" and ctx.vars then return "vars" end
        return nil
      end,
      status = function(id) return statuses[id] ~= nil end,
      ability = function(id) return abilities[id] ~= nil end,
      template = function(id) return templates[id] ~= nil end,
      status_tag = function(tag) tag_reads[#tag_reads + 1] = { tag = tag, path = path } end,
      hook = function(node)
        if node and node.k == "id" and hooks[node.name] then
          hooks[node.name].used = true
          if current_entry then hook_users[current_entry] = true end
        else err(path, "hook_value 的第一個參數要是 hooks 裡宣告的名稱") end
      end,
      state = function(n) return (ctx.state and ctx.state[n]) or doc_state[n] end,
      var = function(n) return ctx.vars and ctx.vars[n] end,
      event = ctx.event,
      rules = rules,
    }
  end
  local function check_expr(v, want, ctx, path)
    if type(v) == "number" then
      if not V.STRUCTURAL_NUMBERS[v] then
        err(path, "未命名的數字 " .. tostring(v) .. "：寫進 values 再用 \"$名稱\" 引用（只允許 0、1、-1）")
      end
      return "num"
    end
    if type(v) == "boolean" then
      if want ~= "bool" and want ~= "any" then err(path, "這裡需要運算式，不是 boolean") end
      return "bool"
    end
    if type(v) ~= "string" then err(path, "必須是運算式字串"); return "any" end
    local ast, perr = X.parse(v)
    if not ast then err(path, "運算式錯誤：" .. perr .. "（" .. v .. "）"); return "any" end
    local t = X.check(ast, expr_env(ctx, path))
    local ok = want == "any" or t == "any" or t == want or (want == "point" and t == "unit")
      or (want == "unit_or_point" and (t == "unit" or t == "point"))
    if not ok then err(path, "需要 " .. want .. "，這個運算式是 " .. t .. "（" .. v .. "）") end
    return t
  end

  -- value exprs: only values, rules, caster/ability/game props.
  for _, key in ipairs(sorted_keys(value_exprs)) do
    local ve = value_exprs[key]
    check_expr(ve.owner.values[key:match("%.([^.]+)$")].expr, "any", { owner = ve.owner, scope = new_scope(nil), state = {} }, ve.path)
  end
  -- cycles among derived values
  do
    local state = {}
    local function visit(key, trail)
      if state[key] == "done" then return end
      if state[key] == "active" then err(value_exprs[key].path, "衍生值循環引用：" .. table.concat(trail, " → ")); return end
      state[key] = "active"
      local ve = value_exprs[key]
      for _, r in ipairs(X.value_refs(ve.ast)) do
        local _, o = find_value(ve.owner, r.owner, r.name)
        if o then
          local k2 = vkey(o, r.name)
          if value_exprs[k2] then
            trail[#trail + 1] = k2
            visit(k2, trail)
            trail[#trail] = nil
          end
        end
      end
      state[key] = "done"
    end
    for _, key in ipairs(sorted_keys(value_exprs)) do visit(key, { key }) end
  end

  -------------------------------------------------------------------------
  -- behavior
  -------------------------------------------------------------------------
  local hook_users = {} -- entry path -> true when the entry uses a hook
  local function deep_has_hook(t, seen)
    seen = seen or {}
    if type(t) == "string" then return t:find("hook_value%s*%(") ~= nil end
    if type(t) ~= "table" or seen[t] then return false end
    seen[t] = true
    if t.op == "hook" then return true end
    for _, v in pairs(t) do if deep_has_hook(v, seen) then return true end end
    return false
  end
  local routine_hooks = {}
  for name, r in pairs(routines) do routine_hooks[name] = deep_has_hook(r) end
  local current_entry
  local check_actions, check_selector

  local BIND_KIND = { create_unit = "unit", create_illusion = "unit", projectile = "projectile", zone = "any",
    tether = "any", apply_status = "any", damage = "num", heal = "num", restore_mana = "num", burn_mana = "num",
    take_gold = "num", reveal = "any", give_item = "item", hook = "any", create_pickup = "any", create_tree = "any",
    dash = "any", attack = "any", random_pick = "any", pick_item = "item" }
  local KIND_WANT = { expr = "num", cond = "bool", unit = "unit", point = "point", vec = "vec", any = "any" }

  local function check_param(kind, v, ctx, path, action)
    if kind:sub(1, 5) == "list:" then
      if type(v) ~= "table" or not is_array(v) then err(path, "必須是陣列"); return end
      for i, x in ipairs(v) do check_param(kind:sub(6), x, ctx, path .. "[" .. i .. "]", action) end
      return
    end
    if kind:sub(1, 5) == "enum:" then
      local allowed = {}
      for w in kind:sub(6):gmatch("[^|]+") do allowed[w] = true end
      enum(v, allowed, path)
      return
    end
    if KIND_WANT[kind] then check_expr(v, KIND_WANT[kind], ctx, path); return end
    if kind == "actions" then
      check_actions(v, ctx, path)
    elseif kind == "selector" then check_selector(v, ctx, path)
    elseif kind == "status" then
      if type(v) ~= "string" or not statuses[v] then err(path, "狀態 `" .. tostring(v) .. "` 沒有宣告") end
    elseif kind == "ability" then
      if type(v) == "string" and abilities[v] then return end
      check_expr(v, "ability", ctx, path)
    elseif kind == "template" then
      if type(v) ~= "string" or not templates[v] then err(path, "單位模板 `" .. tostring(v) .. "` 沒有宣告") end
    elseif kind == "routine" then
      if type(v) ~= "string" or not routines[v] then err(path, "routine `" .. tostring(v) .. "` 沒有宣告") end
    elseif kind == "hook" then
      if type(v) ~= "string" or not hooks[v] then err(path, "hook `" .. tostring(v) .. "` 沒有宣告在 hooks")
      else hooks[v].used = true; if current_entry then hook_users[current_entry] = true end end
    elseif kind == "item_name" then
      if type(v) ~= "string" or not v:match("^item_[a-z0-9_]+$") then err(path, "必須是道具 id（item_...）") end
    elseif kind == "item_ref" then
      if type(v) == "string" and v:match("^item_[a-z0-9_]+$") then return end
      check_expr(v, "any", ctx, path)
    elseif kind == "flag" then
      enum(v, V.SELECT_FLAGS, path)
    elseif kind == "state_flag" then
      enum(v, V.STATES, path)
    elseif kind == "unit_overrides" or kind == "illusion_damage" then
      local keys = kind == "unit_overrides" and V.UNIT_OVERRIDES or V.ILLUSION_DAMAGE_KEYS
      if type(v) ~= "table" then err(path, "必須是 table"); return end
      for _, k in ipairs(sorted_keys(v)) do
        if not keys[k] then err(path, "不認得的鍵 `" .. tostring(k) .. "`（" .. table.concat(sorted_keys(keys), "、") .. "）") end
        check_expr(v[k], "num", ctx, path .. "." .. tostring(k))
      end
    elseif kind == "illusion_copy" then
      if fields(v, V.ILLUSION_COPY_FIELDS, path) then
        for _, k in ipairs({ "level", "abilities", "unit_state" }) do
          if v[k] ~= nil and type(v[k]) ~= "boolean" then err(path .. "." .. k, "必須是 boolean") end
        end
        if v.items ~= nil then enum(v.items, V.ILLUSION_ITEMS, path .. ".items") end
        if v.status_duration ~= nil then enum(v.status_duration, V.ILLUSION_STATUS_DURATION, path .. ".status_duration") end
        if v.status_caster ~= nil then enum(v.status_caster, V.ILLUSION_STATUS_CASTER, path .. ".status_caster") end
        if v.team ~= nil then enum(v.team, V.ILLUSION_TEAM, path .. ".team") end
        for _, k in ipairs({ "status_tags", "status_except" }) do
          if v[k] ~= nil then
            if not is_array(v[k]) then err(path .. "." .. k, "必須是陣列")
            else for i, x in ipairs(v[k]) do if not snake(x) then err(path .. "." .. k .. "[" .. i .. "]", "必須是 snake_case") end end end
          end
        end
        for _, k in ipairs({ "hp", "mana" }) do if v[k] ~= nil then check_expr(v[k], "num", ctx, path .. "." .. k) end end
        for _, k in ipairs({ "except_abilities", "statuses" }) do
          if v[k] ~= nil then
            if not is_array(v[k]) then err(path .. "." .. k, "必須是陣列")
            else for i, x in ipairs(v[k]) do
              if not snake(x) then err(path .. "." .. k .. "[" .. i .. "]", "必須是 snake_case id（\"all\" 表示全部）") end
            end end
          end
        end
      end
    elseif kind == "cue" or kind == "text" then need_string(v, path)
    elseif kind == "name" then
      if not snake(v) then err(path, "必須是 snake_case 名稱") end
    elseif kind == "bool" then
      if type(v) ~= "boolean" then err(path, "必須是 true／false") end
    elseif kind == "table" then
      if type(v) ~= "table" then err(path, "必須是 名稱 → 運算式 的 table"); return end
      for _, k in ipairs(sorted_keys(v)) do
        if not snake(k) then err(path, "鍵 `" .. tostring(k) .. "` 必須是 snake_case") end
        if type(v[k]) == "table" then
          for _, k2 in ipairs(sorted_keys(v[k])) do check_expr(v[k][k2], "any", ctx, path .. "." .. k .. "." .. k2) end
        else check_expr(v[k], "any", ctx, path .. "." .. k) end
      end
    elseif kind == "callbacks" then
      if type(v) ~= "table" or (is_array(v) and next(v) ~= nil) then err(path, "必須是 名稱 → action 陣列 的 table"); return end
      for _, k in ipairs(sorted_keys(v)) do
        if not snake(k) then err(path, "回呼名稱 `" .. tostring(k) .. "` 必須是 snake_case") end
        check_actions(v[k], ctx, path .. "." .. tostring(k))
      end
    elseif kind == "options" then
      if type(v) ~= "table" or not is_array(v) or #v == 0 then err(path, "必須是非空陣列"); return end
      for i, opt in ipairs(v) do
        local p = path .. "[" .. i .. "]"
        if fields(opt, V.OPTION_FIELDS, p) then
          if opt.weight ~= nil then check_expr(opt.weight, "num", ctx, p .. ".weight") end
          if opt.value ~= nil then check_expr(opt.value, "any", ctx, p .. ".value") end
          if opt.actions ~= nil then check_actions(opt.actions, ctx, p .. ".actions") end
        end
      end
    else
      err(path, "（內部）不認得的參數型別 " .. kind)
    end
  end

  local function child_ctx(ctx, names)
    local c = {}
    for k, v in pairs(ctx) do c[k] = v end
    c.scope = new_scope(ctx.scope, names)
    return c
  end

  check_selector = function(s, ctx, path)
    if not fields(s, V.SELECTOR_FIELDS, path) then return end
    if s.from == nil then err(path .. ".from", "必填（起點單位或位置）")
    else check_expr(s.from, "any", ctx, path .. ".from") end
    local shape = s.shape or "circle"
    local req = V.SHAPES[shape]
    if not req then err(path .. ".shape", "不認得的形狀 `" .. tostring(shape) .. "`"); req = {} end
    for _, k in ipairs(req) do if s[k] == nil then err(path .. "." .. k, shape .. " 需要 " .. k) end end
    for _, k in ipairs({ "radius", "inner", "outer", "angle", "length", "width", "limit" }) do
      if s[k] ~= nil then check_expr(s[k], "num", ctx, path .. "." .. k) end
    end
    if s.to ~= nil then check_expr(s.to, "any", ctx, path .. ".to") end
    if s.relative_to ~= nil then check_expr(s.relative_to, "any", ctx, path .. ".relative_to") end
    enum(s.team, V.TEAMS, path .. ".team", true)
    for _, key in ipairs({ "types", "exclude_types" }) do
      if s[key] ~= nil then
        if not is_array(s[key]) then err(path .. "." .. key, "必須是陣列")
        else for i, t in ipairs(s[key]) do enum(t, V.UNIT_TYPES, path .. "." .. key .. "[" .. i .. "]") end end
      end
    end
    if s.flags ~= nil then
      if not is_array(s.flags) then err(path .. ".flags", "必須是陣列")
      else for i, f in ipairs(s.flags) do enum(f, V.SELECT_FLAGS, path .. ".flags[" .. i .. "]") end end
    end
    if s.order ~= nil then enum(s.order, V.ORDERS, path .. ".order") end
    local cand = child_ctx(ctx, { it = "unit" })
    for _, k in ipairs({ "where", "prefer", "post_filter", "take_while" }) do
      if s[k] ~= nil then check_expr(s[k], "bool", cand, path .. "." .. k) end
    end
    if s.visible_to ~= nil then check_expr(s.visible_to, "any", ctx, path .. ".visible_to") end
    if s.exclude ~= nil then check_expr(s.exclude, "any", ctx, path .. ".exclude") end
  end

  -- Nested action lists that bind `it` (the unit the callback is about).
  local IT_LISTS = { on_hit = true, on_enter = true, on_leave = true, on_pickup = true }

  check_actions = function(list, ctx, path)
    if type(list) ~= "table" or not is_array(list) then err(path, "必須是 action 陣列"); return end
    local scope_ctx = child_ctx(ctx, {})
    for i, a in ipairs(list) do
      local p = path .. "[" .. i .. "]"
      if type(a) ~= "table" then err(p, "action 必須是 table")
      else
        local op = a.op
        local spec = V.ACTIONS[op]
        if not spec then err(p .. ".op", "不認得的 action `" .. tostring(op) .. "`")
        else
          p = p .. "(" .. op .. ")"
          for _, k in ipairs(sorted_keys(a)) do
            if k ~= "op" and k ~= "notes" and spec[k] == nil then err(p, "`" .. op .. "` 沒有參數 `" .. tostring(k) .. "`") end
          end
          for _, k in ipairs(sorted_keys(spec)) do
            local kind = spec[k]
            local required = kind:sub(-1) == "!"
            if required then kind = kind:sub(1, -2) end
            local v = a[k]
            if v == nil then
              if required then err(p .. "." .. k, "必填") end
            elseif k == "as" or k == "bind" or (op == "let" and k == "name") or k == "id" and (op == "delay" or op == "repeat") then
              if not snake(v) then err(p .. "." .. k, "必須是 snake_case 名稱") end
            elseif kind == "actions" then
              local names = {}
              if op == "for_each" or op == "chain" then names[op == "chain" and "it" or (a.as or "it")] = "unit" end
              if op == "chain" then names.jump = "num" end
              if op == "repeat" and a.as then names[a.as] = "num" end
              if IT_LISTS[k] then names.it = "unit" end
              if (op == "projectile" or op == "dash" or op == "zone" or op == "tether") and a.as then names[a.as] = BIND_KIND[op] end
              if op == "projectile" then names.here = "point" end
              check_actions(v, child_ctx(scope_ctx, names), p .. "." .. k)
            elseif ((op == "projectile" or op == "zone") and k == "where") or (op == "chain" and (k == "where" or k == "take_while")) then
              check_param(kind, v, child_ctx(scope_ctx, { it = "unit" }), p .. "." .. k, a)
            elseif op == "pick_item" and k == "where" then
              check_param(kind, v, child_ctx(scope_ctx, { it = "item" }), p .. "." .. k, a)
            elseif op == "repeat" and (k == "while" or k == "stop_when") then
              local names = {}
              if a.as then names[a.as] = "num" end
              check_param(kind, v, child_ctx(scope_ctx, names), p .. "." .. k, a)
            elseif op == "delay" and k == "capture" then
              if not is_array(v) then err(p .. ".capture", "必須是名稱陣列")
              else for j, n in ipairs(v) do
                if not lookup(scope_ctx.scope, n) then err(p .. ".capture[" .. j .. "]", "`" .. tostring(n) .. "` 不是目前可用的名稱") end
              end end
            elseif op == "chance" and (k == "counter" or k == "counter_var") and a.pity_after == nil then
              err(p .. "." .. k, k .. " 只在有 pity_after 時有意義")
            elseif op == "chance" and k == "counter_var" then
              if not snake(v) or not ((ctx.state and ctx.state[v]) or doc_state[v]) then
                err(p .. ".counter_var", "保底計數要用已宣告的 unit_state（int）")
              end
              if a.counter ~= nil then err(p .. ".counter", "counter 與 counter_var 只能擇一") end
            else
              check_param(kind, v, scope_ctx, p .. "." .. k, a)
            end
          end
          if op == "remove_status" and a.status == nil and a.dispel == nil and a.tags == nil and a.states == nil then err(p, "要指定 status、dispel、tags 或 states") end
          if op == "remove_status" then
            for _, k in ipairs({ "tags", "exclude_tags" }) do
              for _, tag in ipairs(type(a[k]) == "table" and a[k] or {}) do tag_reads[#tag_reads + 1] = { tag = tag, path = p .. "." .. k } end
            end
          end
          if op == "modify_cooldown" and type(a.item_tag) == "string" then item_tag_uses[#item_tag_uses + 1] = { tag = a.item_tag, path = p .. ".item_tag" } end
          if op == "modify_cooldown" and a.ability == nil and a.item_tag == nil and a.item == nil then
            err(p, "modify_cooldown 要指定 ability、item 或 item_tag")
          end
          if (op == "set_state" or op == "clear_state") and snake(a.var) and not ((ctx.state and ctx.state[a.var]) or doc_state[a.var]) then
            err(p .. ".var", "unit_state 沒有宣告 `" .. a.var .. "`")
          end
          if op == "set_var" and snake(a.var) then
            local vs = ctx.vars
            if a.status and statuses[a.status] then vs = statuses[a.status].vars end
            if not (vs and vs[a.var]) then err(p .. ".var", "狀態 vars 沒有宣告 `" .. a.var .. "`（在別的狀態上要寫 status）") end
          end
          if op == "call" and a.routine and routine_hooks[a.routine] and current_entry then hook_users[current_entry] = true end
          if op == "projectile" and a.to == nil and a.direction == nil and a.path == nil then err(p, "要指定 to、direction 或 path") end
          if op == "repeat" and a.count == nil and a["while"] == nil and a.stop_when == nil then
            err(p, "repeat 要有 count、while 或 stop_when 其中之一，否則永遠不會停")
          end
          if op == "call" and a.routine and routines[a.routine] then
            local params = routines[a.routine].params or {}
            for _, k in ipairs(sorted_keys(a.args or {})) do
              if params[k] == nil then err(p .. ".args." .. k, "routine `" .. a.routine .. "` 沒有參數 " .. k) end
            end
            for _, k in ipairs(sorted_keys(params)) do
              if (a.args or {})[k] == nil then err(p .. ".args." .. k, "routine `" .. a.routine .. "` 需要參數 " .. k) end
            end
          end
          if op == "hook" and a.name and hooks[a.name] then
            local params = hooks[a.name].def.params
            if type(params) == "table" then
              for _, k in ipairs(sorted_keys(a.args or {})) do
                if params[k] == nil then err(p .. ".args." .. k, "hook `" .. a.name .. "` 沒有宣告參數 " .. k) end
              end
            end
          end
          -- bind results for the following actions
          if op == "let" and snake(a.name) then
            scope_ctx.scope.names[a.name] = a.value ~= nil and check_expr(a.value, "any", scope_ctx, p .. ".value(型別)") or "any"
          elseif a.as and BIND_KIND[op] and snake(a.as) then
            scope_ctx.scope.names[a.as] = BIND_KIND[op]
          end
        end
      end
    end
  end

  -- triggers: map event -> actions | { actions, filter, interval, ... }
  local function check_triggers(t, events, ctx, path)
    if t == nil then return end
    if type(t) ~= "table" or (is_array(t) and next(t) ~= nil) then err(path, "必須是 事件名稱 → action 的 table"); return end
    for _, name in ipairs(sorted_keys(t)) do
      local p = path .. "." .. tostring(name)
      local base = tostring(name):gsub("_%d+$", "") -- periodic_2: several periodic triggers
      local params = events[base]
      if not params then err(p, "不認得的事件 `" .. tostring(name) .. "`（可用：" .. table.concat(sorted_keys(events), "、") .. "）")
      else
        local def = t[name]
        local ectx = child_ctx(ctx, {})
        ectx.event = params
        if is_array(def) then
          if base == "periodic" then err(p, "periodic 要寫成 { interval = \"$x\", actions = {...} }") end
          check_actions(def, ectx, p)
        elseif fields(def, V.TRIGGER_FIELDS, p) then
          if def.filter ~= nil then check_expr(def.filter, "bool", ectx, p .. ".filter") end
          if def.interval ~= nil then check_expr(def.interval, "num", ectx, p .. ".interval")
          elseif base == "periodic" or base == "on_channel_tick" then err(p .. ".interval", base .. " 必須有 interval") end
          if def.first ~= nil then check_expr(def.first, "num", ectx, p .. ".first") end
          if def.who ~= nil then enum(def.who, V.TRIGGER_WHO, p .. ".who") end
          if def.phase ~= nil then enum(def.phase, V.set({ "pre_mitigation", "post_mitigation" }), p .. ".phase") end
          if def.priority ~= nil then check_expr(def.priority, "num", ectx, p .. ".priority") end
          check_actions(def.actions, ectx, p .. ".actions")
        end
      end
    end
  end

  local function check_stats(stats, ctx, path, info)
    if stats == nil then return end
    if type(stats) ~= "table" or (is_array(stats) and next(stats) ~= nil) then err(path, "必須是 StatKey → 運算式 的 table"); return end
    local ictx = child_ctx(ctx, {})
    ictx.info = info
    for _, k in ipairs(sorted_keys(stats)) do
      local p = path .. "." .. tostring(k)
      if not V.STATS[k] then err(p, "不認得的 StatKey `" .. tostring(k) .. "`") end
      local v = stats[k]
      if type(v) == "table" then
        if fields(v, V.STAT_ENTRY_FIELDS, p) then
          if v.op ~= nil then enum(v.op, V.STAT_OPS, p .. ".op") end
          check_expr(v.value, "num", ictx, p .. ".value")
          if v.when ~= nil then check_expr(v.when, "bool", ictx, p .. ".when") end
          if v.per_stack ~= nil and type(v.per_stack) ~= "boolean" then err(p .. ".per_stack", "必須是 boolean") end
        end
      else
        check_expr(v, "num", ictx, p)
      end
    end
  end

  local function check_aura(a, ctx, path)
    if a == nil then return end
    if not fields(a, V.AURA_FIELDS, path) then return end
    if a.radius == nil then err(path .. ".radius", "必填") else check_expr(a.radius, "num", ctx, path .. ".radius") end
    if a.status == nil or not statuses[a.status] then err(path .. ".status", "光環要給一個已宣告的狀態") end
    enum(a.team, V.TEAMS, path .. ".team", true)
    if a.linger ~= nil then check_expr(a.linger, "num", ctx, path .. ".linger") end
    if a.include_self ~= nil and type(a.include_self) ~= "boolean" then err(path .. ".include_self", "必須是 boolean") end
    if a.types ~= nil then for i, t in ipairs(a.types) do enum(t, V.UNIT_TYPES, path .. ".types[" .. i .. "]") end end
    if a.flags ~= nil then for i, f in ipairs(a.flags) do enum(f, V.SELECT_FLAGS, path .. ".flags[" .. i .. "]") end end
    if a.where ~= nil then check_expr(a.where, "bool", child_ctx(ctx, { it = "unit" }), path .. ".where") end
  end

  local function check_states(list, path)
    if list == nil then return end
    if not is_array(list) then err(path, "必須是狀態名稱陣列"); return end
    for i, s in ipairs(list) do enum(s, V.STATES, path .. "[" .. i .. "]") end
  end

  local text_check -- forward

  local function check_status(s, so, path, base_ctx)
    if not fields(s, V.STATUS_FIELDS, path) then return end
    local vars = {}
    init_owner, init_names = so, { self = "unit", source = "unit" }
    if s.vars ~= nil then collect_state(s.vars, path .. ".vars", vars) end
    init_names = {}
    local ctx = child_ctx(base_ctx, { self = "unit", source = "unit", status = "status" })
    ctx.owner = so
    ctx.vars = vars
    if s.duration == nil then err(path .. ".duration", "必填（運算式或 \"infinite\"）")
    elseif s.duration ~= "infinite" then check_expr(s.duration, "num", ctx, path .. ".duration") end
    if type(s.stacking) == "table" then
      if fields(s.stacking, V.STACK_FIELDS, path .. ".stacking") then
        if s.stacking.max ~= nil then check_expr(s.stacking.max, "num", ctx, path .. ".stacking.max") end
        if s.stacking.mode ~= nil then enum(s.stacking.mode, V.STACK_MODES, path .. ".stacking.mode") end
        if s.stacking.value_stacks ~= nil and type(s.stacking.value_stacks) ~= "boolean" then err(path .. ".stacking.value_stacks", "必須是 boolean") end
        for _, k in ipairs({ "per_stack_duration", "decay", "initial" }) do
          if s.stacking[k] ~= nil then check_expr(s.stacking[k], "num", ctx, path .. ".stacking." .. k) end
        end
      end
    else enum(s.stacking, V.STACKING_MODES, path .. ".stacking", true) end
    enum(s.dispel, V.DISPEL, path .. ".dispel", true)
    enum(s.visibility, V.VISIBILITY, path .. ".visibility", true)
    enum(s.reveal, V.REVEAL, path .. ".reveal", true)
    if s.per_source ~= nil and type(s.per_source) ~= "boolean" then err(path .. ".per_source", "必須是 boolean") end
    if s.reapply_if_removed ~= nil and type(s.reapply_if_removed) ~= "boolean" then err(path .. ".reapply_if_removed", "必須是 boolean") end
    if s.unique_rule ~= nil then enum(s.unique_rule, V.UNIQUE_RULES, path .. ".unique_rule") end
    if s.status_resistance ~= nil then enum(s.status_resistance, V.set({ "ignore", "normal" }), path .. ".status_resistance") end
    if s.scope ~= nil then enum(s.scope, V.STATUS_SCOPES, path .. ".scope") end
    check_stats(s.stats, ctx, path .. ".stats")
    check_states(s.states, path .. ".states")
    if s.absorb ~= nil and fields(s.absorb, V.ABSORB_FIELDS, path .. ".absorb") then
      if s.absorb.amount == nil then err(path .. ".absorb.amount", "必填") else check_expr(s.absorb.amount, "num", ctx, path .. ".absorb.amount") end
      if s.absorb.fraction ~= nil then check_expr(s.absorb.fraction, "num", ctx, path .. ".absorb.fraction") end
      if s.absorb.types ~= nil then for i, t in ipairs(s.absorb.types) do enum(t, V.DAMAGE_TYPES, path .. ".absorb.types[" .. i .. "]") end end
      local actx = child_ctx(ctx, {})
      actx.event = V.STATUS_EVENTS.on_take_damage
      if s.absorb.on_absorb ~= nil then check_actions(s.absorb.on_absorb, actx, path .. ".absorb.on_absorb") end
      if s.absorb.on_broken ~= nil then check_actions(s.absorb.on_broken, actx, path .. ".absorb.on_broken") end
      if s.absorb.lethal_only ~= nil and type(s.absorb.lethal_only) ~= "boolean" then err(path .. ".absorb.lethal_only", "必須是 boolean") end
      if s.absorb.phase ~= nil then enum(s.absorb.phase, V.set({ "pre_mitigation", "post_mitigation" }), path .. ".absorb.phase") end
    end
    if s.charges ~= nil then check_expr(s.charges, "num", ctx, path .. ".charges") end
    if s.immune_to ~= nil then
      if not is_array(s.immune_to) then err(path .. ".immune_to", "必須是陣列")
      else for i, x in ipairs(s.immune_to) do
        if not (statuses[x] or V.STATES[x] or V.ABILITY_TAGS[x] or x == "debuffs" or x == "magic" or x == "physical"
          or (type(x) == "string" and x:match("^tag:[a-z][a-z0-9_]*$"))) then
          err(path .. ".immune_to[" .. i .. "]", "`" .. tostring(x) .. "` 不是已宣告狀態、狀態名或 tag")
        end
      end end
    end
    if s.block_reapply ~= nil and type(s.block_reapply) ~= "boolean" then err(path .. ".block_reapply", "必須是 boolean") end
    if s.unique_group ~= nil then need_string(s.unique_group, path .. ".unique_group") end
    check_aura(s.aura, ctx, path .. ".aura")
    check_triggers(s.triggers, V.STATUS_EVENTS, ctx, path .. ".triggers")
    if s.cue ~= nil then need_string(s.cue, path .. ".cue") end
    -- v3.3: the BuffStore id the current runtime uses for this status; `{caster}` stands for the
    -- caster handle suffix "id:gen" (omb-script-abi buff_ids), `{rank}` for the owner's rank
    if s.runtime_id ~= nil then
      need_string(s.runtime_id, path .. ".runtime_id")
      if type(s.runtime_id) == "string" then
        local rest = s.runtime_id:gsub("{caster}", ""):gsub("{rank}", "")
        if rest == "" or rest:find("[^%w_:%.%-]") or rest:find("[{}]") then
          err(path .. ".runtime_id", "只能用英數、_ : . - 與 {caster}、{rank}")
        end
      end
    end
    if s.tags ~= nil then for i, t in ipairs(s.tags) do need_string(t, path .. ".tags[" .. i .. "]") end end
    if s.forced_move ~= nil and fields(s.forced_move, V.FORCED_MOVE_FIELDS, path .. ".forced_move") then
      check_expr(s.forced_move.toward, "any", ctx, path .. ".forced_move.toward")
      if s.forced_move.speed ~= nil then check_expr(s.forced_move.speed, "num", ctx, path .. ".forced_move.speed") end
      if s.forced_move.locks ~= nil then
        if not is_array(s.forced_move.locks) then err(path .. ".forced_move.locks", "必須是陣列")
        else for i, l in ipairs(s.forced_move.locks) do enum(l, V.ORDER_LOCKS, path .. ".forced_move.locks[" .. i .. "]") end end
      end
    end
    if s.linger ~= nil then check_expr(s.linger, "num", ctx, path .. ".linger") end
    if s.remove_on ~= nil then for i, r in ipairs(s.remove_on) do enum(r, V.REMOVE_ON, path .. ".remove_on[" .. i .. "]") end end
    if s.text ~= nil then text_check(s.text, so, path .. ".text", false) end
    -- v3.3 (E204): a status the player sees (buff / debuff icon) needs its name and tooltip in both languages
    if s.visibility == "buff" or s.visibility == "debuff" then
      for _, lang in ipairs(V.LANGS) do
        local t = type(s.text) == "table" and s.text[lang]
        if type(t) ~= "table" or type(t.name) ~= "string" or type(t.body) ~= "string" then
          err(path .. ".text." .. lang, "可見狀態（" .. s.visibility .. "）必須寫 name 與 body（狀態圖示的提示）")
        end
      end
    end
  end

  local function check_passive(pv, owner, path, base_ctx)
    if not fields(pv, V.PASSIVE_FIELDS, path) then return end
    local ctx = child_ctx(base_ctx, { self = "unit" })
    check_stats(pv.stats, ctx, path .. ".stats", owner.kind == "item")
    check_states(pv.states, path .. ".states")
    if pv.unique_group ~= nil then need_string(pv.unique_group, path .. ".unique_group") end
    if pv.reveal ~= nil then enum(pv.reveal, V.REVEAL, path .. ".reveal") end
    if pv.visibility ~= nil then enum(pv.visibility, V.VISIBILITY, path .. ".visibility") end
    if pv.orb_slot ~= nil and fields(pv.orb_slot, V.ORB_SLOT_FIELDS, path .. ".orb_slot") then
      enum(pv.orb_slot.release, V.ORB_RELEASE, path .. ".orb_slot.release", true)
      if pv.orb_slot.priority ~= nil then check_expr(pv.orb_slot.priority, "num", ctx, path .. ".orb_slot.priority") end
      if pv.orb_slot.claim ~= nil and type(pv.orb_slot.claim) ~= "boolean" then err(path .. ".orb_slot.claim", "必須是 boolean") end
      if pv.orb_slot.overridable ~= nil and type(pv.orb_slot.overridable) ~= "boolean" then err(path .. ".orb_slot.overridable", "必須是 boolean") end
    end
    check_aura(pv.aura, ctx, path .. ".aura")
    check_triggers(pv.triggers, V.STATUS_EVENTS, ctx, path .. ".triggers")
  end

  -- header (info line) fields of an ability-like table
  local function check_castable(a, owner, path, base_ctx, kind_field_required)
    local info = child_ctx(base_ctx, {})
    info.info = true
    -- v3.3: bot cast policy (v3_bot.lua); thresholds are policy numbers, not gameplay values
    if a.bot ~= nil then
      for _, p in ipairs(BOT.check(a.bot, function(n) return (find_value(owner, nil, n)) end)) do err(path .. ".bot", p) end
    end
    local tt = "none"
    if a.target ~= nil and fields(a.target, V.TARGET_FIELDS, path .. ".target") then
      tt = a.target.type
      enum(tt, V.TARGET_TYPES, path .. ".target.type", true)
      for _, k in ipairs({ "range", "aoe_radius", "min_range", "vector_length" }) do
        if a.target[k] ~= nil then check_expr(a.target[k], "num", info, path .. ".target." .. k) end
      end
      if a.target.team ~= nil then enum(a.target.team, V.TEAMS, path .. ".target.team") end
      if a.target.types ~= nil then for i, t in ipairs(a.target.types) do enum(t, V.UNIT_TYPES, path .. ".target.types[" .. i .. "]") end end
      if a.target.flags ~= nil then for i, f in ipairs(a.target.flags) do enum(f, V.SELECT_FLAGS, path .. ".target.flags[" .. i .. "]") end end
    elseif a.target == nil and kind_field_required then err(path .. ".target", "必填（沒有目標寫 { type = \"none\" }）") end
    if a.cast ~= nil and fields(a.cast, V.CAST_FIELDS, path .. ".cast") then
      for _, k in ipairs({ "cast_point", "backswing", "channel_time" }) do
        if a.cast[k] ~= nil then check_expr(a.cast[k], "num", info, path .. ".cast." .. k) end
      end
      for _, k in ipairs({ "immediate", "interruptible", "move_while_casting", "turn_while_casting", "ignore_backswing" }) do
        if a.cast[k] ~= nil and type(a.cast[k]) ~= "boolean" then err(path .. ".cast." .. k, "必須是 boolean") end
      end
    end
    if a.cost ~= nil and fields(a.cost, V.COST_FIELDS, path .. ".cost") then
      for _, k in ipairs({ "mana", "health", "cooldown", "gold" }) do
        if a.cost[k] ~= nil then check_expr(a.cost[k], "num", info, path .. ".cost." .. k) end
      end
      if a.cost.cooldown_start ~= nil then enum(a.cost.cooldown_start, V.COOLDOWN_STARTS, path .. ".cost.cooldown_start") end
      if a.cost.charges ~= nil and fields(a.cost.charges, V.CHARGES_FIELDS, path .. ".cost.charges") then
        for _, k in ipairs({ "max", "restore", "initial" }) do
          if a.cost.charges[k] ~= nil then check_expr(a.cost.charges[k], "num", info, path .. ".cost.charges." .. k) end
        end
        if a.cost.charges.on_rank_up ~= nil then enum(a.cost.charges.on_rank_up, V.CHARGES_RANK_UP, path .. ".cost.charges.on_rank_up") end
        if a.cost.charges.scope ~= nil then enum(a.cost.charges.scope, V.CHARGES_SCOPES, path .. ".cost.charges.scope") end
      end
      if a.cost.cooldown_groups ~= nil then
        for i, g in ipairs(a.cost.cooldown_groups) do
          local p = path .. ".cost.cooldown_groups[" .. i .. "]"
          if fields(g, V.COOLDOWN_GROUP_FIELDS, p) then
            need_string(g.group, p .. ".group")
            if g.length ~= nil then enum(g.length, V.set({ "own", "group_max", "group_min", "caster" }), p .. ".length") end
            if g.role ~= nil then enum(g.role, V.COOLDOWN_GROUP_ROLES, p .. ".role") end
          end
        end
      end
      if a.cost.refund ~= nil then enum(a.cost.refund, V.set({ "never", "on_phase_cancel", "on_target_invalid" }), path .. ".cost.refund") end
    end
    if a.requires ~= nil and fields(a.requires, V.REQUIRES_FIELDS, path .. ".requires") then
      if a.requires.hero_level ~= nil then check_expr(a.requires.hero_level, "num", info, path .. ".requires.hero_level") end
      for _, k in ipairs({ "has_status", "not_status" }) do
        if a.requires[k] ~= nil and not statuses[a.requires[k]] then err(path .. ".requires." .. k, "狀態沒有宣告") end
      end
      if a.requires.condition ~= nil then check_expr(a.requires.condition, "bool", base_ctx, path .. ".requires.condition") end
    end
    if a.flags ~= nil then
      if not is_array(a.flags) then err(path .. ".flags", "必須是陣列")
      else for i, f in ipairs(a.flags) do enum(f, V.ABILITY_FLAGS, path .. ".flags[" .. i .. "]") end end
    end
    if a.tags ~= nil then
      if not is_array(a.tags) then err(path .. ".tags", "必須是陣列")
      else for i, t in ipairs(a.tags) do enum(t, V.ABILITY_TAGS, path .. ".tags[" .. i .. "]") end end
    end
    -- events
    local names = {}
    if tt == "unit" or tt == "unit_or_point" or tt == "tree" or tt == "item" then names.target = "unit" end
    if tt == "point" or tt == "unit_or_point" or tt == "direction" or tt == "vector" then names.point = "point" end
    if tt == "direction" or tt == "vector" then names.direction = "vec" end
    if tt == "vector" then names.vector_end = "point" end
    check_triggers(a.events, V.ABILITY_EVENTS, child_ctx(base_ctx, names), path .. ".events")
  end

  -------------------------------------------------------------------------
  -- text
  -------------------------------------------------------------------------
  local function digit_free(s) return not (s:find("%d") or s:find("０") or s:find("１") or s:find("２") or s:find("３")
    or s:find("４") or s:find("５") or s:find("６") or s:find("７") or s:find("８") or s:find("９")) end
  text_check = function(t, owner, path, required)
    if t == nil then if required then err(path, "必填（zh_TW 與 en）") end return end
    if type(t) ~= "table" then err(path, "必須是 table"); return end
    local refsets = {}
    for _, k in ipairs(sorted_keys(t)) do
      local ok = false
      for _, l in ipairs(V.LANGS) do if l == k then ok = true end end
      if not ok then err(path, "不認得的語言 `" .. tostring(k) .. "`（" .. table.concat(V.LANGS, "、") .. "）") end
    end
    for _, lang in ipairs(V.LANGS) do
      local lt, p = t[lang], path .. "." .. lang
      if lt == nil then err(p, "必填")
      elseif fields(lt, V.TEXT_FIELDS, p) then
        need_string(lt.name, p .. ".name")
        if required and owner.kind == "ability" then need_string(lt.body, p .. ".body") end
        local refs = {}
        for _, field in ipairs(sorted_keys(lt)) do
          local s = lt[field]
          local fp = p .. "." .. field
          if type(s) ~= "string" then err(fp, "必須是字串")
          else
            local stripped = s:gsub("%b{}", "")
            if stripped:find("[{}]") then err(fp, "大括號不成對") end
            if not digit_free(stripped) then
              err(fp, "說明文字不能有數字（數值要寫進 values，用 {名稱} 引用；結構性的「一」用中文或英文字）：" .. s)
            end
            for ph in s:gmatch("{([^{}]*)}") do
              local name, fmt = ph:match("^([^:]+):?(.*)$")
              name = name or ""
              if fmt ~= "" and not V.TEXT_FORMATS[fmt] then err(fp, "不認得的格式 `:" .. fmt .. "`（" .. table.concat(sorted_keys(V.TEXT_FORMATS), "、") .. "）") end
              if name:sub(1, 1) == "@" then
                local id = name:sub(2)
                if not declared[id] and id:match("^item_[a-z0-9_]+$") then
                  -- another document's item (v3.1): resolved by the build from docs/character_pipeline/items
                elseif not declared[id] then err(fp, "{@" .. id .. "} 指向不存在的 id")
                elseif statuses[id] and statuses[id].def.text == nil then
                  err(fp, "{@" .. id .. "} 指向的狀態沒有 text（名稱）：在狀態加 text = { zh_TW = { name }, en = { name } }")
                end
                refs[name] = true
              elseif name:match("^rules%.") then
                local cur = rules
                for part in name:sub(7):gmatch("[^.]+") do cur = type(cur) == "table" and cur[part] or nil end
                if cur == nil or type(cur) == "table" then err(fp, "{" .. name .. "} 規則層沒有這個值") end
                refs[name] = true
              else
                local qual, vn = name:match("^([%w_]+)%.([%w_]+)$")
                if not qual then vn = name end
                if not snake(vn) then err(fp, "佔位符 {" .. ph .. "} 格式錯誤")
                else
                  local d, o = find_value(owner, qual, vn)
                  if not d then err(fp, "佔位符 {" .. name .. "} 沒有對應的 value")
                  else
                    local key = vkey(o, vn)
                    uses[key], visible[key], in_text[key] = true, true, true
                    if d.hidden then err(fp, "{" .. name .. "} 標了 hidden 卻出現在說明裡") end
                  end
                end
                refs[name] = true
              end
            end
          end
        end
        refsets[lang] = refs
      end
    end
    local a, b = refsets.zh_TW, refsets.en
    if a and b then
      for k in pairs(a) do if not b[k] then err(path .. ".en", "缺少 zh_TW 有的佔位符 {" .. k .. "}") end end
      for k in pairs(b) do if not a[k] then err(path .. ".zh_TW", "缺少 en 有的佔位符 {" .. k .. "}") end end
    end
  end

  -------------------------------------------------------------------------
  -- entries
  -------------------------------------------------------------------------
  local function check_provenance(def, path)
    if type(def.provenance) ~= "table" or not is_array(def.provenance) or #def.provenance == 0 then
      err(path .. ".provenance", "必須列出原作出處（path:行號）")
    else
      for i, s in ipairs(def.provenance) do
        if type(s) ~= "string" or not (s:match("^[%w_./%-]+:%d+") or s:match('^"[^"]+":%d+') or s:match("^[^:\n]+%.%a+:%d+")) then
          err(path .. ".provenance[" .. i .. "]", "格式是 <相對 source.root 的路徑>:<行號或區間>")
        end
      end
    end
    if def.quirks ~= nil then
      if not is_array(def.quirks) then err(path .. ".quirks", "必須是陣列")
      else for i, q in ipairs(def.quirks) do
        local p = path .. ".quirks[" .. i .. "]"
        if fields(q, V.QUIRK_FIELDS, p) then
          need_string(q.text, p .. ".text")
          enum(q.policy, V.QUIRK_POLICIES, p .. ".policy", true)
        end
      end end
    end
  end
  local function check_coverage(def, path)
    enum(def.coverage, V.COVERAGE, path .. ".coverage", true)
    enum(def.confidence, V.CONFIDENCE, path .. ".confidence", true)
    local pend = def.pending
    if pend ~= nil then
      if not is_array(pend) then err(path .. ".pending", "必須是陣列")
      else for i, x in ipairs(pend) do
        local p = path .. ".pending[" .. i .. "]"
        if fields(x, V.PENDING_FIELDS, p) then need_string(x.what, p .. ".what"); need_string(x.why, p .. ".why") end
      end end
    end
    local has_pending = type(pend) == "table" and #pend > 0
    local uses_hook = hook_users[path] == true
    if def.coverage == "expressible" then
      if uses_hook then err(path .. ".coverage", "用了 hook，coverage 必須是 needs_hook") end
      if has_pending then err(path .. ".coverage", "還有 pending 項目，coverage 必須是 pending") end
    elseif def.coverage == "needs_hook" then
      if not uses_hook then err(path .. ".coverage", "needs_hook 但行為裡沒有 hook action") end
      if has_pending then err(path .. ".coverage", "還有 pending 項目，coverage 必須是 pending") end
    elseif def.coverage == "pending" then
      if not has_pending then err(path .. ".pending", "coverage = pending 必須列出待確認項目") end
    elseif def.coverage == "deferred" then
      enum(def.deferred_reason, V.DEFERRED_REASONS, path .. ".deferred_reason", true)
      if has_pending then err(path .. ".coverage", "deferred 不能有 pending") end
    elseif def.coverage == "rust_handler" then
      if def.handler == nil then err(path .. ".handler", "coverage = rust_handler 必須寫 handler = { module, reads }") end
      if uses_hook then err(path .. ".coverage", "用了 hook，coverage 必須是 needs_hook") end
    end
    if def.handler ~= nil and def.coverage ~= "rust_handler" and def.coverage ~= "pending" then
      err(path .. ".handler", "handler 只用在 coverage = rust_handler（行為仍在 Rust）")
    end
    if def.host ~= nil and (def.coverage == "rust_handler" or def.coverage == "deferred") then
      err(path .. ".host", "host 只用在行為是資料的條目；rust_handler 的讀取寫在 handler.reads")
    end
  end

  local slots = {}
  for _, e in ipairs(entry_list) do
    local def, o, path = e.def, e.owner, e.path
    current_entry = path
    local state = {}
    init_owner = o
    collect_state(def.unit_state, path .. ".unit_state", state)
    o.state_kinds = state
    local base = { owner = o, scope = new_scope(nil, {}), state = state }
    if e.kind == "ability" then
      fields(def, V.ABILITY_FIELDS, path)
      need_string(def.source_id, path .. ".source_id")
      enum(def.slot, V.SLOTS, path .. ".slot", true)
      if def.slot and def.slot ~= "sub" and def.slot ~= "hidden" then
        if slots[def.slot] then err(path .. ".slot", def.slot .. " 已被 " .. slots[def.slot] .. " 使用") end
        slots[def.slot] = def.id
      end
      enum(def.kind, V.ABILITY_KINDS, path .. ".kind", true)
      if def.learnable ~= nil and type(def.learnable) ~= "boolean" then err(path .. ".learnable", "必須是 boolean") end
      if def.level_link ~= nil and not abilities[def.level_link] then err(path .. ".level_link", "指向不存在的技能") end
      check_castable(def, o, path, base, def.coverage ~= "deferred")
      if def.passive ~= nil then check_passive(def.passive, o, path .. ".passive", base) end
      if def.kind == "passive" and def.events ~= nil then
        for k in pairs(def.events) do
          if k ~= "on_learn" and k ~= "on_upgrade" then err(path .. ".events." .. k, "被動技能的行為寫在 passive.triggers") end
        end
      end
      text_check(def.text, o, path .. ".text", def.coverage ~= "deferred")
    else
      fields(def, V.ITEM_FIELDS, path)
      need_string(def.source_id, path .. ".source_id")
      if def.tags ~= nil then
        if not is_array(def.tags) then err(path .. ".tags", "必須是道具 tag 陣列")
        else for i, tag in ipairs(def.tags) do
          if not snake(tag) then err(path .. ".tags[" .. i .. "]", "必須是 snake_case") end
          item_tag_uses[#item_tag_uses + 1] = { tag = tag, path = path .. ".tags[" .. i .. "]" }
        end end
      end
      if fields(def.shop or {}, V.SHOP_FIELDS, path .. ".shop") then
        local sh = def.shop or {}
        local info = child_ctx(base, {})
        info.info = true
        if sh.cost == nil then err(path .. ".shop.cost", "必填") else check_expr(sh.cost, "num", info, path .. ".shop.cost") end
        if sh.recipe ~= nil and fields(sh.recipe, V.RECIPE_FIELDS, path .. ".shop.recipe") then
          if not is_array(sh.recipe.components) then err(path .. ".shop.recipe.components", "必須是道具 id 陣列")
          else for i, c in ipairs(sh.recipe.components) do
            if type(c) ~= "string" or not c:match("^item_[a-z0-9_]+$") then err(path .. ".shop.recipe.components[" .. i .. "]", "必須是道具 id") end
          end end
          if sh.recipe.recipe_cost ~= nil then check_expr(sh.recipe.recipe_cost, "num", info, path .. ".shop.recipe.recipe_cost") end
        end
        for _, k in ipairs({ "sellable", "droppable", "shareable", "purchasable", "unique" }) do
          if sh[k] ~= nil and type(sh[k]) ~= "boolean" then err(path .. ".shop." .. k, "必須是 boolean") end
        end
        if sh.category ~= nil then need_string(sh.category, path .. ".shop.category") end
        if type(sh.stock) == "table" then
          if fields(sh.stock, V.STOCK_FIELDS, path .. ".shop.stock") then
            for _, k in ipairs(sorted_keys(sh.stock)) do check_expr(sh.stock[k], "num", info, path .. ".shop.stock." .. k) end
          end
        elseif sh.stock ~= nil then check_expr(sh.stock, "num", info, path .. ".shop.stock") end
        if sh.consumable ~= nil and fields(sh.consumable, V.CONSUMABLE_FIELDS, path .. ".shop.consumable") then
          if sh.consumable.charges ~= nil then check_expr(sh.consumable.charges, "num", info, path .. ".shop.consumable.charges") end
        end
      end
      if def.passive ~= nil then check_passive(def.passive, o, path .. ".passive", base) end
      if def.active ~= nil and fields(def.active, V.ITEM_ACTIVE_FIELDS, path .. ".active") then
        enum(def.active.kind, V.ABILITY_KINDS, path .. ".active.kind", true)
        check_castable(def.active, o, path .. ".active", base, true)
      end
      text_check(def.text, o, path .. ".text", def.coverage ~= "deferred")
    end
    for _, sid in ipairs(sorted_keys(statuses)) do
      local st = statuses[sid]
      if st.owner.parent == o then check_status(st.def, st.owner, st.path, base) end
    end
    check_provenance(def, path)
    current_entry = nil
  end

  -- doc-level statuses
  for _, sid in ipairs(sorted_keys(statuses)) do
    local st = statuses[sid]
    if st.owner.parent == shared then check_status(st.def, st.owner, st.path, { owner = shared, scope = new_scope(nil, {}), state = {} }) end
  end

  -- unit templates
  for _, tid in ipairs(sorted_keys(templates)) do
    local o = templates[tid]
    local u, path = o.def, o.path
    fields(u, V.UNIT_TEMPLATE_FIELDS, path)
    local ctx = { owner = o, scope = new_scope(nil, { self = "unit" }), state = {}, info = true }
    if u.stats ~= nil and fields(u.stats, V.UNIT_STATS, path .. ".stats") then
      for _, k in ipairs(sorted_keys(u.stats)) do check_expr(u.stats[k], "num", ctx, path .. ".stats." .. k) end
    end
    if u.attack ~= nil and fields(u.attack, V.UNIT_ATTACK_FIELDS, path .. ".attack") then
      enum(u.attack.kind, V.set({ "melee", "ranged", "none" }), path .. ".attack.kind", true)
      for _, k in ipairs({ "range", "interval", "damage", "damage_max", "projectile_speed", "attack_point" }) do
        if u.attack[k] ~= nil then check_expr(u.attack[k], "num", ctx, path .. ".attack." .. k) end
      end
      if u.attack.damage_type ~= nil then enum(u.attack.damage_type, V.DAMAGE_TYPES, path .. ".attack.damage_type") end
    end
    if u.bounty ~= nil and fields(u.bounty, V.BOUNTY_FIELDS, path .. ".bounty") then
      for _, k in ipairs({ "gold", "xp" }) do if u.bounty[k] ~= nil then check_expr(u.bounty[k], "num", ctx, path .. ".bounty." .. k) end end
    end
    if u.vision ~= nil and fields(u.vision, V.UNIT_VISION_FIELDS, path .. ".vision") then
      for _, k in ipairs({ "day", "night" }) do if u.vision[k] ~= nil then check_expr(u.vision[k], "num", ctx, path .. ".vision." .. k) end end
    end
    if u.owner_death ~= nil then enum(u.owner_death, V.OWNER_DEATH, path .. ".owner_death") end
    for _, k in ipairs({ "collision_radius", "limit_per_owner", "acquisition_range", "turn_rate" }) do
      if u[k] ~= nil then check_expr(u[k], "num", ctx, path .. "." .. k) end
    end
    if u.ai ~= nil then enum(u.ai, V.UNIT_AI, path .. ".ai") end
    if u.inventory_locked ~= nil and type(u.inventory_locked) ~= "boolean" then err(path .. ".inventory_locked", "必須是 boolean") end
    if u.chase ~= nil and fields(u.chase, V.CHASE_FIELDS, path .. ".chase") then
      if u.chase.anchor ~= nil then enum(u.chase.anchor, V.CHASE_ANCHORS, path .. ".chase.anchor") end
      for _, k in ipairs({ "speed", "leash", "reach_radius", "give_up_after", "max_distance", "teleport_back_distance" }) do
        if u.chase[k] ~= nil then check_expr(u.chase[k], "num", ctx, path .. ".chase." .. k) end
      end
      if u.chase.retarget ~= nil then enum(u.chase.retarget, V.CHASE_RETARGET, path .. ".chase.retarget") end
      if u.chase.on_reach ~= nil then check_actions(u.chase.on_reach, child_ctx(ctx, { target = "unit" }), path .. ".chase.on_reach") end
    end
    if u.ai == "chase_target" and u.chase == nil then err(path .. ".chase", "ai = chase_target 要寫 chase 參數") end
    if u.item_cast_blacklist ~= nil then
      if not is_array(u.item_cast_blacklist) then err(path .. ".item_cast_blacklist", "必須是道具 id 陣列")
      else for i, x in ipairs(u.item_cast_blacklist) do
        if type(x) ~= "string" or not x:match("^item_[a-z0-9_]+$") then err(path .. ".item_cast_blacklist[" .. i .. "]", "必須是道具 id") end
      end end
    end
    if u.over_limit ~= nil then enum(u.over_limit, V.OVER_LIMIT, path .. ".over_limit") end
    for _, k in ipairs({ "blocks_pathing", "flying" }) do
      if u[k] ~= nil and type(u[k]) ~= "boolean" then err(path .. "." .. k, "必須是 boolean") end
    end
    if u.statuses ~= nil then for i, s in ipairs(u.statuses) do
      if not statuses[s] then err(path .. ".statuses[" .. i .. "]", "狀態 `" .. tostring(s) .. "` 沒有宣告") end
    end end
    if u.abilities ~= nil then for i, s in ipairs(u.abilities) do
      if not abilities[s] then err(path .. ".abilities[" .. i .. "]", "技能 `" .. tostring(s) .. "` 沒有宣告") end
    end end
    if u.tags ~= nil then for i, t in ipairs(u.tags) do enum(t, V.UNIT_TYPES, path .. ".tags[" .. i .. "]") end end
    if u.passive ~= nil then check_passive(u.passive, o, path .. ".passive", { owner = o, scope = new_scope(nil, {}), state = {} }) end
    text_check(u.text, o, path .. ".text", true)
    if u.provenance ~= nil or u.quirks ~= nil then check_provenance(u, path) end
  end

  -- routines
  for _, name in ipairs(sorted_keys(routines)) do
    local r, path = routines[name], "routines." .. name
    if fields(r, V.ROUTINE_FIELDS, path) then
      local names = {}
      for _, k in ipairs(sorted_keys(r.params or {})) do
        if not snake(k) then err(path .. ".params." .. tostring(k), "名稱必須是 snake_case") end
        local kind = r.params[k]
        if not ({ num = true, bool = true, unit = true, point = true, vec = true, any = true })[kind] then
          err(path .. ".params." .. k, "型別必須是 num、bool、unit、point、vec、any")
        end
        names[k] = kind
      end
      current_entry = nil
      -- `owner` (v3.1): the ability whose values ($x) and unit_state the routine reads; without it
      -- only document values and $owner.name references resolve.
      local rowner, rstate = shared, {}
      if r.owner ~= nil then
        local ab = abilities[r.owner]
        if not ab then err(path .. ".owner", "`" .. tostring(r.owner) .. "` 不是這份文件的技能或道具")
        else rowner, rstate = ab, ab.state_kinds or {} end
      end
      check_actions(r.actions, { owner = rowner, scope = new_scope(nil, names), state = rstate }, path .. ".actions")
    end
  end

  -- handler (M1) and host (v3.3): the Rust handler's or host component's reads count as uses
  local function reads(e, list, path)
    if list == nil then return end
    if not is_array(list) then err(path, "必須是 \"$名稱\" 陣列"); return end
    for i, r in ipairs(list) do
      local q, n = tostring(r):match("^%$([%w_]+)%.([%w_]+)$")
      if not q then n = tostring(r):match("^%$([%w_]+)$") end
      local d, o
      if n then d, o = find_value(e.owner, q, n) end
      if not d then err(path .. "[" .. i .. "]", "`" .. tostring(r) .. "` 不是存在的 $value")
      else uses[vkey(o, n)] = true end
    end
  end
  for _, e in ipairs(entry_list) do
    local h, path = e.def.handler, e.path .. ".handler"
    if h ~= nil and fields(h, V.HANDLER_FIELDS, path) then
      need_string(h.module, path .. ".module")
      reads(e, h.reads, path .. ".reads")
    end
    -- host = { component, reads } or a list of them
    local host = e.def.host
    if host ~= nil then
      local list = is_array(host) and host or { host }
      for i, c in ipairs(list) do
        local p = e.path .. ".host" .. (is_array(host) and ("[" .. i .. "]") or "")
        if type(c) ~= "table" then err(p, "必須是 { component, reads }")
        elseif fields(c, V.HOST_FIELDS, p) then
          enum(c.component, V.HOST_COMPONENTS, p .. ".component", true)
          if c.reads == nil or (is_array(c.reads) and #c.reads == 0) then err(p .. ".reads", "host 要列出元件讀的 $value") end
          reads(e, c.reads, p .. ".reads")
        end
      end
    end
  end
  for _, id in ipairs(sorted_keys(owners)) do
    local o = owners[id]
    for _, n in ipairs(sorted_keys(type(o.values) == "table" and o.values or {})) do
      local d = o.values[n]
      if type(d) == "table" and d.v2_key ~= nil and not (type(d.v2_key) == "string" and d.v2_key:match("^[a-z][a-z0-9_]*$") or (type(d.v2_key) == "string" and d.v2_key:match("^[%a][%w_]*%.[%w_.]+$"))) then
        err(o.path .. ".values." .. n .. ".v2_key", "必須是 v2 extras 鍵（snake_case），道具是道具列的欄位路徑（例如 active.damage）")
      end
    end
  end

  -- hooks
  for _, name in ipairs(sorted_keys(hooks)) do
    local h = hooks[name]
    if fields(h.def, V.HOOK_FIELDS, h.path) then
      if type(h.def.why) ~= "string" or utf8.len(h.def.why) == nil or utf8.len(h.def.why) < 10 then
        err(h.path .. ".why", "每個 hook 都要寫明為什麼資料表達不了（至少十個字）")
      end
      need_string(h.def.test, h.path .. ".test")
      if h.def.kind ~= nil then enum(h.def.kind, V.HOOK_KINDS, h.path .. ".kind") end
      if h.def.reads ~= nil then
        for i, r in ipairs(h.def.reads) do
          local q, n = tostring(r):match("^%$([%w_]+)%.([%w_]+)$")
          if not q then n = tostring(r):match("^%$([%w_]+)$") end
          local d, o
          if n then d, o = find_value(shared, q, n) end
          if not d then err(h.path .. ".reads[" .. i .. "]", "`" .. tostring(r) .. "` 不是存在的 $value（寫成 $owner.name）")
          else uses[vkey(o, n)] = true end
        end
      end
    end
    if not h.used then err(h.path, "宣告了但沒有任何 hook action 使用") end
  end

  -- unit_state / vars init: expressions of values and rules only
  for _, si in ipairs(state_inits) do
    check_expr(si.init, "any", { owner = si.owner, scope = new_scope(nil, si.names), state = {} }, si.path)
  end

  -- coverage (after behavior so hook use is known)
  for _, e in ipairs(entry_list) do check_coverage(e.def, e.path) end

  -- deferred entries (v3.1)
  if doc.deferred ~= nil then
    if not is_array(doc.deferred) then err("deferred", "必須是陣列")
    else for i, d in ipairs(doc.deferred) do
      local p = "deferred[" .. i .. "]"
      if fields(d, V.DEFERRED_FIELDS, p) then
        need_string(d.source_id, p .. ".source_id")
        enum(d.reason, V.DEFERRED_REASONS, p .. ".reason", true)
        need_string(d.note, p .. ".note")
        if d.id ~= nil then declare(d.id, p) end
      end
    end end
  end

  -- hero block
  if doc.kind == "hero" then
    if fields(doc.hero or {}, V.HERO_FIELDS, "hero") then
      need_string((doc.hero or {}).source_id, "hero.source_id")
      if doc.hero and doc.hero.text ~= nil then
        for _, l in ipairs(V.LANGS) do
          local lt = doc.hero.text[l]
          if type(lt) ~= "table" then err("hero.text." .. l, "必填") else need_string(lt.name, "hero.text." .. l .. ".name") end
        end
      else err("hero.text", "必填（英雄名稱 zh_TW 與 en）") end
      if doc.hero and doc.hero.auto_learn ~= nil then
        for i, id in ipairs(doc.hero.auto_learn) do
          if not abilities[id] then err("hero.auto_learn[" .. i .. "]", "技能不存在") end
        end
      end
    end
    if #entry_list == 0 then err("abilities", "英雄文件至少要有一招") end
  elseif doc.kind == "items" then
    if doc.abilities ~= nil and #doc.abilities > 0 and not opts.allow_item_abilities then
      -- items may carry helper abilities (slot = hidden) for created units
      for _, e in ipairs(entry_list) do
        if e.kind == "ability" and e.def.slot ~= "hidden" and e.def.slot ~= "sub" then
          err(e.path .. ".slot", "道具文件裡的技能只能是 hidden 或 sub（給召喚物或子技能）")
        end
      end
    end
  end

  -------------------------------------------------------------------------
  -- cross-document names (v3.2, schema/v3-shared-registry.lua)
  local registry = load_registry(opts)
  if registry then
    local declared_tags = {}
    for _, st in pairs(statuses) do
      for _, tag in ipairs(type(st.def.tags) == "table" and st.def.tags or {}) do declared_tags[tag] = true end
    end
    for _, st in pairs(statuses) do
      for _, x in ipairs(type(st.def.immune_to) == "table" and st.def.immune_to or {}) do
        local tag = type(x) == "string" and x:match("^tag:(.+)$")
        if tag then tag_reads[#tag_reads + 1] = { tag = tag, path = st.path .. ".immune_to" } end
      end
    end
    local reported = {}
    for _, r in ipairs(tag_reads) do
      if not declared_tags[r.tag] and not (registry.status_tags or {})[r.tag] and not reported[r.tag .. r.path] then
        reported[r.tag .. r.path] = true
        err(r.path, "狀態 tag `" .. r.tag .. "` 沒有在這份文件宣告，也不在 v3-shared-registry.lua 的 status_tags（跨文件的 tag 要登錄）")
      end
    end
    for _, r in ipairs(item_tag_uses) do
      if not (registry.item_tags or {})[r.tag] then
        err(r.path, "道具 tag `" .. r.tag .. "` 要登錄在 v3-shared-registry.lua 的 item_tags")
      end
    end
    -- shared unit_state (scope hero / unit) needs a registered name or a private prefix
    local function check_state(t, path, stems)
      for _, n in ipairs(sorted_keys(type(t) == "table" and t or {})) do
        local d = t[n]
        if type(d) == "table" and (d.scope == "hero" or d.scope == "unit" or d.scope == "match") then
          local reg = (registry.unit_state or {})[n]
          if reg then
            if reg.type ~= d.type then err(path .. "." .. n .. ".type", "登錄表的 `" .. n .. "` 是 " .. tostring(reg.type)) end
          else
            local private = false
            -- private: starts with the document or entry stem (c01r_noncrit in C01, ninja_cloth in item_ninja_cloth)
            for _, s in ipairs(stems) do if n:sub(1, #s) == s then private = true end end
            if not private then
              err(path .. "." .. n, "scope = " .. d.scope .. " 的變數同名就跨文件共用：共用的要登錄在 v3-shared-registry.lua，私有的請加前綴（"
                .. table.concat(stems, "_、") .. "_）")
            end
          end
        end
      end
    end
    local doc_stem = type(doc.id) == "string" and doc.id:lower() or "doc"
    check_state(doc.unit_state, "unit_state", { doc_stem })
    for _, e in ipairs(entry_list) do
      local stems = { doc_stem, e.def.id }
      if type(e.def.source_id) == "string" then stems[#stems + 1] = e.def.source_id:lower() end
      if type(e.def.id) == "string" and e.def.id:match("^item_") then stems[#stems + 1] = e.def.id:sub(6) end
      check_state(e.def.unit_state, e.path .. ".unit_state", stems)
      local function groups(c, p)
        for i, g in ipairs(type(c) == "table" and type(c.cost) == "table" and type(c.cost.cooldown_groups) == "table" and c.cost.cooldown_groups or {}) do
          if type(g) == "table" and g.group and not (registry.cooldown_groups or {})[g.group] then
            err(p .. ".cost.cooldown_groups[" .. i .. "].group", "共用冷卻組 `" .. tostring(g.group) .. "` 要登錄在 v3-shared-registry.lua（名稱用 <系列>_family）")
          end
        end
      end
      groups(e.def, e.path)
      groups(e.def.active, e.path .. ".active")
    end
  end

  -------------------------------------------------------------------------
  -- single source (value-single-source §5)
  -------------------------------------------------------------------------
  for _, id in ipairs(sorted_keys(owners)) do
    local o = owners[id]
    if type(o.values) == "table" then
      for _, n in ipairs(sorted_keys(o.values)) do
        local d, key, p = o.values[n], vkey(o, n), o.path .. ".values." .. n
        if type(d) == "table" then
          if d.v2_only then
            -- M1: a v2 number nothing reads, kept only so the v2 copy can be checked (removed in M5)
            if not d.hidden then err(p, "v2_only 的值必須是 hidden") end
            if not d.v2_key then err(p, "v2_only 的值必須有 v2_key") end
          elseif not uses[key] then err(p, "沒有人用：行為、說明、資訊列都沒有引用這個值（刪掉，或補上使用處）")
          elseif not d.hidden and not visible[key] then
            err(p, "玩家看不到：不是 hidden，但說明與資訊列都沒有出現（加進說明 {" .. n .. "}，或標 hidden = true）")
          end
        end
      end
    end
  end

  return errors, warnings
end

-------------------------------------------------------------------------
-- loading
-------------------------------------------------------------------------
local function is_dir(path)
  local f = io.open(path .. "/v3.lua", "rb")
  if f then f:close(); return true end
  return false
end

function M.load_file(path)
  local chunk, lerr = loadfile(path, "t", {})
  if not chunk then return nil, "載入失敗：" .. tostring(lerr) end
  local ok, fn = pcall(chunk)
  if not ok then return nil, "執行失敗：" .. tostring(fn) end
  if type(fn) ~= "function" then return nil, "必須 return function(ctx)" end
  local ok2, doc = pcall(fn, {})
  if not ok2 then return nil, "呼叫失敗：" .. tostring(doc) end
  if type(doc) ~= "table" then return nil, "function(ctx) 必須回傳 table" end
  return doc
end

function M.validate_file(path, opts)
  local target = path
  if not path:match("%.lua$") and is_dir(path) then target = path .. "/v3.lua" end
  local doc, lerr = M.load_file(target)
  if not doc then return { "SchemaError: " .. target .. ": " .. lerr }, {} end
  return M.validate_doc(doc, opts)
end

-- true when path is a v3 document (a .lua file or a dir holding v3.lua with schema_version 3)
function M.is_v3(path)
  local target = path
  if not path:match("%.lua$") then
    if not is_dir(path) then return false end
    target = path .. "/v3.lua"
  end
  local doc = M.load_file(target)
  return doc ~= nil and doc.schema_version == V.SCHEMA_VERSION
end

-------------------------------------------------------------------------
-- vocabulary as Markdown (for v3-authoring.md)
-------------------------------------------------------------------------
function M.vocab_md()
  local out = {}
  local function w(s) out[#out + 1] = s end
  local function keys(t) return table.concat(sorted_keys(t), "`、`") end
  w("<!-- 由 `tools\\lua\\lua.exe docs/character_pipeline/tools/package_v3.lua --vocab-md` 產生；不要手改 -->")
  w("# package schema v" .. V.SCHEMA_REVISION .. " 詞彙表")
  w("")
  w("## 各層欄位")
  w("")
  for _, row in ipairs({ { "文件", V.DOC_FIELDS }, { "hero", V.HERO_FIELDS }, { "values.*", V.VALUE_FIELDS },
    { "技能", V.ABILITY_FIELDS }, { "target", V.TARGET_FIELDS }, { "cast", V.CAST_FIELDS }, { "cost", V.COST_FIELDS },
    { "cost.charges", V.CHARGES_FIELDS }, { "requires", V.REQUIRES_FIELDS }, { "passive", V.PASSIVE_FIELDS },
    { "道具", V.ITEM_FIELDS }, { "shop", V.SHOP_FIELDS }, { "道具 active", V.ITEM_ACTIVE_FIELDS },
    { "狀態", V.STATUS_FIELDS }, { "stacking（table）", V.STACK_FIELDS }, { "absorb", V.ABSORB_FIELDS },
    { "aura", V.AURA_FIELDS }, { "triggers.*（table）", V.TRIGGER_FIELDS }, { "selector", V.SELECTOR_FIELDS },
    { "單位模板", V.UNIT_TEMPLATE_FIELDS }, { "單位模板 attack", V.UNIT_ATTACK_FIELDS },
    { "create_unit.overrides", V.UNIT_OVERRIDES }, { "create_illusion.copy", V.ILLUSION_COPY_FIELDS },
    { "create_illusion.outgoing／incoming", V.ILLUSION_DAMAGE_KEYS }, { "unit_state.*／vars.*", V.STATE_FIELDS },
    { "hooks.*", V.HOOK_FIELDS } }) do
    w("- " .. row[1] .. "：`" .. keys(row[2]) .. "`")
  end
  w("")
  w("## 數值單位（values.*.unit）")
  w("")
  w("| unit | 意義 |")
  w("|---|---|")
  for _, k in ipairs(sorted_keys(V.UNITS)) do w("| `" .. k .. "` | " .. V.UNITS[k] .. " |") end
  w("")
  w("## 出處前綴（values.*.source）")
  w("")
  for _, k in ipairs(sorted_keys(V.SOURCE_PREFIXES)) do w("- `" .. k .. ":` " .. V.SOURCE_PREFIXES[k]) end
  w("")
  w("## quirk 政策（quirks[].policy）")
  w("")
  for _, k in ipairs(sorted_keys(V.QUIRK_POLICIES)) do w("- `" .. k .. "`：" .. V.QUIRK_POLICIES[k]) end
  w("")
  w("## 揭露政策（reveal）")
  w("")
  for _, k in ipairs(sorted_keys(V.REVEAL)) do w("- `" .. k .. "`：" .. V.REVEAL[k]) end
  w("")
  w("## 說明格式（{name:fmt}）")
  w("")
  for _, k in ipairs(sorted_keys(V.TEXT_FORMATS)) do w("- `" .. k .. "`：" .. V.TEXT_FORMATS[k]) end
  w("")
  w("## Action")
  w("")
  w("`!` 是必填。參數型別：expr 數值運算式、cond 條件、unit／point／vec／any 該型別的運算式、actions 巢狀 action、selector、status／ability／template／routine／hook 已宣告的 id、cue 呈現 id、name 綁定名稱、bool、enum:a|b、list:型別、table 名稱→運算式。")
  w("")
  w("| op | 參數 |")
  w("|---|---|")
  for _, op in ipairs(sorted_keys(V.ACTIONS)) do
    local ps = {}
    for _, k in ipairs(sorted_keys(V.ACTIONS[op])) do ps[#ps + 1] = "`" .. k .. "`=" .. V.ACTIONS[op][k] end
    w("| `" .. op .. "` | " .. (#ps > 0 and table.concat(ps, "，") or "（無）") .. " |")
  end
  w("")
  w("## 技能事件（events）")
  w("")
  for _, k in ipairs(sorted_keys(V.ABILITY_EVENTS)) do
    local ps = sorted_keys(V.ABILITY_EVENTS[k])
    w("- `" .. k .. "`" .. (#ps > 0 and ("：event." .. table.concat(ps, "、event.")) or ""))
  end
  w("")
  w("## 狀態觸發（triggers）")
  w("")
  for _, k in ipairs(sorted_keys(V.STATUS_EVENTS)) do
    local ps = sorted_keys(V.STATUS_EVENTS[k])
    w("- `" .. k .. "`" .. (#ps > 0 and ("：event." .. table.concat(ps, "、event.")) or ""))
  end
  w("")
  w("事件紀錄欄位：")
  w("")
  for _, k in ipairs(sorted_keys(V.EVENT_RECORDS)) do w("- `" .. k .. "`：`" .. keys(V.EVENT_RECORDS[k]) .. "`") end
  w("")
  w("## 運算式函式")
  w("")
  w("| 函式 | 參數 | 回傳 |")
  w("|---|---|---|")
  for _, k in ipairs(sorted_keys(V.FUNCTIONS)) do
    w("| `" .. k .. "` | " .. table.concat(V.FUNCTIONS[k].args, ", ") .. " | " .. V.FUNCTIONS[k].ret .. " |")
  end
  w("")
  w("## 屬性")
  w("")
  w("- 單位（`caster.x`、`it.x`…）：`" .. keys(V.UNIT_PROPS) .. "`")
  w("- 技能（`ability.x`）：`" .. keys(V.ABILITY_PROPS) .. "`")
  w("- 狀態（`status.x`）：`" .. keys(V.STATUS_PROPS) .. "`")
  w("- 道具：`" .. keys(V.ITEM_PROPS) .. "`")
  w("- 對局（`game.x`）：`" .. keys(V.GAME_PROPS) .. "`")
  w("- 投射物：`" .. keys(V.PROJECTILE_PROPS) .. "`")
  w("- 位置：`" .. keys(V.POINT_PROPS) .. "`")
  w("")
  w("## StatKey（stats）")
  w("")
  w("`" .. keys(V.STATS) .. "`")
  w("")
  w("stat 運算子（`{ op, value, when, per_stack }`）：`" .. keys(V.STAT_OPS) .. "`")
  w("")
  w("## 控制與狀態旗標（states）")
  w("")
  w("`" .. keys(V.STATES) .. "`")
  w("")
  w("## 選取")
  w("")
  local shapes = {}
  for _, k in ipairs(sorted_keys(V.SHAPES)) do shapes[#shapes + 1] = "`" .. k .. "`(" .. table.concat(V.SHAPES[k], ", ") .. ")" end
  w("- 形狀：" .. table.concat(shapes, "、"))
  w("- team：`" .. keys(V.TEAMS) .. "`")
  w("- types：`" .. keys(V.UNIT_TYPES) .. "`")
  w("- flags：`" .. keys(V.SELECT_FLAGS) .. "`")
  w("- order：`" .. keys(V.ORDERS) .. "`")
  w("- 地標（landmark）：`" .. keys(V.LANDMARKS) .. "`；forced_move.locks：`" .. keys(V.ORDER_LOCKS) .. "`")
  w("")
  w("## 其他列舉")
  w("")
  w("- slot：`" .. keys(V.SLOTS) .. "`；kind：`" .. keys(V.ABILITY_KINDS) .. "`；tags：`" .. keys(V.ABILITY_TAGS) .. "`")
  w("- target.type：`" .. keys(V.TARGET_TYPES) .. "`；技能 flags：`" .. keys(V.ABILITY_FLAGS) .. "`")
  w("- cooldown_start：`" .. keys(V.COOLDOWN_STARTS) .. "`")
  w("- stacking：`" .. keys(V.STACKING_MODES) .. "` 或 `{ max, mode, value_stacks, per_stack_duration, decay, initial }`")
  w("- dispel：`" .. keys(V.DISPEL) .. "`；visibility：`" .. keys(V.VISIBILITY) .. "`；status scope：`" .. keys(V.STATUS_SCOPES) .. "`")
  w("- unit_state／vars type：`" .. keys(V.STATE_TYPES) .. "`；scope：`" .. keys(V.STATE_SCOPES) .. "`")
  w("- 單位模板 ai：`" .. keys(V.UNIT_AI) .. "`；over_limit：`" .. keys(V.OVER_LIMIT) .. "`；stats：`" .. keys(V.UNIT_STATS) .. "`")
  w("- coverage：`" .. keys(V.COVERAGE) .. "`；confidence：`" .. keys(V.CONFIDENCE) .. "`")
  w("- 規則層暫列（M2 會移進 game_rules.lua）：" .. (function()
    local t = {}
    for a, sub in pairs(V.RULES_PENDING) do for b in pairs(sub) do t[#t + 1] = "`rules." .. a .. "." .. b .. "`" end end
    table.sort(t)
    return table.concat(t, "、")
  end)())
  return table.concat(out, "\n") .. "\n"
end

-------------------------------------------------------------------------
-- CLI
-------------------------------------------------------------------------
if rawget(_G, "PACKAGE_V3_AS_MODULE") == true or not arg or arg[0] == nil
  or not tostring(arg[0]):match("package_v3%.lua$") then return M end

if arg[1] == "--vocab-md" then
  io.write(M.vocab_md())
  os.exit(0)
end
if #arg == 0 then
  io.stderr:write("usage: lua package_v3.lua <file.lua|package_dir> ... | --vocab-md\n")
  os.exit(2)
end
local failed = 0
for _, p in ipairs(arg) do
  local errs, warns = M.validate_file(p)
  for _, w in ipairs(warns) do io.stderr:write(p .. ": " .. w .. "\n") end
  if #errs > 0 then
    failed = failed + 1
    for _, e in ipairs(errs) do io.stderr:write(p .. ": " .. e .. "\n") end
    io.stderr:write(p .. ": FAIL（" .. #errs .. " 個錯誤）\n")
  else
    print("OK: " .. p)
  end
end
os.exit(failed == 0 and 0 or 1)
