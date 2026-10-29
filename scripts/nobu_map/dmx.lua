-- Read-only parser for Source 2 binary DMX (`<!-- dmx encoding binary 9 format ... -->`), the format
-- of history_nobu's .vmap files. Lua 5.4 string.unpack only; nothing is executed or written.
--
--   local dmx = require("nobu_map.dmx")
--   local doc = dmx.parse(bytes)   -- { encoding, version, format, format_version, prefix, elements, root }
--
-- doc.elements[i] = { type = "CMapEntity", name = "...", guid = "<32 hex>", attrs = { [name] = value } }
-- Attribute values: numbers, booleans, strings, element references (the element table, or false for
-- a null reference), { x, y, z } style arrays for vectors, and Lua arrays for array types.
-- Layout (verified against nobu.vmap: every byte is consumed):
--   header text, NUL; prefix: int32 count, then per prefix element int32 attribute count and
--   attributes with inline names and inline string values; string table: int32 count, NUL-terminated
--   strings; element headers: int32 count, then (int32 type string, int32 name string, 16-byte GUID);
--   element bodies: int32 attribute count, then (int32 name string, uint8 type, value).
--   Scalar strings in element bodies are string-table indices; strings inside arrays are inline.
-- Attribute types (Source 2 datamodel): 1 element, 2 int, 3 float, 4 bool, 5 string, 6 binary,
-- 7 time, 8 color, 9 vector2, 10 vector3, 11 vector4, 12 qangle, 13 quaternion, 14 matrix,
-- 15 uint64, 16 uint8; an array of type t is t + 32.
local M = {}

M.TYPES = { "element", "int", "float", "bool", "string", "binary", "time", "color", "vector2", "vector3",
  "vector4", "qangle", "quaternion", "matrix", "uint64", "uint8" }
M.ARRAY_OFFSET = 32

local unpack = string.unpack

local function reader(d, pos)
  local r = { d = d, pos = pos }
  function r.i32() local v; v, r.pos = unpack("<i4", d, r.pos); return v end
  function r.u8() local v; v, r.pos = unpack("<B", d, r.pos); return v end
  function r.f32() local v; v, r.pos = unpack("<f", d, r.pos); return v end
  function r.u64() local v; v, r.pos = unpack("<I8", d, r.pos); return v end
  function r.cstr() local v; v, r.pos = unpack("z", d, r.pos); return v end
  function r.bytes(n) local v = d:sub(r.pos, r.pos + n - 1); r.pos = r.pos + n; return v end
  return r
end

-- one value of base type t; strings come from `str` (table index or inline)
local function value(r, t, str, inline_strings)
  if t == 1 then
    local idx = r.i32()
    if idx == -2 then return { external_guid = r.cstr() } end
    return { ref = idx }
  elseif t == 2 then return r.i32()
  elseif t == 3 then return r.f32()
  elseif t == 4 then return r.u8() ~= 0
  elseif t == 5 then
    if inline_strings then return r.cstr() end
    local idx = r.i32()
    return idx >= 0 and str[idx + 1] or ""
  elseif t == 6 then local n = r.i32(); return { binary_length = n, offset = (function() local p = r.pos; r.pos = r.pos + n; return p end)() }
  elseif t == 7 then return r.i32() / 10000
  elseif t == 8 then return { r.u8(), r.u8(), r.u8(), r.u8() }
  elseif t == 9 then return { r.f32(), r.f32() }
  elseif t == 10 or t == 12 then return { r.f32(), r.f32(), r.f32() }
  elseif t == 11 or t == 13 then return { r.f32(), r.f32(), r.f32(), r.f32() }
  elseif t == 14 then local m = {} for i = 1, 16 do m[i] = r.f32() end return m
  elseif t == 15 then return r.u64()
  elseif t == 16 then return r.u8()
  end
  error(("unknown DMX attribute type %d at byte %d"):format(t, r.pos))
end

local function attribute(r, t, str, inline_scalar_strings)
  if t > M.ARRAY_OFFSET then
    local base = t - M.ARRAY_OFFSET
    local n = r.i32()
    local out = {}
    for i = 1, n do out[i] = value(r, base, str, true) end
    return out, M.TYPES[base] .. "_array"
  end
  return value(r, t, str, inline_scalar_strings), M.TYPES[t]
end

-- bytes -> document (see the header comment)
function M.parse(d)
  local header_end = assert(d:find("\0", 1, true), "no DMX header")
  local header = d:sub(1, header_end - 1)
  local enc, ver, fmt, fver = header:match("<!%-%-%s*dmx%s+encoding%s+(%S+)%s+(%d+)%s+format%s+(%S+)%s+(%d+)%s*%-%->")
  assert(enc == "binary", "not a binary DMX: " .. header)
  ver = tonumber(ver)
  assert(ver == 9, "binary DMX version " .. tostring(ver) .. " is not supported (expected 9)")
  local r = reader(d, header_end + 1)
  local doc = { encoding = enc, version = ver, format = fmt, format_version = tonumber(fver), prefix = {}, elements = {} }
  -- prefix elements: inline names and strings
  for _ = 1, r.i32() do
    local attrs = {}
    for _ = 1, r.i32() do
      local name = r.cstr()
      local t = r.u8()
      attrs[name] = attribute(r, t, nil, true)
    end
    doc.prefix[#doc.prefix + 1] = attrs
  end
  local str = {}
  for i = 1, r.i32() do str[i] = r.cstr() end
  doc.strings = str
  local n = r.i32()
  for i = 1, n do
    local ty, nm = r.i32(), r.i32()
    local guid = r.bytes(16)
    doc.elements[i] = { type = str[ty + 1], name = str[nm + 1], guid = (guid:gsub(".", function(c) return ("%02x"):format(c:byte()) end)), attrs = {}, index = i }
  end
  for i = 1, n do
    local el = doc.elements[i]
    for _ = 1, r.i32() do
      local name = str[r.i32() + 1]
      local t = r.u8()
      el.attrs[name] = attribute(r, t, str, false)
    end
  end
  assert(r.pos == #d + 1, ("DMX parse ended at byte %d of %d"):format(r.pos - 1, #d))
  -- resolve element references
  local function resolve(v)
    if type(v) ~= "table" then return v end
    if v.ref ~= nil and next(v, next(v)) == nil then
      if v.ref < 0 then return false end
      return assert(doc.elements[v.ref + 1], "dangling element reference " .. v.ref)
    end
    for k, x in pairs(v) do
      if type(x) == "table" and x.ref ~= nil and next(x, next(x)) == nil then v[k] = resolve(x) end
    end
    return v
  end
  for _, el in ipairs(doc.elements) do
    for k, v in pairs(el.attrs) do el.attrs[k] = resolve(v) end
  end
  doc.root = doc.elements[1]
  return doc
end

return M
