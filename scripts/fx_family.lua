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
  net = { gain = 1.0 },
  rock = { gain = 1.2, spin = 6, soft = 150 },
  throw_axe = { gain = 1.3, spin = 18 },
  war_hammer = { rot = 1.571, gain = 1.3 },
  fire_wings = { rot = 1.571, gain = 1.3 },
  heal_ring = { gain = 1.3, spin = 0.6 },
  iron_shield = { gain = 1.1, spin = 0.5 },
  banner_rally = { gain = 1.2, spin = 0.4 },
  dust_crack = { gain = 1.3 },
  chain_bolt = { gain = 1.8 },
  slow_chains = { gain = 1.3, spin = -0.8 },
  blink_flash = { gain = 1.5 },
  -- Drawn by scratch code, not generated: a shape that has to read against the ground, so its colours stay as drawn.
  spear_thrust = { rot = -1.571, gain = 1.0 },
  spear_thrust_blue = { rot = -1.571, gain = 1.0 },
  -- Drawn the same way, light bodies that take the cue's tint: one shape per skill.
  stack_spears = { gain = 1.0 },
  axe_arc = { gain = 1.3 },
  twin_muzzle = { rot = -1.571, gain = 1.0 },
  iai_cut = { gain = 1.0 },
  cut_cross = { gain = 1.0 },
  bolt_fork = { gain = 1.8 },
  quake_burst = { gain = 1.0 },
  heal_cross = { gain = 1.0 },
  heal_rise = { gain = 1.0 },
  powder_ring = { gain = 1.0 },
  arcane_bolt = { gain = 1.0 }, crack_quake = { gain = 1.0 }, possession_sigil = { gain = 1.0 }, possession_flare = { gain = 1.0 }, arcane_wisp = { gain = 1.0 }, blink_trace = { gain = 1.0 }, thrust_star = { gain = 1.0 }, arc_fork = { gain = 1.0 }, crack_red = { gain = 1.0 }, crack_earth = { gain = 1.0 }, crack_gold = { gain = 1.0 }, crack_hammer = { gain = 1.0 },
  heal_cross_soft = { gain = 1.0 }, mist_cuts = { gain = 1.0 }, heal_cross_solid = { gain = 1.0 },
  pistol_flare = { rot = -1.571, gain = 1.0 },
  pistol_shot = { rot = -1.571, gain = 1.0 },
  powder_swirl = { gain = 1.0, spin = 8.98 },
}

-- main: takes the cue's image, size and life. The rest keep their own (their colour still follows the cue).
-- Spray in world centimetres: a hero stands about 14,000 tall (a recipe's sizes are in backend units, M.WORLD_CM).
local function SPARKS(count)
  return { ["SpawnBurst_Instantaneous.Spawn Count"] = count, ["AddVelocity.Velocity Speed Scale"] = 120,
    ["InitializeParticle.Uniform Sprite Size Min"] = 180, ["InitializeParticle.Uniform Sprite Size Max"] = 520,
    ["InitializeParticle.Lifetime Min"] = 0.25, ["InitializeParticle.Lifetime Max"] = 0.55 }
end
M.SYSTEMS = {
  -- explosion: a few flipbook puffs thrown outward, with sparks
  burst = { { template = emitter("SimpleSpriteBurst"), main = true,
      set = { ["SpawnBurst_Instantaneous.Spawn Count"] = 5 } },
    { template = emitter("OmnidirectionalBurst"), set = SPARKS(14) } },
  -- one image that flares and fades: flash, glow, lightning strike
  pop = { { template = emitter("SimpleSpriteBurst"), main = true,
      set = { ["SpawnBurst_Instantaneous.Spawn Count"] = 1 } } },
  -- sparks alone, for a small hit
  sparks = { { template = emitter("OmnidirectionalBurst"), set = SPARKS(18) } },
  -- keeps emitting while it lasts: a trail behind a flight, smoke, motes around a unit
  -- fire with volume: many flipbook tongues thrown out and rising, not one flat image
  blaze = { { template = emitter("OmnidirectionalBurst"), main = true,
      set = { ["SpawnBurst_Instantaneous.Spawn Count"] = 14, ["AddVelocity.Velocity Speed Scale"] = 45, ["GravityForce.Gravity"] = { 0, 0, 9000 } } } },
  -- a few thin images around a unit for as long as a status lasts: the hero stays in sight
  aura = { { template = emitter("Fountain"), main = true,
      set = { ["SpawnRate.SpawnRate"] = 9, ["GravityForce.Gravity"] = { 0, 0, 0 }, ["AddVelocity.Velocity Speed Scale"] = 10, ["ShapeLocation.Sphere Radius"] = 5000 } } },
  stream = { { template = emitter("Fountain"), main = true,
      set = { ["SpawnRate.SpawnRate"] = 40, ["GravityForce.Gravity"] = { 0, 0, 0 }, ["AddVelocity.Velocity Speed Scale"] = 8, ["ShapeLocation.Sphere Radius"] = 4500 } } },
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

local ARCANE, GREEN, BLUE, GOLD, STEEL = { 0.5, 0.1, 1.0 }, { 0.1, 0.9, 0.25 }, { 0.08, 0.3, 1.0 }, { 1.0, 0.55, 0.08 }, { 0.12, 0.5, 1.0 }
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
  c22d_hand_glow = { system = "stream", sprite = "spark_star", size = 70, life = 0.5, loop = true, tint = STEEL },
  c22d_return_projectile = { look = "line", tint = STEEL },
  c22d_cast_sound = { system = "burst", sprite = "blink_flash", size = 420, life = 0.5, tint = STEEL,
    ground = "shock_ring", ground_size = 460, ground_seconds = 0.5, grow = 0.2 },
  c22e_voice = { system = "pop", sprite = "crescent_slash", size = 340, life = 0.3, tint = STEEL },
  c22e_swallow = { system = "stream", sprite = "soft_glow", size = 50, life = 0.5, loop = true, face = true, tint = STEEL,
    mesh = "swallow", ground_size = 190, ground_seconds = 20, height = 90 },
  c22e_impact = { system = "burst", sprite = "crescent_slash", size = 340, area = true, life = 0.5, tint = STEEL,
    ground = "blink_flash", ground_size = 340, ground_seconds = 0.4, height = 30, grow = 0.4 },
  c22r_crit_sound = { system = "pop", sprite = "blink_flash", size = 150, life = 0.2, tint = GOLD,
    ground = "crescent_slash", ground_size = 300, ground_seconds = 0.22, height = 90, grow = 0.5, face_camera = 0.9, ground_roll = 60 },
  c22w_voice = { system = "burst", sprite = "swallow", size = 210, life = 0.5, tint = STEEL },
  c22w_swallow_ring = { system = "stream", sprite = "swallow", size = 130, life = 1.1, loop = true, bold = true, tint = STEEL,
    ground = "vortex", ground_size = 480, ground_seconds = 1.5 },
  c22t_thunder_clap = { system = "pop", sprite = "cut_cross", size = 380, fixed_size = true, life = 0.35, tint = STEEL, also = { "fx_layer_mist_cuts" } },
  -- A26 濃姬: gunpowder fire and smoke
  a26w_hit_explosion = { look = "explosion_small" },
  a26e_smoke = { system = "burst", sprite = "smoke_sheet", size = 150, life = 0.8 },
  a26t_bomb_arcana = { look = "flight_flame", ground_size = 170 },
  a26t_small_bomb = { look = "flight_flame", ground_size = 80, size = 40 },
  a26t_overhead_bomb = { ground = "soft_glow", ground_size = 120, ground_seconds = 1.2, height = 220, tint = GOLD },
  a26t_big_explosion = { look = "explosion", size = 340, ground_size = 560 },
  a26t_small_explosion = { look = "explosion_small" },
  spell_hint = { ground = "shock_ring", ground_size = 400, ground_seconds = 0.8, tint = { 1.0, 0.45, 0.2 } },
  -- The cue's radius is the ability's reveal radius, not a mark to draw: the cast shows in the hero's own cue.
  spell_hint_self = { ground = "arcane_circle", ground_size = 400, ground_seconds = 0.8, tint = ARCANE, alpha = 0.01 },
  move_slow = { ground = "slow_chains", ground_size = 140, ground_seconds = 1.2, tint = BLUE },
  c22w_hide_model = { ground = "soft_glow", ground_size = 180, ground_seconds = 1.2, tint = STEEL },
  -- A01 德川家康: gold rally, healing green, thunder blade, a binding net
  a01e_net = { ground = "net", ground_size = 180, ground_seconds = 1.2 },
  a01r_lightning = { system = "pop", sprite = "lightning", size = 240, life = 0.3, tint = BLUE, ground = "blink_flash", ground_size = 160, ground_seconds = 0.25 },
  a01t_banner = { ground = "banner_rally", ground_size = 320, ground_seconds = 1.2 },
  a01t_ring = { ground = "shock_ring", ground_size = 110, ground_seconds = 1.2, tint = GOLD },
  a01w_healing_ring = { ground = "heal_ring", ground_size = 150, ground_seconds = 1.2 },
  a01w_holy_bolt = { look = "flash", tint = GREEN, size = 220, ground = "heal_ring", ground_size = 420, ground_seconds = 0.8, grow = 0.3 },
  -- B20 山縣昌景: rocks and the shattered ground
  b20e_burrow = { ground = "dust_crack", ground_size = 460, ground_seconds = 0.7, grow = 0.5 },
  b20e_thunderclap = { look = "shockwave", ground_size = 520, tint = GOLD },
  b20t_epicenter = { system = "burst", sprite = "rock", size = 110, fixed_size = true, life = 0.5, tint = { 1.0, 0.45, 0.1 }, ground = "crack_earth", ground_size = 360, ground_seconds = 0.5, grow = 0.3 },
  b20w_rock = { ground = "rock", ground_size = 110, ground_seconds = 0.7, height = 80, fly = true, system = "sparks" },
  -- B26 大熊朝秀: fire charging and bursting
  b26d_thunder_clap = { look = "shockwave", tint = { 1.0, 0.5, 0.15 } },
  b26e_model_scale_up = { look = "flash", tint = { 1.0, 0.25, 0.15 }, size = 240 },
  b26e_status_effect = { ground = "shock_ring", ground_size = 150, ground_seconds = 1.2, tint = { 1.0, 0.25, 0.15 } },
  b26t_burst = { look = "explosion", size = 240, ground = "fire_wings", ground_size = 420, grow = 0.5 },
  b26t_wings = { ground = "fire_wings", ground_size = 240, ground_seconds = 1.2 },
  b26w_charge_fire = { ground = "vortex", ground_size = 220, ground_seconds = 1.2, tint = { 1.0, 0.45, 0.1 } },
  b26w_charge_pulse = { look = "shockwave", ground_size = 360, tint = { 1.0, 0.5, 0.15 } },
  b26w_fire_explode = { look = "explosion_small" },
  b26w_shockwave = { look = "explosion", ground_size = 620 },
  b26w_slow_overhead = { ground = "slow_chains", ground_size = 130, ground_seconds = 1.2, height = 200, tint = BLUE },
  -- A14 前田利家: spear thrusts and lightning rage
  a14e_blink_sparkles = { system = "sparks", ground = "blink_flash", ground_size = 240, ground_seconds = 0.3, tint = GOLD },
  a14t_lightning = { ground = "lightning", ground_size = 240, ground_seconds = 1.2, tint = BLUE },
  a14w_impact = { look = "slash", ground = "blink_flash", tint = GOLD },
  a14w_move_slow = { ground = "slow_chains", ground_size = 140, ground_seconds = 1.2, tint = BLUE },
  -- B27 小島貞興: axe throw and ground slam
  b27e_axe = { look = "flight_glow", ground = "throw_axe", ground_size = 170, tint = STEEL },
  b27e_hit_sound = { system = "sparks", tint = STEEL },
  b27r_b15w3_aura = { ground = "iron_shield", ground_size = 180, ground_seconds = 1.2, tint = GOLD },
  b27r_model_grow = { look = "flash", tint = GOLD, size = 220 },
  b27t_move_slow = { ground = "slow_chains", ground_size = 140, ground_seconds = 1.2, tint = BLUE },
  b27t_screen_shake = { look = "shockwave", ground_size = 240, tint = GOLD },
  b27t_slam = { system = "burst", sprite = "smoke_sheet", size = 240, life = 0.7, ground = "dust_crack", ground_size = 900, ground_seconds = 0.8, grow = 0.2 },
  b27w_armlet_glow = { ground = "soft_glow", ground_size = 180, ground_seconds = 1.2, tint = { 1.0, 0.35, 0.2 } },
  -- C03 島津義弘: iron body, quake and the great hammer
  c03e_move_slow = { ground = "slow_chains", ground_size = 140, ground_seconds = 1.2, tint = BLUE },
  c03e_thunder_clap = { system = "sparks", ground = "dust_crack", ground_size = 460, ground_seconds = 0.6, grow = 0.3, tint = GOLD },
  c03r_thunder = { system = "pop", sprite = "lightning", size = 220, life = 0.3, tint = BLUE },
  c03t_hammer = { ground = "war_hammer", ground_size = 420, ground_seconds = 5, height = 60, fly = true },
  c03t_hammer_hit = { look = "explosion_small", ground = "dust_crack", ground_size = 280, ground_seconds = 0.4, grow = 0.5 },
  c03w_grow = { look = "flash", tint = STEEL, size = 200 },
  c03w_iron_glow = { ground = "iron_shield", ground_size = 190, ground_seconds = 1.2 },
  -- C05 前田慶次: lightning spear
  c05e_bolt = { look = "flight_glow", tint = BLUE, ground = "lightning", ground_size = 150 },
  c05e_impact = { system = "pop", sprite = "lightning", size = 180, life = 0.25, tint = BLUE },
  c05r_chain_lightning = { look = "line", ground = "chain_bolt", tint = BLUE },
  c05r_tempest = { ground = "lightning", ground_size = 170, ground_seconds = 1.2, tint = BLUE },
  c05t_aoe_hint = { ground = "arcane_circle", ground_size = 400, ground_seconds = 1.5, tint = BLUE },
  c05t_aoe_hint_enemy = { ground = "arcane_circle", ground_size = 400, ground_seconds = 1.5, tint = { 1.0, 0.25, 0.2 } },
  c05t_blink_end = { look = "flash", tint = BLUE, ground = "blink_flash", ground_size = 340, ground_seconds = 0.35 },
  c05t_blink_start = { look = "flash", tint = BLUE, ground = "blink_flash", ground_size = 340, ground_seconds = 0.35 },
  c05t_eclipse_area = { ground = "vortex", ground_size = 400, ground_seconds = 1.5, tint = BLUE },
  c05w_armlet = { ground = "soft_glow", ground_size = 170, ground_seconds = 1.2, tint = BLUE },
  -- The struck flash on any unit (OmUnitActor::OnDamageCue).
  fx_hit = { ground = "blink_flash", ground_size = 150, ground_seconds = 0.25, height = 30, grow = 0.4, tint = { 1.0, 0.6, 0.25 } },
}

-- One file per hero (scripts/fx_recipes/<hero>.lua) holds its abilities' looks; a row replaces the same id above.
for _, hero in ipairs { "tokugawa_ieyasu", "yamagata_masakage", "okuma_tomohide", "otani_yoshitsugu", "maeda_toshiie", "nohime", "kojima_sadaoki", "shimazu_yoshihiro", "maeda_keiji", "package_cues" } do
  for id, row in pairs(dofile(scripts_dir .. "/fx_recipes/" .. hero .. ".lua").recipes) do M.RECIPES[id] = row end
end
M.RECIPES.fx_stun = { system = "stream", sprite = "spark_star", size = 60, life = 0.5, loop = true, tint = GOLD }

-- Basic attacks, generated: fx_swing_<hero> at the attacker, fx_strike_<hero> at the target (OmUnitActor::PlayAttackFx).
M.WEAPONS = {
  katana = { swing = { ground = "crescent_slash", ground_size = 170, ground_seconds = 0.16, height = 90, grow = 0.6, face_camera = 0.7 },
    strike = { system = "burst", sprite = "spark_star", size = 190, life = 0.3, ground = "crescent_slash", ground_size = 280, ground_seconds = 0.28, height = 80, grow = 0.5, face_camera = 0.7 } },
  axe = { swing = { ground = "crescent_slash", ground_size = 230, ground_seconds = 0.2, height = 110, grow = 0.6, face_camera = 0.9, ground_roll = 90 },
    strike = { system = "burst", sprite = "rock", size = 200, life = 0.4, ground = "crescent_slash", ground_size = 300, ground_seconds = 0.3, height = 90, grow = 0.4, face_camera = 0.9, ground_roll = 90 } },
  spear = { swing = { ground = "chain_bolt", ground_size = 90, ground_seconds = 0.2, line = true },
    strike = { system = "pop", sprite = "spark_star", size = 200, life = 0.26, ground = "chain_bolt", ground_size = 200, ground_seconds = 0.24, height = 80, line = false, grow = 0.5, face_camera = 0.7 } },
  gun = { swing = { system = "pop", sprite = "spark_star", size = 110, life = 0.14,
      ground = "spark_star", ground_size = 90, ground_seconds = 0.2, height = 70, fly = true },
    strike = { system = "burst", sprite = "shock_ring", size = 170, life = 0.3, ground = "blink_flash", ground_size = 170, ground_seconds = 0.25, height = 30, grow = 0.4 } },
  arcane = { swing = { system = "pop", sprite = "soft_glow", size = 60, life = 0.14, ground = "arcane_bolt", ground_size = 120, ground_seconds = 0.26, height = 70, fly = true, face_camera = 0.5 },
    strike = { system = "burst", sprite = "spark_star", size = 170, life = 0.28, ground = "shock_ring", ground_size = 170, ground_fixed = true, ground_seconds = 0.26, height = 30, grow = 0.4 } },
}
M.ATTACKS = {
  sasaki_kojiro = { "katana", STEEL, { system = "pop", sprite = "iai_cut", size = 380, life = 0.24, ground = "soft_glow", ground_size = 150, ground_seconds = 0.2, height = 30, grow = 0.4, face_camera = false } }, tokugawa_ieyasu = { "katana", GOLD }, yamagata_masakage = { "katana", { 1.0, 0.22, 0.06 }, { ground_size = 230, ground_roll = -30 } },
  okuma_tomohide = { "axe", { 1.0, 0.13, 0.01 }, { system = false, sprite = false, size = false, life = false, ground = "axe_arc", ground_size = 135, ground_seconds = 0.26, height = 70, lead = -40, grow = 0.7, face_camera = 0.85, ground_roll = false } },
  kojima_sadaoki = { "axe", { 1.0, 0.75, 0.3 }, { sprite = "spark_star", size = 170, ground = "dust_crack", ground_size = 280, height = false, face_camera = false, ground_roll = false } },
  shimazu_yoshihiro = { "axe", { 1.0, 0.25, 0.12 }, { sprite = "spark_star", size = 190 } },
  maeda_toshiie = { "spear", { 1.0, 0.62, 0.05 }, { sprite = "spark_star", size = 120, life = 0.2, tint = { 1, 1, 1 }, ground = "spear_thrust", ground_size = 260, ground_seconds = 0.3, height = 70, lead = -125, line = false, grow = 0.85, face_camera = 0.85 } }, maeda_keiji = { "spear", BLUE, { sprite = "spark_star", size = 70, life = 0.2, tint = { 1, 1, 1 }, ground = "spear_thrust_blue", ground_size = 200, ground_seconds = 0.3, height = 70, lead = -125, line = false, grow = 0.85, face_camera = 0.85 } },
  nohime = { "gun", { 1.0, 0.6, 0.25 }, { system = "pop", sprite = "blink_flash", size = 220, life = 0.22, ground = "shock_ring", ground_size = 200, ground_seconds = 0.25, height = 30, grow = 0.4, face_camera = false } }, otani_yoshitsugu = { "arcane", ARCANE, { size = 170, area = false, ground_size = 180 } },
}
for hero, row in pairs(M.ATTACKS) do
  for part, recipe in pairs(M.WEAPONS[row[1]]) do
    local id = { tint = row[2] }
    for k, v in pairs(recipe) do id[k] = v end
    -- A hero's own blow where the weapon's would not read.
    if part == "strike" and row[3] then for k, v in pairs(row[3]) do id[k] = v end end
    M.RECIPES["fx_" .. part .. "_" .. hero] = id
  end
end

-- Her shots leave from the muzzle of the pistol firing now (OmUnitActor::IsFiringLeftHand), at its own height.
M.RECIPES.fx_swing_nohime.from_hand, M.RECIPES.fx_swing_nohime.height = true, nil

-- Layers other cues play through `also`: no package names them.
M.RECIPES.fx_layer_mist_cuts = { ground = "mist_cuts", ground_size = 1500, area = true, ground_seconds = 0.7, grow = 0.75 }
M.RECIPES.fx_layer_possession_flare = { ground = "possession_flare", ground_size = 300, ground_fixed = true, ground_seconds = 0.55, grow = 0.45, delay = 0.05 }
M.RECIPES.fx_layer_possession_embers = { system = "burst", sprite = "spark_star", size = 60, life = 0.6, tint = { 0.5, 0.1, 1.0 }, delay = 0.1 }
M.RECIPES.fx_layer_quake_sparks = { system = "sparks", tint = { 1.0, 0.45, 0.1 }, delay = 0.06 }
M.RECIPES.fx_layer_green_motes = { system = "sparks", tint = { 0.2, 1.0, 0.4 }, delay = 0.08 }

-- Abilities whose every look is in the recipes above.
M.FX_ABILITIES = {
  "a01_tosho_gongen", "a01_heaven_net", "a01_thunder_blade", "a01_mikawa_valor", "a01_realm_momentum",
  "b20_rock_rush", "b20_ruin_wave", "b20_tenacity", "b20_red_charge",
  "b26_charging_wave", "b26_release_wave", "b26_blood_fury", "b26_kozuke_valor", "b26_unstoppable",
  "c22_swallow_nest", "c22_flying_swallow", "c22_swallow_flash", "c22_dream_mist", "c22_ganryu",
  "a12_insight_flash", "a12_insight_flash_t", "a12_mystic_pierce", "a12_mystic_field", "a12_awe", "a12_awe_t", "a12_mystic_possession", "a12_scheme", "a12_mystic_arrow",
  "a14_impact_spear", "a14_divine_strike", "a14_stacked_spear", "a14_spear_fury",
  "a26_volley", "a26_powder_kick", "a26_twin_guns", "a26_scorching_bomb",
  "b27_horsemanship", "b27_mighty_strike", "b27_fearless_charge", "b27_rending_blade",
  "c03_iron_shirt", "c03_quake_axe", "c03_thunder_wrath", "c03_iron_hammer",
  "c05_matsukaze", "c05_thunder_dance", "c05_thunder_spear", "c05_flash_thunder",
}

-- Low-poly models (Tripo, asset-ledger.md), imported as static world models; a recipe's `mesh` draws one in the pool.
M.MESH_ROOT = "/Game/Omoba/Structures/TripoStaticV1"
M.MESH_SOURCE = "omfue/SourceArt/Structures/TripoStaticV1"
-- A model built from a side view has its tip along +X and its side toward -Y, so a tip leads as it is.
-- Its origin is at its foot: height is where its foot is, 0 for what stands on the ground.
-- roll = 90 shows that side to whatever the cue faces (face_viewer); tilt: about its long axis, before the roll.
M.MESHES = {
  throw_axe = {}, rocket = {}, cannon_shell = {}, arcane_arrow = {}, thunder_spear = {}, spear_head = {}, swallow = { tilt = -90 },
  rock_chunk = {}, fire_bomb = {}, earth_crown = {}, shock_dome = { translucent = true },
  war_hammer = { roll = 90 }, fissure_blade = { roll = 90 }, iron_bell = { roll = 90, translucent = true }, fire_wings = { roll = 90 }, thunder_sword = { roll = 90 }, war_banner = { roll = 90 },
  tower_shield = { roll = -90 }, oni_mask = { roll = -90 }, fury_spear = { roll = 90 }, wind_spiral = {},
}
function M.material(prefix, image) return ("%s/Instances/%s_fx_%s.%s_fx_%s"):format(M.ROOT, prefix, image, prefix, image) end
function M.system(name) return ("%s/Systems/NS_fx_%s.NS_fx_%s"):format(M.ROOT, name, name) end

-- What every recipe answers to, so no row hides the hero (a hero stands about 140): a body effect is at most
-- 1.5 heroes, only an area skill's own mark (`area`, or a cue with a package radius) goes by its radius. A slash
-- is an arc shown to the camera, never a flat plank; a lasting effect is thin and around the body, not over it.
M.MAX_BODY = 210
M.MAX_GROUND = 420
M.MAX_AURA = 90
function M.fit(r)
  if r.mesh then
    local model = M.MESHES[r.mesh] or {}
    r.ground_roll, r.tilt = r.ground_roll or model.roll, r.tilt or model.tilt
  end
  -- Leaning toward the camera it is no plank, and it keeps the way the blow went.
  if r.ground == "crescent_slash" and not r.face_camera then
    if not r.system then r.system, r.sprite, r.size, r.life = "pop", "crescent_slash", r.ground_size, r.ground_seconds end
    r.ground, r.ground_size, r.ground_seconds, r.height, r.grow = nil, nil, nil, nil, nil
  end
  if r.sprite == "flame_sheet" and not r.tint then r.tint = { 1.0, 0.32, 0.05 } end
  if r.sprite == "fire_wings" and (r.system == "pop" or r.system == "burst") then r.sprite = "flame_sheet" end
  if r.sprite == "flame_sheet" and (r.system == "pop" or r.system == "burst") then r.system, r.size = "blaze", math.min(r.size or 110, 110); r.life = math.max(r.life or 0.7, 0.7) end
  if r.system == "stream" and r.loop and not r.face then
    r.system = "aura"
    -- bold: the status is the skill itself (a channel), so it may stand out more.
    r.size = math.min(r.size or M.MAX_AURA, r.bold and M.MAX_AURA * 1.6 or M.MAX_AURA)
    r.alpha = r.alpha or (r.bold and 0.9 or 0.55)
    if r.life then r.life = math.min(r.life, 0.6) end
  end
  if not r.area then
    if r.size then r.size = math.min(r.size, M.MAX_BODY) end
    if r.ground_size then r.ground_size = math.min(r.ground_size, M.MAX_GROUND) end
  end
  return r
end

-- A recipe with its look applied.
function M.resolve(id)
  local row = assert(M.RECIPES[id], "no recipe for " .. id)
  local out = {}
  if row.look then for k, v in pairs(assert(M.LOOKS[row.look], id .. ": look " .. row.look)) do out[k] = v end end
  for k, v in pairs(row) do out[k] = v end
  return M.fit(out)
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
  local meshes = {}
  for _, id in ipairs(sorted(M.MESHES)) do
    local name = "fx_" .. id
    meshes[#meshes + 1] = { id = "mesh_" .. id, translucent = M.MESHES[id].translucent, file = ("%s%s/%s/%s.fbm/%s_basecolor.PNG"):format(root and (root .. "/") or "", M.MESH_SOURCE, name, name, name) }
  end
  return { schema = "omoba.fx-family.v1", textures = textures, systems = systems, meshes = meshes }
end

function M.check()
  local problems = {}
  for _, id in ipairs(sorted(M.RECIPES)) do
    local r = M.resolve(id)
    if r.system and not M.SYSTEMS[r.system] then problems[#problems + 1] = id .. ": system " .. r.system end
    if r.sprite and not M.IMAGES[r.sprite] then problems[#problems + 1] = id .. ": image " .. r.sprite end
    if r.ground and not M.IMAGES[r.ground] then problems[#problems + 1] = id .. ": ground image " .. r.ground end
    if r.mesh and not M.MESHES[r.mesh] then problems[#problems + 1] = id .. ": model " .. r.mesh end
    if not r.system and not r.ground and not r.mesh then problems[#problems + 1] = id .. ": draws nothing" end
  end
  return problems
end

local function number(v) return (("%.3f"):format(v):gsub("0+$", ""):gsub("%.$", "")) end

-- One CueOverrides row. Radius and duration are not here: they arrive with the cue from the package.
-- Recipes say sizes in backend units, like the packages; the game's world is 100 cm to the unit.
M.WORLD_CM = 100
function M.ini_row(id)
  local r = M.resolve(id)
  local f = {}
  local function add(k, v) f[#f + 1] = k .. "=" .. v end
  if r.system then add("Effect", '"' .. M.system(r.system) .. '"') end
  if r.tint then add("EffectTint", ("(R=%s,G=%s,B=%s,A=%s)"):format(number(r.tint[1]), number(r.tint[2]), number(r.tint[3]), number(r.alpha or 1))) end
  if r.sprite then add("Sprite", '"' .. M.material("MI", r.sprite) .. '"') end
  if r.size then add("FxSizeCm", number(r.size * M.WORLD_CM)) end
  if r.life then add("FxSeconds", number(r.life)) end
  if r.loop then add("bFxLoop", "True") end
  if r.face then add("bFxFaceMotion", "True") end
  if r.mesh then
    add("Ground", ('"%s/Instances/MIG_mesh_%s.MIG_mesh_%s"'):format(M.ROOT, r.mesh, r.mesh))
    add("GroundMesh", ('"%s/%s/%s.%s"'):format(M.MESH_ROOT, "fx_" .. r.mesh, "fx_" .. r.mesh, "fx_" .. r.mesh))
  elseif r.ground then add("Ground", '"' .. M.material("MIG", r.ground) .. '"') end
  if r.spin then add("GroundSpin", number(r.spin)) end
  if r.tilt then add("GroundTilt", number(r.tilt)) end
  if r.scatter then add("GroundScatterCm", number(r.scatter * M.WORLD_CM)) end
  if r.scatter_size then add("GroundScatterSize", number(r.scatter_size)) end
  if r.also then add("CueAlso", '("' .. table.concat(r.also, '","') .. '")') end
  if r.spin_up then add("GroundSpinUp", number(r.spin_up)) end
  if r.face_blend then add("GroundFaceViewerBlend", number(r.face_blend)) end
  if r.from_hand then add("bFromFiringHand", "True") end
  if r.ground_fixed then add("bGroundFixedSize", "True") end
  if r.delay then add("CueDelaySeconds", number(r.delay)) end
  if r.repeat_count then add("CueRepeatCount", tostring(r.repeat_count)); add("CueRepeatSeconds", number(r.repeat_seconds)); add("CueRepeatYaw", number(r.repeat_yaw)) end
  if r.launch then add("GroundLaunchCm", number(r.launch * M.WORLD_CM)) end
  if r.lead then add("GroundLeadCm", number(r.lead * M.WORLD_CM)) end
  if r.linger then add("GroundLingerSeconds", number(r.linger)) end
  if r.mesh_opacity then add("GroundMeshOpacity", number(r.mesh_opacity)) end
  if r.bounce then add("GroundBounceCm", number(r.bounce * M.WORLD_CM)); add("GroundBounceSeconds", number(r.bounce_seconds)) end
  if r.pivot then add("GroundPivotCm", number(r.pivot * M.WORLD_CM)) end
  if r.face_viewer then add("bGroundFaceViewer", "True") end
  if r.ground_size then add("GroundSizeCm", number(r.ground_size * M.WORLD_CM)) end
  if r.ground_seconds then add("GroundSeconds", number(r.ground_seconds)) end
  if r.grow then add("GroundGrow", number(r.grow)) end
  if r.line then add("bGroundLine", "True") end
  if r.from_caster then add("bGroundFromCaster", "True") end
  if r.height then add("GroundHeightCm", number(r.height * M.WORLD_CM)) end
  if r.face_camera then add("GroundFaceCamera", number(r.face_camera)) end
  if r.ground_roll then add("GroundRoll", number(r.ground_roll)) end
  if r.fly then add("bGroundFly", "True") end
  if r.fixed_size then add("bFxFixedSize", "True") end
  if r.no_damage_flash then add("bFxNoDamageFlash", "True") end
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
  -- Their casts draw only their own cues: FxAbilities lines, in place of any earlier ones.
  after = after:gsub("[\r\n]+%+?FxAbilities=[^\r\n]*", "")
  local abilities = {}
  for _, id in ipairs(M.FX_ABILITIES) do abilities[#abilities + 1] = nl .. "+FxAbilities=" .. id end
  return before .. "(" .. table.concat(out, ",") .. ")" .. table.concat(abilities) .. after, nl
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
