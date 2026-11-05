-- gen_glow_set.lua DIR: the glow images. Two kinds. HALO ones are for materials with Halo = 1: brightness is opacity and the hue stays full, so a fade stays coloured over a bright ground.
-- The others keep the plain material: a small white core, saturated body and a dark red rim that holds the outline on a bright ground. Uncompressed PNG, no libraries.
local dir = assert(arg[1])
local N = 512
local pi = math.pi
local function clamp(x) return x < 0 and 0 or x > 1 and 1 or x end
local function smooth(a, b, x) local t = clamp((x - a) / (b - a)); return t * t * (3 - 2 * t) end
local function mix(a, b, t) return a + (b - a) * t end
-- Heat 0 to 1: deep red, orange, yellow, white. Linear colour.
local function heat_hue(h)
  if h < 0.35 then local t = h / 0.35; return 1.0, mix(0.1, 0.38, t), mix(0.0, 0.02, t) end
  if h < 0.7 then local t = (h - 0.35) / 0.35; return 1.0, mix(0.38, 0.8, t), mix(0.02, 0.2, t) end
  local t = (h - 0.7) / 0.3; return 1.0, mix(0.8, 1.0, t), mix(0.2, 1.0, t)
end
local function noise(a, seed)
  return 0.5 + 0.25 * math.sin(a * 3 + seed) + 0.15 * math.sin(a * 7 + seed * 2.3) + 0.1 * math.sin(a * 13 + seed * 3.7)
end
-- Rays round a centre: strength at (angle, radius), each ray thin, tapering to its own length.
local function rays(ang, d, count, from, seed, width)
  local best = 0
  for i = 0, count - 1 do
    local a0 = (i + 0.35 * math.sin(i * 12.9 + seed)) * 2 * pi / count
    local reach = mix(from + 0.25, 0.97, 0.5 + 0.5 * math.sin(i * 7.3 + seed * 1.7))
    local da = math.abs(math.atan(math.sin(ang - a0), math.cos(ang - a0)))
    if d > from * 0.7 and d < reach then
      local along = (d - from * 0.7) / (reach - from * 0.7)
      local w = width * (1 - along)
      local v = (1 - smooth(w * 0.3, w, da * d)) * (1 - along ^ 2)
      if v > best then best = v end
    end
  end
  return best
end

local shapes = {}
-- A thin shot flying up: the head on top, the tail fading.
shapes.glow_tracer = function(x, y)
  local t = (y + 0.95) / 1.85
  if t < 0 or t > 1 then return 0, 0 end
  local w = mix(0.03, 0.11, smooth(0, 0.85, t)) * (1 - smooth(0.9, 1, t) * 0.8)
  local a = math.exp(-(x / w) ^ 2) * smooth(0, 0.7, t) ^ 1.2
  return a, clamp(a * mix(0.25, 0.8, t))
end
-- A crit's shot: a round hot head and a licking tail.
shapes.glow_comet = function(x, y)
  local head = math.exp(-((x * x + (y - 0.45) ^ 2) / 0.06))
  local t = clamp((0.5 - y) / 1.4)
  local w = mix(0.2, 0.03, t) * (0.8 + 0.2 * math.sin(y * 18))
  local tail = y < 0.5 and math.exp(-(x / w) ^ 2) * (1 - t) ^ 1.3 or 0
  local a = math.max(head, tail * 0.9)
  return clamp(a * 1.3), clamp(head * 0.55 + tail * 0.25)
end
-- A hit: small and bright where it touches, thin rays out.
shapes.glow_hit = function(x, y)
  local d, ang = math.sqrt(x * x + y * y), math.atan(y, x)
  local edge = 0.36 + 0.2 * noise(ang, 2.1) + 0.2 * math.max(0, math.sin(ang * 7 + 1)) ^ 4
  local body = 1 - smooth(edge * 0.6, edge, d)
  local r = rays(ang, d, 11, 0.3, 1.3, 0.05)
  return math.max(body, r * 0.9), clamp((1 - d / edge) * 0.8 + math.exp(-(d / 0.08) ^ 2))
end
-- The small burn mark left on what was hit: a broken ring of embers.
shapes.glow_mark = function(x, y)
  local d, ang = math.sqrt(x * x + y * y), math.atan(y, x)
  local ring = math.exp(-((d - 0.62) / 0.05) ^ 2) * (0.55 + 0.45 * math.cos(ang * 6))
  local dot = math.exp(-(d / 0.14) ^ 2) * 0.75
  local a = math.max(ring, dot)
  return a, clamp(0.35 + a * 0.35)
end
-- A kick's powder: sparks thrown outward, the middle left open for the body.
shapes.glow_spark_burst = function(x, y)
  local d, ang = math.sqrt(x * x + y * y), math.atan(y, x)
  local r = rays(ang, d, 26, 0.3, 4.1, 0.05)
  local haze = math.exp(-((d - 0.4) / 0.2) ^ 2) * 0.18
  local a = math.max(r, haze)
  return a, clamp(r * 0.5 * (1.15 - d * 0.7) + 0.1)
end
-- An ultimate's fireball: lobed, hot in the middle, soft red at the rim.
shapes.glow_blast = function(x, y)
  local d, ang = math.sqrt(x * x + y * y), math.atan(y, x)
  local edge = 0.55 + 0.28 * noise(ang, 0.7)
  local body = 1 - smooth(edge * 0.55, edge + 0.12, d)
  local lick = 1 + 0.25 * math.sin(ang * 9 + d * 14) * smooth(0.2, 0.6, d)
  local r = rays(ang, d, 14, 0.5, 2.2, 0.03) * 0.8
  local a = math.max(clamp(body * lick), r)
  return a, clamp((1 - smooth(0.0, edge, d)) * 0.8)
end
-- A volley's hits: a scatter of small round pops, each with its own hot centre.
shapes.glow_pops = function(x, y)
  local spots = { { -0.42, 0.34, 0.26 }, { 0.38, 0.46, 0.2 }, { 0.05, -0.05, 0.3 }, { -0.3, -0.48, 0.2 }, { 0.5, -0.32, 0.24 }, { 0.0, 0.68, 0.14 }, { -0.68, -0.08, 0.13 } }
  local a, h = 0, 0
  for _, s in ipairs(spots) do
    local d = math.sqrt((x - s[1]) ^ 2 + (y - s[2]) ^ 2) / s[3]
    local v = 1 - smooth(0.6, 1, d)
    if v > a then a = v end
    local heat = clamp((1 - d) * 1.1)
    if heat > h then h = heat end
  end
  return a, h
end
-- A crit: long sharp spikes, deep red, far less white than a plain hit.
shapes.glow_crit = function(x, y)
  local d, ang = math.sqrt(x * x + y * y), math.atan(y, x)
  local spike = math.max(0, math.cos(ang * 4 + 0.4)) ^ 6
  local edge = 0.3 + 0.1 * noise(ang, 5.2) + 0.62 * spike
  local body = 1 - smooth(edge * 0.75, edge, d)
  return body, clamp((1 - d / edge) * 0.62 + math.exp(-(d / 0.07) ^ 2))
end
-- What a blast leaves: a burnt patch, embers at its edge.
shapes.glow_scorch = function(x, y)
  local d, ang = math.sqrt(x * x + y * y), math.atan(y, x)
  local edge = 0.62 + 0.25 * noise(ang, 3.3)
  local burn = (1 - smooth(edge * 0.7, edge, d)) * 0.6
  local embers = math.exp(-((d - edge * 0.85) / 0.04) ^ 2) * (0.5 + 0.5 * math.sin(ang * 23))
  return math.max(burn, embers), embers * 0.5
end
-- An area's edge: separate marks, a faint line between them. White, for the cue's tint.
shapes.glow_ring_marks = function(x, y)
  local d, ang = math.sqrt(x * x + y * y), math.atan(y, x)
  local dash = smooth(0.25, 0.45, math.cos(ang * 16) * 0.5 + 0.5)
  local a = math.exp(-((d - 0.94) / 0.03) ^ 2) * mix(0.3, 1, dash)
  return a, 1
end
-- A thin line round an area. White, for the cue's tint.
shapes.glow_thin_ring = function(x, y)
  local d = math.sqrt(x * x + y * y)
  return math.exp(-((d - 0.93) / 0.028) ^ 2) + math.exp(-((d - 0.88) / 0.07) ^ 2) * 0.22, 1
end

local crc_table = {}
for i = 0, 255 do local c = i; for _ = 1, 8 do c = (c & 1) ~= 0 and (0xEDB88320 ~ (c >> 1)) or (c >> 1) end; crc_table[i] = c end
local function crc32(s) local c = 0xFFFFFFFF; for i = 1, #s do c = crc_table[(c ~ s:byte(i)) & 255] ~ (c >> 8) end; return (c ~ 0xFFFFFFFF) & 0xFFFFFFFF end
local function chunk(kind, data) return string.pack(">I4", #data) .. kind .. data .. string.pack(">I4", crc32(kind .. data)) end
-- Halo materials read brightness as opacity, so their low heat stays as it is. The rest go dark at the rim.
local HALO = { glow_spark_burst = true, glow_thin_ring = true }
local function encode(v) return math.floor(clamp(v) ^ (1 / 2.2) * 255 + 0.5) end
for name, shape in pairs(shapes) do
  local rows = {}
  for py = 0, N - 1 do
    local row = { "\0" }
    for px = 0, N - 1 do
      local a, h = shape((px + 0.5) / N * 2 - 1, 1 - (py + 0.5) / N * 2)
      a = clamp(a)
      local r, g, b = heat_hue(clamp(h))
      if not HALO[name] then local v = mix(0.45, 1, smooth(0, 0.3, clamp(h))); r, g, b = r * v, g * v, b * v end
      row[#row + 1] = string.char(encode(r * a), encode(g * a), encode(b * a), encode(a))
    end
    rows[#rows + 1] = table.concat(row)
  end
  local raw = table.concat(rows)
  local s1, s2 = 1, 0
  for i = 1, #raw do s1 = (s1 + raw:byte(i)) % 65521; s2 = (s2 + s1) % 65521 end
  local z = { "\x78\x01" }
  local pos = 1
  while pos <= #raw do
    local take = math.min(65535, #raw - pos + 1)
    z[#z + 1] = string.pack("<BI2I2", pos + take > #raw and 1 or 0, take, take ~ 0xFFFF) .. raw:sub(pos, pos + take - 1)
    pos = pos + take
  end
  z[#z + 1] = string.pack(">I4", (s2 << 16) | s1)
  local path = dir .. "/gen-cue-fx_" .. name .. ".png"
  local f = assert(io.open(path, "wb"))
  f:write("\x89PNG\r\n\x1a\n", chunk("IHDR", string.pack(">I4I4BBBBB", N, N, 8, 6, 0, 0, 0)), chunk("IDAT", table.concat(z)), chunk("IEND", ""))
  f:close()
  print("wrote", path)
end
