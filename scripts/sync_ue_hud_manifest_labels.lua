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
-- Loading-card art is per hero and carries the same authored hero name.
names.loading=names.portrait
local file=p.join(b.root,'omfue/Content/Omoba/UI/hud-assets-v1.json')
local manifest=j.read(file);assert(manifest.schemaVersion==1 and type(manifest.entries)=='table')
local coverage=require('ue_hud_portrait_coverage')
local stale=coverage.prune_placeholders(context.include('templates/heroes.lua'),manifest)
local changed=#stale
for _,hero in ipairs(stale) do print('[moba-ue] HUD portrait placeholder removed for hero no longer in the catalog: '..hero) end
local heroes,abilities=context.include('templates/heroes.lua'),context.include('templates/abilities.lua')
local stale_abilities=coverage.prune_ability_placeholders(heroes,abilities,manifest)
changed=changed+#stale_abilities
for _,ability in ipairs(stale_abilities) do print('[moba-ue] HUD ability placeholder removed for ability no hero equips: '..ability) end
for _,entry in ipairs(manifest.entries) do
 local group=names[entry.kind]
 if group then
  local name=assert(group[entry.id],'HUD asset refers to unknown content: '..entry.kind..':'..entry.id)
  if entry.displayName~=name then entry.displayName=name;changed=changed+1 end
 end
end
-- A new authoring hero gets the shared placeholder portrait instead of failing the build.
local placeholders=require('ue_hud_portrait_coverage').placeholder_entries(context.include('templates/heroes.lua'),manifest)
changed=changed+#placeholders
for _,hero in ipairs(placeholders) do print('[moba-ue] HUD portrait placeholder added for hero without authored art: '..hero) end
-- A hero ability without authored art gets the shared placeholder icon and a build warning, never a blank slot.
local ability_placeholders=coverage.ability_placeholder_entries(heroes,abilities,manifest)
changed=changed+#ability_placeholders
for _,ability in ipairs(ability_placeholders) do print('[moba-ue] HUD ability placeholder added for hero ability without authored icon: '..ability) end
local inspect_ui=function(name)return coverage.inspect_png(p.join(b.root,'omfue/Content/Omoba/UI',name))end
local ability_count,ability_pending,ability_packaged=coverage.validate_abilities(heroes,abilities,manifest,inspect_ui)
for _,ability in ipairs(ability_pending) do print('[moba-ue] WARNING HUD ability icon is the shared placeholder (no authored art): '..ability) end
local portraits=require('ue_hud_portrait_coverage').validate(context.include('templates/heroes.lua'),manifest,
 function(name)return require('ue_hud_portrait_coverage').inspect_png(p.join(b.root,'omfue/Content/Omoba/UI',name))end)
-- A hero without loading-card art is not an error: the loading screen crops its portrait, with a warning.
local loading_count,loading_fallback=coverage.validate_loading(heroes,manifest,inspect_ui)
for _,hero in ipairs(loading_fallback) do print('[moba-ue] WARNING hero has no loading-card art; the loading screen crops its square portrait: '..hero) end
if changed>0 then j.write(file,manifest,true) end
print('[moba-ue] HUD labels: '..#manifest.entries..' entries checked against authoring templates, '..changed..' updated')
print('[moba-ue] Hero portrait coverage: '..portraits..' authoring heroes; source PNG headers/tiles checked, full decoding remains a native check')
print('[moba-ue] Hero loading-card art coverage: '..loading_count..' authored, '..#loading_fallback..' portrait fallback')
print('[moba-ue] Hero ability icon coverage:'..ability_count..' equipped abilities, '..ability_packaged..' packaged, '..#ability_pending..' placeholder')
