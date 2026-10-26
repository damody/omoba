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
    ['--selection-bind']='selection_bind',
    ['--finish-timeout-seconds']='finish_timeout_seconds',['--selection-smoke-hero']='selection_smoke_hero',
    ['--selection-timeout-seconds']='selection_timeout_seconds',
    ['--editor-ready-timeout-seconds']='editor_ready_timeout_seconds',
    ['--selection-cancel-file']='selection_cancel_file',
    ['--managed-session-bootstrap']='managed_session_bootstrap',
    ['--host-simulations']='host_simulations',['--host-renderers']='host_renderers',
    ['--autopilot']='autopilot',['--packaged-game']='packaged_game',['--resolution']='resolution',
    ['--accepted-inputs']='accepted_inputs',['--spotlight-script']='spotlight_script',
    ['--round-time-box-seconds']='round_time_box_seconds',['--managed-rounds']='managed_rounds',['--managed-final-round-seconds']='managed_final_round_seconds'}
  local i=1
  while i<=#args do
    local key=args[i]
    if key=='--prepare-only' then options.prepare_only=true
    elseif key=='--server-only' then options.server_only=true
    elseif key=='--interactive-selection' then options.interactive_selection=true
    elseif key=='--no-build' then options.no_build=true
    elseif key=='--debug-query' then options.debug_query=true
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
  -- Hero spotlight recording (record_hero_spotlight.lua): the autopilot's spotlight profile reads it.
  if options.spotlight_script then
    assert(options.autopilot=='spotlight','--spotlight-script requires --autopilot spotlight')
    assert(not options.server_only,'--spotlight-script belongs to a renderer seat')
  end
  if options.finish_timeout_seconds then
    local raw = options.finish_timeout_seconds
    local seconds = raw:match('^%d+$') and math.tointeger(tonumber(raw))
    assert(seconds and seconds >= 1 and seconds <= 7200, 'finish timeout must be 1..7200 whole seconds')
    assert(not options.prepare_only, 'finish observation cannot be prepare-only')
    options.finish_timeout_seconds = seconds
  end
  if options.round_time_box_seconds then
    -- Live-check time box: an unfinished round is INCONCLUSIVE and retires, never an unbounded wait.
    local raw=options.round_time_box_seconds
    local seconds=type(raw)=='string' and raw:match('^%d+$') and math.tointeger(tonumber(raw))
    assert(seconds and seconds>=1 and seconds<=7200,'round time box must be 1..7200 whole seconds')
    assert(not options.prepare_only,'a round time box needs a launched round')
    options.round_time_box_seconds=seconds
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
  if options.editor_ready_timeout_seconds==nil then
    options.editor_ready_timeout_seconds=1800
  else
    local raw=options.editor_ready_timeout_seconds
    local seconds=type(raw)=='string' and raw:match('^[1-9]%d*$') and math.tointeger(tonumber(raw))
    assert(seconds and seconds>=1 and seconds<=3600,
      'editor ready timeout must be a decimal integer from 1 to 3600 seconds')
    options.editor_ready_timeout_seconds=seconds
  end
  if options.selection_cancel_file then
    assert(options.interactive_selection,'selection cancel file requires interactive selection')
    assert(type(options.selection_cancel_file)=='string' and options.selection_cancel_file~='',
      'selection cancel file must be a nonempty path')
    options.selection_cancel_file=path.absolute(options.selection_cancel_file,b.root)
  end
  -- Same-machine totals across separate --server-only/--connect invocations; each
  -- invocation otherwise budgets as if it owned the whole host.
  for key,limit in pairs({host_simulations=11,host_renderers=10}) do
    local raw=options[key]
    if raw~=nil then
      local count=type(raw)=='string' and raw:match('^%d+$') and math.tointeger(tonumber(raw))
      assert(count and count>=(key=='host_simulations' and 1 or 0) and count<=limit,
        '--'..key:gsub('_','-')..' must be an integer within the host budget limits')
      options[key]=count
    end
  end
  if options.autopilot~=nil then
    -- Client-side rule autopilot (Development/Editor renderers only); never a server Bot.
    assert(type(options.autopilot)=='string' and options.autopilot:match('^[%w_%-]+$'),'--autopilot requires a profile name')
    assert(not options.server_only,'--autopilot drives local renderers; --server-only has none')
    assert(not options.interactive_selection,'--autopilot uses a locked recipe, not interactive selection')
  end
  network.validate(options)
  -- Bounded managed session (coordinator-owned): rounds 1..N-1 offer the next match; round N
  -- is an ordinary single match observed for --managed-final-round-seconds, then the session ends.
  if options.managed_rounds or options.managed_final_round_seconds then
    local rounds=tostring(options.managed_rounds or ''):match('^%d+$') and math.tointeger(tonumber(options.managed_rounds))
    local seconds=tostring(options.managed_final_round_seconds or ''):match('^%d+$') and math.tointeger(tonumber(options.managed_final_round_seconds))
    assert(rounds and rounds>=2 and rounds<=20,'--managed-rounds must be an integer from 2 to 20')
    assert(seconds and seconds>=1 and seconds<=7200,'--managed-final-round-seconds must be 1..7200 whole seconds')
    assert(not options.managed_session_bootstrap,'managed round limits are owned by the root managed session entry')
    options.managed_rounds,options.managed_final_round_seconds=rounds,seconds
  end
  if options.managed_session_bootstrap then
    assert(not options.prepare_only and not options.server_only and not options.connect
      and not options.finish_timeout_seconds and not next(options.local_players),
      'managed provider requires an unbounded host with all humans local')
    options.managed_session_bootstrap=path.absolute(options.managed_session_bootstrap,b.root)
  end
  return options
end

-- Cold owned-Editor startup only. Does not change selection or match budgets.
function M.await_owned_editor_ready(process_api, record, options, predicate)
  local seconds=options and options.editor_ready_timeout_seconds
  assert(math.type(seconds)=='integer' and seconds>=1 and seconds<=3600,
    'editor ready timeout must be a decimal integer from 1 to 3600 seconds')
  local timeout_ms=seconds*1000
  assert(math.type(timeout_ms)=='integer','editor ready timeout overflow')
  return process_api.poll_owned_ready(record, timeout_ms, predicate, 'owned PIE Editor MCP readiness')
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

-- Absent and empty host maps are the same unavailable state. Any named row stays exact.
function M.omit_empty_summoner_loadouts(plan)
  local copy=json.decode(json.encode(plan))
  local map=copy.summoner_loadouts
  assert(map==nil or (map~=json.null and type(map)=='table'),'summoner loadouts must be a map')
  if map==nil or next(map)==nil then copy.summoner_loadouts=nil end
  return copy
end

function M.assert_snapshot_loadouts(snapshot,plan)
  local function norm(value)
    if value==nil then return {} end
    assert(value~=json.null and type(value)=='table','summoner loadouts must be a map')
    if next(value)==nil then return {} end
    return value
  end
  assert(json.encode(norm(snapshot and snapshot.summoner_loadouts))==json.encode(norm(plan and plan.summoner_loadouts)),
    'selection snapshot summoner loadouts differ from the final plan')
end

-- Renderer window size WIDTHxHEIGHT (default 1280x720), for the window/resolution matrix.
function M.resolution(options)
  local value=options.resolution or '1280x720'
  local width,height=value:match('^(%d+)x(%d+)$')
  width,height=tonumber(width),tonumber(height)
  assert(width and height and width>=640 and height>=480 and width<=7680 and height<=4320,
    '--resolution must be WIDTHxHEIGHT between 640x480 and 7680x4320: '..tostring(value))
  return {width,height}
end

-- A packaged Win64 archive: the archive directory, its Windows directory, or Windows/OmGame.exe.
function M.packaged_game(value)
  local candidate=path.absolute(value)
  if path.is_directory(candidate) then
    local direct=path.join(candidate,'OmGame.exe')
    candidate=path.is_file(direct) and direct or path.join(candidate,'Windows','OmGame.exe')
  end
  assert(path.is_file(candidate) and candidate:match('OmGame%.exe$'),'--packaged-game needs a packaged archive or OmGame.exe: '..value)
  return candidate
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
  M.assert_snapshot_loadouts(reply.selection,final)
  local hero
  for _,p in ipairs(final.players or {}) do if p.player_id==player_id then hero=p.hero end end
  assert(hero,'selected human missing from final plan')
  local expected=M.omit_empty_summoner_loadouts(M.select_heroes(baseline,{[player_id]=hero}))
  local actual=M.omit_empty_summoner_loadouts(final)
  table.sort(expected.players,function(a,b) return a.player_id<b.player_id end)
  table.sort(actual.players,function(a,b) return a.player_id<b.player_id end)
  assert(json.encode(actual)==json.encode(expected),'selection changed host-owned match rules or roster')
  return final
end

function M.interactive_select(options,editor,process,time)
  local output=path.absolute(assert(options.output),b.root)..'-selection'
  assert(not path.exists(output),'selection output must be a new directory: '..output)
  local preparation=require('moba_selection_prepare')
  local inspected=preparation.inspect(options)
  local humans=inspected.humans
  require('moba_selection_placement').local_players(options,humans) -- Validate seats before single/shared dispatch.
  local exe=inspected.exe
  local candidate,preflight=preparation.prepare(options,process,output,inspected)
  if #humans>1 or options.selection_bind then
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
  local owned
  local ok,err=xpcall(function()
    local budget=require('moba_selection_deadline')
    local completion_deadline=budget.start(options,time)
    local pid,record=process.spawn_owned(editor,args,{cwd=path.join(b.root,'omfue'),
      env={OMB_LUA_CONTENT='0',OMB_LUA_HOT_RELOAD='0'},
      stdout=path.join(output,'ue.stdout.log'),stderr=path.join(output,'ue.stderr.log')})
    owned=process.validate_owned(record)
    assert(owned.pid==pid,'selection spawn identity mismatch')
    path.write(path.join(output,'owned-process.json'),json.encode(owned))
    -- No server/client World exists yet. Closing the window cancels selection.
    local deadline=require('moba_selection_readiness').deadline(options,completion_deadline,time)
    local ready=false
    local readiness=require('moba_selection_readiness').reader(humans[1],false)
    local function read_ready(final)
      local caught_up
      repeat
        budget.check(completion_deadline,time)
        assert(time.monotonic_ms()<deadline,'selection UI/service handshake timeout')
        ready,caught_up=readiness:poll(path.join(output,'ue.log'),final)
        budget.check(completion_deadline,time)
        assert(time.monotonic_ms()<deadline,'selection UI/service handshake timeout')
      until not final or ready or caught_up
      return ready
    end
    while process.owned_alive(owned) do
      budget.check(completion_deadline,time)
      ready=ready or read_ready(false)
      assert(ready or time.monotonic_ms()<deadline,'selection UI/service did not become ready; do not use an old Unreal binary')
      time.sleep_ms(250)
    end
    assert(process.wait_owned(owned,budget.wait_ms(completion_deadline,time,5000)),'selection renderer has not exited')
    budget.check(completion_deadline,time)
    assert(ready or read_ready(true),'selection renderer exited without the bound service handshake')
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
      if owned then
        if process.owned_alive(owned) then process.stop_owned(owned) end
        assert(process.wait_owned(owned,5000),'selection process remained after stop')
      end
    end)
    path.write(path.join(output,'errors.md'),'# Selection handoff error\n\n'..tostring(err)..
      (cleaned and '' or '\n\nCleanup error: '..tostring(cleanup_err))..
      '\n\nDecision: do not start a match or auto-lock after cancellation; retain artifacts.\n')
    error(tostring(err)..(cleaned and '' or '\nCleanup error: '..tostring(cleanup_err)),0)
  end
end

-- Unreal parses several switches with FParse::Value, a case-insensitive
-- substring match over the whole command line. A project switch containing one
-- of these tokens silently configures the engine too: '-om-...-seconds=120'
-- set MaxTickTime and quit every renderer after 120 s (errors.md E029).
M.ENGINE_SUBSTRING_TOKENS={'seconds=','benchmark','fps=','deterministic','fixedtimestep'}
function M.assert_engine_safe_args(args)
  for _,value in ipairs(args) do
    local lower=tostring(value):lower()
    if lower:match('^%-om%-') then
      for _,token in ipairs(M.ENGINE_SUBSTRING_TOKENS) do
        assert(not lower:find(token,1,true),('project switch %s contains engine token %s'):format(value,token))
      end
    end
  end
  return true
end

-- Derived from the player ID, not the per-invocation ordinal, so separate
-- same-machine --connect clients never bind the same loopback port.
-- Zero-based position of an authored lane in the map template; renderer route ids follow it.
function M.authored_lane_index(root,map_id,lane)
  local ok,maps=pcall(function()
    return dofile(path.join(root,'scripts','lua_data','templates','moba_maps.lua'))({})
  end)
  if not ok or type(maps)~='table' then return nil end
  for _,map in ipairs(maps) do
    if map.id==map_id then
      for index,entry in ipairs(map.lanes or {}) do
        if entry.id==lane then return index-1 end
      end
    end
  end
  return nil
end

function M.presentation_address(port,player_id)
  assert(math.type(player_id)=='integer' and player_id>0,'presentation player ID must be a positive integer')
  local presentation_port=port+player_id
  assert(presentation_port<=65535,'presentation port for player '..player_id..' exceeds 65535; use a lower --port')
  return '127.0.0.1:'..presentation_port
end

function M.prepare(options,process)
  local config=path.absolute(options.config or 'omb/game.toml',b.root)
  local output=path.absolute(assert(options.output,'unique output directory required'),b.root)
  assert(not path.exists(output),'output must be a new directory: '..output)
  local config_command=require('moba_config_command')
  config_command.resolve(options) -- Fail before creating artifacts; never build or switch profiles.
  local value=M.load_recipe(options)
  path.mkdir_p(output)
  local recipe_json=path.join(output,'match-plan.json')
  local candidate_json=path.join(output,'selection-candidate.json')
  path.write(candidate_json,json.encode(value))
  local prepared=config_command.run(options,{'--lock-plan',candidate_json},process)
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
  local checked=config_command.run(options,{'--config',generated},process)
  local report=json.decode(checked.stdout)
  assert(report.scope=='configuration-only' and report.tick_rate_hz==60,'unexpected preflight report')
  if options.managed_session_bootstrap then
    local scope=assert(options.managed_catalog_scope,'managed catalog must be checked before selection')
    for _,key in ipairs({'catalog_identity_hash','content_hash','map_catalog_hash'}) do
      assert(report[key]==scope[key],'prepared managed catalog differs from preselection scope')
    end
    assert(locked.selection.catalog_data_hash==scope.content_hash,'selection content differs from managed catalog')
  end
  local local_humans=options.server_only and {} or network.local_humans(report.humans,options.local_players)
  local own_simulations,own_renderers=#local_humans+(options.connect and 0 or 1),#local_humans
  local host_simulations=options.host_simulations or own_simulations
  local host_renderers=options.host_renderers or own_renderers
  assert(host_simulations>=own_simulations and host_renderers>=own_renderers,
    'host process totals cannot be smaller than this invocation ('..own_simulations..' simulation, '..own_renderers..' renderer)')
  local worker_budget=require('moba_host_budget').local_budget(host_simulations,host_renderers)
  local server_address=(options.connect or options.server_bind)..':'..options.port
  path.write(recipe_json,json.encode(value))
  local plan={schema_version=1,scope='prepared-not-launched',config=generated,output=output,
    catalog_identity_hash=report.catalog_identity_hash,content_hash=report.content_hash,map_catalog_hash=report.map_catalog_hash,
    managed_session_bootstrap=options.managed_session_bootstrap,
    worker_budget=worker_budget,
    finish_timeout_seconds=options.finish_timeout_seconds,
    round_time_box_seconds=options.round_time_box_seconds,
    mode=options.server_only and 'dedicated-server' or (options.connect and 'remote-client' or 'host'),server_address=server_address,
    local_human_count=#local_humans,
    profile=options.profile,graphics=options.graphics,shader_warmup=not options.packaged_game,
    packaged_game=options.packaged_game and M.packaged_game(options.packaged_game) or nil,content_mode='compiled-content-only',recipe_json=recipe_json,selection_report=selection_report,
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
  -- Unfiltered authority queries for local debug tools (omb-mcp); off unless asked.
  if options.debug_query then env.OMB_DEBUG_QUERY='1' end
  -- Replayable record of every accepted input (E086): server/accepted-inputs.jsonl.
  if options.accepted_inputs then env.OMOBA_ACCEPTED_INPUTS=path.absolute(options.accepted_inputs,b.root) end
  for key,value in pairs(worker_budget.env) do env[key]=value end
  if not options.connect then
    plan.server={exe=path.join(b.root,'omb','target',options.profile,'omobab.exe'),args={},cwd=path.join(b.root,'omb'),env=env}
  end
  for _,human in ipairs(local_humans) do
    local id,team=tostring(human.player_id),tostring(human.team_id)
    local name='player'..id
    local address=M.presentation_address(options.port,human.player_id)
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
      '-'..options.graphics,'-noraytracing','-windowed','-ResX='..M.resolution(options)[1],'-ResY='..M.resolution(options)[2],
      '-nosplash','-nop4','-NoLiveCoding','-AllowMultipleInstances',
      '-sessionname=omfue-role-p'..id,'-UserDir='..path.join(output,'ue-p'..id),
      '-abslog='..path.join(output,'ue-p'..id..'.log'),'-stdout','-FullStdOutLogOutput',
      '-ExecCmds=t.MaxFPS 60'}}
    if options.packaged_game then
      -- A cooked game has no project argument and is already a game target.
      local kept={}
      for index,value in ipairs(ue.args) do
        if index>1 and value~='-game' then kept[#kept+1]=value end
      end
      ue.args=kept;ue.cwd=path.parent(M.packaged_game(options.packaged_game))
    end
    for _,hint in ipairs(worker_budget.unreal_args or {}) do ue.args[#ue.args+1]=hint end
    -- Match loading screen (presentation only): the locked seats and hero names this renderer joins.
    ue.args[#ue.args+1]='-om-loading-selection='..selection_report
    ue.args[#ue.args+1]='-om-loading-catalog='..hero_catalog
    -- (project switches are appended below and checked once the list is complete)
    if options.autopilot then
      ue.args[#ue.args+1]='-om-autopilot='..options.autopilot
      ue.args[#ue.args+1]='-om-autopilot-trace='..path.join(output,'autopilot-p'..id..'.jsonl')
      ue.args[#ue.args+1]='-om-autopilot-shot-interval=120'
      -- Evidence: one window capture while the match loading screen is on screen.
      ue.args[#ue.args+1]='-om-loading-screenshot='..path.join(output,'loading-p'..id..'.png')
      if options.spotlight_script then
        ue.args[#ue.args+1]='-om-spotlight-script='..path.absolute(options.spotlight_script,b.root)
        ue.args[#ue.args+1]='-om-spotlight-frames='..path.join(output,'spotlight-frames-p'..id)
        ue.args[#ue.args+1]='-om-spotlight-rate=30'
      end
      -- The virtual player plays the lane its seat was assigned in the locked plan.
      -- Renderer routes are named route_<i> in authored lane order, so pass both.
      for _,player in ipairs(value.players or {}) do
        if player.player_id==human.player_id and type(player.lane)=='string' and player.lane:match('^[%w_]+$') then
          ue.args[#ue.args+1]='-om-autopilot-lane='..player.lane
          local index=M.authored_lane_index(b.root,report.map_id,player.lane)
          if index then ue.args[#ue.args+1]='-om-autopilot-lane-index='..index end
        end
      end
    end
    if options.finish_timeout_seconds then
      -- Only logs an actually visible Finished panel and requests a screenshot.
      -- Unlike -om-match-smoke, this does not issue gameplay inputs.
      ue.args[#ue.args+1]='-om-result-ui-smoke'
      ue.args[#ue.args+1]='-om-result-ui-screenshot='..path.join(output,'result-p'..id..'.png')
    end
    M.assert_engine_safe_args(ue.args)
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
  local dedicated=plan.mode=='dedicated-server'
  assert(plan.mode=='host' or plan.mode=='remote-client' or dedicated,'unknown launch mode')
  if dedicated then
    assert(#plan.clients==0 and plan.local_human_count==0 and plan.server~=nil
      and not plan.finish_timeout_seconds,'invalid dedicated-server plan')
  else
    assert(#plan.clients>0,'interactive launch requires at least one human; use headless for all-Bot recipes')
  end
  assert((plan.mode=='remote-client' and plan.server==nil)
    or (plan.mode~='remote-client' and plan.server~=nil),'launch mode/server ownership mismatch')
  local owned={}
  local control_api=require('moba_managed_control')
  local session_process=require('moba_session_process')
  local managed=plan.managed_session_bootstrap and control_api.install(plan,plan.managed_session_bootstrap)
  local function journal(record,label)
    record=process.validate_owned(record)
    record.label=label
    owned[#owned+1]=record
    path.write(path.join(plan.output,'owned-processes.json'),json.encode(owned),true)
  end
  local function spawn(label,spec)
    assert(path.is_file(spec.exe),'missing executable: '..spec.exe)
    local stdout=path.join(plan.output,label..'.stdout.log')
    local stderr=path.join(plan.output,label..'.stderr.log')
    local pid,record=process.spawn_owned(spec.exe,spec.args,{cwd=spec.cwd,env=spec.env,stdout=stdout,stderr=stderr})
    record=process.validate_owned(record)
    assert(record.pid==pid,'match spawn identity mismatch')
    journal(record,label)
    return record,stdout,stderr
  end
  local timed_out=false
  local ok,err=xpcall(function()
    local server=plan.server and spawn('server',plan.server) or nil
    if dedicated then
      print('Dedicated Rust authority started; no local renderer. This is not LAN/full-match acceptance.')
      while process.owned_alive(server) do time.sleep_ms(500) end
      return
    end
    -- Before any runtime: a cold DDC would stall every renderer at frame 0 (E003).
    local sample=plan.clients[1].unreal
    if plan.shader_warmup then require('ue_shader_warmup').ensure({editor=editor,project=sample.args[1],graphics=plan.graphics or 'd3d11',
      sample_args=sample.args,cwd=sample.cwd,env=sample.env,output=plan.output,journal=journal},process,time) end
    local editors={}
    for _,client in ipairs(plan.clients) do
      local runtime,out,errors=spawn('runtime-p'..client.player_id,client.runtime)
      process.poll_owned_ready(runtime,45000,function()
        assert(not server or process.owned_alive(server),'server exited before runtime ready')
        local text=(path.is_file(out) and path.read(out) or '')..(path.is_file(errors) and path.read(errors) or '')
        return M.runtime_ready(text,client)
      end,'runtime-p'..client.player_id)
      client.unreal.exe=plan.packaged_game or editor
      if managed then
        local label='unreal-p'..client.player_id
        local spec={}
        for key,value in pairs(client.unreal) do spec[key]=value end
        spec.stdout=path.join(plan.output,label..'.stdout.log')
        spec.stderr=path.join(plan.output,label..'.stderr.log')
        local entry=session_process.spawn(label,spec,path.join(plan.output,'process-'..client.player_id),process,time,journal)
        entry.player_id=client.player_id
        editors[#editors+1]=entry
      else
        editors[#editors+1]=spawn('unreal-p'..client.player_id,client.unreal)
      end
    end
    print('Humans admitted; Unreal processes started. This is not renderer/full-match acceptance.')
    local finish_deadline = plan.finish_timeout_seconds and time.monotonic_ms() + plan.finish_timeout_seconds * 1000
    local finish_seen_at, finish_observation
    local finish_reader=finish_deadline and require('moba_role_finish_observer').reader(plan.clients)
    local finish_logs={}
    local round_deadline=plan.round_time_box_seconds and time.monotonic_ms()+plan.round_time_box_seconds*1000
    for _,client in ipairs(plan.clients) do
      finish_logs[client.player_id]=path.join(plan.output,'unreal-p'..client.player_id..'.stdout.log')
    end
    while true do
      local active=false
      if round_deadline and time.monotonic_ms()>=round_deadline then
        timed_out=true
        path.write(path.join(plan.output,'round-time-box.json'),json.encode({schema='omoba-round-time-box/1',
          outcome='time_box_expired',seconds=plan.round_time_box_seconds,reason='round did not finish within its time box; the managed coordinator decides inconclusive vs planned end'}))
        print('Round time box of '..plan.round_time_box_seconds..' s expired; retiring this round (a managed non-final round is INCONCLUSIVE).')
        break
      end
      if managed then control_api.poll(managed,time.monotonic_ms()) end
      for _,record in ipairs(editors) do
        if managed then
          if not record.exit_recorded then
            local alive,receipt=session_process.observe(record,process)
            if alive then active=true else
              control_api.record_exit(managed,record.player_id,receipt.exit_code,time.monotonic_ms())
              record.exit_recorded=true
              assert(receipt.exit_code==0,'managed Unreal failed with exit '..receipt.exit_code)
            end
          end
        elseif process.owned_alive(record) then active=true end
      end
      if not active then
        assert(not finish_deadline, 'renderers exited before the bounded result observation completed')
        break
      end
      assert(not server or process.owned_alive(server),'server exited during interactive match')
      for _,entry in ipairs(owned) do
        if entry.label:match('^runtime') then assert(process.owned_alive(entry),entry.label..' exited during match') end
      end
      if finish_deadline then
        local function check_finish_deadline()
          assert(time.monotonic_ms()<finish_deadline,'bounded match result observation timed out; no forced winner')
        end
        check_finish_deadline()
        local observation = finish_reader:poll(finish_logs)
        check_finish_deadline()
        if observation.complete then
          finish_seen_at = finish_seen_at or time.monotonic_ms()
          finish_observation = observation
          local screenshots = true
          for _, client in ipairs(plan.clients) do
            screenshots = screenshots and path.is_file(path.join(plan.output,'result-p'..client.player_id..'.png'))
          end
          if screenshots and time.monotonic_ms() >= finish_seen_at + 2000 then
            check_finish_deadline()
            path.write(path.join(plan.output,'native-result-observation.json'),json.encode(finish_observation))
            break
          end
        end
        check_finish_deadline()
      end
      time.sleep_ms(500)
    end
  end,debug.traceback)
  local failures={}
  for i=#owned,1,-1 do
    local entry=owned[i]
    local cleaned,reason=pcall(function()
      if process.owned_alive(entry) then process.stop_owned(entry) end
      assert(process.wait_owned(entry,5000),'original child remained after stop: '..entry.pid)
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
  plan.round_timed_out=timed_out or nil
  -- A timed-out round never offers the next match.
  if managed and not timed_out then
    plan.managed_next=control_api.finalize(managed,true,true,time.monotonic_ms())
  end
end
return M
