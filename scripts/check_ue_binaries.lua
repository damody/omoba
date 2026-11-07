local script = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(script:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local bootstrap = require('_bootstrap')
local path = bootstrap.lib('path')
local json = bootstrap.lib('json')
local ue_root = require('_bootstrap').lib('platform').ue_root()
local project = path.join(bootstrap.root, 'omfue', 'om.uproject')
local i = 1
while i <= #arg do
  if arg[i] == '--ue-root' or arg[i] == '--project' then
    local key = arg[i]; i = i + 1; local value = assert(arg[i], key .. ' requires a path')
    if key == '--ue-root' then ue_root = value else project = value end
  else error('unknown argument: ' .. arg[i]) end
  i = i + 1
end
local report = require('ue_binary_preflight').inspect(ue_root, project)
print(json.encode(report))
os.exit(report.ready and 0 or 1)
