-- Read-only launch readiness, not proof of C++ ABI or runtime acceptance.
local path = require('tools.lua.lib.path')
local json = require('tools.lua.lib.json')
local lfs = require('lfs')
local M = {}

function M.inspect(ue_root, project_file, io_api)
  io_api = io_api or {
    read = path.read, is_file = path.is_file,
    plugin_dirs = function(root)
      local dirs = {}
      if path.is_directory(root) then
        for name in lfs.dir(root) do
          if name ~= '.' and name ~= '..' and path.is_directory(path.join(root, name)) then
            dirs[#dirs + 1] = name
          end
        end
      end
      table.sort(dirs)
      return dirs
    end,
  }
  local report = {ready = false, manifests = {}, errors = {}, scope = 'Win64 UnrealEditor local project plugins'}
  local function issue(message) report.errors[#report.errors + 1] = message end
  local function decode(file)
    if not io_api.is_file(file) then issue('missing file: ' .. file); return end
    local ok, value = pcall(function()
      -- Unreal descriptors may have a UTF-8 BOM; preserve the file itself.
      local bytes = io_api.read(file)
      if bytes:sub(1, 3) == '\239\187\191' then bytes = bytes:sub(4) end
      return json.decode(bytes)
    end)
    if not ok or type(value) ~= 'table' or value == json.null then
      issue('invalid JSON object: ' .. file); return
    end
    return value
  end
  local engine_file = path.join(ue_root, 'Engine', 'Binaries', 'Win64', 'UnrealEditor.modules')
  local engine = decode(engine_file)
  local engine_id = engine and engine.BuildId
  if type(engine_id) ~= 'string' or engine_id == '' then issue('engine BuildId unavailable: ' .. engine_file); engine_id = nil end
  report.engine_build_id = engine_id
  local function manifest(file, label, required_modules)
    local value = decode(file)
    if not value then return end
    report.manifests[#report.manifests + 1] = {path = file, label = label, build_id = value.BuildId}
    if type(value.BuildId) ~= 'string' or value.BuildId == '' then
      issue(label .. ': missing BuildId')
    elseif engine_id and value.BuildId ~= engine_id then
      issue(label .. ': BuildId mismatch (engine=' .. engine_id .. ', binary=' .. value.BuildId .. ')')
    end
    if type(value.Modules) ~= 'table' or value.Modules == json.null then issue(label .. ': missing Modules object'); return end
    for _, module in ipairs(required_modules or {}) do
      if type(module) ~= 'table' or type(module.Name) ~= 'string' or not value.Modules[module.Name] then
        issue(label .. ': declared module absent from manifest: ' .. tostring(type(module) == 'table' and module.Name or module))
      end
    end
    for name, dll in pairs(value.Modules) do
      if type(name) ~= 'string' or type(dll) ~= 'string' or not dll:match('^[%w_.%-]+%.dll$') then
        issue(label .. ': unsafe module filename')
      elseif not io_api.is_file(path.join(path.parent(file), dll)) then
        issue(label .. ': missing module binary: ' .. dll)
      end
    end
  end
  local project = decode(project_file)
  if not project then return report end
  local project_root = path.parent(project_file)
  manifest(path.join(project_root, 'Binaries', 'Win64', 'UnrealEditor.modules'), 'project', project.Modules)
  local plugins_root = path.join(project_root, 'Plugins')
  local local_plugins, explicit, enabled = {}, {}, {}
  for _, name in ipairs(io_api.plugin_dirs(plugins_root)) do
    local file = path.join(plugins_root, name, name .. '.uplugin')
    if io_api.is_file(file) then local_plugins[name] = {file = file, root = path.parent(file)} end
  end
  for _, entry in ipairs(project.Plugins or {}) do
    if type(entry) ~= 'table' or type(entry.Name) ~= 'string' or type(entry.Enabled) ~= 'boolean' then
      issue('invalid project plugin declaration')
    else explicit[entry.Name] = entry.Enabled; enabled[entry.Name] = entry.Enabled end
  end
  for name, plugin in pairs(local_plugins) do
    if explicit[name] ~= false then plugin.descriptor = decode(plugin.file) end
    if plugin.descriptor and explicit[name] == nil and plugin.descriptor.EnabledByDefault == true then enabled[name] = true end
  end
  local visited = {}
  local function visit(name)
    if visited[name] or not enabled[name] then return end
    visited[name] = true
    local plugin = local_plugins[name]
    if not plugin or not plugin.descriptor then return end -- Engine plugins remain UBT/Editor's responsibility.
    local descriptor = plugin.descriptor
    if descriptor.Modules and #descriptor.Modules > 0 then
      manifest(path.join(plugin.root, 'Binaries', 'Win64', 'UnrealEditor.modules'), 'plugin ' .. name, descriptor.Modules)
    end
    for _, dependency in ipairs(descriptor.Plugins or {}) do
      if dependency.Enabled == true and explicit[dependency.Name] ~= false then
        enabled[dependency.Name] = true; visit(dependency.Name)
      end
    end
  end
  local names = {}; for name in pairs(local_plugins) do names[#names + 1] = name end; table.sort(names)
  for _, name in ipairs(names) do visit(name) end
  report.ready = #report.errors == 0
  return report
end

function M.require_ready(ue_root, project_file)
  local report = M.inspect(ue_root, project_file)
  assert(report.ready, 'Unreal binary preflight failed; rebuild against the selected engine with UBT (never edit BuildId):\n' .. table.concat(report.errors, '\n'))
  return report
end
return M
