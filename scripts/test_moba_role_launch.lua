local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json,process=b.lib('path'),b.lib('json'),b.lib('process')
local launch=require('moba_role_launch')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','role-launch-tests'))
local root
for i=1,1000 do local candidate=path.join(parent,os.time()..'-'..i);if lfs.mkdir(candidate) then root=candidate;break end end
assert(root)
local tests=0
local function test(name,fn) fn();tests=tests+1;print('PASS '..name) end
local function rejects(fn) assert(not pcall(fn),'expected rejection') end
local function contains(values,wanted) for _,v in ipairs(values) do if v==wanted then return true end end;return false end
local base=path.join(b.root,'omb','game.toml')
local original=path.read(base,true)
local plan
test('shared pre-match selections preserve recipe and reject ownership ambiguity',function()
  local recipe={players={
    {player_id=1,team_id=1,role='top',hero='old',bot=false},
    {player_id=2,team_id=1,role='mid',hero='old',bot=false},
    {player_id=3,team_id=2,role='top',hero='old',bot=true}}}
  local options=launch.options({'--hero','2=training_luminary','--hero','1=training_luminary'})
  local selected=launch.select_heroes(recipe,options.hero_selections)
  assert(selected.players[1].hero=='training_luminary' and selected.players[2].hero=='training_luminary')
  assert(selected.players[1].team_id==1 and selected.players[1].role=='top')
  assert(recipe.players[1].hero=='old' and selected.players[3].hero=='old')
  rejects(function() launch.select_heroes(recipe,{[3]='training_luminary'}) end)
  rejects(function() launch.select_heroes(recipe,{[4]='training_luminary'}) end)
  rejects(function() launch.select_heroes(recipe,{[1]='bad/name'}) end)
  for _,args in ipairs({{'--hero'},{'--hero','0=x'},{'--hero','4294967296=x'},
      {'--hero','1=x','--hero','01=y'},{'--hero','1=bad/name'}}) do
    rejects(function() launch.options(args) end)
  end
end)

test('fixed Lua + full TOML + production Rust configuration preflight',function()
  local options=launch.options({'--prepare-only','--output',path.join(root,'single')})
  plan=launch.prepare(options,process)
  assert(plan.human_count==1 and plan.bot_count==9 and plan.tick_rate_hz==60)
  assert(#plan.clients==1 and plan.clients[1].player_id==1 and plan.clients[1].team_id==1)
  assert(plan.scope=='prepared-not-launched' and plan.profile=='release')
  assert(path.read(base,true)==original,'source config changed')
  assert(contains(plan.clients[1].unreal.args,'-om-presentation-only'))
  assert(math.type(plan.worker_budget.unreal_cores_per_renderer)=='integer'
    and plan.worker_budget.unreal_cores_per_renderer>=1)
  assert(contains(plan.clients[1].unreal.args,'-corelimit='..plan.worker_budget.unreal_cores_per_renderer))
  assert(not contains(plan.clients[1].runtime.args,'--test-mode'))
  assert(plan.server.env.OMB_LUA_HOT_RELOAD=='0')
  assert(plan.content_mode=='compiled-content-only' and plan.server.env.OMB_LUA_CONTENT=='0')
  assert(plan.clients[1].runtime.env.OMB_LUA_CONTENT=='0' and plan.clients[1].unreal.env.OMB_LUA_CONTENT=='0')
  assert(path.read(plan.config):match('LUA_CONTENT%s*=%s*false'))
  assert(plan.clients[1].presentation=='127.0.0.1:57062')
  local locked=json.read(plan.selection_report)
  assert(locked.schema_version==1 and locked.ready and locked.finalized and #locked.seats==10)
  for _,seat in ipairs(locked.seats) do assert(seat.locked) end
  local catalog=json.read(plan.hero_catalog)
  assert(catalog.schema_version==1 and catalog.catalog_data_hash==locked.catalog_data_hash)
  local found=false
  for _,hero in ipairs(catalog.heroes) do if hero.hero=='training_ranger' then found=true;assert(#hero.abilities==4) end end
  assert(found,'compiled selectable hero missing')
  rejects(function() launch.prepare(options,process) end)
end)

test('hero selections flow into server plan; unknown catalog names fail before launch',function()
  local selected=launch.prepare(launch.options({'--prepare-only','--hero','1=training_ranger',
    '--output',path.join(root,'selected')}),process)
  assert(#selected.human_heroes==1 and selected.human_heroes[1].player_id==1)
  assert(selected.human_heroes[1].hero=='training_ranger')
  assert(path.read(selected.config):find('training_ranger',1,true))
  local saved=json.read(selected.recipe_json)
  assert(saved.players[1].hero=='training_ranger' and saved.players[1].bot==false)
  local replay=launch.prepare(launch.options({'--recipe',selected.recipe_json,'--hero','1=training_vanguard',
    '--output',path.join(root,'json-selected')}),process)
  assert(replay.human_heroes[1].hero=='training_vanguard')
  assert(json.read(selected.recipe_json).players[1].hero=='training_ranger','source JSON changed')
  rejects(function() launch.prepare(launch.options({'--hero','1=unknown_selection_hero',
    '--output',path.join(root,'unknown-hero')}),process) end)
  assert(path.read(base,true)==original)
end)
test('same-team humans use separate ordinal IPC ports, all-Bot recipe prepares without clients',function()
  local fixture=path.join(root,'same-team.lua')
  path.write(fixture,'local p=assert(loadfile('..json.encode(path.join(b.root,'scripts','lua_data','moba_role_match.lua'))..'))(); p.players[1].bot=false; p.players[2].bot=false; return p')
  local multi=launch.prepare(launch.options({'--recipe',fixture,'--output',path.join(root,'multi')}),process)
  assert(multi.human_count==2 and multi.bot_count==8)
  assert(multi.clients[1].team_id==1 and multi.clients[2].team_id==1)
  assert(multi.clients[1].presentation~=multi.clients[2].presentation)
  local bots=launch.prepare(launch.options({'--recipe','scripts/lua_data/moba_role_match.lua','--output',path.join(root,'bots')}),process)
  assert(bots.human_count==0 and bots.bot_count==10 and #bots.clients==0)
  rejects(function() launch.launch(bots,'unused',{}, {}) end)
end)
test('runtime readiness requires exact admitted identity, not listener or other team',function()
  local client=plan.clients[1]
  assert(not launch.runtime_ready('client-runtime presentation listening on 127.0.0.1:57062',client))
  assert(not launch.runtime_ready('client-runtime ready player_id=1 team_id=2 replica_tick=1',client))
  assert(not launch.runtime_ready('client-runtime ready player_id=11 team_id=1 replica_tick=1',client))
  assert(launch.runtime_ready('client-runtime ready player_id=1 team_id=1 replica_tick=1',client))
end)
test('original child cleanup is reverse order, waited, and failures propagated',function()
  local function execute(fail_stop)
    local copied=json.decode(json.encode(plan))
    copied.output=path.mkdir_p(path.join(root,fail_stop and 'cleanup-failure' or 'cleanup-success'))
    copied.server.exe=b.lib('platform').lua_executable
    copied.clients[1].runtime.exe=copied.server.exe
    local next_pid,stopped,waited=0,{},{}
    local fake={spawn=function(_,_,options)
      next_pid=next_pid+1
      path.write(options.stdout,'client-runtime ready player_id=1 team_id=1 replica_tick=1',true)
      return next_pid
    end,inspect=function(pid) return pid~=3 end,poll_ready=function(_,_,predicate) assert(predicate()) end,
      stop=function(pid) stopped[#stopped+1]=pid;if fail_stop and pid==2 then error('simulated stop failure') end end,
      wait=function(pid) waited[#waited+1]=pid;return true end}
    local fixture
    fake,fixture=require('tests.selection_owned_fixture').wrap(fake)
    local ok=pcall(launch.launch,copied,copied.server.exe,fake,{sleep_ms=function() error('unexpected wait') end})
    assert(ok~=fail_stop)
    assert(table.concat(stopped,',')=='2,1') -- renderer 3 already exited, never stop its reused PID.
    assert(table.concat(fixture.waited,',')==(fail_stop and '3,1' or '3,2,1'))
    assert(#waited==0) -- all original lifetimes already retired; no PID-only wait.
    local result=json.read(path.join(copied.output,'lifecycle.json'))
    assert(result.cleanup_succeeded~=fail_stop)
    if fail_stop then assert(path.read(path.join(copied.output,'errors.md')):find('simulated stop failure',1,true)) end
  end
  execute(false);execute(true)
end)
test('reused server/runtime/renderer and partial startup retain exact ownership',function()
  for _,mode in ipairs({'server-reused','runtime-reused','renderer-reused','partial-spawn'}) do
    local copied=json.decode(json.encode(plan))
    copied.output=path.mkdir_p(path.join(root,mode))
    copied.server.exe=b.lib('platform').lua_executable
    copied.clients[1].runtime.exe=copied.server.exe
    local serial,live,stops,legacy_waits=0,{},{},{}
    local fixture
    local old={spawn=function(_,_,options)
      serial=serial+1
      if mode=='partial-spawn' and serial==3 then error('injected renderer spawn failure') end
      live[serial]=true
      path.write(options.stdout,'client-runtime ready player_id=1 team_id=1 replica_tick=1',true)
      return serial
    end,inspect=function(pid)return live[pid] end,
      stop=function(pid)stops[#stops+1]=pid;live[pid]=false end,
      wait=function(pid)legacy_waits[#legacy_waits+1]=pid;return not live[pid] end,
      poll_ready=function(pid,_,predicate)
        if mode=='server-reused' then fixture.reuse(1) end
        if mode=='runtime-reused' then fixture.reuse(pid) end
        assert(predicate())
      end}
    local fake
    fake,fixture=require('tests.selection_owned_fixture').wrap(old)
    local ok,err=pcall(launch.launch,copied,copied.server.exe,fake,{sleep_ms=function(ms)
      assert(mode=='renderer-reused' and ms==500)
      fixture.reuse(3)
    end})
    assert(ok==(mode=='renderer-reused'),tostring(err))
    local expected=({['server-reused']='2',['runtime-reused']='1',['renderer-reused']='2,1',['partial-spawn']='2,1'})[mode]
    assert(table.concat(stops,',')==expected,'wrong owned cleanup for '..mode)
    assert(#legacy_waits==0,'PID-only wait reached new or retired process')
    local saved=json.read(path.join(copied.output,'owned-processes.json'))
    assert(#saved==(mode=='partial-spawn' and 2 or serial))
    local reused=({['server-reused']=1,['runtime-reused']=2,['renderer-reused']=3})[mode]
    if reused then
      assert(live[reused],'reused process was stopped')
      assert(saved[reused].creation_token~=fixture.current[reused].creation_token,'original identity overwritten')
    end
  end
end)

test('arguments fail closed; production Rust rejects invalid recipe',function()
  for _,args in ipairs({{'--port','65526'},{'--profile','fast'},{'--graphics','opengl'},{'--recipe'},{'--unknown'}}) do
    rejects(function() launch.options(args) end)
  end
  local fixture=path.join(root,'invalid.lua')
  path.write(fixture,'return {schema_version=1,map_id="unknown",think_hz=5,players={}}')
  rejects(function() launch.prepare(launch.options({'--recipe',fixture,'--output',path.join(root,'invalid')}),process) end)
  assert(path.read(base,true)==original)
end)
print(('role launch functionality: %d/%d passed; no server/runtime/Unreal processes launched'):format(tests,tests))
