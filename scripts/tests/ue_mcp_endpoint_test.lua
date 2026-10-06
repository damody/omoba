local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
require('_bootstrap')
local select_endpoint=require('ue_mcp_endpoint').select
local project='D:/fixture/om.uproject'
local own={project_path=project,pid=42,mcp_http_port=30001,registered_unix_seconds=102}
local foreign={project_path='C:/other/other.uproject',pid=99,mcp_http_port=30000}
local function live(port) return {pid=port==30001 and 42 or 99,path='D:/UE/Engine/Binaries/Win64/UnrealEditor.exe',created_unix_seconds=100} end
assert(select_endpoint({foreign,own},project,live).port==30001)
assert(select_endpoint({own},'d:\\fixture\\om.uproject',live).pid==42)
local count=2
local function reject(records, owner)
  assert(not pcall(select_endpoint,records,project,owner or live)); count=count+1
end
reject({foreign})
reject({own},function() error('not listening') end)
reject({own},function() return {pid=99,path='D:/UE/UnrealEditor.exe',created_unix_seconds=100} end)
reject({own},function() return {pid=42,path='D:/UE/UnrealEditor-Cmd.exe',created_unix_seconds=100} end)
reject({own},function() return {pid=42,path='D:/UE/UnrealEditor.exe',created_unix_seconds=103} end)
assert(select_endpoint({own},project,function() return {pid=42,path='D:/UE/UnrealEditor.exe',created_unix_seconds=1} end).pid==42)
count=count+1 -- An Editor may load/register its MCP plugin long after process creation.
reject({own},function() return {pid=42,path='D:/UE/UnrealEditor.exe'} end)
reject({own,own})
reject({{project_path=project,pid=42,mcp_http_port=0}})
reject({{project_path=project,pid={},mcp_http_port=30001}})
print('Unreal MCP endpoint identity: '..count..' scenarios passed')
local require_owned=require('ue_mcp_endpoint').require_owned
local original={pid=42,executable='D:/UE/UnrealEditor.exe',creation_token='fixture'}
local endpoint={pid=42,process_path='d:/ue/unrealeditor.exe'}
local alive=true
local process={validate_owned=function(record) assert(record==original);return record end,
  owned_alive=function(record) assert(record==original);return alive end}
assert(require_owned(endpoint,original,process)==endpoint)
alive=false
assert(not pcall(require_owned,endpoint,original,process))
alive=true
assert(not pcall(require_owned,{pid=43,process_path=endpoint.process_path},original,process))
assert(not pcall(require_owned,{pid=42,process_path='C:/other/UnrealEditor.exe'},original,process))
print('Owned MCP Editor binding: 4 scenarios passed; injected process lifetime, no HTTP')
