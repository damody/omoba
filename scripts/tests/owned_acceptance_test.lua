local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local p,j,r,t=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('time');local acceptance=require('owned_acceptance')
assert(#arg==1,'requires exclusive evidence directory');r.supervise_workflow(source,arg)
local output=p.absolute(arg[1],b.root);assert(not p.exists(output));p.mkdir_p(output)
local checks=0;local function check(value,message)assert(value,message);checks=checks+1 end
local function copy(v)if type(v)~='table'then return v end;local result={};for k,x in pairs(v)do result[k]=copy(x)end;return result end
local fixture=p.join(b.root,'scripts','tests','owned_acceptance_fixture.lua')
local ok,err=xpcall(function()
 for _,mode in ipairs({'fail','missing','pass'})do
  local summary=p.join(output,mode..'-summary.json');local dir=p.join(output,mode)
  local spec={schema=acceptance.schema,script=fixture,args={mode,summary},output=dir,summary=summary}
  local result=acceptance.run(spec,r,t)
  check(result.success==(mode=='pass'),'only true successful exit and summary accepted')
  check(result.retired and result.cleanup_succeeded,'original lifetimes retired')
  check(not r.owned_alive(result.owner)and not r.owned_alive(result.waiter),'both actual owned processes retired')
  check(p.read(p.join(dir,'stdout.log')):find('exit=0',1,true)~=nil,'all modes include misleading intermediate success')
  local raw=j.read(result.exit_receipt_file)
  check(raw.exit_code==result.exit_code and raw.retired,'raw exact Windows exit retained')
  if mode=='fail'then
   check(result.exit_code~=0 and j.read(summary).success,'successful summary cannot override later failed exit')
   check(p.read(p.join(dir,'stderr.log')):find('deliberate later stage failure',1,true)~=nil,'original failure log preserved')
  elseif mode=='missing'then check(result.exit_code==0 and not p.exists(summary),'zero exit cannot substitute for required summary')end
  for _,mutate in ipairs({function(v)v.identity.creation_token=v.identity.creation_token..'1'end,function(v)v.identity.pid=v.identity.pid+1 end,
   function(v)v.identity.executable=fixture end,function(v)v.retired=false end})do
   local forged=copy(raw);mutate(forged);check(not pcall(acceptance.verify_exit,result.owner,forged,r),'mismatched lifetime/executable/retirement rejected')
  end
  check(not pcall(acceptance.run,spec,r,t),'existing evidence refused before another spawn')
 end
 local summary=p.join(output,'already.json');j.write(summary,{success=true},false)
 local dir=p.join(output,'stale');check(not pcall(acceptance.run,{schema=acceptance.schema,script=fixture,args={'pass',summary},output=dir,summary=summary},r,t),'stale final summary refused')
 check(not p.exists(dir),'stale summary rejected before output or spawn')
 local v=acceptance.run({schema=acceptance.schema,script=fixture,args={'missing','unused'},output=p.join(output,'exit-only')},r,t)
 check(v.success and v.exit_code==0,'summary-free tasks use actual exit only')
 check(v.retired and not r.owned_alive(v.owner)and not r.owned_alive(v.waiter),'summary-free lifetimes retired')
end,debug.traceback)
j.write(p.join(output,'summary.json'),{success=ok,assertions=checks,authority_games=0,authority_ack=false,error=ok and ''or tostring(err)},false)
assert(ok,err);print('PASS '..checks..' owned acceptance assertions; actual late failure rejected; no engine/game')
