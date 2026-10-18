package.path = 'scripts/?.lua;' .. package.path
local launch = require('moba_role_launch')
local b = require('_bootstrap')
local process, time = b.lib('process'), b.lib('time')
local n = 0
local function check(name, fn)
  fn()
  n = n + 1
  print('PASS ' .. name)
end
local function rejects(fn)
  assert(not pcall(fn), 'expected rejection')
end
local record = process.validate_owned({
  pid = 41,
  executable = 'D:/UE5.8/Engine/Binaries/Win64/UnrealEditor.exe',
  creation_token = '133000000000000041',
})

check('default is 1800 and does not invent selection or match budgets', function()
  local options = launch.options({})
  assert(options.editor_ready_timeout_seconds == 1800)
  assert(options.finish_timeout_seconds == nil)
  assert(options.selection_timeout_seconds == nil)
end)

check('explicit boundaries and combined budgets stay independent', function()
  assert(launch.options({'--editor-ready-timeout-seconds', '1'}).editor_ready_timeout_seconds == 1)
  assert(launch.options({'--editor-ready-timeout-seconds', '1800'}).editor_ready_timeout_seconds == 1800)
  assert(launch.options({'--editor-ready-timeout-seconds', '3600'}).editor_ready_timeout_seconds == 3600)
  local combined = launch.options({
    '--interactive-selection', '--finish-timeout-seconds', '7200',
    '--selection-timeout-seconds', '7200', '--editor-ready-timeout-seconds', '90',
  })
  assert(combined.finish_timeout_seconds == 7200)
  assert(combined.selection_timeout_seconds == 7200)
  assert(combined.editor_ready_timeout_seconds == 90)
  assert(launch.options({'--finish-timeout-seconds', '1'}).finish_timeout_seconds == 1)
  assert(launch.options({'--interactive-selection', '--selection-timeout-seconds', '1'}).selection_timeout_seconds == 1)
end)

check('malformed editor startup values fail closed', function()
  local rejected = {
    '0', '00', '01', '-1', '-1800', '1.5', '1.0', '1800.0', '.5', '1.',
    '1e2', '1E3', '18e2', '1800e0', '1e309', 'inf', 'infinity', 'INF',
    '+inf', '-inf', 'Infinity', '+Infinity', 'NaN', 'nan', '+1', '+1800',
    ' 1800', '1800 ', '', '0x10', 'true', '3601',
    '9223372036854775808', '1' .. string.rep('0', 30),
  }
  for _, raw in ipairs(rejected) do
    rejects(function() launch.options({'--editor-ready-timeout-seconds', raw}) end)
  end
  rejects(function() launch.options({'--editor-ready-timeout-seconds'}) end)
end)

check('selection and match parsers keep their previous gates', function()
  for _, args in ipairs({
    {'--finish-timeout-seconds', '0'},
    {'--finish-timeout-seconds', '7201'},
    {'--finish-timeout-seconds', '1.5'},
    {'--finish-timeout-seconds', '1e2'},
    {'--prepare-only', '--finish-timeout-seconds', '60'},
    {'--selection-timeout-seconds', '60'},
    {'--interactive-selection', '--selection-timeout-seconds', '0'},
    {'--interactive-selection', '--selection-timeout-seconds', '1.5'},
    {'--interactive-selection', '--selection-timeout-seconds', '1e2'},
    {'--interactive-selection', '--selection-timeout-seconds', '7201'},
  }) do
    rejects(function() launch.options(args) end)
  end
  local prepared = launch.options({'--prepare-only', '--editor-ready-timeout-seconds', '2'})
  assert(prepared.prepare_only and prepared.editor_ready_timeout_seconds == 2)
  assert(prepared.finish_timeout_seconds == nil and prepared.selection_timeout_seconds == nil)
end)

local function with_owned_clock(alive, fn)
  local now = 0
  local mono, sleep, inspect, poll = time.monotonic_ms, time.sleep_ms, process.inspect_owned, process.poll_owned_ready
  local seen = {}
  time.monotonic_ms = function() return now end
  time.sleep_ms = function(ms)
    assert(math.type(ms) == 'integer' and ms > 0 and ms <= 50, 'unexpected readiness sleep ' .. tostring(ms))
    now = now + ms
  end
  process.inspect_owned = function(pid)
    assert(pid == record.pid, 'readiness inspected a different pid')
    return alive()
  end
  process.poll_owned_ready = function(polled, timeout, predicate, label)
    seen[#seen + 1] = {
      pid = polled.pid,
      token = polled.creation_token,
      executable = polled.executable,
      timeout = timeout,
      label = label,
    }
    return poll(polled, timeout, predicate, label)
  end
  local ok, err = xpcall(function()
    fn(seen, function() return now end, function(value) now = value end)
  end, debug.traceback)
  time.monotonic_ms, time.sleep_ms = mono, sleep
  process.inspect_owned, process.poll_owned_ready = inspect, poll
  assert(ok, err)
end

local function original_alive()
  return {
    pid = record.pid,
    executable = record.executable,
    creation_token = record.creation_token,
  }
end

check('startup poll uses the parsed budget on the original record', function()
  with_owned_clock(original_alive, function(seen, now)
    local calls = 0
    local value = launch.await_owned_editor_ready(process, record, launch.options({}), function()
      calls = calls + 1
      return calls == 3
    end)
    assert(value == true and calls == 3 and now() == 100)
    assert(#seen == 1)
    assert(seen[1].timeout == 1800000 and seen[1].pid == record.pid)
    assert(seen[1].token == record.creation_token and seen[1].executable == record.executable)
    assert(seen[1].label == 'owned PIE Editor MCP readiness')
  end)
  with_owned_clock(original_alive, function(seen, now)
    local value = launch.await_owned_editor_ready(process, record,
      launch.options({'--editor-ready-timeout-seconds', '3600'}), function() return true end)
    assert(value == true and now() == 0 and seen[1].timeout == 3600000)
    value = launch.await_owned_editor_ready(process, record,
      launch.options({'--editor-ready-timeout-seconds', '1'}), function() return true end)
    assert(value == true and seen[2].timeout == 1000)
  end)
end)

check('a startup timeout or late true is not readiness', function()
  with_owned_clock(original_alive, function(seen, now)
    local calls = 0
    local ok, err = pcall(function()
      launch.await_owned_editor_ready(process, record,
        launch.options({'--editor-ready-timeout-seconds', '1'}), function()
          calls = calls + 1
          return false
        end)
    end)
    assert(not ok and calls > 0 and now() == 1000)
    assert(tostring(err):find('owned PIE Editor MCP readiness ready timeout', 1, true))
    assert(seen[1].timeout == 1000 and seen[1].token == record.creation_token)
  end)
  with_owned_clock(original_alive, function(_, now, set_now)
    local calls = 0
    local ok, err = pcall(function()
      launch.await_owned_editor_ready(process, record,
        launch.options({'--editor-ready-timeout-seconds', '1'}), function()
          calls = calls + 1
          set_now(1001)
          return true
        end)
    end)
    assert(not ok and calls == 1 and now() == 1001)
    assert(tostring(err):find('ready timeout', 1, true))
  end)
end)

check('a reused pid is not adopted as the original editor', function()
  with_owned_clock(function()
    return {
      pid = record.pid,
      executable = record.executable,
      creation_token = '133000000000000099',
    }
  end, function(seen, now)
    local calls = 0
    local ok, err = pcall(function()
      launch.await_owned_editor_ready(process, record,
        launch.options({'--editor-ready-timeout-seconds', '1'}), function()
          calls = calls + 1
          return true
        end)
    end)
    assert(not ok and calls == 0 and now() == 0)
    assert(tostring(err):find('original lifetime exited before ready', 1, true))
    assert(seen[1].pid == record.pid and seen[1].token == record.creation_token)
    assert(seen[1].timeout == 1000)
  end)
  with_owned_clock(function() return nil end, function(_, now)
    local calls = 0
    local ok = pcall(function()
      launch.await_owned_editor_ready(process, record, launch.options({}), function()
        calls = calls + 1
        return true
      end)
    end)
    assert(not ok and calls == 0 and now() == 0)
  end)
end)

check('the consumer rejects a non-integer budget before polling', function()
  local polls = 0
  local fake = {poll_owned_ready = function() polls = polls + 1 end}
  rejects(function()
    launch.await_owned_editor_ready(fake, record, {}, function() return true end)
  end)
  for _, seconds in ipairs({0, 1.5, 1800.0, 3601, true, '1800'}) do
    rejects(function()
      launch.await_owned_editor_ready(fake, record, {editor_ready_timeout_seconds = seconds}, function()
        return true
      end)
    end)
  end
  assert(polls == 0)
end)

print('editor ready budget: ' .. n .. ' scenarios passed')
