local source=debug.getinfo(1,'S').source:sub(2)
local tests=assert(source:match('^(.*)[/\\]'))
package.path=assert(tests:match('^(.*)[/\\]tests$'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,platform=b.lib('path'),b.lib('platform')
local gate=require('moba_frontend_preflight')
local count=0
local function test(name,fn) fn();count=count+1;print('PASS '..name) end
local function rejects(fn,reason)
  local ok,err=pcall(fn);assert(not ok,'expected rejection')
  assert(tostring(err):find(reason,1,true),tostring(err))
end
test('selected engine then fixed Lua verify-only process, no build',function()
  local events={}
  local result={exit_code=0,stdout=''}
  assert(gate.verify('fixture-engine',{run=function(exe,args,options)
    events[#events+1]='stage'
    assert(events[1]=='engine' and exe==platform.lua_executable)
    assert(#args==4 and args[1]==path.join(b.root,'scripts','build_ue_moba.lua'))
    assert(args[2]=='--verify-staged-only' and args[3]=='--ue-root' and args[4]=='fixture-engine')
    assert(options.cwd==b.root and options.check==false)
    return result
  end},{require_ready=function(engine,project)
    events[#events+1]='engine'
    assert(engine=='fixture-engine' and project==path.join(b.root,'omfue','om.uproject'))
  end})==result)
  assert(#events==2)
end)
test('binary readiness failure prevents stage process',function()
  rejects(function() gate.verify('fixture-engine',{run=function() error('must not run') end},
    {require_ready=function() error('fixture binary mismatch') end}) end,'fixture binary mismatch')
end)
test('nonzero stage exit rejects successful-looking stdout',function()
  rejects(function() gate.verify('fixture-engine',{run=function()
    return {exit_code=1,stdout='verified',stderr='fixture stage mismatch'}
  end},{require_ready=function() end}) end,'fixture stage mismatch')
end)
test('missing exit code rejects before success',function()
  rejects(function() gate.verify('fixture-engine',{run=function() return {stdout='verified'} end},
    {require_ready=function() end}) end,'frontend staged verification failed')
end)
test('missing or malformed root prevents all callbacks',function()
  local process={run=function() error('must not run') end}
  local dependencies={require_ready=function() error('must not inspect') end}
  for _,root in ipairs({'',false,42,{}}) do
    rejects(function() gate.verify(root,process,dependencies) end,'requires selected Unreal root')
  end
  rejects(function() gate.verify(nil,process,dependencies) end,'requires selected Unreal root')
end)
print(('frontend preflight: %d groups passed; injected readiness/process, no Unreal/build/network'):format(count))
