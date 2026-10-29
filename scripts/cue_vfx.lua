-- Cue effects from the reviewed reference images (M6, work line C): every cue with a generated reference
-- (scripts/generated_art.lua, kind "cue") gets a Niagara system copied from an engine template that suits
-- its cue kind, tinted with the reference's glowing colour (the commandlet samples the image) and scaled
-- for the kind. Nothing is per hero; the template follows the cue catalog kind and the reference prompt.
--
--   tools\lua\lua.exe scripts\cue_vfx.lua --spec OUT.json [--only id,id]   write the commandlet spec
--   tools\lua\lua.exe scripts\cue_vfx.lua --ini                            print the CueOverrides lines
--
-- Then: UnrealEditor-Cmd <project> -run=OmCueVfx -spec=OUT.json  (systems under /Game/Omoba/VFX/Cues).
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local json = b.lib("json")

local M = {}
M.CUES = "docs/character_pipeline/presentation/cue-catalog.lua"
M.ROOT = "/Game/Omoba/VFX/Cues"
local T = "/Niagara/DefaultAssets/Templates/Systems/"
M.TEMPLATE = T .. "SimpleExplosion.SimpleExplosion"
M.DIRECTIONAL = T .. "DirectionalBurst.DirectionalBurst"

-- A shape is an engine template plus per-emitter changes (third review, 2026-10-30: rings must read as
-- rings and streaks as streaks, not as one cloud of sparks). SimpleExplosion: soft puffs
-- SimpleSpriteBurst, sparks OmnidirectionalBurst, rising meshes UpwardMeshBurst. DirectionalBurst:
-- sparks DirectionalBurst shot in a cone, each followed by a ribbon (LocationBasedRibbon).
--   set:   exact module inputs ("<Module>.<Input>" = number or {x, y[, z]})
--   local: the emitter simulates in the component's space, which the game turns toward the cast
--   velocity / gravity / count / size / time: factors on the template's values
local SPARKS, PUFFS, UP = "OmnidirectionalBurst", "SimpleSpriteBurst", "UpwardMeshBurst"
local CONE, RIBBON = "DirectionalBurst", "LocationBasedRibbon"
-- A sphere flattened to the ground with every point on its surface: a ring of points, rim-heavy.
local FLAT_RING = { ["ShapeLocation.Non Uniform Scale"] = { 1, 1, 0.02 }, ["ShapeLocation.Sphere Surface Distribution"] = 1 }
local function with(t, extra) local o = {} for k, v in pairs(t) do o[k] = v end for k, v in pairs(extra) do o[k] = v end return o end
M.SHAPES = {
  -- ground ring / shockwave: points on a flat ring racing outward along the ground
  ring = { template = M.TEMPLATE, emitters = {
    [SPARKS] = { set = with(FLAT_RING, { ["ShapeLocation.Sphere Radius"] = 130 }), velocity = { 2.2, 2.2, 0.02 }, gravity = 0, count = 6, size = 2.2 },
    [PUFFS] = { enabled = false }, [UP] = { enabled = false } } },
  -- lasting aura: a wide flat ring of motes hugging the ground and rising slowly
  aura = { template = M.TEMPLATE, emitters = {
    [SPARKS] = { set = with(FLAT_RING, { ["ShapeLocation.Sphere Radius"] = 140 }), velocity = { 0.05, 0.05, 0.6 }, gravity = -0.1, count = 3, time = 3, size = 1.6 },
    [PUFFS] = { enabled = false }, [UP] = { enabled = false } } },
  -- projectile / tether: a tight cone along the cast direction, each spark trailing a ribbon
  trail = { template = M.DIRECTIONAL, emitters = {
    [CONE] = { local_space = true, set = { ["AddVelocityInCone.Cone Angle"] = 6, ["AddVelocityInCone.Cone Axis"] = { 1, 0, 0 } }, gravity = 0, count = 6, size = 4, time = 2.5 },
    [RIBBON] = { local_space = true, set = { ["InitializeParticle.Ribbon Width"] = 40 }, time = 2.5 } } },
  -- slash / cleave / cone / breath: a wide flat fan along the cast direction, short-lived ribbons
  slash = { template = M.DIRECTIONAL, emitters = {
    [CONE] = { local_space = true, set = { ["AddVelocityInCone.Cone Angle"] = 55, ["AddVelocityInCone.Cone Axis"] = { 1, 0, 0 } }, gravity = 0, count = 8, size = 4, time = 1.5 },
    [RIBBON] = { local_space = true, set = { ["InitializeParticle.Ribbon Width"] = 50 }, time = 1.5 } } },
  -- buff on the body: a column of light rising from the unit
  buff = { template = M.TEMPLATE, emitters = {
    [SPARKS] = { velocity = { 0.25, 0.25, 2.4 }, gravity = -0.2, count = 2 },
    [PUFFS] = { size = 0.7 }, [UP] = { } } },
  -- impact / explosion: the template as it is
  burst = { template = M.TEMPLATE, emitters = {} },
}
-- The generic placeholder each shape draws under its effect (OmCueSettings: shape 1 ring, 2 disc,
-- 3 ribbon; anchor 1 caster, 2 target, 3 caster to target). It carries what particles alone do not:
-- the ring's footprint on the ground, the aura around the unit, and the direction of a trail or slash.
M.PLACEHOLDERS = {
  ring = { shape = 1, anchor = 2, seconds = 0.9 },
  aura = { shape = 1, anchor = 1, size = 120, seconds = 1.2 },
  trail = { shape = 3, anchor = 3, size = 30, seconds = 0.6 },
  slash = { shape = 3, anchor = 3, size = 140, seconds = 0.4 },
}
-- The commandlet's emitter fields use 'local'; Lua keeps local_space to avoid the keyword.
for _, shape in pairs(M.SHAPES) do
  for _, e in pairs(shape.emitters) do if e.local_space ~= nil then e["local"] = e.local_space; e.local_space = nil end end
end

local function has(prompt, words)
  for _, w in ipairs(words) do if prompt:find("%f[%a]" .. w .. "%f[%A]") then return true end end
  return false
end

-- kind and the reviewed reference prompt -> shape, scale, time, keep (also draw the kind's placeholder
-- ring or ribbon under the effect, which keeps its footprint and direction).
function M.style(kind, prompt)
  prompt = (prompt or ""):lower()
  if kind == "projectile" or kind == "link" then return "trail", 0.8, 0.8, true end
  if kind == "impact" then return "burst", 0.7, 0.6, false end
  -- (shapes below: ring, aura, trail and slash always keep the placeholder that draws their form)
  if kind == "area" then return "ring", 1.5, 1.2, true end
  if kind == "aura" or kind == "channel" or kind == "summon" then return "aura", 1.0, 1.4, true end
  if has(prompt, { "slash", "cleave", "crescent", "cone", "breath", "fan" }) then return "slash", 1.1, 1.0, true end
  if has(prompt, { "ring", "shockwave", "circle", "ground", "crack", "cracks", "cracking", "pulse" }) then return "ring", 1.3, 1.1, true end
  if has(prompt, { "projectile", "trail", "beam", "bolt", "forward", "line", "path" }) then return "trail", 1.0, 1.0, true end
  if has(prompt, { "aura", "rising", "rises", "grow", "grows", "shield", "barrier", "pillar", "swirl", "swirling", "vortex", "tornado", "whirlwind", "heal", "healing" }) then return "buff", 1.1, 1.2, false end
  return "burst", 1.1, 1.0, false
end

function M.asset(id) return M.ROOT .. "/NS_" .. id .. ".NS_" .. id end

-- Cues mapped to an original particle that is not imported (M6): the particle's path names what it is
-- (techies_base_attack_explosion, zuus_lightning_bolt...). Its words give the colour; the words and the
-- cue kind give the shape, through the same rules as a reviewed reference prompt.
M.PATH_COLORS = {
  { { "fire", "flame", "burn", "ember", "lava", "explosion", "explode", "blast", "techies", "lina", "ogre", "phoenix" }, { 1.0, 0.45, 0.1 } },
  { { "lightning", "thunder", "zuus", "storm", "static", "arc", "electric", "shock" }, { 0.55, 0.75, 1.0 } },
  { { "ice", "frost", "cold", "crystal", "snow", "freeze", "tusk", "winter" }, { 0.45, 0.9, 1.0 } },
  { { "poison", "venom", "toxic", "viper", "venomancer", "acid", "plague" }, { 0.45, 1.0, 0.25 } },
  { { "blood", "bloodseeker", "rage", "berserk", "crimson", "axe" }, { 1.0, 0.15, 0.15 } },
  { { "heal", "holy", "light", "omniknight", "divine", "bless", "keeper", "sun", "gold" }, { 1.0, 0.85, 0.35 } },
  { { "shadow", "dark", "void", "spectre", "death", "necro", "curse", "bane", "ghost", "soul" }, { 0.6, 0.3, 1.0 } },
  { { "water", "wave", "tide", "kunkka", "torrent", "morph" }, { 0.25, 0.55, 1.0 } },
  { { "earth", "rock", "stone", "sand", "dust", "tremor", "earthshaker", "elder_titan" }, { 0.85, 0.65, 0.35 } },
  { { "wind", "tornado", "cyclone", "invoker", "leaf", "nature", "treant" }, { 0.6, 1.0, 0.6 } },
}
function M.path_words(path)
  local words = {}
  for w in (path or ""):lower():gmatch("[a-z]+") do words[#words + 1] = w end
  return table.concat(words, " ")
end
function M.path_color(path)
  local words = " " .. M.path_words(path) .. " "
  for _, rule in ipairs(M.PATH_COLORS) do
    for _, w in ipairs(rule[1]) do if words:find(" " .. w .. " ", 1, true) then return rule[2] end end
  end
  return { 1.0, 0.8, 0.5 }
end

-- batch number -> its heroes' source ids (m6-waves.md), as m6_presentation_coverage reads them
function M.batch_docs(root, batch)
  M6_COVERAGE_AS_MODULE = true
  local cov = require("m6_presentation_coverage")
  local f = assert(io.open(root .. "/" .. cov.WAVES, "rb")); local text = f:read("a"); f:close()
  local docs = {}
  for _, id in ipairs(cov.batch_heroes(text, batch)) do docs[id] = true end
  return docs
end

function M.build(root, only, batches)
  GENERATED_ART_AS_MODULE = true
  local ga = require("generated_art")
  local refs = ga.rows(ga.read_index(root)).cue
  local catalog = dofile(root .. "/" .. M.CUES).cues
  local ids = {}
  for id in pairs(refs) do if not only or only[id] then ids[#ids + 1] = id end end
  table.sort(ids)
  local cues = {}
  for _, id in ipairs(ids) do
    local kind = catalog[id] and catalog[id].kind or "effect"
    local style, scale, time, keep = M.style(kind, refs[id].prompt)
    cues[#cues + 1] = { id = id, kind = kind, shape = style, template = M.SHAPES[style].template, emitters = M.SHAPES[style].emitters, scale = scale, time = time, keep = keep,
      reference = (root .. "/" .. ga.CUE_DIR .. "/" .. refs[id].file):gsub("\\", "/") }
  end
  -- original-asset cues of the asked batches (a reviewed reference, above, always wins)
  if batches then
    local docs = {}
    for _, n in ipairs(batches) do for id in pairs(M.batch_docs(root, n)) do docs[id] = true end end
    local more = {}
    for id, cue in pairs(catalog) do
      local used = false
      for _, use in ipairs(cue.uses or {}) do if docs[use.doc] then used = true end end
      local particle
      for _, a in ipairs(cue.assets or {}) do if a.kind == "particle" then particle = particle or a.path end end
      if used and particle and not refs[id] and cue.kind ~= "sound" and cue.kind ~= "gesture" and (not only or only[id]) then
        more[#more + 1] = { id = id, kind = cue.kind, path = particle }
      end
    end
    table.sort(more, function(x, y) return x.id < y.id end)
    for _, m in ipairs(more) do
      local style, scale, time, keep = M.style(m.kind, M.path_words(m.path))
      cues[#cues + 1] = { id = m.id, kind = m.kind, shape = style, template = M.SHAPES[style].template, emitters = M.SHAPES[style].emitters,
        scale = scale, time = time, keep = keep, color = M.path_color(m.path), original = m.path }
    end
  end
  return { schema = "omoba.cue-vfx-spec.v1", cues = cues }
end

-- The CueOverrides map of every built cue (Config/DefaultGame.ini, [/Script/OmRuntime.OmCueSettings]),
-- one line as the config system writes a TMap.
M.INI_SECTION = "[/Script/OmRuntime.OmCueSettings]"
function M.ini_lines(spec)
  local rows = {}
  for _, c in ipairs(spec.cues) do
    local ph, extra = M.PLACEHOLDERS[c.shape], ""
    if c.keep then extra = ",bKeepPlaceholder=True" end
    if c.keep and ph then
      extra = extra .. string.format(",PlaceholderShape=%d,PlaceholderAnchor=%d", ph.shape, ph.anchor)
      if ph.size then extra = extra .. string.format(",PlaceholderSizeCm=%g", ph.size) end
      if ph.seconds then extra = extra .. string.format(",PlaceholderSeconds=%g", ph.seconds) end
    end
    rows[#rows + 1] = string.format('("%s",(Effect="%s"%s))', c.id, M.asset(c.id), extra)
  end
  return { M.INI_SECTION, "CueOverrides=(" .. table.concat(rows, ",") .. ")" }
end

function M.main(args)
  local out, only, ini, batches
  local i = 1
  while i <= #args do
    if args[i] == "--spec" then out = args[i + 1]; i = i + 1
    elseif args[i] == "--only" then only = {}; for id in args[i + 1]:gmatch("[^,]+") do only[id] = true end; i = i + 1
    elseif args[i] == "--ini" then ini = true
    elseif args[i] == "--batches" then batches = {}; for n in args[i + 1]:gmatch("%d+") do batches[#batches + 1] = tonumber(n) end; i = i + 1
    else error("unknown argument " .. args[i]) end
    i = i + 1
  end
  local spec = M.build(b.root, only, batches)
  if ini then for _, l in ipairs(M.ini_lines(spec)) do print(l) end end
  if out then
    local f = assert(io.open(out, "wb")); f:write(json.encode(spec)); f:close()
    print("wrote " .. out .. " (" .. #spec.cues .. " cues)")
  end
  return 0
end

if not CUE_VFX_AS_MODULE then os.exit(M.main(arg)) end
return M
