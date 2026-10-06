-- Resolve only a registered local Editor for this project, then prove port owner.
local path = require('tools.lua.lib.path')
local json = require('tools.lua.lib.json')
local host = require('tools.lua.lib.host')
local lfs = require('lfs')
local M = {}
local function normalized(value) return path.absolute(value):gsub('\\','/'):lower() end
-- Optional workflow binding. A registry match alone must never authorize
-- mutations in another Editor opened for the same project.
function M.require_owned(endpoint, owned, process)
  owned=process.validate_owned(owned)
  assert(process.owned_alive(owned),'owned MCP Editor has retired; never replace it')
  assert(endpoint.pid==owned.pid and normalized(endpoint.process_path)==normalized(owned.executable),
    'MCP endpoint is not this workflow\'s original Editor')
  return endpoint
end
function M.select(records, project, owner_of)
  local found = {}
  for _, record in ipairs(records) do
    if type(record) == 'table' and type(record.project_path) == 'string' and normalized(record.project_path) == normalized(project) then
      local pid = type(record.pid) == 'number' and math.tointeger(record.pid)
      local port = type(record.mcp_http_port) == 'number' and math.tointeger(record.mcp_http_port)
      if pid and pid > 0 and port and port >= 1 and port <= 65535 then
        local ok, owner = pcall(owner_of, port)
        local same_lifetime = not record.registered_unix_seconds or
          (ok and owner and type(owner.created_unix_seconds) == 'number' and
           owner.created_unix_seconds <= record.registered_unix_seconds)
        if ok and owner and same_lifetime and owner.pid == pid and type(owner.path) == 'string' and
            owner.path:gsub('\\','/'):lower():match('/unrealeditor%.exe$') then
          found[#found + 1] = {pid=pid, port=port, project=record.project_path,
            process_path=owner.path:gsub('\\','/'):lower(),created_unix_seconds=owner.created_unix_seconds}
        end
      end
    end
  end
  assert(#found == 1, 'MCP requires exactly one registered live UnrealEditor for this project; found ' .. #found .. ' (never fallback to port 30000)')
  return found[1]
end
function M.resolve(project)
  local appdata = assert(os.getenv('LOCALAPPDATA'), 'LOCALAPPDATA unavailable')
  local registry = path.join(appdata,'UECP','instances')
  assert(path.is_directory(registry), 'Unreal MCP instance registry unavailable')
  local records = {}
  for name in lfs.dir(registry) do
    if name:match('^%d+%.json$') then
      local ok, record = pcall(json.read,path.join(registry,name))
      if ok and type(record) == 'table' then
        record.registered_unix_seconds = path.attributes(path.join(registry,name)).modification
        records[#records+1] = record
      end
    end
  end
  return M.select(records,project,function(port) return host.call('tcp_listener',{port=port}) end)
end
function M.wait_ready(project, timeout_ms)
  local time = require('tools.lua.lib.time')
  local last_error
  local endpoint = time.poll(timeout_ms, 250, function()
    local ok, result = pcall(M.resolve, project)
    if ok then return result end
    last_error = tostring(result)
    assert(last_error:find('found 0',1,true) or last_error:find('registry unavailable',1,true),last_error)
  end)
  return assert(endpoint, 'Project-bound MCP readiness timed out: ' .. tostring(last_error))
end
return M
