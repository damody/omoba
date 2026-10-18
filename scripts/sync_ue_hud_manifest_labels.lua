-- Build-time presentation projection. Authoring Lua never runs inside Unreal.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j=b.lib('path'),b.lib('json')
local context=require('content_builder').new(p.join(b.root,'scripts/lua_data'))
local names={portrait={},ability={},summoner={},item={}}
for _,spec in ipairs({{kind='portrait',file='templates/heroes.lua'},{kind='ability',file='templates/abilities.lua'},{kind='summoner',file='templates/moba_summoners.lua'},{kind='item',file='templates/moba_items.lua',name_field='name'}}) do
 for _,entry in ipairs(context.include(spec.file)) do
  assert(not names[spec.kind][entry.id],'duplicate content name: '..entry.id)
  names[spec.kind][entry.id]=assert(entry[spec.name_field or 'display_name'],'missing source display name: '..entry.id)
 end
end
local file=p.join(b.root,'omfue/Content/Omoba/UI/hud-assets-v1.json')
local manifest=j.read(file);assert(manifest.schemaVersion==1 and type(manifest.entries)=='table')
local changed=0
for _,entry in ipairs(manifest.entries) do
 local group=names[entry.kind]
 if group then
  local name=assert(group[entry.id],'HUD asset refers to unknown content: '..entry.kind..':'..entry.id)
  if entry.displayName~=name then entry.displayName=name;changed=changed+1 end
 end
end
local portraits=require('ue_hud_portrait_coverage').validate(context.include('templates/heroes.lua'),manifest,
 function(name)return require('ue_hud_portrait_coverage').inspect_png(p.join(b.root,'omfue/Content/Omoba/UI',name))end)
if changed>0 then j.write(file,manifest,true) end
print('[moba-ue] HUD labels: '..#manifest.entries..' entries checked against authoring templates, '..changed..' updated')
print('[moba-ue] Hero portrait coverage: '..portraits..' authoring heroes; source PNG headers/tiles checked, full decoding remains a native check')
