local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,time,platform=b.lib('path'),b.lib('process'),b.lib('time'),b.lib('platform')
local launch=require('moba_role_launch')
local workflow=require('moba_launch_workflow')
if arg[1]=='--help' then
  print('Usage: tools/lua/lua.exe scripts/run_moba_role_ue.lua [--prepare-only] [--interactive-selection | --server-only] [--recipe FILE.lua|FILE.json] [--hero PLAYER_ID=HERO (repeatable)] [--config FILE.toml] [--output NEW_DIRECTORY] [--port 57061] [--profile release|debug] [--ue-root PATH] [--graphics d3d11|d3d12] [--story FOG_2TEAM_DEMO] [--no-build]')
  print('Interactive selection: 1..10 human seats from the recipe; multiple humans share one local host and each receive an Unreal window. Any cancellation aborts before gameplay.')
  print('LAN host: --server-bind UNICAST_IPV4 [--local-player ID (repeatable)]. Remote client: --connect HOST_IPV4 --recipe HOST_FINAL.json --local-player ID (repeatable). IPC stays loopback; no firewall changes.')
  print('Dedicated Rust host: --server-only [--server-bind UNICAST_IPV4]; zero local clients, no Unreal build/install required. Share the generated match-plan.json with remote clients. Incompatible with --connect, --local-player, interactive selection and native result capture.')
  print('Input record: --accepted-inputs FILE.jsonl makes the authority write every accepted input per tick (replay with the replay_accepted_inputs soak).')
  print('Client autopilot (Development/Editor renderers): --autopilot PROFILE drives each local renderer through synthesized Slate input and writes OUTPUT/autopilot-pID.jsonl plus .coverage.json.')
  print('  --autopilot-plan FILE: directed steps besides the lane (clear a camp, cast at a tree, watch a respawn; scripts/autopilot_plan.lua, e.g. scripts/autopilot_plans/jungle_trees.lua).')
  print('Debug status: --debug-query sets OMB_DEBUG_QUERY=1 on the authority so omb-mcp (match_status, list_players) may read the unfiltered state; local development only, never for LAN players.')
  print('Same-machine split topology: pass identical --host-simulations N --host-renderers M to every --server-only/--connect invocation sharing one host (e.g. 3 and 2 for server + two clients) so the worker budget is shared instead of each assuming the whole machine. Presentation IPC binds 127.0.0.1:(--port + player ID).')
  print('Configuration preflight always uses the selected profile\'s compiled moba-config.exe, never cargo run. --prepare-only and --no-build require prebuilt artifacts; missing tools fail without building or switching profiles.')
  print('LAN selection: --interactive-selection --selection-bind HOST_IPV4 --local-player ID. Only local seats receive windows; send each other seat its private invitation and use scripts/run_moba_selection_join.lua on that machine. Final match-plan.json remains host-owned; do not share the invitation folder.')
  print('Read-only bounded result capture: --finish-timeout-seconds 1..7200. Optional selection automation: --interactive-selection --selection-smoke-hero HERO (requires bounded result capture). Does not force a winner or validate the full match.')
  print('Selection completion: --selection-timeout-seconds 1..7200; covers service startup through final handoff, shared across all local seats. Automation defaults to 120 seconds; manual selection remains unbounded unless specified. Timeout cancels, never auto-locks.')
  print('Cooperative selection cancellation: --selection-cancel-file PATH. Create that path to abort selection and retire only this session\'s original local processes. It is never read or removed; this is not crash/forced-termination supervision.')
  print('Actual execution (not help/prepare-only) is separately supervised by a private Windows Job: an outer Lua exit/crash retires this workflow\'s descendants. No name/PID-only cleanup or remote-host termination.')
  return
end
local options=launch.options(arg)
assert(not options.managed_rounds,'--managed-rounds is owned by the root managed session entry (scripts/run_moba_session_ue.lua)')
if not options.prepare_only then process.supervise_workflow(source,arg) end
if not options.output then
  -- Exclusive mkdir, no reuse/deletion of a previous session directory.
  local lfs=require('lfs')
  local parent=path.mkdir_p(path.join(b.root,'target','role-ue-runs'))
  for suffix=1,1000 do
    local reserved=path.join(parent,os.time()..'-'..suffix)
    if lfs.mkdir(reserved) then options.output=path.join(reserved,'session');break end
  end
  assert(options.output,'cannot reserve run directory')
end
local output=path.absolute(options.output,b.root)
local owns_output=not path.exists(output)
local build_lock -- held through the build phase (E301); released on success or failure
local ok,err=xpcall(function()
assert(owns_output,'output must be a new directory: '..output)
if options.interactive_selection then
  assert(not path.exists(output..'-selection'),'selection output must be a new directory: '..output..'-selection')
end
local editor,ue_root
-- The build phase (UE frontend, then authority, script DLLs and client runtime) runs under one
-- repo build lock (E301): content landed between the bridge build and the authority build gave
-- them different content hashes and the launch was refused. build_ue_moba joins it by token.
local function stage(mode)
  local result=process.run(platform.lua_executable,{path.join(b.root,'scripts','build_ue_moba.lua'),mode,'--ue-root',ue_root},
    {cwd=b.root,env=build_lock and build_lock:env() or nil})
  print(result.stdout)
end
return workflow.execute(options,{
build_server=function() require('moba_server_build').build(options,process) end,
verify_server=function() require('moba_server_build').verify(options) end,
resolve_editor=function() editor,ue_root=launch.editor(options) end,
build_frontend=function()
  build_lock=require('build_lock').acquire(b.root,'run_moba_role_ue build phase')
  stage('--build-only')
end,
build_runtime=function()
  -- Every binary this launch stages is built here, the script DLL set too, and checked against
  -- the sources as of this build (E287): the UE build before it takes minutes, other sessions land
  -- changes meanwhile, and a launch that only checked a set built earlier refused it as stale.
  require('run_artifacts').sources_as_of=os.time()
  local script_dlls=require('moba_server_build').script_dll_command(options.profile=='release' and 'release' or 'debug')
  print(process.run(script_dlls.exe,script_dlls.args,{cwd=b.root}).stdout)
  local builds={
    {'--manifest-path',path.join(b.root,'omb','Cargo.toml'),'-p','omobab','--bin','moba-config','--features','compiled-content-only'},
    {'--manifest-path',path.join(b.root,'omoba-client-runtime','Cargo.toml'),'--features','compiled-content-only'},
  }
  if not options.connect then
    table.insert(builds,2,{'--manifest-path',path.join(b.root,'omb','Cargo.toml'),'-p','omobab','--bin','omobab','--features','compiled-content-only'})
  end
  for _,build in ipairs(builds) do
    table.insert(build,1,'build')
    if options.profile=='release' then build[#build+1]='--release' end
    local result=process.run('cargo',build,{cwd=b.root})
    print(result.stdout)
  end
  if build_lock then build_lock:release();build_lock=nil end
end,
verify_stage=function()
  require('moba_frontend_preflight').verify(ue_root,process)
  if options.managed_session_bootstrap then
    local scope=require('moba_managed_session').catalog_scope(b.lib('json').decode(
      require('moba_config_command').run(options,{'--catalog-scope'},process).stdout))
    if options.managed_catalog_scope then
      for _,key in ipairs({'catalog_identity_hash','content_hash','map_catalog_hash'}) do
        assert(scope[key]==options.managed_catalog_scope[key],'compiled managed catalog changed during selection')
      end
    end
    options.managed_catalog_scope=scope
  end
end,
select=function() launch.interactive_select(options,editor,process,time) end,
prepare=function()
  local plan=launch.prepare(options,process)
  print(('Prepared %s: %d local / %d total human(s), %d Bot(s), %d Hz, server %s: %s'):format(
    plan.mode,plan.local_human_count,plan.human_count,plan.bot_count,plan.tick_rate_hz,plan.server_address,plan.config))
  return plan
end,
launch=function(plan)
  -- One live match per machine (E197); a split child joins the lock its parent holds.
  local lock <close> = require('machine_lock').acquire(b.root,'live_match',{run=plan.output,
    label='run_moba_role_ue '..plan.mode,wait_seconds=options.wait_live_lock or 0})
  launch.launch(plan,editor,process,time,lock)
end,
})
end,debug.traceback)
if build_lock then build_lock:release();build_lock=nil end
if not ok then
  -- Never write a failure record into a refused pre-existing output directory.
  if owns_output then
    path.append(path.join(output,'errors.md'),'\n# Role launcher failure\n\n'..tostring(err)..
      '\n\nDecision: fail closed; do not fallback to another build/profile or stop unrelated processes. See retained configuration and process logs.\n')
  end
  error(err,0)
end
return err
