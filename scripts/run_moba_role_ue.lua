local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,time,platform=b.lib('path'),b.lib('process'),b.lib('time'),b.lib('platform')
local launch=require('moba_role_launch')
local workflow=require('moba_launch_workflow')
if arg[1]=='--help' then
  print('Usage: tools/lua/lua.exe scripts/run_moba_role_ue.lua [--prepare-only | --interactive-selection] [--recipe FILE.lua|FILE.json] [--hero PLAYER_ID=HERO (repeatable)] [--config FILE.toml] [--output NEW_DIRECTORY] [--port 57061] [--profile release|debug] [--ue-root PATH] [--graphics d3d11|d3d12] [--story FOG_2TEAM_DEMO] [--no-build]')
  print('Interactive selection: 1..10 human seats from the recipe; multiple humans share one local host and each receive an Unreal window. Any cancellation aborts before gameplay.')
  print('LAN host: --server-bind UNICAST_IPV4 [--local-player ID (repeatable)]. Remote client: --connect HOST_IPV4 --recipe HOST_FINAL.json --local-player ID (repeatable). IPC stays loopback; no firewall changes.')
  print('Read-only bounded result capture: --finish-timeout-seconds 1..7200. Optional selection automation: --interactive-selection --selection-smoke-hero HERO (requires bounded result capture). Does not force a winner or validate the full match.')
  print('Selection completion: --selection-timeout-seconds 1..7200; covers service startup through final handoff, shared across all local seats. Automation defaults to 120 seconds; manual selection remains unbounded unless specified. Timeout cancels, never auto-locks.')
  return
end
local options=launch.options(arg)
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
local ok,err=xpcall(function()
assert(owns_output,'output must be a new directory: '..output)
if options.interactive_selection then
  assert(not path.exists(output..'-selection'),'selection output must be a new directory: '..output..'-selection')
end
local editor,ue_root
local function stage(mode)
  local result=process.run(platform.lua_executable,{path.join(b.root,'scripts','build_ue_moba.lua'),mode,'--ue-root',ue_root},{cwd=b.root})
  print(result.stdout)
end
workflow.execute(options,{
resolve_editor=function() editor,ue_root=launch.editor(options) end,
build_frontend=function() stage('--build-only') end,
build_runtime=function()
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
end,
verify_stage=function()
  require('ue_binary_preflight').require_ready(ue_root,path.join(b.root,'omfue','om.uproject'))
  stage('--verify-staged-only')
end,
select=function() launch.interactive_select(options,editor,process,time) end,
prepare=function()
  local plan=launch.prepare(options,process)
  print(('Prepared %s: %d local / %d total human(s), %d Bot(s), %d Hz, server %s: %s'):format(
    plan.mode,plan.local_human_count,plan.human_count,plan.bot_count,plan.tick_rate_hz,plan.server_address,plan.config))
  return plan
end,
launch=function(plan) launch.launch(plan,editor,process,time) end,
})
end,debug.traceback)
if not ok then
  -- Never write a failure record into a refused pre-existing output directory.
  if owns_output then
    path.append(path.join(output,'errors.md'),'\n# Role launcher failure\n\n'..tostring(err)..
      '\n\nDecision: fail closed; do not fallback to another build/profile or stop unrelated processes. See retained configuration and process logs.\n')
  end
  error(err,0)
end
