package.path='scripts/?.lua;'..package.path
local b=require('_bootstrap');local p,j=b.lib('path'),b.lib('json');local M=require('ue_hud_portrait_coverage')
local count=0
local function check(value,why)assert(value,why);count=count+1 end
local function fresh()
 local heroes,entries={},{}
 for _,name in ipairs({'alpha','beta','gamma'})do
  heroes[#heroes+1]={id='hero_'..name,display_name='Hero '..name}
  entries[#entries+1]={kind='portrait',id='hero_'..name,displayName='Hero '..name,file=name..'.png',columns=1,rows=1,tile=0}
 end
 return heroes,{schemaVersion=1,entries=entries}
end
local function image()return{width=1254,height=1254}end
local heroes,manifest=fresh();check(M.validate(heroes,manifest,image)==3,'complete current authoring catalog accepted')
local function reject(mutate,why,inspector,expected)
 local hs,ms=fresh();mutate(hs,ms);local ok,err=pcall(M.validate,hs,ms,inspector or image)
 check(not ok and(not expected or tostring(err):find(expected,1,true)),why)
end
reject(function(_,m)table.remove(m.entries)end,'missing published hero diagnosed',nil,'hero_gamma')
reject(function(_,m)m.entries[3].id='future_unpublished' end,'unpublished portrait rejected')
reject(function(_,m)m.entries[3]=m.entries[2]end,'duplicate portrait ID rejected')
local reads=0
reject(function(_,m)m.entries[1].file='../outside.png'end,'traversal rejected before I/O',function()reads=reads+1;return image()end)
check(reads==0,'no arbitrary namespace read')
reject(function(_,m)m.entries[1].file='inside..png'end,'double-dot basename rejected')
reject(function(_,m)m.entries[2].file=m.entries[1].file end,'same portrait tile cannot silently alias another hero')
reject(function(_,m)m.entries[1].displayName='Old name'end,'stale authoring name rejected')
reject(function(_,m)m.entries[1].columns=0 end,'zero-sized atlas rejected')
reject(function(_,m)m.entries[1].tile=0.5 end,'fractional tile rejected')
reject(function(_,m)m.entries[1].tile=1 end,'out-of-range tile rejected')
reject(function()end,'nonsquare portrait tile rejected',function()return{width=512,height=256}end)
reject(function()end,'missing source image rejected',function()return nil end)
reject(function()end,'oversized source rejected',function()return{width=4097,height=4097}end)
heroes,manifest=fresh();for _,row in ipairs(manifest.entries)do row.columns=4;row.rows=4;row.tile=15 end
check(M.validate(heroes,manifest,image)==3,'1254px normalized quarter tiles require no integer divisibility')
reject(function(_,m)m.entries[1].resource='/Game/Foreign.Texture'end,'packaged-item namespace cannot replace a portrait source')
reject(function(h)h[3]=h[2]end,'duplicate authoring identity rejected')
reject(function(h)for i=#h,1,-1 do h[i]=nil end end,'empty authoring catalog rejected')
reject(function(_,m)m.schemaVersion=2 end,'unsupported manifest version rejected')

-- Structural PNG guard only; pixel decoding remains the native image contract.
local directory=p.join(b.root,'.ai-collab/portrait-coverage-test-'..b.lib('time').monotonic_ms()..'-'..math.random(100000,999999));assert(not p.exists(directory));p.mkdir_p(directory)
local file=p.join(directory,'header-fixture.png')
local bytes='\137PNG\13\10\26\10'..string.pack('>I4',13)..'IHDR'..string.pack('>I4I4',1,1)..string.char(8,6,0,0,0)..string.rep('\0',4)..'\0\0\0\0IEND\174\66\96\130'
p.write(file,bytes,false,true);local info=M.inspect_png(file)
check(info.width==1 and info.height==1 and info.structural_check_only,'bounded header inspection explicitly does not claim pixel decoding')
p.write(file,'not a PNG'..bytes:sub(10),true,true);check(not pcall(M.inspect_png,file),'bad signature rejected')
p.write(file,bytes:sub(1,-2),true,true);check(not pcall(M.inspect_png,file),'truncated trailer rejected')
check(not pcall(M.inspect_png,p.join(directory,'missing.png')),'missing file diagnosed')

-- Invoke the real sync entry. A coverage failure after name projection must
-- not persist an otherwise tempting partial manifest update.
local function sync(missing)
 local hs,ms=fresh();ms.entries[1].displayName='Old name';if missing then table.remove(ms.entries)end
 local original=j.encode(ms);local writes,saved=0,nil
 local env=setmetatable({package={path=''},arg={},print=function()end,require=function(name)
  if name=='_bootstrap'then return{root='fixture',lib=function(key)
   if key=='path'then return{join=function(...)return table.concat({...},'/')end,is_file=function()return false end}end
   assert(key=='json');return{read=function()return j.decode(original)end,write=function(_,value)writes=writes+1;saved=value end}
  end}end
  if name=='content_builder'then return{new=function()return{include=function(file)if file=='templates/heroes.lua'then return hs end;return{}end}end}end
  assert(name=='ue_hud_portrait_coverage');return setmetatable({inspect_png=image},{__index=M})
 end},{__index=_G})
 local ok=pcall(assert(loadfile('scripts/sync_ue_hud_manifest_labels.lua','t',env)))
 check(j.encode(ms)==original,'sync leaves the original input unchanged')
 if missing then
  local row=ok and saved and saved.entries[#saved.entries]
  check(ok and writes==1 and row and row.placeholder==true and row.file==M.PLACEHOLDER_FILE,'real sync gives a hero without art the shared placeholder')
 else check(ok and writes==1 and saved.entries[1].displayName=='Hero alpha','real sync persists completed label projection once')end
end
sync(true);sync(false)
-- Placeholder rows: shared file allowed only when declared; authored tiles stay unique.
do
 local hs,ms=fresh()
 hs[#hs+1]={id='hero_delta',display_name='Hero delta'};hs[#hs+1]={id='hero_eps',display_name='Hero eps'}
 local added=M.placeholder_entries(hs,ms)
 check(#added==2 and added[1]=='hero_delta','placeholders added in catalog order')
 check(M.validate(hs,ms,image)==5,'two heroes share the declared placeholder')
 check(#M.placeholder_entries(hs,ms)==0,'placeholder projection is idempotent')
 ms.entries[#ms.entries].file='alpha.png'
 check(not pcall(M.validate,hs,ms,image),'placeholder flag cannot borrow authored art')
 local hs3,ms3=fresh();hs3[#hs3+1]={id='hero_gone',display_name='Gone'};M.placeholder_entries(hs3,ms3);table.remove(hs3)
 local before=#ms3.entries;local removed=M.prune_placeholders(hs3,ms3)
 check(#removed==1 and removed[1]=='hero_gone'and #ms3.entries==before-1 and M.validate(hs3,ms3,image)==3,'placeholder of a hero that left the catalog is pruned')
 local hs4,ms4=fresh();table.remove(hs4,1)
 check(#M.prune_placeholders(hs4,ms4)==0 and not pcall(M.validate,hs4,ms4,image),'authored art for an unknown hero still fails')
 local hs2,ms2=fresh();ms2.entries[1].placeholder=false
 check(not pcall(M.validate,hs2,ms2,image),'placeholder flag must be exactly true when present')
end
-- Hero ability icons: atlas, packaged per-ability texture, or the declared placeholder.
do
 local function kit()
  local heroes={{id='hero_alpha',display_name='Hero alpha',abilities={'alpha_q','alpha_w','shared_r'}},
   {id='hero_beta',display_name='Hero beta',abilities={'beta_q','shared_r'}}}
  local abilities={{id='alpha_q',display_name='Alpha Q'},{id='alpha_w',display_name='Alpha W'},{id='beta_q',display_name='Beta Q'},
   {id='shared_r',display_name='Shared R'},{id='retired_e',display_name='Retired E'}}
  local entries={
   {kind='ability',id='alpha_q',displayName='Alpha Q',file='atlas.png',columns=4,rows=4,tile=3},
   {kind='ability',id='alpha_w',displayName='Alpha W',file='alpha_w.png',columns=1,rows=1,tile=0,resource='/Game/Omoba/UI/Abilities/alpha_w.alpha_w'},
   {kind='ability',id='beta_q',displayName='Beta Q',file='beta_q.png',columns=1,rows=1,tile=0,resource='/Game/Omoba/UI/Abilities/beta_q.beta_q'},
   {kind='ability',id='shared_r',displayName='Shared R',file='atlas.png',columns=4,rows=4,tile=4},
   {kind='ability',id='retired_e',displayName='Retired E',file='atlas.png',columns=4,rows=4,tile=5},
  }
  return heroes,abilities,{schemaVersion=1,entries=entries}
 end
 local sources={}
 local function inspect(name)sources[#sources+1]=name;return image()end
 local hs,as,ms=kit();sources={}
 local count,pending,packaged=M.validate_abilities(hs,as,ms,inspect)
 check(count==4 and #pending==0 and packaged==2,'every equipped ability covered; shared kit ability counted once')
 local packaged_source=false
 for _,name in ipairs(sources)do if name=='Abilities/Source/alpha_w.png'then packaged_source=true end end
 check(packaged_source,'packaged ability is checked against its project source PNG')
 local function reject_ability(mutate,why,expected,inspector)
  local h,a,m=kit();mutate(h,a,m);local ok,err=pcall(M.validate_abilities,h,a,m,inspector or image)
  check(not ok and(not expected or tostring(err):find(expected,1,true)),why)
 end
 reject_ability(function(_,_,m)table.remove(m.entries,3)end,'hero ability without an icon is diagnosed by id','beta_q')
 reject_ability(function(h)h[1].abilities[4]='ghost_spell' end,'hero equipping unknown ability rejected','ghost_spell')
 reject_ability(function(_,_,m)m.entries[1].id='ghost_spell' end,'icon for unknown ability rejected')
 reject_ability(function(_,_,m)m.entries[6]=m.entries[1]end,'duplicate ability icon rejected')
 reject_ability(function(_,_,m)m.entries[1].displayName='Old'end,'stale ability label rejected')
 reject_ability(function(_,_,m)m.entries[2].resource='/Game/Omoba/UI/Items/alpha_w.alpha_w'end,'packaged ability cannot use the item package')
 reject_ability(function(_,_,m)m.entries[2].file='other.png'end,'packaged ability file must match its id')
 reject_ability(function(_,_,m)m.entries[2].columns=4;m.entries[2].rows=4 end,'packaged ability is a single tile')
 reject_ability(function(_,_,m)m.entries[1].file='../outside.png'end,'ability file traversal rejected')
 reject_ability(function(_,_,m)m.entries[1].placeholder=true end,'placeholder flag cannot borrow atlas art')
 reject_ability(function()end,'missing ability PNG rejected',nil,function()return nil end)
 reject_ability(function()end,'nonsquare ability tile rejected',nil,function()return{width=512,height=256}end)
 -- A new hero without icons gets the declared placeholder (with a warning list), never a blank slot.
 hs,as,ms=kit()
 hs[#hs+1]={id='hero_gamma',display_name='Hero gamma',abilities={'gamma_q','alpha_q'}}
 as[#as+1]={id='gamma_q',display_name='Gamma Q'}
 local added=M.ability_placeholder_entries(hs,as,ms)
 check(#added==1 and added[1]=='gamma_q','missing hero ability receives the placeholder icon')
 local row=ms.entries[#ms.entries]
 check(row.kind=='ability'and row.placeholder==true and row.file==M.ABILITY_PLACEHOLDER_FILE and row.displayName=='Gamma Q','placeholder row is declared and labelled')
 check(#M.ability_placeholder_entries(hs,as,ms)==0,'ability placeholder projection is idempotent')
 count,pending=M.validate_abilities(hs,as,ms,image)
 check(count==5 and #pending==1 and pending[1]=='gamma_q','placeholder abilities are reported for a build warning')
 table.remove(hs)
 local removed=M.prune_ability_placeholders(hs,as,ms)
 check(#removed==1 and removed[1]=='gamma_q','placeholder of an ability no hero equips is pruned')
 check(#M.prune_ability_placeholders(hs,as,ms)==0 and M.validate_abilities(hs,as,ms,image)==4,'authored ability rows are never pruned')
end
-- The real repository catalog: every equipped hero ability has a HUD icon row.
do
 local ctx=require('content_builder').new(p.join(b.root,'scripts/lua_data'))
 local real=j.read(p.join(b.root,'omfue/Content/Omoba/UI/hud-assets-v1.json'))
 local count,pending=M.validate_abilities(ctx.include('templates/heroes.lua'),ctx.include('templates/abilities.lua'),real,
  function(name)return M.inspect_png(p.join(b.root,'omfue/Content/Omoba/UI',name))end)
 check(count>0 and #pending==0,'repository hero abilities all have authored icons (no placeholder)')
end
-- The real repository catalog: every hero has authored portrait art, none uses the shared placeholder.
do
 local ctx=require('content_builder').new(p.join(b.root,'scripts/lua_data'))
 local real=j.read(p.join(b.root,'omfue/Content/Omoba/UI/hud-assets-v1.json'))
 local heroes=M.validate(ctx.include('templates/heroes.lua'),real,function(name)return M.inspect_png(p.join(b.root,'omfue/Content/Omoba/UI',name))end)
 local placeholders={}
 for _,row in ipairs(real.entries)do if row.kind=='portrait'and(row.placeholder~=nil or row.file==M.PLACEHOLDER_FILE)then placeholders[#placeholders+1]=row.id end end
 check(heroes>0 and #placeholders==0,'repository heroes all have authored portraits (no placeholder): '..table.concat(placeholders,','))
end
-- Match loading-card art: optional 2:3 art per hero; a hero without it falls back to its cropped portrait.
do
 local function tall()return{width=832,height=1248}end
 local function kit()
  local hs,ms=fresh()
  ms.entries[#ms.entries+1]={kind='loading',id='hero_alpha',displayName='Hero alpha',file='alpha-loading.png',columns=1,rows=1,tile=0}
  ms.entries[#ms.entries+1]={kind='loading',id='hero_beta',displayName='Hero beta',file='beta-loading.png',columns=1,rows=1,tile=0}
  return hs,ms
 end
 local hs,ms=kit()
 local covered,fallback=M.validate_loading(hs,ms,tall)
 check(covered==2 and #fallback==1 and fallback[1]=='hero_gamma','hero without loading art is reported for the portrait fallback warning')
 check(M.validate(hs,ms,image)==3,'loading rows do not disturb portrait coverage')
 local function reject_loading(mutate,why,expected,inspector)
  local h,m=kit();mutate(h,m);local ok,err=pcall(M.validate_loading,h,m,inspector or tall)
  check(not ok and(not expected or tostring(err):find(expected,1,true)),why)
 end
 reject_loading(function(_,m)m.entries[#m.entries].id='ghost_hero' end,'loading art for unknown hero rejected','ghost_hero')
 reject_loading(function(_,m)m.entries[#m.entries]=m.entries[#m.entries-1]end,'duplicate loading art rejected')
 reject_loading(function(_,m)m.entries[#m.entries].file='alpha-loading.png' end,'loading art cannot borrow another hero file')
 reject_loading(function(_,m)m.entries[#m.entries].file=M.PLACEHOLDER_FILE end,'loading art cannot use the portrait placeholder')
 reject_loading(function(_,m)m.entries[#m.entries].file='../x.png' end,'loading art traversal rejected')
 reject_loading(function(_,m)m.entries[#m.entries].displayName='Old' end,'stale loading art name rejected')
 reject_loading(function(_,m)m.entries[#m.entries].columns=2 end,'loading art is one image, not an atlas')
 reject_loading(function(_,m)m.entries[#m.entries].resource='/Game/X.X' end,'loading art has no packaged resource')
 reject_loading(function()end,'square art is not loading art','not 2:3',image)
 reject_loading(function()end,'missing loading PNG rejected',nil,function()return nil end)
 -- The real repository catalog: every hero has authored loading-card art (no portrait fallback).
 local ctx=require('content_builder').new(p.join(b.root,'scripts/lua_data'))
 local real=j.read(p.join(b.root,'omfue/Content/Omoba/UI/hud-assets-v1.json'))
 local real_covered,real_fallback=M.validate_loading(ctx.include('templates/heroes.lua'),real,
  function(name)return M.inspect_png(p.join(b.root,'omfue/Content/Omoba/UI',name))end)
 check(real_covered>0 and #real_fallback==0,'repository heroes all have loading-card art: missing '..table.concat(real_fallback,','))
end
print(('PASS HUD portrait coverage: %d checks; self-contained fixtures, no Unreal/build/network'):format(count))
