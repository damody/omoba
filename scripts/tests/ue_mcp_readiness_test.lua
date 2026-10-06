package.path='scripts/?.lua;'..package.path
local wait=require('ue_mcp_readiness').wait
local count=0
local function scenario(name,resolve,health,pass,wanted)
  local now=0
  local deps={time={monotonic_ms=function() return now end,sleep_ms=function(ms) now=now+ms end},
    resolve=resolve,health=health or function() return true end}
  local ok,result=pcall(wait,'D:/selected/om.uproject',1000,deps)
  assert(ok==pass,name..': '..tostring(result))
  if pass then assert(result.pid==42 and result.port==30001 and result.transport=='project-bound-http')
  elseif wanted then assert(tostring(result):find(wanted,1,true),tostring(result)) end
  count=count+1
end
local function own() return {pid=42,port=30001} end
scenario('exact selected endpoint',own,nil,true)
local calls=0
scenario('delayed registration',function() calls=calls+1;if calls<3 then error('found 0') end;return own() end,nil,true)
scenario('foreign healthy port is never probed',function() error('found 0') end,function() error('must not probe') end,false,'timed out')
scenario('ambiguous registry fails immediately',function() error('found 2') end,nil,false,'found 2')
scenario('stale PID never probes health',function() error('found 0') end,nil,false,'timed out')
scenario('failed or malformed health never ready',own,function() return false,'invalid health' end,false,'timed out')
local probes=0
scenario('identity changes during health rejected',function() probes=probes+1;return {pid=probes==1 and 42 or 99,port=30001} end,nil,false,'identity changed')
local lifetimes=0
scenario('recycled same PID rejected',function() lifetimes=lifetimes+1;return {pid=42,port=30001,
  created_unix_seconds=lifetimes,process_path='d:/ue/unrealeditor.exe'} end,nil,false,'identity changed')
local paths=0
scenario('changed executable rejected',function() paths=paths+1;return {pid=42,port=30001,
  created_unix_seconds=1,process_path=paths==1 and 'd:/ue/unrealeditor.exe' or 'd:/other/unrealeditor.exe'} end,nil,false,'identity changed')
scenario('registry absent is bounded',function() error('registry unavailable') end,nil,false,'timed out')
print('project-bound MCP readiness: '..count..' scenarios passed')
local clock={monotonic_ms=function() return 0 end,sleep_ms=function() error('unexpected retry') end}
assert(wait('selected',1000,{time=clock,resolve=own,health=function() return true end,
  legacy_owner=function(endpoint) assert(endpoint.pid==42);return 9877 end}).legacy_port==9877)
assert(not pcall(wait,'selected',1000,{time=clock,resolve=own,health=function() return true end,
  legacy_owner=function() error('legacy command port belongs to a different process') end}))
print('legacy command owner: 2 scenarios passed')
local parse=require('ue_mcp_readiness').health_response
local json=require('tools.lua.lib.json')
assert(parse(0,'{"server":"uecp"}',json.decode))
for _,body in ipairs({'{"server":"other"}','[]','null','not JSON','{"server":true}'}) do
  assert(not parse(0,body,json.decode))
end
assert(not parse(22,'{"server":"uecp"}',json.decode))
print('MCP health decoder: 7 scenarios passed')
