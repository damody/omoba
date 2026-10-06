local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json,process,time=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('time')
local project,timeout,output,legacy
for i=1,#arg,2 do
  local key,value=arg[i],assert(arg[i+1],'missing argument value')
  if key=='--project' then assert(not project,'duplicate project');project=value
  elseif key=='--timeout-ms' then assert(not timeout,'duplicate timeout');timeout=value
  elseif key=='--out' then assert(not output,'duplicate output');output=value
  elseif key=='--legacy-port' then assert(not legacy,'duplicate legacy port');legacy=value
  else error('unknown argument: '..key) end
end
assert(project and timeout and output,'requires --project --timeout-ms --out')
assert(not path.exists(output),'readiness output already exists')
local ok,result=pcall(function()
  assert(timeout:match('^%d+$'),'timeout must be decimal milliseconds')
  return require('ue_mcp_readiness').wait(path.absolute(project),assert(math.tointeger(tonumber(timeout))),{
    time=time,resolve=require('ue_mcp_endpoint').resolve,
    legacy_owner=legacy and function(endpoint)
      local port=legacy:match('^%d+$') and math.tointeger(tonumber(legacy))
      assert(port and port>=1 and port<=65535,'invalid legacy command port')
      local owner=b.lib('host').call('tcp_listener',{port=port})
      assert(owner.pid==endpoint.pid and owner.created_unix_seconds==endpoint.created_unix_seconds
        and type(owner.path)=='string' and owner.path:gsub('\\','/'):lower()==endpoint.process_path,
        'legacy command port belongs to a different process lifetime; no scan fallback')
      return port
    end or nil,
    health=function(endpoint,remaining)
      local response=process.run('curl.exe',{'--silent','--show-error','--fail','--max-time',
        tostring(math.min(2,remaining/1000)),'http://127.0.0.1:'..endpoint.port..'/mcp/health'},
        {cwd=b.root,check=false})
      return require('ue_mcp_readiness').health_response(response.exit_code,response.stdout,json.decode)
    end})
end)
local report=ok and result or {success=false,error=tostring(result)}
json.write(output,report) -- Caller owns a fresh, exclusive directory.
print(json.encode(report))
os.exit(ok and 0 or 1)
