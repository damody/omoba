-- Shared configuration/launch contract; no gameplay fixture inputs.
local source = debug.getinfo(1,'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path,json,toml = b.lib('path'),b.lib('json'),b.lib('toml')
local exporter = require('export_moba_role_plan')
local M = {}

function M.options(args)
  local options={port=57061,profile='release',graphics='d3d11',story='FOG_2TEAM_DEMO'}
  local values={['--recipe']='recipe',['--config']='config',['--output']='output',
    ['--port']='port',['--profile']='profile',['--ue-root']='ue_root',
    ['--graphics']='graphics',['--story']='story'}
  local i=1
  while i<=#args do
    local key=args[i]
    if key=='--prepare-only' then options.prepare_only=true
    elseif key=='--no-build' then options.no_build=true
    elseif values[key] then i=i+1;options[values[key]]=assert(args[i],key..' requires a value')
    else error('unknown argument: '..key) end
    i=i+1
  end
  options.port=assert(math.tointeger(tonumber(options.port)),'port must be an integer')
  assert(options.port>=1024 and options.port<=65525,'port must be 1024..65525')
  assert(options.profile=='release' or options.profile=='debug','profile must be release or debug')
  assert(options.graphics=='d3d11' or options.graphics=='d3d12','graphics must be d3d11 or d3d12')
  return options
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

function M.prepare(options,process)
  local recipe=path.absolute(options.recipe or 'scripts/lua_data/moba_single_player.lua',b.root)
  local config=path.absolute(options.config or 'omb/game.toml',b.root)
  local output=path.absolute(assert(options.output,'unique output directory required'),b.root)
  -- An existing directory is never recycled, including aliases such as ../.
  assert(not path.exists(output),'output must be a new directory: '..output)
  local value=assert(loadfile(recipe))()
  assert(type(value)=='table','recipe must return a table')
  local scripts=path.join(b.root,'scripts')
  local content=path.join(scripts,'lua_data')
  local fields=exporter.server_fields(value)..table.concat({
    'STEP_FPS=60','SPEED_MULT=1','SERVER_IP="127.0.0.1"',
    'SERVER_PORT='..json.encode(tostring(options.port)),
    'STORY='..json.encode(options.story),
    'SELECTIVE_LOCKSTEP_SHADOW=false','SELECTIVE_LOCKSTEP_DOGFOOD=false',
  },'\n')..'\n'
  local text=toml.update_sections(path.read(config),{
    {path={'server'},fields=fields},
    {path={'content'},fields=table.concat({
      'SCRIPTS_DIR='..json.encode(scripts),
      'DLL_PATH='..json.encode(path.join(scripts,'base_content.dll')),
      'LUA_CONTENT=true','LUA_HOT_RELOAD=false',
      'LUA_CONTENT_ROOT='..json.encode(content),'STORY_DATA_DIR='..json.encode(content),
    },'\n')..'\n'},
  })
  path.mkdir_p(output)
  local generated=path.join(output,'game.toml')
  path.write(generated,text)
  local checked=process.run('cargo',{'run','--quiet','--manifest-path',path.join(b.root,'omb','Cargo.toml'),
    '-p','omobab','--bin','moba-config','--','--config',generated},{cwd=b.root,label='role configuration preflight'})
  local report=json.decode(checked.stdout)
  assert(report.scope=='configuration-only' and report.tick_rate_hz==60,'unexpected preflight report')
  local plan={schema_version=1,scope='prepared-not-launched',config=generated,output=output,
    profile=options.profile,story=report.story,map_id=report.map_id,tick_rate_hz=60,
    base_recovery_enabled=report.base_recovery_enabled,
    mana_enabled=report.mana_enabled,
    human_count=#report.humans,bot_count=#report.bot_player_ids,clients={}}
  local env={OMB_GAME_TOML=generated,OMB_STORY=report.story,OMB_SCENE_PATH='',
    OMB_DLL_PATH=path.join(scripts,'base_content.dll'),OMB_SCRIPTS_DIR=scripts,
    OMB_LUA_CONTENT='1',OMB_LUA_HOT_RELOAD='0',OMB_LUA_CONTENT_ROOT=content,OMB_STORY_DATA_DIR=content,RUST_LOG='info'}
  plan.server={exe=path.join(b.root,'omb','target',options.profile,'omobab.exe'),args={},cwd=path.join(b.root,'omb'),env=env}
  for ordinal,human in ipairs(report.humans) do
    local id,team=tostring(human.player_id),tostring(human.team_id)
    local name='player'..id
    local address='127.0.0.1:'..(options.port+ordinal)
    local runtime_env={}
    for key,v in pairs(env) do runtime_env[key]=v end
    runtime_env.OMB_PLAYER_ID=id;runtime_env.OMB_TEAM_ID=team;runtime_env.OMB_PLAYER_NAME=name
    local runtime={exe=path.join(b.root,'omoba-client-runtime','target',options.profile,'omoba-client-runtime.exe'),
      cwd=b.root,env=runtime_env,args={'--player-id',id,'--team',team,'--player-name',name,
        '--server','127.0.0.1:'..options.port,'--presentation-bind',address,
        '--presentation-hz','60','--protocol-version','2'}}
    local ue={cwd=path.join(b.root,'omfue'),env={OM_RUNTIME_MODE='presentation-ipc',
      OM_PLAYER_ID=id,OM_PLAYER_NAME=name,OM_STORY=report.story,
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
    local server=spawn('server',plan.server)
    local editors={}
    for _,client in ipairs(plan.clients) do
      local runtime,out,errors=spawn('runtime-p'..client.player_id,client.runtime)
      process.poll_ready(runtime,45000,function()
        assert(process.inspect(server),'server exited before runtime ready')
        local text=(path.is_file(out) and path.read(out) or '')..(path.is_file(errors) and path.read(errors) or '')
        return M.runtime_ready(text,client)
      end,'runtime-p'..client.player_id)
      client.unreal.exe=editor
      editors[#editors+1]=spawn('unreal-p'..client.player_id,client.unreal)
    end
    print('Humans admitted; Unreal processes started. This is not renderer/full-match acceptance.')
    while true do
      local active=false
      for _,pid in ipairs(editors) do if process.inspect(pid) then active=true end end
      if not active then break end
      assert(process.inspect(server),'server exited during interactive match')
      for _,entry in ipairs(owned) do
        if entry.label:match('^runtime') then assert(process.inspect(entry.pid),entry.label..' exited during match') end
      end
      time.sleep(500)
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
