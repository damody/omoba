-- The shared skill effect family (work line C, D21): a few pre-compiled Niagara systems and flat images in
-- the combat visual pool, with every cue's look as data. Scopes, durations and damage shapes come from the
-- package values at run time (the cue's radius and duration); nothing here repeats them.
--
--   tools\lua\lua.exe scripts\fx_family.lua --spec OUT.json       write the commandlet spec
--   tools\lua\lua.exe scripts\fx_family.lua --ini INI             merge the recipes into CueOverrides of INI
--   tools\lua\lua.exe scripts\fx_family.lua --check               every recipe names known images and systems
--
-- Then: UnrealEditor-Cmd <project> -run=OmCueVfx -fx-family=OUT.json (assets under /Game/Omoba/VFX/Fx).
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local json = b.lib("json")
local path = b.lib("path")

local M = {}
M.ROOT = "/Game/Omoba/VFX/Fx"
M.ART = "omfue/SourceArt/CueReferences/gen-cue-"
local E = "/Niagara/DefaultAssets/Templates/Emitters/"
local function emitter(name) return E .. name .. "." .. name end

-- Reviewed images (scripts/generated_art.lua, kind cue). rot turns the image so its top is the direction
-- of travel's +X; spin (radians a second) is for flat images only.
M.IMAGES = {
  flame_sheet = { cols = 4, gain = 1.6 },
  smoke_sheet = { cols = 4, gain = 0.6, mono = 1 },
  spark_star = { gain = 1.6 },
  shock_ring = { gain = 1.5 },
  crescent_slash = { rot = 0.785, gain = 1.6 },
  swallow = { rot = 0.785, gain = 1.8 },
  arcane_circle = { gain = 1.2, spin = 0.8 },
  vortex = { gain = 1.2, spin = 2.5 },
  lightning = { gain = 2.0 },
  soft_glow = { gain = 1.4, mono = 1 },
}

-- main: takes the cue's image, size and life. The rest keep their own (their colour still follows the cue).
M.SYSTEMS = {
  -- explosion: a few flipbook puffs thrown outward, with sparks
  burst = { { template = emitter("SimpleSpriteBurst"), main = true,
      set = { ["SpawnBurst_Instantaneous.Spawn Count"] = 5 } },
    { template = emitter("OmnidirectionalBurst"), set = { ["SpawnBurst_Instantaneous.Spawn Count"] = 14 } } },
  -- one image that flares and fades: flash, glow, lightning strike
  pop = { { template = emitter("SimpleSpriteBurst"), main = true,
      set = { ["SpawnBurst_Instantaneous.Spawn Count"] = 1 } } },
  -- sparks alone, for a small hit
  sparks = { { template = emitter("OmnidirectionalBurst"), set = { ["SpawnBurst_Instantaneous.Spawn Count"] = 18 } } },
  -- keeps emitting while it lasts: a trail behind a flight, smoke, motes around a unit
  stream = { { template = emitter("Fountain"), main = true,
      set = { ["SpawnRate.SpawnRate"] = 24 } } },
}

-- Looks shared by many cues: fields of a recipe, overridden by the cue's own row.
M.LOOKS = {
  explosion = { system = "burst", sprite = "flame_sheet", size = 180, life = 0.7, ground = "shock_ring", ground_size = 320, ground_seconds = 0.45, grow = 0.3 },
  explosion_small = { system = "burst", sprite = "flame_sheet", size = 90, life = 0.5 },
  flash = { system = "pop", sprite = "soft_glow", size = 160, life = 0.4 },
  hit_sparks = { system = "sparks" },
  flight_flame = { system = "stream", sprite = "flame_sheet", size = 70, life = 0.4, loop = true, face = true,
    ground = "soft_glow", ground_size = 110, ground_seconds = 20, height = 120 },
  flight_glow = { system = "stream", sprite = "soft_glow", size = 50, life = 0.35, loop = true, face = true,
    ground = "soft_glow", ground_size = 90, ground_seconds = 20, height = 120 },
  aura_ground = { ground = "arcane_circle", ground_size = 260, ground_seconds = 2 },
  slash = { ground = "crescent_slash", ground_size = 340, ground_seconds = 0.3, height = 90, grow = 0.7 },
  shockwave = { ground = "shock_ring", ground_size = 400, ground_seconds = 0.5, grow = 0.15 },
  line = { ground = "lightning", ground_size = 120, ground_seconds = 0.35, line = true },
}

local ARCANE, GREEN, BLUE, GOLD, STEEL = { 0.72, 0.35, 1.0 }, { 0.35, 1.0, 0.5 }, { 0.35, 0.65, 1.0 }, { 1.0, 0.8, 0.35 }, { 0.75, 0.9, 1.0 }
-- cue id -> its look. tint only where the image's own colour is not what the skill says.
M.RECIPES = {
  -- A12 大谷吉繼: arcane purple; insight flash, mystic pierce, awe vortex, mystic arrow, possession
  a12w_burst = { look = "flash", tint = ARCANE, sprite = "arcane_circle", size = 200, life = 0.5 },
  a12w_glow = { look = "flash", tint = ARCANE },
  a12w_hide_ring = { look = "shockwave", tint = ARCANE },
  a12e_heal_number = { look = "flash", tint = GREEN, size = 110 },
  a12e_mana_number = { look = "flash", tint = BLUE, size = 110 },
  a12r_burst = { look = "flash", tint = ARCANE, size = 240, life = 0.5, ground = "vortex", ground_size = 420, ground_seconds = 0.7 },
  a12r_vortex = { ground = "vortex", ground_size = 150, ground_seconds = 1.2, tint = ARCANE },
  a12f_arrow = { look = "flight_glow", tint = ARCANE, ground = "spark_star", ground_size = 150 },
  a12t2 = { look = "flight_glow", tint = ARCANE },
  a12t_aura = { look = "aura_ground", tint = ARCANE },
  -- C22 佐佐木小次郎: steel blue sword light; swallows and a thunder clap
  c22d_hand_glow = { ground = "soft_glow", ground_size = 140, ground_seconds = 1.2, height = 100, tint = STEEL },
  c22d_return_projectile = { look = "line", tint = STEEL },
  c22e_swallow = { look = "flight_glow", tint = STEEL, ground = "swallow", ground_size = 200 },
  c22e_impact = { look = "slash", tint = STEEL },
  c22w_swallow_ring = { system = "stream", sprite = "swallow", size = 70, life = 0.6, loop = true, tint = STEEL,
    ground = "shock_ring", ground_size = 300, ground_seconds = 1 },
  c22t_thunder_clap = { system = "pop", sprite = "lightning", size = 420, life = 0.35, tint = STEEL,
    ground = "shock_ring", ground_size = 500, ground_seconds = 0.5, grow = 0.15 },
  -- A26 濃姬: gunpowder fire and smoke
  a26w_hit_explosion = { look = "explosion_small" },
  a26e_smoke = { system = "burst", sprite = "smoke_sheet", size = 150, life = 0.8 },
  a26t_bomb_arcana = { look = "flight_flame", ground_size = 170 },
  a26t_small_bomb = { look = "flight_flame", ground_size = 80, size = 40 },
  a26t_overhead_bomb = { ground = "soft_glow", ground_size = 120, ground_seconds = 1.2, height = 220, tint = GOLD },
  a26t_big_explosion = { look = "explosion", size = 340, ground_size = 560 },
  a26t_small_explosion = { look = "explosion_small" },
}

function M.material(prefix, image) return ("%s/Instances/%s_fx_%s.%s_fx_%s"):format(M.ROOT, prefix, image, prefix, image) end
function M.system(name) return ("%s/Systems/NS_fx_%s.NS_fx_%s"):format(M.ROOT, name, name) end

-- A recipe with its look applied.
function M.resolve(id)
  local row = assert(M.RECIPES[id], "no recipe for " .. id)
  local out = {}
  if row.look then for k, v in pairs(assert(M.LOOKS[row.look], id .. ": look " .. row.look)) do out[k] = v end end
  for k, v in pairs(row) do out[k] = v end
  return out
end

local function sorted(t)
  local keys = {}
  for k in pairs(t) do keys[#keys + 1] = k end
  table.sort(keys)
  return keys
end

function M.spec(root)
  local textures, systems = {}, {}
  for _, id in ipairs(sorted(M.IMAGES)) do
    local row = { id = "fx_" .. id, file = (root and (root .. "/") or "") .. M.ART .. "fx_" .. id .. ".png" }
    for k, v in pairs(M.IMAGES[id]) do row[k] = v end
    textures[#textures + 1] = row
  end
  for _, id in ipairs(sorted(M.SYSTEMS)) do systems[#systems + 1] = { id = id, emitters = M.SYSTEMS[id] } end
  return { schema = "omoba.fx-family.v1", textures = textures, systems = systems }
end

function M.check()
  local problems = {}
  for _, id in ipairs(sorted(M.RECIPES)) do
    local r = M.resolve(id)
    if r.system and not M.SYSTEMS[r.system] then problems[#problems + 1] = id .. ": system " .. r.system end
    if r.sprite and not M.IMAGES[r.sprite] then problems[#problems + 1] = id .. ": image " .. r.sprite end
    if r.ground and not M.IMAGES[r.ground] then problems[#problems + 1] = id .. ": ground image " .. r.ground end
    if not r.system and not r.ground then problems[#problems + 1] = id .. ": draws nothing" end
  end
  return problems
end

local function number(v) return (("%.3f"):format(v):gsub("0+$", ""):gsub("%.$", "")) end

-- One CueOverrides row. Radius and duration are not here: they arrive with the cue from the package.
function M.ini_row(id)
  local r = M.resolve(id)
  local f = {}
  local function add(k, v) f[#f + 1] = k .. "=" .. v end
  if r.system then add("Effect", '"' .. M.system(r.system) .. '"') end
  if r.tint then add("EffectTint", ("(R=%s,G=%s,B=%s,A=1.0)"):format(number(r.tint[1]), number(r.tint[2]), number(r.tint[3]))) end
  if r.sprite then add("Sprite", '"' .. M.material("MI", r.sprite) .. '"') end
  if r.size then add("FxSizeCm", number(r.size)) end
  if r.life then add("FxSeconds", number(r.life)) end
  if r.loop then add("bFxLoop", "True") end
  if r.face then add("bFxFaceMotion", "True") end
  if r.ground then add("Ground", '"' .. M.material("MIG", r.ground) .. '"') end
  if r.ground_size then add("GroundSizeCm", number(r.ground_size)) end
  if r.ground_seconds then add("GroundSeconds", number(r.ground_seconds)) end
  if r.grow then add("GroundGrow", number(r.grow)) end
  if r.line then add("bGroundLine", "True") end
  if r.height then add("GroundHeightCm", number(r.height)) end
  return ('("%s",(%s))'):format(id, table.concat(f, ","))
end

-- The top-level rows of a CueOverrides=(...) value: { {id, text}... } in order.
function M.split_rows(value)
  local inner = assert(value:match("^%((.*)%)$"), "CueOverrides is not a map")
  local rows, depth, start, quoted = {}, 0, nil, false
  for i = 1, #inner do
    local c = inner:sub(i, i)
    if c == '"' then quoted = not quoted end
    if not quoted then
      if c == "(" then
        if depth == 0 then start = i end
        depth = depth + 1
      elseif c == ")" then
        depth = depth - 1
        if depth == 0 then
          local text = inner:sub(start, i)
          rows[#rows + 1] = { id = assert(text:match('^%("([^"]+)"'), text:sub(1, 40)), text = text }
        end
      end
    end
  end
  assert(depth == 0, "unbalanced CueOverrides")
  return rows
end

function M.merge_ini(text)
  local nl = text:find("\r\n", 1, true) and "\r\n" or "\n"
  local before, value, after = text:match("^(.-CueOverrides=)(%b())(.*)$")
  assert(before, "no CueOverrides in the ini")
  local rows, seen = M.split_rows(value), {}
  for _, row in ipairs(rows) do
    if M.RECIPES[row.id] then row.text = M.ini_row(row.id); seen[row.id] = true end
  end
  for _, id in ipairs(sorted(M.RECIPES)) do
    if not seen[id] then rows[#rows + 1] = { id = id, text = M.ini_row(id) } end
  end
  local out = {}
  for i, row in ipairs(rows) do out[i] = row.text end
  return before .. "(" .. table.concat(out, ",") .. ")" .. after, nl
end

if arg and arg[0] and arg[0]:match("fx_family%.lua$") then
  local i = 1
  while arg[i] do
    if arg[i] == "--spec" then
      json.write(assert(arg[i + 1], "--spec OUT"), M.spec(b.root), true); i = i + 2
    elseif arg[i] == "--ini" then
      local file = assert(arg[i + 1], "--ini INI")
      path.write(file, (M.merge_ini(path.read(file, true))), true, true); i = i + 2
    elseif arg[i] == "--check" then
      local problems = M.check()
      for _, p in ipairs(problems) do print(p) end
      print(#problems .. " problem(s), " .. #sorted(M.RECIPES) .. " recipes")
      if #problems > 0 then os.exit(1) end
      i = i + 1
    else error("unknown argument " .. arg[i]) end
  end
end
return M
