local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/?.lua;' .. package.path

local bootstrap = require('_bootstrap')
local path = bootstrap.lib('path')
local process = bootstrap.lib('process')
local platform = bootstrap.lib('platform')
local stage_contract = require('moba_stage_contract')

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

local function verify_stage_consistency()
  local report=stage_contract.verify(stage_contract.bindings(root))
  for _,artifact in ipairs(report.artifacts) do
    print('[moba-ue] '..artifact.role..' stage SHA-256 verified: '..artifact.sha256)
  end
end

if mode == 'verify' then verify_stage_consistency(); return end

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
  require('ue_binary_preflight').require_ready(
    ue_root or os.getenv('UE_5_8_ROOT') or os.getenv('UE_ROOT') or os.getenv('UE_5_7_ROOT') or 'D:/UE5.8',
    path.join(omfue, 'om.uproject'))
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
  '-p', 'base_content', '--features', 'compiled-content-only',
}, root)

run_stage('stage current base_content.dll', platform.lua_executable, {
  path.join(root, 'scripts', 'dev_run_freshness.lua'), '--action', 'stage-dll',
}, root)

run_restart('generate bridge and compile OmGame', 'build')
verify_stage_consistency()
if mode == 'full' then
  start_editor()
  run_stage('verify project-bound BpGeneratorUltimate MCP readiness', platform.lua_executable, {
    path.join(root, 'scripts', 'ue_mcp.lua'), '--wait-ready', '--tool', 'get_pie_status',
  }, root)
  run_stage('validate generated hero Blueprints through Editor MCP', platform.lua_executable, {
    path.join(root, 'scripts', 'ue_validate_blueprints.lua'), '--create-missing',
  }, root)
end
