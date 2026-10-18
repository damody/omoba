-- Actual selection-only renderer confirmation; never launches a gameplay authority.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json,process,time=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('time')
local launch=require('moba_role_launch')
if arg[1]=='--help' then
  print('Usage: tools/lua/lua.exe scripts/ue_selection_startup_validation.lua --output NEW_DIRECTORY [--ue-root PATH] [--graphics d3d11|d3d12] [--profile release|debug] [--editor-ready-timeout-seconds 1..3600] [--selection-timeout-seconds 1..7200] [--selection-smoke-hero HERO]')
  print('Selection-only actual renderer validation. No authority/client runtime or gameplay match is launched.')
  return
end
local allowed={['--output']=true,['--ue-root']=true,['--graphics']=true,['--profile']=true,
  ['--editor-ready-timeout-seconds']=true,['--selection-timeout-seconds']=true,['--selection-smoke-hero']=true}
local forwarded={'--interactive-selection'}
local smoke_hero='training_ranger'
local seen={};local i=1
while i<=#arg do
  local key=arg[i];assert(allowed[key] and not seen[key],'unknown or duplicate validation option: '..key)
  seen[key]=true;i=i+1;local value=assert(arg[i],'validation option requires a value')
  if key=='--selection-smoke-hero' then
    assert(value:match('^[%w_]+$'),'invalid selection smoke hero');smoke_hero=value
  else forwarded[#forwarded+1]=key;forwarded[#forwarded+1]=value end
  i=i+1
end
if not seen['--selection-timeout-seconds'] then
  forwarded[#forwarded+1]='--selection-timeout-seconds';forwarded[#forwarded+1]='3600'
end
local options=launch.options(forwarded)
-- This fixture calls only the bounded selection adapter, never the gameplay
-- launcher whose automation flag also requires a match-finish observer.
options.selection_smoke_hero=smoke_hero
local output=path.absolute(assert(options.output,'--output required'),b.root)
assert(not path.exists(output),'validation output must be new')
options.output=path.join(output,'match')
local selection=options.output..'-selection'
local editor,engine=launch.editor(options)
require('moba_frontend_preflight').verify(engine,process)
process.supervise_workflow(source,arg)
path.mkdir_p(path.parent(output))
assert(require('lfs').mkdir(output),'validation output reservation failed')
local report={success=false,scope='actual-single-selection-startup-confirmation',simulations_executed=0}
local started=time.monotonic_ms()
local ok,err=xpcall(function()
  launch.interactive_select(options,editor,process,time)
  assert(not path.exists(options.output),'selection unexpectedly created gameplay output')
  local plan=json.read(options.recipe)
  local chosen
  for _,player in ipairs(plan.players) do if player.player_id==1 then chosen=player.hero end end
  assert(chosen==options.selection_smoke_hero,'final plan hero differs from selected hero')
  local log=path.read(path.join(selection,'ue.log'))
  local count=0
  for kind,id in log:gmatch('OM_SELECTION_BUTTON player=1 kind=(%w+) pressed=1 released=1 request=(%d+) submitted_once=1') do
    count=count+1
    assert(kind==({'select','lock','finalize'})[count] and tonumber(id)==count,'unexpected native selection callback sequence')
  end
  assert(count==3,'expected exactly three native selection callbacks')
  assert(not log:find('Started bridge runtime:',1,true),'selection started gameplay bridge')
  local owned=process.validate_owned(json.read(path.join(selection,'owned-process.json')))
  assert(not process.owned_alive(owned),'original selection renderer remained after receipt')
  report.hero=chosen;report.native_pointer_callbacks=count;report.cleanup_verified=true
  report.final_plan=options.recipe;report.success=true
end,debug.traceback)
report.elapsed_ms=time.monotonic_ms()-started
if not ok then report.error=tostring(err) end
json.write(path.join(output,'startup-validation-report.json'),report)
if not ok then path.append(path.join(output,'errors.md'),'\n# Selection startup validation failure\n\n'..tostring(err)..'\n') end
assert(ok,err)
print('PASS actual selection startup, native select/lock/finalize, original renderer exit; no gameplay match: '..selection)
