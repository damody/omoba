local build=assert(loadfile('scripts/lua_data/templates/world_units.lua'))()
local profiles={training_vanguard={},date_masamune={},training_luminary={}}
local first={jungle_camps={{id='north_guardian'},{id='south_guardian'},{id='first_camp',members={{id='first_small'}}}},
 minion_wave={{id='first_melee'}}}
local second={jungle_camps={{id='second_camp',members={{id='second_small'},{id='first_small'}}},{id='first_camp'}},
 minion_wave={{id='second_caster'},{id='first_melee'}}}
first.base_structures={super_minion={id='first_super'}}
second.base_structures={super_minion={id='second_super'}}
local maps={first}
local context={include=function(name)
 if name=='templates/moba_native_visuals.lua'then return profiles end
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
local original=first.base_structures
first.base_structures={super_minion={id='first_melee'}}
assert(not pcall(build,context),'super minion identity collision must fail closed')
first.base_structures=original
maps={first,{jungle_camps={{id='first_melee'}}}}
assert(not pcall(build,context),'ambiguous camp/minion identity must fail closed')
print('multi-map world unit catalog: union, deduplication, stable existing IDs and role collision passed')
