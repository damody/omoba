-- Real nested spawn under captured process.run. No UE, game, or shell fallback.
-- Opt-in because it starts the real renderer fixture publisher.
--
-- LIFETIME_SECONDS (20) is the publisher's own deadline. It is long enough that
-- the small amount of work after spawn_owned cannot consume it, and short
-- enough that an inherited capture pipe fails closed: process.run(start) would
-- return only after this exact lifetime has already exited.
-- STARTUP_BUDGET_MS (15000) is only the wait-startup budget after start has
-- returned. It is not a race on how fast start returns. The publisher writes
-- startup.json before its lifetime loop, so a still-living child becomes ready
-- inside this budget. The inherited-pipe failure is the owned_alive check
-- immediately after start, which does not use this budget.
-- STOP_WAIT_MS (8000) is above the fixture cleanup grace (5000) so graceful
-- stop can finish its listener probe and write cleanup.json.
package.path = 'scripts/?.lua;' .. package.path
local b = require('_bootstrap')
assert(arg[1] == '--run-spawn-lifetime' and #arg == 1, 'explicit --run-spawn-lifetime required')
local path, process, json, platform = b.lib('path'), b.lib('process'), b.lib('json'), b.lib('platform')
local lfs = require('lfs')
assert(platform.is_windows, 'owned spawn lifetime requires Windows')

local LIFETIME_SECONDS = 20
local STARTUP_BUDGET_MS = 15000
local STOP_WAIT_MS = 8000
local RUN_ID = 'renderer-spawn-lifetime'
local lua = platform.lua_executable
local workflow = path.join(b.root, 'scripts', 'moba_renderer_fixture.lua')
local fixture = path.join(b.root, 'omoba-client-runtime', 'target', 'debug', 'omoba-renderer-fixture.exe')
assert(path.is_file(lua), 'fixed Lua is missing')
assert(path.is_file(workflow), 'renderer fixture workflow is missing')
assert(path.is_file(fixture), 'fixture binary is missing; build omoba-renderer-fixture before this test')

local parent_root = path.join(b.root, 'target', 'renderer-spawn-lifetime-tests')
path.mkdir_p(parent_root)
local dir
for i = 1, 100 do
  local candidate = path.join(parent_root, 'run-' .. os.time() .. '-' .. i)
  if lfs.mkdir(candidate) then dir = candidate; break end
  assert(path.exists(candidate), 'cannot reserve spawn-lifetime directory')
end
assert(dir, 'spawn-lifetime reservation exhausted')
local evidence = path.join(dir, RUN_ID)
local ownership = path.join(dir, RUN_ID .. '.ownership.json')

local function ownership_from_stdout(text)
  if type(text) ~= 'string' or not text:find('{', 1, true) then return nil end
  local decoded_ok, value = pcall(json.decode, text)
  if not decoded_ok or type(value) ~= 'table' then return nil end
  if value.schema ~= 'omoba-renderer-fixture-ownership/1' then return nil end
  if type(value.executable) ~= 'string' or type(value.creation_token) ~= 'string' or type(value.pid) ~= 'number' then
    return nil
  end
  -- Kept before validate_owned so a later assert can still retire this child.
  return {pid = value.pid, executable = value.executable, creation_token = value.creation_token}
end

local function workflow_args(command_name, extra)
  local values = {
    workflow, command_name,
    '--run-id', RUN_ID,
    '--evidence-dir', evidence,
    '--ownership', ownership,
    '--lifetime-seconds', tostring(LIFETIME_SECONDS),
  }
  for _, value in ipairs(extra or {}) do values[#values + 1] = value end
  return values
end

local identity
local ok, err = xpcall(function()
  local started = process.run(lua, workflow_args('start'), {cwd = b.root, check = false, label = 'renderer fixture start'})
  identity = ownership_from_stdout(started.stdout)
  assert(started.exit_code == 0, 'start failed (' .. tostring(started.exit_code) .. '): ' .. tostring(started.stderr))
  assert(identity, 'start did not publish the original ownership identity')
  identity = process.validate_owned(identity)
  assert(process.owned_alive(identity), 'captured start returned only after the original lifetime exited')
  local ready = process.run(lua, workflow_args('wait-startup', {'--timeout-ms', tostring(STARTUP_BUDGET_MS)}), {
    cwd = b.root, check = false, label = 'renderer fixture startup',
  })
  assert(ready.exit_code == 0, 'readiness failed (' .. tostring(ready.exit_code) .. '): ' .. tostring(ready.stderr))
  assert(process.owned_alive(identity), 'original lifetime exited during readiness')
  local startup = json.decode(ready.stdout)
  assert(startup.schema == 'omoba-renderer-fixture-startup/1', 'unexpected startup schema')
  assert(startup.run_id == RUN_ID and startup.authority == false and startup.critical_input_result == 'never', 'startup identity mismatch')
  assert(type(startup.address) == 'string' and startup.address:match('^127%.0%.0%.1:%d+$') and not startup.address:match(':0$'), 'startup did not bind a real loopback port')
  local stopped = process.run(lua, workflow_args('stop', {'--timeout-ms', tostring(STOP_WAIT_MS)}), {
    cwd = b.root, check = false, label = 'renderer fixture stop',
  })
  assert(stopped.exit_code == 0, 'stop failed (' .. tostring(stopped.exit_code) .. '): ' .. tostring(stopped.stderr))
  assert(process.wait_owned(identity, STOP_WAIT_MS), 'exact original identity remained after stop')
  assert(not process.owned_alive(identity), 'exact original identity still alive after stop')
  local cleanup = json.decode(stopped.stdout)
  assert(cleanup.schema == 'omoba-renderer-fixture-cleanup/1' and cleanup.run_id == RUN_ID, 'stop did not report this run')
  assert(cleanup.listener_released == true and cleanup.authority == false and cleanup.reason == 'shutdown_file', 'stop cleanup was not a graceful release')
  print('PASS nested process.run start returned while creation token ' .. identity.creation_token .. ' was alive; ready within ' .. STARTUP_BUDGET_MS .. ' ms; stopped that identity (lifetime ' .. LIFETIME_SECONDS .. ' s)')
end, debug.traceback)

if identity then
  local clean, why = pcall(function()
    if process.owned_alive(identity) then process.stop_owned(identity) end
    assert(process.wait_owned(identity, STOP_WAIT_MS), 'owned publisher remained after failure retirement')
    assert(not process.owned_alive(identity), 'owned publisher cleanup failed')
  end)
  if not clean then ok = false; err = tostring(err) .. '\n' .. tostring(why) end
end
if not ok then error(tostring(err) .. '\nretained directory: ' .. dir, 0) end

local function remove_tree(directory)
  for name in lfs.dir(directory) do
    if name ~= '.' and name ~= '..' then
      local child = path.join(directory, name)
      if lfs.attributes(child, 'mode') == 'directory' then
        remove_tree(child)
      else
        assert(os.remove(child), 'cannot remove ' .. child)
      end
    end
  end
  assert(lfs.rmdir(directory), 'cannot remove ' .. directory)
end
remove_tree(dir)
