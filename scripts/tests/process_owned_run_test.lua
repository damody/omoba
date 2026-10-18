local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local p,j,r,t=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('time')
assert(#arg==1,'requires an exclusive new evidence directory')
r.supervise_workflow(source,arg)
local output=p.absolute(arg[1],b.root);assert(not p.exists(output),'evidence directory already exists');p.mkdir_p(output)
local lua=b.lib('platform').lua_executable;local fixture=p.join(b.root,'scripts','tests','owned_run_fixture.lua')
local assertions=0;local owned={}
local function check(value,message)assert(value,message);assertions=assertions+1 end
local function copy(value)if type(value)~='table'then return value end;local result={};for k,v in pairs(value)do result[k]=copy(v)end;return result end
local function poll_file(file,waiter)
 return t.poll(15000,20,function()
  if p.is_file(file)then local ok,value=pcall(j.read,file);if ok then return value end end
  assert(r.owned_alive(waiter),'original waiter must remain alive until its full notice is available')
 end)
end
local ok,err=xpcall(function()
 for index,code in ipairs({0,3,0xC0000005})do
  local dir=p.mkdir_p(p.join(output,'case-'..index))
  local spec={delay_ms=1800,exit_code=code,stdout=p.join(dir,'child.stdout'),stderr=p.join(dir,'child.stderr'),
   ready_file=p.join(dir,'ready.json'),exit_file=p.join(dir,'exit.json')}
  local spec_file=p.join(dir,'spec.json');j.write(spec_file,spec,false)
  local _,waiter=r.spawn_owned(lua,{fixture,'waiter',spec_file},{cwd=b.root,stdout=p.join(dir,'waiter.stdout'),stderr=p.join(dir,'waiter.stderr')})
  owned[#owned+1]=waiter
  local notice=assert(poll_file(spec.ready_file,waiter),'native ready notice timed out')
  local identity=r.validate_owned_ready(notice);owned[#owned+1]=identity
  check(identity.executable:lower()==p.absolute(lua):lower(),'fixed Lua identity is exact')
  check(r.owned_alive(identity),'ready describes a live original child')
  local result=r.poll_owned_exit(identity,waiter,spec.exit_file,15000)
  local receipt=j.read(spec.exit_file)
  check(result.exit_code==code and result.retired,'actual Windows exit code retained')
  check(r.wait_owned(identity,5000) and not r.owned_alive(identity),'original child is retired')
  check(r.wait_owned(waiter,5000) and not r.owned_alive(waiter),'original waiter is retired')
  local original_is_file=p.is_file;local missed=false
  p.is_file=function(file)if file==spec.exit_file and not missed then missed=true;return false end;return original_is_file(file)end
  local recovered,replayed=pcall(r.poll_owned_exit,identity,waiter,spec.exit_file,15000);p.is_file=original_is_file
  check(recovered and missed and replayed.exit_code==code,'final read closes the publish/producer-exit observation race')
  check(p.read(spec.stdout):find('neutral child stdout',1,true)~=nil,'stdout goes to its dedicated file')
  check(p.read(spec.stderr):find('neutral child stderr',1,true)~=nil,'stderr goes to its dedicated file')
  for _,change in ipairs({function(v)v.identity.creation_token=v.identity.creation_token..'1'end,
   function(v)v.identity.executable=p.join(b.root,'tools','lua-host','target','debug','omoba-lua-host.exe')end,
   function(v)v.exit_code=-1 end,function(v)v.retired=false end,function(v)v.pid=identity.pid end})do
   local invalid=copy(receipt);change(invalid);check(not pcall(r.validate_owned_exit,identity,invalid),'forged exit receipt rejected after process retirement')
  end
  local saved=p.read(spec.ready_file,true)
  check(not pcall(r.run_owned_to_files,lua,{fixture,'child','100','0'},{cwd=b.root,stdout=p.join(dir,'negative.stdout'),stderr=p.join(dir,'negative.stderr'),ready_file=spec.ready_file}),
   'existing ready file refuses another spawn')
  check(p.read(spec.ready_file,true)==saved,'existing notice is never overwritten')
  check(not p.exists(p.join(dir,'negative.stdout')) and not p.exists(p.join(dir,'negative.stderr')),'existing notice rejected before creating child logs')
 end
 local dir=p.mkdir_p(p.join(output,'no-job'))
 local spec={stdout=p.join(dir,'child.stdout'),stderr=p.join(dir,'child.stderr'),ready_file=p.join(dir,'ready.json')}
 local spec_file=p.join(dir,'spec.json');j.write(spec_file,spec,false)
 local rejected=r.run(lua,{fixture,'no-job',spec_file},{cwd=b.root,env={_OMOBA_PRIVATE_WORKFLOW_JOB=''},check=false})
 check(rejected.exit_code==0 and rejected.stdout:find('PASS missing workflow rejected',1,true)~=nil,'private job membership is mandatory')
 check(not p.exists(spec.ready_file) and not p.exists(spec.stdout) and not p.exists(spec.stderr),'unmanaged call creates no child or outputs')
end,debug.traceback)
local clean=true
for index=#owned,1,-1 do local record=owned[index];local retired,why=pcall(function()
 if r.owned_alive(record)then r.stop_owned(record)end;assert(r.wait_owned(record,5000) and not r.owned_alive(record),'original process was not retired')
end);if not retired then clean=false;err=(err or '')..'\n'..tostring(why)end end
j.write(p.join(output,'summary.json'),{success=ok and clean,assertions=assertions,cleanup_succeeded=clean,authority_games=0,authority_ack=false,error=ok and '' or err},false)
if not ok or not clean then p.write(p.join(output,'errors.md'),'# Owned run platform failure\n\n'..tostring(err)..'\n\nDecision: refuse receipt adoption; retire only original recorded lifetimes and fail this private workflow. No engine or authority game.\n',false);error(err or 'owned-run cleanup failed',0)end
print('PASS '..assertions..' actual owned-run assertions, three exact exit codes and original lifetime cleanup; no game/engine')
