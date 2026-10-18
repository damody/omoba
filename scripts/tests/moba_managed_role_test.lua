local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local p,j,r,t=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('time')
local root=p.absolute(assert(arg[1],'fresh output required'),b.root)
r.supervise_workflow(source,arg)
assert(not p.exists(root));p.mkdir_p(root)
local launch=require('moba_role_launch');local control=require('moba_managed_control')
local lua=b.lib('platform').lua_executable;local fixture=p.join(b.root,'scripts/tests/managed_role_fixture.lua')
local count,retired=0,0
local function check(value,message)assert(value,message);count=count+1 end
local ok,err=xpcall(function()
 for _,behavior in ipairs({'ready','cancel','manual','crash'})do
  local round=p.join(root,behavior);p.mkdir_p(round)
  local output=p.mkdir_p(p.join(round,'match'))
  local bootstrap=p.join(round,'bootstrap.json')
  j.write(bootstrap,{schema=control.bootstrap_schema,session_id=string.rep('a',64),generation=1},false)
  local plan={mode='host',output=output,human_count=2,local_human_count=2,clients={},
   managed_session_bootstrap=bootstrap,catalog_identity_hash='0123456789abcdef',content_hash='123456789abcdef0',map_catalog_hash='23456789abcdef01',
   server={exe=lua,args={fixture,'server'},cwd=b.root,env={}}}
  for _,id in ipairs({17,99})do
   plan.clients[#plan.clients+1]={player_id=id,team_id=1,
    runtime={exe=lua,args={fixture,'runtime',tostring(id),'1'},cwd=b.root,env={}},
    unreal={args={fixture,'renderer',id==17 and behavior or 'ready'},cwd=b.root,env={}}}
  end
  local success=pcall(launch.launch,plan,lua,r,t)
  check(success==(behavior~='crash'),'original child failure propagated: '..behavior)
  local lifecycle=j.read(p.join(output,'lifecycle.json'))
  check(lifecycle.cleanup_succeeded,'complete original cleanup: '..behavior)
  local records=j.read(p.join(output,'owned-processes.json'))
  check(#records==7,'server, two runtimes, two waiters and two original children journalled')
  for _,record in ipairs(records)do check(not r.owned_alive(record),'original lifetime retired');retired=retired+1 end
  if behavior=='ready'then
   check(plan.managed_next and plan.managed_next.kind=='start_selection' and plan.managed_next.generation==2,'verified close commits new generation')
   check(plan.managed_next.players[1]==17 and plan.managed_next.players[2]==99,'actual roster retained')
  else check(plan.managed_next==nil,'manual close/cancellation/crash cannot restart')end
 end
end,debug.traceback)
j.write(p.join(root,'summary.json'),{success=ok,assertions=count,original_lifetimes_retired=retired,
 scope='actual production role launcher, private Job, native Child leases; neutral Lua participants with synthetic retirement, no Unreal/game',
 authority_games=0,authority_ack=false,error=ok and '' or err},false)
if not ok then p.write(p.join(root,'errors.md'),err,false);error(err,0)end
print('PASS '..count..' production role ownership assertions; '..retired..' original lifetimes retired; neutral participants only')
