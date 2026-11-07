-- Workflow owner for the synthetic renderer fixture publisher.
-- Fixture data stays in omoba-renderer-fixture. This script only builds,
-- starts, waits, cancels, and stops. It does not launch gameplay, UE, or KCP.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, process, args, time, platform =
  b.lib('path'), b.lib('json'), b.lib('process'), b.lib('args'), b.lib('time'), b.lib('platform')

local STARTUP = 'omoba-renderer-fixture-startup/1'
local CONTROL = 'omoba-renderer-fixture-control/1'
local ACK = 'omoba-renderer-fixture-control-ack/1'
local CLEANUP = 'omoba-renderer-fixture-cleanup/1'
local OWNERSHIP = 'omoba-renderer-fixture-ownership/1'
local STAGES = {
  combat_defaults = true, combat_cursor_projection = true, combat_idle_off_projection = true,
  input_point = true, input_keyboard = true, input_locked = true,
  communication_hud = true, communication_ready = true, communication_retire = true, communication_received = true,
  communication_signals = true, communication_signals_expired = true,
  baseline = true, remove_target = true, replace_target = true, source_removal = true,
  source_replacement = true, source_death = true, death = true, disclosure_epoch = true,
  hud_cooldown = true,
  owner_cast_ready = true, owner_upgrade_point = true, owner_short_mana = true,
  owner_cooldown = true, owner_hide_target = true, owner_far_unit = true,
  owner_apprentice = true, owner_point_ready = true,
  owner_support_ready = true, owner_support_enemy = true,
  owner_support_team_cards = true, owner_support_team_cards_dead = true,
  owner_support_conceal = true, owner_support_dead = true, owner_support_neutral = true,
  owner_summoner_ready = true, owner_summoner_cooldown = true,
  owner_summoner_unavailable = true, owner_summoner_malformed = true,
  owner_summoner_unsupported = true,
  mode_tower_defense = true, mode_td_defeat = true, mode_td_no_hero = true,
  mode_moba_dead = true, mode_unavailable = true, mode_td_epoch = true,
  owner_shop_ready = true,
  controlled_owner_shop_current = true,
  controlled_owner_shop_admitted = true,
  controlled_owner_shop_uncertain = true,
  controlled_owner_shop_settled = true,
  controlled_owner_shop_rejected = true,
  controlled_owner_shop_stale = true,
  controlled_owner_shop_epoch = true,
  controlled_owner_shop_full = true,
}
-- Covers the publisher's 2-second listener probe. This is retirement time only.
local CLEANUP_GRACE_MS = 5000

local function usage()
  io.stderr:write([[
Usage: tools\lua\lua.exe scripts\moba_renderer_fixture.lua COMMAND [options]
  check
  build
  test-lib
  start --run-id ID --evidence-dir DIR\ID [--lifetime-seconds 1..1800] [--shutdown-file PATH] [--ownership PATH]
  wait-startup --run-id ID --evidence-dir DIR\ID [--ownership PATH] [--timeout-ms N]
  control --run-id ID --evidence-dir DIR\ID --sequence N --stage STAGE
  wait-ack --run-id ID --evidence-dir DIR\ID --sequence N [--timeout-ms N] [--ownership PATH]
  stop --run-id ID --evidence-dir DIR\ID [--timeout-ms N] [--ownership PATH]
  cancel --run-id ID --evidence-dir DIR\ID [--timeout-ms N] [--ownership PATH]
  supervise --run-id ID --evidence-dir DIR\ID [--lifetime-seconds 1..1800]
Default lifetime is 900 seconds and is never extended.
After that operation deadline, supervise allows 5000 ms only to observe retirement.
start does not enter process supervision: a supervisor would retire the publisher when start returns.
supervise is the long-running owner. Graceful stop requires listener_released true.
cancel only confirms process retirement; missing or false cleanup is not success.
]])
end

local function cargo(arguments, label)
  process.run('cargo', arguments, {cwd = b.root, label = label})
end

local function manifest()
  return path.join(b.root, 'omoba-client-runtime', 'Cargo.toml')
end

local function binary_path()
  local name = platform.is_windows and 'omoba-renderer-fixture.exe' or 'omoba-renderer-fixture'
  return require('run_artifacts').rust_binary(b.root, 'omoba-client-runtime', 'debug', name)
end

local function valid_run_id(value)
  return type(value) == 'string' and #value >= 1 and #value <= 64 and value:match('^[%w][%w%._%-]*$') ~= nil
end

local function run_options(values)
  local options = args.parse(values)
  local run_id = args.required(options, 'run-id')
  assert(valid_run_id(run_id), 'invalid --run-id')
  local evidence = path.absolute(args.required(options, 'evidence-dir'))
  assert(evidence:match('([^/\\]+)$') == run_id, 'evidence directory name must equal --run-id')
  local lifetime = options['lifetime-seconds'] and args.integer(options['lifetime-seconds'], 'lifetime-seconds', 1, 1800) or 900
  local shutdown = options['shutdown-file'] and path.absolute(options['shutdown-file']) or path.join(evidence, 'shutdown.signal')
  local ownership = options.ownership and path.absolute(options.ownership)
    or path.join(path.parent(evidence), run_id .. '.ownership.json')
  return options, run_id, evidence, lifetime, shutdown, ownership
end

local function atomic_write(dest, text)
  local tmp = dest .. '.partial'
  local file = assert(io.open(tmp, 'wb'))
  assert(file:write(text))
  assert(file:flush())
  assert(file:close())
  os.remove(dest)
  assert(os.rename(tmp, dest), 'atomic rename failed: ' .. dest)
end

local function load_ownership(ownership, run_id, evidence)
  local record = json.decode(path.read(ownership, true))
  assert(record.schema == OWNERSHIP, 'unexpected ownership schema')
  assert(record.run_id == run_id and record.evidence_dir == evidence, 'ownership does not match this run')
  return record, {pid = record.pid, executable = record.executable, creation_token = record.creation_token}
end

local function read_json(file)
  return json.decode(path.read(file, true))
end

local function command_check()
  cargo({
    'check', '--manifest-path', manifest(), '-p', 'omoba-client-runtime',
    '--all-targets', '--features', 'compiled-content-only',
  }, 'renderer fixture check')
end

local function command_build()
  cargo({
    'build', '--manifest-path', manifest(), '-p', 'omoba-client-runtime',
    '--bin', 'omoba-renderer-fixture', '--features', 'compiled-content-only',
  }, 'renderer fixture build')
end

local function command_test_lib()
  cargo({
    'test', '--manifest-path', manifest(), '-p', 'omoba-client-runtime',
    '--features', 'compiled-content-only', '--lib', '--',
    'presentation_bridge::tests', 'renderer_fixture::tests',
  }, 'renderer fixture lib tests')
  cargo({
    'test', '--manifest-path', manifest(), '-p', 'omoba-client-runtime',
    '--features', 'compiled-content-only', '--bin', 'omoba-renderer-fixture',
  }, 'renderer fixture bin tests')
end

local function spawn_fixture(run_id, evidence, lifetime, shutdown, ownership)
  assert(not path.exists(evidence), 'evidence directory must be a fresh path')
  assert(path.is_directory(path.parent(evidence)), 'evidence parent directory must already exist')
  assert(not path.exists(shutdown), 'shutdown file must not exist before start')
  assert(not path.exists(ownership), 'ownership file already exists for this run id')
  local built = binary_path()
  assert(path.is_file(built), 'fixture binary is missing; run the build command first')
  -- Run a copy: a running fixture locked target/debug and other sessions' builds failed (E125).
  local name = built:match('([^/\\]+)$')
  local exe = require('run_artifacts').stage_binaries(b.root, {{name = name, source = built}}).files[name]
  local parent = path.parent(evidence)
  local stdout = path.join(parent, run_id .. '.stdout.log')
  local stderr = path.join(parent, run_id .. '.stderr.log')
  local _, owned = process.spawn_owned(exe, {
    '--run-id', run_id,
    '--evidence-dir', evidence,
    '--lifetime-seconds', tostring(lifetime),
    '--shutdown-file', shutdown,
  }, {cwd = b.root, stdout = stdout, stderr = stderr})
  local record = json.object({
    schema = OWNERSHIP,
    run_id = run_id,
    evidence_dir = evidence,
    lifetime_seconds = lifetime,
    shutdown_file = shutdown,
    stdout = stdout,
    stderr = stderr,
    pid = owned.pid,
    executable = owned.executable,
    creation_token = owned.creation_token,
  })
  json.write(ownership, record, true)
  print(json.encode(record))
  return record, owned
end

local function wait_startup(owned, evidence, run_id, timeout_ms)
  local startup = path.join(evidence, 'startup.json')
  return process.poll_owned_ready(owned, timeout_ms, function()
    if not path.is_file(startup) then return nil end
    local value = read_json(startup)
    assert(value.schema == STARTUP, 'unexpected startup schema')
    assert(value.run_id == run_id, 'startup run id mismatch')
    assert(value.authority == false and value.critical_input_result == 'never', 'startup claimed authority')
    assert(type(value.address) == 'string' and value.address:match('^127%.0%.0%.1:%d+$') ~= nil, 'startup address is not loopback')
    assert(value.address:match('^127%.0%.0%.1:0$') == nil, 'startup port is not a real bound port')
    assert(type(value.content_hash) == 'string' and value.content_hash ~= '', 'startup is missing content hash')
    assert(math.type(value.source_render_id) == 'integer' and value.source_render_id > 0, 'startup source render id is invalid')
    assert(math.type(value.target_render_id) == 'integer' and value.target_render_id > 0, 'startup target render id is invalid')
    return value
  end, 'renderer fixture')
end

local function command_start(values)
  local _, run_id, evidence, lifetime, shutdown, ownership = run_options(values)
  spawn_fixture(run_id, evidence, lifetime, shutdown, ownership)
end

local function command_wait_startup(values)
  local options, run_id, evidence, lifetime, _, ownership = run_options(values)
  local _, owned = load_ownership(ownership, run_id, evidence)
  local timeout = options['timeout-ms'] and args.integer(options['timeout-ms'], 'timeout-ms', 0, lifetime * 1000) or lifetime * 1000
  local startup = wait_startup(owned, evidence, run_id, timeout)
  print(json.encode(startup))
end

local function command_control(values)
  local options, run_id, evidence = run_options(values)
  assert(path.is_directory(evidence), 'evidence directory does not exist')
  local sequence = args.integer(args.required(options, 'sequence'), 'sequence', 1, 2^53)
  local stage = args.choice(args.required(options, 'stage'), 'stage', {
    'baseline', 'remove_target', 'replace_target', 'source_removal',
    'source_replacement', 'source_death', 'death', 'disclosure_epoch', 'hud_cooldown',
    'owner_cast_ready', 'owner_upgrade_point', 'owner_short_mana', 'owner_cooldown',
    'owner_hide_target', 'owner_far_unit', 'owner_apprentice', 'owner_point_ready',
    'owner_support_ready', 'owner_support_enemy', 'owner_support_conceal', 'owner_support_dead',
    'owner_support_neutral',
    'owner_summoner_ready', 'owner_summoner_cooldown', 'owner_summoner_unavailable',
    'owner_summoner_malformed', 'owner_summoner_unsupported',
    'mode_tower_defense', 'mode_td_defeat', 'mode_td_no_hero',
    'mode_moba_dead', 'mode_unavailable', 'mode_td_epoch',
    'owner_shop_ready',
    'controlled_owner_shop_current', 'controlled_owner_shop_admitted',
    'controlled_owner_shop_uncertain', 'controlled_owner_shop_settled',
    'controlled_owner_shop_rejected', 'controlled_owner_shop_stale',
    'controlled_owner_shop_epoch',
    'controlled_owner_shop_full',
    'communication_hud', 'communication_ready', 'communication_retire', 'communication_received', 'communication_signals', 'communication_signals_expired',
  })
  assert(STAGES[stage], 'unknown stage')
  local payload = json.object({
    schema = CONTROL, run_id = run_id, sequence = sequence, stage = stage,
  })
  atomic_write(path.join(evidence, 'control.json'), json.encode(payload) .. '\n')
  print(json.encode(payload))
end

local function command_wait_ack(values)
  local options, run_id, evidence, lifetime, _, ownership = run_options(values)
  local _, owned = load_ownership(ownership, run_id, evidence)
  local sequence = args.integer(args.required(options, 'sequence'), 'sequence', 1, 2^53)
  local cap = lifetime * 1000
  local timeout = options['timeout-ms'] and args.integer(options['timeout-ms'], 'timeout-ms', 0, cap) or math.min(5000, cap)
  local ack_path = path.join(evidence, 'control.ack.json')
  local ack = process.poll_owned_ready(owned, timeout, function()
    if not path.is_file(ack_path) then return nil end
    local value = read_json(ack_path)
    if value.sequence ~= sequence then return nil end
    assert(value.schema == ACK and value.run_id == run_id, 'control ack does not match this run')
    assert(value.authority_ack == false, 'control ack claimed authority')
    return value
  end, 'renderer fixture control ack')
  print(json.encode(ack))
end

local function read_cleanup(evidence, run_id)
  local cleanup_path = path.join(evidence, 'cleanup.json')
  if not path.is_file(cleanup_path) then return nil end
  local cleanup = read_json(cleanup_path)
  assert(cleanup.schema == CLEANUP and cleanup.run_id == run_id, 'cleanup does not match this run')
  assert(cleanup.authority == false, 'cleanup claimed authority')
  return cleanup
end

local function require_graceful_cleanup(evidence, run_id)
  local cleanup = read_cleanup(evidence, run_id)
  assert(cleanup, 'missing cleanup.json is not success')
  assert(cleanup.listener_released == true, 'listener_released=false is not a successful stop')
  return cleanup
end

local function command_stop(values)
  local options, run_id, evidence, _, shutdown, ownership = run_options(values)
  local _, owned = load_ownership(ownership, run_id, evidence)
  local timeout = options['timeout-ms'] and args.integer(options['timeout-ms'], 'timeout-ms', 0, 1800 * 1000) or CLEANUP_GRACE_MS
  atomic_write(shutdown, 'stop\n')
  if not process.wait_owned(owned, timeout) then
    process.stop_owned(owned)
    assert(process.wait_owned(owned, timeout), 'fixture remained after stop_owned')
    error('graceful stop did not retire the fixture within the cleanup allowance')
  end
  print(json.encode(require_graceful_cleanup(evidence, run_id)))
end

local function command_cancel(values)
  local options, run_id, evidence, _, _, ownership = run_options(values)
  local _, owned = load_ownership(ownership, run_id, evidence)
  local timeout = options['timeout-ms'] and args.integer(options['timeout-ms'], 'timeout-ms', 0, 1800 * 1000) or CLEANUP_GRACE_MS
  process.stop_owned(owned)
  assert(process.wait_owned(owned, timeout), 'fixture remained after cancel')
  assert(not process.owned_alive(owned), 'original fixture lifetime was not retired')
  local cleanup = read_cleanup(evidence, run_id)
  if cleanup then print(json.encode(cleanup)) end
  if not cleanup or cleanup.listener_released ~= true then
    error('cancel retired the fixture; graceful cleanup proof is missing and is not success')
  end
end

local function remaining_ms(started, deadline)
  local now = time.monotonic_ms()
  assert(now >= started, 'supervise clock moved backwards')
  local left = deadline - now
  if left < 0 then return 0 end
  return left
end

local function command_supervise(values)
  process.supervise_workflow(source, arg)
  local _, run_id, evidence, lifetime, shutdown, ownership = run_options(values)
  local _, owned = spawn_fixture(run_id, evidence, lifetime, shutdown, ownership)
  local started = time.monotonic_ms()
  local operation_deadline = started + lifetime * 1000
  wait_startup(owned, evidence, run_id, remaining_ms(started, operation_deadline))
  if not process.wait_owned(owned, remaining_ms(started, operation_deadline)) then
    if not process.wait_owned(owned, CLEANUP_GRACE_MS) then
      process.stop_owned(owned)
      assert(process.wait_owned(owned, CLEANUP_GRACE_MS), 'fixture remained after the cleanup allowance')
      error('publisher did not retire within the cleanup allowance')
    end
  end
  print(json.encode(require_graceful_cleanup(evidence, run_id)))
end

local M = {
  build = command_build,
  binary_path = binary_path,
  spawn_fixture = spawn_fixture,
  wait_startup = wait_startup,
  read_cleanup = read_cleanup,
  require_graceful_cleanup = require_graceful_cleanup,
  valid_run_id = valid_run_id,
  schemas = {
    startup = STARTUP, control = CONTROL, ack = ACK, cleanup = CLEANUP, ownership = OWNERSHIP,
  },
}

local function same_path(left, right)
  left, right = path.absolute(left), path.absolute(right)
  if platform.is_windows then return left:lower() == right:lower() end
  return left == right
end

-- Requiring this file must not dispatch the caller's arg table.
local function is_entry()
  local entry = arg and arg[0]
  return type(entry) == 'string' and same_path(entry, source)
end

if is_entry() then
  local command = arg[1]
  if command == 'supervise' then
    command_supervise({table.unpack(arg, 2)})
  elseif command == 'check' then command_check()
  elseif command == 'build' then command_build()
  elseif command == 'test-lib' then command_test_lib()
  elseif command == 'start' then command_start({table.unpack(arg, 2)})
  elseif command == 'wait-startup' then command_wait_startup({table.unpack(arg, 2)})
  elseif command == 'control' then command_control({table.unpack(arg, 2)})
  elseif command == 'wait-ack' then command_wait_ack({table.unpack(arg, 2)})
  elseif command == 'stop' then command_stop({table.unpack(arg, 2)})
  elseif command == 'cancel' then command_cancel({table.unpack(arg, 2)})
  else
    usage()
    os.exit(command == nil and 0 or 2)
  end
end

return M
