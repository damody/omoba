local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path=b.lib('path')
local stages=require('moba_stage_contract')
local workflow=require('moba_launch_workflow')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','launch-contract-tests'))
local root
for i=1,1000 do local p=path.join(parent,os.time()..'-'..i);if lfs.mkdir(p) then root=p;break end end
assert(root)
local count=0
local function test(name,fn) fn();count=count+1;print('PASS '..name) end
local function rejects(fn) assert(not pcall(fn),'expected rejection') end
local bindings=stages.bindings(root)
for _,binding in ipairs(bindings) do
  path.write(binding.built,'fixture-'..binding.role)
  for _,copy in ipairs(binding.copies) do path.write(copy,'fixture-'..binding.role) end
end
test('both libraries and all three deployed copies are checked',function()
  local report=stages.verify(bindings)
  assert(report.scope=='artifact-copy-consistency' and #report.artifacts==2)
  assert(#report.artifacts[1].copies==1 and #report.artifacts[2].copies==2)
end)
test('a stale server-side base DLL is rejected without repairing files',function()
  local copy=bindings[2].copies[1]
  path.write(copy,'stale',true)
  rejects(function() stages.verify(bindings) end)
  assert(path.read(copy)=='stale')
  path.write(copy,'fixture-base_content',true)
end)
test('missing build and missing deployed copies fail closed',function()
  local sha=string.rep('a',64)
  for _,missing in ipairs({bindings[1].built,bindings[2].copies[2]}) do
    rejects(function() stages.verify(bindings,{is_file=function(p)return p~=missing end,sha256=function()return sha end}) end)
  end
end)
local function execute(options,fail)
  local seen={}
  local steps={}
  for _,name in ipairs({'resolve_editor','build_frontend','build_runtime','verify_stage','select','prepare','launch'}) do
    steps[name]=function()
      seen[#seen+1]=name
      if name==fail then error('simulated '..name..' failure') end
      if name=='prepare' then return {human_count=1} end
    end
  end
  local ok=pcall(workflow.execute,options,steps)
  return ok,table.concat(seen,',')
end
test('interactive selection happens after every build and initial deployment check',function()
  local ok,seen=execute({interactive_selection=true})
  assert(ok and seen=='resolve_editor,build_frontend,build_runtime,verify_stage,select,prepare,verify_stage,launch',seen)
end)
test('build failure never opens selection or starts a match',function()
  local ok,seen=execute({interactive_selection=true},'build_runtime')
  assert(not ok and seen=='resolve_editor,build_frontend,build_runtime',seen)
end)
test('deployment failure rejects before user selection',function()
  local ok,seen=execute({interactive_selection=true},'verify_stage')
  assert(not ok and seen=='resolve_editor,build_frontend,build_runtime,verify_stage',seen)
end)
test('no-build still checks deployment before and after selection',function()
  local ok,seen=execute({interactive_selection=true,no_build=true})
  assert(ok and seen=='resolve_editor,verify_stage,select,prepare,verify_stage,launch',seen)
end)
test('prepare-only never resolves Unreal, builds or starts processes',function()
  local ok,seen=execute({prepare_only=true})
  assert(ok and seen=='prepare',seen)
end)
print(('launch contract functionality: %d/%d passed; real fixture hashes, mocked builds/processes'):format(count,count))
