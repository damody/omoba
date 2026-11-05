-- gen_pulse_ring.lua OUT: a thick ring on black, soft outside, hot along its inner edge, so a pulse reads as a band and not a line. Uncompressed PNG, no libraries.
local out = assert(arg[1])
local N = 512
local function clamp(x) return x < 0 and 0 or x > 1 and 1 or x end
local function smooth(a, b, x) local t = clamp((x - a) / (b - a)); return t * t * (3 - 2 * t) end
local rows = {}
for y = 0, N - 1 do
  local row = { "\0" }
  for x = 0, N - 1 do
    local px, py = (x + 0.5 - 256) / 256, (y + 0.5 - 256) / 256
    local d = math.sqrt(px * px + py * py)
    -- The band: a sharp inner edge at 0.66, fading out to 0.98.
    local band = smooth(0.64, 0.68, d) * (1 - smooth(0.82, 0.98, d))
    -- Licks of unequal strength round it, so it is not a clean line.
    local licks = 0.82 + 0.18 * math.sin(math.atan(py, px) * 11 + math.sin(math.atan(py, px) * 5) * 2)
    local v = band * licks
    local hot = (1 - smooth(0.66, 0.78, d)) * band
    local r, g, b = v, v * (0.55 + 0.45 * hot), v * (0.2 + 0.6 * hot)
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
