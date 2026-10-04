-- Fail closed on transport errors, compile errors, and unhealthy Blueprint graphs.
local M = {}
function M.package_path(value)
  assert(type(value) == 'string', 'Blueprint asset path must be a string')
  local package, object = value:match('^(/Game/[%w_/]+)%.([%w_]+)$')
  if package then
    local name = package:match('([^/]+)$')
    assert(object == name or object == name .. '_C', 'mismatched Blueprint object path')
    value = package
  end
  assert(value:match('^/Game/[%w_/]+$') and not value:find('//',1,true), 'invalid Blueprint package path')
  return value
end
function M.result(asset, exit_code, response)
  asset = M.package_path(asset)
  local detail
  for _, content in ipairs(type(response) == 'table' and response.content or {}) do
    if content.type == 'text' then
      local ok, value = pcall(require('tools.lua.lib.json').decode, content.text)
      if ok and type(value) == 'table' and value.blueprint_path then detail = value; break end
    end
  end
  local result = {asset = asset, success = false, status = 'pending',
    tool = 'compile_blueprint', exit_code = exit_code, diagnostics = {}}
  if not detail then result.diagnostics = {'missing structured Blueprint compile result'}; return result end
  result.compiler = detail
  local same, actual = pcall(M.package_path, detail.blueprint_path)
  if not same or actual ~= asset then
    result.diagnostics = {'compiler result belongs to another asset'}; return result
  end
  for _, message in ipairs(detail.errors or {}) do result.diagnostics[#result.diagnostics+1] = tostring(message) end
  for _, issue in ipairs(detail.health_issues or {}) do
    result.diagnostics[#result.diagnostics+1] = 'graph health: ' .. require('tools.lua.lib.json').encode(issue)
  end
  if detail.error then result.diagnostics[#result.diagnostics+1] = detail.error end
  result.success = exit_code == 0 and response.isError ~= true and detail.ok == true
    and detail.success ~= false
    and detail.error_count == 0 and #(detail.errors or {}) == 0
    and #(detail.health_issues or {}) == 0 and not detail.action_required
  if result.success then result.status = 'compiled'
  elseif #result.diagnostics == 0 then result.diagnostics = {'compile status incomplete or requires action'} end
  return result
end
function M.compile(b, asset, output)
  local path, process, json = b.lib('path'), b.lib('process'), b.lib('json')
  asset = M.package_path(asset)
  -- The output is unique per attempt. Never read a prior successful response.
  assert(not path.exists(output), 'refusing stale compile response: ' .. output)
  local command = process.run(b.lib('platform').lua_executable, {
    path.join(b.root,'scripts/ue_mcp.lua'),'--tool','compile_blueprint',
    '--arguments',json.encode({blueprint_path=asset}),'--out',output,
  }, {cwd=b.root,check=false})
  local response
  if path.is_file(output) then
    local parsed, value = pcall(json.read, output)
    if parsed then response = value end
  end
  local result = M.result(asset, command.exit_code, response)
  result.raw_response = output
  if not response then result.transport_error = command.stderr end
  return result
end
return M
