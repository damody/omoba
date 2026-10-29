-- Package schema v3 expression language (v3.md §5): one grammar shared by
-- values (`expr`), conditions and behavior parameters.
--
--   expr    := or
--   or      := and { "or" and }
--   and     := not { "and" not }
--   not     := "not" not | cmp
--   cmp     := sum [ ("=="|"~="|"<"|"<="|">"|">=") sum ]
--   sum     := prod { ("+"|"-") prod }
--   prod    := unary { ("*"|"/"|"%") unary }
--   unary   := "-" unary | postfix
--   postfix := primary { "." NAME }
--   primary := NUMBER | "true" | "false" | VALUE | NAME "(" [ expr { "," expr } ] ")" | NAME | "(" expr ")"
--   VALUE   := "$" NAME [ "." NAME ]          -- $damage, $c01_blood_flame.charge_max
--   NAME    := [A-Za-z_][A-Za-z0-9_]*
--   NUMBER  := digits [ "." digits ]
--
-- parse(src) -> ast | nil, err;  check(ast, env) -> type (errors go to env.err).
local V = dofile((debug.getinfo(1, "S").source:sub(2):match("^(.*)[/\\]") or ".") .. "/v3_vocab.lua")
local M = {}

local KEYWORDS = { ["and"] = true, ["or"] = true, ["not"] = true, ["true"] = true, ["false"] = true }

local function tokenize(src)
  local toks, i, n = {}, 1, #src
  while i <= n do
    local c = src:sub(i, i)
    if c:match("%s") then i = i + 1
    elseif c:match("%d") then
      local s, e = src:find("^%d+%.?%d*", i)
      local text = src:sub(s, e)
      if text:sub(-1) == "." then return nil, "數字格式錯誤：" .. text end
      toks[#toks + 1] = { t = "num", v = tonumber(text), text = text }
      i = e + 1
    elseif c == "$" then
      local s, e = src:find("^%$[A-Za-z_][A-Za-z0-9_]*", i)
      if not s then return nil, "`$` 後面要接數值名稱" end
      local name = src:sub(s + 1, e)
      local owner
      local s2, e2 = src:find("^%.[A-Za-z_][A-Za-z0-9_]*", e + 1)
      if s2 then owner, name, e = name, src:sub(s2 + 1, e2), e2 end
      toks[#toks + 1] = { t = "val", owner = owner, name = name }
      i = e + 1
    elseif c:match("[A-Za-z_]") then
      local s, e = src:find("^[A-Za-z_][A-Za-z0-9_]*", i)
      local w = src:sub(s, e)
      toks[#toks + 1] = KEYWORDS[w] and { t = w } or { t = "name", v = w }
      i = e + 1
    else
      local two = src:sub(i, i + 1)
      if two == "==" or two == "~=" or two == "<=" or two == ">=" or two == "!=" then
        toks[#toks + 1] = { t = two == "!=" and "~=" or two }
        i = i + 2
      elseif ("+-*/%(),.<>"):find(c, 1, true) then
        toks[#toks + 1] = { t = c }
        i = i + 1
      else
        return nil, "不認得的字元 `" .. c .. "`"
      end
    end
  end
  toks[#toks + 1] = { t = "eof" }
  return toks
end

function M.parse(src)
  if type(src) ~= "string" then return nil, "運算式必須是字串" end
  local toks, terr = tokenize(src)
  if not toks then return nil, terr end
  local pos = 1
  local function peek() return toks[pos].t end
  local function take() pos = pos + 1; return toks[pos - 1] end
  local function expect(t)
    if peek() ~= t then error({ msg = "預期 `" .. t .. "`，看到 `" .. peek() .. "`" }) end
    return take()
  end
  local expr
  local function primary()
    local tk = toks[pos]
    if tk.t == "num" then take(); return { k = "num", v = tk.v, text = tk.text } end
    if tk.t == "true" or tk.t == "false" then take(); return { k = "bool", v = tk.t == "true" } end
    if tk.t == "val" then take(); return { k = "val", owner = tk.owner, name = tk.name } end
    if tk.t == "name" then
      take()
      if peek() == "(" then
        take()
        local args = {}
        if peek() ~= ")" then
          args[#args + 1] = expr()
          while peek() == "," do take(); args[#args + 1] = expr() end
        end
        expect(")")
        return { k = "call", fn = tk.v, args = args }
      end
      return { k = "id", name = tk.v }
    end
    if tk.t == "(" then
      take()
      local e = expr()
      expect(")")
      return e
    end
    error({ msg = "預期運算元，看到 `" .. tk.t .. "`" })
  end
  local function postfix()
    local e = primary()
    while peek() == "." do
      take()
      local tk = expect("name")
      e = { k = "field", obj = e, name = tk.v }
    end
    return e
  end
  local function unary()
    if peek() == "-" then
      take()
      local e = unary()
      if e.k == "num" then return { k = "num", v = -e.v, text = "-" .. e.text } end
      return { k = "un", op = "-", e = e }
    end
    return postfix()
  end
  local function binary(next_fn, ops)
    return function()
      local l = next_fn()
      while ops[peek()] do
        local op = take().t
        l = { k = "bin", op = op, l = l, r = next_fn() }
      end
      return l
    end
  end
  local prod = binary(unary, { ["*"] = true, ["/"] = true, ["%"] = true })
  local sum = binary(prod, { ["+"] = true, ["-"] = true })
  local function cmp()
    local l = sum()
    local op = peek()
    if op == "==" or op == "~=" or op == "<" or op == "<=" or op == ">" or op == ">=" then
      take()
      return { k = "bin", op = op, l = l, r = sum() }
    end
    return l
  end
  local function lnot()
    if peek() == "not" then take(); return { k = "un", op = "not", e = lnot() } end
    return cmp()
  end
  local land = binary(lnot, { ["and"] = true })
  local lor = binary(land, { ["or"] = true })
  expr = lor
  local ok, res = pcall(function()
    local e = expr()
    if peek() ~= "eof" then error({ msg = "多出的內容 `" .. peek() .. "`" }) end
    return e
  end)
  if not ok then
    if type(res) == "table" then return nil, res.msg end
    error(res)
  end
  return res
end

-- Walks the AST, calling fn(node) on every node.
function M.walk(ast, fn)
  fn(ast)
  if ast.k == "call" then for _, a in ipairs(ast.args) do M.walk(a, fn) end
  elseif ast.k == "field" then M.walk(ast.obj, fn)
  elseif ast.k == "bin" then M.walk(ast.l, fn); M.walk(ast.r, fn)
  elseif ast.k == "un" then M.walk(ast.e, fn) end
end

local function compatible(have, want)
  if want == "any" or have == "any" or have == want then return true end
  if (want == "point" and have == "unit") then return true end -- a unit stands for its position
  return false
end

-- env: {
--   err(msg), value(owner, name) -> def|nil, root(name) -> kind|nil,
--   status(id), ability(id), template(id) -> bool, state(name) -> kind|nil,
--   var(name) -> kind|nil, event -> { param = kind }|nil, rules -> table|nil,
--   allow_numbers -> bool (values' own expr may use plain numbers? no: always structural) }
function M.check(ast, env)
  local function e(msg) env.err(msg); return "any" end
  local check
  local function prop(kind, name)
    local reg = ({ unit = V.UNIT_PROPS, ability = V.ABILITY_PROPS, point = V.POINT_PROPS, game = V.GAME_PROPS,
      item = V.ITEM_PROPS, status = V.STATUS_PROPS, projectile = V.PROJECTILE_PROPS })[kind]
    if kind == "any" then return "any" end
    if kind == "attack" or kind == "damage" or kind == "order" then
      local r = V.EVENT_RECORDS[kind][name]
      return r or e("`" .. kind .. "` 沒有欄位 `" .. name .. "`")
    end
    if not reg then return e("`" .. kind .. "` 型別的值不能取欄位 `" .. name .. "`") end
    return reg[name] or e(kind .. " 沒有屬性 `" .. name .. "`")
  end
  -- Resolves a.b.c paths rooted at rules / event / state / vars.
  local function path_of(node)
    local parts = {}
    while node.k == "field" do table.insert(parts, 1, node.name); node = node.obj end
    if node.k ~= "id" then return nil end
    table.insert(parts, 1, node.name)
    return parts
  end
  local function special_path(parts)
    local root = parts[1]
    if root == "rules" then
      local t = env.rules
      for i = 2, #parts do
        if type(t) ~= "table" or t[parts[i]] == nil then
          return e("規則層沒有 `" .. table.concat(parts, ".") .. "`（scripts/lua_data/templates/game_rules.lua 或 v3_vocab RULES_PENDING）")
        end
        t = t[parts[i]]
      end
      if type(t) == "table" then return e("`" .. table.concat(parts, ".") .. "` 不是單一數值") end
      return "num", true
    end
    return nil
  end
  check = function(node)
    if node.k == "num" then
      if not V.STRUCTURAL_NUMBERS[node.v] then
        return e("未命名的數字 `" .. node.text .. "`：只允許 0、1、-1，其他數字要寫進 values 再用 $名稱 引用")
      end
      return "num"
    elseif node.k == "bool" then return "bool"
    elseif node.k == "val" then
      local def = env.value(node.owner, node.name)
      if not def then
        return e("數值 `$" .. (node.owner and (node.owner .. ".") or "") .. node.name .. "` 不存在")
      end
      return "num"
    elseif node.k == "id" then
      local k = env.root(node.name)
      if not k then return e("名稱 `" .. node.name .. "` 在這裡不存在") end
      if k == "rules" then return e("`rules` 要接完整路徑，例如 rules.hero.base_magic_resistance") end
      return k
    elseif node.k == "field" then
      local parts = path_of(node)
      if parts then
        local sk = special_path(parts)
        if sk then return sk end
        local root = parts[1]
        if root == "event" and env.root("event") then
          local p = env.event and env.event[parts[2]]
          if not p then return e("這個事件沒有參數 `event." .. parts[2] .. "`") end
          local kind = p
          for i = 3, #parts do kind = prop(kind, parts[i]) end
          return kind
        end
        if root == "state" and env.root("state") and #parts >= 2 then
          local k = env.state(parts[2])
          if not k then return e("unit_state 沒有宣告 `" .. parts[2] .. "`") end
          for i = 3, #parts do k = prop(k, parts[i]) end
          return k
        end
        if root == "vars" and env.root("vars") and #parts >= 2 then
          local k = env.var(parts[2])
          if not k then return e("狀態 vars 沒有宣告 `" .. parts[2] .. "`") end
          for i = 3, #parts do k = prop(k, parts[i]) end
          return k
        end
      end
      local ok = check(node.obj)
      return prop(ok, node.name)
    elseif node.k == "un" then
      local t = check(node.e)
      if node.op == "not" then
        if not compatible(t, "bool") then e("`not` 需要條件") end
        return "bool"
      end
      return t
    elseif node.k == "bin" then
      local op = node.op
      local lt, rt = check(node.l), check(node.r)
      if op == "and" or op == "or" then
        if not compatible(lt, "bool") or not compatible(rt, "bool") then e("`" .. op .. "` 兩邊都要是條件") end
        return "bool"
      elseif op == "==" or op == "~=" then return "bool"
      elseif op == "<" or op == "<=" or op == ">" or op == ">=" then
        if not compatible(lt, "num") or not compatible(rt, "num") then e("`" .. op .. "` 兩邊都要是數值") end
        return "bool"
      else
        if lt == "bool" or rt == "bool" then e("`" .. op .. "` 不能用在條件上") end
        if lt == "num" and rt == "num" then return "num" end
        if lt == "point" or rt == "point" then return "point" end
        if lt == "vec" or rt == "vec" then return "vec" end
        return "any"
      end
    elseif node.k == "call" then
      local sig = V.FUNCTIONS[node.fn]
      if not sig then return e("沒有函式 `" .. node.fn .. "`") end
      local args = sig.args
      local variadic = args[#args] == "..."
      local fixed = variadic and #args - 1 or #args
      if (not variadic and #node.args ~= fixed) or (variadic and #node.args < fixed) then
        return e("`" .. node.fn .. "` 需要 " .. fixed .. (variadic and " 個以上" or " 個") .. "參數，給了 " .. #node.args)
      end
      local types = {}
      for i, a in ipairs(node.args) do
        local want = args[i] or (variadic and args[fixed]) or "any"
        if want == "status" or want == "template" or want == "tag" or want == "name"
          or want == "item_name" or want == "state_flag" or want == "landmark"
          or want == "status_tag" or want == "slot" or want == "order_kind"
          or (want == "ability" and a.k == "id" and env.ability(a.name))
          or (want == "team" and a.k == "id" and V.TEAMS[a.name]) then
          if a.k ~= "id" then
            if want ~= "ability" and want ~= "team" then e("`" .. node.fn .. "` 第 " .. i .. " 個參數要寫名稱") end
          elseif want == "status" and not env.status(a.name) then e("狀態 `" .. a.name .. "` 沒有宣告")
          elseif want == "template" and not env.template(a.name) then e("單位模板 `" .. a.name .. "` 沒有宣告")
          elseif want == "tag" and not (V.UNIT_TYPES[a.name] or V.ABILITY_TAGS[a.name]) then e("不認得的 tag `" .. a.name .. "`")
          elseif want == "item_name" and not a.name:match("^item_[a-z0-9_]+$") then e("`" .. a.name .. "` 不是道具 id（item_...，小寫）")
          elseif want == "state_flag" and not V.STATES[a.name] then e("不認得的狀態旗標 `" .. a.name .. "`")
          elseif want == "landmark" and not V.LANDMARKS[a.name] then e("不認得的地標 `" .. a.name .. "`（" .. table.concat((function() local t = {} for k in pairs(V.LANDMARKS) do t[#t + 1] = k end table.sort(t) return t end)(), "、") .. "）")
          elseif want == "name" and not a.name:match("^[a-z][a-z0-9_]*$") then e("`" .. a.name .. "` 必須是 snake_case 名稱")
          elseif want == "status_tag" then if env.status_tag then env.status_tag(a.name) end
          elseif want == "slot" and not V.SLOTS[a.name] then e("不認得的技能格 `" .. a.name .. "`")
          elseif want == "order_kind" and not V.ORDER_KINDS[a.name] then e("不認得的指令種類 `" .. a.name .. "`")
          end
          types[i] = want
        else
          local t = check(a)
          if not compatible(t, want) and want ~= "ability" and want ~= "team" then
            e("`" .. node.fn .. "` 第 " .. i .. " 個參數要是 " .. want .. "，這裡是 " .. t)
          end
          types[i] = t
        end
      end
      if node.fn == "hook_value" and env.hook then env.hook(node.args[1]) end
      if sig.ret == "same" then return types[2] or "any" end
      return sig.ret
    end
    return "any"
  end
  return check(ast)
end

-- Collects every $value reference: returns list of {owner, name}.
function M.value_refs(ast)
  local out = {}
  M.walk(ast, function(n) if n.k == "val" then out[#out + 1] = { owner = n.owner, name = n.name } end end)
  return out
end

return M
