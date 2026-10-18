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
-- Absent is nil. An explicit boolean passes through. Any other present type is malformed.
local function provider_flag(value)
  if value == nil then return nil end
  if type(value) ~= 'boolean' then return 'malformed' end
  return value
end
-- One explicit true from either provider is enough. An explicit false, or a present
-- non-boolean, on either flag fails closed. Truthiness is never coerced.
local function explicit_positive_result(detail)
  local ok_flag = provider_flag(detail.ok)
  local success_flag = provider_flag(detail.success)
  if ok_flag == 'malformed' or success_flag == 'malformed' then return false end
  if ok_flag == false or success_flag == false then return false end
  return ok_flag == true or success_flag == true
end
function M.result(asset, exit_code, response)
  asset = M.package_path(asset)
  local detail, provider_failure
  for _, content in ipairs(type(response) == 'table' and response.content or {}) do
    if content.type == 'text' then
      local ok, value = pcall(require('tools.lua.lib.json').decode, content.text)
      if ok and type(value) == 'table' then
        if value.blueprint_path then detail = detail or value
        elseif value.success == false or value.ok == false
          or (type(value.error) == 'string' and value.error ~= '')
          or (type(value.error_code) == 'string' and value.error_code ~= '') then
          provider_failure = provider_failure or value
        end
      end
    end
  end
  local result = {asset = asset, success = false, status = 'pending',
    tool = 'compile_blueprint', exit_code = exit_code, diagnostics = {}}
  if provider_failure then
    local code = type(provider_failure.error_code) == 'string' and provider_failure.error_code ~= ''
      and provider_failure.error_code or nil
    local message = type(provider_failure.error) == 'string' and provider_failure.error ~= ''
      and provider_failure.error or 'provider rejected Blueprint compile'
    result.provider_error_code = code
    if type(provider_failure.diagnostics) == 'table' then
      result.provider_diagnostics = provider_failure.diagnostics
    end
    result.diagnostics = {code and (code .. ': ' .. message) or message}
    -- original_response and any other content cannot override a provider rejection.
    return result
  end
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
  result.success = exit_code == 0 and response.isError ~= true and explicit_positive_result(detail)
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
