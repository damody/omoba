-- Locates the BpGeneratorUltimate (UECP) plugin under omfue/Plugins by its descriptor.
-- UE loads a plugin by descriptor name, not folder name (the vfx_mcp kit installs it as
-- BpGeneratorUltimate_1.7.3), so build steps must not assume a folder (E104).
local M = {}

function M.locate(root, path)
  local lfs = require('lfs')
  local plugins = path.join(root, 'omfue', 'Plugins')
  local found
  for name in lfs.dir(plugins) do
    local candidate = path.join(plugins, name)
    if name ~= '.' and name ~= '..' and path.is_file(path.join(candidate, 'BpGeneratorUltimate.uplugin')) then
      assert(not found, 'more than one BpGeneratorUltimate plugin under omfue/Plugins: ' .. tostring(found) .. ', ' .. candidate)
      found = candidate
    end
  end
  assert(found, 'BpGeneratorUltimate plugin missing under omfue/Plugins (om.uproject enables it)')
  local descriptor = path.read(path.join(found, 'BpGeneratorUltimate.uplugin'))
  return {
    dir = found,
    -- Relative to omfue/, for callers that build paths from the project directory.
    relative = 'Plugins/' .. assert(found:match('[/\\]([^/\\]+)$')),
    descriptor = descriptor,
    version = descriptor:match('"VersionName"%s*:%s*"([^"]+)"') or '?',
    engine = descriptor:match('"EngineVersion"%s*:%s*"([^"]+)"'),
    -- An installed (precompiled) package is never rebuilt by UBT, so generated or
    -- patched plugin sources have no effect on it.
    installed = descriptor:match('"Installed"%s*:%s*true') ~= nil,
  }
end

return M
