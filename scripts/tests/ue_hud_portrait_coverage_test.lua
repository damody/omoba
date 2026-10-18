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
   if key=='path'then return{join=function(...)return table.concat({...},'/')end}end
   assert(key=='json');return{read=function()return j.decode(original)end,write=function(_,value)writes=writes+1;saved=value end}
  end}end
  if name=='content_builder'then return{new=function()return{include=function(file)if file=='templates/heroes.lua'then return hs end;return{}end}end}end
  assert(name=='ue_hud_portrait_coverage');return{validate=M.validate,inspect_png=image}
 end},{__index=_G})
 local ok=pcall(assert(loadfile('scripts/sync_ue_hud_manifest_labels.lua','t',env)))
 check(j.encode(ms)==original,'sync leaves the original input unchanged')
 if missing then check(not ok and writes==0,'real sync rejects missing hero before any write')
 else check(ok and writes==1 and saved.entries[1].displayName=='Hero alpha','real sync persists completed label projection once')end
end
sync(true);sync(false)
print(('PASS HUD portrait coverage: %d checks; self-contained fixtures, no Unreal/build/network'):format(count))
