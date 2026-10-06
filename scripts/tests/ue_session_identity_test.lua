package.path='scripts/?.lua;'..package.path
require('_bootstrap')
local M=require('ue_session_identity')
local P=require('tools.lua.lib.process')
local n=0
local function test(name,fn) fn();n=n+1;print('PASS '..name) end
local function rejects(fn) assert(not pcall(fn),'expected rejection') end
local function state()
  return {identity_version=1,session_id='fixture',processes={
    {role='server',pid=7,executable='D:/code/omoba/fixture.exe',creation_token='133000000000000001'},
    {role='client',pid=8,executable='D:/code/omoba/fixture.exe',creation_token='133000000000000002'}}}
end
local function mock(failure)
  local stops={}
  return {validate_owned=P.validate_owned,assert_owned=function(r)
    if failure and failure(r) then error('query/lifetime mismatch') end
    return r
  end,stop_owned=function(r) stops[#stops+1]=r.pid end},stops
end
test('exact string token and strict PID/executable validation',function()
  local s=state();assert(P.validate_owned(s.processes[1]).creation_token=='133000000000000001')
  for _,v in ipairs({'0','01','-1','1.0',' 1','18446744073709551616',1}) do
    s.processes[1].creation_token=v;rejects(function() P.validate_owned(s.processes[1]) end)
  end
  s=state();s.processes[1].pid=1.5;rejects(function() P.validate_owned(s.processes[1]) end)
  s=state();s.processes[1].executable='relative.exe';rejects(function() P.validate_owned(s.processes[1]) end)
end)
test('legacy and malformed state never stop',function()
  for _,s in ipairs({{}, {session_id='old',processes=state().processes}}) do
    local p,stops=mock();rejects(function() M.clean(s,p) end);assert(#stops==0)
  end
  local s=state();s.processes[2].creation_token=nil
  local p,stops=mock();rejects(function() M.clean(s,p) end);assert(#stops==0)
end)
test('same executable reused PID or inaccessible later record preflight stops zero',function()
  local p,stops=mock(function(r) return r.pid==8 end)
  rejects(function() M.clean(state(),p) end);assert(#stops==0)
end)
test('duplicate roles PID and sparse records reject without stopping',function()
  for _,mutate in ipairs({function(s)s.processes[2].pid=7 end,
    function(s)s.processes[2].role='server' end,
    function(s)s.processes[7]=s.processes[2] end}) do
    local s=state();mutate(s);local p,stops=mock()
    rejects(function() M.clean(s,p) end);assert(#stops==0)
  end
end)
test('valid session cleans reverse order using captured exact identity',function()
  local p,stops=mock();M.clean(state(),p);assert(stops[1]==8 and stops[2]==7)
end)
test('replacement token must replace full identity not only PID',function()
  local s=state();local replacement=P.validate_owned({pid=9,path='D:/code/omoba/fixture.exe',creation_token='133000000000000009'})
  replacement.role='client';s.processes[2]=replacement
  local p,stops=mock();M.clean(s,p);assert(stops[1]==9)
end)
test('dead or reused PID is not original lifetime; query error propagates',function()
  local original=P.inspect_owned
  local ok,err=pcall(function()
    P.inspect_owned=function() return nil end;assert(not P.owned_alive(state().processes[1]))
    P.inspect_owned=function() return P.validate_owned(state().processes[2]) end
    assert(not P.owned_alive(state().processes[1]));rejects(function() P.assert_owned(state().processes[1]) end)
    P.inspect_owned=function() error('access denied') end
    rejects(function() P.owned_alive(state().processes[1]) end)
  end)
  P.inspect_owned=original;if not ok then error(err,0) end
end)
print(('UE session identity: %d/7 tests passed'):format(n))
