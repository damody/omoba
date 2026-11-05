-- gen_war_fan.lua OUT: a commander's war fan (gunbai) on a black square, handle down: gold paddle, bright rim, dark red centre mark. Uncompressed PNG, no libraries.
local out = assert(arg[1])
local N = 512
local function clamp(x) return x < 0 and 0 or x > 1 and 1 or x end
local function smooth(a, b, x) local t = clamp((x - a) / (b - a)); return t * t * (3 - 2 * t) end
local rows = {}
for y = 0, N - 1 do
  local row = { "\0" }
  for x = 0, N - 1 do
    local px, py = x + 0.5 - 256, y + 0.5 - 200
    local r, g, b = 0, 0, 0
    -- The paddle: a wide rounded gourd, narrowing towards the handle.
    local width = 170 - math.max(py, 0) * 0.42
    local d = math.sqrt((px / width) ^ 2 + (py / 170) ^ 2)
    if d < 1 then
      local rim = smooth(0.84, 0.9, d)
      r, g, b = 1.0, 0.68, 0.12
      -- Ribs fanning out from the neck.
      local rib = math.abs(math.sin(math.atan(px, 200 - py) * 16))
      local shade = 0.72 + 0.28 * smooth(0.0, 0.25, rib)
      r, g, b = r * shade, g * shade, b * shade
      -- A red disc in the middle, the commander's mark.
      local mark = 1 - smooth(50, 56, math.sqrt(px * px + (py + 20) ^ 2))
      r = r + (0.92 - r) * mark; g = g + (0.14 - g) * mark; b = b + (0.08 - b) * mark
      -- A pale gold rim.
      r = r + (1.0 - r) * rim; g = g + (0.93 - g) * rim; b = b + (0.62 - b) * rim
      local edge = 1 - smooth(0.97, 1.0, d)
      r, g, b = r * edge, g * edge, b * edge
    end
    -- The handle.
    if x >= 244 and x < 268 and y >= 330 and y < 492 then
      local fade = (1 - smooth(486, 492, y + 0.5)) * (1 - smooth(9, 12, math.abs(px)))
      r = math.max(r, 0.95 * fade); g = math.max(g, 0.62 * fade); b = math.max(b, 0.14 * fade)
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
