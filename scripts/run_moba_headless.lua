-- Authoritative single-lane acceptance. No Editor or renderer is required.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = assert(script:match('^(.*)[/\\]'))
package.path = dir .. '/?.lua;' .. package.path
local bootstrap = require('_bootstrap')
local path, process = bootstrap.lib('path'), bootstrap.lib('process')
local root = bootstrap.root
local function run(label, args)
  print('[moba-headless] ' .. label)
  local result = process.run('cargo', args, {cwd = root, check = false})
  io.write(result.stdout or '')
  io.stderr:write(result.stderr or '')
  if result.exit_code ~= 0 then os.exit(result.exit_code) end
end
run('build generated script DLL', {'build', '--manifest-path', path.join(root, 'scripts/Cargo.toml'),
  '-p', 'base_content', '--release'})
local args = {'run', '--manifest-path', path.join(root, 'omb/Cargo.toml'),
  '-p', 'omobab', '--bin', 'moba-headless', '--',
  '--scripts-dir', path.join(root, 'scripts/target/release'),
  '--report', path.join(root, 'omb/target/moba-headless/report.json')}
for _, value in ipairs(arg) do args[#args + 1] = value end
run('play a match and replay every tick', args)
