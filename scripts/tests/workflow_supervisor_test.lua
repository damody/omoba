local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]tests[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local process,path,json,time,platform=b.lib('process'),b.lib('path'),b.lib('json'),b.lib('time'),b.lib('platform')
local root=path.absolute(assert(arg[1],'new fixture output required'),b.root)
assert(not path.exists(root),'fixture output must be new')
path.mkdir_p(root)
local fixture=path.join(b.root,'scripts/tests/workflow_supervisor_fixture.lua')
local tracked={}
local function wait_exit(owned)
  assert(process.wait_owned(owned,10000),'original fixture did not retire: '..owned.pid)
end
local ok,err=xpcall(function()
  for _,mode in ipairs({'normal','crash'}) do
    local output=path.mkdir_p(path.join(root,mode))
    local argv={fixture,mode,output,'','a "quoted" b','trailing\\'}
    local _,owner=process.spawn_owned(platform.lua_executable,argv,{cwd=b.root,
      stdout=path.join(output,'owner.stdout.log'),stderr=path.join(output,'owner.stderr.log')})
    tracked[#tracked+1]=owner
    if mode=='normal' then
      -- Readiness is a durable fixture receipt, not a running-service gate.
      assert(time.poll(15000,20,function() return path.exists(path.join(output,'tree-ready')) end),'normal tree did not start')
    else
      assert(process.poll_owned_ready(owner,15000,function() return path.exists(path.join(output,'tree-ready')) end,mode))
    end
    local child=json.read(path.join(output,'child.json'));local leaf=json.read(path.join(output,'leaf.json'))
    tracked[#tracked+1]=child;tracked[#tracked+1]=leaf
    if mode=='crash' then
      assert(process.owned_alive(child) and process.owned_alive(leaf),'fixture exited before fault')
      assert(process.stop_owned(owner),'cannot terminate exact outer fixture')
    end
    wait_exit(owner);wait_exit(child);wait_exit(leaf)
    json.write(path.join(output,'verified.json'),{mode=mode,owner=owner,child=child,leaf=leaf,retired=true})
  end
  local output=path.mkdir_p(path.join(root,'forged-marker'))
  local result=process.run(platform.lua_executable,{fixture,'normal',output,'','a "quoted" b','trailing\\'},
    {cwd=b.root,check=false,env={_OMOBA_PRIVATE_WORKFLOW_JOB='Local\\omoba-workflow-forged'}})
  assert(result.exit_code~=0 and not path.exists(path.join(output,'child.json')),'forged marker bypassed supervision')
  path.write(path.join(output,'stderr.log'),result.stderr)
  local request=path.join(root,'unsafe-caller.json')
  json.write(request,{version=1,operation='workflow_supervise',params={script=fixture,args={'normal',output},cwd=b.root}})
  local result=process.run(b.lib('host').executable(),{request},{cwd=b.root,check=false})
  assert(result.exit_code~=0 and result.stdout:find('workflow caller must be fixed Lua',1,true),'non-Lua helper ancestry was accepted')
  path.write(path.join(root,'unsafe-caller-result.json'),result.stdout)
  -- Direct successful invocation verifies the child's actual exit code too.
  local output=path.mkdir_p(path.join(root,'exit-code'))
  local result=process.run(platform.lua_executable,{fixture,'normal',output,'','a "quoted" b','trailing\\'},{cwd=b.root,check=false})
  assert(result.exit_code==7,'workflow exit code was not forwarded: '..tostring(result.exit_code)..' '..result.stderr)
  wait_exit(json.read(path.join(output,'child.json')));wait_exit(json.read(path.join(output,'leaf.json')))
  json.write(path.join(root,'report.json'),{success=true,scope='native-workflow-only',cases=5,
    normal_tree_retired=true,forced_outer_exit_tree_retired=true,forged_marker_rejected=true,
    unsafe_ancestry_rejected=true,exit_code_and_arguments_preserved=true})
end,debug.traceback)
-- Cleanup only original fixture lifetimes even on assertion failure.
for i=#tracked,1,-1 do
  local owned=tracked[i]
  if process.owned_alive(owned) then process.stop_owned(owned);wait_exit(owned) end
end
if not ok then path.append(path.join(root,'errors.md'),tostring(err)..'\n');error(err,0) end
print('native workflow supervision: 5 cases PASS; '..root)
