-- Exact rational numbers for the site generator and the text renderer.
--
-- Values are authored as integers, decimal strings ("0.35") or fractions
-- ("19/101") (schema v3 §2). They are kept exact as n/d so that derived values,
-- display rounding and the Fixed64 raw (SCALE = 1024, omoba-sim fixed.rs) are
-- computed without floating point; UE's TooltipRenderer follows the same rules
-- (docs/character_pipeline/schema/v3-text-render-vectors.lua).
local M = {}
M.SCALE = 1024

local mt = { __index = M }

local function gcd(a, b)
  a, b = math.abs(a), math.abs(b)
  while b ~= 0 do a, b = b, a % b end
  return a
end

function M.new(n, d)
  d = d or 1
  assert(math.type(n) == "integer" and math.type(d) == "integer", "rational parts must be integers")
  assert(d ~= 0, "rational denominator is zero")
  if d < 0 then n, d = -n, -d end
  local g = gcd(n, d)
  if g > 1 then n, d = n // g, d // g end
  return setmetatable({ n = n, d = d }, mt)
end

function M.is(x) return getmetatable(x) == mt end

local function pow10(k) local p = 1; for _ = 1, k do p = p * 10 end; return p end

-- "12", "-0.35", "1.", "19/101", "1e-05"
local function from_string(s)
  s = s:match("^%s*(.-)%s*$")
  local a, b = s:match("^(%-?%d+)/(%d+)$")
  if a then return M.new(math.tointeger(tonumber(a)), math.tointeger(tonumber(b))) end
  local sign, int, frac, exp = s:match("^([%+%-]?)(%d*)%.?(%d*)[eE]?([%+%-]?%d*)$")
  if not sign or (int == "" and frac == "") then return nil end
  -- more than 9 decimals is far below Fixed64's 1/1024; round so n/d stays in int64
  local round_up = false
  if #frac > 9 then round_up = frac:sub(10, 10) >= "5"; frac = frac:sub(1, 9) end
  local n = math.tointeger(tonumber((int == "" and "0" or int) .. frac))
  assert(n, "number too large: " .. s)
  if round_up then n = n + 1 end
  local d = pow10(#frac)
  local e = exp ~= "" and math.tointeger(tonumber(exp)) or 0
  if e > 0 then n = n * pow10(e) elseif e < 0 then d = d * pow10(-e) end
  if sign == "-" then n = -n end
  return M.new(n, d)
end

-- integer, float (read through its shortest round-trip decimal), string or rational
function M.parse(x)
  if M.is(x) then return x end
  if math.type(x) == "integer" then return M.new(x, 1) end
  if math.type(x) == "float" then
    assert(x == x and x ~= math.huge and x ~= -math.huge, "non-finite number")
    if x == math.floor(x) and math.abs(x) < 2 ^ 53 then return M.new(math.tointeger(x), 1) end
    for digits = 1, 17 do
      local s = string.format("%." .. digits .. "g", x)
      if tonumber(s) == x then return from_string(s) end
    end
  end
  if type(x) == "string" then
    local q = from_string(x)
    if q then return q end
  end
  error("not a number: " .. tostring(x))
end

-- products must stay inside int64; checked in floating point before multiplying
local LIMIT = 2.0 ^ 62
local function prod(x, y)
  assert(math.abs(x + 0.0) * math.abs(y + 0.0) < LIMIT, "rational overflow")
  return x * y
end
function M.add(a, b) return M.new(prod(a.n, b.d) + prod(b.n, a.d), prod(a.d, b.d)) end
function M.sub(a, b) return M.new(prod(a.n, b.d) - prod(b.n, a.d), prod(a.d, b.d)) end
function M.mul(a, b) return M.new(prod(a.n, b.n), prod(a.d, b.d)) end
function M.div(a, b) assert(b.n ~= 0, "division by zero"); return M.new(prod(a.n, b.d), prod(a.d, b.n)) end
function M.neg(a) return M.new(-a.n, a.d) end
function M.abs(a) return M.new(math.abs(a.n), a.d) end
function M.cmp(a, b) local l, r = a.n * b.d, b.n * a.d; return l < r and -1 or (l > r and 1 or 0) end
function M.eq(a, b) return a.n == b.n and a.d == b.d end
function M.sign(a) return a.n > 0 and 1 or (a.n < 0 and -1 or 0) end
function M.floor(a) return M.new(a.n // a.d, 1) end
function M.ceil(a) return M.new(-((-a.n) // a.d), 1) end
-- Lua's % on rationals: a - floor(a/b)*b
function M.mod(a, b) return M.sub(a, M.mul(M.floor(M.div(a, b)), b)) end

-- nearest integer, halves away from zero
function M.round_int(a)
  local n, d = math.abs(a.n), a.d
  local q = (2 * n + d) // (2 * d)
  return a.n < 0 and -q or q
end

-- round to `places` decimals (halves away from zero)
function M.round(a, places)
  local p = pow10(places)
  return M.new(M.round_int(M.new(a.n * p, a.d)), p)
end

-- Fixed64 raw (SCALE 1024), nearest with halves away from zero
function M.to_raw(a) return M.round_int(M.new(a.n * M.SCALE, a.d)) end

-- "n" or "n/d": the exact form stored in the site catalog
function M.exact(a) return a.d == 1 and tostring(a.n) or (a.n .. "/" .. a.d) end

-- decimal with at most `places` digits after the point; no trailing zeros; never "-0"
function M.decimal(a, places)
  local r = M.round(a, places)
  local p = pow10(places)
  local scaled = r.n * (p // r.d)
  local neg = scaled < 0
  scaled = math.abs(scaled)
  local int, frac = scaled // p, scaled % p
  local s = tostring(int)
  if places > 0 and frac > 0 then
    local f = string.format("%0" .. places .. "d", frac):gsub("0+$", "")
    s = s .. "." .. f
  end
  if neg and s ~= "0" then s = "-" .. s end
  return s
end

function M.is_integer(a) return a.d == 1 end

mt.__tostring = M.exact

return M
