package.path='scripts/?.lua;'..package.path
require('_bootstrap')
local P=require('tools.lua.lib.process')
local T=require('tools.lua.lib.time')
local host=require('tools.lua.lib.host')
local record=P.validate_owned({pid=7,executable='D:/code/omoba/fixture.exe',creation_token='133000000000000001'})
local count=0
local function test(name,fn)
  local inspect,clock,sleep,call=P.inspect_owned,T.monotonic_ms,T.sleep_ms,host.call
  local now=0
  T.monotonic_ms=function() return now end
  T.sleep_ms=function(ms) assert(ms>0 and ms<=50);now=now+ms end
  P.inspect_owned=function(pid) assert(pid==record.pid);return record end
  host.call=function(operation) error('unexpected host operation '..tostring(operation)) end
  local ok,err=xpcall(function() fn(function()return now end) end,debug.traceback)
  P.inspect_owned,T.monotonic_ms,T.sleep_ms,host.call=inspect,clock,sleep,call
  assert(ok,err);count=count+1;print('PASS '..name)
end
local function expect_wait(timeout,exited)
  local state={calls=0}
  host.call=function(operation,params)
    state.calls=state.calls+1
    assert(operation=='wait_owned')
    assert(params.pid==record.pid and params.expected_exe==record.executable)
    assert(params.creation_token==record.creation_token and params.timeout_ms==timeout)
    return {exited=exited}
  end
  return state
end
test('retired native result is one boolean and does not sleep in Lua',function(now)
  local state=expect_wait(0,true)
  assert(P.wait_owned(record,0) and now()==0 and state.calls==1)
end)
test('token or executable mismatch stays an error',function(now)
  host.call=function() error('owned process lifetime or executable mismatch; nothing waited') end
  assert(not pcall(P.wait_owned,record,5000))
  assert(now()==0)
end)
test('timeout result stays false and the caller bound is forwarded once',function(now)
  local zero=expect_wait(0,false)
  assert(not P.wait_owned(record,0));assert(now()==0 and zero.calls==1)
  local bounded=expect_wait(73,false)
  assert(not P.wait_owned(record,73));assert(now()==0 and bounded.calls==1)
end)
test('native exit and the default bound are one call each',function(now)
  local exited=expect_wait(1000,true)
  assert(P.wait_owned(record,1000) and now()==0 and exited.calls==1)
  local default=expect_wait(5000,false)
  assert(not P.wait_owned(record) and now()==0 and default.calls==1)
end)
test('query failure and malformed legacy identity fail closed',function()
  host.call=function() error('cannot query exact process identity: Access is denied. (os error 5)') end
  assert(not pcall(P.wait_owned,record,100))
  host.call=function() error('unexpected host operation') end
  assert(not pcall(P.wait_owned,{pid=7,exe=record.executable},100))
  for _,timeout in ipairs({-1,1.5,'100'}) do assert(not pcall(P.wait_owned,record,timeout)) end
end)
test('clock regression rejects rather than hanging',function()
  local calls=0
  T.monotonic_ms=function()calls=calls+1;return calls==1 and 100 or 99 end
  host.call=function() return {exited=true} end
  assert(not pcall(P.wait_owned,record,100))
end)
test('readiness only evaluates predicates for the original live child',function(now)
  local reads=0
  assert(P.poll_owned_ready(record,73,function() reads=reads+1;return now()>=50 and 'ready' end)=='ready')
  assert(reads==2 and now()==50)
  P.inspect_owned=function()return {pid=7,executable=record.executable,creation_token='133000000000000002'}end
  assert(not pcall(P.poll_owned_ready,record,73,function()reads=reads+1;return true end))
  assert(reads==2)
end)
test('readiness bounds timeout and query failures are not ready',function(now)
  assert(not pcall(P.poll_owned_ready,record,73,function()return false end));assert(now()==73)
  P.inspect_owned=function()error('query failure')end
  assert(not pcall(P.poll_owned_ready,record,73,function()return true end))
end)
test('original lifetime must survive the readiness predicate',function()
  assert(not pcall(P.poll_owned_ready,record,100,function()
    P.inspect_owned=function()return nil end
    return true
  end))
end)
test('late readiness predicate cannot bypass the original timeout',function()
  local now=0
  T.monotonic_ms=function()return now end
  assert(not pcall(P.poll_owned_ready,record,100,function()now=101;return true end))
end)
print('owned process wait/readiness: '..count..'/'..count..' tests passed')
