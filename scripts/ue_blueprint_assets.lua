-- Provider negotiation for generated Blueprint existence, direct parent, create, and save.
-- Callbacks are injected. This module never calls the Editor and never invents a tool result.
local M = {}
local validation = require('ue_blueprint_validation')
local json = require('tools.lua.lib.json')

M.native_tools = {'list_assets', 'create_blueprint', 'get_blueprint_summary', 'save_asset'}
M.legacy_tools = {'check_asset_exists', 'create_asset', 'get_blueprint_skeleton', 'save_assets'}

local function is_null(value) return value == json.null end

local function flag(value)
  if value == nil then return nil end
  if type(value) ~= 'boolean' then return 'malformed' end
  return value
end

local function is_sequence(value)
  if type(value) ~= 'table' or is_null(value) then return false end
  local meta = getmetatable(value)
  if meta and meta.__json_object then return false end
  local count, max = 0, 0
  for key in pairs(value) do
    if type(key) ~= 'number' or key < 1 or key % 1 ~= 0 then return false end
    count = count + 1
    if key > max then max = key end
  end
  return max == count
end

function M.is_explicit_negative(value)
  if type(value) ~= 'table' or is_null(value) then return true end
  if flag(value.success) == 'malformed' or flag(value.success) == false then return true end
  if flag(value.ok) == 'malformed' or flag(value.ok) == false then return true end
  if value.verdict == 'FAIL' then return true end
  if value.error ~= nil and value.error ~= '' then return true end
  if value.error_code ~= nil and value.error_code ~= '' then return true end
  return false
end

function M.negative_message(value)
  if type(value) ~= 'table' or is_null(value) then return 'missing structured result' end
  local code = type(value.error_code) == 'string' and value.error_code ~= '' and value.error_code or nil
  local message = type(value.error) == 'string' and value.error ~= '' and value.error or nil
  if code and message then return code .. ': ' .. message end
  if message then return message end
  if code then return code end
  if value.success == false or value.ok == false or value.verdict == 'FAIL' then
    return 'provider rejected blueprint asset operation'
  end
  return 'missing structured result'
end

function M.unwrap_tool(envelope)
  if type(envelope) ~= 'table' or is_null(envelope) or not is_sequence(envelope.content) then return nil end
  local first
  for _, content in ipairs(envelope.content) do
    if type(content) == 'table' and not is_null(content) and content.type == 'text'
      and type(content.text) == 'string' then
      local ok, value = pcall(json.decode, content.text)
      if ok and type(value) == 'table' and not is_null(value) then
        if M.is_explicit_negative(value) then return value end
        first = first or value
      end
    end
  end
  if envelope.isError ~= nil and envelope.isError ~= false then
    return {success=false,error='MCP envelope rejected blueprint asset operation'}
  end
  return first
end

function M.capability_names(payload)
  if M.is_explicit_negative(payload) or not is_sequence(payload.tools) then
    error('malformed tools/list')
  end
  local names = {}
  for _, entry in ipairs(payload.tools) do
    if type(entry) ~= 'table' or is_null(entry) or is_sequence(entry)
      or type(entry.name) ~= 'string' or entry.name == '' or names[entry.name] then
      error('malformed tools/list')
    end
    names[entry.name] = true
  end
  return names
end

function M.require_idle(value)
  assert(not M.is_explicit_negative(value)
    and (value.success == true or value.ok == true) and value.pie_running == false,
    'stop PIE before compiling or creating assets; idle status must be explicit')
end

local function missing_tools(names, tools)
  local gap = {}
  for _, name in ipairs(tools) do
    if not names[name] then gap[#gap + 1] = name end
  end
  return gap
end

function M.select_provider(payload)
  local names = M.capability_names(payload)
  local native_gap = missing_tools(names, M.native_tools)
  local legacy_gap = missing_tools(names, M.legacy_tools)
  if #native_gap == 0 then return 'native' end
  if #legacy_gap == 0 then return 'legacy' end
  error('incomplete blueprint asset provider; native missing: ' .. table.concat(native_gap, ',')
    .. '; legacy missing: ' .. table.concat(legacy_gap, ','))
end

function M.parent_path(parent_short)
  assert(type(parent_short) == 'string' and parent_short:match('^OmHero[%w_]+$'),
    'invalid generated hero parent')
  return '/Script/OmGenerated.' .. parent_short
end

function M.split(asset)
  local canonical = validation.package_path(asset)
  local folder, name = canonical:match('^(.*)/([^/]+)$')
  assert(folder and name, 'invalid Blueprint package path')
  return canonical, folder, name
end

local function same_package(value, asset, what)
  local ok, actual = pcall(validation.package_path, value)
  if not ok or actual ~= asset then error(what) end
  return actual
end

function M.classify_native(folder, asset, value)
  if M.is_explicit_negative(value) then error('asset inventory rejected') end
  if type(value.folder_path) ~= 'string' or value.folder_path ~= folder then
    error('foreign folder inventory')
  end
  if not is_sequence(value.assets) then error('malformed asset inventory') end
  if type(value.count) ~= 'number' or value.count % 1 ~= 0 or value.count ~= #value.assets then
    error('asset inventory count mismatch')
  end
  local classes = {}
  for _, entry in ipairs(value.assets) do
    if type(entry) ~= 'table' or is_null(entry) or is_sequence(entry) then error('malformed asset inventory') end
    if type(entry.name) ~= 'string' or entry.name == ''
      or type(entry.path) ~= 'string' or entry.path == ''
      or type(entry.class) ~= 'string' or entry.class == '' then
      error('malformed asset inventory')
    end
    local ok, package = pcall(validation.package_path, entry.path)
    if not ok then error('malformed asset inventory') end
    local entry_folder, entry_name = package:match('^(.*)/([^/]+)$')
    if entry_folder ~= folder then error('foreign folder inventory') end
    if entry_name ~= entry.name then error('malformed asset identity') end
    if classes[package] then error('duplicate asset path') end
    classes[package] = entry.class
  end
  local class = classes[asset]
  if class == nil then return 'missing' end
  if class ~= 'Blueprint' then error('refusing non-blueprint asset: ' .. asset) end
  return 'present'
end

function M.presence(provider, asset, call)
  if provider ~= 'native' and provider ~= 'legacy' then error('unknown blueprint asset provider') end
  local canonical, folder = M.split(asset)
  if provider == 'native' then
    return M.classify_native(folder, canonical, call('list_assets', {
      folder_path = folder, recursive = false,
    }))
  end
  local value = call('check_asset_exists', {asset_path = canonical})
  if M.is_explicit_negative(value) then error('asset existence rejected') end
  if type(value.exists) ~= 'boolean' then error('asset existence unknown') end
  return value.exists and 'present' or 'missing'
end

-- create_blueprint sets blueprint_path from UBlueprint::GetPathName and echoes name.
-- It does not set asset_path. A present asset_path must still name the same package.
function M.accept_native_create(asset, value)
  if M.is_explicit_negative(value) then error('blueprint creation rejected') end
  if value.success ~= true then error('blueprint creation status unknown') end
  local _, _, name = M.split(asset)
  same_package(value.blueprint_path, asset, 'creation returned another asset')
  if value.name ~= name then error('creation returned another asset') end
  if value.asset_path ~= nil then
    same_package(value.asset_path, asset, 'creation returned another asset')
  end
end

function M.accept_legacy_create(asset, value)
  if M.is_explicit_negative(value) then error('blueprint creation rejected') end
  same_package(value.asset_path, asset, 'creation returned another asset')
end

function M.create(provider, asset, parent_short, call)
  local canonical, folder, name = M.split(asset)
  local parent_path = M.parent_path(parent_short)
  if provider == 'native' then
    M.accept_native_create(canonical, call('create_blueprint', {
      name = name, parent_class = parent_path, save_path = folder,
    }))
  elseif provider == 'legacy' then
    M.accept_legacy_create(canonical, call('create_asset', {
      asset_type = 'Blueprint', name = name, save_path = folder,
      options = {parent_class = parent_path},
    }))
  else
    error('unknown blueprint asset provider')
  end
end

function M.accept_native_parent(asset, parent_short, value)
  if M.is_explicit_negative(value) then error('blueprint parent rejected') end
  if value.success ~= true or value.metadata_version ~= 1
    or type(value.summary) ~= 'string' or value.summary == '' then
    error('blueprint parent status unknown')
  end
  same_package(value.blueprint_path, asset, 'blueprint identity mismatch')
  if value.parent_class ~= parent_short or value.parent_class_path ~= M.parent_path(parent_short) then
    error('refusing mismatched existing Blueprint parent: ' .. asset)
  end
end

function M.accept_legacy_parent(asset, parent_short, value)
  if M.is_explicit_negative(value) then error('blueprint parent rejected') end
  if value.parent_class ~= parent_short then
    error('refusing mismatched existing Blueprint parent: ' .. asset)
  end
end

function M.require_parent(provider, asset, parent_short, call)
  local canonical = validation.package_path(asset)
  if provider == 'native' then
    M.accept_native_parent(canonical, parent_short, call('get_blueprint_summary', {
      blueprint_path = canonical,
    }))
  elseif provider == 'legacy' then
    M.accept_legacy_parent(canonical, parent_short, call('get_blueprint_skeleton', {
      blueprint_path = canonical,
    }))
  else
    error('unknown blueprint asset provider')
  end
end

function M.accept_native_save(asset, value)
  if M.is_explicit_negative(value) then error('asset save rejected') end
  if value.success ~= true then error('asset save status unknown') end
  same_package(value.asset_path, asset, 'asset save identity mismatch')
  same_package(value.normalized_asset_path, asset, 'asset save identity mismatch')
  if value.asset_class ~= 'Blueprint' then error('asset save class mismatch') end
end

function M.accept_legacy_save(value)
  if M.is_explicit_negative(value) then error('asset save rejected') end
  if type(value.failed) ~= 'number' or type(value.saved) ~= 'number'
    or value.failed % 1 ~= 0 or value.saved % 1 ~= 0
    or value.failed ~= 0 or value.saved ~= 1 then
    error('asset save incomplete')
  end
end

function M.save(provider, asset, call)
  local canonical = validation.package_path(asset)
  if provider == 'native' then
    M.accept_native_save(canonical, call('save_asset', {asset_path = canonical}))
  elseif provider == 'legacy' then
    M.accept_legacy_save(call('save_assets', {asset_paths = {canonical}}))
  else
    error('unknown blueprint asset provider')
  end
end

function M.bootstrap(provider, asset, parent_short, deps)
  assert(type(deps) == 'table' and type(deps.call) == 'function' and type(deps.compile) == 'function',
    'blueprint bootstrap requires call and compile callbacks')
  local state = M.presence(provider, asset, deps.call)
  local created = false
  if state == 'missing' then
    M.create(provider, asset, parent_short, deps.call)
    created = true
    if deps.on_created then deps.on_created() end
  elseif state ~= 'present' then
    error('asset existence unknown')
  end
  M.require_parent(provider, asset, parent_short, deps.call)
  local compiled = deps.compile(created)
  local diagnostics = type(compiled) == 'table' and compiled.diagnostics or nil
  local clean = type(compiled) == 'table' and compiled.success == true
    and (diagnostics == nil or (is_sequence(diagnostics) and #diagnostics == 0))
  if not clean then
    if not is_sequence(diagnostics) or #diagnostics == 0 then diagnostics = {'compile failed'} end
    local parts = {}
    for _, item in ipairs(diagnostics) do parts[#parts + 1] = tostring(item) end
    error(validation.package_path(asset) .. ': ' .. table.concat(parts, '; '))
  end
  local saved = false
  if created then
    M.save(provider, asset, deps.call)
    saved = true
  end
  return {created = created, saved = saved, compile = compiled}
end

return M
