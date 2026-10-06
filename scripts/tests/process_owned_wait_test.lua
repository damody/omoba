package.path='scripts/?.lua;'..package.path
require('_bootstrap')
local P=require('tools.lua.lib.process')
local T=require('tools.lua.lib.time')
local record=P.validate_owned({pid=7,executable='D:/code/omoba/fixture.exe',creation_token='133000000000000001'})
local count=0
local function test(name,fn)
  local inspect,clock,sleep=P.inspect_owned,T.monotonic_ms,T.sleep_ms
  local now=0
  T.monotonic_ms=function() return now end
  T.sleep_ms=function(ms) assert(ms>0 and ms<=50);now=now+ms end
  P.inspect_owned=function(pid) assert(pid==record.pid);return record end
  local ok,err=xpcall(function() fn(function()return now end) end,debug.traceback)
  P.inspect_owned,T.monotonic_ms,T.sleep_ms=inspect,clock,sleep
  assert(ok,err);count=count+1;print('PASS '..name)
end
test('already retired lifetime succeeds without PID wait',function(now)
  P.inspect_owned=function() return nil end
  assert(P.wait_owned(record,0) and now()==0)
end)
test('same executable reused PID is not the original lifetime',function(now)
  P.inspect_owned=function() return {pid=7,executable=record.executable,creation_token='133000000000000002'} end
  assert(P.wait_owned(record,5000) and now()==0)
end)
test('bounded timeout and zero timeout never oversleep',function(now)
  assert(not P.wait_owned(record,0));assert(now()==0)
  assert(not P.wait_owned(record,73));assert(now()==73)
end)
test('original exits before timeout and default remains bounded',function(now)
  P.inspect_owned=function() if now()<100 then return record end end
  assert(P.wait_owned(record,1000) and now()==100)
  P.inspect_owned=function()return record end
  assert(not P.wait_owned(record));assert(now()==5100)
end)
test('query failure and malformed legacy identity fail closed',function()
  P.inspect_owned=function()error('inaccessible owned process')end
  assert(not pcall(P.wait_owned,record,100))
  assert(not pcall(P.wait_owned,{pid=7,exe=record.executable},100))
  for _,timeout in ipairs({-1,1.5,'100'}) do assert(not pcall(P.wait_owned,record,timeout)) end
end)
test('clock regression rejects rather than hanging',function()
  local calls=0
  T.monotonic_ms=function()calls=calls+1;return calls==1 and 100 or 99 end
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
