local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local planner=require('ue_engine_bootstrap')
local function opts(changes)
  local value={mode='build',requested=true,source_present=true,build_present=true,
    editor_present=false,parallel=4,project='D:/clone with spaces/omfue/om.uproject'}
  for key,item in pairs(changes or {}) do value[key]=item end
  return value
end
local function reject(value,pattern)
  local ok,err=pcall(planner.plan,value)
  assert(not ok and tostring(err):find(pattern,1,true),tostring(err))
end
local plan=planner.plan(opts())
assert(plan.initialize and plan.args[1]=='OmGameEditor' and plan.args[4]=='D:/clone with spaces/omfue/om.uproject')
assert(plan.args[#plan.args]=='-MaxParallelActions=4')
assert(plan.shader_args[1]=='ShaderCompileWorker' and plan.shader_args[#plan.shader_args]=='-MaxParallelActions=4')
for _,item in ipairs(plan.args) do assert(item~='-NoEngineChanges' and item~='-NoUBA') end
assert(not planner.plan(opts{editor_present=true,source_present=false}).initialize)
assert(planner.plan(opts{editor_present=true,initialization_pending=true}).initialize)
reject(opts{editor_present=true,initialization_pending=true,source_present=false},'requires engine source')
assert(not planner.plan(opts{requested=false,parallel=false,editor_present=true,initialization_pending=true}).initialize)
assert(planner.plan(opts{editor_present=true,refresh=true}).initialize)
local local_plan=planner.plan(opts{editor_present=true,refresh=true,build_executor='local'})
local function has(values,wanted)
  for _,value in ipairs(values) do if value==wanted then return true end end
  return false
end
assert(local_plan.initialize and has(local_plan.args,'-NoUBA') and has(local_plan.shader_args,'-NoUBA'))
assert(not has(local_plan.args,'-NoEngineChanges'), 'initialization is followed by the separate normal engine-change gate')
assert(has(local_plan.args,'-MaxParallelActions=4') and has(local_plan.shader_args,'-MaxParallelActions=4'))
local uncached=planner.plan(opts{editor_present=true,refresh=true,build_executor='local',disable_action_cache=true})
assert(has(uncached.args,'-nocache') and has(uncached.shader_args,'-nocache'))
reject(opts{disable_action_cache=true},'requires a local build')
reject(opts{mode='verify',build_executor='local',disable_action_cache=true},'requires a local build')
assert(not planner.plan(opts{editor_present=true,requested=false,parallel=false,build_executor='local'}).initialize)
reject(opts{build_executor='remote'},'invalid source-engine build executor')
reject(opts{editor_present=true,refresh=true,source_present=false},'requires engine source')
reject(opts{editor_present=true,refresh=true,build_present=false},'requires engine source')
reject(opts{editor_present=true,refresh=true,requested=false,parallel=false},'requires explicit source-engine build admission')
reject(opts{mode='verify',editor_present=true,refresh=true},'cannot be used with --verify-staged-only')
reject(opts{source_present=false},'requires engine source')
reject(opts{build_present=false},'requires engine source')
reject(opts{requested=false,parallel=false},'UnrealEditor.exe is missing')
reject(opts{requested=false},'requires --bootstrap-engine')
reject(opts{mode='verify'},'cannot be used with --verify-staged-only')
assert(not planner.plan{mode='verify',requested=false}.initialize)
for _,value in ipairs{'0','65','1.5','-1','four','1e1',''} do
  assert(not pcall(planner.parallel_actions,value),value)
end
assert(planner.parallel_actions('1')==1 and planner.parallel_actions('64')==64)
print('Source-engine admission and project command tests passed')
local project='D:/clone with spaces/omfue/om.uproject'
for _,parallel in ipairs{1,4,64}do
  local ordinary=planner.ordinary_build_args{project=project,parallel=parallel,disable_action_cache=true}
  assert(ordinary[4]=='-Project='..project, 'project must remain one structured argument')
  local flags={};for _,flag in ipairs(ordinary)do flags[flag]=true end
  assert(flags['-NoEngineChanges'] and flags['-NoUBA'] and flags['-nocache'])
  assert(flags['-MaxParallelActions='..parallel])
end
local defaults=planner.ordinary_build_args{project=project}
assert(#defaults==9 and has(defaults,'-NoUBTMakefiles') and has(plan.args,'-NoUBTMakefiles'), 'ordinary defaults must not add a resource cap or enable action cache')
for _,parallel in ipairs{0,65,1.5,-1}do
  assert(not pcall(planner.ordinary_build_args,{project=project,parallel=parallel}))
end
print('Ordinary NoEngineChanges resource-cap and structured project arguments passed')
for _,version in ipairs{'Latest','Preview','14.44.35207','14.44.35229.0'}do
  assert(planner.compiler_version(version)==version)
  local selected=planner.plan(opts{build_executor='local',compiler_version=version})
  local flag='-CompilerVersion='..version
  assert(has(selected.args,flag) and has(selected.shader_args,flag))
  local normal=planner.ordinary_build_args{project=project,compiler_version=version}
  assert(has(normal,flag) and has(normal,'-NoEngineChanges'))
end
for _,version in ipairs{'','14','14.44','14.44.35207.','latest','Latest -NoEngineChanges','../14.44'}do
  assert(not pcall(planner.compiler_version,version))
end
reject(opts{compiler_version='Latest'},'requires a local build')
reject(opts{compiler_version='Latest',build_executor='local',mode='verify'},'requires a local build')
print('Explicit compiler selection stays identical across bootstrap, shader prerequisite and ordinary gate')
