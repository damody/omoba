-- Verify actual UE JSON writer output against the production Lua contract.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash')
local policy=require('moba_managed_session')
local root=p.absolute(assert(arg[1],'fresh evidence required'),b.root)
assert(not p.exists(root));p.mkdir_p(root)
local native=p.join(b.root,'omfue/Saved/Automation/ManagedNextMatch')
local count,fixtures=0,{}
local function check(value,message)assert(value,message);count=count+1 end
local ok,err=xpcall(function()
 local folders={};for name in require('lfs').dir(native)do if name:match('^[%x]+$') and #name==32 then folders[#folders+1]=name end end
 table.sort(folders)
 for _,name in ipairs(folders)do
  local folder=p.join(native,name);local ready_file=p.join(folder,'ready.json');local retired_file=p.join(folder,'retired.json')
  if p.is_file(ready_file) and p.is_file(retired_file)then
   local ready=policy.request(j.read(ready_file));local retired=policy.retired(j.read(retired_file))
   check(ready.action=='ready','native readiness action')
   local manifest={schema=policy.manifest_schema,players={ready.player_id}}
   for _,key in ipairs({'session_id','generation','catalog_identity_hash','content_hash','map_catalog_hash'})do manifest[key]=ready[key]end
   local model=policy.new(manifest)
   check(model.accept(ready),'actual UE numeric/string encoding accepted by Lua')
   check(model.snapshot().phase=='closing','actual UE readiness completes one-member policy')
   check(model.confirm_retirement(retired),'native retirement has exact same current scope/source epoch')
   check(model.snapshot().retirement_confirmed[1]==ready.player_id,'native retirement recorded once')
   -- No process exit is inferred from a synthetic native test's injected OS boundary.
   check(not model.retire_round(true,true),'JSON alone cannot prove original process retirement')
   check(not model.commit_next_generation(),'JSON alone never allocates another match')
   fixtures[#fixtures+1]={ready_file=ready_file,ready_sha256=h.sha256(ready_file),retired_file=retired_file,retired_sha256=h.sha256(retired_file)}
  end
 end
 check(#fixtures>0,'actual UE output required, not generated Lua fixtures')
end,debug.traceback)
j.write(p.join(root,'summary.json'),{success=ok,assertions=count,fixtures=fixtures,authority_games=0,authority_ack=false,
 scope='actual UE JSON writer files read by production Lua contract; synthetic native Finished and final OS boundary injected, no process success inferred',
 error=ok and '' or err},false)
if not ok then p.write(p.join(root,'errors.md'),err,false);error(err,0)end
print('PASS '..count..' native-to-Lua contract assertions; '..#fixtures..' actual UE file pairs; no process exit inferred')
