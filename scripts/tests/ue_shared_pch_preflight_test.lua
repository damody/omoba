local script=debug.getinfo(1,'S').source:sub(2)
package.path=script:match('^(.*)[/\\]')..'/../?.lua;'..package.path
local m=require('ue_shared_pch_preflight')
local count=0
local function check(value,message) assert(value,message);count=count+1 end
local engine='D:/引擎 空白'
local file=engine..'/Engine/Intermediate/Build/SharedPCH.Editor.pch'
local stat={mode='file',size=3,modification=10}
local function attributes(p) if p:lower()==file:lower() then return stat end end
check(m.canonical('../Intermediate/Build/SharedPCH.Editor.pch',engine..'/Engine/Source')==file,'relative normalization')
check(m.canonical('D:/../../outside')==nil,'drive traversal')
check(m.canonical('//server/share/a/../b')=='//server/share/b','UNC normalization')
local rows=m.candidates({
  '/Fp"../Intermediate/Build/SharedPCH.Editor.pch"\r\n/Fp"'..file..'"',
  '/Fp"../Intermediate/../Source/SharedPCH.Evil.pch"\n/Fp"../Intermediate/Build/Private.pch"',
  '/Fp"../Intermediate/Build/SharedPCH.Missing.pch"\n/Fp"unterminated',
},engine,attributes)
check(#rows==1 and rows[1].path==file,'deduplicate valid, reject outside/private/missing/malformed')
local opened,closed=0,0
local function open(p,mode)
  check(p==file and mode=='rb','read only exact input')
  opened=opened+1
  local read=false
  return {read=function(_,size) check(size==1024*1024,'bounded chunk');if not read then read=true;return 'abc' end end,
    close=function() closed=closed+1;return true end}
end
local clock=0
local function now() clock=clock+1;return clock end
local result=m.warm(rows,{attributes=attributes,open=open,clock=now})
check(result.bytes==3 and result.files==1 and result.rows[1].elapsed_ms==1 and closed==1,'bytes/elapsed/close')
result=m.warm(rows,{attributes=attributes,open=open,clock=now,budget=2})
check(result.skipped==1 and result.files==0 and opened==1,'budget does not open file')
local n=0
check(not pcall(m.warm,rows,{attributes=function() n=n+1;if n==1 then return stat end;return {size=4,modification=11} end,open=open,clock=now}),'changed input fails')
check(not pcall(m.warm,rows,{attributes=function()return {size=3,modification=11}end,open=open,clock=now}),'changed before read fails')
check(#m.candidates({},engine,attributes)==0,'fresh build no-op')
-- Real directory discovery: a plugin's private SDK can itself contain an
-- Intermediate directory. It is not a UE module and must never be traversed.
local b=require('_bootstrap');local p=b.lib('path');local time=b.lib('time')
local temp=p.join(b.root,'.ai-collab','pch-inventory-test-'..time.monotonic_ms()..'-'..math.random(100000,999999))
check(not p.exists(temp),'fresh inventory fixture')
local project=p.join(temp,'project');local actual_engine=p.join(temp,'engine')
local allowed=p.join(actual_engine,'Engine/Intermediate/Build/SharedPCH.Allowed.pch')
local private=p.join(actual_engine,'Engine/Intermediate/Build/SharedPCH.Private.pch')
local plugin=p.join(project,'Plugins/Category/DeclaredPlugin')
local function write(file,bytes)p.mkdir_p(p.parent(file));p.write(file,bytes,false,true)end
write(allowed,'abc');write(private,'private');write(p.join(plugin,'DeclaredPlugin.uplugin'),'{}')
local response='/Fp"'..allowed..'"\n'
write(p.join(plugin,'Intermediate/Build/Module.rsp'),response)
write(p.join(plugin,'PrivateSdk/Intermediate/Build/Unrelated.rsp'),'/Fp"'..private..'"\n')
write(p.join(plugin,'Web/node_modules/vendor/Intermediate/Build/Unrelated.rsp'),'/Fp"'..private..'"\n')
write(p.join(plugin,'Plugins/Nested/Nested.uplugin'),'{}')
write(p.join(plugin,'Plugins/Nested/Intermediate/Build/Nested.rsp'),response)
local discovered=m.run(project,actual_engine,time.monotonic_ms)
check(discovered.files==1 and discovered.bytes==3 and discovered.rows[1].path==m.canonical(allowed),
  'plugin boundary skips private SDK/Web inventories, preserves nested plugins and deduplicates PCH')
print(('PASS shared PCH preflight: %d checks'):format(count))
