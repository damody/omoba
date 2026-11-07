-- M6: cue effects follow the cue kind and the reviewed reference prompt, never a hero; each shape
-- reshapes the one template so the families read apart.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
CUE_VFX_AS_MODULE = true
local M = require('cue_vfx')

local checks = 0
local function ok(cond, msg) checks = checks + 1; if not cond then error(msg, 2) end end

local function shape(kind, prompt) local s, _, _, keep = M.style(kind, prompt); return s, keep end
local s, keep = shape('projectile', 'a bolt')
ok(s == 'trail' and keep, 'projectiles streak and keep the ribbon that shows their path')
s, keep = shape('area', '')
ok(s == 'ring' and keep, 'areas are rings and keep the ground ring')
s, keep = shape('aura', '')
ok(s == 'aura' and keep, 'auras drift')
s = shape('effect', 'a red shockwave ring cracking outward')
ok(s == 'ring', 'ground shockwaves are rings')
s = shape('effect', 'a tight orange fireball, a puff of grey smoke and sparks')
ok(s == 'burst', 'whole words only: "sparks" is not "ring"')
s = shape('effect', 'a wide cone of fire breath')
ok(s == 'slash', 'cones and breaths fan out')
s = shape('effect', 'a golden spell-shield aura rising up as if a warrior grows larger')
ok(s == 'buff', 'body effects rise as a column')
local KNOWN = { OmnidirectionalBurst = M.TEMPLATE, SimpleSpriteBurst = M.TEMPLATE, UpwardMeshBurst = M.TEMPLATE,
  DirectionalBurst = M.DIRECTIONAL, LocationBasedRibbon = M.DIRECTIONAL }
for name, recipe in pairs(M.SHAPES) do
  ok(recipe.template == M.TEMPLATE or recipe.template == M.DIRECTIONAL, name .. ': engine template')
  for emitter, e in pairs(recipe.emitters) do
    ok(KNOWN[emitter] == recipe.template, name .. ': emitter ' .. emitter .. ' belongs to its template')
    ok(e.velocity == nil or #e.velocity == 3, name .. ': velocity is per axis')
  end
end
ok(M.asset('x_y') == '/Game/Omoba/VFX/Cues/NS_x_y.NS_x_y', 'asset path')
local lines = M.ini_lines({ cues = { { id = 'a', keep = true }, { id = 'b' } } })
ok(lines[1] == M.INI_SECTION and lines[2]:find('bKeepPlaceholder=True', 1, true) and select(2, lines[2]:gsub('bKeepPlaceholder', '')) == 1, 'ini map, keep only where asked')

local placed = M.ini_lines({ cues = { { id = 't', shape = 'trail', keep = true }, { id = 'b', shape = 'burst' } } })[2]
ok(placed:find('("t",(Effect="/Game/Omoba/VFX/Cues/NS_t.NS_t",bKeepPlaceholder=True,PlaceholderShape=3,PlaceholderAnchor=3,PlaceholderSizeCm=30,PlaceholderSeconds=0.6))', 1, true), 'a trail draws a ribbon from caster to target')
ok(placed:find('("b",(Effect="/Game/Omoba/VFX/Cues/NS_b.NS_b"))', 1, true), 'a burst keeps only its effect')
for _, name in ipairs({ 'ring', 'aura', 'trail', 'slash' }) do ok(M.PLACEHOLDERS[name], name .. ' has a placeholder form') end
local c = M.path_color('particles/units/heroes/hero_techies/techies_base_attack_explosion_b.vpcf')
ok(c[1] == 1.0 and c[2] < 0.6, 'an explosion particle is fiery')
ok(M.path_color('particles/units/heroes/hero_zuus/zuus_lightning_bolt.vpcf')[3] == 1.0, 'lightning is blue-white')
ok(#M.path_color('particles/x/unknown.vpcf') == 3, 'unknown words still get a colour')
ok(M.style('effect', M.path_words('particles/hero_x/x_aoe_explosion.vpcf')) == 'burst', 'path words pick the shape')
local shared = M.ini_lines({ cues = { { id = 'r', shape = 'ring', keep = true, scale = 1.3 } } }, { r = { 1, 0.5, 0.25 } })[2]
ok(shared:find('("r",(Effect="/Game/Omoba/VFX/CueShapes/NS_shape_ring.NS_shape_ring",bKeepPlaceholder=True,PlaceholderShape=1,PlaceholderAnchor=2,PlaceholderSeconds=0.9,EffectTint=(R=1.000,G=0.500,B=0.250,A=1.0),EffectScale=1.3))', 1, true), 'a cue is its shape system plus its colour and scale')
ok(shared:find('("ground_damage_zone",(Kind=Area,Color=(R=1.000,G=0.450,B=0.150,A=1.0),PlaceholderSeconds=1.6))', 1, true)
  and shared:find('("runtime_tether",(Kind=Link,', 1, true), 'the runtime-fact cues ride along as placeholders')
local own = M.ini_lines({ cues = { { id = 'runtime_tether', shape = 'trail', keep = true } } })[2]
ok(select(2, own:gsub('"runtime_tether"', '')) == 1, 'a built cue of the same id wins')
local shapes = M.shapes_spec({ cues = { { id = 'r', color = { 1, 0, 0 } } } })
ok(#shapes.shapes == 6 and shapes.tints[1].id == 'r', 'six shared systems, one tint row per cue')
print('cue_vfx_test: ' .. checks .. ' checks passed')
