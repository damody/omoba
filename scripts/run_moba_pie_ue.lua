-- Same-world selection -> compiled authority/replica -> native result in PIE.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json,process,time,platform=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('time'),b.lib('platform')
local launch=require('moba_role_launch')
if #arg==1 and (arg[1]=='--help' or arg[1]=='-h') then
  print('Usage: tools/lua/lua.exe scripts/run_moba_pie_ue.lua --no-build --interactive-selection --recipe FILE --output NEW_DIRECTORY --finish-timeout-seconds 1..7200 [--selection-smoke-hero HERO] [--port PORT] [--ue-root PATH]')
  print('Uses prebuilt compiled artifacts and one local human. Owns a new Editor and one PIE world; never adopts another Editor. Manual selection requires explicit human confirmation; smoke automation is opt-in. No forced winner or gameplay Lua.')
  return
end
local options=launch.options(arg)
assert(options.no_build and options.interactive_selection and options.finish_timeout_seconds and options.output,
  'PIE flow requires explicit --no-build, --interactive-selection, bounded finish and new --output')
assert(not options.prepare_only and not options.server_only and not options.connect and not options.selection_bind,
  'PIE local flow cannot be prepare-only, dedicated, remote or shared selection')
process.supervise_workflow(source,arg)
local output=path.absolute(options.output,b.root)
assert(not path.exists(output),'PIE evidence directory must be new')
path.mkdir_p(path.parent(output))
assert(require('lfs').mkdir(output),'PIE evidence directory reservation failed')
local owned,editor_record,started,ordinal={},nil,false,0
local report={success=false,scope='single-local-human-same-PIE-selection-to-native-result',simulations_executed=0,
  full_lan_or_performance=false,results={}}
local function save() json.write(path.join(output,'report.json'),report) end
local function spawn(label,spec)
  local pid,record=process.spawn_owned(spec.exe,spec.args,{cwd=spec.cwd,env=spec.env,
    stdout=path.join(output,label..'.stdout.log'),stderr=path.join(output,label..'.stderr.log')})
  record=process.validate_owned(record);assert(record.pid==pid,'PIE spawn identity mismatch')
  owned[#owned+1]=record
  if label=='server' then report.simulations_executed=1 end
  json.write(path.join(output,'owned-processes.json'),owned,true)
  return record
end
local function mcp(name,args)
  assert(editor_record and process.owned_alive(editor_record),'original PIE Editor retired')
  ordinal=ordinal+1
  local file=path.join(output,('mcp-%03d-%s.json'):format(ordinal,name))
  local result=process.run(platform.lua_executable,{path.join(b.root,'scripts/ue_mcp.lua'),
    '--owned-editor',path.join(output,'owned-editor.json'),'--tool',name,
    '--arguments',json.encode(json.object(args)),'--out',file},{cwd=b.root,check=false})
  assert(result.exit_code==0,'owned PIE MCP failed: '..name..'\n'..result.stderr..result.stdout)
  local envelope=json.read(file);assert(not envelope.isError,'PIE MCP error: '..name)
  for _,item in ipairs(envelope.content or {}) do
    if item.type=='text' then
      local ok,value=pcall(json.decode,item.text)
      if ok and type(value)=='table' then
        assert(value.success~=false and value.ok~=false and value.verdict~='FAIL','PIE MCP negative result: '..name)
        return value
      end
    end
  end
  error('PIE MCP lacks structured result: '..name)
end
local function copy_options(out)
  local copy={};for k,v in pairs(options) do copy[k]=v end
  copy.output=out;copy.interactive_selection=false
  return copy
end
local ok,err=xpcall(function()
  local inspected=require('moba_selection_prepare').inspect(options)
  assert(#inspected.humans==1,'same-world PIE flow requires exactly one human in its recipe')
  require('moba_selection_placement').local_players(options,inspected.humans)
  local editor,ue_root=launch.editor(options)
  require('moba_frontend_preflight').verify(ue_root,process)
  path.mkdir_p(output)
  local selection_output=path.join(output,'selection')
  local candidate,preflight=require('moba_selection_prepare').prepare(options,process,selection_output,inspected)
  local initial=launch.prepare(copy_options(path.join(output,'preselection')),process)
  assert(#initial.clients==1 and initial.server,'PIE requires one local renderer and local authority')
  local human,client=inspected.humans[1],initial.clients[1]
  assert(client.player_id==human,'PIE local seat mismatch')
  report.player_id=human;report.team_id=client.team_id;report.tick_rate_hz=initial.tick_rate_hz
  report.profile=initial.profile;report.content_mode=initial.content_mode
  local receipt=path.join(selection_output,'finalized-reply.json')
  local args={}
  for _,value in ipairs(client.unreal.args) do
    if value~='-game' then args[#args+1]=value end
  end
  args[#args+1]='-om-hero-selection';args[#args+1]='-om-selection-in-place'
  args[#args+1]='-om-selection-exe='..inspected.exe
  args[#args+1]='-om-selection-plan='..candidate
  args[#args+1]='-om-selection-player='..human
  args[#args+1]='-om-selection-result='..receipt
  if options.selection_smoke_hero then args[#args+1]='-om-selection-smoke-hero='..options.selection_smoke_hero end
  -- All original gameplay/result arguments remain fixed before selection;
  -- only hero choice may change, verified by Rust and selection_result below.
  local spec={exe=editor,args=args,cwd=client.unreal.cwd,env=client.unreal.env}
  editor_record=spawn('editor',spec)
  json.write(path.join(output,'owned-editor.json'),editor_record)
  local endpoint_api=require('ue_mcp_endpoint')
  process.poll_owned_ready(editor_record,120000,function()
    local resolved,endpoint=pcall(endpoint_api.resolve,path.join(b.root,'omfue','om.uproject'))
    if resolved then endpoint_api.require_owned(endpoint,editor_record,process);return true end
    local reason=tostring(endpoint)
    assert(reason:find('found 0',1,true) or reason:find('registry unavailable',1,true),reason)
    return false
  end,'owned PIE Editor MCP readiness')
  mcp('enable_extension',{extension_id='Testing',enabled=true})
  local automation=mcp('get_automation_test_results',{})
  assert((automation.running or 0)==0,'automation active in owned Editor')
  assert(not mcp('get_pie_status',{}).pie_running,'fresh Editor already has PIE; never adopt it')
  mcp('begin_play_in_editor',{num_clients=1,net_mode='standalone'});started=true
  local budget=require('moba_selection_deadline')
  local token=budget.start(options,time)
  local readiness=require('moba_selection_readiness').reader(human,false)
  local ready=false
  local editor_log=path.join(output,'editor.stdout.log')
  local ready_deadline=time.monotonic_ms()+45000
  while not path.is_file(receipt) do
    budget.check(token,time)
    assert(process.owned_alive(editor_record),'Editor exited before finalized selection')
    ready=ready or readiness:poll(editor_log,false)
    assert(ready or time.monotonic_ms()<ready_deadline,'PIE selection UI/service handshake timed out')
    time.sleep_ms(250)
  end
  budget.check(token,time)
  while not ready do
    budget.check(token,time)
    assert(time.monotonic_ms()<ready_deadline,'PIE terminal readiness drain timed out')
    local caught_up
    ready,caught_up=readiness:poll(editor_log,false)
    assert(ready or not caught_up,'PIE terminal receipt lacks bound service readiness')
  end
  assert(path.attributes(receipt).size<=1024*1024,'PIE selection receipt exceeds limit')
  local final=launch.selection_result(json.read(receipt),preflight.plan,human,preflight.selection.catalog_data_hash)
  local recipe=path.join(selection_output,'match-plan.json');json.write(recipe,final)
  options.recipe=recipe;options.hero_selections={}
  local game=launch.prepare(copy_options(path.join(output,'game')),process)
  assert(game.clients[1].presentation==client.presentation and game.clients[1].team_id==client.team_id
    and game.clients[1].player_id==human and game.tick_rate_hz==60,'selected plan changed PIE runtime endpoint/identity/rate')
  report.selection_finalized=true;report.recipe=recipe
  -- Existing opt-in secure diagnostics keep only this team's filtered frames;
  -- they are not presentation data and never become an alternate protocol.
  report.evidence_directory=path.join(output,'evidence')
  game.server.env.OMOBA_FOG_EVIDENCE_DIR=report.evidence_directory
  game.clients[1].runtime.args[#game.clients[1].runtime.args+1]='--evidence-dir'
  game.clients[1].runtime.args[#game.clients[1].runtime.args+1]=report.evidence_directory
  local server=spawn('server',game.server);report.simulations_executed=1
  local runtime=spawn('runtime',game.clients[1].runtime)
  process.poll_owned_ready(runtime,45000,function()
    assert(process.owned_alive(server),'PIE authority exited before runtime readiness')
    local text=(path.is_file(path.join(output,'runtime.stdout.log')) and path.read(path.join(output,'runtime.stdout.log')) or '')
      ..(path.is_file(path.join(output,'runtime.stderr.log')) and path.read(path.join(output,'runtime.stderr.log')) or '')
    return launch.runtime_ready(text,game.clients[1])
  end,'PIE compiled player runtime')
  local actors=mcp('get_pie_actors',{})
  local controller
  for _,actor in ipairs(actors.actors or {}) do
    if actor.class=='OmPlayerController' or actor.class=='BP_PlayerController_C' then
      assert(not controller,'ambiguous local PIE player controller');controller=actor.name
    end
  end
  assert(controller,'PIE player controller not found')
  local handoff=mcp('call_pie_blueprint_function',{actor_label=controller,function_name='ContinueAfterHeroSelection'})
  assert(handoff.return_value=='True','native PIE presentation handoff refused')
  report.same_world_handoff=true;report.controller=controller
  local initial_picture=mcp('take_pie_screenshot',{file_path=path.join(output,'pie-gameplay.png')})
  assert(path.is_file(initial_picture.file_path),'PIE gameplay screenshot missing')
  report.gameplay_screenshot=initial_picture.file_path
  local observer=require('moba_role_finish_observer').reader(game.clients)
  local authority_log=require('moba_bounded_log').reader()
  local deadline=time.monotonic_ms()+options.finish_timeout_seconds*1000
  local observed_at
  -- The Editor's preselection argv intentionally has this exact screenshot path.
  local original_screenshot=path.join(initial.output,'result-p'..human..'.png')
  while true do
    assert(time.monotonic_ms()<deadline,'PIE natural match result timed out; never force a winner')
    for _,record in ipairs(owned) do assert(process.owned_alive(record),'owned PIE flow child retired early') end
    authority_log:poll(path.join(output,'server.stdout.log'),false,function(line)
      assert(not line:find('secure match terminated',1,true),
        'authority failed closed; preserve original diagnostic, never wait for or fabricate a victory: '..line)
    end)
    local result=observer:poll({[human]=editor_log})
    if result.complete then
      observed_at=observed_at or time.monotonic_ms()
      if path.is_file(original_screenshot) and time.monotonic_ms()>=observed_at+2000 then
        report.results=result;report.screenshot=original_screenshot
        break
      end
    end
    time.sleep_ms(500)
  end
  report.pie_result=mcp('get_pie_status',{})
  assert(report.pie_result.pie_running and report.pie_result.has_player,'result no longer belongs to live PIE world')
  report.success=true
end,debug.traceback)
if started and editor_record and process.owned_alive(editor_record) then
  local clean,reason=pcall(function() mcp('stop_play_in_editor',{}) end)
  if not clean then report.pie_cleanup_error=tostring(reason);ok=false end
end
local cleanup_errors={}
for i=#owned,1,-1 do
  local clean,reason=pcall(function()
    if process.owned_alive(owned[i]) then process.stop_owned(owned[i]) end
    assert(process.wait_owned(owned[i],5000),'original PIE child remained after stop')
  end)
  if not clean then cleanup_errors[#cleanup_errors+1]=tostring(reason) end
end
report.cleanup_verified=#cleanup_errors==0;report.cleanup_errors=cleanup_errors
report.success=ok and report.success and report.cleanup_verified
if not ok then report.error=tostring(err) end
-- Only this newly reserved output may receive error evidence.
path.mkdir_p(output);save()
if not report.success then
  path.append(path.join(output,'errors.md'),'# PIE selection flow failure\n\n'..tostring(err)..'\n\n'..
    table.concat(cleanup_errors,'\n')..'\n\nDecision: preserve all evidence and retire only original owned children; no forced winner, fallback or unrelated Editor cleanup.\n')
end
assert(report.success,'PIE selection-to-result did not complete: '..output)
print('Same-world PIE selection -> compiled gameplay -> native result PASS: '..output)
