-- The bot hint of a point skill aims where the cast hits (v3_bot.lua derive): the variant sweep
-- found these never cast, their hint not admitted by the executor (bot_auto_policies.lua).
--   * a routine names the entry's own radius `$<id>.<name>` (B09E_old 地裂's bursts);
--   * a channel aimed at a point (B31E_old 徐如林's arrow barrage) took self_area, which has no target;
--   * a splash around what a linear projectile hits is not the aim; the projectile's line is.
--   * a circle around several hit points is filtered by the radius around each (C04R_old 死亡之舞).
--   * a move with no cast range hits around where it lands (C19R_old 奇襲's slow): aimed there.
package.path='docs/character_pipeline/tools/?.lua;scripts/?.lua;'..package.path
local bot=require('v3_bot')
local pv=require('package_v3')
local function hint(hero,id)
  local doc=assert(pv.load_file('docs/character_pipeline/packages/'..hero..'/v3.lua'))
  for _,e in ipairs(doc.abilities) do if e.id==id then return bot.derive(e,'ability',doc) end end
  error('no '..id)
end
local h=hint('sanada_masayuki_old','b09_old_earth_rupture')
assert(h.kind=='enemy_point' and h.radius_key=='radius','地裂: the bursts\' own radius, got '..h.kind)
h=hint('takeda_shingen_old','b31_old_quiet_as_forest')
assert(h.kind=='enemy_line' and h.line_width=='collision_radius','徐如林: the arrows\' line, got '..h.kind)
h=hint('izumo_no_okuni_old','c04_old_dance_of_death')
assert(h.kind=='enemy_point' and h.radius_key=='radius','死亡之舞: the radius around each burst, got '..h.kind)
h=hint('matsuhime_old','c19_old_surprise_raid')
assert(h.kind=='global_enemy_point' and h.radius_key=='slow_radius','奇襲: a teleport anywhere, aimed where its slow lands, got '..h.kind)
h=hint('ise_hime','b23_soul_feast')
assert(h.kind=='self_area' and h.radius_key=='radius','魂食: a no-target channel keeps self_area, got '..h.kind)
print('v3_bot_point_aim_test: ok')
