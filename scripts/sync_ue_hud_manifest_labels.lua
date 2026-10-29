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
-- Original nobu art stands in for missing art (scripts/import_nobu_icons.lua); optional.
local nobu_index_file=p.join(b.root,'omfue/Content/Omoba/UI',coverage.NOBU_INDEX)
local nobu=coverage.nobu_files(p.is_file(nobu_index_file)and j.read(nobu_index_file)or nil)
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
local shop_items=context.include('templates/moba_items.lua')
local stale_items=coverage.prune_items(shop_items,manifest,nobu)
changed=changed+#stale_items
for _,item in ipairs(stale_items) do print('[moba-ue] HUD nobu item icon removed (no longer sold or art gone): '..item) end
local upgraded=coverage.upgrade_placeholders(manifest,nobu)
changed=changed+#upgraded
for _,key in ipairs(upgraded) do print('[moba-ue] HUD placeholder replaced by original nobu art: '..key) end
local placeholders=require('ue_hud_portrait_coverage').placeholder_entries(context.include('templates/heroes.lua'),manifest,nobu)
changed=changed+#placeholders
for _,hero in ipairs(placeholders) do print('[moba-ue] HUD portrait added for hero without authored art ('..(nobu.portrait[hero]and'original nobu art'or'shared placeholder')..'): '..hero) end
-- A hero ability without authored art gets the shared placeholder icon and a build warning, never a blank slot.
local item_icons=coverage.item_entries(shop_items,manifest,nobu)
changed=changed+#item_icons
for _,item in ipairs(item_icons) do print('[moba-ue] HUD item icon added from original nobu art: '..item) end
local ability_placeholders=coverage.ability_placeholder_entries(heroes,abilities,manifest,nobu)
changed=changed+#ability_placeholders
for _,ability in ipairs(ability_placeholders) do print('[moba-ue] HUD ability icon added for hero ability without authored icon ('..(nobu.ability[ability]and'original nobu art'or'shared placeholder')..'): '..ability) end
local inspect_ui=function(name)return coverage.inspect_png(p.join(b.root,'omfue/Content/Omoba/UI',name))end
local ability_count,ability_pending,ability_packaged=coverage.validate_abilities(heroes,abilities,manifest,inspect_ui)
for _,ability in ipairs(ability_pending) do print('[moba-ue] WARNING HUD ability icon is the shared placeholder (no authored art): '..ability) end
local portraits=require('ue_hud_portrait_coverage').validate(context.include('templates/heroes.lua'),manifest,
 function(name)return require('ue_hud_portrait_coverage').inspect_png(p.join(b.root,'omfue/Content/Omoba/UI',name))end)
-- A summoner spell without HUD art is shown as an equipped marker (HUD and loading cards); warn.
local summoner_art,summoners_without_art={}, {}
for _,entry in ipairs(manifest.entries) do if entry.kind=='summoner' then summoner_art[entry.id]=true end end
for id in pairs(names.summoner) do if not summoner_art[id] then summoners_without_art[#summoners_without_art+1]=id end end
table.sort(summoners_without_art)
for _,id in ipairs(summoners_without_art) do print('[moba-ue] WARNING summoner spell has no HUD icon; D/F slots show an equipped marker: '..id) end
-- A hero without loading-card art is not an error: the loading screen crops its portrait, with a warning.
local loading_count,loading_fallback=coverage.validate_loading(heroes,manifest,inspect_ui)
for _,hero in ipairs(loading_fallback) do print('[moba-ue] WARNING hero has no loading-card art; the loading screen crops its square portrait: '..hero) end
if changed>0 then j.write(file,manifest,true) end
print('[moba-ue] HUD labels: '..#manifest.entries..' entries checked against authoring templates, '..changed..' updated')
print('[moba-ue] Hero portrait coverage: '..portraits..' authoring heroes; source PNG headers/tiles checked, full decoding remains a native check')
print('[moba-ue] Hero loading-card art coverage: '..loading_count..' authored, '..#loading_fallback..' portrait fallback')
print('[moba-ue] Hero ability icon coverage:'..ability_count..' equipped abilities, '..ability_packaged..' packaged, '..#ability_pending..' placeholder')
