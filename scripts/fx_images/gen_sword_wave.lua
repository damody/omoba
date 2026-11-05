-- gen_sword_wave.lua OUT: a crescent of sword light on a black square, curved forward (top): white core, blue edge, thin at both tips. Uncompressed PNG, no libraries.
local out = assert(arg[1])
local N = 512
local function clamp(x) return x < 0 and 0 or x > 1 and 1 or x end
local function smooth(a, b, x) local t = clamp((x - a) / (b - a)); return t * t * (3 - 2 * t) end
local CX, CY, R, SPREAD = 256, 440, 300, 0.95
local rows = {}
for y = 0, N - 1 do
  local row = { "\0" }
  for x = 0, N - 1 do
    local dx, dy = x + 0.5 - CX, y + 0.5 - CY
    local angle = math.atan(dx, -dy)            -- 0 straight forward
    local along = math.abs(angle) / SPREAD      -- 0 at the middle, 1 at a tip
    local r, g, b = 0, 0, 0
    if along < 1 then
      local half = 34 * (1 - along * along)     -- thickest in the middle
      local d = math.abs(math.sqrt(dx * dx + dy * dy) - R) / math.max(half, 0.001)
      local body = 1 - smooth(0.7, 1.0, d)
      local core = 1 - smooth(0.2, 0.55, d)
      r = 0.10 * body + 0.90 * core
      g = 0.42 * body + 0.58 * core
      b = 1.00 * body
      b = b + (1 - b) * core
    end
    local a = math.max(r, g, b)
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
