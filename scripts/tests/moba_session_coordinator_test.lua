local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap')
local p,j=b.lib('path'),b.lib('json')
local coordinator=require('moba_session_coordinator')
local launch=require('moba_role_launch')
local root=p.absolute(assert(arg[1],'new evidence output required'),b.root)
assert(not p.exists(root));p.mkdir_p(root)
local count=0
local function check(value,message)assert(value,message);count=count+1 end
local function refused(fn,message)check(not pcall(fn),message)end
local options=launch.options({'--interactive-selection','--profile','debug','--no-build'})
check(coordinator.supported(options),'local host supported')
for _,args in ipairs({{'--prepare-only'},{'--server-only'},{'--finish-timeout-seconds','1'},
 {'--local-player','1'},{'--connect','192.0.2.1','--recipe','host.json','--local-player','1'}})do
 check(not coordinator.supported(launch.options(args)),'single match mode must stay single match')
end
local ok,err=xpcall(function()
 local recipe=launch.load_recipe(options);local humans={}
 for _,player in ipairs(recipe.players)do if player.bot==false then humans[#humans+1]=player.player_id end end
 table.sort(humans)
 local arguments={'--interactive-selection','--recipe','original.lua','--config','original.toml','--output','original',
  '--profile','debug','--no-build','--hero','1=lumen'}
 local original_arguments=j.encode(arguments)
 local calls,session,first_output=0
 coordinator.run(arguments,options,p.join(root,'two-rounds'),function(args)
  calls=calls+1;local current=launch.options(args)
  local bootstrap=j.read(current.managed_session_bootstrap)
  check(bootstrap.generation==calls,'fresh generation')
  session=session or bootstrap.session_id
  check(bootstrap.session_id==session,'stable session identity')
  check(current.interactive_selection and current.no_build and current.profile=='debug','authored launch semantics retained')
  check(current.hero_selections[1]=='lumen','explicit hero binding retained')
  check(j.encode(j.read(current.recipe))==j.encode(recipe),'immutable author snapshot used, not finalized reply')
  check(p.read(current.config,true)==p.read(p.join(b.root,'omb/game.toml'),true),'base config preserved')
  check(not p.exists(current.output),'new match output')
  if first_output then check(first_output~=current.output,'fresh round directory')else first_output=current.output end
  if calls==1 then return {managed_next={kind='start_selection',session_id=session,generation=2,players=humans}}end
  return {}
 end)
 check(calls==2,'only committed effect starts next generation')
 check(j.encode(arguments)==original_arguments,'caller arguments unchanged')
 check(j.read(p.join(root,'two-rounds','completed.json')).generation==2,'completion recorded')
 refused(function()coordinator.run({},options,p.join(root,'two-rounds'),function()error('must not execute')end)end,'exclusive output')
 for _,mode in ipairs({'foreign-session','foreign-roster','foreign-generation','changed-recipe','changed-config','role-failure'})do
  local folder=p.join(root,mode);local invoked=0
  refused(function()coordinator.run({},options,folder,function(args)
   invoked=invoked+1;local current=launch.options(args);local bootstrap=j.read(current.managed_session_bootstrap)
   if mode=='role-failure' then error('injected role failure')end
   if mode=='changed-recipe' then p.write(current.recipe,'{}',true)
   elseif mode=='changed-config' then p.write(current.config,'invalid config',true)end
   return {managed_next={kind='start_selection',session_id=mode=='foreign-session' and string.rep('b',64) or bootstrap.session_id,
    generation=mode=='foreign-generation' and 3 or 2,players=mode=='foreign-roster' and {999} or humans}}
  end)end,'invalid role/snapshot must fail: '..mode)
  check(invoked==1,'invalid state cannot create another role: '..mode)
  check(p.is_file(p.join(folder,'errors.md')),'failure recorded')
  check(not p.exists(p.join(folder,'round-2')),'no next generation allocation')
 end
 refused(function()coordinator.round_args({'--managed-session-bootstrap','other'},'r','c','o','b')end,'caller cannot inject bootstrap')
 -- Bounded session: rounds before the last offer the next match; the last is a bounded single match.
 local bounded_args={'--interactive-selection','--profile','debug','--no-build','--managed-rounds','2','--managed-final-round-seconds','180'}
 local bounded=launch.options(bounded_args)
 check(bounded.managed_rounds==2 and bounded.managed_final_round_seconds==180,'bounded round limits parsed')
 check(coordinator.supported(bounded),'bounded sessions stay managed')
 for _,bad in ipairs({{'--managed-rounds','1','--managed-final-round-seconds','9'},{'--managed-rounds','2'},
  {'--managed-final-round-seconds','9'},{'--managed-rounds','2','--managed-final-round-seconds','0'}})do
  refused(function()launch.options(bad)end,'invalid bounded limits rejected: '..table.concat(bad,' '))
 end
 local rounds={}
 coordinator.run(bounded_args,bounded,p.join(root,'bounded'),function(args)
  local current=launch.options(args);rounds[#rounds+1]=current
  check(not current.managed_rounds and not current.managed_final_round_seconds,'round limits never reach the role')
  if #rounds==1 then
   check(current.managed_session_bootstrap and not current.finish_timeout_seconds,'first round offers the next match')
   local bootstrap=j.read(current.managed_session_bootstrap)
   return {managed_next={kind='start_selection',session_id=bootstrap.session_id,generation=2,players=humans}}
  end
  check(not current.managed_session_bootstrap and current.round_time_box_seconds==180 and not current.finish_timeout_seconds,'final round is a time-boxed single match')
  return {}
 end)
 check(#rounds==2,'bounded session stops after the final round')
 local completion=j.read(p.join(root,'bounded','completed.json'))
 check(completion.generation==2 and completion.bounded_final_round==true,'bounded completion recorded')
 check(not p.exists(p.join(root,'bounded','round-2','bootstrap.json')),'final round has no next-match bootstrap')
 local timed_calls=0
 coordinator.run(bounded_args,bounded,p.join(root,'timed-out'),function(args)
  timed_calls=timed_calls+1;return {round_timed_out=true}
 end)
 local timed=j.read(p.join(root,'timed-out','completed.json'))
 check(timed_calls==1 and timed.generation==1 and timed.verdict=='inconclusive','a time-boxed round stops the session as inconclusive')
 refused(function()coordinator.run(bounded_args,bounded,p.join(root,'bounded-overrun'),function(args)
  local current=launch.options(args)
  if current.managed_session_bootstrap then
   return {managed_next={kind='start_selection',session_id=j.read(current.managed_session_bootstrap).session_id,generation=2,players=humans}}
  end
  return {managed_next={kind='start_selection',session_id='x',generation=3,players=humans}}
 end)end,'final round cannot start a third generation')
end,debug.traceback)
j.write(p.join(root,'summary.json'),{success=ok,assertions=count,scope='real immutable snapshots; injected role boundary, no authority or GUI',
 authority_games=0,error=ok and '' or err},false)
if not ok then p.write(p.join(root,'errors.md'),err,false);error(err,0)end
print('PASS '..count..' coordinator assertions; no game/GUI')
