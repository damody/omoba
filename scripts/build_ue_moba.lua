local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/?.lua;' .. package.path

local bootstrap = require('_bootstrap')
local path = bootstrap.lib('path')
local process = bootstrap.lib('process')
local platform = bootstrap.lib('platform')
local hash = bootstrap.lib('hash')

local root = bootstrap.root
local omfue = path.join(root, 'omfue')
local restart_exe = path.join(omfue, 'restart', 'target', 'debug', 'om_restart.exe')
local mode = 'build'
local ue_root

local i = 1
while i <= #arg do
  local value = arg[i]
  if value == '--full' then
    mode = 'full'
  elseif value == '--verify-staged-only' then
    mode = 'verify'
  elseif value == '--build-only' then
    mode = 'build'
  elseif value == '--ue-root' then
    i = i + 1
    ue_root = assert(arg[i], '--ue-root requires a path')
  elseif value == '--help' or value == '-h' then
    print('Usage: tools/lua/lua.exe scripts/build_ue_moba.lua [--build-only|--full|--verify-staged-only] [--ue-root PATH]')
    return
  else
    error('unknown argument: ' .. value)
  end
  i = i + 1
end

local function verify_bridge_stage()
  local built = path.join(omfue, 'bridge', 'target', 'debug', 'om_bridge.dll')
  local staged = path.join(omfue, 'Plugins', 'OmRuntime', 'Binaries', 'Win64', 'om_bridge.dll')
  local expected, actual = hash.sha256(built), hash.sha256(staged)
  assert(expected == actual, 'staged bridge DLL differs from current build; rebuild/stage before Unreal acceptance: ' .. staged)
  print('[moba-ue] bridge stage SHA-256 verified: ' .. actual)
end

if mode == 'verify' then verify_bridge_stage(); return end

local function run_stage(label, exe, args, cwd, env)
  print('[moba-ue] ' .. label)
  local result = process.run(exe, args, {cwd = cwd, env = env, check = false})
  if result.stdout and result.stdout ~= '' then print(result.stdout) end
  if result.stderr and result.stderr ~= '' then io.stderr:write(result.stderr) end
  assert(result.exit_code == 0, label .. ' failed with exit code ' .. tostring(result.exit_code))
end

local function run_restart(label, command)
  local restart_args = {command}
  local build_env
  if command == 'build' then
    restart_args[#restart_args + 1] = '--with-bridge'
    -- UBT temporarily enters TEMP when switching between case-equivalent
    -- working directories. Keep that operation off the shared user TEMP.
    local build_temp = path.join(omfue, 'Saved', 'BuildTemp', 'ubt')
    path.mkdir_p(build_temp)
    build_env = {TEMP = build_temp, TMP = build_temp}
  end
  if ue_root then
    restart_args[#restart_args + 1] = '--ue-root'
    restart_args[#restart_args + 1] = ue_root
  end
  run_stage(label, restart_exe, restart_args, omfue, build_env)
end

local function start_editor()
  local stdout = path.join(omfue, 'Saved', 'Logs', 'moba_restart_start_stdout.log')
  local stderr = path.join(omfue, 'Saved', 'Logs', 'moba_restart_start_stderr.log')
  local args = {'start', '--output', 'json'}
  if ue_root then
    args[#args + 1] = '--ue-root'
    args[#args + 1] = ue_root
  end
  print('[moba-ue] launch Unreal Editor')
  local pid = process.spawn(restart_exe, args, {cwd = omfue, stdout = stdout, stderr = stderr})
  assert(process.wait(pid, 30000), 'om_restart start did not finish within 30 seconds')
  local result = path.read(stdout)
  assert(result:find('"success":true', 1, true), 'om_restart start failed: ' .. result .. path.read(stderr))
  print(result)
end

run_stage('build om_restart tool', 'cargo', {
  'build', '--manifest-path', path.join(omfue, 'restart', 'Cargo.toml'),
}, root)

run_stage('ensure BpGeneratorUltimate animation import fix', platform.lua_executable, {
  path.join(root, 'scripts', 'ensure_bpgu_animation_import.lua'),
}, root)

if mode == 'full' then
  run_restart('stop matching Unreal Editor before staging DLLs', 'stop')
end

run_stage('build base_content.dll', 'cargo', {
  'build', '--manifest-path', path.join(root, 'scripts', 'Cargo.toml'),
  '-p', 'base_content', '--features', 'runtime-lua-content',
}, root)

run_stage('stage current base_content.dll', platform.lua_executable, {
  path.join(root, 'scripts', 'dev_run_freshness.lua'), '--action', 'stage-dll',
}, root)

run_restart('generate bridge and compile OmGame', 'build')
verify_bridge_stage()
if mode == 'full' then
  start_editor()
  run_restart('verify BpGeneratorUltimate MCP readiness', 'wait-mcp')
  run_stage('validate generated hero Blueprints through Editor MCP', platform.lua_executable, {
    path.join(root, 'scripts', 'ue_validate_blueprints.lua'), '--create-missing',
  }, root)
end
