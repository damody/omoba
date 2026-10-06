-- Local Editor MCP entry point. All HTTP requests use the bundled Lua workflow
-- and curl's structured argv; no shell-interpolated JSON is executed.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = assert(script:match('^(.*)[/\\]'))
package.path = dir .. '/?.lua;' .. package.path
local bootstrap = require('_bootstrap')
local path = bootstrap.lib('path')
local process = bootstrap.lib('process')
local json = bootstrap.lib('json')

local tool, arguments, output, filter
local list = false
local wait_ready = false
local i = 1
while i <= #arg do
  local value = arg[i]
  if value == '--wait-ready' then wait_ready = true
  elseif value == '--list' then list = true
  elseif value == '--tool' or value == '--arguments' or value == '--out' or value == '--filter' then
    i = i + 1
    local next_value = assert(arg[i], value .. ' requires a value')
    if value == '--tool' then tool = next_value
    elseif value == '--arguments' then arguments = json.decode(next_value)
    elseif value == '--out' then output = path.absolute(next_value, bootstrap.root)
    else filter = next_value end
  else error('unknown argument: ' .. value) end
  i = i + 1
end
assert(list ~= (tool ~= nil), 'use --list or --tool NAME [--arguments JSON]')

local work = path.join(bootstrap.root, 'omfue', 'Saved', 'McpAutomation')
path.mkdir_p(work)
local stem = path.join(work, string.format('request-%d-%06d', os.time(), math.random(0, 999999)))
local request_file, headers_file = stem .. '.json', stem .. '.headers'
local auth_file = stem .. '.auth'
local session_id
local ordinal = 0
local endpoint_api = require('ue_mcp_endpoint')
local project_file = path.join(bootstrap.root,'omfue','om.uproject')
local endpoint = wait_ready and endpoint_api.wait_ready(project_file,30000) or endpoint_api.resolve(project_file)

local function decode_response(body)
  local ok, decoded = pcall(json.decode, body)
  if ok then return decoded end
  local fragments = {}
  for line in (body:gsub('\r', '') .. '\n\n'):gmatch('(.-)\n') do
    local data = line:match('^data:%s?(.*)')
    if data then fragments[#fragments + 1] = data
    elseif line == '' and #fragments > 0 then
      local data_ok, response = pcall(json.decode, table.concat(fragments, '\n'))
      if data_ok and response.id then return response end
      fragments = {}
    end
  end
  error('MCP returned an invalid JSON/SSE response: ' .. body:sub(1, 500))
end

local function request(method, params, notification)
  local current = require('ue_mcp_endpoint').resolve(path.join(bootstrap.root,'omfue','om.uproject'))
  assert(current.pid == endpoint.pid and current.port == endpoint.port, 'MCP Editor identity changed during session')
  ordinal = ordinal + 1
  local payload = {jsonrpc = '2.0', method = method, params = json.object(params)}
  if not notification then payload.id = ordinal end
  path.write(request_file, json.encode(payload), true)
  local args = {'--silent', '--show-error', '--fail-with-body', '--max-time', '20',
    '--header', 'Content-Type: application/json', '--header', 'Accept: application/json, text/event-stream',
    '--dump-header', headers_file, '--data-binary', '@' .. request_file,
    'http://127.0.0.1:' .. endpoint.port .. '/mcp'}
  if path.is_file(auth_file) then table.insert(args, 1, '@' .. auth_file); table.insert(args, 1, '--header') end
  if session_id then table.insert(args, 1, 'Mcp-Session-Id: ' .. session_id); table.insert(args, 1, '--header') end
  local result = process.run('curl.exe', args, {cwd = bootstrap.root, check = false})
  local headers = path.is_file(headers_file) and path.read(headers_file) or ''
  session_id = headers:match('[Mm][Cc][Pp]%-[Ss]ession%-[Ii][Dd]:%s*([^\r\n]+)') or session_id
  assert(result.exit_code == 0, 'MCP HTTP request failed: ' .. result.stderr .. result.stdout)
  if notification then return end
  local response = decode_response(result.stdout)
  assert(response.id == ordinal, 'MCP response id mismatch')
  assert(not response.error, 'MCP protocol error: ' .. json.encode(response.error))
  return assert(response.result, 'MCP response missing result')
end

local function detail_of(result, name)
  assert(not result.isError, 'MCP tool reported failure: ' .. name)
  for _, content in ipairs(result.content or {}) do
    if content.type == 'text' then
      local parsed, detail = pcall(json.decode, content.text)
      if parsed and type(detail) == 'table' then
        assert(detail.ok ~= false and detail.success ~= false, 'MCP tool result reported failure: ' .. name)
        return detail
      end
    end
  end
  error('MCP tool returned no structured result: ' .. name)
end

local function call_detail(name, params)
  return detail_of(request('tools/call', {name = name, arguments = json.object(params)}), name)
end

local function verified_animation(result)
  local imported = detail_of(result, 'import_animation')
  local primary = assert(imported.asset_path, 'animation import missing asset path'):gsub('%.[^/]+$', '')
  local candidate = imported.class == 'AnimSequence' and primary or (primary .. '_Anim')
  -- get_anim_sequence_info casts the actual UObject, so a misleading importer
  -- asset_type label cannot turn a SkeletalMesh into an accepted animation.
  local info = call_detail('get_anim_sequence_info', {asset_path = candidate})
  assert(info.play_length and info.play_length > 0 and info.num_sampled_keys > 0,
    'imported animation has no frames')
  local assets = {candidate}
  if imported.class == 'SkeletalMesh' then
    assets[#assets + 1] = primary
    assets[#assets + 1] = primary .. '_PhysicsAsset'
  end
  local saved = call_detail('save_assets', {asset_paths = assets})
  assert(saved.failed == 0 and saved.saved == #assets, 'animation import assets did not save')
  local dependencies = call_detail('get_asset_dependencies', {asset_path = candidate, recursive = false})
  local wanted = assert(arguments and arguments.skeleton_path, 'animation requires skeleton_path'):gsub('%.[^/]+$', '')
  local found = false
  for _, dependency in ipairs(dependencies.dependencies or {}) do
    if dependency == wanted then found = true end
  end
  assert(found, 'imported animation does not reference the requested skeleton')
  return {isError = false, content = {{type = 'text', text = json.encode({
    ok = true, asset_path = candidate, asset_type = 'AnimSequence', class = 'AnimSequence',
    verified = true, skeleton_path = wanted, play_length = info.play_length,
    num_sampled_keys = info.num_sampled_keys, primary_import_asset = primary,
    saved_assets = assets,
  })}}}
end

local ok, error_message = xpcall(function()
  -- The Editor's local session token is used only with this fixed loopback
  -- endpoint. Keep it out of process argv, stdout and saved reports.
  local local_app_data = os.getenv('LOCALAPPDATA')
  local token_path = local_app_data and path.join(local_app_data, 'UECP', 'session_token.txt')
  if token_path and path.is_file(token_path) then
    local token = path.read(token_path):match('^%s*(.-)%s*$')
    assert(token ~= '' and not token:find('[\r\n]'), 'invalid local MCP session token')
    path.write(auth_file, 'Authorization: Bearer ' .. token .. '\n', false)
  end
  request('initialize', {protocolVersion = '2024-11-05', capabilities = json.object(),
    clientInfo = {name = 'omoba-lua', version = '1.0'}})
  request('notifications/initialized', {}, true)
  local result
  if list then
    result = request('tools/list', {})
    -- Preserve complete schemas for later recipe validation, but keep stdout
    -- compact and readable during discovery.
    output = output or path.join(work, 'tools.json')
    path.write(output, json.encode(result), true)
    for _, entry in ipairs(result.tools or {}) do
      if not filter or entry.name:find(filter, 1, true) then print(entry.name) end
    end
  else
    result = request('tools/call', {name = tool, arguments = json.object(arguments)})
    if tool == 'import_animation' then result = verified_animation(result) end
    if output then path.write(output, json.encode(result), true) end
    print(json.encode(result))
    assert(not result.isError, 'MCP tool reported failure: ' .. tool)
    for _, content in ipairs(result.content or {}) do
      if content.type == 'text' then
        local parsed, detail = pcall(json.decode, content.text)
        if parsed and type(detail) == 'table' then
          assert(detail.ok ~= false and detail.success ~= false and detail.verdict ~= 'FAIL',
            'MCP tool result reported failure: ' .. tool)
        end
      end
    end
  end
  if output then print('[ue-mcp] saved ' .. output) end
end, debug.traceback)
if path.is_file(request_file) then assert(os.remove(request_file)) end
if path.is_file(headers_file) then assert(os.remove(headers_file)) end
if path.is_file(auth_file) then assert(os.remove(auth_file)) end
assert(ok, error_message)
