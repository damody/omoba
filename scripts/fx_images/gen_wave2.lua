-- gen_wave2.lua OUTDIR: round two's drawn images, one per skill. Light bodies take the cue's tint, dark edges read against the bright ground. Uncompressed PNG, no library.
local dir = assert(arg[1])
local N = 512
local function clamp(x) return x < 0 and 0 or x > 1 and 1 or x end
local function smooth(a, b, x) local t = clamp((x - a) / (b - a)); return t * t * (3 - 2 * t) end
local function rot(x, y, a) local c, s = math.cos(a), math.sin(a); return x * c - y * s, x * s + y * c end
-- Distance to a segment, and how far along it (0..1).
local function seg(x, y, ax, ay, bx, by)
  local dx, dy = bx - ax, by - ay
  local t = clamp(((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy))
  local px, py = ax + dx * t - x, ay + dy * t - y
  return math.sqrt(px * px + py * py), t
end
-- A shape from a signed "d" (0 centre .. 1 edge): dark edge, tintable body, white core.
local function shade(d, edge, core_limit)
  if d >= 1 then return 0, 0, 0, 0 end
  local body = 1 - smooth(0.62, 0.82, d)
  local v = edge + (1 - edge) * body
  local core = 1 - smooth(core_limit or 0.25, (core_limit or 0.25) + 0.2, d)
  v = v + (1 - v) * core
  local a = 1 - smooth(0.9, 1.0, d)
  return v, v, v, a
end
-- Best of several shapes: the nearest centre wins.
local function bolt(path, width)
  return function(x, y)
    local best = 2
    for _, p in ipairs(path) do
      for i = 1, #p.pts - 1 do
        local a, b = p.pts[i], p.pts[i + 1]
        local dist, t = seg(x, y, a[1], a[2], b[1], b[2])
        local tt = (i - 1 + t) / (#p.pts - 1)
        local w = width * p.w * (p.taper_from + (p.taper_to - p.taper_from) * tt)
        if w > 0 then best = math.min(best, dist / w) end
      end
    end
    return shade(best, 0.12, 0.3)
  end
end
local images = {}
-- 利家 被動: three spearheads closing on the struck target.
images.stack_spears = function(x, y)
  local best = 2
  for i = 0, 2 do
    local px, py = rot(x, y, i * 2.0944)
    -- y runs down the image: a blade above the centre, point towards it
    local v = (-py - 0.14) / 0.78
    if v >= 0 and v <= 1 then
      local half = v < 0.42 and 0.19 * v / 0.42 or 0.19 * (1 - (v - 0.42) / 0.58) ^ 1.4 + 0.02
      best = math.min(best, math.abs(px) / half)
    end
  end
  return shade(best, 0.1, 0.22)
end
-- 大熊 普攻: one heavy chop, wide at the top and tapering to the blow.
images.axe_cleave = function(x, y)
  local v = (y + 0.94) / 1.88
  if v < 0 or v > 1 then return 0, 0, 0, 0 end
  local bend = 0.13 * math.sin(v * math.pi)
  local half = 0.03 + 0.27 * (v < 0.25 and v / 0.25 or (1 - (v - 0.25) / 0.75) ^ 1.3)
  local d = math.abs(x - bend) / half
  if d >= 1 then return 0, 0, 0, 0 end
  -- A hot centre falling off to a dark rim, so it is not a flat cut-out.
  local l = 0.62 + 0.38 * (1 - smooth(0.05, 0.85, d))
  return l, l, l, 1 - smooth(0.88, 1.0, d)
end
-- 濃姬 二丁銃: two muzzle flames, side by side.
images.twin_muzzle = function(x, y)
  -- Drawn in its own colours. Bottom of the image is the muzzle; each jet flares out and licks to a point.
  local best, along = 2, 0
  for _, side in ipairs { -1, 1 } do
    local px, py = rot(x - side * 0.16, y - 0.86, side * 0.2)
    local v = -py / 1.72
    if v >= 0 and v <= 1 then
      local half = 0.04 + 0.2 * (v < 0.28 and (v / 0.28) ^ 0.7 or (1 - (v - 0.28) / 0.72) ^ 1.5)
      local lick = 0.035 * math.sin(v * 17 + side * 1.3) * v
      local d = math.abs(px + lick) / half
      if d < best then best, along = d, v end
    end
  end
  if best >= 1 then return 0, 0, 0, 0 end
  local heat = clamp((1 - best) * 1.5 * (1 - along * 0.55))
  local r, g, b
  if heat < 0.35 then local k = heat / 0.35; r, g, b = 0.65 + 0.35 * k, 0.1 + 0.3 * k, 0.02
  elseif heat < 0.7 then local k = (heat - 0.35) / 0.35; r, g, b = 1, 0.4 + 0.42 * k, 0.02 + 0.1 * k
  else local k = (heat - 0.7) / 0.3; r, g, b = 1, 0.82 + 0.17 * k, 0.12 + 0.78 * k end
  return r, g, b, (1 - smooth(0.84, 1.0, best)) * (1 - smooth(0.82, 1.0, along))
end
-- 佐佐木 普攻: one clean sword cut, thin and long.
images.iai_cut = function(x, y)
  local d, t = seg(x, y, -0.86, 0.72, 0.86, -0.72)
  local w = 0.01 + 0.1 * math.max(0, math.sin(t * math.pi)) ^ 0.8
  return shade(d / w, 0.08, 0.38)
end
-- 佐佐木 R: the mist cut three ways.
images.cut_cross = function(x, y)
  local best = 2
  for i = 0, 2 do
    local px, py = rot(x, y, i * 1.0472 + 0.3)
    local d, t = seg(px, py, -0.92, 0, 0.92, 0)
    local w = 0.008 + 0.075 * math.max(0, math.sin(t * math.pi)) ^ 0.8
    best = math.min(best, d / w)
  end
  return shade(best, 0.08, 0.38)
end
-- 德川 E: one bolt straight down from above, with two forks.
images.bolt_fork = bolt({
  { w = 1, taper_from = 1.0, taper_to = 0.25, pts = { { 0.08, -0.98 }, { -0.1, -0.56 }, { 0.1, -0.3 }, { -0.12, 0.08 }, { 0.08, 0.36 }, { -0.03, 0.96 } } },
  { w = 0.65, taper_from = 0.8, taper_to = 0.1, pts = { { -0.1, -0.56 }, { -0.42, -0.34 }, { -0.36, -0.1 }, { -0.62, 0.2 } } },
  { w = 0.65, taper_from = 0.8, taper_to = 0.1, pts = { { 0.08, 0.36 }, { 0.36, 0.3 }, { 0.32, 0.54 }, { 0.54, 0.74 } } },
}, 0.085)
-- 島津 W: the ground shakes in broken rings going out, not cracks.
images.quake_ripple = function(x, y)
  local r, a = math.sqrt(x * x + y * y), math.atan(y, x)
  local best = 2
  for i, radius in ipairs { 0.3, 0.58, 0.86 } do
    local wob = radius + 0.01 * math.sin(a * 11 + i * 2) + 0.005 * math.sin(a * 29 + i)
    local gap = 0.5 + 0.5 * math.sin(a * (2 + i) + i * 1.7)
    local w = 0.024 * smooth(0.04, 0.2, gap)
    if w > 0.004 then best = math.min(best, math.abs(r - wob) / w) end
  end
  return shade(best, 0.12, 0.1)
end
-- 大熊 普攻: the axe's path. Image right is the way the blow goes: from high behind, down to the low front, thickest where it lands.
images.axe_arc = function(x, y)
  local up = -y
  local dx, dy = x - 0.66, up - 0.5
  local r = math.sqrt(dx * dx + dy * dy)
  local deg = math.deg(math.atan(dy, dx)); if deg < 0 then deg = deg + 360 end
  local s = (deg - 167) / 85.4
  if s < 0 or s > 1 then return 0, 0, 0, 0 end
  local half = s < 0.82 and 0.015 + 0.2 * (s / 0.82) ^ 1.3 or 0.215 * (1 - (s - 0.82) / 0.18) ^ 0.8 + 0.002
  local d = math.abs(r - 1.535) / half
  if d >= 1 then return 0, 0, 0, 0 end
  local l = 0.3 + 0.45 * (1 - smooth(0.6, 0.85, d)) + 0.25 * (1 - smooth(0.1, 0.5, d))
  return l, l, l, (1 - smooth(0.9, 1.0, d)) * smooth(0, 0.3, s)
end
-- 島津 W: flat on the ground, centred on the blow: glowing cracks running out, two broken rings, all gone before the edge. Its own colours.
images.quake_burst = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r >= 0.97 then return 0, 0, 0, 0 end
  local a = math.atan(y, x)
  local crack = 2
  for i = 0, 8 do
    local ang = i * 0.698 + 0.2 + 0.05 * math.sin(r * 16 + i) + 0.03 * math.sin(r * 41 + i * 2)
    local diff = (a - ang + math.pi) % (2 * math.pi) - math.pi
    local reach = 0.62 + 0.3 * ((i * 5) % 3) / 2
    if r < reach then crack = math.min(crack, math.abs(diff) * r / (0.004 + 0.036 * (1 - r / reach))) end
  end
  local edge = 1 - smooth(0.74, 0.96, r)
  if crack < 1 then
    local heat = 1 - crack
    return 1, 0.42 + 0.5 * heat, 0.04 + 0.3 * heat, (1 - smooth(0.8, 1.0, crack)) * (0.35 + 0.65 * edge)
  end
  for i, radius in ipairs { 0.56, 0.8 } do
    local wob = radius + 0.012 * math.sin(a * 11 + i)
    local gap = 0.5 + 0.5 * math.sin(a * (3 + i) + i)
    if gap > 0.25 and math.abs(r - wob) < 0.016 then return 0.2, 0.12, 0.06, 0.75 * edge end
  end
  return 0.27, 0.17, 0.09, 0.62 * (1 - smooth(0.25, 0.82, r))
end
-- 德川 X: a plus, solid, the healer's mark.
images.heal_cross = function(x, y)
  local ax, ay = math.abs(x), math.abs(y)
  local d = math.max(math.min(ax, ay) / 0.24, math.max(ax, ay) / 0.8)
  if d >= 1 then return 0, 0, 0, 0 end
  local l = 0.75 + 0.25 * (1 - smooth(0.2, 0.8, d))
  return l, l, l, 1 - smooth(0.9, 1.0, d)
end
-- 大熊 E: three chevrons climbing, fading as they rise: health coming back.
images.heal_rise = function(x, y)
  local best = 2
  local fade = 1
  for i, cy in ipairs { 0.62, 0.12, -0.38 } do
    local d = math.abs((y - cy) + math.abs(x) * 0.7) / 0.11
    if math.abs(x) < 0.72 and d < best then best = d; fade = 1.15 - i * 0.22 end
  end
  if best >= 1 then return 0, 0, 0, 0 end
  local l = 0.8 + 0.2 * (1 - smooth(0.2, 0.8, best))
  return l, l, l, (1 - smooth(0.85, 1.0, best)) * fade
end
-- 濃姬 W: powder going off around her feet: a soot heart, a band of red-orange flame licking outward, gone before the edge. Its own colours.
images.powder_ring = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r >= 0.98 then return 0, 0, 0, 0 end
  local a = math.atan(y, x)
  local lick = 0.5 + 0.5 * math.sin(a * 13 + 1.1) * math.sin(a * 7 + 0.4)
  local outer = 0.68 + 0.26 * lick
  local inner = 0.34 + 0.05 * math.sin(a * 11)
  if r > inner and r < outer then
    local mid = (inner + outer) / 2
    local heat = 1 - math.abs(r - mid) / ((outer - inner) / 2)
    local cr, cg, cb
    if heat < 0.45 then local k = heat / 0.45; cr, cg, cb = 0.62 + 0.38 * k, 0.08 + 0.2 * k, 0.02
    else local k = (heat - 0.45) / 0.55; cr, cg, cb = 1, 0.28 + 0.52 * k, 0.02 + 0.12 * k end
    return cr, cg, cb, smooth(0, 0.18, heat) * (1 - smooth(0.82, 0.96, r) * 0.6)
  end
  if r <= inner then return 0.16, 0.11, 0.09, 0.62 * smooth(0, inner, r) + 0.2 end
  return 0, 0, 0, 0
end
-- Heat to colour for fire drawn in its own colours: red rim, orange body, pale core.
local function fire(heat)
  if heat < 0.35 then local k = heat / 0.35; return 0.65 + 0.35 * k, 0.1 + 0.3 * k, 0.02 end
  if heat < 0.7 then local k = (heat - 0.35) / 0.35; return 1, 0.4 + 0.42 * k, 0.02 + 0.1 * k end
  local k = (heat - 0.7) / 0.3; return 1, 0.82 + 0.17 * k, 0.12 + 0.78 * k
end
-- 濃姬 爆擊: one short flame out of a pistol. The bottom of the image is the muzzle.
images.pistol_flare = function(x, y)
  local v = (0.9 - y) / 1.8
  if v < 0 or v > 1 then return 0, 0, 0, 0 end
  local half = 0.05 + 0.3 * (v < 0.3 and (v / 0.3) ^ 0.7 or (1 - (v - 0.3) / 0.7) ^ 1.4)
  local d = math.abs(x + 0.04 * math.sin(v * 14) * v) / half
  if d >= 1 then return 0, 0, 0, 0 end
  local r, g, b = fire(math.min(1, (1 - d) * 1.5 * (1 - v * 0.5)))
  return r, g, b, (1 - smooth(0.84, 1.0, d)) * (1 - smooth(0.8, 1.0, v))
end
-- 濃姬 Q: one pistol shot: a hot head at the top of the image, a thin tail fading behind it.
images.pistol_shot = function(x, y)
  local v = (0.92 - y) / 1.84
  if v < 0 or v > 1 then return 0, 0, 0, 0 end
  local half = v < 0.82 and 0.012 + 0.07 * (v / 0.82) ^ 2 or 0.082 * (1 - (v - 0.82) / 0.18) ^ 0.6
  local d = math.abs(x) / math.max(half, 0.002)
  if d >= 1 then return 0, 0, 0, 0 end
  local r, g, b = fire(math.min(1, (1 - d) * 1.4 * (0.45 + 0.55 * v)))
  return r, g, b, (1 - smooth(0.8, 1.0, d)) * smooth(0, 0.35, v)
end
-- 濃姬 W: powder trailing the spin: flat, one comet round the feet, its head at the top of the image.
images.powder_swirl = function(x, y)
  local r = math.sqrt(x * x + y * y)
  local deg = math.deg(math.atan(x, -y)); if deg < 0 then deg = deg + 360 end
  local s = 1 - deg / 300
  if s < 0 or s > 1 then return 0, 0, 0, 0 end
  local half = 0.015 + 0.11 * s ^ 1.3
  local d = math.abs(r - 0.76) / half
  if d >= 1 then return 0, 0, 0, 0 end
  local cr, cg, cb = fire(math.min(1, (1 - d) * 1.4 * (0.4 + 0.6 * s)))
  return cr, cg, cb, (1 - smooth(0.78, 1.0, d)) * smooth(0, 0.3, s)
end
-- A fissure pattern out from the centre: jagged arms that thin toward their tips, hot inside and dark at the rim so they read on a light ground.
local function fissure(arms, seed, reach, hot, body, rim)
  return function(x, y)
    local r = math.sqrt(x * x + y * y)
    if r > 0.98 then return 0, 0, 0, 0 end
    local ang = math.atan(y, x)
    local best = 9
    for i = 1, arms do
      local base = (i - 1) / arms * 2 * math.pi + 0.7 * math.sin(i * seed)
      local long = reach * (0.62 + 0.38 * math.abs(math.sin(i * seed * 1.7)))
      if r < long then
        local wander = 0.2 * math.sin(r * 11 + i * seed) + 0.09 * math.sin(r * 27 + i * 2.1)
        local da = (ang - base - wander * r) % (2 * math.pi)
        if da > math.pi then da = da - 2 * math.pi end
        local width = 0.012 + 0.05 * (1 - r / long) ^ 1.3
        best = math.min(best, math.abs(da) * r / width)
        -- one side branch from each arm
        local start = long * 0.42
        if r > start then
          local side = (i % 2 == 0) and 0.5 or -0.5
          local db = (ang - base - wander * start - side * (r - start) / r) % (2 * math.pi)
          if db > math.pi then db = db - 2 * math.pi end
          best = math.min(best, math.abs(db) * r / (width * 0.7))
        end
      end
    end
    -- the broken ground at the centre
    best = math.min(best, r / (0.13 + 0.03 * math.sin(ang * 7 + seed)))
    if best >= 1.35 then return 0, 0, 0, 0 end
    local fade = 1 - smooth(0.8, 0.98, r / reach)
    if best > 1 then return rim[1], rim[2], rim[3], (1.35 - best) / 0.35 * 0.9 * fade end
    local k = smooth(0.15, 0.85, best)
    local function mix(i) return hot[i] + (body[i] - hot[i]) * k end
    return mix(1), mix(2), mix(3), fade
  end
end
images.crack_earth = fissure(9, 2.3, 0.97, { 1.0, 0.62, 0.18 }, { 0.72, 0.28, 0.06 }, { 0.2, 0.09, 0.03 })
images.crack_gold = fissure(13, 4.1, 0.98, { 1.0, 0.8, 0.28 }, { 0.92, 0.5, 0.06 }, { 0.3, 0.14, 0.02 })
images.crack_hammer = fissure(6, 1.3, 0.92, { 1.0, 0.72, 0.3 }, { 0.8, 0.36, 0.08 }, { 0.16, 0.1, 0.06 })
-- 佐佐木 R: a ring of sword cuts at the edge of the reach, deep blue with a bright edge.
images.mist_cuts = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r > 0.98 or r < 0.3 then return 0, 0, 0, 0 end
  local ang = math.atan(y, x)
  local n = 7
  local s = ((ang / (2 * math.pi) * n) % 1)
  local curve = 0.48 + 0.42 * s
  local half = 0.05 * math.sin(math.pi * s) ^ 0.6
  if half <= 0.001 then return 0, 0, 0, 0 end
  local d = math.abs(r - curve) / half
  if d >= 1 then return 0, 0, 0, 0 end
  local k = 1 - d
  return 0.16 + 0.6 * k, 0.42 + 0.48 * k, 0.86 + 0.14 * k, 1 - smooth(0.75, 1, d)
end
-- 佐佐木 R: mist left behind: ragged bands turning round him, deep blue, not one soft blob.
images.mist_bands = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r > 0.96 then return 0, 0, 0, 0 end
  local ang = math.atan(y, x)
  local band = math.sin(ang * 3 + r * 9 + 0.8 * math.sin(ang * 5 - r * 6))
  local a = smooth(0.1, 0.65, band) * smooth(0.12, 0.34, r) * (1 - smooth(0.68, 0.95, r))
  if a <= 0.01 then return 0, 0, 0, 0 end
  local k = smooth(0.2, 0.9, band)
  return 0.07 + 0.2 * k, 0.2 + 0.32 * k, 0.5 + 0.34 * k, a * 0.85
end
-- 德川 X: a green cross with a dark edge and a bright core: it has an outline, unlike a pale sticker.
images.heal_cross_solid = function(x, y)
  local arm = 0.22
  local d = math.max(math.min(math.abs(x) / arm, math.abs(y) / 0.82), 0)
  local dd = math.min(math.max(math.abs(x) / arm, math.abs(y) / 0.82), math.max(math.abs(y) / arm, math.abs(x) / 0.82))
  if dd >= 1 then return 0, 0, 0, 0 end
  if dd > 0.84 then return 0.03, 0.34, 0.14, 1 end
  local k = 1 - smooth(0.05, 0.8, dd)
  return 0.12 + 0.6 * k, 0.72 + 0.26 * k, 0.26 + 0.42 * k, 1
end
-- 佐佐木 R: thin pale mist behind the cuts: fine soft wisps, cyan-white, mostly see-through.
images.mist_bands = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r > 0.96 then return 0, 0, 0, 0 end
  local ang = math.atan(y, x)
  local wisp = math.sin(ang * 5 + r * 15 + 1.1 * math.sin(ang * 3 - r * 8))
  local fine = 0.5 + 0.5 * math.sin(ang * 11 - r * 24 + 2 * math.sin(ang * 2))
  local alpha = smooth(0.45, 0.95, wisp) * (0.55 + 0.45 * fine) * smooth(0.2, 0.42, r) * (1 - smooth(0.55, 0.94, r)) * 0.36
  if alpha <= 0.005 then return 0, 0, 0, 0 end
  return 0.74, 0.88, 0.92, alpha
end
-- 德川 X: a soft cross flat on the ground: see-through, brightest along its rim, so it never covers him.
images.heal_cross_soft = function(x, y)
  local dd = math.min(math.max(math.abs(x) / 0.2, math.abs(y) / 0.8), math.max(math.abs(y) / 0.2, math.abs(x) / 0.8))
  if dd >= 1.25 then return 0, 0, 0, 0 end
  if dd > 1 then return 0.35, 1, 0.55, (1.25 - dd) / 0.25 * 0.35 end
  local rim = smooth(0.55, 1, dd)
  return 0.28 + 0.2 * rim, 0.9, 0.42 + 0.14 * rim, 0.16 + 0.42 * rim
end
images.crack_red = fissure(11, 3.3, 0.97, { 1.0, 0.42, 0.18 }, { 0.68, 0.07, 0.05 }, { 0.17, 0.02, 0.02 })
-- 慶次 W: a thin arc along the image's width, with short forks: white core, pale blue glow.
do
  local main = { { -1, 0 }, { -0.72, 0.05 }, { -0.5, -0.04 }, { -0.28, 0.06 }, { -0.04, -0.03 }, { 0.2, 0.05 }, { 0.44, -0.05 }, { 0.7, 0.03 }, { 1, 0 } }
  local forks = { { { -0.5, -0.04 }, { -0.4, -0.2 }, { -0.28, -0.26 } }, { { -0.04, -0.03 }, { 0.05, 0.16 }, { 0.2, 0.22 } }, { { 0.44, -0.05 }, { 0.54, -0.22 }, { 0.68, -0.26 } } }
  local function near(path, x, y)
    local best = 9
    for i = 1, #path - 1 do
      local ax, ay, bx, by = path[i][1], path[i][2], path[i + 1][1], path[i + 1][2]
      local vx, vy = bx - ax, by - ay
      local k = clamp(((x - ax) * vx + (y - ay) * vy) / (vx * vx + vy * vy))
      local dx, dy = x - ax - vx * k, y - ay - vy * k
      best = math.min(best, math.sqrt(dx * dx + dy * dy))
    end
    return best
  end
  images.arc_fork = function(x, y)
    local m = near(main, x, y)
    local f = 9
    for _, path in ipairs(forks) do f = math.min(f, near(path, x, y)) end
    local core = math.max(1 - smooth(0.006, 0.018, m), 0.75 * (1 - smooth(0.003, 0.01, f)))
    local glow = math.max(0.75 * (1 - smooth(0.012, 0.075, m)), 0.4 * (1 - smooth(0.006, 0.04, f)))
    local alpha = math.max(core, glow)
    if alpha <= 0.01 then return 0, 0, 0, 0 end
    local k = core / alpha
    return 0.38 + 0.62 * k, 0.72 + 0.28 * k, 1, alpha
  end
end
-- 利家 Q: spear points flung out from the blow: long gold blades of two lengths, dark edged, with a hot centre.
images.thrust_star = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r > 0.98 then return 0, 0, 0, 0 end
  local ang = math.atan(y, x)
  local n = 8
  local s = (ang / (2 * math.pi) * n) % 1
  local off = math.abs(s - 0.5) * 2 / n * 2 * math.pi
  local long = (math.floor(ang / (2 * math.pi) * n + n) % 2 == 0) and 0.96 or 0.66
  local half = 0.11 * (1 - r / long)
  if r < long and half > 0 then
    local d = off * r / half
    if d < 1 then
      local k = 1 - d
      if d > 0.72 then return 0.45, 0.24, 0.02, 1 - smooth(0.9, 1, d) end
      return 0.95 + 0.05 * k, 0.6 + 0.3 * k, 0.1 + 0.32 * k, 1
    end
  end
  if r < 0.16 then return 1, 0.84, 0.42, 1 - smooth(0.12, 0.16, r) end
  return 0, 0, 0, 0
end
-- 利家 W: the blink's track, end to end across the image: a thin gold streak, white at its core.
images.blink_trace = function(x, y)
  local half = 0.06 + 0.1 * (x + 1) / 2
  local d = math.abs(y) / half
  if d >= 1 then return 0, 0, 0, 0 end
  local k = 1 - smooth(0.1, 0.55, d)
  return 1, 0.72 + 0.26 * k, 0.16 + 0.7 * k, (1 - smooth(0.4, 1, d)) * (0.35 + 0.65 * (x + 1) / 2)
end
-- 大谷 R: the seal of the possession: a dark-edged ring, rune ticks, eight teeth biting inward, a faint dark floor.
images.possession_sigil = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r > 0.97 then return 0, 0, 0, 0 end
  local ang = math.atan(y, x)
  local hot, body, rim = { 0.86, 0.5, 1 }, { 0.46, 0.12, 0.84 }, { 0.12, 0.02, 0.26 }
  if r > 0.86 and r < 0.95 then
    local d = math.abs(r - 0.905) / 0.045
    if d > 0.72 then return rim[1], rim[2], rim[3], 1 end
    local k = 1 - d / 0.72
    return body[1] + (hot[1] - body[1]) * k, body[2] + (hot[2] - body[2]) * k, body[3] + (hot[3] - body[3]) * k, 1
  end
  local s = (ang / (2 * math.pi) * 24) % 1
  if r > 0.72 and r < 0.82 and math.abs(s - 0.5) < 0.2 then return body[1], body[2], body[3], 0.95 end
  local n = (ang / (2 * math.pi) * 8) % 1
  local tooth = math.abs(n - 0.5) * 2
  if r > 0.36 and r < 0.68 and tooth < (r - 0.36) / 0.32 * 0.62 then
    if tooth > (r - 0.36) / 0.32 * 0.46 then return rim[1], rim[2], rim[3], 0.95 end
    return body[1], body[2], body[3], 0.95
  end
  if r > 0.36 and r < 0.86 then return rim[1], rim[2], rim[3], 0.24 end
  return 0, 0, 0, 0
end
-- 大谷 R: tongues of violet fire flung out from a hollow middle: pale at the root, deep at the tip.
images.possession_flare = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r > 0.98 or r < 0.3 then return 0, 0, 0, 0 end
  local ang = math.atan(y, x)
  local n = 11
  local v = ang / (2 * math.pi) * n
  local i = math.floor(v)
  local reach = 0.62 + 0.34 * math.abs(math.sin(i * 2.4))
  if r > reach then return 0, 0, 0, 0 end
  local along = (r - 0.3) / (reach - 0.3)
  local bend = 0.16 * math.sin(along * 3.2 + i)
  local off = math.abs((v - i) - 0.5 + bend) * 2
  local width = 0.82 * (1 - along) ^ 0.7
  if off >= width then return 0, 0, 0, 0 end
  local edge = off / width
  if edge > 0.8 then return 0.14, 0.03, 0.3, 0.85 end
  local k = (1 - along) * (1 - edge * 0.6)
  return 0.4 + 0.5 * k, 0.1 + 0.42 * k, 0.76 + 0.24 * k, 1
end
-- 大谷 R: an ember of the same fire for what lingers and what flies off.
images.arcane_wisp = function(x, y)
  local r = math.sqrt(x * x + y * y)
  local ang = math.atan(y, x)
  local e = r / (0.72 + 0.2 * math.sin(ang * 3 + 0.7) + 0.06 * math.sin(ang * 7))
  if e >= 1 then return 0, 0, 0, 0 end
  local k = 1 - smooth(0.05, 0.75, e)
  return 0.42 + 0.48 * k, 0.12 + 0.44 * k, 0.8 + 0.2 * k, 1 - smooth(0.6, 1, e)
end
-- 大谷 R, finer: two thin rings, small rune ticks between them, slim teeth biting in; no floor.
images.possession_sigil = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r > 0.97 then return 0, 0, 0, 0 end
  local ang = math.atan(y, x)
  local function line(at, half)
    local d = math.abs(r - at) / half
    if d >= 1 then return nil end
    if d > 0.6 then return 0.16, 0.04, 0.32, 0.9 end
    local k = 1 - d / 0.6
    return 0.5 + 0.34 * k, 0.2 + 0.32 * k, 0.86 + 0.14 * k, 1
  end
  local cr, cg, cb, ca = line(0.92, 0.028)
  if cr then return cr, cg, cb, ca end
  cr, cg, cb, ca = line(0.7, 0.014)
  if cr then return cr, cg, cb, ca end
  local s = (ang / (2 * math.pi) * 32) % 1
  if r > 0.76 and r < 0.85 and math.abs(s - 0.5) < 0.1 then return 0.5, 0.22, 0.86, 0.9 end
  local n = (ang / (2 * math.pi) * 8) % 1
  local tooth = math.abs(n - 0.5) * 2
  if r > 0.42 and r < 0.69 and tooth < (r - 0.42) / 0.27 * 0.3 then return 0.42, 0.14, 0.76, 0.85 end
  return 0, 0, 0, 0
end
-- 大谷 R, finer: slim tongues, no dark outline, thin at the tip.
images.possession_flare = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r > 0.98 or r < 0.34 then return 0, 0, 0, 0 end
  local ang = math.atan(y, x)
  local n = 13
  local v = ang / (2 * math.pi) * n
  local i = math.floor(v)
  local reach = 0.6 + 0.36 * math.abs(math.sin(i * 2.4))
  if r > reach then return 0, 0, 0, 0 end
  local along = (r - 0.34) / (reach - 0.34)
  local off = math.abs((v - i) - 0.5 + 0.14 * math.sin(along * 3.2 + i)) * 2
  local width = 0.42 * (1 - along) ^ 0.8
  if off >= width then return 0, 0, 0, 0 end
  local k = (1 - along) * (1 - off / width)
  return 0.42 + 0.44 * k, 0.14 + 0.38 * k, 0.8 + 0.2 * k, 0.9 * (1 - smooth(0.6, 1, off / width))
end
-- 大谷 R, finer: a soft ember, translucent, no hard edge.
images.arcane_wisp = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r >= 0.95 then return 0, 0, 0, 0 end
  local k = 1 - smooth(0, 0.5, r)
  return 0.5 + 0.38 * k, 0.2 + 0.38 * k, 0.84 + 0.16 * k, 0.75 * (1 - smooth(0.15, 0.95, r))
end
-- 島津 W: the ground splits around the axe: deep red fire in dark-edged faults.
images.crack_quake = fissure(11, 3.7, 0.98, { 1.0, 0.5, 0.12 }, { 0.7, 0.14, 0.04 }, { 0.16, 0.05, 0.03 })
-- 大谷 普攻: a comet: a bright head at +x, a violet tail thinning behind it, two wisps beside the tail.
images.arcane_bolt = function(x, y)
  local head = math.sqrt(((x - 0.55) / 0.3) ^ 2 + (y / 0.2) ^ 2)
  if head < 1 then
    local k = 1 - smooth(0.1, 0.8, head)
    return 0.56 + 0.34 * k, 0.26 + 0.46 * k, 0.92 + 0.08 * k, 1 - smooth(0.8, 1, head)
  end
  if x > 0.6 or x < -0.95 then return 0, 0, 0, 0 end
  local along = (0.6 - x) / 1.55
  local best = 9
  for _, w in ipairs { { 0, 0.16 }, { 0.09, 0.05 }, { -0.09, 0.05 } } do
    local half = w[2] * (1 - along) ^ 1.1
    local centre = w[1] * math.sin(along * 3.14) 
    if half > 0.001 then best = math.min(best, math.abs(y - centre) / half) end
  end
  if best >= 1 then return 0, 0, 0, 0 end
  return 0.36 + 0.24 * (1 - along), 0.08 + 0.2 * (1 - along), 0.74 + 0.2 * (1 - along), (1 - smooth(0.5, 1, best)) * (1 - along * 0.75)
end
-- 大谷 普攻, slimmer: one long taper from a pointed bright head to nothing, not a ball on a tail.
images.arcane_bolt = function(x, y)
  if x > 0.92 or x < -0.95 then return 0, 0, 0, 0 end
  local widest = 0.42
  local half
  if x > widest then half = 0.15 * math.sqrt(1 - ((x - widest) / 0.5) ^ 2)
  else half = 0.15 * ((x + 0.95) / (widest + 0.95)) ^ 1.6 end
  if half <= 0.002 then return 0, 0, 0, 0 end
  local d = math.abs(y + 0.018 * math.sin(x * 9) * (0.92 - x)) / half
  if d >= 1 then return 0, 0, 0, 0 end
  local along = (0.92 - x) / 1.87
  local k = (1 - smooth(0, 0.8, d)) * (1 - smooth(0.05, 0.6, along))
  return 0.44 + 0.46 * k, 0.14 + 0.56 * k, 0.86 + 0.14 * k, (1 - smooth(0.65, 1, d)) * (1 - 0.7 * along)
end
-- 濃姬 W: the powder goes up: thin jets of fire of many lengths out of a soot ring, the ground showing between them.
images.powder_blast = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r > 0.98 then return 0, 0, 0, 0 end
  local ang = math.atan(y, x)
  local n = 27
  local v = ang / (2 * math.pi) * n
  local i = math.floor(v)
  local reach = 0.52 + 0.46 * math.abs(math.sin(i * 1.9) * math.cos(i * 0.7))
  if r > 0.3 and r < reach then
    local along = (r - 0.3) / (reach - 0.3)
    local off = math.abs((v - i) - 0.5 + 0.1 * math.sin(along * 4 + i)) * 2
    local width = 0.62 * (1 - along) ^ 0.75
    if off < width then
      local k = (1 - along) * (1 - off / width)
      return 0.72 + 0.28 * k, 0.16 + 0.5 * k, 0.03 + 0.14 * k, 1 - smooth(0.65, 1, off / width)
    end
  end
  if r < 0.42 then
    local e = math.abs(r - 0.3) / 0.12
    if e < 1 then return 0.14, 0.09, 0.07, 0.6 * (1 - e) end
    return 0.14, 0.09, 0.07, 0.16
  end
  return 0, 0, 0, 0
end
-- 德川 X: a soft blue mana rune: a blurred ring and six small diamonds round a faint glow; no hard edge anywhere.
images.mana_rune = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r > 0.96 then return 0, 0, 0, 0 end
  local ang = math.atan(y, x)
  local ring = 1 - smooth(0, 0.07, math.abs(r - 0.74))
  local thin = 0.55 * (1 - smooth(0, 0.03, math.abs(r - 0.42)))
  local s = (ang / (2 * math.pi) * 6) % 1 - 0.5
  local du = math.abs(s) * 2 * math.pi / 6 * 0.58
  local dv = math.abs(r - 0.58)
  local glyph = 0.9 * (1 - smooth(0.03, 0.09, du + dv))
  local glow = 0.16 * (1 - smooth(0.05, 0.6, r))
  local a2 = math.max(ring, thin, glyph, glow)
  if a2 <= 0.01 then return 0, 0, 0, 0 end
  local k = math.max(ring, glyph)
  return 0.16 + 0.34 * k, 0.42 + 0.34 * k, 0.95 + 0.05 * k, a2 * (1 - smooth(0.84, 0.96, r))
end
-- 大熊 X: tongues of fire out of a hollow, so the hero stays in sight; orange root, deep red tip, no outline.
images.blast_flare = function(x, y)
  local r = math.sqrt(x * x + y * y)
  if r > 0.98 or r < 0.28 then return 0, 0, 0, 0 end
  local ang = math.atan(y, x)
  local n = 15
  local v = ang / (2 * math.pi) * n
  local i = math.floor(v)
  local reach = 0.56 + 0.4 * math.abs(math.sin(i * 1.7))
  if r > reach then return 0, 0, 0, 0 end
  local along = (r - 0.28) / (reach - 0.28)
  local off = math.abs((v - i) - 0.5 + 0.16 * math.sin(along * 3 + i * 2)) * 2
  local width = 0.55 * (1 - along) ^ 0.8
  if off >= width then return 0, 0, 0, 0 end
  local k = (1 - along) * (1 - off / width)
  return 0.74 + 0.26 * k, 0.16 + 0.54 * k, 0.03 + 0.2 * k, 0.95 * (1 - smooth(0.55, 1, off / width))
end
-- 大熊 Q: 4x4 frames of a flame with a dark core and an orange-red outside, never pale. Rises, licks, dies.
images.flame_ember = function(x, y)
  local u, v = (x + 1) * 2, (y + 1) * 2
  local cx, cy = math.min(math.floor(u), 3), math.min(math.floor(v), 3)
  local lu, lv = (u - cx) * 2 - 1, (v - cy) * 2 - 1
  local frame = cy * 4 + cx
  local phase = frame / 16 * 2 * math.pi
  local h = (1 - lv) / 2
  local reach = 0.5 + 0.46 * math.sin(math.pi * (frame + 1) / 17) ^ 0.6
  if h >= reach then return 0, 0, 0, 0 end
  local along = h / reach
  local centre = along * (0.2 * math.sin(along * 4.5 + phase * 2) + 0.08 * math.sin(along * 11 - phase * 3))
  local width = 0.62 * (1 - along) ^ 0.7 * math.min(1, 0.45 + along * 4) * (0.85 + 0.15 * math.sin(along * 9 + phase))
  local d = math.abs(lu - centre) / math.max(width, 0.001)
  if d >= 1 then return 0, 0, 0, 0 end
  local inner = smooth(0.3, 0.7, d)
  local outer = smooth(0.68, 0.95, d)
  -- The sprite's opacity comes from its brightness, in linear light: a core darker than this is a hole.
  local r = 0.68 + (0.88 - 0.68) * inner + (1.0 - 0.88) * outer
  local g = 0.1 + (0.22 - 0.1) * inner + (0.52 - 0.22) * outer
  local b = 0.04 + (0.03 - 0.04) * inner + (0.08 - 0.03) * outer
  local fade = (1 - smooth(0.86, 1, d)) * (1 - smooth(0.72, 1, along)) * smooth(0, 0.12, along)
  return r * fade, g * fade, b * fade, 1
end
-- 大熊 Q: a thin broken ring of fire, dark at its edges, for the pulse.
images.ring_thin_fire = function(x, y)
  local r = math.sqrt(x * x + y * y)
  local ang = math.atan(y, x)
  local curve = 0.9 + 0.012 * math.sin(ang * 13) + 0.008 * math.sin(ang * 29)
  local half = 0.03 * (0.55 + 0.45 * math.abs(math.sin(ang * 3.5 + 0.6)))
  local d = math.abs(r - curve) / half
  if d >= 1 then return 0, 0, 0, 0 end
  local k = 1 - smooth(0.2, 0.9, d)
  return 0.62 + 0.38 * k, 0.1 + 0.42 * k, 0.03 + 0.06 * k, 0.95 * (1 - smooth(0.75, 1, d))
end
local crc_table = {}
for i = 0, 255 do local c = i; for _ = 1, 8 do c = (c & 1) ~= 0 and (0xEDB88320 ~ (c >> 1)) or (c >> 1) end; crc_table[i] = c end
local function crc32(s) local c = 0xFFFFFFFF; for i = 1, #s do c = crc_table[(c ~ s:byte(i)) & 255] ~ (c >> 8) end; return (c ~ 0xFFFFFFFF) & 0xFFFFFFFF end
local function chunk(kind, data) return string.pack(">I4", #data) .. kind .. data .. string.pack(">I4", crc32(kind .. data)) end
for name, shape in pairs(images) do
  local rows = {}
  for py = 0, N - 1 do
    local row = { "\0" }
    local y = (py + 0.5) / N * 2 - 1
    for px = 0, N - 1 do
      local x = (px + 0.5) / N * 2 - 1
      local r, g, b, al = shape(x, y)
      row[#row + 1] = string.char(math.floor(r * 255 + 0.5), math.floor(g * 255 + 0.5), math.floor(b * 255 + 0.5), math.floor(al * 255 + 0.5))
    end
    rows[#rows + 1] = table.concat(row)
  end
  local raw = table.concat(rows)
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
  local f = assert(io.open(dir .. "/gen-cue-fx_" .. name .. ".png", "wb"))
  f:write("\x89PNG\r\n\x1a\n", chunk("IHDR", string.pack(">I4I4BBBBB", N, N, 8, 6, 0, 0, 0)), chunk("IDAT", table.concat(z)), chunk("IEND", ""))
  f:close()
  print("wrote", name)
end
