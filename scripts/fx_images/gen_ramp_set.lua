-- gen_ramp_set.lua DIR: shapes for materials with Ramp = 1. Red is heat, green is opacity; the colour comes from the cue's tint
-- (dark rim, full body, a pale hot centre, white only at the very top of the heat). One image serves every hero. Uncompressed PNG, no libraries.
local dir = assert(arg[1])
local N = 256
local pi = math.pi
local function clamp(x) return x < 0 and 0 or x > 1 and 1 or x end
local function smooth(a, b, x) local t = clamp((x - a) / (b - a)); return t * t * (3 - 2 * t) end
local function mix(a, b, t) return a + (b - a) * t end
local function noise(a, seed)
  return 0.5 + 0.25 * math.sin(a * 3 + seed) + 0.15 * math.sin(a * 7 + seed * 2.3) + 0.1 * math.sin(a * 13 + seed * 3.7)
end
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
local function polar(x, y) return math.sqrt(x * x + y * y), math.atan(y, x) end

local shapes = {}
-- A small burst at the point of contact.
shapes.ramp_hit = function(x, y)
  local d, ang = polar(x, y)
  local edge = 0.4 + 0.2 * noise(ang, 2.1) + 0.2 * math.max(0, math.sin(ang * 7 + 1)) ^ 4
  local r = rays(ang, d, 11, 0.3, 1.3, 0.05)
  return math.max(1 - smooth(edge * 0.65, edge, d), r * 0.9), clamp((1 - d / edge) * 0.8 + math.exp(-(d / 0.08) ^ 2))
end
-- A scatter of small round pops.
shapes.ramp_pops = function(x, y)
  local spots = { { -0.42, 0.34, 0.26 }, { 0.38, 0.46, 0.2 }, { 0.05, -0.05, 0.3 }, { -0.3, -0.48, 0.2 }, { 0.5, -0.32, 0.24 }, { 0.0, 0.68, 0.14 }, { -0.68, -0.08, 0.13 } }
  local a, h = 0, 0
  for _, s in ipairs(spots) do
    local d = math.sqrt((x - s[1]) ^ 2 + (y - s[2]) ^ 2) / s[3]
    a = math.max(a, 1 - smooth(0.6, 1, d))
    h = math.max(h, clamp((1 - d) * 1.1))
  end
  return a, h
end
-- Long sharp spikes, for a blow out of the ordinary.
shapes.ramp_star = function(x, y)
  local d, ang = polar(x, y)
  local spike = math.max(0, math.cos(ang * 4 + 0.4)) ^ 6
  local edge = 0.3 + 0.1 * noise(ang, 5.2) + 0.62 * spike
  return 1 - smooth(edge * 0.75, edge, d), clamp((1 - d / edge) * 0.7 + math.exp(-(d / 0.07) ^ 2))
end
-- A shot flying up: the head on top, the tail fading.
shapes.ramp_tracer = function(x, y)
  local t = (y + 0.95) / 1.85
  if t < 0 or t > 1 then return 0, 0 end
  local w = mix(0.03, 0.11, smooth(0, 0.85, t)) * (1 - smooth(0.9, 1, t) * 0.8)
  local a = math.exp(-(x / w) ^ 2) * smooth(0, 0.7, t) ^ 1.2
  return clamp(a * 1.5), clamp(a * mix(0.25, 0.9, t))
end
-- Sword cuts swirling out over a wide area: each thin, pointed at both ends, bright on its outer edge.
shapes.ramp_mist_cuts = function(x, y)
  local d, ang = polar(x, y)
  local alpha, heat = 0, 0
  for i = 0, 8 do
    local a0 = i * 2 * pi / 9 + 0.5 * math.sin(i * 5.1)
    local r0 = i % 3 == 0 and 0.42 or i % 3 == 1 and 0.62 or 0.8
    local t = math.atan(math.sin(ang - a0), math.cos(ang - a0)) / 0.62
    if t > -1 and t < 1 then
      local span = 1 - t * t
      local r = d - (r0 + 0.07 * t)
      local v = (r > 0 and math.exp(-(r / (0.01 * span + 0.002)) ^ 2) or math.exp(r / (0.06 * span + 0.003))) * smooth(0, 0.25, span)
      alpha = math.max(alpha, clamp(v * 2.4))
      heat = math.max(heat, clamp(v ^ 3 * (0.35 + 0.5 * span) + 0.3 * v))
    end
  end
  return alpha, heat
end
-- Fissures running out from the blow: thin, jagged, hot at the middle of each line and near the centre, dark along their edges.
shapes.ramp_crack = function(x, y)
  local d, ang = polar(x, y)
  local alpha, heat = 0, 0
  local function line(a0, from, reach, width, seed)
    if d < from or d > reach then return end
    local along = (d - from) / (reach - from)
    local off = d * math.sin(ang - a0 - 0.16 * math.sin(d * 13 + seed) - 0.07 * math.sin(d * 31 + seed * 2))
    if math.cos(ang - a0) < 0 then return end
    local w = width * (1 - along * 0.85)
    alpha = math.max(alpha, clamp(math.exp(-(off / (w * 1.5)) ^ 2) * 2.2) * (1 - smooth(0.8, 1, along)))
    heat = math.max(heat, math.exp(-(off / (w * 0.6)) ^ 2) * (0.85 - 0.35 * d) * (1 - smooth(0.7, 1, along)))
  end
  for i = 0, 6 do
    local a0 = i * 2 * pi / 7 + 0.4 * math.sin(i * 3.7)
    local reach = 0.72 + 0.23 * math.sin(i * 8.3) ^ 2
    line(a0, 0.03, reach, 0.052, i * 1.9)
    line(a0 + (i % 2 == 0 and 0.42 or -0.42), reach * 0.42, reach * 0.85, 0.03, i * 2.7 + 1)
  end
  return alpha, heat
end
-- The same fissures, cooler: a strong tint stays deep along them.
shapes.ramp_crack_deep = function(x, y)
  local a, h = shapes.ramp_crack(x, y)
  return a, h * 0.6
end
-- A thrown spear pointing up: a broad head with a hot point, a collar, a shaft; dark along every edge.
shapes.ramp_spear = function(x, y)
  if y < -0.95 or y > 0.95 then return 0, 0 end
  local w
  if y > 0.3 then w = 0.15 * math.min(1, (0.95 - y) / 0.5)
  elseif y > 0.2 then w = 0.075
  else w = 0.035 end
  local e = math.abs(x) / math.max(w, 0.001)
  local a = (1 - smooth(0.82, 1.05, e)) * smooth(-0.95, -0.7, y)
  local h = (1 - smooth(0.25, 0.95, e)) * (y > 0.3 and mix(0.55, 1.0, (y - 0.3) / 0.65) or (y > 0.2 and 0.75 or 0.5))
  return a, clamp(h)
end
-- A slash: a crescent with a thin tail at both tips, hot along its outer edge.
shapes.ramp_slash = function(x, y)
  local d, ang = polar(x, y + 0.5)
  local a0 = math.abs(ang - pi / 2)
  local span = clamp(1 - a0 / 1.25)
  local r = d - 0.92
  local v = (r > 0 and math.exp(-(r / (0.0375 * span + 0.004)) ^ 2) or math.exp(r / (0.165 * span + 0.006))) * smooth(0, 0.2, span)
  return clamp(v * 1.3) ^ 1.1, clamp(v ^ 2 * (0.35 + 0.35 * span))
end
-- Sparks thrown outward, the middle open for the body.
shapes.ramp_rays = function(x, y)
  local d, ang = polar(x, y)
  local r = rays(ang, d, 26, 0.3, 4.1, 0.05)
  return r, clamp(r * 0.6 * (1.15 - d * 0.7) + 0.1)
end
-- A soft lobed blast with a dark rim.
shapes.ramp_blast = function(x, y)
  local d, ang = polar(x, y)
  local edge = 0.55 + 0.28 * noise(ang, 0.7)
  local lick = 1 + 0.25 * math.sin(ang * 9 + d * 14) * smooth(0.2, 0.6, d)
  return clamp((1 - smooth(edge * 0.55, edge + 0.12, d)) * lick), clamp((1 - smooth(0, edge, d)) * 0.85)
end
-- A small mark on what was struck: a broken ring and a dot.
shapes.ramp_mark = function(x, y)
  local d, ang = polar(x, y)
  local ring = math.exp(-((d - 0.62) / 0.06) ^ 2) * (0.55 + 0.45 * math.cos(ang * 6))
  local a = math.max(ring, math.exp(-(d / 0.14) ^ 2) * 0.8)
  return clamp(a * 1.3), clamp(0.3 + a * 0.35)
end
-- An area's edge: separate marks with a faint line between them.
shapes.ramp_ring_marks = function(x, y)
  local d, ang = polar(x, y)
  local dash = smooth(0.25, 0.45, math.cos(ang * 16) * 0.5 + 0.5)
  return clamp(math.exp(-((d - 0.94) / 0.03) ^ 2) * mix(0.3, 1, dash) * 1.3), 0.5
end
-- A thin line round an area, with a soft inner glow.
shapes.ramp_thin_ring = function(x, y)
  local d = polar(x, y)
  return clamp(math.exp(-((d - 0.93) / 0.028) ^ 2) * 1.2 + math.exp(-((d - 0.86) / 0.08) ^ 2) * 0.22), clamp(math.exp(-((d - 0.93) / 0.02) ^ 2) * 0.8 + 0.25)
end
-- A pulse: a band with a hot inner edge, soft outside.
shapes.ramp_band = function(x, y)
  local d, ang = polar(x, y)
  local band = smooth(0.64, 0.68, d) * (1 - smooth(0.8, 0.98, d)) * (0.85 + 0.15 * math.sin(ang * 11))
  return band, clamp((1 - smooth(0.66, 0.9, d)) * 0.9)
end

local crc_table = {}
for i = 0, 255 do local c = i; for _ = 1, 8 do c = (c & 1) ~= 0 and (0xEDB88320 ~ (c >> 1)) or (c >> 1) end; crc_table[i] = c end
local function crc32(s) local c = 0xFFFFFFFF; for i = 1, #s do c = crc_table[(c ~ s:byte(i)) & 255] ~ (c >> 8) end; return (c ~ 0xFFFFFFFF) & 0xFFFFFFFF end
local function chunk(kind, data) return string.pack(">I4", #data) .. kind .. data .. string.pack(">I4", crc32(kind .. data)) end
-- The textures are sRGB: written so the shader reads heat and opacity back as they were computed.
local function encode(v) return math.floor(clamp(v) ^ (1 / 2.2) * 255 + 0.5) end
local SIZES = { ramp_mist_cuts = 512, ramp_crack = 512, ramp_crack_deep = 512 }
for name, shape in pairs(shapes) do
  local N = SIZES[name] or N
  local rows = {}
  for py = 0, N - 1 do
    local row = { "\0" }
    for px = 0, N - 1 do
      local a, h = shape((px + 0.5) / N * 2 - 1, 1 - (py + 0.5) / N * 2)
      row[#row + 1] = string.char(encode(clamp(h)), encode(clamp(a)), 0, 255)
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
