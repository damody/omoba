-- Shared configuration/launch contract; no gameplay fixture inputs.
local source = debug.getinfo(1,'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path,json,toml = b.lib('path'),b.lib('json'),b.lib('toml')
local exporter = require('export_moba_role_plan')
local network = require('moba_network_launch')
local M = {}

function M.options(args)
  local options={port=57061,profile='release',graphics='d3d11',story='FOG_2TEAM_DEMO',hero_selections={},local_players={}}
  local values={['--recipe']='recipe',['--config']='config',['--output']='output',
    ['--port']='port',['--profile']='profile',['--ue-root']='ue_root',
    ['--graphics']='graphics',['--story']='story',['--server-bind']='server_bind',['--connect']='connect',
    ['--finish-timeout-seconds']='finish_timeout_seconds',['--selection-smoke-hero']='selection_smoke_hero',
    ['--selection-timeout-seconds']='selection_timeout_seconds'}
  local i=1
  while i<=#args do
    local key=args[i]
    if key=='--prepare-only' then options.prepare_only=true
    elseif key=='--interactive-selection' then options.interactive_selection=true
    elseif key=='--no-build' then options.no_build=true
    elseif key=='--local-player' then
      i=i+1
      local raw=assert(args[i],'--local-player requires a player ID')
      local id=raw:match('^%d+$') and math.tointeger(tonumber(raw))
      assert(id and id>0 and id<=4294967295,'local player must be a nonzero u32')
      assert(not options.local_players[id],'duplicate local player: '..id)
      options.local_players[id]=true
    elseif key=='--hero' then
      i=i+1
      local selection=assert(args[i],'--hero requires PLAYER_ID=HERO')
      local id,hero=selection:match('^(%d+)=([%w_]+)$')
      id=id and math.tointeger(tonumber(id))
      assert(id and id>0 and id<=4294967295,'--hero requires a nonzero u32 PLAYER_ID=HERO')
      assert(not options.hero_selections[id],'duplicate hero selection for player '..id)
      options.hero_selections[id]=hero
    elseif values[key] then i=i+1;options[values[key]]=assert(args[i],key..' requires a value')
    else error('unknown argument: '..key) end
    i=i+1
  end
  options.port=assert(math.tointeger(tonumber(options.port)),'port must be an integer')
  assert(options.port>=1024 and options.port<=65525,'port must be 1024..65525')
  assert(options.profile=='release' or options.profile=='debug','profile must be release or debug')
  assert(options.graphics=='d3d11' or options.graphics=='d3d12','graphics must be d3d11 or d3d12')
  assert(not (options.prepare_only and options.interactive_selection),'interactive selection cannot be prepare-only')
  if options.finish_timeout_seconds then
    local raw = options.finish_timeout_seconds
    local seconds = raw:match('^%d+$') and math.tointeger(tonumber(raw))
    assert(seconds and seconds >= 1 and seconds <= 7200, 'finish timeout must be 1..7200 whole seconds')
    assert(not options.prepare_only, 'finish observation cannot be prepare-only')
    options.finish_timeout_seconds = seconds
  end
  if options.selection_smoke_hero then
    assert(options.interactive_selection and options.finish_timeout_seconds,
      'selection automation requires interactive selection and a bounded finish observer')
    assert(options.selection_smoke_hero:match('^[%w_]+$'), 'invalid selection smoke hero')
  end
  if options.selection_timeout_seconds then
    local raw=options.selection_timeout_seconds
    local seconds=raw:match('^%d+$') and math.tointeger(tonumber(raw))
    assert(seconds and seconds>=1 and seconds<=7200,'selection timeout must be 1..7200 whole seconds')
    assert(options.interactive_selection,'selection timeout requires interactive selection')
    options.selection_timeout_seconds=seconds
  end
  network.validate(options)
  return options
end

-- Shared pre-match adapter for CLI and future UI. Never mutates the recipe.
-- Catalog/role-plan validation stays with the production Rust preflight.
function M.select_heroes(value,selections)
  local selected=json.decode(json.encode(value))
  local players={}
  for _,player in ipairs(assert(selected.players,'recipe requires players')) do
    assert(not players[player.player_id],'duplicate recipe player ID')
    players[player.player_id]=player
  end
  for id,hero in pairs(selections or {}) do
    assert(math.type(id)=='integer' and id>0 and id<=4294967295,'invalid selection player ID')
    assert(type(hero)=='string' and hero:match('^[%w_]+$'),'invalid selection hero name')
    local player=assert(players[id],'selection player is not in recipe: '..id)
    assert(player.bot==false,'hero selection requires a declared human: '..id)
    player.hero=hero
  end
  return selected
end

function M.editor(options)
  local candidates={options.ue_root or os.getenv('UE_5_8_ROOT') or os.getenv('UE_ROOT') or os.getenv('UE_5_7_ROOT')}
  if not candidates[1] then
    candidates={'D:/UE5.8','D:/UE_5.8','C:/Program Files/Epic Games/UE_5.8',
      'D:/UE5.7','D:/UE_5.7','C:/Program Files/Epic Games/UE_5.7'}
  end
  for _,root in ipairs(candidates) do
    local exe=path.join(root,'Engine','Binaries','Win64','UnrealEditor.exe')
    if path.is_file(exe) then return path.absolute(exe),path.absolute(root) end
  end
  error('UnrealEditor.exe not found; specify --ue-root')
end

function M.load_recipe(options)
  local recipe=path.absolute(options.recipe or 'scripts/lua_data/moba_single_player.lua',b.root)
  local value
  if recipe:lower():match('%.json$') then
    value=json.read(recipe)
  else
    assert(recipe:lower():match('%.lua$'),'recipe must be Lua authoring or generated JSON')
    value=assert(loadfile(recipe))()
  end
  assert(type(value)=='table','recipe must return a table')
  return M.select_heroes(value,options.hero_selections)
end

function M.selection_result(reply,baseline,player_id,hash)
  assert(reply.protocol_version==1 and reply.admitted_player_id==player_id,
    'selection protocol or bound identity mismatch')
  assert(reply.error==nil or reply.error==json.null,'selection service returned an error')
  assert(reply.selection and reply.selection.ready==true and reply.selection.finalized==true,
    'selection did not finalize')
  assert(reply.selection.catalog_data_hash==hash,'selection content hash mismatch')
  local final=assert(reply.plan,'selection result lacks final plan')
  local hero
  for _,p in ipairs(final.players or {}) do if p.player_id==player_id then hero=p.hero end end
  assert(hero,'selected human missing from final plan')
  local expected=M.select_heroes(baseline,{[player_id]=hero})
  local actual=json.decode(json.encode(final))
  table.sort(expected.players,function(a,b) return a.player_id<b.player_id end)
  table.sort(actual.players,function(a,b) return a.player_id<b.player_id end)
  assert(json.encode(actual)==json.encode(expected),'selection changed host-owned match rules or roster')
  return final
end

function M.interactive_select(options,editor,process,time)
  local output=path.absolute(assert(options.output),b.root)..'-selection'
  assert(not path.exists(output),'selection output must be a new directory: '..output)
  local value=M.load_recipe(options)
  local humans={}
  for _,p in ipairs(value.players) do if p.bot==false then humans[#humans+1]=p.player_id end end
  assert(#humans>=1 and #humans<=10,'interactive selection requires 1..10 human seats')
  table.sort(humans)
  local exe=path.join(b.root,'omb','target',options.profile,'moba-config.exe')
  assert(path.is_file(exe),'selection executable missing: '..exe)
  path.mkdir_p(output)
  local candidate=path.join(output,'candidate.json')
  path.write(candidate,json.encode(value))
  -- This validates a copy only; it is not used as consent or the UI state.
  local preflight=json.decode(process.run(exe,{'--lock-plan',candidate},
    {cwd=b.root,env={OMB_LUA_CONTENT='0',OMB_LUA_HOT_RELOAD='0'}}).stdout)
  assert(preflight.scope=='host-prepared-selection','invalid selection preflight')
  if #humans>1 then
    return require('moba_shared_selection').run(options,editor,process,time,output,candidate,exe,humans,preflight)
  end
  local result=path.join(output,'finalized-reply.json')
  local args={path.join(b.root,'omfue','om.uproject'),'/Game/Map/Main','-game',
    '-om-hero-selection','-om-presentation-only','-om-selection-exe='..exe,'-om-selection-plan='..candidate,
    '-om-selection-player='..humans[1],'-om-selection-result='..result,
    '-'..options.graphics,'-noraytracing','-windowed','-ResX=1280','-ResY=720',
    '-nosplash','-nop4','-NoLiveCoding','-AllowMultipleInstances',
    '-UserDir='..path.join(output,'ue-user'),'-abslog='..path.join(output,'ue.log')}
  if options.selection_smoke_hero then
    args[#args+1]='-om-selection-smoke-hero='..options.selection_smoke_hero
  end
  local pid
  local ok,err=xpcall(function()
    local budget=require('moba_selection_deadline')
    local completion_deadline=budget.start(options,time)
    pid=process.spawn(editor,args,{cwd=path.join(b.root,'omfue'),
      env={OMB_LUA_CONTENT='0',OMB_LUA_HOT_RELOAD='0'},
      stdout=path.join(output,'ue.stdout.log'),stderr=path.join(output,'ue.stderr.log')})
    path.write(path.join(output,'owned-process.json'),json.encode({pid=pid,exe=editor}))
    -- No server/client World exists yet. Closing the window cancels selection.
    local deadline=time.monotonic_ms()+45000
    local ready=false
    local function read_ready()
      local log=path.join(output,'ue.log')
      return path.is_file(log) and path.read(log):find(
        ('OM_SELECTION_READY player=%d protocol=1'):format(humans[1]),1,true)~=nil
    end
    while process.inspect(pid) do
      budget.check(completion_deadline,time)
      ready=ready or read_ready()
      assert(ready or time.monotonic_ms()<deadline,'selection UI/service did not become ready; do not use an old Unreal binary')
      time.sleep_ms(250)
    end
    assert(process.wait(pid,budget.wait_ms(completion_deadline,time,5000)),'selection renderer has not exited')
    budget.check(completion_deadline,time)
    assert(ready or read_ready(),'selection renderer exited without the bound service handshake')
    assert(path.is_file(result),'selection cancelled or renderer failed; no match started')
    local final=M.selection_result(json.read(result),preflight.plan,humans[1],preflight.selection.catalog_data_hash)
    budget.check(completion_deadline,time)
    local final_path=path.join(output,'match-plan.json')
    path.write(final_path,json.encode(final))
    options.recipe=final_path
    options.hero_selections={} -- Do not override the explicit UI choice again.
  end,debug.traceback)
  if not ok then
    local cleaned,cleanup_err=pcall(function()
      if pid and process.inspect(pid) then
        process.stop(pid,editor)
        assert(process.wait(pid,5000),'selection process remained after stop')
      end
    end)
    path.write(path.join(output,'errors.md'),'# Selection handoff error\n\n'..tostring(err)..
      (cleaned and '' or '\n\nCleanup error: '..tostring(cleanup_err))..
      '\n\nDecision: do not start a match or auto-lock after cancellation; retain artifacts.\n')
    error(tostring(err)..(cleaned and '' or '\nCleanup error: '..tostring(cleanup_err)),0)
  end
end

function M.prepare(options,process)
  local config=path.absolute(options.config or 'omb/game.toml',b.root)
  local output=path.absolute(assert(options.output,'unique output directory required'),b.root)
  assert(not path.exists(output),'output must be a new directory: '..output)
  local value=M.load_recipe(options)
  path.mkdir_p(output)
  local recipe_json=path.join(output,'match-plan.json')
  local candidate_json=path.join(output,'selection-candidate.json')
  path.write(candidate_json,json.encode(value))
  local prepared=process.run('cargo',{'run','--quiet','--manifest-path',path.join(b.root,'omb','Cargo.toml'),
    '-p','omobab','--bin','moba-config','--features','compiled-content-only','--','--lock-plan',candidate_json},
    {cwd=b.root,env={OMB_LUA_CONTENT='0',OMB_LUA_HOT_RELOAD='0'},label='host hero selection lock'})
  local locked=json.decode(prepared.stdout)
  assert(locked.scope=='host-prepared-selection' and locked.selection.finalized and locked.selection.ready,
    'unexpected hero selection lock report')
  value=locked.plan
  local selection_report=path.join(output,'selection-lock.json')
  path.write(selection_report,json.encode(locked.selection))
  local hero_catalog=path.join(output,'hero-catalog.json')
  path.write(hero_catalog,json.encode({schema_version=1,catalog_data_hash=locked.selection.catalog_data_hash,
    heroes=locked.hero_catalog}))
  local scripts=path.join(b.root,'scripts')
  local content=path.join(scripts,'lua_data')
  local fields=exporter.server_fields(value)..table.concat({
    'STEP_FPS=60','SPEED_MULT=1','SERVER_IP='..json.encode(options.server_bind),
    'SERVER_PORT='..json.encode(tostring(options.port)),
    'STORY='..json.encode(options.story),
    'SELECTIVE_LOCKSTEP_SHADOW=false','SELECTIVE_LOCKSTEP_DOGFOOD=false',
  },'\n')..'\n'
  local text=toml.update_sections(path.read(config),{
    {path={'server'},fields=fields},
    {path={'content'},fields=table.concat({
      'SCRIPTS_DIR='..json.encode(scripts),
      'DLL_PATH='..json.encode(path.join(scripts,'base_content.dll')),
      'LUA_CONTENT=false','LUA_HOT_RELOAD=false',
      'LUA_CONTENT_ROOT='..json.encode(content),'STORY_DATA_DIR='..json.encode(content),
    },'\n')..'\n'},
  })
  local generated=path.join(output,'game.toml')
  path.write(generated,text)
  local checked=process.run('cargo',{'run','--quiet','--manifest-path',path.join(b.root,'omb','Cargo.toml'),
    '-p','omobab','--bin','moba-config','--features','compiled-content-only','--','--config',generated},
    {cwd=b.root,env={OMB_LUA_CONTENT='0',OMB_LUA_HOT_RELOAD='0'},label='role configuration preflight'})
  local report=json.decode(checked.stdout)
  assert(report.scope=='configuration-only' and report.tick_rate_hz==60,'unexpected preflight report')
  local local_humans=network.local_humans(report.humans,options.local_players)
  local worker_budget=require('moba_host_budget').local_budget(#local_humans+(options.connect and 0 or 1),#local_humans)
  local server_address=(options.connect or options.server_bind)..':'..options.port
  path.write(recipe_json,json.encode(value))
  local plan={schema_version=1,scope='prepared-not-launched',config=generated,output=output,
    worker_budget=worker_budget,
    finish_timeout_seconds=options.finish_timeout_seconds,
    mode=options.connect and 'remote-client' or 'host',server_address=server_address,
    local_human_count=#local_humans,
    profile=options.profile,content_mode='compiled-content-only',recipe_json=recipe_json,selection_report=selection_report,
    hero_catalog=hero_catalog,
    story=report.story,map_id=report.map_id,tick_rate_hz=60,
    base_recovery_enabled=report.base_recovery_enabled,
    mana_enabled=report.mana_enabled,
    human_count=#report.humans,bot_count=#report.bot_player_ids,clients={},human_heroes={}}
  for _,player in ipairs(value.players) do
    if player.bot==false then
      plan.human_heroes[#plan.human_heroes+1]={player_id=player.player_id,hero=player.hero}
    end
  end
  table.sort(plan.human_heroes,function(a,b) return a.player_id<b.player_id end)
  local env={OMB_GAME_TOML=generated,OMB_STORY=report.story,OMB_SCENE_PATH='',
    OMB_DLL_PATH=path.join(scripts,'base_content.dll'),OMB_SCRIPTS_DIR=scripts,
    OMB_LUA_CONTENT='0',OMB_LUA_HOT_RELOAD='0',OMB_LUA_CONTENT_ROOT='',OMB_STORY_DATA_DIR='',RUST_LOG='info'}
  for key,value in pairs(worker_budget.env) do env[key]=value end
  if not options.connect then
    plan.server={exe=path.join(b.root,'omb','target',options.profile,'omobab.exe'),args={},cwd=path.join(b.root,'omb'),env=env}
  end
  for ordinal,human in ipairs(local_humans) do
    local id,team=tostring(human.player_id),tostring(human.team_id)
    local name='player'..id
    local address='127.0.0.1:'..(options.port+ordinal)
    local runtime_env={}
    for key,v in pairs(env) do runtime_env[key]=v end
    runtime_env.OMB_PLAYER_ID=id;runtime_env.OMB_TEAM_ID=team;runtime_env.OMB_PLAYER_NAME=name
    local runtime={exe=path.join(b.root,'omoba-client-runtime','target',options.profile,'omoba-client-runtime.exe'),
      cwd=b.root,env=runtime_env,args={'--player-id',id,'--team',team,'--player-name',name,
        '--server',server_address,'--presentation-bind',address,
        '--presentation-hz','60',}}
    local ue={cwd=path.join(b.root,'omfue'),env={OM_RUNTIME_MODE='presentation-ipc',
      OM_PLAYER_ID=id,OM_PLAYER_NAME=name,OM_STORY=report.story,
      OMB_LUA_CONTENT='0',OMB_LUA_HOT_RELOAD='0',OMB_LUA_CONTENT_ROOT='',OMB_STORY_DATA_DIR='',
      OM_PRESENTATION_ADDRESS=address,OMFX_PRESENTATION_ADDR=address,
      OMB_PLAYER_ID=id,OMB_TEAM_ID=team,OMB_PLAYER_NAME=name},args={
      path.join(b.root,'omfue','om.uproject'),'/Game/Map/Main','-game','-om-networked',
      '-om-presentation-only','-om-native-content','-om-player='..id,'-om-team='..team,
      '-om-player-name='..name,'-om-story='..report.story,'-om-presentation='..address,
      '-'..options.graphics,'-noraytracing','-windowed','-ResX=1280','-ResY=720',
      '-nosplash','-nop4','-NoLiveCoding','-AllowMultipleInstances',
      '-sessionname=omfue-role-p'..id,'-UserDir='..path.join(output,'ue-p'..id),
      '-abslog='..path.join(output,'ue-p'..id..'.log'),'-stdout','-FullStdOutLogOutput',
      '-ExecCmds=t.MaxFPS 60'}}
    if options.finish_timeout_seconds then
      -- Only logs an actually visible Finished panel and requests a screenshot.
      -- Unlike -om-match-smoke, this does not issue gameplay inputs.
      ue.args[#ue.args+1]='-om-result-ui-smoke'
      ue.args[#ue.args+1]='-om-result-ui-screenshot='..path.join(output,'result-p'..id..'.png')
    end
    plan.clients[#plan.clients+1]={player_id=human.player_id,team_id=human.team_id,
      presentation=address,runtime=runtime,unreal=ue}
  end
  path.write(path.join(output,'launch-plan.json'),json.encode(plan))
  return plan
end

function M.runtime_ready(text,client)
  -- The earlier listener line does not prove KCP admission or team replica readiness.
  return text:find(('client-runtime ready player_id=%d team_id=%d '):format(client.player_id,client.team_id),1,true)~=nil
end

function M.launch(plan,editor,process,time)
  assert(#plan.clients>0,'interactive launch requires at least one human; use headless for all-Bot recipes')
  assert((plan.mode=='remote-client' and plan.server==nil)
    or (plan.mode~='remote-client' and plan.server~=nil),'launch mode/server ownership mismatch')
  local owned={}
  local function spawn(label,spec)
    assert(path.is_file(spec.exe),'missing executable: '..spec.exe)
    local stdout=path.join(plan.output,label..'.stdout.log')
    local stderr=path.join(plan.output,label..'.stderr.log')
    local pid=process.spawn(spec.exe,spec.args,{cwd=spec.cwd,env=spec.env,stdout=stdout,stderr=stderr})
    owned[#owned+1]={pid=pid,exe=spec.exe,label=label}
    -- Persist after every spawn, including a partial startup that later fails.
    path.write(path.join(plan.output,'owned-processes.json'),json.encode(owned),true)
    return pid,stdout,stderr
  end
  local ok,err=xpcall(function()
    local server=plan.server and spawn('server',plan.server) or nil
    local editors={}
    for _,client in ipairs(plan.clients) do
      local runtime,out,errors=spawn('runtime-p'..client.player_id,client.runtime)
      process.poll_ready(runtime,45000,function()
        assert(not server or process.inspect(server),'server exited before runtime ready')
        local text=(path.is_file(out) and path.read(out) or '')..(path.is_file(errors) and path.read(errors) or '')
        return M.runtime_ready(text,client)
      end,'runtime-p'..client.player_id)
      client.unreal.exe=editor
      editors[#editors+1]=spawn('unreal-p'..client.player_id,client.unreal)
    end
    print('Humans admitted; Unreal processes started. This is not renderer/full-match acceptance.')
    local finish_deadline = plan.finish_timeout_seconds and time.monotonic_ms() + plan.finish_timeout_seconds * 1000
    local finish_seen_at, finish_observation
    local finish_reader=finish_deadline and require('moba_role_finish_observer').reader(plan.clients)
    local finish_logs={}
    for _,client in ipairs(plan.clients) do
      finish_logs[client.player_id]=path.join(plan.output,'unreal-p'..client.player_id..'.stdout.log')
    end
    while true do
      local active=false
      for _,pid in ipairs(editors) do if process.inspect(pid) then active=true end end
      if not active then
        assert(not finish_deadline, 'renderers exited before the bounded result observation completed')
        break
      end
      assert(not server or process.inspect(server),'server exited during interactive match')
      for _,entry in ipairs(owned) do
        if entry.label:match('^runtime') then assert(process.inspect(entry.pid),entry.label..' exited during match') end
      end
      if finish_deadline then
        local observation = finish_reader:poll(finish_logs)
        if observation.complete then
          finish_seen_at = finish_seen_at or time.monotonic_ms()
          finish_observation = observation
          local screenshots = true
          for _, client in ipairs(plan.clients) do
            screenshots = screenshots and path.is_file(path.join(plan.output,'result-p'..client.player_id..'.png'))
          end
          if screenshots and time.monotonic_ms() >= finish_seen_at + 2000 then
            path.write(path.join(plan.output,'native-result-observation.json'),json.encode(finish_observation))
            break
          end
        end
        assert(time.monotonic_ms() < finish_deadline, 'bounded match result observation timed out; no forced winner')
      end
      time.sleep_ms(500)
    end
  end,debug.traceback)
  local failures={}
  for i=#owned,1,-1 do
    local entry=owned[i]
    local cleaned,reason=pcall(function()
      process.stop(entry.pid,entry.exe)
      assert(process.wait(entry.pid,5000),'PID remained after stop: '..entry.pid)
    end)
    if not cleaned then failures[#failures+1]=tostring(reason) end
  end
  path.write(path.join(plan.output,'lifecycle.json'),json.encode({scope='process-lifecycle-only',
    launch_succeeded=ok,cleanup_succeeded=#failures==0,cleanup_errors=failures,error=ok and '' or err}))
  if not ok or #failures>0 then
    path.write(path.join(plan.output,'errors.md'),'# Role launch error\n\n'..
      (ok and '' or tostring(err))..'\n\nCleanup errors:\n'..table.concat(failures,'\n')..
      '\n\nDecision: stop only PIDs created by this run, verify executable identity, and wait for exit. Do not relaunch automatically.\n',true)
  end
  assert(#failures==0,table.concat(failures,'\n'))
  assert(ok,err)
end
return M
