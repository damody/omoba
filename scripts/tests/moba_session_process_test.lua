local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local p,j,r,t=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('time')
assert(#arg==1,'requires a new evidence directory');r.supervise_workflow(source,arg)
local adapter=require('moba_session_process')
local output=p.absolute(arg[1],b.root);assert(not p.exists(output));p.mkdir_p(output)
local owned,assertions={},0
local function check(value,message)assert(value,message);assertions=assertions+1 end
local function journal(record,label)owned[#owned+1]=r.validate_owned(record);j.write(p.join(output,'owned-'..#owned..'.json'),{label=label,record=record},false)end
local ok,err=xpcall(function()
 for index,code in ipairs({0,3,0xC0000005})do
  local dir=p.mkdir_p(p.join(output,'case-'..index))
  local spec={exe=b.lib('platform').lua_executable,args={p.join(b.root,'scripts','tests','owned_run_fixture.lua'),'child','1800',tostring(code)},cwd=b.root,
   stdout=p.join(dir,'child.stdout'),stderr=p.join(dir,'child.stderr')}
  local entry=adapter.spawn('unreal-p'..(index*17),spec,p.join(dir,'owned'),r,t,journal)
  check(#owned==index*2,'both original child and waiter journalled')
  check(r.owned_alive(entry.child) and r.owned_alive(entry.waiter),'both originals alive at startup')
  check(adapter.observe(entry,r)==true,'live child has a live waiter')
  local result=adapter.exit(entry,r)
  check(result.exit_code==code and result.retired,'production waiter returns exact exit code')
  local live,repeated=adapter.observe(entry,r)
  check(live==false and repeated.exit_code==code,'completed child observation uses same immutable exit lease')
  check(not r.owned_alive(entry.child) and not r.owned_alive(entry.waiter),'both lifetimes retired')
  check(not pcall(adapter.spawn,'unreal-p17',spec,entry.directory,r,t,journal),'cannot reuse a prior process output')
 end
 local bad={exe=b.lib('platform').lua_executable,args={},stdout=p.join(output,'bad.stdout'),stderr=p.join(output,'bad.stderr'),pid=99}
 local bad_directory=p.join(output,'bad')
 check(not pcall(adapter.spawn,'bad',bad,bad_directory,r,t,journal),'spec cannot grant a PID')
 check(not p.exists(bad_directory),'malformed spec rejected before creating an output directory')
end,debug.traceback)
local clean=true
for index=#owned,1,-1 do local record=owned[index];local retired,why=pcall(function()
 if r.owned_alive(record)then r.stop_owned(record)end;assert(r.wait_owned(record,5000) and not r.owned_alive(record),'original process not retired')
end);if not retired then clean=false;err=(err or '')..'\n'..tostring(why)end end
j.write(p.join(output,'summary.json'),{success=ok and clean,assertions=assertions,cleanup_succeeded=clean,authority_games=0,authority_ack=false,error=ok and '' or err},false)
if not ok or not clean then p.write(p.join(output,'errors.md'),'# Managed process adapter failed\n\n'..tostring(err)..'\n\nDecision: retire only original records, fail the private workflow and allocate no match.\n',false);error(err,0)end
print('PASS '..assertions..' production process-adapter assertions; six original lifetimes retired, no game/engine')
