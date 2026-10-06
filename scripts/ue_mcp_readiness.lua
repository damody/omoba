-- Read-only readiness through the existing project/lifetime/port-owner gate.
local M={}
function M.health_response(exit_code,body,decode)
  if exit_code~=0 then return false,'selected MCP HTTP health request failed' end
  local ok,value=pcall(decode,body)
  return ok and type(value)=='table' and value.server=='uecp','invalid MCP health response'
end
function M.wait(project,timeout_ms,deps)
  assert(math.type(timeout_ms)=='integer' and timeout_ms>=1 and timeout_ms<=600000,'MCP timeout must be 1..600000ms')
  local deadline=deps.time.monotonic_ms()+timeout_ms
  local last='project Editor is not registered'
  while deps.time.monotonic_ms()<deadline do
    local ok,endpoint=pcall(deps.resolve,project)
    if not ok then
      last=tostring(endpoint)
      assert(last:find('found 0',1,true) or last:find('registry unavailable',1,true),last)
    else
      local healthy,why=deps.health(endpoint,math.max(1,deadline-deps.time.monotonic_ms()))
      local current=deps.resolve(project) -- Revalidate after HTTP, before claiming readiness.
      assert(current.pid==endpoint.pid and current.port==endpoint.port
        and current.process_path==endpoint.process_path
        and current.created_unix_seconds==endpoint.created_unix_seconds,
        'MCP identity changed during health probe')
      if healthy==true and deps.time.monotonic_ms()<deadline then
        local legacy_port=deps.legacy_owner and deps.legacy_owner(endpoint) or nil
        return {success=true,project=project,pid=endpoint.pid,port=endpoint.port,
          legacy_port=legacy_port,transport='project-bound-http'}
      end
      last=why or 'selected Editor MCP health not ready'
    end
    deps.time.sleep_ms(250)
  end
  error('Project-bound MCP readiness timed out: '..last)
end
return M
