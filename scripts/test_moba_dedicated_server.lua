local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json,process,platform=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('platform')
local launch,workflow=require('moba_role_launch'),require('moba_launch_workflow')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target/dedicated-server-tests'))
local root
for i=1,1000 do local file=path.join(parent,os.time()..'-'..i);if lfs.mkdir(file) then root=file;break end end
assert(root)
local count=0
local function test(name,fn) fn();count=count+1;print('PASS '..name) end
local function rejects(fn) assert(not pcall(fn),'expected rejection') end
test('explicit role and mutually exclusive render/client flags',function()
  assert(launch.options({'--server-only','--prepare-only'}).server_only)
  for _,extra in ipairs({{'--local-player','1'},{'--interactive-selection'},
    {'--connect','192.0.2.10','--recipe','host.json','--local-player','1'},
    {'--finish-timeout-seconds','5'},{'--selection-timeout-seconds','5'},
    {'--selection-smoke-hero','training_ranger'}}) do
    local args={'--server-only'};for _,arg in ipairs(extra) do args[#args+1]=arg end
    rejects(function() launch.options(args) end)
  end
end)
test('workflow omits all frontend callbacks, including no-build and prepare-only',function()
  for _,mode in ipairs({'build','no-build','prepare-only'}) do
    local calls={}
    local plan={mode='dedicated-server',clients={},server={}}
    local function step(name,value) return function() calls[#calls+1]=name;return value end end
    local steps={prepare=step('prepare',plan),build_server=step('build'),
      verify_server=step('verify'),launch=step('launch')}
    setmetatable(steps,{__index=function(_,name) error('unexpected frontend callback: '..name) end})
    assert(workflow.execute({server_only=true,no_build=mode=='no-build',prepare_only=mode=='prepare-only'},steps)==plan)
    local expected=mode=='build' and 'build,verify,prepare,verify,launch'
      or (mode=='no-build' and 'verify,prepare,verify,launch' or 'prepare')
    assert(table.concat(calls,',')==expected)
  end
end)
local plan
test('real Rust config preflight preserves roster, compiled-only and 60Hz without clients',function()
  plan=launch.prepare(launch.options({'--server-only','--prepare-only','--server-bind','192.0.2.10',
    '--output',path.join(root,'prepared')}),process)
  assert(plan.mode=='dedicated-server' and plan.server and #plan.clients==0 and plan.local_human_count==0)
  assert(plan.human_count==1 and plan.bot_count==9 and #plan.human_heroes==1)
  assert(plan.tick_rate_hz==60 and plan.content_mode=='compiled-content-only')
  assert(plan.worker_budget.simulation_processes==1 and plan.worker_budget.renderer_processes==0)
  assert(plan.server.env.OMB_LUA_CONTENT=='0' and plan.server.env.OMB_LUA_HOT_RELOAD=='0')
  assert(#json.read(plan.recipe_json).players==10)
end)
test('owned authority stays alive until original exits, one child only',function()
  local value=json.decode(json.encode(plan));value.output=path.mkdir_p(path.join(root,'lifecycle'))
  value.server.exe=platform.lua_executable
  local alive,spawns,sleeps=true,0,0
  local old={spawn=function() spawns=spawns+1;return 101 end,
    inspect=function() return alive end,stop=function() error('normal retired authority must not be stopped') end,
    wait=function() return not alive end}
  local api,fixture=require('tests/selection_owned_fixture').wrap(old)
  launch.launch(value,nil,api,{sleep_ms=function(ms) assert(ms==500);sleeps=sleeps+1;if sleeps==2 then alive=false end end})
  assert(spawns==1 and sleeps==2 and #fixture.stopped==0 and #fixture.waited==1)
  assert(#json.read(path.join(value.output,'owned-processes.json'))==1)
  local result=json.read(path.join(value.output,'lifecycle.json'))
  assert(result.launch_succeeded and result.cleanup_succeeded)
end)
test('interruption cleans only original authority and retains error',function()
  local value=json.decode(json.encode(plan));value.output=path.mkdir_p(path.join(root,'interrupted'))
  value.server.exe=platform.lua_executable
  local alive=true
  local old={spawn=function() return 102 end,inspect=function() return alive end,
    stop=function(pid) assert(pid==102);alive=false end,wait=function() return not alive end}
  local api,fixture=require('tests/selection_owned_fixture').wrap(old)
  rejects(function() launch.launch(value,nil,api,{sleep_ms=function() error('injected interruption') end}) end)
  assert(#fixture.stopped==1 and fixture.stopped[1]==102 and #fixture.waited==1)
  local result=json.read(path.join(value.output,'lifecycle.json'))
  assert(not result.launch_succeeded and result.cleanup_succeeded)
  assert(path.read(path.join(value.output,'errors.md')):find('injected interruption',1,true))
end)
test('reused PID is not stopped or treated as original authority',function()
  local value=json.decode(json.encode(plan));value.output=path.mkdir_p(path.join(root,'reused'))
  value.server.exe=platform.lua_executable
  local old={spawn=function() return 103 end,inspect=function() return true end,
    stop=function() error('must not stop replacement') end,wait=function() error('must not wait replacement') end}
  local api,fixture=require('tests/selection_owned_fixture').wrap(old)
  local sleeps=0
  launch.launch(value,nil,api,{sleep_ms=function() sleeps=sleeps+1;fixture.reuse(103) end})
  assert(sleeps==1 and #fixture.stopped==0 and #fixture.waited==1)
end)
test('invalid direct plans rejected before spawn',function()
  for _,change in ipairs({function(value) value.clients={{}} end,
    function(value) value.server=nil end,function(value) value.local_human_count=1 end,
    function(value) value.finish_timeout_seconds=5 end,function(value) value.mode='unknown' end}) do
    local value=json.decode(json.encode(plan));change(value)
    rejects(function() launch.launch(value,nil,{spawn_owned=function() error('must not spawn') end},{}) end)
  end
end)
print(('dedicated server: %d groups passed; native configuration only, injected lifecycle; no game/Unreal/LAN acceptance'):format(count))
