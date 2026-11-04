-- gen_thrust.lua OUT: a straight spear thrust on a transparent square: dark bronze edge, gold body, white core, brightest at the tip (top). Uncompressed PNG, no library.
local out = assert(arg[1])
-- Optional second argument "blue": the same shape in blue, navy edge, white core.
local BLUE = arg[2] == "blue"
local N = 512
local function clamp(x) return x < 0 and 0 or x > 1 and 1 or x end
local function smooth(a, b, x) local t = clamp((x - a) / (b - a)); return t * t * (3 - 2 * t) end
local rows = {}
for y = 0, N - 1 do
  local v = (y + 0.5) / N            -- 0 at the tip end
  local row = { "\0" }
  for x = 0, N - 1 do
    local u = math.abs((x + 0.5) / N - 0.5) * 2
    -- A blade: from the point at v=.04 out to its widest at v=.22, then a shaft narrowing to the tail at v=.98.
    local half
    if v < 0.04 or v > 0.98 then half = 0
    elseif v < 0.22 then half = 0.17 * (v - 0.04) / 0.18
    else half = 0.17 * (1 - (v - 0.22) / 0.76) ^ 1.5 + 0.03 * (1 - (v - 0.22) / 0.76) end
    local a, r, g, b = 0, 0, 0, 0
    if half > 0 and u < half then
      local d = u / half             -- 0 centre, 1 edge
      local tip = 1 - smooth(0.05, 0.6, v)
      local core = 1 - smooth(0.25, 0.6, d)
      local body = 1 - smooth(0.6, 0.82, d)
      -- edge colour -> body gold -> core white-gold
      r, g, b = 0.20, 0.09, 0.02
      if BLUE then r, g, b = 0.02, 0.06, 0.25 end
      local br, bg, bb = 1.00, 0.72, 0.10
      if BLUE then br, bg, bb = 0.15, 0.55, 1.00 end
      r = r + (br - r) * body; g = g + (bg - g) * body; b = b + (bb - b) * body
      local w = clamp(core * (0.55 + 0.45 * tip))
      local wr, wg, wb = 1.00, 0.97, 0.78
      if BLUE then wr, wg, wb = 0.85, 0.97, 1.00 end
      r = r + (wr - r) * w; g = g + (wg - g) * w; b = b + (wb - b) * w
      a = 1 - smooth(0.92, 1.0, d)
      a = a * smooth(0.98, 0.72, v) ^ 0.5
    end
    row[#row + 1] = string.char(math.floor(r * 255 + 0.5), math.floor(g * 255 + 0.5), math.floor(b * 255 + 0.5), math.floor(a * 255 + 0.5))
  end
  rows[#rows + 1] = table.concat(row)
end
local raw = table.concat(rows)
local crc_table = {}
for i = 0, 255 do local c = i; for _ = 1, 8 do c = (c & 1) ~= 0 and (0xEDB88320 ~ (c >> 1)) or (c >> 1) end; crc_table[i] = c end
local function crc32(s) local c = 0xFFFFFFFF; for i = 1, #s do c = crc_table[(c ~ s:byte(i)) & 255] ~ (c >> 8) end; return (c ~ 0xFFFFFFFF) & 0xFFFFFFFF end
local function chunk(kind, data) return string.pack(">I4", #data) .. kind .. data .. string.pack(">I4", crc32(kind .. data)) end
local a, b = 1, 0
for i = 1, #raw do a = (a + raw:byte(i)) % 65521; b = (b + a) % 65521 end
local z = { "\x78\x01" }
local pos = 1
while pos <= #raw do
  local take = math.min(65535, #raw - pos + 1)
  z[#z + 1] = string.pack("<BI2I2", pos + take > #raw and 1 or 0, take, take ~ 0xFFFF) .. raw:sub(pos, pos + take - 1)
  pos = pos + take
end
z[#z + 1] = string.pack(">I4", (b << 16) | a)
local f = assert(io.open(out, "wb"))
f:write("\x89PNG\r\n\x1a\n", chunk("IHDR", string.pack(">I4I4BBBBB", N, N, 8, 6, 0, 0, 0)), chunk("IDAT", table.concat(z)), chunk("IEND", ""))
f:close()
print("wrote", out)
