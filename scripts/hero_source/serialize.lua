-- Deterministic Lua table serializer: sorted keys, stable array order, UTF-8
-- bytes kept as-is so 繁中 text stays readable in the generated catalog.
local M = {}

local function quote(s)
  return '"' .. s:gsub('[%c"\\]', function(c)
    if c == "\n" then return "\\n" elseif c == "\r" then return "\\r" elseif c == "\t" then return "\\t"
    elseif c == '"' then return '\\"' elseif c == "\\" then return "\\\\" end
    return string.format("\\%03d", c:byte())
  end) .. '"'
end
M.quote = quote

-- A table is a sequence only when its keys are exactly 1..n; `#t` is not
-- reliable for tables with holes, which must keep explicit [i] keys.
local function is_array(t)
  local n, count = #t, 0
  for k in pairs(t) do
    if math.type(k) ~= "integer" or k < 1 or k > n then return false end
    count = count + 1
  end
  return count == n
end

local KEYWORDS = {}
for w in ([[and break do else elseif end false for function goto if in local nil not or repeat
  return then true until while]]):gmatch("%a+") do KEYWORDS[w] = true end

local function key_repr(k)
  if type(k) == "string" and k:match("^[%a_][%w_]*$") and not KEYWORDS[k] then return k end
  if type(k) == "string" then return "[" .. quote(k) .. "]" end
  return "[" .. tostring(k) .. "]"
end

local function value_repr(v)
  local t = type(v)
  if t == "string" then return quote(v)
  elseif t == "number" then
    if math.type(v) == "integer" then return tostring(v) end
    assert(v == v and v ~= math.huge and v ~= -math.huge, "non-finite number in catalog")
    local s = string.format("%.17g", v)
    if tonumber(string.format("%.15g", v)) == v then s = string.format("%.15g", v) end
    if not s:find("[%.eEn]") then s = s .. ".0" end
    return s
  elseif t == "boolean" then return tostring(v) end
  error("cannot serialize " .. t)
end

local function compare_keys(a, b)
  local ta, tb = type(a), type(b)
  if ta ~= tb then return ta < tb end
  return a < b
end

local function emit(out, v, indent)
  if type(v) ~= "table" then out[#out + 1] = value_repr(v); return end
  if next(v) == nil then out[#out + 1] = "{}"; return end
  local pad = string.rep("  ", indent + 1)
  local simple = is_array(v)
  if not simple then
    -- Small records of scalars go on one line to keep the catalog compact.
    local parts, keys, size = {}, {}, 0
    for k, x in pairs(v) do
      if type(x) == "table" then keys = nil break end
      keys[#keys + 1] = k
    end
    if keys then
      table.sort(keys, compare_keys)
      for i, k in ipairs(keys) do
        parts[i] = key_repr(k) .. " = " .. value_repr(v[k])
        size = size + #parts[i]
      end
      if size <= 240 then
        out[#out + 1] = "{ " .. table.concat(parts, ", ") .. " }"
        return
      end
    end
  end
  if simple then
    local flat = true
    for _, x in ipairs(v) do if type(x) == "table" then flat = false break end end
    if flat and #v <= 16 then
      local parts = {}
      for i, x in ipairs(v) do parts[i] = value_repr(x) end
      out[#out + 1] = "{ " .. table.concat(parts, ", ") .. " }"
      return
    end
    out[#out + 1] = "{\n"
    for _, x in ipairs(v) do
      out[#out + 1] = pad
      emit(out, x, indent + 1)
      out[#out + 1] = ",\n"
    end
  else
    local keys = {}
    for k in pairs(v) do keys[#keys + 1] = k end
    table.sort(keys, compare_keys)
    out[#out + 1] = "{\n"
    for _, k in ipairs(keys) do
      out[#out + 1] = pad .. key_repr(k) .. " = "
      emit(out, v[k], indent + 1)
      out[#out + 1] = ",\n"
    end
  end
  out[#out + 1] = string.rep("  ", indent) .. "}"
end

function M.serialize(v, header)
  local out = {}
  if header then out[#out + 1] = header end
  out[#out + 1] = "return "
  emit(out, v, 0)
  out[#out + 1] = "\n"
  return table.concat(out)
end

return M
