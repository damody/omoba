local build=assert(loadfile('scripts/lua_data/templates/world_units.lua'))()
local profiles={training_vanguard={mesh_asset='cc0'},date_masamune={mesh_asset='cc0'},training_luminary={mesh_asset='cc0'}}
local first={jungle_camps={{id='north_guardian'},{id='south_guardian'},{id='first_camp',members={{id='first_small'}}}},
 minion_wave={{id='first_melee'}}}
local second={jungle_camps={{id='second_camp',members={{id='second_small'},{id='first_small'}}},{id='first_camp'},{id='second_epic'}},
 minion_wave={{id='second_caster'},{id='first_melee'}}}
first.base_structures={super_minion={id='first_super'}}
second.base_structures={super_minion={id='second_super'}}
local maps={first}
local context={include=function(name)
 if name=='templates/moba_native_visuals.lua'then return profiles end
 if name=='templates/moba_tripo_hero_visuals.lua'then return {profile=function(id)return {tripo=id,mesh_asset=id}end} end
 assert(name=='templates/moba_maps.lua');return maps
end}
local function indexed(rows)
 local by_id,ids={},{}
 for _,row in ipairs(rows)do
  assert(not by_id[row.id],'duplicate identity: '..row.id);by_id[row.id]=row
  local key=row.kind..':'..row.catalog_id
  assert(not ids[key],'duplicate numeric identity');ids[key]=true
 end
 return by_id
end
local before=indexed(build(context))
assert(before.first_camp and before.first_melee and not before.second_camp)
maps={first,second}
local after=indexed(build(context))
assert(after.second_camp and after.second_caster,'all maps must contribute identities')
for id,row in pairs(before)do assert(after[id].catalog_id==row.catalog_id,'appending a map changed existing identity: '..id)end
assert(after.north_guardian.catalog_id==32769 and after.south_guardian.catalog_id==32770)
assert(after.first_melee.catalog_id~=after.second_caster.catalog_id)
assert(after.first_small.catalog_id==40000 and after.second_small.catalog_id==40001)
assert(after.first_super.catalog_id==60000 and after.second_super.catalog_id==60001)
assert(after.moba_inhibitor.kind=='Tower' and after.moba_inhibitor.catalog_id==32770)
assert(after.moba_nexus_tower.kind=='Tower' and after.moba_nexus_tower.catalog_id==32771)
-- Wards wear a modelled lantern with team colour variants, never the default cube.
local ward=after.moba_ward
assert(ward.kind=='Tower' and ward.catalog_id==32772 and ward.ue.native_visual.static_mesh_asset:find('ward_lantern',1,true))
assert(ward.ue.native_visual.ally_material_sections and ward.ue.native_visual.enemy_material_sections)
-- A unit without a model fails the build.
profiles.training_vanguard={}
assert(not pcall(build,context),'a world unit without a model must fail the build')
profiles.training_vanguard={mesh_asset='cc0'}
-- Neutral camps wear Tripo bodies without team variants: tengu guardians, oni epic, kappa members.
local function body(id)return after[id].ue.native_visual end
assert(body('north_guardian').tripo=='jungle_tengu' and body('first_camp').tripo=='jungle_tengu')
assert(body('second_epic').tripo=='jungle_oni' and body('second_epic').height_world_units==170)
assert(body('first_small').tripo=='jungle_kappa' and body('first_small').height_world_units==50)
local original=first.base_structures
first.base_structures={super_minion={id='first_melee'}}
assert(not pcall(build,context),'super minion identity collision must fail closed')
first.base_structures=original
maps={first,{jungle_camps={{id='first_melee'}}}}
assert(not pcall(build,context),'ambiguous camp/minion identity must fail closed')
print('multi-map world unit catalog: union, deduplication, stable existing IDs and role collision passed')
