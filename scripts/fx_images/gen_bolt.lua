-- gen_bolt.lua OUT: a thin jagged bolt across a black square, left to right: white core, blue halo, two short branches. Uncompressed PNG, no libraries.
local out = assert(arg[1])
local N = 512
local function clamp(x) return x < 0 and 0 or x > 1 and 1 or x end
local function smooth(a, b, x) local t = clamp((x - a) / (b - a)); return t * t * (3 - 2 * t) end
-- The zigzag: sharp turns of alternating side, fixed so the image is the same on every run.
local main = { { 0, 256 }, { 52, 216 }, { 96, 282 }, { 148, 208 }, { 196, 292 }, { 246, 226 }, { 298, 298 }, { 348, 214 }, { 398, 280 }, { 452, 228 }, { 512, 256 } }
local branches = {
  { { 148, 208 }, { 176, 150 }, { 160, 118 }, { 196, 74 } },
  { { 298, 298 }, { 330, 352 }, { 312, 388 }, { 352, 434 } },
}
local function segments(line, into, weight)
  for i = 1, #line - 1 do into[#into + 1] = { line[i][1], line[i][2], line[i + 1][1], line[i + 1][2], weight } end
end
local segs = {}
segments(main, segs, 1)
for _, b in ipairs(branches) do segments(b, segs, 0.6) end
-- Distance from the point to the nearest segment, a branch counting as further than it is, so it stays thinner.
local function distance(x, y)
  local best = 1e9
  for _, s in ipairs(segs) do
    local dx, dy = s[3] - s[1], s[4] - s[2]
    local t = clamp(((x - s[1]) * dx + (y - s[2]) * dy) / (dx * dx + dy * dy))
    local px, py = s[1] + dx * t - x, s[2] + dy * t - y
    local d = math.sqrt(px * px + py * py) / s[5]
    if d < best then best = d end
  end
  return best
end
local rows = {}
for y = 0, N - 1 do
  local row = { "\0" }
  for x = 0, N - 1 do
    local d = distance(x + 0.5, y + 0.5)
    local core = 1 - smooth(2.4, 4.0, d)
    local halo = 1 - smooth(3.0, 13.0, d)
    local r = 0.10 * halo + (1 - 0.10 * halo) * core
    local g = 0.42 * halo + (1 - 0.42 * halo) * core
    local b = 1.00 * halo + (1 - 1.00 * halo) * core
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
