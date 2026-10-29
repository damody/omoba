local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/?.lua;' .. package.path

local bootstrap = require('_bootstrap')
local path = bootstrap.lib('path')
local process = bootstrap.lib('process')
local platform = bootstrap.lib('platform')
local stage_contract = require('moba_stage_contract')
local engine_bootstrap = require('ue_engine_bootstrap')

local root = bootstrap.root
local omfue = path.join(root, 'omfue')
local restart_exe = path.join(omfue, 'restart', 'target', 'debug', 'om_restart.exe')
local mode = 'build'
local ue_root
local build_executor = 'default'
local disable_action_cache = false
local prefetch_shared_pch = false
local bootstrap_engine, refresh_engine, max_parallel_actions, compiler_version

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
  elseif value == '--build-executor' then
    i = i + 1
    build_executor = assert(arg[i], '--build-executor requires default or local')
    assert(build_executor == 'default' or build_executor == 'local', '--build-executor requires default or local')
  elseif value == '--disable-action-cache' then
    disable_action_cache = true
  elseif value == '--prefetch-shared-pch' then
    prefetch_shared_pch = true
  elseif value == '--refresh-engine' then
    bootstrap_engine = true
    refresh_engine = true
  elseif value == '--bootstrap-engine' then
    bootstrap_engine = true
  elseif value == '--max-parallel-actions' then
    i = i + 1
    max_parallel_actions = engine_bootstrap.parallel_actions(arg[i])
  elseif value == '--compiler-version' then
    i = i + 1
    compiler_version = engine_bootstrap.compiler_version(arg[i])
  elseif value == '--help' or value == '-h' then
    print('Optional local compiler selection: --compiler-version Latest|Preview|MSVC_VERSION (applies to all UE build stages)')
    print('Usage: tools/lua/lua.exe scripts/build_ue_moba.lua [--build-only|--full|--verify-staged-only] [--ue-root PATH] [--build-executor default|local] [--disable-action-cache] [--prefetch-shared-pch] [--max-parallel-actions 1..64] [--bootstrap-engine|--refresh-engine]')
    return
  else
    error('unknown argument: ' .. value)
  end
  i = i + 1
end

assert(build_executor ~= 'local' or mode ~= 'verify',
  '--build-executor local cannot be combined with verification')
assert(not disable_action_cache or (build_executor == 'local' and mode ~= 'verify'),
  '--disable-action-cache requires a local build')
assert(not max_parallel_actions or (build_executor == 'local' and mode ~= 'verify'),
  '--max-parallel-actions requires a local build')
assert(not prefetch_shared_pch or (build_executor == 'local' and mode ~= 'verify'),
  '--prefetch-shared-pch requires a local build')
assert(not compiler_version or (build_executor == 'local' and mode ~= 'verify'),
  '--compiler-version requires a local build')
if mode == 'build' then process.supervise_workflow(script, arg) end
-- Concurrent build_ue_moba runs overwrote each other's staged DLLs (E106): queue on a
-- repo-wide lock, taken after supervise_workflow so only the supervised child holds it.
local build_lock <close> = require('build_lock').acquire(root, mode .. ' ' .. table.concat(arg, ' '))

local function verify_stage_consistency()
  -- A build verifies what it staged into live omfue; --verify-staged-only checks the binaries
  -- a launch will run (OM_UE_RUN_BINARIES when a session runs its own shadow build).
  -- A launch runs its own base_content copy and checks the bridge against its stage record
  -- (E124), so another session's rebuild never blocks it; a build checks what it just staged.
  local report=mode=='build' and stage_contract.verify(stage_contract.bindings(root,path.join(root,'omfue')))
    or require('ue_bridge_stage').verify_launch(root)
  require('ue_map_biome_materials').require_ready(root)
  require('ue_hud_portrait_coverage').require_ready(root)
  require('voice_audio_import').require_ready(root)
  for _,artifact in ipairs(report.artifacts) do
    print('[moba-ue] '..artifact.role..' stage SHA-256 verified: '..artifact.sha256)
  end
end

if mode == 'verify' then
  assert(not bootstrap_engine, '--bootstrap-engine cannot be used with --verify-staged-only')
  verify_stage_consistency()
  return
end
local engine_root = path.absolute(ue_root or os.getenv('UE_5_8_ROOT') or os.getenv('UE_ROOT') or os.getenv('UE_5_7_ROOT') or 'D:/UE5.8')
local editor_exe = path.join(engine_root, 'Engine/Binaries/Win64/UnrealEditor.exe')
local engine_build = path.join(engine_root, 'Engine/Build/BatchFiles/Build.bat')
local bootstrap_marker = path.join(omfue,'Saved/BuildTemp/source-engine-bootstrap.json')
local json = bootstrap.lib('json')
local pending = path.is_file(bootstrap_marker) and json.read(bootstrap_marker)
if pending then
  assert(type(pending)=='table' and type(pending.engine_root)=='string', 'invalid source-engine bootstrap checkpoint')
end
local engine_plan = engine_bootstrap.plan{
	build_executor=build_executor,
  compiler_version=compiler_version,
  disable_action_cache=disable_action_cache,
  requested=bootstrap_engine, refresh=refresh_engine, parallel=bootstrap_engine and max_parallel_actions or nil, mode=mode,
  editor_present=path.is_file(editor_exe),
  source_present=path.is_file(path.join(engine_root,'Engine/Source/UnrealEditor.Target.cs'))
    and not path.is_file(path.join(engine_root,'Engine/Build/InstalledBuild.txt')),
  build_present=path.is_file(engine_build), project=path.join(omfue,'om.uproject'),
  initialization_pending=pending and path.absolute(pending.engine_root):lower()==engine_root:lower(),
}
if engine_plan.initialize and pending then
  assert(path.absolute(pending.engine_root):lower()==engine_root:lower(),
    'finish the existing source-engine bootstrap checkpoint before initializing another engine')
end
-- Every later stage must use the same engine selected for admission.
ue_root = engine_root
local cbindgen_version = require('ue_build_prerequisites').require_ready(process)
print('[moba-ue] cbindgen prerequisite ready: '..cbindgen_version)

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
    -- Keep both executors on the same clone-local Lua bridge pipeline.
    run_stage('generate and stage bridge', platform.lua_executable,
      {path.join(root,'scripts/build_ue_bridge.lua')}, root)
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
run_stage('generate native MCP static dispatch catalog', platform.lua_executable, {
  path.join(root,'scripts/generate_ue_mcp_catalog.lua'),
}, root)
run_stage('synchronize original HUD content names', platform.lua_executable, {
  path.join(root,'scripts/sync_ue_hud_manifest_labels.lua'),
}, root)

if mode == 'full' and path.is_file(editor_exe) then
  run_restart('stop matching Unreal Editor before staging DLLs', 'stop')
end

run_stage('build script DLLs (base_content and hero DLLs)', 'cargo', require('script_dlls').cargo_args({
  'build', '--manifest-path', path.join(root, 'scripts', 'Cargo.toml'),
}, 'compiled-content-only'), root)

run_stage('stage current base_content.dll', platform.lua_executable, {
  path.join(root, 'scripts', 'dev_run_freshness.lua'), '--action', 'stage-dll',
}, root)

if engine_plan.initialize then
  -- A late interrupted source build may have linked Editor.exe already. Keep
  -- an explicit checkpoint so a retry can finish the initial engine graph.
  path.mkdir_p(path.parent(bootstrap_marker))
  json.write(bootstrap_marker,{engine_root=engine_root,project=path.join(omfue,'om.uproject')},true)
  run_stage('generate and stage bridge before source-engine initialization', platform.lua_executable, {path.join(root,'scripts/build_ue_bridge.lua')}, omfue)
  local build_temp = path.join(omfue,'Saved/BuildTemp/ubt')
  path.mkdir_p(build_temp)
  run_stage('build source-engine ShaderCompileWorker prerequisite', engine_build, engine_plan.shader_args, omfue, {TEMP=build_temp,TMP=build_temp})
  run_stage('initialize required engine modules with OmGameEditor', engine_build, engine_plan.args, omfue, {TEMP=build_temp,TMP=build_temp})
  assert(path.is_file(editor_exe), 'engine initialization completed without UnrealEditor.exe')
  assert(path.is_file(path.join(engine_root,'Engine/Binaries/Win64/ShaderCompileWorker.exe')),
    'engine initialization completed without ShaderCompileWorker.exe')
end
-- The initialized engine goes through the ordinary NoEngineChanges build gate.
if build_executor == 'local' then
  local build_temp = path.join(omfue, 'Saved/BuildTemp/ubt')
  path.mkdir_p(build_temp)
  local build_env = {TEMP = build_temp, TMP = build_temp}
  run_stage('generate and stage bridge', platform.lua_executable, {path.join(root,'scripts/build_ue_bridge.lua')}, omfue, build_env)
  if prefetch_shared_pch then
    local result=require('ue_shared_pch_preflight').run(omfue,engine_root,bootstrap.lib('time').monotonic_ms,function(row)
      print(('[moba-ue] read-only PCH prefetch: %s; %d bytes; %d ms'):format(row.path,row.bytes,row.elapsed_ms))
    end)
    print(('[moba-ue] PCH prefetch: %d files, %d bytes, %d over-budget skips; not an integrity check'):format(result.files,result.bytes,result.skipped))
  end
  local local_build_args = engine_bootstrap.ordinary_build_args{
    project=path.join(omfue,'om.uproject'), parallel=max_parallel_actions, disable_action_cache=disable_action_cache,
    compiler_version=compiler_version}
  -- UE5.8 always uses the UBA local executor; NoUBA disables detours, not
  -- its action cache. This explicit per-run option isolates cache stalls.
  run_stage('compile OmGame with local executor', engine_build, local_build_args, omfue, build_env)
else
  run_restart('compile OmGame', 'build')
end
run_stage('ensure public map biome materials',platform.lua_executable,{
  path.join(root,'scripts/ue_map_biome_materials.lua'),'--ue-root',engine_root,'--ensure',
},root)
run_stage('ensure verified CC0 match voice',platform.lua_executable,{
  path.join(root,'scripts/import_moba_voice.lua'),'--ue-root',engine_root,'--ensure',
},root)
verify_stage_consistency()
if engine_plan.initialize then assert(os.remove(bootstrap_marker), 'cannot retire source-engine bootstrap checkpoint') end
if mode == 'full' then
  start_editor()
  run_stage('verify project-bound BpGeneratorUltimate MCP readiness', platform.lua_executable, {
    path.join(root, 'scripts', 'ue_mcp.lua'), '--wait-ready', '--tool', 'get_pie_status',
  }, root)
  run_stage('validate generated hero Blueprints through Editor MCP', platform.lua_executable, {
    path.join(root, 'scripts', 'ue_validate_blueprints.lua'), '--create-missing',
  }, root)
end
