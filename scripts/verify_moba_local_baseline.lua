-- One full build + one fixed-workload acceptance; no retries/sample selection.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local process,path,json,platform=b.lib('process'),b.lib('path'),b.lib('json'),b.lib('platform')
assert(#arg==1 and arg[1]:match('^[%w_-]+$'),'usage: tools/lua/lua.exe scripts/verify_moba_local_baseline.lua UNIQUE_RUN_ID')
process.supervise_workflow(source,arg)
local id=arg[1]
local output=path.join(b.root,'target','local-baseline-verification',id)
local run=path.join(b.root,'target','interactive-runs',id)
assert(not path.exists(output) and not path.exists(run),'verification output/run must both be new')
-- Old launcher's session cleanup must not touch a different live run.
assert(not path.exists(path.join(b.root,'target','interactive-runs','active-ue-session.json')),
  'another UE session record exists; preserve it and do not clean it implicitly')
path.mkdir_p(output)
local function command(label,exe,argv,environment)
  print('Final baseline: '..label);io.stdout:flush()
  local result=process.run(exe,argv,{cwd=b.root,check=false,env=environment})
  path.write(path.join(output,label..'.stdout.log'),result.stdout or '')
  path.write(path.join(output,label..'.stderr.log'),result.stderr or '')
  assert(result.exit_code==0,label..' failed, retained logs: '..output)
  return result
end
local ok,err=xpcall(function()
  local editor,ue_root=require('moba_role_launch').editor({})
  command('frontend-build',platform.lua_executable,{path.join(b.root,'scripts/build_ue_moba.lua'),'--build-only','--ue-root',ue_root})
  for _,build in ipairs({
    {label='content',args={'build','--release','--manifest-path','scripts/Cargo.toml','-p','base_content','--features','compiled-content-only'}},
    {label='server',args={'build','--release','--manifest-path','omb/Cargo.toml','-p','omobab','--features','compiled-content-only'}},
    {label='client-runtime',args={'build','--release','--manifest-path','omoba-client-runtime/Cargo.toml','--features','compiled-content-only'}},
  }) do command('build-'..build.label, 'cargo',build.args) end
  local env={OMOBA_RUN_ID=id,OMOBA_UE_SMOKE_SECONDS='120',OMOBA_UE_STEP_FPS='60',
    OMOBA_SKIP_BUILD='1',OMOBA_SKIP_UE_BUILD='1',OMOBA_UE_RECONNECT_SMOKE='1',
    OMOBA_UE_RHI='d3d11',UE_ROOT=ue_root,UE_5_8_ROOT=ue_root,OMOBA_RELEASE='1',
    OMOBA_UE_ABILITY_SMOKE='0',OMOBA_UE_MATCH_SMOKE='0',OMOBA_UE_RESULT_UI_SMOKE='0',
    OMOBA_UE_SHOP_SMOKE='0',OMOBA_UE_SHOP_BUTTON_SMOKE='0',OMOBA_UE_MINIMAP_SMOKE='0',
    OMOBA_UE_MINIMAP_MOVE_SMOKE='0',OMOBA_UE_RECALL_SMOKE='0',OMOBA_UE_UPGRADE_SMOKE='0',
    OMOBA_UE_FIRST_LEARN_SMOKE='0',OMOBA_UE_FIRST_LEARN_CAST_SMOKE='0',
    OMOBA_UE_SCOREBOARD_SMOKE='0',OMOBA_UE_SCOREBOARD_DEATH_SMOKE='0'}
  command('live-reconnect',platform.lua_executable,{path.join(b.root,'scripts/run_2player_ue.lua'),'--single-lane'},env)
  command('saved-reconnect',platform.lua_executable,{path.join(b.root,'scripts/tests/ue_renderer_reconnect_acceptance.lua'),run})
  local live=json.read(path.join(run,'unreal-ipc-smoke-report.json'))
  assert(live.profile=='release' and live.tick_rate_hz==60 and live.content_mode=='compiled-content-only',
    'final capture switched profile/rate/content mode')
  local baseline=path.join(b.root,'docs/plans/baselines/moba-60hz-local-two-player-v1.json')
  local results,passed={},true
  local server_log=path.join(output,'server-combined.log')
  path.write(server_log,path.read(path.join(run,'logs','server.stdout.log'))..'\n'..
    path.read(path.join(run,'logs','server.stderr.log')))
  for player=1,2 do
    -- Team1's old and reconnected renderer logs are both part of this workload.
    local ue=path.read(path.join(run,'logs','ue-p'..player..'.stdout.log'))
    if player==1 then ue=ue..'\n'..path.read(path.join(run,'logs','ue-p1-reconnect.stdout.log')) end
    local joined=path.join(output,'ue-p'..player..'-combined.log');path.write(joined,ue)
    local code,report=require('moba_performance_report').main({
      '--server',server_log,
      '--runtime',path.join(run,'logs','runtime-p'..player..'.stderr.log'),
      '--unreal',joined,'--baseline',baseline,'--out',path.join(output,'performance-p'..player..'.json'),
    },{stdout=function() end})
    results[#results+1]={player_id=player,status=report.status,threshold_result=report.threshold_result,
      comparisons=report.thresholds,errors=report.errors,missing=report.missing}
    passed=passed and code==0 and report.threshold_result=='pass' and #report.thresholds==12
  end
  json.write(path.join(output,'report.json'),{success=passed,scope='fixed-local-two-player-60hz-reconnect-baseline',
    run=run,baseline=baseline,simulations_executed=1,full_ui_or_lan=false,players=results})
  assert(passed,'fixed baseline failed; retain both complete captures, never relax limits or retry for a passing sample')
end,debug.traceback)
if not ok then path.append(path.join(output,'errors.md'),'# Final baseline failure\n\n'..tostring(err)..'\n');error(err,0) end
print('Final fixed local baseline PASS: '..output)
