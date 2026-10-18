-- Build-time, read-only coverage of the authoring hero catalog. Unreal retains
-- the bounded registry and image decoder as the runtime resource contract.
local M={}
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
  assert(not tiles[tile],'duplicate portrait tile: '..row.id);tiles[tile]=true
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
function M.require_ready(root)
 local b=require('_bootstrap');local p,j=b.lib('path'),b.lib('json');local lfs=require('lfs')
 local ctx=require('content_builder').new(p.join(root,'scripts/lua_data'))
 local directory=p.join(root,'omfue/Content/Omoba/UI');local file=p.join(directory,'hud-assets-v1.json')
 local size=lfs.attributes(file,'size');assert(size and size>0 and size<=256*1024,'HUD manifest missing or too large')
 return M.validate(ctx.include('templates/heroes.lua'),j.read(file),function(name)return M.inspect_png(p.join(directory,name))end)
end
return M
