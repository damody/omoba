-- Build-time, read-only coverage of the authoring hero catalog. Unreal retains
-- the bounded registry and image decoder as the runtime resource contract.
local M={}
-- Shared fallback for heroes without authored art (generated heroes start here);
-- replacing it with authored art is an Editor-only manifest/art change.
M.PLACEHOLDER_FILE='hero-placeholder-v1.png'
function M.placeholder_entries(heroes,manifest)
 local present,added={},{}
 for _,row in ipairs(manifest.entries)do if row.kind=='portrait'then present[row.id]=true end end
 for _,hero in ipairs(heroes)do
  if not present[hero.id]then
   local row={kind='portrait',id=hero.id,displayName=hero.display_name,file=M.PLACEHOLDER_FILE,
    columns=1,rows=1,tile=0,placeholder=true}
   manifest.entries[#manifest.entries+1]=row;added[#added+1]=hero.id;present[hero.id]=true
  end
 end
 return added
end
-- Placeholder rows are derived from the catalog, so one whose hero left the
-- catalog (a rolled-back authoring build) is dropped; authored art still fails.
function M.prune_placeholders(heroes,manifest)
 local known,kept,removed={}, {},{}
 for _,hero in ipairs(heroes)do known[hero.id]=true end
 for _,row in ipairs(manifest.entries)do
  if row.kind=='portrait'and row.placeholder==true and not known[row.id]then removed[#removed+1]=row.id
  else kept[#kept+1]=row end
 end
 manifest.entries=kept
 return removed
end
local function integer(value,minimum,maximum)
 return type(value)=='number'and value%1==0 and value>=minimum and value<=maximum
end
local function id(value)
 return type(value)=='string'and #value>0 and #value<=128 and value:match('^[a-z0-9_]+$')
end
function M.validate(heroes,manifest,inspect)
 assert(type(heroes)=='table'and #heroes>0 and #heroes<=512,'invalid authoring hero catalog')
 assert(type(manifest)=='table'and manifest.schemaVersion==1 and type(manifest.entries)=='table'
  and #manifest.entries>0 and #manifest.entries<=512,'invalid HUD manifest')
 assert(type(inspect)=='function','portrait resource inspector required')
 local expected,seen,tiles={}, {},{}
 for _,hero in ipairs(heroes)do
  assert(type(hero)=='table'and id(hero.id)and type(hero.display_name)=='string'and hero.display_name~=''and not expected[hero.id],'invalid or duplicate authoring hero')
  expected[hero.id]=hero.display_name
 end
 for _,row in ipairs(manifest.entries)do if row.kind=='portrait'then
  assert(id(row.id)and expected[row.id],'portrait refers to unknown hero: '..tostring(row.id))
  assert(not seen[row.id],'duplicate hero portrait: '..row.id);seen[row.id]=true
  assert(row.displayName==expected[row.id],'stale portrait display name: '..row.id)
  assert(type(row.file)=='string'and #row.file<=128 and row.file:match('^[a-z0-9_.-]+%.png$')
   and not row.file:find('..',1,true)and row.resource==nil,'invalid portrait file namespace: '..row.id)
  assert(integer(row.columns,1,16)and integer(row.rows,1,16)and integer(row.tile,0,row.columns*row.rows-1),'invalid portrait atlas: '..row.id)
  local tile=row.file..':'..row.columns..':'..row.rows..':'..row.tile
  -- Only the declared shared placeholder may be reused; authored art stays one tile per hero.
  if row.placeholder==true then
   assert(row.file==M.PLACEHOLDER_FILE and row.columns==1 and row.rows==1 and row.tile==0,'invalid placeholder portrait: '..row.id)
  else
   assert(row.placeholder==nil,'invalid placeholder flag: '..row.id)
   assert(not tiles[tile],'duplicate portrait tile: '..row.id);tiles[tile]=true
  end
  local image=inspect(row.file)
  assert(type(image)=='table'and integer(image.width,1,4096)and integer(image.height,1,4096),'missing or invalid portrait PNG: '..row.id)
  -- Normalized atlas UVs need not be integer-pixel divisible (1254 / 4 is valid).
  assert(image.width*row.rows==image.height*row.columns,'portrait source tile is not square: '..row.id)
 end end
 local count=0
 for hero in pairs(expected)do assert(seen[hero],'missing portrait for authoring hero: '..hero);count=count+1 end
 return count
end
function M.inspect_png(file)
 local lfs=require('lfs');local size=lfs.attributes(file,'size')
 assert(lfs.attributes(file,'mode')=='file'and size and size>=45 and size<=32*1024*1024,'portrait PNG missing or outside size limit: '..file)
 local stream=assert(io.open(file,'rb'));local header=stream:read(33)
 stream:seek('end',-12);local ending=stream:read(12);assert(stream:close())
 assert(header and #header==33 and header:sub(1,8)=='\137PNG\13\10\26\10'
  and string.unpack('>I4',header,9)==13 and header:sub(13,16)=='IHDR'
  and ending=='\0\0\0\0IEND\174\66\96\130','invalid portrait PNG structure: '..file)
 local width,height=string.unpack('>I4I4',header,17)
 return {width=width,height=height,structural_check_only=true}
end
-- Hero ability icons. Every ability a catalog hero equips must have exactly one
-- HUD ability row: legacy atlas tile, packaged per-ability texture (resource
-- under /Game/Omoba/UI/Abilities, source PNG in Abilities/Source), or the
-- declared shared placeholder. A hero ability is never silently blank.
M.ABILITY_PLACEHOLDER_FILE='ability-placeholder-v1.png'
M.ABILITY_RESOURCE_ROOT='/Game/Omoba/UI/Abilities'
M.ABILITY_SOURCE_DIR='Abilities/Source'
local function hero_abilities(heroes,abilities)
 local names,order,seen={},{},{}
 for _,row in ipairs(abilities)do names[row.id]=row.display_name end
 for _,hero in ipairs(heroes)do
  for _,ability in ipairs(hero.abilities or{})do
   assert(id(ability)and names[ability],'hero '..tostring(hero.id)..' equips unknown ability: '..tostring(ability))
   if not seen[ability]then seen[ability]=true;order[#order+1]=ability end
  end
 end
 return order,names
end
function M.ability_placeholder_entries(heroes,abilities,manifest)
 local order,names=hero_abilities(heroes,abilities);local present,added={}, {}
 for _,row in ipairs(manifest.entries)do if row.kind=='ability'then present[row.id]=true end end
 for _,ability in ipairs(order)do
  if not present[ability]then
   manifest.entries[#manifest.entries+1]={kind='ability',id=ability,displayName=names[ability],
    file=M.ABILITY_PLACEHOLDER_FILE,columns=1,rows=1,tile=0,placeholder=true}
   added[#added+1]=ability;present[ability]=true
  end
 end
 return added
end
-- Placeholder ability rows are derived from hero kits; drop one no hero equips.
function M.prune_ability_placeholders(heroes,abilities,manifest)
 local order=hero_abilities(heroes,abilities);local equipped,kept,removed={}, {},{}
 for _,ability in ipairs(order)do equipped[ability]=true end
 for _,row in ipairs(manifest.entries)do
  if row.kind=='ability'and row.placeholder==true and not equipped[row.id]then removed[#removed+1]=row.id
  else kept[#kept+1]=row end
 end
 manifest.entries=kept
 return removed
end
function M.validate_abilities(heroes,abilities,manifest,inspect)
 assert(type(heroes)=='table'and #heroes>0 and type(abilities)=='table','invalid authoring ability catalog')
 assert(type(manifest)=='table'and manifest.schemaVersion==1 and type(manifest.entries)=='table','invalid HUD manifest')
 assert(type(inspect)=='function','ability resource inspector required')
 local order,names=hero_abilities(heroes,abilities);local rows,placeholders={},{}
 for _,row in ipairs(manifest.entries)do if row.kind=='ability'then
  assert(id(row.id)and names[row.id],'ability icon refers to unknown ability: '..tostring(row.id))
  assert(not rows[row.id],'duplicate ability icon: '..row.id);rows[row.id]=row
 end end
 local packaged=0
 for _,ability in ipairs(order)do
  local row=assert(rows[ability],'missing HUD icon for hero ability: '..ability)
  assert(row.displayName==names[ability],'stale ability display name: '..ability)
  assert(type(row.file)=='string'and #row.file<=128 and row.file:match('^[a-z0-9_.-]+%.png$')and not row.file:find('..',1,true),'invalid ability file namespace: '..ability)
  assert(integer(row.columns,1,16)and integer(row.rows,1,16)and integer(row.tile,0,row.columns*row.rows-1),'invalid ability atlas: '..ability)
  local source=row.file
  if row.placeholder==true then
   assert(row.file==M.ABILITY_PLACEHOLDER_FILE and row.columns==1 and row.rows==1 and row.tile==0 and row.resource==nil,'invalid placeholder ability icon: '..ability)
   placeholders[#placeholders+1]=ability
  else
   assert(row.placeholder==nil,'invalid placeholder flag: '..ability)
   if row.resource~=nil then
    assert(row.resource==M.ABILITY_RESOURCE_ROOT..'/'..ability..'.'..ability and row.file==ability..'.png'
     and row.columns==1 and row.rows==1 and row.tile==0,'invalid packaged ability icon: '..ability)
    source=M.ABILITY_SOURCE_DIR..'/'..row.file;packaged=packaged+1
   end
  end
  local image=inspect(source)
  assert(type(image)=='table'and integer(image.width,1,4096)and integer(image.height,1,4096),'missing or invalid ability icon PNG: '..ability)
  assert(image.width*row.rows==image.height*row.columns,'ability icon tile is not square: '..ability)
 end
 return #order,placeholders,packaged
end
-- Tall (2:3) match loading-card art. Optional per hero: a hero without a
-- loading row is shown with its square portrait cropped to 2:3, and the build
-- warns. Rows never borrow another hero's file and never use the placeholder.
M.LOADING_KIND='loading'
function M.validate_loading(heroes,manifest,inspect)
 assert(type(heroes)=='table'and #heroes>0,'invalid authoring hero catalog')
 assert(type(manifest)=='table'and manifest.schemaVersion==1 and type(manifest.entries)=='table','invalid HUD manifest')
 assert(type(inspect)=='function','loading art inspector required')
 local names,order={}, {}
 for _,hero in ipairs(heroes)do names[hero.id]=hero.display_name;order[#order+1]=hero.id end
 local rows,files={}, {}
 for _,row in ipairs(manifest.entries)do if row.kind==M.LOADING_KIND then
  assert(id(row.id)and names[row.id],'loading art refers to unknown hero: '..tostring(row.id))
  assert(not rows[row.id],'duplicate hero loading art: '..row.id);rows[row.id]=row
  assert(row.displayName==names[row.id],'stale loading art display name: '..row.id)
  assert(type(row.file)=='string'and #row.file<=128 and row.file:match('^[a-z0-9_.-]+%.png$')
   and not row.file:find('..',1,true)and row.resource==nil and row.placeholder==nil,'invalid loading art file namespace: '..row.id)
  assert(row.file~=M.PLACEHOLDER_FILE and not files[row.file],'loading art must be the hero own file: '..row.id);files[row.file]=true
  assert(row.columns==1 and row.rows==1 and row.tile==0,'loading art is a single image: '..row.id)
  local image=inspect(row.file)
  assert(type(image)=='table'and integer(image.width,1,4096)and integer(image.height,1,4096),'missing or invalid loading art PNG: '..row.id)
  assert(image.width*3==image.height*2,'loading art is not 2:3: '..row.id)
 end end
 local covered,fallback=0,{}
 for _,hero in ipairs(order)do if rows[hero]then covered=covered+1 else fallback[#fallback+1]=hero end end
 return covered,fallback
end
function M.require_ready(root)
 local b=require('_bootstrap');local p,j=b.lib('path'),b.lib('json');local lfs=require('lfs')
 local ctx=require('content_builder').new(p.join(root,'scripts/lua_data'))
 local directory=p.join(root,'omfue/Content/Omoba/UI');local file=p.join(directory,'hud-assets-v1.json')
 local size=lfs.attributes(file,'size');assert(size and size>0 and size<=256*1024,'HUD manifest missing or too large')
 local manifest=j.read(file);local inspect=function(name)return M.inspect_png(p.join(directory,name))end
 local heroes=ctx.include('templates/heroes.lua')
 local _,placeholders=M.validate_abilities(heroes,ctx.include('templates/abilities.lua'),manifest,inspect)
 for _,ability in ipairs(placeholders)do print('[moba-ue] WARNING HUD ability icon is the shared placeholder (no authored art): '..ability)end
 local _,loading_fallback=M.validate_loading(heroes,manifest,inspect)
 for _,hero in ipairs(loading_fallback)do print('[moba-ue] WARNING hero has no loading-card art; the loading screen crops its square portrait: '..hero)end
 return M.validate(heroes,manifest,inspect)
end
return M
