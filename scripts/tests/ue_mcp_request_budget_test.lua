package.path='scripts/?.lua;'..package.path
local budget=require('ue_mcp_request_budget')
local count=0
for _,value in ipairs({'1','20','180','3600',180}) do
  assert(budget.parse(value)==tonumber(value));count=count+1
end
for _,value in ipairs({'0','-1','3601','1.5','nan','inf','1e999','',false,math.huge,-math.huge,0/0}) do
  assert(not pcall(budget.parse,value),tostring(value));count=count+1
end
assert(not pcall(budget.parse,nil));count=count+1
assert(budget.for_tool('begin_play_in_editor')==180)
for _,name in ipairs({'ping','get_pie_status','stop_play_in_editor','take_pie_screenshot'}) do
  assert(budget.for_tool(name)==20)
end
-- Invalid CLI values must fail during parsing, before endpoint discovery or HTTP.
local b=require('_bootstrap')
local process,path,platform=b.lib('process'),b.lib('path'),b.lib('platform')
for _,value in ipairs({'0','3601','1.5','nan'}) do
  local result=process.run(platform.lua_executable,{path.join(b.root,'scripts/ue_mcp.lua'),
    '--tool','ping','--request-timeout-seconds',value},{cwd=b.root,check=false})
  assert(result.exit_code~=0)
  assert((result.stderr..result.stdout):find('must be an integer from 1 to 3600',1,true))
  count=count+1
end
print('MCP bounded request budget: '..count..' validation cases and tool policy passed')
